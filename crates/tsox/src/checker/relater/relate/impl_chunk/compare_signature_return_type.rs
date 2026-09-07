#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn compare_signature_return_type(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        check_mode: SignatureCheckMode,
        relation: RelationKind,
        mut result: Ternary,
    ) -> Ternary {
        if !check_mode.contains(SignatureCheckMode::IgnoreReturnTypes) {
            let target_return = self.get_non_circular_return_type_of_signature(&target);

            let target_return_own_tp = target_return.flags.contains(TypeFlags::TypeParameter)
                && target
                    .type_parameters
                    .iter()
                    .any(|tp| crate::checker::utilities::type_parameters_match(tp, &target_return));
            if !Arc::ptr_eq(&target_return, &self.void_type())
                && !target_return.flags.contains(TypeFlags::Any)
                && !(target_return.flags.contains(TypeFlags::TypeParameter)
                    && !target_return_own_tp)
            {
                let source_return = self.get_non_circular_return_type_of_signature(&source);
                let target_type_predicate = self.get_type_predicate_of_signature(&target).cloned();
                if let Some(target_tp) = target_type_predicate {
                    let source_tp = self.get_type_predicate_of_signature(&source).cloned();
                    match source_tp {
                        Some(source_tp) => {
                            result = result.and(self.compare_type_predicate_related_to(
                                &source_tp, &target_tp, relation,
                            ));
                        }
                        None => {
                            if matches!(
                                target_tp.kind,
                                TypePredicateKind::Identifier | TypePredicateKind::This
                            ) {
                                return Ternary::False;
                            }
                        }
                    }
                    if result.is_false() {
                        return result;
                    }
                } else {
                    let mut related = Ternary::False;
                    if check_mode.contains(SignatureCheckMode::BivariantCallback) {
                        related = self.compare_types(
                            target_return.clone(),
                            source_return.clone(),
                            relation,
                            false,
                        );
                    }
                    if related.is_false() {
                        related = self.compare_types(
                            source_return.clone(),
                            target_return.clone(),
                            relation,
                            false,
                        );
                    }
                    result = result.and(related);
                    if result.is_false() {
                        if self.relater_chain_active {
                            let sr_head = self.type_to_string(&source_return);
                            let tr_head = self.type_to_string(&target_return);
                            self.push_relation_head_with_tp_note(
                            &source_return,
                            &target_return,
                            crate::diagnostics::messages_generated::
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                            vec![sr_head, tr_head],
                        );
                            let no_args =
                                source.parameters.is_empty() && target.parameters.is_empty();
                            let construct = source
                                .flags
                                .contains(crate::checker::types::SignatureFlags::Construct);
                            let message = match (construct, no_args) {
                            (false, true) => crate::diagnostics::messages_generated::
                                CALL_SIGNATURES_WITH_NO_ARGUMENTS_HAVE_INCOMPATIBLE_RETURN_TYPES_0_AND_1,
                            (true, true) => crate::diagnostics::messages_generated::
                                CONSTRUCT_SIGNATURES_WITH_NO_ARGUMENTS_HAVE_INCOMPATIBLE_RETURN_TYPES_0_AND_1,
                            (false, false) => crate::diagnostics::messages_generated::
                                CALL_SIGNATURE_RETURN_TYPES_0_AND_1_ARE_INCOMPATIBLE,
                            (true, false) => crate::diagnostics::messages_generated::
                                CONSTRUCT_SIGNATURE_RETURN_TYPES_0_AND_1_ARE_INCOMPATIBLE,
                        };
                            let sr = self.type_to_string(&source_return);
                            let tr = self.type_to_string(&target_return);
                            self.relater_report_error(message, vec![sr, tr]);
                        }
                        return result;
                    }
                }
            }
        }
        result
    }
}
