#![allow(unused_imports)]

use crate::checker::symbolaccessibility::*;

impl Checker {
    pub(crate) fn symbol_to_string_ex_enclosing(
        &mut self,
        symbol: &Arc<Symbol>,
        _enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        flags: crate::checker::types::SymbolFormatFlags,
    ) -> String { ::tsox_core::fntrace::enter("symbol_to_string_ex_enclosing"); 
        self.symbol_to_string_ex(symbol, flags, meaning)
    }

    pub(crate) fn resolve_alias(&mut self, symbol: &Arc<Symbol>) -> Arc<Symbol> { ::tsox_core::fntrace::enter("resolve_alias"); 
        if symbol.flags.intersects(SymbolFlags::Alias) {
            let resolved = self.resolve_alias_base(Arc::clone(symbol));
            if !Arc::ptr_eq(&resolved, symbol) {
                return resolved;
            }
        }
        self.get_merged_symbol(symbol)
    }

    pub(crate) fn get_exports_of_symbol(&self, symbol: &Arc<Symbol>) -> SymbolTable { ::tsox_core::fntrace::enter("get_exports_of_symbol"); 
        symbol.exports.clone()
    }

    pub(crate) fn get_symbol_if_same_reference(
        &self,
        symbol: &Arc<Symbol>,
        other: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_if_same_reference"); 
        if symbol.id() == other.id() {
            Some(Arc::clone(symbol))
        } else {
            None
        }
    }

    pub(crate) fn get_parent_of_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_parent_of_symbol"); 
        symbol.parent().clone()
    }

    pub(crate) fn sort_symbols(&self, symbols: &mut Vec<Arc<Symbol>>) { ::tsox_core::fntrace::enter("sort_symbols"); 
        symbols.sort_by(|a, b| a.name.cmp(&b.name));
    }
}
