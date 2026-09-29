use crate::scanner::CommentRange;
use crate::format::mig::m4t::get_default_indent_size;
use crate::format::mig::m4t_4::EmitContext;
use tsox_core::stringutil::{is_ascii_letter, is_digit, is_white_space_single_line};
use tsox_core::tspath::get_base_file_name;

use crate::scanner::is_jsx_line_break::CommentRangeKind;

fn decode_rune(text: &str, pos: usize) -> (char, usize) {
    match text[pos..].chars().next() {
        Some(ch) => (ch, ch.len_utf8()),
        None => ('\u{FFFD}', 0),
    }
}

pub fn has_leading_hash(text: &str) -> bool {
    !text.is_empty() && text.as_bytes()[0] == b'#'
}

pub fn remove_leading_hash(text: &str) -> &str {
    if has_leading_hash(text) {
        &text[1..]
    } else {
        text
    }
}

pub fn ensure_leading_hash(text: &str) -> String {
    if has_leading_hash(text) {
        text.to_string()
    } else {
        format!("#{}", text)
    }
}

pub fn format_generated_name(private_name: bool, prefix: &str, base: &str, suffix: &str) -> String {
    let name = format!(
        "{}{}{}",
        remove_leading_hash(prefix),
        remove_leading_hash(base),
        remove_leading_hash(suffix)
    );
    if private_name {
        return ensure_leading_hash(&name);
    }
    name
}

pub fn is_ascii_word_character(ch: char) -> bool {
    is_ascii_letter(ch) || is_digit(ch) || ch == '_'
}

pub fn make_identifier_from_module_name(module_name: &str) -> String {
    let module_name = get_base_file_name(module_name);
    let mut builder = String::new();
    let mut start = 0;
    let mut pos = 0;
    let bytes = module_name.as_bytes();
    while pos < bytes.len() {
        let ch = bytes[pos] as char;
        if pos == 0 && is_digit(ch) {
            builder.push('_');
        } else if !is_ascii_word_character(ch) {
            if start < pos {
                builder.push_str(&module_name[start..pos]);
            }
            builder.push('_');
            start = pos + 1;
        }
        pos += 1;
    }
    if start < pos {
        builder.push_str(&module_name[start..pos]);
    }
    builder
}

pub fn find_span_end_with_emit_context<T>(
    c: &EmitContext,
    array: &[T],
    test: impl Fn(&EmitContext, &T) -> bool,
    start: usize,
) -> usize {
    let mut i = start;
    while i < array.len() && test(c, &array[i]) {
        i += 1;
    }
    i
}

pub fn find_span_end<T>(array: &[T], test: impl Fn(&T) -> bool, start: usize) -> usize {
    let mut i = start;
    while i < array.len() && test(&array[i]) {
        i += 1;
    }
    i
}

pub fn skip_white_space_single_line(text: &str, pos: &mut usize) {
    while *pos < text.len() {
        let (ch, size) = decode_rune(text, *pos);
        if !is_white_space_single_line(ch) {
            break;
        }
        *pos += size;
    }
}

pub fn match_white_space_single_line(text: &str, pos: &mut usize) -> bool {
    let start_pos = *pos;
    skip_white_space_single_line(text, pos);
    *pos != start_pos
}

pub fn match_rune(text: &str, pos: &mut usize, expected: char) -> bool {
    let (ch, size) = decode_rune(text, *pos);
    if ch == expected {
        *pos += size;
        return true;
    }
    false
}

pub fn match_string(text: &str, pos: &mut usize, expected: &str) -> bool {
    let mut text_pos = *pos;
    let mut expected_pos = 0;
    let expected_bytes = expected.as_bytes();
    while expected_pos < expected_bytes.len() {
        if text_pos >= text.len() {
            return false;
        }

        let (expected_rune, expected_size) = decode_rune(expected, expected_pos);
        if !match_rune(text, &mut text_pos, expected_rune) {
            return false;
        }

        expected_pos += expected_size;
    }

    *pos = text_pos;
    true
}

pub fn match_quoted_string(text: &str, pos: &mut usize) -> bool {
    let mut text_pos = *pos;
    let quote_char;
    if match_rune(text, &mut text_pos, '\'') {
        quote_char = '\'';
    } else if match_rune(text, &mut text_pos, '"') {
        quote_char = '"';
    } else {
        return false;
    }
    while text_pos < text.len() {
        let (ch, size) = decode_rune(text, text_pos);
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
        let text = &text[comment_range.pos + 3..comment_range.end];
        let mut pos = 0;
        skip_white_space_single_line(text, &mut pos);
        if !match_rune(text, &mut pos, '<') {
            return false;
        }
        if match_string(text, &mut pos, "reference") {
            if !match_white_space_single_line(text, &mut pos) {
                return false;
            }
            if !match_string(text, &mut pos, "path")
                && !match_string(text, &mut pos, "types")
                && !match_string(text, &mut pos, "lib")
                && !match_string(text, &mut pos, "no-default-lib")
            {
                return false;
            }
            skip_white_space_single_line(text, &mut pos);
            if !match_rune(text, &mut pos, '=') {
                return false;
            }
            skip_white_space_single_line(text, &mut pos);
            if !match_quoted_string(text, &mut pos) {
                return false;
            }
        } else if match_string(text, &mut pos, "amd-dependency") {
            if !match_white_space_single_line(text, &mut pos) {
                return false;
            }
            if !match_string(text, &mut pos, "path") {
                return false;
            }
            skip_white_space_single_line(text, &mut pos);
            if !match_rune(text, &mut pos, '=') {
                return false;
            }
            skip_white_space_single_line(text, &mut pos);
            if !match_quoted_string(text, &mut pos) {
                return false;
            }
        } else if match_string(text, &mut pos, "amd-module") {
            skip_white_space_single_line(text, &mut pos);
        } else {
            return false;
        }
        return text[pos..].find("/>").is_some();
    }

    false
}

pub fn is_jsdoc_like_text(text: &str, comment: &CommentRange) -> bool {
    comment.kind == CommentRangeKind::MultiLine
        && comment.end - comment.pos >= 5
        && text.as_bytes()[comment.pos + 2] == b'*'
        && text.as_bytes()[comment.pos + 3] != b'/'
}

pub fn is_pinned_comment(text: &str, comment: &CommentRange) -> bool {
    comment.kind == CommentRangeKind::MultiLine
        && comment.end - comment.pos > 5
        && text.as_bytes()[comment.pos + 2] == b'!'
}

pub fn calculate_indent(text: &str, pos: usize, end: usize) -> i64 {
    let mut current_line_indent: i64 = 0;
    let indent_size = get_default_indent_size() as i64;
    let mut pos = pos;
    while pos < end {
        let (ch, size) = decode_rune(text, pos);
        if !is_white_space_single_line(ch) {
            break;
        }
        if ch == '\t' {
            current_line_indent += indent_size - (current_line_indent % indent_size);
        } else {
            current_line_indent += 1;
        }
        pos += size;
    }

    current_line_indent
}
