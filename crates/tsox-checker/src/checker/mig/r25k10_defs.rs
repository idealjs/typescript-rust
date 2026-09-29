#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::Symbol;

use crate::checker::types_impl_chunk::Type;

thread_local! {
    static MODULE_IMPORT_ATTRIBUTES_TYPES: RefCell<HashMap<usize, Arc<Type>>> =
        RefCell::new(HashMap::new());
}

pub(crate) fn module_import_attributes_types_get(symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
    MODULE_IMPORT_ATTRIBUTES_TYPES.with(|s| s.borrow().get(&(Arc::as_ptr(symbol) as usize)).cloned())
}

pub(crate) fn module_import_attributes_types_insert(symbol: &Arc<Symbol>, t: Arc<Type>) {
    MODULE_IMPORT_ATTRIBUTES_TYPES.with(|s| {
        s.borrow_mut()
            .insert(Arc::as_ptr(symbol) as usize, t);
    });
}
