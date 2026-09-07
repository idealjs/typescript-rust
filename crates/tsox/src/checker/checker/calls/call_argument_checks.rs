#![allow(unused_imports)]

use super::*;

impl Checker {
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
                    let tps = self.declared_type_parameter_types(&class_sym);
                    if tps.is_empty() {
                        sig.type_parameters.len()
                    } else {
                        tps.len()
                    }
                })
                .unwrap_or_else(|| sig.type_parameters.len())
        } else {
            sig.type_parameters.len()
        };
        if provided != 0 && provided != expected && !callee_type.flags.contains(TypeFlags::Any) {
            let loc = match &node.data {
                crate::ast::NodeData::CallExpression(d) => d
                    .type_arguments
                    .as_ref()
                    .and_then(|t| t.iter().next())
                    .map(|t| t.loc)
                    .unwrap_or(node.loc),
                crate::ast::NodeData::NewExpression(d) => d
                    .type_arguments
                    .as_ref()
                    .and_then(|t| t.iter().next())
                    .map(|t| t.loc)
                    .unwrap_or(node.loc),
                _ => node.loc,
            };
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                loc,
                crate::diagnostics::messages_generated::EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
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
            let arg_type = self.get_type_of_node(arg);

            let display_param = if i < sig.parameters.len() {
                let param_optional = sig.parameters[i]
                    .flags
                    .contains(crate::ast::SymbolFlags::Optional)
                    || sig.parameters[i].declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            crate::ast::NodeData::ParameterDeclaration(pd)
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
                None,
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
