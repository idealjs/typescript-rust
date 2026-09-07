#![allow(unused_imports)]

use super::*;

pub fn is_private_within_ambient(node: &Node) -> bool {
    (crate::ast::has_syntactic_modifier(node, ModifierFlags::Private))
        && node.flags.contains(NodeFlags::Ambient)
}

pub fn pseudo_big_int_to_string(value: &crate::jsnum::PseudoBigInt) -> String {
    value.to_string()
}

pub fn value_to_string(value: &LiteralValue) -> String {
    match value {
        LiteralValue::String(s) => format!("\"{}\"", s),
        LiteralValue::Number(n) => n.to_string(),
        LiteralValue::Boolean(b) => b.to_string(),
        LiteralValue::BigInt(b) => format!("{}n", b),
        LiteralValue::None => String::new(),
    }
}

pub fn get_non_rest_parameter_count(sig: &Signature) -> usize {
    let has_rest = sig.flags.contains(SignatureFlags::HasRestParameter);
    sig.parameters.len() - if has_rest { 1 } else { 0 }
}

pub fn contains_non_missing_undefined_type(t: &Type) -> bool {
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            if let Some(first) = u.union_or_intersection.types.first() {
                return first.flags.contains(TypeFlags::Undefined);
            }
        }
        false
    } else {
        t.flags.contains(TypeFlags::Undefined)
    }
}

pub fn try_get_property_access_or_identifier_to_string(expr: &Node) -> String {
    if crate::ast::is_identifier(expr) {
        return expr.text().to_string();
    }
    String::new()
}

pub fn get_set_accessor_value_parameter(accessor: &Node) -> Option<Arc<Node>> {
    let _ = accessor;
    None
}

pub fn get_super_container(node: &Node, _stop_on_functions: bool) -> Option<Arc<Node>> {
    node.parent.clone()
}

pub fn get_alias_declaration_from_name(node: &Node) -> Option<Arc<Node>> {
    let _ = node;
    None
}

pub fn get_containing_object_literal(f: &Node) -> Option<Arc<Node>> {
    let _ = f;
    None
}

pub fn is_import_type_qualifier_part(node: &Node) -> Option<Arc<Node>> {
    let _ = node;
    None
}

pub fn is_in_name_of_expression_with_type_arguments(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn is_in_right_side_of_import_or_export_assignment(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn is_class_instance_property(node: &Node) -> bool {
    node.parent
        .as_ref()
        .map(|p| {
            crate::ast::is_class_like(p)
                && crate::ast::is_property_declaration(node)
                && !crate::ast::has_accessor_modifier(node)
        })
        .unwrap_or(false)
}

pub fn is_this_initialized_object_binding_expression(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn get_members_of_declaration(node: &Node) -> Vec<Arc<Node>> {
    let _ = node;
    Vec::new()
}

pub fn expression_result_is_unused(node: &Node) -> bool {
    let _ = node;
    false
}

pub fn for_each_yield_expression(body: &Node, _visitor: impl Fn(&Node)) {
    let _ = body;
}

pub fn is_jsdoc_optional_parameter(_node: &Node) -> bool {
    false
}
