use crate::checker::checker::*;
use std::sync::Arc;

impl Checker {
    pub fn get_mapped_target_or_self(&self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_mapped_target_or_self"); 
        match t.target() {
            Some(x) => Arc::clone(x),
            None => Arc::clone(t),
        }
    }

    pub fn mapped_name_type_is_none(&self, declaration: &Option<Arc<tsox_frontend::ast::Node>>) -> bool { ::tsox_core::fntrace::enter("mapped_name_type_is_none"); 
        declaration.is_some()
    }

    pub fn get_contextual_type_for_binding_element(
        &mut self,
        _node: &Arc<tsox_frontend::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_contextual_type_for_binding_element"); 
        None
    }
}
