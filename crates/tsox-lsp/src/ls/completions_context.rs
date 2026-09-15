use std::sync::Arc;

use tsox_frontend::ast::{Node, SourceFile, SyntaxKind, is_function_like_kind, is_keyword_kind};
use tsox_frontend::scanner::{CommentRange, CommentRangeKind, Scanner, get_leading_comment_ranges};

#[derive(Debug, Clone, Copy)]
pub(super) struct ScanToken {
    pub kind: SyntaxKind,
    pub pos: usize,
    pub end: usize,
}

fn new_scanner(text: &str, jsx: bool) -> Scanner {
    let mut scanner = Scanner::new(text.to_string());
    if jsx {
        scanner.set_language_variant(tsox_frontend::ast::LanguageVariant::Jsx);
    }
    scanner
}

/// 扫描从 from 起、token 起点 < to 的 token。
/// 裸扫描无 parser 驱动，模板续段须自管：TemplateHead（`${` 收尾）入栈，
/// 栈顶表达式括号深度归零时遇 `}` 重扫为 TemplateMiddle/Tail
/// （Go 走语法树 FindPrecedingToken 无此问题）
pub(super) fn scan_tokens(text: &str, jsx: bool, from: usize, to: usize) -> Vec<ScanToken> {
    let mut scanner = new_scanner(text, jsx);
    let limit = to.min(text.len());
    scanner.set_range(from.min(text.len()), text.len());
    let mut out: Vec<ScanToken> = Vec::new();
    let mut guard = 0usize;
    let mut templates: Vec<[i32; 3]> = Vec::new();
    loop {
        let mut kind = scanner.scan();
        if kind == SyntaxKind::EndOfFile {
            break;
        }
        match kind {
            SyntaxKind::TemplateHead | SyntaxKind::TemplateMiddle => {
                templates.push([0, 0, 0]);
            }
            SyntaxKind::OpenBraceToken => {
                if let Some(top) = templates.last_mut() {
                    top[0] += 1;
                }
            }
            SyntaxKind::OpenBracketToken => {
                if let Some(top) = templates.last_mut() {
                    top[1] += 1;
                }
            }
            SyntaxKind::OpenParenToken => {
                if let Some(top) = templates.last_mut() {
                    top[2] += 1;
                }
            }
            SyntaxKind::CloseBracketToken => {
                if let Some(top) = templates.last_mut() {
                    top[1] -= 1;
                }
            }
            SyntaxKind::CloseParenToken => {
                if let Some(top) = templates.last_mut() {
                    top[2] -= 1;
                }
            }
            SyntaxKind::CloseBraceToken => {
                let at_zero = templates
                    .last()
                    .is_some_and(|top| top.iter().all(|d| *d == 0));
                if at_zero {
                    kind = scanner.re_scan_template_token();
                    if kind == SyntaxKind::TemplateTail {
                        templates.pop();
                    }
                } else if let Some(top) = templates.last_mut() {
                    top[0] -= 1;
                }
            }
            _ => {}
        }
        let pos = scanner.token_pos();
        let end = scanner.token_end();
        if pos >= limit {
            break;
        }
        out.push(ScanToken { kind, pos, end });
        guard += 1;
        if guard > text.len() + 16 {
            break;
        }
    }
    out
}

/// Go getRelevantTokens：previous 为光标前一个 token；若它是成员名或关键字，
/// context 退到再前一个
pub(super) fn relevant_tokens(
    text: &str,
    jsx: bool,
    from: usize,
    position: usize,
) -> (Option<ScanToken>, Option<ScanToken>) {
    let tokens = scan_tokens(text, jsx, from, position);
    let previous = tokens.last().copied();
    if let Some(prev) = &previous
        && position <= prev.end
        && (prev.kind == SyntaxKind::Identifier
            || prev.kind == SyntaxKind::PrivateIdentifier
            || is_keyword_kind(prev.kind))
    {
        let context = tokens.iter().rev().nth(1).copied();
        return (context, previous);
    }
    (previous, previous)
}

pub(super) fn token_containing(text: &str, jsx: bool, position: usize) -> Option<ScanToken> {
    scan_tokens(text, jsx, 0, position + 1)
        .into_iter()
        .find(|t| t.pos <= position && position < t.end)
}

pub(super) fn comment_ranges(text: &str, jsx: bool) -> Vec<CommentRange> {
    let mut scanner = new_scanner(text, jsx);
    let mut ranges: Vec<CommentRange> = Vec::new();
    let mut guard = 0usize;
    loop {
        let kind = scanner.scan();
        let full_start = scanner.full_start_pos();
        let token_pos = scanner.token_pos();
        if token_pos > full_start {
            for r in get_leading_comment_ranges(text, full_start) {
                if r.pos >= token_pos {
                    break;
                }
                ranges.push(r);
            }
        }
        if kind == SyntaxKind::EndOfFile || guard > text.len() + 16 {
            break;
        }
        guard += 1;
    }
    ranges
}

pub(super) fn is_jsx_file(file: &SourceFile) -> bool {
    file.file_name.ends_with(".tsx") || file.file_name.ends_with(".jsx")
}

pub(super) fn enclosing_comment(
    ranges: &[CommentRange],
    text: &str,
    position: usize,
) -> Option<CommentRange> {
    ranges
        .iter()
        .find(|r| {
            (r.pos < position && position < r.end)
                || (position == r.end
                    && (r.kind == CommentRangeKind::SingleLine || position == text.len()))
        })
        .copied()
}

pub(super) fn is_doc_comment(range: &CommentRange, text: &str) -> bool {
    text[range.pos..].starts_with("/**") && !text[range.pos..].starts_with("/***")
}

pub(super) fn line_of_position(text: &str, position: usize) -> usize {
    text[..position.min(text.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
}
