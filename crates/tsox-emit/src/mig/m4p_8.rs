#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::compiler_options_kinds::ResolutionMode;
use tsox_core::core::text::TextRange;
use tsox_core::core::tristate::Tristate;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::position_is_synthesized;
use tsox_frontend::format::mig::m4t_3::range_is_on_single_line;
use tsox_frontend::scanner;

use tsox_frontend::format::mig::m4o::{CommentSeparator, EmitFlags, ListFormat};
use tsox_frontend::format::mig::m4o_2::WriteKind;

use crate::mig::m4r_4::{get_closing_bracket, get_opening_bracket};
use crate::printer::EmitContext;
use crate::printer::mig::m4m_2::SynthesizedComment;

use super::m4p::{CommentState, EmitFn, FileReference, Printer};

impl Printer {
    pub fn emit_directive(&mut self, kind: &str, refs: &[FileReference]) { ::tsox_core::fntrace::enter("emit_directive"); 
        for r in refs {
            let mut resolution_mode = String::new();
            if r.resolution_mode != ResolutionMode::None {
                resolution_mode = format!(
                    "resolution-mode=\"{}\" ",
                    if r.resolution_mode == ResolutionMode::ESNext { "import" } else { "require" }
                );
            }
            let preserve = if r.preserve { "preserve=\"true\" " } else { "" };
            self.write_comment(&format!(
                "/// <reference {}=\"{}\" {}{}/>",
                kind, r.file_name, resolution_mode, preserve
            ));
            self.write_line();
        }
    }

    pub fn emit_list(
        &mut self,
        emit: EmitFn,
        parent_node: &Arc<Node>,
        children: &Arc<NodeList>,
        mut format: ListFormat,
    ) { ::tsox_core::fntrace::enter("emit_list"); 
        if self.should_emit_on_multiple_lines(parent_node) {
            format = format | ListFormat::PREFER_NEW_LINE | ListFormat::INDENTED;
        }
        self.emit_list_range(emit, parent_node, Some(children), format, -1, -1);
    }

    pub fn emit_list_range(
        &mut self,
        emit: EmitFn,
        parent_node: &Arc<Node>,
        children: Option<&Arc<NodeList>>,
        mut format: ListFormat,
        mut start: i64,
        mut count: i64,
    ) { ::tsox_core::fntrace::enter("emit_list_range"); 
        let is_nil = children.is_none();
        let mut length = 0;
        if !is_nil {
            length = children.unwrap().nodes.len() as i64;
        }
        if start < 0 {
            start = 0;
        }
        if count < 0 {
            count = length - start;
        }
        if is_nil && format.intersects(ListFormat::OPTIONAL_IF_NIL) {
            return;
        }
        let is_empty = is_nil || start >= length || count <= 0;
        if is_empty && format.intersects(ListFormat::OPTIONAL_IF_EMPTY) {
            if let Some(on_before) = &self.on_before_emit_node_list {
                on_before(children.unwrap());
            }
            if let Some(on_after) = &self.on_after_emit_node_list {
                on_after(children.unwrap());
            }
            return;
        }
        if format.intersects(ListFormat::BRACKETS_MASK) {
            self.write_punctuation(get_opening_bracket(format.0));
            if is_empty && !is_nil {
                self.emit_trailing_comments(children.unwrap().pos(), CommentSeparator::Before);
            }
        }
        if let Some(on_before) = &self.on_before_emit_node_list {
            on_before(children.unwrap());
        }
        if is_empty {
            if format.intersects(ListFormat::MULTI_LINE)
                && !(self.options.preserve_source_newlines
                    && self.current_source_file.is_some()
                    && range_is_on_single_line(
                        parent_node.loc,
                        self.current_source_file.as_ref().unwrap(),
                    ))
            {
                self.write_line();
            } else if format.intersects(ListFormat::SPACE_BETWEEN_BRACES)
                && !format.intersects(ListFormat::NO_SPACE_IF_EMPTY)
            {
                self.write_space();
            }
        } else {
            let end = (start + count).min(length) as usize;
            let s = start as usize;
            let slice = &children.unwrap().nodes[s..end];
            self.emit_list_items(
                emit,
                Some(parent_node),
                slice,
                format,
                self.has_trailing_comma(parent_node, children.unwrap()),
                children.unwrap().loc,
            );
        }
        if let Some(on_after) = &self.on_after_emit_node_list {
            on_after(children.unwrap());
        }
        if format.intersects(ListFormat::BRACKETS_MASK) {
            if is_empty && !is_nil {
                self.emit_leading_comments(children.unwrap().end(), false);
            }
            self.write_punctuation(get_closing_bracket(format.0));
        }
    }

    pub fn emit_detached_comments_and_update_comments_info(&mut self, text_range: TextRange) { ::tsox_core::fntrace::enter("emit_detached_comments_and_update_comments_info"); 
        if self.current_source_file.is_none() {
            return;
        }
        if let Some(info) = self.emit_detached_comments(text_range) {
            self.detached_comments_info.push_back(info);
        }
    }

    pub fn emit_detached_comments_before_statement_list(
        &mut self,
        node: &Arc<Node>,
        detached_range: TextRange,
    ) -> Option<CommentState> { ::tsox_core::fntrace::enter("emit_detached_comments_before_statement_list"); 
        if !self.should_emit_detached_comments(node) {
            return None;
        }
        let emit_flags = self.emit_context.emit_flags(node);
        let container_pos = self.container_pos;
        let container_end = self.container_end;
        let declaration_list_container_end = self.declaration_list_container_end;
        let skip_leading_comments = position_is_synthesized(detached_range.pos())
            || emit_flags.intersects(EmitFlags::NO_LEADING_COMMENTS);
        if !skip_leading_comments {
            self.emit_detached_comments_and_update_comments_info(detached_range);
        }
        if emit_flags.intersects(EmitFlags::NO_NESTED_COMMENTS) {
            self.comments_disabled = true;
        }
        Some(CommentState::new(
            emit_flags,
            detached_range,
            container_pos,
            container_end,
            declaration_list_container_end,
        ))
    }

    pub fn emit_leading_comments_of_node(
        &mut self,
        node: &Arc<Node>,
        emit_flags: EmitFlags,
        comment_range: TextRange,
    ) { ::tsox_core::fntrace::enter("emit_leading_comments_of_node"); 
        let pos = comment_range.pos();
        let end = comment_range.end();
        if (!position_is_synthesized(pos) || !position_is_synthesized(end)) && pos != end {
            let skip_leading_comments = position_is_synthesized(pos)
                || emit_flags.intersects(EmitFlags::NO_LEADING_COMMENTS)
                || node.kind == SyntaxKind::JsxText;
            let skip_trailing_comments = position_is_synthesized(end)
                || emit_flags.intersects(EmitFlags::NO_TRAILING_COMMENTS)
                || node.kind == SyntaxKind::JsxText;
            if !skip_leading_comments {
                self.emit_leading_comments(pos, node.kind == SyntaxKind::NotEmittedStatement);
            }
            if !skip_leading_comments || (pos >= 0 && emit_flags.intersects(EmitFlags::NO_LEADING_COMMENTS)) {
                self.container_pos = pos;
            }
            if !skip_trailing_comments || (end >= 0 && emit_flags.intersects(EmitFlags::NO_TRAILING_COMMENTS)) {
                self.container_end = end;
                if node.kind == SyntaxKind::VariableDeclarationList {
                    self.declaration_list_container_end = end;
                }
            }
        }
    }

    pub fn emit_leading_synthetic_comments_of_node(&mut self, node: &Arc<Node>, emit_flags: EmitFlags) { ::tsox_core::fntrace::enter("emit_leading_synthetic_comments_of_node"); 
        if emit_flags.intersects(EmitFlags::NO_LEADING_COMMENTS) {
            return;
        }
        let synth = self.emit_context.get_synthetic_leading_comments(node).to_vec();
        for c in synth {
            self.emit_leading_synthesized_comment(&c);
        }
    }

    pub fn emit_leading_synthesized_comment(&mut self, comment: &SynthesizedComment) { ::tsox_core::fntrace::enter("emit_leading_synthesized_comment"); 
        if comment.has_leading_new_line || comment.kind == SyntaxKind::SingleLineCommentTrivia {
            self.writer.write_line();
        }
        self.write_synthesized_comment(comment);
        if comment.has_trailing_new_line || comment.kind == SyntaxKind::SingleLineCommentTrivia {
            self.writer.write_line();
        } else {
            self.writer.write_space(" ");
        }
    }

    pub fn emit_leading_comments(&mut self, mut pos: usize, elided: bool) -> bool { ::tsox_core::fntrace::enter("emit_leading_comments"); 
        if self.comments_disabled
            || self.current_source_file.is_none()
            || position_is_synthesized(pos)
            || pos == self.container_pos
        {
            return false;
        }
        let mut triple_slash = Tristate::Unknown;
        if !elided {
            if pos == 0 && self.current_source_file.as_ref().unwrap().is_declaration_file {
                triple_slash = Tristate::False;
            }
        } else if pos == 0 {
            triple_slash = Tristate::True;
        } else {
            return false;
        }
        if !self.detached_comments_info.is_empty() {
            if let Some(info) = self.detached_comments_info.back() {
                if info.node_pos == pos {
                    pos = self.detached_comments_info.pop_back().unwrap().detached_comment_end_pos;
                }
            }
        }
        let mut comments = Vec::new();
        let source_file = self.current_source_file.as_ref().unwrap();
        for comment in
            scanner::get_leading_comment_ranges(&source_file.text, pos)
        {
            if self.should_write_comment(&comment)
                && self.should_emit_comment_if_triple_slash(&comment, triple_slash)
            {
                comments.push(comment);
            }
        }
        if !comments.is_empty()
            && self.should_emit_new_line_before_leading_comment_of_position(pos, comments[0].pos)
        {
            self.write_line();
        }
        self.emit_comments(&comments, CommentSeparator::After)
    }

    pub fn emit_leading_comments_of_position(&mut self, pos: usize) { ::tsox_core::fntrace::enter("emit_leading_comments_of_position"); 
        if self.comments_disabled || pos == usize::MAX {
            return;
        }
        self.emit_leading_comments(pos, false);
    }
}
