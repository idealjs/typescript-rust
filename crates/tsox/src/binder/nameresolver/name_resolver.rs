#![allow(dead_code)]

use crate::core::compiler_options::CompilerOptions;
use crate::core::tristate::Tristate;
use crate::diagnostics::Message;
use std::sync::Arc;

use super::*;

pub struct NameResolver {
    pub compiler_options: Option<Arc<CompilerOptions>>,
    pub get_symbol_of_declaration_fn: Option<Box<dyn Fn(&Arc<Node>) -> Option<Arc<Symbol>>>>,
    pub error_fn: Option<Box<dyn Fn(&Arc<Node>, &Message, &[String]) -> Option<Diagnostic>>>,
    pub globals: SymbolTable,
    pub arguments_symbol: Option<Arc<Symbol>>,
    pub require_symbol: Option<Arc<Symbol>>,
    pub lookup_fn: Option<Box<dyn Fn(&SymbolTable, &str, SymbolFlags) -> Option<Arc<Symbol>>>>,
    pub symbol_referenced_fn: Option<Box<dyn Fn(&Arc<Symbol>, SymbolFlags)>>,
    pub set_requires_scope_change_cache_fn: Option<Box<dyn Fn(&Arc<Node>, Tristate)>>,
    pub get_requires_scope_change_cache_fn: Option<Box<dyn Fn(&Arc<Node>) -> Tristate>>,
    pub on_property_with_invalid_initializer_fn:
        Option<Box<dyn Fn(&Arc<Node>, &str, &Arc<Node>, Option<&Arc<Symbol>>) -> bool>>,
    pub on_failed_to_resolve_symbol_fn:
        Option<Box<dyn Fn(&Arc<Node>, &str, SymbolFlags, &Message)>>,
    pub on_successfully_resolved_symbol_fn: Option<
        Box<
            dyn Fn(
                &Arc<Node>,
                &Arc<Symbol>,
                SymbolFlags,
                Option<&Arc<Node>>,
                Option<&Arc<Node>>,
                bool,
            ),
        >,
    >,
}

impl Default for NameResolver {
    fn default() -> Self {
        Self::new()
    }
}
