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
    let k1 = RelationCacheKey {
        source_id: 0x1000,
        target_id: 0x2000,
        relation: RelationKind::Assignable,
    };
    let k2 = RelationCacheKey {
        source_id: 0x1000,
        target_id: 0x2000,
        relation: RelationKind::Subtype,
    };
    assert_ne!(k1, k2);

    let mut set = std::collections::HashSet::new();
    set.insert(k1);

    assert!(!set.contains(&k2));
    set.insert(k2);
    assert_eq!(set.len(), 2);
}

#[test]
fn relation_cache_key_distinguishes_type_pointers() {
    let k1 = RelationCacheKey {
        source_id: 0x1000,
        target_id: 0x2000,
        relation: RelationKind::Assignable,
    };
    let k2 = RelationCacheKey {
        source_id: 0x3000,
        target_id: 0x2000,
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
    assert!(SignatureCheckMode::Callback.contains(SignatureCheckMode::BivariantCallback));
    assert!(SignatureCheckMode::Callback.contains(SignatureCheckMode::StrictCallback));
    assert_eq!(SignatureCheckMode::Callback, SIGNATURE_CHECK_MODE_CALLBACK);
}

#[test]
fn type_arguments_related_covariant_by_default() {
    let result = Ternary::True.and(Ternary::True);
    assert_eq!(result, Ternary::True);

    assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Covariant));
    assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Contravariant));
    assert!(VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Independent));
    assert!(!VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Unmeasurable));
    assert!(!VARIANCE_FLAGS_VARIANCE_MASK.contains(VarianceFlags::Unreliable));
}

#[test]
fn index_signature_helpers_bit_layout() {
    let inferable =
        ObjectFlags::JSLiteral | ObjectFlags::ObjectRestType | ObjectFlags::ReverseMapped;
    assert!(inferable.contains(ObjectFlags::JSLiteral));
    assert!(inferable.contains(ObjectFlags::ObjectRestType));
    assert!(inferable.contains(ObjectFlags::ReverseMapped));

    assert!(!inferable.contains(ObjectFlags::FreshLiteral));
}
