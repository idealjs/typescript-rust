use crate::binder::referenceresolver_hooks::ReferenceResolverHooks;
use crate::binder::referenceresolver_reference_resolver::ReferenceResolver;
use std::sync::Arc;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::*;

use crate::binder::nameresolver::NameResolver;

#[allow(dead_code)]
pub struct ReferenceResolverImpl {
    resolver: Option<NameResolver>,
    options: Option<Arc<CompilerOptions>>,
    hooks: ReferenceResolverHooks,
}

pub fn new_reference_resolver(
    options: Option<Arc<CompilerOptions>>,
    hooks: ReferenceResolverHooks,
) -> ReferenceResolverImpl {
    ReferenceResolverImpl {
        resolver: None,
        options,
        hooks,
    }
}

impl ReferenceResolverImpl {
    fn get_resolved_symbol(&self, node: Option<&Arc<Node>>) -> Option<Arc<Symbol>> {
        if let Some(node) = node {
            if let Some(callback) = &self.hooks.get_resolved_symbol_fn {
                return callback(node);
            }
        }
        None
    }

    fn get_merged_symbol(&self, symbol: Option<&Arc<Symbol>>) -> Option<Arc<Symbol>> {
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_merged_symbol_fn {
                return callback(symbol);
            }
            return Some(Arc::clone(symbol));
        }
        None
    }






    fn get_export_symbol_of_value_symbol_if_exported(
        &self,
        symbol: Option<&Arc<Symbol>>,
    ) -> Option<Arc<Symbol>> {
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_export_symbol_of_value_symbol_if_exported_fn {
                return callback(symbol);
            }
            let mut symbol = Arc::clone(symbol);
            if symbol.flags.intersects(SymbolFlags::ExportValue) {
                if let Some(export) = &symbol.export_symbol {
                    symbol = Arc::clone(export);
                }
            }
            return self.get_merged_symbol(Some(&symbol));
        }
        None
    }
}

impl ReferenceResolver for ReferenceResolverImpl {
    fn get_referenced_export_container(
        &self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>> {
        let start_in_declaration_container = node.parent().as_ref().map_or(false, |parent| {
            (parent.kind == SyntaxKind::ModuleDeclaration
                || parent.kind == SyntaxKind::EnumDeclaration)
                && parent.name().map(|n| Arc::ptr_eq(n, node)).unwrap_or(false)
        });

        let _ = prefix_locals;
        let _ = start_in_declaration_container;
        None
    }

    fn get_referenced_import_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let _ = node;
        None
    }

    fn get_referenced_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let _ = node;
        None
    }

    fn get_referenced_value_declarations(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        let _ = node;
        Vec::new()
    }

    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String {
        if let Some(callback) = &self.hooks.get_element_access_expression_name_fn {
            if let Some(name) = callback(expression) {
                return name;
            }
        }
        String::new()
    }

    fn get_referenced_member_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut s = self.get_resolved_symbol(Some(node));
        if s.is_none() {
            if let Some(sym) = node_symbol(node) {
                s = self.get_merged_symbol(Some(&sym));
            }
        }
        let s = match s {
            Some(s) => s,
            None => return None,
        };
        self.get_export_symbol_of_value_symbol_if_exported(Some(&s))
            .as_ref()
            .and_then(|sym| sym.value_declaration.clone())
    }
}

fn node_symbol(_node: &Arc<Node>) -> Option<Arc<Symbol>> {
    None
}


