#![allow(dead_code)]
#![allow(unused_variables)]

//! Public API exports for the type checker.
//!
//! Ported 1:1 from `internal/checker/exports.go` in the Go implementation.
//! This module provides the public-facing `Checker` methods consumed by
//! the Language Service and other clients: type/symbol accessors,
//! property and signature queries, module resolution helpers, and more.
//!
//! Many of these are thin wrappers around internal checker methods.
//! Methods whose internal implementation has not yet been ported are
//! stubbed with sensible defaults and `// TODO` markers.
//!
//! NOTE: Some exports.go methods are already defined in other checker
//! submodules (checker.rs, services.rs, relater.rs, etc.) and are not
//! duplicated here. These include the primitive type accessors
//! (`get_string_type`, `get_number_type`, …), `get_properties_of_type`,
//! `get_signatures_of_type`, `is_array_type`, `is_type_assignable_to`,
//! `get_type_of_symbol`, `get_apparent_type`, `resolve_external_module_symbol`,
//! `get_union_type`, `get_type_from_type_node`, and others.

use std::sync::Arc;

use crate::ast::{ModifierFlags, Node, Symbol, SymbolFlags};
use crate::core::compiler_options::ResolutionMode;
use crate::diagnostics::Message;

use super::checker::Checker;
use super::types::*;

// ────────────────────────────────────────────────────────────────────────────
// UnionReduction
// ────────────────────────────────────────────────────────────────────────────

/// Controls how union types are reduced during creation.
///
/// Mirrors Go's `UnionReduction` constants (checker.go).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnionReduction {
    #[default]
    None,
    Literal,
    Subtype,
}

// ────────────────────────────────────────────────────────────────────────────
// Free functions (package-level in Go)
// ────────────────────────────────────────────────────────────────────────────

/// Returns the declaration modifier flags for a symbol.
///
/// Go: `GetDeclarationModifierFlagsFromSymbol` (exports.go).
/// TODO: Full implementation not yet ported.
pub fn get_declaration_modifier_flags_from_symbol(s: &Symbol) -> ModifierFlags {
    // TODO: Port from Go's getDeclarationModifierFlagsFromSymbol
    ModifierFlags::empty()
}

// NOTE: The following free functions from exports.go are already defined
// in `utilities.rs` and are re-exported via `super::utilities`:
//   - `is_type_usable_as_property_name(t: &Type) -> bool`
//   - `get_property_name_from_type(t: &Type) -> String`

impl Checker {
    // ────────────────────────────────────────────────────────────────────────
    // Unknown signature accessor
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetUnknownSignature`.
    pub fn get_unknown_signature(&self) -> Option<Arc<Signature>> {
        self.unknown_signature.get().cloned()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Symbol name-type accessor
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetNameTypeOfSymbol`.
    pub fn get_name_type_of_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        self.value_symbol_links
            .get(symbol)
            .and_then(|links| links.name_type.clone())
    }

    // ────────────────────────────────────────────────────────────────────────
    // Global symbol / type resolution
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetGlobalSymbol`.
    ///
    /// Resolves a symbol from the global scope by name and meaning.
    /// TODO: Full `getGlobalSymbol` implementation not yet ported (diagnostic
    /// reporting, meaning filtering). Falls back to a plain globals lookup.
    pub fn get_global_symbol(
        &self,
        name: &str,
        meaning: SymbolFlags,
        diagnostic: Option<&Message>,
    ) -> Option<Arc<Symbol>> {
        // TODO: call self.get_global_symbol_internal(name, meaning, diagnostic)
        self.globals.get(name).cloned()
    }

    /// Resolve a global symbol by name and meaning.
    ///
    /// Convenience wrapper matching Go's internal `getGlobalSymbolByName`.
    /// TODO: Full meaning-based filtering not yet ported.
    pub fn get_global_symbol_by_name(
        &self,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        self.globals.get(name).cloned()
    }

    /// Resolve a global type by name.
    ///
    /// Mirrors Go's `getGlobalTypeByName`.
    /// TODO: Full type resolution not yet ported.
    pub fn get_global_type_by_name(&self, name: &str) -> Option<Arc<Type>> {
        // TODO: resolve the symbol and get its declared type
        let _symbol = self.globals.get(name)?;
        None
    }

    /// Internal symbol lookup by name and meaning.
    ///
    /// Mirrors Go's `getSymbolByName`.
    /// TODO: Full implementation not yet ported.
    pub fn get_symbol_by_name(&self, name: &str, meaning: SymbolFlags) -> Option<Arc<Symbol>> {
        self.globals.get(name).cloned()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Merged symbol
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetMergedSymbol`.
    ///
    /// NOTE: The private `get_merged_symbol` is already defined in
    /// `checker.rs`. This public wrapper delegates to it when accessible,
    /// otherwise falls back to returning the symbol unchanged.
    pub fn get_merged_symbol_public(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        // TODO: delegate to the private get_merged_symbol once it is pub(crate)
        Some(Arc::clone(symbol))
    }

    // ────────────────────────────────────────────────────────────────────────
    // Ambient module resolution
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `TryFindAmbientModule`.
    ///
    /// TODO: Full `tryFindAmbientModule` implementation not yet ported.
    pub fn try_find_ambient_module(&self, module_name: &str) -> Option<Arc<Symbol>> {
        // TODO: call self.try_find_ambient_module_internal(module_name, true)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Alias resolution
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetImmediateAliasedSymbol`.
    ///
    /// TODO: Full `getImmediateAliasedSymbol` implementation not yet ported.
    pub fn get_immediate_aliased_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        // TODO: call self.get_immediate_aliased_symbol_internal(symbol)
        None
    }

    /// Go: `GetTypeOnlyAliasDeclaration`.
    ///
    /// TODO: Full `getTypeOnlyAliasDeclaration` implementation not yet ported.
    pub fn get_type_only_alias_declaration(&self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
        // TODO: call self.get_type_only_alias_declaration_internal(symbol)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // External module resolution
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `ResolveExternalModuleName`.
    ///
    /// TODO: Full `resolveExternalModuleName` implementation not yet ported.
    pub fn resolve_external_module_name(
        &self,
        module_specifier: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        // TODO: call self.resolve_external_module_name_internal(
        //   module_specifier, module_specifier, true)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Contextual typing
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetTypeOfPropertyOfContextualType` — implemented in
    /// `inference.rs` (`get_type_of_property_of_contextual_type`).

    // ────────────────────────────────────────────────────────────────────────
    // Declared type of symbol
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetDeclaredTypeOfSymbol`.
    ///
    /// TODO: Full `getDeclaredTypeOfSymbol` implementation not yet ported.
    pub fn get_declared_type_of_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // TODO: call self.get_declared_type_of_symbol_internal(symbol)
        self.any_type()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Resolution mode
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetResolutionModeOverride`.
    ///
    /// TODO: Full `getResolutionModeOverride` implementation not yet ported.
    pub fn get_resolution_mode_override(
        &self,
        node: &Arc<Node>,
        report_errors: bool,
    ) -> ResolutionMode {
        // TODO: call self.get_resolution_mode_override_internal(node, report_errors)
        ResolutionMode::default()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Type predicate
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `TypePredicateToString`.
    ///
    /// TODO: Full `typePredicateToString` implementation not yet ported.
    pub fn type_predicate_to_string(&self, t: &TypePredicate) -> String {
        // TODO: call self.type_predicate_to_string_internal(t)
        String::new()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Expanded parameters
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetExpandedParameters`.
    ///
    /// TODO: Full `getExpandedParameters` implementation not yet ported.
    pub fn get_expanded_parameters(
        &self,
        signature: &Arc<Signature>,
        skip_union_expanding: bool,
    ) -> Vec<Vec<Arc<Symbol>>> {
        // TODO: call self.get_expanded_parameters_internal(
        //   signature, skip_union_expanding)
        Vec::new()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Resolved signature
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetResolvedSignature`.
    ///
    /// TODO: Full `getResolvedSignature` implementation not yet ported.
    pub fn get_resolved_signature(&self, node: &Arc<Node>) -> Option<Arc<Signature>> {
        // TODO: call self.get_resolved_signature_internal(
        //   node, None, CheckMode::Normal)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Contextual type for argument
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetContextualTypeForArgumentAtIndex`.
    ///
    /// TODO: Full `getContextualTypeForArgumentAtIndex` not yet ported.
    pub fn get_contextual_type_for_argument_at_index(
        &self,
        node: &Arc<Node>,
        arg_index: usize,
    ) -> Option<Arc<Type>> {
        // TODO: call self.get_contextual_type_for_argument_at_index_internal(
        //   node, arg_index)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Index signatures at location
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetIndexSignaturesAtLocation`.
    ///
    /// TODO: Full `getIndexSignaturesAtLocation` implementation not yet ported.
    pub fn get_index_signatures_at_location(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        // TODO: call self.get_index_signatures_at_location_internal(node)
        Vec::new()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Resolved symbol
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetResolvedSymbol`.
    ///
    /// TODO: Full `getResolvedSymbol` implementation not yet ported.
    pub fn get_resolved_symbol(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        // TODO: call self.get_resolved_symbol_internal(node)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // JSX
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetJsxFragmentFactory`.
    ///
    /// Returns the name of the JSX fragment factory (e.g. `"Fragment"`).
    /// TODO: Full `getJsxFragmentFactoryEntity` implementation not yet ported.
    pub fn get_jsx_fragment_factory(&self, location: &Arc<Node>) -> String {
        // TODO: resolve entity via getJsxFragmentFactoryEntity and return
        // ast.GetFirstIdentifier(entity).Text()
        String::new()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Name resolution
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `ResolveName`.
    ///
    /// TODO: Full `resolveName` implementation not yet ported.
    pub fn resolve_name(
        &self,
        name: &str,
        location: &Arc<Node>,
        meaning: SymbolFlags,
        exclude_globals: bool,
    ) -> Option<Arc<Symbol>> {
        // TODO: call self.resolve_name_internal(
        //   location, name, meaning, None, true, exclude_globals)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Symbol flags
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetSymbolFlags`.
    ///
    /// TODO: Full `getSymbolFlags` implementation not yet ported.
    pub fn get_symbol_flags(&self, symbol: &Arc<Symbol>) -> SymbolFlags {
        // TODO: call self.get_symbol_flags_internal(symbol)
        symbol.flags
    }

    // ────────────────────────────────────────────────────────────────────────
    // Base types
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetBaseTypes`.
    ///
    /// TODO: Full `getBaseTypes` implementation not yet ported.
    pub fn get_base_types(&self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        // TODO: call self.get_base_types_internal(t)
        Vec::new()
    }

    /// Go: `GetBaseConstructorTypeOfClass`.
    ///
    /// TODO: Full `getBaseConstructorTypeOfClass` not yet ported.
    pub fn get_base_constructor_type_of_class(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        // TODO: call self.get_base_constructor_type_of_class_internal(t)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Rest type of signature
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetRestTypeOfSignature`.
    ///
    /// TODO: Full `getRestTypeOfSignature` implementation not yet ported.
    pub fn get_rest_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        // TODO: call self.get_rest_type_of_signature_internal(sig)
        None
    }

    // ────────────────────────────────────────────────────────────────────────
    // Context sensitivity
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `IsContextSensitive`.
    ///
    /// TODO: Full `isContextSensitive` implementation not yet ported.
    pub fn is_context_sensitive(&self, node: &Arc<Node>) -> bool {
        // TODO: call self.is_context_sensitive_internal(node)
        false
    }

    // ────────────────────────────────────────────────────────────────────────
    // Type arguments
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `FillMissingTypeArguments`.
    ///
    /// TODO: Full `fillMissingTypeArguments` not yet ported.
    pub fn fill_missing_type_arguments(
        &self,
        type_arguments: &[Arc<Type>],
        type_parameters: &[Arc<Type>],
        min_type_argument_count: usize,
        is_java_script_implicit_any: bool,
    ) -> Vec<Arc<Type>> {
        // TODO: call self.fill_missing_type_arguments_internal(
        //   type_arguments, type_parameters,
        //   min_type_argument_count, is_java_script_implicit_any)
        type_arguments.to_vec()
    }

    /// Go: `GetMinTypeArgumentCount`.
    ///
    /// TODO: Full `getMinTypeArgumentCount` implementation not yet ported.
    pub fn get_min_type_argument_count(&self, type_parameters: &[Arc<Type>]) -> usize {
        // TODO: call self.get_min_type_argument_count_internal(type_parameters)
        type_parameters.len()
    }

    // ────────────────────────────────────────────────────────────────────────
    // Union type (extended)
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `GetUnionTypeEx`.
    ///
    /// TODO: Full `getUnionTypeEx` implementation not yet ported.
    pub fn get_union_type_ex(
        &self,
        types: Vec<Arc<Type>>,
        union_reduction: UnionReduction,
    ) -> Arc<Type> {
        // TODO: call self.get_union_type_ex_internal(
        //   types, union_reduction, None, None)
        self.build_union_from_types(types)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Implicit undefined
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `RequiresAddingImplicitUndefined`.
    ///
    /// TODO: Full implementation not yet ported (requires
    /// `ast.FindAncestor`, `node.Symbol()`, `GetEmitResolver`).
    pub fn requires_adding_implicit_undefined(&self, node: &Arc<Node>) -> bool {
        // TODO: port full logic:
        //   enclosingDeclaration := ast.FindAncestor(node, ast.IsDeclaration)
        //   if enclosingDeclaration == nil {
        //       enclosingDeclaration = ast.GetSourceFileOfNode(node).AsNode()
        //   }
        //   symbol := node.Symbol()
        //   if symbol == nil { return false }
        //   return c.GetEmitResolver().RequiresAddingImplicitUndefined(
        //       node, symbol, enclosingDeclaration)
        false
    }

    /// Go: `RemoveMissingOrUndefinedType`.
    ///
    /// TODO: Full `removeMissingOrUndefinedType` not yet ported.
    pub fn remove_missing_or_undefined_type(&self, t: &Arc<Type>) -> Arc<Type> {
        // TODO: call self.remove_missing_or_undefined_type_internal(t)
        Arc::clone(t)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Symbol comparison
    // ────────────────────────────────────────────────────────────────────────

    /// Go: `CompareSymbols`.
    ///
    /// Returns: -1 (s1 < s2), 0 (equal), 1 (s1 > s2).
    /// TODO: Full `compareSymbols` implementation not yet ported.
    pub fn compare_symbols(&self, s1: &Arc<Symbol>, s2: &Arc<Symbol>) -> i32 {
        // TODO: call self.compare_symbols_internal(s1, s2)
        0
    }

    // ────────────────────────────────────────────────────────────────────────
    // Lib-resolved types
    // ────────────────────────────────────────────────────────────────────────

    /// Resolve the `default` keyword type from the loaded libraries.
    ///
    /// Mirrors Go's `getDefaultKeywordType`.
    /// TODO: Full lib resolution not yet ported.
    pub fn get_default_keyword_type(&self) -> Option<Arc<Type>> {
        // TODO: resolve from global scope
        self.get_global_type_by_name("default")
    }

    /// Resolve the `Promise` type from the loaded libraries.
    ///
    /// Mirrors Go's `getPromiseType`.
    /// TODO: Full lib resolution not yet ported.
    pub fn get_promise_type(&self) -> Option<Arc<Type>> {
        self.global_promise_type.get().cloned()
    }

    /// Resolve the `PromiseLike` type from the loaded libraries.
    ///
    /// Mirrors Go's `getPromiseLikeType`.
    /// TODO: Full lib resolution not yet ported.
    pub fn get_promise_like_type(&self) -> Option<Arc<Type>> {
        // TODO: resolve from global scope
        self.get_global_type_by_name("PromiseLike")
    }

    // ────────────────────────────────────────────────────────────────────────
    // Type checker cache management
    // ────────────────────────────────────────────────────────────────────────

    /// Create a type checker cache.
    ///
    /// Mirrors Go's `createTypeCheckerCache`.
    /// TODO: Full implementation not yet ported.
    pub fn create_type_checker_cache(&self) {
        // TODO: allocate and return a TypeCheckerCache
    }

    /// Clear possible type requests.
    ///
    /// Mirrors Go's `clearPossibleTypeRequests`.
    /// TODO: Full implementation not yet ported.
    pub fn clear_possible_type_requests(&mut self) {
        // TODO: clear pending type resolution requests
    }
}
