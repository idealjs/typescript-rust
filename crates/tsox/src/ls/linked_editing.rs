#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::{NodeData, for_each_child};
use crate::ast::{Node, SyntaxKind};
use crate::lsp::lsproto::lsp::{Position, Range};

use super::language_service::LanguageService;
use super::types::{LinkedEditingRangeParams, LinkedEditingRanges};

pub const JSX_TAG_WORD_PATTERN: &str = "[a-zA-Z0-9:\\-\\._$]*";

impl LanguageService {
    pub fn provide_linked_editing_range(
        &self,
        params: &LinkedEditingRangeParams,
    ) -> Option<LinkedEditingRanges> {
        let (program, source_file) = self.get_program_and_file(&params.text_document.uri);
        let line_map = &source_file.line_map;

        let offset = lsp_position_to_offset(line_map, &params.position);

        let node = find_deepest_node(&source_file.node, offset);
        let _ = &program;

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

fn find_jsx_linked_ranges(node: &Arc<Node>, line_map: &LineMap) -> Vec<Range> {
    let tag_node = find_jsx_tag_ancestor(node);

    let tag_node = match tag_node {
        Some(n) => n,
        None => return Vec::new(),
    };

    match tag_node.kind {
        SyntaxKind::JsxOpeningElement => {
            let opening_range = jsx_tag_name_range(&tag_node, line_map);

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
            let closing_range = jsx_tag_name_range(&tag_node, line_map);

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
        SyntaxKind::JsxSelfClosingElement => match jsx_tag_name_range(&tag_node, line_map) {
            Some(r) => vec![r],
            None => Vec::new(),
        },
        _ => Vec::new(),
    }
}

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

fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
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
