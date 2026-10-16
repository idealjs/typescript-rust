use crate::ast::*;
use std::sync::Arc;

use super::m3b::is_type_only;
use super::m3e_4::get_jsdoc_deprecated_tag;
use super::m3f_3::is_callee_worker;
use super::m3g_2::{is_parameter_property_modifier, is_signed_numeric_literal};
use super::m3g_3::skip_parentheses;

fn is_const_type_reference(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_const_type_reference"); 
    if !is_type_reference_node(node) {
        return false;
    }
    match &node.data {
        NodeData::TypeReferenceNode(d) => {
            d.type_arguments.is_none() && is_identifier(&d.type_name) && d.type_name.text() == "const"
        }
        _ => false,
    }
}

pub fn is_decorator_target(
    node: &Arc<Node>,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_decorator_target"); 
    is_callee_worker(
        node,
        is_decorator,
        include_element_access,
        skip_past_outer_expressions,
    )
}

pub fn is_call_like_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_call_like_expression"); 
    match node.kind {
        SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxOpeningFragment
        | SyntaxKind::CallExpression
        | SyntaxKind::NewExpression
        | SyntaxKind::TaggedTemplateExpression
        | SyntaxKind::Decorator => true,
        SyntaxKind::BinaryExpression => match &node.data {
            NodeData::BinaryExpression(d) => d.operator_token.kind == SyntaxKind::InstanceOfKeyword,
            _ => false,
        },
        _ => false,
    }
}

pub fn is_call_like_or_function_like_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_call_like_or_function_like_expression"); 
    is_call_like_expression(node) || is_function_expression_or_arrow_function(node)
}

pub fn is_call_or_new_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_call_or_new_expression"); 
    is_call_expression(node) || is_new_expression(node)
}

pub fn is_catch_clause_variable_declaration_or_binding_element(declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_catch_clause_variable_declaration_or_binding_element"); 
    let node = get_root_declaration(declaration);
    node.kind == SyntaxKind::VariableDeclaration
        && node
            .parent()
            .is_some_and(|parent| parent.kind == SyntaxKind::CatchClause)
}

pub fn is_check_js_enabled_for_file(
    source_file: &SourceFile,
    compiler_options: &tsox_core::core::compiler_options::CompilerOptions,
) -> bool { ::tsox_core::fntrace::enter("is_check_js_enabled_for_file"); 
    let _ = source_file;
    compiler_options.check_js == tsox_core::core::tristate::Tristate::True
}

pub fn is_class_member_modifier(token: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_class_member_modifier"); 
    is_parameter_property_modifier(token)
        || token == SyntaxKind::StaticKeyword
        || token == SyntaxKind::OverrideKeyword
        || token == SyntaxKind::AccessorKeyword
}

pub fn is_class_or_type_element(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_class_or_type_element"); 
    is_class_element(node) || is_type_element(node)
}

pub fn is_comma_sequence(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_comma_sequence"); 
    is_comma_expression(node)
}

pub fn is_computed_non_literal_name(name: &Node) -> bool { ::tsox_core::fntrace::enter("is_computed_non_literal_name"); 
    is_computed_property_name(name)
        && !name.expression().is_some_and(|e| is_string_or_numeric_literal_like(e))
}

pub fn is_const_assertion(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_const_assertion"); 
    match node.kind {
        SyntaxKind::AsExpression | SyntaxKind::TypeAssertionExpression => node
            .type_node()
            .is_some_and(|t| is_const_type_reference(t)),
        _ => false,
    }
}

pub fn is_contextual_keyword(token: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_contextual_keyword"); 
    (SyntaxKind::AbstractKeyword as i16..=SyntaxKind::DeferKeyword as i16)
        .contains(&(token as i16))
}

pub fn is_declaration_binding_element(binding_element: &Node) -> bool { ::tsox_core::fntrace::enter("is_declaration_binding_element"); 
    matches!(
        binding_element.kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
    )
}

pub fn is_declaration_name(name: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_declaration_name"); 
    !is_source_file(name)
        && !is_binding_pattern(name)
        && name
            .parent()
            .is_some_and(|parent| {
                is_declaration(&parent) && parent.name().is_some_and(|n| Arc::ptr_eq(n, name))
            })
}

pub fn is_declaration_name_or_import_property_name(name: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_declaration_name_or_import_property_name"); 
    match name.parent() {
        Some(parent) => match parent.kind {
            SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier => {
                is_identifier(name) || name.kind == SyntaxKind::StringLiteral
            }
            _ => is_declaration_name(name),
        },
        None => is_declaration_name(name),
    }
}

pub fn is_default_import(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_default_import"); 
    match node.kind {
        SyntaxKind::ImportDeclaration | SyntaxKind::JSImportDeclaration => {
            node_import_clause(node).is_some_and(|clause| match &clause.data {
                NodeData::ImportClause(d) => d.name.is_some(),
                _ => false,
            })
        }
        _ => false,
    }
}

pub fn node_import_clause(node: &Node) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("node_import_clause"); 
    match &node.data {
        NodeData::ImportDeclaration(d) => d.import_clause.as_ref(),
        NodeData::JSDocImportTag(d) => d.import_clause.as_ref(),
        _ => None,
    }
}

pub fn is_deprecated_declaration(declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_deprecated_declaration"); 
    is_deprecated_declaration_with_cached_flags(declaration, get_combined_node_flags(declaration))
}

pub fn is_deprecated_declaration_with_cached_flags(
    declaration: &Arc<Node>,
    combined_flags: NodeFlags,
) -> bool { ::tsox_core::fntrace::enter("is_deprecated_declaration_with_cached_flags"); 
    if !combined_flags.contains(NodeFlags::PossiblyContainsDeprecatedTag) {
        return false;
    }
    let mut current = Some(Arc::clone(declaration));
    while let Some(node) = current {
        if node.flags.contains(NodeFlags::PossiblyContainsDeprecatedTag) {
            return get_jsdoc_deprecated_tag(&node).is_some();
        }
        current = node.parent();
    }
    false
}

pub fn is_destructuring_assignment(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_destructuring_assignment"); 
    if is_assignment_expression(node, true) {
        if let NodeData::BinaryExpression(d) = &node.data {
            return d.left.kind == SyntaxKind::ObjectLiteralExpression
                || d.left.kind == SyntaxKind::ArrayLiteralExpression;
        }
    }
    false
}

pub fn is_dotted_name(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_dotted_name"); 
    match node.kind {
        SyntaxKind::Identifier
        | SyntaxKind::ThisKeyword
        | SyntaxKind::SuperKeyword
        | SyntaxKind::MetaProperty => true,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ParenthesizedExpression => node
            .expression()
            .is_some_and(|expr| is_dotted_name(expr)),
        _ => false,
    }
}

pub fn is_dynamic_name(name: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_dynamic_name"); 
    let expr: Option<Arc<Node>> = match name.kind {
        SyntaxKind::ComputedPropertyName => name.expression().map(Arc::clone),
        SyntaxKind::ElementAccessExpression => match &name.data {
            NodeData::ElementAccessExpression(d) => Some(skip_parentheses(&d.argument_expression)),
            _ => None,
        },
        _ => return false,
    };
    match expr {
        Some(expr) => !is_string_or_numeric_literal_like(&expr) && !is_signed_numeric_literal(&expr),
        None => false,
    }
}

pub fn is_emittable_import(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_emittable_import"); 
    match node.kind {
        SyntaxKind::ImportDeclaration => node_import_clause(node)
            .is_some_and(|clause| !is_type_only(clause)),
        SyntaxKind::ExportDeclaration | SyntaxKind::ImportEqualsDeclaration => {
            !is_type_only(node)
        }
        SyntaxKind::CallExpression => is_import_call(node),
        _ => false,
    }
}
