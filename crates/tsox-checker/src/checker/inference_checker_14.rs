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
}
