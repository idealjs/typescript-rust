use crate::checker::types::*;
use std::sync::Arc;

pub(super) fn type_parameters_same(a: &[Arc<Type>], b: &[Arc<Type>]) -> bool { ::tsox_core::fntrace::enter("type_parameters_same"); 
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

pub(crate) fn type_contains_type_parameter(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_contains_type_parameter"); 
    type_contains_type_parameter_skipping(t, &[])
}

pub(crate) fn type_contains_type_parameter_skipping(
    t: &Arc<Type>,
    skip: &[Arc<Type>],
) -> bool { ::tsox_core::fntrace::enter("type_contains_type_parameter_skipping"); 
    if t.flags.contains(TypeFlags::TypeParameter) {
        return !skip.iter().any(|s| Arc::ptr_eq(s, t));
    }
    let recur = |x: &Arc<Type>| type_contains_type_parameter_skipping(x, skip);
    match &t.data {
        TypeData::Union(u) => u.union_or_intersection.types.iter().any(recur),
        TypeData::Intersection(i) => i.union_or_intersection.types.iter().any(recur),
        TypeData::Object(o) => {
            o.type_arguments.iter().any(recur)
                || o.target
                    .as_ref()
                    .map(recur)
                    .unwrap_or(false)
        }
        TypeData::Conditional(ct) => {
            ct.check_type
                .as_ref()
                .map(recur)
                .unwrap_or(false)
                || ct
                    .extends_type
                    .as_ref()
                    .map(recur)
                    .unwrap_or(false)
                || ct
                    .resolved_true_type
                    .get()
                    .map(recur)
                    .unwrap_or(false)
                || ct
                    .resolved_false_type
                    .get()
                    .map(recur)
                    .unwrap_or(false)
        }
        TypeData::Mapped(m) => {
            m.constraint_type
                .as_ref()
                .map(recur)
                .unwrap_or(false)
                || m.template_type
                    .as_ref()
                    .map(recur)
                    .unwrap_or(false)
                || m.name_type
                    .as_ref()
                    .map(recur)
                    .unwrap_or(false)
                || m.type_parameter
                    .as_ref()
                    .map(recur)
                    .unwrap_or(false)
        }
        TypeData::TypeParameter(_) => true,
        TypeData::IndexedAccess(ia) => {
            ia.object_type
                .as_ref()
                .map(recur)
                .unwrap_or(false)
                || ia
                    .index_type
                    .as_ref()
                    .map(recur)
                    .unwrap_or(false)
        }
        TypeData::Index(it) => it
            .target
            .as_ref()
            .map(recur)
            .unwrap_or(false),
        _ => false,
    }
}


