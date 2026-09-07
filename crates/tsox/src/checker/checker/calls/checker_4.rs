#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn report_no_overload_matches(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> bool {
        let saved = self.diagnostics.take_inner();
        let mut entries: Vec<crate::ast::Diagnostic> = Vec::new();
        let mut all_failed = true;
        for sig in signatures.iter() {
            match self.probe_first_argument_error(node, sig, arguments) {
                Some(d) => entries.push(d),
                None => {
                    all_failed = false;
                    break;
                }
            }
        }
        let _probe_only = self.diagnostics.take_inner();
        self.diagnostics.set_inner(saved);
        if !all_failed {
            return false;
        }
        let file = self.current_file.clone();
        let anchor = entries.first().map(|d| d.loc).unwrap_or(node.loc);
        let mut chain: Vec<crate::ast::Diagnostic> = Vec::new();
        for (i, (entry, sig)) in entries.into_iter().zip(signatures.iter()).enumerate() {
            let sig_str = self.signature_display_colon(sig, "");
            let mut d = crate::ast::Diagnostic::new(
                file.clone(),
                anchor,
                crate::diagnostics::messages_generated::OVERLOAD_0_OF_1_2_GAVE_THE_FOLLOWING_ERROR,
                vec![(i + 1).to_string(), signatures.len().to_string(), sig_str],
            );
            d.message_chain = vec![entry];
            chain.push(d);
        }
        let mut head = crate::ast::Diagnostic::new(
            file,
            anchor,
            crate::diagnostics::messages_generated::NO_OVERLOAD_MATCHES_THIS_CALL,
            Vec::new(),
        );
        head.message_chain = chain;
        self.diagnostics.add(head);
        true
    }

    pub(crate) fn probe_first_argument_error(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> Option<crate::ast::Diagnostic> {
        let arg_count = arguments.len();
        let max_params = if sig.has_rest_parameter() {
            usize::MAX
        } else {
            sig.parameters.len()
        };
        if arg_count > max_params || arg_count < sig.min_argument_count.max(0) as usize {
            return None;
        }
        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        let rest_element_type = if has_rest {
            match self.signature_instantiated_param_type(sig, rest_index) {
                Some(arr) => Some(self.get_array_element_type(&arr)),
                None => match self.try_get_type_at_position(sig, rest_index) {
                    Some(t) => Some(t),
                    None => {
                        let rest_param_type = self.get_type_of_symbol(&sig.parameters[rest_index]);
                        Some(self.get_array_element_type(&rest_param_type))
                    }
                },
            }
        } else {
            None
        };
        let inferred_types = self.infer_call_type_arguments(node, sig, &arguments.nodes);
        for (i, arg) in arguments.iter().enumerate() {
            let base_param_type = if has_rest && i >= rest_index {
                Arc::clone(rest_element_type.as_ref().unwrap())
            } else if i < sig.parameters.len() {
                self.signature_instantiated_param_type(sig, i)
                    .or_else(|| self.try_get_type_at_position(sig, i))
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
            } else {
                base_param_type
            };
            let inference_empty = !sig.type_parameters.is_empty() && inferred_types.is_empty();
            if param_type.flags.contains(TypeFlags::Any)
                || (inference_empty && param_type.is_type_parameter())
            {
                continue;
            }
            let arg_type = self.get_type_of_node(arg);
            if self.is_type_related_to(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
            ) {
                continue;
            }
            let param_optional = i < sig.parameters.len()
                && (sig.parameters[i]
                    .flags
                    .contains(crate::ast::SymbolFlags::Optional)
                    || sig.parameters[i].declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            crate::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some() || pd.initializer.is_some()
                        )
                    }));
            let display_param = if param_optional {
                Some(self.strip_optional_undefined(&param_type))
            } else {
                None
            };
            let mut out: Vec<crate::ast::Diagnostic> = Vec::new();
            self.check_type_related_to_and_elaborate_display(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
                Some(arg),
                Some(arg),
                Some(&ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1),
                Some(&mut out),
                display_param.as_ref(),
            );
            return out.into_iter().next();
        }
        None
    }
    pub(crate) fn get_return_type_of_call_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let callee = match &node.data {
            crate::ast::NodeData::CallExpression(data) => {
                (&data.expression, data.arguments.clone())
            }
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(&callee.0);
        if let Some(structured) = callee_type.as_structured() {
            let signatures = structured.call_signatures();
            if signatures.is_empty() {
                return self.get_any_type();
            }

            let matching_idx = if signatures.len() == 1 {
                0
            } else {
                self.find_matching_signature(node, signatures, &callee.1)
            };
            let sig = &signatures[matching_idx];
            if let Some(rt) = self.get_return_type_of_signature(sig) {
                if !sig.type_parameters.is_empty() {
                    let args: Vec<Arc<Node>> = callee.1.iter().cloned().collect();
                    let inferred = self.infer_call_type_arguments(node, sig, &args);
                    self.in_return_substitution = true;
                    let r =
                        self.substitute_infer_type_parameters(&rt, &sig.type_parameters, &inferred);
                    self.in_return_substitution = false;
                    return r;
                }
                return rt;
            }

            return self.get_any_type();
        }
        self.get_any_type()
    }

    pub(crate) fn get_return_type_of_new_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (callee, args) = match &node.data {
            crate::ast::NodeData::NewExpression(data) => {
                (&data.expression, data.arguments.clone().unwrap_or_default())
            }
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(callee);
        if let Some(structured) = callee_type.as_structured() {
            for sig in structured.construct_signatures() {
                if let Some(rt) = self.get_return_type_of_signature(sig) {
                    let rt = if !sig.type_parameters.is_empty() {
                        let arg_vec: Vec<Arc<Node>> = args.iter().cloned().collect();
                        let inferred = self.infer_call_type_arguments(node, sig, &arg_vec);
                        self.substitute_infer_type_parameters(&rt, &sig.type_parameters, &inferred)
                    } else {
                        rt
                    };

                    if let crate::ast::NodeData::NewExpression(d) = &node.data
                        && let Some(type_args) = &d.type_arguments
                        && let Some(class_sym) = rt.symbol.clone()
                    {
                        let tps = self.declared_type_parameter_types(&class_sym);
                        let arg_types: Vec<Arc<Type>> = type_args
                            .iter()
                            .map(|t| self.get_type_from_type_node(t))
                            .collect();
                        if !tps.is_empty() && tps.len() == arg_types.len() {
                            return self.attach_explicit_type_arguments_cached(&rt, arg_types);
                        }
                    }
                    return rt;
                }
                return self.get_any_type();
            }
        }
        self.get_any_type()
    }
}
