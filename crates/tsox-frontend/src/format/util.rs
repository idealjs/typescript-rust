//! Go format/util.go 与 scanner 行工具的移植：行/列换算、缩进字符串、
//! token 起点（skip trivia）、kind 判定。

use std::sync::Arc;

use tsox_core::core::text::TextRange;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use crate::ast::SourceFile;

pub(crate) fn line_of_position(file: &SourceFile, pos: usize) -> usize {
    file.line_map.line_at(pos)
}

pub(crate) fn line_start_position_for_position(file: &SourceFile, position: usize) -> usize {
    let line = line_of_position(file, position);
    line_start_of_line(file, line)
}

pub(crate) fn line_and_byte_offset_of_position(file: &SourceFile, pos: usize) -> (usize, usize) {
    let line = line_of_position(file, pos);
    let start = line_start_of_line(file, line);
    (line, pos - start)
}

pub(crate) fn line_start_of_line(file: &SourceFile, line: usize) -> usize {
    let starts = &file.line_map.line_starts;
    if line >= starts.len() {
        return starts.last().map(|s| *s as usize).unwrap_or(0);
    }
    starts[line] as usize
}

pub(crate) fn end_line_position(file: &SourceFile, line: usize) -> usize {
    let text = &file.text;
    let mut pos = line_start_of_line(file, line);
    while pos < text.len() {
        let ch = text[pos..].chars().next().unwrap();
        if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
            // Go 可返回 -1；这里收敛到 0 防御下溢
            return pos.saturating_sub(1);
        }
        pos += ch.len_utf8();
    }
    text.len().saturating_sub(1)
}

pub(crate) fn position_of_line_and_byte_offset(file: &SourceFile, line: usize, byte_offset: usize) -> usize {
    line_start_of_line(file, line) + byte_offset
}

/// Go GetTokenPosOfNode：missing 节点不跳 trivia，其余从节点起点跳过
/// 前缀 trivia 得到首 token 位置。
pub(crate) fn token_pos_of_node(file: &SourceFile, node: &Node) -> usize {
    if crate::astnav::is_missing_node(node) {
        return node.pos();
    }
    if node.kind == SyntaxKind::JsxText {
        return crate::scanner::skip_trivia_ex(
            &file.text,
            node.pos(),
            &crate::scanner::SkipTriviaOptions { stop_at_comments: true, ..Default::default() },
            None,
        );
    }
    crate::scanner::skip_trivia(&file.text, node.pos())
}

pub(crate) fn with_token_start(file: &SourceFile, node: &Node) -> TextRange {
    TextRange::new(token_pos_of_node(file, node), node.end())
}

/// "0\t2$"：character 是行内实际字节下标，column 是 tab 展开后的列。
pub(crate) fn find_first_non_whitespace_character_and_column(
    file: &SourceFile,
    start_pos: usize,
    end_pos: usize,
    tab_size: u32,
) -> (usize, u32) {
    let text = &file.text;
    let mut column: u32 = 0;
    let mut pos = start_pos;
    while pos < end_pos {
        let Some(ch) = text[pos..].chars().next() else { break };
        if !is_whitespace_single_line(ch) {
            break;
        }
        if ch == '\t' {
            if tab_size > 0 {
                column += tab_size + (column % tab_size);
            }
        } else {
            column += 1;
        }
        pos += ch.len_utf8();
    }
    (pos - start_pos, column)
}

pub(crate) fn find_first_non_whitespace_column(
    file: &SourceFile,
    start_pos: usize,
    end_pos: usize,
    tab_size: u32,
) -> u32 {
    find_first_non_whitespace_character_and_column(file, start_pos, end_pos, tab_size).1
}

pub(crate) fn get_indentation_string(indentation: usize, convert_tabs_to_spaces: bool, tab_size: u32) -> String {
    if !convert_tabs_to_spaces {
        if tab_size == 0 {
            return String::new();
        }
        let tabs = indentation / tab_size as usize;
        let spaces = indentation - tabs * tab_size as usize;
        let mut res = "\t".repeat(tabs);
        if spaces > 0 {
            res.push_str(&" ".repeat(spaces));
        }
        res
    } else {
        " ".repeat(indentation)
    }
}

pub(crate) fn is_comment(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::SingleLineCommentTrivia || kind == SyntaxKind::MultiLineCommentTrivia
}

pub(crate) fn is_string_or_regular_expression_or_template_literal(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::StringLiteral
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead
            | SyntaxKind::TemplateMiddle
            | SyntaxKind::TemplateTail
    )
}

pub(crate) fn is_line_break(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

pub(crate) fn is_whitespace_single_line(ch: char) -> bool {
    matches!(ch, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}')
}

pub(crate) fn node_is_missing(node: &Arc<Node>) -> bool {
    crate::astnav::is_missing_node(node)
}
