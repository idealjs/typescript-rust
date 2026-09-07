use crate::ast::*;
use crate::diagnostics::Message;
use std::sync::Arc;

pub struct ReferenceResolverHooks {
    pub resolve_name_fn: Option<
        Box<
            dyn Fn(
                &Arc<Node>,
                &str,
                SymbolFlags,
                Option<&Message>,
                bool,
                bool,
            ) -> Option<Arc<Symbol>>,
        >,
    >,
    pub get_resolved_symbol_fn: Option<Box<dyn Fn(&Arc<Node>) -> Option<Arc<Symbol>>>>,
    pub get_merged_symbol_fn: Option<Box<dyn Fn(&Arc<Symbol>) -> Option<Arc<Symbol>>>>,
    pub get_parent_of_symbol_fn: Option<Box<dyn Fn(&Arc<Symbol>) -> Option<Arc<Symbol>>>>,
    pub get_symbol_of_declaration_fn: Option<Box<dyn Fn(&Arc<Node>) -> Option<Arc<Symbol>>>>,
    pub get_type_only_alias_declaration_fn:
        Option<Box<dyn Fn(&Arc<Symbol>, SymbolFlags) -> Option<Arc<Node>>>>,
    pub get_export_symbol_of_value_symbol_if_exported_fn:
        Option<Box<dyn Fn(&Arc<Symbol>) -> Option<Arc<Symbol>>>>,
    pub get_element_access_expression_name_fn: Option<Box<dyn Fn(&Arc<Node>) -> Option<String>>>,
}

impl Default for ReferenceResolverHooks {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceResolverHooks {
    pub fn new() -> Self {
        Self {
            resolve_name_fn: None,
            get_resolved_symbol_fn: None,
            get_merged_symbol_fn: None,
            get_parent_of_symbol_fn: None,
            get_symbol_of_declaration_fn: None,
            get_type_only_alias_declaration_fn: None,
            get_export_symbol_of_value_symbol_if_exported_fn: None,
            get_element_access_expression_name_fn: None,
        }
    }
}
