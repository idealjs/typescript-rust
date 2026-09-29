#![allow(unused_imports)]

use std::sync::Arc;
use tsox_frontend::ast;

use crate::checker::types::{Type, TypeMapper};

impl super::super::checker_checker::Checker {
    pub fn map_type_with_composite_mapper(
        &mut self,
        t: &Arc<Type>,
        m1: Option<&Arc<TypeMapper>>,
        m2: &Arc<TypeMapper>,
    ) -> Arc<Type> {
        let m1 = match m1 {
            None => return m2.map(t),
            Some(m1) => m1,
        };
        let t1 = m1.map(t);
        if !Arc::ptr_eq(&t1, t) {
            return self.instantiate_type(&t1, Some(m2));
        }
        m2.map(t)
    }
}
