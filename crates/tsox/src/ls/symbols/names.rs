use std::sync::Arc;

use crate::ast::{Node, NodeData};
use crate::scanner;

pub(super) fn get_node_name(node: &Arc<Node>, text: &str) -> Option<String> {
    match &node.data {
        NodeData::ClassDeclaration(d) => d.name.as_ref().map(|n| identifier_text(n, text)),
        NodeData::InterfaceDeclaration(d) => Some(identifier_text(&d.name, text)),
        NodeData::EnumDeclaration(d) => Some(identifier_text(&d.name, text)),
        NodeData::FunctionDeclaration(d) => d.name.as_ref().map(|n| identifier_text(n, text)),
        NodeData::FunctionExpression(d) => d.name.as_ref().map(|n| identifier_text(n, text)),
        NodeData::MethodDeclaration(d) => Some(property_name_text(&d.name, text)),
        NodeData::GetAccessorDeclaration(d) => Some(property_name_text(&d.name, text)),
        NodeData::SetAccessorDeclaration(d) => Some(property_name_text(&d.name, text)),
        NodeData::ConstructorDeclaration(_) => Some("constructor".to_string()),
        NodeData::VariableDeclaration(d) => Some(binding_name_text(&d.name, text)),
        NodeData::TypeAliasDeclaration(d) => Some(identifier_text(&d.name, text)),
        NodeData::EnumMember(d) => Some(property_name_text(&d.name, text)),
        NodeData::PropertyDeclaration(d) => Some(property_name_text(&d.name, text)),
        NodeData::PropertySignatureDeclaration(d) => Some(property_name_text(&d.name, text)),
        NodeData::MethodSignatureDeclaration(d) => Some(property_name_text(&d.name, text)),
        NodeData::PropertyAssignment(d) => Some(property_name_text(&d.name, text)),
        NodeData::ModuleDeclaration(d) => Some(module_name_text(&d.name, text)),
        NodeData::ImportSpecifier(d) => Some(identifier_text(&d.name, text)),
        NodeData::ImportClause(d) => d.name.as_ref().map(|n| identifier_text(n, text)),
        NodeData::ExportSpecifier(d) => Some(identifier_text(
            d.property_name.as_ref().unwrap_or(&d.name),
            text,
        )),
        _ => None,
    }
}

pub(super) fn get_name_range(node: &Arc<Node>, text: &str, node_start: usize) -> (usize, usize) {
    let name_ref: Option<&Arc<Node>> = match &node.data {
        NodeData::ClassDeclaration(d) => d.name.as_ref(),
        NodeData::FunctionDeclaration(d) => d.name.as_ref(),
        NodeData::VariableDeclaration(d) => Some(&d.name),
        NodeData::InterfaceDeclaration(d) => Some(&d.name),
        NodeData::EnumDeclaration(d) => Some(&d.name),
        NodeData::TypeAliasDeclaration(d) => Some(&d.name),
        _ => None,
    };

    if let Some(name) = name_ref {
        let start = scanner::skip_trivia(text, name.pos());
        let end = name.end();
        return (start.max(node_start), end.max(node_start));
    }

    (node_start, node_start)
}

pub(super) fn identifier_text(node: &Arc<Node>, text: &str) -> String {
    text[node.pos()..node.end()].trim().to_string()
}

pub(super) fn property_name_text(node: &Arc<Node>, text: &str) -> String {
    match &node.data {
        NodeData::Identifier(d) => d.text.clone(),
        NodeData::StringLiteral(d) => format!("\"{}\"", d.text),
        NodeData::NumericLiteral(d) => d.text.clone(),
        NodeData::ComputedPropertyName(d) => {
            let inner = &d.expression;
            text[inner.pos()..inner.end()].to_string()
        }
        _ => text[node.pos()..node.end()].trim().to_string(),
    }
}

pub(super) fn binding_name_text(node: &Arc<Node>, text: &str) -> String {
    match &node.data {
        NodeData::Identifier(d) => d.text.clone(),
        _ => text[node.pos()..node.end()].trim().to_string(),
    }
}

pub(super) fn module_name_text(node: &Arc<Node>, text: &str) -> String {
    text[node.pos()..node.end()].trim().to_string()
}
