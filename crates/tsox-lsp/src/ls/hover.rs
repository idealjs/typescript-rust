#![allow(dead_code)]

use std::sync::Arc;

use crate::ls::display_parts_writer::DisplayPartsWriter;
use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp::Position;
use crate::lsp::lsproto_lsp::Range;
use tsox_checker::checker::nodebuilder::SymbolDisplayPart;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::node::LineMap;

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
        // LSP character 是 UTF-16 列：行内按码元推进，不得与字节行首直接相加
        let mut offset = line_start;
        let mut units = 0usize;
        for c in source_file.text[line_start..].chars() {
            if units >= character {
                break;
            }
            units += c.len_utf16();
            offset += c.len_utf8();
        }

        // tsc quickinfo：GetTouchingPropertyName（谓词含关键字/私有名，带前 token 回退）；
        // JSDoc 内位置需附带 JSDoc 子树（Go VisitEachChildAndJSDoc）
        let node =
            tsox_frontend::astnav::get_touching_property_name_with_jsdoc(&source_file, offset)?;
        if node.kind == tsox_frontend::ast::SyntaxKind::SourceFile {
            return None;
        }
        let node = get_node_for_quick_info(&node, offset);

        let mut checker = program.build_checker();
        // Go getQuickInfoAndDeclarationAtLocation 结构化移植优先（骨架转正中）；
        // 迁移期回退：旧补丁式路径
        let mut parts = checker.quick_info_display_for_node(&node);
        if parts.is_empty() {
            parts = checker.quick_info_parts(&node);
        }
        if parts.is_empty() {
            parts = checker.get_quick_info_display_parts(&node);
        }
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

fn get_node_for_quick_info(node: &Arc<Node>, offset: usize) -> Arc<Node> {
    use tsox_frontend::ast::SyntaxKind;
    let Some(parent) = node.parent() else {
        return Arc::clone(node);
    };
    if parent.kind == SyntaxKind::NewExpression
        && node.pos() == parent.pos()
        && let Some(expr) = parent.expression()
    {
        return Arc::clone(expr);
    }
    if node.kind == SyntaxKind::NewExpression
        && let Some(expr) = node.expression()
        && offset < expr.pos()
    {
        return Arc::clone(expr);
    }
    if parent.kind == SyntaxKind::NamedTupleMember && node.pos() == parent.pos() {
        return Arc::clone(&parent);
    }
    if parent.kind == SyntaxKind::MetaProperty {
        if let tsox_frontend::ast::NodeData::MetaProperty(mp) = &parent.data {
            if mp.keyword_token == SyntaxKind::ImportKeyword && Arc::ptr_eq(&mp.name, node) {
                return Arc::clone(&parent);
            }
        }
    }
    if parent.kind == SyntaxKind::JsxNamespacedName {
        return Arc::clone(&parent);
    }
    Arc::clone(node)
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
