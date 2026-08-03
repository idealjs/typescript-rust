//! Selection ranges provider (1:1 port of Go's `internal/ls/selectionranges.go`).
//!
//! Builds hierarchical selection ranges by walking up the AST parent chain
//! from the deepest node containing the cursor position.

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::{Node, SourceFile};
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range};

use super::language_service::LanguageService;
use super::types::SelectionRange;

impl LanguageService {
    /// Provide selection ranges for positions in a document.
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

/// Compute a smart selection range for a position by walking up the AST.
pub fn get_smart_selection_range(source_file: &Arc<SourceFile>, pos: usize) -> SelectionRange {
    let line_map = &source_file.line_map;

    // Find the deepest node containing the position.
    let node = find_deepest_node(&source_file.node, pos);

    // Build selection ranges by walking up the parent chain.
    let mut current: Option<SelectionRange> = None;
    let mut node_ref: Option<&Arc<Node>> = Some(&node);

    while let Some(n) = node_ref {
        let start = n.pos();
        let end = n.end();

        // Skip empty ranges or ranges that don't contain the position.
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

    // If nothing was found, return the position itself.
    current.unwrap_or_else(|| SelectionRange {
        range: offset_range_to_lsp_range(line_map, pos, pos),
        parent: None,
    })
}

/// Find the deepest AST node containing a byte offset.
fn find_deepest_node<'a>(node: &'a Arc<Node>, pos: usize) -> Arc<Node> {
    // Check if this node contains the position.
    if pos < node.pos() || pos > node.end() {
        return node.clone();
    }

    // Check children via for_each_child.
    let mut deepest: Option<Arc<Node>> = None;
    crate::ast::node_data_generated::for_each_child(node, |child| {
        if child.pos() <= pos && pos <= child.end() {
            deepest = Some(find_deepest_node(child, pos));
            return true; // Stop after first match.
        }
        false
    });

    deepest.unwrap_or_else(|| node.clone())
}

/// Convert byte offsets to an LSP Range using the LineMap.
fn offset_range_to_lsp_range(line_map: &LineMap, start: usize, end: usize) -> Range {
    Range {
        start: offset_to_position(line_map, start),
        end: offset_to_position(line_map, end),
    }
}

/// Convert a byte offset to an LSP Position (0-based line/character).
fn offset_to_position(line_map: &LineMap, offset: usize) -> Position {
    let line = line_of_offset(line_map, offset);
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    Position {
        line: line as u32,
        character: offset.saturating_sub(line_start) as u32,
    }
}

/// Binary search for the line number of a byte offset.
fn line_of_offset(line_map: &LineMap, offset: usize) -> usize {
    match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    }
}

/// Convert an LSP Position to a byte offset.
fn lsp_position_to_offset(source_file: &Arc<SourceFile>, position: &Position) -> usize {
    let line_map = &source_file.line_map;
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}
