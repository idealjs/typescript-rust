use std::sync::Arc;

use super::x3::{traced_type_adapter, TracedType};
use crate::checker::types::Type;

pub fn wrap_type(t: Option<&Type>) -> Option<TracedType> { ::tsox_core::fntrace::enter("wrap_type"); 
    t.map(traced_type_adapter::new)
}

pub fn wrap_types(types: &[Arc<Type>]) -> Vec<TracedType> { ::tsox_core::fntrace::enter("wrap_types"); 
    if types.is_empty() {
        return Vec::new();
    }
    types.iter().filter_map(|t| wrap_type(Some(t))).collect()
}
