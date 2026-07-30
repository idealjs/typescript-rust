//! Type mapper factory functions.
//!
//! Ported from `internal/checker/mapper.go`. The Go implementation uses
//! an interface-based `TypeMapperData` pattern with several concrete
//! implementations (SimpleTypeMapper, ArrayTypeMapper, MergedTypeMapper,
//! etc.). In Rust, `TypeMapper` holds a closure (`MapFn`), and these
//! factory functions create mappers with the appropriate behavior.

use std::sync::Arc;

use super::types::{Type, TypeMapper, TypeMapperKind};

/// Create a mapper that maps a single source type to a single target type.
pub fn new_simple_type_mapper(source: Arc<Type>, target: Arc<Type>) -> TypeMapper {
    let maps_this_only = is_this_type_parameter(&source);
    TypeMapper::new(
        Arc::new(move |t: &Arc<Type>| {
            if Arc::ptr_eq(t, &source) {
                Arc::clone(&target)
            } else {
                // No mapping applies: return the input type unchanged.
                // This matches Go's `SimpleTypeMapper.map` returning `t` when
                // `t != source`.
                Arc::clone(t)
            }
        }),
        TypeMapperKind::Simple,
        maps_this_only,
    )
}

/// Create a mapper that maps multiple source types to corresponding targets.
pub fn new_array_type_mapper(sources: Vec<Arc<Type>>, targets: Vec<Arc<Type>>) -> TypeMapper {
    let maps_this_only = sources.len() == 1 && is_this_type_parameter(&sources[0]);
    TypeMapper::new(
        Arc::new(move |t: &Arc<Type>| {
            for (i, s) in sources.iter().enumerate() {
                if Arc::ptr_eq(t, s) {
                    return Arc::clone(&targets[i]);
                }
            }
            // Type not in source list: return unchanged (Go returns `t`).
            Arc::clone(t)
        }),
        TypeMapperKind::Array,
        maps_this_only,
    )
}

/// Create a mapper that maps all source types to a single target.
pub fn new_array_to_single_type_mapper(sources: Vec<Arc<Type>>, target: Arc<Type>) -> TypeMapper {
    let maps_this_only = sources.len() == 1 && is_this_type_parameter(&sources[0]);
    TypeMapper::new(
        Arc::new(move |t: &Arc<Type>| {
            for s in &sources {
                if Arc::ptr_eq(t, s) {
                    return Arc::clone(&target);
                }
            }
            // Type not in source list: return unchanged (Go returns `t`).
            Arc::clone(t)
        }),
        TypeMapperKind::Array,
        maps_this_only,
    )
}

/// Create a mapper from a function.
pub fn new_function_type_mapper(
    map_fn: impl Fn(&Arc<Type>) -> Arc<Type> + Send + Sync + 'static,
) -> TypeMapper {
    TypeMapper::new(Arc::new(map_fn), TypeMapperKind::Unknown, false)
}

/// Merge two mappers (apply m1 first, then m2).
pub fn merge_type_mappers(m1: Option<&TypeMapper>, m2: Option<&TypeMapper>) -> Option<TypeMapper> {
    match (m1, m2) {
        (Some(m1), Some(m2)) => {
            let m1 = m1.clone();
            let m2 = m2.clone();
            Some(TypeMapper::new(
                Arc::new(move |t: &Arc<Type>| {
                    let t1 = m1.map(t);
                    m2.map(&t1)
                }),
                TypeMapperKind::Merged,
                false,
            ))
        }
        (Some(m1), None) => Some(m1.clone()),
        (None, Some(m2)) => Some(m2.clone()),
        (None, None) => None,
    }
}

/// Prepend a mapping to an existing mapper.
pub fn prepend_type_mapping(
    source: Arc<Type>,
    target: Arc<Type>,
    mapper: Option<&TypeMapper>,
) -> TypeMapper {
    match mapper {
        None => new_simple_type_mapper(source, target),
        Some(m) => {
            let simple = new_simple_type_mapper(Arc::clone(&source), Arc::clone(&target));
            merge_type_mappers(Some(&simple), Some(m)).unwrap_or(simple)
        }
    }
}

/// Append a mapping to an existing mapper.
pub fn append_type_mapping(
    mapper: Option<&TypeMapper>,
    source: Arc<Type>,
    target: Arc<Type>,
) -> TypeMapper {
    match mapper {
        None => new_simple_type_mapper(source, target),
        Some(m) => {
            let simple = new_simple_type_mapper(Arc::clone(&source), Arc::clone(&target));
            merge_type_mappers(Some(m), Some(&simple)).unwrap_or(simple)
        }
    }
}

/// Whether a type is a `this` type parameter.
fn is_this_type_parameter(t: &Type) -> bool {
    if let super::types::TypeData::TypeParameter(tp) = &t.data {
        tp.is_this_type
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
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
        // Regression: previously the placeholder returned `target` for any
        // non-matching input. The correct behavior (matching Go) is to
        // return the input type unchanged.
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
        // Matching input maps to target.
        let mapped = mapper.map(&source);
        assert!(Arc::ptr_eq(&mapped, &target));
        // Non-matching input returns the input itself, NOT target.
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
}
