#![allow(unused_imports)]

use crate::checker::types::*;
use std::collections::HashMap;
use std::sync::Arc;

use super::x3::{wrap_type, TracedType};

pub fn traced_conditional_check_type(t: &Type) -> Option<TracedType> {
    if !t.flags.intersects(TypeFlags::Conditional) {
        return None;
    }
    let check_type = t.as_conditional_type()?.check_type.as_deref()?;
    Some(wrap_type(check_type))
}

pub fn traced_conditional_extends_type(t: &Type) -> Option<TracedType> {
    if !t.flags.intersects(TypeFlags::Conditional) {
        return None;
    }
    let extends_type = t.as_conditional_type()?.extends_type.as_deref()?;
    Some(wrap_type(extends_type))
}

pub fn traced_conditional_true_type(t: &Type) -> Option<TracedType> {
    if !t.flags.intersects(TypeFlags::Conditional) {
        return None;
    }
    let resolved = t.as_conditional_type()?.resolved_true_type.get()?;
    Some(wrap_type(resolved))
}

pub fn traced_conditional_false_type(t: &Type) -> Option<TracedType> {
    if !t.flags.intersects(TypeFlags::Conditional) {
        return None;
    }
    let resolved = t.as_conditional_type()?.resolved_false_type.get()?;
    Some(wrap_type(resolved))
}

pub fn copy_with_checker_index(checker_index: u64, args: &HashMap<String, String>) -> HashMap<String, String> {
    let mut with_checker_index = HashMap::with_capacity(args.len() + 1);
    with_checker_index.extend(args.iter().map(|(k, v)| (k.clone(), v.clone())));
    with_checker_index.insert("checkerId".to_string(), checker_index.to_string());
    with_checker_index
}
