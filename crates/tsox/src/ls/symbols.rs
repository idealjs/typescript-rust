#![allow(dead_code)]

mod names;

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::{Node, SourceFile, SyntaxKind, node_data_generated::for_each_child};
use crate::lsp::lsproto::lsp::{DocumentUri, Range};
use crate::scanner;

use super::language_service::LanguageService;
use super::types::{DocumentSymbol, SymbolKind};

use names::*;

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
