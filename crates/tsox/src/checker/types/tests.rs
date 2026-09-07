use super::*;

#[test]
fn ternary_and_or() {
    assert_eq!(Ternary::True.and(Ternary::False), Ternary::False);
    assert_eq!(Ternary::True.or(Ternary::False), Ternary::True);
    assert_eq!(Ternary::Unknown.and(Ternary::Maybe), Ternary::Unknown);
    assert_eq!(Ternary::Unknown.or(Ternary::Maybe), Ternary::Maybe);
    assert_eq!(!Ternary::True, Ternary::False);
    assert_eq!(!Ternary::False, Ternary::True);
    assert_eq!(!Ternary::Unknown, Ternary::Unknown);
}

#[test]
fn type_flags_composites() {
    assert!(TYPE_FLAGS_LITERAL.contains(TypeFlags::StringLiteral));
    assert!(TYPE_FLAGS_LITERAL.contains(TypeFlags::NumberLiteral));
    assert!(TYPE_FLAGS_NULLABLE.contains(TypeFlags::Undefined));
    assert!(TYPE_FLAGS_NULLABLE.contains(TypeFlags::Null));
    assert!(TYPE_FLAGS_STRING_LIKE.contains(TypeFlags::String));
    assert!(TYPE_FLAGS_STRING_LIKE.contains(TypeFlags::StringLiteral));
    assert!(TYPE_FLAGS_UNION_OR_INTERSECTION.contains(TypeFlags::Union));
    assert!(TYPE_FLAGS_UNION_OR_INTERSECTION.contains(TypeFlags::Intersection));
}

#[test]
fn object_flags_composites() {
    assert!(OBJECT_FLAGS_CLASS_OR_INTERFACE.contains(ObjectFlags::Class));
    assert!(OBJECT_FLAGS_CLASS_OR_INTERFACE.contains(ObjectFlags::Interface));
}

#[test]
fn signature_flags_propagating() {
    assert!(SIGNATURE_FLAGS_PROPAGATING_FLAGS.contains(SignatureFlags::HasRestParameter));
    assert!(SIGNATURE_FLAGS_PROPAGATING_FLAGS.contains(SignatureFlags::Construct));
    assert!(!SIGNATURE_FLAGS_PROPAGATING_FLAGS.contains(SignatureFlags::IsInnerCallChain));
}

#[test]
fn literal_value_to_string() {
    assert_eq!(
        LiteralValue::String("hello".to_string()).to_string(),
        "\"hello\""
    );
    assert_eq!(LiteralValue::Boolean(true).to_string(), "true");
    assert_eq!(LiteralValue::Boolean(false).to_string(), "false");
    assert_eq!(LiteralValue::None.to_string(), "");
}

#[test]
fn type_data_pattern_matching() {
    let t = Type::new(
        TypeFlags::String,
        TypeData::Intrinsic(IntrinsicTypeData {
            intrinsic_name: "string".to_string(),
        }),
    );
    assert!(t.is_string());
    assert!(!t.is_union());
    assert_eq!(t.intrinsic_name(), Some("string"));
}

#[test]
fn structured_type_call_construct_signatures() {
    let mut structured = StructuredTypeData::default();
    structured.call_signature_count = 2;

    structured.signatures = vec![
        Arc::new(Signature::new()),
        Arc::new(Signature::new()),
        Arc::new(Signature::new()),
    ];
    assert_eq!(structured.call_signatures().len(), 2);
    assert_eq!(structured.construct_signatures().len(), 1);
}

#[test]
fn cache_hash_key() {
    let k1 = CacheHashKey::new(1, 2);
    let k2 = CacheHashKey::new(1, 2);
    let k3 = CacheHashKey::new(3, 4);
    assert_eq!(k1, k2);
    assert_ne!(k1, k3);
}
