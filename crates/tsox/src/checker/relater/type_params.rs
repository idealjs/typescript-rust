use super::super::types::*;
use std::sync::Arc;

pub(super) fn type_parameters_same(a: &[Arc<Type>], b: &[Arc<Type>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

pub(crate) fn type_contains_type_parameter(t: &Arc<Type>) -> bool {
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
        TypeData::IndexedAccess(ia) => {
            ia.object_type
                .as_ref()
                .map(type_contains_type_parameter)
                .unwrap_or(false)
                || ia
                    .index_type
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
        }
        TypeData::Index(it) => it
            .target
            .as_ref()
            .map(type_contains_type_parameter)
            .unwrap_or(false),
        _ => false,
    }
}

pub(crate) fn type_mentions_type_parameter(t: &Arc<Type>, needle: &Arc<Type>) -> bool {
    if Arc::ptr_eq(t, needle) {
        return true;
    }
    match &t.data {
        TypeData::Union(u) => u
            .union_or_intersection
            .types
            .iter()
            .any(|ty| type_mentions_type_parameter(ty, needle)),
        TypeData::Intersection(i) => i
            .union_or_intersection
            .types
            .iter()
            .any(|ty| type_mentions_type_parameter(ty, needle)),
        TypeData::Object(o) => o
            .type_arguments
            .iter()
            .any(|ty| type_mentions_type_parameter(ty, needle)),
        TypeData::Tuple(tu) => tu
            .element_infos
            .iter()
            .filter_map(|e| e.type_.as_ref())
            .any(|ty| type_mentions_type_parameter(ty, needle)),
        _ => false,
    }
}

pub(crate) fn collect_free_type_parameters(t: &Arc<Type>, out: &mut Vec<Arc<Type>>) {
    match &t.data {
        TypeData::TypeParameter(_) => {
            if !out.iter().any(|p| Arc::ptr_eq(p, t)) {
                out.push(Arc::clone(t));
            }
        }
        TypeData::Union(u) => {
            for ty in &u.union_or_intersection.types {
                collect_free_type_parameters(ty, out);
            }
        }
        TypeData::Intersection(i) => {
            for ty in &i.union_or_intersection.types {
                collect_free_type_parameters(ty, out);
            }
        }
        TypeData::Object(o) => {
            for ty in &o.type_arguments {
                collect_free_type_parameters(ty, out);
            }
        }
        TypeData::Tuple(tu) => {
            for ei in &tu.element_infos {
                if let Some(ty) = &ei.type_ {
                    collect_free_type_parameters(ty, out);
                }
            }
        }
        _ => {}
    }
}
