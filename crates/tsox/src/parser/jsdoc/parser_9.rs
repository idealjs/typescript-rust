#![allow(unused_imports)]

use super::*;

impl crate::parser::Parser {
    pub(crate) fn skip_whitespace(&mut self) {
        if self.token == SyntaxKind::WhitespaceTrivia || self.token == SyntaxKind::NewLineTrivia {
            if self.is_next_nonwhitespace_token_eof() {
                return;
            }
        }
        while self.token == SyntaxKind::WhitespaceTrivia || self.token == SyntaxKind::NewLineTrivia
        {
            self.next_token_jsdoc();
        }
    }

    pub(crate) fn is_next_nonwhitespace_token_eof(&mut self) -> bool {
        loop {
            self.next_token_jsdoc();
            if self.token == SyntaxKind::EndOfFile {
                return true;
            }
            if self.token != SyntaxKind::WhitespaceTrivia && self.token != SyntaxKind::NewLineTrivia
            {
                return false;
            }
        }
    }

    pub(crate) fn skip_whitespace_or_asterisk(&mut self) -> String {
        let mut indent_text = String::new();
        let mut preceding_line_break = false;
        let mut seen_line_break = false;

        loop {
            match self.token {
                SyntaxKind::WhitespaceTrivia => {
                    if preceding_line_break {
                        indent_text = String::new();
                        seen_line_break = true;
                    }
                    indent_text.push_str(self.scanner.token_text());
                    preceding_line_break = false;
                }
                SyntaxKind::NewLineTrivia => {
                    preceding_line_break = true;
                }
                SyntaxKind::AsteriskToken => {
                    preceding_line_break = false;
                }
                _ => break,
            }
            self.next_token_jsdoc();
        }

        if seen_line_break {
            indent_text
        } else {
            String::new()
        }
    }
}

pub(crate) fn is_jsdoc_like_text(text: &str) -> bool {
    text.starts_with("/**") && !text.starts_with("/**/")
}

pub(crate) fn is_jsdoc_link_tag(kind: &str) -> bool {
    matches!(kind, "link" | "linkcode" | "linkplain")
}

pub(crate) fn is_identifier_or_keyword_token(token: SyntaxKind) -> bool {
    token == SyntaxKind::Identifier || crate::ast::is_keyword_kind(token)
}

pub(crate) fn push_comment(
    comments: &mut Vec<String>,
    indent: &mut usize,
    margin: &mut i32,
    text: &str,
) {
    if *margin == -1 {}
    comments.push(text.to_string());
    *indent += text.len();
}

pub(crate) fn remove_leading_newlines(mut comments: Vec<String>) -> Vec<String> {
    let mut i = 0;
    while i < comments.len()
        && comments[i]
            .trim_matches(|c| c == '\r' || c == '\n')
            .is_empty()
    {
        i += 1;
    }
    comments.drain(..i);
    comments
}

pub(crate) fn trim_end(s: &str) -> String {
    s.trim_end_matches(|c: char| c.is_whitespace() || c == '\u{2028}' || c == '\u{2029}')
        .to_string()
}

pub(crate) fn remove_trailing_whitespace(mut comments: Vec<String>) -> Vec<String> {
    let mut end = comments.len();
    for i in (0..comments.len()).rev() {
        let trimmed = trim_end(&comments[i]);
        if trimmed.is_empty() {
            end = i;
        } else {
            comments[i] = trimmed;
            break;
        }
    }
    comments.truncate(end);
    comments
}

pub fn get_jsdoc_comment_ranges(text: &str, node: &Node) -> Vec<crate::scanner::CommentRange> {
    use crate::ast::SyntaxKind as SK;
    use crate::scanner::{get_leading_comment_ranges, get_trailing_comment_ranges};

    let token_pos = node.pos();

    let full_start = find_full_start(text, token_pos);

    let mut ranges = match node.kind {
        SK::Parameter
        | SK::TypeParameter
        | SK::FunctionExpression
        | SK::ArrowFunction
        | SK::ParenthesizedExpression
        | SK::VariableDeclaration
        | SK::ExportSpecifier => {
            let mut r = get_trailing_comment_ranges(text, token_pos);
            r.extend(get_leading_comment_ranges(text, full_start));
            r
        }
        _ => get_leading_comment_ranges(text, full_start),
    };

    let node_end = node.end();
    ranges.retain(|c| {
        let comment_start = c.pos;
        let comment_len = c.end.saturating_sub(comment_start);
        c.end <= node_end
            && comment_len >= 4
            && text.as_bytes().get(comment_start + 1) == Some(&b'*')
            && text.as_bytes().get(comment_start + 2) == Some(&b'*')
            && text.as_bytes().get(comment_start + 3) != Some(&b'/')
    });
    ranges
}

pub(crate) fn find_full_start(text: &str, token_pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = token_pos;

    while i > 0 {
        while i > 0
            && (bytes[i - 1] == b' '
                || bytes[i - 1] == b'\t'
                || bytes[i - 1] == b'\n'
                || bytes[i - 1] == b'\r')
        {
            i -= 1;
        }

        if i >= 2 && bytes[i - 2] == b'*' && bytes[i - 1] == b'/' {
            let mut j = i - 2;
            while j >= 2 {
                if bytes[j - 2] == b'/' && bytes[j - 1] == b'*' {
                    i = j - 2;
                    break;
                }
                j -= 1;
            }
            if j < 2 {
                break;
            }
        } else if i >= 2 && bytes[i - 2] == b'/' && bytes[i - 1] == b'/' {
            while i > 0 && bytes[i - 1] != b'\n' {
                i -= 1;
            }
        } else {
            break;
        }
    }
    i
}

thread_local! {
    static JSDOC_PARSER: std::cell::RefCell<Option<((u64, u64, u64), crate::parser::Parser)>> =
        const { std::cell::RefCell::new(None) };
}

pub(crate) fn fnv1a(bytes: &[u8], seed: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325u64 ^ seed;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn jsdoc_parser_key(file_name: &str, text: &str) -> (u64, u64, u64) {
    let bytes = text.as_bytes();
    let head = fnv1a(&bytes[..bytes.len().min(64)], 0x9e3779b9);
    let tail = fnv1a(&bytes[bytes.len().saturating_sub(64)..], 0x85ebca6b);
    (
        fnv1a(file_name.as_bytes(), 0),
        (text.len() as u64) ^ head,
        tail,
    )
}

pub fn parse_jsdoc_for_node(source_file: &crate::ast::SourceFile, node: &Node) -> Vec<Arc<Node>> {
    let text = &source_file.text;
    let ranges = get_jsdoc_comment_ranges(text, node);
    if ranges.is_empty() {
        return Vec::new();
    }

    let key = jsdoc_parser_key(&source_file.file_name, text);
    let mut jsdocs: Vec<Arc<Node>> = Vec::with_capacity(ranges.len());
    let mut pos = node.pos();
    JSDOC_PARSER.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.as_ref().map(|(k, _)| *k) != Some(key) {
            *slot = Some((key, crate::parser::Parser::new(text.clone())));
        }
        let parser = &mut slot.as_mut().unwrap().1;
        for comment in &ranges {
            if let Some(parsed) = parser.parse_jsdoc_comment(comment.pos, comment.end, pos) {
                pos = parsed.end();
                jsdocs.push(parsed);
            }
        }
    });
    jsdocs
}
