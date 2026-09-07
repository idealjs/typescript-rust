#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_call_arguments_against(
        &mut self,
        node: &Arc<Node>,
        callee_type: &Arc<Type>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) {
        if callee_type.flags.contains(TypeFlags::Any) {
            return;
        }

        let cond_constraint;
        let callee_type: &Arc<Type> = if callee_type.flags.contains(TypeFlags::Conditional) {
            match self.deferred_default_constraint_of_conditional(callee_type) {
                Some(constraint) => {
                    cond_constraint = constraint;
                    &cond_constraint
                }
                None => callee_type,
            }
        } else {
            callee_type
        };

        let Some(signatures) = self.collect_callee_signatures(callee_expr, callee_type, is_new)
        else {
            return;
        };

        let type_arg_filtered: Vec<Arc<Signature>>;
        let signatures: &[Arc<Signature>] = {
            let provided = Self::explicit_type_argument_count(node);
            if provided != 0 && signatures.len() > 1 {
                type_arg_filtered = signatures
                    .iter()
                    .filter(|s| s.type_parameters.len() == provided)
                    .cloned()
                    .collect();
                if !type_arg_filtered.is_empty() {
                    &type_arg_filtered
                } else {
                    &signatures
                }
            } else {
                &signatures
            }
        };
        if signatures.is_empty() {
            self.check_uncallable_callee(node, callee_type, arguments, callee_expr, is_new);
            return;
        }

        let matching_idx = if signatures.len() == 1 {
            0
        } else {
            let no_match = {
                self.speculation_depth += 1;
                let r = !signatures
                    .iter()
                    .any(|s| self.signature_accepts_arguments(node, s, &arguments));
                self.speculation_depth -= 1;
                r
            };
            if no_match && self.report_no_overload_matches(node, signatures, &arguments) {
                return;
            }
            self.find_matching_signature(node, signatures, &arguments)
        };
        let sig = Arc::clone(&signatures[matching_idx]);

        if !self.check_call_arity(node, &sig, &arguments, callee_expr, is_new) {
            return;
        }
        let _file = self.current_file.clone();

        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        let rest_element_type = if has_rest {
            let ret = match self.try_get_type_at_position(&sig, rest_index) {
                Some(t) => Some(t),
                None => {
                    let rest_param_type = self.get_type_of_symbol(&sig.parameters[rest_index]);
                    Some(self.get_array_element_type(&rest_param_type))
                }
            };
            ret
        } else {
            None
        };

        if !sig.type_parameters.is_empty() || Self::has_explicit_type_arguments(node) {
            self.check_explicit_type_argument_count(node, &sig, is_new, callee_type);
        }

        let inferred_types = self.infer_call_type_arguments(node, &sig, &arguments.nodes);

        let new_explicit_subst: Option<(Vec<Arc<Type>>, Vec<Arc<Type>>)> = if is_new {
            self.get_return_type_of_signature(&sig)
                .and_then(|rt| rt.symbol.clone())
                .and_then(|class_sym| {
                    let tps = self.declared_type_parameter_types(&class_sym);
                    if tps.is_empty() {
                        return None;
                    }

                    let args: Option<Vec<Arc<Type>>> = match &node.data {
                        crate::ast::NodeData::NewExpression(d) => d
                            .type_arguments
                            .as_ref()
                            .map(|ta| ta.iter().map(|t| self.get_type_from_type_node(t)).collect()),
                        _ => None,
                    };
                    let args = match args {
                        Some(a) if a.len() == tps.len() => Some(a),

                        _ if callee_expr.kind == SyntaxKind::SuperKeyword => self
                            .heritage_type_arguments_for_base(&class_sym)
                            .filter(|a| a.len() == tps.len()),
                        _ => None,
                    };
                    args.map(|args| (tps, args))
                })
        } else {
            None
        };
        if std::env::var_os("TSOX_DEBUG_INFER").is_some() {
            eprintln!(
                "[infer] sig params={} tp={}",
                sig.parameters.len(),
                sig.type_parameters.len()
            );
            for (i, t) in inferred_types.iter().enumerate() {
                eprintln!("[infer]   {} -> {}", i, self.type_to_string(t));
            }
        }
        self.check_call_arguments_loop(
            node,
            &sig,
            arguments,
            has_rest,
            rest_index,
            &rest_element_type,
            &inferred_types,
            &new_explicit_subst,
        );
    }
}
