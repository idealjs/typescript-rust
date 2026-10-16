#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::node_is_synthesized;
use super::Printer;
use tsox_frontend::format::mig::m4t_3::{
    get_lines_between_position_and_next_non_whitespace_character,
    get_lines_between_position_and_preceding_non_whitespace_character,
    get_lines_between_range_end_and_range_start, range_end_is_on_same_line_as_range_start,
    range_end_positions_are_on_same_line, range_is_on_single_line,
    range_start_positions_are_on_same_line,
};

use crate::mig::m4q::r33k12_defs::{
    EmitFlags, ListFormat, LF_MULTI_LINE, LF_NO_TRAILING_NEW_LINE, LF_NONE, LF_PREFER_NEW_LINE,
    LF_PRESERVE_LINES, TextRange, position_is_synthesized,
};

impl<'a> Printer<'a> {
    pub fn should_emit_on_single_line(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_emit_on_single_line"); 
        self.emit_context.emit_flags(node) & EmitFlags::SINGLE_LINE.0 != 0
    }

    pub fn should_elide_indentation(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_elide_indentation"); 
        self.emit_context.emit_flags(node) & EmitFlags::NO_INDENTATION.0 != 0
    }

    pub fn should_emit_on_new_line(&self, node: &Arc<Node>, format: ListFormat) -> bool { ::tsox_core::fntrace::enter("should_emit_on_new_line"); 
        if self.emit_context.emit_flags(node) & EmitFlags::START_ON_NEW_LINE.0 != 0 {
            return true;
        }
        format.0 & LF_PREFER_NEW_LINE != 0
    }

    pub fn get_effective_lines(&self, get_line_difference: impl Fn(bool) -> i32) -> i32 { ::tsox_core::fntrace::enter("get_effective_lines"); 
        let lines = get_line_difference(true);
        if lines == 0 {
            return get_line_difference(false);
        }
        lines
    }

    pub fn get_lines_between_nodes(
        &mut self,
        parent: &Arc<Node>,
        node1: &Arc<Node>,
        node2: &Arc<Node>,
    ) -> i32 { ::tsox_core::fntrace::enter("get_lines_between_nodes"); 
        if self.should_elide_indentation(parent) {
            return 0;
        }

        let parent = r39k17_skip_synthesized_parentheses(parent);
        let node1 = r39k17_skip_synthesized_parentheses(node1);
        let node2 = r39k17_skip_synthesized_parentheses(node2);

        if self.should_emit_on_new_line(node2, ListFormat(LF_NONE)) {
            return 1;
        }

        if self.current_source_file.is_some()
            && !node_is_synthesized(parent)
            && !node_is_synthesized(node1)
            && !node_is_synthesized(node2)
        {
            let source_file = self.current_source_file.as_deref().unwrap();
            if self.preserve_source_newlines {
                return self.get_effective_lines(|include_comments| {
                    get_lines_between_range_end_and_range_start(
                        node1.loc,
                        node2.loc,
                        source_file,
                        include_comments,
                    ) as i32
                });
            }
            return if range_end_is_on_same_line_as_range_start(node1.loc, node2.loc, source_file) {
                0
            } else {
                1
            };
        }

        0
    }

    pub fn get_leading_line_terminator_count(
        &mut self,
        parent_node: Option<&Arc<Node>>,
        first_child: Option<&Arc<Node>>,
        format: ListFormat,
    ) -> i32 { ::tsox_core::fntrace::enter("get_leading_line_terminator_count"); 
        if format.0 & LF_PRESERVE_LINES != 0 || self.preserve_source_newlines {
            if format.0 & LF_PREFER_NEW_LINE != 0 {
                return 1;
            }

            let Some(first_child) = first_child else {
                return if parent_node.is_none()
                    || (self.current_source_file.is_some()
                        && range_is_on_single_line(
                            parent_node.unwrap().loc,
                            self.current_source_file.as_deref().unwrap(),
                        ))
                {
                    0
                } else {
                    1
                };
            };
            if self.next_list_element_pos > 0 && first_child.pos() == self.next_list_element_pos {
                return 0;
            }
            if first_child.kind == SyntaxKind::JsxText {
                return 0;
            }
            if let (Some(source_file), Some(parent_node)) =
                (self.current_source_file.as_deref(), parent_node)
            {
                if !position_is_synthesized(parent_node.pos())
                    && !node_is_synthesized(first_child)
                    && first_child.parent().is_none()
                {
                    if self.preserve_source_newlines {
                        return self.get_effective_lines(|include_comments| {
                            get_lines_between_position_and_preceding_non_whitespace_character(
                                first_child.pos() as i64,
                                parent_node.pos() as i64,
                                source_file,
                                include_comments,
                            ) as i32
                        });
                    }
                    return if range_start_positions_are_on_same_line(
                        parent_node.loc,
                        first_child.loc,
                        source_file,
                    ) {
                        0
                    } else {
                        1
                    };
                }
            }
            if self.should_emit_on_new_line(first_child, format) {
                return 1;
            }
        }
        if format.0 & LF_MULTI_LINE != 0 {
            1
        } else {
            0
        }
    }

    pub fn get_closing_line_terminator_count(
        &mut self,
        parent_node: Option<&Arc<Node>>,
        last_child: Option<&Arc<Node>>,
        format: ListFormat,
        children_text_range: TextRange,
    ) -> i32 { ::tsox_core::fntrace::enter("get_closing_line_terminator_count"); 
        if format.0 & LF_PRESERVE_LINES != 0 || self.preserve_source_newlines {
            if format.0 & LF_PREFER_NEW_LINE != 0 {
                return 1;
            }
            let Some(last_child) = last_child else {
                return if parent_node.is_none()
                    || (self.current_source_file.is_some()
                        && range_is_on_single_line(
                            parent_node.unwrap().loc,
                            self.current_source_file.as_deref().unwrap(),
                        ))
                {
                    0
                } else {
                    1
                };
            };
            if let (Some(source_file), Some(parent_node)) =
                (self.current_source_file.as_deref(), parent_node)
            {
                if !position_is_synthesized(parent_node.pos())
                    && !node_is_synthesized(last_child)
                    && last_child
                        .parent()
                        .map_or(true, |p| Arc::ptr_eq(&p, parent_node))
                {
                    if self.preserve_source_newlines {
                        let end = std::cmp::max(last_child.end(), children_text_range.end()) as i64;
                        return self.get_effective_lines(|include_comments| {
                            get_lines_between_position_and_next_non_whitespace_character(
                                end,
                                parent_node.end() as i64,
                                source_file,
                                include_comments,
                            ) as i32
                        });
                    }
                    return if range_end_positions_are_on_same_line(
                        parent_node.loc,
                        last_child.loc,
                        source_file,
                    ) {
                        0
                    } else {
                        1
                    };
                }
            }
            if self.should_emit_on_new_line(last_child, format) {
                return 1;
            }
        }
        if format.0 & LF_MULTI_LINE != 0 && format.0 & LF_NO_TRAILING_NEW_LINE == 0 {
            1
        } else {
            0
        }
    }
}

fn r39k17_skip_synthesized_parentheses(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("r39k17_skip_synthesized_parentheses"); 
    let mut current = node;
    while current.kind == SyntaxKind::ParenthesizedExpression && node_is_synthesized(current) {
        match current.expression() {
            Some(expr) => current = expr,
            None => break,
        }
    }
    current
}
