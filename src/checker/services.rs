#![allow(dead_code)]
#![allow(unused_variables)]

//! Public API services for the Language Service layer.
//!
//! Ported from `internal/checker/services.go` (~1,140 lines in Go). This
//! module exposes the public-facing checker methods consumed by the
//! Language Service: symbol/type queries, contextual typing, signature
//! resolution, property access validation, reference finding, etc.
//!
//! Methods are organized in the same order as the Go source file.

use std::sync::Arc;

use crate::ast::{
    CheckFlags, Node, NodeData, SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::evaluator::EvalValue;

use super::checker::Checker;
use super::types::*;
use super::utilities::{
    get_property_name_from_type, is_literal_type, is_tuple_type, is_type_any,
    is_type_usable_as_property_name,
};

// ────────────────────────────────────────────────────────────────────────────
// Free helper functions (used by services.go methods)
// ────────────────────────────────────────────────────────────────────────────

/// Whether a name is a reserved internal member name (e.g. `\u{FE}call`).
/// Mirrors Go's `isReservedMemberName` (utilities.go).
pub fn is_reserved_member_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c == '\u{FE}' => match chars.next() {
            Some('@') | Some('#') => false,
            Some(_) => true,
            None => false,
        },
        _ => false,
    }
}

/// Convert a `SymbolTable` to a `Vec`, filtering out reserved member names.
/// Mirrors Go's `symbolsToArray` (utilities.go).
pub fn symbols_to_array(symbols: &SymbolTable) -> Vec<Arc<Symbol>> {
    symbols
        .entries
        .values()
        .filter(|s| !is_reserved_member_name(&s.name))
        .cloned()
        .collect()
}

/// Whether a node kind introduces an `arguments` exotic object.
/// Mirrors Go's `introducesArgumentsExoticObject` (utilities.go).
pub fn introduces_arguments_exotic_object(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
    )
}

/// Known generic type names whose first type argument is often queried
/// for quick info / completions. Mirrors Go's `knownGenericTypeNames`.
pub const KNOWN_GENERIC_TYPE_NAMES: &[&str] = &[
    "Array",
    "ArrayLike",
    "ReadonlyArray",
    "Promise",
    "PromiseLike",
    "Iterable",
    "IterableIterator",
    "AsyncIterable",
    "Set",
    "WeakSet",
    "ReadonlySet",
    "Map",
    "WeakMap",
    "ReadonlyMap",
    "Partial",
    "Required",
    "Readonly",
    "Pick",
    "Omit",
    "NonNullable",
];

/// Whether a name is a known generic type name.
/// Mirrors Go's `isKnownGenericTypeName`.
fn is_known_generic_name(name: &str) -> bool {
    KNOWN_GENERIC_TYPE_NAMES.contains(&name)
}

// ────────────────────────────────────────────────────────────────────────────
// impl Checker — public API services
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    // ── Scope & module queries ──────────────────────────────────────────────

    /// Get all symbols in scope at a location that match the given meaning.
    /// Go: `(c *Checker) GetSymbolsInScope`.
    ///
    /// TODO: Full scope-walking implementation depends on `node.locals()`
    /// (binder side-table), `getMembersOfSymbol`, and other unported
    /// internals. Currently returns an empty result.
    pub fn get_symbols_in_scope(
        &self,
        _location: &Arc<Node>,
        _meaning: SymbolFlags,
    ) -> Vec<Arc<Symbol>> {
        // TODO: Port the full scope-walking algorithm from services.go.
        // The Go implementation walks the parent chain, copying locals,
        // module exports, class/interface members, and globals into a
        // symbol table, then converts to an array.
        Vec::new()
    }

    /// Get the exports of a module symbol as an array.
    /// Go: `(c *Checker) GetExportsOfModule`.
    pub fn get_exports_of_module(&self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        symbols_to_array(&self.get_exports_of_module_table(symbol))
    }

    /// Get the exports of a module symbol as a `SymbolTable`.
    /// Go: `(c *Checker) getExportsOfModule` (defined in checker.go).
    ///
    /// The full Go implementation resolves `export *` declarations via
    /// `getExportsOfModuleWorker` and caches the result in
    /// `moduleSymbolLinks`. This stub returns the symbol's direct exports.
    pub fn get_exports_of_module_table(&self, module_symbol: &Arc<Symbol>) -> SymbolTable {
        if let Some(links) = self.module_symbol_links.get(module_symbol) {
            if !links.resolved_exports.is_empty() {
                return links.resolved_exports.clone();
            }
        }
        // Fallback: return direct exports.
        module_symbol.exports.clone()
    }

    /// Iterate over all exports and properties of a module symbol.
    /// Go: `(c *Checker) ForEachExportAndPropertyOfModule`.
    pub fn for_each_export_and_property_of_module(
        &mut self,
        module_symbol: &Arc<Symbol>,
        cb: &mut dyn FnMut(&Arc<Symbol>, &str),
    ) {
        for (key, exported_symbol) in self
            .get_exports_of_module_table(module_symbol)
            .entries
            .iter()
        {
            if !is_reserved_member_name(key) {
                cb(exported_symbol, key);
            }
        }

        let export_equals = self.resolve_external_module_symbol(module_symbol, false);
        if Arc::ptr_eq(&export_equals, module_symbol) {
            return;
        }

        let type_of_symbol = self.get_type_of_symbol(&export_equals);
        if !self.should_treat_properties_of_external_module_as_exports(&type_of_symbol) {
            return;
        }

        // TODO: forEachPropertyOfType — requires getReducedApparentType
        // and resolveStructuredTypeMembers (not yet ported).
        let reduced_type = self.get_reduced_apparent_type(&type_of_symbol);
        if !reduced_type.flags.intersects(TYPE_FLAGS_STRUCTURED_TYPE) {
            return;
        }
        if let Some(structured) = reduced_type.as_structured() {
            for (name, symbol) in structured.members.entries.iter() {
                if self.is_named_member(symbol, name) {
                    cb(symbol, name);
                }
            }
        }
    }

    // ── Property access validation ──────────────────────────────────────────

    /// Whether a property access expression is valid.
    /// Go: `(c *Checker) IsValidPropertyAccess`.
    pub fn is_valid_property_access(&mut self, node: &Arc<Node>, property_name: &str) -> bool {
        match node.kind {
            SyntaxKind::PropertyAccessExpression => {
                let is_super = if let Some(expr) = node.expression() {
                    expr.kind == SyntaxKind::SuperKeyword
                } else {
                    false
                };
                let t = if let Some(expr) = node.expression() {
                    self.check_expression(expr);
                    let widened = self.get_widened_type_of_expression(expr);
                    widened
                } else {
                    self.any_type()
                };
                self.is_valid_property_access_with_type(node, is_super, property_name, &t)
            }
            SyntaxKind::QualifiedName => {
                let t = if let NodeData::QualifiedName(data) = &node.data {
                    self.check_expression(&data.left);
                    self.get_widened_type_of_expression(&data.left)
                } else {
                    self.any_type()
                };
                self.is_valid_property_access_with_type(node, false, property_name, &t)
            }
            SyntaxKind::ImportType => {
                let t = self.get_type_from_type_node(node);
                self.is_valid_property_access_with_type(node, false, property_name, &t)
            }
            _ => {
                panic!(
                    "Unexpected node kind in isValidPropertyAccess: {:?}",
                    node.kind
                )
            }
        }
    }

    /// Helper for `is_valid_property_access`.
    /// Go: `(c *Checker) isValidPropertyAccessWithType`.
    pub fn is_valid_property_access_with_type(
        &self,
        node: &Arc<Node>,
        is_super: bool,
        property_name: &str,
        t: &Arc<Type>,
    ) -> bool {
        // Short-circuit for `any` type.
        if is_type_any(t) {
            return true;
        }
        let prop = self.get_property_of_type(t, property_name);
        prop.is_some() && self.is_property_accessible(node, is_super, false, t, &prop.unwrap())
    }

    /// Whether a property access is valid for completion purposes.
    /// Go: `(c *Checker) IsValidPropertyAccessForCompletions`.
    pub fn is_valid_property_access_for_completions(
        &self,
        node: &Arc<Node>,
        t: &Arc<Type>,
        property: &Arc<Symbol>,
    ) -> bool {
        let is_super = node.kind == SyntaxKind::PropertyAccessExpression
            && node
                .expression()
                .map(|e| e.kind == SyntaxKind::SuperKeyword)
                .unwrap_or(false);
        self.is_property_accessible(node, is_super, false, t, property)
    }

    /// Get all possible properties of a union of types.
    /// Go: `(c *Checker) GetAllPossiblePropertiesOfTypes`.
    pub fn get_all_possible_properties_of_types(
        &mut self,
        types: &[Arc<Type>],
    ) -> Vec<Arc<Symbol>> {
        let union_type = self.get_union_type(types.to_vec());
        if !union_type.flags.contains(TypeFlags::Union) {
            return self.get_augmented_properties_of_type(&union_type);
        }

        let mut props: std::collections::HashMap<String, Arc<Symbol>> =
            std::collections::HashMap::new();
        for member_type in types {
            let augmented_props = self.get_augmented_properties_of_type(member_type);
            for p in augmented_props {
                if !props.contains_key(&p.name) {
                    // TODO: createUnionOrIntersectionProperty (not yet ported).
                    // For now, use the property directly.
                    props.insert(p.name.clone(), Arc::clone(&p));
                }
            }
        }
        props.into_values().collect()
    }

    // ── Symbol identity checks ──────────────────────────────────────────────

    /// Whether a symbol is the checker's `unknownSymbol`.
    /// Go: `(c *Checker) IsUnknownSymbol`.
    pub fn is_unknown_symbol(&self, symbol: &Arc<Symbol>) -> bool {
        self.unknown_symbol
            .as_ref()
            .map(|s| Arc::ptr_eq(s, symbol))
            .unwrap_or(false)
    }

    /// Whether a symbol is the checker's `undefinedSymbol`.
    /// Go: `(c *Checker) IsUndefinedSymbol`.
    pub fn is_undefined_symbol(&self, symbol: &Arc<Symbol>) -> bool {
        self.undefined_symbol
            .as_ref()
            .map(|s| Arc::ptr_eq(s, symbol))
            .unwrap_or(false)
    }

    /// Whether a symbol is the checker's `argumentsSymbol`.
    /// Go: `(c *Checker) IsArgumentsSymbol`.
    pub fn is_arguments_symbol(&self, symbol: &Arc<Symbol>) -> bool {
        self.arguments_symbol
            .as_ref()
            .map(|s| Arc::ptr_eq(s, symbol))
            .unwrap_or(false)
    }

    // ── Type queries ────────────────────────────────────────────────────────

    /// Remove the optional marker from a type.
    /// Go: `(c *Checker) GetNonOptionalType`.
    pub fn get_non_optional_type(&self, t: &Arc<Type>) -> Arc<Type> {
        self.remove_optional_type_marker(t)
    }

    /// Get the string index type of a type.
    /// Go: `(c *Checker) GetStringIndexType`.
    pub fn get_string_index_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_index_type_of_type(t, IndexKind::String)
    }

    /// Get the number index type of a type.
    /// Go: `(c *Checker) GetNumberIndexType`.
    pub fn get_number_index_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_index_type_of_type(t, IndexKind::Number)
    }

    /// Get the element type of an array type.
    /// Go: `(c *Checker) GetElementTypeOfArrayType`.
    pub fn get_element_type_of_array_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        // Array<T> has the element type as the first type argument.
        let type_args = self.get_type_arguments(t);
        if let Some(elem) = type_args.first() {
            return Some(Arc::clone(elem));
        }
        None
    }

    /// Get the call signatures of a type.
    /// Go: `(c *Checker) GetCallSignatures`.
    pub fn get_call_signatures(&self, t: &Arc<Type>) -> Vec<Arc<Signature>> {
        self.get_signatures_of_type(t, SignatureKind::Call)
    }

    /// Get the construct signatures of a type.
    /// Go: `(c *Checker) GetConstructSignatures`.
    pub fn get_construct_signatures(&self, t: &Arc<Type>) -> Vec<Arc<Signature>> {
        self.get_signatures_of_type(t, SignatureKind::Construct)
    }

    /// Get the apparent properties of a type (including augmented function
    /// properties).
    /// Go: `(c *Checker) GetApparentProperties`.
    pub fn get_apparent_properties(&mut self, t: &Arc<Type>) -> Vec<Arc<Symbol>> {
        self.get_augmented_properties_of_type(t)
    }

    /// Get augmented properties of a type (apparent type properties plus
    /// function-specific properties if the type is callable/constructable).
    /// Go: `(c *Checker) getAugmentedPropertiesOfType`.
    pub fn get_augmented_properties_of_type(&mut self, t: &Arc<Type>) -> Vec<Arc<Symbol>> {
        let apparent = self.get_apparent_type(t);
        let props_list = self.get_properties_of_type(&apparent);
        let mut props_by_name: std::collections::HashMap<String, Arc<Symbol>> =
            std::collections::HashMap::new();
        for p in &props_list {
            props_by_name.insert(p.name.clone(), Arc::clone(p));
        }

        let call_sigs = self.get_signatures_of_type(&apparent, SignatureKind::Call);
        let construct_sigs = self.get_signatures_of_type(&apparent, SignatureKind::Construct);
        let function_type = if !call_sigs.is_empty() {
            self.global_callable_function_type()
        } else if !construct_sigs.is_empty() {
            self.global_newable_function_type()
        } else {
            None
        };

        if let Some(ref ft) = function_type {
            for p in self.get_properties_of_type(ft) {
                if !props_by_name.contains_key(&p.name) {
                    props_by_name.insert(p.name.clone(), Arc::clone(&p));
                }
            }
        }

        self.get_named_members(&props_by_name)
    }

    // ── Module member lookup ────────────────────────────────────────────────

    /// Try to find a member in module exports and properties.
    /// Go: `(c *Checker) TryGetMemberInModuleExportsAndProperties`.
    pub fn try_get_member_in_module_exports_and_properties(
        &mut self,
        member_name: &str,
        module_symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        if let Some(symbol) = self.try_get_member_in_module_exports(member_name, module_symbol) {
            return Some(symbol);
        }

        let export_equals = self.resolve_external_module_symbol(module_symbol, false);
        if Arc::ptr_eq(&export_equals, module_symbol) {
            return None;
        }

        let t = self.get_type_of_symbol(&export_equals);
        if self.should_treat_properties_of_external_module_as_exports(&t) {
            return self.get_property_of_type(&t, member_name);
        }
        None
    }

    /// Try to find a member in module exports only.
    /// Go: `(c *Checker) TryGetMemberInModuleExports`.
    pub fn try_get_member_in_module_exports(
        &self,
        member_name: &str,
        module_symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let symbol_table = self.get_exports_of_module_table(module_symbol);
        symbol_table.get(member_name).cloned()
    }

    /// Whether properties of an external module's `export =` value should be
    /// treated as exports.
    /// Go: `(c *Checker) shouldTreatPropertiesOfExternalModuleAsExports`.
    pub fn should_treat_properties_of_external_module_as_exports(
        &self,
        resolved_external_module_type: &Arc<Type>,
    ) -> bool {
        !resolved_external_module_type
            .flags
            .intersects(TYPE_FLAGS_PRIMITIVE)
            || resolved_external_module_type
                .object_flags
                .contains(ObjectFlags::Class)
            || self.is_array_type(resolved_external_module_type)
            || is_tuple_type(resolved_external_module_type)
    }

    // ── Contextual typing ───────────────────────────────────────────────────

    /// Get the contextual type for an expression node.
    /// Go: `(c *Checker) GetContextualType`.
    ///
    /// Note: The Go public wrapper adds inference-blocking logic when
    /// `ContextFlags::IgnoreNodeInferences` is set. That machinery
    /// (`runWithInferenceBlockedFromSourceNode`) depends on unported
    /// internals. The core `get_contextual_type` is defined in
    /// `inference.rs`.
    pub fn get_contextual_type_for_services(
        &mut self,
        node: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        // TODO: Add inference-blocking logic when ContextFlags::IgnoreNodeInferences
        // is set, matching Go's `runWithInferenceBlockedFromSourceNode`.
        self.get_contextual_type(node, context_flags)
    }

    /// Get the resolved signature for signature help.
    /// Go: `GetResolvedSignatureForSignatureHelp` (package-level function).
    pub fn get_resolved_signature_for_signature_help(
        &mut self,
        node: &Arc<Node>,
        argument_count: i32,
    ) -> (Option<Arc<Signature>>, Vec<Arc<Signature>>) {
        // TODO: Add caching bypass logic matching Go's
        // `runWithoutResolvedSignatureCaching`.
        self.get_resolved_signature_worker(node, CheckMode::IsForSignatureHelp, argument_count)
    }

    // ── Alias & root symbol resolution ─────────────────────────────────────

    /// Skip an alias symbol, resolving to its target.
    /// Go: `(c *Checker) SkipAlias`.
    pub fn skip_alias(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if symbol.flags.contains(SymbolFlags::Alias) {
            return self.get_aliased_symbol(symbol);
        }
        Arc::clone(symbol)
    }

    /// Get the aliased symbol target of an import/export alias.
    /// Go: `(c *Checker) GetAliasedSymbol` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn get_aliased_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(links) = self.alias_symbol_links.get(symbol) {
            if let Some(ref target) = links.alias_target {
                return Arc::clone(target);
            }
            if let Some(ref target) = links.immediate_target {
                return Arc::clone(target);
            }
        }
        Arc::clone(symbol)
    }

    /// Get the root symbols of a symbol (following synthetic/transient
    /// origins).
    /// Go: `(c *Checker) GetRootSymbols`.
    pub fn get_root_symbols(&self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        let roots = self.get_immediate_root_symbols(symbol);
        if roots.is_empty() {
            return vec![Arc::clone(symbol)];
        }
        let mut result = Vec::new();
        for root in &roots {
            result.extend(self.get_root_symbols(root));
        }
        result
    }

    /// Get the immediate root symbols of a symbol.
    /// Go: `(c *Checker) getImmediateRootSymbols`.
    pub fn get_immediate_root_symbols(&self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        if symbol.check_flags.intersects(CheckFlags::SYNTHETIC) {
            // Synthetic property: look up the containing type's constituents.
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref containing) = links.containing_type {
                    let types = containing.types().unwrap_or(&[]);
                    let mut result = Vec::new();
                    for t in types {
                        if let Some(prop) = self.get_property_of_type(t, &symbol.name) {
                            result.push(prop);
                        }
                    }
                    return result;
                }
            }
        }
        if symbol.flags.contains(SymbolFlags::Transient) {
            if let Some(links) = self.spread_links.get(symbol) {
                if links.left_spread.is_some() {
                    let mut result = Vec::new();
                    if let Some(ref left) = links.left_spread {
                        result.push(Arc::clone(left));
                    }
                    if let Some(ref right) = links.right_spread {
                        result.push(Arc::clone(right));
                    }
                    return result;
                }
            }
            if let Some(links) = self.mapped_symbol_links.get(symbol) {
                if let Some(ref origin) = links.synthetic_origin {
                    return vec![Arc::clone(origin)];
                }
            }
            let target = self.try_get_target(symbol);
            if let Some(target) = target {
                return vec![target];
            }
        }
        Vec::new()
    }

    /// Follow the `target` chain of a symbol.
    /// Go: `(c *Checker) tryGetTarget`.
    pub fn try_get_target(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let mut target = None;
        let mut next = Arc::clone(symbol);
        loop {
            let resolved = if let Some(links) = self.value_symbol_links.get(&next) {
                links.target.clone()
            } else if let Some(links) = self.export_type_links.get(&next) {
                links.target.clone()
            } else {
                None
            };
            match resolved {
                Some(n) => {
                    target = Some(Arc::clone(&n));
                    next = n;
                }
                None => break,
            }
        }
        target
    }

    /// Get the mapped-type symbol that contains a property.
    /// Go: `(c *Checker) GetMappedTypeSymbolOfProperty`.
    pub fn get_mapped_type_symbol_of_property(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        if let Some(value_links) = self.value_symbol_links.get(symbol) {
            if let Some(ref containing) = value_links.containing_type {
                return containing.symbol.clone();
            }
        }
        None
    }

    /// Get the export symbol of a symbol.
    /// Go: `(c *Checker) GetExportSymbolOfSymbol`.
    pub fn get_export_symbol_of_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        // Inline get_merged_symbol since it's private in checker.rs.
        let source = if let Some(ref export) = symbol.export_symbol {
            Arc::clone(export)
        } else {
            Arc::clone(symbol)
        };
        // get_merged_symbol currently just returns the symbol itself.
        source
    }

    /// Get the local target symbol of an export specifier.
    /// Go: `(c *Checker) GetExportSpecifierLocalTargetSymbol`.
    pub fn get_export_specifier_local_target_symbol(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        match node.kind {
            SyntaxKind::ExportSpecifier => {
                // TODO: resolveEntityName + getExternalModuleMember.
                // Requires full entity name resolution (not yet ported).
                None
            }
            SyntaxKind::Identifier => {
                // TODO: resolveEntityName.
                None
            }
            _ => {
                panic!(
                    "Unhandled case in getExportSpecifierLocalTargetSymbol, node should be ExportSpecifier | Identifier"
                )
            }
        }
    }

    /// Get the value symbol of a shorthand property assignment.
    /// Go: `(c *Checker) GetShorthandAssignmentValueSymbol`.
    pub fn get_shorthand_assignment_value_symbol(
        &mut self,
        location: Option<&Arc<Node>>,
    ) -> Option<Arc<Symbol>> {
        if let Some(loc) = location {
            if loc.kind == SyntaxKind::ShorthandPropertyAssignment {
                if let Some(name) = loc.name() {
                    // TODO: resolveEntityName with SymbolFlagsValue | Alias.
                    return None;
                }
            }
        }
        None
    }

    /// Get the parameter and property symbols of a parameter property
    /// declaration.
    /// Go: `(c *Checker) GetSymbolsOfParameterPropertyDeclaration`.
    pub fn get_symbols_of_parameter_property_declaration(
        &self,
        parameter: &Arc<Node>,
        parameter_name: &str,
    ) -> Option<(Arc<Symbol>, Arc<Symbol>)> {
        let constructor_declaration = parameter.parent.as_ref()?;
        let class_declaration = constructor_declaration.parent.as_ref()?;

        // TODO: getSymbol from constructor locals and class members.
        // Requires locals lookup (not yet ported).
        let _ = parameter_name;
        let _ = class_declaration;
        None
    }

    // ── Declaration usage & reference finding ──────────────────────────────

    /// Whether an import declaration identifier is used in the source file.
    /// Go: `(c *Checker) IsDeclarationUsed`.
    pub fn is_declaration_used(
        &mut self,
        source_file: &Arc<SourceFile>,
        identifier: &Arc<Node>,
        jsx_elements_present: bool,
        jsx_mode_needs_explicit_import: bool,
    ) -> bool {
        if jsx_elements_present && jsx_mode_needs_explicit_import {
            // TODO: getJsxNamespace / GetJsxFragmentFactory.
            let identifier_text = identifier.text();
            // Simplified: check against common JSX namespaces.
            if identifier_text == "React" || identifier_text == "h" {
                return true;
            }
        }

        let symbol = self.get_symbol_at_location(identifier);
        let symbol = match symbol {
            Some(s) => s,
            None => return true,
        };

        self.is_symbol_referenced_in_file(source_file, identifier, &symbol)
    }

    /// Whether a symbol is referenced in a source file (besides its definition).
    /// Go: `(c *Checker) IsSymbolReferencedInFile`.
    pub fn is_symbol_referenced_in_file(
        &mut self,
        source_file: &Arc<SourceFile>,
        definition: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {
        let identifier_text = definition.text();
        for token in get_possible_symbol_reference_nodes(source_file, identifier_text, None) {
            if token.kind != SyntaxKind::Identifier {
                continue;
            }
            if token.text() != identifier_text {
                continue;
            }
            if Arc::ptr_eq(&token, definition) {
                continue;
            }
            let ref_symbol = self.get_symbol_at_location(&token);
            if let Some(ref ref_sym) = ref_symbol {
                if Arc::ptr_eq(ref_sym, symbol) {
                    return true;
                }
            }
            if let Some(ref parent) = token.parent {
                if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
                    let shorthand_symbol = self.get_shorthand_assignment_value_symbol(Some(parent));
                    if let Some(ref shorthand) = shorthand_symbol {
                        if Arc::ptr_eq(shorthand, symbol) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Get all identifier nodes in the file that reference the given symbol.
    /// Go: `(c *Checker) GetReferencesToSymbolInFile`.
    pub fn get_references_to_symbol_in_file(
        &mut self,
        source_file: &Arc<SourceFile>,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Node>> {
        let identifier_text = &symbol.name;
        let mut result = Vec::new();
        for token in get_possible_symbol_reference_nodes(source_file, identifier_text, None) {
            if token.kind != SyntaxKind::Identifier {
                continue;
            }
            if token.text() != identifier_text.as_str() {
                continue;
            }
            let ref_symbol = self.get_symbol_at_location(&token);
            if let Some(ref ref_sym) = ref_symbol {
                if Arc::ptr_eq(ref_sym, symbol) {
                    result.push(Arc::clone(&token));
                    continue;
                }
            }
            if let Some(ref parent) = token.parent {
                if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
                    let shorthand_symbol = self.get_shorthand_assignment_value_symbol(Some(parent));
                    if let Some(ref shorthand) = shorthand_symbol {
                        if Arc::ptr_eq(shorthand, symbol) {
                            result.push(Arc::clone(&token));
                            continue;
                        }
                    }
                }
            }
        }
        result
    }

    // ── Type argument constraints ──────────────────────────────────────────

    /// Get the type argument constraint of a type node.
    /// Go: `(c *Checker) GetTypeArgumentConstraint`.
    pub fn get_type_argument_constraint(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        // TODO: Full implementation requires getUninstantiatedSignatures,
        // getTypeParametersForTypeReferenceOrImport, instantiateType,
        // newTypeMapper, getEffectiveTypeArguments — not yet ported.
        None
    }

    // ── Union discriminant checks ──────────────────────────────────────────

    /// Whether a type is invalid due to a union discriminant.
    /// Go: `(c *Checker) IsTypeInvalidDueToUnionDiscriminant`.
    pub fn is_type_invalid_due_to_union_discriminant(
        &mut self,
        contextual_type: &Arc<Type>,
        obj: &Arc<Node>,
    ) -> bool {
        let properties: Vec<Arc<Node>> = match &obj.data {
            NodeData::ObjectLiteralExpression(data) => data.properties.nodes.clone(),
            _ => Vec::new(),
        };
        for property in &properties {
            let mut name_type = None;
            if let Some(property_name) = property.name() {
                if property_name.kind == SyntaxKind::JsxNamespacedName {
                    name_type = Some(self.get_string_literal_type(property_name.text()));
                } else {
                    name_type = self.get_literal_type_from_property_name(property_name);
                }
            }
            let mut name = String::new();
            if let Some(ref nt) = name_type {
                if is_type_usable_as_property_name(nt) {
                    name = get_property_name_from_type(nt);
                }
            }
            let mut expected = None;
            if !name.is_empty() {
                expected = self.get_type_of_property_of_type(contextual_type, &name);
            }
            if let Some(ref exp) = expected {
                if is_literal_type(exp) {
                    let prop_type = self.get_type_of_node(property);
                    if !self.is_type_assignable_to(&prop_type, exp) {
                        return true;
                    }
                }
            }
        }
        false
    }

    // ── Module exports & properties ─────────────────────────────────────────

    /// Get exports and properties of a module (includes `export =` properties).
    /// Go: `(c *Checker) GetExportsAndPropertiesOfModule`.
    pub fn get_exports_and_properties_of_module(
        &mut self,
        module_symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Symbol>> {
        let mut exports = self.get_exports_of_module_as_array(module_symbol);
        let export_equals = self.resolve_external_module_symbol(module_symbol, false);
        if !Arc::ptr_eq(&export_equals, module_symbol) {
            let t = self.get_type_of_symbol(&export_equals);
            if self.should_treat_properties_of_external_module_as_exports(&t) {
                exports.extend(self.get_properties_of_type(&t));
            }
        }
        exports
    }

    /// Get exports of a module as an array (internal helper).
    /// Go: `(c *Checker) getExportsOfModuleAsArray`.
    pub fn get_exports_of_module_as_array(&self, module_symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        symbols_to_array(&self.get_exports_of_module_table(module_symbol))
    }

    // ── JSX ────────────────────────────────────────────────────────────────

    /// Get all JSX intrinsic tag names.
    /// Go: `(c *Checker) GetJsxIntrinsicTagNamesAt`.
    pub fn get_jsx_intrinsic_tag_names_at(&mut self, location: &Arc<Node>) -> Vec<Arc<Symbol>> {
        // TODO: getJsxType(JsxNames.IntrinsicElements, location).
        let intrinsics = self.get_jsx_type_symbol("IntrinsicElements", location);
        if let Some(intrinsics) = intrinsics {
            return self.get_properties_of_type(&intrinsics);
        }
        Vec::new()
    }

    /// Get the contextual type for a JSX attribute.
    /// Go: `(c *Checker) GetContextualTypeForJsxAttribute`.
    pub fn get_contextual_type_for_jsx_attribute(
        &mut self,
        attribute: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        // TODO: getContextualTypeForJsxAttribute (not yet ported).
        None
    }

    // ── Constant values ────────────────────────────────────────────────────

    /// Get the constant value of an enum member or computed property.
    /// Go: `(c *Checker) GetConstantValue`.
    pub fn get_constant_value_for_services(&mut self, node: &Arc<Node>) -> Option<EvalValue> {
        if node.kind == SyntaxKind::EnumMember {
            return self.get_enum_member_value(node).value;
        }

        // Ensure cached resolved symbol is set.
        // TODO: checkExpressionCached to populate symbolNodeLinks.resolvedSymbol.
        self.check_expression(node);

        let symbol = self
            .symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone());

        if let Some(ref sym) = symbol {
            if sym.flags.contains(SymbolFlags::EnumMember) {
                // Inline property/index accesses only for const enums.
                if let Some(ref member) = sym.value_declaration {
                    if let Some(ref parent) = member.parent {
                        // Check if the parent enum declaration is `const`.
                        if parent.flags.contains(crate::ast::NodeFlags::Const) {
                            return self.get_enum_member_value(member).value;
                        }
                    }
                }
            }
        }

        None
    }

    // ── Signature resolution ────────────────────────────────────────────────

    /// Get the resolved signature for a call-like expression.
    /// Go: `(c *Checker) getResolvedSignatureWorker`.
    pub fn get_resolved_signature_worker(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: CheckMode,
        _argument_count: i32,
    ) -> (Option<Arc<Signature>>, Vec<Arc<Signature>>) {
        // TODO: Requires printer.NewEmitContext().ParseNode and
        // getResolvedSignature (not yet ported).
        (None, Vec::new())
    }

    /// Get candidate signatures for string literal completions.
    /// Go: `(c *Checker) GetCandidateSignaturesForStringLiteralCompletions`.
    pub fn get_candidate_signatures_for_string_literal_completions(
        &mut self,
        _call: &Arc<Node>,
        _editing_argument: &Arc<Node>,
    ) -> Vec<Arc<Signature>> {
        // TODO: Requires runWithInferenceBlockedFromSourceNode and
        // runWithoutResolvedSignatureCaching (not yet ported).
        Vec::new()
    }

    // ── Signature parameter queries ─────────────────────────────────────────

    /// Get the type of a parameter at a given index in a signature.
    /// Go: `(c *Checker) GetTypeAtPosition`.
    ///
    /// Note: delegates to `get_type_at_position` in relater.rs.
    pub fn get_type_at_position_for_services(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        self.get_type_at_position(sig, pos)
    }

    /// Get the type parameter at a given position in a signature.
    /// Go: `(c *Checker) GetTypeParameterAtPosition`.
    pub fn get_type_parameter_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        let t = self.get_type_at_position(sig, pos);
        // If the type is an index type whose target is a `this` type parameter,
        // return the constraint's index type.
        if t.flags.contains(TypeFlags::Index) {
            if let TypeData::Index(index_data) = &t.data {
                if let Some(ref target) = index_data.target {
                    if self.is_this_type_parameter(target) {
                        if let Some(constraint) = self.get_base_constraint_of_type(target) {
                            return self.get_index_type(&constraint);
                        }
                    }
                }
            }
        }
        t
    }

    /// Get the contextual type for an array literal element at a position.
    /// Go: `(c *Checker) GetContextualTypeForArrayLiteralAtPosition`.
    pub fn get_contextual_type_for_array_literal_at_position(
        &mut self,
        contextual_array_type: Option<&Arc<Type>>,
        array_literal: &Arc<Node>,
        position: usize,
    ) -> Option<Arc<Type>> {
        let contextual_array_type = contextual_array_type?;
        let mut first_spread_index = -1i32;
        let mut last_spread_index = -1i32;
        let mut element_index = 0i32;

        let elements: Vec<Arc<Node>> = match &array_literal.data {
            NodeData::ArrayLiteralExpression(data) => data.elements.nodes.clone(),
            _ => return None,
        };

        for (i, elem) in elements.iter().enumerate() {
            if elem.pos() < position {
                element_index += 1;
            }
            if elem.kind == SyntaxKind::SpreadElement {
                if first_spread_index == -1 {
                    first_spread_index = i as i32;
                }
                last_spread_index = i as i32;
            }
        }

        // The array may be incomplete, so we don't know its final length.
        self.get_contextual_type_for_element_expression(
            contextual_array_type,
            element_index as usize,
            None,
            first_spread_index,
            last_spread_index,
        )
    }

    // ── Known generic type queries ──────────────────────────────────────────

    /// Get the first type argument from a known generic type (e.g. `Array<T>`,
    /// `Promise<T>`).
    /// Go: `(c *Checker) GetFirstTypeArgumentFromKnownType`.
    pub fn get_first_type_argument_from_known_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.object_flags.contains(ObjectFlags::Reference) {
            if let Some(ref symbol) = t.symbol {
                if is_known_generic_name(&symbol.name) {
                    // TODO: getGlobalSymbol and compare with target symbol.
                    let type_args = self.get_type_arguments(t);
                    return type_args.into_iter().next();
                }
            }
        }
        if let Some(ref alias) = t.alias {
            if let Some(ref alias_symbol) = alias.symbol {
                if is_known_generic_name(&alias_symbol.name) {
                    // TODO: getGlobalSymbol and compare with alias symbol.
                    return alias.type_arguments.first().cloned();
                }
            }
        }
        None
    }

    // ── Property symbol queries ────────────────────────────────────────────

    /// Get property symbols from a contextual type for a property name node.
    /// Go: `(c *Checker) GetPropertySymbolsFromContextualType`.
    pub fn get_property_symbols_from_contextual_type(
        &mut self,
        node: &Arc<Node>,
        contextual_type: &Arc<Type>,
        union_symbol_ok: bool,
    ) -> Vec<Arc<Symbol>> {
        let name = node.name().map(|n| n.text()).unwrap_or("");
        if name.is_empty() {
            return Vec::new();
        }

        if !contextual_type.flags.contains(TypeFlags::Union) {
            if let Some(symbol) = self.get_property_of_type(contextual_type, name) {
                return vec![symbol];
            }
            return Vec::new();
        }

        let mut filtered_types: Vec<Arc<Type>> = contextual_type
            .types()
            .unwrap_or(&[])
            .iter()
            .cloned()
            .collect();

        // Filter by discriminant for object literals / JSX attributes.
        if let Some(ref parent) = node.parent {
            if parent.kind == SyntaxKind::ObjectLiteralExpression
                || parent.kind == SyntaxKind::JsxAttributes
            {
                filtered_types
                    .retain(|t| !self.is_type_invalid_due_to_union_discriminant(t, parent));
            }
        }

        let mut discriminated_property_symbols: Vec<Arc<Symbol>> = filtered_types
            .iter()
            .filter_map(|t| self.get_property_of_type(t, name))
            .collect();

        let constituent_count = contextual_type.types().map(|t| t.len()).unwrap_or(0);

        if union_symbol_ok
            && (discriminated_property_symbols.is_empty()
                || discriminated_property_symbols.len() == constituent_count)
        {
            if let Some(symbol) = self.get_property_of_type(contextual_type, name) {
                return vec![symbol];
            }
        }

        if filtered_types.is_empty() && discriminated_property_symbols.is_empty() {
            // Bad discriminant — try again without filtering.
            return contextual_type
                .types()
                .unwrap_or(&[])
                .iter()
                .filter_map(|t| self.get_property_of_type(t, name))
                .collect();
        }

        // Eliminate duplicates.
        let mut seen = std::collections::HashSet::new();
        discriminated_property_symbols.retain(|s| seen.insert(s.id()));
        discriminated_property_symbols
    }

    /// Get the property symbol corresponding to a destructuring assignment.
    /// Go: `(c *Checker) GetPropertySymbolOfDestructuringAssignment`.
    pub fn get_property_symbol_of_destructuring_assignment(
        &mut self,
        location: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        // location.Parent.Parent should be a destructuring pattern.
        let parent = location.parent.as_ref()?;
        let grandparent = parent.parent.as_ref()?;

        if is_array_literal_or_object_literal_destructuring_pattern(grandparent) {
            if let Some(type_of_object_literal) = self.get_type_of_assignment_pattern(grandparent) {
                return self.get_property_of_type(&type_of_object_literal, location.text());
            }
        }
        None
    }

    /// Get the type of a destructuring assignment pattern.
    /// Go: `(c *Checker) getTypeOfAssignmentPattern`.
    pub fn get_type_of_assignment_pattern(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> {
        // TODO: Full implementation requires checkRightHandSideOfForOf,
        // checkDestructuringAssignment, checkIteratedTypeOrElementType,
        // checkObjectLiteralDestructuringPropertyAssignment,
        // checkArrayLiteralDestructuringElementAssignment — not yet ported.
        None
    }

    // ── Signature from declaration ──────────────────────────────────────────

    /// Get the signature from a function-like declaration.
    /// Go: `(c *Checker) GetSignatureFromDeclaration`.
    pub fn get_signature_from_declaration(&mut self, _node: &Arc<Node>) -> Option<Arc<Signature>> {
        // TODO: Requires internal getSignatureFromDeclaration (not yet ported).
        None
    }

    // ── Library file checks ────────────────────────────────────────────────

    /// Whether a symbol is declared in a lib file.
    /// Go: `(c *Checker) IsLibSymbolForHoverVerbosity`.
    pub fn is_lib_symbol_for_hover_verbosity(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(sf) = self.get_source_file_of_node(decl) {
                if self.program.is_source_file_default_library(&sf.file_name) {
                    return true;
                }
            }
        }
        false
    }

    /// Whether a type is declared in a lib file.
    /// Go: `(c *Checker) IsLibTypeForHoverVerbosity`.
    pub fn is_lib_type_for_hover_verbosity(&self, t: &Arc<Type>) -> bool {
        let symbol = if t.object_flags.contains(ObjectFlags::Reference) {
            t.target().and_then(|target| target.symbol.clone())
        } else {
            t.symbol.clone()
        };
        if let Some(ref sym) = symbol {
            if self.is_lib_symbol_for_hover_verbosity(sym) {
                return true;
            }
        }
        is_tuple_type(t)
    }

    // ── Internal helpers used by services.go methods ────────────────────────

    /// Resolve an external module symbol (following `export =`).
    /// Go: `resolveExternalModuleSymbol` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn resolve_external_module_symbol(
        &self,
        module_symbol: &Arc<Symbol>,
        _dont_resolve_alias: bool,
    ) -> Arc<Symbol> {
        // Check for `export=` in the module's exports.
        if let Some(export_equals) = module_symbol.exports.get("export=") {
            return Arc::clone(export_equals);
        }
        Arc::clone(module_symbol)
    }

    /// Get the members of a symbol.
    /// Go: `getMembersOfSymbol` (defined in checker.go).
    pub fn get_members_of_symbol(&self, symbol: &Arc<Symbol>) -> SymbolTable {
        symbol.members.clone()
    }

    /// Remove the optional type marker from a type.
    /// Go: `removeOptionalTypeMarker` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn remove_optional_type_marker(&self, t: &Arc<Type>) -> Arc<Type> {
        // The optional marker is stored in StructuredTypeData/ObjectFlags.
        // For now, return the type as-is.
        Arc::clone(t)
    }

    /// Get the index type of a type (string or number index signature).
    /// Go: `getIndexTypeOfType` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn get_index_type_of_type(
        &self,
        t: &Arc<Type>,
        _index_kind: IndexKind,
    ) -> Option<Arc<Type>> {
        if let Some(structured) = t.as_structured() {
            for info in &structured.index_infos {
                if let Some(ref key_type) = info.key_type {
                    let matches = match _index_kind {
                        IndexKind::String => key_type.flags.contains(TypeFlags::String),
                        IndexKind::Number => key_type.flags.contains(TypeFlags::Number),
                    };
                    if matches {
                        return info.value_type.clone();
                    }
                }
            }
        }
        None
    }

    /// Get the apparent type of a type.
    /// Go: `getApparentType` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn get_apparent_type(&self, t: &Arc<Type>) -> Arc<Type> {
        // For primitive types, the apparent type is the corresponding
        // global interface (e.g. `string` → `String`). For object types,
        // the apparent type is the type itself.
        // TODO: Resolve primitive apparent types from lib.d.ts globals.
        Arc::clone(t)
    }

    /// Get the reduced apparent type of a type.
    /// Go: `getReducedApparentType` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn get_reduced_apparent_type(&self, t: &Arc<Type>) -> Arc<Type> {
        self.get_apparent_type(t)
    }

    /// Resolve structured type members.
    /// Go: `resolveStructuredTypeMembers` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn resolve_structured_type_members(&self, t: &Arc<Type>) -> Arc<Type> {
        Arc::clone(t)
    }

    /// Whether a symbol/name pair is a named member.
    /// Go: `isNamedMember` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn is_named_member(&self, _symbol: &Arc<Symbol>, _name: &str) -> bool {
        !is_reserved_member_name(_name)
    }

    /// Get named members from a symbol map.
    /// Go: `getNamedMembers` (defined in checker.go).
    pub fn get_named_members(
        &self,
        props_by_name: &std::collections::HashMap<String, Arc<Symbol>>,
    ) -> Vec<Arc<Symbol>> {
        props_by_name
            .values()
            .filter(|s| !is_reserved_member_name(&s.name))
            .cloned()
            .collect()
    }

    /// Check whether a property is accessible from a location.
    /// Go: `isPropertyAccessible` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn is_property_accessible(
        &self,
        _node: &Arc<Node>,
        _is_super: bool,
        _is_write: bool,
        _t: &Arc<Type>,
        _property: &Arc<Symbol>,
    ) -> bool {
        true
    }

    /// Get the widened type of an expression.
    /// Go: `getWidenedType` applied to expression checking.
    fn get_widened_type_of_expression(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        let t = self.get_type_of_node(expr);
        self.get_widened_type(&t)
    }

    /// Get the type of a property of a type.
    /// Go: `getTypeOfPropertyOfType` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn get_type_of_property_of_type(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Type>> {
        if let Some(prop) = self.get_property_of_type(t, name) {
            return Some(self.get_type_of_symbol(&prop));
        }
        None
    }

    /// Get the literal type from a property name node.
    /// Go: `getLiteralTypeFromPropertyName` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn get_literal_type_from_property_name(
        &mut self,
        property_name: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        match property_name.kind {
            SyntaxKind::StringLiteral => Some(self.get_string_literal_type(property_name.text())),
            SyntaxKind::NumericLiteral => {
                // TODO: getLiteralTypeForNumberLiteral.
                None
            }
            SyntaxKind::PrivateIdentifier => {
                // TODO: getUniqueSymbolType.
                None
            }
            SyntaxKind::ComputedPropertyName => {
                // TODO: resolve computed property name.
                None
            }
            _ => None,
        }
    }

    /// Whether a type parameter is a `this` type parameter.
    /// Go: `isThisTypeParameter` (defined in utilities.go).
    pub fn is_this_type_parameter(&self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::TypeParameter) {
            return false;
        }
        if let TypeData::TypeParameter(tp) = &t.data {
            tp.is_this_type
        } else {
            false
        }
    }

    /// Get the contextual type for an element expression.
    /// Go: `getContextualTypeForElementExpression` (defined in checker.go).
    /// TODO: Full implementation not yet ported.
    pub fn get_contextual_type_for_element_expression(
        &mut self,
        _contextual_type: &Arc<Type>,
        _element_index: usize,
        _length: Option<usize>,
        _first_spread_index: i32,
        _last_spread_index: i32,
    ) -> Option<Arc<Type>> {
        None
    }

    /// Get the global callable function type.
    /// Go: `c.globalCallableFunctionType`.
    /// TODO: Full implementation not yet ported.
    fn global_callable_function_type(&self) -> Option<Arc<Type>> {
        // TODO: Resolve from lib.d.ts `CallableFunction` interface.
        None
    }

    /// Get the global newable function type.
    /// Go: `c.globalNewableFunctionType`.
    /// TODO: Full implementation not yet ported.
    fn global_newable_function_type(&self) -> Option<Arc<Type>> {
        // TODO: Resolve from lib.d.ts `NewableFunction` interface.
        None
    }

    /// Get the JSX type symbol for a name.
    /// Go: `getJsxType` (defined in jsx.go).
    /// TODO: Full implementation not yet ported.
    fn get_jsx_type_symbol(&self, _name: &str, _location: &Arc<Node>) -> Option<Arc<Type>> {
        None
    }

    /// Get the source file of a node.
    /// Go: `ast.GetSourceFileOfNode`.
    pub fn get_source_file_of_node(&self, node: &Arc<Node>) -> Option<Arc<SourceFile>> {
        let mut current = Arc::clone(node);
        loop {
            if current.kind == SyntaxKind::SourceFile {
                // Find the matching SourceFile in our files list.
                let node_id = current.id();
                for file in &self.files {
                    if file.node.id() == node_id {
                        return Some(Arc::clone(file));
                    }
                }
                return None;
            }
            current = current.parent.clone()?;
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Free functions
// ────────────────────────────────────────────────────────────────────────────

/// Get possible symbol reference nodes in a source file.
/// Go: `getPossibleSymbolReferenceNodes`.
fn get_possible_symbol_reference_nodes(
    source_file: &Arc<SourceFile>,
    symbol_name: &str,
    container: Option<&Arc<Node>>,
) -> Vec<Arc<Node>> {
    let positions = get_possible_symbol_reference_positions(source_file, symbol_name, container);
    let mut result = Vec::new();
    for pos in positions {
        // TODO: astnav.GetTouchingPropertyName — not yet ported.
        // For now, find the identifier at the position by scanning the AST.
        if let Some(node) = find_identifier_at_pos(source_file, pos) {
            result.push(node);
        }
    }
    result
}

/// Get possible symbol reference positions in a source file.
/// Go: `getPossibleSymbolReferencePositions`.
fn get_possible_symbol_reference_positions(
    source_file: &Arc<SourceFile>,
    symbol_name: &str,
    container: Option<&Arc<Node>>,
) -> Vec<usize> {
    let mut positions = Vec::new();

    // Be resilient in the face of a symbol with no name or zero length name.
    if symbol_name.is_empty() {
        return positions;
    }

    let text = source_file.text.as_str();
    let symbol_name_len = symbol_name.len();

    let search_start = container.and_then(|c| Some(c.pos())).unwrap_or(0);
    let end_pos = container.and_then(|c| Some(c.end())).unwrap_or(text.len());

    let mut search_from = search_start;
    while search_from < end_pos {
        let remainder = &text[search_from..end_pos];
        let relative_pos = match remainder.find(symbol_name) {
            Some(p) => p,
            None => break,
        };
        let position = search_from + relative_pos;
        let end_position = position + symbol_name_len;

        // Check that the match is not part of a larger word.
        let prev_ok = position == 0 || !is_identifier_part_byte(text.as_bytes()[position - 1]);
        let next_ok =
            end_position >= text.len() || !is_identifier_part_byte(text.as_bytes()[end_position]);

        if prev_ok && next_ok {
            positions.push(position);
        }

        search_from = position + symbol_name_len + 1;
        if search_from > text.len() {
            break;
        }
    }

    positions
}

/// Whether a byte is an identifier-part character (simplified).
/// Mirrors Go's `scanner.IsIdentifierPart` for ASCII.
fn is_identifier_part_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'$' || b == b'_'
}

/// Find an identifier node at a given position (simplified AST walk).
/// TODO: Replace with proper `astnav.GetTouchingPropertyName`.
fn find_identifier_at_pos(source_file: &Arc<SourceFile>, pos: usize) -> Option<Arc<Node>> {
    // Walk top-level statements and their children to find an identifier
    // whose position matches. This is a simplified version.
    let file_node = &source_file.node;
    find_node_at_pos(file_node, pos)
}

/// Recursively search for an identifier at a position.
fn find_node_at_pos(node: &Arc<Node>, pos: usize) -> Option<Arc<Node>> {
    if node.pos() <= pos && pos < node.end() {
        if node.kind == SyntaxKind::Identifier && node.pos() == pos {
            return Some(Arc::clone(node));
        }
        // Recurse into children using for_each_child.
        let mut found = None;
        crate::ast::for_each_child(node, |child| {
            if found.is_none() {
                if let Some(f) = find_node_at_pos(child, pos) {
                    found = Some(Arc::clone(&f));
                    return true; // stop iteration
                }
            }
            false
        });
        return found;
    }
    None
}

/// Whether a node is an array literal or object literal destructuring pattern.
/// Go: `ast.IsArrayLiteralOrObjectLiteralDestructuringPattern`.
fn is_array_literal_or_object_literal_destructuring_pattern(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
    ) && node
        .parent
        .as_ref()
        .map(|p| {
            p.kind == SyntaxKind::BinaryExpression
                || p.kind == SyntaxKind::ForOfStatement
                || p.kind == SyntaxKind::VariableDeclaration
        })
        .unwrap_or(false)
}

// ────────────────────────────────────────────────────────────────────────────
// IndexKind enum (for getIndexTypeOfType)
// ────────────────────────────────────────────────────────────────────────────

/// The kind of index signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    String,
    Number,
}
