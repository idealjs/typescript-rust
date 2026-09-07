use crate::ast::*;
use std::sync::Arc;

pub fn is_identifier_name(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::EnumMember
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::PropertyAccessExpression => {
            parent.name().is_some_and(|n| Arc::ptr_eq(n, node))
        }
        SyntaxKind::QualifiedName => {
            matches!(&parent.data, NodeData::QualifiedName(q) if Arc::ptr_eq(&q.right, node))
        }
        SyntaxKind::BindingElement => {
            matches!(&parent.data, NodeData::BindingElement(b) if b
                .property_name
                .as_ref()
                .is_some_and(|n| Arc::ptr_eq(n, node)))
        }
        SyntaxKind::ImportSpecifier => {
            matches!(&parent.data, NodeData::ImportSpecifier(i) if i
                .property_name
                .as_ref()
                .is_some_and(|n| Arc::ptr_eq(n, node)))
        }
        SyntaxKind::ExportSpecifier
        | SyntaxKind::JsxAttribute
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxClosingElement => true,
        _ => false,
    }
}

pub fn is_in_top_level_context(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return true;
    };
    !find_ancestor(parent, is_function_like).is_some()
}

pub fn is_module_with_string_literal_name(node: &Node) -> bool {
    is_module_declaration(node)
        && node
            .name()
            .map(|n| n.kind == SyntaxKind::StringLiteral)
            .unwrap_or(false)
}

pub fn is_ambient_module(node: &Node) -> bool {
    if !is_module_declaration(node) {
        return false;
    }
    match &node.data {
        NodeData::ModuleDeclaration(d) => {
            d.name.kind == SyntaxKind::StringLiteral || is_global_scope_augmentation(node)
        }
        _ => false,
    }
}

pub fn is_global_scope_augmentation(node: &Node) -> bool {
    if !is_module_declaration(node) {
        return false;
    }
    if let NodeData::ModuleDeclaration(d) = &node.data {
        return d.keyword == SyntaxKind::GlobalKeyword;
    }
    false
}

pub fn is_ambient_module_symbol_name(s: &str) -> bool {
    s.starts_with('"') && s.ends_with('"')
}
