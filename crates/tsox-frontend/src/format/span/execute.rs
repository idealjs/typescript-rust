//! Go span.go execute()：主循环、剩余 trivia、末尾 pair 补编辑。

use std::sync::Arc;

use crate::ast::SyntaxKind;
use tsox_core::core::text::TextRange;

use crate::format::scanner::FormattingScanner;
use crate::format::span::{FormatSpanWorker, LineAction};

use super::super::util;

impl FormatSpanWorker {
    pub(super) fn run(&mut self, scanner: &mut FormattingScanner) -> Vec<crate::format::TextChange> {
        self.indentation_on_last_indented_line = -1;
        self.last_indented_line = -1;
        self.formatting_context = crate::format::rule_context::FormattingContext::new(
            Arc::clone(&self.source_file),
            self.request_kind,
            self.options.clone(),
        );

        scanner.advance();

        if scanner.is_on_token() {
            let start_line = util::line_of_position(
                &self.source_file,
                util::token_pos_of_node(&self.source_file, &self.enclosing_node),
            );
            let mut undecorated_start_line = start_line;
            if super::super::indenter::has_decorators(&self.enclosing_node) {
                undecorated_start_line = util::line_of_position(
                    &self.source_file,
                    super::super::span::get_non_decorator_token_pos_of_node(
                        &self.source_file,
                        &self.enclosing_node,
                    ),
                );
            }
            let enclosing = Arc::clone(&self.enclosing_node);
            self.process_node(
                scanner,
                &enclosing,
                &enclosing,
                start_line,
                undecorated_start_line,
                self.initial_indentation,
                self.delta,
            );
        }

        // range 结束在 leading trivia 中间时，token 不在范围内，剩余
        // trivia 在这里补处理（缩进 + 逐条 processRange）
        let remaining_trivia = scanner.get_current_leading_trivia().to_vec();
        if !remaining_trivia.is_empty() {
            let mut indentation = self.initial_indentation;
            if super::super::indenter::node_will_indent_child(
                &self.options,
                &self.enclosing_node,
                None,
                Some(&self.source_file),
                false,
            ) {
                indentation += self.options.editor_settings.indent_size as i64;
            }

            let mut indent_next = true;
            for item in &remaining_trivia {
                let trivia_in_range = item.loc.contained_by(&self.original_range);
                match item.kind {
                    SyntaxKind::MultiLineCommentTrivia => {
                        if trivia_in_range {
                            self.indent_multiline_comment(item.loc, indentation, !indent_next, true);
                        }
                        indent_next = false;
                    }
                    SyntaxKind::SingleLineCommentTrivia => {
                        if indent_next && trivia_in_range {
                            let (start_line, start_char) = util::line_and_byte_offset_of_position(
                                &self.source_file,
                                item.loc.pos(),
                            );
                            let enclosing = Arc::clone(&self.enclosing_node);
                            self.process_range(
                                scanner,
                                item.clone(),
                                start_line,
                                start_char,
                                enclosing,
                                None,
                                None,
                            );
                            self.insert_indentation(item.loc.pos(), indentation, false);
                        }
                        indent_next = false;
                    }
                    SyntaxKind::NewLineTrivia => {
                        indent_next = true;
                    }
                    _ => {}
                }
            }

            if self.options.editor_settings.trim_trailing_whitespace {
                self.trim_trailing_whitespaces_for_remaining_range(&remaining_trivia);
            }
        }

        self.process_trailing_pair(scanner);
        std::mem::take(&mut self.edits)
    }

    /// Go execute() 末尾的 trailing-pair 连续性检查与补编辑。
    fn process_trailing_pair(&mut self, scanner: &mut FormattingScanner) {
        let previous = match &self.previous_range {
            Some(r) if !self.previous_range_is_zero() => r.clone(),
            _ => return,
        };
        if scanner.get_token_full_start() < self.original_range.end() {
            return;
        }
        let token_info = if scanner.is_on_eof() {
            scanner.read_eof_token_range()
        } else if scanner.is_on_token() {
            let enclosing = Arc::clone(&self.enclosing_node);
            match scanner.read_token_info(Some(&enclosing)).token {
                Some(t) => t,
                None => return,
            }
        } else {
            return;
        };

        // range 结束在 token 中间时 previousRange 与 tokenInfo 不连续，
        // 此时 pair 会跨越未处理的 token，忽略
        if token_info.loc.pos() != self.previous_range_trivia_end {
            return;
        }

        let mut parent = crate::astnav::find_preceding_token(&self.source_file.node, token_info.loc.end());
        parent = parent.and_then(|p| p.parent());
        let parent = parent
            .unwrap_or_else(|| self.previous_parent.clone().unwrap_or_else(|| Arc::clone(&self.source_file.node)));
        let line = util::line_of_position(&self.source_file, token_info.loc.pos());
        self.process_pair(
            scanner,
            token_info,
            line,
            parent.clone(),
            previous,
            self.previous_range_start_line,
            self.previous_parent.clone().unwrap_or_else(|| parent.clone()),
            Some(parent),
            None,
        );
    }
}

impl FormatSpanWorker {
    pub(super) fn previous_range_is_zero(&self) -> bool {
        match &self.previous_range {
            None => true,
            Some(r) => r.loc.pos() == 0 && r.loc.end() == 0 && r.kind == SyntaxKind::Unknown,
        }
    }
}

impl crate::format::scanner::FormatSpanWorkerLike for super::FormatSpanWorker {
    fn execute(&mut self, scanner: &mut FormattingScanner) -> Vec<crate::format::TextChange> {
        self.run(scanner)
    }
}
