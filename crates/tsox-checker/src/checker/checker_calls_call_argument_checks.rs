#![allow(unused_imports)]

use crate::checker::checker_calls::*;

impl Checker {
    /// 显式类型实参的期望数量：按首个声明的类型参数表。增强声明的同名 T
    /// 是独立符号，按并集计数会让 Set<T>（4 处增强）虚报 4 个
    pub(crate) fn first_declared_type_parameter_count(
        &self,
        class_sym: &Arc<tsox_frontend::ast::Symbol>,
    ) -> usize {
        class_sym
            .declarations
            .iter()
            .find_map(|d| match &d.data {
                tsox_frontend::ast::NodeData::InterfaceDeclaration(i) => {
                    i.type_parameters.as_ref().map(|t| t.len())
                }
                tsox_frontend::ast::NodeData::ClassDeclaration(c) => {
                    c.type_parameters.as_ref().map(|t| t.len())
                }
                _ => None,
            })
            .unwrap_or(0)
    }

    pub(crate) fn check_explicit_type_argument_count(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        is_new: bool,
        callee_type: &Arc<Type>,
    ) {
        let provided = Self::explicit_type_argument_count(node);

        let expected = if is_new {
            self.get_return_type_of_signature(&sig)
                .and_then(|rt| rt.symbol.clone())
                .map(|class_sym| {
                    let first_decl_count =
                        self.first_declared_type_parameter_count(&class_sym);
                    if first_decl_count == 0 {
                        sig.type_parameters.len()
                    } else {
                        first_decl_count
                    }
                })
                .unwrap_or_else(|| sig.type_parameters.len())
        } else {
            sig.type_parameters.len()
        };
        if provided != 0 && provided != expected && !callee_type.flags.contains(TypeFlags::Any) {
            let loc = match &node.data {
                tsox_frontend::ast::NodeData::CallExpression(d) => d
                    .type_arguments
                    .as_ref()
                    .and_then(|t| t.iter().next())
                    .map(|t| t.loc)
                    .unwrap_or(node.loc),
                tsox_frontend::ast::NodeData::NewExpression(d) => d
                    .type_arguments
                    .as_ref()
                    .and_then(|t| t.iter().next())
                    .map(|t| t.loc)
                    .unwrap_or(node.loc),
                _ => node.loc,
            };
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                loc,
                tsox_core::diagnostics::messages_generated::EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
                vec![expected.to_string(), provided.to_string()],
            ));
        }
    }

    pub(crate) fn check_call_arguments_loop(
        &mut self,
        _node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
        has_rest: bool,
        rest_index: usize,
        rest_element_type: &Option<Arc<Type>>,
        inferred_types: &[Arc<Type>],
        new_explicit_subst: &Option<(Vec<Arc<Type>>, Vec<Arc<Type>>)>,
    ) {
        for (i, arg) in arguments.iter().enumerate() {
            let base_param_type = if has_rest && i >= rest_index {
                Arc::clone(rest_element_type.as_ref().unwrap())
            } else if i < sig.parameters.len() {
                self.try_get_type_at_position(&sig, i)
                    .unwrap_or_else(|| self.get_type_of_symbol(&sig.parameters[i]))
            } else {
                continue;
            };

            let param_type = if !inferred_types.is_empty() {
                self.substitute_infer_type_parameters(
                    &base_param_type,
                    &sig.type_parameters,
                    &inferred_types,
                )
            } else if let Some((tps, args)) = new_explicit_subst.as_ref() {
                self.substitute_infer_type_parameters(&base_param_type, tps, args)
            } else {
                Arc::clone(&base_param_type)
            };

            let inference_empty = !sig.type_parameters.is_empty() && inferred_types.is_empty();
            if param_type.flags.contains(TypeFlags::Any)
                || (inference_empty && param_type.is_type_parameter())
            {
                continue;
            }

            if matches!(
                arg.kind,
                SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
            ) {
                let pt = Arc::clone(&param_type);
                self.check_contextual_elements(arg, &pt, arg.loc);
            }
            // 上下文敏感实参（箭头/函数表达式含无注解参数）：用固定后的参数类型重定型，
            // 与 infer_type_arguments 两阶段一致（节点缓存的类型是未固定形态）；
            // 泛型 callee 的实参在 walk 期被 check_call_arg_with_context 跳过
            //（防未固定 T 污染），此处定型完成后补跑表达式检查（体内语句诊断）
            let arg_type = if self.is_context_sensitive(arg) {
                let t = self.type_of_context_sensitive_arg(arg, &param_type);
                if !sig.type_parameters.is_empty() {
                    self.check_expression(arg);
                }
                t
            } else {
                self.get_type_of_node(arg)
            };

            let display_param = if i < sig.parameters.len() {
                let param_optional = sig.parameters[i]
                    .flags
                    .contains(tsox_frontend::ast::SymbolFlags::Optional)
                    || sig.parameters[i].declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            tsox_frontend::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some() || pd.initializer.is_some()
                        )
                    });
                if param_optional {
                    Some(self.strip_optional_undefined(&param_type))
                } else {
                    None
                }
            } else {
                None
            };

            let elements_reported = matches!(
                arg.kind,
                SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
            ) && self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.loc.pos() >= arg.loc.pos() && d.loc.end() <= arg.loc.end());
            if elements_reported {
                continue;
            }
            let ok = self.check_type_related_to_and_elaborate_display(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
                Some(arg),
                Some(arg),
                Some(&ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1),
                None,
                display_param.as_ref(),
            );

            if !ok {
                break;
            }
        }
    }
}
