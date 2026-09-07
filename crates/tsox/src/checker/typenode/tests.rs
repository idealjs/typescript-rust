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
