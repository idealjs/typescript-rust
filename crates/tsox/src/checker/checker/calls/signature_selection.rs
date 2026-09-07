#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn collect_callee_signatures(
        &mut self,
        callee_expr: &Arc<Node>,
        callee_type: &Arc<Type>,
        is_new: bool,
    ) -> Option<Vec<Arc<Signature>>> {
        let mut union_signatures: Vec<Arc<Signature>> = Vec::new();
        let signatures: &[Arc<Signature>] = if callee_type.as_union_or_intersection().is_some() {
            let mut leaves: Vec<&Arc<Type>> = Vec::new();
            flatten_union_leaves(callee_type, &mut leaves);
            if is_new {
                let all_constructable = !leaves.is_empty()
                    && leaves.iter().all(|m| {
                        m.as_structured()
                            .is_some_and(|s| !s.construct_signatures().is_empty())
                    });
                if all_constructable {
                    for m in &leaves {
                        if let Some(s) = m.as_structured() {
                            union_signatures.extend(s.construct_signatures().iter().cloned());
                        }
                    }
                    &union_signatures
                } else {
                    self.report_invocation_error(callee_expr, callee_type, is_new);
                    return None;
                }
            } else {
                let mut expanded_leaves: Vec<Arc<Type>> = Vec::new();
                for m in leaves.iter().copied() {
                    if m.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
                        continue;
                    }
                    if m.flags.contains(TypeFlags::Conditional) {
                        if let Some(constraint) = self.deferred_default_constraint_of_conditional(m)
                        {
                            if let Some(u) = constraint.as_union_or_intersection() {
                                for c in u.types.iter() {
                                    if !c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null)
                                        && !c.flags.contains(TypeFlags::Never)
                                    {
                                        expanded_leaves.push(Arc::clone(c));
                                    }
                                }
                            } else if !constraint
                                .flags
                                .intersects(TypeFlags::Undefined | TypeFlags::Null)
                                && !constraint.flags.contains(TypeFlags::Never)
                            {
                                expanded_leaves.push(constraint);
                            }
                            continue;
                        }
                    }
                    expanded_leaves.push(Arc::clone(m));
                }
                let all_callable = !expanded_leaves.is_empty()
                    && expanded_leaves.iter().all(|m| {
                        m.as_structured()
                            .is_some_and(|s| !s.call_signatures().is_empty())
                    });
                if all_callable {
                    for m in &expanded_leaves {
                        if let Some(s) = m.as_structured() {
                            union_signatures.extend(s.call_signatures().iter().cloned());
                        }
                    }
                    &union_signatures
                } else {
                    self.report_invocation_error(callee_expr, callee_type, is_new);
                    return None;
                }
            }
        } else if let Some(structured) = callee_type.as_structured() {
            if is_new {
                structured.construct_signatures()
            } else {
                structured.call_signatures()
            }
        } else {
            if !is_new && self.report_get_accessor_call(callee_expr) {
                return None;
            }
            self.report_invocation_error(callee_expr, callee_type, is_new);
            return None;
        };
        Some(signatures.to_vec())
    }

    pub(crate) fn check_uncallable_callee(
        &mut self,
        node: &Arc<Node>,
        callee_type: &Arc<Type>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) {
        if !is_new {
            if callee_expr.kind == SyntaxKind::Identifier
                && let Some(structured) = callee_type.as_structured()
                && !structured.construct_signatures().is_empty()
            {
                let type_str = self.type_to_string(callee_type);
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    callee_expr.loc,
                    crate::diagnostics::messages_generated::
                        VALUE_OF_TYPE_0_IS_NOT_CALLABLE_DID_YOU_MEAN_TO_INCLUDE_NEW,
                    vec![type_str],
                ));
                return;
            }
        }
        if is_new {
            if let Some(structured) = callee_type.as_structured() {
                let call_sigs: &[Arc<Signature>] = structured.call_signatures();
                if !call_sigs.is_empty() {
                    if !self.no_implicit_any {
                        let matching = self.find_matching_signature(node, call_sigs, &arguments);
                        let ret_is_void = self
                            .get_return_type_of_signature(&call_sigs[matching])
                            .is_some_and(|t| t.flags.contains(TypeFlags::Void));
                        if !ret_is_void {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
                                    ONLY_A_VOID_FUNCTION_CAN_BE_CALLED_WITH_THE_NEW_KEYWORD,
                                Vec::new(),
                            ));
                        }
                    }
                    self.check_call_arguments_against(
                        node,
                        callee_type,
                        &arguments,
                        callee_expr,
                        false,
                    );
                    return;
                }
            }
        }

        if !is_new && self.report_get_accessor_call(callee_expr) {
            return;
        }
        self.report_invocation_error(callee_expr, callee_type, is_new);
        return;
    }
}
