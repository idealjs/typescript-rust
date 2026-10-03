use std::sync::Arc;

use crate::checker::types::*;
use crate::checker::TypeMapper;

impl Type {
    pub fn set_type_parameter_mapper(&mut self, mapper: &TypeMapper) { ::tsox_core::fntrace::enter("set_type_parameter_mapper"); 
        if let TypeData::TypeParameter(d) = &mut self.data {
            d.mapper = Some(Arc::new(mapper.clone()));
        }
    }
}
