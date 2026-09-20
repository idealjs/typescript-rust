#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    pub(crate) fn infer_from_object_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);

        if Arc::ptr_eq(source, target) && !target_args.is_empty() {
            self.infer_from_type_arguments(state, &target_args, &target_args, &[]);
            return;
        }

        let same_target = match (source.target(), target.target()) {
            (Some(st), Some(tt)) => Arc::ptr_eq(st, tt),
            _ => false,
        };
        if source.object_flags.contains(ObjectFlags::Reference)
            && target.object_flags.contains(ObjectFlags::Reference)
            && (same_target || self.is_array_type(source) && self.is_array_type(target))
        {
            self.infer_from_type_arguments(state, &source_args, &target_args, &[]);
            return;
        }
        if !source_args.is_empty() && !target_args.is_empty() {
            self.infer_from_type_arguments(state, &source_args, &target_args, &[]);
        }
        self.infer_from_properties(state, source, target);
        self.infer_from_signatures(state, source, target);
        self.infer_from_index_types(state, source, target);
    }

    pub(crate) fn infer_from_type_arguments(
        &mut self,
        state: &mut InferenceState,
        source_types: &[Arc<Type>],
        target_types: &[Arc<Type>],
        _variances: &[VarianceFlags],
    ) {
        let count = source_types.len().min(target_types.len());
        for i in 0..count {
            self.infer_from_types(state, &source_types[i], &target_types[i]);
        }
    }

    pub(crate) fn infer_from_properties(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_struct = source.as_structured();
        let target_struct = target.as_structured();
        if let (Some(source_s), Some(target_s)) = (source_struct, target_struct) {
            for target_prop in &target_s.properties {
                for source_prop in &source_s.properties {
                    if source_prop.name == target_prop.name {
                        let source_type = self.get_type_of_symbol(source_prop);
                        let target_type = self.get_type_of_symbol(target_prop);
                        self.infer_from_types(state, &source_type, &target_type);
                        break;
                    }
                }
            }
        }
    }

    pub(crate) fn infer_from_signatures(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Call);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Call);
        if source_sigs.len() == 1 && target_sigs.len() == 1 {
            self.infer_from_signature(state, &source_sigs[0], &target_sigs[0]);
        }
        let source_ctors = self.get_signatures_of_type(source, SignatureKind::Construct);
        let target_ctors = self.get_signatures_of_type(target, SignatureKind::Construct);
        if source_ctors.len() == 1 && target_ctors.len() == 1 {
            self.infer_from_signature(state, &source_ctors[0], &target_ctors[0]);
        }
    }

    pub(crate) fn infer_from_signature(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
    ) {
        // Go applyToParameterTypes：非 rest 参数成对（源无 rest 时按源参数数截断），
        // 目标带 rest 时用源在 rest 起点的展开类型对位（...t: T vs ...rest: infer R
        // 推出 R = T[number][]）
        let source_count = self.get_parameter_count(source);
        let target_count = self.get_parameter_count(target);
        let source_rest = self.get_effective_rest_type(source);
        let target_rest = self.get_effective_rest_type(target);
        let target_non_rest = target_count - usize::from(target_rest.is_some());
        let param_count = if source_rest.is_some() {
            target_non_rest
        } else {
            source_count.min(target_non_rest)
        };
        for i in 0..param_count {
            let st = self.get_type_at_position(source, i);
            let tt = self.get_type_at_position(target, i);
            let save_contra = state.contravariant;
            let save_biv = state.bivariant;
            state.contravariant = true;
            state.bivariant = false;
            self.infer_from_types(state, &tt, &st);
            state.contravariant = save_contra;
            state.bivariant = save_biv;
        }
        if let Some(target_rest_type) = target_rest {
            let source_rest_at = self.source_rest_type_at(source, param_count);
            let save_contra = state.contravariant;
            let save_biv = state.bivariant;
            state.contravariant = true;
            state.bivariant = false;
            self.infer_from_types(state, &target_rest_type, &source_rest_at);
            state.contravariant = save_contra;
            state.bivariant = save_biv;
        }

        let st = self.get_return_type_of_signature(source);
        let tt = self.get_return_type_of_signature(target);
        if let (Some(st), Some(tt)) = (st, tt) {
            if std::env::var_os("TSOX_DEBUG_INFER").is_some() {
                eprintln!(
                    "[infer-sig] ret {} -> {}",
                    self.type_to_string(&st),
                    self.type_to_string(&tt)
                );
            }
            self.infer_from_types(state, &st, &tt);
        } else if std::env::var_os("TSOX_DEBUG_INFER").is_some() {
            eprintln!(
                "[infer-sig] ret MISSING src={} tgt={}",
                source.resolved_return_type.get().is_some(),
                target.resolved_return_type.get().is_some()
            );
        }
    }

    pub(crate) fn infer_from_index_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let mut priority = InferencePriority::None;
        if source.object_flags.intersects(ObjectFlags::Mapped)
            && target.object_flags.intersects(ObjectFlags::Mapped)
        {
            priority = InferencePriority::HomomorphicMappedType;
        }
        let target_infos = self.get_index_infos_of_type(target);
        if self.is_object_type_with_inferable_index(source) {
            for target_info in &target_infos {
                let Some(target_key) = target_info.key_type.clone() else {
                    continue;
                };
                let mut prop_types: Vec<Arc<Type>> = Vec::new();
                for prop in self.get_properties_of_type(source) {
                    let literal_key = self.get_literal_type_from_property(&prop);
                    if self.is_applicable_index_type(&literal_key, &target_key) {
                        let prop_type = self.get_type_of_symbol(&prop);
                        let prop_type = if prop.flags.contains(SymbolFlags::Optional) {
                            self.remove_missing_or_undefined_type(&prop_type)
                        } else {
                            prop_type
                        };
                        prop_types.push(prop_type);
                    }
                }
                for info in self.get_index_infos_of_type(source) {
                    if let Some(src_key) = &info.key_type
                        && self.is_applicable_index_type(src_key, &target_key)
                        && let Some(v) = &info.value_type
                    {
                        prop_types.push(Arc::clone(v));
                    }
                }
                if !prop_types.is_empty()
                    && let Some(tv) = &target_info.value_type
                {
                    let union = self.get_union_type(prop_types);
                    self.infer_with_priority(state, &union, tv, priority);
                }
            }
        }
        for target_info in &target_infos {
            let Some(target_key) = target_info.key_type.clone() else {
                continue;
            };
            if let Some(source_info) = self.get_applicable_index_info(source, &target_key)
                && let (Some(sv), Some(tv)) = (&source_info.value_type, &target_info.value_type)
            {
                self.infer_with_priority(state, sv, tv, priority);
            }
        }
    }

    pub(crate) fn infer_from_matching_types(
        &mut self,
        state: &mut InferenceState,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
        use_identical: bool,
    ) -> (Vec<Arc<Type>>, Vec<Arc<Type>>) {
        let mut remaining_sources: Vec<Arc<Type>> = sources.to_vec();
        let mut remaining_targets: Vec<Arc<Type>> = targets.to_vec();
        let mut i = 0;
        while i < remaining_sources.len() {
            let mut matched = false;
            let mut j = 0;
            while j < remaining_targets.len() {
                let is_match = if use_identical {
                    self.is_type_identical_to(&remaining_sources[i], &remaining_targets[j])
                } else {
                    self.is_type_identical_to(&remaining_sources[i], &remaining_targets[j])
                };
                if is_match {
                    self.infer_from_types(state, &remaining_sources[i], &remaining_targets[j]);
                    remaining_sources.remove(i);
                    remaining_targets.remove(j);
                    matched = true;
                    break;
                }
                j += 1;
            }
            if !matched {
                i += 1;
            }
        }
        (remaining_sources, remaining_targets)
    }

    pub(crate) fn infer_matching_types_identical(
        &mut self,
        state: &mut InferenceState,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
    ) -> (Vec<Arc<Type>>, Vec<Arc<Type>>) {
        self.infer_from_matching_types(state, sources, targets, true)
    }

    pub(crate) fn infer_with_priority(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
        new_priority: InferencePriority,
    ) {
        let save = state.priority;
        state.priority = new_priority;
        self.infer_from_types(state, source, target);
        state.priority = save;
    }

    pub(crate) fn could_contain_type_variables(&self, t: &Type) -> bool {
        t.flags.intersects(TypeFlags::Union)
            || t.flags.intersects(TypeFlags::Intersection)
            || t.flags.intersects(TypeFlags::Object)
            || t.flags.intersects(TypeFlags::TypeParameter)
            || t.flags.intersects(TypeFlags::IndexedAccess)
            || t.flags.intersects(TypeFlags::Conditional)
            || t.flags.intersects(TypeFlags::Substitution)
    }
    pub(crate) fn is_no_infer_type(&self, _t: &Type) -> bool {
        false
    }

    pub(crate) fn is_from_inference_blocked_source(&self, _source: &Type) -> bool {
        false
    }

    pub fn infer_type_arguments(
        &mut self,
        node: &tsox_frontend::ast::Node,
        signature: &Arc<Signature>,
        args: &[Arc<tsox_frontend::ast::Node>],
        context: &mut InferenceContext,
    ) -> Vec<Arc<Type>> {
        if matches!(
            node.kind,
            SyntaxKind::CallExpression | SyntaxKind::NewExpression
        ) {
            if let Some(contextual_type) = self.get_contextual_type_for_call_or_new(node) {
                if let Some(return_type) = self.get_return_type_of_signature(signature) {
                    if self.could_contain_type_variables(&return_type) {
                        self.infer_types(
                            &mut context.inferences,
                            Some(contextual_type),
                            Some(return_type),
                            InferencePriority::ReturnType,
                            false,
                        );
                    }
                }
            }
        }

        let has_rest = signature.has_rest_parameter();
        let rest_index = if has_rest {
            signature.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        for i in 0..args.len() {
            let param_type = if has_rest && i >= rest_index {
                let rest_type = self.get_type_of_symbol(&signature.parameters[rest_index]);
                self.get_array_element_type(&rest_type)
            } else if i < signature.parameters.len() {
                self.get_type_of_symbol(&signature.parameters[i])
            } else {
                continue;
            };
            if self.could_contain_type_variables(&param_type) {
                let arg_type = self.get_type_of_node(&args[i]);
                self.infer_types(
                    &mut context.inferences,
                    Some(arg_type),
                    Some(param_type),
                    InferencePriority::None,
                    false,
                );
            }
        }

        self.get_inferred_types(context)
    }

}

