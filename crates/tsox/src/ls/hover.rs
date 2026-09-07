#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::{Node, node_data_generated::for_each_child};
use crate::checker::nodebuilder::SymbolDisplayPart;
use crate::ls::display_parts_writer::DisplayPartsWriter;
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range};

use super::language_service::LanguageService;
use super::types::{Hover, HoverContent};

pub const SYMBOL_FORMAT_FLAGS: u32 = 0;

pub const TYPE_FORMAT_FLAGS: u32 = 0;

pub struct SymbolDisplayInfo {
    pub display_parts: DisplayPartsWriter,
    pub declaration: Option<Arc<Node>>,
}

impl LanguageService {
    pub fn provide_hover(&self, document_uri: &DocumentUri, position: Position) -> Option<Hover> {
        let (program, source_file) = self.get_program_and_file(document_uri);

        let line_map = &source_file.line_map;
        let line = position.line as usize;
        let character = position.character as usize;
        let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
        let offset = line_start + character;

        let node = find_deepest_node(&source_file.node, offset);

        let mut checker = program.build_checker();
        let parts = checker.get_quick_info_display_parts(&node);
        let type_str = if parts.is_empty() {
            checker.get_quick_info_text(&node)
        } else {
            display_parts_to_string(&parts)
        };

        if type_str.is_empty() {
            return None;
        }

        let hover_range = node_range_to_lsp_range(line_map, &node);

        Some(Hover {
            contents: HoverContent {
                markup_content: Some(crate::lsp::lsproto::lsp::MarkupContent {
                    kind: crate::lsp::lsproto::lsp::MarkupKind::Markdown,
                    value: format_code_block("typescript", &type_str),
                }),
                string: None,
            },
            range: Some(hover_range),
            can_increase_verbosity: None,
        })
    }
}

pub fn format_quick_info(quick_info: &str) -> String {
    if quick_info.is_empty() {
        return String::new();
    }
    format_code_block("typescript", quick_info)
}

pub fn format_code_block(lang: &str, code: &str) -> String {
    if code.is_empty() {
        return String::new();
    }
    let mut ticks = 3;
    let tick_str = |n: usize| "`".repeat(n);
    while code.contains(&tick_str(ticks)) {
        ticks += 1;
    }
    let mut result = tick_str(ticks);
    result.push_str(lang);
    result.push('\n');
    result.push_str(code);
    result.push('\n');
    result.push_str(&tick_str(ticks));
    result.push('\n');
    result
}

fn display_parts_to_string(parts: &[SymbolDisplayPart]) -> String {
    parts.iter().map(|p| p.text.as_str()).collect()
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

fn node_range_to_lsp_range(line_map: &LineMap, node: &Arc<Node>) -> Range {
    let start = offset_to_position(line_map, node.pos());
    let end = offset_to_position(line_map, node.end());
    Range { start, end }
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
