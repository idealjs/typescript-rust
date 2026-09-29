#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::position_is_synthesized;
use tsox_core::core::mig::m3j::last_or_nil;
use tsox_frontend::format::mig::m4t_4::greatest_end;

use tsox_frontend::format::mig::m4o_2::WriteKind;
use tsox_frontend::format::mig::m4o::ListFormat;

use super::m4p::{EmitFn, Printer};

impl Printer {
    pub fn emit_list_items(
        &mut self,
        emit: EmitFn,
        parent_node: Option<&Arc<Node>>,
        children: &[Arc<Node>],
        format: ListFormat,
        has_trailing_comma: bool,
        children_text_range: TextRange,
    ) {
        let may_emit_intervening_comments = !format.intersects(ListFormat::NO_INTERVENING_COMMENTS);
        let mut should_emit_intervening_comments = may_emit_intervening_comments;

        let mut leading_line_terminator_count = 0;
        if let Some(first) = children.first() {
            leading_line_terminator_count = self.get_leading_line_terminator_count(
                parent_node.unwrap(),
                Some(first),
                format,
            );
        }
        if leading_line_terminator_count > 0 {
            for _ in 0..leading_line_terminator_count {
                self.write_line();
            }
            should_emit_intervening_comments = false;
        } else if format.intersects(ListFormat::SPACE_BETWEEN_BRACES) {
            self.write_space();
        }

        if format.intersects(ListFormat::INDENTED) {
            self.increase_indent();
        }

        let parent_end: i64 = parent_node.map_or(-1, |p| p.end() as i64);

        let mut previous_sibling: Option<&Arc<Node>> = None;
        let mut should_decrease_indent_after_emit = false;
        for child in children {
            if format.intersects(ListFormat::ASTERISK_DELIMITED) {
                self.write_line();
                self.write_delimiter(format);
            } else if let Some(prev) = previous_sibling {
                if format.intersects(ListFormat::DELIMITERS_MASK) && prev.end() as i64 != parent_end {
                    if !self.comments_disabled && self.should_emit_trailing_comments(prev) {
                        self.emit_leading_comments(prev.end(), false);
                    }
                }
                self.write_delimiter(format);
                let separating_line_terminator_count =
                    self.get_separating_line_terminator_count(prev, child, format);
                if separating_line_terminator_count > 0 {
                    if format & ListFormat::LINES_MASK == ListFormat::NONE
                        && format.intersects(ListFormat::INDENTED)
                    {
                        self.increase_indent();
                        should_decrease_indent_after_emit = true;
                    }
                    if should_emit_intervening_comments
                        && format.intersects(ListFormat::DELIMITERS_MASK)
                        && !position_is_synthesized(child.pos())
                        && self.should_emit_leading_comments(child)
                    {
                        let comment_range = self.emit_context.comment_range(child);
                        self.emit_trailing_comments_of_position(
                            comment_range.pos(),
                            format.intersects(ListFormat::SPACE_BETWEEN_SIBLINGS),
                            true,
                        );
                    }
                    for _ in 0..separating_line_terminator_count {
                        self.write_line();
                    }
                    should_emit_intervening_comments = false;
                } else if format.intersects(ListFormat::SPACE_BETWEEN_SIBLINGS) {
                    self.write_space();
                }
            }

            if should_emit_intervening_comments && self.should_emit_leading_comments(child) {
                let comment_range = self.emit_context.comment_range(child);
                self.emit_trailing_comments_of_position(comment_range.pos(), false, false);
            } else {
                should_emit_intervening_comments = may_emit_intervening_comments;
            }

            self.next_list_element_pos = child.pos();
            emit(self, child);

            if should_decrease_indent_after_emit {
                self.decrease_indent();
                should_decrease_indent_after_emit = false;
            }
            previous_sibling = Some(child);
        }

        let skip_trailing_comments = self.comments_disabled
            || !previous_sibling.is_some_and(|prev| self.should_emit_trailing_comments(prev));
        let emit_trailing_comma = has_trailing_comma
            && format.intersects(ListFormat::ALLOW_TRAILING_COMMA)
            && format.intersects(ListFormat::COMMA_DELIMITED);
        if emit_trailing_comma {
            if let Some(prev) = previous_sibling {
                if !skip_trailing_comments {
                    self.emit_token(SyntaxKind::CommaToken, prev.end(), WriteKind::Punctuation, prev);
                } else {
                    self.write_punctuation(",");
                }
            } else {
                self.write_punctuation(",");
            }
        }

        if let Some(prev) = previous_sibling {
            if parent_end != prev.end() as i64
                && format.intersects(ListFormat::DELIMITERS_MASK)
                && !skip_trailing_comments
            {
                let comments_pos = if emit_trailing_comma && children_text_range.end() > 0 {
                    children_text_range.end()
                } else {
                    prev.end()
                };
                self.emit_leading_comments(comments_pos, false);
            }
        }

        if format.intersects(ListFormat::INDENTED) {
            self.decrease_indent();
        }

        let closing_line_terminator_count = self.get_closing_line_terminator_count(
            parent_node.unwrap(),
            last_or_nil(children).as_ref(),
            format,
            children_text_range,
        );
        if closing_line_terminator_count > 0 {
            for _ in 0..closing_line_terminator_count {
                self.write_line();
            }
        } else if format.intersects(ListFormat::SPACE_AFTER_LIST | ListFormat::SPACE_BETWEEN_BRACES) {
            self.write_space();
        }
    }
}
