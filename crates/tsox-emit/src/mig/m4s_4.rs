#![allow(unused_imports)]

use std::sync::Arc;

use super::m4q::r33k12_defs::{TextPos, TextRange, compute_ecma_line_starts, new_text_range, format_synthesized_comment, compute_line_of_position, calculate_indent, LF_NONE, LF_AMPERSAND_DELIMITED, LF_ASTERISK_DELIMITED, LF_BAR_DELIMITED, LF_COMMA_DELIMITED, LF_DELIMITERS_MASK, ListFormat, SynthesizedComment, WriteKind, EmitFlags, position_is_synthesized};
use tsox_core::stringutil::{is_line_break, split_lines};
use tsox_core::stringutil::mig::m3m_2::guess_indentation;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::node_is_synthesized;
use tsox_frontend::format::mig::m4t_3::{
    get_ecma_line_starts, get_lines_between_position_and_next_non_whitespace_character,
    get_lines_between_position_and_preceding_non_whitespace_character,
    get_lines_between_range_end_and_range_start, range_end_is_on_same_line_as_range_start,
    range_end_positions_are_on_same_line, range_is_on_single_line,
    range_start_positions_are_on_same_line,
};

use tsox_frontend::scanner::token_to_string;
use tsox_frontend::scanner::{CommentRange, CommentRangeKind};

use crate::mig::m4s::{EmitTextWriter, get_default_indent_size, get_indent_string};

#[path = "r39k17_defs.rs"]
pub mod r39k17_defs;
#[path = "r39k17b_defs.rs"]
pub mod r39k17b_defs;


pub struct Printer<'a> {
    pub writer: &'a mut dyn EmitTextWriter,
    pub preserve_source_newlines: bool,
    pub current_source_file: Option<Arc<SourceFile>>,
    pub emit_context: tsox_frontend::format::mig::m4o_2::EmitContext,
    pub write_kind: WriteKind,
    pub next_list_element_pos: usize,
}

impl<'a> Printer<'a> {
    pub fn write_comment_range(&mut self, comment: CommentRange) {
        let source_file = match &self.current_source_file {
            Some(file) => file.clone(),
            None => return,
        };

        let text = source_file.text.clone();
        let line_map = get_ecma_line_starts(&source_file);
        self.write_comment_range_worker(
            &text,
            &line_map,
            comment.kind,
            new_text_range(comment.pos, comment.end),
        );
    }

    pub fn write_delimiter(&mut self, format: ListFormat) {
        let delimiters = format.0 & LF_DELIMITERS_MASK;
        if delimiters == LF_NONE {
        } else if delimiters == LF_COMMA_DELIMITED {
            self.write_punctuation(",");
        } else if delimiters == LF_BAR_DELIMITED {
            self.write_space();
            self.write_punctuation("|");
        } else if delimiters == LF_ASTERISK_DELIMITED {
            self.write_space();
            self.write_punctuation("*");
            self.write_space();
        } else if delimiters == LF_AMPERSAND_DELIMITED {
            self.write_space();
            self.write_punctuation("&");
        }
    }

    pub fn write_keyword(&mut self, text: &str) {
        self.writer.write_keyword(text);
    }

    pub fn write_line(&mut self) {
        self.writer.write_line();
    }

    pub fn write_line_or_space(
        &mut self,
        parent_node: &Arc<Node>,
        prev_child_node: &Arc<Node>,
        next_child_node: &Arc<Node>,
    ) {
        if self.should_emit_on_single_line(parent_node) {
            self.write_space();
        } else if self.preserve_source_newlines {
            let lines = self.get_lines_between_nodes(parent_node, prev_child_node, next_child_node);
            if lines > 0 {
                self.write_line_repeat(lines as usize);
            } else {
                self.write_space();
            }
        } else {
            self.write_line();
        }
    }

    pub fn write_line_repeat(&mut self, count: usize) {
        for _ in 0..count {
            self.write_line();
        }
    }

    pub fn write_line_separators_after(&mut self, node: &Arc<Node>, parent: &Arc<Node>) {
        if self.preserve_source_newlines {
            let trailing_newlines = self.get_closing_line_terminator_count(
                Some(parent),
                Some(node),
                ListFormat(LF_NONE),
                TextRange { pos: -1, end: -1 },
            );
            if trailing_newlines > 0 {
                self.write_line_repeat(trailing_newlines as usize);
            }
        }
    }

    pub fn write_line_separators_and_indent_before(
        &mut self,
        node: &Arc<Node>,
        parent: &Arc<Node>,
    ) -> bool {
        if self.preserve_source_newlines {
            let leading_newlines =
                self.get_leading_line_terminator_count(Some(parent), Some(node), ListFormat(LF_NONE));
            if leading_newlines > 0 {
                self.write_lines_and_indent(leading_newlines as usize, false);
                return true;
            }
        }
        false
    }

    pub fn write_lines(&mut self, text: &str) {
        let lines = split_lines(text);
        let indentation = guess_indentation(&lines);
        for line in lines {
            let line = if indentation > 0 { &line[indentation..] } else { &line[..] };
            if !line.is_empty() {
                self.write_line();
                self.write(line);
            }
        }
    }

    pub fn write_lines_and_indent(&mut self, line_count: usize, write_space_if_not_indenting: bool) {
        if line_count > 0 {
            self.increase_indent();
            self.write_line_repeat(line_count);
        } else if write_space_if_not_indenting {
            self.write_space();
        }
    }

    pub fn write_literal(&mut self, text: &str) {
        self.writer.write_literal(text);
    }

    pub fn write_operator(&mut self, text: &str) {
        self.writer.write_operator(text);
    }

    pub fn write_parameter(&mut self, text: &str) {
        self.writer.write_parameter(text);
    }

    pub fn write_property(&mut self, text: &str) {
        self.writer.write_property(text);
    }

    pub fn write_punctuation(&mut self, text: &str) {
        self.writer.write_punctuation(text);
    }

    pub fn write_space(&mut self) {
        self.writer.write_space(" ");
    }

    pub fn write_symbol(&mut self, text: &str, opt_symbol: Option<&Symbol>) {
        match opt_symbol {
            None => self.write(text),
            Some(symbol) => self.writer.write_symbol(text, Some(symbol)),
        }
    }

    pub fn write_synthesized_comment(&mut self, comment: SynthesizedComment) {
        let text = format_synthesized_comment(&comment);
        let kind = if comment.kind == SyntaxKind::MultiLineCommentTrivia {
            CommentRangeKind::MultiLine
        } else {
            CommentRangeKind::SingleLine
        };
        let line_map = if kind == CommentRangeKind::MultiLine {
            compute_ecma_line_starts(&text)
        } else {
            Vec::new()
        };
        self.write_comment_range_worker(
            &text,
            &line_map,
            kind,
            new_text_range(0, text.len()),
        );
    }

    pub fn write_token_text(&mut self, token: SyntaxKind, write_kind: WriteKind, pos: usize) -> usize {
        let token_string = token_to_string(token);
        self.write_as(&token_string, write_kind);
        if position_is_synthesized(pos) {
            pos
        } else {
            pos + token_string.len()
        }
    }

    pub fn write_trailing_semicolon(&mut self) {
        self.writer.write_trailing_semicolon(";");
    }
}
