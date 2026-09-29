#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::Symbol;

thread_local! {
    static UNRESOLVED_SYMBOLS: RefCell<HashMap<String, Arc<Symbol>>> =
        RefCell::new(HashMap::new());
}

pub(crate) fn unresolved_symbols_get(path: &str) -> Option<Arc<Symbol>> {
    UNRESOLVED_SYMBOLS.with(|s| s.borrow().get(path).cloned())
}

pub(crate) fn unresolved_symbols_insert(path: String, symbol: Arc<Symbol>) {
    UNRESOLVED_SYMBOLS.with(|s| {
        s.borrow_mut().insert(path, symbol);
    });
}
