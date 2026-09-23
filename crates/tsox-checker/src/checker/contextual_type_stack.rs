use std::sync::Arc;

use crate::checker::inference::*;
use crate::checker::types::{Type, TypeData, TypeFlags, TYPE_FLAGS_ANY_OR_UNKNOWN};

pub(crate) type ContextualReturnMapper = (Vec<Arc<Type>>, Vec<Arc<Type>>);

pub(crate) struct ContextualFrame {
    pub node_id: u64,
    pub type_: Arc<Type>,
    pub return_mapper: Option<ContextualReturnMapper>,
}

impl Checker {
    pub(crate) fn push_contextual_frame(
        &mut self,
        node_id: u64,
        type_: Arc<Type>,
        return_mapper: Option<ContextualReturnMapper>,
    ) -> usize {
        self.contextual_arg_frames.push(ContextualFrame {
            node_id,
            type_,
            return_mapper,
        });
        self.contextual_arg_frames.len()
    }

    pub(crate) fn pop_contextual_frame(&mut self, saved_len: usize) {
        self.contextual_arg_frames.truncate(saved_len);
    }

    pub(crate) fn pushed_contextual_type(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
    ) -> Option<Arc<Type>> {
        let node_id = node.id();
        for frame in self.contextual_arg_frames.iter().rev() {
            if frame.node_id == node_id {
                let t = Arc::clone(&frame.type_);
                let mapper = frame.return_mapper.clone();
                return Some(self.apply_contextual_return_mapper(t, mapper));
            }
        }
        None
    }

    fn apply_contextual_return_mapper(
        &mut self,
        t: Arc<Type>,
        mapper: Option<ContextualReturnMapper>,
    ) -> Arc<Type> {
        let Some((params, subs)) = mapper else {
            return t;
        };
        let instantiable = TypeFlags::TypeParameter
            | TypeFlags::IndexedAccess
            | TypeFlags::Conditional
            | TypeFlags::Substitution;
        if !t.flags.intersects(instantiable | TypeFlags::Union | TypeFlags::Intersection) {
            return t;
        }
        let instantiated = self.instantiate_instantiable_types(&t, &params, &subs);
        if instantiated.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN) {
            return t;
        }
        instantiated
    }

    fn instantiate_instantiable_types(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        subs: &[Arc<Type>],
    ) -> Arc<Type> {
        let instantiable = TypeFlags::TypeParameter
            | TypeFlags::IndexedAccess
            | TypeFlags::Conditional
            | TypeFlags::Substitution;
        if t.flags.intersects(instantiable) {
            return self.substitute_infer_type_parameters(t, params, subs);
        }
        match &t.data {
            TypeData::Union(u) => {
                let mapped: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|c| self.instantiate_instantiable_types(c, params, subs))
                    .collect();
                self.get_union_type(mapped)
            }
            TypeData::Intersection(i) => {
                let mapped: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|c| self.instantiate_instantiable_types(c, params, subs))
                    .collect();
                self.get_intersection_type(mapped)
            }
            _ => Arc::clone(t),
        }
    }

    pub(crate) fn build_return_type_mapper(
        &mut self,
        type_parameters: &[Arc<Type>],
        source: Arc<Type>,
        target: Arc<Type>,
    ) -> Option<ContextualReturnMapper> {
        let inferences: Vec<InferenceInfo> = type_parameters
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut return_context = InferenceContext::new(inferences);
        self.infer_types(
            &mut return_context.inferences,
            Some(source),
            Some(target),
            InferencePriority::None,
            false,
        );
        let has_candidates = return_context
            .inferences
            .iter()
            .any(|i| !i.candidates.is_empty() || !i.contra_candidates.is_empty());
        if !has_candidates {
            return None;
        }
        let inferred = self.get_inferred_types(&return_context);
        let subs: Vec<Arc<Type>> = type_parameters
            .iter()
            .zip(inferred.into_iter())
            .map(|(tp, t)| {
                if t.flags.contains(TypeFlags::Unknown) {
                    Arc::clone(tp)
                } else {
                    t
                }
            })
            .collect();
        Some((type_parameters.to_vec(), subs))
    }
}
