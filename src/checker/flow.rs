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

use crate::ast::{FlowFlags, FlowNode, Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

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

        // SWITCH_CLAUSE → narrow based on the switch case expression.
        // Mirrors Go's `getTypeAtSwitchClause` (flow.go ~L1046). We first
        // recurse into the antecedent to get the narrowed type at this flow
        // point, then apply switch-specific narrowing on top.
        if flow.flags.contains(FlowFlags::SWITCH_CLAUSE) {
            let antecedent_type = if let Some(antecedent) = &flow.antecedent {
                self.narrow_type(type_, antecedent, symbol, depth + 1)
            } else {
                Arc::clone(type_)
            };
            return self.narrow_by_switch_clause(&antecedent_type, flow, symbol);
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

        // `instanceof`: `x instanceof Foo` — narrow to the instance type
        // of `Foo` in the true branch; remove it in the false branch.
        if op == SyntaxKind::InstanceOfKeyword {
            return self.narrow_by_instanceof(type_, &bin.left, &bin.right, symbol, kind);
        }

        // `in`: `"prop" in x` — narrow `x` by property presence.
        if op == SyntaxKind::InKeyword {
            return self.narrow_by_in_keyword(type_, &bin.left, &bin.right, symbol, kind);
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

        // Discriminated union narrowing: `obj.kind === "value"` narrows
        // `obj` to the union constituent whose `kind` property matches.
        if let Some(narrowed) = self.try_narrow_by_discriminant_property(
            type_, expr, symbol, kind,
        ) {
            return narrowed;
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

    /// `x instanceof Foo` — narrow `x` to the instance type of `Foo`.
    ///
    /// Mirrors Go's `narrowTypeByInstanceof` (flow.go ~L798). We resolve
    /// the instance type via the constructor's `prototype` property or
    /// its construct signatures, then either keep only constituents that
    /// are assignable to the candidate (true branch) or remove them
    /// (false branch).
    fn narrow_by_instanceof(
        &mut self,
        type_: &Arc<Type>,
        left: &Arc<Node>,
        right: &Arc<Node>,
        symbol: &Arc<Symbol>,
        kind: NarrowKind,
    ) -> Arc<Type> {
        if !self.is_symbol_identifier(left, symbol) {
            return Arc::clone(type_);
        }
        let right_type = self.get_type_of_node(right);
        let Some(instance_type) = self.get_instance_type_of_constructor(&right_type) else {
            return Arc::clone(type_);
        };
        match kind {
            NarrowKind::TrueBranch => self.narrow_to_subtype(type_, &instance_type),
            NarrowKind::FalseBranch => self.remove_subtype_from_union(type_, &instance_type),
        }
    }

    /// `"prop" in x` — narrow `x` by property presence.
    ///
    /// In the true branch we keep only constituents that have (or might
    /// have) the property; in the false branch we keep only constituents
    /// that lack it. Mirrors Go's `narrowTypeByInKeyword` (flow.go ~L988).
    fn narrow_by_in_keyword(
        &mut self,
        type_: &Arc<Type>,
        left: &Arc<Node>,
        right: &Arc<Node>,
        symbol: &Arc<Symbol>,
        kind: NarrowKind,
    ) -> Arc<Type> {
        if !self.is_symbol_identifier(right, symbol) {
            return Arc::clone(type_);
        }
        let Some(prop_name) = Self::get_accessed_property_name_from_node(left) else {
            return Arc::clone(type_);
        };
        let keep_present = match kind {
            NarrowKind::TrueBranch => true,
            NarrowKind::FalseBranch => false,
        };
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let has_prop = self.type_has_property(t, &prop_name);
                keep_present == has_prop.is_definitely()
                    && (keep_present || !has_prop.is_definitely_not())
            })
            .collect();
        self.rebuild_union_or_never(type_, filtered)
    }

    /// Try to narrow a union by a discriminant property comparison like
    /// `obj.kind === "foo"` or `obj.kind === Kind.Foo`.
    ///
    /// Returns `Some(narrowed)` when the expression matches the pattern
    /// and narrowing applied, or `None` to fall through to other rules.
    fn try_narrow_by_discriminant_property(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
        kind: NarrowKind,
    ) -> Option<Arc<Type>> {
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return None;
        };
        let op = bin.operator_token.kind;
        // Only strict equality is supported for discriminant narrowing.
        let is_strict_eq = op == SyntaxKind::EqualsEqualsEqualsToken
            || op == SyntaxKind::ExclamationEqualsEqualsToken;
        if !is_strict_eq {
            return None;
        }
        // Find which side is the property access on `symbol`.
        let (access_node, value_node) =
            if self.is_property_access_on_symbol(&bin.left, symbol) {
                (&bin.left, &bin.right)
            } else if self.is_property_access_on_symbol(&bin.right, symbol) {
                (&bin.right, &bin.left)
            } else {
                return None;
            };
        let prop_name = Self::get_accessed_property_name_from_node(access_node)?;
        // For non-union types, narrowing by discriminant is a no-op.
        if !type_.is_union() {
            return Some(Arc::clone(type_));
        }
        let value_type = self.get_type_of_node(value_node);
        let is_equality = op == SyntaxKind::EqualsEqualsEqualsToken;
        let keep_matching = if is_equality {
            kind == NarrowKind::TrueBranch
        } else {
            kind == NarrowKind::FalseBranch
        };
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);
                let matches = prop_type
                    .map(|pt| self.types_overlap(&pt, &value_type))
                    .unwrap_or(false);
                if keep_matching {
                    matches
                } else {
                    !matches
                }
            })
            .collect();
        Some(self.rebuild_union_or_never(type_, filtered))
    }

    /// Narrow `type_` based on a switch clause.
    ///
    /// Mirrors Go's `getTypeAtSwitchClause` (flow.go ~L1046). Dispatches to
    /// the appropriate narrowing strategy based on the switch discriminant:
    ///
    /// - `switch (x)` → `narrow_by_switch_on_discriminant`
    /// - `switch (obj.kind)` → `narrow_by_switch_on_discriminant_property`
    ///
    /// `typeof x` and `switch (true)` variants are not yet supported.
    fn narrow_by_switch_clause(
        &mut self,
        type_: &Arc<Type>,
        flow: &Arc<FlowNode>,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        let Some(switch_stmt) = &flow.switch_statement else {
            return Arc::clone(type_);
        };
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Arc::clone(type_);
        };
        let discriminant = &switch_data.expression;
        let Some(clause) = &flow.node else {
            return Arc::clone(type_);
        };

        // Case 1: discriminant is the symbol itself → `switch (x) { ... }`
        if self.is_symbol_identifier(discriminant, symbol) {
            return self.narrow_by_switch_on_discriminant(type_, clause, switch_stmt);
        }

        // Case 2: discriminant is a property access on the symbol →
        // `switch (obj.kind) { ... }`
        if self.is_property_access_on_symbol(discriminant, symbol) {
            return self.narrow_by_switch_on_discriminant_property(
                type_, clause, switch_stmt, discriminant,
            );
        }

        // TODO: `switch (typeof x)` and `switch (true)` patterns.
        Arc::clone(type_)
    }

    /// Narrow for `switch (x) { case value: ... }` where `x` is the symbol.
    ///
    /// Mirrors Go's `narrowTypeBySwitchOnDiscriminant` (flow.go ~L1078). For
    /// a `CaseClause`, narrows to the case expression's type; for a
    /// `DefaultClause`, narrows to the types not covered by any case.
    fn narrow_by_switch_on_discriminant(
        &mut self,
        type_: &Arc<Type>,
        clause: &Arc<Node>,
        switch_stmt: &Arc<Node>,
    ) -> Arc<Type> {
        let case_types = self.get_switch_clause_types(switch_stmt);
        if clause.kind == SyntaxKind::DefaultClause {
            // Default clause: narrow to types not covered by any case.
            // Keep constituents that don't overlap with any case type.
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| {
                    !case_types
                        .iter()
                        .any(|ct| self.types_overlap(t, ct))
                })
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
        // CaseClause: narrow to the case expression's type.
        let NodeData::CaseOrDefaultClause(clause_data) = &clause.data else {
            return Arc::clone(type_);
        };
        let case_type = self.get_type_of_node(&clause_data.expression);
        self.intersect_or_narrow(type_, &case_type)
    }

    /// Narrow for `switch (obj.kind) { case "value": ... }` where `obj.kind`
    /// is a property access on the symbol.
    ///
    /// Mirrors Go's `narrowTypeBySwitchOnDiscriminantProperty` (flow.go
    /// ~L1210). For a `CaseClause`, keeps only the union constituents whose
    /// discriminant property matches the case type; for a `DefaultClause`,
    /// keeps only constituents whose discriminant property does not match
    /// any case.
    fn narrow_by_switch_on_discriminant_property(
        &mut self,
        type_: &Arc<Type>,
        clause: &Arc<Node>,
        switch_stmt: &Arc<Node>,
        access: &Arc<Node>,
    ) -> Arc<Type> {
        let Some(prop_name) = Self::get_accessed_property_name_from_node(access) else {
            return Arc::clone(type_);
        };
        // Only narrow unions.
        if !type_.is_union() {
            return Arc::clone(type_);
        }
        let case_types = self.get_switch_clause_types(switch_stmt);
        let is_default = clause.kind == SyntaxKind::DefaultClause;
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);
                let Some(prop_type) = prop_type else {
                    // No property → keep only in default clause.
                    return is_default;
                };
                if is_default {
                    // Default: keep constituents whose property doesn't
                    // match any case type.
                    !case_types
                        .iter()
                        .any(|ct| self.types_overlap(&prop_type, ct))
                } else {
                    // Case: keep constituents whose property matches at
                    // least one case type.
                    case_types
                        .iter()
                        .any(|ct| self.types_overlap(&prop_type, ct))
                }
            })
            .collect();
        self.rebuild_union_or_never(type_, filtered)
    }

    /// Get the types of all case clauses in a switch statement.
    ///
    /// Mirrors Go's `getSwitchClauseTypes` (flow.go ~L2005). Returns a
    /// `Vec` with one entry per clause: the case expression's type for
    /// `CaseClause`s, and `never` for `DefaultClause`s.
    fn get_switch_clause_types(&mut self, switch_stmt: &Arc<Node>) -> Vec<Arc<Type>> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Vec::new();
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return Vec::new();
        };
        let mut types = Vec::with_capacity(case_block.clauses.len());
        for clause in &case_block.clauses.nodes {
            if clause.kind == SyntaxKind::CaseClause {
                if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
                    types.push(self.get_type_of_node(&cd.expression));
                    continue;
                }
            }
            types.push(self.never_type());
        }
        types
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

    // ─────────────────────────────────────────────────────────────────
    // Property / instance-type helpers
    // ─────────────────────────────────────────────────────────────────

    /// Look up a property symbol by name on a structured type.
    /// Returns `None` for non-structured types or missing properties.
    fn get_property_of_type(&self, t: &Arc<Type>, name: &str) -> Option<Arc<Symbol>> {
        t.as_structured()?.members.get(name).cloned()
    }

    /// Get the type of a named property on a type, if the property exists.
    fn get_property_type_of_type(&mut self, t: &Arc<Type>, name: &str) -> Option<Arc<Type>> {
        let sym = self.get_property_of_type(t, name)?;
        Some(self.get_type_of_symbol(&sym))
    }

    /// Whether a structured type has a non-optional declaration of `name`.
    /// Returns a `PropertyPresence` tri-state.
    fn type_has_property(&self, t: &Arc<Type>, name: &str) -> PropertyPresence {
        if let Some(structured) = t.as_structured() {
            if let Some(sym) = structured.members.get(name) {
                if sym.flags.contains(SymbolFlags::Optional) {
                    return PropertyPresence::Maybe;
                }
                return PropertyPresence::Definitely;
            }
            if !structured.index_infos.is_empty() {
                return PropertyPresence::Maybe;
            }
            return PropertyPresence::DefinitelyNot;
        }
        // For object types without structured data, be conservative.
        if t.flags.contains(TypeFlags::Object) {
            return PropertyPresence::Maybe;
        }
        // Primitives, literals, etc. don't have properties.
        PropertyPresence::DefinitelyNot
    }

    /// Get the instance type of a constructor function type.
    ///
    /// Tries (in order):
    ///   1. The `prototype` property's type (if not `any`).
    ///   2. The union of return types of the construct signatures.
    ///   3. `None` to signal "no instance type available".
    ///
    /// Mirrors Go's `getInstanceType` (flow.go ~L953).
    fn get_instance_type_of_constructor(
        &mut self,
        ctor_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // 1. Try the `prototype` property.
        if let Some(prop_sym) = self.get_property_of_type(ctor_type, "prototype") {
            let prop_type = self.get_type_of_symbol(&prop_sym);
            if !prop_type.flags.contains(TypeFlags::Any) {
                return Some(prop_type);
            }
        }
        // 2. Fall back to construct signatures' return types.
        let construct_sigs =
            self.get_signatures_of_type(ctor_type, SignatureKind::Construct);
        if !construct_sigs.is_empty() {
            let mut return_types: Vec<Arc<Type>> = Vec::new();
            for sig in &construct_sigs {
                if let Some(rt) = self.get_return_type_of_signature(sig) {
                    if !return_types.iter().any(|t| Arc::ptr_eq(t, &rt)) {
                        return_types.push(rt);
                    }
                }
            }
            if !return_types.is_empty() {
                return Some(self.get_union_type(return_types));
            }
        }
        None
    }

    /// Get the property name from a node that's expected to be a string
    /// literal, number literal, identifier, or property access expression
    /// (`x.kind`, `x["kind"]`).
    ///
    /// Used by `in` narrowing and discriminant narrowing to extract the
    /// property name being tested.
    fn get_accessed_property_name_from_node(node: &Arc<Node>) -> Option<String> {
        match &node.data {
            NodeData::StringLiteral(s) => Some(s.text.clone()),
            NodeData::NumericLiteral(n) => Some(n.text.clone()),
            NodeData::Identifier(id) => Some(id.text.clone()),
            NodeData::PropertyAccessExpression(pa) => {
                Some(pa.name.text().to_string())
            }
            NodeData::ElementAccessExpression(ea) => {
                Self::get_accessed_property_name_from_node(&ea.argument_expression)
            }
            _ => None,
        }
    }

    /// Whether `node` is a property access on `symbol`, e.g.
    /// `symbol.kind` or `symbol["kind"]`.
    fn is_property_access_on_symbol(
        &self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {
        match &node.data {
            NodeData::PropertyAccessExpression(pa) => {
                self.is_symbol_identifier(&pa.expression, symbol)
            }
            NodeData::ElementAccessExpression(ea) => {
                self.is_symbol_identifier(&ea.expression, symbol)
            }
            _ => false,
        }
    }

    /// Filter `type_` (a union) to keep only constituents assignable to
    /// `candidate`. For non-union types, return `candidate` if the
    /// current type is assignable to it, otherwise the original type.
    fn narrow_to_subtype(
        &mut self,
        type_: &Arc<Type>,
        candidate: &Arc<Type>,
    ) -> Arc<Type> {
        // `any` → candidate (matches Go's getNarrowedTypeWorker).
        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(candidate);
        }
        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let matching: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| self.is_type_assignable_to(t, candidate))
                .collect();
            return self.rebuild_union_or_never(type_, matching);
        }
        // Non-union: narrow to candidate if it's a subtype of the current
        // type; otherwise leave unchanged.
        if self.is_type_assignable_to(candidate, type_) {
            Arc::clone(candidate)
        } else {
            Arc::clone(type_)
        }
    }

    /// Remove from a union all constituents assignable to `candidate`.
    /// For non-union types, return `never` if the type is assignable to
    /// `candidate`, otherwise the original type.
    fn remove_subtype_from_union(
        &mut self,
        type_: &Arc<Type>,
        candidate: &Arc<Type>,
    ) -> Arc<Type> {
        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !self.is_type_assignable_to(t, candidate))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
        if self.is_type_assignable_to(type_, candidate) {
            self.never_type()
        } else {
            Arc::clone(type_)
        }
    }

    /// Rebuild a union from the filtered constituents. Returns `never`
    /// when the list is empty, the single type when only one remains,
    /// or builds a fresh `Union` type otherwise.
    fn rebuild_union_or_never(
        &mut self,
        original: &Arc<Type>,
        constituents: Vec<Arc<Type>>,
    ) -> Arc<Type> {
        if constituents.is_empty() {
            return self.never_type();
        }
        if constituents.len() == 1 {
            return constituents.into_iter().next().expect("exactly one");
        }
        // If the constituents are pointer-identical to the original, return
        // the original to preserve caching.
        if let TypeData::Union(u) = &original.data {
            if u.union_or_intersection.types.len() == constituents.len()
                && u
                    .union_or_intersection
                    .types
                    .iter()
                    .zip(constituents.iter())
                    .all(|(a, b)| Arc::ptr_eq(a, b))
            {
                return Arc::clone(original);
            }
        }
        self.get_union_type(constituents)
    }
}

/// Tri-state for whether a property is present on a type.
#[derive(Clone, Copy, PartialEq)]
enum PropertyPresence {
    /// The type definitely has the property (non-optional declaration).
    Definitely,
    /// The type might have the property (optional declaration or index
    /// signature).
    Maybe,
    /// The type definitely does not have the property.
    DefinitelyNot,
}

impl PropertyPresence {
    fn is_definitely(self) -> bool {
        matches!(self, PropertyPresence::Definitely)
    }
    fn is_definitely_not(self) -> bool {
        matches!(self, PropertyPresence::DefinitelyNot)
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
