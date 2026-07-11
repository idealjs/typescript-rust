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
    pub fn is_type_identical_to(&self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
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
    pub fn is_type_assignable_to(&self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_simple_type_related_to(source, target, RelationKind::Assignable)
    }

    /// Check if `source` is a subtype of `target`.
    pub fn is_type_subtype_of(&self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_simple_type_related_to(source, target, RelationKind::Subtype)
    }

    /// Check if `source` is a strict subtype of `target`.
    pub fn is_type_strict_subtype_of(&self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_simple_type_related_to(source, target, RelationKind::StrictSubtype)
    }

    /// Check if `source` is comparable to `target`.
    pub fn is_type_comparable_to(&self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_simple_type_related_to(source, target, RelationKind::Comparable)
    }

    /// Check if two types are comparable (in either direction).
    pub fn are_types_comparable(&self, type1: &Arc<Type>, type2: &Arc<Type>) -> bool {
        self.is_type_comparable_to(type1, type2) || self.is_type_comparable_to(type2, type1)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Simple type relation checking
    // ────────────────────────────────────────────────────────────────────────

    /// Check if two types are identical at a simple level.
    ///
    /// This handles intrinsic types (by name), literal types (by value),
    /// and type parameters (by ID).
    fn is_simple_type_identical_to(&self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
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
        &self,
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
        if s.contains(TYPE_FLAGS_STRING_LIKE) && t.contains(TypeFlags::String) {
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
        if s.contains(TYPE_FLAGS_NUMBER_LIKE) && t.contains(TypeFlags::Number) {
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
        if s.contains(TYPE_FLAGS_BIG_INT_LIKE) && t.contains(TypeFlags::BigInt) {
            return true;
        }

        // Boolean-like → Boolean
        if s.contains(TYPE_FLAGS_BOOLEAN_LIKE) && t.contains(TypeFlags::Boolean) {
            return true;
        }

        // Symbol-like → ESSymbol
        if s.contains(TYPE_FLAGS_ES_SYMBOL_LIKE) && t.contains(TypeFlags::ESSymbol) {
            return true;
        }

        // Enum → Enum (same name): simplified, full check needs symbol comparison
        // TODO: port isEnumTypeRelatedTo

        // EnumLiteral → EnumLiteral: simplified
        if s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::EnumLiteral)
            && s.contains(TYPE_FLAGS_LITERAL)
            && t.contains(TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        // In non-strictNullChecks mode, `undefined` and `null` are assignable
        // to anything except `never`. Since unions and intersections may reduce
        // to `never`, we exclude them here.
        if s.contains(TypeFlags::Undefined)
            && (!self.strict_null_checks && !t.contains(TYPE_FLAGS_UNION_OR_INTERSECTION)
                || t.contains(TypeFlags::Undefined | TypeFlags::Void))
        {
            return true;
        }

        if s.contains(TypeFlags::Null)
            && (!self.strict_null_checks && !t.contains(TYPE_FLAGS_UNION_OR_INTERSECTION)
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
