//! Control flow narrowing.
//!
//! Ported from `internal/checker/flow.go`. Walks the flow graph built by
//! the binder to narrow types based on control-flow constraints (e.g.
//! `if (x !== null)` narrows `x` to exclude `null` in the then-branch).
//!
//! ## Algorithm
//!
//! `narrow_type` walks backwards from the current flow node through the
//! antecedent chain. For each flow node it checks whether the associated
//! AST expression constrains the target symbol:
//!
//! - **TRUE_CONDITION** `x !== null` → remove `null` from the union.
//! - **TRUE_CONDITION** `x === null` → narrow to `null`.
//! - **FALSE_CONDITION** applies the inverse of the above.
//! - **ASSIGNMENT** to `x` → replace the type with the RHS type.
//! - **Junction** (multiple antecedents) → union of narrowed types.
//!
//! Recursion is capped at `FLOW_MAX_DEPTH` to prevent stack overflow on
//! cyclic or very deep flow graphs.

use std::sync::Arc;

use crate::ast::{FlowFlags, FlowNode, Node, NodeData, Symbol, SyntaxKind};

use super::checker::Checker;
use super::types::*;

/// Maximum recursion depth for `narrow_type`. Prevents stack overflow on
/// very deep or cyclic flow graphs. The Go implementation uses a similar
/// cap via `relationStackDepth`.
const FLOW_MAX_DEPTH: u32 = 200;

/// The kind of narrowing to apply for a condition.
#[derive(Clone, Copy, PartialEq)]
enum NarrowKind {
    /// The condition is true; narrow to types satisfying the constraint.
    /// E.g. `x !== null` (true) → remove `null`.
    TrueBranch,
    /// The condition is false; narrow to types NOT satisfying the constraint.
    /// E.g. `x !== null` (false) → narrow to `null`.
    FalseBranch,
}

impl Checker {
    /// Get the narrowed type of a symbol at a given flow point.
    ///
    /// Mirrors Go's `getNarrowedTypeOfSymbol`. Returns the declared type
    /// when `flow` is `None` (no flow context available).
    pub fn get_narrowed_type_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        flow: Option<&Arc<FlowNode>>,
    ) -> Arc<Type> {
        let declared = self.get_type_of_symbol(symbol);
        let Some(flow) = flow else {
            return declared;
        };
        if self.flow_analysis_disabled {
            return declared;
        }
        // Cache lookup: combine symbol ID and flow node pointer.
        let key = self.flow_cache_key(symbol, flow);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Arc::clone(cached);
        }
        // Reserve a slot to break cycles (write `declared` first so that
        // recursive lookups during narrowing get the un-narrowed type).
        self.flow_type_cache.insert(key, Arc::clone(&declared));
        let narrowed = self.narrow_type(&declared, flow, symbol, 0);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));
        narrowed
    }

    /// Compute a cache key for (symbol, flow) pairs.
    fn flow_cache_key(&self, symbol: &Arc<Symbol>, flow: &Arc<FlowNode>) -> u64 {
        let sym_id = symbol.id();
        let flow_ptr = Arc::as_ptr(flow) as *const FlowNode as u64;
        // Mix the two: rotate symbol ID and XOR with flow pointer.
        sym_id.rotate_left(17) ^ flow_ptr
    }

    /// Narrow `type_` by walking the flow graph backwards from `flow`.
    ///
    /// This is the core narrowing routine. It inspects each flow node,
    /// applies any applicable narrowing to `type_`, and recurses into
    /// antecedent(s).
    fn narrow_type(
        &mut self,
        type_: &Arc<Type>,
        flow: &Arc<FlowNode>,
        symbol: &Arc<Symbol>,
        depth: u32,
    ) -> Arc<Type> {
        if depth >= FLOW_MAX_DEPTH {
            return Arc::clone(type_);
        }

        // UNREACHABLE flow → the code path is dead; `never`.
        if flow.flags.contains(FlowFlags::UNREACHABLE) {
            return self.never_type();
        }

        // START flow → no narrowing possible.
        if flow.flags.contains(FlowFlags::START) {
            return Arc::clone(type_);
        }

        // TRUE_CONDITION / FALSE_CONDITION → narrow based on the condition.
        if flow.flags.contains(FlowFlags::CONDITION) {
            let kind = if flow.flags.contains(FlowFlags::TRUE_CONDITION) {
                NarrowKind::TrueBranch
            } else {
                NarrowKind::FalseBranch
            };
            let narrowed = if let Some(expr) = &flow.node {
                self.narrow_by_expression(type_, expr, symbol, kind, depth)
            } else {
                Arc::clone(type_)
            };
            // Recurse into the antecedent.
            if let Some(antecedent) = &flow.antecedent {
                return self.narrow_type(&narrowed, antecedent, symbol, depth + 1);
            }
            return narrowed;
        }

        // ASSIGNMENT → if the assignment is to `symbol`, the type becomes
        // the RHS type.
        if flow.flags.contains(FlowFlags::ASSIGNMENT) {
            if let Some(expr) = &flow.node {
                if let Some(rhs_type) = self.assignment_rhs_type_for_symbol(expr, symbol) {
                    // Recurse into the antecedent with the new type.
                    if let Some(antecedent) = &flow.antecedent {
                        return self.narrow_type(&rhs_type, antecedent, symbol, depth + 1);
                    }
                    return rhs_type;
                }
            }
            // Not an assignment to our symbol; continue.
            if let Some(antecedent) = &flow.antecedent {
                return self.narrow_type(type_, antecedent, symbol, depth + 1);
            }
            return Arc::clone(type_);
        }

        // ARRAY_MUTATION / CALL → these may invalidate narrowing, but for
        // now we just recurse into the antecedent.
        if flow.flags.contains(FlowFlags::ARRAY_MUTATION)
            || flow.flags.contains(FlowFlags::CALL)
        {
            if let Some(antecedent) = &flow.antecedent {
                return self.narrow_type(type_, antecedent, symbol, depth + 1);
            }
            return Arc::clone(type_);
        }

        // Junction (multiple antecedents): narrow through each and compute
        // the union of results. This handles if/else merge points, loop
        // back-edges, and switch clause falls.
        if flow.antecedents.len() > 1 {
            let mut narrowed_types: Vec<Arc<Type>> = Vec::new();
            for antecedent in &flow.antecedents {
                let narrowed = self.narrow_type(type_, antecedent, symbol, depth + 1);
                if !narrowed_types.iter().any(|t| Arc::ptr_eq(t, &narrowed)) {
                    narrowed_types.push(narrowed);
                }
            }
            // If only one distinct result, return it.
            if narrowed_types.len() == 1 {
                return narrowed_types.into_iter().next().expect("exactly one");
            }
            // If multiple distinct results, compute their union.
            if narrowed_types.is_empty() {
                return Arc::clone(type_);
            }
            return self.get_union_type(narrowed_types);
        }

        // Single antecedent → recurse.
        if let Some(antecedent) = &flow.antecedent {
            return self.narrow_type(type_, antecedent, symbol, depth + 1);
        }

        Arc::clone(type_)
    }

    /// Narrow a type based on a single condition expression.
    ///
    /// `kind` indicates whether the condition is true or false.
    fn narrow_by_expression(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
        kind: NarrowKind,
        depth: u32,
    ) -> Arc<Type> {
        // Logical AND: `a && b` — both sides are true in the true branch.
        if expr.kind == SyntaxKind::BinaryExpression {
            if let NodeData::BinaryExpression(bin) = &expr.data {
                if bin.operator_token.kind == SyntaxKind::AmpersandAmpersandToken {
                    if kind == NarrowKind::TrueBranch {
                        let narrowed = self.narrow_by_expression(
                            type_,
                            &bin.left,
                            symbol,
                            kind,
                            depth,
                        );
                        return self.narrow_by_expression(
                            &narrowed,
                            &bin.right,
                            symbol,
                            kind,
                            depth,
                        );
                    }
                    // False branch of `a && b`: either `a` is false OR
                    // (`a` is true AND `b` is false). We can't narrow
                    // precisely, so just check the left side.
                    return self.narrow_by_expression(
                        type_,
                        &bin.left,
                        symbol,
                        NarrowKind::FalseBranch,
                        depth,
                    );
                }
                if bin.operator_token.kind == SyntaxKind::BarBarToken {
                    if kind == NarrowKind::FalseBranch {
                        // False branch of `a || b`: both `a` and `b` are false.
                        let narrowed = self.narrow_by_expression(
                            type_,
                            &bin.left,
                            symbol,
                            kind,
                            depth,
                        );
                        return self.narrow_by_expression(
                            &narrowed,
                            &bin.right,
                            symbol,
                            kind,
                            depth,
                        );
                    }
                    // True branch of `a || b`: at least one is true. Check left.
                    return self.narrow_by_expression(
                        type_,
                        &bin.left,
                        symbol,
                        NarrowKind::TrueBranch,
                        depth,
                    );
                }
            }
        }

        // Logical NOT: `!x` — invert the branch.
        if expr.kind == SyntaxKind::PrefixUnaryExpression {
            if let NodeData::PrefixUnaryExpression(unary) = &expr.data {
                if unary.operator == SyntaxKind::ExclamationToken {
                    let inverted = if kind == NarrowKind::TrueBranch {
                        NarrowKind::FalseBranch
                    } else {
                        NarrowKind::TrueBranch
                    };
                    return self.narrow_by_expression(
                        type_,
                        &unary.operand,
                        symbol,
                        inverted,
                        depth,
                    );
                }
            }
        }

        // Binary comparison: `x === value`, `x !== null`, `typeof x === "string"`, etc.
        if expr.kind == SyntaxKind::BinaryExpression {
            return self.narrow_by_binary(type_, expr, symbol, kind);
        }

        // Bare identifier: `if (x)` — truthiness narrowing.
        if self.is_symbol_identifier(expr, symbol) {
            return self.narrow_by_truthiness(type_, kind);
        }

        // `typeof x === "string"` is a BinaryExpression, handled above.
        // `x instanceof Foo` is also a BinaryExpression.

        Arc::clone(type_)
    }

    /// Narrow based on a binary expression (comparison, typeof, instanceof, in).
    fn narrow_by_binary(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return Arc::clone(type_);
        };
        let op = bin.operator_token.kind;

        // `instanceof`: `x instanceof Foo` — narrow to `Foo` in true branch.
        if op == SyntaxKind::InstanceOfKeyword {
            if self.is_symbol_identifier(&bin.left, symbol) {
                if kind == NarrowKind::TrueBranch {
                    // Narrow to the right-hand side type (the constructor's
                    // instance type). For now, return the declared type
                    // (instance type resolution is TODO).
                    return Arc::clone(type_);
                }
                // False branch: remove the type of the right-hand side.
                return Arc::clone(type_);
            }
            return Arc::clone(type_);
        }

        // `in`: `"prop" in x` — narrow `x` to a type having `prop`.
        if op == SyntaxKind::InKeyword {
            // For now, no narrowing on `in` expressions.
            return Arc::clone(type_);
        }

        // Equality/inequality: `===`, `!==`, `==`, `!=`
        let is_strict = op == SyntaxKind::EqualsEqualsEqualsToken
            || op == SyntaxKind::ExclamationEqualsEqualsToken;
        let is_loose = op == SyntaxKind::EqualsEqualsToken
            || op == SyntaxKind::ExclamationEqualsToken;
        if !is_strict && !is_loose {
            return Arc::clone(type_);
        }

        let is_equality = op == SyntaxKind::EqualsEqualsEqualsToken
            || op == SyntaxKind::EqualsEqualsToken;
        // For `x === value`:
        //   true branch  → narrow to `value` type
        //   false branch → remove `value` type from union
        // For `x !== value`:
        //   true branch  → remove `value` type from union
        //   false branch → narrow to `value` type
        let narrow_to_value = if is_equality {
            kind == NarrowKind::TrueBranch
        } else {
            kind == NarrowKind::FalseBranch
        };

        // Handle `typeof x === "string"` patterns.
        if bin.left.kind == SyntaxKind::TypeOfExpression
            && self.typeof_expr_matches_symbol(&bin.left, symbol)
        {
            return self.narrow_by_typeof(type_, &bin.right, narrow_to_value, is_loose);
        }
        if bin.right.kind == SyntaxKind::TypeOfExpression
            && self.typeof_expr_matches_symbol(&bin.right, symbol)
        {
            return self.narrow_by_typeof(type_, &bin.left, narrow_to_value, is_loose);
        }

        // Simple `x === value` or `value === x` patterns.
        let (value_node, is_symbol_on_left) = if self.is_symbol_identifier(&bin.left, symbol) {
            (&bin.right, true)
        } else if self.is_symbol_identifier(&bin.right, symbol) {
            (&bin.left, false)
        } else {
            return Arc::clone(type_);
        };
        let _ = is_symbol_on_left;

        let value_type = self.get_type_of_node(value_node);
        if narrow_to_value {
            // Narrow to the value's type (intersect with current type for
            // union members).
            self.intersect_or_narrow(type_, &value_type)
        } else {
            // Remove the value's type from the union.
            self.remove_type_from_union(type_, &value_type)
        }
    }

    /// Check if `expr` is `typeof <symbol>`.
    fn typeof_expr_matches_symbol(&self, expr: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        let NodeData::TypeOfExpression(typeof_data) = &expr.data else {
            return false;
        };
        self.is_symbol_identifier(&typeof_data.expression, symbol)
    }

    /// Narrow by `typeof x === "typename"`.
    ///
    /// `narrow_to_value` = true means the typeof check passed (e.g.
    /// `typeof x === "string"` is true), so we narrow to the matching type.
    fn narrow_by_typeof(
        &self,
        type_: &Arc<Type>,
        type_name_node: &Arc<Node>,
        narrow_to_value: bool,
        is_loose: bool,
    ) -> Arc<Type> {
        let type_name = match &type_name_node.data {
            NodeData::StringLiteral(data) => data.text.as_str(),
            _ => return Arc::clone(type_),
        };
        let matching_flags = match type_name {
            "string" => TYPE_FLAGS_STRING_LIKE,
            "number" => TYPE_FLAGS_NUMBER_LIKE,
            "boolean" => TYPE_FLAGS_BOOLEAN_LIKE,
            "bigint" => TYPE_FLAGS_BIG_INT_LIKE,
            "symbol" => TYPE_FLAGS_ES_SYMBOL_LIKE,
            "undefined" => TypeFlags::Undefined,
            "function" => TypeFlags::Object,
            "object" => {
                // "object" matches object types, null, and arrays but not
                // primitives. For loose equality also matches undefined.
                if narrow_to_value {
                    return self.filter_type_by_object(type_, is_loose);
                }
                return self.remove_object_from_union(type_);
            }
            _ => return Arc::clone(type_),
        };
        if narrow_to_value {
            self.filter_type_by_flags(type_, matching_flags)
        } else {
            self.remove_flags_from_union(type_, matching_flags)
        }
    }

    /// Narrow by truthiness: `if (x)` removes falsy types (undefined, null,
    /// void, false, 0, "") in the true branch.
    fn narrow_by_truthiness(&self, type_: &Arc<Type>, kind: NarrowKind) -> Arc<Type> {
        match kind {
            NarrowKind::TrueBranch => {
                // Remove null, undefined, void, false, 0, "" from the union.
                let falsy_flags = TypeFlags::Undefined
                    | TypeFlags::Null
                    | TypeFlags::Void
                    | TypeFlags::BooleanLiteral
                    | TypeFlags::StringLiteral
                    | TypeFlags::NumberLiteral;
                self.remove_falsy_from_union(type_, falsy_flags)
            }
            NarrowKind::FalseBranch => {
                // Narrow to falsy types only.
                self.filter_to_falsy(type_)
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Union manipulation helpers
    // ─────────────────────────────────────────────────────────────────

    /// Get the constituent types of a union, or `[type_]` for non-unions.
    /// Returns empty for `never`.
    fn constituent_types(&self, type_: &Arc<Type>) -> Vec<Arc<Type>> {
        if type_.is_union() {
            if let TypeData::Union(u) = &type_.data {
                return u.union_or_intersection.types.clone();
            }
        }
        if type_.flags.contains(TypeFlags::Never) {
            return Vec::new();
        }
        vec![Arc::clone(type_)]
    }

    /// Remove all types from `type_` that match `value_type`.
    fn remove_type_from_union(&self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !self.types_overlap(t, value_type))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        // Rebuild the union without using `&mut self` (the helper is `&self`).
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    /// Remove all types from `type_` whose flags intersect `flags`.
    fn remove_flags_from_union(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !t.flags.intersects(flags))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    /// Filter `type_` to only types whose flags intersect `flags`.
    fn filter_type_by_flags(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| t.flags.intersects(flags))
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    /// Narrow `type_` to types that are object-like (for `typeof === "object"`).
    fn filter_type_by_object(&self, type_: &Arc<Type>, is_loose: bool) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let mut matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                // `typeof` returns "object" for objects, arrays, and null.
                t.flags.contains(TypeFlags::Object)
                    || t.flags.contains(TypeFlags::Null)
                    || (is_loose && t.flags.contains(TypeFlags::Undefined))
            })
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching.drain(..).collect(),
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    /// Remove object types from a union (for `typeof !== "object"`).
    fn remove_object_from_union(&self, type_: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                !t.flags.contains(TypeFlags::Object) && !t.flags.contains(TypeFlags::Null)
            })
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    /// Remove falsy types from a union (for truthiness narrowing).
    /// Removes undefined, null, void, false, literal "" and literal 0.
    fn remove_falsy_from_union(&self, type_: &Arc<Type>, falsy_flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                if t.flags.intersects(falsy_flags) {
                    // Keep `true` (boolean true is truthy).
                    if t.flags.contains(TypeFlags::BooleanLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            return matches!(lit.value, LiteralValue::Boolean(true));
                        }
                    }
                    // Keep non-empty string literals.
                    if t.flags.contains(TypeFlags::StringLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            if let LiteralValue::String(s) = &lit.value {
                                return !s.is_empty();
                            }
                        }
                        return false;
                    }
                    // Keep non-zero number literals.
                    if t.flags.contains(TypeFlags::NumberLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            if let LiteralValue::Number(n) = &lit.value {
                                return n.0 != 0.0;
                            }
                        }
                        return false;
                    }
                    return false;
                }
                true
            })
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    /// Filter `type_` to only falsy types (for `if (!x)` true branch).
    fn filter_to_falsy(&self, type_: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let falsy_flags =
            TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Void | TypeFlags::BooleanLiteral;
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                if t.flags.intersects(falsy_flags) {
                    // For BooleanLiteral, only `false` is falsy.
                    if t.flags.contains(TypeFlags::BooleanLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            return matches!(lit.value, LiteralValue::Boolean(false));
                        }
                    }
                    return true;
                }
                // Empty string literal.
                if t.flags.contains(TypeFlags::StringLiteral) {
                    if let TypeData::Literal(lit) = &t.data {
                        if let LiteralValue::String(s) = &lit.value {
                            return s.is_empty();
                        }
                    }
                }
                // Zero number literal.
                if t.flags.contains(TypeFlags::NumberLiteral) {
                    if let TypeData::Literal(lit) = &t.data {
                        if let LiteralValue::Number(n) = &lit.value {
                            return n.0 == 0.0;
                        }
                    }
                }
                false
            })
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    /// Intersect `type_` with `value_type`. If `type_` is a union, keep
    /// only the constituents that are assignable to `value_type`. If `type_`
    /// itself is assignable, return `value_type`.
    fn intersect_or_narrow(&mut self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {
        // If the value type is a subtype of the current type, narrow to it.
        if self.is_type_assignable_to(value_type, type_) {
            return Arc::clone(value_type);
        }
        // If the current type is a union, try to find the matching constituent.
        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let matching: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| self.is_type_assignable_to(value_type, t))
                .collect();
            if matching.len() == 1 {
                return matching.into_iter().next().expect("exactly one");
            }
            if matching.is_empty() {
                return Arc::clone(value_type);
            }
            return self.get_union_type(matching);
        }
        Arc::clone(value_type)
    }

    /// Check if two types overlap (share at least one constituent).
    fn types_overlap(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        // Direct flag comparison for primitives.
        if a.flags.intersects(b.flags) {
            return true;
        }
        // If either is a union, check all pairs.
        let a_types = self.constituent_types(a);
        let b_types = self.constituent_types(b);
        for at in &a_types {
            for bt in &b_types {
                if at.flags == bt.flags {
                    // Same primitive type.
                    if !at.flags.contains(TypeFlags::Union) {
                        return true;
                    }
                }
            }
        }
        false
    }

    // ─────────────────────────────────────────────────────────────────
    // Symbol/expression matching helpers
    // ─────────────────────────────────────────────────────────────────

    /// Check if `node` is an identifier that resolves to `symbol`.
    /// Uses the symbol_map for a direct lookup (avoids mutating scope state).
    fn is_symbol_identifier(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        if node.kind != SyntaxKind::Identifier {
            return false;
        }
        // Try the symbol_map first (set by the binder on declaration nodes).
        // For reference nodes, the symbol may not be set, so we fall back
        // to name-based matching.
        let symbol_map = self.program.symbol_map();
        if let Some(sym) = symbol_map.symbol_of(node) {
            return Arc::ptr_eq(sym, symbol);
        }
        // Fallback: compare by name. The identifier's text must match the
        // symbol's name (set by the binder when the declaration was bound).
        let node_name = match &node.data {
            NodeData::Identifier(data) => &data.text,
            _ => return false,
        };
        node_name == &symbol.name
    }

    /// If `expr` is an assignment to `symbol`, return the type of the RHS.
    fn assignment_rhs_type_for_symbol(
        &mut self,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        // `x = value`, `x += value`, etc.
        if expr.kind == SyntaxKind::BinaryExpression {
            if let NodeData::BinaryExpression(bin) = &expr.data {
                if is_assignment_operator(bin.operator_token.kind)
                    && self.is_symbol_identifier(&bin.left, symbol)
                {
                    return Some(self.get_type_of_node(&bin.right));
                }
            }
        }
        // `x++`, `x--`
        if expr.kind == SyntaxKind::PostfixUnaryExpression {
            if let NodeData::PostfixUnaryExpression(unary) = &expr.data {
                if (unary.operator == SyntaxKind::PlusPlusToken
                    || unary.operator == SyntaxKind::MinusMinusToken)
                    && self.is_symbol_identifier(&unary.operand, symbol)
                {
                    return Some(self.number_type());
                }
            }
        }
        if expr.kind == SyntaxKind::PrefixUnaryExpression {
            if let NodeData::PrefixUnaryExpression(unary) = &expr.data {
                if (unary.operator == SyntaxKind::PlusPlusToken
                    || unary.operator == SyntaxKind::MinusMinusToken)
                    && self.is_symbol_identifier(&unary.operand, symbol)
                {
                    return Some(self.number_type());
                }
            }
        }
        None
    }
}

/// Check if a syntax kind is an assignment operator (`=`, `+=`, etc.).
///
/// Mirrors `binder.isAssignmentOperator` / `ast.IsAssignmentOperator`.
fn is_assignment_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
    )
}
