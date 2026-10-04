#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use std::sync::{Arc, OnceLock};

use crate::checker::types_impl_chunk::ConstrainedTypeData;
use crate::checker::types_impl_chunk_2::TypeParameterData;

pub fn get_recursion_identity(t: &Arc<Type>) -> crate::checker::relater_recursion_identity::RecursionIdentity { ::tsox_core::fntrace::enter("get_recursion_identity"); 
    crate::checker::relater_recursion_identity::RecursionIdentity::Type(t.id)
}

impl Checker {
    pub fn new_type_parameter(&mut self, symbol: Option<Arc<Symbol>>) -> Arc<Type> { ::tsox_core::fntrace::enter("new_type_parameter"); 
        let data = TypeParameterData {
            constrained: ConstrainedTypeData::default(),
            constraint: None,
            target: None,
            mapper: None,
            is_this_type: false,
            resolved_default_type: OnceLock::new(),
        };
        let mut t = self.new_type(
            TypeFlags::TypeParameter,
            ObjectFlags::empty(),
            TypeData::TypeParameter(data),
        );
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            t_mut.symbol = symbol;
        }
        t
    }

    pub fn get_global_promise_type(&self) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_promise_type"); 
        self.global_promise_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type())
    }

    pub fn get_global_iterable_type_checked(&mut self) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_iterable_type_checked"); 
        self.get_global_type_by_name("Iterable")
            .unwrap_or_else(|| self.any_type())
    }

    pub fn get_alias_for_type_node(&mut self, node: &Arc<Node>) -> Option<crate::checker::types::TypeAlias> { ::tsox_core::fntrace::enter("get_alias_for_type_node"); 
        let symbol = self.get_alias_symbol_for_type_node(node)?;
        let type_arguments = self.get_type_arguments_for_alias_symbol(Some(&symbol));
        Some(crate::checker::types::TypeAlias::new(Some(symbol), type_arguments))
    }
}
