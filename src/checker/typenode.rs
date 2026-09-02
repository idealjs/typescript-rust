//! Type node resolution: converting AST type nodes to `Type` objects.
//!
//! Ported from `getTypeFromTypeNode` and related functions in
//! `internal/checker/checker.go` (lines ~22713–22910).
//!
//! This is the primary entry point for converting a type annotation in the
//! AST (e.g. `string`, `number[]`, `Array<T>`, `A | B`, `keyof T`) into the
//! corresponding `Type` used by the checker.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::ast::node_data_generated::NodeData;
use crate::ast::{
    CheckFlags, ModifierFlags, ModifierList, Node, NodeList, Symbol, SymbolFlags, SymbolTable,
    SyntaxKind,
};
use crate::jsnum;

use super::checker::Checker;
use super::types::*;

/// Go `indexTypeLessThan` (checker.go ~L27452): every union constituent is
/// a numeric literal with `0 <= value < limit` — such an index into a tuple
/// resolves eagerly to a fixed element instead of deferring.
fn index_type_less_than_fixed(index_type: &Arc<Type>, limit: usize) -> bool {
    let constituents: Vec<Arc<Type>> = if index_type.flags.contains(TypeFlags::Union) {
        index_type
            .types()
            .map(|ts| ts.to_vec())
            .unwrap_or_default()
    } else {
        vec![Arc::clone(index_type)]
    };
    if constituents.is_empty() {
        return false;
    }
    constituents.iter().all(|c| {
        if let Some(LiteralValue::Number(n)) = c.literal_value() {
            let text = n.to_string();
            if let Ok(index) = text.parse::<f64>() {
                return index >= 0.0 && index < limit as f64;
            }
        }
        false
    })
}

/// Whether the given modifier list contains the `static` modifier.
/// Used to skip static members when building a class's instance type for
/// `implements` checks. Mirrors `ast.HasStaticModifier` / the
/// `ModifierFlagsStatic` test in Go's `utilities.go`.
fn is_static_modifier(modifiers: &Option<Arc<ModifierList>>) -> bool {
    modifiers
        .as_ref()
        .map(|m| m.modifier_flags.contains(ModifierFlags::Static))
        .unwrap_or(false)
}

/// Extract the cooked text of a template token (`TemplateHead`,
/// `TemplateMiddle`, `TemplateTail`). These tokens carry a `text` field
/// in their node data (the cooked form, with escapes resolved).
fn template_token_text(node: &Arc<Node>) -> String {
    match &node.data {
        NodeData::TemplateHead(d) => d.text.clone(),
        NodeData::TemplateMiddle(d) => d.text.clone(),
        NodeData::TemplateTail(d) => d.text.clone(),
        _ => String::new(),
    }
}

impl Checker {
    /// Convert an AST type node into a `Type`.
    ///
    /// Mirrors `Checker.getTypeFromTypeNode` in Go. This is the public
    /// entry point (from `exports.go`).
    pub fn get_type_from_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        // Memoisation keyed by (node, substitution stack) — Go's per-mapper
        // instantiation cache (checker.go ~L22200). The worker's own per-node
        // cache (`get_cached_type`) is bypassed while a substitution is
        // active; including the stack hash in the key keeps generic bodies
        // cached across repeated evaluations, which is what prevents the
        // exponential re-resolution blow-up on nested conditional/mapped
        // types (deeplyNestedConditionalTypes aborted on OOM before this).
        let key = (node.id() as usize, self.type_argument_stack_hash());
        if let Some(t) = self.type_node_subst_cache.get(&key) {
            return Arc::clone(t);
        }
        // Heritage-degradation epoch (see `heritage_degraded_events`): a
        // resolution whose subtree passed through a degraded interface
        // resolution must not be memoised — the degraded form would be
        // re-served forever, preventing convergence against the complete
        // bases once they finish resolving (the r9 timeout storm).
        let degraded_epoch = self.heritage_degraded_events;
        // Cycle detection (Go `pushTypeResolution`, checker.go ~L18817): a
        // node re-entered under the same substitution is a circular type
        // reference — the inner query yields the error type while the outer
        // frames complete (Go caches error results for circular references
        // too, e.g. TS2456 handling).
        if !self.type_node_resolving.insert(key) {
            return self.error_type();
        }
        self.type_node_query_epochs.push(degraded_epoch);
        // Depth guard (Go `instantiateType` limit, checker.go ~L22170:
        // depth 100 or 5M instantiations → TS2589 + error type). Our
        // counter conflates lexical node-nesting with instantiation depth
        // (a legal 100-level nested conditional costs ~2-3 frames per
        // level, deeplyNestedConditionalTypes expects no error), so the
        // limit is 500 here; runaway work is bounded by the 5M budget and
        // the (node, stack) cycle detection below, and workers run on
        // large stacks.
        let over_budget = !self.type_argument_stack.is_empty() && {
            self.type_instantiation_count += 1;
            self.type_instantiation_count >= 5_000_000
        };
        let result = if self.type_resolution_depth >= 500 || over_budget {
            if !self.type_instantiation_limit_reported {
                self.type_instantiation_limit_reported = true;
                let file = self.current_file.clone();
                let loc = self
                    .current_node
                    .as_ref()
                    .map(|n| n.loc)
                    .unwrap_or(node.loc);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    loc,
                    crate::diagnostics::messages_generated::
                        TYPE_INSTANTIATION_IS_EXCESSIVELY_DEEP_AND_POSSIBLY_INFINITE,
                    Vec::new(),
                ));
            }
            self.error_type()
        } else {
            self.type_resolution_depth += 1;
            let r = self.get_type_from_type_node_worker(node);
            self.type_resolution_depth -= 1;
            r
        };
        self.type_node_resolving.remove(&key);
        self.type_node_query_epochs.pop();
        if self.heritage_degraded_events == degraded_epoch {
            // Capacity bound (see `type_node_subst_cache_limit`): the key
            // embeds the substitution-stack hash, whose combination count
            // grows with nested generic instantiations. The cache is a
            // pure function cache, so clearing it wholesale at the cap is
            // correctness-preserving while keeping resident memory
            // bounded (the pre-cap OOM: deeplyNestedConditionalTypes-style
            // programs left millions of entries live for the Checker's
            // whole lifetime).
            if self.type_node_subst_cache.len() >= self.type_node_subst_cache_limit {
                self.type_node_subst_cache.clear();
            }
            self.type_node_subst_cache.insert(key, Arc::clone(&result));
        }
        result
    }

    /// Hash the current `type_argument_stack` substitution context: each
    /// frame contributes its (symbol, type) pointer pairs in sorted order so
    /// equal stacks hash equal. Frames are hashed bottom-up (storage order),
    /// matching the stack's LIFO push/pop discipline.
    fn type_argument_stack_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        if self.type_argument_stack.is_empty() {
            return 0;
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for map in &self.type_argument_stack {
            let mut entries: Vec<(usize, usize)> = map
                .iter()
                .map(|(k, v)| (*k as usize, Arc::as_ptr(v) as usize))
                .collect();
            entries.sort_unstable();
            entries.len().hash(&mut h);
            for e in entries {
                e.hash(&mut h);
            }
        }
        h.finish()
    }

    /// Worker for `get_type_from_type_node`.
    fn get_type_from_type_node_worker(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match node.kind {
            SyntaxKind::AnyKeyword | SyntaxKind::JSDocAllType => self.any_type(),
            SyntaxKind::JSDocNonNullableType => {
                let inner = node
                    .type_node()
                    .expect("JSDocNonNullableType has type")
                    .clone();
                self.get_type_from_type_node(&inner)
            }
            SyntaxKind::JSDocNullableType => {
                let inner = node
                    .type_node()
                    .expect("JSDocNullableType has type")
                    .clone();
                let t = self.get_type_from_type_node(&inner);
                if self.strict_null_checks {
                    self.get_nullable_type(&t, TypeFlags::Null)
                } else {
                    t
                }
            }
            SyntaxKind::JSDocVariadicType => {
                let inner = node
                    .type_node()
                    .expect("JSDocVariadicType has type")
                    .clone();
                let elem_type = self.get_type_from_type_node(&inner);
                self.create_array_type(elem_type)
            }
            SyntaxKind::JSDocOptionalType => {
                let inner = node
                    .type_node()
                    .expect("JSDocOptionalType has type")
                    .clone();
                let t = self.get_type_from_type_node(&inner);
                self.add_optionality(&t)
            }
            SyntaxKind::UnknownKeyword => self.unknown_type(),
            SyntaxKind::StringKeyword => self.string_type(),
            SyntaxKind::NumberKeyword => self.number_type(),
            SyntaxKind::BigIntKeyword => self.bigint_type(),
            SyntaxKind::BooleanKeyword => self.boolean_type(),
            SyntaxKind::SymbolKeyword => self.es_symbol_type(),
            SyntaxKind::VoidKeyword => self.void_type(),
            SyntaxKind::UndefinedKeyword => self.undefined_type(),
            SyntaxKind::NullKeyword => self.null_type(),
            SyntaxKind::NeverKeyword => self.never_type(),
            SyntaxKind::ObjectKeyword => self.non_primitive_type(),
            // `const` keyword type node from `as const` — should only appear
            // as the type operand of an `AsExpression`, handled there.
            // If reached directly, return any (no false errors).
            SyntaxKind::ConstKeyword => self.any_type(),
            SyntaxKind::ThisType | SyntaxKind::ThisKeyword => {
                self.get_type_from_this_type_node(node)
            }
            SyntaxKind::LiteralType => self.get_type_from_literal_type_node(node),
            SyntaxKind::TypeReference | SyntaxKind::ExpressionWithTypeArguments => {
                self.get_type_from_type_reference(node)
            }
            SyntaxKind::TypePredicate => {
                if let NodeData::TypePredicateNode(data) = &node.data {
                    if data.asserts_modifier.is_some() {
                        return self.void_type();
                    }
                }
                self.boolean_type()
            }
            SyntaxKind::TypeQuery => self.get_type_from_type_query_node(node),
            SyntaxKind::ArrayType | SyntaxKind::TupleType => {
                self.get_type_from_array_or_tuple_type_node(node)
            }
            SyntaxKind::OptionalType => self.get_type_from_optional_type_node(node),
            SyntaxKind::UnionType => self.get_type_from_union_type_node(node),
            SyntaxKind::IntersectionType => self.get_type_from_intersection_type_node(node),
            SyntaxKind::NamedTupleMember => self.get_type_from_named_tuple_type_node(node),
            SyntaxKind::ParenthesizedType => {
                let inner = node
                    .type_node()
                    .expect("ParenthesizedType has type")
                    .clone();
                self.get_type_from_type_node(&inner)
            }
            SyntaxKind::RestType => self.get_type_from_rest_type_node(node),
            SyntaxKind::FunctionType | SyntaxKind::ConstructorType | SyntaxKind::TypeLiteral => {
                self.get_type_from_type_literal_or_function_or_constructor_type_node(node)
            }
            SyntaxKind::TypeOperator => self.get_type_from_type_operator_node(node),
            SyntaxKind::IndexedAccessType => self.get_type_from_indexed_access_type_node(node),
            SyntaxKind::TemplateLiteralType => self.get_type_from_template_type_node(node),
            SyntaxKind::MappedType => self.get_type_from_mapped_type_node(node),
            SyntaxKind::ConditionalType => self.get_type_from_conditional_type_node(node),
            SyntaxKind::InferType => self.get_type_from_infer_type_node(node),
            SyntaxKind::ImportType => self.get_type_from_import_type_node(node),
            _ => self.error_type(),
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Cached resolution helpers
    //
    // These use a check-then-compute-then-store pattern to avoid borrow
    // conflicts: we first check the cache with an immutable borrow, then
    // compute the type (which may need &mut self), then store it back.
    // ────────────────────────────────────────────────────────────────────────

    /// Check if a type node has already been resolved.
    ///
    /// When a type-argument substitution is active (the
    /// `type_argument_stack` is non-empty), the resolved type depends on
    /// the substitution context (e.g. the mapped-type key `K` being
    /// substituted with `"a"` vs `"b"`), so the per-node cache must be
    /// bypassed to avoid returning a result computed under a different
    /// substitution.
    fn get_cached_type(&self, node: &Arc<Node>) -> Option<Arc<Type>> {
        if !self.type_argument_stack.is_empty() {
            return None;
        }
        self.type_node_links
            .get(node)
            .and_then(|l| l.resolved_type.clone())
    }

    /// Store a resolved type for a type node.
    ///
    /// See `get_cached_type`: caching is skipped while a type-argument
    /// substitution is active, and while the enclosing `get_type_from_
    /// type_node` query passed through a heritage degradation (the
    /// errorType an interface cycle guard returns would otherwise be
    /// pinned here and re-served to every later query, preventing
    /// convergence — see `heritage_degraded_events`).
    fn cache_type(&mut self, node: &Arc<Node>, t: Arc<Type>) {
        if !self.type_argument_stack.is_empty() {
            return;
        }
        if let Some(epoch) = self.type_node_query_epochs.last()
            && *epoch != self.heritage_degraded_events
        {
            return;
        }
        self.type_node_links.get_or_default(node).resolved_type = Some(t);
    }

    // ────────────────────────────────────────────────────────────────────────
    // Individual type node resolvers
    // ────────────────────────────────────────────────────────────────────────

    fn get_type_from_this_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_literal_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let literal = match &node.data {
            NodeData::LiteralTypeNode(data) => &data.literal,
            _ => return self.error_type(),
        };
        if literal.kind == SyntaxKind::NullKeyword {
            return self.null_type();
        }
        let result = self.literal_type_from_literal_node(literal);
        self.cache_type(node, result.clone());
        result
    }

    fn literal_type_from_literal_node(&mut self, literal: &Arc<Node>) -> Arc<Type> {
        match literal.kind {
            SyntaxKind::StringLiteral => self.get_string_literal_type(literal.text()),
            SyntaxKind::NumericLiteral => {
                if let Ok(n) = literal.text().parse::<f64>() {
                    self.get_number_literal_type(crate::jsnum::Number::from(n))
                } else {
                    self.number_type()
                }
            }
            SyntaxKind::BigIntLiteral => {
                let text = literal.text();
                if let Some(t) = self.bigint_literal_types.get(text).cloned() {
                    return t;
                }
                let (neg, digits) = if let Some(rest) = text.strip_prefix('-') {
                    (true, rest.trim_end_matches('n'))
                } else {
                    (false, text.trim_end_matches('n'))
                };
                let t = Arc::new(Type::new(
                    TypeFlags::BigIntLiteral,
                    TypeData::Literal(LiteralTypeData {
                        value: LiteralValue::BigInt(crate::jsnum::PseudoBigInt::new(digits, neg)),
                        fresh_type: std::sync::OnceLock::new(),
                        regular_type: std::sync::OnceLock::new(),
                    }),
                ));
                self.bigint_literal_types
                    .insert(text.to_string(), Arc::clone(&t));
                t
            }
            SyntaxKind::TrueKeyword => self.true_type(),
            SyntaxKind::FalseKeyword => self.false_type(),
            _ => self.error_type(),
        }
    }

    fn get_type_from_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_reference(node);
        self.cache_type(node, result.clone());
        result
    }

    /// Resolve a `TypeReference` (`Foo`, `Foo<T>`) or
    /// `ExpressionWithTypeArguments` to its `Type`.
    ///
    /// Currently resolves type aliases (`type Foo = ...`) by recursively
    /// resolving the alias's declared type. Interface/class/enum references
    /// fall back to `error_type` (= `any`) so no false positives are produced
    /// until their object-type construction is ported.
    fn resolve_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(data) => (&data.type_name, data.type_arguments.clone()),
            NodeData::ExpressionWithTypeArguments(data) => {
                (&data.expression, data.type_arguments.clone())
            }
            _ => return self.error_type(),
        };
        // Qualified names (`A.B`) resolve through module/namespace exports
        // (Go's resolveEntityName); identifiers use the scope stack.
        // `intrinsic` is TypeScript's special keyword type (the alias bodies
        // of `Uppercase`/`Lowercase`/… in lib.es5.d.ts) — never resolved as
        // a name and never reported.
        if type_name.kind == SyntaxKind::Identifier && type_name.text() == "intrinsic" {
            return self.error_type();
        }
        let mut symbol = if type_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier(type_name) {
                Some(s) => s,
                None => {
                    // Report TS2304 "Cannot find name '{0}'." for unresolved type
                    // references. Mirrors Go's NameResolver which is called with
                    // `nameNotFoundMessage = Cannot_find_name_0` for type nodes.
                    //
                    // Suppressed while building interface call/construct
                    // signatures (see `suppress_cannot_find_name_in_type_nodes`),
                    // where lib.d.ts signatures may reference signature-level type
                    // parameters the binder has no symbol for.
                    if self.ts2304_reporting_allowed_for(type_name) {
                        use crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                        let name_text = type_name.text();
                        // Attribute to the reference node's own file — the
                        // resolution may be triggered from a foreign file's
                        // processing (cross-file global lookup).
                        let file = self
                            .get_source_file_of_node(type_name)
                            .or_else(|| self.current_file.clone());
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            CANNOT_FIND_NAME_0,
                            vec![name_text.to_string()],
                        ));
                    }
                    return self.error_type();
                }
            }
        } else if matches!(
            type_name.kind,
            SyntaxKind::Identifier
                | SyntaxKind::QualifiedName
                // Heritage/type-position qualified names parse in
                // expression form (`interface I extends NS.Base {}`).
                | SyntaxKind::PropertyAccessExpression
        ) {
            match self.resolve_qualified_symbol_traced(type_name) {
                Ok(s) => s,
                Err((segment, ns_path, member)) => {
                    // TS2694: the namespace resolved but lacks the member;
                    // TS2503: a namespace segment itself didn't resolve.
                    // The PropertyAccessExpression (heritage expression)
                    // form stays SILENT on failure — non-entity bases
                    // (`(foo()).B`) are legitimate and typed elsewhere, and
                    // the entity form's failures degrade like the
                    // pre-expression-form path (error type, no report).
                    // Suppressed for BUNDLED-lib nodes (attributed by the
                    // node's own file, not the current file — a lib
                    // interface's heritage may be resolved while checking a
                    // user file): lib-internal globalThis/qualified
                    // references that our global-merge can't see must stay
                    // silent, like the pre-PropertyAccess path.
                    let attributed_file = self
                        .get_source_file_of_node(type_name)
                        .or_else(|| self.current_file.clone());
                    let reportable = type_name.kind == SyntaxKind::QualifiedName;
                    if reportable
                        && self.ts2304_reporting_allowed_for(type_name)
                        && attributed_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = attributed_file;
                        if ns_path.is_empty() {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                segment.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_FIND_NAMESPACE_0,
                                vec![segment.text().to_string()],
                            ));
                        } else {
                            // Go resolveQualifiedName resolves the LEFT side
                            // with NAMESPACE meaning only — a leftmost name
                            // bound to a non-namespace symbol (a variable)
                            // fails there and maps through
                            // onFailedToResolveSymbol: spelling suggestion →
                            // TS2833, type-meaningful → TS2702, else TS2503
                            // — never a member-level TS2694.
                            let leftmost = super::checker::base_identifier_of(type_name);
                            let left_hit = self
                                .resolve_identifier(&leftmost)
                                .map(|s| self.resolve_alias_base(s));
                            let left_non_namespace = left_hit
                                .as_ref()
                                .is_some_and(|b| !b.flags.intersects(SymbolFlags::NAMESPACE));
                            if left_non_namespace {
                                let name_text = leftmost.text().to_string();
                                if let Some(sugg) =
                                    self.find_name_suggestion(&name_text, SymbolFlags::NAMESPACE)
                                {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1,
                                        vec![name_text.clone(), sugg],
                                    ));
                                } else if left_hit
                                    .as_ref()
                                    .is_some_and(|b| b.flags.intersects(SymbolFlags::TYPE))
                                {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                        vec![name_text],
                                    ));
                                } else {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            CANNOT_FIND_NAMESPACE_0,
                                        vec![name_text],
                                    ));
                                }
                            } else {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    segment.loc,
                                    crate::diagnostics::messages_generated::
                                        NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                    vec![ns_path, member],
                                ));
                            }
                        }
                    }
                    return self.error_type();
                }
            }
        } else {
            return self.error_type();
        };
        // Import aliases (`import D from "m"`, `import { T } from "m"`):
        // the TYPE meaning comes from the alias's TARGET symbol — follow
        // the import to the exported member and resolve that. A value-only
        // target (a const/function default import used as a type) reports
        // TS2749 under the ALIAS's own name
        // (allowImportClausesToMergeWithTypes: `const y: originalZZZ`).
        if symbol.flags == SymbolFlags::Alias {
            let alias_name = type_name.text().to_string();
            if let Some(target) = self.resolve_import_alias_target_symbol(&symbol) {
                let target_has_type_meaning = target.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::ENUM
                        | SymbolFlags::TypeParameter,
                );
                if target_has_type_meaning {
                    symbol = target;
                } else {
                    if type_name.kind == SyntaxKind::Identifier
                        && self.ts2304_reporting_allowed_for(type_name)
                        && !self.has_same_named_type_symbol(&alias_name)
                        && self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                            vec![alias_name.clone(), alias_name],
                        ));
                    }
                    return self.error_type();
                }
            }
        }
        // TS2314 / TS2344: type-reference arity + constraint checks (Go's
        // checkTypeArguments at resolveTypeReference time). Purely
        // syntactic on the declared type-parameter list — no instantiation
        // needed. Skipped for the bundled lib (its generic instantiations
        // resolve imperfectly in this port and would mis-fire).
        if !self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"))
            && symbol
                .flags
                .intersects(SymbolFlags::Interface | SymbolFlags::Class | SymbolFlags::TypeAlias)
            && !type_name_inside_conditional_branch(type_name)
            && !type_name_shadowed_by_type_parameter(type_name)
            && !self.check_type_reference_arguments(node, type_name, &symbol)
        {
            // Arity error already reported (TS2314); the reference resolves
            // to the error type so dependent checks see any/error and skip
            // (Go's `return c.errorType` in getTypeFromClassOrInterface
            // Reference) — e.g. TS2564's property-type exemption.
            return self.error_type();
        }
        // Type parameter: build a TypeParameter type with the constraint
        // resolved from the declaration (`<T extends Constraint>`).
        if symbol.flags.contains(SymbolFlags::TypeParameter) {
            // Static members cannot reference class type parameters
            // (TS2322). Mirrors Go's NameResolver check for
            // `ast.IsStatic(lastLocation)` when resolving a type parameter
            // declared in a class container.
            if self.in_static_member_type {
                // Only type parameters declared on an enclosing CLASS are
                // restricted — a FUNCTION-scoped type parameter (a class
                // expression inside `function F<T>()`) is visible to
                // statics too (declarationNoDanglingGenerics).
                let tp_decl = symbol.value_declaration.clone().or_else(|| {
                    symbol.declarations.first().cloned()
                });
                let owned_by_class = tp_decl.is_some_and(|d| {
                    let mut cur = d.parent.as_ref();
                    while let Some(a) = cur {
                        match a.kind {
                            crate::ast::SyntaxKind::ClassDeclaration
                            | crate::ast::SyntaxKind::ClassExpression => return true,
                            crate::ast::SyntaxKind::SourceFile => return false,
                            _ => cur = a.parent.as_ref(),
                        }
                    }
                    false
                });
                if owned_by_class {
                    use crate::diagnostics::messages_generated::STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS;
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        type_name.loc,
                        STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                        Vec::new(),
                    ));
                }
            }
            // Check the type-argument substitution stack first — if this
            // type parameter is being instantiated with a concrete type,
            // return the substitution instead of the TypeParameter type.
            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
            for map in self.type_argument_stack.iter().rev() {
                if let Some(t) = map.get(&key) {
                    return Arc::clone(t);
                }
            }
            // Heritage-instantiation frames carry the OWNER's type-parameter
            // SYMBOLS alongside the substituted types: match by pointer
            // first; a NAME match is only accepted when both symbols share
            // the same declaring container (multi-declaration forks of one
            // generic interface bind one symbol per declaration). A bare
            // name match wrongly instantiated a METHOD's own same-named
            // type parameter with the heritage argument (D3-sig:
            // `class D extends Base<number>` made `Base`'s `make<T>(x: T)`
            // take number).
            for frame in self.type_argument_name_frames.iter().rev() {
                for (frame_sym, t) in frame.iter().rev() {
                    if Arc::ptr_eq(frame_sym, &symbol)
                        || (frame_sym.name == symbol.name
                            && self.type_param_symbols_share_container(frame_sym, &symbol))
                    {
                        return Arc::clone(t);
                    }
                }
            }
            return self.get_type_parameter_from_symbol(&symbol);
        }
        if symbol.flags.contains(SymbolFlags::Interface) {
            // Interface: build an anonymous object type from the interface's
            // members (PropertySignature, MethodSignature, IndexSignature).
            return self.resolve_interface_type(&symbol, type_arguments);
        }
        if symbol.flags.intersects(SymbolFlags::ENUM) {
            // Enum: build a union of all enum member literal types.
            return self.resolve_enum_type(&symbol);
        }
        if symbol.flags.contains(SymbolFlags::Class) {
            // Class annotation (`x: MyClass`): the instance type built from
            // the class's members (including `extends` bases). Memoized on
            // the class symbol; guarded against self-referential hierarchies
            // (`class A extends A`). For a class MERGED with a namespace
            // (`class N {} namespace N {}`), the shared declared-type slot
            // also receives namespace/merged VALUE-side types (constructor +
            // exports) — bypass it so the annotation keeps getting the
            // instance type, like Go's distinct class-instance vs
            // getDeclaredTypeOfSymbol types.
            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
            let merged_with_ns = symbol.flags.contains(SymbolFlags::ValueModule);
            if !merged_with_ns {
                if let Some(cached) = self
                    .type_alias_links
                    .get(&symbol)
                    .and_then(|l| l.declared_type.clone())
                {
                    return cached;
                }
            }
            if !self.resolving_type_aliases.insert(key) {
                return self.error_type();
            }
            let class_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                .cloned();
            let instance_type = match class_node {
                Some(node) => self.build_class_instance_type_with_base(&node),
                None => self.error_type(),
            };
            self.resolving_type_aliases.remove(&key);
            if !merged_with_ns {
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&instance_type));
            }
            // A generic class referenced WITH type arguments (`B<number>`)
            // carries the instantiation on the instance type — member reads
            // substitute through them (`substituted_member_type_of`).
            let arg_types: Option<Vec<Arc<Type>>> = type_arguments.map(|nodes| {
                nodes
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect()
            });
            if let Some(arg_types) = arg_types {
                let tps = self.declared_type_parameter_types(&symbol);
                if !tps.is_empty() && tps.len() == arg_types.len() {
                    return self.attach_explicit_type_arguments_cached(&instance_type, arg_types);
                }
            }
            return instance_type;
        }
        if !symbol.flags.contains(SymbolFlags::TypeAlias) {
            // A pure VALUE symbol (a variable/value import) in a type
            // position is TS2749 (suggest `typeof`). Reported for
            // identifier names; suppressed in signature-building contexts
            // Class-expression heritage (`class y extends x` where x is a
            // variable holding a class — extendClassExpressionFromModule):
            // the heritage expression is a VALUE reference; its type's
            // construct signatures provide the instance type (Go resolves
            // the heritage node as an expression, never as a type
            // reference). A VALUE symbol here is legitimate, not TS2749.
            if matches!(&node.data, NodeData::ExpressionWithTypeArguments(_))
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::TypeParameter,
                )
            {
                let value_type = self.get_type_of_symbol(&symbol);
                if let Some(structured) = value_type.as_structured() {
                    for sig in structured.construct_signatures() {
                        if let Some(ret) = self.get_return_type_of_signature(sig) {
                            return ret;
                        }
                    }
                }
                return self.get_any_type();
            }
            // like TS2304 above.
            if type_name.kind == SyntaxKind::Identifier
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeParameter
                        | SymbolFlags::TypeAlias
                        // Import aliases may legitimately name types
                        // (`import A = NS; let x: A.B`, or a module whose
                        // default export is an interface) — resolving the
                        // alias target is future work.
                        | SymbolFlags::Alias,
                )
                && self.ts2304_reporting_allowed_for(type_name)
                && !self.has_same_named_type_symbol(type_name.text())
                // Only user files: lib.d.ts var+interface merges keep
                // separate symbols in our binder and would over-report.
                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let name_text = type_name.text().to_string();
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    crate::diagnostics::messages_generated::
                        X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                    vec![name_text.clone(), name_text],
                ));
            }
            // Class/etc.: defer to error_type (any) for now.
            return self.error_type();
        }
        // Cycle guard: a recursive alias (`type A = B; type B = A`) would
        // otherwise infinite-loop here. Use the stack-based resolution
        // cycle detection (mirrors Go's pushTypeResolution).
        let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }
        // For non-generic aliases (no type arguments on the reference), use
        // the cached declared type. For generic references (type arguments
        // present), we must re-resolve the alias body with the type
        // parameters substituted by the type arguments, bypassing the cache.
        let has_type_args = type_arguments.is_some();
        let resolved = if !has_type_args {
            // Reuse a previously-computed declared type if present.
            let cached = self
                .type_alias_links
                .get(&symbol)
                .and_then(|l| l.declared_type.clone());
            cached.unwrap_or_else(|| {
                // The alias body is a separate lexical scope; drop the
                // `in_static_member_type` flag so any type-parameter reference
                // resolved while expanding it is not spuriously flagged TS2302.
                let saved_static = self.in_static_member_type;
                self.in_static_member_type = false;
                let found = self.resolve_alias_body(&symbol);
                self.in_static_member_type = saved_static;
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&found));
                found
            })
        } else {
            // Generic type alias instantiation: collect the alias's type
            // parameters, resolve the type arguments, push the mapping,
            // resolve the body, and pop.
            let (tp_symbols, type_node) = self.collect_alias_type_params_and_body(&symbol);
            let arg_types: Vec<Arc<Type>> = match &type_arguments {
                Some(args) => args
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect(),
                None => Vec::new(),
            };
            let mut mapping = HashMap::new();
            for (i, tp_sym) in tp_symbols.iter().enumerate() {
                if i < arg_types.len() {
                    let tp_key = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                    mapping.insert(tp_key, Arc::clone(&arg_types[i]));
                }
            }
            self.type_argument_stack.push(mapping);
            // The alias body is a separate lexical scope: type parameters
            // referenced inside it belong to the alias, not the referencing
            // static member. Drop the flag so nested resolution does not emit
            // spurious TS2302. Mirrors Go's NameResolver lexical check.
            // The alias DECLARATION is also pushed as a scope (mirroring
            // `resolve_alias_body`): the alias's own type parameters must
            // shadow same-named OUTER declarations for references in the
            // body — otherwise a lazily-triggered instantiation resolves
            // `U` in `type Ex<T, U> = T extends U ? T : never` against a
            // same-named top-level alias (intersectionMemberOfUnion
            // NarrowsCorrectly), silently replacing the extends clause.
            let alias_decl = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::TypeAliasDeclaration)
                .cloned();
            if let Some(decl) = &alias_decl {
                self.push_scope(decl);
            }
            let saved_static = self.in_static_member_type;
            self.in_static_member_type = false;
            let found = self.get_type_from_type_node(&type_node);
            self.in_static_member_type = saved_static;
            if alias_decl.is_some() {
                self.pop_scope();
            }
            self.type_argument_stack.pop();
            found
        };
        self.pop_type_resolution();
        resolved
    }

    /// Resolve the declared type of a type alias symbol (the alias body).
    ///
    /// Public so that the nodebuilder (hover info) can trigger resolution
    /// of an alias that has not been encountered during normal checking.
    /// Does NOT perform cycle protection or cache the result — callers are
    /// responsible for both. `resolve_type_reference` (the in-checker
    /// caller) maintains its own cycle guard and cache; the nodebuilder's
    /// `try_get_type_alias_declared_type` does the same.
    pub fn resolve_alias_body(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {
                // The alias declaration is a scope (Go: TypeAliasDeclaration
                // is IsContainer|HasLocals): its type parameters live in the
                // alias symbol's members, visible to the RHS only while this
                // declaration is on the scope stack.
                self.push_scope(decl);
                let result = self.get_type_from_type_node(&data.type_node);
                self.pop_scope();
                return result;
            }
        }
        self.error_type()
    }

    /// Collect a type alias's type-parameter symbols and its body type node.
    fn collect_alias_type_params_and_body(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> (Vec<Arc<Symbol>>, Arc<Node>) {
        let mut tp_symbols = Vec::new();
        let mut type_node = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {
                type_node = Some(Arc::clone(&data.type_node));
                if let Some(tps) = &data.type_parameters {
                    for tp in tps.iter() {
                        if let Some(tp_sym) = self.program.symbol_map().symbol_of(tp) {
                            tp_symbols.push(Arc::clone(tp_sym));
                        }
                    }
                }
                break;
            }
        }
        (
            tp_symbols,
            type_node.unwrap_or_else(|| Arc::clone(&symbol.declarations[0])),
        )
    }

    /// Resolve an `interface` declaration to an anonymous object type.
    ///
    /// Builds the type from the interface's members (PropertySignature,
    /// MethodSignature, IndexSignature). For generic interfaces (`Box<T>`),
    /// the type arguments are pushed onto the substitution stack before
    /// resolving members, so type-parameter references inside member types
    /// resolve to the corresponding type arguments.
    ///
    /// Heritage clauses (`extends A`) are not yet merged — only the
    /// interface's own members are included.
    pub(crate) fn resolve_interface_type(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Type> {
        let arg_types = type_arguments.map(|nodes| {
            nodes
                .iter()
                .map(|a| self.get_type_from_type_node(a))
                .collect()
        });
        self.resolve_interface_type_ex(symbol, arg_types)
    }

    /// `resolve_interface_type` with already-resolved type arguments —
    /// used by `create_array_type` (Go's
    /// `createTypeFromGenericGlobalType(globalArrayType, [elementType])`),
    /// where the element type is a `Type`, not a type node.
    pub(crate) fn resolve_interface_type_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        type_args: Option<Vec<Arc<Type>>>,
    ) -> Arc<Type> {
        // For non-generic interfaces, reuse a cached declared type —
        // except an ERROR cached form (a transiently failed resolution
        // from a cycle window; Go's lazy members never observe it).
        let has_type_args = type_args.is_some();
        if !has_type_args {
            if let Some(cached) = self
                .type_alias_links
                .get(symbol)
                .and_then(|l| l.declared_type.clone())
            {
                if !crate::checker::utilities::is_type_error(&cached) {
                    return cached;
                }
            }
        }
        // Memoize generic instantiations per (symbol, argument pointers) —
        // argument types are shared singletons in practice (intrinsics and
        // symbol-cached type parameters), so pointer identity is a sound
        // key, and repeated resolutions of e.g. `ConcatArray<T>[]` inside
        // Array's own members collapse into one instance (without this,
        // fresh instances defeat `array_type_cache` and member resolution
        // blows up exponentially on lib scale).
        let instantiation_key: Option<Vec<usize>> = type_args.as_ref().map(|args| {
            let mut key = Vec::with_capacity(args.len() + 1);
            key.push(Arc::as_ptr(symbol) as *const Symbol as usize);
            key.extend(
                args.iter()
                    .map(|t| Arc::as_ptr(t) as *const crate::checker::types::Type as usize),
            );
            key
        });
        // Pin the argument types for the cache entry below: freshly-built
        // argument types (unions, literals) are transient, and an unpinned
        // pointer key could be recycled by a different type after they
        // drop, returning a foreign instantiation.
        let pinned_args: Option<Vec<Arc<Type>>> = type_args.clone();
        if let Some(key) = &instantiation_key
            && let Some(cached) = self.interface_instantiation_cache.get(key)
        {
            return Arc::clone(&cached.1);
        }
        // Cycle guard for recursive interface references. NOTE: unlike Go,
        // the key is the bare symbol pointer (NOT arg-aware): an
        // arg-specific key un-blocks nested instantiations of the same
        // interface with different arguments, and the default libs'
        // mutually-recursive generic interfaces then expand without bound.
        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            // Cycle-guard failure: the error type returned here flows into
            // the CALLER's heritage merge (or a member type) as a degraded
            // result — count it so enclosing node-memo frames refuse to
            // cache whatever they compute from it (see
            // `heritage_degraded_events`; lib.dom's dense heritage graph
            // hits this on first resolution order).
            self.heritage_degraded_events += 1;
            return self.error_type();
        }
        // Collect ALL InterfaceDeclaration nodes so declaration merging
        // (`interface Foo { a }; interface Foo { b }`) produces a single
        // type with the union of members. Mirrors Go's
        // `getDeclaredTypeOfInterface` which walks every declaration.
        let interface_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))
            .cloned()
            .collect();
        // Set when a heritage base resolved to the error type through the
        // cycle guard — the merged result is incomplete and must not be
        // cached (see the declared-type write below). The entry epoch
        // additionally catches TRANSITIVE degradation: a base that was
        // itself resolved against an incomplete base produces a valid-
        // looking (non-error) type whose merge is still incomplete —
        // HTMLElement merging an own-members-only Element must not cache.
        let epoch_at_entry = self.heritage_degraded_events;
        let mut heritage_degraded = false;
        let result = match interface_decls.first() {
            Some(first) => {
                let data = match &first.data {
                    NodeData::InterfaceDeclaration(d) => d,
                    _ => unreachable!(),
                };
                // Collect type-parameter symbols for substitution (from the
                // first declaration — merged interfaces must have identical
                // type parameter lists).
                let tp_symbols = match &data.type_parameters {
                    Some(tps) => {
                        let sym_map = self.program.symbol_map();
                        let collected: Vec<Arc<Symbol>> = tps
                            .iter()
                            .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                            .collect();
                        collected
                    }
                    None => Vec::new(),
                };
                // Push the type-argument substitution mapping for generic
                // interfaces. Merged declarations bind SEPARATE symbols for
                // same-named type parameters (the second declaration's
                // binder conflict path replaces the first's in the merged
                // symbol's members) — key the mapping by EVERY declaration's
                // type-parameter symbols, matched by position among
                // same-named parameters, so members from either declaration
                // substitute (fork1: `interface I<T> {a: T}` + `interface
                // I<T> {b: T}` with `I<number>` must give `a: number`).
                let arg_types: Vec<Arc<Type>> = type_args.unwrap_or_default();
                if has_type_args {
                    let mut mapping = HashMap::new();
                    for (i, tp_sym) in tp_symbols.iter().enumerate() {
                        if let Some(arg) = arg_types.get(i) {
                            let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                            mapping.insert(k, Arc::clone(arg));
                        }
                    }
                    for decl in &interface_decls {
                        let NodeData::InterfaceDeclaration(d) = &decl.data else {
                            continue;
                        };
                        let Some(tps) = &d.type_parameters else {
                            continue;
                        };
                        let sym_map = self.program.symbol_map();
                        for (i, tp) in tps.iter().enumerate() {
                            let Some(tp_sym) = sym_map.symbol_of(tp) else {
                                continue;
                            };
                            // Match by name against the first declaration's
                            // parameter at the same position; fall back to
                            // the raw index.
                            let idx = if let Some(first_sym) = tp_symbols.get(i) {
                                if first_sym.name == tp_sym.name { i } else {
                                    tp_symbols.iter().position(|s| s.name == tp_sym.name).unwrap_or(i)
                                }
                            } else {
                                i
                            };
                            if let Some(arg) = arg_types.get(idx) {
                                let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                                mapping.insert(k, Arc::clone(arg));
                            }
                        }
                    }
                    self.type_argument_stack.push(mapping);
                }
                // Push scope so type-parameter references in member types
                // resolve via the scope stack.
                self.push_scope(
                    symbol
                        .declarations
                        .iter()
                        .next()
                        .expect("interface has a declaration"),
                );
                // Build a merged member list from all interface declarations.
                // Members are processed in declaration order so later
                // declarations can't shadow earlier ones with a conflicting
                // type (real TS would report TS2717 here).
                let merged_members: Vec<Arc<Node>> = interface_decls
                    .iter()
                    .flat_map(|decl| match &decl.data {
                        NodeData::InterfaceDeclaration(d) => d.members.iter().cloned(),
                        _ => unreachable!(),
                    })
                    .collect();
                let merged_list = Arc::new(NodeList::new(merged_members));
                // Resolving an interface's member types crosses into a new
                // lexical scope: type-parameter references here belong to the
                // interface's own declaration, not the referencing static
                // member. Drop `in_static_member_type` so nested member
                // resolution does not emit spurious TS2302. Mirrors Go's
                // NameResolver, which only fires when the reference's own
                // `lastLocation` is a static member.
                let saved_static = self.in_static_member_type;
                self.in_static_member_type = false;
                let own_result = self.build_interface_type_from_members(&merged_list);
                self.in_static_member_type = saved_static;
                // `extends` bases: merge each base interface's members and
                // signatures into the derived type. Derived members override
                // same-named base members; CALL signatures concatenate (the
                // derived interface overloads the base — `interface Bar
                // extends Foo { (key: string): string }` keeps Foo's
                // `(): string` too). Mirrors Go's `resolveBaseTypes` +
                // `resolveDeclaredMembers` for interfaces. Resolved INSIDE
                // the interface's scope so its type parameters resolve in
                // base references (`extends Base<T>`).
                // (heritage type-ref node, resolved base type) — the node
                // identifies the `extends` clause for TS2430 dedup.
                let mut base_types: Vec<(Arc<Node>, Arc<Type>)> = Vec::new();
                for decl in &interface_decls {
                    if let NodeData::InterfaceDeclaration(d) = &decl.data {
                        if let Some(heritage) = &d.heritage_clauses {
                            for clause in heritage.iter() {
                                if let NodeData::HeritageClause(hc) = &clause.data
                                    && hc.token == SyntaxKind::ExtendsKeyword
                                {
                                    for type_ref in hc.types.iter() {
                                        let bt = self.get_type_from_type_node(type_ref);
                                        // A base that resolved to the error
                                        // type through the cycle guard (the
                                        // base's own resolution is mid-flight
                                        // and needed THIS interface) leaves
                                        // the merged result incomplete —
                                        // remember that so the degraded form
                                        // is NOT cached (a later resolution
                                        // retries with the complete base).
                                        // NOTE: the test must be exact
                                        // (`is_type_error`), not the Any
                                        // flag: the error type IS Any-flagged
                                        // but so is every legitimate `any`
                                        // base (`interface I extends
                                        // AnyAlias`), which must NOT count
                                        // as degradation.
                                        if crate::checker::utilities::is_type_error(&bt)
                                            && !heritage_degraded
                                        {
                                            heritage_degraded = true;
                                            self.heritage_degraded_events += 1;
                                        }
                                        base_types.push((Arc::clone(type_ref), bt));
                                    }
                                }
                            }
                        }
                    }
                }
                self.pop_scope();
                if has_type_args {
                    self.type_argument_stack.pop();
                }
                let result = if base_types.is_empty() {
                    own_result.clone()
                } else {
                    let mut merged = own_result.clone();
                    for (_, base) in &base_types {
                        merged = self.merge_interface_type_with_base(&merged, base);
                    }
                    merged
                };
                // TS2430: a derived interface member must be assignable to
                // the same-named base member (`interface Bar extends Foo`).
                // Checked against the OWN members (before merging) so
                // inherited members don't self-compare. Reported on the
                // interface's name (Go's checkTypeStack relation error).
                // Only the declared (uninstantiated) resolution reports —
                // Go checks heritage once per declaration in the check
                // phase — and each (interface, base clause) pair reports at
                // most once regardless of re-resolution.
                if !has_type_args && !base_types.is_empty() {
                    let own_structured = match &own_result.data {
                        TypeData::Object(o) => Some(&o.structured),
                        _ => None,
                    };
                    let name_loc = interface_decls.first().and_then(|d| {
                        match &d.data {
                            NodeData::InterfaceDeclaration(d) => Some(d.name.loc),
                            _ => None,
                        }
                    });
                                            if let (Some(own), Some(name_loc)) = (own_structured, name_loc) {
                        for (type_ref_node, base) in &base_types {
                            // One TS2430 per (interface symbol, heritage
                            // type-ref node), matching the official
                            // baseline's single line per extends pair.
                            let dedup_key = (
                                Arc::as_ptr(symbol) as *const crate::ast::Symbol,
                                Arc::as_ptr(type_ref_node) as *const crate::ast::Node,
                            );
                            if self.interface_extends_reported.contains(&dedup_key) {
                                continue;
                            }
                            let base_structured = match &base.data {
                                TypeData::Object(o) => Some(&o.structured),
                                _ => None,
                            };
                            let Some(base_structured) = base_structured else {
                                continue;
                            };
                            for own_prop in &own.properties {
                                let Some(base_prop) = base_structured
                                    .members
                                    .get(&own_prop.name)
                                else {
                                    continue;
                                };
                                let derived_type = self
                                    .value_symbol_links
                                    .get(own_prop)
                                    .and_then(|l| l.resolved_type.clone());
                                let base_type = self
                                    .value_symbol_links
                                    .get(base_prop)
                                    .and_then(|l| l.resolved_type.clone());
                                if let (Some(dt), Some(bt)) = (derived_type, base_type) {
                                    // A RAW generic base member (referenced
                                    // without type arguments, e.g.
                                    // `writable: WritableStream` in
                                    // `interface GenericTransformStream`)
                                    // behaves like an any-arg instantiation
                                    // (official implicit-any args) —
                                    // substitute its declaration's type
                                    // parameters with `any` before the check.
                                    let bt = match bt.symbol.as_ref() {
                                        Some(bsym) => {
                                            let tps = self.declared_type_parameter_types(bsym);
                                            if !tps.is_empty()
                                                && bt.as_object().is_none_or(|o| {
                                                    o.type_arguments.is_empty()
                                                })
                                            {
                                                let anys: Vec<Arc<Type>> = std::iter::repeat(
                                                    self.get_any_type(),
                                                )
                                                .take(tps.len())
                                                .collect();
                                                self.resolve_interface_type_ex(
                                                    bsym,
                                                    Some(anys),
                                                )
                                            } else {
                                                bt
                                            }
                                        }
                                        None => bt,
                                    };
                                    // Open a relation-chain window around the
                                    // heritage member check — the captured
                                    // entries (signature-return/parameter
                                    // markers, nested heads) render under the
                                    // TS2430 head exactly like Go's heritage
                                    // `checkTypeAssignableTo` custom-message
                                    // path (reportRelationError +
                                    // createDiagnosticChainFromErrorChain).
                                    let saved_chain =
                                        std::mem::take(&mut self.relater_error_chain);
                                    let was_active = self.relater_chain_active;
                                    self.relater_chain_active = true;
                                    let incompatible = !self.is_type_assignable_to(&dt, &bt);
                                    let captured = std::mem::replace(
                                        &mut self.relater_error_chain,
                                        saved_chain,
                                    );
                                    self.relater_chain_active = was_active;
                                    if incompatible {
                                        self.interface_extends_reported.insert(dedup_key);
                                        let base_name = base
                                            .symbol
                                            .as_ref()
                                            .map(|s| s.name.clone())
                                            .unwrap_or_default();
                                        let file = self.current_file.clone();
                                        let mut diag = crate::ast::Diagnostic::new(
                                            file,
                                            name_loc,
                                            crate::diagnostics::messages_generated::
                                                INTERFACE_0_INCORRECTLY_EXTENDS_INTERFACE_1,
                                            vec![symbol.name.clone(), base_name],
                                        );
                                        // The member-level tail head (Go
                                        // reportErrorResults pushes a default
                                        // relation head for the failed
                                        // member comparison) followed by the
                                        // property entry — relater_
                                        // report_error's transforms collapse
                                        // [head, marker] into "The types
                                        // returned by 'x(...)'" / chain
                                        // dotted names as needed.
                                        let dt_str = self.type_to_string(&dt);
                                        let bt_str = self.type_to_string(&bt);
                                        self.relater_error_chain = captured;
                                        self.relater_chain_active = true;
                                        self.push_relation_head_with_tp_note(
                                            &dt,
                                            &bt,
                                            crate::diagnostics::messages_generated::
                                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                            vec![dt_str, bt_str],
                                        );
                                        self.relater_report_error(
                                            crate::diagnostics::messages_generated::
                                                TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                                            vec![self.chain_property_arg_name(own_prop)],
                                        );
                                        let entries = std::mem::take(
                                            &mut self.relater_error_chain,
                                        );
                                        self.relater_chain_active = was_active;
                                        // Fold the chronological entries into
                                        // the nested pyramid (first pushed =
                                        // innermost; the 2430 head outermost).
                                        let mut child: Option<
                                            crate::ast::Diagnostic,
                                        > = None;
                                        for entry in entries
                                            .iter()
                                            .filter(|e| {
                                                !e.message.elided_in_compatibility_pyramid
                                            })
                                        {
                                            let mut d = crate::ast::Diagnostic::new(
                                                None,
                                                name_loc,
                                                entry.message,
                                                entry.args.clone(),
                                            );
                                            if let Some(c) = child.take() {
                                                d.message_chain = vec![c];
                                            }
                                            child = Some(d);
                                        }
                                        if let Some(c) = child {
                                            diag.message_chain = vec![c];
                                        }
                                        self.diagnostics.add(diag);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                // Attach the interface symbol so the type printer uses the
                // declared name (Go's `Type.Symbol`), and record the type
                // arguments so generic instantiations display as `I<A, B>`
                // (official keeps a type reference with its arguments; our
                // anonymous object loses them otherwise). The type was just
                // created above and is not yet shared.
                {
                    let result_mut = Arc::as_ptr(&result) as *mut crate::checker::types::Type;
                    unsafe {
                        (*result_mut).symbol = Some(Arc::clone(symbol));
                        if has_type_args
                            && let TypeData::Object(o) = &mut (*result_mut).data
                        {
                            o.type_arguments = arg_types.clone();
                        }
                    }
                }
                result
            }
            None => self.error_type(),
        };
        self.pop_type_resolution();
        // A heritage-degraded resolution (a base hit the cycle guard's
        // error type because the base needed THIS interface mid-flight)
        // must not poison the declared-type cache — skip caching so a
        // later reference re-resolves against the now-complete base. The
        // epoch comparison extends this to transitively degraded bases
        // (see `epoch_at_entry` above).
        if self.heritage_degraded_events != epoch_at_entry {
            heritage_degraded = true;
        }
        // Convergence guarantee: a degraded resolution is retried (left
        // uncached) at most HERITAGE_RETRY_LIMIT times per symbol. True
        // base-type cycles degrade on every retry — uncached retries
        // would re-walk the inheritance graph on every reference (the r9
        // timeout storm). Once the limit is reached the degraded form is
        // accepted: it is cached, and the epoch counter is rolled back to
        // this resolution's entry value so ENCLOSING node-memo frames
        // also cache their (final) results. Go obtains the same stability
        // from its lazy member resolution — the shared type object of a
        // circular base is incomplete but stable (TS2310).
        let mut degraded_accepted = false;
        if heritage_degraded {
            let sym_key = Arc::as_ptr(symbol) as *const crate::ast::Symbol as usize;
            let retries = self.heritage_retry_counts.entry(sym_key).or_insert(0);
            *retries += 1;
            degraded_accepted = *retries > crate::checker::checker::HERITAGE_RETRY_LIMIT;
        }
        let cache_result = !heritage_degraded || degraded_accepted;
        if degraded_accepted && self.heritage_degraded_events != epoch_at_entry {
            self.heritage_degraded_events = epoch_at_entry;
        }
        if heritage_degraded {
            // The result's member table may be transiently incomplete (a
            // base hit the in-flight guard). Record it so comparisons
            // against this object suppress diagnostics instead of
            // reporting phantom incompatibilities (the react16/lib.dom
            // D1 family — Go's lazy member resolution never produces
            // these mid-flight forms).
            self.degraded_type_ptrs
                .insert(Arc::as_ptr(&result) as *const crate::checker::types::Type as usize);
        }
        if !has_type_args && cache_result {
            self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        }
        if let Some(key) = instantiation_key {
            if cache_result {
                // Pin the ARGUMENT types in the value: freshly-built
                // argument types (unions, literals) are transient, and an
                // unpinned pointer key could be recycled by a different
                // type after they drop, returning a foreign instantiation.
                let pin = pinned_args.clone().unwrap_or_default();
                self.interface_instantiation_cache
                    .insert(key, (pin, Arc::clone(&result)));
            }
        }
        result
    }

    /// Build an anonymous object type from an interface's member list.
    /// Handles PropertySignature, MethodSignature, and IndexSignature.
    pub(crate) fn build_interface_type_from_members(
        &mut self,
        members: &Arc<NodeList>,
    ) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();
        // Call and construct signatures are tracked separately so that, after
        // the member loop, they can be concatenated with call signatures first
        // (StructuredTypeData stores call sigs at indices
        // 0..call_signature_count, then construct sigs). Without this, an
        // interface like `interface FC { (props): any }` loses its call
        // signature, causing TS2604 for JSX components typed via such an
        // interface (e.g. React's `ExoticComponent`/`StrictMode`).
        let mut call_signatures: Vec<Arc<Signature>> = Vec::new();
        let mut construct_signatures: Vec<Arc<Signature>> = Vec::new();
        for member in members.iter() {
            match &member.data {
                NodeData::PropertySignatureDeclaration(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    let mut prop_type = self.get_type_from_type_node(&data.type_node);
                    let is_optional = data
                        .postfix_token
                        .as_ref()
                        .map(|t| t.kind == SyntaxKind::QuestionToken)
                        .unwrap_or(false);
                    // Optional properties (`x?: T`) have type `T | undefined`
                    // so that `undefined` is a valid value and missing
                    // properties are allowed in object-literal assignment.
                    if is_optional {
                        prop_type = self.get_optional_type(prop_type);
                    }
                    let mut flags = SymbolFlags::Property;
                    if is_optional {
                        flags |= SymbolFlags::Optional;
                    }
                    let mut symbol = Symbol::new(flags, name.clone());
                    // Attach the declaration so display paths can recover
                    // the name node's source form (string-literal members
                    // keep their quotes in chain messages).
                    symbol.declarations = vec![Arc::clone(member)];
                    // Propagate the `readonly` modifier onto the synthetic
                    // symbol so the checker can detect TS2540 assignments
                    // to readonly interface properties. Mirrors Go's
                    // `getDeclarationModifierFlagsFromSymbol` returning
                    // `ModifierFlagsReadonly`.
                    if let Some(m) = &data.modifiers {
                        if m.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                    }
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::MethodSignatureDeclaration(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    // Build a function type from the method signature. The
                    // member node is pushed as a scope: the binder declares
                    // signature-level type parameters into the signature's
                    // own locals (Go `GetContainerFlags` gives MethodSignature
                    // HasLocals), so `K` references in the parameter and
                    // return annotations resolve against THIS signature —
                    // not against other same-named methods' parameters.
                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                        /* is_construct */ false,
                        /* contextual_signature */ None,
                        /* declaration */ Some(Arc::clone(member)),
                    );
                    self.pop_scope();
                    // Interface method overload group (`create(o): any;
                    // create(o, props): any`): a same-named signature that
                    // follows an earlier one is an overload — merge its
                    // signature into the existing symbol's call signatures
                    // instead of replacing the symbol (which would drop the
                    // earlier overloads). Mirrors tsc's interface members
                    // resolution collecting all same-named signatures.
                    if let Some(existing) = symbol_table.get(&name).cloned() {
                        let existing_type = self
                            .value_symbol_links
                            .get(&existing)
                            .and_then(|l| l.resolved_type.clone());
                        let merged_sigs = existing_type
                            .as_ref()
                            .and_then(|t| t.as_structured().map(|s| s.call_signatures().to_vec()))
                            .unwrap_or_default();
                        let mut all_sigs = merged_sigs;
                        all_sigs.push(sig);
                        let fn_type = self.create_function_or_constructor_type(all_sigs, false);
                        self.value_symbol_links.insert(
                            &existing,
                            ValueSymbolLinks {
                                resolved_type: Some(fn_type),
                                ..Default::default()
                            },
                        );
                        continue;
                    }
                    let fn_type = self.create_function_or_constructor_type(vec![sig], false);
                    let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(fn_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::IndexSignatureDeclaration(data) => {
                    let mut key_type = None;
                    let mut value_type = None;
                    if let Some(param) = data.parameters.iter().next() {
                        if let NodeData::ParameterDeclaration(pd) = &param.data {
                            key_type = pd
                                .type_node
                                .as_ref()
                                .map(|t| self.get_type_from_type_node(t));
                        }
                    }
                    value_type = Some(self.get_type_from_type_node(&data.type_node));
                    let is_readonly = member
                        .modifiers()
                        .as_ref()
                        .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
                    index_infos.push(Arc::new(crate::checker::IndexInfo {
                        key_type,
                        value_type,
                        is_readonly,
                        declaration: Some(Arc::clone(member)),
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }
                // Class instance members. `PropertyDeclaration` and
                // `MethodDeclaration` carry the same name/postfix/parameters/
                // type_node shape as their `*SignatureDeclaration` counterparts
                // but the type_node is `Option` (a class property may be
                // initialized without an explicit annotation). Constructors and
                // static members are skipped here — they don't contribute to
                // the instance type used by `implements` checks.
                NodeData::PropertyDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    let mut prop_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => match data.initializer.as_ref() {
                            Some(init) => {
                                // Go checkDeclarationInitializer +
                                // widenTypeInferredFromInitializer: a
                                // mutable class property widens fresh
                                // literal initializers (`D = 'lit'` →
                                // string), a `readonly` one preserves
                                // them. A bare variable-reference
                                // initializer reads the symbol's DECLARED
                                // type — property initializers live in
                                // their own flow container and don't see
                                // outer assignments, so
                                // `const d: AB = 'A'; class { P = d }`
                                // types P as AB (not the narrowed 'A').
                                let raw = match &init.data {
                                    NodeData::Identifier(_) => {
                                        match self.resolve_identifier(init) {
                                            Some(sym) if sym.flags.intersects(
                                                SymbolFlags::BlockScopedVariable
                                                    | SymbolFlags::FunctionScopedVariable,
                                            ) => self.get_type_of_symbol(&sym),
                                            _ => self.get_type_of_node(init),
                                        }
                                    }
                                    _ => self.get_type_of_node(init),
                                };
                                let is_readonly = data
                                    .modifiers
                                    .as_ref()
                                    .is_some_and(|m| {
                                        m.modifier_flags.contains(ModifierFlags::Readonly)
                                    });
                                let widened = if is_readonly {
                                    raw
                                } else if self.is_empty_array_literal(init) {
                                    // Go getTypeForVariableLikeDeclaration:
                                    // properties never get the flow-tracked
                                    // autoArrayType — the empty array's
                                    // element widens instead. Under
                                    // strictNullChecks the implicit-never
                                    // element stays `never[]`; without it
                                    // the widening-undefined element
                                    // widens to plain `any[]`.
                                    if self.strict_null_checks {
                                        self.get_widened_literal_type(&raw)
                                    } else {
                                        self.create_array_type(self.get_any_type())
                                    }
                                } else {
                                    self.get_widened_literal_type(&raw)
                                };
                                let regularized =
                                    self.get_regular_type_of_literal_type(&widened);
                                self.widen_initializer_type(&regularized)
                            }
                            None => self.get_any_type(),
                        },
                    };
                    let is_optional = data
                        .postfix_token
                        .as_ref()
                        .map(|t| t.kind == SyntaxKind::QuestionToken)
                        .unwrap_or(false);
                    if is_optional {
                        prop_type = self.get_optional_type(prop_type);
                    }
                    let mut flags = SymbolFlags::Property;
                    if is_optional {
                        flags |= SymbolFlags::Optional;
                    }
                    let mut symbol = Symbol::new(flags, name.clone());
                    // Attach the declaring member node so the checker can
                    // inspect its modifiers (e.g. `readonly` for TS2540,
                    // `private` for TS2341) — mirroring Go's
                    // `getDeclarationModifierFlagsFromDeclarations`.
                    symbol.declarations.push(Arc::clone(member));
                    // Propagate the `readonly` modifier so the checker can
                    // emit TS2540 for assignments to readonly class properties
                    // outside the constructor. Mirrors Go's
                    // `getDeclarationModifierFlagsFromDeclarations` returning
                    // `ModifierFlagsReadonly`.
                    if let Some(m) = &data.modifiers {
                        if m.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                    }
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::MethodDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    // The method node is pushed as a scope (D3-sig): the
                    // binder declares the method's own type parameters into
                    // the METHOD node's locals (Go gives MethodDeclaration
                    // HasLocals). Without the push, `X<T>` parameter
                    // annotations resolve against the CLASS symbol's members
                    // — a same-named class type parameter wins and the call
                    // site checks against the class instantiation's T
                    // (genericClassWithObjectTypeArgsAndConstraints).
                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                        /* is_construct */ false,
                        /* contextual_signature */ None,
                        /* declaration */ Some(Arc::clone(member)),
                    );
                    self.pop_scope();
                    // Overload group handling: a same-named body-less
                    // declaration that follows an earlier one is an
                    // overload — merge its signature into the existing
                    // symbol's call-signature list. The body-carrying
                    // IMPLEMENTATION declaration never joins the callable
                    // set (Go's getSignaturesOfType only exposes body-less
                    // overload declarations; the implementation is checked
                    // for compatibility, not called through).
                    if let Some(existing) = symbol_table.get(&name).cloned() {
                        if data.body.is_some() {
                            // Record the declaration (modifier/accessor
                            // checks read it) but keep the overload list.
                            let existing_mut =
                                Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                            continue;
                        }
                        let existing_type = self
                            .value_symbol_links
                            .get(&existing)
                            .and_then(|l| l.resolved_type.clone());
                        let merged_sigs = existing_type
                            .as_ref()
                            .and_then(|t| {
                                t.as_structured().map(|s| s.call_signatures().to_vec())
                            })
                            .unwrap_or_default();
                        let mut all_sigs = merged_sigs;
                        all_sigs.push(sig);
                        let fn_type =
                            self.create_function_or_constructor_type(all_sigs, false);
                        self.value_symbol_links.insert(
                            &existing,
                            ValueSymbolLinks {
                                resolved_type: Some(fn_type),
                                ..Default::default()
                            },
                        );
                        continue;
                    }
                    let fn_type = self.create_function_or_constructor_type(vec![sig], false);
                    let mut symbol = Symbol::new(SymbolFlags::Property, name.clone());
                    // Attach the declaring member node so the checker can
                    // inspect its modifiers (`private`/`protected` for
                    // TS2341/TS2411).
                    symbol.declarations.push(Arc::clone(member));
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(fn_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::GetAccessorDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    // The accessor pair shares one property symbol: a getter
                    // defines the property's type (its return annotation);
                    // a setter-only property uses the parameter type.
                    let prop_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    match symbol_table.get(&name).cloned() {
                        Some(existing) => {
                            // Setter (or earlier getter) already inserted —
                            // union the accessor flag and refresh the type
                            // from the getter.
                            let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).flags |= SymbolFlags::GetAccessor;
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                            self.value_symbol_links.insert(
                                &existing,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                        }
                        None => {
                            let mut symbol =
                                Symbol::new(SymbolFlags::Property | SymbolFlags::GetAccessor, name.clone());
                            symbol.declarations.push(Arc::clone(member));
                            let symbol = Arc::new(symbol);
                            self.value_symbol_links.insert(
                                &symbol,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                            symbol_table.insert(name, Arc::clone(&symbol));
                            props.push(symbol);
                        }
                    }
                }
                NodeData::SetAccessorDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    // Setter-only property: type from the value parameter.
                    let prop_type = data
                        .parameters
                        .iter()
                        .next()
                        .and_then(|p| {
                            if let NodeData::ParameterDeclaration(pd) = &p.data {
                                pd.type_node.as_ref().map(|tn| self.get_type_from_type_node(tn))
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| self.get_any_type());
                    match symbol_table.get(&name).cloned() {
                        Some(existing) => {
                            // A getter already defined the property — just
                            // attach the setter declaration.
                            let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).flags |= SymbolFlags::SetAccessor;
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                        }
                        None => {
                            let mut symbol =
                                Symbol::new(SymbolFlags::Property | SymbolFlags::SetAccessor, name.clone());
                            symbol.declarations.push(Arc::clone(member));
                            let symbol = Arc::new(symbol);
                            self.value_symbol_links.insert(
                                &symbol,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                            symbol_table.insert(name, Arc::clone(&symbol));
                            props.push(symbol);
                        }
                    }
                }
                NodeData::CallSignatureDeclaration(data) => {
                    // Suppress TS2304 while resolving the signature's
                    // parameter/return types: lib.d.ts call/construct
                    // signatures may reference signature-level type parameters
                    // (e.g. `<TArrayBuffer>`) that have no binder symbol; such
                    // names degrade to `any` instead of erroring. Scoped to
                    // BUNDLED lib files only — user files report normally
                    // (Go resolves signature-scoped type parameters; the
                    // blanket suppression hid valid errors like
                    // `typeof arguments` in call-signature params).
                    let suppress = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.file_name.starts_with("bundled://"));
                    if suppress {
                        self.push_ts2304_suppression();
                    }
                    // The signature node is pushed as a scope: the binder
                    // declares signature-level type parameters into the
                    // signature's own locals (Go gives CallSignature
                    // HasLocals), so `<T>(x: T): T` annotations resolve
                    // against THIS signature — not against an outer
                    // same-named declaration (interface member building runs
                    // lazily from foreign contexts).
                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                        /* is_construct */ false,
                        /* contextual_signature */ None,
                        /* declaration */ Some(Arc::clone(member)),
                    );
                    self.pop_scope();
                    if suppress {
                        self.pop_ts2304_suppression();
                    }
                    call_signatures.push(sig);
                }
                NodeData::ConstructSignatureDeclaration(data) => {
                    let suppress = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.file_name.starts_with("bundled://"));
                    if suppress {
                        self.push_ts2304_suppression();
                    }
                    // Same scope push as the CallSignature branch above.
                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                        /* is_construct */ true,
                        /* contextual_signature */ None,
                        /* declaration */ Some(Arc::clone(member)),
                    );
                    self.pop_scope();
                    if suppress {
                        self.pop_ts2304_suppression();
                    }
                    construct_signatures.push(sig);
                }
                NodeData::ConstructorDeclaration(data) => {
                    // Parameter properties: `constructor(private N: number)`
                    // declares an instance property `N` on the class. Mirrors
                    // Go's `resolveDeclaredMembers` constructor case, which
                    // creates a Property symbol for each parameter carrying a
                    // parameter-property modifier (public/private/protected/
                    // readonly) with a simple identifier name.
                    for param in data.parameters.iter() {
                        let NodeData::ParameterDeclaration(pd) = &param.data else {
                            continue;
                        };
                        if pd.name.kind != SyntaxKind::Identifier {
                            continue;
                        }
                        let Some(modifiers) = &pd.modifiers else {
                            continue;
                        };
                        if !modifiers.modifier_flags.intersects(
                            ModifierFlags::Public
                                | ModifierFlags::Private
                                | ModifierFlags::Protected
                                | ModifierFlags::Readonly,
                        ) {
                            continue;
                        }
                        let name = pd.name.text().to_string();
                        if name.is_empty() || symbol_table.get(&name).is_some() {
                            continue;
                        }
                        let prop_type = match pd.type_node.as_ref() {
                            Some(tn) => self.get_type_from_type_node(tn),
                            None => match pd.initializer.as_ref() {
                                Some(init) => self.get_type_of_node(init),
                                None => self.get_any_type(),
                            },
                        };
                        let mut symbol = Symbol::new(SymbolFlags::Property, name.clone());
                        // Attach the parameter declaration so modifier checks
                        // (e.g. TS2341 `private`) can inspect it.
                        symbol.declarations.push(Arc::clone(param));
                        if modifiers.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                        let symbol = Arc::new(symbol);
                        self.value_symbol_links.insert(
                            &symbol,
                            ValueSymbolLinks {
                                resolved_type: Some(prop_type),
                                ..Default::default()
                            },
                        );
                        symbol_table.insert(name, Arc::clone(&symbol));
                        props.push(symbol);
                    }
                }
                _ => {}
            }
        }
        // Concatenate call signatures first, then construct signatures.
        let call_signature_count = call_signatures.len();
        let mut signatures = call_signatures;
        signatures.extend(construct_signatures);
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    /// Merge a derived interface type with one of its `extends` base types:
    /// derived members override same-named base members (base keeps list
    /// position), and base call/construct signatures are concatenated so
    /// both overload sets remain callable.
    fn merge_interface_type_with_base(
        &mut self,
        derived: &Arc<Type>,
        base: &Arc<Type>,
    ) -> Arc<Type> {
        if base.flags.contains(TypeFlags::Any) {
            return Arc::clone(derived);
        }
        let derived_data = match &derived.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let base_data = match &base.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        // Derived (own) members first, then base members not already
        // present — Go resolveObjectTypeMembers declares first and only
        // ADDS inherited members (addInheritedMembers), so own members
        // lead the property order (missing-property chains follow it).
        for prop in &derived_data.properties {
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        for prop in &base_data.properties {
            if symbol_table.get(&prop.name).is_some() {
                continue;
            }
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        let mut index_infos = derived_data.index_infos.clone();
        index_infos.extend(base_data.index_infos.iter().cloned());
        // Concatenate overload sets: derived call signatures first, then
        // base's (Go `core.Concatenate(callSignatures, base)` in
        // resolveObjectTypeMembers — derived overloads take precedence);
        // construct signatures after call signatures (StructuredTypeData
        // layout).
        let mut call_signatures: Vec<Arc<Signature>> =
            derived_data.call_signatures().to_vec();
        let derived_call_count = call_signatures.len();
        call_signatures.extend(base_data.call_signatures().iter().cloned());
        let mut signatures = call_signatures;
        signatures.extend(derived_data.construct_signatures().iter().cloned());
        signatures.extend(base_data.construct_signatures().iter().cloned());
        let merged = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count: derived_call_count
                        + base_data.call_signatures().len(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        // A degraded side makes the merged table incomplete — propagate
        // the marker so comparisons against the merged object suppress
        // diagnostics (see `degraded_type_ptrs`).
        let merged_degraded = {
            let p = |t: &Arc<Type>| Arc::as_ptr(t) as *const Type as usize;
            self.degraded_type_ptrs.contains(&p(base))
                || self.degraded_type_ptrs.contains(&p(derived))
        };
        if merged_degraded {
            self.degraded_type_ptrs
                .insert(Arc::as_ptr(&merged) as *const Type as usize);
        }
        merged
    }

    /// Resolve an `enum` declaration to a union of its member literal types.
    ///
    /// Mirrors Go's `getEnumMemberType`/enum resolution in
    /// `internal/checker/checker.go`. Numeric enums with no explicit
    /// initializer auto-increment from the previous numeric value (starting
    /// at 0); string-enum members without an initializer fall back to `any`
    /// (real TS reports an error there). Each member's literal type is cached
    /// on its symbol via `value_symbol_links` so that `Color.Red` property
    /// access can recover the literal type.
    pub fn resolve_enum_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // Reuse a cached declared type if present.
        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }
        // Cycle guard for recursive enum references.
        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }
        // Collect members from ALL EnumDeclaration nodes in the symbol's
        // declarations list. Merged enums (`enum E { A } enum E { B }`) share
        // a single symbol whose `declarations` carries each declaration; the
        // resulting enum type is the union of every declaration's members.
        // Mirrors Go's `getDeclaredTypeOfEnum`.
        let sym_map = self.program.symbol_map();
        let mut entries: Vec<(Option<Arc<Symbol>>, String, Option<Arc<Node>>)> = Vec::new();
        for decl in symbol.declarations.iter() {
            if let NodeData::EnumDeclaration(data) = &decl.data {
                for member_node in data.members.iter() {
                    let NodeData::EnumMember(member) = &member_node.data else {
                        continue;
                    };
                    let member_name = member.name.text().to_string();
                    let member_sym = sym_map.symbol_of(member_node).map(Arc::clone);
                    entries.push((member_sym, member_name, member.initializer.clone()));
                }
            }
        }
        let result = if entries.is_empty() {
            self.error_type()
        } else {
            let mut member_types: Vec<Arc<Type>> = Vec::new();
            let mut next_value: Option<f64> = Some(0.0);
            for (member_sym, member_name, initializer) in &entries {
                let base = match initializer {
                    Some(init) => {
                        // Resolve the initializer expression's type.
                        let t = self.get_type_of_node(init);
                        // Track numeric values for auto-increment of
                        // subsequent initializer-less members.
                        if t.flags.contains(TypeFlags::NumberLiteral) {
                            if let TypeData::Literal(LiteralTypeData {
                                value: LiteralValue::Number(n),
                                ..
                            }) = &t.data
                            {
                                next_value = Some(n.0 + 1.0);
                            }
                        } else if t.flags.contains(TypeFlags::StringLiteral) {
                            // String enum: no auto-increment.
                            next_value = None;
                        }
                        t
                    }
                    None => match next_value {
                        Some(v) => {
                            next_value = Some(v + 1.0);
                            self.get_number_literal_type(jsnum::Number::from(v))
                        }
                        None => {
                            // String enum without initializer — error in
                            // real TS, fall back to any.
                            self.get_any_type()
                        }
                    },
                };
                // Enum members type as enum LITERAL types (Go
                // getDeclaredTypeOfEnum: getEnumLiteralType gives each
                // member its own literal instance carrying the member
                // symbol, and the member SYMBOL's declared type is the
                // FRESH variant — so `let v = E.A` widens back to `E`
                // while `const` keeps the literal). The enum's own type
                // (the union below) is built from the REGULAR variants.
                let member_type = if base
                    .flags
                    .intersects(TypeFlags::NumberLiteral | TypeFlags::StringLiteral)
                {
                    let value = match &base.data {
                        TypeData::Literal(lit) => lit.value.clone(),
                        _ => LiteralValue::None,
                    };
                    let enum_literal_flags = base.flags | TypeFlags::EnumLiteral;
                    let mut regular_ty = Type::new(
                        enum_literal_flags,
                        TypeData::Literal(LiteralTypeData {
                            value: value.clone(),
                            fresh_type: OnceLock::new(),
                            regular_type: OnceLock::new(),
                        }),
                    );
                    regular_ty.symbol = member_sym.clone();
                    let regular_ty = Arc::new(regular_ty);
                    let mut fresh_ty = Type::new(
                        enum_literal_flags,
                        TypeData::Literal(LiteralTypeData {
                            value,
                            fresh_type: OnceLock::new(),
                            regular_type: OnceLock::from(Arc::clone(&regular_ty)),
                        }),
                    );
                    fresh_ty.symbol = member_sym.clone();
                    let fresh_ty = Arc::new(fresh_ty);
                    // Back-link so `get_fresh_type_of_literal_type` on the
                    // regular variant yields the same fresh instance.
                    if let TypeData::Literal(reg_lit) = &regular_ty.data {
                        let _ = reg_lit.fresh_type.set(Arc::clone(&fresh_ty));
                    }
                    if let Some(ms) = member_sym {
                        self.value_symbol_links.insert(
                            ms,
                            ValueSymbolLinks {
                                resolved_type: Some(fresh_ty),
                                ..Default::default()
                            },
                        );
                    }
                    regular_ty
                } else {
                    // Computed/unknown-valued member: keep prior behavior.
                    if let Some(ms) = member_sym {
                        self.value_symbol_links.insert(
                            ms,
                            ValueSymbolLinks {
                                resolved_type: Some(Arc::clone(&base)),
                                ..Default::default()
                            },
                        );
                    }
                    base
                };
                let _ = member_name; // name recorded for future diagnostics
                member_types.push(member_type);
            }
            match member_types.len() {
                0 => self.never_type(),
                1 => member_types.into_iter().next().unwrap(),
                _ => self.get_union_type(member_types),
            }
        };
        self.pop_type_resolution();
        self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        result
    }

    /// Go `getTypeOfPrototypeProperty` (checker.go ~L18096): the automatic
    /// static `prototype` property of a class is typed as the class
    /// (instance) type instantiated with `any` for each type parameter.
    pub(crate) fn get_type_of_prototype_property(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let Some(parent) = symbol.parent.clone() else {
            return self.get_any_type();
        };
        let Some(class_decl) = parent
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ClassDeclaration(_)))
            .cloned()
        else {
            return self.get_any_type();
        };
        let ctor_type = self.get_type_of_class_declaration(&class_decl);
        let instance_type = ctor_type
            .as_structured()
            .and_then(|s| s.construct_signatures().first().cloned())
            .and_then(|sig| self.get_return_type_of_signature(&sig))
            .unwrap_or_else(|| self.get_any_type());
        let tp_types: Vec<Arc<Type>> = match &class_decl.data {
            NodeData::ClassDeclaration(d) => match &d.type_parameters {
                Some(tps) => {
                    let sym_map = self.program.symbol_map();
                    let tp_syms: Vec<Arc<Symbol>> = tps
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect();
                    tp_syms
                        .iter()
                        .map(|s| self.get_type_parameter_from_symbol(s))
                        .collect()
                }
                None => Vec::new(),
            },
            _ => Vec::new(),
        };
        if tp_types.is_empty() {
            return instance_type;
        }
        let any_t = self.get_any_type();
        let anys: Vec<Arc<Type>> = tp_types.iter().map(|_| Arc::clone(&any_t)).collect();
        self.substitute_infer_type_parameters(&instance_type, &tp_types, &anys)
    }

    /// Build a `TypeParameter` type from a `TypeParameter` symbol, resolving
    /// its constraint (if any) from the declaration.
    fn get_type_parameter_from_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // Cached parameter type stored on the symbol's links.
        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(ref t) = links.declared_type {
                return Arc::clone(t);
            }
        }
        let sym_key = Arc::as_ptr(symbol) as usize;
        if !self.type_parameter_resolving.insert(sym_key) {
            // The constraint references the parameter itself (e.g.
            // `T extends Array<T>`, `A extends Attributes<keyof A>`). Go's
            // type-parameter types exist independently of their
            // constraints (a lazy link), so the inner query yields the
            // parameter without resolving anything; we approximate with a
            // fresh unconstrained placeholder carrying the same symbol.
            // Genuine circularity is detected AFTER resolution by walking
            // the constraint chain (see below), mirroring Go's
            // pushTypeResolution cycle marking for ResolvedBaseConstraint.
            return Arc::new(Type {
                flags: TypeFlags::TypeParameter,
                object_flags: ObjectFlags::None,
                id: 0,
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::TypeParameter(TypeParameterData {
                    constrained: ConstrainedTypeData::default(),
                    constraint: None,
                    target: None,
                    mapper: None,
                    is_this_type: false,
                    resolved_default_type: OnceLock::new(),
                }),
            });
        }
        let mut constraint: Option<Arc<Type>> = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeParameterDeclaration(data) = &decl.data {
                if let Some(constraint_node) = &data.constraint {
                    constraint = Some(self.get_type_from_type_node(constraint_node));
                }
                break;
            }
        }
        // TS2313: the constraint chain must actually cycle back to this
        // parameter (`T extends T`, `T extends U extends T`). A constraint
        // that merely REFERENCES the parameter through a proper type
        // (`Array<T>`) is legal — Go resolves those lazily without error.
        // A circular chain behaves as unconstrained (Go's
        // `circularConstraintType` → `getBaseConstraintOfType` nil).
        if let Some(c) = &constraint
            && self.constraint_chain_is_circular(sym_key, c)
        {
            if self.ts2313_reported.insert(sym_key) {
                let loc = symbol
                    .declarations
                    .iter()
                    .find_map(|d| match &d.data {
                        NodeData::TypeParameterDeclaration(td) => {
                            td.constraint.as_ref().map(|cn| cn.loc)
                        }
                        _ => None,
                    })
                    .or_else(|| symbol.declarations.first().map(|d| d.loc))
                    .unwrap_or_default();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    crate::diagnostics::messages_generated::
                        TYPE_PARAMETER_0_HAS_A_CIRCULAR_CONSTRAINT,
                    vec![symbol.name.clone()],
                ));
            }
            constraint = None;
        }
        let tp = Arc::new(Type {
            flags: TypeFlags::TypeParameter,
            object_flags: ObjectFlags::None,
            id: 0,
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::TypeParameter(TypeParameterData {
                constrained: ConstrainedTypeData::default(),
                constraint,
                target: None,
                mapper: None,
                is_this_type: false,
                resolved_default_type: OnceLock::new(),
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&tp));
        self.type_parameter_resolving.remove(&sym_key);
        tp
    }

    /// Whether following `constraint` (a chain of type-parameter
    /// constraints) revisits `start_key` — i.e. the chain is circular.
    /// Unconstrained links only count when they ARE the starting
    /// parameter's placeholder (the cycle-break marker above): a genuinely
    /// unconstrained `U extends T` terminates the walk.
    fn constraint_chain_is_circular(&self, start_key: usize, constraint: &Arc<Type>) -> bool {
        let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut current = constraint;
        for _ in 0..50 {
            let TypeData::TypeParameter(tp) = &current.data else {
                return false;
            };
            let Some(sym) = &current.symbol else { return false };
            let key = Arc::as_ptr(sym) as usize;
            if !visited.insert(key) {
                return true;
            }
            match &tp.constraint {
                Some(next) => current = next,
                None => return key == start_key,
            }
        }
        false
    }

    /// Resolve a namespace (`namespace N { ... }`) symbol to an anonymous
    /// object type whose properties are the namespace's exported members.
    /// Mirrors the namespace slice of Go's `getDeclaredTypeOfSymbol`.
    ///
    /// Each entry in the namespace symbol's `exports` table becomes a
    /// property on the resulting anonymous object type; the property type
    /// is resolved from the member symbol via `get_type_of_symbol` (which
    /// handles variables, functions, classes, etc.). Merged namespaces
    /// already share a single symbol (see `can_merge_symbols`), so all
    /// exported members from every merged declaration are visible here.
    /// Only exported members are accessible via `N.x` from outside the
    /// namespace — non-exported members live in the namespace's `locals`
    /// and are visible only inside the namespace body.
    pub fn resolve_namespace_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // Reuse a cached declared type if present.
        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }
        // Collect exported member symbols first to avoid borrowing
        // `self.exports` while calling `get_type_of_symbol` (which needs
        // `&mut self`).
        let mut members: Vec<(String, Arc<Symbol>)> = symbol
            .exports
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        // Ambient namespaces in an implicit-EXPORT context (Go
        // `setExportContextFlag`: ambient + no `export {}` declarations)
        // expose their non-exported locals too — declarations are
        // implicitly exported there (`declare module "M" { export
        // namespace N { function f(): C; } }` → `M.N.f()` is legal from
        // outside). Checker-side merge, matching the
        // `ambient_namespace_local` fallback used by identifier-member
        // resolution.
        if self.ambient_namespace_locals_visible(symbol) {
            let local_members: Vec<(String, Arc<Symbol>)> = symbol
                .declarations
                .iter()
                .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                .filter_map(|d| {
                    self.program
                        .symbol_map()
                        .locals
                        .get(&d.id())
                        .map(|l| {
                            l.iter()
                                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                                .collect::<Vec<(String, Arc<Symbol>)>>()
                        })
                })
                .flatten()
                .collect();
            for (k, v) in local_members {
                if !members.iter().any(|(mk, _)| *mk == k) {
                    members.push((k, v));
                }
            }
        }
        // FILE modules: the binder routes top-level declarations into the
        // file's locals (export visibility rides on modifiers / clauses),
        // so the exports table alone misses them — recover the export face
        // from the statement list (the same source the per-name module
        // member queries use). Ambient namespaces keep their populated
        // exports table and are untouched.
        let mut file_exported: Vec<(String, Arc<Symbol>)> = Vec::new();
        if symbol
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
            && members.is_empty()
        {
            let sym_map = self.program.symbol_map();
            // (exported name, clause-local name or None for direct decls)
            let mut wanted: Vec<(String, Option<String>)> = Vec::new();
            let mut default_node: Option<Arc<Node>> = None;
            self.for_each_module_statement(symbol, |stmt| {
                let has_export =
                    stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
                match &stmt.data {
                    NodeData::ExportDeclaration(d) => {
                        if let Some(clause) = &d.export_clause
                            && let NodeData::NamedExports(ne) = &clause.data
                        {
                            for el in ne.elements.iter() {
                                if let NodeData::ExportSpecifier(spec) = &el.data {
                                    let exported = spec
                                        .name
                                        .text()
                                        .trim_matches(['"', '\'', '`'])
                                        .to_string();
                                    let local = spec
                                        .property_name
                                        .as_ref()
                                        .unwrap_or(&spec.name)
                                        .text()
                                        .trim_matches(['"', '\'', '`'])
                                        .to_string();
                                    wanted.push((exported, Some(local)));
                                }
                            }
                        }
                    }
                    NodeData::ExportAssignment(ea) => {
                        if !ea.is_export_equals && default_node.is_none() {
                            default_node = Some(Arc::clone(stmt));
                        }
                    }
                    NodeData::VariableStatement(vs) if has_export => {
                        if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                            for decl in vdl.declarations.iter() {
                                if let Some(name) = decl.name() {
                                    wanted.push((name.text().to_string(), None));
                                }
                            }
                        }
                    }
                    _ if has_export => {
                        if let Some(name) = stmt.name() {
                            wanted.push((name.text().to_string(), None));
                        }
                    }
                    _ => {}
                }
                false
            });
            let file_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SourceFile);
            let locals = file_node.and_then(|f| sym_map.locals.get(&f.id()));
            for (exported, clause_local) in wanted.iter() {
                if members.iter().any(|(k, _)| *k == *exported)
                    || file_exported.iter().any(|(k, _)| *k == *exported)
                {
                    continue;
                }
                let lookup = clause_local.as_deref().unwrap_or(exported);
                if let Some(s) = locals.and_then(|l| l.get(lookup).cloned()) {
                    file_exported.push((exported.clone(), s));
                } else if let Some(s) = symbol.members.get(lookup).cloned() {
                    file_exported.push((exported.clone(), s));
                }
            }
            if let Some(node) = default_node
                && !members.iter().any(|(k, _)| k == "default")
                && !file_exported.iter().any(|(k, _)| k == "default")
            {
                // `export default <expr>`: the assignment's own symbol (the
                // binder declares "default" on the statement); fall back to
                // the exported expression's declaration symbol.
                let s = sym_map
                    .symbol_of(&node)
                    .cloned()
                    .or_else(|| node.expression().and_then(|e| sym_map.symbol_of(e).cloned()));
                if let Some(s) = s {
                    file_exported.push(("default".to_string(), s));
                }
            }
        }
        // Cross-file re-export clauses (`export { x } from "..."`) never
        // materialize in the exports table (the binder leaves the alias
        // unlinked) — chase them through the member resolver so the
        // namespace's property face includes forwarded names
        // (`(await import("inner")).x`).
        let mut reexported: Vec<(String, Arc<Symbol>)> = Vec::new();
        {
            let mut clause_specs: Vec<(String, String, String)> = Vec::new();
            self.for_each_module_statement(symbol, |stmt| {
                if let NodeData::ExportDeclaration(d) = &stmt.data
                    && let Some(clause) = &d.export_clause
                    && let NodeData::NamedExports(ne) = &clause.data
                    && let Some(module_spec) = &d.module_specifier
                {
                    for el in ne.elements.iter() {
                        if let NodeData::ExportSpecifier(spec) = &el.data {
                            let exported = spec
                                .name
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let imported = spec
                                .property_name
                                .as_ref()
                                .unwrap_or(&spec.name)
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let module_text = module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            if !exported.is_empty() && !module_text.is_empty() {
                                clause_specs.push((exported, imported, module_text));
                            }
                        }
                    }
                }
                false
            });
            for (exported, imported, module_text) in clause_specs {
                if members.iter().any(|(k, _)| *k == exported)
                    || reexported.iter().any(|(k, _)| *k == exported)
                {
                    continue;
                }
                let target = self
                    .resolve_module_spec_from(symbol, &module_text)
                    .and_then(|m| self.resolve_module_member_symbol(&m, &imported, 8));
                if let Some(t) = target {
                    reexported.push((exported, t));
                }
            }
        }
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for (name, member_sym) in members.iter().chain(file_exported.iter()).chain(reexported.iter()) {
            // Skip compiler-internal symbols only. Internal names carry the
            // `\u{FE}` prefix (INTERNAL_SYMBOL_NAME_PREFIX, mirroring Go's
            // InternalSymbolName prefix); user code may legally export
            // `__`-prefixed names (the official suites' `__val__*` pattern),
            // and `export=` is the module-export-assignment slot.
            if name.starts_with(crate::ast::INTERNAL_SYMBOL_NAME_PREFIX)
                || name == crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            {
                continue;
            }
            let member_type = self.get_type_of_symbol(member_sym);
            // Build a property symbol carrying the resolved member type so
            // `has_property_of_type` can find it by name.
            let prop_sym = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
            self.value_symbol_links.insert(
                &prop_sym,
                ValueSymbolLinks {
                    resolved_type: Some(member_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name.clone(), Arc::clone(&prop_sym));
            props.push(prop_sym);
        }
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&result));
        result
    }

    fn get_type_from_type_query_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_query(node);
        self.cache_type(node, result.clone());
        result
    }

    /// `typeof X` — the type of the VALUE `X`. For a class reference this is
    /// the class's constructor type (construct signatures + statics), which
    /// The TARGET symbol of an import alias, for type-position resolution:
    /// `import D from "m"` (ImportClause — member "default") and
    /// `import { X } from "m"` (ImportSpecifier — the property name).
    /// Other alias forms (`import * as NS`, `import x = require("m")`)
    /// delegate to `resolve_alias_base`.
    fn resolve_import_alias_target_symbol(
        &mut self,
        alias: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let (member_name, import_decl): (String, Arc<Node>) = {
            let decl = alias
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::ImportClause | SyntaxKind::ImportSpecifier
                    )
                })?
                .clone();
            match &decl.data {
                NodeData::ImportClause(_) => ("default".to_string(), decl),
                NodeData::ImportSpecifier(d) => (
                    d.property_name
                        .as_ref()
                        .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
                    decl,
                ),
                _ => return None,
            }
        };
        let mut import_decl = import_decl.parent.as_ref()?;
        while !matches!(import_decl.data, NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent.as_ref()?;
        }
        let module_spec = match &import_decl.data {
            NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
            _ => return None,
        };
        let module_sym = self
            .resolve_module_file_symbol(&module_spec)
            .or_else(|| {
                let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let cur = self.current_file.clone()?;
                let path = self.program.resolve_external_module_path(
                    &trimmed,
                    &cur.file_name,
                    crate::core::compiler_options::ModuleKind::None,
                )?;
                let sf = self.program.get_source_file(&path)?;
                self.program.symbol_map().symbol_of(&sf.node).cloned()
            })?;
        let resolved = self
            .resolve_module_member_symbol(&module_sym, &member_name, 8)
            .or_else(|| {
                // `export default` of a file whose exports table is empty:
                // recover the default from the assignment node's symbol
                // (resolve_namespace_type's file-face reconstruction).
                self.file_module_exported_member(&module_sym, &member_name)
            });
        let resolved = match resolved {
            Some(t)
                // A value/alias-flavored `default` (the binder's export-
                // assignment alias, or an `export default <identifier>`
                // re-export) — follow the alias chain to the named
                // declaration so a default INTERFACE keeps its type meaning
                // (TS2749 regression guard, allowSyntheticDefaultImports
                // CanPaintCrossModuleDeclaration).
                if !t.flags.intersects(
                    crate::ast::SymbolFlags::Interface
                        | crate::ast::SymbolFlags::TypeAlias
                        | crate::ast::SymbolFlags::Class
                        | crate::ast::SymbolFlags::ENUM
                        | crate::ast::SymbolFlags::TypeParameter,
                ) =>
            {
                let mut cur = Arc::clone(&t);
                for _ in 0..4 {
                    if cur.flags != crate::ast::SymbolFlags::Alias {
                        break;
                    }
                    // `export default X` / `export = X`: resolve X's name in
                    // the module's own scope.
                    let next = cur
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .and_then(|d| match &d.data {
                            NodeData::ExportAssignment(ea)
                                if matches!(
                                    ea.expression.kind,
                                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                ) =>
                            {
                                Some(ea.expression.text().to_string())
                            }
                            _ => None,
                        })
                        .and_then(|n| {
                            module_sym
                                .members
                                .get(&n)
                                .cloned()
                                .or_else(|| module_sym.exports.get(&n).cloned())
                        });
                    match next {
                        Some(n) => cur = n,
                        None => break,
                    }
                }
                let has_type_meaning = cur.flags.intersects(
                    crate::ast::SymbolFlags::Interface
                        | crate::ast::SymbolFlags::TypeAlias
                        | crate::ast::SymbolFlags::Class
                        | crate::ast::SymbolFlags::ENUM
                        | crate::ast::SymbolFlags::TypeParameter,
                );
                if has_type_meaning {
                    Some(cur)
                } else {
                    Some(t)
                }
            }
            other => other,
        };
        resolved
    }

    /// One member of a FILE module's reconstructed export face (see
    /// `resolve_namespace_type`): direct exported declarations by name and
    /// the `export default` assignment's symbol.
    fn file_module_exported_member(
        &self,
        module_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if !module_sym
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
        {
            return None;
        }
        if let Some(s) = module_sym.exports.get(name) {
            return Some(Arc::clone(s));
        }
        let sym_map = self.program.symbol_map();
        let mut found: Option<Arc<Symbol>> = None;
        self.for_each_module_statement(module_sym, |stmt| {
            match &stmt.data {
                NodeData::ExportAssignment(ea) => {
                    if !ea.is_export_equals && name == "default" && found.is_none() {
                        // `export default <expr>`: the assignment's own
                        // symbol; for a bare IDENTIFIER re-export, resolve
                        // the name in the module's scope first so a default
                        // INTERFACE (`export default Color`) keeps its type
                        // meaning (otherwise a value-flavored assignment
                        // symbol misreports TS2749 on type uses).
                        let by_name = match &ea.expression.kind {
                            SyntaxKind::Identifier => module_sym
                                .members
                                .get(ea.expression.text())
                                .cloned()
                                .or_else(|| module_sym.exports.get(ea.expression.text()).cloned()),
                            _ => None,
                        };
                        found = by_name.or_else(|| {
                            sym_map
                                .symbol_of(stmt)
                                .cloned()
                                .or_else(|| {
                                    stmt.expression()
                                        .and_then(|e| sym_map.symbol_of(e).cloned())
                                })
                        });
                    }
                }
                NodeData::VariableStatement(vs) => {
                    if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                        for decl in vdl.declarations.iter() {
                            if decl.name().is_some_and(|n| n.text() == name) {
                                found = sym_map.symbol_of(decl).cloned();
                            }
                        }
                    }
                }
                _ => {
                    if stmt.name().is_some_and(|n| n.text() == name)
                        && (stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
                            || stmt
                                .has_syntactic_modifier(crate::ast::ModifierFlags::Default))
                    {
                        found = sym_map.symbol_of(stmt).cloned();
                    }
                }
            }
            false
        });
        found
    }

    /// `typeof X` — the type of the VALUE `X`. For a class reference this is
    /// the class's constructor type (construct signatures + statics), which
    /// is what makes `declare const c: typeof A | typeof B` unions
    /// constructable/abstract-checkable. Mirrors Go's
    /// `getTypeFromTypeQueryNode` (identifier case).
    fn resolve_type_query(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let NodeData::TypeQueryNode(d) = &node.data else {
            return self.error_type();
        };
        // Qualified `typeof a.b` — resolve the entity name to a symbol,
        // then its value/constructor type.
        fn report_unresolved(c: &mut Checker, seg: &Arc<Node>) {
            if c.ts2304_reporting_allowed_for(seg) {
                use crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                let file = c
                    .get_source_file_of_node(seg)
                    .or_else(|| c.current_file.clone());
                c.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    seg.loc,
                    CANNOT_FIND_NAME_0,
                    vec![seg.text().to_string()],
                ));
            }
        }
        let symbol = if d.expr_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier(&d.expr_name) {
                Some(s) => s,
                None => {
                    report_unresolved(self, &d.expr_name);
                    return self.error_type();
                }
            }
        } else {
            match self.resolve_qualified_symbol(&d.expr_name) {
                Some(s) => s,
                None => {
                    report_unresolved(self, &d.expr_name);
                    return self.error_type();
                }
            }
        };
        // Import aliases (`import x = require("m")`, `import { y } from
        // "m"`): the alias's value type is the imported module instance /
        // member — resolve through the shared import-symbol typing path so
        // `typeof x` sees the real namespace (an uncached alias here would
        // otherwise fall to errorType and silently pass every check).
        if symbol.flags == crate::ast::SymbolFlags::Alias {
            // TS2708: `typeof a` where the alias targets a NON-instantiated
            // namespace queries a value the namespace doesn't have — the
            // written name is reported (Go resolves the typeof operand with
            // VALUE meaning, hitting the same namespace-as-value check as a
            // plain identifier use).
            if d.expr_name.kind == SyntaxKind::Identifier {
                let base = self.resolve_alias_base(Arc::clone(&symbol));
                let is_true_namespace = base.declarations.iter().any(|dd| {
                    dd.kind == SyntaxKind::ModuleDeclaration
                        && dd.name().is_some_and(|n| {
                            !matches!(n.kind, SyntaxKind::StringLiteral)
                        })
                });
                if base.flags.contains(crate::ast::SymbolFlags::ValueModule)
                    && is_true_namespace
                    && !self.namespace_usable_as_value(&base)
                {
                    let file = self
                        .get_source_file_of_node(&d.expr_name)
                        .or_else(|| self.current_file.clone());
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        d.expr_name.loc,
                        crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                        vec![d.expr_name.text().to_string()],
                    ));
                    return self.error_type();
                }
            }
            if let Some(t) = self.type_of_imported_symbol(&symbol) {
                return t;
            }
        }
        // TS2708 for a DIRECT namespace name: `var x: typeof M;` where M is
        // a non-instantiated namespace (InvalidNonInstantiatedModule).
        if symbol.flags.contains(crate::ast::SymbolFlags::ValueModule)
            && d.expr_name.kind == SyntaxKind::Identifier
            && symbol.declarations.iter().any(|dd| {
                dd.kind == SyntaxKind::ModuleDeclaration
                    && dd.name().is_some_and(|n| !matches!(n.kind, SyntaxKind::StringLiteral))
            })
            && !self.namespace_usable_as_value(&symbol)
        {
            let file = self
                .get_source_file_of_node(&d.expr_name)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                d.expr_name.loc,
                crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                vec![d.expr_name.text().to_string()],
            ));
            return self.error_type();
        }
        // A class reference yields the class (constructor) type; with type
        // arguments (`typeof Cls<number>`) the constructor object is
        // REBUILT with the class's type parameters substituted (Go's
        // `getInstantiationExpressionType` over the generic construct
        // signatures — class ctor sigs carry the class TPs in Go). The
        // class-declaration cache can't be reused here: it holds the
        // generic build; the instantiation is memoized separately (Go's
        // `instantiationExpressionTypes`).
        let class_decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned();
        if let Some(decl) = class_decl {
            if d.type_arguments.is_none() {
                return self.get_type_of_class_declaration(&decl);
            }
            let sym_map = self.program.symbol_map();
            let tp_symbols: Vec<Arc<crate::ast::Symbol>> = match &decl.data {
                NodeData::ClassDeclaration(cd) => match &cd.type_parameters {
                    Some(tps) => tps
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect(),
                    None => Vec::new(),
                },
                _ => Vec::new(),
            };
            let arg_types: Vec<Arc<Type>> = d
                .type_arguments
                .as_ref()
                .unwrap()
                .iter()
                .map(|a| self.get_type_from_type_node(a))
                .collect();
            if tp_symbols.is_empty() || arg_types.len() > tp_symbols.len() {
                return self.get_type_of_class_declaration(&decl);
            }
            let mut key = Vec::with_capacity(arg_types.len() + 1);
            key.push(decl.id() as usize);
            key.extend(
                arg_types
                    .iter()
                    .map(|t| Arc::as_ptr(t) as *const Type as usize),
            );
            if let Some(cached) = self.typequery_instantiation_cache.get(&key) {
                return Arc::clone(&cached.1);
            }
            let mut mapping = HashMap::new();
            for (i, tp) in tp_symbols.iter().enumerate() {
                if i < arg_types.len() {
                    mapping.insert(
                        Arc::as_ptr(tp) as *const crate::ast::Symbol,
                        Arc::clone(&arg_types[i]),
                    );
                }
            }
            self.type_argument_stack.push(mapping);
            let members = match &decl.data {
                NodeData::ClassDeclaration(data) => Arc::clone(&data.members),
                _ => unreachable!("class decl checked above"),
            };
            // Direct uncached build: the mapping substitutes the class's
            // type parameters in member annotations, the instance type,
            // and the synthesized construct signature's return.
            let built = if self.class_type_resolution_stack.contains(&decl.id()) {
                // Re-entrant build (self-referential class) — fall back to
                // the cached generic form to avoid a half-built shell.
                self.get_type_of_class_declaration(&decl)
            } else {
                self.class_type_resolution_stack.push(decl.id());
                let built = self.build_type_of_class_declaration(&decl, &members);
                self.class_type_resolution_stack.pop();
                built
            };
            self.type_argument_stack.pop();
            // The instantiated constructor object displays structurally
            // (`{ new (): Cls<…>; prototype: … }`), not as `typeof Cls` —
            // drop the display symbol the builder attached.
            let stripped = Arc::new(Type::new(
                built.flags,
                crate::checker::types::TypeData::Object(
                    match &built.data {
                        crate::checker::types::TypeData::Object(o) => ObjectTypeData {
                            structured: StructuredTypeData {
                                members: o.structured.members.clone(),
                                properties: o.structured.properties.clone(),
                                signatures: o.structured.signatures.clone(),
                                call_signature_count: o.structured.call_signature_count,
                                index_infos: o.structured.index_infos.clone(),
                                ..Default::default()
                            },
                            target: o.target.clone(),
                            mapper: o.mapper.clone(),
                            type_arguments: o.type_arguments.clone(),
                        },
                        _ => unreachable!("class build yields an object type"),
                    },
                ),
            ));
            {
                let t_mut = Arc::as_ptr(&stripped) as *mut Type;
                unsafe {
                    (*t_mut).object_flags =
                        built.object_flags | crate::checker::types::ObjectFlags::Instantiated;
                }
            }
            // Pin the argument types so the pointer key cannot be recycled.
            self.typequery_instantiation_cache
                .insert(key, (arg_types, Arc::clone(&stripped)));
            return stripped;
        }
        // Other values: reuse the symbol's resolved value type when present.
        let value_type = self
            .value_symbol_links
            .get(&symbol)
            .and_then(|l| l.resolved_type.clone());
        if let Some(t) = value_type {
            if d.type_arguments.is_some() {
                let arg_types: Vec<Arc<Type>> = d
                    .type_arguments
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect();
                return self.instantiate_value_type_for_type_query(&t, &arg_types);
            }
            return t;
        }
        self.error_type()
    }

    /// Go `getInstantiationExpressionType` (checker.go ~L10705), scoped to
    /// `typeof value<Args>` type queries: instantiate every APPLICABLE
    /// generic signature found in the value's type — a signature is
    /// applicable when it carries type parameters matching the argument
    /// arity; a CONSTRUCT signature whose return is a generic class
    /// instance additionally counts as applicable through the class's
    /// declared type parameters (Go's class ctor sigs carry them, ours
    /// don't — this branch supplies the equivalent). Intersections and
    /// unions map over their constituents; an object with instantiated
    /// signatures is re-shelled with the original members.
    fn instantiate_value_type_for_type_query(
        &mut self,
        base: &Arc<Type>,
        arg_types: &[Arc<Type>],
    ) -> Arc<Type> {
        if arg_types.is_empty() {
            return Arc::clone(base);
        }
        match &base.data {
            TypeData::Intersection(i) => {
                let parts: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|p| self.instantiate_value_type_for_type_query(p, arg_types))
                    .collect();
                self.get_intersection_type(parts)
            }
            TypeData::Union(u) => {
                let parts: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|p| self.instantiate_value_type_for_type_query(p, arg_types))
                    .collect();
                self.get_union_type(parts)
            }
            TypeData::Object(o) => {
                let call_count = o.structured.call_signature_count;
                let sigs = &o.structured.signatures;
                let mut changed = false;
                let mut new_sigs: Vec<Arc<crate::checker::types::Signature>> =
                    Vec::with_capacity(sigs.len());
                for (idx, sig) in sigs.iter().enumerate() {
                    let is_construct = idx >= call_count;
                    // (params, arity ok?) — either the signature's own TPs
                    // (generic function value) or, for construct sigs, the
                    // return's class type parameters.
                    let mut params: Vec<Arc<Type>> = sig.type_parameters.clone();
                    if params.is_empty() && is_construct {
                        if let Some(rt) = self.get_return_type_of_signature(sig)
                            && let Some(class_sym) = &rt.symbol
                        {
                            let class_tps = self.declared_type_parameter_types(class_sym);
                            if !class_tps.is_empty() {
                                params = class_tps;
                            }
                        }
                    }
                    if params.is_empty() || arg_types.len() > params.len() {
                        new_sigs.push(Arc::clone(sig));
                        continue;
                    }
                    let inst = self.get_signature_instantiation(sig, arg_types);
                    // The construct-return of a generic class is the class
                    // INSTANCE type — a named object our substitute walker
                    // leaves untouched by design. Deep-substitute its
                    // (eagerly cached) member types so the instantiation
                    // reaches `Cls<T>` members. The instantiated
                    // signature's return is an OnceLock already set by
                    // `get_signature_instantiation`, so the substituted
                    // return goes into a rebuilt signature.
                    let rt0 = self.get_return_type_of_signature(&inst);
                    let inst = match rt0 {
                        Some(rt0) => {
                            let deep = self
                                .substitute_object_properties_deep(&rt0, &params, arg_types);
                            let mut rebuilt = crate::checker::types::Signature::new();
                            rebuilt.flags = inst.flags;
                            rebuilt.min_argument_count = inst.min_argument_count;
                            rebuilt.resolved_min_argument_count =
                                inst.resolved_min_argument_count;
                            rebuilt.declaration = inst.declaration.clone();
                            rebuilt.parameters = inst.parameters.clone();
                            rebuilt.this_parameter = inst.this_parameter.clone();
                            rebuilt.resolved_type_predicate =
                                inst.resolved_type_predicate.clone();
                            rebuilt.target = inst.target.clone();
                            rebuilt.mapper = inst.mapper.clone();
                            rebuilt.instantiated_parameter_types =
                                inst.instantiated_parameter_types.clone();
                            if let Some(it) = inst.isolated_signature_type.get() {
                                let _ = rebuilt.isolated_signature_type.set(it.clone());
                            }
                            let _ = rebuilt.resolved_return_type.set(deep);
                            Arc::new(rebuilt)
                        }
                        None => inst,
                    };
                    changed = true;
                    new_sigs.push(inst);
                }
                if !changed {
                    return Arc::clone(base);
                }
                let shell = Arc::new(Type::new(
                    base.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData {
                            members: o.structured.members.clone(),
                            properties: o.structured.properties.clone(),
                            signatures: new_sigs,
                            call_signature_count: call_count,
                            index_infos: o.structured.index_infos.clone(),
                            ..Default::default()
                        },
                        target: o.target.clone(),
                        mapper: o.mapper.clone(),
                        type_arguments: o.type_arguments.clone(),
                    }),
                ));
                {
                    let t_mut = Arc::as_ptr(&shell) as *mut Type;
                    unsafe {
                        (*t_mut).object_flags =
                            base.object_flags | crate::checker::types::ObjectFlags::Instantiated;
                        (*t_mut).symbol = base.symbol.clone();
                    }
                }
                shell
            }
            _ => Arc::clone(base),
        }
    }

    fn get_type_from_array_or_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ArrayTypeNode(d) => {
                let elem_type = self.get_type_from_type_node(&d.element_type);
                self.create_array_type(elem_type)
            }
            NodeData::TupleTypeNode(d) => {
                let mut element_types = Vec::new();
                // Variadic (spread) element types feed Go's normalized-tuple
                // cross-product cap: `[...A | B, ...C | D]` would distribute
                // into a union of tuples sized by the product of the spread
                // unions (TS2590 when ≥ 100000).
                let mut variadic_types: Vec<Arc<Type>> = Vec::new();
                let mut has_variadic_union = false;
                for elem in d.elements.iter() {
                    if let NodeData::RestTypeNode(rd) = &elem.data {
                        let inner = Arc::clone(&rd.type_node);
                        let inner_t = self.get_type_from_type_node(&inner);
                        has_variadic_union |= inner_t.flags.contains(TypeFlags::Never)
                            || matches!(&inner_t.data, TypeData::Union(_));
                        variadic_types.push(inner_t);
                    }
                    element_types.push(self.get_type_from_type_node(elem));
                }
                if has_variadic_union
                    && !self.check_cross_product_union(node, &variadic_types)
                {
                    return self.error_type();
                }
                self.create_tuple_type(element_types)
            }
            _ => self.error_type(),
        }
    }

    /// TS2314 (arity) and TS2344 (constraint satisfaction) for a type
    /// reference to a generic interface/class/type-alias. Message formats
    /// follow the official baselines (which differ slightly from the
    /// current tsgo CLI): TS2314 names the generic WITH its parameter
    /// list ('G<T, U>'); a constraint failure caused by missing
    /// properties appends a TS2741-style elaboration chain.
    fn check_type_reference_arguments(
        &mut self,
        _node: &Arc<Node>,
        type_name: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {
        // Declared type parameters: the first declaration carrying a list.
        let params: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .find_map(|d| {
                let tps = match &d.data {
                    NodeData::InterfaceDeclaration(i) => i.type_parameters.as_ref(),
                    NodeData::ClassDeclaration(c) => c.type_parameters.as_ref(),
                    NodeData::TypeAliasDeclaration(t) => t.type_parameters.as_ref(),
                    _ => None,
                }?;
                Some(tps.iter().cloned().collect())
            })
            .unwrap_or_default();
        if params.is_empty() {
            return true;
        }
        let provided: Vec<Arc<Node>> = type_name
            .parent
            .as_ref()
            .and_then(|p| match &p.data {
                NodeData::TypeReferenceNode(tr) => tr.type_arguments.clone(),
                // Extends/implements clause members carry their type
                // arguments on the ExpressionWithTypeArguments node.
                NodeData::ExpressionWithTypeArguments(e) => e.type_arguments.clone(),
                _ => None,
            })
            .map(|list| list.iter().cloned().collect())
            .unwrap_or_default();
        // Params with defaults may be omitted from the tail.
        let required = params
            .iter()
            .rposition(|p| {
                !matches!(&p.data, NodeData::TypeParameterDeclaration(d) if d.default_type.is_some())
            })
            .map_or(0, |i| i + 1);
        let file = self
            .get_source_file_of_node(type_name)
            .or_else(|| self.current_file.clone());
        if provided.len() < required || provided.len() > params.len() {
            let display = format!(
                "{}<{}>",
                symbol.name,
                params
                    .iter()
                    .filter_map(|p| p.name().map(|n| n.text().to_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2314 && d.loc == type_name.loc);
            if !already {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    crate::diagnostics::messages_generated::GENERIC_TYPE_0_REQUIRES_1_TYPE_ARGUMENT_S,
                    vec![display, params.len().to_string()],
                ));
            }
            // Arity error → the reference resolves to the error type (Go's
            // `getTypeFromClassOrInterfaceReference` returns `c.errorType`
            // after reporting — consumers then skip dependent checks, e.g.
            // TS2564's any/error property-type exemption).
            return false;
        }
        // Constraint satisfaction for each provided argument.
        for (i, arg_node) in provided.iter().enumerate() {
            let Some(param) = params.get(i) else { continue };
            let NodeData::TypeParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let Some(constraint_node) = &pd.constraint else {
                continue;
            };
            // Constraints referencing the SAME declaration's type
            // parameters ('C<T>' in 'A<T, U extends C<T>>') require
            // instantiation with the earlier arguments — not available
            // here; skip to avoid false positives (left triaged).
            let param_names: Vec<String> = params
                .iter()
                .filter_map(|p| p.name().map(|n| n.text().to_string()))
                .collect();
            if type_node_references_names(constraint_node, &param_names) {
                continue;
            }
            let arg_type = self.get_type_from_type_node(arg_node);
            // Skip unresolved/deferred arguments: type parameters of an
            // enclosing generic (constraint is checked after substitution
            // in Go), error types, any, and never.
            if arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                || arg_type.is_type_parameter()
            {
                continue;
            }
            let constraint_type = self.get_type_from_type_node(constraint_node);
            if constraint_type.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
                continue;
            }
            if self.is_type_assignable_to(&arg_type, &constraint_type) {
                continue;
            }
            // Only report clear-cut failures where the relater's verdict is
            // trustworthy: primitive/literal arguments (TS2344 core case),
            // or object-vs-object missing-property failures (the elaborated
            // form). Composite shapes (tuples, function types, unions,
            // intersections, indexed accesses) depend on relater features
            // that are still being ported — defer those.
            let primitive_like = |t: &Arc<Type>| {
                t.flags.intersects(
                    TypeFlags::String
                        | TypeFlags::Number
                        | TypeFlags::Boolean
                        | TypeFlags::BigInt
                        | TypeFlags::ESSymbol
                        | TypeFlags::Enum
                        | TypeFlags::StringLiteral
                        | TypeFlags::NumberLiteral
                        | TypeFlags::BooleanLiteral
                        | TypeFlags::EnumLiteral
                        | TypeFlags::Null
                        | TypeFlags::Undefined,
                )
            };
            let object_like = |t: &Arc<Type>| {
                t.flags.contains(TypeFlags::Object)
                    && t.as_structured().is_some()
                    && !t.object_flags.contains(ObjectFlags::Tuple)
                    && !t.object_flags.contains(ObjectFlags::Reference)
                    && t.as_structured()
                        .is_some_and(|s| s.call_signature_count == 0)
            };
            let clear_cut = (primitive_like(&arg_type) && (primitive_like(&constraint_type) || object_like(&constraint_type)))
                || (object_like(&arg_type) && object_like(&constraint_type));
            if !clear_cut {
                continue;
            }
            // Suppress when either side came out of a heritage-degradation
            // window — the verdict against a transiently incomplete table
            // is garbage (see `degraded_type_ptrs`).
            {
                let ap = Arc::as_ptr(&arg_type) as *const Type as usize;
                let cp = Arc::as_ptr(&constraint_type) as *const Type as usize;
                if self.degraded_type_ptrs.contains(&ap)
                    || self.degraded_type_ptrs.contains(&cp)
                {
                    continue;
                }
            }
            let arg_str = self.type_to_string(&arg_type);
            let constraint_str = self.type_to_string(&constraint_type);
            let mut diag = crate::ast::Diagnostic::new(
                file.clone(),
                arg_node.loc,
                crate::diagnostics::messages_generated::TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1,
                vec![arg_str.clone(), constraint_str.clone()],
            );
            // Missing-property failures elaborate with the TS2741-style
            // chain entry, like the official baselines.
            let missing = self.get_missing_required_properties(&arg_type, &constraint_type);
            if missing.len() == 1 {
                diag.message_chain.push(crate::ast::Diagnostic::new(
                    None,
                    arg_node.loc,
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), arg_str.clone(), constraint_str.clone()],
                ));
            }
            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2344 && d.loc == arg_node.loc);
            if !already {
                self.diagnostics.add(diag);
            }
        }
        true
    }

    fn get_type_from_optional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("OptionalType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.add_optionality(&t)
    }

    fn get_type_from_union_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let types = match &node.data {
            NodeData::UnionTypeNode(data) => &data.types,
            _ => return self.error_type(),
        };
        let mut member_types = Vec::new();
        for member in types.iter() {
            member_types.push(self.get_type_from_type_node(member));
        }
        let result = self.get_union_type(member_types);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_intersection_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let types = match &node.data {
            NodeData::IntersectionTypeNode(data) => &data.types,
            _ => return self.error_type(),
        };
        let mut member_types = Vec::new();
        for member in types.iter() {
            member_types.push(self.get_type_from_type_node(member));
        }
        // Go `getIntersectionTypeEx` distributes an intersection of unions
        // into a cross-product union, capped at 100000 constituents (TS2590).
        // The all-undefined / all-null union shapes take special no-distribution
        // branches in Go and never reach the cross product.
        let has_union = member_types
            .iter()
            .any(|t| matches!(&t.data, TypeData::Union(_)));
        if has_union {
            let all_undefined = member_types.iter().all(|t| {
                let TypeData::Union(u) = &t.data else {
                    return false;
                };
                u.union_or_intersection
                    .types
                    .first()
                    .is_some_and(|f| f.flags.contains(TypeFlags::Undefined))
            });
            let all_null = member_types.iter().all(|t| {
                let TypeData::Union(u) = &t.data else {
                    return false;
                };
                u.union_or_intersection
                    .types
                    .iter()
                    .take(2)
                    .any(|f| f.flags.contains(TypeFlags::Null))
            });
            if !all_undefined
                && !all_null
                && !self.check_cross_product_union(node, &member_types)
            {
                let e = self.error_type();
                self.cache_type(node, e.clone());
                return e;
            }
        }
        let result = self.get_intersection_type(member_types);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_named_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("NamedTupleMember has type").clone();
        self.get_type_from_type_node(&inner)
    }

    fn get_type_from_rest_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("RestType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.create_array_type(t)
    }

    fn get_type_from_type_literal_or_function_or_constructor_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        // Reserve a cache slot first to break cycles on recursive types
        // like `type Box = { value: number; next: Box | null }`. During
        // member resolution a back-reference to this node returns the
        // reserved `error_type` (≈ `any`) instead of infinite-looping.
        self.cache_type(node, self.error_type());
        let result = match &node.data {
            NodeData::TypeLiteralNode(data) => {
                // Reuse the interface member builder — it covers ALL member
                // kinds (call/construct signatures, method signatures,
                // optionality), while the old literal-only walker skipped
                // signatures (`{ (n: number): string }` lost its call
                // signature → false TS2349).
                self.build_interface_type_from_members(&data.members)
            }
            NodeData::FunctionTypeNode(_) => self.get_type_from_function_type_node(node),
            NodeData::ConstructorTypeNode(_) => self.get_type_from_constructor_type_node(node),
            _ => self.error_type(),
        };
        self.cache_type(node, result.clone());
        result
    }

    /// Build an anonymous object type from a `TypeLiteral`'s member list
    /// (e.g. `{ a: number; b: string }`).
    ///
    /// Handles `PropertySignature` and `IndexSignature` members. Other
    /// member kinds (MethodSignature, CallSignature, ConstructSignature)
    /// are skipped — their support requires signature construction which is
    /// part of P3.7.
    fn get_type_from_type_literal_members(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();
        for member in members.iter() {
            match &member.data {
                NodeData::PropertySignatureDeclaration(data) => {
                    // `node.text()` returns the name for identifier/string/
                    // numeric literal names, and "" for computed property names.
                    let name = data.name.text().to_string();
                    if name.is_empty() {
                        continue;
                    }
                    let prop_type = self.get_type_from_type_node(&data.type_node);
                    let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::IndexSignatureDeclaration(data) => {
                    // `[key: string]: number` — extract key and value types.
                    let mut key_type = None;
                    let mut value_type = None;
                    if let Some(param) = data.parameters.iter().next() {
                        if let NodeData::ParameterDeclaration(pd) = &param.data {
                            key_type = pd
                                .type_node
                                .as_ref()
                                .map(|t| self.get_type_from_type_node(t));
                        }
                    }
                    value_type = Some(self.get_type_from_type_node(&data.type_node));
                    let is_readonly = member
                        .modifiers()
                        .as_ref()
                        .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
                    index_infos.push(Arc::new(crate::checker::IndexInfo {
                        key_type,
                        value_type,
                        is_readonly,
                        declaration: Some(Arc::clone(member)),
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }
                _ => {}
            }
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    /// Resolve a `FunctionType` node `(x: number) => string` to an anonymous
    /// object type with a single call signature. Parameter types and the
    /// return type are resolved from the AST. Rest parameters
    /// (`...args: number[]`) and optional parameters (`x?: number`) are
    /// honored via `SignatureFlags::HasRestParameter` and
    /// `min_argument_count` respectively. Generic type parameters
    /// (`<T>(x: T) => T`) are skipped for now (the signature is non-generic).
    /// Note: cache handling is done by the caller
    /// (`get_type_from_type_literal_or_function_or_constructor_type_node`).
    fn get_type_from_function_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::FunctionTypeNode(data) => {
                // Resolve parameter and return types WITHOUT suppressing
                // TS2304: unresolved names in a function-type annotation
                // (`(b: B) => C2`) are reported like any other type
                // reference (Go's checker resolves the signature parts with
                // the normal nameNotFoundMessage). Signature-level type
                // parameters (`<T>(x: T) => T`) live in the function-type
                // node's own locals (Go gives FunctionType HasLocals), so
                // the node is pushed as a scope around both parts.
                self.push_scope(node);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    /* is_construct */ false,
                    /* contextual_signature */ None,
                    /* declaration */ Some(Arc::clone(node)),
                );
                self.pop_scope();
                self.create_function_or_constructor_type(vec![sig], false)
            }
            _ => self.error_type(),
        }
    }

    /// Resolve a `ConstructorType` node `new (x: number) => Foo` to an
    /// anonymous object type with a single construct signature. Cache
    /// handling is done by the caller.
    fn get_type_from_constructor_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ConstructorTypeNode(data) => {
                // No TS2304 suppression — same policy as FunctionTypeNode
                // above: unresolved names in a constructor-type annotation
                // are reported like any other type reference. The node is
                // pushed as a scope: `new <T>(x: T) => T` type parameters
                // live in the constructor-type node's own locals (Go gives
                // ConstructorType HasLocals).
                self.push_scope(node);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    /* is_construct */ true,
                    /* contextual_signature */ None,
                    /* declaration */ Some(Arc::clone(node)),
                );
                self.pop_scope();
                self.create_function_or_constructor_type(vec![sig], true)
            }
            _ => self.error_type(),
        }
    }

    /// Build a `Signature` from a function-like type node's parameter list
    /// and a pre-resolved return type. Each parameter is turned into a
    /// `Symbol` whose resolved type is stored in `value_symbol_links` (so
    /// the relater's `get_type_of_symbol` returns it during signature
    /// comparison).
    ///
    /// The return type is passed in already-resolved (rather than as a type
    /// node) so that this helper can be shared by both type-annotation
    /// resolution (where the return type comes from a `TypeNode`) and
    /// function-expression type inference (where the return type is inferred
    /// from the body via `infer_function_return_type`).
    ///
    /// Whether `fn` is the callee of an immediately-invoked call with fewer
    /// arguments than the function's parameter count (Go
    /// `ast.GetImmediatelyInvokedFunctionExpression` + the
    /// `len(parameters) > len(iife.Arguments())` clause of the min-arity
    /// optionality rule). Such calls treat their untyped trailing
    /// parameters as optional.
    fn iife_with_too_few_arguments(
        declaration: &Option<Arc<Node>>,
        parameter_count: usize,
    ) -> bool {
        let Some(decl) = declaration else {
            return false;
        };
        if !matches!(
            decl.kind,
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
        ) {
            return false;
        }
        let mut prev: Arc<Node> = Arc::clone(decl);
        let mut parent: Option<Arc<Node>> = decl.parent.clone();
        while matches!(
            parent.as_ref().map(|p| p.kind),
            Some(SyntaxKind::ParenthesizedExpression)
        ) {
            prev = parent.clone().expect("checked Some above");
            parent = prev.parent.clone();
        }
        let Some(parent) = parent else {
            return false;
        };
        if parent.kind != SyntaxKind::CallExpression {
            return false;
        }
        let crate::ast::NodeData::CallExpression(call) = &parent.data else {
            return false;
        };
        // The callee must be the function's outermost paren wrapper (Go:
        // `parent.Expression() == prev`, no callee-side unwrapping).
        Arc::ptr_eq(&call.expression, &prev) && parameter_count > call.arguments.nodes.len()
    }

    /// `contextual_signature` carries the contextual function type's call
    /// signature (when the function expression is the initializer of a
    /// variable/parameter/property with a function-type annotation). When a
    /// parameter lacks an explicit type annotation, its type is taken from
    /// the corresponding position in the contextual signature — this is what
    /// makes `let f: (x: number) => number = (x) => x + 1;` type-check `x`
    /// as `number` inside the body. Parameters beyond the contextual
    /// signature's length (or with no contextual signature) fall back to
    /// `any`.
    pub fn build_signature_from_function_like_type_node(
        &mut self,
        parameters: &Arc<NodeList>,
        return_type: Arc<Type>,
        is_construct: bool,
        contextual_signature: Option<&Arc<Signature>>,
        declaration: Option<Arc<Node>>,
    ) -> Arc<Signature> {
        // Collect the declaration's type parameters (the `<T>` in
        // `function f<T>(x: T): T`). Each is resolved via
        // `get_type_parameter_from_symbol`, which caches the `TypeParameter`
        // type on the symbol — so the same `Arc<Type>` is shared by this
        // list and by type references in parameter/return annotations.
        // That pointer-equality is what lets call-site inference substitute
        // inferred type arguments into the signature. Resolved here (before
        // the parameter loop) so type-parameter constraint resolution, which
        // may walk the scope stack, runs while the caller's scope is pushed.
        let type_parameters = self.type_parameters_of_declaration(&declaration);
        let mut param_symbols: Vec<Arc<Symbol>> = Vec::with_capacity(parameters.len());
        let mut flags = SignatureFlags::None;
        if is_construct {
            flags |= SignatureFlags::Construct;
        }
        // `min_argument_count` = number of leading required (non-optional,
        // non-rest) parameters.
        let mut min_argument_count: i32 = 0;
        let mut reached_optional_or_rest = false;
        // A leading `this` parameter (Go `getSignatureFromDeclaration` /
        // `IsThisInTypeScript`) is stored separately on the signature — it
        // does not count toward arity and argument positions shift down.
        let mut this_parameter: Option<Arc<Symbol>> = None;
        for (i, param) in parameters.iter().enumerate() {
            let NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let is_rest = pd.dot_dot_dot_token.is_some();
            let is_optional = pd.question_token.is_some();
            let is_this_param = i == 0
                && !is_rest
                && matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this");
            // Resolve the parameter's type annotation. When no annotation is
            // present, fall back to the contextual signature's parameter
            // type at the same position (contextual typing for arrow/function
            // expression parameters), then to `any`.
            let param_type = match pd.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => {
                    let mut t = None;
                    if let Some(ctx_sig) = contextual_signature {
                        if i < ctx_sig.parameters.len() {
                            // Instantiated contextual signatures carry
                            // substituted parameter types in the override
                            // table (element-substituted array members).
                            t = self
                                .signature_instantiated_param_type(ctx_sig, i)
                                .or_else(|| {
                                    Some(self.get_type_of_symbol(&ctx_sig.parameters[i]))
                                });
                        }
                    }
                    t.unwrap_or_else(|| self.get_any_type())
                }
            };
            // Go `assignParameterType`: a `?`-marked parameter without an
            // initializer carries `| undefined` in its resolved type under
            // strictNullChecks — the signature's parameter type and every
            // body reference share it (the la5/2722/18048 chain and the
            // optional-params assignability family depend on this).
            let param_type = if pd.question_token.is_some() && pd.initializer.is_none() {
                self.add_optional_undefined(param_type)
            } else {
                param_type
            };
            // Use the parameter name when it's an identifier; otherwise
            // synthesize a positional name.
            let name = pd.name.text().to_string();
            let name = if name.is_empty() {
                format!("__arg{}", i)
            } else {
                name
            };
            // Prefer the binder's actual parameter symbol when present, so
            // that both the call signature and the function body share the
            // same symbol. The body resolves parameters via the scope stack
            // (which finds the binder's symbol), so storing the resolved
            // parameter type on that symbol makes `get_type_of_symbol` return
            // it from within the body too — this is what lets contextual
            // typing flow into the function body. For synthetic type-annotation
            // nodes (FunctionType/ConstructorType) that have no binder symbol,
            // fall back to a fresh `Property` symbol.
            let sym = match self.program.symbol_map().symbol_of(param) {
                Some(s) => Arc::clone(s),
                None => Arc::new(Symbol::new(SymbolFlags::Property, name)),
            };
            self.value_symbol_links.insert(
                &sym,
                ValueSymbolLinks {
                    resolved_type: Some(param_type),
                    ..Default::default()
                },
            );
            param_symbols.push(sym);
            if is_this_param && this_parameter.is_none() {
                // Strip the just-pushed `this` parameter from the parameter
                // list — it lives in `this_parameter` (arity/positions
                // exclude it).
                this_parameter = param_symbols.pop();
                continue;
            }
            if is_rest {
                flags |= SignatureFlags::HasRestParameter;
                reached_optional_or_rest = true;
            } else if is_optional
                || pd.initializer.is_some()
                || (pd.type_node.is_none()
                    && Self::iife_with_too_few_arguments(&declaration, parameters.len()))
            {
                // Go's optionality rules for min-arity (checker.go ~L19931):
                // `?`, an initializer, rest, or an untyped parameter of an
                // immediately-invoked function expression whose parameter
                // count exceeds the call's argument count — `((a) => {})()`
                // is legal, `((a: number) => {})()` is TS2554.
                reached_optional_or_rest = true;
            }
            if !reached_optional_or_rest {
                min_argument_count += 1;
            }
        }
        let sig = Arc::new(Signature {
            id: 0,
            flags,
            min_argument_count,
            resolved_min_argument_count: -1,
            declaration,
            type_parameters,
            parameters: param_symbols,
            this_parameter,
            resolved_return_type: std::sync::OnceLock::new(),
            resolved_type_predicate: None,
            target: None,
            mapper: None,
            isolated_signature_type: std::sync::OnceLock::new(),
            instantiated_parameter_types: None,
        });
        // Eagerly populate the resolved return type so
        // `get_return_type_of_signature` returns `Some(...)` without a
        // separate inference pass.
        let _ = sig.resolved_return_type.set(return_type);
        sig
    }

    /// Collect the type-parameter `Type`s declared by a function-like node
    /// (the `<T>` in `function f<T>(...)`). Returns an empty vec when
    /// `declaration` is `None` or has no type parameters. Mirrors the
    /// collection Go performs in `createSignature` /
    /// `getSignatureFromDeclaration`.
    fn type_parameters_of_declaration(
        &mut self,
        declaration: &Option<Arc<Node>>,
    ) -> Vec<Arc<Type>> {
        let Some(decl) = declaration else {
            return Vec::new();
        };
        let tp_list = match &decl.data {
            NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::FunctionExpression(d) => d.type_parameters.as_ref(),
            NodeData::ArrowFunction(d) => d.type_parameters.as_ref(),
            NodeData::MethodDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::MethodSignatureDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ConstructorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::SetAccessorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::FunctionTypeNode(d) => d.type_parameters.as_ref(),
            NodeData::ConstructorTypeNode(d) => d.type_parameters.as_ref(),
            // Interface/object-literal call & construct signatures carry
            // their own type parameters (`interface F { <T>(x: T): T }`) —
            // declared into the signature node's locals by the binder
            // (Go gives the signature kinds HasLocals). Without these arms
            // the signature was built NON-generic.
            NodeData::CallSignatureDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ConstructSignatureDeclaration(d) => d.type_parameters.as_ref(),
            _ => None,
        };
        let Some(list) = tp_list else {
            return Vec::new();
        };
        // Collect symbols first to avoid borrowing `self.program` (immutable)
        // across the mutable `get_type_parameter_from_symbol` call.
        let symbols: Vec<Arc<Symbol>> = list
            .iter()
            .filter_map(|tp| self.program.symbol_map().symbol_of(tp).map(Arc::clone))
            .collect();
        symbols
            .iter()
            .map(|s| self.get_type_parameter_from_symbol(s))
            .collect()
    }

    /// Create an anonymous object type with the given signatures. When
    /// `is_construct` is false, all signatures are call signatures
    /// (`call_signature_count = sigs.len()`); when true, all are construct
    /// signatures (`call_signature_count = 0`).
    pub fn create_function_or_constructor_type(
        &self,
        sigs: Vec<Arc<Signature>>,
        is_construct: bool,
    ) -> Arc<Type> {
        let call_signature_count = if is_construct { 0 } else { sigs.len() };
        let mut structured = StructuredTypeData::default();
        structured.signatures = sigs;
        structured.call_signature_count = call_signature_count;
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured,
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        })
    }

    fn get_type_from_type_operator_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = match &node.data {
            NodeData::TypeOperatorNode(data) => match data.operator {
                SyntaxKind::KeyOfKeyword => {
                    let arg_type = self.get_type_from_type_node(&data.type_node);
                    self.get_index_type(&arg_type)
                }
                SyntaxKind::UniqueKeyword => {
                    if data.type_node.kind == SyntaxKind::SymbolKeyword {
                        self.es_symbol_type()
                    } else {
                        self.error_type()
                    }
                }
                SyntaxKind::ReadonlyKeyword => {
                    let inner = self.get_type_from_type_node(&data.type_node);
                    // `readonly [A, B]` — flag the tuple so display and
                    // mutation checks see the readonly modifier.
                    if let TypeData::Tuple(tuple) = &inner.data {
                        if !tuple.readonly {
                            return Arc::new(Type {
                                flags: inner.flags,
                                object_flags: inner.object_flags,
                                id: 0,
                                symbol: None,
                                alias: None,
                                data: TypeData::Tuple(TupleTypeData {
                                    interface_data: InterfaceTypeData::default(),
                                    element_infos: tuple.element_infos.clone(),
                                    min_length: tuple.min_length,
                                    fixed_length: tuple.fixed_length,
                                    combined_flags: tuple.combined_flags,
                                    readonly: true,
                                }),
                            });
                        }
                    }
                    inner
                }
                _ => self.error_type(),
            },
            _ => self.error_type(),
        };
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_indexed_access_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = {
            let (object_type_node, index_type_node) = match &node.data {
                NodeData::IndexedAccessTypeNode(data) => {
                    (Arc::clone(&data.object_type), Arc::clone(&data.index_type))
                }
                _ => return self.error_type(),
            };
            let object_type = self.get_type_from_type_node(&object_type_node);
            let index_type = self.get_type_from_type_node(&index_type_node);
            // Go `getIndexedAccessTypeOrUndefined` (checker.go ~L27028) first
            // consults `shouldDeferIndexedAccessType` (~L27438): a generic
            // index type — or, in type position, a generic object type —
            // defers the access into an `IndexedAccess` type without
            // resolving properties and without reporting diagnostics (the
            // TS2536/TS4105 validation happens in the check phase via
            // `checkIndexedAccessIndexType`). Under instantiation the node
            // re-resolves with substituted (concrete) types and resolves
            // eagerly.
            if self.should_defer_indexed_access_type(&object_type, &index_type) {
                Arc::new(Type::new(
                    TypeFlags::IndexedAccess,
                    TypeData::IndexedAccess(IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: Some(Arc::clone(&object_type)),
                        index_type: Some(Arc::clone(&index_type)),
                        access_flags: AccessFlags::None,
                    }),
                ))
            } else {
                // Go `getIndexedAccessTypeOrUndefined`'s final else
                // (checker.go ~L27274): an index type that isn't
                // string/number/symbol-like at all cannot resolve — TS2538
                // at the index node (e.g. `any[[]]`, a tuple used as an
                // index). Deduped per node: re-resolution under an
                // enclosing instantiation reports through the same node
                // (Go caches the resolution on the node link).
                if !self.index_type_is_kind_usable(&index_type)
                    && self
                        .indexed_access_2538_reported
                        .insert(Arc::as_ptr(&index_type_node) as *const crate::ast::Node)
                {
                    // A degraded index/object type (heritage-degradation
                    // window) makes the kind verdict garbage — suppress
                    // (see `degraded_type_ptrs`).
                    let ip = Arc::as_ptr(&index_type) as *const Type as usize;
                    let op = Arc::as_ptr(&object_type) as *const Type as usize;
                    let degraded = self.degraded_type_ptrs.contains(&ip)
                        || self.degraded_type_ptrs.contains(&op);
                    if !degraded {
                        let type_str = if index_type_node.kind == SyntaxKind::BigIntLiteral {
                            "bigint".to_string()
                        } else {
                            self.type_to_string(&index_type)
                        };
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            index_type_node.loc,
                            crate::diagnostics::messages_generated::
                                TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                            vec![type_str],
                        ));
                    }
                }
                // NOTE: no further diagnostics here. Go reports type-position
                // index errors (TS2536/TS4105) from the check phase
                // (`checkIndexedAccessType` → `checkIndexedAccessIndexType`),
                // never during type resolution — resolution re-runs per
                // instantiation and would duplicate the error.
                self.get_indexed_access_type(&object_type, &index_type)
            }
        };
        self.cache_type(node, result.clone());
        result
    }

    /// Go `shouldDeferIndexedAccessType` (checker.go ~L27438), type-position
    /// branch (`IndexedAccessTypeNode`): a generic index type defers
    /// outright; otherwise a generic object type defers unless it is a
    /// tuple indexed by a numeric literal within the fixed element count
    /// (which resolves eagerly to the tuple element). Go's
    /// `isGenericReducibleType` (reducible union/intersection bookkeeping
    /// over `uniqueLiteralMapper` instantiations) has no equivalent here.
    fn should_defer_indexed_access_type(
        &self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> bool {
        if self.type_flags_is_generic_index_type(index_type) {
            return true;
        }
        if self.type_flags_is_generic_object_type(object_type) {
            if let TypeData::Tuple(tup) = &object_type.data {
                if index_type_less_than_fixed(index_type, tup.fixed_length) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Go `isTypeAssignableToKind(indexType, StringLike | NumberLike |
    /// ESSymbolLike)` (checker.go ~L27152): every union constituent must be
    /// assignable to one of the indexable primitive kinds — primitives are
    /// matched on flags directly; generic constituents (type parameters,
    /// indexed accesses, conditionals) go through the relater, which
    /// resolves their constraints.
    fn index_type_is_kind_usable(&mut self, t: &Arc<Type>) -> bool {
        let primitive_index_kinds = TypeFlags::from_bits_truncate(
            TypeFlags::Any.bits()
                | TypeFlags::Unknown.bits()
                | TypeFlags::Never.bits()
                | TypeFlags::String.bits()
                | TypeFlags::StringLiteral.bits()
                | TypeFlags::StringMapping.bits()
                | TypeFlags::TemplateLiteral.bits()
                | TypeFlags::Number.bits()
                | TypeFlags::NumberLiteral.bits()
                | TypeFlags::ESSymbol.bits()
                | TypeFlags::UniqueESSymbol.bits()
                | TypeFlags::Enum.bits()
                | TypeFlags::EnumLiteral.bits(),
        );
        let constituents: Vec<Arc<Type>> = if t.flags.contains(TypeFlags::Union) {
            t.types().map(|ts| ts.to_vec()).unwrap_or_default()
        } else {
            vec![Arc::clone(t)]
        };
        if constituents.is_empty() {
            return true;
        }
        for c in &constituents {
            if c.flags.intersects(primitive_index_kinds) {
                continue;
            }
            let ok = self.is_type_assignable_to(c, &self.string_type())
                || self.is_type_assignable_to(c, &self.number_type())
                || self.is_type_assignable_to(c, &self.es_symbol_type());
            if !ok {
                return false;
            }
        }
        true
    }

    fn get_type_from_template_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_template_literal_type(node);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_mapped_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_mapped_type(node);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_conditional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_conditional_type(node);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_infer_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        // TS1338 (Go `checkInferType`): an `infer` declaration is legal only
        // inside the extends clause of a conditional type — some node on the
        // chain starting AT the InferType node itself (Go's FindAncestor
        // includes the node; the InferType node often IS the extends node)
        // must be a ConditionalType's extends node.
        {
            let mut in_extends_clause = false;
            let mut cur = Some(Arc::clone(node));
            while let Some(n) = cur {
                if let Some(p) = n.parent.clone()
                    && let NodeData::ConditionalTypeNode(cd) = &p.data
                    && Arc::ptr_eq(&cd.extends_type, &n)
                {
                    in_extends_clause = true;
                    break;
                }
                cur = n.parent.clone();
            }
            if !in_extends_clause {
                let already = self
                    .diagnostics
                    .get_all()
                    .iter()
                    .any(|d| d.code == 1338 && d.loc.pos() == node.loc.pos());
                if !already {
                    self.grammar_error_on_node(
                        node,
                        &crate::diagnostics::messages_generated::
                            X_INFER_DECLARATIONS_ARE_ONLY_PERMITTED_IN_THE_EXTENDS_CLAUSE_OF_A_CONDITIONAL_TYPE,
                    );
                }
            }
        }
        // Mirrors Go's `getTypeFromInferTypeNode`: resolve to the declared
        // type of the infer type parameter's symbol.
        let result = {
            let tp_node = match &node.data {
                NodeData::InferTypeNode(data) => &data.type_parameter,
                _ => return self.error_type(),
            };
            let symbol = self.program.symbol_map().symbol_of(tp_node).map(Arc::clone);
            match symbol {
                Some(sym) => self.get_type_parameter_from_symbol(&sym),
                None => self.error_type(),
            }
        };
        self.cache_type(node, result.clone());
        result
    }

    /// Build a `Conditional` type from a `ConditionalTypeNode` and attempt to
    /// resolve it. Mirrors Go's `getTypeFromConditionalTypeNode` +
    /// `getConditionalType`.
    ///
    /// When the check type is concrete (no remaining type parameters), the
    /// conditional is resolved immediately to the true or false branch. When
    /// the check type is generic (deferred), the unresolved `Conditional`
    /// type is returned — it will be resolved later when the type
    /// parameters are substituted (e.g. during `is_type_assignable_to`).
    fn build_conditional_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (check_type_node, extends_type_node) = match &node.data {
            NodeData::ConditionalTypeNode(data) => {
                (Arc::clone(&data.check_type), Arc::clone(&data.extends_type))
            }
            _ => return self.error_type(),
        };

        let check_type = self.get_type_from_type_node(&check_type_node);
        let extends_type = self.get_type_from_type_node(&extends_type_node);

        // Collect `infer R` type parameters from the ConditionalType node's
        // locals. The binder declares them there (see `bind_type_parameter`).
        let infer_type_parameters = self.collect_infer_type_parameters(node);

        // Distributiveness is a property of the ROOT, evaluated on the check
        // type AS WRITTEN (Go: `isDistributive: checkType.flags&
        // TypeFlagsTypeParameter != 0` in getTypeFromConditionalTypeNode,
        // computed before any alias-instantiation mapping is applied). When
        // a generic alias like `Awaited<T> = T extends ... ? ... : ...` is
        // referenced with concrete arguments, our type_argument_stack is
        // already pushed at this point, so `check_type` above resolves to
        // the substituted (possibly union) type. Re-resolve the check node
        // with the substitution stack temporarily removed to recover the
        // naked type parameter and its symbol.
        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let saved_name_frames = std::mem::take(&mut self.type_argument_name_frames);
        let unmapped_check_type = self.get_type_from_type_node(&check_type_node);
        self.type_argument_stack = saved_stack;
        self.type_argument_name_frames = saved_name_frames;
        let is_distributive = unmapped_check_type.flags.contains(TypeFlags::TypeParameter);
        let check_type_parameter_symbol = if is_distributive {
            unmapped_check_type.symbol.clone()
        } else {
            None
        };

        let root = Box::new(ConditionalRoot {
            node: Some(Arc::clone(node)),
            check_type: Some(Arc::clone(&check_type)),
            extends_type: Some(Arc::clone(&extends_type)),
            is_distributive,
            check_type_parameter_symbol,
            infer_type_parameters: infer_type_parameters.clone(),
            outer_type_parameters: Vec::new(),
            alias: None,
            creation_scopes: self.scope_stack.clone(),
        });

        let cond_type = Arc::new(Type::new(
            TypeFlags::Conditional,
            TypeData::Conditional(ConditionalTypeData {
                constrained: ConstrainedTypeData::default(),
                root: Some(root),
                check_type: Some(Arc::clone(&check_type)),
                extends_type: Some(Arc::clone(&extends_type)),
                resolved_true_type: OnceLock::new(),
                resolved_false_type: OnceLock::new(),
                resolved_inferred_true_type: OnceLock::new(),
                resolved_default_constraint: OnceLock::new(),
                resolved_constraint_of_distributive: OnceLock::new(),
                mapper: None,
                combined_mapper: None,
                creation_type_argument_stack: self
                    .type_argument_stack
                    .iter()
                    .map(|frame| {
                        frame
                            .iter()
                            .map(|(k, v)| (*k as usize, Arc::clone(v)))
                            .collect::<HashMap<_, _>>()
                    })
                    .collect(),
            }),
        ));

        // Try to resolve the conditional immediately. If the check type is
        // still generic, `resolve_conditional_type` returns `None` and we
        // return the unresolved conditional type.
        if let Some(resolved) = self.resolve_conditional_type(&cond_type) {
            resolved
        } else {
            cond_type
        }
    }

    /// Collect the `infer R` type parameters declared as locals of a
    /// `ConditionalType` node. Mirrors Go's `getInferTypeParameters`.
    fn collect_infer_type_parameters(&mut self, node: &Arc<Node>) -> Vec<Arc<Type>> {
        // Collect the type-parameter symbols first to avoid holding an
        // immutable borrow of `self.program.symbol_map()` across the
        // mutable `get_type_parameter_from_symbol` call.
        let symbols: Vec<Arc<Symbol>> = self
            .program
            .symbol_map()
            .locals_of(node)
            .map(|locals| {
                locals
                    .iter()
                    .filter(|(_, sym)| sym.flags.contains(SymbolFlags::TypeParameter))
                    .map(|(_, sym)| Arc::clone(sym))
                    .collect()
            })
            .unwrap_or_default();
        symbols
            .into_iter()
            .map(|sym| self.get_type_parameter_from_symbol(&sym))
            .collect()
    }

    fn get_type_from_import_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        // Grammar check on the attribute clause (Go reports the
        // resolution-mode family errors from the import-type path with
        // reportErrors=true): TS2839/TS2862/TS1453 land on the clause
        // before any resolution is attempted.
        if let NodeData::ImportTypeNode(d) = &node.data
            && let Some(attrs) = &d.attributes
        {
            let attrs = Arc::clone(attrs);
            let _ = self.get_resolution_mode_override(&attrs, true);
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    // ────────────────────────────────────────────────────────────────────────
    // Type creation helpers
    // ────────────────────────────────────────────────────────────────────────

    /// Estimated size of the union a cross product over `types` would
    /// produce — Go `getCrossProductUnionSize`: union members contribute
    /// their constituent count, anything else contributes 1, and a `never`
    /// member collapses the whole product to 0.
    fn cross_product_union_size(types: &[Arc<Type>]) -> u64 {
        let mut size: u64 = 1;
        for t in types {
            if let TypeData::Union(u) = &t.data {
                size = size.saturating_mul(u.union_or_intersection.types.len() as u64);
            } else if t.flags.contains(TypeFlags::Never) {
                return 0;
            }
        }
        size
    }

    /// Go `checkCrossProductUnion`: before distributing a cross product of
    /// unions (template literal spans, variadic tuple elements, intersection
    /// members), verify the estimated union size stays under 100000
    /// constituents; on overflow report TS2590 at `node` and tell the caller
    /// to fall back to the error type. Repeated resolution of the same node
    /// must not re-report (the tuple/array arm has no per-node cache).
    fn check_cross_product_union(&mut self, node: &Arc<Node>, types: &[Arc<Type>]) -> bool {
        if Self::cross_product_union_size(types) < 100_000 {
            return true;
        }
        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == 2590 && d.loc.pos() == node.loc.pos());
        if !already {
            let file = self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    EXPRESSION_PRODUCES_A_UNION_TYPE_THAT_IS_TOO_COMPLEX_TO_REPRESENT,
                vec![],
            ));
        }
        false
    }

    /// Build the optional type `T | undefined` for a property `x?: T`.
    /// Mirrors Go's `getOptionalType`.
    pub fn get_optional_type(&mut self, t: Arc<Type>) -> Arc<Type> {
        self.get_union_type(vec![t, self.undefined_type()])
    }

    pub fn get_union_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.never_type();
        }
        // `never` constituents are absorbed by the union (`0 | never`
        // reduces to `0`) — mirrors tsc's `getUnionType`, which removes
        // never members unless the union would be empty. This matters for
        // flow junctions where dead branches (e.g. after `break`) narrow
        // to `never`; keeping them would poison comparability checks.
        let types: Vec<Arc<Type>> = types
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Never))
            .collect();
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        // Flatten nested unions and drop duplicate constituents (Go's
        // `getUnionType` deduplicates by type identity). A flow junction
        // that unions the pre-narrowing type (`number | null`) with the
        // narrowed one (`number`) must collapse to `number | null` — the
        // intrinsics are cached singletons, so pointer identity catches
        // them (destructuringTypeGuardFlow's chain-condition re-read
        // displayed `number | null | number` without this).
        let mut flattened: Vec<Arc<Type>> = Vec::with_capacity(types.len());
        for t in types {
            if let TypeData::Union(u) = &t.data {
                for inner in &u.union_or_intersection.types {
                    if !flattened.iter().any(|s| Arc::ptr_eq(s, inner)) {
                        flattened.push(Arc::clone(inner));
                    }
                }
            } else if !flattened.iter().any(|s| Arc::ptr_eq(s, &t)) {
                flattened.push(t);
            }
        }
        if flattened.is_empty() {
            return self.never_type();
        }
        if flattened.len() == 1 {
            return flattened.into_iter().next().expect("exactly one");
        }
        // Primitive-only unions sort by the intrinsic type id order
        // (Go's addTypesToUnion + type-id sort: `string | number`, never
        // `number | string`); unions with structural members keep their
        // insertion order.
        // Go orders union members by TypeFlags BIT VALUE
        // (`getSortOrderFlags` — Any 1, Unknown 2, Undefined 4, Null 8,
        // …), giving `string | number | boolean` and
        // `number | null | undefined`. Stable sort keeps insertion order
        // among equal ranks (object members by declaration position).
        {
            let rank = |t: &Arc<Type>| -> u32 {
                if t.flags
                    .intersects(TypeFlags::EnumLiteral | TypeFlags::Enum)
                {
                    return TypeFlags::Enum.bits();
                }
                let b = t.flags.bits();
                b & b.wrapping_neg()
            };
            flattened.sort_by_key(rank);
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: flattened,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub fn get_intersection_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.unknown_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Intersection,
            TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types,
                },
                resolved_apparent_type: std::sync::OnceLock::new(),
                unique_literal_filled_instantiation: std::sync::OnceLock::new(),
            }),
        ))
    }

    pub fn create_array_type(&mut self, element_type: Arc<Type>) -> Arc<Type> {
        // Bare reference shape with the Array symbol and the element as its
        // type argument (display prints `T[]`; `is_array_type` sees the
        // Reference flag). Array interface MEMBERS are never eagerly built
        // here — Go keeps instantiation lazy (deferred type references),
        // and eagerly resolving the ~40-member table per element type melts
        // down on default-lib scale (thousands of distinct elements, each
        // cascading into ConcatArray/ReadonlyArray instantiations). Member
        // types are element-substituted at property-access time instead —
        // see `instantiate_array_member_type`.
        let array_symbol = self.globals.get("Array").cloned();
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Reference,
            id: 0,
            symbol: array_symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData::default(),
                target: None,
                mapper: None,
                type_arguments: vec![element_type],
            }),
        })
    }

    /// The type-parameter symbols of the global `Array<T>` interface
    /// (lazily collected, then cached).
    pub(crate) fn array_type_parameter_symbols(&mut self) -> Vec<Arc<Symbol>> {
        if let Some(cached) = &self.array_type_parameter_symbols {
            return cached.clone();
        }
        let collected = self
            .globals
            .get("Array")
            .and_then(|sym| {
                let decl = sym
                    .declarations
                    .iter()
                    .find(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))?;
                let NodeData::InterfaceDeclaration(d) = &decl.data else {
                    return None;
                };
                let sym_map = self.program.symbol_map();
                Some(
                    d.type_parameters
                        .as_ref()?
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect::<Vec<_>>(),
                )
            })
            .unwrap_or_default();
        self.array_type_parameter_symbols = Some(collected.clone());
        collected
    }

    /// Element-substituted type of an `Array` interface member accessed on
    /// an array type (`arr.push`, `arr.every`, …): the raw member's type
    /// keeps the interface's type parameter free, so substitute it with the
    /// array's element type (Go resolves members through the instantiation
    /// mapper; we substitute at access time and memoize per
    /// (element, member)). Returns `None` when `member` doesn't come from
    /// the Array fallback or its type doesn't mention the type parameter
    /// (`length: number`).
    pub(crate) fn instantiate_array_member_type(
        &mut self,
        obj_type: &Arc<Type>,
        member: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        // Only the bare array shape (no structured members of its own);
        // evolving array literals (`const x = []` … `x.push(v)`) resolve
        // through the same table with their EVOLVED element union.
        let is_evolving = obj_type
            .object_flags
            .contains(ObjectFlags::EvolvingArray);
        if !self.is_array_type(obj_type) && !is_evolving {
            return None;
        }
        if let Some(structured) = obj_type.as_structured()
            && structured.members.get(&member.name).is_some()
        {
            // The member belongs to the type's own table, not the Array
            // fallback — nothing to substitute.
            return None;
        }
        let element = match &obj_type.data {
            TypeData::Object(o) => match o.type_arguments.first() {
                Some(e) => Arc::clone(e),
                None => return None,
            },
            TypeData::EvolvingArray(e) => e
                .element_type
                .clone()
                .unwrap_or_else(|| self.never_type()),
            _ => return None,
        };
        // Type the member through the DECLARED `Array<T>` type's structured
        // member table: those synthetic symbols were resolved IN the
        // interface's scope during declaration building, so `T` is the real
        // type parameter (the binder member symbol resolves `T` to any
        // outside that scope).
        let declared = match self
            .globals
            .get("Array")
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            }) {
            Some(d) => Some(d),
            // Not yet resolved: force the declared resolution now.
            None => self.globals.get("Array").cloned().map(|sym| {
                self.resolve_interface_type(&sym, None)
            }),
        };
        let raw = declared
            .as_ref()
            .and_then(|d| d.as_structured())
            .and_then(|s| s.members.get(&member.name).cloned())
            .map(|synthetic| self.get_type_of_symbol(&synthetic))?;
        let key = (
            Arc::as_ptr(&element) as *const crate::checker::types::Type as usize,
            Arc::as_ptr(member) as *const crate::ast::Symbol as usize,
        );
        if let Some(cached) = self.array_member_type_cache.get(&key) {
            return Some(Arc::clone(cached));
        }
        // Collect the FREE type parameters from the member's own
        // parameter/return types, recursing into function-type signatures
        // (a method like `map` keeps its type parameters inside the
        // CALLBACK's signature, which the shallow collector misses — the
        // raw member then leaks the interface's `T` into call targets).
        let mut free_tps: Vec<Arc<Type>> = Vec::new();
        for sig in self.get_signatures_of_type(&raw, SignatureKind::Call) {
            // Read parameter types from the parameter SYMBOLS directly
            // (`try_get_type_at_position` yields None for raw rest
            // parameters outside the override table).
            for param in &sig.parameters {
                let pt = self.get_type_of_symbol(param);
                self.collect_free_type_parameters_deep(&pt, &mut free_tps);
            }
            if let Some(rt) = self.get_return_type_of_signature(&sig) {
                self.collect_free_type_parameters_deep(&rt, &mut free_tps);
            }
        }
        // Substitute ONLY the interface's own type parameters with the
        // element type; the member's own type parameters (`map`'s `U`,
        // `flat`'s `A`/`D`) stay free so call-site inference can bind
        // them (Go resolves members through the instantiation mapper,
        // which maps the interface's parameters only).
        let array_tps = self.array_type_parameter_symbols();
        let subst_tps: Vec<Arc<Type>> = free_tps
            .iter()
            .filter(|tp| {
                tp.symbol
                    .as_ref()
                    .is_some_and(|s| array_tps.iter().any(|a| Arc::ptr_eq(a, s)))
            })
            .cloned()
            .collect();
        if subst_tps.is_empty() {
            // Non-generic member (`length: number`) or one that only
            // mentions its own type parameters — return its type as-is.
            return Some(raw);
        }
        let substitutions: Vec<Arc<Type>> = std::iter::repeat(Arc::clone(&element))
            .take(subst_tps.len())
            .collect();
        let substituted =
            self.substitute_infer_type_parameters(&raw, &subst_tps, &substitutions);
        self.array_member_type_cache
            .insert(key, Arc::clone(&substituted));
        Some(substituted)
    }

    /// A synthetic member symbol from the DECLARED `Array<T>` interface's
    /// structured member table, by name (`length`, `concat`, `join`, …).
    /// Bare array types resolve their properties through this table.
    pub(crate) fn declared_array_member_symbol(&mut self, name: &str) -> Option<Arc<Symbol>> {
        let array_sym = self.globals.get("Array").cloned();
        let declared = array_sym
            .as_ref()
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            })
            .or_else(|| {
                array_sym
                    .as_ref()
                    .map(|sym| self.resolve_interface_type(&sym, None))
            })?;
        declared
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
    }

    /// The DECLARED `Array<T>` interface's property list in declaration
    /// order — the member table a bare array TARGET exposes to structural
    /// comparison (Go `getPropertiesOfType` on `Array<T>` resolves the
    /// reference's declared members).
    pub(crate) fn declared_array_member_symbols(&mut self) -> Vec<Arc<Symbol>> {
        let array_sym = self.globals.get("Array").cloned();
        let declared = array_sym
            .as_ref()
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            })
            .or_else(|| {
                array_sym
                    .as_ref()
                    .map(|sym| self.resolve_interface_type(&sym, None))
            });
        declared
            .and_then(|t| t.as_structured().map(|s| s.properties.clone()))
            .unwrap_or_default()
    }

    /// A member of a global builtin interface (`Object`, `Function`) by
    /// name — the apparent-type fallbacks of Go `getPropertyOfTypeEx`
    /// (checker.go ~L18960): property lookup on any object type falls
    /// back to the global `Object` interface, and call/construct-typed
    /// sources first augment with the `Function` interface's members.
    pub(crate) fn global_interface_member_symbol(
        &mut self,
        interface_name: &str,
        member: &str,
    ) -> Option<Arc<Symbol>> {
        let sym = self.globals.get(interface_name).cloned()?;
        let declared = self
            .type_alias_links
            .get(&sym)
            .and_then(|l| l.declared_type.clone())
            .or_else(|| Some(self.resolve_interface_type(&sym, None)))?;
        declared
            .as_structured()
            .and_then(|s| s.members.get(member).cloned())
    }

    /// The type of `prop` as a member of the (possibly instantiated) type
    /// `owner`. Instantiated generic interfaces/classes carry their type
    /// arguments on the instance (`ConcatArray<number>`), but the synthetic
    /// member symbols' cached types keep the DECLARED type parameters —
    /// substitute them with the instance's arguments (Go reads member types
    /// through the instantiation mapper). Memoized per (owner, prop).
    pub(crate) fn substituted_member_type_of(
        &mut self,
        owner: &Arc<Type>,
        prop: &Arc<Symbol>,
    ) -> Arc<Type> {
        let Some(obj) = owner.as_object() else {
            return self.get_type_of_symbol(prop);
        };
        if obj.type_arguments.is_empty() {
            return self.get_type_of_symbol(prop);
        }
        let Some(owner_sym) = owner.symbol.clone() else {
            return self.get_type_of_symbol(prop);
        };
        let key = (
            Arc::as_ptr(owner) as *const crate::checker::types::Type as usize,
            Arc::as_ptr(prop) as *const crate::ast::Symbol as usize,
        );
        if let Some(cached) = self.instantiated_member_type_cache.get(&key) {
            return Arc::clone(&cached.1);
        }
        // Interfaces: read the member from a PROPERLY instantiated
        // instance (resolve_interface_type_ex substitutes member types at
        // construction through the type-argument stack) — substitution-
        // rebuilt references (e.g. the element-substituted Array members'
        // rest types) carry the declared member table with raw type
        // parameters. Classes have no such instantiation entry point and
        // fall back to substituting the raw member type.
        let result = if owner_sym.flags.contains(SymbolFlags::Interface) {
            let proper =
                self.resolve_interface_type_ex(&owner_sym, Some(obj.type_arguments.clone()));
            let prop_sym = proper
                .as_structured()
                .and_then(|s| s.members.get(&prop.name).cloned());
            match prop_sym {
                Some(ps) => self.get_type_of_symbol(&ps),
                None => self.get_type_of_symbol(prop),
            }
        } else {
            self.substitute_member_type_fallback(&owner_sym, prop, &obj.type_arguments)
        };
        // Capacity bound (see `instantiated_member_type_cache_limit`) —
        // same unbounded-growth hazard and same clear-at-cap remedy as
        // `type_node_subst_cache` above.
        if self.instantiated_member_type_cache.len() >= self.instantiated_member_type_cache_limit
        {
            self.instantiated_member_type_cache.clear();
        }
        // The owner Arc is pinned in the value so the pointer key can
        // never be recycled by a different type while the entry lives.
        self.instantiated_member_type_cache
            .insert(key, (Arc::clone(owner), Arc::clone(&result)));
        result
    }

    /// Fallback for non-interface owners (classes): substitute the raw
    /// member type's declaration type parameters with the instance's
    /// arguments.
    fn substitute_member_type_fallback(
        &mut self,
        owner_sym: &Arc<Symbol>,
        prop: &Arc<Symbol>,
        args: &[Arc<Type>],
    ) -> Arc<Type> {
        let decl_tps = self.declared_type_parameter_types(owner_sym);
        if decl_tps.len() == args.len() && !decl_tps.is_empty() {
            let raw = self.get_type_of_symbol(prop);
            let substitutions = args.to_vec();
            let r = self.substitute_infer_type_parameters(&raw, &decl_tps, &substitutions);
            r
        } else {
            self.get_type_of_symbol(prop)
        }
    }

    /// The type-parameter TYPES of a generic interface/class declaration
    /// (the first declaration's list — merged declarations must agree).
    pub(crate) fn declared_type_parameter_types(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Type>> {
        let decl = symbol.declarations.iter().find(|d| {
            matches!(
                d.data,
                NodeData::InterfaceDeclaration(_) | NodeData::ClassDeclaration(_)
            )
        });
        let Some(decl) = decl else {
            return Vec::new();
        };
        let tps = match &decl.data {
            NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
            _ => None,
        };
        let Some(tps) = tps else {
            return Vec::new();
        };
        let tp_syms: Vec<Arc<Symbol>> = {
            let sym_map = self.program.symbol_map();
            tps.iter()
                .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                .collect()
        };
        // Constraint resolution inside runs in the CALLER's scope — a
        // parameter's constraint may reference later-declared parameters
        // (`class Field<T extends TR, TR>`) whose symbols only resolve in
        // the declaration's own scope. Suppress TS2304 for the lookups;
        // constraints still resolve lazily through the symbol links.
        self.push_ts2304_suppression();
        let types = tp_syms
            .iter()
            .map(|tp_sym| self.get_type_parameter_from_symbol(tp_sym))
            .collect();
        self.pop_ts2304_suppression();
        types
    }

    /// Deep free-type-parameter collection: like
    /// `relater::collect_free_type_parameters`, but also recurses into
    /// structured types' call/construct signatures (parameter and return
    /// types), indexed accesses, and the object member tables that array
    /// method members are built from.
    fn collect_free_type_parameters_deep(&mut self, t: &Arc<Type>, out: &mut Vec<Arc<Type>>) {
        match &t.data {
            TypeData::TypeParameter(_) => {
                if !out.iter().any(|p| Arc::ptr_eq(p, t)) {
                    out.push(Arc::clone(t));
                }
            }
            TypeData::Union(u) => {
                for ty in &u.union_or_intersection.types {
                    self.collect_free_type_parameters_deep(ty, out);
                }
            }
            TypeData::Intersection(i) => {
                for ty in &i.union_or_intersection.types {
                    self.collect_free_type_parameters_deep(ty, out);
                }
            }
            TypeData::Object(o) => {
                for ty in &o.type_arguments {
                    self.collect_free_type_parameters_deep(ty, out);
                }
                // Function types: walk the signatures' parameter and return
                // types (the callback parameter of `map`/`forEach` keeps
                // the interface parameter inside its own signature).
                for sig in o.structured.signatures.clone() {
                    for param in sig.parameters.iter() {
                        let pt = self.get_type_of_symbol(param);
                        self.collect_free_type_parameters_deep(&pt, out);
                    }
                    if let Some(rt) = sig.resolved_return_type.get() {
                        let rt = Arc::clone(rt);
                        self.collect_free_type_parameters_deep(&rt, out);
                    }
                }
            }
            TypeData::Tuple(tu) => {
                for ei in &tu.element_infos {
                    if let Some(ty) = &ei.type_ {
                        self.collect_free_type_parameters_deep(ty, out);
                    }
                }
            }
            TypeData::IndexedAccess(ia) => {
                if let Some(obj) = &ia.object_type {
                    self.collect_free_type_parameters_deep(obj, out);
                }
                if let Some(idx) = &ia.index_type {
                    self.collect_free_type_parameters_deep(idx, out);
                }
            }
            _ => {}
        }
    }

    pub fn create_tuple_type(&mut self, element_types: Vec<Arc<Type>>) -> Arc<Type> {
        let element_infos: Vec<TupleElementInfo> = element_types
            .iter()
            .map(|t| TupleElementInfo {
                flags: ElementFlags::Required,
                labeled_declaration: None,
                type_: Some(Arc::clone(t)),
            })
            .collect();
        let fixed_length = element_types.len();
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Tuple,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Tuple(TupleTypeData {
                interface_data: InterfaceTypeData::default(),
                element_infos,
                min_length: fixed_length,
                fixed_length,
                combined_flags: ElementFlags::Required,
                readonly: false,
            }),
        })
    }

    /// Compute `keyof T` — the union of string-literal property names of `T`.
    ///
    /// Mirrors Go's `getIndexType`. Currently handles:
    /// - Object/Interface/Tuple/Mapped types: union of `properties[*].name`
    ///   as string-literal types. If there are no properties, returns `never`.
    /// - Union types: `keyof (A | B)` = `keyof A & keyof B` (common keys).
    /// - Intersection types: `keyof (A & B)` = `keyof A | keyof B`.
    /// - Type parameters: `keyof T` = `keyof Constraint<T>` (if constrained).
    /// - `never`: returns `never`.
    /// - `any`/`error`: returns `string` (simplified — Go returns
    ///   `string | number | symbol`).
    /// - Other types: `never`.
    pub fn get_index_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        // `keyof never` is `never`; `keyof any` is `string | number | symbol`
        // (approximated as `string` here).
        if t.flags.contains(TypeFlags::Never) {
            return self.never_type();
        }
        if t.flags.contains(TypeFlags::Any) {
            // `keyof any` = string | number | symbol. We approximate with
            // `string` since number/symbol literal keys are rare in tests.
            return self.string_type();
        }
        // Union: `keyof (A | B)` = `keyof A & keyof B` — only keys present in
        // ALL constituents are valid. Since keyof of an object type yields a
        // union of string-literal types, the intersection reduces to the set
        // of common literal names. We compute this directly to avoid leaving
        // an unsimplified `Union & Union` intersection type.
        if t.flags.contains(TypeFlags::Union) {
            let types = match &t.data {
                TypeData::Union(u) => &u.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let mut common: Option<Vec<String>> = None;
            for constituent in types {
                let k = self.get_index_type(constituent);
                let names = self.string_literal_values(&k);
                common = Some(match common.take() {
                    None => names,
                    Some(acc) => acc.into_iter().filter(|n| names.contains(n)).collect(),
                });
            }
            let names = common.unwrap_or_default();
            if names.is_empty() {
                return self.never_type();
            }
            let literals: Vec<Arc<Type>> = names
                .into_iter()
                .map(|n| self.get_string_literal_type(&n))
                .collect();
            return self.get_union_type(literals);
        }
        // Intersection: `keyof (A & B)` = `keyof A | keyof B` — keys present
        // in ANY constituent are valid.
        if t.flags.contains(TypeFlags::Intersection) {
            let types = match &t.data {
                TypeData::Intersection(i) => &i.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let keys: Vec<Arc<Type>> = types.iter().map(|c| self.get_index_type(c)).collect();
            return self.get_union_type(keys);
        }
        // Type parameter: resolve through the constraint.
        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
                return self.get_index_type(&constraint);
            }
            // Unconstrained type parameter: `keyof T` = `string | number | symbol`,
            // approximated as `string`.
            return self.string_type();
        }
        // Mapped `{ [P in K]: T }`: `keyof` is the (constraint-reduced)
        // domain K — `keyof { [P in E[keyof E]]: X }` with
        // `E extends Record<string, …>` reduces through E's constraint to
        // `string | number`. A `string` domain also admits number-like keys
        // (numeric keys coerce to strings), matching the structured branch
        // below.
        if let TypeData::Mapped(m) = &t.data
            && let Some(constraint) = &m.constraint_type
        {
            let generic = constraint.flags.intersects(
                TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index,
            ) || matches!(&constraint.data, TypeData::IndexedAccess(_));
            let domain = if generic {
                match self.constraint_of_indexed_access(constraint) {
                    Some(reduced) => reduced,
                    None => Arc::clone(constraint),
                }
            } else {
                Arc::clone(constraint)
            };
            if domain.flags.contains(TypeFlags::String) {
                let mut keys = vec![domain, self.number_type()];
                return self.get_union_type(keys);
            }
            return domain;
        }
        // Object-like types: collect property names as string-literal types,
        // unioned with the key types of any index signatures
        // (`keyof Record<string, X>` is `string | number`, not `never`).
        if let Some(structured) = t.as_structured() {
            let mut keys: Vec<Arc<Type>> = structured
                .properties
                .iter()
                // Private-identifier members never appear in `keyof` (Go's
                // mangled symbol names can't match a string literal; raw-text
                // keying needs the explicit filter).
                .filter(|p| !p.name.starts_with('#'))
                .map(|p| self.get_string_literal_type(&p.name))
                .collect();
            for info in &structured.index_infos {
                if let Some(key) = &info.key_type {
                    keys.push(Arc::clone(key));
                    // A `string` index signature is also reachable through
                    // number-like keys (numeric keys are coerced to strings),
                    // so it contributes `string | number` to `keyof`
                    // (`keyof Record<string, X>` is `string | number`).
                    if key.flags.contains(TypeFlags::String) {
                        keys.push(self.number_type());
                    }
                }
            }
            if keys.is_empty() {
                return self.never_type();
            }
            return self.get_union_type(keys);
        }
        // Other types (literals, etc.): `keyof` is `never`.
        self.never_type()
    }

    /// Resolve an indexed access type `objectType[indexType]`.
    ///
    /// Mirrors a simplified subset of Go's `getIndexedAccessTypeOrUndefined`:
    ///   - `any`/`unknown` object → `any`/`unknown`
    ///   - string-literal index → named property type
    ///   - union index → union of property types (e.g. `keyof T` result)
    ///   - `number` index on array/tuple → element type
    ///   - index signature match → index signature value type
    ///   - type parameter object → resolve through constraint
    ///   - otherwise → `any` (no error node here, so callers see `any`)
    pub fn get_indexed_access_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> Arc<Type> {
        // `any`/`unknown` propagate.
        if object_type.flags.contains(TypeFlags::Any) {
            return self.any_type();
        }
        if object_type.flags.contains(TypeFlags::Unknown) {
            return self.unknown_type();
        }
        if index_type.flags.contains(TypeFlags::Any) {
            return self.any_type();
        }
        // Union index: `T[A | B]` = `T[A] | T[B]`.
        if index_type.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &index_type.data {
                let prop_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|c| self.get_indexed_access_type(object_type, c))
                    .collect();
                if prop_types.is_empty() {
                    return self.any_type();
                }
                return self.get_union_type(prop_types);
            }
        }
        // Type-parameter object: resolve through the constraint.
        if object_type.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(object_type) {
                return self.get_indexed_access_type(&constraint, index_type);
            }
            return self.any_type();
        }
        // Mapped object `{ [P in K]: T }[X]` (Go getIndexedAccessType's
        // mapped branch): an index within the mapped constraint's
        // (constraint-reduced) domain resolves to the template type —
        // `TypeMap<E>[string]` is the template union regardless of which
        // key matched. The domain of a generic constraint reduces through
        // the object parameter's own constraint chain.
        if let TypeData::Mapped(m) = &object_type.data
            && let (Some(constraint), Some(template)) = (&m.constraint_type, &m.template_type)
        {
            let generic = constraint.flags.intersects(
                TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index,
            ) || matches!(&constraint.data, TypeData::IndexedAccess(_));
            let domain = if generic {
                match self.constraint_of_indexed_access(constraint) {
                    Some(reduced) => reduced,
                    None => Arc::clone(constraint),
                }
            } else {
                Arc::clone(constraint)
            };
            if self.is_type_assignable_to(index_type, &domain) {
                return Arc::clone(template);
            }
        }
        // String-literal index: `T["prop"]`.
        if index_type.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &index_type.data {
                if let LiteralValue::String(name) = &lit.value {
                    if let Some(structured) = object_type.as_structured() {
                        if let Some(sym) = structured.members.get(name) {
                            return self.get_type_of_symbol(sym);
                        }
                        // Fall back to index signature.
                        if let Some(value_type) =
                            self.lookup_index_signature_value(structured, index_type)
                        {
                            return value_type;
                        }
                    }
                    return self.any_type();
                }
            }
        }
        // `number` index on array/tuple → element type.
        if index_type.flags.contains(TypeFlags::Number)
            || index_type.flags.contains(TypeFlags::NumberLiteral)
        {
            if self.is_array_type(object_type) {
                return self.get_array_element_type(object_type);
            }
            // Tuple: `T[number]` → union of element types.
            if self.is_tuple_type(object_type) {
                if let Some(structured) = object_type.as_structured() {
                    let elem_types: Vec<Arc<Type>> = structured
                        .properties
                        .iter()
                        .map(|p| self.get_type_of_symbol(p))
                        .collect();
                    if !elem_types.is_empty() {
                        return self.get_union_type(elem_types);
                    }
                }
            }
        }
        // Index signature lookup.
        if let Some(structured) = object_type.as_structured() {
            if let Some(value_type) = self.lookup_index_signature_value(structured, index_type) {
                return value_type;
            }
        }
        self.any_type()
    }

    /// Look up an index signature whose key type is compatible with
    /// `index_type` and return its value type. Mirrors the index-signature
    /// fallback in Go's `getPropertyTypeForIndexType`.
    pub fn lookup_index_signature_value(
        &mut self,
        structured: &StructuredTypeData,
        index_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        for info in &structured.index_infos {
            let key_matches = match info.key_type.as_ref() {
                Some(key) => {
                    // `string` index signature matches string-like indices;
                    // `number` index signature matches number-like indices.
                    if key.flags.contains(TypeFlags::String) {
                        index_type.flags.contains(TypeFlags::String)
                            || index_type.flags.contains(TypeFlags::StringLiteral)
                    } else if key.flags.contains(TypeFlags::Number) {
                        index_type.flags.contains(TypeFlags::Number)
                            || index_type.flags.contains(TypeFlags::NumberLiteral)
                    } else {
                        false
                    }
                }
                None => true,
            };
            if key_matches {
                return info.value_type.clone();
            }
        }
        None
    }

    /// Build a `TemplateLiteral` type (or a flattened `StringLiteral` when
    /// all spans are concrete) from a `TemplateLiteralTypeNode`.
    ///
    /// Mirrors a simplified subset of Go's
    /// `getTypeFromTypeNode`→`getTemplateLiteralType`:
    ///   - Collect the head text and each span's (type, literal-text) pair.
    ///   - If every span type is a concrete literal
    ///     (string/number/boolean/null/undefined), flatten the whole
    ///     template into a single `StringLiteral` type — e.g.
    ///     `` `a-${1}-b` `` → `"a-1-b"`.
    ///   - Otherwise (e.g. `` `${string}` ``, `` `${T}` ``), keep a
    ///     `TemplateLiteral` type with the texts/types arrays so the
    ///     relater/nodebuilder can handle it.
    fn build_template_literal_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (head, spans) = match &node.data {
            NodeData::TemplateLiteralTypeNode(data) => {
                (Arc::clone(&data.head), Arc::clone(&data.template_spans))
            }
            _ => return self.error_type(),
        };
        let head_text = template_token_text(&head);
        // Collect (type, literal_text) for each span.
        let mut span_types: Vec<Arc<Type>> = Vec::new();
        let mut span_texts: Vec<String> = Vec::new();
        for span_node in spans.iter() {
            let (type_node, literal_node) = match &span_node.data {
                NodeData::TemplateLiteralTypeSpan(data) => {
                    (Arc::clone(&data.type_node), Arc::clone(&data.literal))
                }
                _ => return self.error_type(),
            };
            span_types.push(self.get_type_from_type_node(&type_node));
            span_texts.push(template_token_text(&literal_node));
        }
        // Go `getTemplateLiteralType`: a span whose type is a union (or
        // never) distributes the template into a cross product of template
        // literals; that product is capped at 100000 constituents (TS2590).
        // We keep our deferred representation below the cap — only the
        // overflow diagnostic and error-type fallback are ported.
        if span_types
            .iter()
            .any(|t| t.flags.contains(TypeFlags::Never) || matches!(&t.data, TypeData::Union(_)))
            && !self.check_cross_product_union(node, &span_types)
        {
            return self.error_type();
        }
        // Attempt to flatten: every span type must be a concrete literal.
        let all_literal = span_types.iter().all(|t| {
            t.flags
                .intersects(TYPE_FLAGS_LITERAL | TypeFlags::Null | TypeFlags::Undefined)
        });
        if all_literal {
            let mut sb = String::new();
            sb.push_str(&head_text);
            for (t, text) in span_types.iter().zip(span_texts.iter()) {
                sb.push_str(&self.template_string_for_type(t));
                sb.push_str(text);
            }
            return self.get_string_literal_type(&sb);
        }
        // Build a TemplateLiteral type.
        let mut texts = Vec::with_capacity(span_types.len() + 1);
        texts.push(head_text);
        for t in span_texts {
            texts.push(t);
        }
        Arc::new(Type::new(
            TypeFlags::TemplateLiteral,
            TypeData::TemplateLiteral(TemplateLiteralTypeData {
                constrained: ConstrainedTypeData::default(),
                texts,
                types: span_types,
            }),
        ))
    }

    /// String representation of a literal type for template-literal
    /// flattening. Mirrors Go's `getTemplateStringForType`:
    /// string-literal → the literal value, number → its decimal form,
    /// boolean → "true"/"false", null → "null", undefined → "undefined".
    fn template_string_for_type(&self, t: &Arc<Type>) -> String {
        if t.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::String(s) = &lit.value {
                    return s.clone();
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::Number(n) = &lit.value {
                    return n.to_string();
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::Boolean(b) = &lit.value {
                    return if *b { "true".into() } else { "false".into() };
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::Null) {
            return "null".into();
        }
        if t.flags.contains(TypeFlags::Undefined) {
            return "undefined".into();
        }
        String::new()
    }

    /// Build the result of a mapped type `{ [K in C]: V }`.
    ///
    /// Mirrors a simplified subset of Go's `getMappedType`/`instantiateMappedType`.
    /// When the constraint `C` resolves to a concrete union of string
    /// literals (i.e. `keyof T` for a concrete `T`, or `"a" | "b"`), the
    /// mapped type is eagerly resolved: for each key, the type parameter
    /// `K` is substituted with the key's string-literal type and the value
    /// type `V` is resolved, producing an anonymous object type with those
    /// properties. Optional (`?`) and readonly modifiers are applied.
    ///
    /// When the constraint is generic (`keyof T` where `T` is a type
    /// parameter) or not a union of string literals, the mapped type
    /// cannot be eagerly resolved and `any` is returned (no false
    /// positives).
    fn build_mapped_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        // The mapped-type node is a scope (Go: MappedType is
        // IsContainer|HasLocals): its `P` in `[P in K]` lives in the node's
        // own locals and is visible to the template/name-type expressions
        // only while the node is on the scope stack.
        self.push_scope(node);
        let result = self.build_mapped_type_inner(node);
        self.pop_scope();
        result
    }

    fn build_mapped_type_inner(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let data = match &node.data {
            NodeData::MappedTypeNode(data) => data,
            _ => return self.error_type(),
        };
        // Resolve the constraint type (the `in` clause).
        let constraint_node = match &data.type_parameter.data {
            NodeData::TypeParameterDeclaration(tp) => match &tp.constraint {
                Some(c) => Arc::clone(c),
                None => return self.error_type(),
            },
            _ => return self.error_type(),
        };
        let constraint_type = self.get_type_from_type_node(&constraint_node);
        // TS7039: a mapped type with no template (`{[P in K]}`) has an
        // implicit `any` template — reported under noImplicitAny at the
        // mapped-type node (Go's checkMappedType).
        if data.type_node.is_none()
            && self.no_implicit_any
            && self
                .current_file
                .as_ref()
                .is_some_and(|f| !f.file_name.starts_with("bundled://"))
        {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    MAPPED_OBJECT_TYPE_IMPLICITLY_HAS_AN_ANY_TEMPLATE_TYPE,
                Vec::new(),
            ));
        }
        // Get the set of key names from the constraint. Only concrete
        // unions of string literals (or a single string literal) can be
        // eagerly resolved.
        let keys = self.string_literal_values(&constraint_type);
        // A constraint that mixes literal and NON-literal constituents
        // (`keyof (T & Named)` = `string | "name"` — the type-parameter
        // keyof approximates to `string`) must NOT eagerly resolve just the
        // literal subset: the mapped type would silently drop every other
        // key (homomorphicMappedTypeIntersectionAssignability — Readonly<T
        // & Named> collapsed to `{ name }`). Keep it DEFERRED like a
        // generic constraint; deferred relation checks handle it.
        let constraint_all_literals = self.union_is_all_string_literals(&constraint_type);
        if keys.is_empty() || !constraint_all_literals {
            // Generic constraint (`keyof T` over a type parameter, `string`,
            // …): build a DEFERRED mapped type (Go getMappedType keeps the
            // type parameter, constraint, and template for contextual
            // substitution and deferred relation checks) instead of
            // collapsing to `any`.
            let tp_type = self.get_type_from_type_node(&data.type_parameter);
            let template_type = match &data.type_node {
                Some(tn) => {
                    // Resolve the template ONCE with the mapped type
                    // parameter free; contextual substitution replaces it
                    // per property name (C1/getIndexedMappedTypeSubstituted
                    // TypeOfContextualType).
                    self.get_type_from_type_node(tn)
                }
                None => self.get_any_type(),
            };
            let name_type = data
                .name_type
                .as_ref()
                .map(|n| self.get_type_from_type_node(n));
            return Arc::new(Type {
                flags: TypeFlags::Object,
                object_flags: crate::checker::types::ObjectFlags::Mapped,
                id: 0,
                symbol: None,
                alias: None,
                data: TypeData::Mapped(MappedTypeData {
                    object: ObjectTypeData {
                        structured: StructuredTypeData::default(),
                        ..Default::default()
                    },
                    declaration: Some(Arc::clone(node)),
                    type_parameter: Some(tp_type),
                    constraint_type: Some(constraint_type),
                    name_type,
                    template_type: Some(template_type),
                    modifiers_type: None,
                    resolved_apparent_type: OnceLock::new(),
                    contains_error: false,
                }),
            });
        }
        // Find the type-parameter symbol so we can substitute it.
        let tp_symbol = self
            .program
            .symbol_map()
            .symbol_of(&data.type_parameter)
            .map(Arc::clone);
        let tp_key = tp_symbol
            .as_ref()
            .map(|s| Arc::as_ptr(s) as *const crate::ast::Symbol);
        // Optional (`?`) modifier.
        let is_optional = data
            .question_token
            .as_ref()
            .map(|t| t.kind == SyntaxKind::QuestionToken)
            .unwrap_or(false);
        // Build the properties.
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for key in &keys {
            let mut prop_type = match &data.type_node {
                Some(tn) => {
                    // Push the type-parameter substitution.
                    if let Some(k) = tp_key {
                        let mut mapping = HashMap::new();
                        mapping.insert(k, self.get_string_literal_type(key));
                        self.type_argument_stack.push(mapping);
                    }
                    let t = self.get_type_from_type_node(tn);
                    if tp_key.is_some() {
                        self.type_argument_stack.pop();
                    }
                    t
                }
                None => self.get_any_type(),
            };
            if is_optional {
                prop_type = self.get_optional_type(prop_type);
            }
            let mut flags = SymbolFlags::Property;
            if is_optional {
                flags |= SymbolFlags::Optional;
            }
            let symbol = Arc::new(Symbol::new(flags, key.clone()));
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(prop_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(key.clone(), Arc::clone(&symbol));
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos: Vec::new(),
                    signatures: Vec::new(),
                    call_signature_count: 0,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    /// Flatten a type into its string-literal values.
    ///
    /// Used by `get_index_type` to compute common keys across union
    /// constituents: `keyof (A | B)` collects the keyof of each arm (a
    /// union of string-literal types, or `never`) and intersects the name
    /// sets. This helper extracts those names.
    fn string_literal_values(&self, t: &Arc<Type>) -> Vec<String> {
        if t.flags.contains(TypeFlags::Never) {
            return Vec::new();
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::String(s) = &lit.value {
                    return vec![s.clone()];
                }
            }
            return Vec::new();
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .flat_map(|c| self.string_literal_values(c))
                    .collect();
            }
        }
        Vec::new()
    }

    /// Whether a mapped-type constraint consists ONLY of string-literal
    /// constituents (`"a" | "b"`). A `string`/`number`/type-parameter
    /// constituent alongside literals means the key set is open — the
    /// mapped type cannot be eagerly resolved to the literal subset.
    fn union_is_all_string_literals(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::StringLiteral) {
            return true;
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .all(|c| self.union_is_all_string_literals(c));
            }
        }
        false
    }

    pub fn add_optionality(&self, t: &Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks {
            self.make_union_two(Arc::clone(t), self.undefined_type())
        } else {
            Arc::clone(t)
        }
    }

    fn make_union_two(&self, a: Arc<Type>, b: Arc<Type>) -> Arc<Type> {
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: vec![a, b],
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub fn get_nullable_type(&self, t: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let mut types = vec![Arc::clone(t)];
        if flags.contains(TypeFlags::Null) {
            types.push(self.null_type());
        }
        if flags.contains(TypeFlags::Undefined) {
            types.push(self.undefined_type());
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    // ────────────────────────────────────────────────────────────────────────
    // Function return type inference (P3.8b)
    // ────────────────────────────────────────────────────────────────────────

    /// Walk a function body collecting the types of every `return expr;`
    /// statement, skipping nested function bodies (those have their own
    /// return type). Used to infer the return type of unannotated
    /// functions. Mirrors the spirit of Go's `collectReturnStatements`
    /// + `getReturnTypeOfFunction` flow.
    pub fn collect_return_types_from_node(&mut self, node: &Arc<Node>, types: &mut Vec<Arc<Type>>) {
        use crate::ast::node_data_generated::for_each_child;
        match node.kind {
            SyntaxKind::ReturnStatement => {
                if let crate::ast::NodeData::ReturnStatement(data) = &node.data {
                    if let Some(expr) = &data.expression {
                        types.push(self.get_type_of_node(expr));
                    }
                }
                return;
            }
            // Don't descend into nested function-like bodies — they have
            // their own return type.
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => return,
            _ => {}
        }
        for_each_child(node, |child| {
            self.collect_return_types_from_node(child, types);
            false
        });
    }

    /// Infer a function's return type. If an explicit return-type annotation
    /// is present (`type_node`), it wins. Otherwise the body is walked for
    /// `return expr;` statements and the union of their types is returned.
    /// If no return statements are found, the inferred type is `void` (or
    /// `any` in non-strict-null-checks mode, mirroring Go's `voidType`).
    ///
    /// Mirrors Go's `getReturnTypeFromBody` (checker.go ~L20031). Literal
    /// return types are widened to their primitive base (e.g. `42` →
    /// `number`, `"foo"` → `string`) when no explicit return-type annotation
    /// is present, matching TypeScript's literal-widening rules.
    pub fn infer_function_return_type(
        &mut self,
        body: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
    ) -> Arc<Type> {
        if let Some(type_node) = type_node {
            return self.get_type_from_type_node(type_node);
        }
        let Some(body) = body else {
            return self.void_type();
        };
        // Arrow functions can have an expression body (`() => expr`) rather
        // than a block body. In that case the expression IS the return value.
        if body.kind != SyntaxKind::Block {
            let t = self.get_type_of_node(body);
            return self.get_widened_type(&t);
        }
        let mut types: Vec<Arc<Type>> = Vec::new();
        self.collect_return_types_from_node(body, &mut types);
        if types.is_empty() {
            return self.void_type();
        }
        let inferred = if types.len() == 1 {
            types.into_iter().next().expect("exactly one")
        } else {
            self.get_union_type(types)
        };
        // Widen fresh literal types to their primitive base (e.g. `42` →
        // `number`) for unannotated function returns.
        self.get_widened_type(&inferred)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nullable_flags_correct() {
        assert!(TYPE_FLAGS_NULLABLE.contains(TypeFlags::Undefined));
        assert!(TYPE_FLAGS_NULLABLE.contains(TypeFlags::Null));
    }

    #[test]
    fn union_flags_set() {
        let t = Type::new(
            TypeFlags::Union,
            TypeData::Intrinsic(IntrinsicTypeData {
                intrinsic_name: "test".to_string(),
            }),
        );
        assert!(t.flags.contains(TypeFlags::Union));
    }
}

/// Walk a type node looking for Identifier leaves matching any of `names`.
fn type_node_references_names(node: &Arc<Node>, names: &[String]) -> bool {
    let mut found = false;
    NodeWalker { names, found: &mut found }.walk(node);
    found
}

struct NodeWalker<'a> {
    names: &'a [String],
    found: &'a mut bool,
}

impl<'a> NodeWalker<'a> {
    fn walk(&mut self, node: &Arc<Node>) {
        if *self.found {
            return;
        }
        if node.kind == SyntaxKind::Identifier && names_contain(self.names, node.text()) {
            *self.found = true;
            return;
        }
        crate::ast::node_data_generated::for_each_child(node, |c| {
            self.walk(c);
            *self.found
        });
    }
}

fn names_contain(names: &[String], text: &str) -> bool {
    names.iter().any(|n| n == text)
}

/// Whether a type reference sits inside a CONDITIONAL type's true/false
/// branch — those are deferred (resolved only when the conditional
/// instantiates), so arity/constraint errors there are not reported
/// eagerly.
fn type_name_inside_conditional_branch(node: &Arc<Node>) -> bool {
    let mut cur = node.parent.as_ref();
    while let Some(a) = cur {
        if matches!(&a.data, NodeData::ConditionalTypeNode(_)) {
            // The node is in a branch if it's under the extendsType's
            // parent conditional's trueType/falseType — approximate: any
            // position inside the conditional except the checkType/
            // extendsType themselves.
            if let NodeData::ConditionalTypeNode(c) = &a.data {
                if node_inside(node, &c.check_type) || node_inside(node, &c.extends_type) {
                    cur = a.parent.as_ref();
                    continue;
                }
            }
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}

fn node_inside(node: &Arc<Node>, root: &Arc<Node>) -> bool {
    if Arc::ptr_eq(node, root) {
        return true;
    }
    let mut cur = node.parent.as_ref();
    while let Some(a) = cur {
        if Arc::ptr_eq(a, root) {
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}

/// Whether an enclosing declaration declares a TYPE PARAMETER with the
/// reference's name — the local parameter shadows any same-named global
/// generic ('type Or<A, B> = [A, B] ...' with a global 'interface A<T>').
/// Our resolution doesn't always honor that shadowing, so the arity check
/// is skipped for such names.
fn type_name_shadowed_by_type_parameter(type_name: &Arc<Node>) -> bool {
    let name = type_name.text();
    let mut cur = type_name.parent.as_ref();
    while let Some(a) = cur {
        let tps = match &a.data {
            NodeData::TypeAliasDeclaration(t) => t.type_parameters.as_ref(),
            NodeData::InterfaceDeclaration(i) => i.type_parameters.as_ref(),
            NodeData::ClassDeclaration(c) => c.type_parameters.as_ref(),
            NodeData::MethodDeclaration(m) => m.type_parameters.as_ref(),
            NodeData::FunctionDeclaration(f) => f.type_parameters.as_ref(),
            _ => None,
        };
        if let Some(list) = tps
            && list.iter().any(|p| {
                p.name().is_some_and(|n| n.text() == name)
            })
        {
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}
