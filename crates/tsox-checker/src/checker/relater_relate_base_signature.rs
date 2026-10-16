#![allow(unused_imports)]

use crate::checker::mig::m2e_2::{has_inference_candidates, has_overlapping_inferences};
use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub fn get_base_signature(&mut self, sig: &Arc<Signature>) -> Arc<Signature> { ::tsox_core::fntrace::enter("get_base_signature"); 
        if sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        let saved_strict = self.erase_signature_strict;
        self.erase_signature_strict = true;
        let result = self.get_base_signature_inner(sig);
        self.erase_signature_strict = saved_strict;
        result
    }

    fn get_base_signature_inner(&mut self, sig: &Arc<Signature>) -> Arc<Signature> { ::tsox_core::fntrace::enter("get_base_signature_inner"); 
        if sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        let tps = sig.type_parameters.clone();
        let unknown = self.unknown_type();
        let mut base_constraints: Vec<Arc<Type>> = tps
            .iter()
            .map(|tp| {
                self.get_constraint_of_type_parameter(tp)
                    .unwrap_or_else(|| Arc::clone(&unknown))
            })
            .collect();
        for _ in 0..tps.len().saturating_sub(1) {
            let next: Vec<Arc<Type>> = base_constraints
                .iter()
                .map(|t| self.substitute_infer_type_parameters(t, &tps, &base_constraints))
                .collect();
            base_constraints = next;
        }
        let any = self.any_type();
        let any_args: Vec<Arc<Type>> = tps.iter().map(|_| Arc::clone(&any)).collect();
        base_constraints = base_constraints
            .iter()
            .map(|t| self.substitute_infer_type_parameters(t, &tps, &any_args))
            .collect();
        self.get_signature_instantiation(sig, &base_constraints, &[])
    }

    pub(crate) fn single_signature_of_kind(
        &mut self,
        t: &Arc<Type>,
        construct: bool,
    ) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("single_signature_of_kind");
        let kind = if construct {
            SignatureKind::Construct
        } else {
            SignatureKind::Call
        };
        let sigs = self.get_signatures_of_type(t, kind);
        if sigs.len() == 1 {
            sigs.into_iter().next()
        } else {
            None
        }
    }

    /// Go instantiateTypeWithSingleGenericCallSignature（推断实参位）：
    /// 单泛型调用/构造签名的实参表达式按上下文签名（外层已固定类型参数
    /// 代入）实例化，返回实例化后的函数型；不适用返回 None
    pub(crate) fn instantiate_generic_call_arg_type(
        &mut self,
        arg_type: &Arc<Type>,
        contextual_type: &Arc<Type>,
        outer: &mut crate::checker::inference::InferenceContext,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("instantiate_generic_call_arg_type"); 
        let construct;
        let sig = match self.single_signature_of_kind(arg_type, false) {
            Some(s) => {
                construct = false;
                s
            }
            None => match self.single_signature_of_kind(arg_type, true) {
                Some(s) => {
                    construct = true;
                    s
                }
                None => return None,
            },
        };
        if sig.type_parameters.is_empty() {
            return None;
        }
        let ctx_type = self.get_non_nullable_type_of(contextual_type);
        let ctx_sig = self.single_signature_of_kind(&ctx_type, construct)?;
        if !ctx_sig.type_parameters.is_empty() {
            return None;
        }

        if !outer.inferences.iter().all(|info| has_inference_candidates(info)) {
            let outer_return_signature = outer
                .signature
                .as_ref()
                .and_then(|s| self.get_return_type_of_signature(s))
                .and_then(|rt| self.get_single_call_or_construct_signature(&rt));
            if let Some(outer_return_signature) = outer_return_signature {
                if outer_return_signature.type_parameters.is_empty() {
                    let unique_type_parameters =
                        self.get_unique_type_parameters(outer, &sig.type_parameters);
                    let instantiated_signature = self.get_signature_instantiation(
                        &sig,
                        &unique_type_parameters,
                        &[],
                    );
                    let inferences: Vec<crate::checker::inference::InferenceInfo> = outer
                        .inferences
                        .iter()
                        .map(|info| {
                            crate::checker::inference::InferenceInfo::new(Arc::clone(
                                &info.type_parameter,
                            ))
                        })
                        .collect();
                    let mut inferences = inferences;
                    let mut param_pairs: Vec<(Arc<Type>, Arc<Type>)> = Vec::new();
                    self.apply_to_parameter_types(
                        &instantiated_signature,
                        &ctx_sig,
                        &mut |source: &Arc<Type>, target: &Arc<Type>| {
                            param_pairs.push((Arc::clone(source), Arc::clone(target)));
                        },
                    );
                    for (source, target) in &param_pairs {
                        self.infer_types(
                            &mut inferences,
                            Some(Arc::clone(source)),
                            Some(Arc::clone(target)),
                            crate::checker::inference::InferencePriority::None,
                            true,
                        );
                    }
                    if inferences.iter().any(|info| has_inference_candidates(info)) {
                        let mut return_pairs: Vec<(Arc<Type>, Arc<Type>)> = Vec::new();
                        self.apply_to_return_types(
                            &instantiated_signature,
                            &ctx_sig,
                            &mut |source: &Arc<Type>, target: &Arc<Type>| {
                                return_pairs.push((Arc::clone(source), Arc::clone(target)));
                            },
                        );
                        for (source, target) in &return_pairs {
                            self.infer_types(
                                &mut inferences,
                                Some(Arc::clone(source)),
                                Some(Arc::clone(target)),
                                crate::checker::inference::InferencePriority::None,
                                false,
                            );
                        }
                        if !has_overlapping_inferences(&outer.inferences, &inferences) {
                            self.merge_inferences(&mut outer.inferences, &inferences);
                            outer.inferred_type_parameters.extend(
                                unique_type_parameters.iter().map(|tp| Some(Arc::clone(tp))),
                            );
                            return Some(
                                self.get_or_create_type_from_signature(&instantiated_signature),
                            );
                        }
                    }
                }
            }
        }

        let n = outer.inferences.len();
        let outer_tps: Vec<Arc<Type>> = outer
            .inferences
            .iter()
            .map(|i| Arc::clone(&i.type_parameter))
            .collect();
        // Go instantiateSignatureInContextOf（checker.go:20632-20645）经外层推断
        // 语境的 fixing mapper 实例化上下文签名（mapper.go:342-357）：被上下文
        // 签名参数位引用的外层类型参数按推断值固定，无候选回退 unknown；未被
        // 参数位引用者不经 mapper、不锁定，后续实参候选仍可进入（inference.go:188
        // isFixed 门只挡已固定者）。上下文签名带类型参数 rest 时走
        // nonFixingMapper（checker.go:20631-20635）：同值代入但不置 isFixed。
        // 返回位不参与：Go instantiateSignatureEx（checker.go:21850-21852，
        // GH#17600）对返回位/谓词留惰性代入，且外层语境非空时不从返回位
        // 推断（checker.go:20647-20651），返回位引用的外层类型参留给后续实参推断
        let mut ctx_types: Vec<Arc<Type>> = Vec::new();
        for i in 0..self.get_parameter_count(&ctx_sig) {
            ctx_types.push(self.get_type_at_position(&ctx_sig, i));
        }
        let has_tp_rest = self
            .get_effective_rest_type(&ctx_sig)
            .is_some_and(|r| r.flags.contains(TypeFlags::TypeParameter));
        let unknown = self.unknown_type();
        let mut effective: Vec<Arc<Type>> = Vec::with_capacity(n);
        let mut any_referenced = false;
        for i in 0..n {
            let referenced = ctx_types.iter().any(|t| {
                let probe = self.substitute_infer_type_parameters(
                    t,
                    &[Arc::clone(&outer_tps[i])],
                    &[Arc::clone(&unknown)],
                );
                !Arc::ptr_eq(&probe, t)
            });
            if referenced {
                let t = outer.inferences[i]
                    .inferred_type
                    .clone()
                    .unwrap_or_else(|| self.get_inferred_type(outer, i));
                let info = &mut outer.inferences[i];
                if !has_tp_rest {
                    info.is_fixed = true;
                }
                info.inferred_type = Some(Arc::clone(&t));
                any_referenced = true;
                effective.push(t);
            } else {
                effective.push(Arc::clone(&outer_tps[i]));
            }
        }
        let ctx_sig_inst = if any_referenced {
            self.substitute_signature_outer(&ctx_sig, &outer_tps, &effective)
        } else {
            ctx_sig
        };

        let inner_tps = sig.type_parameters.clone();
        let inner_inferences: Vec<crate::checker::inference::InferenceInfo> = inner_tps
            .iter()
            .map(|p| crate::checker::inference::InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut inner = crate::checker::inference::InferenceContext::new(inner_inferences);
        inner.signature = Some(Arc::clone(&sig));

        let source_count = self.get_parameter_count(&ctx_sig_inst);
        let target_count = self.get_parameter_count(&sig);
        let count = source_count.min(target_count);
        for i in 0..count {
            let s = self.get_type_at_position(&ctx_sig_inst, i);
            let t = self.get_type_at_position(&sig, i);
            self.infer_types(
                &mut inner.inferences,
                Some(s),
                Some(t),
                crate::checker::inference::InferencePriority::None,
                false,
            );
        }
        let inferred = self.get_inferred_types(&mut inner);
        let inst = self.get_signature_instantiation(&sig, &inferred, &[]);
        let inst = self.fixup_instantiated_predicate(&sig, &inst, &inner_tps, &inferred);
        Some(self.create_function_or_constructor_type(vec![inst], construct))
    }

    fn substitute_signature_outer(
        &mut self,
        sig: &Arc<Signature>,
        outer_tps: &[Arc<Type>],
        effective: &[Arc<Type>],
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("substitute_signature_outer");
        // Go instantiateSignatureEx（checker.go:21850-21858）：参数位急切代入，
        // 返回位与谓词留空惰性（GH#17600：急切代入会经 fixing mapper 使推断
        // 提前固定）；本路径不消费该签名的返回位
        let count = self.get_parameter_count(sig);
        let mut inst = Signature::new();
        inst.flags = sig.flags;
        inst.min_argument_count = sig.min_argument_count;
        inst.resolved_min_argument_count = sig.resolved_min_argument_count;
        inst.declaration = sig.declaration.clone();
        inst.target = None;
        inst.parameters = sig.parameters.clone();
        inst.this_parameter = sig.this_parameter.clone();
        inst.type_parameters = Vec::new();
        let mut param_types = Vec::with_capacity(count);
        for i in 0..count {
            let t = self.get_type_at_position(sig, i);
            param_types.push(self.substitute_infer_type_parameters(&t, outer_tps, effective));
        }
        inst.instantiated_parameter_types = Some(param_types);
        Arc::new(inst)
    }

    fn fixup_instantiated_predicate(
        &mut self,
        original: &Arc<Signature>,
        inst: &Arc<Signature>,
        tps: &[Arc<Type>],
        args: &[Arc<Type>],
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("fixup_instantiated_predicate"); 
        let Some(pred) = self.compute_type_predicate_of_signature(original) else {
            return Arc::clone(inst);
        };
        let Some(t) = pred.t.as_ref() else {
            return Arc::clone(inst);
        };
        let sub = self.substitute_infer_type_parameters(t, tps, args);
        if Arc::ptr_eq(&sub, t) {
            return Arc::clone(inst);
        }
        let mut new_pred = pred;
        new_pred.t = Some(sub);
        let mut rebuilt = Signature::new();
        rebuilt.flags = inst.flags;
        rebuilt.min_argument_count = inst.min_argument_count;
        rebuilt.resolved_min_argument_count = inst.resolved_min_argument_count;
        rebuilt.declaration = inst.declaration.clone();
        rebuilt.target = inst.target.clone();
        rebuilt.parameters = inst.parameters.clone();
        rebuilt.this_parameter = inst.this_parameter.clone();
        rebuilt.type_parameters = inst.type_parameters.clone();
        rebuilt.instantiated_parameter_types = inst.instantiated_parameter_types.clone();
        if let Some(rt) = self.get_return_type_of_signature(inst) {
            let _ = rebuilt.resolved_return_type.set(rt);
        }
        let _ = rebuilt
            .resolved_type_predicate
            .set(Some(Box::new(new_pred)));
        Arc::new(rebuilt)
    }
}
