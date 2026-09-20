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
                .to_string()
        } else if self.callee_has_overload_arity_split(callee_type) {
            if provided != 0 && !callee_type.flags.contains(TypeFlags::Any) {
                self.check_overload_type_argument_arity(node, callee_type, provided);
            }
            return;
        } else {
            let min_count = Self::declared_min_type_argument_count(sig);
            let max_count = sig.type_parameters.len();
            if min_count < max_count {
                format!("{min_count}-{max_count}")
            } else {
                max_count.to_string()
            }
        };
        if provided != 0 && expected.parse::<usize>() != Ok(provided)
            && !(expected.contains('-')
                && (|| {
                    let (lo, hi) = expected.split_once('-')?;
                    Some(
                        provided >= lo.parse::<usize>().ok()?
                            && provided <= hi.parse::<usize>().ok()?,
                    )
                })()
                .unwrap_or(false))
            && !callee_type.flags.contains(TypeFlags::Any)
        {
            self.report_type_argument_count_mismatch(node, provided, expected);
        }
    }

    fn callee_has_overload_arity_split(&self, callee_type: &Arc<Type>) -> bool {
        let sigs = self.get_signatures_of_type(
            callee_type,
            crate::checker::types_type_id::SignatureKind::Call,
        );
        if sigs.len() <= 1 {
            return false;
        }
        let counts: Vec<(usize, usize)> = sigs
            .iter()
            .map(|s| (Self::declared_min_type_argument_count(s), s.type_parameters.len()))
            .collect();
        counts.iter().any(|c| *c != counts[0])
    }

    // Go getTypeArgumentArityError 重载分支：below/above 最近计数，
    // 两者齐备报 2743，否则按最近侧计数报 2558
    fn check_overload_type_argument_arity(
        &mut self,
        node: &Arc<Node>,
        callee_type: &Arc<Type>,
        arg_count: usize,
    ) {
        let sigs = self.get_signatures_of_type(
            callee_type,
            crate::checker::types_type_id::SignatureKind::Call,
        );
        let mut below: Option<usize> = None;
        let mut above: Option<usize> = None;
        for sig in &sigs {
            let min_count = Self::declared_min_type_argument_count(sig);
            let max_count = sig.type_parameters.len();
            if min_count > arg_count {
                above = Some(above.map_or(min_count, |a| a.min(min_count)));
            } else if max_count < arg_count {
                below = Some(below.map_or(max_count, |b| b.max(max_count)));
            }
        }
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
        match (below, above) {
            (Some(b), Some(a)) => {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    loc,
                    tsox_core::diagnostics::messages_generated::
                        NO_OVERLOAD_EXPECTS_0_TYPE_ARGUMENTS_BUT_OVERLOADS_DO_EXIST_THAT_EXPECT_EITHER_1_OR_2_TYPE_ARGUMENTS,
                    vec![arg_count.to_string(), b.to_string(), a.to_string()],
                ));
            }
            _ => {
                let expected = below.or(above).unwrap_or_default().to_string();
                self.report_type_argument_count_mismatch(node, arg_count, expected);
            }
        }
    }

    // 声明级最小实参数：default 子句前的非默认形参个数（resolved 默认惰性，
    // 语义位取声明节点）
    fn declared_min_type_argument_count(sig: &Arc<Signature>) -> usize {
        let Some(decl) = &sig.declaration else {
            return sig.type_parameters.len();
        };
        let tps = match &decl.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => &d.type_parameters,
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => &d.type_parameters,
            tsox_frontend::ast::NodeData::FunctionTypeNode(d) => &d.type_parameters,
            tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => &d.type_parameters,
            _ => return sig.type_parameters.len(),
        };
        let Some(tps) = tps else {
            return 0;
        };
        for (i, tp) in tps.iter().enumerate() {
            if let tsox_frontend::ast::NodeData::TypeParameterDeclaration(tpd) = &tp.data
                && tpd.default_type.is_some()
            {
                return i;
            }
        }
        tps.len()
    }

    fn report_type_argument_count_mismatch(&mut self, node: &Arc<Node>, provided: usize, expected: String) {
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
