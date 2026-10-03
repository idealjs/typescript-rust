#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use std::sync::Arc;
use tsox_frontend::ast::Symbol;

pub fn merge_global_symbol(checker: &mut Checker, symbol: &Arc<Symbol>) { ::tsox_core::fntrace::enter("merge_global_symbol"); 
    match checker.globals.get(&symbol.name).cloned() {
        Some(global_symbol) => {
            checker.merge_global_symbols(&global_symbol, symbol);
            checker.globals.insert(symbol.name.clone(), global_symbol);
        }
        None => {
            let merged = checker.get_merged_symbol(symbol);
            checker.globals.insert(symbol.name.clone(), merged);
        }
    }
}

pub fn get_global_nan_symbol_or_nil(checker: &mut Checker) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_nan_symbol_or_nil"); 
    (checker.get_global_value_symbol_resolver("NaN", false))(checker)
}

pub fn contains_missing_type(checker: &Checker, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("contains_missing_type"); 
    let missing = checker.missing_type();
    Arc::ptr_eq(t, &missing)
        || t.flags.intersects(TypeFlags::Union)
            && t.types()
                .is_some_and(|types| Arc::ptr_eq(&types[0], &missing))
}
