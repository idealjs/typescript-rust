#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use crate::checker::types::{Type, TypeFlags};

pub(crate) fn substitution_base_type(t: &Type) -> Option<Arc<Type>> {
    if !t.flags.intersects(TypeFlags::Substitution) {
        return None;
    }
    t.as_substitution_type()?.base_type.clone()
}

pub(crate) fn substitution_constraint_type(t: &Type) -> Option<Arc<Type>> {
    if !t.flags.intersects(TypeFlags::Substitution) {
        return None;
    }
    t.as_substitution_type()?.constraint.clone()
}
