use super::super::types::*;
use super::*;
use std::sync::OnceLock;

#[test]
fn simple_mapper() {
    let source = Arc::new(Type::new(
        TypeFlags::TypeParameter,
        TypeData::TypeParameter(TypeParameterData {
            constrained: ConstrainedTypeData::default(),
            constraint: None,
            target: None,
            mapper: None,
            is_this_type: false,
            resolved_default_type: OnceLock::new(),
        }),
    ));
    let target = Arc::new(Type::new(
        TypeFlags::String,
        TypeData::Intrinsic(IntrinsicTypeData {
            intrinsic_name: "string".to_string(),
        }),
    ));

    let mapper = new_simple_type_mapper(Arc::clone(&source), Arc::clone(&target));
    assert_eq!(mapper.kind, TypeMapperKind::Simple);
    assert!(!mapper.maps_this_only());
}

#[test]
fn this_type_parameter_mapper() {
    let source = Arc::new(Type::new(
        TypeFlags::TypeParameter,
        TypeData::TypeParameter(TypeParameterData {
            constrained: ConstrainedTypeData::default(),
            constraint: None,
            target: None,
            mapper: None,
            is_this_type: true,
            resolved_default_type: OnceLock::new(),
        }),
    ));
    let target = Arc::new(Type::new(
        TypeFlags::String,
        TypeData::Intrinsic(IntrinsicTypeData {
            intrinsic_name: "string".to_string(),
        }),
    ));

    let mapper = new_simple_type_mapper(source, target);
    assert!(mapper.maps_this_only());
}

#[test]
fn merge_mappers() {
    let m1 = new_function_type_mapper(|t: &Arc<Type>| {
        Arc::new(Type::new(
            t.flags,
            TypeData::Intrinsic(IntrinsicTypeData {
                intrinsic_name: "m1".to_string(),
            }),
        ))
    });
    let m2 = new_function_type_mapper(|t: &Arc<Type>| {
        Arc::new(Type::new(
            t.flags,
            TypeData::Intrinsic(IntrinsicTypeData {
                intrinsic_name: "m2".to_string(),
            }),
        ))
    });

    let merged = merge_type_mappers(Some(&m1), Some(&m2));
    assert!(merged.is_some());
    assert_eq!(merged.unwrap().kind, TypeMapperKind::Merged);
}

#[test]
fn merge_with_none() {
    let m1 = new_function_type_mapper(|t: &Arc<Type>| {
        Arc::new(Type::new(
            t.flags,
            TypeData::Intrinsic(IntrinsicTypeData {
                intrinsic_name: "m1".to_string(),
            }),
        ))
    });

    let merged = merge_type_mappers(Some(&m1), None);
    assert!(merged.is_some());
    assert_eq!(merged.unwrap().kind, TypeMapperKind::Unknown);

    let merged = merge_type_mappers(None, None);
    assert!(merged.is_none());
}

#[test]
fn simple_mapper_returns_input_when_no_match() {
    let source = Arc::new(Type::new(
        TypeFlags::TypeParameter,
        TypeData::TypeParameter(TypeParameterData {
            constrained: ConstrainedTypeData::default(),
            constraint: None,
            target: None,
            mapper: None,
            is_this_type: false,
            resolved_default_type: OnceLock::new(),
        }),
    ));
    let target = Arc::new(Type::new(
        TypeFlags::String,
        TypeData::Intrinsic(IntrinsicTypeData {
            intrinsic_name: "string".to_string(),
        }),
    ));
    let other = Arc::new(Type::new(
        TypeFlags::Number,
        TypeData::Intrinsic(IntrinsicTypeData {
            intrinsic_name: "number".to_string(),
        }),
    ));

    let mapper = new_simple_type_mapper(Arc::clone(&source), Arc::clone(&target));

    let mapped = mapper.map(&source);
    assert!(Arc::ptr_eq(&mapped, &target));

    let passthrough = mapper.map(&other);
    assert!(Arc::ptr_eq(&passthrough, &other));
}

#[test]
fn array_mapper_returns_input_when_no_match() {
    let source = Arc::new(Type::new(
        TypeFlags::TypeParameter,
        TypeData::TypeParameter(TypeParameterData {
            constrained: ConstrainedTypeData::default(),
            constraint: None,
            target: None,
            mapper: None,
            is_this_type: false,
            resolved_default_type: OnceLock::new(),
        }),
    ));
    let target = Arc::new(Type::new(
        TypeFlags::String,
        TypeData::Intrinsic(IntrinsicTypeData {
            intrinsic_name: "string".to_string(),
        }),
    ));
    let other = Arc::new(Type::new(
        TypeFlags::Number,
        TypeData::Intrinsic(IntrinsicTypeData {
            intrinsic_name: "number".to_string(),
        }),
    ));

    let mapper = new_array_type_mapper(vec![Arc::clone(&source)], vec![Arc::clone(&target)]);
    let passthrough = mapper.map(&other);
    assert!(Arc::ptr_eq(&passthrough, &other));
}
