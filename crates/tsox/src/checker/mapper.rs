use std::sync::Arc;

use super::types::{Type, TypeMapper, TypeMapperKind};

pub fn new_simple_type_mapper(source: Arc<Type>, target: Arc<Type>) -> TypeMapper {
    let maps_this_only = is_this_type_parameter(&source);
    TypeMapper::new(
        Arc::new(move |t: &Arc<Type>| {
            if Arc::ptr_eq(t, &source) {
                Arc::clone(&target)
            } else {
                Arc::clone(t)
            }
        }),
        TypeMapperKind::Simple,
        maps_this_only,
    )
}

pub fn new_array_type_mapper(sources: Vec<Arc<Type>>, targets: Vec<Arc<Type>>) -> TypeMapper {
    let maps_this_only = sources.len() == 1 && is_this_type_parameter(&sources[0]);
    TypeMapper::new(
        Arc::new(move |t: &Arc<Type>| {
            for (i, s) in sources.iter().enumerate() {
                if Arc::ptr_eq(t, s) {
                    return Arc::clone(&targets[i]);
                }
            }

            Arc::clone(t)
        }),
        TypeMapperKind::Array,
        maps_this_only,
    )
}

pub fn new_array_to_single_type_mapper(sources: Vec<Arc<Type>>, target: Arc<Type>) -> TypeMapper {
    let maps_this_only = sources.len() == 1 && is_this_type_parameter(&sources[0]);
    TypeMapper::new(
        Arc::new(move |t: &Arc<Type>| {
            for s in &sources {
                if Arc::ptr_eq(t, s) {
                    return Arc::clone(&target);
                }
            }

            Arc::clone(t)
        }),
        TypeMapperKind::Array,
        maps_this_only,
    )
}

pub fn new_function_type_mapper(
    map_fn: impl Fn(&Arc<Type>) -> Arc<Type> + Send + Sync + 'static,
) -> TypeMapper {
    TypeMapper::new(Arc::new(map_fn), TypeMapperKind::Unknown, false)
}

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

fn is_this_type_parameter(t: &Type) -> bool {
    if let super::types::TypeData::TypeParameter(tp) = &t.data {
        tp.is_this_type
    } else {
        false
    }
}

#[cfg(test)]
mod tests;
