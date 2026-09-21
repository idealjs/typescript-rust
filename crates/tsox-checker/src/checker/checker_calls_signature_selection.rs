#![allow(unused_imports)]

use crate::checker::checker_calls::*;

impl Checker {
    pub(crate) fn collect_callee_signatures(
        &mut self,
        callee_expr: &Arc<Node>,
        callee_type: &Arc<Type>,
        is_new: bool,
    ) -> Option<Vec<Arc<Signature>>> {
        let mut union_signatures: Vec<Arc<Signature>> = Vec::new();
        let sig_kind = if is_new {
            SignatureKind::Construct
        } else {
            SignatureKind::Call
        };
        let signatures: &[Arc<Signature>] = if callee_type.is_intersection() {
            // Go getSignaturesOfStructuredType：交集签名 = 各成分签名拼接，
            // 不可调用的成分（原始类型等）不贡献签名也不阻断
            for m in callee_type.types().into_iter().flatten() {
                union_signatures.extend(self.get_signatures_of_type(m, sig_kind));
            }
            if union_signatures.is_empty() {
                self.report_invocation_error(callee_expr, callee_type, is_new);
                return None;
            }
            &union_signatures
        } else if callee_type.as_union_or_intersection().is_some() {
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
                // Go resolveUnionTypeMembers：Function 接口成分贡献 unknown 签名
                let all_callable = !expanded_leaves.is_empty()
                    && expanded_leaves.iter().all(|m| {
                        m.as_structured()
                            .is_some_and(|s| !s.call_signatures().is_empty())
                            || self.is_global_function_type(m)
                    });
                if all_callable {
                    // Go getUnionSignatures 产出的组合签名携带 composite(原始
                    // 签名表),推断与上下文定型会分布回原始签名(上下文敏感
                    // 实参参数取并集)。本地组合无 composite 分布,真交集参数
                    // 会误伤混合类型参数等场景,故调用路径维持逐成员拼接
                    for m in &expanded_leaves {
                        if self.is_global_function_type(m) {
                            union_signatures.push(self.untyped_call_signature());
                            continue;
                        }
                        if let Some(s) = m.as_structured() {
                            union_signatures.extend(s.call_signatures().iter().cloned());
                        }
                    }
                    if let Some(combined) = self.try_combine_union_call_signatures(&union_signatures)
                    {
                        union_signatures = vec![combined];
                    }
                    if union_signatures.is_empty() {
                        self.report_invocation_error(callee_expr, callee_type, is_new);
                        return None;
                    }
                    &union_signatures
                } else {
                    self.report_invocation_error(callee_expr, callee_type, is_new);
                    return None;
                }
            }        } else {
            // Go isUntypedFunctionCall：无任何签名且为全局 Function 型时按
            // untyped 调用，不报不可调用
            if !is_new
                && callee_type.as_structured().is_some_and(|s| {
                    s.call_signatures().is_empty() && s.construct_signatures().is_empty()
                })
                && self.is_global_function_type(callee_type)
            {
                union_signatures.push(self.untyped_call_signature());
                return Some(union_signatures);
            }
            let resolved = self.get_signatures_of_type(callee_type, sig_kind);
            if resolved.is_empty() {
                let other_kind = if is_new {
                    SignatureKind::Call
                } else {
                    SignatureKind::Construct
                };
                let other = self.get_signatures_of_type(callee_type, other_kind);
                if !is_new && !other.is_empty() {
                    let type_str = self.type_to_string(callee_type);
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        callee_expr.loc,
                        tsox_core::diagnostics::messages_generated::
                            VALUE_OF_TYPE_0_IS_NOT_CALLABLE_DID_YOU_MEAN_TO_INCLUDE_NEW,
                        vec![type_str],
                    ));
                    return None;
                }
                if is_new && !other.is_empty() {
                    self.new_call_fallback_signature = true;
                    return Some(other);
                }
                if !is_new && self.report_get_accessor_call(callee_expr) {
                    return None;
                }
                self.report_invocation_error(callee_expr, callee_type, is_new);
                return None;
            }
            union_signatures = resolved;
            &union_signatures
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
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    callee_expr.loc,
                    tsox_core::diagnostics::messages_generated::
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
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                node.loc,
                                tsox_core::diagnostics::messages_generated::
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

impl Checker {
    // Go unknownSignature：无参任意返回的合成调用签名
    fn untyped_call_signature(&mut self) -> Arc<Signature> {
        self.build_signature_from_function_like_type_node(
            &Arc::new(NodeList::default()),
            self.get_any_type(),
            false,
            None,
            None,
        )
    }

    // Go t == globalFunctionType 判定
    fn is_global_function_type(&self, t: &Arc<Type>) -> bool {
        t.flags.contains(TypeFlags::Object)
            && self
                .globals
                .get("Function")
                .zip(t.symbol.as_ref())
                .is_some_and(|(function_sym, sym)| Arc::ptr_eq(function_sym, sym))
    }
}
