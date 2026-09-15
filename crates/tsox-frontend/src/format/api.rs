//! Go format/api.go 的移植：FormatSpan 驱动入口与 FormatDocument /
//! FormatSelection / FormatOn* 请求。

use std::sync::Arc;

use tsox_core::core::text::TextRange;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use crate::ast::SourceFile;
use crate::format::rule_context::FormatRequestKind;
use crate::format::TextChange;

use super::indentation;
use super::lists;
use super::util;

/// Go prepareRangeContainsErrorFunction
pub(crate) fn prepare_range_contains_error_function(
    errors: &[TextRange],
    original_range: TextRange,
) -> Box<dyn Fn(TextRange) -> bool> {
    if errors.is_empty() {
        return Box::new(|_r| false);
    }
    let mut sorted: Vec<TextRange> = errors
        .iter()
        .filter(|d| original_range.overlaps(d))
        .copied()
        .collect();
    if sorted.is_empty() {
        return Box::new(|_r| false);
    }
    sorted.sort_by_key(|d| d.pos());
    Box::new(move |r: TextRange| {
        for err in &sorted {
            if r.end() <= err.pos() {
                return false;
            }
            if r.overlaps(err) {
                return true;
            }
        }
        false
    })
}

/// Go FormatSpan
pub(crate) fn format_span(
    span: TextRange,
    file: &Arc<SourceFile>,
    kind: FormatRequestKind,
    options: super::FormatCodeSettings,
    new_line_character: &str,
) -> Vec<TextChange> {
    let enclosing_node = indentation::find_enclosing_node(span, file);
    let scan_start = indentation::get_scan_start_position(&enclosing_node, span, file);
    let initial_indentation =
        indentation::get_indentation_for_node(&enclosing_node, &span, file, &options);
    let delta = indentation::get_own_or_inherited_delta(&enclosing_node, &options, file);
    super::span::format_span(
        file,
        span,
        options,
        new_line_character,
        kind,
        prepare_range_contains_error_function(&file.parse_error_spans, span),
        enclosing_node,
        initial_indentation,
        delta,
        scan_start,
    )
}

#[allow(dead_code)]
fn default_options() -> super::FormatCodeSettings {
    super::get_default_format_code_settings()
}

/// 带 FormatContext 设置的入口（Go 从 ctx 取 options）
pub(crate) fn format_document_with(ctx: &super::FormatContext, file: &Arc<SourceFile>) -> Vec<TextChange> {
    let options = ctx.settings.clone();
    let out = format_span(
        TextRange::new(0, file.text.len()),
        file,
        FormatRequestKind::FormatDocument,
        options,
        &ctx.new_line_character,
    );
    out
}

pub(crate) fn format_selection_with(
    ctx: &super::FormatContext,
    file: &Arc<SourceFile>,
    start: usize,
    end: usize,
) -> Vec<TextChange> {
    let line_start = util::line_start_position_for_position(file, start);
    format_span(
        TextRange::new(line_start, end),
        file,
        FormatRequestKind::FormatSelection,
        ctx.settings.clone(),
        &ctx.new_line_character,
    )
}

/// Go FormatDocument（引擎内部默认设置版本）
#[allow(dead_code)]
pub(crate) fn format_document(file: &Arc<SourceFile>, new_line_character: &str) -> Vec<TextChange> {
    format_span(
        TextRange::new(0, file.text.len()),
        file,
        FormatRequestKind::FormatDocument,
        default_options(),
        new_line_character,
    )
}

/// Go FormatSelection：起点对齐到行首
#[allow(dead_code)]
pub(crate) fn format_selection(
    file: &Arc<SourceFile>,
    start: usize,
    end: usize,
    new_line_character: &str,
) -> Vec<TextChange> {
    let line_start = util::line_start_position_for_position(file, start);
    format_span(
        TextRange::new(line_start, end),
        file,
        FormatRequestKind::FormatSelection,
        default_options(),
        new_line_character,
    )
}

/// Go formatNodeLines
fn format_node_lines(
    ctx: &super::FormatContext,
    file: &Arc<SourceFile>,
    node: &Arc<Node>,
    request_kind: FormatRequestKind,
) -> Vec<TextChange> {
    let token_start = util::token_pos_of_node(file, node);
    let line_start = util::line_start_position_for_position(file, token_start);
    format_span(
        TextRange::new(line_start, node.end()),
        file,
        request_kind,
        ctx.settings.clone(),
        &ctx.new_line_character,
    )
}

/// Go findImmediatelyPrecedingTokenOfKind。
/// Go astnav 对分号等不在 AST 上的 token 由 scanner 在 AST 子节点空档内
/// 合成（sourceFile.GetOrCreateToken）；这里在最后一个 AST token 与目标
/// 位置的空档内做同样的扫描合成。
fn find_immediately_preceding_token_of_kind(
    end: usize,
    expected_token_kind: SyntaxKind,
    file: &Arc<SourceFile>,
) -> Option<Arc<Node>> {
    let preceding = preceding_token_with_synthesis(file, end)?;
    if preceding.kind != expected_token_kind || preceding.end() != end {
        return None;
    }
    Some(preceding)
}

fn preceding_token_with_synthesis(file: &Arc<SourceFile>, position: usize) -> Option<Arc<Node>> {
    let ast_token = crate::astnav::find_preceding_token(&file.node, position);
    let scan_from = ast_token.as_ref().map(|t| t.end()).unwrap_or(0);
    if scan_from >= position {
        return ast_token;
    }
    let mut scanner = crate::scanner::Scanner::new(file.text.clone());
    scanner.set_language_variant(file.language_variant);
    scanner.set_range(scan_from, position);

    let mut synthesized: Option<(SyntaxKind, TextRange)> = None;
    loop {
        let kind = scanner.scan();
        if kind == SyntaxKind::EndOfFile {
            break;
        }
        let token_end = scanner.token_end();
        if token_end > position {
            break;
        }
        synthesized = Some((kind, TextRange::new(scanner.token_pos(), token_end)));
        if token_end == position {
            break;
        }
    }
    let (kind, loc) = synthesized?;
    let mut token = Arc::new(Node::with_loc(kind, crate::ast::NodeData::Token, loc));
    if let Some(t) = Arc::get_mut(&mut token) {
        let parent = indentation::find_enclosing_node(loc, file);
        t.set_parent(&parent);
    }
    Some(token)
}

pub(crate) fn format_on_opening_curly_with(
    ctx: &super::FormatContext,
    file: &Arc<SourceFile>,
    position: usize,
) -> Vec<TextChange> {
    format_on_opening_curly_inner(file, position, &ctx.settings.clone(), &ctx.new_line_character)
}

pub(crate) fn format_on_closing_curly_with(
    ctx: &super::FormatContext,
    file: &Arc<SourceFile>,
    position: usize,
) -> Vec<TextChange> {
    format_on_closing_curly_inner(file, position, &ctx.settings.clone(), &ctx.new_line_character)
}

pub(crate) fn format_on_semicolon_with(
    ctx: &super::FormatContext,
    file: &Arc<SourceFile>,
    position: usize,
) -> Vec<TextChange> {
    format_on_semicolon_inner(file, position, &ctx.settings.clone(), &ctx.new_line_character)
}

pub(crate) fn format_on_enter_with(
    ctx: &super::FormatContext,
    file: &Arc<SourceFile>,
    position: usize,
) -> Vec<TextChange> {
    format_on_enter_inner(file, position, &ctx.settings.clone(), &ctx.new_line_character)
}

/// Go FormatOnOpeningCurly
fn format_on_opening_curly_inner(
    file: &Arc<SourceFile>,
    position: usize,
    options: &super::FormatCodeSettings,
    new_line_character: &str,
) -> Vec<TextChange> {
    let Some(opening_curly) =
        find_immediately_preceding_token_of_kind(position, SyntaxKind::OpenBraceToken, file)
    else {
        return Vec::new();
    };
    let outermost =
        find_outermost_node_within_list_level_with_file(opening_curly.parent(), file)
            .unwrap_or_else(|| Arc::clone(&file.node));
    let token_pos = util::token_pos_of_node(file, &outermost);
    let line_start = util::line_start_position_for_position(file, token_pos);
    format_span(
        TextRange::new(line_start, position),
        file,
        FormatRequestKind::FormatOnOpeningCurlyBrace,
        options.clone(),
        new_line_character,
    )
}

/// Go FormatOnClosingCurly
fn format_on_closing_curly_inner(
    file: &Arc<SourceFile>,
    position: usize,
    options: &super::FormatCodeSettings,
    new_line_character: &str,
) -> Vec<TextChange> {
    let preceding_token =
        find_immediately_preceding_token_of_kind(position, SyntaxKind::CloseBraceToken, file);
    match find_outermost_node_within_list_level_with_file(preceding_token, file) {
        Some(node) => {
            format_node_lines(&ctx_options_holder(options, new_line_character), file, &node, FormatRequestKind::FormatOnClosingCurlyBrace)
        }
        None => Vec::new(),
    }
}

/// Go FormatOnSemicolon
fn format_on_semicolon_inner(
    file: &Arc<SourceFile>,
    position: usize,
    options: &super::FormatCodeSettings,
    new_line_character: &str,
) -> Vec<TextChange> {
    let semicolon =
        find_immediately_preceding_token_of_kind(position, SyntaxKind::SemicolonToken, file);
    match find_outermost_node_within_list_level_with_file(semicolon, file) {
        Some(node) => format_node_lines(&ctx_options_holder(options, new_line_character), file, &node, FormatRequestKind::FormatOnSemicolon),
        None => Vec::new(),
    }
}

fn ctx_options_holder(options: &super::FormatCodeSettings, new_line: &str) -> super::FormatContext {
    super::FormatContext { settings: options.clone(), new_line_character: new_line.to_string() }
}

/// Go FormatOnEnter
fn format_on_enter_inner(
    file: &Arc<SourceFile>,
    position: usize,
    options: &super::FormatCodeSettings,
    new_line_character: &str,
) -> Vec<TextChange> {
    let line = util::line_of_position(file, position);
    if line == 0 {
        return Vec::new();
    }
    let start_pos = file.line_map.line_starts[line - 1] as usize;
    let mut end_of_format_span = util::end_line_position(file, line);
    while end_of_format_span > start_pos {
        let Some(ch) = file.text.get(end_of_format_span..).and_then(|s| s.chars().next())
        else {
            end_of_format_span -= 1;
            continue;
        };
        if util::is_whitespace_single_line(ch) || util::is_line_break(ch) {
            end_of_format_span -= 1;
            continue;
        }
        break;
    }

    let ch = file
        .text
        .get(end_of_format_span..)
        .and_then(|s| s.chars().next());
    // Go 用 int：换行收尾可减到 -1，产生空/反向 span 即无编辑
    let span_end = if ch.is_some_and(util::is_line_break) {
        end_of_format_span as isize
    } else {
        end_of_format_span as isize + 1
    };
    if span_end <= start_pos as isize {
        return Vec::new();
    }

    if std::env::var_os("TSOX_DEBUG_FMT").is_some() {
        eprintln!("[on-enter] pos={} line={} span={}..{}", position, line, start_pos, span_end);
    }
    let edits = format_span(
        TextRange::new(start_pos, span_end as usize),
        file,
        FormatRequestKind::FormatOnEnter,
        options.clone(),
        new_line_character,
    );
    if std::env::var_os("TSOX_DEBUG_FMT").is_some() {
        eprintln!("[on-enter] edits={}", edits.len());
        for e in &edits {
            eprintln!("[on-enter-edit] {}..{} -> {:?}", e.pos, e.end, e.new_text);
        }
    }
    edits
}

/// Go findOutermostNodeWithinListLevel
fn find_outermost_node_within_list_level_with_file(
    node: Option<Arc<Node>>,
    file: &Arc<SourceFile>,
) -> Option<Arc<Node>> {
    let node = node?;
    let mut current = Some(node.clone());
    while let Some(cur) = current {
        let Some(parent) = cur.parent() else {
            return Some(cur);
        };
        if parent.end() != node.end() || lists::is_list_element(&parent, &cur, file) {
            return Some(cur);
        }
        current = Some(parent);
    }
    None
}
