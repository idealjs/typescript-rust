#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::{Node, NodeData, SourceFile, SyntaxKind, node_data_generated::for_each_child};
use crate::lsp::lsproto::lsp::{DocumentUri, Range};
use crate::scanner;

use super::language_service::LanguageService;
use super::types::{DocumentSymbol, SymbolKind};

impl LanguageService {

    pub fn provide_document_symbols(&self, document_uri: &DocumentUri) -> Vec<DocumentSymbol> {
        let (_program, source_file) = self.get_program_and_file(document_uri);
        get_document_symbols_for_children(&source_file.node, &source_file)
    }
}

pub fn get_document_symbols_for_children(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<DocumentSymbol> {
    let text = &source_file.text;
    let line_map = &source_file.line_map;
    let mut symbols = Vec::new();

    for_each_child(node, |child| {
        visit_for_symbols(child, text, line_map, &mut symbols);
        false
    });

    symbols
}

fn visit_for_symbols(
    node: &Arc<Node>,
    text: &str,
    line_map: &LineMap,
    symbols: &mut Vec<DocumentSymbol>,
) {
    let kind = node.kind;

    match kind {

        SyntaxKind::ClassDeclaration
        | SyntaxKind::ClassExpression
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::EnumDeclaration => {
            let children = get_children_symbols(node, text, line_map);
            if let Some(sym) = new_document_symbol(node, text, line_map, children) {
                symbols.push(sym);
            }
        }

        SyntaxKind::ModuleDeclaration => {
            let children = get_children_symbols(node, text, line_map);
            if let Some(sym) = new_document_symbol(node, text, line_map, children) {
                symbols.push(sym);
            }
        }

        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::Constructor => {
            let children = get_children_symbols(node, text, line_map);
            if let Some(sym) = new_document_symbol(node, text, line_map, children) {
                symbols.push(sym);
            }
        }

        SyntaxKind::VariableDeclaration => {
            if let Some(sym) = new_document_symbol(node, text, line_map, Vec::new()) {
                symbols.push(sym);
            }
        }

        SyntaxKind::TypeAliasDeclaration => {
            if let Some(sym) = new_document_symbol(node, text, line_map, Vec::new()) {
                symbols.push(sym);
            }
        }

        SyntaxKind::EnumMember => {
            if let Some(sym) = new_document_symbol(node, text, line_map, Vec::new()) {
                symbols.push(sym);
            }
        }

        SyntaxKind::PropertySignature
        | SyntaxKind::MethodSignature
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::ShorthandPropertyAssignment => {
            if let Some(sym) = new_document_symbol(node, text, line_map, Vec::new()) {
                symbols.push(sym);
            }
        }

        SyntaxKind::ImportSpecifier | SyntaxKind::ImportClause => {
            if let Some(sym) = new_document_symbol(node, text, line_map, Vec::new()) {
                symbols.push(sym);
            }
        }

        SyntaxKind::ExportSpecifier => {
            if let Some(sym) = new_document_symbol(node, text, line_map, Vec::new()) {
                symbols.push(sym);
            }
        }

        SyntaxKind::VariableStatement => {
            for_each_child(node, |child| {
                visit_for_symbols(child, text, line_map, symbols);
                false
            });
        }

        SyntaxKind::VariableDeclarationList => {
            for_each_child(node, |child| {
                visit_for_symbols(child, text, line_map, symbols);
                false
            });
        }

        _ => {
            for_each_child(node, |child| {
                visit_for_symbols(child, text, line_map, symbols);
                false
            });
        }
    }
}

fn get_children_symbols(node: &Arc<Node>, text: &str, line_map: &LineMap) -> Vec<DocumentSymbol> {
    let mut children = Vec::new();
    for_each_child(node, |child| {
        visit_for_symbols(child, text, line_map, &mut children);
        false
    });
    children
}

fn new_document_symbol(
    node: &Arc<Node>,
    text: &str,
    line_map: &LineMap,
    children: Vec<DocumentSymbol>,
) -> Option<DocumentSymbol> {
    let name = get_node_name(node, text)?;
    if name.is_empty() {
        return None;
    }

    let node_start = scanner::skip_trivia(text, node.pos());
    let node_end = node.end();
    let kind = symbol_kind_from_node(node.kind);

    let (name_start, name_end) = get_name_range(node, text, node_start);

    Some(DocumentSymbol {
        name,
        detail: None,
        kind,
        range: offset_range_to_lsp_range(line_map, node_start, node_end),
        selection_range: offset_range_to_lsp_range(line_map, name_start, name_end),
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
        tags: None,
        deprecated: None,
    })
}

fn get_node_name(node: &Arc<Node>, text: &str) -> Option<String> {
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

fn get_name_range(node: &Arc<Node>, text: &str, node_start: usize) -> (usize, usize) {
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

fn identifier_text(node: &Arc<Node>, text: &str) -> String {
    text[node.pos()..node.end()].trim().to_string()
}

fn property_name_text(node: &Arc<Node>, text: &str) -> String {
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

fn binding_name_text(node: &Arc<Node>, text: &str) -> String {
    match &node.data {
        NodeData::Identifier(d) => d.text.clone(),
        _ => text[node.pos()..node.end()].trim().to_string(),
    }
}

fn module_name_text(node: &Arc<Node>, text: &str) -> String {
    text[node.pos()..node.end()].trim().to_string()
}

fn symbol_kind_from_node(kind: SyntaxKind) -> SymbolKind {
    match kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => SymbolKind::Class,
        SyntaxKind::InterfaceDeclaration => SymbolKind::Interface,
        SyntaxKind::EnumDeclaration => SymbolKind::Enum,
        SyntaxKind::EnumMember => SymbolKind::EnumMember,
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction => SymbolKind::Function,
        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => SymbolKind::Method,
        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => SymbolKind::Property,
        SyntaxKind::Constructor => SymbolKind::Constructor,
        SyntaxKind::VariableDeclaration => SymbolKind::Variable,
        SyntaxKind::TypeAliasDeclaration => SymbolKind::TypeParameter,
        SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::ShorthandPropertyAssignment => SymbolKind::Property,
        SyntaxKind::ModuleDeclaration => SymbolKind::Namespace,
        SyntaxKind::ImportSpecifier | SyntaxKind::ImportClause | SyntaxKind::ExportSpecifier => {
            SymbolKind::Module
        }
        _ => SymbolKind::Variable,
    }
}

fn offset_range_to_lsp_range(line_map: &LineMap, start: usize, end: usize) -> Range {
    Range {
        start: offset_to_position(line_map, start),
        end: offset_to_position(line_map, end),
    }
}

fn offset_to_position(line_map: &LineMap, offset: usize) -> crate::lsp::lsproto::lsp::Position {
    let line = line_of_offset(line_map, offset);
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    crate::lsp::lsproto::lsp::Position {
        line: line as u32,
        character: offset.saturating_sub(line_start) as u32,
    }
}

fn line_of_offset(line_map: &LineMap, offset: usize) -> usize {
    match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    }
}
