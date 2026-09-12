//! Go span.go consumeTokenAndAdvanceScanner：trivia 处理、缩进判定、
//! 缩进应用，最后推进格式化扫描器。

use std::sync::Arc;

use crate::ast::node::Node;
use crate::format::indenter::IndenterRef;
use crate::format::scanner::{FormattingScanner, TextRangeWithKind, TokenInfo};
use crate::format::span::{FormatSpanWorker, LineAction};

use super::super::util;

impl FormatSpanWorker {
    /// Go consumeTokenAndAdvanceScanner
    pub(super) fn consume_token_and_advance_scanner(
        &mut self,
        scanner: &mut FormattingScanner,
        current_token_info: TokenInfo,
        parent: &Arc<Node>,
        dynamic_indentation: &IndenterRef,
        container: &Arc<Node>,
        is_list_end_token: bool,
    ) {
        let last_trivia_was_new_line = scanner.last_trailing_trivia_was_new_line();
        let Some(token) = current_token_info.token.clone() else {
            return;
        };
        let mut indent_token = false;

        if !current_token_info.leading_trivia.is_empty() {
            let child_context = self.child_context_node.clone();
            self.process_trivia(
                scanner,
                &current_token_info.leading_trivia,
                Arc::clone(parent),
                child_context,
                Some(dynamic_indentation),
            );
        }

        let mut line_action = LineAction::None;
        let is_token_in_range = token.loc.contained_by(&self.original_range);
        let (token_start_line, token_start_char) =
            util::line_and_byte_offset_of_position(&self.source_file, token.loc.pos());

        if is_token_in_range {
            let range_has_error = (self.range_contains_error)(token.loc);
            let save_previous_range = self.previous_range.clone();
            let child_context = self.child_context_node.clone();
            line_action = self.process_range(
                scanner,
                token.clone(),
                token_start_line,
                token_start_char,
                Arc::clone(parent),
                child_context,
                Some(dynamic_indentation),
            );
            if !range_has_error {
                if line_action == LineAction::None {
                    match save_previous_range {
                        Some(prev) => {
                            let prev_end_line =
                                util::line_of_position(&self.source_file, prev.loc.end());
                            // 仅当 trivia 换行且 token 起始行不同于前项结束行才缩进
                            indent_token =
                                last_trivia_was_new_line && token_start_line != prev_end_line;
                        }
                        None => {
                            indent_token = last_trivia_was_new_line;
                        }
                    }
                } else {
                    indent_token = line_action == LineAction::LineAdded;
                }
            }
        }

        if !current_token_info.trailing_trivia.is_empty() {
            self.previous_range_trivia_end = current_token_info
                .trailing_trivia
                .last()
                .map(|t| t.loc.end())
                .unwrap_or(self.previous_range_trivia_end);
            for trivia in &current_token_info.trailing_trivia {
                if util::is_comment(trivia.kind) && !trivia.loc.contained_by(&self.original_range) {
                    self.previous_range_trivia_end = trivia.loc.pos();
                    break;
                }
            }
            let child_context = self.child_context_node.clone();
            self.process_trivia(
                scanner,
                &current_token_info.trailing_trivia,
                Arc::clone(parent),
                child_context,
                Some(dynamic_indentation),
            );
        }

        if indent_token {
            let mut token_indentation: i64 = -1;
            if is_token_in_range && !(self.range_contains_error)(token.loc) {
                token_indentation = dynamic_indentation.borrow().get_indentation_for_token(
                    token_start_line,
                    token.kind,
                    container,
                    is_list_end_token,
                );
            }
            let mut indent_next_token_or_trivia = true;
            if !current_token_info.leading_trivia.is_empty() {
                let comment_indentation = dynamic_indentation.borrow().get_indentation_for_comment(
                    token.kind,
                    token_indentation,
                    container,
                );
                indent_next_token_or_trivia = self.indent_trivia_items(
                    current_token_info.leading_trivia.clone(),
                    comment_indentation,
                    indent_next_token_or_trivia,
                    |w, item: &TextRangeWithKind| {
                        w.insert_indentation(item.loc.pos(), comment_indentation, false);
                    },
                );
            }

            if token_indentation != -1 && indent_next_token_or_trivia {
                self.insert_indentation(
                    token.loc.pos(),
                    token_indentation,
                    line_action == LineAction::LineAdded,
                );
                self.last_indented_line = token_start_line as i64;
                self.indentation_on_last_indented_line = token_indentation;
            }
        }

        scanner.advance();

        self.child_context_node = Some(Arc::clone(parent));
    }
}
