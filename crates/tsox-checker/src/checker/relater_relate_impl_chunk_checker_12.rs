#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub fn compare_type_predicate_related_to(
        &mut self,
        source: &TypePredicate,
        target: &TypePredicate,
        relation: RelationKind,
    ) -> Ternary {
        if source.kind != target.kind {
            return Ternary::False;
        }
        if matches!(
            source.kind,
            TypePredicateKind::Identifier | TypePredicateKind::AssertsIdentifier
        ) && source.parameter_index != target.parameter_index
        {
            return Ternary::False;
        }
        match (&source.t, &target.t) {
            (None, None) => Ternary::True,
            (Some(_s), None) => Ternary::True,
            (Some(s), Some(t)) => self.compare_types(s.clone(), t.clone(), relation, false),
            (None, Some(_)) => Ternary::False,
        }
    }

    pub fn compare_types(
        &mut self,
        source: Arc<Type>,
        target: Arc<Type>,
        relation: RelationKind,
        _report_errors: bool,
    ) -> Ternary {
        if self.is_type_related_to(&source, &target, relation) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub(crate) fn signature_is_method_or_constructor(&self, sig: &Arc<Signature>) -> bool {
        let Some(decl) = sig.declaration.as_ref() else {
            return false;
        };
        matches!(
            decl.kind,
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature | SyntaxKind::Constructor
        )
    }

    pub fn signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        kind: SignatureKind,
        relation: RelationKind,
    ) -> Ternary {
        if Arc::ptr_eq(source, &self.any_function_type()) {
            return Ternary::True;
        }

        if Arc::ptr_eq(target, &self.any_function_type()) {
            return Ternary::False;
        }

        let source_sigs = self.get_signatures_of_type(source, kind);
        let target_sigs = self.get_signatures_of_type(target, kind);

        if kind == SignatureKind::Construct && !source_sigs.is_empty() && !target_sigs.is_empty() {}

        if relation == RelationKind::Identity {
            return self.signatures_identical_to(source, target, kind);
        }

        let check_mode = match relation {
            RelationKind::Subtype => SignatureCheckMode::StrictTopSignature,
            RelationKind::StrictSubtype => SignatureCheckMode::from_bits_truncate(
                SignatureCheckMode::StrictTopSignature.bits()
                    | SignatureCheckMode::StrictArity.bits(),
            ),
            _ => SignatureCheckMode::None,
        };

        let mut result = Ternary::True;

        let source_instantiated = source.object_flags.contains(ObjectFlags::Instantiated);
        let target_instantiated = target.object_flags.contains(ObjectFlags::Instantiated);
        let same_target = match (source.target(), target.target()) {
            (Some(s), Some(t)) => Arc::ptr_eq(&s, &t),
            _ => false,
        };
        if (source_instantiated && target_instantiated && same_target)
            || (source.object_flags.contains(ObjectFlags::Reference)
                && target.object_flags.contains(ObjectFlags::Reference)
                && same_target)
        {
            let min_len = source_sigs.len().min(target_sigs.len());
            for i in 0..min_len {
                let s = self.get_erased_signature(&source_sigs[i]);
                let t = self.get_erased_signature(&target_sigs[i]);
                let related = self.compare_signatures_related(&s, &t, check_mode, relation);
                if related.is_false() {
                    return Ternary::False;
                }
                result = result.and(related);
            }

            if source_sigs.len() != target_sigs.len() {
                for t in &target_sigs[min_len..] {
                    let t = self.get_erased_signature(t);
                    let mut found = false;
                    for s in &source_sigs[min_len..] {
                        let s = self.get_erased_signature(s);
                        let related = self.compare_signatures_related(&s, &t, check_mode, relation);
                        if !related.is_false() {
                            result = result.and(related);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Ternary::False;
                    }
                }
            }
        } else if source_sigs.len() == 1 && target_sigs.len() == 1 {
            let erase = relation == RelationKind::Comparable;
            let s = if erase {
                self.get_erased_signature(&source_sigs[0])
            } else {
                Arc::clone(&source_sigs[0])
            };
            let t = if erase {
                self.get_erased_signature(&target_sigs[0])
            } else {
                Arc::clone(&target_sigs[0])
            };
            result = self.compare_signatures_related(&s, &t, check_mode, relation);
        } else {
            for t in &target_sigs {
                let t = self.get_erased_signature(t);
                let chain_len_before = self.relater_error_chain.len();
                let mut should_elaborate_errors = self.relater_chain_active;
                let mut found = false;
                for s in &source_sigs {
                    let s = self.get_erased_signature(s);
                    let saved_active = self.relater_chain_active;
                    if !should_elaborate_errors {
                        self.relater_chain_active = false;
                    }
                    let related = self.compare_signatures_related(&s, &t, check_mode, relation);
                    self.relater_chain_active = saved_active;
                    if !related.is_false() {
                        result = result.and(related);
                        self.relater_error_chain.truncate(chain_len_before);
                        found = true;
                        break;
                    }
                    should_elaborate_errors = false;
                }
                if !found {
                    if should_elaborate_errors {
                        let source_str = self.type_to_string(source);
                        let sig_str = self.signature_display_colon(
                            &t,
                            if kind == SignatureKind::Construct { "new " } else { "" },
                        );
                        self.relater_report_error(
                            tsox_core::diagnostics::messages_generated::
                                TYPE_0_PROVIDES_NO_MATCH_FOR_THE_SIGNATURE_1,
                            vec![source_str, sig_str],
                        );
                    }
                    return Ternary::False;
                }
            }
        }
        result
    }

    pub fn signatures_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        kind: SignatureKind,
    ) -> Ternary {
        let source_sigs = self.get_signatures_of_type(source, kind);
        let target_sigs = self.get_signatures_of_type(target, kind);
        if source_sigs.len() != target_sigs.len() {
            return Ternary::False;
        }
        let mut result = Ternary::True;
        for i in 0..source_sigs.len() {
            let related = self.compare_signatures_identical(
                &source_sigs[i],
                &target_sigs[i],
                false,
                false,
                false,
            );
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    pub fn compare_signatures_identical(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        _partial_match: bool,
        _ignore_this_types: bool,
        ignore_return_types: bool,
    ) -> Ternary {
        if source.type_parameters.len() != target.type_parameters.len() {
            return Ternary::False;
        }
        if !target.type_parameters.is_empty() {
            let mapper = crate::checker::mapper::new_array_type_mapper(
                source.type_parameters.iter().map(Arc::clone).collect(),
                target.type_parameters.iter().map(Arc::clone).collect(),
            );
            for i in 0..target.type_parameters.len() {
                let s = Arc::clone(&source.type_parameters[i]);
                let t = Arc::clone(&target.type_parameters[i]);
                if Arc::ptr_eq(&s, &t) {
                    continue;
                }
                let s_constraint = self
                    .get_constraint_of_type_parameter(&s)
                    .unwrap_or_else(|| self.unknown_type());
                let t_constraint = self
                    .get_constraint_of_type_parameter(&t)
                    .unwrap_or_else(|| self.unknown_type());
                let s_constraint = mapper.map(&s_constraint);
                let s_default = self
                    .get_default_from_type_parameter(&s)
                    .unwrap_or_else(|| self.unknown_type());
                let s_default = mapper.map(&s_default);
                let t_default = self
                    .get_default_from_type_parameter(&t)
                    .unwrap_or_else(|| self.unknown_type());
                if !self.is_type_related_to(&s_constraint, &t_constraint, RelationKind::Identity)
                    || !self.is_type_related_to(&s_default, &t_default, RelationKind::Identity)
                {
                    return Ternary::False;
                }
            }
        }
        let mut mode = SignatureCheckMode::StrictArity;
        if ignore_return_types {
            mode |= SignatureCheckMode::IgnoreReturnTypes;
        }
        // Go compareSignaturesIdentical：源签名以目标类型参数直接擦除
        //（newTypeMapper + eraseTypeParameters），不走推断式上下文实例化
        //（推断会把裸 T 擦成 any，吞掉 T 与 any 的不一致）
        let target_params: Vec<Arc<Type>> =
            target.type_parameters.iter().map(Arc::clone).collect();
        let source = if target_params.is_empty() {
            Arc::clone(source)
        } else {
            self.get_signature_instantiation(source, &target_params)
        };
        self.compare_signatures_related(&source, target, mode, RelationKind::Identity)
    }

    pub fn has_effective_rest_parameter(&mut self, sig: &Arc<Signature>) -> bool {
        if !sig.has_rest_parameter() {
            return false;
        }
        let Some(last) = sig.parameters.last() else {
            return true;
        };
        let rest_type = self.get_type_of_symbol(last);
        if is_tuple_type(&rest_type) {
            if let TypeData::Tuple(t) = &rest_type.data {
                return t.combined_flags.contains(ElementFlags::Variadic);
            }
        }
        true
    }

    pub fn get_parameter_count(&mut self, sig: &Arc<Signature>) -> usize {
        let length = sig.parameters.len();
        if !sig.has_rest_parameter() {
            return length;
        }
        let Some(last) = sig.parameters.last() else {
            return length;
        };
        let rest_type = self.get_type_of_symbol(last);
        if is_tuple_type(&rest_type) {
            if let TypeData::Tuple(t) = &rest_type.data {
                let variadic_offset = if t.combined_flags.contains(ElementFlags::Variadic) {
                    0
                } else {
                    1
                };
                return length + t.fixed_length - variadic_offset;
            }
        }
        length
    }

    pub fn get_min_argument_count(&mut self, sig: &Arc<Signature>) -> usize {
        if sig.resolved_min_argument_count != -1 {
            return sig.resolved_min_argument_count.max(0) as usize;
        }

        let mut min_argument_count: i32 = -1;
        if sig.has_rest_parameter() {
            if let Some(last) = sig.parameters.last() {
                let rest_type = self.get_type_of_symbol(last);
                if is_tuple_type(&rest_type) {
                    if let TypeData::Tuple(t) = &rest_type.data {
                        let first_optional = t
                            .element_infos
                            .iter()
                            .position(|info| !info.flags.contains(ElementFlags::Required));
                        let required_count = match first_optional {
                            Some(i) => i,
                            None => t.fixed_length,
                        };
                        if required_count > 0 {
                            min_argument_count = (sig.parameters.len() - 1 + required_count) as i32;
                        }
                    }
                }
            }
        }
        if min_argument_count == -1 {
            min_argument_count = sig.min_argument_count;
        }

        let mut mc = min_argument_count;
        let mut i = mc - 1;
        while i >= 0 {
            match self.try_get_type_at_position(sig, i as usize) {
                Some(t) if t.flags.contains(TypeFlags::Void) => {
                    mc = i;
                }
                _ => break,
            }
            i -= 1;
        }
        mc.max(0) as usize
    }

    pub fn get_type_at_position(&mut self, sig: &Arc<Signature>, pos: usize) -> Arc<Type> {
        self.try_get_type_at_position(sig, pos)
            .unwrap_or_else(|| self.any_type())
    }
}
