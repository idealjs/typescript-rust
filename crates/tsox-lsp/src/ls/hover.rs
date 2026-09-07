#![allow(dead_code)]

use std::sync::Arc;

use crate::ls::display_parts_writer::DisplayPartsWriter;
use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp::Position;
use crate::lsp::lsproto_lsp::Range;
use tsox_checker::checker::nodebuilder::SymbolDisplayPart;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::node::LineMap;
use tsox_frontend::ast::node_data_generated::for_each_child;

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

        let mut node = find_deepest_node(&source_file.node, offset);
        // tsc quickinfo：节点不宜悬停（标点/非标识符）时回退 findPrecedingToken（边界取前 token）
        if !is_hoverable_node(&node) {
            if let Some(prev) = find_deepest_token_ending_at(&source_file.node, offset) {
                node = prev;
            }
        }

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
                markup_content: Some(crate::lsp::lsproto_lsp::MarkupContent {
                    kind: crate::lsp::lsproto_lsp::MarkupKind::Markdown,
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

fn is_leaf_token(node: &Arc<Node>) -> bool {
    let mut has_child = false;
    for_each_child(node, |_| {
        has_child = true;
        false
    });
    !has_child
}

fn is_hoverable_node(node: &Arc<Node>) -> bool {
    use tsox_frontend::ast::SyntaxKind;
    matches!(
        node.kind,
        SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
    ) || is_declaration_kind(node.kind)
}

fn is_declaration_kind(kind: tsox_frontend::ast::SyntaxKind) -> bool {
    use tsox_frontend::ast::SyntaxKind;
    matches!(
        kind,
        SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::PropertySignature
    )
}

/// 边界回退（tsc findPrecedingToken 语义）：包含下钻到最深，再向子级/祖先兄弟找 end==offset 的叶子
fn find_deepest_token_ending_at(root: &Arc<Node>, offset: usize) -> Option<Arc<Node>> {
    let start = find_deepest_node(root, offset);
    let mut cur: Option<Arc<Node>> = Some(Arc::clone(&start));
    while let Some(n) = cur {
        let mut hit: Option<Arc<Node>> = None;
        for_each_child(&n, |ch| {
            if ch.end() == offset {
                hit = Some(Arc::clone(ch));
            }
            false
        });
        if let Some(h) = hit {
            let mut d = h;
            loop {
                let mut nx: Option<Arc<Node>> = None;
                for_each_child(&d, |ch| {
                    if ch.end() == offset {
                        nx = Some(Arc::clone(ch));
                    }
                    false
                });
                match nx {
                    Some(child) => d = child,
                    None => break,
                }
            }
            return Some(d);
        }
        cur = n.parent.clone();
    }
    None
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
