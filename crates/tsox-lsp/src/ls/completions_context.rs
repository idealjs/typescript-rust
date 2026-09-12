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

/// 扫描从 from 起、token 起点 < to 的 token
pub(super) fn scan_tokens(text: &str, jsx: bool, from: usize, to: usize) -> Vec<ScanToken> {
    let mut scanner = new_scanner(text, jsx);
    let limit = to.min(text.len());
    scanner.set_range(from.min(text.len()), text.len());
    let mut out: Vec<ScanToken> = Vec::new();
    let mut guard = 0usize;
    loop {
        let kind = scanner.scan();
        if kind == SyntaxKind::EndOfFile {
            break;
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

pub(super) fn is_completion_list_blocker(
    context_token: Option<&ScanToken>,
    previous_token: Option<&ScanToken>,
    containing_token: Option<&ScanToken>,
    text: &str,
    position: usize,
    node_at_position: &Arc<Node>,
) -> bool {
    // Go isCompletionListBlocker 用 contextToken：正则尾（含 flags）与
    // 未闭合串的 end 位也算「在内」
    let literal_token = context_token.or(containing_token);
    if let Some(tok) = literal_token
        && is_in_string_or_regular_expression_or_template(tok, text, position)
    {
        return true;
    }
    let Some(context) = context_token else {
        return false;
    };
    is_solely_identifier_definition_location(context, node_at_position, previous_token, position)
        || is_dot_of_numeric_literal(context, text)
        || context.kind == SyntaxKind::BigIntLiteral
}

fn is_in_string_or_regular_expression_or_template(
    tok: &ScanToken,
    text: &str,
    position: usize,
) -> bool {
    let is_string = matches!(
        tok.kind,
        SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
    );
    let is_regex = tok.kind == SyntaxKind::RegularExpressionLiteral;
    if !is_string && !is_regex {
        return false;
    }
    if tok.pos < position && position < tok.end {
        return true;
    }
    position == tok.end && (is_unterminated_literal(tok, text) || is_regex)
}

fn is_unterminated_literal(tok: &ScanToken, text: &str) -> bool {
    let raw = &text[tok.pos.min(text.len())..tok.end.min(text.len())];
    let (quote, body) = match raw.chars().next() {
        Some(quote @ ('"' | '\'' | '`')) => (quote, &raw[1..]),
        _ => return false,
    };
    !body.ends_with(quote)
}

fn is_dot_of_numeric_literal(tok: &ScanToken, text: &str) -> bool {
    tok.kind == SyntaxKind::NumericLiteral && text[tok.pos..tok.end].ends_with('.')
}

fn is_solely_identifier_definition_location(
    context: &ScanToken,
    node_at_position: &Arc<Node>,
    previous_token: Option<&ScanToken>,
    position: usize,
) -> bool {
    let blocked_by_switch = match context.kind {
        SyntaxKind::ColonToken => node_at_position.kind == SyntaxKind::BindingElement,
        SyntaxKind::OpenBracketToken | SyntaxKind::DotToken => {
            node_at_position.kind == SyntaxKind::ArrayBindingPattern
        }
        SyntaxKind::OpenParenToken => {
            node_at_position.kind == SyntaxKind::CatchClause
                || is_function_like_but_not_constructor(node_at_position)
        }
        SyntaxKind::OpenBraceToken => node_at_position.kind == SyntaxKind::EnumDeclaration,
        SyntaxKind::LessThanToken => {
            matches!(
                node_at_position.kind,
                SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::TypeAliasDeclaration
            ) || is_function_like_kind(node_at_position.kind)
        }
        SyntaxKind::CommaToken => {
            matches!(
                node_at_position.kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::VariableStatement
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::ArrayBindingPattern
                    | SyntaxKind::TypeAliasDeclaration
            ) || is_function_like_but_not_constructor(node_at_position)
        }
        SyntaxKind::DotDotDotToken => {
            node_at_position.kind == SyntaxKind::Parameter
                || node_at_position
                    .parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::ArrayBindingPattern)
        }
        SyntaxKind::PublicKeyword | SyntaxKind::PrivateKeyword | SyntaxKind::ProtectedKeyword => {
            node_at_position.kind == SyntaxKind::Parameter
                && node_at_position
                    .parent()
                    .as_ref()
                    .is_none_or(|p| p.kind != SyntaxKind::Constructor)
        }
        SyntaxKind::AsKeyword => matches!(
            node_at_position.kind,
            SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier | SyntaxKind::NamespaceImport
        ),
        SyntaxKind::TypeKeyword => node_at_position.kind != SyntaxKind::ImportSpecifier,
        SyntaxKind::AsteriskToken => {
            is_function_like_kind(node_at_position.kind)
                && node_at_position.kind != SyntaxKind::MethodDeclaration
        }
        SyntaxKind::ClassKeyword
        | SyntaxKind::EnumKeyword
        | SyntaxKind::InterfaceKeyword
        | SyntaxKind::FunctionKeyword
        | SyntaxKind::VarKeyword
        | SyntaxKind::ImportKeyword
        | SyntaxKind::LetKeyword
        | SyntaxKind::ConstKeyword
        | SyntaxKind::InferKeyword => true,
        _ => false,
    };
    blocked_by_switch
}

fn is_function_like_but_not_constructor(node: &Arc<Node>) -> bool {
    is_function_like_kind(node.kind) && node.kind != SyntaxKind::Constructor
}

pub(super) fn find_ancestor(
    node: &Arc<Node>,
    predicate: &dyn Fn(&Arc<Node>) -> bool,
) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        if predicate(&n) {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

pub(super) fn line_of_position(text: &str, position: usize) -> usize {
    text[..position.min(text.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
}
