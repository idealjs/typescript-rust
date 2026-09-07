#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::{Node, SourceFile};
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range};

use super::language_service::LanguageService;
use super::types::SelectionRange;

impl LanguageService {
    pub fn provide_selection_ranges(
        &self,
        document_uri: &DocumentUri,
        positions: &[Position],
    ) -> Vec<SelectionRange> {
        let (_program, source_file) = self.get_program_and_file(document_uri);
        let mut results = Vec::new();
        for position in positions {
            let pos = lsp_position_to_offset(&source_file, position);
            let sr = get_smart_selection_range(&source_file, pos);
            results.push(sr);
        }
        results
    }
}

pub fn get_smart_selection_range(source_file: &Arc<SourceFile>, pos: usize) -> SelectionRange {
    let line_map = &source_file.line_map;

    let node = find_deepest_node(&source_file.node, pos);

    let mut current: Option<SelectionRange> = None;
    let mut node_ref: Option<&Arc<Node>> = Some(&node);

    while let Some(n) = node_ref {
        let start = n.pos();
        let end = n.end();

        if start < end && start <= pos && pos <= end {
            let lsp_range = offset_range_to_lsp_range(line_map, start, end);
            let dup = current
                .as_ref()
                .map(|c| c.range == lsp_range)
                .unwrap_or(false);
            if !dup {
                current = Some(SelectionRange {
                    range: lsp_range,
                    parent: current.map(|c| Box::new(c)),
                });
            }
        }

        node_ref = n.parent.as_ref();
    }

    current.unwrap_or_else(|| SelectionRange {
        range: offset_range_to_lsp_range(line_map, pos, pos),
        parent: None,
    })
}

fn find_deepest_node<'a>(node: &'a Arc<Node>, pos: usize) -> Arc<Node> {
    if pos < node.pos() || pos > node.end() {
        return node.clone();
    }

    let mut deepest: Option<Arc<Node>> = None;
    crate::ast::node_data_generated::for_each_child(node, |child| {
        if child.pos() <= pos && pos <= child.end() {
            deepest = Some(find_deepest_node(child, pos));
            return true;
        }
        false
    });

    deepest.unwrap_or_else(|| node.clone())
}

fn offset_range_to_lsp_range(line_map: &LineMap, start: usize, end: usize) -> Range {
    Range {
        start: offset_to_position(line_map, start),
        end: offset_to_position(line_map, end),
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

fn lsp_position_to_offset(source_file: &Arc<SourceFile>, position: &Position) -> usize {
    let line_map = &source_file.line_map;
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}
