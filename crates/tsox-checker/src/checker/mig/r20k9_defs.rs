#![allow(unused_imports)]

use crate::checker::types::{Type, TypeData};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags};

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool { ::tsox_core::fntrace::enter("eq"); 
        self.id == other.id
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) { ::tsox_core::fntrace::enter("hash"); 
        self.id.hash(state);
    }
}

pub fn type_types_list(t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("type_types_list"); 
    match &t.data {
        TypeData::Union(u) => u.union_or_intersection.types.clone(),
        TypeData::Intersection(i) => i.union_or_intersection.types.clone(),
        _ => Vec::new(),
    }
}

pub trait SymbolUpdateExt {
    fn update_flags_and_declarations(&self, flags: SymbolFlags, declarations: Vec<Arc<Node>>);
}

impl SymbolUpdateExt for Arc<Symbol> {
    fn update_flags_and_declarations(&self, flags: SymbolFlags, declarations: Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("update_flags_and_declarations"); 
        let ptr = Arc::as_ptr(self) as *mut Symbol;
        unsafe {
            (*ptr).flags = flags;
            (*ptr).declarations = declarations;
        }
    }
}
