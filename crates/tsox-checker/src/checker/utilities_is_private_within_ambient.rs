#![allow(unused_imports)]

use crate::checker::utilities::*;
use tsox_frontend::ast::NodeData;

pub fn is_private_within_ambient(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_private_within_ambient"); 
    (tsox_frontend::ast::has_syntactic_modifier(node, ModifierFlags::Private))
        && node.flags.contains(NodeFlags::Ambient)
}

pub fn pseudo_big_int_to_string(value: &tsox_core::jsnum::PseudoBigInt) -> String { ::tsox_core::fntrace::enter("pseudo_big_int_to_string"); 
    value.to_string()
}

pub fn value_to_string(value: &LiteralValue) -> String { ::tsox_core::fntrace::enter("value_to_string"); 
    match value {
        LiteralValue::String(s) => format!("\"{}\"", s),
        LiteralValue::Number(n) => n.to_string(),
        LiteralValue::Boolean(b) => b.to_string(),
        LiteralValue::BigInt(b) => format!("{}n", b),
        LiteralValue::None => String::new(),
    }
}

pub fn get_non_rest_parameter_count(sig: &Signature) -> usize { ::tsox_core::fntrace::enter("get_non_rest_parameter_count"); 
    let has_rest = sig.flags.contains(SignatureFlags::HasRestParameter);
    sig.parameters.len() - if has_rest { 1 } else { 0 }
}

pub fn contains_non_missing_undefined_type(t: &Type) -> bool { ::tsox_core::fntrace::enter("contains_non_missing_undefined_type"); 
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

pub fn try_get_property_access_or_identifier_to_string(expr: &Node) -> String { ::tsox_core::fntrace::enter("try_get_property_access_or_identifier_to_string"); 
    if tsox_frontend::ast::is_identifier(expr) {
        return expr.text().to_string();
    }
    String::new()
}

pub fn get_set_accessor_value_parameter(accessor: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_set_accessor_value_parameter"); 
    let parameters = tsox_frontend::ast::mig::m3b::parameters(accessor);
    if !parameters.is_empty() {
        let has_this = parameters.len() == 2
            && tsox_frontend::ast::mig::m3g_2::is_this_parameter(&parameters[0]);
        let index = if has_this { 1 } else { 0 };
        return Some(std::sync::Arc::clone(&parameters[index]));
    }
    None
}

pub fn get_super_container(node: &Node, _stop_on_functions: bool) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_super_container"); 
    node.parent()
}

pub fn get_alias_declaration_from_name(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_alias_declaration_from_name"); 
    let _ = node;
    None
}

pub fn get_containing_object_literal(f: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_containing_object_literal"); 
    let _ = f;
    None
}

pub fn is_import_type_qualifier_part(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("is_import_type_qualifier_part"); 
    let _ = node;
    None
}

pub fn is_in_name_of_expression_with_type_arguments(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_in_name_of_expression_with_type_arguments"); 
    let _ = node;
    false
}

pub fn is_in_right_side_of_import_or_export_assignment(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_in_right_side_of_import_or_export_assignment");
    // Go checker/utilities.go:1235-1241：沿限定名上爬到顶，判所在声明
    let Some(mut cur) = node.parent() else {
        return false;
    };
    while cur.kind == SyntaxKind::QualifiedName {
        let Some(next) = cur.parent() else {
            return false;
        };
        cur = next;
    }
    match cur.parent() {
        Some(decl) => match &decl.data {
            NodeData::ImportEqualsDeclaration(d) => {
                std::ptr::eq(d.module_reference.as_ref(), cur.as_ref())
            }
            NodeData::ExportAssignment(e) => std::ptr::eq(e.expression.as_ref(), cur.as_ref()),
            _ => false,
        },
        None => false,
    }
}

pub fn is_class_instance_property(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_class_instance_property"); 
    node.parent()
        .as_ref()
        .map(|p| {
            tsox_frontend::ast::is_class_like(p)
                && tsox_frontend::ast::is_property_declaration(node)
                && !tsox_frontend::ast::has_accessor_modifier(node)
        })
        .unwrap_or(false)
}

pub fn is_this_initialized_object_binding_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_this_initialized_object_binding_expression"); 
    let _ = node;
    false
}

pub fn get_members_of_declaration(node: &Node) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_members_of_declaration"); 
    let _ = node;
    Vec::new()
}

pub fn expression_result_is_unused(node: &Node) -> bool { ::tsox_core::fntrace::enter("expression_result_is_unused"); 
    let _ = node;
    false
}

pub fn for_each_yield_expression(body: &Node, _visitor: impl Fn(&Node)) { ::tsox_core::fntrace::enter("for_each_yield_expression"); 
    let _ = body;
}

pub fn is_jsdoc_optional_parameter(_node: &Node) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_optional_parameter"); 
    false
}
