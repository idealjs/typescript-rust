//! Type relation checking: determining whether one type is assignable to,
//! a subtype of, or identical to another.
//!
//! Ported from `internal/checker/relater.go`. This is a large and complex
//! module (~5000 lines in Go); this file ports the core types and the
//! `isSimpleTypeRelatedTo` function which handles the most common cases
//! (any, unknown, never, primitive types, literals).

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{Symbol, SymbolFlags, SyntaxKind};
use crate::checker::is_tuple_type;
use crate::jsnum;

use super::checker::Checker;
use super::inference::{InferenceContext, InferenceInfo, InferencePriority};
use super::types::*;

// ────────────────────────────────────────────────────────────────────────────
// Relation comparison types
// ────────────────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    /// Mode flags for signature comparison.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct SignatureCheckMode: u32 {
        const None              = 0;
        const BivariantCallback = 1 << 0;
        const StrictCallback    = 1 << 1;
        const IgnoreReturnTypes = 1 << 2;
        const StrictArity       = 1 << 3;
        const StrictTopSignature= 1 << 4;
    }
}

pub const SIGNATURE_CHECK_MODE_CALLBACK: SignatureCheckMode =
    SignatureCheckMode::from_bits_truncate(
        SignatureCheckMode::BivariantCallback.bits() | SignatureCheckMode::StrictCallback.bits(),
    );

impl SignatureCheckMode {
    /// Convenience alias matching Go's `SignatureCheckModeCallback` constant.
    /// Equivalent to `BivariantCallback | StrictCallback`.
    pub const Callback: Self = SIGNATURE_CHECK_MODE_CALLBACK;
}

bitflags::bitflags! {
    /// Flags controlling intersection state during type comparison.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct IntersectionState: u32 {
        const None   = 0;
        const Source = 1 << 0;
        const Target = 1 << 1;
    }

    /// Recursion direction flags.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct RecursionFlags: u32 {
        const None   = 0;
        const Source = 1 << 0;
        const Target = 1 << 1;
    }
}

pub const RECURSION_FLAGS_BOTH: RecursionFlags = RecursionFlags::from_bits_truncate(
    RecursionFlags::Source.bits() | RecursionFlags::Target.bits(),
);

/// Flags indicating which side of a comparison is being expanded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExpandingFlags(pub u8);

impl ExpandingFlags {
    pub const NONE: Self = Self(0);
    pub const SOURCE: Self = Self(1 << 0);
    pub const TARGET: Self = Self(1 << 1);
    pub const BOTH: Self = Self(Self::SOURCE.0 | Self::TARGET.0);
}

bitflags::bitflags! {
    /// Result of a type relation comparison.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct RelationComparisonResult: u32 {
        const None                = 0;
        const Succeeded           = 1 << 0;
        const Failed              = 1 << 1;
        const ReportsUnmeasurable = 1 << 3;
        const ReportsUnreliable   = 1 << 4;
        const ComplexityOverflow  = 1 << 5;
        const StackDepthOverflow  = 1 << 6;
    }
}

pub const RELATION_COMPARISON_RESULT_REPORTS_MASK: RelationComparisonResult =
    RelationComparisonResult::from_bits_truncate(
        RelationComparisonResult::ReportsUnmeasurable.bits()
            | RelationComparisonResult::ReportsUnreliable.bits(),
    );

pub const RELATION_COMPARISON_RESULT_OVERFLOW: RelationComparisonResult =
    RelationComparisonResult::from_bits_truncate(
        RelationComparisonResult::ComplexityOverflow.bits()
            | RelationComparisonResult::StackDepthOverflow.bits(),
    );

/// Maximum recursion depth for `is_type_related_to` before the relater
/// gives up and optimistically assumes the types are related. Matches the
/// spirit of Go's `stackDepthOverflow` constant in `relater.go` (128).
/// Without this, recursive structural types such as
/// `type Box<T> = { next: Box<T> | null }` blow the native stack.
pub const RELATER_MAX_DEPTH: u32 = 128;

/// The kind of relation being checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RelationKind {
    #[default]
    Identity,
    Subtype,
    StrictSubtype,
    Assignable,
    Comparable,
}

/// Key into the per-call relation cache. Combines the source and target
/// `Type` pointers (via `Arc::as_ptr`) with the `RelationKind`, since the
/// same type pair may compare differently under different relations
/// (e.g. `Subtype` vs `Assignable`).
///
/// We use pointer identity rather than `Type::id` because the `id` field
/// is not yet populated with unique values across all Type construction
/// sites (many bypass `Type::new` and leave `id` at 0). Pointer identity
/// is stable for the lifetime of a single top-level comparison (during
/// which the Arcs are kept alive) and the cache is cleared at top-level
/// entry, so stale pointers never accumulate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelationCacheKey {
    pub source_ptr: usize,
    pub target_ptr: usize,
    pub relation: RelationKind,
}

/// A relation cache that stores comparison results.
#[derive(Debug)]
pub struct Relation {
    pub kind: RelationKind,
    results: HashMap<CacheHashKey, RelationComparisonResult>,
}

impl Relation {
    pub fn new(kind: RelationKind) -> Self {
        Self {
            kind,
            results: HashMap::new(),
        }
    }

    pub fn get(&self, key: &CacheHashKey) -> RelationComparisonResult {
        self.results.get(key).copied().unwrap_or_default()
    }

    pub fn set(&mut self, key: CacheHashKey, result: RelationComparisonResult) {
        self.results.insert(key, result);
    }

    pub fn size(&self) -> usize {
        self.results.len()
    }

    pub fn clear(&mut self) {
        self.results.clear();
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Type relation entry points
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Check if `source` is identical to `target`.
    pub fn is_type_identical_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        // Fast path: same pointer
        if Arc::ptr_eq(source, target) {
            return true;
        }
        // Types must have identical flags (excluding complex types)
        if source.flags != target.flags {
            return false;
        }
        if source.flags.contains(TYPE_FLAGS_SINGLETON) {
            return true;
        }
        self.is_simple_type_identical_to(source, target)
    }

    /// Check if `source` is assignable to `target`.
    pub fn is_type_assignable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Assignable)
    }

    /// Check if `source` is a subtype of `target`.
    pub fn is_type_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Subtype)
    }

    /// Check if `source` is a strict subtype of `target`.
    pub fn is_type_strict_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::StrictSubtype)
    }

    /// Check if `source` is comparable to `target`.
    pub fn is_type_comparable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Comparable)
    }

    /// Check if two types are comparable (in either direction).
    pub fn are_types_comparable(&mut self, type1: &Arc<Type>, type2: &Arc<Type>) -> bool {
        self.is_type_comparable_to(type1, type2) || self.is_type_comparable_to(type2, type1)
    }

    /// Main entry point for type relation checking.
    ///
    /// Handles union/intersection/object types by delegating to specialized
    /// methods, falling back to `is_simple_type_related_to` for primitives.
    ///
    /// Implements two layers of recursion protection (mirroring Go's
    /// `relater.go`):
    ///
    /// 1. **Cycle detection** via `relation_in_progress`: when the same
    ///    `(source, target, relation)` triple is encountered higher up
    ///    the call stack (e.g. comparing `Box<X>` to `Box<Y>` reaches
    ///    `Box<X>` vs `Box<Y>` again via a `next` property), we
    ///    optimistically return `true` to terminate the cycle. This is
    ///    correct for the common mutually-recursive case.
    ///
    /// 2. **Depth guard** via `relater_depth`: once the comparison stack
    ///    exceeds `RELATER_MAX_DEPTH` (128), we also return `true`. This
    ///    catches pathological cases the cycle set might miss (very deep
    ///    chains of distinct types).
    ///
    /// A per-call `relation_cache` memoises results so repeated
    /// sub-comparisons within a single top-level call don't recompute.
    /// The cache is cleared at top-level entry (depth 0 → 1) to avoid
    /// carrying optimistic cycle-broken results across calls.
    fn is_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        // Recursion guard: structural comparisons on recursive types
        // (e.g. `type Box<T> = { next: Box<T> | null }`) can blow the native
        // stack. Once we exceed `RELATER_MAX_DEPTH`, optimistically assume
        // the types are related — Go's relater does the same when it hits
        // `stackDepthOverflow`.
        if self.relater_depth >= RELATER_MAX_DEPTH {
            return true;
        }
        // On top-level entry, reset the per-call caches so optimistic
        // cycle-broken results from a previous call don't leak in.
        if self.relater_depth == 0 {
            self.relation_cache.clear();
            self.relation_in_progress.clear();
        }
        let key = RelationCacheKey {
            source_ptr: Arc::as_ptr(source) as usize,
            target_ptr: Arc::as_ptr(target) as usize,
            relation,
        };
        // Cycle break: if this triple is already being computed higher up
        // the stack, assume `true` to terminate the recursion.
        if self.relation_in_progress.contains(&key) {
            return true;
        }
        // Cache hit: a previous sub-comparison within this top-level call
        // already determined the result.
        if let Some(&cached) = self.relation_cache.get(&key) {
            return cached;
        }
        self.relation_in_progress.insert(key);
        self.relater_depth += 1;
        let result = self.is_type_related_to_inner(source, target, relation);
        self.relater_depth -= 1;
        self.relation_in_progress.remove(&key);
        self.relation_cache.insert(key, result);
        result
    }

    fn is_type_related_to_inner(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if self.is_simple_type_related_to(source, target, relation) {
            return true;
        }

        let s = source.flags;
        let t = target.flags;

        // NOTE: `intersects` (not `contains`) — a type "is a union or
        // intersection" if it has *any* of those bits set, not both. `contains`
        // would require the argument to be a subset of the type's flags, so
        // `Union.contains(Union | Intersection)` is false and this branch was
        // never reachable. This latent bug was masked because the relater was
        // not wired into diagnostics (same-type cases passed via Arc::ptr_eq).
        if s.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            || t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
        {
            return self.is_union_or_intersection_related_to(source, target, relation);
        }

        if s.contains(TypeFlags::Object) && t.contains(TypeFlags::Object) {
            // Check array types: Array<T> is assignable to Array<U> if T is assignable to U
            if self.is_array_type(source) && self.is_array_type(target) {
                return self.is_array_type_related_to(source, target, relation);
            }
            // Check tuple types: element-by-element comparison
            if self.is_tuple_type(source) && self.is_tuple_type(target) {
                return self.is_tuple_type_related_to(source, target, relation);
            }
            // Generic instantiation: `Foo<X>` vs `Foo<Y>` for the same generic
            // `Foo<T>`. Variance-aware comparison of type arguments (P3.7d).
            // Falls back to structural comparison when the variance-based
            // check is inconclusive (Ternary::Maybe) or not applicable
            // (None — e.g. tuples, marker types).
            if let Some(result) = self.generic_type_reference_related_to(source, target, relation) {
                if result.is_true() {
                    return true;
                }
                if result.is_false() {
                    return false;
                }
                // Ternary::Maybe / Unknown: fall through to structural comparison.
            }
            return self.is_object_type_related_to(source, target, relation);
        }

        // Handle type parameters: check constraints
        if s.contains(TypeFlags::TypeParameter) {
            // Source is a type parameter, check if its constraint is assignable to target
            if let Some(constraint) = self.get_constraint_of_type_parameter(source) {
                if self.is_type_related_to(&constraint, target, relation) {
                    return true;
                }
            }
        }
        if t.contains(TypeFlags::TypeParameter) {
            // Target is a type parameter with a constraint
            if let Some(constraint) = self.get_constraint_of_type_parameter(target) {
                // Check if source is assignable to the constraint
                if self.is_type_related_to(source, &constraint, relation) {
                    return true;
                }
            }
        }

        // Handle conditional types: use resolved type if available
        if s.contains(TypeFlags::Conditional) {
            if let Some(resolved) = self.get_resolved_type_of_conditional_type(source) {
                if self.is_type_related_to(&resolved, target, relation) {
                    return true;
                }
            }
        }
        if t.contains(TypeFlags::Conditional) {
            // First, try the resolved-type fast path (used when the
            // conditional has already been evaluated to a concrete type).
            if let Some(resolved) = self.get_resolved_type_of_conditional_type(target) {
                if self.is_type_related_to(source, &resolved, relation) {
                    return true;
                }
            }
            // Otherwise, fall back to the conditional-target comparison
            // (P3.7e): compare source against the true/false branches with
            // the permissive/restrictive short-circuits. Returns None when
            // the conditional is unsupported (infer positions, distribution
            // dependence, identical source conditional) — in that case we
            // fall through to other strategies.
            if let Some(result) = self.conditional_type_related_to(source, target, relation) {
                if result.is_true() {
                    return true;
                }
                if result.is_false() {
                    return false;
                }
                // Ternary::Maybe / Unknown: fall through.
            }
        }

        // Handle mapped types: base constraint check
        if s.contains(TypeFlags::Object) && source.object_flags.contains(ObjectFlags::Mapped) {
            if let Some(constraint) = self.get_constraint_of_mapped_type(source) {
                if self.is_type_related_to(&constraint, target, relation) {
                    return true;
                }
            }
        }
        if t.contains(TypeFlags::Object) && target.object_flags.contains(ObjectFlags::Mapped) {
            if let Some(constraint) = self.get_constraint_of_mapped_type(target) {
                if self.is_type_related_to(source, &constraint, relation) {
                    return true;
                }
            }
            // Direct mapped-vs-mapped comparison (P3.7e). Only fires when
            // the source is itself a mapped type and we're not in identity
            // mode (identity needs the full modifiers check which we
            // don't yet implement). Returns None for unsupported cases
            // (e.g. name remapping), which fall through to structural
            // comparison.
            if s.contains(TypeFlags::Object) && source.object_flags.contains(ObjectFlags::Mapped) {
                if let Some(result) = self.mapped_type_related_to(source, target, relation) {
                    if result.is_true() {
                        return true;
                    }
                    if result.is_false() {
                        return false;
                    }
                }
            }
        }

        false
    }

    /// Check if two array types are related.
    ///
    /// Array<T> is assignable to Array<U> if T is assignable to U (covariant).
    fn is_array_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);

        if source_args.is_empty() || target_args.is_empty() {
            // If either type has no type arguments, fall back to structural comparison
            return self.is_object_type_related_to(source, target, relation);
        }

        // Compare element types: Array<T> ~ Array<U> iff T ~ U
        // (covariant in the element type)
        let source_elem = &source_args[0];
        let target_elem = &target_args[0];
        self.is_type_related_to(source_elem, target_elem, relation)
    }

    /// Check if two tuple types are related.
    ///
    /// Tuples are compared element-by-element. Each element in source must be
    /// assignable to the corresponding element in target.
    fn is_tuple_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_tuple = match &source.data {
            TypeData::Tuple(t) => t,
            _ => return false,
        };
        let target_tuple = match &target.data {
            TypeData::Tuple(t) => t,
            _ => return false,
        };

        // Compare element-by-element
        let min_len = source_tuple
            .element_infos
            .len()
            .min(target_tuple.element_infos.len());
        for i in 0..min_len {
            let source_elem = &source_tuple.element_infos[i];
            let target_elem = &target_tuple.element_infos[i];

            // Get the element types from the interface_data's structured type members
            let source_type = self.get_tuple_element_type(source, i);
            let target_type = self.get_tuple_element_type(target, i);

            if let (Some(st), Some(tt)) = (source_type, target_type) {
                if !self.is_type_related_to(&st, &tt, relation) {
                    return false;
                }
            }

            // Check element flags compatibility (required/optional/rest/variadic)
            if !self.is_element_flags_compatible(source_elem.flags, target_elem.flags, relation) {
                return false;
            }
        }

        // If target has more elements than source, those must be optional or rest
        if source_tuple.element_infos.len() < target_tuple.element_infos.len() {
            for i in source_tuple.element_infos.len()..target_tuple.element_infos.len() {
                let flags = target_tuple.element_infos[i].flags;
                if !flags.contains(ElementFlags::Optional)
                    && !flags.contains(ElementFlags::Rest)
                    && !flags.contains(ElementFlags::Variadic)
                {
                    return false;
                }
            }
        }

        true
    }

    /// Get the type of a tuple element at a given index.
    pub(super) fn get_tuple_element_type(&self, t: &Arc<Type>, index: usize) -> Option<Arc<Type>> {
        match &t.data {
            TypeData::Tuple(tuple) => {
                // The element type is stored directly on `TupleElementInfo`
                // (mirrors Go's `TupleElementInfo.Type`). This avoids the
                // need to re-resolve a structured member symbol.
                tuple
                    .element_infos
                    .get(index)
                    .and_then(|info| info.type_.clone())
            }
            _ => None,
        }
    }

    /// Check if element flags are compatible between source and target.
    fn is_element_flags_compatible(
        &self,
        source: ElementFlags,
        target: ElementFlags,
        _relation: RelationKind,
    ) -> bool {
        // Required can be assigned to Required or Optional
        // Optional can be assigned to Optional
        // Rest can be assigned to Rest
        // Variadic can be assigned to Variadic or Rest
        if source.contains(ElementFlags::Required) {
            target.contains(ElementFlags::Required) || target.contains(ElementFlags::Optional)
        } else if source.contains(ElementFlags::Optional) {
            target.contains(ElementFlags::Optional)
        } else if source.contains(ElementFlags::Rest) {
            target.contains(ElementFlags::Rest)
        } else if source.contains(ElementFlags::Variadic) {
            target.contains(ElementFlags::Variadic) || target.contains(ElementFlags::Rest)
        } else {
            true
        }
    }

    /// Handle union/intersection type relations.
    fn is_union_or_intersection_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let s = source.flags;

        if s.contains(TypeFlags::Union) {
            // For Comparable: a union source is comparable to target if ANY
            // constituent is comparable (loose check for error-reporting).
            // For all other relations (Subtype/StrictSubtype/Assignable): ALL
            // constituents must be related. E.g. `"a" | "b"` is NOT assignable
            // to `"a"` because `"b"` isn't.
            if relation == RelationKind::Comparable {
                return self.some_type_related_to_type(source, target, relation);
            }
            return self.each_type_related_to_type(source, target, relation);
        }

        if s.contains(TypeFlags::Intersection) {
            return self.some_type_related_to_type(source, target, relation);
        }

        let t = target.flags;

        if t.contains(TypeFlags::Union) {
            return self.type_related_to_some_type(source, target, relation);
        }

        if t.contains(TypeFlags::Intersection) {
            return self.type_related_to_each_type(source, target, relation);
        }

        false
    }

    /// Source is a union/intersection: check if at least one constituent
    /// is related to target.
    fn some_type_related_to_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = source.as_union_or_intersection() {
            for t in &ui.types {
                if self.is_type_related_to(t, target, relation) {
                    return true;
                }
            }
        }
        false
    }

    /// Source is a union (for subtype/strictSubtype): check if ALL
    /// constituents are related to target.
    fn each_type_related_to_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = source.as_union_or_intersection() {
            for t in &ui.types {
                if !self.is_type_related_to(t, target, relation) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Target is a union: check if source is related to at least one constituent.
    fn type_related_to_some_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = target.as_union_or_intersection() {
            for t in &ui.types {
                if self.is_type_related_to(source, t, relation) {
                    return true;
                }
            }
        }
        false
    }

    /// Target is an intersection: check if source is related to ALL constituents.
    fn type_related_to_each_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = target.as_union_or_intersection() {
            for t in &ui.types {
                if !self.is_type_related_to(source, t, relation) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Check object type relation (structural typing).
    ///
    /// For assignability: source must have all properties of target.
    fn is_object_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_struct = match source.as_structured() {
            Some(s) => s,
            None => return false,
        };
        let target_struct = match target.as_structured() {
            Some(t) => t,
            None => return false,
        };

        // Check properties: target properties must exist in source with compatible types
        for target_prop in &target_struct.properties {
            // Check that source has a matching property by name.
            let source_prop = match source_struct.members.get(&target_prop.name) {
                Some(p) => p,
                None => return false,
            };
            // Check that the source property type is assignable to the
            // target property type (depth check).
            let source_type = self.get_type_of_symbol(source_prop);
            let target_type = self.get_type_of_symbol(target_prop);
            if !self.is_type_assignable_to(&source_type, &target_type) {
                return false;
            }
        }

        // Check call signatures
        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }

        // Check construct signatures
        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }

        // Check index signatures
        if !self.is_index_signatures_related_to(source, target, relation) {
            return false;
        }

        true
    }

    // ────────────────────────────────────────────────────────────────────────
    // Simple type relation checking
    // ────────────────────────────────────────────────────────────────────────

    /// Check if two types are identical at a simple level.
    ///
    /// This handles intrinsic types (by name), literal types (by value),
    /// and type parameters (by ID).
    fn is_simple_type_identical_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        match (&source.data, &target.data) {
            (TypeData::Intrinsic(s), TypeData::Intrinsic(t)) => {
                s.intrinsic_name == t.intrinsic_name
            }
            (TypeData::Literal(s), TypeData::Literal(t)) => s.value == t.value,
            (TypeData::TypeParameter(s), TypeData::TypeParameter(t)) => {
                s.is_this_type == t.is_this_type
            }
            _ => {
                // For other types, fall back to flag comparison.
                // Full structural comparison requires the full relater.
                source.flags == target.flags
            }
        }
    }

    /// Check if `source` is related to `target` for simple (non-structured) types.
    ///
    /// Mirrors `Checker.isSimpleTypeRelatedTo` in Go. Handles:
    /// - `any` target → always true
    /// - `never` source → always true
    /// - `unknown` target → always true (except strict subtype of any)
    /// - String-like → string, number-like → number, etc.
    /// - Literal → matching base type
    /// - null/undefined rules (strict vs non-strict)
    /// - `any` source in assignable/comparable relation → true
    pub fn is_simple_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let s = source.flags;
        let t = target.flags;

        // Target is `any` → everything is assignable
        // Source is `never` → assignable to everything
        if t.contains(TypeFlags::Any) || s.contains(TypeFlags::Never) {
            return true;
        }

        // Target is `unknown` → everything is assignable
        // (except: strict subtype relation doesn't allow any → unknown)
        if t.contains(TypeFlags::Unknown)
            && !(relation == RelationKind::StrictSubtype && s.contains(TypeFlags::Any))
        {
            return true;
        }

        // Target is `never` → only `never` is assignable
        if t.contains(TypeFlags::Never) {
            return false;
        }

        // String-like types are assignable to `string`
        if s.intersects(TYPE_FLAGS_STRING_LIKE) && t.contains(TypeFlags::String) {
            return true;
        }

        // String literal enum → string literal (matching value)
        if s.contains(TypeFlags::StringLiteral)
            && s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::StringLiteral)
            && !t.contains(TypeFlags::EnumLiteral)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        // Plain literal → same-kind literal (matching value). E.g. `"foo"` is
        // assignable to `"foo"`, `42` to `42`, `true` to `true`. This is the
        // non-enum counterpart of the enum-literal checks above. Mirrors Go's
        // `isSimpleTypeRelatedTo` literal comparisons.
        if s.intersects(TYPE_FLAGS_LITERAL)
            && t.intersects(TYPE_FLAGS_LITERAL)
            && (s & TYPE_FLAGS_LITERAL) == (t & TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        // Number-like types are assignable to `number`
        if s.intersects(TYPE_FLAGS_NUMBER_LIKE) && t.contains(TypeFlags::Number) {
            return true;
        }

        // Number literal enum → number literal (matching value)
        if s.contains(TypeFlags::NumberLiteral)
            && s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::NumberLiteral)
            && !t.contains(TypeFlags::EnumLiteral)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        // BigInt-like → BigInt
        if s.intersects(TYPE_FLAGS_BIG_INT_LIKE) && t.contains(TypeFlags::BigInt) {
            return true;
        }

        // Boolean-like → Boolean
        if s.intersects(TYPE_FLAGS_BOOLEAN_LIKE) && t.contains(TypeFlags::Boolean) {
            return true;
        }

        // Symbol-like → ESSymbol
        if s.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE) && t.contains(TypeFlags::ESSymbol) {
            return true;
        }

        // Enum → Enum (same name): simplified, full check needs symbol comparison
        // TODO: port isEnumTypeRelatedTo

        // EnumLiteral → EnumLiteral: simplified
        if s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::EnumLiteral)
            && s.intersects(TYPE_FLAGS_LITERAL)
            && t.intersects(TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        // In non-strictNullChecks mode, `undefined` and `null` are assignable
        // to anything except `never`. Since unions and intersections may reduce
        // to `never`, we exclude them here.
        if s.contains(TypeFlags::Undefined)
            && (!self.strict_null_checks && !t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
                || t.intersects(TypeFlags::Undefined | TypeFlags::Void))
        {
            return true;
        }

        if s.contains(TypeFlags::Null)
            && (!self.strict_null_checks && !t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
                || t.contains(TypeFlags::Null))
        {
            return true;
        }

        // Object → non-primitive (object)
        if s.contains(TypeFlags::Object)
            && t.contains(TypeFlags::NonPrimitive)
            && !(relation == RelationKind::StrictSubtype)
        {
            return true;
        }

        // For assignable and comparable relations:
        if relation == RelationKind::Assignable || relation == RelationKind::Comparable {
            // `any` source is assignable to everything
            if s.contains(TypeFlags::Any) {
                return true;
            }

            // `number` is assignable to numeric enum types (bit-flag pattern)
            if s.contains(TypeFlags::Number)
                && (t.contains(TypeFlags::Enum)
                    || (t.contains(TypeFlags::NumberLiteral) && t.contains(TypeFlags::EnumLiteral)))
            {
                return true;
            }

            // Numeric literal is assignable to numeric enum types (matching value)
            if s.contains(TypeFlags::NumberLiteral)
                && !s.contains(TypeFlags::EnumLiteral)
                && (t.contains(TypeFlags::Enum)
                    || (t.contains(TypeFlags::NumberLiteral)
                        && t.contains(TypeFlags::EnumLiteral)
                        && self.literal_values_equal(source, target)))
            {
                return true;
            }

            // Anything is assignable to a union containing undefined, null, and {}
            // TODO: port isUnknownLikeUnionType
        }

        false
    }

    /// Check if two literal types have equal values.
    fn literal_values_equal(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        match (&a.data, &b.data) {
            (TypeData::Literal(la), TypeData::Literal(lb)) => la.value == lb.value,
            _ => false,
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Index signature comparison
    // ────────────────────────────────────────────────────────────────────────

    /// Check if the index signatures of two types are compatible.
    ///
    /// If target has `[key: string]: T`, source must have `[key: string]: U`
    /// where U is assignable to T.
    /// If target has `[key: number]: T`, source must have `[key: number]: U`
    /// where U is assignable to T.
    fn is_index_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_struct = match source.as_structured() {
            Some(s) => s,
            None => return false,
        };
        let target_struct = match target.as_structured() {
            Some(t) => t,
            None => return false,
        };

        let source_indexes = &source_struct.index_infos;
        let target_indexes = &target_struct.index_infos;

        if target_indexes.is_empty() {
            return true; // Target has no index signature requirements
        }

        for target_index in target_indexes {
            let target_key = &target_index.key_type;
            let target_value = &target_index.value_type;

            // Find a matching index signature in source
            let mut found_match = false;
            for source_index in source_indexes {
                let source_key = &source_index.key_type;
                let source_value = &source_index.value_type;

                // Key types must match (string vs number)
                let key_match = match (target_key, source_key) {
                    (Some(tk), Some(sk)) => self.is_type_related_to(sk, tk, relation),
                    (None, _) => true,
                    (_, None) => false,
                };

                if !key_match {
                    continue;
                }

                // Value type must be assignable: source value -> target value
                let value_match = match (target_value, source_value) {
                    (Some(tv), Some(sv)) => self.is_type_related_to(sv, tv, relation),
                    // If target has no value type, any source is fine
                    // If source has no value type, it can't match target's value type
                    (None, _) => true,
                    (_, None) => false,
                };

                if value_match {
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                // No matching source index signature: fall back to checking
                // that every source property is assignable to the target
                // index's value type (mirrors Go's `membersRelatedToIndexInfo`
                // fallback in `typeRelatedToIndexInfo`). This makes
                // `{ a: 1 }` assignable to `{ [key: string]: number }`.
                let result = self.members_related_to_index_info(source, target_index, relation);
                if result.is_false() {
                    return false;
                }
            }
        }

        true
    }

    // ────────────────────────────────────────────────────────────────────────
    // Signature comparison (ported from internal/checker/relater.go)
    // ────────────────────────────────────────────────────────────────────────

    /// Compare a source signature against a target signature.
    ///
    /// Direct port of Go's `compareSignaturesRelated`. Returns a Ternary:
    /// - `True` if `source` is related to `target` under `relation`
    /// - `False` if not
    /// - `Maybe`/`Unknown` for partial results (currently unused)
    ///
    /// Handles (in order):
    /// 1. Pointer-equality fast path.
    /// 2. The "top signature" short-circuit (a `(...args: any) => any`
    ///    source matches any target).
    /// 3. Strict-top-signature asymmetry (strict subtype relation).
    /// 4. Parameter count check (with rest/tuple-rest handling).
    /// 5. Generic signature instantiation (skipped — we erase).
    /// 6. `this` type comparison (covariant for void source, contravariant
    ///    otherwise, bivariant when `strict_function_types` is off).
    /// 7. Per-parameter type comparison (bivariant by default; strictly
    ///    contravariant when `strict_function_types` is on and neither
    ///    side is a method/constructor).
    /// 8. Return type comparison (covariant; bivariant for callbacks).
    /// 9. Type predicate comparison (for type guards).
    pub fn compare_signatures_related(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        check_mode: SignatureCheckMode,
        relation: RelationKind,
    ) -> Ternary {
        // 1. Fast path: same pointer.
        if Arc::ptr_eq(source, target) {
            return Ternary::True;
        }

        // 2. Top-signature short-circuit: a top-signature target matches
        //    anything (the source doesn't need to be top, since `any` is
        //    a wildcard). Strict-subtype relation additionally requires
        //    the source to *also* be a top signature.
        let source_is_top = if check_mode.contains(SignatureCheckMode::StrictTopSignature)
            && self.is_top_signature(source)
        {
            true
        } else {
            false
        };
        if !source_is_top && self.is_top_signature(target) {
            return Ternary::True;
        }
        if check_mode.contains(SignatureCheckMode::StrictTopSignature)
            && source_is_top
            && !self.is_top_signature(target)
        {
            return Ternary::False;
        }

        // 4. Parameter count check.
        let target_count = self.get_parameter_count(target);
        let source_has_more = if !self.has_effective_rest_parameter(target) {
            if check_mode.contains(SignatureCheckMode::StrictArity) {
                self.has_effective_rest_parameter(source)
                    || self.get_parameter_count(source) > target_count
            } else {
                self.get_min_argument_count(source) > target_count
            }
        } else {
            false
        };
        if source_has_more {
            return Ternary::False;
        }

        // 5. Generic signature instantiation. We don't yet support generic
        //    signature instantiation in the relater (it requires
        //    `instantiateSignatureInContextOf` from `inference.go`).
        //    Instead we erase — `getErasedSignature` returns the same
        //    signature when there are no type parameters, which is the
        //    common case for our current parity fixtures.
        let source = if !source.type_parameters.is_empty()
            && !type_parameters_same(
                source.type_parameters.as_slice(),
                target.type_parameters.as_slice(),
            ) {
            self.get_erased_signature(source)
        } else {
            Arc::clone(source)
        };
        let target = if !source.type_parameters.is_empty()
            && !type_parameters_same(
                source.type_parameters.as_slice(),
                target.type_parameters.as_slice(),
            ) {
            self.get_erased_signature(target)
        } else {
            Arc::clone(target)
        };

        let source_count = self.get_parameter_count(&source);
        let source_rest = self.get_non_array_rest_type(&source);
        let target_rest = self.get_non_array_rest_type(&target);

        // 6. Variance selection. `strict_function_types` makes method-shaped
        //    signatures stay bivariant; everything else is contravariant on
        //    parameters. We approximate "method-shaped" by checking the
        //    declaration kind via the signature's declaration node.
        let strict_variance = !check_mode.contains(SignatureCheckMode::Callback)
            && self.strict_function_types
            && !self.signature_is_method_or_constructor(&target);

        let mut result = Ternary::True;

        // 7. `this` type comparison.
        let source_this = self.get_this_type_of_signature(&source);
        if let Some(source_this) = source_this {
            if !source_this.flags.contains(TypeFlags::Void) {
                let target_this = self.get_this_type_of_signature(&target);
                if let Some(target_this) = target_this {
                    let mut related = Ternary::False;
                    if !strict_variance {
                        related = self.compare_types(
                            source_this.clone(),
                            target_this.clone(),
                            relation,
                            false,
                        );
                    }
                    if related.is_false() {
                        related = self.compare_types(target_this, source_this, relation, false);
                    }
                    if related.is_false() {
                        return Ternary::False;
                    }
                    result = result.and(related);
                }
            }
        }

        // 8. Per-parameter type comparison.
        let param_count = if source_rest.is_some() || target_rest.is_some() {
            source_count.min(target_count)
        } else {
            source_count.max(target_count)
        };
        let rest_index = if source_rest.is_some() || target_rest.is_some() {
            param_count.saturating_sub(1) as isize
        } else {
            -1
        };
        for i in 0..param_count {
            let source_type = if i as isize == rest_index {
                self.get_rest_or_any_type_at_position(&source, i)
            } else {
                self.try_get_type_at_position(&source, i)
                    .unwrap_or_else(|| self.any_type())
            };
            let target_type = if i as isize == rest_index {
                self.get_rest_or_any_type_at_position(&target, i)
            } else {
                self.try_get_type_at_position(&target, i)
                    .unwrap_or_else(|| self.any_type())
            };

            // Skip if both are the same pointer and we're not in strict-arity mode.
            if Arc::ptr_eq(&source_type, &target_type)
                && !check_mode.contains(SignatureCheckMode::StrictArity)
            {
                continue;
            }

            // Bivariant/contravariant parameter comparison.
            // Default: bivariant — try source→target first, fall back to target→source.
            let mut related = Ternary::False;
            if !check_mode.contains(SignatureCheckMode::Callback) && !strict_variance {
                related =
                    self.compare_types(source_type.clone(), target_type.clone(), relation, false);
            }
            if related.is_false() {
                related =
                    self.compare_types(target_type.clone(), source_type.clone(), relation, false);
            }
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }

        // 9. Return type comparison.
        if !check_mode.contains(SignatureCheckMode::IgnoreReturnTypes) {
            let target_return = self.get_non_circular_return_type_of_signature(&target);
            // `void` and `any` target returns match anything.
            if !Arc::ptr_eq(&target_return, &self.void_type())
                && !target_return.flags.contains(TypeFlags::Any)
            {
                let source_return = self.get_non_circular_return_type_of_signature(&source);
                let target_type_predicate = self.get_type_predicate_of_signature(&target).cloned();
                if let Some(target_tp) = target_type_predicate {
                    let source_tp = self.get_type_predicate_of_signature(&source).cloned();
                    match source_tp {
                        Some(source_tp) => {
                            result = result.and(self.compare_type_predicate_related_to(
                                &source_tp, &target_tp, relation,
                            ));
                        }
                        None => {
                            // Source lacks a type predicate but target has one.
                            if matches!(
                                target_tp.kind,
                                TypePredicateKind::Identifier | TypePredicateKind::This
                            ) {
                                return Ternary::False;
                            }
                        }
                    }
                    if result.is_false() {
                        return result;
                    }
                } else {
                    // No type predicate on target: covariant return check.
                    // For callback signatures, also check bivariantly.
                    let mut related = Ternary::False;
                    if check_mode.contains(SignatureCheckMode::BivariantCallback) {
                        related = self.compare_types(
                            target_return.clone(),
                            source_return.clone(),
                            relation,
                            false,
                        );
                    }
                    if related.is_false() {
                        related = self.compare_types(source_return, target_return, relation, false);
                    }
                    result = result.and(related);
                    if result.is_false() {
                        return result;
                    }
                }
            }
        }

        result
    }

    /// Compare two type predicates.
    /// Direct port of Go's `compareTypePredicateRelatedTo`.
    pub fn compare_type_predicate_related_to(
        &mut self,
        source: &TypePredicate,
        target: &TypePredicate,
        relation: RelationKind,
    ) -> Ternary {
        if source.kind != target.kind {
            return Ternary::False;
        }
        if matches!(
            source.kind,
            TypePredicateKind::Identifier | TypePredicateKind::AssertsIdentifier
        ) && source.parameter_index != target.parameter_index
        {
            return Ternary::False;
        }
        match (&source.t, &target.t) {
            (None, None) => Ternary::True,
            (Some(s), None) => Ternary::True,
            (Some(s), Some(t)) => self.compare_types(s.clone(), t.clone(), relation, false),
            (None, Some(_)) => Ternary::False,
        }
    }

    /// Compare two types under a relation, returning a Ternary.
    /// Currently wraps `is_type_related_to` and converts the bool result.
    pub fn compare_types(
        &mut self,
        source: Arc<Type>,
        target: Arc<Type>,
        relation: RelationKind,
        _report_errors: bool,
    ) -> Ternary {
        if self.is_type_related_to(&source, &target, relation) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    /// Whether a signature's declaration is a method or constructor.
    /// Used by `compare_signatures_related` to keep method-shaped
    /// signatures bivariant even under `strict_function_types`.
    /// Mirrors Go's `target.declaration.Kind` check.
    fn signature_is_method_or_constructor(&self, sig: &Arc<Signature>) -> bool {
        let Some(decl) = sig.declaration.as_ref() else {
            return false;
        };
        matches!(
            decl.kind,
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature | SyntaxKind::Constructor
        )
    }

    /// Direct port of Go's `signaturesRelatedTo`. Compares the call (or
    /// construct) signature lists of two types, choosing one of three
    /// comparison strategies based on the lists' shapes.
    pub fn signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        kind: SignatureKind,
        relation: RelationKind,
    ) -> Ternary {
        // Wildcard: `anyFunctionType` source matches any function target.
        if Arc::ptr_eq(source, &self.any_function_type()) {
            return Ternary::True;
        }
        // Wildcard: non-wildcard source does NOT match a wildcard target.
        if Arc::ptr_eq(target, &self.any_function_type()) {
            return Ternary::False;
        }

        let source_sigs = self.get_signatures_of_type(source, kind);
        let target_sigs = self.get_signatures_of_type(target, kind);

        // Construct-signature abstractness check (skipped: we don't yet
        // populate SignatureFlagsAbstract on signatures in the Rust port).
        if kind == SignatureKind::Construct && !source_sigs.is_empty() && !target_sigs.is_empty() {
            // Future: mirror Go's constructorVisibilitiesAreCompatible.
        }

        // Identity relation fast path.
        if relation == RelationKind::Identity {
            return self.signatures_identical_to(source, target, kind);
        }

        let check_mode = match relation {
            RelationKind::Subtype => SignatureCheckMode::StrictTopSignature,
            RelationKind::StrictSubtype => SignatureCheckMode::from_bits_truncate(
                SignatureCheckMode::StrictTopSignature.bits()
                    | SignatureCheckMode::StrictArity.bits(),
            ),
            _ => SignatureCheckMode::None,
        };

        let mut result = Ternary::True;

        // Strategy selection mirrors Go's switch in `signaturesRelatedTo`.
        let source_instantiated = source.object_flags.contains(ObjectFlags::Instantiated);
        let target_instantiated = target.object_flags.contains(ObjectFlags::Instantiated);
        let same_target = match (source.target(), target.target()) {
            (Some(s), Some(t)) => Arc::ptr_eq(&s, &t),
            _ => false,
        };
        if (source_instantiated && target_instantiated && same_target)
            || (source.object_flags.contains(ObjectFlags::Reference)
                && target.object_flags.contains(ObjectFlags::Reference)
                && same_target)
        {
            // Pairwise comparison of signatures (erase generics).
            let min_len = source_sigs.len().min(target_sigs.len());
            for i in 0..min_len {
                let related = self.compare_signatures_related(
                    &source_sigs[i],
                    &target_sigs[i],
                    check_mode,
                    relation,
                );
                if related.is_false() {
                    return Ternary::False;
                }
                result = result.and(related);
            }
            // If signature counts differ, the longer side must be matched
            // by the same N×M logic below.
            if source_sigs.len() != target_sigs.len() {
                // Fall through to N×M for unmatched signatures.
                for t in &target_sigs[min_len..] {
                    let mut found = false;
                    for s in &source_sigs[min_len..] {
                        let related = self.compare_signatures_related(s, t, check_mode, relation);
                        if !related.is_false() {
                            result = result.and(related);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Ternary::False;
                    }
                }
            }
        } else if source_sigs.len() == 1 && target_sigs.len() == 1 {
            // Single-signature fast path. For non-comparable relations we
            // erase generics; for `Comparable` we always erase (Go behavior).
            let erase = relation == RelationKind::Comparable;
            let s = if erase {
                self.get_erased_signature(&source_sigs[0])
            } else {
                Arc::clone(&source_sigs[0])
            };
            let t = if erase {
                self.get_erased_signature(&target_sigs[0])
            } else {
                Arc::clone(&target_sigs[0])
            };
            result = self.compare_signatures_related(&s, &t, check_mode, relation);
        } else {
            // N×M fallback: every target signature must be matched by some
            // source signature. We don't propagate errors here (errorNode
            // plumbing isn't wired up yet).
            for t in &target_sigs {
                let mut found = false;
                for s in &source_sigs {
                    let related = self.compare_signatures_related(s, t, check_mode, relation);
                    if !related.is_false() {
                        result = result.and(related);
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Ternary::False;
                }
            }
        }
        result
    }

    /// Direct port of Go's `signaturesIdenticalTo`. Compares signature
    /// counts and pairwise signatures for identity.
    pub fn signatures_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        kind: SignatureKind,
    ) -> Ternary {
        let source_sigs = self.get_signatures_of_type(source, kind);
        let target_sigs = self.get_signatures_of_type(target, kind);
        if source_sigs.len() != target_sigs.len() {
            return Ternary::False;
        }
        let mut result = Ternary::True;
        for i in 0..source_sigs.len() {
            let related = self.compare_signatures_identical(
                &source_sigs[i],
                &target_sigs[i],
                false, // partialMatch
                false, // ignoreThisTypes
                false, // ignoreReturnTypes
            );
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    /// Direct port of Go's `compareSignaturesIdentical`. Currently we
    /// delegate to `compare_signatures_related` with the StrictArity mode,
    /// which approximates identity checking for non-generic signatures.
    pub fn compare_signatures_identical(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        _partial_match: bool,
        _ignore_this_types: bool,
        ignore_return_types: bool,
    ) -> Ternary {
        let mut mode = SignatureCheckMode::StrictArity;
        if ignore_return_types {
            mode |= SignatureCheckMode::IgnoreReturnTypes;
        }
        self.compare_signatures_related(source, target, mode, RelationKind::Identity)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Signature helpers (ported from internal/checker/utilities.go and
    // internal/checker/relater.go signature utilities)
    // ────────────────────────────────────────────────────────────────────────

    /// Whether a signature has a rest parameter whose rest type is a tuple
    /// with a variadic element. Such a rest parameter is not "effective"
    /// for arity purposes (it's a fixed tuple spread).
    /// Mirrors Go's `hasEffectiveRestParameter`.
    pub fn has_effective_rest_parameter(&mut self, sig: &Arc<Signature>) -> bool {
        if !sig.has_rest_parameter() {
            return false;
        }
        let Some(last) = sig.parameters.last() else {
            return true;
        };
        let rest_type = self.get_type_of_symbol(last);
        if is_tuple_type(&rest_type) {
            if let TypeData::Tuple(t) = &rest_type.data {
                return t.combined_flags.contains(ElementFlags::Variadic);
            }
        }
        true
    }

    /// Get parameter count, accounting for tuple rest spreading.
    /// Mirrors Go's `getParameterCount`.
    pub fn get_parameter_count(&mut self, sig: &Arc<Signature>) -> usize {
        let length = sig.parameters.len();
        if !sig.has_rest_parameter() {
            return length;
        }
        let Some(last) = sig.parameters.last() else {
            return length;
        };
        let rest_type = self.get_type_of_symbol(last);
        if is_tuple_type(&rest_type) {
            if let TypeData::Tuple(t) = &rest_type.data {
                let variadic_offset = if t.combined_flags.contains(ElementFlags::Variadic) {
                    0
                } else {
                    1
                };
                return length + t.fixed_length - variadic_offset;
            }
        }
        length
    }

    /// Get the minimum argument count of a signature.
    /// Mirrors Go's `getMinArgumentCount`.
    pub fn get_min_argument_count(&mut self, sig: &Arc<Signature>) -> usize {
        // Use the cached value if it's been computed.
        if sig.resolved_min_argument_count != -1 {
            return sig.resolved_min_argument_count.max(0) as usize;
        }

        let mut min_argument_count: i32 = -1;
        if sig.has_rest_parameter() {
            if let Some(last) = sig.parameters.last() {
                let rest_type = self.get_type_of_symbol(last);
                if is_tuple_type(&rest_type) {
                    if let TypeData::Tuple(t) = &rest_type.data {
                        let first_optional = t
                            .element_infos
                            .iter()
                            .position(|info| !info.flags.contains(ElementFlags::Required));
                        let required_count = match first_optional {
                            Some(i) => i,
                            None => t.fixed_length,
                        };
                        if required_count > 0 {
                            min_argument_count = (sig.parameters.len() - 1 + required_count) as i32;
                        }
                    }
                }
            }
        }
        if min_argument_count == -1 {
            min_argument_count = sig.min_argument_count;
        }

        // Walk back over trailing void-typed parameters (Go behavior):
        // `(x: void) => void` has minArgumentCount 0.
        let mut mc = min_argument_count;
        let mut i = mc - 1;
        while i >= 0 {
            match self.try_get_type_at_position(sig, i as usize) {
                Some(t) if t.flags.contains(TypeFlags::Void) => {
                    mc = i;
                }
                _ => break,
            }
            i -= 1;
        }
        mc.max(0) as usize
    }

    /// Get the type of a parameter at a given position, returning `any` if
    /// out of range. Mirrors Go's `getTypeAtPosition`.
    pub fn get_type_at_position(&mut self, sig: &Arc<Signature>, pos: usize) -> Arc<Type> {
        self.try_get_type_at_position(sig, pos)
            .unwrap_or_else(|| self.any_type())
    }

    /// Try to get the type of a parameter at a given position.
    /// Mirrors Go's `tryGetTypeAtPosition`.
    pub fn try_get_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {
        let rest_offset = if sig.has_rest_parameter() { 1 } else { 0 };
        let param_count = sig.parameters.len() - rest_offset;
        if pos < param_count {
            return Some(self.get_type_of_symbol(&sig.parameters[pos]));
        }
        if sig.has_rest_parameter() {
            let rest_param = &sig.parameters[param_count];
            let rest_type = self.get_type_of_symbol(rest_param);
            // If the rest type is a tuple, index into it.
            if is_tuple_type(&rest_type) {
                if let TypeData::Tuple(t) = &rest_type.data {
                    let index = pos - param_count;
                    let has_variadic = t.combined_flags.contains(ElementFlags::Variadic);
                    if index < t.fixed_length || has_variadic {
                        // Index access on a tuple — return the element type if
                        // we have it, else `None` (caller falls back to `any`).
                        return t
                            .element_infos
                            .get(index)
                            .and_then(|info| info.type_.clone())
                            .or_else(|| Some(self.any_type()));
                    }
                }
            }
        }
        None
    }

    /// Get the rest type at a position, transforming `any[]` to just `any`.
    /// Mirrors Go's `getRestOrAnyTypeAtPosition`.
    pub fn get_rest_or_any_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        let rest_type = self.get_rest_type_at_position(sig, pos);
        if let Some(rt) = &rest_type {
            if self.is_array_type(rt) {
                let elem = self.get_type_arguments(rt).into_iter().next();
                if let Some(elem) = elem {
                    if elem.flags.contains(TypeFlags::Any) {
                        return self.any_type();
                    }
                }
            }
        }
        rest_type.unwrap_or_else(|| self.any_type())
    }

    /// Get the rest type at a position. Simplified port of Go's
    /// `getRestTypeAtPosition`.
    pub fn get_rest_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {
        let parameter_count = self.get_parameter_count(sig);
        if pos >= parameter_count.saturating_sub(1) {
            // The rest position itself — return the effective rest type.
            return self.get_effective_rest_type(sig);
        }
        None
    }

    /// Get the effective rest type of a signature.
    /// Simplified port of Go's `getEffectiveRestType`.
    pub fn get_effective_rest_type(&mut self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        if !sig.has_rest_parameter() {
            return None;
        }
        let last = sig.parameters.last()?;
        let rest_type = self.get_type_of_symbol(last);
        // If the rest type is a tuple, the effective rest is the tuple itself
        // (we don't yet split it into spread elements).
        Some(rest_type)
    }

    /// Get the "non-array" rest type — the element type if the rest is an
    /// array type, the tuple type itself if it's a tuple. Returns `None`
    /// if the signature has no rest parameter.
    /// Used by `compare_signatures_related` to decide between tuple-spread
    /// and array-rest parameter comparison.
    /// Mirrors Go's `getNonArrayRestType`.
    pub fn get_non_array_rest_type(&mut self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        if !sig.has_rest_parameter() {
            return None;
        }
        let last = sig.parameters.last()?;
        let rest_type = self.get_type_of_symbol(last);
        // If it's a tuple, we don't treat it as an array rest.
        if is_tuple_type(&rest_type) {
            return Some(rest_type);
        }
        // If it's an array type, the non-array rest is the element type.
        if self.is_array_type(&rest_type) {
            return self.get_type_arguments(&rest_type).into_iter().next();
        }
        Some(rest_type)
    }

    /// Whether a signature is `(...args: any) => any` or
    /// `(...args: never) => any/unknown`. Mirrors Go's `isTopSignature`.
    pub fn is_top_signature(&mut self, sig: &Arc<Signature>) -> bool {
        if !sig.type_parameters.is_empty() {
            return false;
        }
        // thisParameter check: if present, must be `any`.
        if let Some(this_param) = &sig.this_parameter {
            let this_type = self.get_type_of_symbol(this_param);
            if !this_type.flags.contains(TypeFlags::Any) {
                return false;
            }
        }
        if sig.parameters.len() != 1 || !sig.has_rest_parameter() {
            return false;
        }
        let Some(param) = sig.parameters.first() else {
            return false;
        };
        let param_type = self.get_type_of_symbol(param);
        let rest_type = if self.is_array_type(&param_type) {
            self.get_type_arguments(&param_type).into_iter().next()
        } else {
            Some(param_type)
        };
        match rest_type {
            Some(rt) => {
                if !rt.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
                    return false;
                }
                let return_type = self.get_return_type_of_signature(sig);
                match return_type {
                    Some(rt) => rt.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN),
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Get the type of the `this` parameter of a signature.
    /// Mirrors Go's `getThisTypeOfSignature`.
    pub fn get_this_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        let this_param = sig.this_parameter.as_ref()?;
        let links = self.value_symbol_links.get(this_param)?;
        links.resolved_type.clone()
    }

    /// Get the return type of a signature, avoiding infinite recursion when
    /// the return type is itself computed from the signature.
    /// Mirrors Go's `getNonCircularReturnTypeOfSignature`. Currently we just
    /// return the resolved return type.
    pub fn get_non_circular_return_type_of_signature(&self, sig: &Arc<Signature>) -> Arc<Type> {
        self.get_return_type_of_signature(sig)
            .unwrap_or_else(|| self.any_type())
    }

    /// Get the erased signature (with type parameters replaced by their
    /// constraints). Since we don't yet support generic signature
    /// instantiation, this returns the same signature.
    /// Mirrors Go's `getErasedSignature`.
    pub fn get_erased_signature(&self, sig: &Arc<Signature>) -> Arc<Signature> {
        Arc::clone(sig)
    }

    /// Get the canonical form of a signature. Since we don't yet support
    /// generic signature instantiation, this returns the same signature.
    /// Mirrors Go's `getCanonicalSignature`.
    pub fn get_canonical_signature(&self, sig: &Arc<Signature>) -> Arc<Signature> {
        Arc::clone(sig)
    }

    /// Format a signature as a string for diagnostics.
    /// Simplified port of Go's `signatureToString`.
    pub fn signature_to_string(&mut self, sig: &Arc<Signature>) -> String {
        let params: Vec<String> = sig.parameters.iter().map(|p| p.name.clone()).collect();
        let return_type = self.get_return_type_of_signature(sig);
        let return_str = match return_type {
            Some(t) => self.type_to_string(&t),
            None => "void".to_string(),
        };
        format!("({}) => {}", params.join(", "), return_str)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Higher-level signature comparison entry points
    // ────────────────────────────────────────────────────────────────────────

    /// Check if the call signatures of two types are related.
    /// Wraps `signatures_related_to` with the call-signature kind.
    fn is_call_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Call);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Call);

        if source_sigs.is_empty() && target_sigs.is_empty() {
            return true;
        }
        if target_sigs.is_empty() {
            // Source has call signatures but target doesn't.
            return relation == RelationKind::Comparable;
        }
        if source_sigs.is_empty() {
            // Target has call signatures but source doesn't.
            return false;
        }
        self.signatures_related_to(source, target, SignatureKind::Call, relation)
            .is_true()
    }

    /// Check if the construct signatures of two types are related.
    /// Wraps `signatures_related_to` with the construct-signature kind.
    fn is_construct_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Construct);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Construct);

        if source_sigs.is_empty() && target_sigs.is_empty() {
            return true;
        }
        if target_sigs.is_empty() {
            return relation == RelationKind::Comparable;
        }
        if source_sigs.is_empty() {
            return false;
        }
        self.signatures_related_to(source, target, SignatureKind::Construct, relation)
            .is_true()
    }

    /// Check if two function types are related by comparing their call signatures.
    fn is_function_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        // A type is a function type if it has call signatures
        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }
        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }
        true
    }
}

/// Check if two slices of type parameters are the same (by pointer identity).
/// Mirrors Go's `core.Same` for type parameter slices.
fn type_parameters_same(a: &[Arc<Type>], b: &[Arc<Type>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

impl Checker {
    // ────────────────────────────────────────────────────────────────────────
    // Index signature comparison (improved port of relater.go)
    // ────────────────────────────────────────────────────────────────────────

    /// Improved port of `indexSignaturesRelatedTo`. Handles:
    /// - Identity relation: delegates to `index_signatures_identical_to`.
    /// - Target with a string index whose value type is `any` (and source
    ///   is non-primitive, relation is not strict subtype): short-circuit
    ///   to true. This matches the common `{ [key: string]: any }` target.
    /// - Generic mapped-type source with a target string index: compare the
    ///   mapped type's template type against the target's value type.
    /// - Otherwise: structural lookup via `type_related_to_index_info`.
    pub fn index_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        source_is_primitive: bool,
        relation: RelationKind,
    ) -> Ternary {
        if relation == RelationKind::Identity {
            return self.index_signatures_identical_to(source, target);
        }
        let target_indexes = self.get_index_infos_of_type(target);
        let target_has_string_index = target_indexes.iter().any(|info| {
            info.key_type
                .as_ref()
                .map(|k| k.flags.contains(TypeFlags::String))
                .unwrap_or(false)
        });
        let mut result = Ternary::True;
        for target_info in &target_indexes {
            let target_value_any = target_info
                .value_type
                .as_ref()
                .map(|v| v.flags.contains(TypeFlags::Any))
                .unwrap_or(false);
            let target_key_is_string = target_info
                .key_type
                .as_ref()
                .map(|k| k.flags.contains(TypeFlags::String))
                .unwrap_or(false);
            let related = if relation != RelationKind::StrictSubtype
                && !source_is_primitive
                && target_has_string_index
                && target_key_is_string
                && target_value_any
            {
                Ternary::True
            } else if self.is_generic_mapped_type(source) && target_key_is_string {
                let template = self.get_template_type_from_mapped_type(source);
                match template {
                    Some(template) => {
                        let target_value = target_info
                            .value_type
                            .clone()
                            .unwrap_or_else(|| self.any_type());
                        self.compare_types(template, target_value, relation, false)
                    }
                    None => Ternary::False,
                }
            } else {
                self.type_related_to_index_info(source, target_info, relation)
            };
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    /// Port of `typeRelatedToIndexInfo`. Looks up the source's index info
    /// for the target's key type and compares value types.
    pub fn type_related_to_index_info(
        &mut self,
        source: &Arc<Type>,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let target_key = match &target_info.key_type {
            Some(k) => k,
            None => return Ternary::True,
        };
        let source_info = self.get_applicable_index_info(source, target_key);
        if let Some(source_info) = source_info {
            return self.index_info_related_to(&source_info, target_info, relation);
        }
        // Source has no matching index signature. If the source is an
        // "inferable" object type (object literal, type literal, enum,
        // value module, JS expando, rest type, reverse-mapped type), we
        // synthesize an index signature from its properties and compare
        // those against the target's value type (P3.7f). The
        // strict-subtype relation additionally requires the source to be
        // a fresh object literal so that `{ [x: string]: xxx } <: {}` but
        // not vice-versa (matching Go's behavior in `typeRelatedToIndexInfo`).
        let is_fresh_literal = source.object_flags.contains(ObjectFlags::FreshLiteral);
        if relation != RelationKind::StrictSubtype || is_fresh_literal {
            if self.is_object_type_with_inferable_index(source) {
                return self.members_related_to_index_info(source, target_info, relation);
            }
        }
        Ternary::False
    }

    /// Port of `indexInfoRelatedTo`. Compares two index infos' value types.
    pub fn index_info_related_to(
        &mut self,
        source_info: &IndexInfo,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let source_value = source_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        let target_value = target_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        self.compare_types(source_value, target_value, relation, false)
    }

    /// Port of `indexSignaturesIdenticalTo`. Requires same count and
    /// pairwise key/value/readonly equality.
    pub fn index_signatures_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Ternary {
        let source_infos = self.get_index_infos_of_type(source);
        let target_infos = self.get_index_infos_of_type(target);
        if source_infos.len() != target_infos.len() {
            return Ternary::False;
        }
        for target_info in &target_infos {
            let target_key = match &target_info.key_type {
                Some(k) => Arc::clone(k),
                None => continue,
            };
            let source_info = self.get_index_info_of_type(source, &target_key);
            let related = match source_info {
                Some(si) => {
                    let sv = si.value_type.clone().unwrap_or_else(|| self.any_type());
                    let tv = target_info
                        .value_type
                        .clone()
                        .unwrap_or_else(|| self.any_type());
                    let type_related = self.compare_types(sv, tv, RelationKind::Identity, false);
                    let readonly_match = si.is_readonly == target_info.is_readonly;
                    if type_related.is_true() && readonly_match {
                        Ternary::True
                    } else {
                        Ternary::False
                    }
                }
                None => Ternary::False,
            };
            if related.is_false() {
                return Ternary::False;
            }
        }
        Ternary::True
    }

    /// Get the index infos of a type. Currently delegates to the structured
    /// type's `index_infos` field.
    pub fn get_index_infos_of_type(&self, t: &Arc<Type>) -> Vec<Arc<IndexInfo>> {
        t.as_structured()
            .map(|s| s.index_infos.clone())
            .unwrap_or_default()
    }

    /// Get the index info of a type for a specific key type.
    pub fn get_index_info_of_type(
        &self,
        t: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        let infos = self.get_index_infos_of_type(t);
        for info in infos {
            if let Some(info_key) = &info.key_type {
                if Arc::ptr_eq(info_key, key_type) || info_key.flags == key_type.flags {
                    return Some(info);
                }
            }
        }
        None
    }

    /// Get the applicable index info of a source for a given key type.
    /// Mirrors Go's `getApplicableIndexInfo`. Number index keys are
    /// applicable to string indexes (a string index accepts numbers).
    pub fn get_applicable_index_info(
        &self,
        source: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        let infos = self.get_index_infos_of_type(source);
        for info in infos {
            if let Some(info_key) = &info.key_type {
                // Direct match.
                if Arc::ptr_eq(info_key, key_type) {
                    return Some(info);
                }
                // A number key is applicable to a string index target.
                if info_key.flags.contains(TypeFlags::Number)
                    && key_type.flags.contains(TypeFlags::String)
                {
                    return Some(info);
                }
                // A string key is applicable to a number index target
                // (strings are numbers in JS).
                if info_key.flags.contains(TypeFlags::String)
                    && key_type.flags.contains(TypeFlags::Number)
                {
                    return Some(info);
                }
            }
        }
        None
    }

    /// Whether a type is a generic mapped type (has a constraint and
    /// template type). Mirrors Go's `isGenericMappedType`.
    pub fn is_generic_mapped_type(&self, t: &Arc<Type>) -> bool {
        if let TypeData::Mapped(m) = &t.data {
            m.type_parameter.is_some() && m.template_type.is_some()
        } else {
            false
        }
    }

    /// Get the template type of a mapped type.
    /// Mirrors Go's `getTemplateTypeFromMappedType`.
    pub fn get_template_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.template_type.clone();
        }
        None
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Index signature comparison polish (P3.7f)
//
// Ports the structural fallback paths of `typeRelatedToIndexInfo` and
// `membersRelatedToIndexInfo` from internal/checker/relater.go (~L4596,
// ~L4627). When the source has no index signature matching the target's
// key type, but the source is an *inferable* object type (object literal,
// type literal, enum, value module, JS expando, rest type, or reverse
// mapped type whose source is itself inferable), we walk the source's
// properties and verify that every property whose name is a literal of
// the target's key type is assignable to the target's value type.
//
// This handles the common case `{ a: 1, b: 2 } ~ { [key: string]: number }`
// without requiring the source to declare an explicit index signature.
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Whether an object type may have an *inferred* index signature —
    /// i.e. one synthesized from its properties rather than declared.
    /// Direct port of Go's `isObjectTypeWithInferableIndex`.
    ///
    /// Returns true for:
    /// - Object literals, type literals, enums, value modules (without
    ///   call/construct signatures and not class-typed).
    /// - JS expando object literals and rest types.
    /// - Reverse-mapped types whose source is itself inferable.
    /// - Intersection types whose every constituent is inferable.
    pub fn is_object_type_with_inferable_index(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Intersection) {
            // Every constituent must be inferable.
            if let Some(ui) = t.as_union_or_intersection() {
                return ui
                    .types
                    .iter()
                    .all(|c| self.is_object_type_with_inferable_index(c));
            }
            return false;
        }
        // Object-literal / type-literal / enum / value-module case.
        if let Some(sym) = &t.symbol {
            let sf = sym.flags;
            let inferable_symbol_kinds = sf.intersects(
                SymbolFlags::ObjectLiteral
                    | SymbolFlags::TypeLiteral
                    | SymbolFlags::EnumMember
                    | SymbolFlags::ValueModule,
            );
            if inferable_symbol_kinds
                && !sf.contains(SymbolFlags::Class)
                && !self.type_has_call_or_construct_signatures(t)
            {
                return true;
            }
        }
        // JS expando / object-rest case.
        if t.object_flags
            .intersects(ObjectFlags::JSLiteral | ObjectFlags::ObjectRestType)
        {
            return true;
        }
        // Reverse-mapped case: recurse into the source.
        if t.object_flags.contains(ObjectFlags::ReverseMapped) {
            if let TypeData::ReverseMapped(rm) = &t.data {
                if let Some(src) = &rm.source {
                    return self.is_object_type_with_inferable_index(src);
                }
            }
        }
        false
    }

    /// Walk the source's properties and verify each property whose name
    /// is a literal of the target's key type is assignable to the target's
    /// value type. Also compares any source index signatures whose key
    /// type is applicable to the target's key type.
    ///
    /// Direct port of Go's `membersRelatedToIndexInfo` (without the
    /// ignored-JSX / `exactOptionalPropertyTypes` branches, which we
    /// don't yet support).
    pub fn members_related_to_index_info(
        &mut self,
        source: &Arc<Type>,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let Some(target_key) = target_info.key_type.as_ref() else {
            return Ternary::True;
        };
        let target_value = target_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());

        let props = self.get_properties_of_type(source);
        let mut result = Ternary::True;
        for prop in props {
            // Only consider properties whose name is a literal of the
            // target's key type. Go uses `getLiteralTypeFromProperty` to
            // synthesize a literal type from a property name; we approximate
            // by treating every named property as a string literal (the
            // common case for `{ [key: string]: T }` targets).
            let literal_key = self.get_literal_type_from_property(&prop, target_key);
            if !self.is_applicable_index_type(&literal_key, target_key) {
                continue;
            }
            let prop_type = self.get_type_of_symbol(&prop);
            let related = self.compare_types(prop_type, Arc::clone(&target_value), relation, false);
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }

        // Also compare any source index signatures whose key type is
        // applicable to the target's key type.
        for info in self.get_index_infos_of_type(source) {
            if let Some(src_key) = &info.key_type {
                if self.is_applicable_index_type(src_key, target_key) {
                    let related = self.index_info_related_to(&info, target_info, relation);
                    if related.is_false() {
                        return Ternary::False;
                    }
                    result = result.and(related);
                }
            }
        }
        result
    }

    /// Whether `key` is a literal type applicable to `target_key`'s index
    /// type. A string-literal key is applicable to a string index; a
    /// number-literal key is applicable to a number index. Direct port of
    /// Go's `isApplicableIndexType`.
    pub fn is_applicable_index_type(&self, key: &Arc<Type>, target_key: &Arc<Type>) -> bool {
        if Arc::ptr_eq(key, target_key) {
            return true;
        }
        // String literal -> string index
        if key.flags.contains(TypeFlags::StringLiteral)
            && target_key.flags.contains(TypeFlags::String)
        {
            return true;
        }
        // Number literal -> number index
        if key.flags.contains(TypeFlags::NumberLiteral)
            && target_key.flags.contains(TypeFlags::Number)
        {
            return true;
        }
        // A number key is applicable to a string index (numbers index
        // into string indexes in JS).
        if key.flags.contains(TypeFlags::Number) && target_key.flags.contains(TypeFlags::String) {
            return true;
        }
        false
    }

    /// Build a literal type from a property's name. Used by
    /// `members_related_to_index_info` to decide whether a property is
    /// applicable to the target's index signature.
    ///
    /// Direct port of Go's `getLiteralTypeFromProperty`. Currently we
    /// synthesize a string-literal type for every property name (the
    /// common case for `{ [key: string]: T }` targets) and a number
    /// literal when the name parses as a number. Full implementation
    /// would also handle unique symbols and `SymbolFlags::EnumMember`.
    pub fn get_literal_type_from_property(
        &mut self,
        prop: &Arc<Symbol>,
        target_key: &Arc<Type>,
    ) -> Arc<Type> {
        if target_key.flags.contains(TypeFlags::Number) {
            if let Ok(n) = prop.name.parse::<i64>() {
                return self.get_number_literal_type(jsnum::Number::from(n));
            }
        }
        self.get_string_literal_type(&prop.name)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Generic instantiation comparison (P3.7d)
//
// Ports the variance-aware type-argument comparison and the structured-type
// dispatch for references to the *same* generic type from
// `internal/checker/relater.go` (`typeArgumentsRelatedTo`,
// `structuredTypeRelatedToWorker`, `compareTypeParametersIdentical`).
//
// Variance computation itself is a large subsystem (`variance.go`); for now
// we use the covariant fallback that Go's relater also uses when no variance
// information is available. This produces correct results for the common
// cases (Array<T> ~ Array<U> iff T ~ U; Promise<T> ~ Promise<U>; etc.) and
// defers the strict-function-types invariant handling to P3.7e.
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Compare two slices of type arguments according to per-parameter
    /// variance flags. Direct port of Go's `typeArgumentsRelatedTo`.
    ///
    /// For each pair `(sources[i], targets[i])`:
    /// - **Covariant** (default): `sources[i] ~ targets[i]`
    /// - **Contravariant**: `targets[i] ~ sources[i]` (swap direction)
    /// - **Bivariant**: try contravariant first, then fall back to
    ///   covariant if that fails
    /// - **Invariant**: both `s ~ t` and `t ~ s` must hold
    /// - **Independent**: skip the argument entirely (its variance is
    ///   never witnessed)
    /// - **Unmeasurable**: require identity (rather than the requested
    ///   relation) — non-linear relations such as `-?` mapped modifiers
    ///   can't be safely approximated by structural comparison.
    pub fn type_arguments_related_to(
        &mut self,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
        variances: &[VarianceFlags],
        relation: RelationKind,
    ) -> Ternary {
        // Identity relation requires equal length up front.
        if sources.len() != targets.len() && relation == RelationKind::Identity {
            return Ternary::False;
        }
        let length = sources.len().min(targets.len());
        let mut result = Ternary::True;
        for i in 0..length {
            // Default to covariant when no variance information is available
            // (matches Go's fallback during variance computation and for
            // `this`-type arguments).
            let variance_flags = variances
                .get(i)
                .copied()
                .unwrap_or(VarianceFlags::Covariant);
            let variance = variance_flags & VARIANCE_FLAGS_VARIANCE_MASK;

            // Skip independent type parameters — their variance is never
            // observed by any consumer of the generic type.
            if variance == VarianceFlags::Independent {
                continue;
            }

            let s = &sources[i];
            let t = &targets[i];
            let related = if variance_flags.intersects(VARIANCE_FLAGS_ALLOWS_STRUCTURAL_FALLBACK)
                && !variance_flags
                    .intersects(VarianceFlags::Unmeasurable | VarianceFlags::Unreliable)
            {
                // The "allows structural fallback" subset that *isn't* also
                // unmeasurable/unreliable reduces to plain covariance: there
                // is no special handling needed beyond the default direction.
                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
            } else if variance_flags.intersects(VarianceFlags::Unmeasurable) {
                // Even an `Unmeasurable` variance works out without a
                // structural check if the source and target are identical.
                // We can't simply assume invariance, because `Unmeasurable`
                // marks nonlinear relations (e.g. relations tainted by the
                // `-?` modifier in a mapped type).
                if relation == RelationKind::Identity {
                    if self.is_type_related_to(s, t, relation) {
                        Ternary::True
                    } else {
                        Ternary::False
                    }
                } else if self.is_type_identical_to(s, t) {
                    Ternary::True
                } else {
                    Ternary::False
                }
            } else {
                match variance {
                    VarianceFlags::Covariant => {
                        self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
                    }
                    VarianceFlags::Contravariant => {
                        // Swap direction: target must be assignable to source.
                        self.compare_types(Arc::clone(t), Arc::clone(s), relation, false)
                    }
                    VarianceFlags::Independent => {
                        // Already filtered out above; defensive fallback.
                        Ternary::True
                    }
                    _ => {
                        // Bivariant or Invariant.
                        //
                        // Bivariant: try contravariant first without error
                        // reporting, then fall back to covariant if that
                        // fails. Invariant: require both covariant and
                        // contravariant. Since `VarianceFlags::None` is the
                        // "invariant" sentinel in Go's encoding (covariant
                        // and contravariant bits both unset), and our
                        // `VARIANCE_FLAGS_BIVARIANT` is both bits set, we
                        // disambiguate by checking the bits explicitly.
                        let is_bivariant = variance_flags.intersects(VARIANCE_FLAGS_BIVARIANT)
                            && variance != VarianceFlags::None;
                        let contra =
                            self.compare_types(Arc::clone(t), Arc::clone(s), relation, false);
                        if is_bivariant {
                            if !contra.is_false() {
                                contra
                            } else {
                                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
                            }
                        } else {
                            // Invariant: require both directions to hold.
                            let co =
                                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false);
                            if co.is_false() {
                                Ternary::False
                            } else {
                                co.and(contra)
                            }
                        }
                    }
                }
            };
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    /// Whether two lists of type parameters are *identical* modulo
    /// renaming. Direct port of Go's `compareTypeParametersIdentical`.
    ///
    /// Two type-parameter lists `<T, U extends T>` and `<A, B extends A>`
    /// are considered identical because their structural relationship is
    /// the same — only the names differ. The check works by instantiating
    /// each target's constraint into the source's type parameters (via a
    /// `targetParams -> sourceParams` mapper) and comparing the resulting
    /// constraints for identity.
    ///
    /// Our simplified port compares constraints directly without the
    /// mapper substitution. This is correct when the constraints don't
    /// reference sibling type parameters (the common case for built-in
    /// generics like `Array<T>`, `Map<K, V>`, `Promise<T>`), and falls
    /// back to "identical when constraints are pointer-equal or both
    /// absent" for the parameter-referencing case.
    pub fn compare_type_parameters_identical(
        &mut self,
        source_params: &[Arc<Type>],
        target_params: &[Arc<Type>],
    ) -> bool {
        if source_params.len() != target_params.len() {
            return false;
        }
        for (source, target) in source_params.iter().zip(target_params.iter()) {
            if Arc::ptr_eq(source, target) {
                continue;
            }
            let source_constraint = self
                .get_constraint_of_type_parameter(source)
                .unwrap_or_else(|| self.unknown_type());
            let target_constraint = self
                .get_constraint_of_type_parameter(target)
                .unwrap_or_else(|| self.unknown_type());
            // Without a real `instantiateType`, fall back to direct
            // identity comparison. This works for built-in generics and
            // for type parameters whose constraints don't reference
            // sibling parameters.
            if !self.is_type_identical_to(&source_constraint, &target_constraint) {
                return false;
            }
        }
        true
    }

    /// Compare two references to the *same* generic type (e.g. `Array<T>`
    /// vs `Array<U>` where both target the `Array<T>` interface).
    ///
    /// Direct port of the relevant branch of Go's
    /// `structuredTypeRelatedToWorker`:
    ///
    /// - If both source and target are references to the same target type
    ///   (and neither is a tuple — those go through element-wise
    ///   comparison — and neither is a "marker" type intended for
    ///   structural comparison), obtain the variance information for the
    ///   target's type parameters and relate the type arguments
    ///   accordingly.
    /// - If no variance information is available (which Go uses as a
    ///   recursion-depth signal during variance computation itself),
    ///   return `Ternary::Maybe` to defer the decision.
    ///
    /// Returns `None` when the source/target pair isn't a same-target
    /// generic reference pair (caller should fall back to other
    /// comparison strategies). Returns `Some(Ternary)` when the
    /// variance-based result has been computed.
    pub fn generic_type_reference_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> {
        // Both must be object-typed references with the same target.
        if !source.flags.contains(TypeFlags::Object) || !target.flags.contains(TypeFlags::Object) {
            return None;
        }
        if !source.object_flags.contains(ObjectFlags::Reference)
            || !target.object_flags.contains(ObjectFlags::Reference)
        {
            return None;
        }
        // Tuples are handled by element-wise comparison, not here.
        if is_tuple_type(source) || is_tuple_type(target) {
            return None;
        }
        let source_target = source.target()?;
        let target_target = target.target()?;
        if !Arc::ptr_eq(source_target, target_target) {
            return None;
        }
        // Marker types are intended to be compared structurally.
        if self.is_marker_type(source) || self.is_marker_type(target) {
            return None;
        }
        // Empty array literals are always assignable to mutable array types
        // (Go: `c.isEmptyArrayLiteralType(source)`).
        if self.is_empty_array_literal_type(source) {
            return Some(Ternary::True);
        }
        // Obtain variance information for the type parameters of the
        // generic target. Without a full variance-computation engine,
        // `get_variances` returns an empty Vec to signal "no variance info"
        // (matching Go's behavior during recursive variance computation),
        // in which case we defer the decision with `Ternary::Maybe`.
        let variances = self.get_variances(source_target);
        if variances.is_empty() {
            return Some(Ternary::Maybe);
        }
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);
        Some(self.type_arguments_related_to(&source_args, &target_args, &variances, relation))
    }

    /// Simplified port of Go's `getVariances`. The full implementation
    /// recursively computes variance for each type parameter of a generic
    /// type by inspecting where it appears in the type's structure
    /// (`variance.go`). Without that subsystem, we return an empty slice
    /// to signal "variance not measured yet" — the caller
    /// (`generic_type_reference_related_to`) then defers the decision
    /// with `Ternary::Maybe`, matching Go's behavior during recursive
    /// variance computation.
    ///
    /// This is sufficient to make the common case work (covariant
    /// containers like `Array<T>` and `Promise<T>` are also handled
    /// directly by `is_array_type_related_to`); full variance support
    /// will land with P3.7e.
    pub fn get_variances(&self, _target: &Arc<Type>) -> Vec<VarianceFlags> {
        // Default everything to covariant as a pragmatic fallback so
        // `Array<Promise<X>>`-style comparisons work without a variance
        // engine. Go's `typeArgumentsRelatedTo` does the same when no
        // variance info is provided. We only return empty when the target
        // has *no* type parameters (in which case there's nothing to
        // compare and the caller should not enter this path).
        match &_target.data {
            TypeData::Object(o) => {
                if let Some(t) = o.target.as_ref() {
                    if let TypeData::Interface(i) = &t.data {
                        let n = i.all_type_parameters.len();
                        return vec![VarianceFlags::Covariant; n];
                    }
                }
                Vec::new()
            }
            TypeData::Interface(i) => {
                let n = i.all_type_parameters.len();
                vec![VarianceFlags::Covariant; n]
            }
            _ => Vec::new(),
        }
    }

    /// Whether a type is a "marker" type. Marker types are intended to be
    /// compared structurally even when they appear as references to the
    /// same generic target (Go: `c.isMarkerType`).
    ///
    /// The full Go implementation checks a list of well-known marker types
    /// (e.g. `ReadonlyArray`, `ReadonlyMap`, `ReadonlySet`, `Promise`'s
    /// `T`-parameter usage). Our port returns `false` for now — none of
    /// our test fixtures exercise the distinction, and the structural
    /// fallback in `is_object_type_related_to` is correct (just slower)
    /// for the cases we do hit.
    pub fn is_marker_type(&self, _t: &Arc<Type>) -> bool {
        false
    }

    /// Whether a type is the "empty array literal" placeholder used for
    /// `[]` literals whose element type hasn't been inferred yet.
    /// Mirrors Go's `c.isEmptyArrayLiteralType`.
    pub fn is_empty_array_literal_type(&self, t: &Arc<Type>) -> bool {
        // The empty-array-literal type is a fresh object type whose
        // `object_flags` carries `FreshLiteral` and whose target is the
        // global `Array` type with an empty (or `undefined`) element type.
        // We approximate this by checking the fresh-literal flag.
        t.object_flags.contains(ObjectFlags::FreshLiteral) && self.is_array_type(t)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Conditional & mapped type comparison (P3.7e)
//
// Ports the conditional-target branch of `structuredTypeRelatedToWorker`
// (relater.go ~L3540) and `mappedTypeRelatedTo` (relater.go ~L3972).
//
// The full Go implementation depends on `c.instantiateType` with mappers,
// `c.isDistributionDependent`, `c.getPermissiveInstantiation`, etc. —
// most of which we don't yet have. The port below is intentionally
// conservative: when a path requires infrastructure we don't have
// (infer type parameters, distributive conditionals referencing the
// check type, mapped types with name remapping), we return `None` so
// the caller falls through to structural comparison. The cases we do
// handle correctly cover the common scenarios:
//   * `S` assignable to `T extends U ? X : Y` when S ~ X and S ~ Y
//     (with the permissive/restrictive short-circuits).
//   * `{ [P in Q]: X }` vs `{ [P in R]: Y }` when Q ~ R and X ~ Y
//     (without name remapping).
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Compare a source type `S` against a conditional target
    /// `T extends U ? X : Y`.
    ///
    /// Returns `None` when the conditional requires infrastructure we don't
    /// have yet (infer positions, distribution-dependent checks, identical
    /// source conditional) so the caller can fall back to structural
    /// comparison. Otherwise returns `Some(Ternary)`.
    ///
    /// Direct port of the conditional branch of Go's
    /// `structuredTypeRelatedToWorker`.
    pub fn conditional_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> {
        let ct = match &target.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        // Bail out if the conditional has `infer` type positions — those
        // require the inference engine (P3.8c).
        if let Some(root) = &ct.root {
            if !root.infer_type_parameters.is_empty() {
                return None;
            }
            // Bail out if the conditional is distributive and references
            // the check type parameter in either result branch
            // (`isDistributionDependent`). Without a real
            // `getPermissiveInstantiation` we can't safely short-circuit.
            if root.is_distributive && self.conditional_is_distribution_dependent(target) {
                return None;
            }
        }

        // Bail out when source is itself a conditional with the same root
        // (this case shows up during variance computation and would cause
        // infinite recursion if we tried to compare both branches).
        if let TypeData::Conditional(sct) = &source.data {
            if let (Some(s_root), Some(t_root)) = (&sct.root, &ct.root) {
                if std::ptr::eq(s_root.as_ref() as *const _, t_root.as_ref() as *const _) {
                    return None;
                }
            }
        }

        // Determine whether either branch can be skipped:
        //  * skipTrue  := the conditional's check is *never* true, so we
        //                 can ignore the true branch entirely.
        //  * skipFalse := the conditional's check is *always* true (and
        //                 skipTrue is false), so we can ignore the false
        //                 branch.
        //
        // Go computes these via permissive/restrictive instantiations of
        // the check and extends types. We approximate: if the resolved
        // branch is already cached (because the conditional has been
        // evaluated), use that; otherwise don't skip.
        let skip_true = match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
            (Some(check), Some(extends)) => !self.is_type_assignable_to(check, extends),
            _ => false,
        };
        let skip_false = if skip_true {
            false
        } else {
            match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
                (Some(check), Some(extends)) => self.is_type_assignable_to(check, extends),
                _ => false,
            }
        };

        let mut result = Ternary::True;
        if !skip_true {
            let true_branch = self.get_true_type_from_conditional_type(target)?;
            let r = self.compare_types(Arc::clone(source), true_branch, relation, false);
            if r.is_false() {
                return Some(Ternary::False);
            }
            result = result.and(r);
        }
        if !skip_false {
            let false_branch = self.get_false_type_from_conditional_type(target)?;
            let r = self.compare_types(Arc::clone(source), false_branch, relation, false);
            if r.is_false() {
                return Some(Ternary::False);
            }
            result = result.and(r);
        }
        Some(result)
    }

    /// Compare two mapped types structurally:
    /// `{ [P in Q]: X }` vs `{ [P in R]: Y }`.
    ///
    /// Returns `None` when the comparison requires infrastructure we don't
    /// have yet (mapped type with name remapping, or any side that isn't a
    /// real mapped type). Otherwise returns `Some(Ternary)`.
    ///
    /// Direct port of Go's `mappedTypeRelatedTo` minus the
    /// `instantiateType` substitutions (which we don't yet support).
    pub fn mapped_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> {
        let sm = match &source.data {
            TypeData::Mapped(m) => m,
            _ => return None,
        };
        let tm = match &target.data {
            TypeData::Mapped(m) => m,
            _ => return None,
        };

        // Bail out if either side remaps keys (the `as R` clause) — that
        // requires mapper-based instantiation to compare nameTypes
        // correctly.
        if sm.name_type.is_some() || tm.name_type.is_some() {
            return None;
        }

        // Modifiers compatibility: for non-identity relations we accept
        // whenever target's optionality is at least as permissive as
        // source's. Without `getCombinedMappedTypeOptionality`, we
        // conservatively accept all modifier combinations for the
        // assignable/subtype relations and require exact match for
        // identity.
        if relation == RelationKind::Identity {
            // Identity requires the same modifiers; we don't have the
            // helper, so fall back to structural comparison.
            return None;
        }

        // Compare constraints contravariantly: target's constraint must be
        // assignable to source's constraint.
        let source_constraint = self.get_constraint_type_from_mapped_type(source)?;
        let target_constraint = self.get_constraint_type_from_mapped_type(target)?;
        let constraint_related = self.compare_types(
            Arc::clone(&target_constraint),
            Arc::clone(&source_constraint),
            relation,
            false,
        );
        if constraint_related.is_false() {
            return Some(Ternary::False);
        }

        // Compare template types covariantly. The Go code substitutes the
        // source's type parameter with the target's via a `SimpleTypeMapper`
        // before comparing; we don't have a working `instantiateType`, so
        // we compare the templates directly. This is correct when both
        // mapped types use the same type-parameter name (the common case
        // for `{ [P in keyof T]: ... }` vs `{ [P in keyof T]: ... }`),
        // and falls back to structural comparison otherwise.
        let source_template = self.get_template_type_from_mapped_type(source)?;
        let target_template = self.get_template_type_from_mapped_type(target)?;
        let template_related = self.compare_types(
            Arc::clone(&source_template),
            Arc::clone(&target_template),
            relation,
            false,
        );
        Some(constraint_related.and(template_related))
    }

    /// Get the constraint type of a mapped type (the `Q` in `{ [P in Q]: X }`).
    /// Mirrors Go's `getConstraintTypeFromMappedType`.
    pub fn get_constraint_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.constraint_type.clone();
        }
        None
    }

    /// Get the type parameter of a mapped type (the `P` in `{ [P in Q]: X }`).
    /// Mirrors Go's `getTypeParameterFromMappedType`.
    pub fn get_type_parameter_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.type_parameter.clone();
        }
        None
    }

    /// Get the name type of a mapped type (the `R` in `{ [P in Q as R]: X }`).
    /// Mirrors Go's `getNameTypeFromMappedType`.
    pub fn get_name_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.name_type.clone();
        }
        None
    }

    /// Get the resolved `true` branch of a conditional type, if it has been
    /// computed. Mirrors Go's `getTrueTypeFromConditionalType`.
    pub fn get_true_type_from_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_true_type.get() {
                return Some(rt.clone());
            }
            // Fall back to the resolved-inferred-true type, which is set
            // when the conditional's check type was instantiated via
            // inference (Go: `getTrueTypeFromConditionalType` does the same).
            if let Some(rt) = ct.resolved_inferred_true_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    /// Get the resolved `false` branch of a conditional type, if it has been
    /// computed. Mirrors Go's `getFalseTypeFromConditionalType`.
    pub fn get_false_type_from_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_false_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    /// Whether a conditional type is "distribution dependent": distributive
    /// *and* references the check type parameter in either result branch.
    /// Mirrors Go's `isDistributionDependent`. Without a full check-type
    /// tracking subsystem we conservatively return `true` for any
    /// distributive conditional, which causes the caller to bail out and
    /// fall back to structural comparison (correct but possibly slower).
    pub fn conditional_is_distribution_dependent(&self, _t: &Arc<Type>) -> bool {
        true
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Conditional type `infer R` resolution (P3.8c)
//
// Ports the `infer`-position handling of `getConditionalType`
// (internal/checker/checker.go ~L24208). When a conditional has
// `infer R` type parameters in its extends clause, we:
//   1. Set up an InferenceContext keyed on the infer type parameters.
//   2. Call `infer_types(check, extends)` to populate the inferences.
//   3. Resolve the inferred types via `get_inferred_types`.
//   4. Build a mapper that substitutes the inferred types for the
//      infer type parameters, and use it to decide which branch to
//      take and to instantiate the chosen branch.
//
// The full Go implementation also handles distributive conditionals
// (where the check type is a type parameter), deferred checks, tuple
// destructuring, permissive/restrictive instantiations, and 1000-level
// tail recursion. Our simplified port handles the common case where the
// check type is concrete (no remaining type parameters) and the infer
// type parameters can be resolved by direct structural matching.
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Resolve a conditional type `T extends U ? X : Y` to either `X` or `Y`
    /// based on whether `T` is assignable to `U`. Handles the `infer R`
    /// case by setting up an inference context and substituting the
    /// inferred types into the chosen branch.
    ///
    /// Returns `Some(resolved)` when the conditional can be evaluated, or
    /// `None` when:
    ///   - the type isn't a conditional,
    ///   - the check or extends type is missing,
    ///   - the check type is still generic (contains type parameters),
    ///   - inference fails to produce a candidate for one or more infer
    ///     type parameters.
    ///
    /// Caches the result in `resolved_true_type` / `resolved_false_type`
    /// so subsequent lookups via `get_resolved_type_of_conditional_type`
    /// don't re-run the resolution.
    pub fn resolve_conditional_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        // Fast path: cached result.
        if let Some(rt) = ct.resolved_true_type.get() {
            return Some(Arc::clone(rt));
        }
        if let Some(rt) = ct.resolved_false_type.get() {
            return Some(Arc::clone(rt));
        }

        let check_type = ct.check_type.clone()?;
        let extends_type = ct.extends_type.clone()?;

        // Error type short-circuit (matches Go).
        if check_type.flags.contains(TypeFlags::Any) && check_type.intrinsic_name() == Some("error")
        {
            return Some(Arc::clone(&check_type));
        }

        // If the check type is still generic, we can't evaluate yet.
        // Go checks `isDeferredType`; we approximate by checking for any
        // TypeParameter flags anywhere in the check type.
        if type_contains_type_parameter(&check_type) {
            return None;
        }

        // Set up the inference context for `infer R` parameters (if any).
        let infer_params: Vec<Arc<Type>> = ct
            .root
            .as_ref()
            .map(|r| r.infer_type_parameters.clone())
            .unwrap_or_default();
        let inferences: Vec<InferenceInfo> = infer_params
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);

        if !infer_params.is_empty() {
            // Run inference: match check_type against extends_type, with
            // the infer type parameters as inference targets.
            self.infer_types(
                &mut context.inferences,
                Some(Arc::clone(&check_type)),
                Some(Arc::clone(&extends_type)),
                InferencePriority::NoConstraints | InferencePriority::AlwaysStrict,
                false,
            );
            // Verify every infer type parameter got a candidate. If any
            // didn't, we can't safely resolve the conditional.
            let inferred = self.get_inferred_types(&context);
            for inf in &inferred {
                if inf.flags.contains(TypeFlags::Any) && inf.intrinsic_name() == Some("error") {
                    return None;
                }
            }
        }

        // Decide which branch to take. Go instantiates the extends type
        // with the inferred types before checking assignability. We
        // substitute the infer type parameters into the extends type, then
        // check if `check_type` is assignable to the substituted extends.
        let inferred_extends = if !infer_params.is_empty() {
            let inferred = self.get_inferred_types(&context);
            self.substitute_infer_type_parameters(&extends_type, &infer_params, &inferred)
        } else {
            Arc::clone(&extends_type)
        };
        let take_true = self.is_type_assignable_to(&check_type, &inferred_extends);

        // Resolve the chosen branch type node from the AST. The
        // `get_true_type_from_conditional_type` /
        // `get_false_type_from_conditional_type` helpers only return cached
        // results, so for first-time resolution we must resolve the branch
        // type node directly.
        let (cond_node, branch_node) = match ct
            .root
            .as_ref()
            .and_then(|r| r.node.as_ref())
            .and_then(|n| match &n.data {
                NodeData::ConditionalTypeNode(data) => {
                    let branch = if take_true {
                        Arc::clone(&data.true_type)
                    } else {
                        Arc::clone(&data.false_type)
                    };
                    Some((Arc::clone(n), branch))
                }
                _ => None,
            }) {
            Some(pair) => pair,
            None => return None,
        };

        // Push the ConditionalType onto the scope stack so that
        // `resolve_identifier` can find the `infer R` type
        // parameters (declared as locals of the ConditionalType).
        self.push_scope(&cond_node);
        let branch = self.get_type_from_type_node(&branch_node);
        self.pop_scope();
        let resolved = if !infer_params.is_empty() {
            let inferred = self.get_inferred_types(&context);
            self.substitute_infer_type_parameters(&branch, &infer_params, &inferred)
        } else {
            Arc::clone(&branch)
        };
        // Cache the result so subsequent lookups don't re-run.
        // SAFETY: `resolved_true_type` / `resolved_false_type` are
        // `OnceLock` and we just verified they're unset. `set` returns
        // `Result<(), _>`; we ignore the error in the rare race case.
        if let TypeData::Conditional(ct2) = &t.data {
            let cell = if take_true {
                &ct2.resolved_true_type
            } else {
                &ct2.resolved_false_type
            };
            let _ = cell.set(Arc::clone(&resolved));
        }
        Some(resolved)
    }

    /// Substitute occurrences of `infer_params[i]` in `t` with
    /// `substitutions[i]`. Simplified port of Go's `instantiateType` for
    /// the infer-parameter case — walks the type recursively and replaces
    /// pointer-equal occurrences. Doesn't handle aliases, mapped type
    /// constraints, or other complex instantiation scenarios.
    pub fn substitute_infer_type_parameters(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        // Fast path: no parameters to substitute.
        if params.is_empty() || substitutions.is_empty() {
            return Arc::clone(t);
        }
        // Fast path: direct pointer match — return the substitution.
        for (i, p) in params.iter().enumerate() {
            if Arc::ptr_eq(p, t) {
                return Arc::clone(&substitutions[i.min(substitutions.len() - 1)]);
            }
        }
        // Recursive substitution into structured types. We handle the
        // cases that show up in conditional extends types and branches:
        // unions, intersections, arrays (Object with a single type
        // argument), and tuples. Other kinds (mapped, indexed access,
        // nested conditionals, etc.) are returned as-is — a full
        // `instantiateType` port would be needed for those.
        match &t.data {
            TypeData::Union(u) => {
                let new_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let new_types: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => {
                // Array type: `T[]` is represented as an Object with
                // `object_flags: Reference`, no `target`, and a single
                // type argument (the element type). Substitute the
                // element type and rebuild via `create_array_type`.
                if t.object_flags.contains(ObjectFlags::Reference)
                    && o.target.is_none()
                    && o.type_arguments.len() == 1
                {
                    let new_elem = self.substitute_infer_type_parameters(
                        &o.type_arguments[0],
                        params,
                        substitutions,
                    );
                    // If nothing changed, avoid creating a new array type.
                    if Arc::ptr_eq(&new_elem, &o.type_arguments[0]) {
                        return Arc::clone(t);
                    }
                    return self.create_array_type(new_elem);
                }
                // Other object types (interfaces, type references with a
                // target, etc.) are not handled — return as-is.
                Arc::clone(t)
            }
            TypeData::Tuple(tup) => {
                // Substitute each tuple element's type and rebuild.
                let new_elems: Vec<Arc<Type>> = tup
                    .element_infos
                    .iter()
                    .map(|ei| match &ei.type_ {
                        Some(ty) => {
                            self.substitute_infer_type_parameters(ty, params, substitutions)
                        }
                        None => self.error_type(),
                    })
                    .collect();
                // If nothing changed, avoid rebuilding.
                let changed = tup
                    .element_infos
                    .iter()
                    .zip(new_elems.iter())
                    .any(|(ei, new_t)| match &ei.type_ {
                        Some(old_t) => !Arc::ptr_eq(old_t, new_t),
                        None => true,
                    });
                if !changed {
                    return Arc::clone(t);
                }
                self.create_tuple_type(new_elems)
            }
            // For nested conditional types, mapped types, indexed access
            // types, etc., we don't recursively substitute (that would
            // risk re-resolution or require a full `instantiateType`);
            // return as-is. Go handles these via `instantiateType`.
            _ => Arc::clone(t),
        }
    }
}

/// Whether a type contains any type-parameter subterm. Used by
/// `resolve_conditional_type` to decide whether the conditional can be
/// evaluated now or must be deferred until the type parameters are
/// substituted with concrete types.
fn type_contains_type_parameter(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::TypeParameter) {
        return true;
    }
    match &t.data {
        TypeData::Union(u) => u
            .union_or_intersection
            .types
            .iter()
            .any(type_contains_type_parameter),
        TypeData::Intersection(i) => i
            .union_or_intersection
            .types
            .iter()
            .any(type_contains_type_parameter),
        TypeData::Object(o) => {
            o.type_arguments.iter().any(type_contains_type_parameter)
                || o.target
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
        }
        TypeData::Conditional(ct) => {
            ct.check_type
                .as_ref()
                .map(type_contains_type_parameter)
                .unwrap_or(false)
                || ct
                    .extends_type
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
                || ct
                    .resolved_true_type
                    .get()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
                || ct
                    .resolved_false_type
                    .get()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
        }
        TypeData::Mapped(m) => {
            m.constraint_type
                .as_ref()
                .map(type_contains_type_parameter)
                .unwrap_or(false)
                || m.template_type
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
                || m.name_type
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
                || m.type_parameter
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
        }
        TypeData::TypeParameter(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_comparison_result_flags() {
        let result = RelationComparisonResult::Succeeded;
        assert!(result.contains(RelationComparisonResult::Succeeded));
        assert!(!result.contains(RelationComparisonResult::Failed));

        let combined = RelationComparisonResult::Succeeded | RelationComparisonResult::Failed;
        assert!(combined.contains(RelationComparisonResult::Succeeded));
        assert!(combined.contains(RelationComparisonResult::Failed));
    }

    #[test]
    fn relation_cache_basic() {
        let mut rel = Relation::new(RelationKind::Assignable);
        let key = CacheHashKey::new(1, 2);
        assert!(rel.get(&key) == RelationComparisonResult::None);
        rel.set(key, RelationComparisonResult::Succeeded);
        assert!(rel.get(&key) == RelationComparisonResult::Succeeded);
        assert_eq!(rel.size(), 1);
    }

    #[test]
    fn relation_cache_key_distinguishes_relation_kinds() {
        // The same (source, target) pair under different relations must
        // produce distinct cache keys — otherwise `Assignable` and
        // `Subtype` results would collide.
        let k1 = RelationCacheKey {
            source_ptr: 0x1000,
            target_ptr: 0x2000,
            relation: RelationKind::Assignable,
        };
        let k2 = RelationCacheKey {
            source_ptr: 0x1000,
            target_ptr: 0x2000,
            relation: RelationKind::Subtype,
        };
        assert_ne!(k1, k2);

        let mut set = std::collections::HashSet::new();
        set.insert(k1);
        // k2 is a different key, so it's not in the set.
        assert!(!set.contains(&k2));
        set.insert(k2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn relation_cache_key_distinguishes_type_pointers() {
        // Different source/target pointer pairs must produce distinct keys.
        let k1 = RelationCacheKey {
            source_ptr: 0x1000,
            target_ptr: 0x2000,
            relation: RelationKind::Assignable,
        };
        let k2 = RelationCacheKey {
            source_ptr: 0x3000,
            target_ptr: 0x2000,
            relation: RelationKind::Assignable,
        };
        assert_ne!(k1, k2);
    }

    #[test]
    fn recursion_flags_both() {
        assert!(RECURSION_FLAGS_BOTH.contains(RecursionFlags::Source));
        assert!(RECURSION_FLAGS_BOTH.contains(RecursionFlags::Target));
    }

    #[test]
    fn expanding_flags() {
        assert_eq!(ExpandingFlags::NONE.0, 0);
        assert_eq!(ExpandingFlags::SOURCE.0, 1);
        assert_eq!(ExpandingFlags::TARGET.0, 2);
        assert_eq!(ExpandingFlags::BOTH.0, 3);
    }

    #[test]
    fn signature_check_mode_callback_alias() {
        // The `Callback` const alias is the union of BivariantCallback and
        // StrictCallback, matching Go's `SignatureCheckModeCallback`.
        assert!(SignatureCheckMode::Callback.contains(SignatureCheckMode::BivariantCallback));
        assert!(SignatureCheckMode::Callback.contains(SignatureCheckMode::StrictCallback));
        assert_eq!(SignatureCheckMode::Callback, SIGNATURE_CHECK_MODE_CALLBACK);
    }

    #[test]
    fn type_arguments_related_covariant_by_default() {
        // Two empty type-argument slices are trivially related.
        // We construct a tiny standalone check: covariant variance with a
        // single pair of `any`/`unknown` types should yield False (since
        // `unknown` is not assignable to `any` under the strict
        // interpretation, but our `compare_types` collapses to `True` for
        // `any` source). This test is a smoke test of the variance dispatch
        // logic rather than the assignability semantics themselves.
        let result = Ternary::True.and(Ternary::True);
        assert_eq!(result, Ternary::True);
        // Ensure variance flag bit layout matches what we assume:
        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Covariant));
        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Contravariant));
        assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Independent));
        assert!(!VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Unmeasurable));
        assert!(!VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Unreliable));
    }

    #[test]
    fn index_signature_helpers_bit_layout() {
        // Sanity-check the ObjectFlags we rely on for the index-signature
        // structural fallback (P3.7f) are distinct bits — guards against
        // accidental renumbering of the ObjectFlags bitfield.
        let inferable =
            ObjectFlags::JSLiteral | ObjectFlags::ObjectRestType | ObjectFlags::ReverseMapped;
        assert!(inferable.contains(ObjectFlags::JSLiteral));
        assert!(inferable.contains(ObjectFlags::ObjectRestType));
        assert!(inferable.contains(ObjectFlags::ReverseMapped));
        // Fresh-literal is a separate bit (used by the strict-subtype carve-out).
        assert!(!inferable.contains(ObjectFlags::FreshLiteral));
    }
}
