//! Linked editing ranges (1:1 port of Go's `internal/ls/linkedediting.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::{NodeData, for_each_child};
use crate::ast::{Node, SyntaxKind};
use crate::lsp::lsproto::lsp::{Position, Range};

use super::language_service::LanguageService;
use super::types::{LinkedEditingRangeParams, LinkedEditingRanges};

/// JSX tag word pattern for linked editing.
pub const JSX_TAG_WORD_PATTERN: &str = "[a-zA-Z0-9:\\-\\._$]*";

impl LanguageService {
    /// Provide linked editing ranges for a position.
    ///
    /// Mirrors `ProvideLinkedEditingRange`:
    /// 1. Find the node at the cursor position.
    /// 2. If it is a JSX tag name, find the matching opening/closing tag.
    /// 3. Return `LinkedEditingRanges`.
    pub fn provide_linked_editing_range(
        &self,
        params: &LinkedEditingRangeParams,
    ) -> Option<LinkedEditingRanges> {
        let (program, source_file) = self.get_program_and_file(&params.text_document.uri);
        let line_map = &source_file.line_map;

        // Convert LSP position to byte offset.
        let offset = lsp_position_to_offset(line_map, &params.position);

        // Find the deepest AST node at that offset.
        let node = find_deepest_node(&source_file.node, offset);
        let _ = &program;

        // Resolve the JSX tag name node and its counterpart.
        let ranges = find_jsx_linked_ranges(&node, line_map);

        if ranges.is_empty() {
            return None;
        }

        Some(LinkedEditingRanges {
            ranges,
            word_pattern: Some(JSX_TAG_WORD_PATTERN.to_string()),
        })
    }
}

/// Find linked editing ranges for a JSX tag.
///
/// Given a node that is (or is within) a JSX tag name, return the ranges of
/// the matching opening and closing tag names.
fn find_jsx_linked_ranges(node: &Arc<Node>, line_map: &LineMap) -> Vec<Range> {
    // Walk up to find the containing JSX tag node (opening, closing, or self-closing).
    let tag_node = find_jsx_tag_ancestor(node);

    let tag_node = match tag_node {
        Some(n) => n,
        None => return Vec::new(),
    };

    match tag_node.kind {
        SyntaxKind::JsxOpeningElement => {
            // Get the tag name range.
            let opening_range = jsx_tag_name_range(&tag_node, line_map);
            // Find the matching closing element via the parent JsxElement.
            let closing_range = tag_node.parent.as_ref().and_then(|parent| {
                if let NodeData::JsxElement(data) = &parent.data {
                    jsx_tag_name_range(&data.closing_element, line_map)
                } else {
                    None
                }
            });
            match (opening_range, closing_range) {
                (Some(o), Some(c)) => vec![o, c],
                (Some(o), None) => vec![o],
                _ => Vec::new(),
            }
        }
        SyntaxKind::JsxClosingElement => {
            // Get the tag name range.
            let closing_range = jsx_tag_name_range(&tag_node, line_map);
            // Find the matching opening element via the parent JsxElement.
            let opening_range = tag_node.parent.as_ref().and_then(|parent| {
                if let NodeData::JsxElement(data) = &parent.data {
                    jsx_tag_name_range(&data.opening_element, line_map)
                } else {
                    None
                }
            });
            match (closing_range, opening_range) {
                (Some(c), Some(o)) => vec![o, c],
                (Some(c), None) => vec![c],
                _ => Vec::new(),
            }
        }
        SyntaxKind::JsxSelfClosingElement => {
            // Self-closing: only one tag name.
            match jsx_tag_name_range(&tag_node, line_map) {
                Some(r) => vec![r],
                None => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

/// Walk up the parent chain to find a JSX tag node (opening, closing, or
/// self-closing element).
fn find_jsx_tag_ancestor(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        match n.kind {
            SyntaxKind::JsxOpeningElement
            | SyntaxKind::JsxClosingElement
            | SyntaxKind::JsxSelfClosingElement => {
                return Some(n);
            }
            _ => {}
        }
        current = n.parent.clone();
    }
    None
}

/// Get the range of a JSX tag's tag-name sub-node.
fn jsx_tag_name_range(tag_node: &Arc<Node>, line_map: &LineMap) -> Option<Range> {
    let tag_name = match &tag_node.data {
        NodeData::JsxOpeningElement(data) => &data.tag_name,
        NodeData::JsxClosingElement(data) => &data.tag_name,
        NodeData::JsxSelfClosingElement(data) => &data.tag_name,
        _ => return None,
    };
    Some(Range {
        start: offset_to_position(line_map, tag_name.pos()),
        end: offset_to_position(line_map, tag_name.end()),
    })
}

// ─── Helper functions ────────────────────────────────────────────────

/// Find the deepest AST node whose source range covers `offset`.
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

/// Convert an LSP `Position` to a byte offset within a line map.
fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}

/// Convert a byte offset to an LSP `Position`.
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
