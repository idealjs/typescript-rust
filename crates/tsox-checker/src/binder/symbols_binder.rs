#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    pub(crate) fn new_symbol(
        &mut self,
        flags: SymbolFlags,
        name: impl Into<String>,
    ) -> Arc<Symbol> {
        self.symbol_count += 1;
        Arc::new(Symbol::new(flags, name))
    }
}
