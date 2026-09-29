#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::node_data_generated::is_string_literal;
use tsox_frontend::ast::utilities::{
    get_source_file_of_node, is_in_json_file, is_member_name, is_prologue_directive,
    node_is_synthesized, position_is_synthesized,
};
use tsox_frontend::format::mig::m4t_2::GET_LITERAL_TEXT_FLAGS_JSX_ATTRIBUTE_ESCAPE;
use tsox_frontend::scanner::mig::m3i::get_source_text_of_node_from_source_file;
use tsox_frontend::scanner::{get_leading_comment_ranges, get_trailing_comment_ranges};
use tsox_frontend::ast::mig::m3c;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::{Node, SyntaxKind};
use tsox_frontend::scanner::CommentRange;

include!("r33k3_types.rs");
include!("r36k23_defs.rs");

#[path = "r39k06_defs.rs"]
pub mod r39k06_defs;
pub use r39k06_defs::*;

pub struct Printer {
    pub options: PrinterOptions,
    pub emit_context: EmitContext,
    pub current_source_file: Option<Arc<SourceFile>>,
    pub unique_helper_names: HashMap<String, Option<Arc<Node>>>,
    pub external_helpers_module_name: Option<Arc<Node>>,
    pub next_list_element_pos: usize,
    pub writer: Box<dyn EmitTextWriter>,
    pub write_kind: WriteKind,
    pub source_maps_disabled: bool,
    pub source_map_generator: Option<SourceMapGenerator>,
    pub source_map_source: Option<Arc<SourceFile>>,
    pub source_map_source_index: SourceIndex,
    pub source_map_source_is_json: bool,
    pub most_recent_source_map_source: Option<Arc<SourceFile>>,
    pub most_recent_source_map_source_index: SourceIndex,
    pub container_pos: i32,
    pub container_end: i32,
    pub declaration_list_container_end: i32,
    pub detached_comments_info: Vec<DetachedCommentsInfo>,
    pub comments_disabled: bool,
    pub in_extends: bool,
    pub name_generator: NameGenerator,
    pub make_file_level_optimistic_unique_name: Option<Box<dyn Fn(&str) -> String>>,
    pub comment_state_arena: Vec<CommentState>,
    pub source_map_state_arena: Vec<SourceMapState>,
    pub has_global_name: Option<String>,
    pub on_before_emit_node: Option<Box<dyn FnMut(Option<&Node>)>>,
    pub on_after_emit_node: Option<Box<dyn FnMut(Option<&Node>)>>,
    pub on_before_emit_token: Option<Box<dyn FnMut(Option<&Node>)>>,
    pub on_after_emit_token: Option<Box<dyn FnMut(Option<&Node>)>>,
    pub source_map_line_char_cache: Option<tsox_frontend::format::mig::m4t_3::LineCharacterCache>,
}

impl Printer {
    pub fn write_as(&mut self, text: &str, write_kind: WriteKind) {
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

    pub fn write(&mut self, text: &str) {
        let kind = self.write_kind;
        self.write_as(text, kind);
    }

    pub fn set_write_kind(&mut self, kind: WriteKind) -> WriteKind {
        let previous = self.write_kind;
        self.write_kind = kind;
        previous
    }

    pub fn write_comment(&mut self, text: &str) {
        self.writer.write_comment(text);
    }

    pub fn increase_indent(&mut self) {
        self.writer.increase_indent();
    }

    pub fn increase_indent_if(&mut self, indent_requested: bool) {
        if indent_requested {
            self.increase_indent();
        }
    }

    pub fn get_literal_text_of_node(
        &mut self,
        node: &Arc<Node>,
        source_file: Option<&Arc<Node>>,
        mut flags: GetLiteralTextFlags,
    ) -> String {
        if is_string_literal(node) {
            if let Some(text_source_node) = self.emit_context.text_source(node) {
                let text = match text_source_node.kind {
                    SyntaxKind::NumericLiteral => text_source_node.text().to_string(),
                    SyntaxKind::Identifier
                    | SyntaxKind::PrivateIdentifier
                    | SyntaxKind::JsxNamespacedName => self.get_text_of_node(&text_source_node, false),
                    _ => {
                        return self.get_literal_text_of_node(
                            &text_source_node,
                            get_source_file_of_node(&text_source_node).as_ref(),
                            flags,
                        );
                    }
                };
                if flags & GET_LITERAL_TEXT_FLAGS_JSX_ATTRIBUTE_ESCAPE != 0 {
                    return format!("\"{}\"", escape_jsx_attribute_string(&text, QUOTE_CHAR_DOUBLE_QUOTE));
                }
                if flags & GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE != 0
                    || self.emit_context.emit_flags(node) & (EF_NO_ASCII_ESCAPING as u32) != 0
                {
                    return format!("\"{}\"", escape_string(&text, QUOTE_CHAR_DOUBLE_QUOTE));
                }
                return format!("\"{}\"", escape_non_ascii_string(&text, QUOTE_CHAR_DOUBLE_QUOTE));
            }
        }
        if self.emit_context.emit_flags(node) & (EF_NO_ASCII_ESCAPING as u32) != 0 {
            flags |= GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE;
        }
        if self.options.target >= SCRIPT_TARGET_ES2021 {
            flags |= GET_LITERAL_TEXT_FLAGS_ALLOW_NUMERIC_SEPARATOR;
        }
        let _ = source_file;
        get_literal_text(node, self.current_source_file.as_ref(), flags)
    }

    pub fn get_text_of_node(&mut self, node: &Arc<Node>, include_trivia: bool) -> String {
        if is_member_name(node) && self.emit_context.has_auto_generate_info(node) {
            return self.get_text_of_node(node, false);
        }

        if is_string_literal(node) {
            if let Some(text_source_node) = self.emit_context.text_source(node) {
                return self.get_text_of_node(&text_source_node, include_trivia);
            }
        }

        let can_use_source_file = self.current_source_file.is_some()
            && node.parent().is_some()
            && !node_is_synthesized(node);

        match node.kind {
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier | SyntaxKind::JsxNamespacedName => {
                if !can_use_source_file || !self.r36k23_source_file_is_current(node) {
                    return node.text().to_string();
                }
            }
            SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead
            | SyntaxKind::TemplateMiddle
            | SyntaxKind::TemplateTail => {
                return self.get_literal_text_of_node(node, None, GET_LITERAL_TEXT_FLAGS_NONE);
            }
            _ => panic!("unexpected node: {:?}", node.kind),
        }
        let source_file = self.current_source_file.as_deref().unwrap();
        get_source_text_of_node_from_source_file(source_file, node, include_trivia)
    }

    pub fn get_lines_between_nodes(&mut self, parent: &Node, node1: &Node, node2: &Node) -> i32 {
        if self.should_elide_indentation(parent) {
            return 0;
        }

        let parent = r36k23_skip_synthesized_parentheses(parent);
        let node1 = r36k23_skip_synthesized_parentheses(node1);
        let node2 = r36k23_skip_synthesized_parentheses(node2);

        if self.should_emit_on_new_line(node2, LF_NONE) {
            return 1;
        }

        if self.current_source_file.is_some()
            && !node_is_synthesized(parent)
            && !node_is_synthesized(node1)
            && !node_is_synthesized(node2)
        {
            let source_file = self.current_source_file.as_deref().unwrap();
            if self.options.preserve_source_newlines {
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

    pub fn get_effective_lines(&self, get_line_difference: impl Fn(bool) -> i32) -> i32 {
        let lines = get_line_difference(true);
        if lines == 0 {
            return get_line_difference(false);
        }
        lines
    }

    pub fn get_leading_line_terminator_count(
        &mut self,
        parent_node: Option<&Arc<Node>>,
        first_child: Option<&Arc<Node>>,
        format: ListFormat,
    ) -> i32 {
        if format & LF_PRESERVE_LINES != 0 || self.options.preserve_source_newlines {
            if format & LF_PREFER_NEW_LINE != 0 {
                return 1;
            }

            let Some(first_child) = first_child else {
                return if parent_node.is_none()
                    || (self.current_source_file.is_some()
                        && range_is_on_single_line(parent_node.unwrap().loc, self.current_source_file.as_deref().unwrap()))
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
                if !position_is_synthesized(parent_node.pos()) && !node_is_synthesized(first_child) && first_child.parent().is_none()
                {
                    if self.options.preserve_source_newlines {
                        return self.get_effective_lines(|include_comments| {
                            get_lines_between_position_and_preceding_non_whitespace_character(
                                first_child.pos() as i64,
                                parent_node.pos() as i64,
                                source_file,
                                include_comments,
                            ) as i32
                        });
                    }
                    return if range_start_positions_are_on_same_line(parent_node.loc, first_child.loc, source_file)
                    {
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
        if format & LF_MULTI_LINE != 0 {
            1
        } else {
            0
        }
    }

    pub fn get_separating_line_terminator_count(
        &mut self,
        previous_node: Option<&Arc<Node>>,
        next_node: Option<&Arc<Node>>,
        format: ListFormat,
    ) -> i32 {
        if format & LF_PRESERVE_LINES != 0 || self.options.preserve_source_newlines {
            let (Some(previous_node), Some(next_node)) = (previous_node, next_node) else {
                return 0;
            };
            if next_node.kind == SyntaxKind::JsxText {
                return 0;
            }
            if self.current_source_file.is_some()
                && !node_is_synthesized(previous_node)
                && !node_is_synthesized(next_node)
            {
                let source_file = self.current_source_file.as_deref().unwrap();
                if self.options.preserve_source_newlines
                    && sibling_node_positions_are_comparable(
                        &tsox_frontend::format::mig::m4t_4::EmitContext::default(),
                        previous_node,
                        next_node,
                    )
                {
                    return self.get_effective_lines(|include_comments| {
                        get_lines_between_range_end_and_range_start(
                            previous_node.loc,
                            next_node.loc,
                            source_file,
                            include_comments,
                        ) as i32
                    });
                }
                if !self.options.preserve_source_newlines
                    && original_nodes_have_same_parent(
                        &tsox_frontend::format::mig::m4t_4::EmitContext::default(),
                        previous_node,
                        next_node,
                    )
                {
                    return if range_end_is_on_same_line_as_range_start(previous_node.loc, next_node.loc, source_file)
                    {
                        0
                    } else {
                        1
                    };
                }
                return if format & LF_PREFER_NEW_LINE != 0 { 1 } else { 0 };
            }
            if self.should_emit_on_new_line(previous_node, format)
                || self.should_emit_on_new_line(next_node, format)
            {
                return 1;
            }
        } else if let Some(next_node) = next_node {
            if self.should_emit_on_new_line(next_node, LF_NONE) {
                return 1;
            }
        }
        if format & LF_MULTI_LINE != 0 {
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
    ) -> i32 {
        if format & LF_PRESERVE_LINES != 0 || self.options.preserve_source_newlines {
            if format & LF_PREFER_NEW_LINE != 0 {
                return 1;
            }
            let Some(last_child) = last_child else {
                return if parent_node.is_none()
                    || (self.current_source_file.is_some()
                        && range_is_on_single_line(parent_node.unwrap().loc, self.current_source_file.as_deref().unwrap()))
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
                    if self.options.preserve_source_newlines {
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
                    return if range_end_positions_are_on_same_line(parent_node.loc, last_child.loc, source_file) {
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
        if format & LF_MULTI_LINE != 0 && format & LF_NO_TRAILING_NEW_LINE == 0 {
            1
        } else {
            0
        }
    }

    pub fn should_emit_comments(&self, node: &Node) -> bool {
        !self.comments_disabled
            && self.current_source_file.is_some()
            && node.kind != SyntaxKind::SourceFile
    }

    pub fn should_write_comment(&self, comment: CommentRange) -> bool {
        !self.options.only_print_jsdoc_style
            || (self.current_source_file.is_some()
                && is_jsdoc_like_text(self.current_source_file.as_deref().unwrap().text(), &comment))
            || (self.current_source_file.is_some()
                && is_pinned_comment(self.current_source_file.as_deref().unwrap().text(), &comment))
    }

    pub fn should_emit_indented(&self, node: &Node) -> bool {
        self.emit_context.k06_emit_flags(node) & (EF_INDENTED as u32) != 0
    }

    pub fn should_elide_indentation(&self, node: &Node) -> bool {
        self.emit_context.k06_emit_flags(node) & (EF_NO_INDENTATION as u32) != 0
    }

    pub fn should_emit_on_single_line(&self, node: &Node) -> bool {
        self.emit_context.k06_emit_flags(node) & (EF_SINGLE_LINE as u32) != 0
    }

    pub fn should_emit_on_multiple_lines(&self, node: &Node) -> bool {
        self.emit_context.k06_emit_flags(node) & (EF_MULTI_LINE as u32) != 0
    }

    pub fn should_emit_block_function_body_on_single_line(&mut self, body: &Arc<Node>) -> bool {
        if self.should_emit_on_single_line(body) {
            return true;
        }

        if body.multi_line() {
            return false;
        }

        if !node_is_synthesized(body) && self.current_source_file.is_some() && !range_is_on_single_line(body.loc, self.current_source_file.as_deref().unwrap()) {
            return false;
        }

        let statements = m3c::statements(body);
        let statements_loc = m3c::statement_list(body)
            .map(|l| l.loc)
            .unwrap_or_else(TextRange::undefined);
        if self.get_leading_line_terminator_count(Some(body), statements.first(), LF_PRESERVE_LINES) > 0
            || self.get_closing_line_terminator_count(Some(body), statements.last(), LF_PRESERVE_LINES, statements_loc) > 0
        {
            return false;
        }

        let mut previous_statement: Option<&Arc<Node>> = None;
        for statement in statements {
            if self.get_separating_line_terminator_count(previous_statement, Some(statement), LF_PRESERVE_LINES) > 0 {
                return false;
            }
            previous_statement = Some(statement);
        }

        true
    }

    pub fn should_emit_on_new_line(&self, node: &Node, format: ListFormat) -> bool {
        if self.emit_context.k06_emit_flags(node) & (EF_START_ON_NEW_LINE as u32) != 0 {
            return true;
        }
        format & LF_PREFER_NEW_LINE != 0
    }

    pub fn should_emit_source_maps(&self, node: &Node) -> bool {
        !self.source_maps_disabled
            && self.source_map_source.is_some()
            && node.kind != SyntaxKind::SourceFile
            && !is_in_json_file(node)
    }

    pub fn should_emit_token_source_maps(
        &self,
        token: SyntaxKind,
        pos: i32,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> bool {
        flags & TEF_NO_SOURCE_MAPS == 0
            && self.should_emit_source_maps(context_node)
            && !self.options.omit_brace_source_map_positions
            && (token == SyntaxKind::OpenBraceToken || token == SyntaxKind::CloseBraceToken)
    }

    pub fn should_emit_leading_comments(&self, node: &Node) -> bool {
        self.emit_context.k06_emit_flags(node) & (EF_NO_LEADING_COMMENTS as u32) == 0
    }

    pub fn should_emit_trailing_comments(&self, node: &Node) -> bool {
        self.emit_context.k06_emit_flags(node) & (EF_NO_TRAILING_COMMENTS as u32) == 0
    }

    pub fn should_emit_nested_comments(&self, node: &Node) -> bool {
        self.emit_context.k06_emit_flags(node) & (EF_NO_NESTED_COMMENTS as u32) == 0
    }

    pub fn should_emit_detached_comments(&self, node: &Node) -> bool {
        if node.kind != SyntaxKind::SourceFile {
            return true;
        }

        let statements = m3c::statements(node);
        if statements.is_empty() {
            return true;
        }
        let first = &statements[0];
        !is_prologue_directive(first) || node_is_synthesized(first)
    }

    pub fn has_comments_at_position(&self, pos: i32) -> bool {
        let Some(source_file) = self.current_source_file.as_deref() else {
            return false;
        };
        if get_trailing_comment_ranges(source_file.text(), (pos + 1) as usize)
            .into_iter()
            .next()
            .is_some()
        {
            return true;
        }
        get_leading_comment_ranges(source_file.text(), (pos + 1) as usize)
            .into_iter()
            .next()
            .is_some()
    }

    pub fn should_emit_indirect_call(&self, node: &Node) -> bool {
        self.emit_context.k06_emit_flags(node) & (EF_INDIRECT_CALL as u32) != 0
    }
}
