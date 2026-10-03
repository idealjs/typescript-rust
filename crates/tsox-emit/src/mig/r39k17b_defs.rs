#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::NodeList;

use super::Printer;
use crate::mig::m4q::r33k12_defs::{
    LF_AMPERSAND_DELIMITED, LF_ASTERISK_DELIMITED, LF_BAR_DELIMITED, LF_COMMA_DELIMITED,
    LF_DELIMITERS_MASK, LF_NONE, TextRange, WriteKind, calculate_indent, compute_line_of_position,
    new_text_range,
};
use crate::mig::m4s::{EmitTextWriter, get_default_indent_size, get_indent_string};
use crate::printer::mig::m4m::{ChangeTrackerWriter, TriviaPositionKey};
use tsox_frontend::format::mig::m4o_2::PrintHandlers;
use tsox_core::stringutil::is_line_break;
use tsox_frontend::scanner::CommentRangeKind;

pub fn change_tracker_print_handlers(writer: &mut ChangeTrackerWriter) -> PrintHandlers { ::tsox_core::fntrace::enter("change_tracker_print_handlers"); 
    let ct: *mut ChangeTrackerWriter = writer;
    let set_pos = move |key: TriviaPositionKey| unsafe {
        (*ct).pos.insert(key, (*ct).last_non_trivia_position);
    };
    let set_end = move |key: TriviaPositionKey| unsafe {
        (*ct).end.insert(key, (*ct).last_non_trivia_position);
    };
    PrintHandlers {
        has_global_name: None,
        map_source_position: None,
        on_before_emit_node: {
            let set_pos = set_pos;
            Some(Box::new(move |node_opt: Option<&Arc<tsox_frontend::ast::Node>>| {
                if let Some(node) = node_opt {
                    set_pos(TriviaPositionKey::Node(Arc::as_ptr(node)));
                }
            }))
        },
        on_after_emit_node: {
            let set_end = set_end;
            Some(Box::new(move |node_opt: Option<&Arc<tsox_frontend::ast::Node>>| {
                if let Some(node) = node_opt {
                    set_end(TriviaPositionKey::Node(Arc::as_ptr(node)));
                }
            }))
        },
        on_before_emit_node_list: {
            let set_pos = set_pos;
            Some(Box::new(move |nodes_opt: Option<&NodeList>| {
                if let Some(nodes) = nodes_opt {
                    set_pos(TriviaPositionKey::NodeList(nodes as *const NodeList));
                }
            }))
        },
        on_after_emit_node_list: {
            let set_end = set_end;
            Some(Box::new(move |nodes_opt: Option<&NodeList>| {
                if let Some(nodes) = nodes_opt {
                    set_end(TriviaPositionKey::NodeList(nodes as *const NodeList));
                }
            }))
        },
        on_before_emit_token: {
            let set_pos = set_pos;
            Some(Box::new(move |node_opt: Option<&Arc<tsox_frontend::ast::Node>>| {
                if let Some(node) = node_opt {
                    set_pos(TriviaPositionKey::Node(Arc::as_ptr(node)));
                }
            }))
        },
        on_after_emit_token: {
            let set_end = set_end;
            Some(Box::new(move |node_opt: Option<&Arc<tsox_frontend::ast::Node>>| {
                if let Some(node) = node_opt {
                    set_end(TriviaPositionKey::Node(Arc::as_ptr(node)));
                }
            }))
        },
    }
}

impl<'a> Printer<'a> {
    pub fn write_comment_range_worker(
        &mut self,
        text: &str,
        line_map: &[crate::mig::m4q::r33k12_defs::TextPos],
        kind: CommentRangeKind,
        loc: TextRange,
    ) { ::tsox_core::fntrace::enter("write_comment_range_worker"); 
        if kind == CommentRangeKind::MultiLine {
            let indent_size = get_default_indent_size();
            let first_line = compute_line_of_position(line_map, loc.pos);
            let line_count = line_map.len();
            let mut first_comment_line_indent: i64 = -1;
            let mut pos = loc.pos as usize;
            let mut current_line = first_line;
            while pos < loc.end as usize {
                let next_line_start = if current_line + 1 == line_count {
                    text.len() + 1
                } else {
                    line_map[current_line + 1] as usize
                };

                if pos != loc.pos as usize {
                    if first_comment_line_indent == -1 {
                        first_comment_line_indent = calculate_indent(
                            text,
                            line_map[first_line] as usize,
                            loc.pos as usize,
                        );
                    }

                    let current_writer_indent_spacing =
                        (self.writer.get_indent() * indent_size) as i64;

                    let spaces_to_emit = current_writer_indent_spacing
                        - first_comment_line_indent
                        + calculate_indent(text, pos, next_line_start);
                    if spaces_to_emit > 0 {
                        let number_of_single_spaces_to_emit = spaces_to_emit % indent_size as i64;
                        let indent_size_space_string = get_indent_string(
                            ((spaces_to_emit - number_of_single_spaces_to_emit)
                                / indent_size as i64) as usize,
                            indent_size,
                        );

                        self.writer.raw_write(&indent_size_space_string);

                        let mut n = number_of_single_spaces_to_emit;
                        while n > 0 {
                            self.writer.raw_write(" ");
                            n -= 1;
                        }
                    } else {
                        self.writer.raw_write("");
                    }
                }

                let mut end = (loc.end as usize).min(next_line_start);
                for (scan, ch) in text[pos..end].char_indices() {
                    if is_line_break(ch) {
                        end = pos + scan;
                        break;
                    }
                }
                let current_line_text = text[pos..end].trim();
                if !current_line_text.is_empty() {
                    self.writer.write_comment(current_line_text);
                    if end != loc.end as usize {
                        self.write_line();
                    }
                } else {
                    self.writer.write_line_force(true);
                }

                pos = next_line_start;
                current_line += 1;
            }
        } else {
            self.writer
                .write_comment(&text[loc.pos as usize..loc.end as usize]);
        }
    }

    pub fn write_comment(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_comment"); 
        self.writer.write_comment(text);
    }

    pub fn write(&mut self, text: &str) { ::tsox_core::fntrace::enter("write"); 
        self.write_as(text, self.write_kind);
    }

    pub fn write_as(&mut self, text: &str, write_kind: WriteKind) { ::tsox_core::fntrace::enter("write_as"); 
        match write_kind {
            WriteKind::None => self.writer.write(text),
            WriteKind::Parameter => self.write_parameter(text),
            WriteKind::Keyword => self.write_keyword(text),
            WriteKind::Operator => self.write_operator(text),
            WriteKind::Property => self.write_property(text),
            WriteKind::Punctuation => self.write_punctuation(text),
            WriteKind::StringLiteral => self.writer.write_string_literal(text),
            WriteKind::Comment => self.write_comment(text),
            WriteKind::Literal => self.write_literal(text),
        }
    }

    pub fn increase_indent(&mut self) { ::tsox_core::fntrace::enter("increase_indent"); 
        self.writer.increase_indent();
    }
}
