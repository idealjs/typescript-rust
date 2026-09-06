#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, SourceFile};
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range};

use super::language_service::LanguageService;
use super::types::{DocumentHighlight, DocumentHighlightKind, MultiDocumentHighlight};

impl LanguageService {

    pub fn provide_document_highlights(
        &self,
        document_uri: &DocumentUri,
        document_position: Position,
    ) -> Vec<DocumentHighlight> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;
        let offset = lsp_position_to_offset(line_map, &document_position);

        let node = find_deepest_node(&source_file.node, offset);

        let mut checker = program.build_checker();

        let symbol = match checker.get_symbol_at_location(&node) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let target = checker.skip_alias(&symbol);

        let references = checker.get_references_to_symbol_in_file(&source_file, &target);

        references
            .iter()
            .map(|ref_node| DocumentHighlight {
                range: node_range_to_lsp_range(line_map, ref_node),
                kind: Some(DocumentHighlightKind::Text),
            })
            .collect()
    }

    pub fn provide_multi_document_highlights(
        &self,
        document_uri: &DocumentUri,
        document_position: Position,
        files_to_search: &[DocumentUri],
    ) -> Vec<MultiDocumentHighlight> {

        let highlights = self.provide_document_highlights(document_uri, document_position.clone());
        let primary = MultiDocumentHighlight {
            uri: DocumentUri(document_uri.0.clone()),
            highlights,
        };

        let mut result = vec![primary];

        for uri in files_to_search {
            if uri.0 == document_uri.0 {
                continue;
            }
            let highlights = self.provide_document_highlights(uri, document_position.clone());
            if !highlights.is_empty() {
                result.push(MultiDocumentHighlight {
                    uri: DocumentUri(uri.0.clone()),
                    highlights,
                });
            }
        }

        result
    }

    pub fn get_semantic_document_highlights(
        &self,
        _position: usize,
        node: &Arc<Node>,
        program: &Arc<Program>,
        source_files: &[Arc<SourceFile>],
    ) -> Vec<MultiDocumentHighlight> {
        let mut checker = program.build_checker();
        let symbol = match checker.get_symbol_at_location(node) {
            Some(s) => s,
            None => return Vec::new(),
        };
        let target = checker.skip_alias(&symbol);

        source_files
            .iter()
            .filter_map(|file| {
                let refs = checker.get_references_to_symbol_in_file(file, &target);
                if refs.is_empty() {
                    return None;
                }
                let line_map = &file.line_map;
                let highlights: Vec<DocumentHighlight> = refs
                    .iter()
                    .map(|ref_node| DocumentHighlight {
                        range: node_range_to_lsp_range(line_map, ref_node),
                        kind: Some(DocumentHighlightKind::Text),
                    })
                    .collect();
                Some(MultiDocumentHighlight {
                    uri: DocumentUri(file.file_name.clone()),
                    highlights,
                })
            })
            .collect()
    }

    pub fn get_syntactic_document_highlights(
        &self,
        node: &Arc<Node>,
        source_file: &Arc<SourceFile>,
    ) -> Vec<DocumentHighlight> {

        let text = node.text();
        if text.is_empty() {
            return Vec::new();
        }
        let line_map = &source_file.line_map;
        let mut result = Vec::new();
        collect_matching_identifiers(&source_file.node, text, &mut |n| {
            result.push(DocumentHighlight {
                range: node_range_to_lsp_range(line_map, n),
                kind: Some(DocumentHighlightKind::Text),
            });
        });
        result
    }
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

fn collect_matching_identifiers(node: &Arc<Node>, name: &str, cb: &mut impl FnMut(&Arc<Node>)) {
    use crate::ast::SyntaxKind;
    if node.kind == SyntaxKind::Identifier && node.text() == name {
        cb(node);
    }
    for_each_child(node, |child| {
        collect_matching_identifiers(child, name, cb);
        false
    });
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
