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
        if self
            .global_this_symbol
            .as_ref()
            .is_some_and(|g| Arc::ptr_eq(g, symbol))
        {
            return self.globals.clone();
        }
        symbol.exports.clone()
    }

    pub(crate) fn get_exports_of_symbol_resolved(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> SymbolTable { ::tsox_core::fntrace::enter("get_exports_of_symbol_resolved");
        if self
            .global_this_symbol
            .as_ref()
            .is_some_and(|g| Arc::ptr_eq(g, symbol))
        {
            return self.globals.clone();
        }
        // Go getExportsOfSymbol（checker.go:16867-16876）：Module 位符号经
        // getExportsOfModule 取展开 export * 的 resolved exports；本仓对应
        // get_exports_of_module_table（含缓存）。__export 占位仍在表内，
        // 星号追逐调用方不受影响
        if symbol
            .flags
            .intersects(tsox_frontend::ast::SymbolFlags::MODULE)
        {
            return self.get_exports_of_module_table(symbol);
        }
        symbol.exports.clone()
    }

    pub(crate) fn get_symbol_if_same_reference(
        &mut self,
        symbol: &Arc<Symbol>,
        other: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_if_same_reference");
        let left = self.resolve_alias(&self.get_merged_symbol(symbol));
        let right = self.resolve_alias(&self.get_merged_symbol(other));
        if self.get_merged_symbol(&left).id() == self.get_merged_symbol(&right).id() {
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
