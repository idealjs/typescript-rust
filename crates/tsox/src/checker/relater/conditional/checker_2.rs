#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_false_type_from_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_false_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    pub fn conditional_is_distribution_dependent(&self, _t: &Arc<Type>) -> bool {
        true
    }
}
