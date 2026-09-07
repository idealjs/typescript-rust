#![allow(unused_imports)]

use super::*;

pub(crate) fn skip_white_space_single_line(text: &str, pos: &mut usize) {
    while *pos < text.len() {
        let (ch, size) = decode_char_at(text, *pos);
        if !stringutil::is_white_space_single_line(ch) {
            break;
        }
        *pos += size;
    }
}

pub(crate) fn match_white_space_single_line(text: &str, pos: &mut usize) -> bool {
    let start = *pos;
    skip_white_space_single_line(text, pos);
    *pos != start
}

pub(crate) fn match_rune(text: &str, pos: &mut usize, expected: char) -> bool {
    if *pos < text.len() {
        let (ch, size) = decode_char_at(text, *pos);
        if ch == expected {
            *pos += size;
            return true;
        }
    }
    false
}

pub(crate) fn match_string(text: &str, pos: &mut usize, expected: &str) -> bool {
    let mut text_pos = *pos;
    for expected_ch in expected.chars() {
        if !match_rune(text, &mut text_pos, expected_ch) {
            return false;
        }
    }
    *pos = text_pos;
    true
}

pub(crate) fn match_quoted_string(text: &str, pos: &mut usize) -> bool {
    let mut text_pos = *pos;
    let quote_char = if match_rune(text, &mut text_pos, '\'') {
        '\''
    } else if match_rune(text, &mut text_pos, '"') {
        '"'
    } else {
        return false;
    };
    while text_pos < text.len() {
        let (ch, size) = decode_char_at(text, text_pos);
        text_pos += size;
        if ch == quote_char {
            *pos = text_pos;
            return true;
        }
    }
    false
}

pub fn is_recognized_triple_slash_comment(text: &str, comment_range: &CommentRange) -> bool {
    if comment_range.kind == CommentRangeKind::SingleLine
        && comment_range.end - comment_range.pos > 2
        && text.as_bytes()[comment_range.pos + 1] == b'/'
        && text.as_bytes()[comment_range.pos + 2] == b'/'
    {
        let start = comment_range.pos + 3;
        let inner = &text[start..comment_range.end];
        let mut pos = 0;
        skip_white_space_single_line(inner, &mut pos);
        if !match_rune(inner, &mut pos, '<') {
            return false;
        }
        if match_string(inner, &mut pos, "reference") {
            if !match_white_space_single_line(inner, &mut pos) {
                return false;
            }
            if !match_string(inner, &mut pos, "path")
                && !match_string(inner, &mut pos, "types")
                && !match_string(inner, &mut pos, "lib")
                && !match_string(inner, &mut pos, "no-default-lib")
            {
                return false;
            }
            skip_white_space_single_line(inner, &mut pos);
            if !match_rune(inner, &mut pos, '=') {
                return false;
            }
            skip_white_space_single_line(inner, &mut pos);
            if !match_quoted_string(inner, &mut pos) {
                return false;
            }
        } else if match_string(inner, &mut pos, "amd-dependency") {
            if !match_white_space_single_line(inner, &mut pos) {
                return false;
            }
            if !match_string(inner, &mut pos, "path") {
                return false;
            }
            skip_white_space_single_line(inner, &mut pos);
            if !match_rune(inner, &mut pos, '=') {
                return false;
            }
            skip_white_space_single_line(inner, &mut pos);
            if !match_quoted_string(inner, &mut pos) {
                return false;
            }
        } else if match_string(inner, &mut pos, "amd-module") {
            skip_white_space_single_line(inner, &mut pos);
        } else {
            return false;
        }
        return inner[pos..].contains("/>");
    }
    false
}

#[allow(dead_code)]
pub(crate) fn get_module_block_statements(node: &Arc<Node>) -> Option<&[Arc<Node>]> {
    match &node.data {
        crate::ast::node_data_generated::NodeData::ModuleBlock(d) => Some(&d.statements.nodes),
        _ => None,
    }
}
