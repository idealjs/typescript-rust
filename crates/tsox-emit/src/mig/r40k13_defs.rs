#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::mig::m3e_3::{
    get_binary_operator_precedence, get_operator_precedence, OperatorPrecedence,
    OperatorPrecedenceFlags, OPERATOR_PRECEDENCE_DISALLOW_COMMA, OPERATOR_PRECEDENCE_HIGHEST,
    OPERATOR_PRECEDENCE_LOWEST,
};
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{
    is_not_emitted_statement, is_partially_emitted_expression, NodeData,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{is_optional_chain, node_is_synthesized, position_is_synthesized};
use tsox_frontend::format::mig::m4t_3::{
    get_lines_between_positions, get_lines_between_range_end_and_range_start,
    get_start_position_of_range, range_end_is_on_same_line_as_range_start, range_is_on_single_line,
};
use tsox_frontend::format::mig::m4t_4::mixing_binary_operators_requires_parentheses;
use tsox_frontend::scanner::{self, TOKEN_FLAGS_WITH_SPECIFIER};

use crate::mig::m4q::r33k12_defs::{
    CommentSeparator, EmitFlags, ListFlags, TokenEmitFlags, WriteKind, LF_ASTERISK_DELIMITED,
    LF_ALLOW_TRAILING_COMMA, LF_ANGLE_BRACKETS, LF_BRACES, LF_BRACKETS_MASK, LF_CLASS_HERITAGE_CLAUSES,
    LF_CLASS_MEMBERS, LF_COMMA_DELIMITED, LF_CASE_BLOCK_CLAUSES, LF_CASE_OR_DEFAULT_CLAUSE_STATEMENTS,
    LF_CALL_EXPRESSION_ARGUMENTS, LF_DELIMITERS_MASK, LF_HERITAGE_CLAUSE_TYPES, LF_INDENTED,
    LF_JSX_ELEMENT_ATTRIBUTES, LF_JSX_ELEMENT_OR_FRAGMENT_CHILDREN, LF_LINES_MASK, LF_MULTI_LINE,
    LF_MULTI_LINE_FUNCTION_BODY_STATEMENTS, LF_NO_INTERVENING_COMMENTS, LF_NO_SPACE_IF_EMPTY,
    LF_NO_TRAILING_NEW_LINE, LF_OPTIONAL_IF_EMPTY, LF_OPTIONAL_IF_NIL, LF_PARENTHESIS,
    LF_PREFER_NEW_LINE, LF_PRESERVE_LINES, LF_SINGLE_LINE, LF_SINGLE_LINE_FUNCTION_BODY_STATEMENTS,
    LF_SPACE_AFTER_LIST, LF_SPACE_BETWEEN_BRACES, LF_SPACE_BETWEEN_SIBLINGS,
    LF_ARRAY_LITERAL_EXPRESSION_ELEMENTS,
};
use crate::mig::m4q_2::r39k07_defs::{
    greatest_end07, is_binary_operation07, skip_partially_emitted_expressions07,
};
use crate::mig::m4i_3::r37k14_defs::R37K14DataExt;
use crate::mig::m4q_3::r36k15_defs::NodeDataExt15;
use crate::mig::m4q::r39k08_defs::{EmitContextExtK08, PrinterExtK08};
use crate::mig::m4q::Printer;
use crate::printer::GetLiteralTextFlags;

pub type EmitFn13 = fn(&mut Printer, &Node);

fn get_opening_bracket13(format: ListFlags) -> &'static str {
    if format & LF_BRACES != 0 {
        "{"
    } else if format & LF_PARENTHESIS != 0 {
        "("
    } else if format & LF_ANGLE_BRACKETS != 0 {
        "<"
    } else {
        "["
    }
}

fn get_closing_bracket13(format: ListFlags) -> &'static str {
    if format & LF_BRACES != 0 {
        "}"
    } else if format & LF_PARENTHESIS != 0 {
        ")"
    } else if format & LF_ANGLE_BRACKETS != 0 {
        ">"
    } else {
        "]"
    }
}

impl Printer {
    pub(crate) fn emit_list(
        &mut self,
        emit: EmitFn13,
        parent_node: &Node,
        children: &NodeList,
        mut format: ListFlags,
    ) {
        if self.should_emit_on_multiple_lines(parent_node) {
            format |= LF_PREFER_NEW_LINE | LF_INDENTED;
        }
        self.emit_list_range(emit, parent_node, Some(children), format, 0, usize::MAX);
    }

    pub(crate) fn emit_list_range(
        &mut self,
        emit: EmitFn13,
        parent_node: &Node,
        children: Option<&NodeList>,
        format: ListFlags,
        start: usize,
        count: usize,
    ) {
        let is_nil = children.is_none();
        let length = children.map_or(0, |c| c.nodes.len());
        let start = if start == usize::MAX { 0 } else { start };
        let count = if count == usize::MAX {
            length.saturating_sub(start)
        } else {
            count
        };
        if is_nil && format & LF_OPTIONAL_IF_NIL != 0 {
            return;
        }
        let is_empty = is_nil || start >= length || count == 0;
        if is_empty && format & LF_OPTIONAL_IF_EMPTY != 0 {
            if let Some(children) = children {
                if let Some(on_before) = self.on_before_emit_node_list.as_mut() {
                    on_before(children);
                }
                if let Some(on_after) = self.on_after_emit_node_list.as_mut() {
                    on_after(children);
                }
            }
            return;
        }
        if format & LF_BRACKETS_MASK != 0 {
            self.write_punctuation(get_opening_bracket13(format));
            if is_empty && !is_nil {
                self.emit_trailing_comments(children.unwrap().pos(), CommentSeparator::Before);
            }
        }
        if let Some(children) = children {
            if let Some(on_before) = self.on_before_emit_node_list.as_mut() {
                on_before(children);
            }
        }
        if is_empty {
            if format & LF_MULTI_LINE != 0
                && !(self.options.preserve_source_newlines
                    && self.current_source_file.is_some()
                    && range_is_on_single_line(
                        parent_node.loc,
                        self.current_source_file.as_ref().unwrap(),
                    ))
            {
                self.write_line();
            } else if format & LF_SPACE_BETWEEN_BRACES != 0 && format & LF_NO_SPACE_IF_EMPTY == 0 {
                self.write_space();
            }
        } else {
            let children = children.unwrap();
            let end = (start + count).min(length);
            let has_trailing_comma = children.has_trailing_comma();
            self.emit_list_items(
                emit,
                parent_node,
                &children.nodes[start..end],
                format,
                has_trailing_comma,
                children.loc,
            );
        }
        if let Some(children) = children {
            if let Some(on_after) = self.on_after_emit_node_list.as_mut() {
                on_after(children);
            }
        }
        if format & LF_BRACKETS_MASK != 0 {
            if is_empty && !is_nil {
                self.emit_leading_comments(children.unwrap().end(), false);
            }
            self.write_punctuation(get_closing_bracket13(format));
        }
    }

    pub(crate) fn emit_list_items(
        &mut self,
        emit: EmitFn13,
        parent_node: &Node,
        children: &[Arc<Node>],
        format: ListFlags,
        has_trailing_comma: bool,
        children_text_range: TextRange,
    ) {
        let may_emit_intervening_comments = format & LF_NO_INTERVENING_COMMENTS == 0;
        let mut should_emit_intervening_comments = may_emit_intervening_comments;

        let mut leading_line_terminator_count = 0;
        if let Some(first) = children.first() {
            leading_line_terminator_count =
                self.get_leading_line_terminator_count07(parent_node, Some(first), format);
        }
        if leading_line_terminator_count > 0 {
            self.write_line_repeat(leading_line_terminator_count);
            should_emit_intervening_comments = false;
        } else if format & LF_SPACE_BETWEEN_BRACES != 0 {
            self.write_space();
        }

        if format & LF_INDENTED != 0 {
            self.increase_indent();
        }

        let parent_end = greatest_end07(0, &[Some(parent_node)]);

        let mut previous_sibling: Option<&Node> = None;
        let mut should_decrease_indent_after_emit = false;
        for child in children {
            let child_arc: &Arc<Node> = child;
            let child: &Node = child;
            if format & LF_ASTERISK_DELIMITED != 0 {
                self.write_line();
                self.write_delimiter(format);
            } else if let Some(previous_sibling) = previous_sibling {
                if format & LF_DELIMITERS_MASK != 0 && previous_sibling.end() != parent_end {
                    if !self.comments_disabled
                        && self.should_emit_trailing_comments13(previous_sibling)
                    {
                        self.emit_leading_comments(previous_sibling.end(), false);
                    }
                }

                self.write_delimiter(format);

                let separating_line_terminator_count = self.get_separating_line_terminator_count13(
                    Some(previous_sibling),
                    Some(child),
                    format,
                );
                if separating_line_terminator_count > 0 {
                    if format & (LF_LINES_MASK | LF_INDENTED) == LF_SINGLE_LINE {
                        self.increase_indent();
                        should_decrease_indent_after_emit = true;
                    }

                    if should_emit_intervening_comments
                        && format & LF_DELIMITERS_MASK != 0
                        && !position_is_synthesized(child.pos())
                        && self.should_emit_leading_comments13(child)
                    {
                        let comment_range = self.emit_context.comment_range(child_arc);
                        self.emit_trailing_comments_of_position(
                            comment_range.pos(),
                            format & LF_SPACE_BETWEEN_SIBLINGS != 0,
                            true,
                        );
                    }

                    self.write_line_repeat(separating_line_terminator_count);

                    should_emit_intervening_comments = false;
                } else if format & LF_SPACE_BETWEEN_SIBLINGS != 0 {
                    self.write_space();
                }
            }

            if should_emit_intervening_comments && self.should_emit_leading_comments13(child) {
                let comment_range = self.emit_context.comment_range(child_arc);
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
            || !previous_sibling.is_some_and(|s| self.should_emit_trailing_comments13(s));
        let emit_trailing_comma = has_trailing_comma
            && format & LF_ALLOW_TRAILING_COMMA != 0
            && format & LF_COMMA_DELIMITED != 0;
        if emit_trailing_comma {
            match previous_sibling {
                Some(previous_sibling) if !skip_trailing_comments => {
                    self.emit_token(
                        SyntaxKind::CommaToken,
                        previous_sibling.end(),
                        WriteKind::Punctuation,
                        previous_sibling,
                    );
                }
                _ => self.write_punctuation(","),
            }
        }

        if let Some(previous_sibling) = previous_sibling {
            if parent_end != previous_sibling.end()
                && format & LF_DELIMITERS_MASK != 0
                && !skip_trailing_comments
            {
                let comments_pos = if emit_trailing_comma && children_text_range.end() > 0 {
                    children_text_range.end()
                } else {
                    previous_sibling.end()
                };
                self.emit_leading_comments(comments_pos, false);
            }
        }

        if format & LF_INDENTED != 0 {
            self.decrease_indent();
        }

        let closing_line_terminator_count =
            self.get_closing_line_terminator_count07(parent_node, previous_sibling, format);
        if closing_line_terminator_count > 0 {
            self.write_line_repeat(closing_line_terminator_count);
        } else if format & (LF_SPACE_AFTER_LIST | LF_SPACE_BETWEEN_BRACES) != 0 {
            self.write_space();
        }
    }

    fn write_delimiter(&mut self, format: ListFlags) {
        let masked = format & LF_DELIMITERS_MASK;
        if masked == 0 {
            return;
        }
        if masked & LF_COMMA_DELIMITED != 0 {
            self.write_punctuation(",");
        } else if masked & crate::mig::m4q::r33k12_defs::LF_BAR_DELIMITED != 0 {
            self.write_space();
            self.write_punctuation("|");
        } else if masked & LF_ASTERISK_DELIMITED != 0 {
            self.write_space();
            self.write_punctuation("*");
            self.write_space();
        } else {
            self.write_space();
            self.write_punctuation("&");
        }
    }

    fn should_emit_leading_comments13(&self, node: &Node) -> bool {
        !self.comments_disabled
            && !is_not_emitted_statement(node)
            && self.emit_context.emit_flags_of(node) & EmitFlags::NO_LEADING_COMMENTS.0 == 0
    }

    fn should_emit_trailing_comments13(&self, node: &Node) -> bool {
        !self.comments_disabled
            && !is_not_emitted_statement(node)
            && self.emit_context.emit_flags_of(node) & EmitFlags::NO_TRAILING_COMMENTS.0 == 0
    }

    fn get_separating_line_terminator_count13(
        &mut self,
        previous_node: Option<&Node>,
        next_node: Option<&Node>,
        format: ListFlags,
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
                if self.options.preserve_source_newlines {
                    return self.get_effective_lines13(|include_comments| {
                        get_lines_between_range_end_and_range_start(
                            previous_node.loc,
                            next_node.loc,
                            source_file,
                            include_comments,
                        ) as i32
                    });
                }
                return if range_end_is_on_same_line_as_range_start(
                    previous_node.loc,
                    next_node.loc,
                    source_file,
                ) {
                    0
                } else {
                    1
                };
            }
            if self.should_emit_on_new_line07(previous_node, format)
                || self.should_emit_on_new_line07(next_node, format)
            {
                return 1;
            }
            return if format & LF_PREFER_NEW_LINE != 0 { 1 } else { 0 };
        }
        if format & LF_MULTI_LINE != 0 && format & LF_NO_TRAILING_NEW_LINE == 0 {
            1
        } else {
            0
        }
    }

    fn get_effective_lines13(&self, get_line_difference: impl Fn(bool) -> i32) -> i32 {
        let lines = get_line_difference(true);
        if lines == 0 {
            return get_line_difference(false);
        }
        lines
    }

    pub(crate) fn emit_leading_comments_of_position(&mut self, pos: usize) {
        if self.comments_disabled || pos == usize::MAX {
            return;
        }
        self.emit_leading_comments(pos, false);
    }

    pub(crate) fn emit_leading_comments(&mut self, pos: usize, elided: bool) -> bool {
        if self.comments_disabled
            || self.current_source_file.is_none()
            || position_is_synthesized(pos)
            || pos == self.container_pos
        {
            return false;
        }
        if elided && pos != 0 {
            return false;
        }
        let source_file = self.current_source_file.as_ref().unwrap();
        let mut comments = Vec::new();
        for comment in scanner::get_leading_comment_ranges(&source_file.text, pos) {
            if self.should_write_comment(&comment) {
                comments.push(comment);
            }
        }
        if comments.is_empty() {
            return false;
        }
        self.emit_comments(&comments, CommentSeparator::After);
        true
    }

    pub(crate) fn get_lines_between_nodes(
        &mut self,
        parent: &Node,
        node1: &Node,
        node2: &Node,
    ) -> i32 {
        if self.should_elide_indentation(parent) {
            return 0;
        }

        let parent = skip_synthesized_parentheses13(parent);
        let node1 = skip_synthesized_parentheses13(node1);
        let node2 = skip_synthesized_parentheses13(node2);

        if self.should_emit_on_new_line07(node2, 0) {
            return 1;
        }

        if self.current_source_file.is_some()
            && !node_is_synthesized(parent)
            && !node_is_synthesized(node1)
            && !node_is_synthesized(node2)
        {
            let source_file = self.current_source_file.as_deref().unwrap();
            if self.options.preserve_source_newlines {
                return self.get_effective_lines13(|include_comments| {
                    get_lines_between_range_end_and_range_start(
                        node1.loc,
                        node2.loc,
                        source_file,
                        include_comments,
                    ) as i32
                });
            }
            return if range_end_is_on_same_line_as_range_start(node1.loc, node2.loc, source_file)
            {
                0
            } else {
                1
            };
        }

        0
    }

    pub(crate) fn write_lines_and_indent(
        &mut self,
        line_count: i32,
        write_space_if_not_indenting: bool,
    ) {
        if line_count > 0 {
            self.increase_indent();
            self.write_line_repeat(line_count);
        } else if write_space_if_not_indenting {
            self.write_space();
        }
    }

    pub(crate) fn may_need_dot_dot_for_property_access(&mut self, expression: &Node) -> bool {
        let expression = skip_partially_emitted_expressions07(expression);
        if expression.kind == SyntaxKind::NumericLiteral {
            let text = self.get_literal_text_of_node(
                expression,
                None,
                GetLiteralTextFlags::NEVER_ASCII_ESCAPE,
            );
            let has_specifier = match &expression.data {
                NodeData::NumericLiteral(d) => d.token_flags & TOKEN_FLAGS_WITH_SPECIFIER != 0,
                _ => false,
            };
            return !has_specifier
                && !text.contains('.')
                && !text.contains('E')
                && !text.contains('e');
        }
        false
    }

    pub(crate) fn parenthesize_expression_for_no_asi(&mut self, node: &Node) -> bool {
        if self.comments_disabled {
            return false;
        }
        self.will_emit_leading_new_line(node)
    }

    fn will_emit_leading_new_line(&mut self, node: &Node) -> bool {
        if self.current_source_file.is_none() {
            return false;
        }
        let mut has_leading_comment_ranges = false;
        let mut has_new_line_comment = false;
        {
            let source_file = self.current_source_file.as_ref().unwrap();
            for comment in scanner::get_leading_comment_ranges(&source_file.text, node.pos()) {
                has_leading_comment_ranges = true;
                if comment.has_trailing_new_line {
                    has_new_line_comment = true;
                }
            }
        }
        if has_leading_comment_ranges {
            return true;
        }
        if has_new_line_comment {
            return true;
        }
        if is_partially_emitted_expression(node) {
            let pee = node.as_partially_emitted_expression();
            if node.pos() != pee.expression.pos() {
                let source_file = self.current_source_file.as_ref().unwrap();
                for comment in
                    scanner::get_trailing_comment_ranges(&source_file.text, pee.expression.pos())
                {
                    if comment.has_trailing_new_line {
                        return true;
                    }
                }
            }
            return self.will_emit_leading_new_line(&pee.expression);
        }
        false
    }

    pub(crate) fn emit_expression_no_asi(&mut self, node: &Node, precedence: OperatorPrecedence) {
        if self.parenthesize_expression_for_no_asi(node) {
            self.write_punctuation("(");
            self.emit_expression(node, precedence);
            self.write_punctuation(")");
        } else {
            self.emit_expression(node, precedence);
        }
    }

    pub(crate) fn emit_expression(&mut self, node: &Node, precedence: OperatorPrecedence) {
        let parens =
            get_expression_precedence13(skip_partially_emitted_expressions07(node)) < precedence;
        if parens {
            self.write_punctuation("(");
        }

        match node.kind {
            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword | SyntaxKind::NullKeyword => {
                self.emit_token_node(Some(node))
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword | SyntaxKind::ImportKeyword => {
                self.emit_keyword_expression(node)
            }
            SyntaxKind::NumericLiteral => self.emit_numeric_literal(node),
            SyntaxKind::BigIntLiteral => self.emit_big_int_literal(node),
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            SyntaxKind::RegularExpressionLiteral => self.emit_regular_expression_literal(node),
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                self.emit_no_substitution_template_literal(node)
            }
            SyntaxKind::Identifier => self.emit_identifier_reference(node),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(node),
            SyntaxKind::ArrayLiteralExpression => self.emit_array_literal_expression(node),
            SyntaxKind::ObjectLiteralExpression => self.emit_object_literal_expression(node),
            SyntaxKind::PropertyAccessExpression => self.emit_property_access_expression(node),
            SyntaxKind::ElementAccessExpression => self.emit_element_access_expression(node),
            SyntaxKind::CallExpression => self.emit_call_expression(node),
            SyntaxKind::NewExpression => self.emit_new_expression(node),
            SyntaxKind::TaggedTemplateExpression => self.emit_tagged_template_expression(node),
            SyntaxKind::TypeAssertionExpression => self.emit_type_assertion_expression(node),
            SyntaxKind::ParenthesizedExpression => self.emit_parenthesized_expression(node),
            SyntaxKind::FunctionExpression => self.emit_function_expression(node),
            SyntaxKind::ArrowFunction => self.emit_arrow_function(node),
            SyntaxKind::DeleteExpression => self.emit_delete_expression(node),
            SyntaxKind::TypeOfExpression => self.emit_type_of_expression(node),
            SyntaxKind::VoidExpression => self.emit_void_expression(node),
            SyntaxKind::AwaitExpression => self.emit_await_expression(node),
            SyntaxKind::PrefixUnaryExpression => self.emit_prefix_unary_expression(node),
            SyntaxKind::PostfixUnaryExpression => self.emit_postfix_unary_expression(node),
            SyntaxKind::BinaryExpression => self.emit_binary_expression(node),
            SyntaxKind::ConditionalExpression => self.emit_conditional_expression(node),
            SyntaxKind::TemplateExpression => self.emit_template_expression(node),
            SyntaxKind::YieldExpression => self.emit_yield_expression(node),
            SyntaxKind::SpreadElement => self.emit_spread_element(node),
            SyntaxKind::ClassExpression => self.emit_class_expression(node),
            SyntaxKind::OmittedExpression => self.emit_omitted_expression(node),
            SyntaxKind::AsExpression => self.emit_as_expression(node),
            SyntaxKind::NonNullExpression => self.emit_non_null_expression(node),
            SyntaxKind::ExpressionWithTypeArguments => {
                self.emit_expression_with_type_arguments(node)
            }
            SyntaxKind::SatisfiesExpression => self.emit_satisfies_expression(node),
            SyntaxKind::MetaProperty => self.emit_meta_property(node),
            SyntaxKind::SyntheticExpression => panic!("SyntheticExpression should never be printed."),
            SyntaxKind::MissingDeclaration => {}
            SyntaxKind::JsxElement => self.emit_jsx_element(node),
            SyntaxKind::JsxSelfClosingElement => self.emit_jsx_self_closing_element(node),
            SyntaxKind::JsxFragment => self.emit_jsx_fragment(node),
            SyntaxKind::SyntaxList => panic!("SyntaxList should not be printed"),
            SyntaxKind::NotEmittedStatement => return,
            SyntaxKind::PartiallyEmittedExpression => self.emit_partially_emitted_expression(node),
            SyntaxKind::SyntheticReferenceExpression => {
                panic!("SyntheticReferenceExpression should not be printed")
            }
            _ => panic!("unexpected Expression: {:?}", node.kind),
        }

        if parens {
            self.write_punctuation(")");
        }
    }

    pub(crate) fn emit_keyword_expression(&mut self, node: &Node) {
        self.emit_keyword_node(node);
    }

    pub(crate) fn emit_array_literal_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (elements, multi_line) = match &node.data {
            NodeData::ArrayLiteralExpression(d) => (&d.elements, d.multi_line),
            _ => panic!("unexpected ArrayLiteralExpression: {:?}", node.kind),
        };
        let format =
            LF_ARRAY_LITERAL_EXPRESSION_ELEMENTS | if multi_line { LF_PREFER_NEW_LINE } else { 0 };
        self.emit_list(Self::emit_array_literal_expression_element, node, elements, format);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_array_literal_expression_element(&mut self, node: &Node) {
        self.emit_expression(node, OperatorPrecedence::Spread);
    }

    pub(crate) fn emit_element_access_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (expression, question_dot_token, argument_expression) = match &node.data {
            NodeData::ElementAccessExpression(d) => {
                (&d.expression, d.question_dot_token.as_deref(), &d.argument_expression)
            }
            _ => panic!("unexpected ElementAccessExpression: {:?}", node.kind),
        };
        self.emit_expression(
            expression,
            if is_optional_chain(node) {
                OperatorPrecedence::OptionalChain
            } else {
                OperatorPrecedence::Member
            },
        );
        self.emit_token_node(question_dot_token);
        self.emit_token(
            SyntaxKind::OpenBracketToken,
            greatest_end07(0, &[Some(expression.as_ref()), question_dot_token]),
            WriteKind::Punctuation,
            node,
        );
        self.emit_expression(argument_expression, OperatorPrecedence::Comma);
        self.emit_token(
            SyntaxKind::CloseBracketToken,
            argument_expression.end(),
            WriteKind::Punctuation,
            node,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_call_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (expression, type_arguments, arguments) = match &node.data {
            NodeData::CallExpression(d) => (&d.expression, d.type_arguments.as_deref(), &d.arguments),
            _ => panic!("unexpected CallExpression: {:?}", node.kind),
        };
        self.emit_callee(expression, node);
        self.emit_token_node(call_expression_question_dot_token13(node));
        self.emit_type_arguments(node, type_arguments);
        self.emit_list(Self::emit_argument, node, arguments, LF_CALL_EXPRESSION_ARGUMENTS);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_callee(&mut self, callee: &Node, parent_node: &Node) {
        if self.should_emit_indirect_call(parent_node) {
            self.write_punctuation("(");
            self.write_as("0", WriteKind::Literal);
            self.write_punctuation(",");
            self.write_space();
            self.emit_expression(callee, OperatorPrecedence::Comma);
            self.write_punctuation(")");
        } else if parent_node.kind == SyntaxKind::CallExpression
            && is_new_expression_without_arguments13(skip_partially_emitted_expressions07(callee))
        {
            self.emit_expression(callee, OperatorPrecedence::Parentheses);
        } else {
            self.emit_expression(
                callee,
                if is_optional_chain(parent_node) {
                    OperatorPrecedence::OptionalChain
                } else {
                    OperatorPrecedence::Member
                },
            );
        }
    }

    fn should_emit_indirect_call(&self, node: &Node) -> bool {
        self.emit_context.emit_flags_of(node) & EmitFlags::INDIRECT_CALL.0 != 0
    }

    pub(crate) fn emit_type_assertion_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (type_node, expression) = match &node.data {
            NodeData::TypeAssertion(d) => (&d.type_node, &d.expression),
            _ => panic!("unexpected TypeAssertion: {:?}", node.kind),
        };
        self.write_punctuation("<");
        self.emit_type_node_outside_extends(type_node);
        self.write_punctuation(">");
        self.emit_expression(expression, OperatorPrecedence::Update);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_function_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (name, modifiers, asterisk_token, type_parameters, body) = match &node.data {
            NodeData::FunctionExpression(d) => (
                d.name.as_ref(),
                d.modifiers.as_deref(),
                d.asterisk_token.as_deref(),
                d.type_parameters.as_deref(),
                d.body.as_ref(),
            ),
            _ => panic!("unexpected FunctionExpression: {:?}", node.kind),
        };
        self.generate_name_if_needed(name);
        self.emit_modifier_list(node, modifiers, false);
        self.write_keyword("function");
        self.emit_token_node(asterisk_token);
        self.write_space();
        if let Some(name) = name {
            self.emit_identifier_name(name);
        }
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.emit_function_body_node(Some(body));
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_arrow_function(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, type_parameters, parameters, type_node, equals_greater_than_token) =
            match &node.data {
                NodeData::ArrowFunction(d) => (
                    d.modifiers.as_deref(),
                    d.type_parameters.as_deref(),
                    &d.parameters,
                    d.type_node.as_deref(),
                    &d.equals_greater_than_token,
                ),
                _ => panic!("unexpected ArrowFunction: {:?}", node.kind),
            };
        self.emit_modifier_list(node, modifiers, false);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_type_parameters(node, type_parameters);
        self.emit_parameters_for_arrow(node, parameters);
        self.emit_type_annotation(type_node);
        self.write_space();
        self.emit_token_node(Some(equals_greater_than_token));
        self.write_space();
        self.emit_concise_body(Some(node.as_arrow_function().body.as_ref()));
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_concise_body(&mut self, body: Option<&Node>) {
        let Some(body) = body else { return };
        if body.kind == SyntaxKind::Block {
            self.emit_function_body(Some(body));
        } else {
            self.emit_expression(body, OperatorPrecedence::Yield);
        }
    }

    pub(crate) fn emit_delete_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = expression_of13(node);
        self.emit_token(SyntaxKind::DeleteKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(expression, OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_type_of_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = expression_of13(node);
        self.emit_token(SyntaxKind::TypeOfKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(expression, OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_void_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = expression_of13(node);
        self.emit_token(SyntaxKind::VoidKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(expression, OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_await_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = expression_of13(node);
        self.emit_token(SyntaxKind::AwaitKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(expression, OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_binary_expression(&mut self, node: &Node) {
        let (left, operator_token, right) = match &node.data {
            NodeData::BinaryExpression(d) => (&d.left, &d.operator_token, &d.right),
            _ => panic!("unexpected BinaryExpression: {:?}", node.kind),
        };
        let (mut left_prec, mut right_prec) = get_binary_expression_precedence13(node);
        let emitted_left = skip_partially_emitted_expressions07(left);
        if node_is_synthesized(emitted_left)
            && emitted_left.kind == SyntaxKind::BinaryExpression
            && mixing_binary_operators_requires_parentheses(
                operator_token.kind,
                binary_operator_token_kind13(emitted_left),
            )
        {
            left_prec = OPERATOR_PRECEDENCE_HIGHEST;
        }
        let emitted_right = skip_partially_emitted_expressions07(right);
        if node_is_synthesized(emitted_right)
            && emitted_right.kind == SyntaxKind::BinaryExpression
            && mixing_binary_operators_requires_parentheses(
                operator_token.kind,
                binary_operator_token_kind13(emitted_right),
            )
        {
            right_prec = OPERATOR_PRECEDENCE_HIGHEST;
        }
        let state = self.enter_node(node);
        self.emit_expression(left, left_prec);
        let lines_before_operator = self.get_lines_between_nodes(node, left, operator_token);
        let lines_after_operator = self.get_lines_between_nodes(node, operator_token, right);
        self.write_lines_and_indent(
            lines_before_operator,
            operator_token.kind != SyntaxKind::CommaToken,
        );
        self.emit_token_node_ex(Some(operator_token), TokenEmitFlags::NO_SOURCE_MAPS);
        self.write_lines_and_indent(lines_after_operator, true);
        self.emit_expression(right, right_prec);
        self.decrease_indent_if(lines_after_operator > 0);
        self.decrease_indent_if(lines_before_operator > 0);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_conditional_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (condition, question_token, when_true, colon_token, when_false) = match &node.data {
            NodeData::ConditionalExpression(d) => (
                &d.condition,
                &d.question_token,
                &d.when_true,
                &d.colon_token,
                &d.when_false,
            ),
            _ => panic!("unexpected ConditionalExpression: {:?}", node.kind),
        };
        let lines_before_question = self.get_lines_between_nodes(node, condition, question_token);
        let lines_after_question = self.get_lines_between_nodes(node, question_token, when_true);
        let lines_before_colon = self.get_lines_between_nodes(node, when_true, colon_token);
        let lines_after_colon = self.get_lines_between_nodes(node, colon_token, when_false);
        self.emit_short_circuit_expression(condition);
        self.write_lines_and_indent(lines_before_question, true);
        self.emit_punctuation_node(Some(question_token));
        self.write_lines_and_indent(lines_after_question, true);
        self.emit_expression(when_true, OperatorPrecedence::Yield);
        self.decrease_indent_if(lines_after_question > 0);
        self.decrease_indent_if(lines_before_question > 0);
        self.write_lines_and_indent(lines_before_colon, true);
        self.emit_punctuation_node(Some(colon_token));
        self.write_lines_and_indent(lines_after_colon, true);
        self.emit_expression(when_false, OperatorPrecedence::Yield);
        self.decrease_indent_if(lines_after_colon > 0);
        self.decrease_indent_if(lines_before_colon > 0);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_yield_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (asterisk_token, expression) = match &node.data {
            NodeData::YieldExpression(d) => (d.asterisk_token.as_deref(), d.expression.as_deref()),
            _ => panic!("unexpected YieldExpression: {:?}", node.kind),
        };
        self.emit_token(SyntaxKind::YieldKeyword, node.pos(), WriteKind::Keyword, node);
        self.emit_punctuation_node(asterisk_token);
        if let Some(expression) = expression {
            self.write_space();
            self.emit_expression_no_asi(expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_as_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (expression, type_node) = match &node.data {
            NodeData::AsExpression(d) => (&d.expression, &d.type_node),
            _ => panic!("unexpected AsExpression: {:?}", node.kind),
        };
        self.emit_expression(expression, OperatorPrecedence::Relational);
        self.write_space();
        self.write_keyword("as");
        self.write_space();
        self.emit_type_node_outside_extends(type_node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_class_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (name, modifiers, type_parameters, heritage_clauses, members) = match &node.data {
            NodeData::ClassExpression(d) => (
                d.name.as_ref(),
                d.modifiers.as_deref(),
                d.type_parameters.as_deref(),
                d.heritage_clauses.as_deref(),
                &d.members,
            ),
            _ => panic!("unexpected ClassExpression: {:?}", node.kind),
        };
        self.generate_name_if_needed(name);

        let pos = self.emit_modifier_list(node, modifiers, true);
        self.emit_token(SyntaxKind::ClassKeyword, pos, WriteKind::Keyword, node);

        if let Some(name) = name {
            self.write_space();
            self.emit_identifier_name(name);
        }

        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);

        self.emit_type_parameters(node, type_parameters);
        if let Some(heritage_clauses) = heritage_clauses {
            self.emit_list(
                Self::emit_heritage_clause,
                node,
                heritage_clauses,
                LF_CLASS_HERITAGE_CLAUSES,
            );
        }
        self.write_space();
        self.write_punctuation("{");
        self.push_name_generation_scope(node);
        self.generate_all_member_names(members);
        self.emit_list(Self::emit_class_element, node, members, LF_CLASS_MEMBERS);
        self.pop_name_generation_scope(node);
        self.write_punctuation("}");

        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_heritage_clause(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (token, types) = match &node.data {
            NodeData::HeritageClause(d) => (d.token, &d.types),
            _ => panic!("unexpected HeritageClause: {:?}", node.kind),
        };
        self.write_space();
        self.emit_token(token, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_list(
            Self::emit_heritage_clause_element,
            node,
            types,
            LF_HERITAGE_CLAUSE_TYPES,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_class_element(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::PropertyDeclaration => self.emit_property_declaration(node),
            SyntaxKind::MethodDeclaration => self.emit_method_declaration(node),
            SyntaxKind::GetAccessor => self.emit_get_accessor_declaration(node),
            SyntaxKind::SetAccessor => self.emit_set_accessor_declaration(node),
            SyntaxKind::IndexSignature => self.emit_index_signature(node),
            SyntaxKind::Constructor => self.emit_constructor(node),
            SyntaxKind::SemicolonClassElement => self.emit_semicolon_class_element(node),
            _ => panic!("unhandled ClassElement: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_constructor(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, type_parameters, parameters, type_node, body) = match &node.data {
            NodeData::ConstructorDeclaration(d) => (
                d.modifiers.as_deref(),
                d.type_parameters.as_deref(),
                &d.parameters,
                d.type_node.as_deref(),
                d.body.as_deref(),
            ),
            _ => panic!("unexpected ConstructorDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, true);
        self.emit_token(SyntaxKind::ConstructorKeyword, node.pos(), WriteKind::Keyword, node);
        self.emit_type_parameters(node, type_parameters);
        self.emit_parameters(node, parameters);
        self.emit_type_annotation(type_node);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_function_body_node(body);
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_index_signature(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, parameters, type_node) = match &node.data {
            NodeData::IndexSignatureDeclaration(d) => {
                (d.modifiers.as_deref(), &d.parameters, &d.type_node)
            }
            _ => panic!("unexpected IndexSignatureDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, false);
        self.emit_parameters_for_index_signature(node, parameters);
        self.emit_type_annotation(Some(type_node));
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_function_body_node(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            self.write_trailing_semicolon();
            return;
        };
        self.write_space();
        self.emit_function_body(Some(node));
    }

    pub(crate) fn emit_function_body(&mut self, body: Option<&Node>) {
        let Some(body) = body else { return };
        let statements = match &body.data {
            NodeData::Block(d) => &d.statements,
            _ => panic!("unexpected function body: {:?}", body.kind),
        };
        self.generate_names(Some(body));

        self.write_punctuation("{");

        self.increase_indent();
        let statement_offset = self.emit_prologue_directives(statements);
        if self.should_emit_block_function_body_on_single_line(body) && statement_offset == 0 {
            self.decrease_indent();
            self.emit_list_range(
                Self::emit_statement,
                body,
                Some(statements),
                LF_SINGLE_LINE_FUNCTION_BODY_STATEMENTS,
                statement_offset,
                usize::MAX,
            );
            self.increase_indent();
        } else {
            self.emit_list_range(
                Self::emit_statement,
                body,
                Some(statements),
                LF_MULTI_LINE_FUNCTION_BODY_STATEMENTS,
                statement_offset,
                usize::MAX,
            );
        }
        self.decrease_indent();

        self.emit_token_ex(
            SyntaxKind::CloseBraceToken,
            statements.end(),
            WriteKind::Punctuation,
            body,
            TokenEmitFlags::NONE,
        );
    }

    fn should_emit_block_function_body_on_single_line(&mut self, body: &Node) -> bool {
        if self.should_emit_on_single_line(body) {
            return true;
        }
        let (multi_line, statements) = match &body.data {
            NodeData::Block(d) => (d.multi_line, &d.statements),
            _ => panic!("unexpected Block: {:?}", body.kind),
        };
        if multi_line {
            return false;
        }
        if !node_is_synthesized(body)
            && self.current_source_file.is_some()
            && !range_is_on_single_line(body.loc, self.current_source_file.as_ref().unwrap())
        {
            return false;
        }
        if self.get_leading_line_terminator_count07(
            body,
            statements.nodes.first().map(|n| n.as_ref()),
            LF_PRESERVE_LINES,
        ) > 0
            || self.get_closing_line_terminator_count07(
                body,
                statements.nodes.last().map(|n| n.as_ref()),
                LF_PRESERVE_LINES,
            ) > 0
        {
            return false;
        }
        let mut previous_statement: Option<&Node> = None;
        for statement in &statements.nodes {
            if self.get_separating_line_terminator_count13(
                previous_statement,
                Some(statement),
                LF_PRESERVE_LINES,
            ) > 0
            {
                return false;
            }
            previous_statement = Some(statement);
        }
        true
    }

    pub(crate) fn emit_decorator(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = expression_of13(node);
        self.write_punctuation("@");
        self.emit_expression(expression, OperatorPrecedence::LeftHandSide);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_external_module_reference(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = expression_of13(node);
        self.write_keyword("require");
        self.write_punctuation("(");
        self.emit_expression(expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.write_punctuation(")");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_case_block(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let clauses = match &node.data {
            NodeData::CaseBlock(d) => &d.clauses,
            _ => panic!("unexpected CaseBlock: {:?}", node.kind),
        };
        self.emit_token(SyntaxKind::OpenBraceToken, node.pos(), WriteKind::Punctuation, node);
        self.emit_list(
            Self::emit_case_or_default_clause_node,
            node,
            clauses,
            LF_CASE_BLOCK_CLAUSES,
        );
        self.emit_token_ex(
            SyntaxKind::CloseBraceToken,
            clauses.end(),
            WriteKind::Punctuation,
            node,
            TokenEmitFlags::INDENT_LEADING_COMMENTS,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_case_or_default_clause_node(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::CaseClause => self.emit_case_clause(node),
            SyntaxKind::DefaultClause => self.emit_default_clause(node),
            _ => panic!("unhandled CaseOrDefaultClause: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_case_clause(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = match &node.data {
            NodeData::CaseOrDefaultClause(d) => &d.expression,
            _ => panic!("unexpected CaseOrDefaultClause: {:?}", node.kind),
        };
        self.emit_token(SyntaxKind::CaseKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_case_or_default_clause_statements(node, expression.end());
        self.exit_node(node, state);
    }

    pub(crate) fn emit_default_clause(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let pos = self.emit_token(SyntaxKind::DefaultKeyword, node.pos(), WriteKind::Keyword, node);
        self.emit_case_or_default_clause_statements(node, pos);
        self.exit_node(node, state);
    }

    fn emit_case_or_default_clause_statements(&mut self, node: &Node, colon_pos: usize) {
        let statements = match &node.data {
            NodeData::CaseOrDefaultClause(d) => &d.statements,
            _ => panic!("unexpected CaseOrDefaultClause: {:?}", node.kind),
        };
        let emit_as_single_statement = statements.nodes.len() == 1
            && (self.current_source_file.is_none()
                || node_is_synthesized(node)
                || node_is_synthesized(&statements.nodes[0])
                || range_start_positions_are_on_same_line13(
                    node.loc,
                    statements.nodes[0].loc,
                    self.current_source_file.as_ref().unwrap(),
                ));

        let mut format = LF_CASE_OR_DEFAULT_CLAUSE_STATEMENTS;
        if emit_as_single_statement {
            self.write_token_text(SyntaxKind::ColonToken, WriteKind::Punctuation, colon_pos);
            self.write_space();
            format &= !(LF_MULTI_LINE | LF_INDENTED);
        } else {
            self.emit_token(SyntaxKind::ColonToken, colon_pos, WriteKind::Punctuation, node);
        }

        self.emit_list(Self::emit_statement, node, statements, format);
    }

    pub(crate) fn emit_jsx_element(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (opening_element, children, closing_element) = match &node.data {
            NodeData::JsxElement(d) => (&d.opening_element, &d.children, &d.closing_element),
            _ => panic!("unexpected JsxElement: {:?}", node.kind),
        };
        self.emit_jsx_opening_element(opening_element);
        self.emit_list(
            Self::emit_jsx_child,
            node,
            children,
            LF_JSX_ELEMENT_OR_FRAGMENT_CHILDREN,
        );
        self.emit_jsx_closing_element(closing_element);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_self_closing_element(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (tag_name, type_arguments, attributes) = match &node.data {
            NodeData::JsxSelfClosingElement(d) => {
                (&d.tag_name, d.type_arguments.as_deref(), &d.attributes)
            }
            _ => panic!("unexpected JsxSelfClosingElement: {:?}", node.kind),
        };
        self.write_punctuation("<");
        self.emit_jsx_tag_name(tag_name);
        self.emit_type_arguments(node, type_arguments);
        self.write_space();
        self.emit_jsx_attributes(attributes);
        self.write_punctuation("/>");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_fragment(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (opening_fragment, children, closing_fragment) = match &node.data {
            NodeData::JsxFragment(d) => (&d.opening_fragment, &d.children, &d.closing_fragment),
            _ => panic!("unexpected JsxFragment: {:?}", node.kind),
        };
        self.emit_jsx_opening_fragment(opening_fragment);
        self.emit_list(
            Self::emit_jsx_child,
            node,
            children,
            LF_JSX_ELEMENT_OR_FRAGMENT_CHILDREN,
        );
        self.emit_jsx_closing_fragment(closing_fragment);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_opening_element(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (tag_name, type_arguments, attributes) = match &node.data {
            NodeData::JsxOpeningElement(d) => {
                (&d.tag_name, d.type_arguments.as_deref(), &d.attributes)
            }
            _ => panic!("unexpected JsxOpeningElement: {:?}", node.kind),
        };
        self.write_punctuation("<");
        let indented = self.write_line_separators_and_indent_before(Some(tag_name), node);
        self.emit_jsx_tag_name(tag_name);
        self.emit_type_arguments(node, type_arguments);
        if !jsx_attributes_is_empty13(attributes) {
            self.write_space();
        }
        self.emit_jsx_attributes(attributes);
        self.write_line_separators_after(Some(attributes), node);
        self.decrease_indent_if(indented);
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_closing_element(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let tag_name = match &node.data {
            NodeData::JsxClosingElement(d) => &d.tag_name,
            _ => panic!("unexpected JsxClosingElement: {:?}", node.kind),
        };
        self.write_punctuation("</");
        self.emit_jsx_tag_name(tag_name);
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_opening_fragment(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.write_punctuation("<");
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_closing_fragment(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.write_punctuation("</");
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_attributes(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let properties = match &node.data {
            NodeData::JsxAttributes(d) => &d.properties,
            _ => panic!("unexpected JsxAttributes: {:?}", node.kind),
        };
        self.emit_list(
            Self::emit_jsx_attribute_like,
            node,
            properties,
            LF_JSX_ELEMENT_ATTRIBUTES,
        );
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_attribute_like(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::JsxAttribute => self.emit_jsx_attribute(node),
            SyntaxKind::JsxSpreadAttribute => self.emit_jsx_spread_attribute(node),
            _ => panic!("unhandled JsxAttributeLike: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_jsx_attribute(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (name, initializer) = match &node.data {
            NodeData::JsxAttribute(d) => (&d.name, d.initializer.as_deref()),
            _ => panic!("unexpected JsxAttribute: {:?}", node.kind),
        };
        self.emit_jsx_attribute_name(name);
        self.write_punctuation("=");
        if let Some(initializer) = initializer {
            self.emit_jsx_attribute_value(initializer);
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_spread_attribute(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = match &node.data {
            NodeData::JsxSpreadAttribute(d) => &d.expression,
            _ => panic!("unexpected JsxSpreadAttribute: {:?}", node.kind),
        };
        self.write_punctuation("{");
        self.write_punctuation("...");
        self.emit_expression(expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (dot_dot_dot_token, expression) = match &node.data {
            NodeData::JsxExpression(d) => (d.dot_dot_dot_token.as_deref(), d.expression.as_deref()),
            _ => panic!("unexpected JsxExpression: {:?}", node.kind),
        };
        self.emit_punctuation_node(dot_dot_dot_token);
        if let Some(expression) = expression {
            self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_namespaced_name(&mut self, node: &Node) {
        let (namespace, name) = match &node.data {
            NodeData::JsxNamespacedName(d) => (&d.namespace, &d.name),
            _ => panic!("unexpected JsxNamespacedName: {:?}", node.kind),
        };
        self.emit_identifier_name(namespace);
        self.write_punctuation(":");
        self.emit_identifier_name(name);
    }

    pub(crate) fn emit_jsx_attribute_name(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::JsxNamespacedName => self.emit_jsx_namespaced_name(node),
            _ => panic!("unhandled JsxAttributeName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_jsx_attribute_value(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            SyntaxKind::JsxExpression => self.emit_jsx_expression(node),
            SyntaxKind::JsxElement => self.emit_jsx_element(node),
            SyntaxKind::JsxSelfClosingElement => self.emit_jsx_self_closing_element(node),
            SyntaxKind::JsxFragment => self.emit_jsx_fragment(node),
            _ => self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST),
        }
    }

    pub(crate) fn emit_jsx_child(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::JsxText => self.emit_jsx_text(node),
            SyntaxKind::JsxExpression => self.emit_jsx_expression(node),
            SyntaxKind::JsxElement => self.emit_jsx_element(node),
            SyntaxKind::JsxSelfClosingElement => self.emit_jsx_self_closing_element(node),
            SyntaxKind::JsxFragment => self.emit_jsx_fragment(node),
            _ => panic!("unhandled JsxChild: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_jsx_text(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let text = match &node.data {
            NodeData::JsxText(d) => &d.text,
            _ => panic!("unexpected JsxText: {:?}", node.kind),
        };
        self.write_as(text, WriteKind::Literal);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_jsx_tag_name(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_reference(node),
            SyntaxKind::ThisKeyword => self.emit_keyword_expression(node),
            SyntaxKind::JsxNamespacedName => self.emit_jsx_namespaced_name(node),
            SyntaxKind::PropertyAccessExpression => self.emit_property_access_expression(node),
            _ => panic!("unhandled JsxTagName: {:?}", node.kind),
        }
    }
}

fn expression_of13(node: &Node) -> &Node {
    match &node.data {
        NodeData::DeleteExpression(d) => &d.expression,
        NodeData::TypeOfExpression(d) => &d.expression,
        NodeData::VoidExpression(d) => &d.expression,
        NodeData::AwaitExpression(d) => &d.expression,
        NodeData::NonNullExpression(d) => &d.expression,
        NodeData::Decorator(d) => &d.expression,
        NodeData::ExternalModuleReference(d) => &d.expression,
        _ => panic!("expression() on {:?}", node.kind),
    }
}

fn call_expression_question_dot_token13(node: &Node) -> Option<&Node> {
    match &node.data {
        NodeData::CallExpression(d) => d.question_dot_token.as_deref(),
        _ => None,
    }
}

fn new_expression_has_arguments13(node: &Node) -> bool {
    match &node.data {
        NodeData::NewExpression(d) => {
            !d.arguments.as_deref().is_some_and(|arguments| !arguments.nodes.is_empty())
        }
        _ => false,
    }
}

fn is_new_expression_without_arguments13(node: &Node) -> bool {
    node.kind == SyntaxKind::NewExpression && !new_expression_has_arguments13(node)
}

fn binary_operator_token_kind13(node: &Node) -> SyntaxKind {
    match &node.data {
        NodeData::BinaryExpression(d) => d.operator_token.kind,
        _ => SyntaxKind::Unknown,
    }
}

fn get_binary_expression_precedence13(node: &Node) -> (OperatorPrecedence, OperatorPrecedence) {
    match &node.data {
        NodeData::BinaryExpression(d) => {
            let precedence = get_binary_operator_precedence(d.operator_token.kind);
            (precedence, precedence)
        }
        _ => (OperatorPrecedence::Invalid, OperatorPrecedence::Invalid),
    }
}

fn get_expression_precedence13(node: &Node) -> OperatorPrecedence {
    let node = skip_partially_emitted_expressions07(node);
    let operator = match &node.data {
        NodeData::BinaryExpression(d) => d.operator_token.kind,
        NodeData::PrefixUnaryExpression(d) => d.operator,
        NodeData::PostfixUnaryExpression(d) => d.operator,
        _ => node.kind,
    };
    let mut flags = OperatorPrecedenceFlags::NONE;
    if node.kind == SyntaxKind::NewExpression && !new_expression_has_arguments13(node) {
        flags = OperatorPrecedenceFlags::NEW_WITHOUT_ARGUMENTS;
    } else if is_optional_chain(node) {
        flags = OperatorPrecedenceFlags::OPTIONAL_CHAIN;
    }
    get_operator_precedence(node.kind, operator, flags)
}

fn skip_synthesized_parentheses13(node: &Node) -> &Node {
    let mut current = node;
    while current.kind == SyntaxKind::ParenthesizedExpression && node_is_synthesized(current) {
        match &current.data {
            NodeData::ParenthesizedExpression(d) => current = &d.expression,
            _ => break,
        }
    }
    current
}

fn range_start_positions_are_on_same_line13(
    range1: TextRange,
    range2: TextRange,
    source_file: &tsox_frontend::ast::node_source_file::SourceFile,
) -> bool {
    let start1 = get_start_position_of_range(range1, source_file, true);
    let start2 = get_start_position_of_range(range2, source_file, true);
    get_lines_between_positions(source_file, start1, start2) == 0
}

fn jsx_attributes_is_empty13(node: &Node) -> bool {
    match &node.data {
        NodeData::JsxAttributes(d) => d.properties.nodes.is_empty(),
        _ => panic!("jsxAttributes() on {:?}", node.kind),
    }
}
