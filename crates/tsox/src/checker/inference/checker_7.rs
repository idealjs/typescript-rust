#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn infer_types(
        &mut self,
        inferences: &mut [InferenceInfo],
        original_source: Option<Arc<Type>>,
        original_target: Option<Arc<Type>>,
        priority: InferencePriority,
        contravariant: bool,
    ) {
        let mut state = InferenceState {
            inferences,
            original_source: original_source.clone(),
            original_target: original_target.clone(),
            priority,
            inference_priority: InferencePriority::MaxValue,
            contravariant,
            bivariant: false,
            expanding_flags: ExpandingFlags::None,
            propagation_type: None,
            visited: HashMap::new(),
            depth: 0,
        };
        if let (Some(source), Some(target)) = (original_source, original_target) {
            self.infer_from_types(&mut state, &source, &target);
        }
    }

    pub(crate) fn infer_from_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        if !self.could_contain_type_variables(target) || self.is_no_infer_type(target) {
            return;
        }

        let key = (source.id, target.id);
        if state.visited.contains_key(&key) {
            return;
        }
        state.visited.insert(key, state.priority);
        self.infer_from_types_inner(state, source, target);
        state.visited.remove(&key);
    }

    pub(crate) fn infer_from_types_inner(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        if let (TypeData::Conditional(sc), TypeData::Conditional(tc)) = (&source.data, &target.data)
        {
            let same_root = match (
                sc.root.as_ref().and_then(|r| r.node.as_ref()),
                tc.root.as_ref().and_then(|r| r.node.as_ref()),
            ) {
                (Some(sn), Some(tn)) => sn.id() == tn.id(),
                _ => false,
            };
            let no_infers = |c: &crate::checker::types::ConditionalTypeData| {
                c.root
                    .as_ref()
                    .map(|r| r.infer_type_parameters.is_empty())
                    .unwrap_or(true)
            };
            if same_root && no_infers(sc) && no_infers(tc) {
                if let (Some(scheck), Some(tcheck)) = (&sc.check_type, &tc.check_type) {
                    self.infer_from_types(state, scheck, tcheck);
                }
                if let (Some(sextends), Some(textends)) = (&sc.extends_type, &tc.extends_type) {
                    self.infer_from_types(state, sextends, textends);
                }
                return;
            }
        }

        if Arc::ptr_eq(source, target)
            && source
                .flags
                .intersects(TypeFlags::Union | TypeFlags::Intersection)
        {
            for t in source.types().unwrap_or_default() {
                self.infer_from_types(state, t, t);
            }
            return;
        }

        if target.flags.contains(TypeFlags::Union) {
            let source_types = if source.flags.contains(TypeFlags::Union) {
                source.types().unwrap_or_default().to_vec()
            } else {
                vec![Arc::clone(source)]
            };
            let target_types = target.types().unwrap_or_default().to_vec();
            let (temp_sources, temp_targets) =
                self.infer_from_matching_types(state, &source_types, &target_types, true);
            if temp_targets.is_empty() {
                return;
            }
            let target = self.get_union_type(temp_targets);
            if temp_sources.is_empty() {
                self.infer_with_priority(
                    state,
                    source,
                    &target,
                    InferencePriority::NakedTypeVariable,
                );
                return;
            }
            let source = self.get_union_type(temp_sources);
            self.infer_from_types_union(state, &source, &target);
            return;
        }

        if target.flags.contains(TypeFlags::Intersection) {
            self.infer_from_types_intersection(state, source, target);
            return;
        }

        if target.flags.contains(TypeFlags::TypeParameter) {
            self.infer_to_type_variable(state, source, target);
            return;
        }

        if target.flags.contains(TypeFlags::Object) {
            self.infer_from_object_types(state, source, target);
            return;
        }
    }

    pub(crate) fn infer_from_types_union(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_types = if source.flags.contains(TypeFlags::Union) {
            source.types().unwrap_or_default().to_vec()
        } else {
            vec![Arc::clone(source)]
        };
        let target_types = target.types().unwrap_or_default().to_vec();
        let (sources, targets) =
            self.infer_from_matching_types(state, &source_types, &target_types, false);
        if targets.is_empty() {
            return;
        }
        let target = self.get_union_type(targets);
        if sources.is_empty() {
            self.infer_with_priority(state, source, &target, InferencePriority::NakedTypeVariable);
            return;
        }
        let source = self.get_union_type(sources);
        for t in target.types().unwrap_or(&[]) {
            self.infer_from_types(state, &source, t);
        }
    }

    pub(crate) fn infer_from_types_intersection(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_types = if source.flags.contains(TypeFlags::Intersection) {
            source.types().unwrap_or_default().to_vec()
        } else {
            vec![Arc::clone(source)]
        };
        let target_types = target.types().unwrap_or_default().to_vec();
        let (sources, targets) =
            self.infer_matching_types_identical(state, &source_types, &target_types);
        if sources.is_empty() || targets.is_empty() {
            return;
        }
        let source = self.get_intersection_type(sources);
        let target = self.get_intersection_type(targets);
        for t in target.types().unwrap_or(&[]) {
            self.infer_from_types(state, &source, t);
        }
    }

    pub(crate) fn infer_to_type_variable(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        if self.is_from_inference_blocked_source(source) {
            return;
        }

        let inference_idx = state.inferences.iter().position(|info| {
            crate::checker::utilities::type_parameters_match(&info.type_parameter, target)
        });
        let Some(idx) = inference_idx else { return };

        let priority = state.priority;
        let contravariant = state.contravariant;
        let bivariant = state.bivariant;
        let depth = state.depth;
        let propagation_type = state.propagation_type.clone();

        let mut cleared = false;
        let inference = &mut state.inferences[idx];
        if source.object_flags.contains(ObjectFlags::NonInferrableType) {
            return;
        }
        if !inference.is_fixed {
            let candidate = propagation_type.unwrap_or_else(|| Arc::clone(source));
            if priority.bits() < inference.priority.bits() {
                inference.candidates.clear();
                inference.candidate_depths.clear();
                inference.contra_candidates.clear();
                inference.top_level = true;
                inference.priority = priority;
                cleared = true;
            }
            if priority == inference.priority {
                if contravariant && !bivariant {
                    if !inference
                        .contra_candidates
                        .iter()
                        .any(|c| Arc::ptr_eq(c, &candidate))
                    {
                        if std::env::var_os("TSOX_DEBUG_INFER").is_some() {
                            eprintln!(
                                "[contra-rec] depth={} biv={} tp={} cand={}",
                                depth,
                                bivariant,
                                self.type_to_string(&inference.type_parameter),
                                self.type_to_string(&candidate)
                            );
                        }
                        inference.contra_candidates.push(candidate);
                        cleared = true;
                    }
                } else {
                    if !inference
                        .candidates
                        .iter()
                        .any(|c| Arc::ptr_eq(c, &candidate))
                    {
                        inference.candidates.push(candidate);
                        inference.candidate_depths.push(depth);
                        cleared = true;
                    }
                }
            }
            if !priority.contains(InferencePriority::ReturnType)
                && target.flags.contains(TypeFlags::TypeParameter)
                && inference.top_level
            {
                inference.top_level = false;
                cleared = true;
            }
        }
        let _ = inference;
        if cleared {
            for info in state.inferences.iter_mut() {
                info.inferred_type = None;
            }
        }
        state.inference_priority = if state.inference_priority.bits() < state.priority.bits() {
            state.inference_priority
        } else {
            state.priority
        };
    }

}
