use crate::ast::*;
use std::sync::Arc;

pub fn is_void_zero(node: &Node) -> bool {
    if !is_void_expression(node) {
        return false;
    }
    match node.expression() {
        Some(expr) => is_numeric_literal(expr) && expr.text() == "0",
        None => false,
    }
}

pub fn is_exports_identifier(node: &Node) -> bool {
    is_identifier(node) && node.text() == "exports"
}

pub fn is_module_identifier(node: &Node) -> bool {
    is_identifier(node) && node.text() == "module"
}

pub fn is_this_identifier(node: Option<&Node>) -> bool {
    match node {
        Some(n) => is_identifier(n) && n.text() == "this",
        None => false,
    }
}

pub fn is_super_call(node: &Node) -> bool {
    if !is_call_expression(node) {
        return false;
    }
    match node.expression() {
        Some(expr) => expr.kind == SyntaxKind::SuperKeyword,
        None => false,
    }
}

pub fn is_import_call(node: &Node) -> bool {
    if !is_call_expression(node) {
        return false;
    }
    match node.expression() {
        Some(expr) => expr.kind == SyntaxKind::ImportKeyword,
        None => false,
    }
}

pub fn is_instance_of_expression(node: &Node) -> bool {
    if let NodeData::BinaryExpression(d) = &node.data {
        return d.operator_token.kind == SyntaxKind::InstanceOfKeyword;
    }
    false
}

pub fn is_any_import_or_re_export(node: &Node) -> bool {
    is_import_node(node) || is_export_declaration(node)
}

pub fn is_import_node(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::JSImportDeclaration
    )
}

pub fn is_any_import_syntax(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportDeclaration | SyntaxKind::ImportEqualsDeclaration
    )
}

pub fn is_question_token(node: Option<&Node>) -> bool {
    match node {
        Some(n) => n.kind == SyntaxKind::QuestionToken,
        None => false,
    }
}

pub fn is_jsx_tag_name(node: &Arc<Node>) -> bool {
    let parent = match &node.parent {
        Some(p) => p,
        None => return false,
    };
    match parent.kind {
        SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxClosingElement
        | SyntaxKind::JsxSelfClosingElement => match &parent.data {
            NodeData::JsxOpeningElement(d) => Arc::ptr_eq(&d.tag_name, node),
            NodeData::JsxClosingElement(d) => Arc::ptr_eq(&d.tag_name, node),
            NodeData::JsxSelfClosingElement(d) => Arc::ptr_eq(&d.tag_name, node),
            _ => false,
        },
        _ => false,
    }
}
