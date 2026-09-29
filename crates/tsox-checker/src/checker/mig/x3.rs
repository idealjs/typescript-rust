use super::m3a::format_type_flags;
use crate::checker::Checker;
use crate::checker::types::{AccessFlags, ConditionalTypeData, ObjectFlags, TupleTypeData, Type, TypeFlags};
use crate::checker::types_type_flags_instantiable_non_primitive::Ternary;
use tsox_core::core::tristate::Tristate;

impl TupleTypeData {
    pub(crate) fn fixed_length(&self) -> usize {
        self.fixed_length
    }
}

impl ConditionalTypeData {
    pub(crate) fn extends_type(&self) -> Option<&Arc<Type>> {
        self.extends_type.as_ref()
    }
}

impl Checker {
    pub(crate) fn get_best_match_indexed_access_type_or_undefined(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        name_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let idx = self.get_indexed_access_type_or_undefined(target, name_type, AccessFlags::None, None, None);
        if idx.is_some() {
            return idx;
        }
        if target.flags.intersects(TypeFlags::Union) {
            let best = self.get_best_matching_type(source, target, &|_a, _b| Ternary::True);
            if let Some(best) = best {
                return self.get_indexed_access_type_or_undefined(&best, name_type, AccessFlags::None, None, None);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TracedType {
    pub(crate) t: *const Type,
}

impl TracedType {
    pub(crate) fn as_type(&self) -> &Type {
        unsafe { &*self.t }
    }
}

pub(crate) mod traced_type_adapter {
    use super::*;

    pub(crate) fn new(t: &Type) -> TracedType {
        TracedType { t }
    }
}

pub(crate) fn wrap_type(t: &Type) -> TracedType {
    TracedType { t }
}

pub(crate) fn wrap_types(types: &[Arc<Type>]) -> Vec<TracedType> {
    if types.is_empty() {
        return Vec::new();
    }
    types.iter().map(|t| wrap_type(t)).collect()
}

use std::sync::Arc;

pub(crate) struct TracedTypeAdapter {
    pub(crate) t: *const Type,
}

impl TracedTypeAdapter {
    pub(crate) fn evolving_array_element_type(&self) -> Option<TracedType> {
        let t = unsafe { &*self.t };
        if !t.flags.intersects(TypeFlags::Object) || !t.object_flags.intersects(ObjectFlags::EvolvingArray) {
            return None;
        }
        t.as_evolving_array_type()?
            .element_type
            .as_deref()
            .map(wrap_type)
    }

    pub(crate) fn evolving_array_final_type(&self) -> Option<TracedType> {
        let t = unsafe { &*self.t };
        if !t.flags.intersects(TypeFlags::Object) || !t.object_flags.intersects(ObjectFlags::EvolvingArray) {
            return None;
        }
        t.as_evolving_array_type()?
            .final_array_type
            .get()
            .map(|t| wrap_type(t.as_ref()))
    }

    pub(crate) fn format_flags(&self) -> Vec<&'static str> {
        let t = unsafe { &*self.t };
        format_type_flags(t.flags)
    }
}
