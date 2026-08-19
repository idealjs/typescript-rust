//! Control flow narrowing.
//!
//! Ported from `internal/checker/flow.go`. Walks the flow graph built by
//! the binder to narrow types based on control-flow constraints (e.g.
//! `if (x !== null)` narrows `x` to exclude `null` in the then-branch).
//!
//! ## Algorithm
//!
//! `type_at_flow_node` walks the flow graph built by the binder to compute
//! the type of a symbol AT a given flow node. Each node's type is a pure
//! function of its antecedents' types plus the node's own constraint:
//!
//! - **TRUE_CONDITION/FALSE_CONDITION** — narrow the antecedent type by the
//!   associated condition (`if (x !== null)` removes `null`).
//! - **ASSIGNMENT** to `x` — the node's type is the RHS type.
//! - **Junction** (multiple antecedents) — union of antecedent types.
//!
//! Because a node's type depends only on the node (for a fixed symbol and
//! query), results are memoized per flow node within one
//! `get_narrowed_type_of_symbol` query — mirroring Go's `flowTypeCache`.
//! Without the memo, walks through junction-heavy graphs (labeled loops with
//! breaks/continues) explode combinatorially. Loop back-edges are cut by
//! seeding the memo with the declared type when a node is re-entered while
//! still being computed (one-unrolling, mirroring Go's cache pre-seeding).
//! Recursion is additionally capped at `FLOW_MAX_DEPTH`.

use std::sync::Arc;

use crate::ast::{FlowFlags, FlowNode, Node, NodeData, NodeFlags, Symbol, SymbolFlags, SyntaxKind};

use super::checker::Checker;
use super::types::*;

/// Maximum recursion depth for `type_at_flow_node`. Go reports TS2563 at
/// 2000 recursive invocations (flow.go ~L118) — matching that number keeps
/// legitimate long chains (binaryArithmeticControlFlowGraphNotTooLarge,
/// ~1600 assignments) below the limit while 10k-node bodies
/// (largeControlFlowGraph) still trip it, as in official. Deep walks are
/// safe stack-wise: the harness worker compiles on a dedicated large-stack
/// thread.
const FLOW_MAX_DEPTH: u32 = 2000;

/// The reference a flow query narrows. Go threads a single `f.reference`
/// node through `getTypeAtFlowNode` and matches against flow nodes with
/// `isMatchingReference` (flow.go ~L1597); identifiers match by resolved
/// symbol and property accesses match structurally. This port keeps the
/// identifier path keyed by resolved symbol (`FlowRef::Symbol`) and adds
/// the structural path for property/element-access references
/// (`FlowRef::Node`).
#[derive(Clone)]
pub(super) enum FlowRef {
    Symbol(Arc<Symbol>),
    Node(Arc<Node>),
}

impl FlowRef {
    /// A node to anchor TS2563 on (Go's `f.reference`): the reference node
    /// itself, or the first declaration of the referenced symbol.
    fn anchor_node(&self) -> Option<Arc<Node>> {
        match self {
            FlowRef::Node(n) => Some(Arc::clone(n)),
            FlowRef::Symbol(s) => s.declarations.first().map(Arc::clone),
        }
    }
}

/// Per-query memoization state for `type_at_flow_node`. `memo` caches the
/// computed type for each visited flow node; `on_path` tracks nodes currently
/// being computed (loop back-edge cycle detection).
#[derive(Default)]
struct FlowQuery {
    memo: std::collections::HashMap<usize, Arc<Type>>,
    on_path: std::collections::HashSet<usize>,
    /// Active reduce-label scopes (Go `f.reduceLabels`): each entry maps a
    /// branch-label node to the reduced antecedent set to use instead of
    /// its full one while the walk is inside that reduce label.
    reduce_labels: Vec<(std::sync::Arc<FlowNode>, Vec<std::sync::Arc<FlowNode>>)>,
}

/// The kind of narrowing to apply for a condition.
#[derive(Clone, Copy, Debug, PartialEq)]
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
        let target = FlowRef::Symbol(Arc::clone(symbol));
        let key = self.flow_cache_key(&target, flow, &declared);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Arc::clone(cached);
        }
        self.flow_type_cache.insert(key, Arc::clone(&declared));
        let mut query = FlowQuery::default();
        let narrowed = self.type_at_flow_node(&declared, &declared, flow, &target, 0, &mut query);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));
        narrowed
    }

    /// Flow-narrow an arbitrary reference expression (`obj.val`,
    /// `obj['val']`, `this.x`) from its declared type. Mirrors Go's
    /// `getFlowTypeOfReference` (flow.go ~L77): reads the flow node the
    /// binder attached to the reference and walks the flow graph matching
    /// references structurally via `isMatchingReference`.
    pub fn get_flow_type_of_reference(
        &mut self,
        reference: &Arc<Node>,
        declared: &Arc<Type>,
    ) -> Arc<Type> {
        let Some(flow) = self.program.symbol_map().flow_node_of(reference).map(Arc::clone)
        else {
            return Arc::clone(declared);
        };
        if self.flow_analysis_disabled {
            return Arc::clone(declared);
        }
        let target = FlowRef::Node(Arc::clone(reference));
        let key = self.flow_cache_key(&target, &flow, declared);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Arc::clone(cached);
        }
        self.flow_type_cache.insert(key, Arc::clone(declared));
        let mut query = FlowQuery::default();
        let narrowed = self.type_at_flow_node(declared, declared, &flow, &target, 0, &mut query);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));
        // The reference is the operand of a non-null assertion (`x!`) and
        // the narrowed type would become `never` once null/undefined are
        // excluded (without already being `never`): the assertion asserts
        // the declared type instead. Mirrors Go's guard in
        // `getFlowTypeOfReferenceEx` (flow.go ~L111).
        if let Some(parent) = &reference.parent {
            if parent.kind == SyntaxKind::NonNullExpression
                && !narrowed.flags.contains(TypeFlags::Never)
                && self.type_is_never_after_removing_nullable(&narrowed)
            {
                return Arc::clone(declared);
            }
        }
        narrowed
    }

    /// Whether removing null/undefined constituents empties the type (i.e.
    /// every constituent is nullable). Used by the non-null-assertion guard
    /// in `get_flow_type_of_reference`.
    fn type_is_never_after_removing_nullable(&self, t: &Arc<Type>) -> bool {
        if !self.strict_null_checks {
            return false;
        }
        if t.is_union() {
            return self.constituent_types(t).iter().all(|c| {
                c.flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
            });
        }
        t.flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
    }

    /// Definite-assignment query: the flow type of `symbol` at `node`'s
    /// flow point when the declaration seeds `declared | undefined` at the
    /// flow start. Mirrors Go's `checkIdentifier` path (checker.go ~L11226)
    /// where `initialType = getOptionalType(t)` and a flow type that still
    /// contains `undefined` (while the declared type doesn't) triggers
    /// TS2454. Returns the flow type; the caller decides reporting.
    pub fn get_definite_assignment_flow_type(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        if self.flow_analysis_disabled {
            return None;
        }
        let flow = self
            .program
            .symbol_map()
            .flow_node_of(node)
            .map(Arc::clone)?;
        let declared = self.get_type_of_symbol(symbol);
        let undefined = self.undefined_type();
        let initial = if self.type_contains_undefined_local(&declared) {
            Arc::clone(&declared)
        } else {
            self.get_union_type(vec![Arc::clone(&declared), undefined])
        };
        let target = FlowRef::Symbol(Arc::clone(symbol));
        let key = self.flow_cache_key(&target, &flow, &initial);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Some(Arc::clone(cached));
        }
        self.flow_type_cache.insert(key, Arc::clone(&declared));
        let mut query = FlowQuery::default();
        let narrowed = self.type_at_flow_node(&declared, &initial, &flow, &target, 0, &mut query);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));
        Some(narrowed)
    }

    /// Local undefined-containment check for flow queries.
    fn type_contains_undefined_local(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Undefined) {
            return true;
        }
        if t.is_union() {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|c| c.flags.contains(TypeFlags::Undefined));
            }
        }
        false
    }

    /// Compute a cache key for (target, flow, initial-type) triples. The
    /// initial type participates because Go's flow cache keys on
    /// `(declaredType, initialType)` — a definite-assignment query seeded
    /// with `T | undefined` differs from a plain narrowing query.
    fn flow_cache_key(&self, target: &FlowRef, flow: &Arc<FlowNode>, initial: &Arc<Type>) -> u64 {
        let ref_part = match target {
            FlowRef::Symbol(symbol) => symbol.id(),
            FlowRef::Node(node) => node.id(),
        };
        let flow_ptr = Arc::as_ptr(flow) as *const FlowNode as u64;
        let initial_ptr = Arc::as_ptr(initial) as *const Type as u64;
        // Mix the three: rotate the reference part, XOR with flow pointer
        // and the initial type's identity.
        (ref_part.rotate_left(17) ^ flow_ptr).rotate_left(29) ^ initial_ptr
    }

    /// Compute the type of `symbol` AT `flow` — the core flow-typing
    /// routine. Each node's type is derived from its ANTECEDENTS' types plus
    /// the node's own constraint (condition/assignment/junction/…), so a
    /// node's result is a pure function of the node and can be memoized for
    /// the rest of the query.
    ///
    /// The walk is guarded against cycles (loop back-edges: `continue`
    /// targets the loop's pre-loop label, which the body flows back into).
    /// A node re-entered while still being computed gets the declared type
    /// seeded into the memo — mirroring how Go's `getTypeAtFlowNode` breaks
    /// back-edges via its pre-seeded flow-type cache (one unrolling).
    fn type_at_flow_node(
        &mut self,
        declared: &Arc<Type>,
        initial: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
        depth: u32,
        query: &mut FlowQuery,
    ) -> Arc<Type> {
        let key = Arc::as_ptr(flow) as usize;
        if let Some(t) = query.memo.get(&key) {
            return Arc::clone(t);
        }
        if !query.on_path.insert(key) {
            // Cycle: stop walking this path (one-unrolling semantics). The
            // seeded entry is overwritten once the in-flight node completes.
            query.memo.insert(key, Arc::clone(initial));
            return Arc::clone(initial);
        }
        let result = if depth >= FLOW_MAX_DEPTH {
            // Go (flow.go ~L105 `getTypeAtFlowNode`): after 2000 recursive
            // invocations it reports TS2563 and disables further control
            // flow analysis for the containing function/module body so
            // subsequent references skip the walk entirely (without this,
            // a 10k-assignment body re-walks the graph per reference —
            // quadratic time and unbounded memo growth, observed as an
            // OOM abort on largeControlFlowGraph).
            if !self.flow_analysis_disabled {
                self.flow_analysis_disabled = true;
                self.report_flow_control_error(target);
            }
            Arc::clone(initial)
        } else {
            self.compute_type_at_flow_node(declared, initial, flow, target, depth, query)
        };
        query.on_path.remove(&key);
        query.memo.insert(key, Arc::clone(&result));
        result
    }

    /// TS2563 (Go `reportFlowControlError`, flow.go ~L1590): the containing
    /// function-or-module body is too large for control flow analysis.
    /// Reported on the first token of the enclosing block's statement list.
    fn report_flow_control_error(&mut self, target: &FlowRef) {
        use crate::ast::SyntaxKind;
        let Some(anchor) = target.anchor_node() else { return };
        let mut block: Option<Arc<Node>> = None;
        let mut cur = anchor.parent.clone();
        while let Some(n) = cur {
            let is_function_or_module_block = match n.kind {
                SyntaxKind::SourceFile | SyntaxKind::ModuleBlock => true,
                SyntaxKind::Block => n
                    .parent
                    .as_ref()
                    .is_some_and(|p| crate::ast::utilities::is_function_like_kind(p.kind)),
                _ => false,
            };
            if is_function_or_module_block {
                block = Some(Arc::clone(&n));
                break;
            }
            cur = n.parent.clone();
        }
        let Some(block) = block else { return };
        // Range of the first token of the block's statement list (Go:
        // `scanner.GetRangeOfTokenAtPosition(sourceFile, statements.Pos())`).
        let mut loc = block.loc;
        if let Some(stmts) = match &block.data {
            crate::ast::NodeData::SourceFile(d) => Some(&d.statements),
            crate::ast::NodeData::ModuleBlock(d) => Some(&d.statements),
            crate::ast::NodeData::Block(d) => Some(&d.statements),
            _ => None,
        } && let Some(first) = stmts.nodes.first()
        {
            loc = crate::core::text::TextRange::new(first.loc.pos as usize, first.loc.pos as usize + 1);
        }
        self.diagnostics.add(crate::ast::Diagnostic::new(
            self.current_file.clone(),
            loc,
            crate::diagnostics::messages_generated::
                THE_CONTAINING_FUNCTION_OR_MODULE_BODY_IS_TOO_LARGE_FOR_CONTROL_FLOW_ANALYSIS,
            Vec::new(),
        ));
    }

    fn compute_type_at_flow_node(
        &mut self,
        declared: &Arc<Type>,
        initial: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
        depth: u32,
        query: &mut FlowQuery,
    ) -> Arc<Type> {
        // UNREACHABLE flow → the code path is dead; `never`.
        if flow.flags.contains(FlowFlags::UNREACHABLE) {
            return self.never_type();
        }

        // START flow → the query's initial type (Go's `f.initialType`;
        // definite-assignment queries seed `T | undefined` here).
        if flow.flags.contains(FlowFlags::START) {
            return Arc::clone(initial);
        }

        // TRUE_CONDITION / FALSE_CONDITION → narrow the ANTECEDENT type by
        // the condition. Use `intersects` (not `contains`) because CONDITION
        // is a composite mask of TRUE_CONDITION | FALSE_CONDITION, and a flow
        // node only has one of those bits set. Mirrors Go's
        // `flags&FlowFlagsCondition != 0`.
        if flow.flags.intersects(FlowFlags::CONDITION) {
            let kind = if flow.flags.contains(FlowFlags::TRUE_CONDITION) {
                NarrowKind::TrueBranch
            } else {
                NarrowKind::FalseBranch
            };
            let antecedent_type = self.antecedent_type_at(declared, initial, flow, target, depth, query);
            if let Some(expr) = &flow.node {
                return self.narrow_by_expression(&antecedent_type, expr, target, kind, depth);
            }
            return antecedent_type;
        }

        // ASSIGNMENT → mirrors Go's `getTypeAtFlowAssignment` (flow.go
        // ~L220): a matching assignment reduces a union declared type by
        // the assigned type (`getAssignmentReducedType`); a non-union
        // declared type is returned unchanged; evolving arrays keep the
        // assigned type; assignments to a left-hand part of a dotted
        // reference reset it to the declared type.
        if flow.flags.contains(FlowFlags::ASSIGNMENT) {
            if let Some(expr) = &flow.node {
                if let Some(t) = self.assignment_flow_type(expr, target, declared) {
                    return t;
                }
                // An assignment to a left-hand part of a dotted reference
                // (`obj` of `obj.val`) invalidates narrowing: back to the
                // declared type. Mirrors Go's `containsMatchingReference`
                // branch in `getTypeAtFlowAssignment` (flow.go ~L255).
                if let FlowRef::Node(reference) = target {
                    if self.contains_matching_reference(reference, expr) {
                        return Arc::clone(declared);
                    }
                }
                // `for (const k in ref)` acts as a non-null assertion on
                // `ref` (Go flow.go ~L269).
                if expr.kind == SyntaxKind::VariableDeclaration {
                    if let Some(for_in_expr) = Self::for_in_expression_of(expr) {
                        if self.expr_matches_target(&for_in_expr, target)
                            || self.optional_chain_contains_target(&for_in_expr, target)
                        {
                            let ante = self.antecedent_type_at(
                                declared, initial, flow, target, depth, query,
                            );
                            if self.strict_null_checks {
                                return self.remove_nullable_from_union(&ante);
                            }
                            return ante;
                        }
                    }
                }
            }
            // Not an assignment to our target; continue through the antecedent.
            return self.antecedent_type_at(declared, initial, flow, target, depth, query);
        }

        // SWITCH_CLAUSE → narrow based on the switch case expression.
        // Mirrors Go's `getTypeAtSwitchClause` (flow.go ~L1046). We first
        // recurse into the antecedent to get the narrowed type at this flow
        // point, then apply switch-specific narrowing on top.
        if flow.flags.contains(FlowFlags::SWITCH_CLAUSE) {
            let antecedent_type = self.antecedent_type_at(declared, initial, flow, target, depth, query);
            return self.narrow_by_switch_clause(&antecedent_type, flow, target);
        }

        // ARRAY_MUTATION → evolve the element type of an evolving array.
        // Mirrors Go's `getTypeAtFlowArrayMutation` (flow.go ~L1383). Only
        // applies when the declared type is `autoType`/`autoArrayType` (an
        // evolving array). If the mutated array (`arr` in `arr.push(1)`)
        // is the same reference as our symbol, the element type is evolved
        // by unioning each argument's type onto the antecedent's type.
        if flow.flags.contains(FlowFlags::ARRAY_MUTATION) {
            if let Some(node) = &flow.node {
                let is_evolving = declared.object_flags.contains(ObjectFlags::EvolvingArray)
                    || self.is_auto_array_type(declared);
                if is_evolving {
                    let pre_type = self.antecedent_type_at(declared, initial, flow, target, depth, query);
                    if let Some(evolved) =
                        self.evolve_array_at_mutation(node, &pre_type, target)
                    {
                        return evolved;
                    }
                    return pre_type;
                }
            }
            // Not an evolving array or not our symbol; recurse.
            return self.antecedent_type_at(declared, initial, flow, target, depth, query);
        }

        // CALL → assertion function narrowing. If the call is to an
        // assertion function (`asserts x` or `asserts x is T`), the
        // argument is narrowed after the call (since the function throws
        // if the assertion fails). Mirrors Go's `getTypeAtFlowCall`
        // (flow.go ~L288). Non-assertion calls just recurse.
        if flow.flags.contains(FlowFlags::CALL) {
            let antecedent_type = self.antecedent_type_at(declared, initial, flow, target, depth, query);
            if let Some(call_expr) = &flow.node {
                return self.narrow_by_assertion_call(&antecedent_type, call_expr, target);
            }
            return antecedent_type;
        }

        // REDUCE_LABEL (Go flow.go ~L181): while the walk is inside this
        // node, the target branch label's antecedent set is replaced by
        // this node's reduced antecedents. This is how control flow past a
        // try-finally only considers the normal-completion (or return, or
        // exception) flows through the finally block.
        if flow.flags.contains(FlowFlags::REDUCE_LABEL) {
            if let Some(reduce_target) = &flow.reduce_target {
                query.reduce_labels.push((
                    Arc::clone(reduce_target),
                    flow.antecedents.clone(),
                ));
                let t = self.antecedent_type_at(declared, initial, flow, target, depth, query);
                query.reduce_labels.pop();
                return t;
            }
            return self.antecedent_type_at(declared, initial, flow, target, depth, query);
        }

        // Junction (multiple antecedents): the union of the antecedent
        // types. This handles if/else merge points, loop back-edges, and
        // switch clause falls. A branch label that is the target of an
        // active reduce-label scope uses the reduced antecedent set
        // instead (Go `getBranchLabelAntecedents`).
        if flow.antecedents.len() > 1 {
            let effective: Vec<Arc<FlowNode>> = query
                .reduce_labels
                .iter()
                .rev()
                .find(|(reduce_target, _)| Arc::ptr_eq(reduce_target, flow))
                .map(|(_, ants)| ants.clone())
                .unwrap_or_else(|| flow.antecedents.clone());
            if effective.len() == 1 {
                return self.type_at_flow_node(
                    declared,
                    initial,
                    &effective[0],
                    target,
                    depth + 1,
                    query,
                );
            }
            let mut antecedent_types: Vec<Arc<Type>> = Vec::new();
            for antecedent in &effective {
                let t =
                    self.type_at_flow_node(declared, initial, antecedent, target, depth + 1, query);
                if !antecedent_types.iter().any(|u| Arc::ptr_eq(u, &t)) {
                    antecedent_types.push(t);
                }
            }
            // If only one distinct result, return it.
            if antecedent_types.len() == 1 {
                return antecedent_types.into_iter().next().expect("exactly one");
            }
            // If multiple distinct results, compute their union. Dead
            // branches contribute `never`, which the union absorbs.
            if antecedent_types.is_empty() {
                return Arc::clone(declared);
            }
            return self.get_union_type(antecedent_types);
        }

        // Single antecedent (or a branch label reduced to one) → recurse.
        self.antecedent_type_at(declared, initial, flow, target, depth, query)
    }

    /// The type at `flow`'s single antecedent, or the declared type when the
    /// node has no antecedent.
    fn antecedent_type_at(
        &mut self,
        declared: &Arc<Type>,
        initial: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
        depth: u32,
        query: &mut FlowQuery,
    ) -> Arc<Type> {
        match &flow.antecedent {
            Some(antecedent) => {
                self.type_at_flow_node(declared, initial, antecedent, target, depth + 1, query)
            }
            None => Arc::clone(initial),
        }
    }

    /// Narrow a type based on a single condition expression.
    ///
    /// `kind` indicates whether the condition is true or false.
    fn narrow_by_expression(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
        depth: u32,
    ) -> Arc<Type> {
        // Parenthesized expression: unwrap and recurse.
        if expr.kind == SyntaxKind::ParenthesizedExpression {
            if let NodeData::ParenthesizedExpression(p) = &expr.data {
                return self.narrow_by_expression(type_, &p.expression, target, kind, depth);
            }
        }
        // Logical AND: `a && b` — both sides are true in the true branch.
        if expr.kind == SyntaxKind::BinaryExpression {
            if let NodeData::BinaryExpression(bin) = &expr.data {
                if bin.operator_token.kind == SyntaxKind::AmpersandAmpersandToken {
                    if kind == NarrowKind::TrueBranch {
                        let narrowed =
                            self.narrow_by_expression(type_, &bin.left, target, kind, depth);
                        return self
                            .narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                    }
                    // False branch of `a && b`: either `a` is false OR
                    // (`a` is true AND `b` is false). We can't narrow
                    // precisely, so just check the left side.
                    return self.narrow_by_expression(
                        type_,
                        &bin.left,
                        target,
                        NarrowKind::FalseBranch,
                        depth,
                    );
                }
                if bin.operator_token.kind == SyntaxKind::BarBarToken {
                    if kind == NarrowKind::FalseBranch {
                        // False branch of `a || b`: both `a` and `b` are false.
                        let narrowed =
                            self.narrow_by_expression(type_, &bin.left, target, kind, depth);
                        return self
                            .narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                    }
                    // True branch of `a || b`: at least one is true. Check left.
                    return self.narrow_by_expression(
                        type_,
                        &bin.left,
                        target,
                        NarrowKind::TrueBranch,
                        depth,
                    );
                }
                if bin.operator_token.kind == SyntaxKind::QuestionQuestionToken {
                    // Nullish coalescing: `a ?? b`. Mirrors Go's
                    // `bindLogicalLikeExpression` (binder.go ~L2261) combined
                    // with `narrowType`'s `??` parent check (flow.go ~L379).
                    if kind == NarrowKind::TrueBranch {
                        // True branch: the result is truthy. We can't narrow
                        // `a` (it could be null/undefined if `b` is truthy)
                        // or `b` (only evaluated when `a` is null/undefined).
                        // Go's flow analysis unions (a non-null) with
                        // (a null/undefined, b truthy), which cancels out to
                        // the original type.
                        return Arc::clone(type_);
                    }
                    // False branch: `a` is null/undefined and `b` is falsy.
                    // Narrow left by optionality (keep only null/undefined),
                    // then narrow right by truthiness (falsy).
                    let narrowed =
                        self.narrow_by_optionality(type_, &bin.left, target, kind, depth);
                    return self.narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
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
                        target,
                        inverted,
                        depth,
                    );
                }
            }
        }

        // Binary comparison: `x === value`, `x !== null`, `typeof x === "string"`, etc.
        if expr.kind == SyntaxKind::BinaryExpression {
            return self.narrow_by_binary(type_, expr, target, kind);
        }

        // Call expression: `isString(x)` — type predicate narrowing.
        // A user-defined type guard function (declared with `x is T`) narrows
        // its argument in the true/false branches.
        if expr.kind == SyntaxKind::CallExpression {
            return self.narrow_by_call_expression(type_, expr, target, kind);
        }

        // Const alias inlining: if `expr` is an Identifier that resolves to
        // a `const` variable with a simple initializer (no type annotation),
        // and the identifier is NOT the symbol being narrowed, narrow by
        // the initializer expression instead. Mirrors Go's `narrowType`
        // KindIdentifier case (flow.go ~L383). Capped at 5 levels to
        // prevent infinite recursion.
        if expr.kind == SyntaxKind::Identifier
            && !self.expr_matches_target(expr, target)
            && self.flow_inline_level < 5
        {
            if let Some(init_expr) = self.const_alias_initializer(expr) {
                self.flow_inline_level += 1;
                let result = self.narrow_by_expression(type_, &init_expr, target, kind, depth);
                self.flow_inline_level -= 1;
                return result;
            }
        }

        // Bare identifier: `if (x)` — truthiness narrowing.
        if self.expr_matches_target(expr, target) {
            return self.narrow_by_truthiness(type_, kind);
        }

        // Optional chain containing the symbol: `if (x?.a)` — in the true
        // branch, `x` cannot be null/undefined (otherwise `x?.a` would be
        // `undefined`, which is falsy). Mirrors Go's `narrowTypeByTruthiness`
        // optional chain containment check (flow.go ~L432).
        if kind == NarrowKind::TrueBranch {
            let contains = self.optional_chain_contains_target(expr, target);
            if contains {
                return self.remove_nullable_from_union(type_);
            }
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
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return Arc::clone(type_);
        };
        let op = bin.operator_token.kind;

        // `instanceof`: `x instanceof Foo` — narrow to the instance type
        // of `Foo` in the true branch; remove it in the false branch.
        if op == SyntaxKind::InstanceOfKeyword {
            return self.narrow_by_instanceof(type_, &bin.left, &bin.right, target, kind);
        }

        // `in`: `"prop" in x` — narrow `x` by property presence.
        if op == SyntaxKind::InKeyword {
            return self.narrow_by_in_keyword(type_, &bin.left, &bin.right, target, kind);
        }

        // Equality/inequality: `===`, `!==`, `==`, `!=`
        let is_strict = op == SyntaxKind::EqualsEqualsEqualsToken
            || op == SyntaxKind::ExclamationEqualsEqualsToken;
        let is_loose =
            op == SyntaxKind::EqualsEqualsToken || op == SyntaxKind::ExclamationEqualsToken;
        if !is_strict && !is_loose {
            return Arc::clone(type_);
        }

        let is_equality =
            op == SyntaxKind::EqualsEqualsEqualsToken || op == SyntaxKind::EqualsEqualsToken;
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
            && self.typeof_expr_matches_target(&bin.left, target)
        {
            return self.narrow_by_typeof(type_, &bin.right, narrow_to_value, is_loose);
        }
        if bin.right.kind == SyntaxKind::TypeOfExpression
            && self.typeof_expr_matches_target(&bin.right, target)
        {
            return self.narrow_by_typeof(type_, &bin.left, narrow_to_value, is_loose);
        }

        // Handle `typeof obj.prop === "string"` patterns — typeof on a
        // discriminant property access. Mirrors Go's `narrowTypeByTypeof`
        // (flow.go ~L612) which calls `getDiscriminantPropertyAccess` when
        // the typeof target isn't the symbol directly but a property
        // access on it.
        if bin.left.kind == SyntaxKind::TypeOfExpression {
            if let Some(narrowed) = self.try_narrow_by_typeof_discriminant(
                type_,
                &bin.left,
                &bin.right,
                target,
                narrow_to_value,
            ) {
                return narrowed;
            }
        }
        if bin.right.kind == SyntaxKind::TypeOfExpression {
            if let Some(narrowed) = self.try_narrow_by_typeof_discriminant(
                type_,
                &bin.right,
                &bin.left,
                target,
                narrow_to_value,
            ) {
                return narrowed;
            }
        }

        // Discriminated union narrowing: `obj.kind === "value"` narrows
        // `obj` to the union constituent whose `kind` property matches.
        if let Some(narrowed) = self.try_narrow_by_discriminant_property(type_, expr, target, kind)
        {
            return narrowed;
        }

        // Optional chain containment: `x?.a === value` — if the value
        // excludes null/undefined, then `x` cannot be null/undefined.
        // Mirrors Go's `narrowTypeByOptionalChainContainment` (flow.go
        // ~L1019).
        if self.optional_chain_contains_target(&bin.left, target) {
            return self.narrow_by_optional_chain_containment(type_, op, &bin.right, kind);
        }
        if self.optional_chain_contains_target(&bin.right, target) {
            return self.narrow_by_optional_chain_containment(type_, op, &bin.left, kind);
        }

        // Simple `x === value` or `value === x` patterns.
        let (value_node, is_symbol_on_left) = if self.expr_matches_target(&bin.left, target) {
            (&bin.right, true)
        } else if self.expr_matches_target(&bin.right, target) {
            (&bin.left, false)
        } else {
            return Arc::clone(type_);
        };
        let _ = is_symbol_on_left;

        let value_type = self.get_type_of_node(value_node);
        self.narrow_by_equality(type_, &value_type, narrow_to_value, is_loose)
    }

    /// Narrow `type_` based on an equality comparison with `value_type`.
    ///
    /// Mirrors Go's `narrowTypeByEquality` (flow.go ~L556). Improvements over
    /// the previous simple intersect/remove logic:
    ///
    /// - Skips narrowing for `any`.
    /// - Distinguishes `null` vs `undefined` for strict equality (`===`/`!==`)
    ///   while treating both together for loose equality (`==`/`!=`).
    /// - In the true branch, filters constituents to those comparable to the
    ///   value (or coercible under `==`), then replaces primitive types with
    ///   matching literal types from the value (e.g. `string` → `"foo"`).
    /// - In the false branch, only narrows when the value is a unit type
    ///   (literal/enum), removing constituents that are unit-like and
    ///   comparable to the value.
    fn narrow_by_equality(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
        narrow_to_value: bool,
        is_loose: bool,
    ) -> Arc<Type> {
        // `any` is not narrowed by equality comparisons.
        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(type_);
        }
        // Boolean comparison narrowing: `x === true` / `x === false`.
        // When the type contains `Boolean` and the value is a boolean
        // literal, narrow to the matching literal (true branch) or the
        // opposite literal (false branch). Mirrors Go's
        // `narrowTypeByBooleanComparison` (flow.go ~L793).
        if value_type.flags.contains(TypeFlags::BooleanLiteral)
            && type_.flags.contains(TypeFlags::Boolean)
            && !is_loose
        {
            let is_true_value = match value_type.literal_value() {
                Some(LiteralValue::Boolean(b)) => *b,
                _ => true,
            };
            let target_is_true = if narrow_to_value {
                is_true_value
            } else {
                !is_true_value
            };
            return if target_is_true {
                self.true_type()
            } else {
                self.false_type()
            };
        }
        // Nullable value type (null or undefined): narrow by facts.
        if value_type.flags.intersects(TYPE_FLAGS_NULLABLE) {
            if !self.strict_null_checks {
                return Arc::clone(type_);
            }
            let value_is_null = value_type.flags.contains(TypeFlags::Null);
            // For loose equality (==/!=), both null and undefined match each
            // other. For strict equality (===/!==), narrow to the specific
            // nullable kind.
            return if is_loose {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TYPE_FLAGS_NULLABLE)
                } else {
                    self.remove_flags_from_union(type_, TYPE_FLAGS_NULLABLE)
                }
            } else if value_is_null {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TypeFlags::Null)
                } else {
                    self.remove_flags_from_union(type_, TypeFlags::Null)
                }
            } else {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TypeFlags::Undefined)
                } else {
                    self.remove_flags_from_union(type_, TypeFlags::Undefined)
                }
            };
        }
        if narrow_to_value {
            // True branch of `===`/`==` (or false branch of `!==`/`!=`):
            // keep constituents comparable to the value, or coercible under
            // loose equality. Then replace primitives with matching literals.
            let filtered = self.filter_comparable_or_coercible(type_, value_type, is_loose);
            self.replace_primitives_with_literals(&filtered, value_type)
        } else {
            // False branch: only narrow when the value is a unit type
            // (literal, enum, unique symbol). Remove constituents that are
            // unit-like and comparable to the value.
            if !value_type.flags.intersects(TYPE_FLAGS_UNIT) {
                return Arc::clone(type_);
            }
            self.remove_comparable_units(type_, value_type)
        }
    }

    /// Filter `type_` to keep only constituents comparable to `value_type`.
    /// For loose equality (`==`), also keeps constituents coercible under the
    /// double-equals rules (number/string/boolean vs number/string/boolean).
    ///
    /// Mirrors Go's `filterType` call in `narrowTypeByEquality` (flow.go ~L589).
    fn filter_comparable_or_coercible(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
        is_loose: bool,
    ) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let value_constituents = self.constituent_types(value_type);
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                // Keep if comparable to any value constituent.
                let comparable = value_constituents
                    .iter()
                    .any(|vc| self.is_type_comparable_to(t, vc));
                if comparable {
                    return true;
                }
                // For loose equality, also keep coercible types.
                is_loose
                    && value_constituents
                        .iter()
                        .any(|vc| Self::is_coercible_under_double_equals(t, vc))
            })
            .collect();
        self.rebuild_union_or_never(type_, matching)
    }

    /// Remove from `type_` all constituents that are unit-like (literal/enum)
    /// and comparable to `value_type`.
    ///
    /// Mirrors Go's `filterType` call in `narrowTypeByEquality` (flow.go ~L595).
    fn remove_comparable_units(&mut self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let value_constituents = self.constituent_types(value_type);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                // Only remove unit-like types.
                if !t.flags.intersects(TYPE_FLAGS_UNIT) {
                    return true;
                }
                // Keep if NOT comparable to any value constituent.
                !value_constituents
                    .iter()
                    .any(|vc| self.is_type_comparable_to(t, vc))
            })
            .collect();
        self.rebuild_union_or_never(type_, remaining)
    }

    /// Replace primitive types (string/number/bigint) in `type_` with the
    /// matching literal types from `value_type`.
    ///
    /// E.g. when comparing `string | number` against `"foo"`, the `string`
    /// constituent is replaced with `"foo"`. Mirrors Go's
    /// `replacePrimitivesWithLiterals` (flow.go ~L1886).
    fn replace_primitives_with_literals(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        // Only replace if the type has primitives and the value has literals.
        let has_primitives = type_
            .flags
            .intersects(TypeFlags::String | TypeFlags::Number | TypeFlags::BigInt);
        let has_literals = value_type
            .flags
            .intersects(TYPE_FLAGS_LITERAL | TypeFlags::TemplateLiteral | TypeFlags::StringMapping);
        if !has_primitives || !has_literals {
            return Arc::clone(type_);
        }
        // Collect literal constituents from the value type, grouped by kind.
        let value_constituents = self.constituent_types(value_type);
        let string_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| {
                t.flags.intersects(
                    TypeFlags::StringLiteral
                        | TypeFlags::TemplateLiteral
                        | TypeFlags::StringMapping,
                )
            })
            .cloned()
            .collect();
        let number_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| t.flags.contains(TypeFlags::NumberLiteral))
            .cloned()
            .collect();
        let bigint_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| t.flags.contains(TypeFlags::BigIntLiteral))
            .cloned()
            .collect();
        let constituents = self.constituent_types(type_);
        let mut result: Vec<Arc<Type>> = Vec::new();
        for t in constituents {
            if t.flags.contains(TypeFlags::String) {
                // Replace `string` with matching string literals. If the value
                // also has a `string` constituent, keep the primitive.
                let has_string_value =
                    value_type.flags.contains(TypeFlags::String) || string_literals.is_empty();
                if has_string_value {
                    result.push(t);
                } else {
                    result.extend(string_literals.iter().cloned());
                }
            } else if t.flags.contains(TypeFlags::Number) {
                let has_number_value =
                    value_type.flags.contains(TypeFlags::Number) || number_literals.is_empty();
                if has_number_value {
                    result.push(t);
                } else {
                    result.extend(number_literals.iter().cloned());
                }
            } else if t.flags.contains(TypeFlags::BigInt) {
                let has_bigint_value =
                    value_type.flags.contains(TypeFlags::BigInt) || bigint_literals.is_empty();
                if has_bigint_value {
                    result.push(t);
                } else {
                    result.extend(bigint_literals.iter().cloned());
                }
            } else {
                result.push(t);
            }
        }
        self.rebuild_union_or_never(type_, result)
    }

    /// Check if `source` is coercible to `target` under the `==` operator.
    ///
    /// Mirrors Go's `isCoercibleUnderDoubleEquals` (flow.go ~L1907). A type
    /// is coercible if it is a number/string/boolean-literal and the target
    /// is a number/string/boolean (or vice versa).
    fn is_coercible_under_double_equals(source: &Arc<Type>, target: &Arc<Type>) -> bool {
        source
            .flags
            .intersects(TypeFlags::Number | TypeFlags::String | TypeFlags::BooleanLiteral)
            && target
                .flags
                .intersects(TypeFlags::Number | TypeFlags::String | TypeFlags::Boolean)
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
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        if !self.expr_matches_target(left, target) {
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
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        if !self.expr_matches_target(right, target) {
            return Arc::clone(type_);
        }
        let Some(prop_name) = Self::get_accessed_property_name_from_node(left) else {
            return Arc::clone(type_);
        };
        let keep_present = match kind {
            NarrowKind::TrueBranch => true,
            NarrowKind::FalseBranch => false,
        };
        // Mirrors Go's `narrowTypeByInKeyword` (flow.go ~L988):
        // - True branch (`'p' in x` is true): remove types that definitely
        //   DON'T have `p`; keep types that have it or where presence is
        //   unknown (index signatures etc.).
        // - False branch (`'p' in x` is false): remove types that definitely
        //   HAVE `p`; keep types that don't have it or where presence is
        //   unknown.
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let has_prop = self.type_has_property(t, &prop_name);
                if keep_present {
                    !has_prop.is_definitely_not()
                } else {
                    !has_prop.is_definitely()
                }
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
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Option<Arc<Type>> {
        // Discriminant narrowing selects the union constituent that owns a
        // matching `kind` property; it only applies when the narrowed target
        // is the *receiver* (`obj` of `obj.kind`), never a property-access
        // reference (whose own type the plain equality path narrows).
        let FlowRef::Symbol(symbol) = target else {
            return None;
        };
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
        // Find which side is the property access on `symbol` — directly
        // (`obj.kind`) or through a const alias (`const k = obj.kind`,
        // `const { kind: k } = obj`). Mirrors Go's
        // `getCandidateDiscriminantPropertyAccess` identifier case
        // (flow.go ~L1460).
        let (access_node, value_node) = if let Some(alias) =
            self.discriminant_alias_access(&bin.left, symbol)
        {
            (alias, &bin.right)
        } else if let Some(alias) = self.discriminant_alias_access(&bin.right, symbol) {
            (alias, &bin.left)
        } else if self.is_property_access_on_symbol(&bin.left, symbol) {
            (Arc::clone(&bin.left), &bin.right)
        } else if self.is_property_access_on_symbol(&bin.right, symbol) {
            (Arc::clone(&bin.right), &bin.left)
        } else {
            return None;
        };
        let prop_name = Self::get_accessed_property_name_from_node(&access_node)?;
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
                if keep_matching { matches } else { !matches }
            })
            .collect();
        Some(self.rebuild_union_or_never(type_, filtered))
    }

    /// Try to narrow a union by a `typeof obj.prop === "typename"` comparison.
    ///
    /// Mirrors Go's `narrowTypeByTypeof` (flow.go ~L602) when the typeof
    /// target is a discriminant property access on `symbol` (e.g.,
    /// `typeof obj.kind === "string"`). Returns `Some(narrowed)` when the
    /// pattern matches, or `None` to fall through.
    fn try_narrow_by_typeof_discriminant(
        &mut self,
        type_: &Arc<Type>,
        typeof_expr: &Arc<Node>,
        type_name_node: &Arc<Node>,
        target: &FlowRef,
        narrow_to_value: bool,
    ) -> Option<Arc<Type>> {
        let FlowRef::Symbol(symbol) = target else {
            return None;
        };
        let NodeData::TypeOfExpression(typeof_data) = &typeof_expr.data else {
            return None;
        };
        let target = &typeof_data.expression;
        // Check if target is `obj.prop` (a property access on symbol) or a
        // const alias of one (`const k = obj.kind`).
        let owned = match self.discriminant_alias_access(target, symbol) {
            Some(alias) => alias,
            None if self.is_property_access_on_symbol(target, symbol) => Arc::clone(target),
            None => return None,
        };
        let prop_name = Self::get_accessed_property_name_from_node(&owned)?;
        // For non-union types, narrowing by discriminant is a no-op.
        if !type_.is_union() {
            return Some(Arc::clone(type_));
        }
        let type_name = match &type_name_node.data {
            NodeData::StringLiteral(data) => data.text.as_str(),
            _ => return None,
        };
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);
                let Some(prop_type) = prop_type else {
                    return false;
                };
                if narrow_to_value {
                    // True branch: keep constituents whose property type
                    // could match the typeof string (any constituent).
                    self.type_matches_typeof_any(&prop_type, type_name)
                } else {
                    // False branch: keep constituents whose property type
                    // could NOT match (i.e., not all constituents match).
                    !self.type_matches_typeof_all(&prop_type, type_name)
                }
            })
            .collect();
        Some(self.rebuild_union_or_never(type_, filtered))
    }

    /// Check if ANY constituent of `t` matches the typeof string.
    /// Used for the true branch of `typeof obj.prop === "typename"`.
    fn type_matches_typeof_any(&self, t: &Arc<Type>, type_name: &str) -> bool {
        let constituents = self.constituent_types(t);
        constituents
            .iter()
            .any(|c| self.constituent_matches_typeof(c, type_name))
    }

    /// Check if ALL constituents of `t` match the typeof string.
    /// Used for the false branch of `typeof obj.prop === "typename"`.
    fn type_matches_typeof_all(&self, t: &Arc<Type>, type_name: &str) -> bool {
        let constituents = self.constituent_types(t);
        !constituents.is_empty()
            && constituents
                .iter()
                .all(|c| self.constituent_matches_typeof(c, type_name))
    }

    /// Check if a single (non-union) type matches a typeof string.
    /// Mirrors the flag mappings in `narrow_by_typeof`.
    fn constituent_matches_typeof(&self, t: &Arc<Type>, type_name: &str) -> bool {
        match type_name {
            "string" => t.flags.intersects(TYPE_FLAGS_STRING_LIKE),
            "number" => t.flags.intersects(TYPE_FLAGS_NUMBER_LIKE),
            "boolean" => t.flags.intersects(TYPE_FLAGS_BOOLEAN_LIKE),
            "bigint" => t.flags.intersects(TYPE_FLAGS_BIG_INT_LIKE),
            "symbol" => t.flags.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE),
            "undefined" => t.flags.contains(TypeFlags::Undefined),
            "function" => !self
                .get_signatures_of_type(t, SignatureKind::Call)
                .is_empty(),
            "object" => t.flags.contains(TypeFlags::Object) || t.flags.contains(TypeFlags::Null),
            _ => false,
        }
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
        target: &FlowRef,
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

        // Case 1: discriminant is the narrowed target → `switch (x) { ... }` /
        // `switch (obj.val) { ... }`
        if self.expr_matches_target(discriminant, target) {
            return self.narrow_by_switch_on_discriminant(type_, clause, switch_stmt);
        }

        // Case 2: discriminant is a property access on the symbol —
        // directly (`switch (obj.kind)`) or through a const alias
        // (`const k = obj.kind; switch (k)`).
        if let FlowRef::Symbol(symbol) = target {
        if let Some(access) = self
            .discriminant_alias_access(discriminant, symbol)
            .or_else(|| {
                self.is_property_access_on_symbol(discriminant, symbol)
                    .then(|| Arc::clone(discriminant))
            })
        {
            return self.narrow_by_switch_on_discriminant_property(
                type_,
                clause,
                switch_stmt,
                &access,
            );
        }
        }

        // Case 3: discriminant is `typeof x` where `x` is the target →
        // `switch (typeof x) { case "string": ... }`
        if discriminant.kind == SyntaxKind::TypeOfExpression {
            if let NodeData::TypeOfExpression(typeof_data) = &discriminant.data {
                if self.expr_matches_target(&typeof_data.expression, target) {
                    return self.narrow_by_switch_on_typeof(type_, clause, switch_stmt);
                }
            }
        }

        // Case 4: discriminant is `true` → `switch (true) { case cond: ... }`.
        // Each case clause's expression is a boolean condition that narrows
        // the symbol. Mirrors Go's `narrowTypeBySwitchOnTrue` (flow.go ~L1166).
        if discriminant.kind == SyntaxKind::TrueKeyword {
            return self.narrow_by_switch_on_true(type_, clause, switch_stmt, target);
        }

        Arc::clone(type_)
    }

    /// Narrow for `switch (true) { case cond: ... }` where the discriminant
    /// is the literal `true`.
    ///
    /// Mirrors Go's `narrowTypeBySwitchOnTrue` (flow.go ~L1166). Each case
    /// clause's expression is a boolean condition (e.g. `x === "foo"`,
    /// `isString(x)`). The narrowing works as follows:
    ///
    /// - For all case clauses *preceding* the current one, the condition was
    ///   false (otherwise we'd have entered that case). Narrow with the
    ///   false branch.
    /// - For the current `CaseClause`, the condition is true. Narrow with
    ///   the true branch.
    /// - For the `DefaultClause`, all case conditions are false. Narrow with
    ///   the false branch for all cases.
    ///
    /// Fallthrough (multiple cases without `break`) is not yet handled; each
    /// clause is treated independently. This matches the common case where
    /// each case has a `break`.
    fn narrow_by_switch_on_true(
        &mut self,
        type_: &Arc<Type>,
        clause: &Arc<Node>,
        switch_stmt: &Arc<Node>,
        target: &FlowRef,
    ) -> Arc<Type> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Arc::clone(type_);
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return Arc::clone(type_);
        };
        let clauses = &case_block.clauses.nodes;

        // Find the current clause index.
        let current_idx = match clauses.iter().position(|c| Arc::ptr_eq(c, clause)) {
            Some(i) => i,
            None => return Arc::clone(type_),
        };

        let is_default = clause.kind == SyntaxKind::DefaultClause;
        let mut t = Arc::clone(type_);

        // Narrow away all preceding case clauses (they didn't match).
        for i in 0..current_idx {
            if clauses[i].kind == SyntaxKind::CaseClause {
                if let NodeData::CaseOrDefaultClause(cd) = &clauses[i].data {
                    t = self.narrow_by_expression(
                        &t,
                        &cd.expression,
                        target,
                        NarrowKind::FalseBranch,
                        0,
                    );
                }
            }
        }

        if is_default {
            // Default clause: also narrow away all subsequent case clauses.
            for i in (current_idx + 1)..clauses.len() {
                if clauses[i].kind == SyntaxKind::CaseClause {
                    if let NodeData::CaseOrDefaultClause(cd) = &clauses[i].data {
                        t = self.narrow_by_expression(
                            &t,
                            &cd.expression,
                            target,
                            NarrowKind::FalseBranch,
                            0,
                        );
                    }
                }
            }
            return t;
        }

        // CaseClause: narrow with the condition being true.
        if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
            t = self.narrow_by_expression(&t, &cd.expression, target, NarrowKind::TrueBranch, 0);
        }
        t
    }

    /// Narrow for `switch (typeof x) { case "string": ... }` where `typeof x`
    /// is the discriminant and `x` is the symbol.
    ///
    /// Mirrors Go's `narrowTypeBySwitchOnTypeOf` (flow.go ~L1136). For a
    /// `CaseClause`, narrows `x` to the type implied by the typeof string
    /// (e.g. `case "string":` → `string`). For a `DefaultClause`, narrows
    /// `x` to exclude the types covered by all cases.
    ///
    /// The mapping from typeof string to type mirrors Go's
    /// `narrowTypeByTypeName` (flow.go ~L645):
    /// - `"string"` → `string`
    /// - `"number"` → `number`
    /// - `"bigint"` → `bigint`
    /// - `"boolean"` → `boolean`
    /// - `"symbol"` → `symbol`
    /// - `"undefined"` → `undefined`
    /// - `"object"` → `object | null` (typeof null === "object")
    /// - `"function"` → `Function`
    /// - other/unknown → `object` (host object)
    fn narrow_by_switch_on_typeof(
        &mut self,
        type_: &Arc<Type>,
        clause: &Arc<Node>,
        switch_stmt: &Arc<Node>,
    ) -> Arc<Type> {
        let witnesses = self.get_switch_clause_typeof_witnesses(switch_stmt);
        let Some(witnesses) = witnesses else {
            return Arc::clone(type_);
        };
        // Pre-compute the implied types for all witnesses (avoids mutable
        // borrows inside closures).
        let implied_types: Vec<Arc<Type>> = witnesses
            .iter()
            .map(|w| self.typeof_string_to_type(w))
            .collect();
        if clause.kind == SyntaxKind::DefaultClause {
            // Default clause: keep constituents that don't match any case's
            // typeof string.
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !implied_types.iter().any(|it| self.types_overlap(t, it)))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
        // CaseClause: narrow to the type implied by the case's typeof string.
        let NodeData::CaseOrDefaultClause(clause_data) = &clause.data else {
            return Arc::clone(type_);
        };
        let case_text = self.literal_text_of(&clause_data.expression);
        let Some(case_text) = case_text else {
            return Arc::clone(type_);
        };
        let implied = self.typeof_string_to_type(&case_text);
        // Intersect: keep the part of `type_` that is assignable to `implied`.
        // For unions, filter to constituents that overlap `implied`.
        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let matching: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| {
                    // `case "function"` keeps only CALLABLE constituents (a
                    // plain `object` constituent is excluded — `typeof {} ===
                    // "object"`); other names keep overlapping constituents.
                    if case_text == "function" {
                        return self.types_overlap(t, &implied)
                            && !self
                                .get_signatures_of_type(t, SignatureKind::Call)
                                .is_empty();
                    }
                    self.types_overlap(t, &implied)
                })
                .collect();
            return self.rebuild_union_or_never(type_, matching);
        }
        // Non-union: if the type overlaps the implied type, narrow to the
        // implied type.
        if self.types_overlap(type_, &implied) {
            if self.is_type_assignable_to(type_, &implied) {
                return Arc::clone(type_);
            }
            return implied;
        }
        // No overlap → never (this case is unreachable for this symbol).
        self.never_type()
    }

    /// Get the typeof string witnesses for each case clause in a switch.
    ///
    /// Mirrors Go's `getSwitchClauseTypeOfWitnesses` (flow.go ~L1968).
    /// Returns `None` if any case clause's expression is not a string
    /// literal (in which case typeof narrowing doesn't apply).
    fn get_switch_clause_typeof_witnesses(
        &mut self,
        switch_stmt: &Arc<Node>,
    ) -> Option<Vec<String>> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return None;
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return None;
        };
        let mut witnesses = Vec::with_capacity(case_block.clauses.len());
        for clause in &case_block.clauses.nodes {
            if clause.kind == SyntaxKind::CaseClause {
                if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
                    let text = self.literal_text_of(&cd.expression);
                    match text {
                        Some(t) => witnesses.push(t),
                        None => return None, // non-string-literal case
                    }
                } else {
                    witnesses.push(String::new());
                }
            } else {
                // DefaultClause: empty witness marker.
                witnesses.push(String::new());
            }
        }
        Some(witnesses)
    }

    /// Map a typeof result string to the corresponding `Type`.
    ///
    /// Mirrors Go's `narrowTypeByTypeName` (flow.go ~L645). For unknown
    /// strings, falls back to `object` (host object), matching Go's
    /// `TypeFactsTypeofEQHostObject` behavior.
    fn typeof_string_to_type(&mut self, text: &str) -> Arc<Type> {
        match text {
            "string" => self.string_type(),
            "number" => self.number_type(),
            "bigint" => self.bigint_type(),
            "boolean" => self.boolean_type(),
            "symbol" => self.es_symbol_type(),
            "undefined" => self.undefined_type(),
            "object" => {
                // typeof null === "object", so "object" includes null.
                // Also includes non-primitive objects.
                let non_primitive = self.non_primitive_type();
                let null = self.null_type();
                self.get_union_type(vec![non_primitive, null])
            }
            "function" => {
                // Use the global Function type if available, otherwise
                // fall back to any_function_type.
                if let Some(f) = self.any_function_type.get() {
                    Arc::clone(f)
                } else {
                    self.any_type()
                }
            }
            _ => self.non_primitive_type(),
        }
    }

    /// Get the string text of a string-literal-like expression node.
    ///
    /// Mirrors Go's `ast.IsStringLiteralLike` + `Text()` combo. Returns
    /// `None` if the node is not a string literal.
    fn literal_text_of(&self, node: &Arc<Node>) -> Option<String> {
        match node.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(data) = &node.data {
                    Some(data.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                if let NodeData::NoSubstitutionTemplateLiteral(data) = &node.data {
                    Some(data.text.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
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
                .filter(|t| !case_types.iter().any(|ct| self.types_overlap(t, ct)))
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
        let is_default = clause.kind == SyntaxKind::DefaultClause;
        // For `case "foo":`, narrow to constituents whose discriminant
        // property matches the *current* clause's type. For `default:`,
        // keep constituents whose discriminant property doesn't match any
        // case type.
        let current_case_type = if is_default {
            None
        } else if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
            Some(self.get_type_of_node(&cd.expression))
        } else {
            None
        };
        let all_case_types = if is_default {
            self.get_switch_clause_types(switch_stmt)
        } else {
            Vec::new()
        };
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
                    !all_case_types
                        .iter()
                        .any(|ct| self.types_overlap(&prop_type, ct))
                } else {
                    // Case: keep constituents whose property matches the
                    // current clause's case type.
                    current_case_type
                        .as_ref()
                        .map(|ct| self.types_overlap(&prop_type, ct))
                        .unwrap_or(false)
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

    /// Check if `source` is an optional chain (`?.`) whose root expression
    /// resolves to `symbol`.
    ///
    /// Mirrors Go's `optionalChainContainsReference` (flow.go ~L1830).
    /// Walks down the optional chain: `x?.a?.b` → checks if `x` is `symbol`.
    fn optional_chain_contains_target(&self, source: &Arc<Node>, target: &FlowRef) -> bool {
        let symbol = match target {
            FlowRef::Symbol(symbol) => symbol,
            FlowRef::Node(reference) => {
                return self.optional_chain_contains_reference(source, reference)
            }
        };
        self.optional_chain_contains_symbol(source, symbol)
    }

    fn optional_chain_contains_symbol(&self, source: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        let mut current = Arc::clone(source);
        loop {
            let (inner, is_optional) = match &current.data {
                NodeData::PropertyAccessExpression(pa) => {
                    (&pa.expression, pa.question_dot_token.is_some())
                }
                NodeData::ElementAccessExpression(ea) => {
                    (&ea.expression, ea.question_dot_token.is_some())
                }
                NodeData::CallExpression(ce) => (&ce.expression, ce.question_dot_token.is_some()),
                NodeData::NonNullExpression(ne) => (&ne.expression, false),
                NodeData::ParenthesizedExpression(pe) => (&pe.expression, false),
                _ => return false,
            };
            if is_optional && self.is_symbol_identifier(inner, symbol) {
                return true;
            }
            if !is_optional
                && !matches!(
                    &current.data,
                    NodeData::NonNullExpression(_) | NodeData::ParenthesizedExpression(_)
                )
            {
                // Not an optional chain and not a transparent wrapper — stop.
                return false;
            }
            current = Arc::clone(inner);
        }
    }

    /// Node-reference variant: the optional chain's root structurally matches
    /// the reference being narrowed (`obj?.x` contains the reference `obj`).
    fn optional_chain_contains_reference(
        &self,
        source: &Arc<Node>,
        reference: &Arc<Node>,
    ) -> bool {
        let mut current = Arc::clone(source);
        loop {
            let (inner, is_optional) = match &current.data {
                NodeData::PropertyAccessExpression(pa) => {
                    (&pa.expression, pa.question_dot_token.is_some())
                }
                NodeData::ElementAccessExpression(ea) => {
                    (&ea.expression, ea.question_dot_token.is_some())
                }
                NodeData::CallExpression(ce) => (&ce.expression, ce.question_dot_token.is_some()),
                NodeData::NonNullExpression(ne) => (&ne.expression, false),
                NodeData::ParenthesizedExpression(pe) => (&pe.expression, false),
                _ => return false,
            };
            if is_optional && self.is_matching_reference(reference, inner) {
                return true;
            }
            if !is_optional
                && !matches!(
                    &current.data,
                    NodeData::NonNullExpression(_) | NodeData::ParenthesizedExpression(_)
                )
            {
                // Not an optional chain and not a transparent wrapper — stop.
                return false;
            }
            current = Arc::clone(inner);
        }
    }

    /// Narrow by optional chain containment: `x?.a === value`.
    ///
    /// Mirrors Go's `narrowTypeByOptionalChainContainment` (flow.go ~L1019).
    /// When the comparison value excludes null/undefined, removes null and
    /// undefined from `x`'s type in the branch where the comparison holds.
    fn narrow_by_optional_chain_containment(
        &mut self,
        type_: &Arc<Type>,
        op: SyntaxKind,
        value_node: &Arc<Node>,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let is_equality =
            op == SyntaxKind::EqualsEqualsEqualsToken || op == SyntaxKind::EqualsEqualsToken;
        let is_loose =
            op == SyntaxKind::EqualsEqualsToken || op == SyntaxKind::ExclamationEqualsToken;
        // For loose equality (==/!=), nullable = null | undefined.
        // For strict equality (===/!==), nullable = undefined only.
        let nullable_flags = if is_loose {
            TypeFlags::Undefined | TypeFlags::Null
        } else {
            TypeFlags::Undefined
        };
        let value_type = self.get_type_of_node(value_node);
        // If the value type excludes null/undefined (i.e. none of its
        // constituents have the nullable flags), remove nullable from `x`
        // in the branch where the comparison holds.
        // If the value type IS null/undefined, remove nullable in the
        // opposite branch.
        let value_is_nullable = self.type_contains_flags(&value_type, nullable_flags);
        let value_excludes_nullable = !value_is_nullable;
        let remove_nullable = if is_equality {
            // `x?.a === value`: remove nullable if value excludes it (true
            // branch), or if value IS nullable (false branch).
            (kind == NarrowKind::TrueBranch && value_excludes_nullable)
                || (kind == NarrowKind::FalseBranch && value_is_nullable)
        } else {
            // `x?.a !== value`: remove nullable if value excludes it (false
            // branch), or if value IS nullable (true branch).
            (kind == NarrowKind::FalseBranch && value_excludes_nullable)
                || (kind == NarrowKind::TrueBranch && value_is_nullable)
        };
        if remove_nullable {
            self.remove_nullable_from_union(type_)
        } else {
            Arc::clone(type_)
        }
    }

    /// Remove `null` and `undefined` from a union type.
    fn remove_nullable_from_union(&self, type_: &Arc<Type>) -> Arc<Type> {
        self.remove_flags_from_union(type_, TypeFlags::Undefined | TypeFlags::Null)
    }

    /// Check if `type_` or any of its union constituents has any of `flags`.
    fn type_contains_flags(&self, type_: &Arc<Type>, flags: TypeFlags) -> bool {
        if type_.flags.intersects(flags) {
            return true;
        }
        if type_.is_union() {
            if let TypeData::Union(u) = &type_.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|t| t.flags.intersects(flags));
            }
        }
        false
    }

    /// Narrow based on a call expression with a type predicate.
    ///
    /// Mirrors Go's `narrowTypeByCallExpression` (flow.go ~L444). When the
    /// called function has a type predicate (`x is T`), narrows the argument
    /// matching the predicate's parameter to the predicate type.
    fn narrow_by_call_expression(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let NodeData::CallExpression(call) = &expr.data else {
            return Arc::clone(type_);
        };
        let callee_type = self.get_type_of_node(&call.expression);
        let signatures = self.get_signatures_of_type(&callee_type, SignatureKind::Call);
        let assume_true = kind == NarrowKind::TrueBranch;
        for sig in &signatures {
            let Some(predicate) = self.compute_type_predicate_of_signature(sig) else {
                continue;
            };
            // Only `x is T` and `asserts x is T` predicates narrow arguments.
            if predicate.kind != TypePredicateKind::Identifier
                && predicate.kind != TypePredicateKind::AssertsIdentifier
            {
                // `this is T` predicates (Array#every/some overloads like
                // `every<S extends T>(cb: (v: T) => v is S): this is S[]`)
                // narrow the method RECEIVER when the callback argument
                // carries its own `value is U` predicate (Go
                // narrowTypeByCallExpression's TypePredicateKindThis arm via
                // the effects signature instantiated at the call site).
                if predicate.kind == TypePredicateKind::This
                    && let Some(pred_type) = &predicate.t
                {
                    let receiver = match &call.expression.data {
                        NodeData::PropertyAccessExpression(pa) => Some(&pa.expression),
                        _ => None,
                    };
                    let Some(receiver) = receiver else {
                        continue;
                    };
                    if !self.expr_matches_target(receiver, target) {
                        continue;
                    }
                    let Some(callback_arg) = call.arguments.nodes.first() else {
                        continue;
                    };
                    let Some(u) = self.callback_predicate_type(callback_arg) else {
                        continue;
                    };
                    // Instantiate the declared predicate `S[]` with the
                    // callback's predicate type before narrowing.
                    let instantiated = if sig.type_parameters.is_empty() {
                        Arc::clone(pred_type)
                    } else {
                        let args: Vec<Arc<Type>> = sig
                            .type_parameters
                            .iter()
                            .map(|_| Arc::clone(&u))
                            .collect();
                        self.substitute_infer_type_parameters(
                            pred_type,
                            &sig.type_parameters,
                            &args,
                        )
                    };
                    return self.narrow_by_type_predicate(type_, &instantiated, assume_true);
                }
                continue;
            }
            let Some(pred_type) = &predicate.t else {
                continue;
            };
            let param_idx = predicate.parameter_index as usize;
            let Some(arg) = call.arguments.nodes.get(param_idx) else {
                continue;
            };
            // The argument must be the target being narrowed.
            if !self.expr_matches_target(arg, target) {
                continue;
            }
            return self.narrow_by_type_predicate(type_, pred_type, assume_true);
        }
        Arc::clone(type_)
    }

    /// The `U` of a callback argument's own `value is U` signature — used by
    /// `this is S[]` receiver narrowing (`arr.every(x is U)` narrows `arr`
    /// to `U[]`).
    fn callback_predicate_type(&mut self, arg: &Arc<Node>) -> Option<Arc<Type>> {
        let arg_type = self.get_type_of_node(arg);
        let sigs = self.get_signatures_of_type(&arg_type, SignatureKind::Call);
        for sig in &sigs {
            if let Some(pred) = self.compute_type_predicate_of_signature(sig)
                && pred.kind == TypePredicateKind::Identifier
                && let Some(t) = pred.t
            {
                return Some(t);
            }
        }
        None
    }

    /// Narrow `type_` after an assertion function call.
    ///
    /// Mirrors Go's `getTypeAtFlowCall` (flow.go ~L288). If `call_expr`
    /// calls an assertion function (`asserts x` or `asserts x is T`), the
    /// argument corresponding to `target` is narrowed:
    ///
    /// - `asserts x is T` → narrow to `T` (intersect with the predicate type).
    /// - `asserts x` (no type) → narrow to truthy (remove `null`/`undefined`).
    ///
    /// Non-assertion calls leave `type_` unchanged.
    fn narrow_by_assertion_call(
        &mut self,
        type_: &Arc<Type>,
        call_expr: &Arc<Node>,
        target: &FlowRef,
    ) -> Arc<Type> {
        let NodeData::CallExpression(call) = &call_expr.data else {
            return Arc::clone(type_);
        };
        let callee_type = self.get_type_of_node(&call.expression);
        let signatures = self.get_signatures_of_type(&callee_type, SignatureKind::Call);
        for sig in &signatures {
            let Some(predicate) = self.compute_type_predicate_of_signature(sig) else {
                continue;
            };
            // Only assertion functions narrow after the call.
            if predicate.kind != TypePredicateKind::AssertsIdentifier
                && predicate.kind != TypePredicateKind::AssertsThis
            {
                continue;
            }
            // For `asserts this` / `asserts this is T`, the asserted value
            // is the receiver (the `this` of the method call), not an
            // argument. We don't track `this` narrowing yet.
            if predicate.kind == TypePredicateKind::AssertsThis {
                continue;
            }
            let param_idx = predicate.parameter_index as usize;
            let Some(arg) = call.arguments.nodes.get(param_idx) else {
                continue;
            };
            // The argument must be the target being narrowed.
            if !self.expr_matches_target(arg, target) {
                continue;
            }
            if let Some(pred_type) = &predicate.t {
                // `asserts x is T` → narrow to T.
                return self.intersect_or_narrow(type_, pred_type);
            }
            // Plain `asserts x` → narrow to truthy (remove null/undefined).
            return self.remove_flags_from_union(type_, TYPE_FLAGS_NULLABLE);
        }
        Arc::clone(type_)
    }

    /// Narrow `type_` to (or away from) the predicate type.
    ///
    /// In the true branch, intersect with the predicate type (e.g. `x is
    /// string` → narrow to `string`). In the false branch, remove the
    /// predicate type from the union.
    fn narrow_by_type_predicate(
        &mut self,
        type_: &Arc<Type>,
        pred_type: &Arc<Type>,
        assume_true: bool,
    ) -> Arc<Type> {
        if assume_true {
            self.intersect_or_narrow(type_, pred_type)
        } else {
            self.remove_type_from_union(type_, pred_type)
        }
    }

    /// Check if `expr` is `typeof <target>`.
    fn typeof_expr_matches_target(&self, expr: &Arc<Node>, target: &FlowRef) -> bool {
        let NodeData::TypeOfExpression(typeof_data) = &expr.data else {
            return false;
        };
        self.expr_matches_target(&typeof_data.expression, target)
    }

    /// Narrow by `typeof x === "typename"`.
    ///
    /// `narrow_to_value` = true means the typeof check passed (e.g.
    /// `typeof x === "string"` is true), so we narrow to the matching type.
    fn narrow_by_typeof(
        &mut self,
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
            "function" => {
                // `typeof x === "function"` narrows to callable types (types
                // with call signatures), not all object types. Mirrors Go's
                // `narrowTypeByTypeName` "function" case (flow.go ~L662)
                // which narrows to `globalFunctionType` via
                // `TypeFactsTypeofEQFunction`.
                return self.filter_type_by_callable(type_, narrow_to_value);
            }
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

    /// Narrow by optionality: used for `??` (nullish coalescing) narrowing.
    /// Mirrors Go's `narrowTypeByOptionality` (flow.go ~L415).
    ///
    /// - True branch (assume present): remove null and undefined.
    /// - False branch (assume absent): keep only null and undefined.
    fn narrow_by_optionality(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
        _depth: u32,
    ) -> Arc<Type> {
        // If the expression is a direct reference to our target, apply
        // optionality narrowing (remove/keep null/undefined).
        if self.expr_matches_target(expr, target) {
            return match kind {
                NarrowKind::TrueBranch => self.remove_nullable_from_union(type_),
                NarrowKind::FalseBranch => {
                    self.filter_type_by_flags(type_, TypeFlags::Undefined | TypeFlags::Null)
                }
            };
        }
        // Const alias inlining: if expr is a const variable alias of the
        // symbol, recurse through the initializer.
        if expr.kind == SyntaxKind::Identifier && self.flow_inline_level < 5 {
            if let Some(init_expr) = self.const_alias_initializer(expr) {
                self.flow_inline_level += 1;
                let result = self.narrow_by_optionality(type_, &init_expr, target, kind, _depth);
                self.flow_inline_level -= 1;
                return result;
            }
        }
        // Expression doesn't reference our target; no narrowing.
        Arc::clone(type_)
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
    pub fn remove_flags_from_union(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
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

    /// Filter `type_` to keep only callable constituents (types with call
    /// signatures) when `keep_callable` is true, or only non-callable
    /// constituents when false. Used for `typeof x === "function"` and
    /// `typeof x !== "function"` narrowing.
    ///
    /// Mirrors Go's `TypeFactsTypeofEQFunction` / `TypeFactsTypeofNEFunction`
    /// handling in `narrowTypeByTypeFacts` (flow.go ~L673). A type is
    /// callable if it has call signatures (i.e., `structured.call_signatures()`
    /// is non-empty).
    fn filter_type_by_callable(&self, type_: &Arc<Type>, keep_callable: bool) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let is_callable = !self
                    .get_signatures_of_type(t, SignatureKind::Call)
                    .is_empty();
                if keep_callable {
                    is_callable
                } else {
                    !is_callable
                }
            })
            .collect();
        if filtered.is_empty() {
            return self.never_type();
        }
        if filtered.len() == 1 {
            return filtered.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: filtered,
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
            .filter(|t| !t.flags.contains(TypeFlags::Object) && !t.flags.contains(TypeFlags::Null))
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
        // If either is a union/intersection, compare constituents pairwise.
        if a.flags.contains(TypeFlags::Union)
            || b.flags.contains(TypeFlags::Union)
            || a.flags.contains(TypeFlags::Intersection)
            || b.flags.contains(TypeFlags::Intersection)
        {
            let a_types = self.constituent_types(a);
            let b_types = self.constituent_types(b);
            for at in &a_types {
                for bt in &b_types {
                    if self.literals_overlap(at, bt) {
                        return true;
                    }
                }
            }
            return false;
        }
        self.literals_overlap(a, b)
    }

    /// Check if two non-union types overlap, with literal-aware comparison.
    ///
    /// `string` and `"foo"` overlap; `"foo"` and `"bar"` do not; `"foo"` and
    /// `"foo"` do.
    fn literals_overlap(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        // If either is a literal type, we must compare literal values
        // (not just flags), because two string-literal types share the
        // `StringLiteral` flag but are distinct types when their values
        // differ.
        let a_is_literal = a.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral,
        );
        let b_is_literal = b.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral,
        );
        if a_is_literal && b_is_literal {
            // Both literals: compare values directly.
            return match (&a.data, &b.data) {
                (TypeData::Literal(a_lit), TypeData::Literal(b_lit)) => a_lit.value == b_lit.value,
                _ => false,
            };
        }
        if a_is_literal {
            // `a` is literal, `b` is not: overlap if `b` is the literal's
            // primitive base (e.g. `"foo"` overlaps with `string`).
            return a.flags.intersects(b.flags);
        }
        if b_is_literal {
            return a.flags.intersects(b.flags);
        }
        // Neither is a literal: fall back to flag intersection.
        a.flags.intersects(b.flags)
    }

    // ─────────────────────────────────────────────────────────────────
    // Symbol/expression matching helpers
    // ─────────────────────────────────────────────────────────────────

    /// Check if `node` is an identifier that resolves to `symbol`.
    /// Uses the symbol_map for a direct lookup (avoids mutating scope state).
    fn is_symbol_identifier(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        // A flow-assignment's node may be the declaration itself (a
        // VariableDeclaration or BindingElement — Go's isMatchingReference
        // matches an identifier reference against the declaration's symbol).
        if matches!(
            node.kind,
            SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
        ) {
            return self
                .program
                .symbol_map()
                .symbol_of(node)
                .is_some_and(|s| Arc::ptr_eq(s, symbol));
        }
        if node.kind != SyntaxKind::Identifier {
            return false;
        }
        // Try the symbol_map first (set by the binder on declaration nodes).
        // For reference nodes, the symbol may not be set, so we fall back
        // to name-based matching.
        let symbol_map = self.program.symbol_map();
        if let Some(sym) = symbol_map.symbol_of(node) {
            let eq = Arc::ptr_eq(sym, symbol);
            return eq;
        }
        // Fallback: compare by name. The identifier's text must match the
        // symbol's name (set by the binder when the declaration was bound).
        let node_name = match &node.data {
            NodeData::Identifier(data) => &data.text,
            _ => return false,
        };
        let eq = node_name == &symbol.name;
        eq
    }

    /// Whether `node` is the expression the flow query narrows. Identifier
    /// targets match by symbol; property-access targets match structurally.
    fn expr_matches_target(&self, node: &Arc<Node>, target: &FlowRef) -> bool {
        match target {
            FlowRef::Symbol(symbol) => self.is_symbol_identifier(node, symbol),
            FlowRef::Node(reference) => self.is_matching_reference(reference, node),
        }
    }

    /// Structurally compare two reference expressions for flow narrowing.
    ///
    /// Mirrors Go's `isMatchingReference` (flow.go ~L1597): unwraps
    /// parentheses/non-null/assignment/comma wrappers on the flow side,
    /// matches identifiers by resolved symbol, `this`/`super` by kind, and
    /// property/element accesses by accessed property name plus recursively
    /// matching receivers.
    fn is_matching_reference(&self, source: &Arc<Node>, target: &Arc<Node>) -> bool {
        match target.kind {
            SyntaxKind::ParenthesizedExpression | SyntaxKind::NonNullExpression => {
                let inner = Self::skip_parentheses(target);
                return self.is_matching_reference(source, &inner);
            }
            SyntaxKind::BinaryExpression => {
                if let NodeData::BinaryExpression(bin) = &target.data {
                    if is_assignment_operator(bin.operator_token.kind)
                        && self.is_matching_reference(source, &bin.left)
                    {
                        return true;
                    }
                    if bin.operator_token.kind == SyntaxKind::CommaToken
                        && self.is_matching_reference(source, &bin.right)
                    {
                        return true;
                    }
                }
                return false;
            }
            _ => {}
        }
        match source.kind {
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier => {
                if target.kind == SyntaxKind::Identifier {
                    return match (
                        self.resolve_identifier(source),
                        self.resolve_identifier(target),
                    ) {
                        (Some(s), Some(t)) => Arc::ptr_eq(&s, &t),
                        _ => false,
                    };
                }
                // A flow-assignment's node may be the declaration itself
                // (Go isMatchingReference): an identifier matches a
                // VariableDeclaration/BindingElement whose declared symbol it
                // resolves to (export-symbol unwrapped).
                if matches!(
                    target.kind,
                    SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
                ) {
                    let Some(source_sym) = self.resolve_identifier(source) else {
                        return false;
                    };
                    let Some(target_sym) =
                        self.program.symbol_map().symbol_of(target).cloned()
                    else {
                        return false;
                    };
                    // Unwrap `export symbol` one hop (Go
                    // getExportSymbolOfValueSymbolIfExported).
                    let source_unwrapped = source_sym
                        .export_symbol
                        .clone()
                        .unwrap_or_else(|| Arc::clone(&source_sym));
                    let target_unwrapped = target_sym
                        .export_symbol
                        .clone()
                        .unwrap_or(target_sym);
                    return Arc::ptr_eq(&source_unwrapped, &target_unwrapped);
                }
                false
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => target.kind == source.kind,
            SyntaxKind::NonNullExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::SatisfiesExpression => {
                if let Some(inner) = source.expression() {
                    self.is_matching_reference(&inner, target)
                } else {
                    false
                }
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(source_prop_name) = self.get_accessed_property_name(source) {
                    if matches!(
                        target.kind,
                        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                    ) {
                        if let Some(target_prop_name) = self.get_accessed_property_name(target) {
                            if target_prop_name == source_prop_name {
                                let source_receiver = source.expression();
                                let target_receiver = target.expression();
                                if let (Some(s), Some(t)) = (source_receiver, target_receiver) {
                                    return self.is_matching_reference(&s, &t);
                                }
                            }
                        }
                    }
                }
                // `a[i]` matches `a[j]` when `i` and `j` resolve to the same
                // constant (or never-reassigned parameter/local) variable.
                if source.kind == SyntaxKind::ElementAccessExpression
                    && target.kind == SyntaxKind::ElementAccessExpression
                {
                    let (NodeData::ElementAccessExpression(source_ea),
                         NodeData::ElementAccessExpression(target_ea)) =
                        (&source.data, &target.data)
                    else {
                        return false;
                    };
                    if source_ea.argument_expression.kind == SyntaxKind::Identifier
                        && target_ea.argument_expression.kind == SyntaxKind::Identifier
                    {
                        let matching_args = match (
                            self.resolve_identifier(&source_ea.argument_expression),
                            self.resolve_identifier(&target_ea.argument_expression),
                        ) {
                            (Some(s), Some(t)) if Arc::ptr_eq(&s, &t) => {
                                self.symbol_is_const_variable(&s)
                                    || (self.is_parameter_or_mutable_local(&s)
                                        && !self.symbol_is_assigned(&s))
                            }
                            _ => false,
                        };
                        if matching_args {
                            let (Some(s), Some(t)) = (
                                source.expression(),
                                target.expression(),
                            ) else {
                                return false;
                            };
                            return self.is_matching_reference(&s, &t);
                        }
                    }
                }
                false
            }
            SyntaxKind::QualifiedName => {
                if let NodeData::QualifiedName(qualified) = &source.data {
                    if matches!(
                        target.kind,
                        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                    ) {
                        if let Some(target_prop_name) = self.get_accessed_property_name(target) {
                            if qualified.right.text() == target_prop_name {
                                if let Some(t) = target.expression() {
                                    return self.is_matching_reference(&qualified.left, &t);
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Whether any left-hand part of the dotted `source` reference matches
    /// `target` (`obj.val` contains `obj`). Mirrors Go's
    /// `containsMatchingReference` (flow.go ~L1841).
    fn contains_matching_reference(&self, source: &Arc<Node>, target: &Arc<Node>) -> bool {
        let mut source = Arc::clone(source);
        while matches!(
            source.kind,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
        ) {
            let Some(inner) = source.expression() else {
                break;
            };
            if self.is_matching_reference(inner, target) {
                return true;
            }
            source = Arc::clone(inner);
        }
        false
    }

    /// The accessed property name of a property/element access, used for
    /// structural reference matching. Mirrors Go's `getAccessedPropertyName`
    /// (flow.go ~L1727): `a.b` → `b`, `a["b"]` → `b`, `a[0]` → `0`.
    /// Returns `None` for non-literal element-access arguments.
    fn get_accessed_property_name(&self, access: &Arc<Node>) -> Option<String> {
        match &access.data {
            NodeData::PropertyAccessExpression(pa) => Some(pa.name.text().to_string()),
            NodeData::ElementAccessExpression(ea) => {
                match &ea.argument_expression.data {
                    NodeData::StringLiteral(s) => Some(s.text.clone()),
                    NodeData::NumericLiteral(n) => Some(n.text.clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Whether `symbol` is a function parameter or a block-scoped local
    /// (not a property, export, etc.). Mirrors the
    /// `isParameterOrMutableLocalVariable` half of Go's element-access
    /// argument matching (flow.go ~L1634).
    fn is_parameter_or_mutable_local(&self, symbol: &Arc<Symbol>) -> bool {
        symbol
            .flags
            .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
    }

    /// Whether `symbol` is assigned anywhere in its enclosing function or
    /// source file. Mirrors Go's `isSymbolAssigned` (flow.go ~L2661, backed
    /// by `markNodeAssignments`). This port conservatively scans assignment
    /// targets by identifier name in the enclosing container: over-reporting
    /// assignments only removes narrowing opportunities, never adds wrong
    /// narrowing.
    fn symbol_is_assigned(&self, symbol: &Arc<Symbol>) -> bool {
        let Some(decl) = symbol.value_declaration.as_ref() else {
            return true;
        };
        let Some(container) = Self::enclosing_function_or_source_file(decl) else {
            return true;
        };
        let mut assigned = false;
        Self::scan_assignment_targets(&container, &symbol.name, &mut assigned);
        assigned
    }

    /// Nearest function-like or source-file ancestor of `node`.
    pub(super) fn enclosing_function_or_source_file(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(node);
        loop {
            if matches!(
                current.kind,
                SyntaxKind::SourceFile
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::Constructor
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                return Some(current);
            }
            current = Arc::clone(current.parent.as_ref()?);
        }
    }

    /// Recursively mark identifiers in assignment-target positions whose
    /// name matches `name`.
    fn scan_assignment_targets(node: &Arc<Node>, name: &str, assigned: &mut bool) {
        if *assigned {
            return;
        }
        match &node.data {
            NodeData::BinaryExpression(bin) => {
                if is_assignment_operator(bin.operator_token.kind)
                    && bin.left.kind == SyntaxKind::Identifier
                    && bin.left.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && unary.operand.kind == SyntaxKind::Identifier
                    && unary.operand.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            NodeData::PostfixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && unary.operand.kind == SyntaxKind::Identifier
                    && unary.operand.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            _ => {}
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            Self::scan_assignment_targets(child, name, assigned);
            *assigned
        });
    }

    /// If `expr` is an Identifier referring to a `const` variable with a
    /// simple initializer (no type annotation), return the initializer
    /// expression (with parentheses unwrapped). Mirrors Go's
    /// `getCandidateVariableDeclarationInitializer` (flow.go ~L1475)
    /// combined with the `isConstantVariable` check in `narrowType`
    /// (flow.go ~L388).
    fn const_alias_initializer(&self, expr: &Arc<Node>) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }
        // Resolve the identifier reference to its symbol. The symbol_map
        // only stores symbols for declaration nodes, so we use
        // `resolve_identifier` which walks the scope stack (available
        // during narrowing, since narrowing happens while checking
        // expressions in the current scope context).
        let sym = self.resolve_identifier(expr)?;
        if !self.symbol_is_const_variable(&sym) {
            return None;
        }
        let decl = sym.value_declaration.as_ref()?;
        if decl.kind != SyntaxKind::VariableDeclaration {
            return None;
        }
        let NodeData::VariableDeclaration(var_data) = &decl.data else {
            return None;
        };
        // Go requires `declaration.Type() == nil` — a const variable with
        // an explicit type annotation is not inlined.
        if var_data.type_node.is_some() {
            return None;
        }
        let init = var_data.initializer.as_ref()?;
        Some(Self::skip_parentheses(init))
    }

    /// Check if a symbol represents a `const` variable declaration.
    /// Mirrors Go's `isConstantVariable`. The `const`/`let` keyword is
    /// carried on the parent `VariableDeclarationList`'s `NodeFlags`.
    pub(super) fn symbol_is_const_variable(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(parent) = &decl.parent {
                if parent.kind == SyntaxKind::VariableDeclarationList
                    && parent.flags.contains(NodeFlags::Const)
                {
                    return true;
                }
            }
        }
        false
    }

    /// Unwrap parenthesized expressions: `((x))` → `x`.
    fn skip_parentheses(node: &Arc<Node>) -> Arc<Node> {
        let mut current = Arc::clone(node);
        loop {
            if let NodeData::ParenthesizedExpression(p) = &current.data {
                current = Arc::clone(&p.expression);
                continue;
            }
            return current;
        }
    }

    /// Evolve an evolving array type at an ARRAY_MUTATION flow node.
    ///
    /// Mirrors Go's `getTypeAtFlowArrayMutation` (flow.go ~L1383). If the
    /// mutated array (`arr` in `arr.push(1)`) is the same reference as
    /// `symbol`, evolve `pre_type` (the type at the mutation's antecedent)
    /// by adding each argument's (widened) type to the element type.
    /// Returns `None` if the mutation doesn't target our symbol.
    fn evolve_array_at_mutation(
        &mut self,
        node: &Arc<Node>,
        pre_type: &Arc<Type>,
        target: &FlowRef,
    ) -> Option<Arc<Type>> {
        // Extract the mutated array reference: `arr.push(1)` → `arr`.
        let receiver = self.get_array_mutation_receiver(node)?;
        if !self.expr_matches_target(&receiver, target) {
            return None;
        }
        // If the pre-mutation type is the auto-array marker, convert it
        // to an evolving array with element `never`.
        let evolving = if pre_type.object_flags.contains(ObjectFlags::EvolvingArray) {
            Arc::clone(pre_type)
        } else if self.is_auto_array_type(pre_type) {
            self.get_evolving_array_type(self.never_type())
        } else {
            // Not an evolving array; nothing to evolve.
            return Some(Arc::clone(pre_type));
        };
        // Collect argument nodes first (to release the borrow on self),
        // then resolve each type.
        let args = self.get_call_arguments(node);
        let mut arg_types: Vec<Arc<Type>> = Vec::with_capacity(args.len());
        for arg in &args {
            let t = self.get_type_of_node(arg);
            arg_types.push(self.get_widened_type_of_literal(&t));
        }
        // Evolve: add each argument's (widened) type to the element type.
        // For an element-access assignment `arr[i] = x` (BinaryExpression
        // mutation node), the "argument" is the RHS, applied only when the
        // index is number-like (Go `getTypeAtFlowArrayMutation`,
        // flow.go ~L1420).
        let mut evolved = evolving;
        match &node.data {
            NodeData::BinaryExpression(bin)
                if is_assignment_operator(bin.operator_token.kind) =>
            {
                if let NodeData::ElementAccessExpression(ea) = &bin.left.data {
                    let index_type = self.get_type_of_node(&ea.argument_expression);
                    if index_type
                        .flags
                        .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
                    {
                        let t = self.get_type_of_node(&bin.right);
                        let widened = self.get_widened_type_of_literal(&t);
                        evolved = self.add_evolving_array_element_type(&evolved, widened);
                    }
                }
            }
            _ => {
                for arg_type in arg_types {
                    evolved = self.add_evolving_array_element_type(&evolved, arg_type);
                }
            }
        }
        Some(evolved)
    }

    /// Extract the receiver of an array-mutation call (`arr.push(x)` → `arr`).
    /// The flow node is either a CallExpression (`arr.push(x)`) or a
    /// BinaryExpression (`arr[i] = x`). Mirrors Go's
    /// `getTypeAtFlowArrayMutation` node extraction.
    fn get_array_mutation_receiver(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            NodeData::CallExpression(call) => {
                // `arr.push(x)` → call.expression is PropertyAccessExpression.
                if let NodeData::PropertyAccessExpression(prop) = &call.expression.data {
                    return Some(Arc::clone(&prop.expression));
                }
                None
            }
            NodeData::BinaryExpression(bin) => {
                // `arr[i] = x` → bin.left is ElementAccessExpression.
                if let NodeData::ElementAccessExpression(ea) = &bin.left.data {
                    return Some(Arc::clone(&ea.expression));
                }
                None
            }
            _ => None,
        }
    }

    /// Get the arguments of a call expression node.
    fn get_call_arguments(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        match &node.data {
            NodeData::CallExpression(call) => call.arguments.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }

    /// The flow type after a matching assignment. Mirrors Go's
    /// `getTypeAtFlowAssignment` (flow.go ~L220): evolving arrays keep the
    /// assigned type (the ARRAY_MUTATION machinery finalizes them); a union
    /// declared type is reduced by the assigned type
    /// (`getAssignmentReducedType`); any other declared type is returned
    /// unchanged. Returns `None` when the assignment doesn't match the
    /// target, or is compound (the antecedent walk then applies).
    /// Whether a BindingElement sits inside a `var`-declared destructuring
    /// pattern (BindingElement → BindingPattern → VariableDeclaration →
    /// VariableDeclarationList without let/const flags).
    fn binding_element_in_var_pattern(element: &Arc<Node>) -> bool {
        let pattern = element.parent.as_ref();
        let Some(decl) = pattern.and_then(|p| p.parent.as_ref()) else {
            return false;
        };
        if decl.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let Some(list) = decl.parent.as_ref() else {
            return false;
        };
        if list.kind != SyntaxKind::VariableDeclarationList {
            return false;
        }
        // `var` lists carry neither the Let nor the Const flag.
        !(list
            .flags
            .intersects(crate::ast::node_flags::NodeFlags::Let)
            || list
                .flags
                .intersects(crate::ast::node_flags::NodeFlags::Const))
    }

    fn assignment_flow_type(
        &mut self,
        expr: &Arc<Node>,
        target: &FlowRef,
        declared: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let evolving = declared.object_flags.contains(ObjectFlags::EvolvingArray)
            || self.is_auto_array_type(declared);
        match &expr.data {
            // `x = value` — reduce the declared union by the RHS type.
            NodeData::BinaryExpression(bin) => {
                if !is_assignment_operator(bin.operator_token.kind) {
                    return None;
                }
                if !self.expr_matches_target(&bin.left, target) {
                    return None;
                }
                // Compound assignments (`x += v`) recompute from the
                // antecedent type in Go (getBaseTypeOfLiteralType); the
                // caller's antecedent walk approximates that.
                if bin.operator_token.kind != SyntaxKind::EqualsToken {
                    return None;
                }
                let assigned = self.get_type_of_node(&bin.right);
                Some(self.reduced_assignment_type(declared, &assigned, evolving))
            }
            // `x++`, `x--` — numeric after the update.
            NodeData::PostfixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
                {
                    Some(self.number_type())
                } else {
                    None
                }
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
                {
                    Some(self.number_type())
                } else {
                    None
                }
            }
            // Declaration flow nodes (from `bindInitializedVariableFlow`):
            // `let x = init`, destructured `const {a} = obj`, for-in/for-of
            // loop variables.
            NodeData::VariableDeclaration(_) | NodeData::BindingElement(_) => {
                let FlowRef::Symbol(symbol) = target else {
                    return None;
                };
                let element_symbol = self.program.symbol_map().symbol_of(expr).cloned();
                let matched = match &element_symbol {
                    Some(s) => {
                        Arc::ptr_eq(s, symbol)
                            || symbol
                                .export_symbol
                                .as_ref()
                                .is_some_and(|e| Arc::ptr_eq(s, e))
                    }
                    // Bare destructuring patterns (`({ a: a = 1 } = o)`,
                    // `for ({ b } of xs)`) declare no element symbol: the
                    // element's name identifier references the existing
                    // variable.
                    None => match &expr.data {
                        NodeData::BindingElement(be) => be
                            .name
                            .as_ref()
                            .and_then(|name| self.resolve_identifier(name))
                            .is_some_and(|s| Arc::ptr_eq(&s, symbol)),
                        _ => false,
                    },
                } || (
                    // `var`-declared patterns hoist: the pattern element's
                    // symbol and the outer `var` symbol are the same variable
                    // (Go merges them into one symbol; our binder routes the
                    // element to the loop's locals). Fall back to a name
                    // match when the queried symbol is a function-scoped
                    // variable and the element sits in a `var` pattern.
                    element_symbol.as_ref().is_some_and(|s| {
                        s.name == symbol.name
                            && symbol
                                .flags
                                .contains(crate::ast::SymbolFlags::FunctionScopedVariable)
                            && Self::binding_element_in_var_pattern(expr)
                    })
                );
                if !matched {
                    return None;
                }
                let assigned = self.initial_type_of_declaration(expr)?;
                Some(self.reduced_assignment_type(declared, &assigned, evolving))
            }
            // A bare reference assignment target (binder
            // `bindAssignmentTargetFlow` — destructuring assignment
            // patterns and bare for-in/of heads): a match marks the
            // variable assigned; Go's `getTypeAtFlowAssignment` returns the
            // declared type for non-union declared types (the
            // definite-assignment mechanism that clears `undefined`).
            NodeData::Identifier(_) if self.expr_matches_target(expr, target) => {
                Some(Arc::clone(declared))
            }
            _ => None,
        }
    }

    /// Apply Go's assignment narrowing: evolving arrays keep the assigned
    /// type; unions reduce; everything else returns the declared type.
    fn reduced_assignment_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
        evolving: bool,
    ) -> Arc<Type> {
        if evolving {
            return Arc::clone(assigned);
        }
        if !declared.is_union() {
            return Arc::clone(declared);
        }
        self.get_assignment_reduced_type(declared, assigned)
    }

    /// Remove those constituents of `declared` to which no constituent of
    /// `assigned` is assignable (e.g. `number | string` assigned `5` keeps
    /// only `number`). When the crude filter produces an invalid result
    /// (assigned not assignable to the remainder), give up and don't
    /// narrow. Mirrors Go's `getAssignmentReducedType` (flow.go ~L2399).
    fn get_assignment_reduced_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
    ) -> Arc<Type> {
        if Arc::ptr_eq(declared, assigned) {
            return Arc::clone(declared);
        }
        if assigned.flags.contains(TypeFlags::Never) {
            return Arc::clone(assigned);
        }
        let constituents = self.constituent_types(declared);
        let kept: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| self.type_maybe_assignable_to(assigned, t))
            .collect();
        let reduced = self.rebuild_union_or_never(declared, kept);
        if self.is_type_assignable_to(assigned, &reduced) {
            reduced
        } else {
            Arc::clone(declared)
        }
    }

    /// Whether any constituent of `source` is assignable to `target`.
    /// Mirrors Go's `typeMaybeAssignableTo` (flow.go ~L2430).
    fn type_maybe_assignable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if !source.is_union() {
            return self.is_type_assignable_to(source, target);
        }
        let constituents = self.constituent_types(source);
        if constituents.iter().any(|t| Arc::ptr_eq(t, target)) {
            return true;
        }
        constituents
            .iter()
            .any(|t| self.is_type_assignable_to(t, target))
    }

    /// The initial (or assigned) type of a declaration flow node. Mirrors
    /// Go's `getInitialType` family (flow.go ~L2234): a VariableDeclaration
    /// uses its initializer (`string` for for-in, the iterated element type
    /// for for-of); a BindingElement destructures from its parent pattern's
    /// type, applying any default initializer
    /// (`getTypeWithDefault`: non-undefined part ∪ default).
    fn initial_type_of_declaration(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> {
        match &expr.data {
            NodeData::VariableDeclaration(vd) => {
                if let Some(init) = &vd.initializer {
                    return Some(self.get_type_of_node(init));
                }
                let for_stmt = Self::for_in_or_of_statement_of(expr)?;
                let NodeData::ForInOrOfStatement(data) = &for_stmt.data else {
                    return None;
                };
                match for_stmt.kind {
                    SyntaxKind::ForInStatement => Some(self.string_type()),
                    SyntaxKind::ForOfStatement => {
                        let rhs = self.get_type_of_node(&data.expression);
                        Some(self.iterated_element_type(&rhs))
                    }
                    _ => None,
                }
            }
            NodeData::BindingElement(be) => {
                let pattern = Arc::clone(expr.parent.as_ref()?);
                let pattern_parent = Arc::clone(pattern.parent.as_ref()?);
                let parent_type = self.initial_type_of_declaration(&pattern_parent);
                let mut t = match (&parent_type, pattern.kind) {
                    (Some(parent_type), SyntaxKind::ObjectBindingPattern) => {
                        match Self::binding_element_property_name(expr) {
                            Some(name) => self.get_property_type_of_type(parent_type, &name),
                            None => None,
                        }
                    }
                    (
                        Some(parent_type),
                        SyntaxKind::ArrayBindingPattern,
                    ) if be.dot_dot_dot_token.is_none() => {
                        match Self::binding_element_index(&pattern, expr) {
                            Some(index) => {
                                self.destructured_array_element_type(parent_type, index)
                            }
                            None => None,
                        }
                    }
                    _ => None,
                };
                if let Some(default_expr) = &be.initializer {
                    let default_type = self.get_type_of_node(default_expr);
                    // Go's getTypeWithDefault: a missing/undefined base type
                    // falls back entirely to the default (`for (var {x: f = 1}
                    // of [])` — iterating `never` still assigns the default).
                    t = match t {
                        Some(t) => {
                            let non_undefined = self.remove_flags_from_union(&t, TypeFlags::Undefined);
                            Some(self.get_union_type(vec![non_undefined, default_type]))
                        }
                        None => Some(default_type),
                    };
                }
                t
            }
            _ => None,
        }
    }

    /// The property name a binding element destructures
    /// (`{a: b}` → `a`, `{a}` → `a`). Mirrors Go's
    /// `getBindingElementPropertyName`.
    fn binding_element_property_name(element: &Arc<Node>) -> Option<String> {
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        if let Some(pn) = &be.property_name {
            return Some(pn.text().to_string());
        }
        be.name.as_ref().map(|n| n.text().to_string())
    }

    /// The index of `element` within its binding pattern.
    fn binding_element_index(pattern: &Arc<Node>, element: &Arc<Node>) -> Option<usize> {
        let NodeData::BindingPattern(data) = &pattern.data else {
            return None;
        };
        data.elements
            .nodes
            .iter()
            .position(|e| Arc::ptr_eq(e, element))
    }

    /// The element type of a destructured array pattern position: tuples
    /// give the positional element, arrays the element type, evolving
    /// arrays their accumulated element.
    fn destructured_array_element_type(
        &mut self,
        parent_type: &Arc<Type>,
        index: usize,
    ) -> Option<Arc<Type>> {
        if self.is_tuple_type(parent_type) {
            return self.get_tuple_element_type(parent_type, index);
        }
        if self.is_array_type(parent_type) {
            return Some(self.get_array_element_type(parent_type));
        }
        Some(self.get_any_type())
    }

    /// The element type produced by iterating `rhs` in a for-of statement
    /// (`checkRightHandSideOfForOf`): arrays/tuples yield their element
    /// type, strings yield `string`, anything else `any`.
    fn iterated_element_type(&mut self, rhs: &Arc<Type>) -> Arc<Type> {
        if self.is_array_type(rhs) {
            return self.get_array_element_type(rhs);
        }
        if rhs.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral) {
            return self.string_type();
        }
        self.get_any_type()
    }

    /// The for-in/for-of statement whose head declares `decl`, if any.
    fn for_in_or_of_statement_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let list = decl.parent.as_ref()?;
        if list.kind != SyntaxKind::VariableDeclarationList {
            return None;
        }
        let stmt = list.parent.as_ref()?;
        if matches!(
            stmt.kind,
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
        ) {
            Some(Arc::clone(stmt))
        } else {
            None
        }
    }

    /// The iterated expression of the for-in statement whose head declares
    /// `decl`.
    fn for_in_expression_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let stmt = Self::for_in_or_of_statement_of(decl)?;
        if stmt.kind != SyntaxKind::ForInStatement {
            return None;
        }
        match &stmt.data {
            NodeData::ForInOrOfStatement(d) => Some(Arc::clone(&d.expression)),
            _ => None,
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Property / instance-type helpers
    // ─────────────────────────────────────────────────────────────────

    /// Look up a property symbol by name on a type.
    ///
    /// First checks the type's own structured members, then falls back to the
    /// global `Array<T>` / `String` / `Number` / `Boolean` / `BigInt` interface
    /// symbols (whose cross-file members may be incomplete in this port). This
    /// mirrors the existence check in `has_property_of_type`. Returns `None`
    /// for missing properties; callers then fall back to `any`.
    pub(super) fn get_property_of_type(&self, t: &Arc<Type>, name: &str) -> Option<Arc<Symbol>> {
        // DEFERRED mapped types (`{ [K in keyof T]: V }` with a generic
        // constraint) have no materialized members; property access stays
        // permissive (any) as under the previous any-collapse — contextual
        // typing recovers precise types via
        // get_type_of_property_of_contextual_type.
        if let TypeData::Mapped(m) = &t.data
            && m.type_parameter.is_some()
        {
            let sym = Symbol::new(SymbolFlags::Property, name.to_string());
            return Some(Arc::new(sym));
        }
        // Structured member lookup. Uses a nested match (not `?`) so that
        // non-structured types (primitives) fall through to the fallbacks
        // below instead of short-circuiting to `None`.
        if let Some(structured) = t.as_structured() {
            if let Some(sym) = structured.members.get(name) {
                return Some(Arc::clone(sym));
            }
        }
        // Fallback for array types: array types created by `create_array_type`
        // carry no members, so resolve the property against the global
        // `Array<T>` interface symbol. Evolving arrays (`let x = []` widened)
        // are array types too — their members resolve the same way.
        let is_array_like = self.is_array_type(t)
            || matches!(&t.data, TypeData::EvolvingArray(_));
        if is_array_like {
            if let Some(array_sym) = self.globals.get("Array") {
                if let Some(member) = array_sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }
        // Fallback for pure function types (an anonymous object whose only
        // structure is its call signatures): function-typed values expose
        // the members of the global `Function` interface (`bind`, `call`,
        // `apply`, `length`, …). Mirrors Go's `getApparentType` merging
        // `globalFunctionType` for signature types.
        if t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::Anonymous)
            && let Some(structured) = t.as_structured()
            && structured.call_signature_count > 0
            && !self.is_array_type(t)
            && !matches!(&t.data, TypeData::EvolvingArray(_))
        {
            if let Some(function_sym) = self.globals.get("Function") {
                if let Some(member) = function_sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }
        // Fallback for primitive types (string/number/boolean/bigint) and
        // their literals: these have no structured members of their own, so
        // resolve the property symbol against the corresponding global
        // interface symbol. Mirrors the Array fallback above; member coverage
        // is best-effort (cross-file interface augmentations may be incomplete
        // in this port), with callers falling back to `any` otherwise.
        if let Some(interface_name) = self.primitive_interface_name(t) {
            if let Some(sym) = self.globals.get(interface_name) {
                if let Some(member) = sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }
        None
    }

    /// Return the global interface name for a primitive type, if any.
    /// Maps `string`/`StringLiteral` → `"String"`, `number`/`NumberLiteral` →
    /// `"Number"`, `boolean`/`BooleanLiteral` → `"Boolean"`,
    /// `bigint`/`BigIntLiteral` → `"BigInt"`.
    fn primitive_interface_name(&self, t: &Arc<Type>) -> Option<&'static str> {
        if t.flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            Some("String")
        } else if t
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            Some("Number")
        } else if t
            .flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            Some("Boolean")
        } else if t
            .flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            Some("BigInt")
        } else {
            None
        }
    }

    /// Get the type of a named property on a type, if the property exists.
    pub(super) fn get_property_type_of_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> {
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
    fn get_instance_type_of_constructor(&mut self, ctor_type: &Arc<Type>) -> Option<Arc<Type>> {
        // 1. Try the `prototype` property.
        if let Some(prop_sym) = self.get_property_of_type(ctor_type, "prototype") {
            let prop_type = self.get_type_of_symbol(&prop_sym);
            if !prop_type.flags.contains(TypeFlags::Any) {
                return Some(prop_type);
            }
        }
        // 2. Fall back to construct signatures' return types.
        let construct_sigs = self.get_signatures_of_type(ctor_type, SignatureKind::Construct);
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
            NodeData::PropertyAccessExpression(pa) => Some(pa.name.text().to_string()),
            NodeData::ElementAccessExpression(ea) => {
                Self::get_accessed_property_name_from_node(&ea.argument_expression)
            }
            // Destructured discriminant: `{ kind: k }` → `kind`, `{ k }` →
            // `k`. Mirrors Go's `getDestructuringPropertyName`.
            NodeData::BindingElement(be) => be
                .property_name
                .as_ref()
                .map(|pn| pn.text().to_string())
                .or_else(|| be.name.as_ref().map(|n| n.text().to_string())),
            _ => None,
        }
    }

    /// Resolve a const alias of a discriminant property access on `symbol`:
    /// `const k = obj.kind` (identifier alias) or `const { kind: k } = obj`
    /// (destructured alias). Returns the aliased access node. Mirrors Go's
    /// `getCandidateDiscriminantPropertyAccess` identifier case (flow.go
    /// ~L1460).
    fn discriminant_alias_access(
        &self,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(expr)?;
        if !self.symbol_is_const_variable(&sym) {
            return None;
        }
        let decl = Arc::clone(sym.value_declaration.as_ref()?);
        // `const k = obj.kind` — the initializer is an access whose receiver
        // matches the symbol.
        if let Some(init) = Self::candidate_variable_declaration_initializer(&decl) {
            if matches!(
                init.kind,
                SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
            ) {
                if let Some(recv) = init.expression() {
                    if self.is_symbol_identifier(recv, symbol) {
                        return Some(init);
                    }
                }
            }
        }
        // `const { kind: k } = obj` — a binding element alias without a
        // default; the root declaration's initializer references the symbol.
        if decl.kind == SyntaxKind::BindingElement {
            let NodeData::BindingElement(be) = &decl.data else {
                return None;
            };
            if be.dot_dot_dot_token.is_none() && be.initializer.is_none() {
                let pattern = decl.parent.as_ref()?;
                let var_decl = Arc::clone(pattern.parent.as_ref()?);
                if let Some(init) = Self::candidate_variable_declaration_initializer(&var_decl) {
                    let init_matches = match init.kind {
                        SyntaxKind::Identifier => self.is_symbol_identifier(&init, symbol),
                        SyntaxKind::PropertyAccessExpression
                        | SyntaxKind::ElementAccessExpression => init
                            .expression()
                            .is_some_and(|recv| self.is_symbol_identifier(recv, symbol)),
                        _ => false,
                    };
                    if init_matches {
                        return Some(decl);
                    }
                }
            }
        }
        None
    }

    /// A variable declaration's parenthesis-stripped initializer when the
    /// declaration has no type annotation. Mirrors Go's
    /// `getCandidateVariableDeclarationInitializer` (flow.go ~L1501).
    fn candidate_variable_declaration_initializer(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let NodeData::VariableDeclaration(data) = &decl.data else {
            return None;
        };
        if data.type_node.is_some() {
            return None;
        }
        data.initializer.as_ref().map(Self::skip_parentheses)
    }

    /// Whether `node` is a property access on `symbol`, e.g.
    /// `symbol.kind` or `symbol["kind"]`.
    fn is_property_access_on_symbol(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        match &node.data {
            NodeData::PropertyAccessExpression(pa) => {
                // Optional chains (`x?.a`) must NOT be treated as discriminant
                // property accesses: the value may be `undefined` regardless of
                // the property type. They're handled by the optional-chain
                // containment narrowing instead.
                pa.question_dot_token.is_none() && self.is_symbol_identifier(&pa.expression, symbol)
            }
            NodeData::ElementAccessExpression(ea) => {
                ea.question_dot_token.is_none() && self.is_symbol_identifier(&ea.expression, symbol)
            }
            _ => false,
        }
    }

    /// Filter `type_` (a union) to keep only constituents assignable to
    /// `candidate`. For non-union types, return `candidate` if the
    /// current type is assignable to it, otherwise the original type.
    fn narrow_to_subtype(&mut self, type_: &Arc<Type>, candidate: &Arc<Type>) -> Arc<Type> {
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
    fn remove_subtype_from_union(&mut self, type_: &Arc<Type>, candidate: &Arc<Type>) -> Arc<Type> {
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
                && u.union_or_intersection
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
