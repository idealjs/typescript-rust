#![allow(unused_imports)]

use std::sync::Arc;

use crate::checker::checker_checker_checker::Checker;
use crate::checker::mapper::new_function_type_mapper;
use crate::checker::types::{Type, TypeFlags, TypeMapper};

impl Checker {
    pub(crate) fn unique_literal_mapper(&self) -> Arc<TypeMapper> { ::tsox_core::fntrace::enter("unique_literal_mapper"); 
        let unique_literal_type = Arc::clone(&self.unique_literal_type);
        Arc::new(new_function_type_mapper(move |t: &Arc<Type>| {
            if t.flags.intersects(TypeFlags::TypeParameter) {
                Arc::clone(&unique_literal_type)
            } else {
                Arc::clone(t)
            }
        }))
    }
}
