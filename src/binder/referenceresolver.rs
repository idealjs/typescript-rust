#![allow(dead_code)]

use crate::ast::*;
use crate::core::compiler_options::CompilerOptions;
use crate::diagnostics::Message;
use std::sync::Arc;

use super::nameresolver::NameResolver;

pub trait ReferenceResolver {

    fn get_referenced_export_container(
        &self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>>;

    fn get_referenced_import_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;

    fn get_referenced_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;

    fn get_referenced_value_declarations(&self, node: &Arc<Node>) -> Vec<Arc<Node>>;

    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String;

    fn get_referenced_member_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
}

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

    fn get_parent_of_symbol(&self, symbol: Option<&Arc<Symbol>>) -> Option<Arc<Symbol>> {
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_parent_of_symbol_fn {
                return callback(symbol);
            }
            return symbol.parent.clone();
        }
        None
    }

    fn get_symbol_of_declaration(&self, declaration: Option<&Arc<Node>>) -> Option<Arc<Symbol>> {
        if let Some(declaration) = declaration {
            if let Some(callback) = &self.hooks.get_symbol_of_declaration_fn {
                return callback(declaration);
            }

            return node_symbol(declaration);
        }
        None
    }

    fn get_referenced_value_symbol(
        &mut self,
        reference: &Arc<Node>,
        start_in_declaration_container: bool,
    ) -> Option<Arc<Symbol>> {
        let resolved_symbol = self.get_resolved_symbol(Some(reference));
        if let Some(resolved) = resolved_symbol {
            return Some(resolved);
        }

        let location = if start_in_declaration_container {

            Arc::clone(reference)
        } else {
            Arc::clone(reference)
        };

        if let Some(callback) = &self.hooks.resolve_name_fn {
            return callback(
                &location,
                reference.text(),
                SymbolFlags::ExportValue
                    .union(SymbolFlags::VALUE)
                    .union(SymbolFlags::Alias),
                None,
                false,
                false,
            );
        }

        if self.resolver.is_none() {
            self.resolver = Some(NameResolver {
                compiler_options: self.options.clone(),
                ..NameResolver::default()
            });
        }

        let resolver = self.resolver.as_mut().unwrap();
        resolver.resolve(
            &location,
            reference.text(),
            SymbolFlags::ExportValue
                .union(SymbolFlags::VALUE)
                .union(SymbolFlags::Alias),
            None,
            false,
            false,
        )
    }

    fn is_type_only_alias_declaration(&self, symbol: Option<&Arc<Symbol>>) -> bool {
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_type_only_alias_declaration_fn {
                return callback(symbol, SymbolFlags::VALUE).is_some();
            }

            let mut node = self.get_declaration_of_alias_symbol(Some(symbol));
            while let Some(current) = node {
                match current.kind {
                    SyntaxKind::ImportEqualsDeclaration | SyntaxKind::ExportDeclaration => {

                        return node_is_type_only(&current);
                    }
                    SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::ExportSpecifier => {

                        if node_is_type_only(&current) {
                            return true;
                        }
                        node = current.parent.clone();
                        continue;
                    }
                    SyntaxKind::NamedImports | SyntaxKind::NamedExports => {
                        node = current.parent.clone();
                        continue;
                    }
                    _ => break,
                }
            }
        }
        false
    }

    fn get_declaration_of_alias_symbol(&self, symbol: Option<&Arc<Symbol>>) -> Option<Arc<Node>> {
        if let Some(symbol) = symbol {

            return symbol
                .declarations
                .iter()
                .rev()
                .find(|d| is_alias_symbol_declaration(d))
                .cloned();
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

        let start_in_declaration_container = node.parent.as_ref().map_or(false, |parent| {
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

fn node_is_type_only(_node: &Arc<Node>) -> bool {
    false
}

fn is_alias_symbol_declaration(_node: &Arc<Node>) -> bool {
    false
}
