#![allow(dead_code)]

use std::sync::Arc;

use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp::Position;
use crate::lsp::lsproto_lsp::Range;
use tsox_checker::checker::Checker;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::node::LineMap;
use tsox_frontend::ast::node_data_generated::for_each_child;
use tsox_frontend::scanner;

use super::language_service::LanguageService;
use super::types::LocationLink;

impl LanguageService {
    pub fn provide_definition(
        &self,
        document_uri: &DocumentUri,
        position: Position,
    ) -> Vec<LocationLink> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;

        let offset = lsp_position_to_offset(line_map, &position);

        let node = find_deepest_node(&source_file.node, offset);

        let checker = program.build_checker();
        let declarations = get_declarations_from_location(&checker, &node);

        if declarations.is_empty() {
            return Vec::new();
        }

        let origin_selection_range = node_range_to_lsp_range(line_map, &node);

        declarations
            .iter()
            .filter_map(|decl| {
                let decl_file = get_source_file_of_node(decl, &source_file);
                let decl_line_map = &decl_file.line_map;
                let target_range = node_range_to_lsp_range(decl_line_map, decl);
                let target_selection_range =
                    name_range_to_lsp_range(decl_line_map, decl, &decl_file.text);

                Some(LocationLink {
                    origin_selection_range: Some(origin_selection_range.clone()),
                    target_uri: DocumentUri(decl_file.file_name.clone()),
                    target_range,
                    target_selection_range,
                })
            })
            .collect()
    }

    pub fn provide_type_definition(
        &self,
        document_uri: &DocumentUri,
        position: Position,
    ) -> Vec<LocationLink> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;
        let offset = lsp_position_to_offset(line_map, &position);
        let node = find_deepest_node(&source_file.node, offset);
        let mut checker = program.build_checker();

        let ty = checker.get_type_of_node(&node);
        let declarations = get_declarations_from_type(&ty);
        if !declarations.is_empty() {
            let origin_selection_range = node_range_to_lsp_range(line_map, &node);
            return declarations
                .iter()
                .filter_map(|decl| {
                    let decl_file = get_source_file_of_node(decl, &source_file);
                    let decl_line_map = &decl_file.line_map;
                    Some(LocationLink {
                        origin_selection_range: Some(origin_selection_range.clone()),
                        target_uri: DocumentUri(decl_file.file_name.clone()),
                        target_range: node_range_to_lsp_range(decl_line_map, decl),
                        target_selection_range: name_range_to_lsp_range(
                            decl_line_map,
                            decl,
                            &decl_file.text,
                        ),
                    })
                })
                .collect();
        }

        Vec::new()
    }
}

pub fn get_declarations_from_location(checker: &Checker, node: &Arc<Node>) -> Vec<Arc<Node>> {
    if let Some(symbol) = checker.get_symbol_at_location(node) {
        return symbol.declarations.clone();
    }
    Vec::new()
}

pub fn try_get_signature_declaration(_checker: &Checker, _node: &Arc<Node>) -> Option<Arc<Node>> {
    None
}

pub fn get_declarations_from_type(ty: &tsox_checker::checker::Type) -> Vec<Arc<Node>> {
    if let Some(symbol) = ty.symbol.as_ref() {
        return symbol.declarations.clone();
    }
    Vec::new()
}

fn find_deepest_node(node: &Arc<Node>, offset: usize) -> Arc<Node> {
    let mut deepest = Arc::clone(node);
    loop {
        let current = Arc::clone(&deepest);
        let mut next: Option<Arc<Node>> = None;
        for_each_child(&current, |child| {
            if child.pos() <= offset && offset < child.end() {
                next = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        match next {
            Some(child) => deepest = child,
            None => break,
        }
    }
    deepest
}

fn get_source_file_of_node(node: &Arc<Node>, fallback: &Arc<SourceFile>) -> Arc<SourceFile> {
    let _ = node;
    Arc::clone(fallback)
}

fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}

fn node_range_to_lsp_range(line_map: &LineMap, node: &Arc<Node>) -> Range {
    Range {
        start: offset_to_position(line_map, node.pos()),
        end: offset_to_position(line_map, node.end()),
    }
}

fn name_range_to_lsp_range(line_map: &LineMap, node: &Arc<Node>, text: &str) -> Range {
    let node_start = tsox_frontend::scanner::skip_trivia(text, node.pos());

    let name_node: Option<&Arc<Node>> = match &node.data {
        tsox_frontend::ast::NodeData::ClassDeclaration(d) => d.name.as_ref(),
        tsox_frontend::ast::NodeData::InterfaceDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::EnumDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::FunctionDeclaration(d) => d.name.as_ref(),
        tsox_frontend::ast::NodeData::FunctionExpression(d) => d.name.as_ref(),
        tsox_frontend::ast::NodeData::VariableDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::TypeAliasDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::MethodDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::PropertyDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::MethodSignatureDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::EnumMember(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::ModuleDeclaration(d) => Some(&d.name),
        tsox_frontend::ast::NodeData::ImportSpecifier(d) => Some(&d.name),
        _ => None,
    };

    if let Some(name) = name_node {
        let start = tsox_frontend::scanner::skip_trivia(text, name.pos());
        let end = name.end();
        return Range {
            start: offset_to_position(line_map, start),
            end: offset_to_position(line_map, end),
        };
    }

    let pos = offset_to_position(line_map, node_start);
    Range {
        start: pos.clone(),
        end: pos,
    }
}

fn offset_to_position(line_map: &LineMap, offset: usize) -> Position {
    let line = line_of_offset(line_map, offset);
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    Position {
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
