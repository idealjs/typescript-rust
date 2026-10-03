use crate::ast::*;
use std::sync::Arc;
use tsox_core::core::compiler_options::ModuleKind;

use super::m3f_3::is_argument_expression_of_element_access;
use super::m3g_2::is_right_side_of_property_access;
use super::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use super::x6a::is_entity_name_expression_ex;

pub fn climb_past_property_access(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("climb_past_property_access"); 
    if is_right_side_of_property_access(node) {
        return node.parent().unwrap_or_else(|| Arc::clone(node));
    }
    Arc::clone(node)
}

pub fn try_get_text_of_property_name(name: &Node) -> Option<String> { ::tsox_core::fntrace::enter("try_get_text_of_property_name"); 
    match name.kind {
        SyntaxKind::Identifier
        | SyntaxKind::PrivateIdentifier
        | SyntaxKind::StringLiteral
        | SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral => Some(name.text().to_string()),
        SyntaxKind::ComputedPropertyName => {
            let expr = name.expression()?;
            if is_string_or_numeric_literal_like(expr) {
                Some(expr.text().to_string())
            } else {
                None
            }
        }
        SyntaxKind::JsxNamespacedName => name.jsx_namespaced_name_text(),
        _ => None,
    }
}

pub fn walk_up_binding_elements_and_patterns(binding: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("walk_up_binding_elements_and_patterns"); 
    let mut node = binding.parent()?;
    while node.parent().is_some_and(|p| is_binding_element(&p)) {
        node = node.parent().and_then(|p| p.parent())?;
    }
    node.parent()
}

pub fn walk_up_parenthesized_expressions(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("walk_up_parenthesized_expressions"); 
    let mut current = Some(Arc::clone(node));
    while let Some(n) = &current {
        if n.kind != SyntaxKind::ParenthesizedExpression {
            return current;
        }
        current = n.parent();
    }
    None
}

pub fn walk_up_parenthesized_types(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("walk_up_parenthesized_types"); 
    let mut current = Some(Arc::clone(node));
    while let Some(n) = &current {
        if n.kind != SyntaxKind::ParenthesizedType {
            return current;
        }
        current = n.parent();
    }
    None
}

pub fn climb_past_property_or_element_access(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("climb_past_property_or_element_access"); 
    if is_right_side_of_property_access(node) || is_argument_expression_of_element_access(node) {
        return node.parent().unwrap_or_else(|| Arc::clone(node));
    }
    Arc::clone(node)
}

pub fn find_clone_in_node(node: &Arc<Node>, original: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_clone_in_node"); 
    let mut node = Arc::clone(node);
    loop {
        if node.kind == original.kind && node.loc == original.loc {
            return Some(node);
        }
        let mut next: Option<Arc<Node>> = None;
        for_each_child(&node, |n| {
            if original.loc.contained_by(&n.loc) {
                next = Some(Arc::clone(n));
                return true;
            }
            false
        });
        node = next?;
    }
}

pub fn get_import_type_node_literal(node: &Node) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("get_import_type_node_literal"); 
    if !is_import_type_node(node) {
        return None;
    }
    let NodeData::ImportTypeNode(import_type_node) = &node.data else {
        return None;
    };
    if !is_literal_type_node(&import_type_node.argument) {
        return None;
    }
    let NodeData::LiteralTypeNode(literal_type_node) = &import_type_node.argument.data else {
        return None;
    };
    if is_string_literal(&literal_type_node.literal) {
        Some(&literal_type_node.literal)
    } else {
        None
    }
}

pub fn get_question_dot_token(node: &Node) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("get_question_dot_token"); 
    match &node.data {
        NodeData::PropertyAccessExpression(d) => d.question_dot_token.as_ref(),
        NodeData::ElementAccessExpression(d) => d.question_dot_token.as_ref(),
        NodeData::CallExpression(d) => d.question_dot_token.as_ref(),
        _ => None,
    }
}

pub fn has_comment(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("has_comment"); 
    matches!(
        kind,
        SyntaxKind::JSDoc
            | SyntaxKind::JSDocUnknownTag
            | SyntaxKind::JSDocAugmentsTag
            | SyntaxKind::JSDocImplementsTag
            | SyntaxKind::JSDocDeprecatedTag
            | SyntaxKind::JSDocPublicTag
            | SyntaxKind::JSDocPrivateTag
            | SyntaxKind::JSDocProtectedTag
            | SyntaxKind::JSDocReadonlyTag
            | SyntaxKind::JSDocOverrideTag
            | SyntaxKind::JSDocCallbackTag
            | SyntaxKind::JSDocOverloadTag
            | SyntaxKind::JSDocParameterTag
            | SyntaxKind::JSDocPropertyTag
            | SyntaxKind::JSDocReturnTag
            | SyntaxKind::JSDocThisTag
            | SyntaxKind::JSDocTypeTag
            | SyntaxKind::JSDocTemplateTag
            | SyntaxKind::JSDocTypedefTag
            | SyntaxKind::JSDocSeeTag
            | SyntaxKind::JSDocThrowsTag
            | SyntaxKind::JSDocSatisfiesTag
            | SyntaxKind::JSDocImportTag
    )
}

pub fn is_argument_of_element_access_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_argument_of_element_access_expression"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    match &parent.data {
        NodeData::ElementAccessExpression(d) => Arc::ptr_eq(&d.argument_expression, node),
        _ => false,
    }
}

pub fn is_callee_worker(
    node: &Arc<Node>,
    pred: &dyn Fn(&Node) -> bool,
    callee_selector: &dyn Fn(&Arc<Node>) -> Option<Arc<Node>>,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_callee_worker"); 
    let mut target: Arc<Node> = if include_element_access {
        climb_past_property_or_element_access(node)
    } else {
        climb_past_property_access(node)
    };
    if skip_past_outer_expressions && is_expression(&target) {
        target = skip_outer_expressions(&target, OuterExpressionKinds::ALL);
    }
    let Some(parent) = target.parent() else {
        return false;
    };
    pred(&parent) && callee_selector(&parent).is_some_and(|selected| Arc::ptr_eq(&selected, &target))
}

pub fn is_common_js_containing_module_kind(kind: ModuleKind) -> bool { ::tsox_core::fntrace::enter("is_common_js_containing_module_kind"); 
    kind == ModuleKind::CommonJS || (ModuleKind::Node16 <= kind && kind <= ModuleKind::NodeNext)
}

pub fn is_element_access_entity_name_expression(node: &Node, allow_js: bool) -> bool { ::tsox_core::fntrace::enter("is_element_access_entity_name_expression"); 
    if !is_element_access_expression(node) {
        return false;
    }
    let argument_is_literal_like = match &node.data {
        NodeData::ElementAccessExpression(d) => is_string_or_numeric_literal_like(&d.argument_expression),
        _ => false,
    };
    argument_is_literal_like
        && node
            .expression()
            .is_some_and(|expr| is_entity_name_expression_ex(expr, allow_js))
}

pub fn is_identifier_in_non_emitting_heritage_clause(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_identifier_in_non_emitting_heritage_clause"); 
    if !is_identifier(node) {
        return false;
    }
    let Some(mut parent) = node.parent() else {
        return false;
    };
    while is_property_access_expression(&parent) || is_expression_with_type_arguments(&parent) {
        match parent.parent() {
            Some(p) => parent = p,
            None => return false,
        }
    }
    if !is_heritage_clause(&parent) {
        return false;
    }
    let token = match &parent.data {
        NodeData::HeritageClause(d) => d.token,
        _ => return false,
    };
    if token == SyntaxKind::ImplementsKeyword {
        return true;
    }
    parent.parent().is_some_and(|gp| is_interface_declaration(&gp))
}

pub fn is_part_of_possibly_valid_type_or_abstract_computed_property_name(
    node: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_part_of_possibly_valid_type_or_abstract_computed_property_name"); 
    let mut node = Arc::clone(node);
    while matches!(
        node.kind,
        SyntaxKind::Identifier | SyntaxKind::PropertyAccessExpression
    ) {
        match node.parent() {
            Some(parent) => node = parent,
            None => return false,
        }
    }
    if node.kind != SyntaxKind::ComputedPropertyName {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.has_syntactic_modifier(ModifierFlags::Abstract) {
        return true;
    }
    parent.parent().is_some_and(|gp| {
        matches!(
            gp.kind,
            SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeLiteral
        )
    })
}
