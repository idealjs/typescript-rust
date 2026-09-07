#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn report_get_accessor_call(&mut self, callee_expr: &Arc<Node>) -> bool {
        let crate::ast::NodeData::PropertyAccessExpression(pa) = &callee_expr.data else {
            return false;
        };
        if pa.name.kind != SyntaxKind::Identifier {
            return false;
        }
        let target_type = self.get_type_of_node(&pa.expression);
        let name = pa.name.text().to_string();
        let is_getter = target_type
            .as_structured()
            .and_then(|s| s.properties.iter().find(|p| p.name == name))
            .is_some_and(|sym| sym.flags.contains(SymbolFlags::GetAccessor));
        if !is_getter {
            return false;
        }
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,

            pa.name.loc,
            crate::diagnostics::messages_generated::
                THIS_EXPRESSION_IS_NOT_CALLABLE_BECAUSE_IT_IS_A_GET_ACCESSOR_DID_YOU_MEAN_TO_USE_IT_WITHOUT,
            vec![],
        ));
        true
    }
    pub(crate) fn check_call_arity(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) -> bool {
        let arg_count = arguments.len();

        if let Some(spread_idx) = arguments
            .nodes
            .iter()
            .position(|a| matches!(a.data, crate::ast::NodeData::SpreadElement(_)))
        {
            let min_count = self.get_min_argument_count(sig);
            let max_count = self.get_parameter_count(sig);
            let has_rest = self.has_effective_rest_parameter(sig);
            let spread_ok = spread_idx >= min_count && (has_rest || spread_idx < max_count);
            if !spread_ok {
                let file = self.current_file.clone();
                let spread_node = Arc::clone(&arguments.nodes[spread_idx]);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    spread_node.loc,
                    A_SPREAD_ARGUMENT_MUST_EITHER_HAVE_A_TUPLE_TYPE_OR_BE_PASSED_TO_A_REST_PARAMETER,
                    vec![],
                ));
                return false;
            }

            return true;
        }

        let min_count = self.get_min_argument_count(sig);
        let max_count = self.get_parameter_count(sig);
        let has_rest = self.has_effective_rest_parameter(sig);

        if !has_rest && arg_count > max_count {
            let file = self.current_file.clone();
            let loc = self.extra_arguments_range(arguments, max_count);
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                loc,
                EXPECTED_0_ARGUMENTS_BUT_GOT_1,
                vec![min_count.to_string(), arg_count.to_string()],
            ));
            return false;
        }

        if arg_count < min_count {
            let file = self.current_file.clone();

            let error_loc = if is_new {
                node.loc
            } else if let crate::ast::NodeData::PropertyAccessExpression(d) = &callee_expr.data {
                d.name.loc
            } else {
                callee_expr.loc
            };
            let message = if has_rest {
                EXPECTED_AT_LEAST_0_ARGUMENTS_BUT_GOT_1
            } else {
                EXPECTED_0_ARGUMENTS_BUT_GOT_1
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                error_loc,
                message,
                vec![min_count.to_string(), arg_count.to_string()],
            ));
            return false;
        }

        true
    }

    pub(crate) fn extra_arguments_range(
        &self,
        arguments: &Arc<NodeList>,
        max_count: usize,
    ) -> TextRange {
        if max_count >= arguments.nodes.len() {
            return arguments.loc;
        }
        let start = arguments.nodes[max_count].loc.pos;
        let mut end = arguments
            .nodes
            .last()
            .map(|a| a.loc.end)
            .unwrap_or(arguments.loc.end);
        if end < start {
            end = start;
        }
        TextRange { pos: start, end }
    }

    pub(crate) fn signature_accepts_arguments(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> bool {
        if arguments.len() < sig.min_argument_count.max(0) as usize {
            return false;
        }

        let inferred_types = if sig.type_parameters.is_empty() {
            Vec::new()
        } else {
            self.infer_call_type_arguments(node, sig, &arguments.nodes)
        };

        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        for (i, arg) in arguments.iter().enumerate() {
            let param_type = if has_rest && i >= rest_index {
                match self.try_get_type_at_position(sig, i) {
                    Some(t) => t,
                    None => {
                        let rt = self.get_type_of_symbol(&sig.parameters[rest_index]);
                        match self.get_array_element_type_of(&rt) {
                            Some(e) => e,
                            None => rt,
                        }
                    }
                }
            } else if i < sig.parameters.len() {
                match self.try_get_type_at_position(sig, i) {
                    Some(t) => t,

                    None => continue,
                }
            } else {
                return false;
            };
            let param_type = if !inferred_types.is_empty() {
                self.substitute_infer_type_parameters(
                    &param_type,
                    &sig.type_parameters,
                    &inferred_types,
                )
            } else {
                param_type
            };

            if param_type.flags.contains(TypeFlags::Any) {
                continue;
            }
            let arg_type = self.get_type_of_node(arg);
            if !self.is_type_assignable_to(&arg_type, &param_type) {
                return false;
            }
        }
        true
    }

    pub(crate) fn find_matching_signature(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> usize {
        self.speculation_depth += 1;
        let result = (|| {
            for (idx, sig) in signatures.iter().enumerate() {
                if self.signature_accepts_arguments(node, sig, arguments) {
                    return idx;
                }
            }

            let arg_count = arguments.len();
            for (idx, sig) in signatures.iter().enumerate() {
                let max_params = if sig.has_rest_parameter() {
                    usize::MAX
                } else {
                    sig.parameters.len()
                };
                if arg_count <= max_params && arg_count >= sig.min_argument_count.max(0) as usize {
                    return idx;
                }
            }
            0
        })();
        self.speculation_depth -= 1;
        result
    }
    pub(crate) fn check_call_arguments(&mut self, node: &Arc<Node>, is_new: bool) {
        let (callee_expr, arguments) = match &node.data {
            crate::ast::NodeData::CallExpression(data) => {
                (&data.expression, data.arguments.clone())
            }
            crate::ast::NodeData::NewExpression(data) => {
                (&data.expression, data.arguments.clone().unwrap_or_default())
            }
            _ => return,
        };

        if !is_new && callee_expr.kind == SyntaxKind::SuperKeyword {
            let Some(base_ctor_type) = self.resolve_base_class_constructor_type() else {
                return;
            };
            self.check_call_arguments_against(node, &base_ctor_type, &arguments, callee_expr, true);
            return;
        }
        let callee_type = self.get_type_of_node(callee_expr);

        if !is_new {
            let optional_call = matches!(
                &node.data,
                crate::ast::NodeData::CallExpression(d) if d.question_dot_token.is_some()
            );
            if !optional_call {
                self.report_possibly_null_or_undefined(callee_expr, &callee_type, true);
            }
        }
        self.check_call_arguments_against(node, &callee_type, &arguments, callee_expr, is_new);
    }
}
