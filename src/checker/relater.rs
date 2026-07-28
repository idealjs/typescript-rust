//! Type relation checking: determining whether one type is assignable to,
//! a subtype of, or identical to another.
//!
//! Ported from `internal/checker/relater.go`. This is a large and complex
//! module (~5000 lines in Go); this file ports the core types and the
//! `isSimpleTypeRelatedTo` function which handles the most common cases
//! (any, unknown, never, primitive types, literals).

use std::collections::HashMap;
use std::sync::Arc;

use super::checker::Checker;
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

/// The kind of relation being checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelationKind {
    #[default]
    Identity,
    Subtype,
    StrictSubtype,
    Assignable,
    Comparable,
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
    fn is_type_related_to(
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
            if let Some(resolved) = self.get_resolved_type_of_conditional_type(target) {
                if self.is_type_related_to(source, &resolved, relation) {
                    return true;
                }
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
        }

        false
    }

    /// Generate a cache key for comparing two types.
    fn make_cache_key(source: &Type, target: &Type) -> CacheHashKey {
        CacheHashKey::new(source.id as u64, target.id as u64)
    }

    /// Check if a comparison result is cached.
    fn get_cached_result(&self, source: &Type, target: &Type, relation: RelationKind) -> Option<bool> {
        // For now, we don't use the Relation cache directly since we don't have
        // a relation object per Checker. This is a placeholder for future use.
        None
    }

    /// Cache a comparison result.
    fn cache_result(&mut self, source: &Type, target: &Type, relation: RelationKind, result: bool) {
        // Placeholder for future caching
        _ = source;
        _ = target;
        _ = relation;
        _ = result;
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
        let min_len = source_tuple.element_infos.len().min(target_tuple.element_infos.len());
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
                if !flags.contains(ElementFlags::Optional) && !flags.contains(ElementFlags::Rest) && !flags.contains(ElementFlags::Variadic) {
                    return false;
                }
            }
        }

        true
    }

    /// Get the type of a tuple element at a given index.
    fn get_tuple_element_type(&self, t: &Arc<Type>, index: usize) -> Option<Arc<Type>> {
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
    fn is_element_flags_compatible(&self, source: ElementFlags, target: ElementFlags, _relation: RelationKind) -> bool {
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
            if relation == RelationKind::Assignable || relation == RelationKind::Comparable {
                return self.some_type_related_to_type(source, target, relation);
            } else {
                return self.each_type_related_to_type(source, target, relation);
            }
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
                // If target has a string index signature, check if source has a number index
                // signature that could serve as a string index (in practice, this doesn't work)
                return false;
            }
        }

        true
    }

    // ────────────────────────────────────────────────────────────────────────
    // Signature comparison
    // ────────────────────────────────────────────────────────────────────────

    /// Check if two signatures are related.
    ///
    /// A function type `(a: A) => R` is assignable to `(b: B) => S` if:
    /// - `S` is assignable to `R` (return type is covariant)
    /// - `B` is assignable to `A` (parameter types are contravariant)
    fn is_signature_related_to(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        relation: RelationKind,
    ) -> bool {
        // Check parameter count compatibility
        // Source must have at least as many required parameters as target
        let source_params = &source.parameters;
        let target_params = &target.parameters;

        // For non-identity relations, check minimum argument count
        if relation != RelationKind::Identity {
            // Source must have enough parameters to cover target's required params
            let source_min = source.min_argument_count() as usize;
            let target_min = target.min_argument_count() as usize;
            if source_min < target_min {
                return false;
            }
        }

        // Compare parameter types (contravariant: target param must be assignable to source param)
        let param_count = source_params.len().min(target_params.len());
        for i in 0..param_count {
            let source_param_type = self.get_type_of_symbol(&source_params[i]);
            let target_param_type = self.get_type_of_symbol(&target_params[i]);

            // Parameter types are contravariant: target -> source direction
            if !self.is_type_related_to(&target_param_type, &source_param_type, relation) {
                return false;
            }
        }

        // If source has more parameters than target, check they are optional/rest
        if source_params.len() > target_params.len() {
            for i in target_params.len()..source_params.len() {
                if !source.has_rest_parameter() && i == source_params.len() - 1 {
                    // Last parameter is not rest, so extra params are fine (JS allows calling with extra args)
                    break;
                }
                // In TypeScript, extra parameters in source are fine (they're just ignored)
                // But only if they could be optional
            }
        }

        // Compare return types (covariant: source return must be assignable to target return)
        let source_return = self.get_return_type_of_signature(source);
        let target_return = self.get_return_type_of_signature(target);

        match (source_return, target_return) {
            (Some(sr), Some(tr)) => {
                if !self.is_type_related_to(&sr, &tr, relation) {
                    return false;
                }
            }
            // If source has no return type, it's assignable to anything
            // If target has no return type, source must have none too
            (None, Some(_)) => return false,
            _ => {}
        }

        true
    }

    /// Check if the call signatures of two types are related.
    fn is_call_signatures_related_to(
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

        let source_calls = source_struct.call_signatures();
        let target_calls = target_struct.call_signatures();

        if source_calls.is_empty() && target_calls.is_empty() {
            return true; // Both have no call signatures
        }
        if source_calls.is_empty() {
            return false; // Target has call signatures but source doesn't
        }
        if target_calls.is_empty() {
            // Source has call signatures but target doesn't - only ok for comparable
            return relation == RelationKind::Comparable;
        }

        // Each target call signature must be matched by a source call signature
        for target_sig in target_calls {
            let mut found_match = false;
            for source_sig in source_calls {
                if self.is_signature_related_to(source_sig, target_sig, relation) {
                    found_match = true;
                    break;
                }
            }
            if !found_match {
                return false;
            }
        }

        true
    }

    /// Check if the construct signatures of two types are related.
    fn is_construct_signatures_related_to(
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

        let source_constructs = source_struct.construct_signatures();
        let target_constructs = target_struct.construct_signatures();

        if source_constructs.is_empty() && target_constructs.is_empty() {
            return true;
        }
        if source_constructs.is_empty() {
            return false;
        }
        if target_constructs.is_empty() {
            return relation == RelationKind::Comparable;
        }

        for target_sig in target_constructs {
            let mut found_match = false;
            for source_sig in source_constructs {
                if self.is_signature_related_to(source_sig, target_sig, relation) {
                    found_match = true;
                    break;
                }
            }
            if !found_match {
                return false;
            }
        }

        true
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
}
