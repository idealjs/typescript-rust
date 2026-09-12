//! Go span.go 的缩进与修剪：trimTrailingWhitespaces*、insertIndentation、
//! indentTriviaItems、indentMultilineComment。

use tsox_core::core::text::TextRange;

use crate::ast::SyntaxKind;
use crate::format::scanner::TextRangeWithKind;
use crate::format::span::FormatSpanWorker;

use super::super::util;

impl FormatSpanWorker {
    /// Go trimTrailingWhitespacesForRemainingRange
    pub(super) fn trim_trailing_whitespaces_for_remaining_range(
        &mut self,
        trivias: &[TextRangeWithKind],
    ) {
        let mut start_pos = self.original_range.pos();
        if !self.previous_range_is_zero() {
            start_pos = self
                .previous_range
                .as_ref()
                .expect("previous range")
                .loc
                .end();
        }

        for trivia in trivias {
            if util::is_comment(trivia.kind) {
                if start_pos < trivia.loc.pos() {
                    self.trim_trailing_whitespaces_for_positions(start_pos, trivia.loc.pos() - 1);
                }
                start_pos = trivia.loc.end() + 1;
            }
        }

        if start_pos < self.original_range.end() {
            self.trim_trailing_whitespaces_for_positions(start_pos, self.original_range.end());
        }
    }

    /// Go trimTrailingWitespacesForPositions
    fn trim_trailing_whitespaces_for_positions(&mut self, start_pos: usize, end_pos: usize) {
        let start_line = util::line_of_position(&self.source_file, start_pos);
        let end_line = util::line_of_position(&self.source_file, end_pos);
        self.trim_trailing_whitespaces_for_lines(start_line, end_line + 1, None);
    }

    /// Go trimTrailingWhitespacesForLines
    pub(super) fn trim_trailing_whitespaces_for_lines(
        &mut self,
        line1: usize,
        line2: usize,
        r: Option<&TextRangeWithKind>,
    ) {
        let line_count = self.source_file.line_map.line_starts.len();
        for line in line1..line2 {
            if line >= line_count {
                break;
            }
            let line_start_position = self.source_file.line_map.line_starts[line] as usize;
            let line_end_position = util::end_line_position(&self.source_file, line);

            // 不修剪注释或模板/字符串跨行内容
            if let Some(r) = r {
                if (util::is_comment(r.kind)
                    || util::is_string_or_regular_expression_or_template_literal(r.kind))
                    && r.loc.pos() <= line_end_position
                    && r.loc.end() > line_end_position
                {
                    continue;
                }
            }

            let whitespace_start =
                self.get_trailing_whitespace_start_position(line_start_position, line_end_position);
            if whitespace_start != -1 {
                self.record_delete(
                    whitespace_start as usize,
                    line_end_position + 1 - whitespace_start as usize,
                );
            }
        }
    }

    /// Go getTrailingWhitespaceStartPosition
    fn get_trailing_whitespace_start_position(&self, start: usize, end: usize) -> i64 {
        let text = &self.source_file.text;
        let mut pos = end as i64;
        while pos >= start as i64 {
            let Some(ch) = text.get(pos as usize..).and_then(|s| s.chars().next()) else {
                pos -= 1;
                continue;
            };
            if !util::is_whitespace_single_line(ch) {
                break;
            }
            pos -= 1;
        }
        if pos != end as i64 {
            return pos + 1;
        }
        -1
    }

    /// Go insertIndentation
    pub(super) fn insert_indentation(&mut self, pos: usize, indentation: i64, line_added: bool) {
        let indentation_string = util::get_indentation_string(
            indentation.max(0) as usize,
            self.options.editor_settings.convert_tabs_to_spaces,
            self.options.editor_settings.tab_size,
        );
        if line_added {
            // 规则已在 token 前加了换行，把缩进插到 token 最前
            self.record_replace(pos, 0, &indentation_string);
        } else {
            let (token_start_line, token_start_character) =
                util::line_and_byte_offset_of_position(&self.source_file, pos);
            let start_line_position =
                self.source_file.line_map.line_starts[token_start_line] as usize;
            let column =
                self.character_to_column(start_line_position, token_start_character);
            let different =
                self.indentation_is_different(&indentation_string, start_line_position);
            if indentation != column as i64 || different {
                self.record_replace(
                    start_line_position,
                    token_start_character,
                    &indentation_string,
                );
            }
        }
    }

    fn character_to_column(&self, start_line_position: usize, character_in_line: usize) -> u32 {
        let mut column: u32 = 0;
        let text = &self.source_file.text;
        for i in 0..character_in_line {
            if text.as_bytes()[start_line_position + i] == b'\t' {
                let tab_size = self.options.editor_settings.tab_size;
                if tab_size > 0 {
                    column += tab_size - (column % tab_size);
                }
            } else {
                column += 1;
            }
        }
        column
    }

    fn indentation_is_different(
        &self,
        indentation_string: &str,
        start_line_position: usize,
    ) -> bool {
        let text = &self.source_file.text;
        let end = start_line_position + indentation_string.len();
        if end > text.len() {
            return true;
        }
        indentation_string != &text[start_line_position..end]
    }

    /// Go indentTriviaItems
    pub(super) fn indent_trivia_items(
        &mut self,
        trivia: Vec<TextRangeWithKind>,
        comment_indentation: i64,
        mut indent_next_token_or_trivia: bool,
        mut indent_single_line: impl FnMut(&mut Self, &TextRangeWithKind),
    ) -> bool {
        for trivia_item in &trivia {
            let trivia_in_range = trivia_item.loc.contained_by(&self.original_range);
            match trivia_item.kind {
                SyntaxKind::MultiLineCommentTrivia => {
                    if trivia_in_range {
                        self.indent_multiline_comment(
                            trivia_item.loc,
                            comment_indentation,
                            !indent_next_token_or_trivia,
                            true,
                        );
                    }
                    indent_next_token_or_trivia = false;
                }
                SyntaxKind::SingleLineCommentTrivia => {
                    if indent_next_token_or_trivia && trivia_in_range {
                        indent_single_line(self, trivia_item);
                    }
                    indent_next_token_or_trivia = false;
                }
                SyntaxKind::NewLineTrivia => {
                    indent_next_token_or_trivia = true;
                }
                _ => {}
            }
        }
        indent_next_token_or_trivia
    }

    /// Go indentMultilineComment
    pub(super) fn indent_multiline_comment(
        &mut self,
        comment_range: TextRange,
        indentation: i64,
        first_line_is_indented: bool,
        indent_final_line: bool,
    ) {
        let start_line = util::line_of_position(&self.source_file, comment_range.pos());
        let end_line = util::line_of_position(&self.source_file, comment_range.end());

        if start_line == end_line {
            if !first_line_is_indented {
                self.insert_indentation(comment_range.pos(), indentation, false);
            }
            return;
        }

        let mut parts: Vec<TextRange> = Vec::new();
        let mut start_pos = comment_range.pos();
        for line in start_line..end_line {
            let end_of_line = util::end_line_position(&self.source_file, line);
            parts.push(TextRange::new(start_pos, end_of_line));
            start_pos = self.source_file.line_map.line_starts[line + 1] as usize;
        }

        if indent_final_line {
            parts.push(TextRange::new(start_pos, comment_range.end()));
        }

        if parts.is_empty() {
            return;
        }

        let start_line_pos = self.source_file.line_map.line_starts[start_line] as usize;
        let tab_size = self.options.editor_settings.tab_size;
        let (non_ws_char, non_ws_column) = util::find_first_non_whitespace_character_and_column(
            &self.source_file,
            start_line_pos,
            parts[0].pos(),
            tab_size,
        );

        let mut start_line = start_line;
        let mut start_index = 0;
        if first_line_is_indented {
            start_index = 1;
            start_line += 1;
        }

        let delta = indentation - non_ws_column as i64;
        for i in start_index..parts.len() {
            let line_pos = self.source_file.line_map.line_starts[start_line] as usize;
            let (non_ws_character, non_ws_column_i) = if i != 0 {
                util::find_first_non_whitespace_character_and_column(
                    &self.source_file,
                    parts[i].pos(),
                    parts[i].end(),
                    tab_size,
                )
            } else {
                (non_ws_char, non_ws_column)
            };
            let new_indentation = non_ws_column_i as i64 + delta;
            if new_indentation > 0 {
                let indentation_string = util::get_indentation_string(
                    new_indentation as usize,
                    self.options.editor_settings.convert_tabs_to_spaces,
                    self.options.editor_settings.tab_size,
                );
                self.record_replace(line_pos, non_ws_character, &indentation_string);
            } else {
                self.record_delete(line_pos, non_ws_character);
            }
            start_line += 1;
        }
    }
}
