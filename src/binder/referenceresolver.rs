//! Reference resolver, ported from
//! `internal/binder/referenceresolver.go`.
//!
//! The reference resolver maps an identifier reference to the
//! declaration (or declarations) it refers to, resolving through
//! aliases, imports, and module exports. It is used by the emit
//! resolver and other downstream consumers that need to track where a
//! referenced value comes from.
//!
//! Mirrors `binder.ReferenceResolver` in Go.

#![allow(dead_code)]

use crate::ast::*;
use crate::core::compiler_options::CompilerOptions;
use crate::diagnostics::Message;
use std::sync::Arc;

use super::nameresolver::NameResolver;

/// The reference resolver interface.
///
/// Mirrors `binder.ReferenceResolver` in Go. Implemented by
/// [`ReferenceResolverImpl`].
pub trait ReferenceResolver {
    /// Get the container (source file, module declaration, or enum
    /// declaration) whose exports are referenced by `node`.
    fn get_referenced_export_container(
        &self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>>;

    /// Get the import declaration that `node` references, if it is an
    /// alias of an import.
    fn get_referenced_import_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;

    /// Get the single value declaration that `node` references.
    fn get_referenced_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;

    /// Get all value declarations that `node` references.
    fn get_referenced_value_declarations(&self, node: &Arc<Node>) -> Vec<Arc<Node>>;

    /// Get the name of an element-access expression if it is a literal
    /// string/numeric access.
    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String;

    /// Get the value declaration of the member referenced by `node`
    /// (`this.x` or `this[x]`).
    fn get_referenced_member_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
}

/// Hooks that allow the checker to override the reference resolver's
/// default behavior (delegating name resolution and symbol access to
/// the checker's own state).
///
/// Mirrors `binder.ReferenceResolverHooks` in Go.
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
    /// Create a new hooks struct with no overrides set.
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

/// Default implementation of [`ReferenceResolver`].
///
/// Mirrors the unexported `referenceResolver` struct in Go.
pub struct ReferenceResolverImpl {
    resolver: Option<NameResolver>,
    options: Option<Arc<CompilerOptions>>,
    hooks: ReferenceResolverHooks,
}

/// Create a new reference resolver.
///
/// Mirrors `binder.NewReferenceResolver` in Go.
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
    /// Get the resolved symbol for `node` via the hook (if any).
    ///
    /// Mirrors `referenceResolver.getResolvedSymbol` in Go.
    fn get_resolved_symbol(&self, node: Option<&Arc<Node>>) -> Option<Arc<Symbol>> {
        if let Some(node) = node {
            if let Some(callback) = &self.hooks.get_resolved_symbol_fn {
                return callback(node);
            }
        }
        None
    }

    /// Get the merged symbol for `symbol` via the hook (if any),
    /// falling back to the symbol itself.
    ///
    /// Mirrors `referenceResolver.getMergedSymbol` in Go.
    fn get_merged_symbol(&self, symbol: Option<&Arc<Symbol>>) -> Option<Arc<Symbol>> {
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_merged_symbol_fn {
                return callback(symbol);
            }
            return Some(Arc::clone(symbol));
        }
        None
    }

    /// Get the parent symbol of `symbol` via the hook (if any),
    /// falling back to the symbol's own `parent`.
    ///
    /// Mirrors `referenceResolver.getParentOfSymbol` in Go.
    fn get_parent_of_symbol(&self, symbol: Option<&Arc<Symbol>>) -> Option<Arc<Symbol>> {
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_parent_of_symbol_fn {
                return callback(symbol);
            }
            return symbol.parent.clone();
        }
        None
    }

    /// Get the symbol of a declaration via the hook (if any), falling
    /// back to the declaration's own symbol.
    ///
    /// Mirrors `referenceResolver.getSymbolOfDeclaration` in Go.
    fn get_symbol_of_declaration(&self, declaration: Option<&Arc<Node>>) -> Option<Arc<Symbol>> {
        if let Some(declaration) = declaration {
            if let Some(callback) = &self.hooks.get_symbol_of_declaration_fn {
                return callback(declaration);
            }
            // Default: declaration.Symbol().
            return node_symbol(declaration);
        }
        None
    }

    /// Resolve the value symbol referenced by `reference`.
    ///
    /// Mirrors `referenceResolver.getReferencedValueSymbol` in Go. First
    /// checks the resolved-symbol side table; otherwise performs a name
    /// lookup starting from the declaration container (when
    /// `start_in_declaration_container` is set).
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
            // TODO: `ast.IsDeclaration(reference.Parent) &&
            // reference.Parent.Name() == reference` requires the
            // declaration-container helper (`ast.GetDeclarationContainer`).
            // For now, fall back to the reference itself.
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

    /// Whether `symbol` is a type-only alias declaration.
    ///
    /// Mirrors `referenceResolver.isTypeOnlyAliasDeclaration` in Go.
    fn is_type_only_alias_declaration(&self, symbol: Option<&Arc<Symbol>>) -> bool {
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_type_only_alias_declaration_fn {
                return callback(symbol, SymbolFlags::VALUE).is_some();
            }

            let mut node = self.get_declaration_of_alias_symbol(Some(symbol));
            while let Some(current) = node {
                match current.kind {
                    SyntaxKind::ImportEqualsDeclaration | SyntaxKind::ExportDeclaration => {
                        // TODO: `node.IsTypeOnly()` requires a node accessor.
                        return node_is_type_only(&current);
                    }
                    SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::ExportSpecifier => {
                        // TODO: `node.IsTypeOnly()` requires a node accessor.
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

    /// Get the last alias-symbol declaration for `symbol`.
    ///
    /// Mirrors `referenceResolver.getDeclarationOfAliasSymbol` in Go.
    fn get_declaration_of_alias_symbol(&self, symbol: Option<&Arc<Symbol>>) -> Option<Arc<Node>> {
        if let Some(symbol) = symbol {
            // TODO: `core.FindLast(symbol.Declarations, ast.IsAliasSymbolDeclaration)`
            // requires `FindLast` and `IsAliasSymbolDeclaration`.
            return symbol
                .declarations
                .iter()
                .rev()
                .find(|d| is_alias_symbol_declaration(d))
                .cloned();
        }
        None
    }

    /// Get the export symbol of a value symbol if it is exported.
    ///
    /// Mirrors `referenceResolver.getExportSymbolOfValueSymbolIfExported` in Go.
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
        // When resolving the export for the name of a module or enum
        // declaration, we need to start resolution at the declaration's
        // container. Otherwise, we could incorrectly resolve the export
        // as the declaration if it contains an exported member with the
        // same name.
        // TODO: requires a mutable self for get_referenced_value_symbol;
        // the value-symbol resolution is reproduced structurally below.
        let start_in_declaration_container = node.parent.as_ref().map_or(false, |parent| {
            (parent.kind == SyntaxKind::ModuleDeclaration
                || parent.kind == SyntaxKind::EnumDeclaration)
                && parent.name().map(|n| Arc::ptr_eq(n, node)).unwrap_or(false)
        });
        // TODO: full container resolution requires `get_referenced_value_symbol`,
        // `getMergedSymbol`, `getParentOfSymbol`, and UMD-export /
        // ancestor-container checks.
        let _ = prefix_locals;
        let _ = start_in_declaration_container;
        None
    }

    fn get_referenced_import_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        // TODO: requires `get_referenced_value_symbol`,
        // `ast.IsNonLocalAlias`, and `getDeclarationOfAliasSymbol`.
        let _ = node;
        None
    }

    fn get_referenced_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        // TODO: requires `get_referenced_value_symbol` and
        // `getExportSymbolOfValueSymbolIfExported(...).ValueDeclaration`.
        let _ = node;
        None
    }

    fn get_referenced_value_declarations(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        // TODO: requires `get_referenced_value_symbol` and iteration over
        // the symbol's declarations filtered by value-declaration kinds.
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
        // Member references are `this.something` or `this[something]`,
        // so should always simply have a resolved symbol.
        let mut s = self.get_resolved_symbol(Some(node));
        if s.is_none() {
            // Might be a declaration instead of a ref; get the merged
            // declaration symbol.
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

// ────────────────────────────────────────────────────────────────────────────
// Stubs for AST helpers that are not yet ported
// ────────────────────────────────────────────────────────────────────────────

/// TODO: port `node.Symbol()` access — look up the symbol side-table
/// entry for `node`.
fn node_symbol(_node: &Arc<Node>) -> Option<Arc<Symbol>> {
    None
}

/// TODO: port `node.IsTypeOnly()` — whether an import/export node is in
/// a type-only context.
fn node_is_type_only(_node: &Arc<Node>) -> bool {
    false
}

/// TODO: port `ast.IsAliasSymbolDeclaration` — whether `node` is a
/// declaration that introduces an alias symbol.
fn is_alias_symbol_declaration(_node: &Arc<Node>) -> bool {
    false
}
