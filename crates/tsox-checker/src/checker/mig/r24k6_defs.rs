use std::sync::Arc;

use tsox_frontend::ast::Symbol;

use crate::checker::checker::Checker;

impl Checker {
    pub fn get_global_extract_symbol(&mut self) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_extract_symbol"); 
        let resolver = self.get_global_type_alias_resolver("Extract", 2, true);
        resolver(self)
    }

    pub fn get_global_non_nullable_type_alias_or_nil(&mut self) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_non_nullable_type_alias_or_nil"); 
        let resolver = self.get_global_type_alias_resolver("NonNullable", 1, false);
        resolver(self)
    }
}
