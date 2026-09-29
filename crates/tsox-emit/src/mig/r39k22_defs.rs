#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_core::core::tristate::Tristate;
use tsox_core::stringutil::split_lines;
use tsox_core::stringutil::mig::m3m_2::guess_indentation;
use tsox_frontend::ast::node::{Node, NodeList, SourceFile};
use tsox_frontend::ast::node_data_generated::{
    is_partially_emitted_expression, NodeData, PropertyAccessExpressionData,
};
use tsox_frontend::ast::node_source_file::ScriptKind;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{
    is_member_name, is_optional_chain, is_prologue_directive, node_is_synthesized,
    position_is_synthesized,
};
use tsox_frontend::ast::mig::m3e_3::{OperatorPrecedence, OPERATOR_PRECEDENCE_DISALLOW_COMMA};
use tsox_frontend::ast::mig::m3g_3::{is_var_await_using, is_var_const, is_var_let, is_var_using};
use tsox_frontend::ast::skip_partially_emitted_expressions_arc as skip_partially_emitted_expressions;
use tsox_frontend::format::mig::m4o::{CommentSeparator, EmitFlags, ListFormat};
use tsox_frontend::format::mig::m4o_2::WriteKind;
use tsox_frontend::format::mig::m4t_2::{
    GetLiteralTextFlags, GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE, GET_LITERAL_TEXT_FLAGS_NONE,
};
use tsox_frontend::format::mig::m4t_3::{
    get_lines_between_positions, positions_are_on_same_line, range_end_is_on_same_line_as_range_start,
    range_is_on_single_line,
};
use tsox_frontend::format::mig::m4t_6::get_literal_text;
use tsox_frontend::scanner::mig::m3i::get_source_text_of_node_from_source_file;
use tsox_frontend::scanner::{
    get_leading_comment_ranges, get_trailing_comment_ranges, skip_trivia, CommentRange,
    CommentRangeKind, TOKEN_FLAGS_WITH_SPECIFIER,
};

use crate::mig::m4r::EmitTextWriter;
use crate::mig::m4r_4::format_synthesized_comment;
use crate::mig::m4p::{DetachedCommentsInfo, Printer};
use crate::printer::mig::m4m_2::SynthesizedComment;
use crate::printer::mig::w7pr::is_jsdoc_like_text;

pub const LF_HERITAGE_CLAUSE_TYPES: ListFormat = ListFormat(
    ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0,
);
const LF_ARRAY_LITERAL_ELEMENTS: ListFormat = ListFormat(
    ListFormat::PRESERVE_LINES.0
        | ListFormat::COMMA_DELIMITED.0
        | ListFormat::SPACE_BETWEEN_SIBLINGS.0
        | ListFormat::ALLOW_TRAILING_COMMA.0
        | ListFormat::INDENTED.0
        | ListFormat::SQUARE_BRACKETS.0,
);
const LF_OBJECT_LITERAL_PROPERTIES: ListFormat = ListFormat(
    ListFormat::PRESERVE_LINES.0
        | ListFormat::COMMA_DELIMITED.0
        | ListFormat::SPACE_BETWEEN_SIBLINGS.0
        | ListFormat::SPACE_BETWEEN_BRACES.0
        | ListFormat::INDENTED.0
        | ListFormat::BRACES.0
        | ListFormat::NO_SPACE_IF_EMPTY.0,
);
const LF_TEMPLATE_EXPRESSION_SPANS: ListFormat = ListFormat(ListFormat::NONE.0);
const LF_VARIABLE_DECLARATION_LIST: ListFormat = ListFormat(
    ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0,
);
const LF_BINDING_ELEMENTS: ListFormat =
    ListFormat(ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0);
const LF_CALL_EXPRESSION_ARGUMENTS: ListFormat = ListFormat(
    ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::PARENTHESIS.0,
);

pub trait R39k22NodeExt {
    fn heritage_token22(&self) -> SyntaxKind;
    fn heritage_types22(&self) -> &Arc<NodeList>;
}

impl R39k22NodeExt for Node {
    fn heritage_token22(&self) -> SyntaxKind {
        match &self.data {
            NodeData::HeritageClause(d) => d.token,
            _ => panic!("HeritageClause expected, got {:?}", self.kind),
        }
    }

    fn heritage_types22(&self) -> &Arc<NodeList> {
        match &self.data {
            NodeData::HeritageClause(d) => &d.types,
            _ => panic!("HeritageClause expected, got {:?}", self.kind),
        }
    }
}

impl Printer {
    pub(crate) fn write(&mut self, text: &str) {
        self.writer.write(text);
    }

    pub(crate) fn write_comment(&mut self, text: &str) {
        self.writer.write_comment(text);
    }

    pub(crate) fn write_literal(&mut self, text: &str) {
        self.writer.write_literal(text);
    }

    pub(crate) fn write_symbol(
        &mut self,
        text: &str,
        symbol: Option<&Arc<tsox_frontend::ast::Symbol>>,
    ) {
        match symbol {
            Some(symbol) => self.writer.write_symbol(text, symbol),
            None => self.write(text),
        }
    }

    pub(crate) fn write_lines(&mut self, text: &str) {
        let lines = split_lines(text);
        let indentation = guess_indentation(&lines);
        for line in &lines {
            let line: &str = if indentation > 0 && line.len() >= indentation {
                &line[indentation..]
            } else {
                line.as_str()
            };
            if !line.is_empty() {
                self.write_line();
                self.write(line);
            }
        }
    }

    pub(crate) fn write_line_repeat(&mut self, count: i64) {
        for _ in 0..count {
            self.write_line();
        }
    }

    pub(crate) fn write_lines_and_indent(
        &mut self,
        line_count: i64,
        write_space_if_not_indenting: bool,
    ) {
        if line_count > 0 {
            self.increase_indent();
            self.write_line_repeat(line_count);
        } else if write_space_if_not_indenting {
            self.write_space();
        }
    }

    pub(crate) fn write_line_or_space(
        &mut self,
        parent_node: &Arc<Node>,
        prev_child_node: &Arc<Node>,
        next_child_node: &Arc<Node>,
    ) {
        if self.should_emit_on_single_line(parent_node) {
            self.write_space();
        } else if self.options.preserve_source_newlines {
            let lines = self.get_lines_between_nodes(parent_node, prev_child_node, next_child_node);
            if lines > 0 {
                self.write_line_repeat(lines);
            } else {
                self.write_space();
            }
        } else {
            self.write_line();
        }
    }

    pub(crate) fn write_delimiter(&mut self, format: ListFormat) {
        match format & ListFormat::DELIMITERS_MASK {
            f if f.0 == ListFormat::NONE.0 => {}
            f if f.intersects(ListFormat::COMMA_DELIMITED) => self.write_punctuation(","),
            f if f.intersects(ListFormat::BAR_DELIMITED) => {
                self.write_space();
                self.write_punctuation("|");
            }
            f if f.intersects(ListFormat::ASTERISK_DELIMITED) => {
                self.write_space();
                self.write_punctuation("*");
                self.write_space();
            }
            f if f.intersects(ListFormat::AMPERSAND_DELIMITED) => {
                self.write_space();
                self.write_punctuation("&");
            }
            _ => {}
        }
    }

    pub(crate) fn get_lines_between_nodes(
        &self,
        parent: &Arc<Node>,
        node1: &Arc<Node>,
        node2: &Arc<Node>,
    ) -> i64 {
        if self.should_elide_indentation22(parent) {
            return 0;
        }
        if self.current_source_file.is_some()
            && !node_is_synthesized(parent)
            && !node_is_synthesized(node1)
            && !node_is_synthesized(node2)
        {
            if self.should_emit_on_new_line(node2, ListFormat::NONE) {
                return 1;
            }
            let source_file = self.current_source_file.as_deref().unwrap();
            if range_end_is_on_same_line_as_range_start(node1.loc, node2.loc, source_file) {
                0
            } else {
                1
            }
        } else {
            0
        }
    }

    pub(crate) fn should_elide_indentation22(&self, node: &Arc<Node>) -> bool {
        self.emit_context
            .emit_flags(node)
            .intersects(EmitFlags::NO_INDENTATION)
    }

    pub(crate) fn should_emit_on_multiple_lines(&self, node: &Arc<Node>) -> bool {
        self.emit_context
            .emit_flags(node)
            .intersects(EmitFlags::MULTI_LINE)
    }

    pub(crate) fn should_emit_leading_comments(&self, node: &Arc<Node>) -> bool {
        !self
            .emit_context
            .emit_flags(node)
            .intersects(EmitFlags::NO_LEADING_COMMENTS)
    }

    pub(crate) fn should_emit_trailing_comments(&self, node: &Arc<Node>) -> bool {
        !self
            .emit_context
            .emit_flags(node)
            .intersects(EmitFlags::NO_TRAILING_COMMENTS)
    }

    pub(crate) fn should_write_comment(&self, comment: &CommentRange) -> bool {
        if !self.options.only_print_jsdoc_style {
            return true;
        }
        let Some(source_file) = self.current_source_file.as_ref() else {
            return false;
        };
        is_jsdoc_like_text(&source_file.text, comment)
            || source_file.text[comment.pos..comment.end].starts_with("/*!")
    }

    pub(crate) fn is_triple_slash_comment(&self, comment: &CommentRange) -> bool {
        let Some(source_file) = self.current_source_file.as_ref() else {
            return false;
        };
        let text = &source_file.text[comment.pos..comment.end];
        comment.kind == CommentRangeKind::SingleLine
            && text.starts_with("///")
            && !text.starts_with("////")
    }

    pub(crate) fn should_emit_comment_if_triple_slash(
        &self,
        comment: &CommentRange,
        triple_slash: Tristate,
    ) -> bool {
        match triple_slash {
            Tristate::True => self.is_triple_slash_comment(comment),
            Tristate::False => !self.is_triple_slash_comment(comment),
            Tristate::Unknown => true,
        }
    }

    pub(crate) fn has_comments_at_position(&mut self, pos: usize) -> bool {
        let Some(source_file) = self.current_source_file.as_ref() else {
            return false;
        };
        let text: &str = &source_file.text;
        !get_trailing_comment_ranges(text, pos + 1).is_empty()
            || !get_leading_comment_ranges(text, pos + 1).is_empty()
    }

    pub(crate) fn should_emit_new_line_before_leading_comment_of_position(
        &self,
        pos: usize,
        comment_pos: usize,
    ) -> bool {
        let Some(source_file) = self.current_source_file.as_ref() else {
            return false;
        };
        pos != comment_pos
            && !positions_are_on_same_line(pos as i64, comment_pos as i64, source_file)
    }

    pub(crate) fn emit_comment(&mut self, comment: &CommentRange) {
        let Some(source_file) = self.current_source_file.clone() else {
            return;
        };
        let text = source_file.text[comment.pos..comment.end].to_string();
        self.write_comment(&text);
    }

    pub(crate) fn emit_comments(
        &mut self,
        comments: &[CommentRange],
        separator: CommentSeparator,
    ) -> bool {
        if comments.is_empty() {
            return false;
        }
        if separator == CommentSeparator::Before {
            self.write_space();
        }
        let mut intervening_separator = false;
        for comment in comments {
            if intervening_separator {
                self.write_space();
                intervening_separator = false;
            }
            self.emit_comment(comment);
            if comment.kind == CommentRangeKind::SingleLine
                || (comment.has_trailing_new_line && separator != CommentSeparator::None)
            {
                self.write_line();
            } else {
                intervening_separator = separator != CommentSeparator::None;
            }
        }
        if intervening_separator && separator == CommentSeparator::After {
            self.write_space();
        }
        true
    }

    pub(crate) fn emit_trailing_comments_of_position(
        &mut self,
        pos: usize,
        prefix_space: bool,
        force_no_newline: bool,
    ) {
        if self.comments_disabled || self.current_source_file.is_none() {
            return;
        }
        let mut comments = Vec::new();
        if let Some(source_file) = self.current_source_file.as_ref() {
            let text: &str = &source_file.text;
            for comment in get_trailing_comment_ranges(text, pos + 1) {
                if self.should_write_comment(&comment) {
                    comments.push(comment);
                }
            }
        }
        if comments.is_empty() {
            return;
        }
        if prefix_space {
            self.write_space();
        }
        let _ = force_no_newline;
        self.emit_comments(&comments, CommentSeparator::None);
    }

    pub(crate) fn should_emit_detached_comments(&self, node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::SourceFile {
            return true;
        }
        let statements = match &node.data {
            NodeData::SourceFile(d) => &d.statements,
            _ => return true,
        };
        statements.nodes.is_empty()
            || !is_prologue_directive(&statements.nodes[0])
            || node_is_synthesized(&statements.nodes[0])
    }

    pub(crate) fn emit_detached_comments(
        &mut self,
        text_range: TextRange,
    ) -> Option<DetachedCommentsInfo> {
        let source_file = self.current_source_file.clone()?;
        let text = source_file.text.clone();

        let mut leading_comments: Vec<CommentRange> = Vec::new();
        if self.comments_disabled {
            if text_range.pos() == 0 {
                for comment in get_leading_comment_ranges(&text, text_range.pos()) {
                    if text[comment.pos..comment.end].starts_with("/*!") {
                        leading_comments.push(comment);
                    }
                }
            }
        } else {
            leading_comments = get_leading_comment_ranges(&text, text_range.pos());
        }

        if leading_comments.is_empty() {
            return None;
        }
        let mut detached_comments: Vec<CommentRange> = Vec::new();
        let mut last_comment: Option<CommentRange> = None;
        for comment in &leading_comments {
            if let Some(last) = last_comment {
                let comment_line = get_lines_between_positions(&source_file, last.end as i64, comment.pos as i64);
                if comment_line >= 2 {
                    break;
                }
            }
            detached_comments.push(comment.clone());
            last_comment = Some(comment.clone());
        }
        if detached_comments.is_empty() {
            return None;
        }
        let last = detached_comments.last().unwrap();
        let node_pos = skip_trivia(&text, text_range.pos());
        let node_line = get_lines_between_positions(&source_file, last.end as i64, node_pos as i64);
        if node_line >= 2 {
            let comments_to_emit: Vec<CommentRange> = detached_comments
                .iter()
                .filter(|c| self.should_write_comment(c))
                .cloned()
                .collect();
            if !comments_to_emit.is_empty() {
                if self.should_emit_new_line_before_leading_comment_of_position(
                    text_range.pos(),
                    comments_to_emit[0].pos,
                ) {
                    self.write_line();
                }
                self.emit_comments(&comments_to_emit, CommentSeparator::After);
            }
            return Some(DetachedCommentsInfo {
                node_pos: text_range.pos(),
                detached_comment_end_pos: last.end,
            });
        }
        None
    }

    pub(crate) fn write_synthesized_comment(&mut self, comment: &SynthesizedComment) {
        let text = format_synthesized_comment(comment);
        self.write_comment(&text);
    }

    pub(crate) fn has_trailing_comma(
        &self,
        _parent_node: &Arc<Node>,
        children: &Arc<NodeList>,
    ) -> bool {
        if !children.has_trailing_comma() {
            return false;
        }
        if self.current_source_file.is_none() {
            return false;
        }
        self.current_source_file.as_deref().unwrap().script_kind != ScriptKind::Json
    }

    pub(crate) fn should_allow_trailing_comma(
        &self,
        node: &Arc<Node>,
        list: &Arc<NodeList>,
    ) -> bool {
        if self.current_source_file.is_none()
            || self.current_source_file.as_deref().unwrap().script_kind == ScriptKind::Json
        {
            return false;
        }
        should_allow_trailing_comma_worker22(node, list)
    }

    pub(crate) fn get_separating_line_terminator_count(
        &mut self,
        previous_node: &Arc<Node>,
        next_node: &Arc<Node>,
        format: ListFormat,
    ) -> usize {
        if format.intersects(ListFormat::PRESERVE_LINES) || self.options.preserve_source_newlines {
            if format.intersects(ListFormat::PRESERVE_LINES)
                && self.current_source_file.is_some()
                && !position_is_synthesized(previous_node.end())
                && !position_is_synthesized(next_node.pos())
            {
                let source_file = self.current_source_file.as_ref().unwrap();
                return usize::from(!range_end_is_on_same_line_as_range_start(
                    previous_node.loc,
                    next_node.loc,
                    source_file,
                ));
            }
            if !format.intersects(ListFormat::PRESERVE_LINES)
                && self.get_lines_between_nodes(previous_node, previous_node, next_node) > 0
            {
                return 1;
            }
            if self.should_emit_on_new_line(next_node, format) {
                return 1;
            }
        }
        0
    }

    pub(crate) fn get_closing_line_terminator_count(
        &mut self,
        parent_node: &Arc<Node>,
        last_child: Option<&Arc<Node>>,
        format: ListFormat,
        children_text_range: TextRange,
    ) -> usize {
        if format.intersects(ListFormat::PRESERVE_LINES) || self.options.preserve_source_newlines {
            if format.intersects(ListFormat::PRESERVE_LINES)
                && self.current_source_file.is_some()
                && children_text_range.end() != 0
                && !position_is_synthesized(children_text_range.end())
            {
                if format.intersects(ListFormat::MULTI_LINE) {
                    return 0;
                }
                let source_file = self.current_source_file.as_ref().unwrap();
                return usize::from(!range_is_on_single_line(children_text_range, source_file));
            }
            if let Some(last_child) = last_child {
                if !position_is_synthesized(last_child.end())
                    && self.should_emit_on_new_line(last_child, format)
                {
                    return 1;
                }
            } else if self.current_source_file.is_some() {
                let source_file = self.current_source_file.as_ref().unwrap();
                return usize::from(!range_is_on_single_line(parent_node.loc, source_file));
            }
        }
        0
    }

    pub(crate) fn write_line_separators_and_indent_before(
        &mut self,
        node: &Arc<Node>,
        parent: &Arc<Node>,
    ) -> i64 {
        if self.options.preserve_source_newlines {
            let leading_newlines =
                self.get_leading_line_terminator_count(parent, Some(node), ListFormat::NONE);
            if leading_newlines > 0 {
                self.write_lines_and_indent(leading_newlines as i64, false);
                return leading_newlines as i64;
            }
        }
        0
    }

    pub(crate) fn write_line_separators_after(&mut self, node: &Arc<Node>, parent: &Arc<Node>) {
        if self.options.preserve_source_newlines {
            let trailing_newlines = self.get_closing_line_terminator_count(
                parent,
                Some(node),
                ListFormat::NONE,
                TextRange::new(usize::MAX, 0),
            );
            if trailing_newlines > 0 {
                self.write_line_repeat(trailing_newlines as i64);
            }
        }
    }

    pub(crate) fn get_literal_text_of_node(
        &mut self,
        node: &Arc<Node>,
        _source_file: Option<&Arc<Node>>,
        mut flags: GetLiteralTextFlags,
    ) -> String {
        if self
            .emit_context
            .emit_flags(node)
            .intersects(EmitFlags::NO_ASCII_ESCAPING)
        {
            flags |= GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE;
        }
        get_literal_text(node, self.current_source_file.as_ref(), flags)
    }

    pub(crate) fn get_text_of_node(&mut self, node: &Arc<Node>, include_trivia: bool) -> String {
        let can_use_source_file =
            self.current_source_file.is_some() && !node_is_synthesized(node);
        match node.kind {
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier | SyntaxKind::JsxNamespacedName => {
                if !can_use_source_file {
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

    pub(crate) fn generate_name(&mut self, _name: &Arc<Node>) {}

    pub(crate) fn emit_type_reference(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(d) => (&d.type_name, d.type_arguments.as_ref()),
            _ => panic!("unexpected TypeReference: {:?}", node.kind),
        };
        self.emit_entity_name(type_name);
        self.emit_type_arguments(node, type_arguments);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_property_name(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(node),
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                self.emit_no_substitution_template_literal(node)
            }
            SyntaxKind::NumericLiteral => self.emit_numeric_literal(node),
            SyntaxKind::BigIntLiteral => self.emit_big_int_literal(node),
            SyntaxKind::ComputedPropertyName => {
                self.write_punctuation("[");
                let expression = match &node.data {
                    NodeData::ComputedPropertyName(d) => d.expression.clone(),
                    _ => unreachable!(),
                };
                self.emit_expression(&expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
                self.write_punctuation("]");
            }
            _ => panic!("unexpected PropertyName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_array_literal_expression_element(&mut self, node: &Arc<Node>) {
        self.emit_expression(node, OperatorPrecedence::Spread);
    }

    pub(crate) fn emit_array_literal_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (elements, multi_line) = match &node.data {
            NodeData::ArrayLiteralExpression(d) => (d.elements.clone(), d.multi_line),
            _ => panic!("unexpected ArrayLiteralExpression: {:?}", node.kind),
        };
        let format = if multi_line {
            LF_ARRAY_LITERAL_ELEMENTS | ListFormat::PREFER_NEW_LINE
        } else {
            LF_ARRAY_LITERAL_ELEMENTS
        };
        self.emit_list(
            Self::emit_array_literal_expression_element,
            node,
            &elements,
            format,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_object_literal_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (properties, multi_line) = match &node.data {
            NodeData::ObjectLiteralExpression(d) => (d.properties.clone(), d.multi_line),
            _ => panic!("unexpected ObjectLiteralExpression: {:?}", node.kind),
        };
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.generate_all_member_names(&properties);
        let mut format = LF_OBJECT_LITERAL_PROPERTIES;
        if multi_line {
            format = format | ListFormat::PREFER_NEW_LINE;
        }
        if self.should_allow_trailing_comma(node, &properties) {
            format = format | ListFormat::ALLOW_TRAILING_COMMA;
        }
        self.emit_list(Self::emit_object_literal_element22, node, &properties, format);
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_object_literal_element22(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::PropertyAssignment => {
                let (name, initializer) = match &node.data {
                    NodeData::PropertyAssignment(d) => (d.name.clone(), d.initializer.clone()),
                    _ => unreachable!(),
                };
                self.emit_property_name(&name);
                self.write_punctuation(":");
                self.write_space();
                self.emit_expression(&initializer, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                let (name, object_assignment_initializer) = match &node.data {
                    NodeData::ShorthandPropertyAssignment(d) => {
                        (d.name.clone(), d.object_assignment_initializer.clone())
                    }
                    _ => unreachable!(),
                };
                self.emit_property_name(&name);
                if let Some(init) = object_assignment_initializer {
                    self.write_punctuation("=");
                    self.emit_expression(&init, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
                }
            }
            SyntaxKind::SpreadAssignment => {
                let expression = match &node.data {
                    NodeData::SpreadAssignment(d) => d.expression.clone(),
                    _ => unreachable!(),
                };
                self.write_punctuation("...");
                self.emit_expression(&expression, OperatorPrecedence::Spread);
            }
            _ => panic!("unexpected ObjectLiteralElement: {:?}", node.kind),
        }
    }

    pub(crate) fn may_need_dot_dot_for_property_access(&mut self, expression: &Arc<Node>) -> bool {
        let expression = skip_partially_emitted_expressions(expression);
        if let NodeData::NumericLiteral(d) = &expression.data {
            let text = self.get_literal_text_of_node(
                &expression,
                None,
                GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE,
            );
            return d.token_flags & TOKEN_FLAGS_WITH_SPECIFIER == 0
                && !text.contains('.')
                && !text.contains('E')
                && !text.contains('e');
        }
        false
    }

    pub(crate) fn emit_property_access_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let data = match &node.data {
            NodeData::PropertyAccessExpression(d) => PropertyAccessExpressionData {
                expression: d.expression.clone(),
                question_dot_token: d.question_dot_token.clone(),
                name: d.name.clone(),
            },
            _ => panic!("unexpected PropertyAccessExpression: {:?}", node.kind),
        };
        let precedence = if is_optional_chain(node) {
            OperatorPrecedence::OptionalChain
        } else {
            OperatorPrecedence::Member
        };
        self.emit_expression(&data.expression, precedence);
        let token = match &data.question_dot_token {
            Some(token) => Some(token.clone()),
            None => {
                let token = Arc::new(Node::with_loc(
                    SyntaxKind::DotToken,
                    NodeData::Token,
                    TextRange::new(data.expression.end(), data.name.pos()),
                ));
                self.emit_context.add_emit_flags(&token, EmitFlags::NO_SOURCE_MAP);
                Some(token)
            }
        };
        let token_ref = token.as_ref().unwrap();
        let lines_before_dot = self.get_lines_between_nodes(node, &data.expression, token_ref);
        self.write_line_repeat(lines_before_dot);
        self.increase_indent_if(lines_before_dot > 0);
        let should_emit_dot_dot = token_ref.kind != SyntaxKind::QuestionDotToken
            && self.may_need_dot_dot_for_property_access(&data.expression)
            && !self.writer.has_trailing_comment()
            && !self.writer.has_trailing_whitespace();
        if should_emit_dot_dot {
            self.write_punctuation(".");
        }
        if data.question_dot_token.is_some() {
            self.emit_token_node(token.as_ref());
        } else {
            self.emit_token(
                SyntaxKind::DotToken,
                data.expression.end(),
                WriteKind::Punctuation,
                node,
            );
        }
        let lines_after_dot = self.get_lines_between_nodes(node, token_ref, &data.name);
        self.write_line_repeat(lines_after_dot);
        self.increase_indent_if(lines_after_dot > 0);
        self.emit_member_name22(&data.name);
        self.decrease_indent_if(lines_after_dot > 0);
        self.decrease_indent_if(lines_before_dot > 0);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_member_name22(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(node),
            _ => panic!("unexpected MemberName: {:?}", node.kind),
        }
    }

    fn is_new_expression_without_arguments(node: &Arc<Node>) -> bool {
        node.kind == SyntaxKind::NewExpression
            && match &node.data {
                NodeData::NewExpression(d) => d
                    .arguments
                    .as_ref()
                    .map(|a| a.nodes.is_empty())
                    .unwrap_or(true),
                _ => false,
            }
    }

    pub(crate) fn emit_callee(&mut self, callee: &Arc<Node>, parent_node: &Arc<Node>) {
        if self
            .emit_context
            .emit_flags(parent_node)
            .intersects(EmitFlags::INDIRECT_CALL)
        {
            self.write_punctuation("(");
            self.write_literal("0");
            self.write_punctuation(",");
            self.write_space();
            self.emit_expression(callee, OperatorPrecedence::Comma);
            self.write_punctuation(")");
        } else if parent_node.kind == SyntaxKind::CallExpression
            && Self::is_new_expression_without_arguments(&skip_partially_emitted_expressions(callee))
        {
            self.emit_expression(callee, OperatorPrecedence::Parentheses);
        } else {
            let precedence = if is_optional_chain(parent_node) {
                OperatorPrecedence::OptionalChain
            } else {
                OperatorPrecedence::Member
            };
            self.emit_expression(callee, precedence);
        }
    }

    pub(crate) fn emit_call_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (expression, question_dot_token, type_arguments, arguments) = match &node.data {
            NodeData::CallExpression(d) => (
                d.expression.clone(),
                d.question_dot_token.clone(),
                d.type_arguments.clone(),
                d.arguments.clone(),
            ),
            _ => panic!("unexpected CallExpression: {:?}", node.kind),
        };
        self.emit_callee(&expression, node);
        self.emit_token_node(question_dot_token.as_ref());
        self.emit_type_arguments(node, type_arguments.as_ref());
        self.emit_list(Self::emit_argument, node, &arguments, LF_CALL_EXPRESSION_ARGUMENTS);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_new_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (expression, type_arguments, arguments) = match &node.data {
            NodeData::NewExpression(d) => (
                d.expression.clone(),
                d.type_arguments.clone(),
                d.arguments.clone(),
            ),
            _ => panic!("unexpected NewExpression: {:?}", node.kind),
        };
        self.emit_token(SyntaxKind::NewKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        if skip_partially_emitted_expressions(&expression).kind == SyntaxKind::CallExpression {
            self.emit_expression(&expression, OperatorPrecedence::Parentheses);
        } else {
            self.emit_expression(&expression, OperatorPrecedence::Member);
        }
        self.emit_type_arguments(node, type_arguments.as_ref());
        let arguments = arguments.unwrap_or_else(|| Arc::new(NodeList::new(Vec::new())));
        self.emit_list(
            Self::emit_argument,
            node,
            &arguments,
            ListFormat::COMMA_DELIMITED | ListFormat::SPACE_BETWEEN_SIBLINGS | ListFormat::PARENTHESIS,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_tagged_template_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (tag, type_arguments, template) = match &node.data {
            NodeData::TaggedTemplateExpression(d) => {
                (d.tag.clone(), d.type_arguments.clone(), d.template.clone())
            }
            _ => panic!("unexpected TaggedTemplateExpression: {:?}", node.kind),
        };
        self.emit_callee(&tag, node);
        self.emit_type_arguments(node, type_arguments.as_ref());
        self.write_space();
        match template.kind {
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                self.emit_no_substitution_template_literal(&template)
            }
            SyntaxKind::TemplateExpression => self.emit_template_expression(&template),
            _ => panic!("unhandled TemplateLiteral: {:?}", template.kind),
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_assertion_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (type_node, expression) = match &node.data {
            NodeData::TypeAssertion(d) => (d.type_node.clone(), d.expression.clone()),
            _ => panic!("unexpected TypeAssertion: {:?}", node.kind),
        };
        self.write_punctuation("<");
        self.emit_type_node_outside_extends(&type_node);
        self.write_punctuation(">");
        self.emit_expression(&expression, OperatorPrecedence::Update);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_parenthesized_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let expression = match &node.data {
            NodeData::ParenthesizedExpression(d) => d.expression.clone(),
            _ => panic!("unexpected ParenthesizedExpression: {:?}", node.kind),
        };
        let open_paren_pos =
            self.emit_token(SyntaxKind::OpenParenToken, node.pos(), WriteKind::Punctuation, node);
        let indented = self.write_line_separators_and_indent_before(&expression, node);
        self.emit_expression(&expression, OperatorPrecedence::Comma);
        self.write_line_separators_after(&expression, node);
        self.decrease_indent_if(indented > 0);
        let close_paren_pos = if node_is_synthesized(&expression) {
            open_paren_pos
        } else {
            expression.end()
        };
        self.emit_token(
            SyntaxKind::CloseParenToken,
            close_paren_pos,
            WriteKind::Punctuation,
            node,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_conditional_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (condition, question_token, when_true, colon_token, when_false) = match &node.data {
            NodeData::ConditionalExpression(d) => (
                d.condition.clone(),
                d.question_token.clone(),
                d.when_true.clone(),
                d.colon_token.clone(),
                d.when_false.clone(),
            ),
            _ => panic!("unexpected ConditionalExpression: {:?}", node.kind),
        };
        let lines_before_question = self.get_lines_between_nodes(node, &condition, &question_token);
        let lines_after_question = self.get_lines_between_nodes(node, &question_token, &when_true);
        let lines_before_colon = self.get_lines_between_nodes(node, &when_true, &colon_token);
        let lines_after_colon = self.get_lines_between_nodes(node, &colon_token, &when_false);
        let is_coalesce = match &skip_partially_emitted_expressions(&condition).data {
            NodeData::BinaryExpression(d) => d.operator_token.kind == SyntaxKind::QuestionQuestionToken,
            _ => false,
        };
        if is_coalesce {
            self.emit_expression(&condition, OperatorPrecedence::LogicalOr);
        } else {
            self.emit_expression(&condition, OperatorPrecedence::LogicalOr);
        }
        self.write_lines_and_indent(lines_before_question, true);
        self.emit_punctuation_node(&question_token);
        self.write_lines_and_indent(lines_after_question, true);
        self.emit_expression(&when_true, OperatorPrecedence::Yield);
        self.decrease_indent_if(lines_after_question > 0);
        self.decrease_indent_if(lines_before_question > 0);
        self.write_lines_and_indent(lines_before_colon, true);
        self.emit_punctuation_node(&colon_token);
        self.write_lines_and_indent(lines_after_colon, true);
        self.emit_expression(&when_false, OperatorPrecedence::Yield);
        self.decrease_indent_if(lines_after_colon > 0);
        self.decrease_indent_if(lines_before_colon > 0);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_template_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (head, template_spans) = match &node.data {
            NodeData::TemplateExpression(d) => (d.head.clone(), d.template_spans.clone()),
            _ => panic!("unexpected TemplateExpression: {:?}", node.kind),
        };
        self.emit_template_head22(&head);
        self.emit_list(
            Self::emit_template_span_node22,
            node,
            &template_spans,
            LF_TEMPLATE_EXPRESSION_SPANS,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_template_head22(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_template_span_node22(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (expression, literal) = match &node.data {
            NodeData::TemplateSpan(d) => (d.expression.clone(), d.literal.clone()),
            _ => panic!("unexpected TemplateSpan: {:?}", node.kind),
        };
        self.emit_expression(&expression, OperatorPrecedence::Comma);
        match literal.kind {
            SyntaxKind::TemplateMiddle | SyntaxKind::TemplateTail => {
                let inner = self.enter_node(&literal);
                self.emit_literal(&literal, GET_LITERAL_TEXT_FLAGS_NONE);
                self.exit_node(&literal, inner);
            }
            _ => {}
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_partially_emitted_expression(&mut self, node: &Arc<Node>) {
        let mut stack: Vec<(Arc<Node>, crate::mig::m4p_3::PrinterState)> = Vec::new();
        let mut node = node.clone();
        loop {
            let state = self.enter_node(&node);
            let emit_flags = self.emit_context.emit_flags(&node);
            let expression = match &node.data {
                NodeData::PartiallyEmittedExpression(d) => d.expression.clone(),
                _ => panic!("unexpected PartiallyEmittedExpression: {:?}", node.kind),
            };
            if !emit_flags.intersects(EmitFlags::NO_LEADING_COMMENTS) && node.pos() != expression.pos()
            {
                self.emit_trailing_comments_of_position(expression.pos(), false, false);
            }
            stack.push((node.clone(), state));
            if !is_partially_emitted_expression(&expression) {
                break;
            }
            node = expression;
        }
        let expression = match &node.data {
            NodeData::PartiallyEmittedExpression(d) => d.expression.clone(),
            _ => unreachable!(),
        };
        self.emit_expression(&expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);

        while let Some((current, state)) = stack.pop() {
            let expression = match &current.data {
                NodeData::PartiallyEmittedExpression(d) => d.expression.clone(),
                _ => unreachable!(),
            };
            let emit_flags = self.emit_context.emit_flags(&current);
            if !emit_flags.intersects(EmitFlags::NO_TRAILING_COMMENTS)
                && current.end() != expression.end()
            {
                self.emit_leading_comments_of_position(expression.end());
            }
            self.exit_node(&current, state);
        }
    }

    pub(crate) fn emit_variable_declaration_list(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let declarations = match &node.data {
            NodeData::VariableDeclarationList(d) => d.declarations.clone(),
            _ => panic!("unexpected VariableDeclarationList: {:?}", node.kind),
        };
        if is_var_let(node) {
            self.write_keyword("let");
        } else if is_var_const(node) {
            self.write_keyword("const");
        } else if is_var_using(node) {
            self.write_keyword("using");
        } else if is_var_await_using(node) {
            self.write_keyword("await");
            self.write_space();
            self.write_keyword("using");
        } else {
            self.write_keyword("var");
        }
        self.write_space();
        self.emit_list(
            Self::emit_variable_declaration_node22,
            node,
            &declarations,
            LF_VARIABLE_DECLARATION_LIST,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_variable_declaration_node22(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (name, exclamation_token, type_node, initializer) = match &node.data {
            NodeData::VariableDeclaration(d) => (
                d.name.clone(),
                d.exclamation_token.clone(),
                d.type_node.clone(),
                d.initializer.clone(),
            ),
            _ => panic!("unexpected VariableDeclaration: {:?}", node.kind),
        };
        let name_end = name.end();
        self.emit_binding_name22(&name);
        if let Some(exclamation) = exclamation_token {
            self.emit_punctuation_node(&exclamation);
        }
        self.emit_type_annotation(type_node.as_ref());
        self.emit_initializer(initializer.as_ref(), name_end, node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_binding_name22(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::Identifier => {
                let state = self.enter_node(node);
                self.emit_identifier_text(node);
                self.exit_node(node, state);
            }
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
                let state = self.enter_node(node);
                let elements = match &node.data {
                    NodeData::BindingPattern(d) => d.elements.clone(),
                    _ => panic!("unexpected BindingPattern: {:?}", node.kind),
                };
                let is_object = node.kind == SyntaxKind::ObjectBindingPattern;
                self.write_punctuation(if is_object { "{" } else { "[" });
                self.emit_list(
                    Self::emit_binding_element_node22,
                    node,
                    &elements,
                    LF_BINDING_ELEMENTS,
                );
                self.write_punctuation(if is_object { "}" } else { "]" });
                self.exit_node(node, state);
            }
            _ => panic!("unexpected BindingName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_binding_element_node22(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let (dot_dot_dot_token, property_name, name, initializer) = match &node.data {
            NodeData::BindingElement(d) => (
                d.dot_dot_dot_token.clone(),
                d.property_name.clone(),
                d.name.clone(),
                d.initializer.clone(),
            ),
            _ => panic!("unexpected BindingElement: {:?}", node.kind),
        };
        self.emit_token_node(dot_dot_dot_token.as_ref());
        if let Some(property_name) = property_name {
            self.emit_property_name(&property_name);
            self.write_punctuation(":");
            self.write_space();
        }
        if let Some(name) = name {
            let name_end = name.end();
            self.emit_binding_name22(&name);
            self.emit_initializer(initializer.as_ref(), name_end, node);
        }
        self.exit_node(node, state);
    }
}

fn should_allow_trailing_comma_worker22(node: &Arc<Node>, list: &Arc<NodeList>) -> bool {
    use SyntaxKind as K;
    match node.kind {
        K::ObjectLiteralExpression => true,
        K::ArrayLiteralExpression
        | K::ArrowFunction
        | K::Constructor
        | K::GetAccessor
        | K::SetAccessor
        | K::TypeAliasDeclaration
        | K::JSTypeAliasDeclaration
        | K::FunctionType
        | K::ConstructorType
        | K::CallSignature
        | K::ConstructSignature
        | K::TaggedTemplateExpression
        | K::ObjectBindingPattern
        | K::ArrayBindingPattern
        | K::NamedImports
        | K::NamedExports
        | K::ImportAttributes => true,
        K::ClassExpression | K::ClassDeclaration | K::InterfaceDeclaration => node
            .type_parameters()
            .map(|t| Arc::ptr_eq(&t, list))
            .unwrap_or(false),
        K::FunctionDeclaration | K::FunctionExpression | K::MethodDeclaration => true,
        K::CallExpression => true,
        K::NewExpression => true,
        _ => false,
    }
}
