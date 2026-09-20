#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub fn get_base_signature(&mut self, sig: &Arc<Signature>) -> Arc<Signature> {
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
        self.get_signature_instantiation(sig, &base_constraints)
    }

    fn single_signature_of_kind(
        &mut self,
        t: &Arc<Type>,
        construct: bool,
    ) -> Option<Arc<Signature>> {
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
    ) -> Option<Arc<Type>> {
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

        let n = outer.inferences.len();
        let outer_tps: Vec<Arc<Type>> = outer
            .inferences
            .iter()
            .map(|i| Arc::clone(&i.type_parameter))
            .collect();
        let mut effective: Vec<Arc<Type>> = Vec::with_capacity(n);
        let mut any_fixed = false;
        for i in 0..n {
            let needs_fixing = {
                let info = &outer.inferences[i];
                info.is_fixed || !info.candidates.is_empty() || !info.contra_candidates.is_empty()
            };
            if needs_fixing {
                let t = outer.inferences[i]
                    .inferred_type
                    .clone()
                    .unwrap_or_else(|| self.get_inferred_type(outer, i));
                let info = &mut outer.inferences[i];
                info.is_fixed = true;
                info.inferred_type = Some(Arc::clone(&t));
                any_fixed = true;
                effective.push(t);
            } else {
                effective.push(Arc::clone(&outer_tps[i]));
            }
        }
        let ctx_sig_inst = if any_fixed {
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
        let inst = self.get_signature_instantiation(&sig, &inferred);
        let inst = self.fixup_instantiated_predicate(&sig, &inst, &inner_tps, &inferred);
        Some(self.create_function_or_constructor_type(vec![inst], construct))
    }

    fn substitute_signature_outer(
        &mut self,
        sig: &Arc<Signature>,
        outer_tps: &[Arc<Type>],
        effective: &[Arc<Type>],
    ) -> Arc<Signature> {
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
        if let Some(rt) = self.get_return_type_of_signature(sig) {
            let sub = self.substitute_infer_type_parameters(&rt, outer_tps, effective);
            let _ = inst.resolved_return_type.set(sub);
        }
        inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
        Arc::new(inst)
    }

    fn fixup_instantiated_predicate(
        &mut self,
        original: &Arc<Signature>,
        inst: &Arc<Signature>,
        tps: &[Arc<Type>],
        args: &[Arc<Type>],
    ) -> Arc<Signature> {
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
        rebuilt.resolved_type_predicate = Some(Box::new(new_pred));
        Arc::new(rebuilt)
    }
}
