#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn symbol_to_string_ex_enclosing(
        &mut self,
        symbol: &Arc<Symbol>,
        _enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        flags: crate::checker::types::SymbolFormatFlags,
    ) -> String {
        self.symbol_to_string_ex(symbol, flags, meaning)
    }

    pub(crate) fn resolve_alias(&mut self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        self.get_merged_symbol(symbol)
    }

    pub(crate) fn get_exports_of_symbol(&self, symbol: &Arc<Symbol>) -> SymbolTable {
        symbol.exports.clone()
    }

    pub(crate) fn get_symbol_if_same_reference(
        &self,
        symbol: &Arc<Symbol>,
        other: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        if symbol.id() == other.id() {
            Some(Arc::clone(symbol))
        } else {
            None
        }
    }

    pub(crate) fn get_parent_of_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        symbol.parent.clone()
    }

    pub(crate) fn sort_symbols(&self, symbols: &mut Vec<Arc<Symbol>>) {
        symbols.sort_by(|a, b| a.name.cmp(&b.name));
    }
}
