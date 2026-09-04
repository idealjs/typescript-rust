#![allow(dead_code)]
#![allow(unused_variables)]

//! Symbol accessibility checking.
//!
//! Ported 1:1 from `internal/checker/symbolaccessibility.go` (876 lines).
//! These functions check whether a symbol is accessible from a given
//! declaration context (public/private/protected visibility, module scope,
//! alias chains, etc.).
//!
//! Many internal methods depend on infrastructure not yet ported to Rust
//! (full scope walking, alias resolution, module re-export chains, etc.).
//! Such methods are stubbed with `// TODO` and return sensible defaults.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::{
    INTERNAL_SYMBOL_NAME_DEFAULT, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS, Node, NodeFlags, SourceFile,
    Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};

use super::checker::Checker;
use super::types::{
    AccessibleChainCacheKey, SymbolAccessibility, SymbolAccessibilityResult, TypeFlags,
};
use super::utilities::can_have_locals;

// ────────────────────────────────────────────────────────────────────────────
// symbolTableID — uniquely identifies a symbol table by encoding its source
// ────────────────────────────────────────────────────────────────────────────

/// Uniquely identifies a symbol table by encoding its source.
/// The high 3 bits encode the kind, and the remaining bits encode the
/// NodeId or SymbolId of the source.
///
/// Mirrors Go's `symbolTableID` (symbolaccessibility.go).
pub type SymbolTableId = u64;

const ST_KIND_SHIFT: u32 = 61;

const ST_KIND_LOCALS: SymbolTableId = 0 << ST_KIND_SHIFT;
const ST_KIND_EXPORTS: SymbolTableId = 1 << ST_KIND_SHIFT;
const ST_KIND_MEMBERS: SymbolTableId = 2 << ST_KIND_SHIFT;
const ST_KIND_GLOBALS: SymbolTableId = 3 << ST_KIND_SHIFT;
const ST_KIND_RESOLVED_EXPORTS: SymbolTableId = 4 << ST_KIND_SHIFT;

/// Mask extracting the kind bits from a SymbolTableId.
const ST_KIND_MASK: SymbolTableId = 0x7 << ST_KIND_SHIFT;

fn symbol_table_id_from_locals(node: &Node) -> SymbolTableId {
    ST_KIND_LOCALS | node.id()
}

fn symbol_table_id_from_exports(sym: &Symbol) -> SymbolTableId {
    ST_KIND_EXPORTS | sym.id()
}

/// Returns an ID for resolved/derived export tables (e.g. from
/// `getExportsOfSymbol`/`getExportsOfModule` which may include `export *`
/// resolution and late-bound members). This is distinct from
/// `symbol_table_id_from_exports` to prevent cache collisions with raw
/// `sym.exports` tables passed by `some_symbol_table_in_scope`.
fn symbol_table_id_from_resolved_exports(sym: &Symbol) -> SymbolTableId {
    ST_KIND_RESOLVED_EXPORTS | sym.id()
}

fn symbol_table_id_from_members(sym: &Symbol) -> SymbolTableId {
    ST_KIND_MEMBERS | sym.id()
}

fn symbol_table_id_from_globals() -> SymbolTableId {
    ST_KIND_GLOBALS
}

// ────────────────────────────────────────────────────────────────────────────
// accessibleSymbolChainContext
// ────────────────────────────────────────────────────────────────────────────

/// Context for accessible-symbol-chain queries.
///
/// Mirrors Go's `accessibleSymbolChainContext` (symbolaccessibility.go).
pub struct AccessibleSymbolChainContext {
    pub symbol: Arc<Symbol>,
    pub enclosing_declaration: Option<Arc<Node>>,
    pub meaning: SymbolFlags,
    pub use_only_external_aliasing: bool,
    /// Set of (symbol_id → set of visited table IDs) to prevent infinite
    /// recursion through export cycles. Uses `RefCell` for interior
    /// mutability since the context is passed by shared reference.
    pub visited_symbol_tables_map: RefCell<HashMap<u64, HashMap<SymbolTableId, ()>>>,
}

impl Clone for AccessibleSymbolChainContext {
    fn clone(&self) -> Self {
        Self {
            symbol: Arc::clone(&self.symbol),
            enclosing_declaration: self.enclosing_declaration.clone(),
            meaning: self.meaning,
            use_only_external_aliasing: self.use_only_external_aliasing,
            visited_symbol_tables_map: RefCell::new(
                self.visited_symbol_tables_map.borrow().clone(),
            ),
        }
    }
}

/// A collected symbol table entry during scope walking.
struct SymbolTableInScope {
    table: SymbolTable,
    table_id: SymbolTableId,
    ignore_qualification: bool,
    is_local_name_lookup: bool,
    scope_node: Option<Arc<Node>>,
}

// ────────────────────────────────────────────────────────────────────────────
// Free functions
// ────────────────────────────────────────────────────────────────────────────

/// Whether a declaration is a module with a string-literal name or an
/// external/CommonJS source file (i.e. a non-global augmentation module).
///
/// Mirrors Go's `hasNonGlobalAugmentationExternalModuleSymbol`.
fn has_non_global_augmentation_external_module_symbol(declaration: &Arc<Node>) -> bool {
    // TODO: Port ast.IsModuleWithStringLiteralName and ast.IsExternalOrCommonJSModule
    declaration.kind == SyntaxKind::ModuleDeclaration
}

/// Whether a declaration is an ambient module or an external/CommonJS
/// source file.
///
/// Mirrors Go's `hasExternalModuleSymbol`.
fn has_external_module_symbol(declaration: &Arc<Node>) -> bool {
    // TODO: Port ast.IsAmbientModule
    declaration.kind == SyntaxKind::ModuleDeclaration
        || (declaration.kind == SyntaxKind::SourceFile)
    // TODO: && ast.IsExternalOrCommonJSModule(declaration)
}

/// If we are looking in value space, the parent meaning is value, otherwise
/// it is namespace.
///
/// Mirrors Go's `getQualifiedLeftMeaning`.
fn get_qualified_left_meaning(right_meaning: SymbolFlags) -> SymbolFlags {
    if right_meaning == SymbolFlags::VALUE {
        SymbolFlags::VALUE
    } else {
        SymbolFlags::NAMESPACE
    }
}

/// Whether a symbol's declarations are all property/method declarations.
///
/// Mirrors Go's `isPropertyOrMethodDeclarationSymbol`.
fn is_property_or_method_declaration_symbol(symbol: &Symbol) -> bool {
    if !symbol.declarations.is_empty() {
        for declaration in &symbol.declarations {
            match declaration.kind {
                SyntaxKind::PropertyDeclaration
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => continue,
                _ => return false,
            }
        }
        true
    } else {
        false
    }
}

/// Whether a symbol is a UMD export symbol (namespace export declaration).
///
/// Mirrors Go's `isUMDExportSymbol`.
fn is_umd_export_symbol(symbol: &Symbol) -> bool {
    !symbol.declarations.is_empty()
        && symbol
            .declarations
            .first()
            .map(|d| d.kind == SyntaxKind::NamespaceExportDeclaration)
            .unwrap_or(false)
}

/// Whether a node is a namespace re-export (export ... in ...).
///
/// Mirrors Go's `isNamespaceReexportDeclaration`.
fn is_namespace_reexport_declaration(node: &Arc<Node>) -> bool {
    // TODO: Port ast.IsNamespaceExport and node.ModuleSpecifier()
    node.kind == SyntaxKind::NamespaceExport
}

// ────────────────────────────────────────────────────────────────────────────
// Checker methods
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    // ── Public entry points ──────────────────────────────────────────────

    /// Whether a type symbol is accessible from `enclosing_declaration`.
    ///
    /// Mirrors Go's `IsTypeSymbolAccessible`.
    pub fn is_type_symbol_accessible(
        &mut self,
        type_symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool {
        let access = self.is_symbol_accessible_worker(
            type_symbol,
            enclosing_declaration,
            SymbolFlags::TYPE, // shouldComputeAliasesToMakeVisible
            false,             // allowModules
            true,              // shouldComputeAliasesToMakeVisible
        );
        access.accessibility == SymbolAccessibility::Accessible
    }

    /// Whether a value symbol is accessible from `enclosing_declaration`.
    ///
    /// Mirrors Go's `IsValueSymbolAccessible`.
    pub fn is_value_symbol_accessible(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool {
        let access = self.is_symbol_accessible_worker(
            symbol,
            enclosing_declaration,
            SymbolFlags::VALUE,
            false, // allowModules
            true,  // shouldComputeAliasesToMakeVisible
        );
        access.accessibility == SymbolAccessibility::Accessible
    }

    /// Whether a symbol is accessible by flags from `enclosing_declaration`.
    ///
    /// Mirrors Go's `IsSymbolAccessibleByFlags`.
    pub fn is_symbol_accessible_by_flags(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: SymbolFlags,
    ) -> bool {
        let access = self.is_symbol_accessible_worker(
            symbol,
            enclosing_declaration,
            flags,
            false, // allowModules
            false, // shouldComputeAliasesToMakeVisible
                   // TODO: Strada bug? Why is this allowModules: false?
        );
        access.accessibility == SymbolAccessibility::Accessible
    }

    /// Check if any of the given symbols is accessible.
    ///
    /// Mirrors Go's `IsAnySymbolAccessible`.
    pub fn is_any_symbol_accessible(
        &mut self,
        symbols: &[Arc<Symbol>],
        enclosing_declaration: Option<&Arc<Node>>,
        initial_symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        should_compute_aliases_to_make_visible: bool,
        allow_modules: bool,
    ) -> Option<SymbolAccessibilityResult> {
        if symbols.is_empty() {
            return None;
        }

        let mut had_accessible_chain: Option<Arc<Symbol>> = None;
        let mut early_module_bail = false;
        for symbol in symbols {
            // Symbol is accessible if it by itself is accessible
            let accessible_symbol_chain = self.get_accessible_symbol_chain(
                symbol,
                enclosing_declaration,
                meaning, // useOnlyExternalAliasing
                false,
            );
            if !accessible_symbol_chain.is_empty() {
                had_accessible_chain = Some(Arc::clone(symbol));
                // TODO: going through emit resolver here is weird. Relayer these APIs.
                let has_accessible_declarations = self.has_visible_declarations_with_aliases(
                    &accessible_symbol_chain[0],
                    should_compute_aliases_to_make_visible,
                );
                if let Some(result) = has_accessible_declarations {
                    return Some(result);
                }
            }
            if allow_modules {
                if symbol
                    .declarations
                    .iter()
                    .any(has_non_global_augmentation_external_module_symbol)
                {
                    if should_compute_aliases_to_make_visible {
                        early_module_bail = true;
                        // Generally speaking, we want to use the aliases that already exist to refer to a module, if present
                        // In order to do so, we need to find those aliases in order to retain them in declaration emit; so
                        // if we are in declaration emit, we cannot use the fast path for module visibility until we've exhausted
                        // all other visibility options (in order to capture the possible aliases used to reference the module)
                        continue;
                    }
                    // Any meaning of a module symbol is always accessible via an `import` type
                    return Some(SymbolAccessibilityResult {
                        accessibility: SymbolAccessibility::Accessible,
                        ..Default::default()
                    });
                }
            }

            // If we haven't got the accessible symbol, it doesn't mean the symbol is actually inaccessible.
            // It could be a qualified symbol and hence verify the path
            // e.g.:
            // module m {
            //     export class c {
            //     }
            // }
            // const x: typeof m.c
            // In the above example when we start with checking if typeof m.c symbol is accessible,
            // we are going to see if c can be accessed in scope directly.
            // But it can't, hence the accessible is going to be undefined, but that doesn't mean m.c is inaccessible
            // It is accessible if the parent m is accessible because then m.c can be accessed through qualification

            let containers = self.get_containers_of_symbol(symbol, enclosing_declaration, meaning);
            let mut next_meaning = meaning;
            if initial_symbol.id() == symbol.id() {
                next_meaning = get_qualified_left_meaning(meaning);
            }
            let parent_result = self.is_any_symbol_accessible(
                &containers,
                enclosing_declaration,
                initial_symbol,
                next_meaning,
                should_compute_aliases_to_make_visible,
                allow_modules,
            );
            if let Some(result) = parent_result {
                return Some(result);
            }
        }

        if early_module_bail {
            return Some(SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::Accessible,
                ..Default::default()
            });
        }

        if let Some(ref had_chain) = had_accessible_chain {
            let mut module_name = String::new();
            if had_chain.id() != initial_symbol.id() {
                module_name = self.symbol_to_string_ex_enclosing(
                    had_chain,
                    enclosing_declaration,
                    SymbolFlags::NAMESPACE,
                    super::types::SymbolFormatFlags::AllowAnyNodeKind,
                );
            }
            return Some(SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::NotAccessible,
                error_symbol_name: self.symbol_to_string_ex_enclosing(
                    initial_symbol,
                    enclosing_declaration,
                    meaning,
                    super::types::SymbolFormatFlags::AllowAnyNodeKind,
                ),
                error_module_name: module_name,
                ..Default::default()
            });
        }
        None
    }

    /// Check if the given symbol in given enclosing declaration is accessible
    /// and mark all associated alias to be visible if requested.
    ///
    /// Mirrors Go's `IsSymbolAccessible`.
    pub fn is_symbol_accessible(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        should_compute_aliases_to_make_visible: bool,
    ) -> SymbolAccessibilityResult {
        self.is_symbol_accessible_worker(
            symbol,
            enclosing_declaration,
            meaning,
            should_compute_aliases_to_make_visible,
            true, // allowModules
        )
    }

    /// Core symbol-accessibility worker.
    ///
    /// Mirrors Go's `isSymbolAccessibleWorker`.
    fn is_symbol_accessible_worker(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        should_compute_aliases_to_make_visible: bool,
        allow_modules: bool,
    ) -> SymbolAccessibilityResult {
        if let (Some(_), Some(_)) = (Some(symbol), enclosing_declaration) {
            let symbols = vec![Arc::clone(symbol)];
            if let Some(result) = self.is_any_symbol_accessible(
                &symbols,
                enclosing_declaration,
                symbol,
                meaning,
                should_compute_aliases_to_make_visible,
                allow_modules,
            ) {
                return result;
            }

            // This could be a symbol that is not exported in the external module
            // or it could be a symbol from different external module that is not aliased and hence cannot be named
            let symbol_external_module = self.get_external_module_container_of_symbol(symbol);
            if let Some(ref symbol_external_module) = symbol_external_module {
                let enclosing_external_module =
                    enclosing_declaration.and_then(|enc| self.get_external_module_container(enc));
                if symbol_external_module.id()
                    != enclosing_external_module
                        .as_ref()
                        .map(|s| s.id())
                        .unwrap_or(0)
                {
                    // name from different external module that is not visible
                    let error_node = if enclosing_declaration
                        .map(|n| n.flags.contains(NodeFlags::JavaScriptFile))
                        .unwrap_or(false)
                    {
                        enclosing_declaration.cloned()
                    } else {
                        None
                    };
                    return SymbolAccessibilityResult {
                        accessibility: SymbolAccessibility::CannotBeNamed,
                        error_symbol_name: self.symbol_to_string_ex_enclosing(
                            symbol,
                            enclosing_declaration,
                            meaning,
                            super::types::SymbolFormatFlags::AllowAnyNodeKind,
                        ),
                        error_module_name: self.symbol_to_string(symbol_external_module),
                        error_node,
                        ..Default::default()
                    };
                }
            }

            // Just a local name that is not accessible
            return SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::NotAccessible,
                error_symbol_name: self.symbol_to_string_ex_enclosing(
                    symbol,
                    enclosing_declaration,
                    meaning,
                    super::types::SymbolFormatFlags::AllowAnyNodeKind,
                ),
                ..Default::default()
            };
        }

        SymbolAccessibilityResult {
            accessibility: SymbolAccessibility::Accessible,
            ..Default::default()
        }
    }

    // ── getWithAlternativeContainers ─────────────────────────────────────

    /// Mirrors Go's `getWithAlternativeContainers`.
    fn get_with_alternative_containers(
        &mut self,
        container: &Arc<Symbol>,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> Vec<Arc<Symbol>> {
        let additional_containers: Vec<Arc<Symbol>> = container
            .declarations
            .iter()
            .filter_map(|d| {
                self.get_file_symbol_if_file_symbol_export_equals_container(d, container)
            })
            .collect();

        let reexport_containers = if enclosing_declaration.is_some() {
            self.get_alternative_containing_modules(symbol, enclosing_declaration)
        } else {
            Vec::new()
        };

        let object_literal_container =
            self.get_variable_declaration_of_object_literal(container, meaning);
        let left_meaning = get_qualified_left_meaning(meaning);

        if enclosing_declaration.is_some()
            && container.flags.intersects(left_meaning)
            && !self
                .get_accessible_symbol_chain(
                    container,
                    enclosing_declaration,
                    SymbolFlags::NAMESPACE, // useOnlyExternalAliasing
                    false,
                )
                .is_empty()
        {
            // This order expresses a preference for the real container if it is in scope
            let mut res = vec![Arc::clone(container)];
            res.extend(additional_containers.iter().cloned());
            res.extend(reexport_containers.iter().cloned());
            if let Some(olc) = object_literal_container {
                res.push(olc);
            }
            return res;
        }

        // we potentially have a symbol which is a member of the instance side of something
        // - look for a variable in scope with the container's type
        // which may be acting like a namespace (eg, `Symbol` acts like a namespace
        // when looking up `Symbol.toStringTag`)
        let mut variable_matches: Vec<Arc<Symbol>> = Vec::new();
        if meaning == SymbolFlags::VALUE
            && !container.flags.intersects(left_meaning)
            && container.flags.intersects(SymbolFlags::TYPE)
            && self
                .get_declared_type_of_symbol(container)
                .flags
                .intersects(TypeFlags::Object)
        {
            let tables = self.collect_symbol_tables_in_scope(enclosing_declaration);
            for info in &tables {
                let mut found = false;
                for s in info.table.entries.values() {
                    if s.flags.intersects(left_meaning)
                        && Arc::ptr_eq(
                            &self.get_type_of_symbol(s),
                            &self.get_declared_type_of_symbol(container),
                        )
                    {
                        variable_matches.push(Arc::clone(s));
                        found = true;
                    }
                }
                if found {
                    break;
                }
            }
            self.sort_symbols(&mut variable_matches);
        }

        let mut res: Vec<Arc<Symbol>> = Vec::new();
        res.extend(variable_matches);
        res.extend(additional_containers);
        res.push(Arc::clone(container));
        if let Some(olc) = object_literal_container {
            res.push(olc);
        }
        res.extend(reexport_containers);
        res
    }

    /// Mirrors Go's `getAlternativeContainingModules`.
    fn get_alternative_containing_modules(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> Vec<Arc<Symbol>> {
        let enclosing_declaration = match enclosing_declaration {
            Some(enc) => enc,
            None => return Vec::new(),
        };

        let containing_file = self.get_source_file_of_node(enclosing_declaration);
        let id = containing_file.as_ref().map(|f| f.id()).unwrap_or(0);

        // Check cache
        if let Some(links) = self.symbol_container_links.get(symbol) {
            if let Some(existing) = links.extended_containers_by_file.get(&id) {
                return existing.clone();
            }
        }

        // TODO: Full implementation requires resolveExternalModuleName and
        // getAliasForSymbolInContainer which depend on module resolution
        // infrastructure not yet ported.
        let mut results: Vec<Arc<Symbol>> = Vec::new();

        // Check if we already have extended_containers computed
        if let Some(links) = self.symbol_container_links.get(symbol) {
            if let Some(ref extended) = links.extended_containers {
                return extended.clone();
            }
        }

        // No results from files already being imported by this file - expand search
        // (expensive, but not location-specific, so cached)
        let other_files: Vec<Arc<SourceFile>> = self.files.clone();
        for file in &other_files {
            // TODO: ast.IsExternalModule check
            let sym = self.get_symbol_of_declaration(&file.node);
            if let Some(ref sym) = sym {
                let ref_sym = self.get_alias_for_symbol_in_container(sym, symbol);
                if ref_sym.is_some() {
                    results.push(Arc::clone(sym));
                }
            }
        }

        self.symbol_container_links
            .get_or_default(symbol)
            .extended_containers = Some(results.clone());
        self.symbol_container_links
            .get_or_default(symbol)
            .extended_containers_by_file
            .insert(id, results.clone());
        results
    }

    /// If we're trying to reference some object literal in, eg
    /// `var a = { x: 1 }`, the symbol for the literal, `__object`, is distinct
    /// from the symbol of the declaration it is being assigned to.
    ///
    /// Mirrors Go's `getVariableDeclarationOfObjectLiteral`.
    fn get_variable_declaration_of_object_literal(
        &self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        if !meaning.intersects(SymbolFlags::VALUE) {
            return None;
        }
        if symbol.declarations.is_empty() {
            return None;
        }
        let first_decl = &symbol.declarations[0];
        let parent = first_decl.parent.as_ref()?;
        // TODO: Port ast.IsVariableDeclaration, ast.IsObjectLiteralExpression,
        // ast.IsTypeLiteralNode, firstDecl.Parent.Initializer(), firstDecl.Parent.Type()
        // For now, return None as the full node-accessor infrastructure isn't ported.
        None
    }

    /// Mirrors Go's `getExternalModuleContainer`.
    fn get_external_module_container(&self, declaration: &Arc<Node>) -> Option<Arc<Symbol>> {
        // TODO: Port ast.FindAncestor with hasExternalModuleSymbol predicate
        // For now, check if the declaration itself qualifies
        if has_external_module_symbol(declaration) {
            return self.get_symbol_of_declaration(declaration);
        }
        // Walk up parents
        let mut node = declaration.parent.as_ref();
        while let Some(n) = node {
            if has_external_module_symbol(n) {
                return self.get_symbol_of_declaration(n);
            }
            node = n.parent.as_ref();
        }
        None
    }

    /// Helper: get the external module container of a symbol (first declaration).
    fn get_external_module_container_of_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        for d in &symbol.declarations {
            if let Some(sym) = self.get_external_module_container(d) {
                return Some(sym);
            }
        }
        None
    }

    /// Mirrors Go's `getFileSymbolIfFileSymbolExportEqualsContainer`.
    fn get_file_symbol_if_file_symbol_export_equals_container(
        &self,
        d: &Arc<Node>,
        container: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let file_symbol = self.get_external_module_container(d)?;
        let exported = file_symbol
            .exports
            .get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)?;
        if self
            .get_symbol_if_same_reference(exported, container)
            .is_some()
        {
            Some(file_symbol)
        } else {
            None
        }
    }

    // ── getContainersOfSymbol ────────────────────────────────────────────

    /// Attempts to find the symbol corresponding to the container a symbol
    /// is in — usually this is just its `.parent`, but for locals, this
    /// value is `undefined`.
    ///
    /// Mirrors Go's `getContainersOfSymbol`.
    fn get_containers_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> Vec<Arc<Symbol>> {
        let container = self.get_parent_of_symbol(symbol);

        // Type parameters end up in the `members` lists but are not externally visible
        if let Some(ref container) = container {
            if !symbol.flags.intersects(SymbolFlags::TypeParameter) {
                return self.get_with_alternative_containers(
                    container,
                    symbol,
                    enclosing_declaration,
                    meaning,
                );
            }
        }

        let mut candidates: Vec<Arc<Symbol>> = Vec::new();
        for d in &symbol.declarations {
            // TODO: Port ast.IsAmbientModule
            if let Some(ref parent) = d.parent {
                // direct children of a module
                if has_non_global_augmentation_external_module_symbol(parent) {
                    if let Some(sym) = self.get_symbol_of_declaration(parent) {
                        if !candidates.iter().any(|c| c.id() == sym.id()) {
                            candidates.push(sym);
                        }
                    }
                    continue;
                }
                // export ='d member of an ambient module
                // TODO: Port ast.IsModuleBlock and related logic
            }
            // TODO: Port class expression / binary expression / module.exports logic
        }

        if candidates.is_empty() {
            return Vec::new();
        }

        let mut best_containers: Vec<Arc<Symbol>> = Vec::new();
        let mut alternative_containers: Vec<Arc<Symbol>> = Vec::new();
        for container in &candidates {
            if self
                .get_alias_for_symbol_in_container(container, symbol)
                .is_none()
            {
                continue;
            }
            let all_alts = self.get_with_alternative_containers(
                container,
                symbol,
                enclosing_declaration,
                meaning,
            );
            if all_alts.is_empty() {
                continue;
            }
            best_containers.push(Arc::clone(&all_alts[0]));
            if all_alts.len() > 1 {
                alternative_containers.extend(all_alts[1..].iter().cloned());
            }
        }
        best_containers.extend(alternative_containers);
        best_containers
    }

    /// Mirrors Go's `getAliasForSymbolInContainer`.
    fn get_alias_for_symbol_in_container(
        &mut self,
        container: &Arc<Symbol>,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        if let Some(parent) = self.get_parent_of_symbol(symbol) {
            if parent.id() == container.id() {
                // fast path, `symbol` is either already the alias or isn't aliased
                return Some(Arc::clone(symbol));
            }
        }

        // Check if container is a thing with an `export=` which points directly at `symbol`
        if let Some(export_equals) = container.exports.get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS) {
            if self
                .get_symbol_if_same_reference(export_equals, symbol)
                .is_some()
            {
                return Some(Arc::clone(container));
            }
        }

        let exports = self.get_exports_of_symbol(container);
        if let Some(quick) = exports.get(&symbol.name) {
            if self.get_symbol_if_same_reference(quick, symbol).is_some() {
                return Some(Arc::clone(quick));
            }
        }

        let mut candidates: Vec<Arc<Symbol>> = Vec::new();
        for exported in exports.entries.values() {
            if self
                .get_symbol_if_same_reference(exported, symbol)
                .is_some()
            {
                candidates.push(Arc::clone(exported));
            }
        }
        if !candidates.is_empty() {
            self.sort_symbols(&mut candidates); // _must_ sort exports for stable results
            return candidates.into_iter().next();
        }
        None
    }

    // ── getAccessibleSymbolChain ─────────────────────────────────────────

    /// Mirrors Go's `getAccessibleSymbolChain`.
    fn get_accessible_symbol_chain(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        use_only_external_aliasing: bool,
    ) -> Vec<Arc<Symbol>> {
        let ctx = AccessibleSymbolChainContext {
            symbol: Arc::clone(symbol),
            enclosing_declaration: enclosing_declaration.cloned(),
            meaning,
            use_only_external_aliasing,
            visited_symbol_tables_map: RefCell::new(HashMap::new()),
        };
        self.get_accessible_symbol_chain_ex(ctx)
    }

    /// Public version of `getAccessibleSymbol_chain`.
    ///
    /// Mirrors Go's `GetAccessibleSymbolChain`.
    pub fn get_accessible_symbol_chain_public(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        use_only_external_aliasing: bool,
    ) -> Vec<Arc<Symbol>> {
        self.get_accessible_symbol_chain(
            symbol,
            enclosing_declaration,
            meaning,
            use_only_external_aliasing,
        )
    }

    /// Mirrors Go's `getAccessibleSymbolChainEx`.
    fn get_accessible_symbol_chain_ex(
        &mut self,
        ctx: AccessibleSymbolChainContext,
    ) -> Vec<Arc<Symbol>> {
        if is_property_or_method_declaration_symbol(&ctx.symbol) {
            return Vec::new();
        }

        // Go from enclosingDeclaration to the first scope we check, so the
        // cache is keyed off the scope and thus shared more
        let tables = self.collect_symbol_tables_in_scope(ctx.enclosing_declaration.as_ref());
        let first_relevant_location = tables.first().and_then(|t| t.scope_node.clone());

        let link_key = AccessibleChainCacheKey {
            use_only_external_aliasing: ctx.use_only_external_aliasing,
            location: first_relevant_location,
            meaning: ctx.meaning,
        };

        // Check cache
        if let Some(links) = self.symbol_container_links.get(&ctx.symbol) {
            if let Some(existing) = links.accessible_chain_cache.get(&link_key) {
                return existing.clone();
            }
        }

        let mut result: Vec<Arc<Symbol>> = Vec::new();

        for info in &tables {
            let res = self.get_accessible_symbol_chain_from_symbol_table(
                &ctx,
                &info.table,
                info.table_id,
                info.ignore_qualification,
                info.is_local_name_lookup,
            );
            if !res.is_empty() {
                result = res;
                break;
            }
        }

        self.symbol_container_links
            .get_or_default(&ctx.symbol)
            .accessible_chain_cache
            .insert(link_key, result.clone());
        result
    }

    /// Mirrors Go's `getAccessibleSymbolChainFromSymbolTable`.
    fn get_accessible_symbol_chain_from_symbol_table(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        t: &SymbolTable,
        table_id: SymbolTableId,
        ignore_qualification: bool,
        is_local_name_lookup: bool,
    ) -> Vec<Arc<Symbol>> {
        let sym_id = ctx.symbol.id();
        {
            let mut visited_map = ctx.visited_symbol_tables_map.borrow_mut();
            let visited_symbol_tables = visited_map.entry(sym_id).or_default();

            if visited_symbol_tables.contains_key(&table_id) {
                return Vec::new();
            }
            visited_symbol_tables.insert(table_id, ());
        }

        let res =
            self.try_symbol_table(ctx, t, table_id, ignore_qualification, is_local_name_lookup);

        {
            let mut visited_map = ctx.visited_symbol_tables_map.borrow_mut();
            if let Some(visited_symbol_tables) = visited_map.get_mut(&sym_id) {
                visited_symbol_tables.remove(&table_id);
            }
        }
        res
    }

    /// Returns only the alias symbols from a symbol table, caching the
    /// result by tableId to avoid repeated iteration over large tables.
    ///
    /// Mirrors Go's `getSymbolTableAliases`.
    fn get_symbol_table_aliases(
        &mut self,
        symbols: &SymbolTable,
        table_id: SymbolTableId,
    ) -> Vec<Arc<Symbol>> {
        let kind = table_id & ST_KIND_MASK;
        // Members tables never contain alias symbols; skip entirely.
        if kind == ST_KIND_MEMBERS {
            return Vec::new();
        }
        // Cache globals and exports tables (which are large and revisited often).
        if kind == ST_KIND_GLOBALS || kind == ST_KIND_EXPORTS || kind == ST_KIND_RESOLVED_EXPORTS {
            if let Some(aliases) = self.symbol_table_alias_cache.get(&table_id) {
                return aliases.clone();
            }
        }
        let mut aliases: Vec<Arc<Symbol>> = Vec::new();
        for sym in symbols.entries.values() {
            if sym.flags.intersects(SymbolFlags::Alias) {
                aliases.push(Arc::clone(sym));
            }
        }
        if kind == ST_KIND_GLOBALS || kind == ST_KIND_EXPORTS || kind == ST_KIND_RESOLVED_EXPORTS {
            self.symbol_table_alias_cache
                .insert(table_id, aliases.clone());
        }
        aliases
    }

    /// Mirrors Go's `trySymbolTable`.
    fn try_symbol_table(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbols: &SymbolTable,
        table_id: SymbolTableId,
        ignore_qualification: bool,
        is_local_name_lookup: bool,
    ) -> Vec<Arc<Symbol>> {
        let is_globals = table_id == ST_KIND_GLOBALS;
        // If symbol is directly available by its name in the symbol table
        if let Some(res) = symbols.get(&ctx.symbol.name) {
            if self.is_accessible(ctx, res, None, ignore_qualification) {
                return vec![Arc::clone(&ctx.symbol)];
            }

            // Check for ExportSymbol by direct name lookup
            if let Some(ref export_sym) = res.export_symbol {
                let merged = self.get_merged_symbol(export_sym);
                if self.is_accessible(ctx, &merged, None, ignore_qualification) {
                    return vec![Arc::clone(&ctx.symbol)];
                }
            }
        }

        let mut candidate_chains: Vec<Vec<Arc<Symbol>>> = Vec::new();

        // Iterate only alias symbols from the table (cached per tableId).
        let aliases = self.get_symbol_table_aliases(symbols, table_id);
        for symbol_from_symbol_table in &aliases {
            let enclosing_is_external_module = ctx
                .enclosing_declaration
                .as_ref()
                .map(|n| {
                    // TODO: ast.IsExternalModule(ast.GetSourceFileOfNode(enclosingDeclaration))
                    false
                })
                .unwrap_or(false);

            if symbol_from_symbol_table.name != INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
                && symbol_from_symbol_table.name != INTERNAL_SYMBOL_NAME_DEFAULT
                && !(is_umd_export_symbol(symbol_from_symbol_table)
                    && ctx.enclosing_declaration.is_some()
                    && enclosing_is_external_module)
                && (!ctx.use_only_external_aliasing
                    || symbol_from_symbol_table
                        .declarations
                        .iter()
                        .any(|d| d.kind == SyntaxKind::ExternalModuleReference))
                && (!is_local_name_lookup
                    || !symbol_from_symbol_table
                        .declarations
                        .iter()
                        .any(is_namespace_reexport_declaration))
                && (ignore_qualification
                    || !symbol_from_symbol_table
                        .declarations
                        .iter()
                        .any(|d| d.kind == SyntaxKind::ExportSpecifier))
            {
                let resolved_imported_symbol = self.resolve_alias(symbol_from_symbol_table);
                let candidate = self.get_candidate_list_for_symbol(
                    ctx,
                    symbol_from_symbol_table,
                    &resolved_imported_symbol,
                    ignore_qualification,
                );
                if !candidate.is_empty() {
                    candidate_chains.push(candidate);
                }
            }
        }

        if !candidate_chains.is_empty() {
            // pick first, shortest
            candidate_chains.sort_by(|a, b| self.compare_symbol_chains(a, b));
            return candidate_chains.into_iter().next().unwrap_or_default();
        }

        // If there's no result and we're looking at the global symbol table,
        // treat `globalThis` like an alias and try to lookup thru that
        if is_globals {
            if let Some(global_this) = self.global_this_symbol.clone() {
                return self.get_candidate_list_for_symbol(
                    ctx,
                    &global_this,
                    &global_this,
                    ignore_qualification,
                );
            }
        }
        Vec::new()
    }

    /// Mirrors Go's `compareSymbolChainsWorker`.
    fn compare_symbol_chains(&self, a: &[Arc<Symbol>], b: &[Arc<Symbol>]) -> std::cmp::Ordering {
        let chain_len = a.len().cmp(&b.len());
        if chain_len != std::cmp::Ordering::Equal {
            return chain_len;
        }

        for idx in 0..a.len() {
            let cmp = self.compare_symbols(&a[idx], &b[idx]);
            let comparison = match cmp {
                x if x < 0 => std::cmp::Ordering::Less,
                0 => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Greater,
            };
            if comparison != std::cmp::Ordering::Equal {
                return comparison;
            }
        }
        std::cmp::Ordering::Equal
    }

    /// Mirrors Go's `getCandidateListForSymbol`.
    fn get_candidate_list_for_symbol(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        resolved_imported_symbol: &Arc<Symbol>,
        ignore_qualification: bool,
    ) -> Vec<Arc<Symbol>> {
        if self.is_accessible(
            ctx,
            symbol_from_symbol_table,
            Some(resolved_imported_symbol),
            ignore_qualification,
        ) {
            return vec![Arc::clone(symbol_from_symbol_table)];
        }

        // Look in the exported members
        let candidate_table = self.get_exports_of_symbol(resolved_imported_symbol);
        let candidate_table_id = symbol_table_id_from_resolved_exports(resolved_imported_symbol);
        let accessible_symbols_from_exports = self.get_accessible_symbol_chain_from_symbol_table(
            ctx,
            &candidate_table,
            candidate_table_id, // ignoreQualification
            true,
            false,
        );
        if accessible_symbols_from_exports.is_empty() {
            return Vec::new();
        }
        if !self.can_qualify_symbol(
            ctx,
            symbol_from_symbol_table,
            get_qualified_left_meaning(ctx.meaning),
        ) {
            return Vec::new();
        }
        let mut result = vec![Arc::clone(symbol_from_symbol_table)];
        result.extend(accessible_symbols_from_exports);
        result
    }

    /// Mirrors Go's `isAccessible`.
    fn is_accessible(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        resolved_alias_symbol: Option<&Arc<Symbol>>,
        ignore_qualification: bool,
    ) -> bool {
        let mut like_symbols = false;
        if let Some(ref resolved) = resolved_alias_symbol {
            if ctx.symbol.id() == resolved.id() {
                like_symbols = true;
            }
        }
        if ctx.symbol.id() == symbol_from_symbol_table.id() {
            like_symbols = true;
        }
        let symbol = self.get_merged_symbol(&ctx.symbol);
        if let Some(ref resolved) = resolved_alias_symbol {
            let merged_resolved = self.get_merged_symbol(resolved);
            if symbol.id() == merged_resolved.id() {
                like_symbols = true;
            }
        }
        let merged_from_table = self.get_merged_symbol(symbol_from_symbol_table);
        if symbol.id() == merged_from_table.id() {
            like_symbols = true;
        }
        if !like_symbols {
            return false;
        }
        // if the symbolFromSymbolTable is not external module and if
        // symbolFromSymbolTable or alias resolution matches the symbol,
        // check the symbol can be qualified
        !symbol_from_symbol_table
            .declarations
            .iter()
            .any(has_non_global_augmentation_external_module_symbol)
            && (ignore_qualification
                || self.can_qualify_symbol(
                    ctx,
                    &self.get_merged_symbol(symbol_from_symbol_table),
                    ctx.meaning,
                ))
    }

    /// Mirrors Go's `canQualifySymbol`.
    fn can_qualify_symbol(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> bool {
        // If the symbol is equivalent and doesn't need further qualification
        if !self.needs_qualification(
            symbol_from_symbol_table,
            ctx.enclosing_declaration.as_ref(),
            meaning,
        ) {
            return true;
        }
        // If symbol needs qualification, make sure that parent is accessible
        if let Some(ref parent) = symbol_from_symbol_table.parent {
            let parent_ctx = AccessibleSymbolChainContext {
                symbol: Arc::clone(parent),
                enclosing_declaration: ctx.enclosing_declaration.clone(),
                meaning: get_qualified_left_meaning(meaning),
                use_only_external_aliasing: ctx.use_only_external_aliasing,
                visited_symbol_tables_map: RefCell::new(
                    ctx.visited_symbol_tables_map.borrow().clone(),
                ),
            };
            !self.get_accessible_symbol_chain_ex(parent_ctx).is_empty()
        } else {
            false
        }
    }

    /// Mirrors Go's `needsQualification`.
    fn needs_qualification(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool {
        let mut qualify = false;
        let tables = self.collect_symbol_tables_in_scope(enclosing_declaration);
        for info in &tables {
            // If symbol of this name is not available in the symbol table we are ok
            let res = match info.table.get(&symbol.name) {
                Some(r) => r,
                None => continue,
            };
            let mut symbol_from_symbol_table = self.get_merged_symbol(res);
            // If the symbol with this name is present it should refer to the symbol
            if symbol_from_symbol_table.id() == symbol.id() {
                // No need to qualify
                return false;
            }

            // Qualify if the symbol from symbol table has same meaning as expected
            let should_resolve_alias = symbol_from_symbol_table
                .flags
                .intersects(SymbolFlags::Alias)
                && symbol_from_symbol_table
                    .declarations
                    .iter()
                    .all(|d| d.kind != SyntaxKind::ExportSpecifier);
            if should_resolve_alias {
                symbol_from_symbol_table = self.resolve_alias(&symbol_from_symbol_table);
            }
            let mut flags = symbol_from_symbol_table.flags;
            if should_resolve_alias {
                flags = self.get_symbol_flags(&symbol_from_symbol_table);
            }
            if flags.intersects(meaning) {
                qualify = true;
                break;
            }
            // Continue to the next symbol table
        }
        qualify
    }

    // ── someSymbolTableInScope ───────────────────────────────────────────

    /// Collect all symbol tables in scope from `enclosing_declaration` upward.
    ///
    /// This is the Rust adaptation of Go's `someSymbolTableInScope`. Instead
    /// of using a callback (which would cause borrow-checker conflicts when
    /// the callback also borrows `&mut self`), we collect all tables into a
    /// Vec and let the caller iterate.
    ///
    /// Mirrors Go's `someSymbolTableInScope`.
    fn collect_symbol_tables_in_scope(
        &mut self,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> Vec<SymbolTableInScope> {
        let mut result: Vec<SymbolTableInScope> = Vec::new();
        let mut location = enclosing_declaration.cloned();

        while let Some(loc) = location {
            // Locals of a source file are not in scope (because they get
            // merged into the global symbol table)
            if can_have_locals(loc.kind) {
                if let Some(locals) = self.program.symbol_map().locals_of(&loc) {
                    // TODO: !ast.IsGlobalSourceFile(location)
                    let is_global_source_file = loc.kind == SyntaxKind::SourceFile
                        && !Checker::is_external_or_common_js_module(&loc);
                    if !is_global_source_file && !locals.is_empty() {
                        result.push(SymbolTableInScope {
                            table: locals.clone(),
                            table_id: symbol_table_id_from_locals(&loc),
                            ignore_qualification: false,
                            is_local_name_lookup: true,
                            scope_node: Some(Arc::clone(&loc)),
                        });
                    }
                }
            }

            match loc.kind {
                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                    // TODO: ast.IsSourceFile && !ast.IsExternalOrCommonJSModule → break
                    if loc.kind == SyntaxKind::SourceFile
                        && !Checker::is_external_or_common_js_module(&loc)
                    {
                        // Non-module source files don't contribute exports
                    } else {
                        // TODO: ast.GetReparsedNodeForNode(location)
                        if let Some(sym) = self.get_symbol_of_declaration(&loc) {
                            if !sym.exports.is_empty() {
                                result.push(SymbolTableInScope {
                                    table: sym.exports.clone(),
                                    table_id: symbol_table_id_from_exports(&sym),
                                    ignore_qualification: false,
                                    is_local_name_lookup: true,
                                    scope_node: Some(Arc::clone(&loc)),
                                });
                            }
                        }
                    }
                }
                SyntaxKind::ClassDeclaration
                | SyntaxKind::ClassExpression
                | SyntaxKind::InterfaceDeclaration => {
                    // Type parameters are bound into `members` lists
                    if let Some(sym) = self.get_symbol_of_declaration(&loc) {
                        let mut table = SymbolTable::new();
                        for (key, member_symbol) in sym.members.entries.iter() {
                            if member_symbol
                                .flags
                                .intersects(SymbolFlags::TYPE.difference(SymbolFlags::Assignment))
                            {
                                table.insert(key.clone(), Arc::clone(member_symbol));
                            }
                        }
                        if !table.is_empty() {
                            result.push(SymbolTableInScope {
                                table,
                                table_id: symbol_table_id_from_members(&sym),
                                ignore_qualification: false,
                                is_local_name_lookup: false,
                                scope_node: Some(Arc::clone(&loc)),
                            });
                        }

                        // Class expression names
                        if loc.kind == SyntaxKind::ClassExpression {
                            // TODO: check if class expression has a name
                            if let Some(name_table) = self.get_class_expression_name_table(&loc) {
                                if !name_table.is_empty() {
                                    result.push(SymbolTableInScope {
                                        table: name_table,
                                        table_id: symbol_table_id_from_locals(&loc),
                                        ignore_qualification: false,
                                        is_local_name_lookup: true,
                                        scope_node: Some(Arc::clone(&loc)),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            location = loc.parent.clone();
        }

        // Globals table is always checked last
        if !self.globals.is_empty() {
            result.push(SymbolTableInScope {
                table: self.globals.clone(),
                table_id: symbol_table_id_from_globals(),
                ignore_qualification: false,
                is_local_name_lookup: true,
                scope_node: None,
            });
        }

        result
    }

    /// Returns a cached symbol table containing the class expression's name
    /// binding.
    ///
    /// Mirrors Go's `getClassExpressionNameTable`.
    fn get_class_expression_name_table(&mut self, location: &Arc<Node>) -> Option<SymbolTable> {
        let node_id = location.id();

        if let Some(table) = self.class_expression_name_tables.get(&node_id) {
            return Some(table.clone());
        }

        let class_symbol = self.get_symbol_of_declaration(location)?;
        // TODO: Get name text from class expression
        // let name_text = location.AsClassExpression().Name().Text()
        // For now, use the symbol's name
        let name_text = class_symbol.name.clone();
        if name_text.is_empty() {
            return None;
        }
        let mut table = SymbolTable::new();
        table.insert(name_text, class_symbol);
        self.class_expression_name_tables
            .insert(node_id, table.clone());
        Some(table)
    }

    // ── Stub helper methods (depend on unported infrastructure) ──────────

    /// Wrapper for emit resolver's `hasVisibleDeclarations` with the
    /// `shouldComputeAliasesToMakeVisible` parameter.
    /// TODO: Full implementation in emitresolver.rs doesn't yet support
    /// the alias computation.
    fn has_visible_declarations_with_aliases(
        &mut self,
        symbol: &Arc<Symbol>,
        _should_compute_aliases_to_make_visible: bool,
    ) -> Option<SymbolAccessibilityResult> {
        self.has_visible_declarations(symbol)
    }

    /// Format a symbol with enclosing declaration context.
    /// Wraps the existing `symbol_to_string_ex` (which doesn't yet use
    /// the enclosing declaration for scoping).
    fn symbol_to_string_ex_enclosing(
        &mut self,
        symbol: &Arc<Symbol>,
        _enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        flags: super::types::SymbolFormatFlags,
    ) -> String {
        // TODO: enclosing_declaration not yet wired into symbol_to_string_ex
        self.symbol_to_string_ex(symbol, flags, meaning)
    }

    // ── Stub methods for unported checker methods ────────────────────────

    /// TODO: Port from Go's `resolveAlias`.
    fn resolve_alias(&mut self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        // TODO: Port full alias resolution
        self.get_merged_symbol(symbol)
    }

    /// TODO: Port from Go's `getExportsOfSymbol`.
    fn get_exports_of_symbol(&self, symbol: &Arc<Symbol>) -> SymbolTable {
        // TODO: Port full implementation (includes export * resolution)
        symbol.exports.clone()
    }

    /// TODO: Port from Go's `getSymbolIfSameReference`.
    fn get_symbol_if_same_reference(
        &self,
        symbol: &Arc<Symbol>,
        other: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        // TODO: Port full reference-equality check (handles merged symbols)
        if symbol.id() == other.id() {
            Some(Arc::clone(symbol))
        } else {
            None
        }
    }

    /// TODO: Port from Go's `getParentOfSymbol`.
    fn get_parent_of_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        symbol.parent.clone()
    }

    /// TODO: Port from Go's `sortSymbols`.
    fn sort_symbols(&self, symbols: &mut Vec<Arc<Symbol>>) {
        // TODO: Port full implementation (sorts by symbol name for stable results)
        symbols.sort_by(|a, b| a.name.cmp(&b.name));
    }
}
