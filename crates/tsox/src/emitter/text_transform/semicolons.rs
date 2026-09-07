use super::super::UNMAPPED;
use super::fold_tracked::fold_expression_newlines_tracked;
use super::fold_untracked::fold_expression_newlines;
use super::reindent::{reindent_and_dedup, reindent_and_dedup_tracked};

pub(crate) fn add_implicit_semicolons(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }
        let last = trimmed.chars().last().unwrap_or(' ');

        let skip = matches!(
            last,
            '{' | '(' | '[' | ',' | ';' | ':' | '.' | '|' | '&' | '=' | '>' | '?'
        ) || trimmed.ends_with("=>")
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.ends_with("*/");
        if skip {
            result.push_str(trimmed);
        } else if last == '}' {
            result.push_str(trimmed);
        } else {
            result.push_str(trimmed);
            result.push(';');
        }
        result.push('\n');
    }
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

#[allow(dead_code)]
pub(crate) fn normalize_js_output(text: &str) -> String {
    let folded = fold_expression_newlines(text);
    let reindented = reindent_and_dedup(&folded);
    add_implicit_semicolons(&reindented)
}

pub(crate) fn add_implicit_semicolons_tracked(
    text: &str,
    src_offsets: &[u32],
) -> (String, Vec<u32>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(text.len());
    let mut out_offsets: Vec<u32> = Vec::new();
    let had_trailing_newline = n > 0 && chars[n - 1] == '\n';

    let mut i = 0;
    while i < n {
        let line_start = i;
        while i < n && chars[i] != '\n' {
            i += 1;
        }
        let line_end = i;
        let has_newline = i < n && chars[i] == '\n';
        if has_newline {
            i += 1;
        }

        let mut content_end = line_end;
        while content_end > line_start && chars[content_end - 1].is_whitespace() {
            content_end -= 1;
        }

        if content_end == line_start {
            out_text.push('\n');
            if has_newline {
                out_offsets.push(src_offsets[line_end]);
            } else {
                out_offsets.push(UNMAPPED);
            }
            continue;
        }

        let last = chars[content_end - 1];
        let trimmed_str: String = chars[line_start..content_end].iter().collect();
        let skip = matches!(
            last,
            '{' | '(' | '[' | ',' | ';' | ':' | '.' | '|' | '&' | '=' | '>' | '?'
        ) || trimmed_str.ends_with("=>")
            || trimmed_str.starts_with("//")
            || trimmed_str.starts_with("/*")
            || trimmed_str.ends_with("*/");

        for j in line_start..content_end {
            out_text.push(chars[j]);
            out_offsets.push(src_offsets[j]);
        }

        if !skip && last != '}' {
            out_text.push(';');
            out_offsets.push(UNMAPPED);
        }

        out_text.push('\n');
        if has_newline {
            out_offsets.push(src_offsets[line_end]);
        } else {
            out_offsets.push(UNMAPPED);
        }
    }

    if !had_trailing_newline && out_text.ends_with('\n') {
        out_text.pop();
        out_offsets.pop();
    }
    (out_text, out_offsets)
}

pub(crate) fn normalize_js_output_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let (text, offsets) = fold_expression_newlines_tracked(text, src_offsets);
    let (text, offsets) = reindent_and_dedup_tracked(&text, &offsets);
    add_implicit_semicolons_tracked(&text, &offsets)
}
