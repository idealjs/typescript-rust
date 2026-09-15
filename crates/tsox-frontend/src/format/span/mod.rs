//! Go format/span.go 的忠实移植：formatSpanWorker 的状态与入口。
//! 递归遍历在 process.rs，execute 主循环在 execute.rs，token 消费在
//! consume.rs，编辑产出在 edits.rs，缩进与修剪在 trim.rs。

use std::sync::Arc;

use crate::ast::node::Node;
use tsox_core::core::text::TextRange;

use crate::format::rule_context::{FormatRequestKind, FormattingContext};
use crate::format::scanner::{new_formatting_scanner, FormattingScanner, TextRangeWithKind};
use crate::format::{FormatCodeSettings, TextChange};

mod consume;
mod edits;
mod execute;
mod process;
mod trim;

pub(crate) struct FormatSpanWorker {
    pub(super) original_range: TextRange,
    pub(super) enclosing_node: Arc<Node>,
    pub(super) initial_indentation: i64,
    pub(super) delta: i64,
    pub(super) request_kind: FormatRequestKind,
    pub(super) range_contains_error: Box<dyn Fn(TextRange) -> bool>,
    pub(super) source_file: Arc<crate::ast::SourceFile>,
    pub(super) new_line_character: String,
    pub(super) options: FormatCodeSettings,

    pub(super) formatting_context: FormattingContext,

    pub(super) edits: Vec<TextChange>,
    pub(super) previous_range: Option<TextRangeWithKind>,
    pub(super) previous_range_trivia_end: usize,
    pub(super) previous_parent: Option<Arc<Node>>,
    pub(super) previous_range_start_line: usize,
    pub(super) previous_range_start_character: usize,

    pub(super) child_context_node: Option<Arc<Node>>,
    pub(super) last_indented_line: i64,
    pub(super) indentation_on_last_indented_line: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineAction {
    None,
    LineAdded,
    LineRemoved,
}

/// Go FormatSpan：驱动格式化扫描器跑完整个 span。
#[allow(clippy::too_many_arguments)]
pub(crate) fn format_span(
    source_file: &Arc<crate::ast::SourceFile>,
    span: TextRange,
    options: FormatCodeSettings,
    new_line_character: &str,
    kind: FormatRequestKind,
    range_contains_error: Box<dyn Fn(TextRange) -> bool>,
    enclosing_node: Arc<Node>,
    initial_indentation: i64,
    delta: i64,
    scan_start: usize,
) -> Vec<TextChange> {
    let mut worker = FormatSpanWorker {
        original_range: span,
        enclosing_node,
        initial_indentation,
        delta,
        request_kind: kind,
        range_contains_error,
        source_file: Arc::clone(source_file),
        new_line_character: new_line_character.to_string(),
        options: options.clone(),
        formatting_context: FormattingContext::new(Arc::clone(source_file), kind, options),
        edits: Vec::new(),
        previous_range: None,
        previous_range_trivia_end: 0,
        previous_parent: None,
        previous_range_start_line: 0,
        previous_range_start_character: 0,
        child_context_node: None,
        last_indented_line: -1,
        indentation_on_last_indented_line: -1,
    };
    new_formatting_scanner(
        &source_file.text,
        source_file.language_variant,
        scan_start,
        span.end(),
        &mut worker,
    )
}

#[allow(dead_code)]
pub(super) type ScannerRef<'a> = &'a mut FormattingScanner;

/// Go getNonDecoratorTokenPosOfNode
pub(crate) fn get_non_decorator_token_pos_of_node(
    file: &crate::ast::SourceFile,
    node: &Node,
) -> usize {
    let mut last_decorator_end: Option<usize> = None;
    if super::indenter::has_decorators(node) {
        if let Some(mods) = node.modifiers() {
            if let Some(last) = mods.list.nodes.iter().rfind(|n| n.kind == crate::ast::SyntaxKind::Decorator) {
                last_decorator_end = Some(last.end());
            }
        }
    }
    match last_decorator_end {
        None => super::util::token_pos_of_node(file, node),
        Some(end) => crate::scanner::skip_trivia(&file.text, end),
    }
}
