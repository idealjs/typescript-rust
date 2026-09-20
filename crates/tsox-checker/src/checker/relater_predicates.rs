use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::Symbol;

pub fn is_hyphenated_jsx_name(name: &str) -> bool {
    name.contains('-')
}

pub fn is_excess_property_check_target(t: &Type) -> bool {
    if matches!(&t.data, TypeData::Mapped(m) if m.type_parameter.is_some()) {
        return false;
    }
    if t.flags.contains(TypeFlags::Object)
        && !t
            .object_flags
            .contains(ObjectFlags::ObjectLiteralPatternWithComputedProperties)
    {
        return true;
    }
    if t.flags.contains(TypeFlags::NonPrimitive) {
        return true;
    }
    if t.flags.contains(TypeFlags::Substitution) {
        if let TypeData::Substitution(s) = &t.data {
            return s
                .base_type
                .as_ref()
                .map(|t| is_excess_property_check_target(t))
                .unwrap_or(false);
        }
    }
    if t.flags.contains(TypeFlags::Union) {
        if let Some(types) = t.types() {
            return types.iter().any(|t| is_excess_property_check_target(t));
        }
    }
    if t.flags.contains(TypeFlags::Intersection) {
        if let Some(types) = t.types() {
            return types.iter().all(|t| is_excess_property_check_target(t));
        }
    }
    false
}

pub fn is_object_or_instantiable_non_primitive(t: &Type) -> bool {
    t.flags
        .intersects(TypeFlags::Object | TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE)
}

pub fn is_non_primitive_type(t: &Type) -> bool {
    t.flags.contains(TypeFlags::NonPrimitive)
}

pub fn visibility_to_string(flags: tsox_frontend::ast::ModifierFlags) -> String {
    if flags == tsox_frontend::ast::ModifierFlags::Private {
        "private".to_string()
    } else if flags == tsox_frontend::ast::ModifierFlags::Protected {
        "protected".to_string()
    } else {
        "public".to_string()
    }
}

pub fn exclude_properties(
    properties: &[Arc<Symbol>],
    excluded_properties: &std::collections::HashSet<String>,
) -> Vec<Arc<Symbol>> {
    properties
        .iter()
        .filter(|p| !excluded_properties.contains(&p.name))
        .cloned()
        .collect()
}

pub fn should_check_as_excess_property(prop: &Symbol, container: &Symbol) -> bool {
    let prop_decl = prop.value_declaration.clone().or_else(|| prop.declarations.first().cloned());
    let container_decl = container
        .value_declaration
        .clone()
        .or_else(|| container.declarations.first().cloned());
    match (prop_decl, container_decl) {
        (Some(p), Some(c)) => p
            .parent()
            .is_some_and(|pp| Arc::ptr_eq(&pp, &c)),
        _ => false,
    }
}

pub fn is_ignored_jsx_property(_source: &Type, _source_prop: &Symbol) -> bool {
    false
}

pub struct TypeDiscriminator {
    pub names: Vec<String>,
}

impl TypeDiscriminator {
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn name(&self, index: usize) -> &str {
        &self.names[index]
    }

    pub fn matches(&self, _index: usize, _t: &Arc<Type>) -> bool {
        false
    }
}
