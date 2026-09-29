use crate::ast::*;
use std::sync::Arc;

use super::m3c::type_arguments;
use super::m3f_4::is_declaration_name;
use super::m3g::is_literal_computed_property_declaration_name;
use super::m3h::is_argument_of_element_access_expression;
use super::w5::get_leftmost_access_expression;

fn node_is_type_only(node: &Node) -> bool {
    match &node.data {
        NodeData::ExportSpecifier(d) => d.is_type_only,
        NodeData::ImportSpecifier(d) => d.is_type_only,
        NodeData::ExportDeclaration(d) => d.is_type_only,
        NodeData::ImportEqualsDeclaration(d) => d.is_type_only,
        NodeData::ImportClause(d) => d.phase_modifier == Some(SyntaxKind::TypeKeyword),
        _ => false,
    }
}

pub fn is_part_of_type_node_in_parent(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind == SyntaxKind::TypeQuery {
        return false;
    }
    if parent.kind == SyntaxKind::ImportType {
        return match &parent.data {
            NodeData::ImportTypeNode(d) => !d.is_type_of,
            _ => false,
        };
    }
    if (parent.kind as i16) >= (SyntaxKind::TypePredicate as i16)
        && (parent.kind as i16) <= (SyntaxKind::ImportType as i16)
    {
        return true;
    }
    match parent.kind {
        SyntaxKind::ExpressionWithTypeArguments => {
            is_part_of_type_expression_with_type_arguments(&parent)
        }
        SyntaxKind::TypeParameter => match &parent.data {
            NodeData::TypeParameterDeclaration(d) => d
                .constraint
                .as_ref()
                .is_some_and(|c| Arc::ptr_eq(c, node)),
            _ => false,
        },
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::Constructor
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::IndexSignature
        | SyntaxKind::TypeAssertionExpression => {
            parent.type_node().is_some_and(|t| Arc::ptr_eq(t, node))
        }
        SyntaxKind::CallExpression
        | SyntaxKind::NewExpression
        | SyntaxKind::TaggedTemplateExpression => type_arguments(&parent)
            .iter()
            .any(|t| Arc::ptr_eq(t, node)),
        _ => false,
    }
}

pub fn is_part_of_type_expression_with_type_arguments(node: &Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if is_heritage_clause(&parent) {
        let parent_parent_is_class_like = parent.parent().is_some_and(|gp| is_class_like(&gp));
        let token_implements = match &parent.data {
            NodeData::HeritageClause(d) => d.token == SyntaxKind::ImplementsKeyword,
            _ => false,
        };
        return !parent_parent_is_class_like || token_implements;
    }
    is_jsdoc_implements_tag(&parent) || is_jsdoc_augments_tag(&parent)
}

pub fn is_shorthand_property_name_use_site(use_site: &Arc<Node>) -> bool {
    if !is_identifier(use_site) {
        return false;
    }
    match use_site.parent() {
        Some(parent) => {
            is_shorthand_property_assignment(&parent)
                && parent.name().is_some_and(|name| Arc::ptr_eq(name, use_site))
        }
        None => false,
    }
}

pub fn is_type_only_export_declaration(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::ExportSpecifier => {
            node_is_type_only(node)
                || node
                    .parent()
                    .and_then(|p| p.parent())
                    .is_some_and(|gp| node_is_type_only(&gp))
        }
        SyntaxKind::ExportDeclaration => {
            match &node.data {
                NodeData::ExportDeclaration(d) => {
                    d.is_type_only && d.module_specifier.is_some() && d.export_clause.is_none()
                }
                _ => false,
            }
        }
        SyntaxKind::NamespaceExport => node.parent().is_some_and(|p| node_is_type_only(&p)),
        _ => false,
    }
}

pub fn is_variable_declaration_initialized_with_require_helper(
    node: &Node,
    allow_accessed_require: bool,
) -> bool {
    if !is_in_js_file(node) {
        return false;
    }
    if node.kind != SyntaxKind::VariableDeclaration {
        return false;
    }
    let initializer: Option<&Arc<Node>> = match &node.data {
        NodeData::VariableDeclaration(d) => d.initializer.as_ref(),
        _ => None,
    };
    let Some(initializer) = initializer else {
        return false;
    };
    let initializer: Arc<Node> = if allow_accessed_require {
        get_leftmost_access_expression(initializer)
    } else {
        Arc::clone(initializer)
    };
    let parent_parent_exported = node
        .parent()
        .and_then(|p| p.parent())
        .is_some_and(|gp| gp.has_syntactic_modifier(ModifierFlags::Export));
    !parent_parent_exported && node.type_node().is_none() && is_require_call(&initializer, true)
}

pub fn literal_is_name(node: &Arc<Node>) -> bool {
    is_declaration_name(node)
        || node
            .parent()
            .is_some_and(|p| p.kind == SyntaxKind::ExternalModuleReference)
        || is_argument_of_element_access_expression(node)
        || is_literal_computed_property_declaration_name(node)
}

fn set_parent_in_children_visit(node: &Arc<Node>, parent: &mut Option<Arc<Node>>) -> bool {
    if let Some(p) = parent.as_ref() {
        node.set_parent(p);
    }
    let save_parent = parent.take();
    *parent = Some(Arc::clone(node));
    for_each_child(node, |n| {
        set_parent_in_children_visit(n, parent);
        false
    });
    *parent = save_parent;
    false
}

pub fn new_parent_in_children_setter() -> impl FnMut(&Arc<Node>) -> bool {
    let mut state: Option<Arc<Node>> = None;
    move |node| set_parent_in_children_visit(node, &mut state)
}

pub fn push_ancestor(ancestors: &[Arc<Node>], parent: Arc<Node>) -> Vec<Arc<Node>> {
    let mut result = ancestors.to_vec();
    result.push(parent);
    result
}

pub fn pop_ancestor(ancestors: &[Arc<Node>], node: &Node) -> (Vec<Arc<Node>>, Option<Arc<Node>>) {
    if ancestors.is_empty() {
        return (Vec::new(), node.parent());
    }
    let n = ancestors.len() - 1;
    (ancestors[..n].to_vec(), Some(Arc::clone(&ancestors[n])))
}

pub fn select_expression_of_call_or_new_expression_or_decorator(
    node: &Arc<Node>,
) -> Option<Arc<Node>> {
    if is_call_expression(node) || is_new_expression(node) || is_decorator(node) {
        return node.expression().cloned();
    }
    None
}

pub fn select_tag_of_tagged_template_expression(node: &Arc<Node>) -> Option<Arc<Node>> {
    if is_tagged_template_expression(node) {
        if let NodeData::TaggedTemplateExpression(d) = &node.data {
            return Some(d.tag.clone());
        }
    }
    None
}

pub fn select_tag_name_of_jsx_opening_like_element(node: &Arc<Node>) -> Option<Arc<Node>> {
    if is_jsx_opening_element(node) || is_jsx_self_closing_element(node) {
        return Some(super::m3c::tag_name(node).clone());
    }
    None
}
