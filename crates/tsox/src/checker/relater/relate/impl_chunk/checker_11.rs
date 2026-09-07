#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn compare_signatures_related(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        check_mode: SignatureCheckMode,
        relation: RelationKind,
    ) -> Ternary {
        if Arc::ptr_eq(source, target) {
            return Ternary::True;
        }

        let source_is_top = if check_mode.contains(SignatureCheckMode::StrictTopSignature)
            && self.is_top_signature(source)
        {
            true
        } else {
            false
        };
        if !source_is_top && self.is_top_signature(target) {
            return Ternary::True;
        }
        if check_mode.contains(SignatureCheckMode::StrictTopSignature)
            && source_is_top
            && !self.is_top_signature(target)
        {
            return Ternary::False;
        }

        let target_count = self.get_parameter_count(target);
        let source_has_more = if !self.has_effective_rest_parameter(target) {
            if check_mode.contains(SignatureCheckMode::StrictArity) {
                self.has_effective_rest_parameter(source)
                    || self.get_parameter_count(source) > target_count
            } else {
                self.get_min_argument_count(source) > target_count
            }
        } else {
            false
        };
        if source_has_more {
            if self.relater_chain_active && !check_mode.contains(SignatureCheckMode::StrictArity) {
                let min_args = self.get_min_argument_count(source).max(0);
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TARGET_SIGNATURE_PROVIDES_TOO_FEW_ARGUMENTS_EXPECTED_0_OR_MORE_BUT_GOT_1,
                    vec![min_args.to_string(), target_count.to_string()],
                );
            }
            return Ternary::False;
        }

        let mut source = Arc::clone(source);
        let mut target = Arc::clone(target);
        if !source.type_parameters.is_empty()
            && !type_parameters_same(
                source.type_parameters.as_slice(),
                target.type_parameters.as_slice(),
            )
        {
            let canonical_target = self.get_canonical_signature(&target);
            source = self.instantiate_signature_in_context_of(&source, &canonical_target);
            target = canonical_target;
        }

        let strict_variance = !check_mode.contains(SignatureCheckMode::Callback)
            && self.strict_function_types
            && !self.signature_is_method_or_constructor(&target);

        let mut result = Ternary::True;

        let source_this = self.get_this_type_of_signature(&source);
        if let Some(source_this) = source_this {
            if !source_this.flags.contains(TypeFlags::Void) {
                let target_this = self.get_this_type_of_signature(&target);
                if let Some(target_this) = target_this {
                    let mut related = Ternary::False;
                    if !strict_variance {
                        related = self.compare_types(
                            source_this.clone(),
                            target_this.clone(),
                            relation,
                            false,
                        );
                    }
                    if related.is_false() {
                        related = self.compare_types(target_this, source_this, relation, false);
                    }
                    if related.is_false() {
                        return Ternary::False;
                    }
                    result = result.and(related);
                }
            }
        }

        result = result.and(self.compare_signature_parameters(
            &source,
            &target,
            target_count,
            check_mode,
            strict_variance,
            relation,
        ));
        if result.is_false() {
            return Ternary::False;
        }

        result = self.compare_signature_return_type(&source, &target, check_mode, relation, result);

        result
    }
}
