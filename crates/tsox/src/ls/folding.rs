#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::{Node, NodeData, SourceFile, SyntaxKind, node_data_generated::for_each_child};
use crate::lsp::lsproto::lsp::DocumentUri;

use super::language_service::LanguageService;
use super::types::FoldingRange;

pub struct RegionDelimiterResult {
    pub is_start: bool,
    pub name: String,
}

impl LanguageService {
    pub fn provide_folding_range(&self, document_uri: &DocumentUri) -> Vec<FoldingRange> {
        let (_program, source_file) = self.get_program_and_file(document_uri);
        let mut res = add_node_outlining_spans(&source_file);
        res.extend(add_region_outlining_spans(&source_file));
        res.sort_by(|a, b| match a.start_line.cmp(&b.start_line) {
            std::cmp::Ordering::Equal => a
                .start_character
                .unwrap_or(0)
                .cmp(&b.start_character.unwrap_or(0)),
            ord => ord,
        });
        res
    }
}

fn add_node_outlining_spans(source_file: &Arc<SourceFile>) -> Vec<FoldingRange> {
    let line_map = &source_file.line_map;
    let mut ranges = Vec::new();

    if let NodeData::SourceFile(d) = &source_file.node.data {
        for stmt in d.statements.iter() {
            visit_node_for_folding(stmt, line_map, &mut ranges, 40);
        }
    }
    ranges
}

fn add_region_outlining_spans(source_file: &Arc<SourceFile>) -> Vec<FoldingRange> {
    let text = &source_file.text;
    let line_map = &source_file.line_map;
    let mut regions: Vec<FoldingRange> = Vec::new();
    let mut out = Vec::new();

    for &line_start in &line_map.line_starts {
        let ls = line_start as usize;
        let line_end = get_line_end(text, ls);
        let line_text = &text[ls..line_end.min(text.len())];
        if let Some(result) = parse_region_delimiter(line_text) {
            if result.is_start {
                let comment_offset = line_text.find("//").unwrap_or(0);
                let abs_offset = ls + comment_offset;
                let (start_line, start_char) = offset_to_line_col(line_map, abs_offset);
                regions.push(FoldingRange {
                    start_line,
                    start_character: Some(start_char),
                    end_line: 0,
                    end_character: None,
                    kind: Some("region".to_string()),
                    collapsed_text: if result.name.is_empty() {
                        None
                    } else {
                        Some(result.name.clone())
                    },
                });
            } else if !regions.is_empty() {
                let mut region = regions.pop().unwrap();
                let (end_line, end_char) = offset_to_line_col(line_map, line_end);
                region.end_line = end_line;
                region.end_character = Some(end_char);
                out.push(region);
            }
        }
    }
    out
}

fn visit_node_for_folding(
    node: &Arc<Node>,
    line_map: &LineMap,
    ranges: &mut Vec<FoldingRange>,
    depth_remaining: usize,
) {
    if depth_remaining == 0 {
        return;
    }
    if let Some(span) = get_outlining_span_for_node(node, line_map) {
        ranges.push(span);
    }
    for_each_child(node, |child| {
        visit_node_for_folding(child, line_map, ranges, depth_remaining - 1);
        false
    });
}

fn get_outlining_span_for_node(node: &Arc<Node>, line_map: &LineMap) -> Option<FoldingRange> {
    let pos = node.pos();
    let end = node.end();
    if positions_are_on_same_line(pos, end, line_map) {
        return None;
    }
    let kind_str = match node.kind {
        SyntaxKind::Block
        | SyntaxKind::ModuleBlock
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::ClassExpression
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::CaseBlock
        | SyntaxKind::TypeLiteral
        | SyntaxKind::ObjectBindingPattern
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::TupleType
        | SyntaxKind::ArrayBindingPattern
        | SyntaxKind::JsxElement
        | SyntaxKind::JsxFragment
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxOpeningElement
        | SyntaxKind::TemplateExpression
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::ArrowFunction
        | SyntaxKind::CallExpression
        | SyntaxKind::NewExpression
        | SyntaxKind::ParenthesizedExpression
        | SyntaxKind::CaseClause
        | SyntaxKind::DefaultClause => "",
        SyntaxKind::NamedImports | SyntaxKind::NamedExports => "imports",
        _ => return None,
    };
    create_folding_range_from_bounds(pos, end, kind_str, line_map)
}

fn create_folding_range_from_bounds(
    start: usize,
    end: usize,
    kind: &str,
    line_map: &LineMap,
) -> Option<FoldingRange> {
    let (start_line, start_char) = offset_to_line_col(line_map, start);
    let (end_line, end_char) = offset_to_line_col(line_map, end);
    if start_line == end_line {
        return None;
    }
    Some(FoldingRange {
        start_line,
        start_character: Some(start_char),
        end_line,
        end_character: Some(end_char),
        kind: if kind.is_empty() {
            None
        } else {
            Some(kind.to_string())
        },
        collapsed_text: None,
    })
}

fn positions_are_on_same_line(pos1: usize, pos2: usize, line_map: &LineMap) -> bool {
    line_of_offset(line_map, pos1) == line_of_offset(line_map, pos2)
}

fn line_of_offset(line_map: &LineMap, offset: usize) -> usize {
    match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    }
}

fn offset_to_line_col(line_map: &LineMap, offset: usize) -> (u32, u32) {
    let line = line_of_offset(line_map, offset);
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    (line as u32, offset.saturating_sub(line_start) as u32)
}

fn get_line_end(text: &str, line_start: usize) -> usize {
    match text[line_start..].find('\n') {
        Some(idx) => line_start + idx,
        None => text.len(),
    }
}

pub fn parse_region_delimiter(line_text: &str) -> Option<RegionDelimiterResult> {
    let line_text = line_text.trim_start();
    let line_text = line_text.strip_prefix("//")?;
    let line_text = line_text.trim();
    let line_text = line_text.strip_suffix('\r').unwrap_or(line_text);
    let line_text = line_text.strip_prefix('#')?;
    let is_start = if let Some(rest) = line_text.strip_prefix("end") {
        let _ = rest;
        false
    } else {
        true
    };
    let rest = if line_text.starts_with("end") {
        &line_text[3..]
    } else {
        line_text
    };
    let rest = rest.strip_prefix("region")?;
    Some(RegionDelimiterResult {
        is_start,
        name: rest.trim().to_string(),
    })
}
