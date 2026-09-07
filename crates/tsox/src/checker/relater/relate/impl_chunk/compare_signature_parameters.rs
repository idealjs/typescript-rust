#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn compare_signature_parameters(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        target_count: usize,
        check_mode: SignatureCheckMode,
        strict_variance: bool,
        relation: RelationKind,
    ) -> Ternary {
        let source_count = self.get_parameter_count(source);
        let source_rest = self.get_non_array_rest_type(source);
        let target_rest = self.get_non_array_rest_type(target);
        let mut result = Ternary::True;
        let param_count = if source_rest.is_some() || target_rest.is_some() {
            source_count.min(target_count)
        } else {
            source_count.max(target_count)
        };
        let rest_index = if source_rest.is_some() || target_rest.is_some() {
            param_count.saturating_sub(1) as isize
        } else {
            -1
        };
        for i in 0..param_count {
            let source_type = if i as isize == rest_index {
                self.get_rest_or_any_type_at_position(&source, i)
            } else {
                self.try_get_type_at_position(&source, i)
                    .unwrap_or_else(|| self.any_type())
            };
            let target_type = if i as isize == rest_index {
                self.get_rest_or_any_type_at_position(&target, i)
            } else {
                self.try_get_type_at_position(&target, i)
                    .unwrap_or_else(|| self.any_type())
            };

            if Arc::ptr_eq(&source_type, &target_type)
                && !check_mode.contains(SignatureCheckMode::StrictArity)
            {
                continue;
            }

            let mut source_sig: Option<Arc<Signature>> = None;
            if !check_mode.contains(SignatureCheckMode::Callback)
                && !self.is_instantiated_generic_parameter(&source, i)
            {
                let non_nullable = self.get_non_nullable_type_of(&source_type);
                source_sig = self.get_single_call_signature(&non_nullable);
            }
            let mut target_sig: Option<Arc<Signature>> = None;
            if !check_mode.contains(SignatureCheckMode::Callback)
                && !self.is_instantiated_generic_parameter(&target, i)
            {
                let non_nullable = self.get_non_nullable_type_of(&target_type);
                target_sig = self.get_single_call_signature(&non_nullable);
            }
            let callbacks = source_sig.is_some()
                && target_sig.is_some()
                && self
                    .get_type_predicate_of_signature(source_sig.as_ref().unwrap())
                    .is_none()
                && self
                    .get_type_predicate_of_signature(target_sig.as_ref().unwrap())
                    .is_none()
                && self.type_is_undefined_or_null(&source_type)
                    == self.type_is_undefined_or_null(&target_type);

            let mut related = Ternary::False;
            if callbacks {
                let callback_mode = if check_mode.contains(SignatureCheckMode::StrictArity) {
                    SignatureCheckMode::StrictArity
                } else {
                    SignatureCheckMode::None
                } | if strict_variance {
                    SignatureCheckMode::StrictCallback
                } else {
                    SignatureCheckMode::BivariantCallback
                };

                related = self.compare_signatures_related(
                    target_sig.as_ref().unwrap(),
                    source_sig.as_ref().unwrap(),
                    callback_mode,
                    relation,
                );
            } else {
                if !check_mode.contains(SignatureCheckMode::Callback) && !strict_variance {
                    related = self.compare_types(
                        source_type.clone(),
                        target_type.clone(),
                        relation,
                        false,
                    );
                }
                if related.is_false() {
                    related = self.compare_types(
                        target_type.clone(),
                        source_type.clone(),
                        relation,
                        false,
                    );
                }
            }
            if related.is_false() {
                if self.relater_chain_active {
                    let ts = self.type_to_string(&target_type);
                    let ss = self.type_to_string(&source_type);
                    self.push_relation_head_with_tp_note(
                        &target_type,
                        &source_type,
                        crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                        vec![ts, ss],
                    );
                    let sn = source.parameters.get(i).map(|p| p.name.clone());
                    let tn = target.parameters.get(i).map(|p| p.name.clone());
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            TYPES_OF_PARAMETERS_0_AND_1_ARE_INCOMPATIBLE,
                        vec![sn.unwrap_or_default(), tn.unwrap_or_default()],
                    );
                }
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }
}
