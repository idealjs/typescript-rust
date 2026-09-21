#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    fn find_tracked_inference_index(
        &self,
        state: &InferenceState,
        target: &Arc<Type>,
    ) -> Option<usize> {
        if !target
            .flags
            .intersects(TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Substitution)
        {
            return None;
        }
        state.inferences.iter().position(|info| {
            crate::checker::utilities::type_parameters_match(&info.type_parameter, target)
        })
    }

    pub(crate) fn infer_to_multiple_types_non_union(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        targets: &[Arc<Type>],
        group_priority: InferencePriority,
    ) {
        let save = state.priority;
        state.priority |= group_priority;
        let mut type_variable_count = 0usize;
        for t in targets {
            if self.find_tracked_inference_index(state, t).is_some() {
                type_variable_count += 1;
            } else {
                self.infer_from_types(state, source, t);
            }
        }
        if type_variable_count > 0 {
            for t in targets {
                if self.find_tracked_inference_index(state, t).is_some() {
                    let save_priority = state.priority;
                    state.priority |= InferencePriority::NakedTypeVariable;
                    self.infer_from_types(state, source, t);
                    state.priority = save_priority;
                }
            }
        }
        state.priority = save;
    }

    pub(crate) fn infer_to_multiple_types_union(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        targets: &[Arc<Type>],
    ) {
        let sources: Vec<Arc<Type>> = if source.flags.contains(TypeFlags::Union) {
            source.types().unwrap_or_default().to_vec()
        } else {
            vec![Arc::clone(source)]
        };
        let mut matched = vec![false; sources.len()];
        let mut inference_circularity = false;
        let mut type_variable_count = 0usize;
        let mut naked_type_variable: Option<Arc<Type>> = None;
        for t in targets {
            if self.find_tracked_inference_index(state, t).is_some() {
                naked_type_variable = Some(Arc::clone(t));
                type_variable_count += 1;
            } else {
                for (i, s) in sources.iter().enumerate() {
                    let save = state.inference_priority;
                    state.inference_priority = InferencePriority::MaxValue;
                    self.infer_from_types(state, s, t);
                    if state.inference_priority == state.priority {
                        matched[i] = true;
                    }
                    if state.inference_priority == InferencePriority::Circularity {
                        inference_circularity = true;
                    }
                    state.inference_priority = if state.inference_priority.bits() < save.bits() {
                        state.inference_priority
                    } else {
                        save
                    };
                }
            }
        }
        if type_variable_count == 0 {
            return;
        }
        if type_variable_count == 1 && !inference_circularity {
            let unmatched: Vec<Arc<Type>> = sources
                .iter()
                .enumerate()
                .filter(|(i, _)| !matched[*i])
                .map(|(_, s)| Arc::clone(s))
                .collect();
            if !unmatched.is_empty()
                && let Some(naked) = &naked_type_variable
            {
                let union = self.get_union_type(unmatched);
                self.infer_from_types(state, &union, naked);
                return;
            }
        }
        for t in targets {
            if self.find_tracked_inference_index(state, t).is_some() {
                self.infer_with_priority(state, source, t, InferencePriority::NakedTypeVariable);
            }
        }
    }
}
