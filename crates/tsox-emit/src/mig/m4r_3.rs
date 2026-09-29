#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use super::m4r::*;
use tsox_frontend::ast::mig::m3g_3::{
    is_var_await_using, is_var_const, is_var_let, is_var_using,
};
use tsox_frontend::ast::mig::m3g::is_modifier;
use tsox_frontend::ast::node_data_generated::{
    is_block, is_decorator, is_literal_kind, is_parenthesized_expression,
    is_partially_emitted_expression, NodeData, PartiallyEmittedExpressionData,
};
use super::m4r_4::{get_closing_bracket, get_opening_bracket};
use tsox_frontend::ast::utilities::{node_is_synthesized, position_is_synthesized};
use tsox_frontend::scanner::{get_leading_comment_ranges, get_trailing_comment_ranges};
use tsox_frontend::ast::node_source_file::{FileReference, SourceFile};
use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::{Node, NodeList, SyntaxKind};

include!("r36k16_defs.rs");

fn block_scoped_flags_of(node: &Node) -> tsox_frontend::ast::node_flags::NodeFlags {
    node.flags.intersection(tsox_frontend::ast::node_flags::NodeFlags::BlockScoped)
}

fn is_var_let_node(node: &Node) -> bool {
    block_scoped_flags_of(node) == tsox_frontend::ast::node_flags::NodeFlags::Let
}

fn is_var_const_node(node: &Node) -> bool {
    block_scoped_flags_of(node) == tsox_frontend::ast::node_flags::NodeFlags::Const
}

fn is_var_using_node(node: &Node) -> bool {
    block_scoped_flags_of(node) == tsox_frontend::ast::node_flags::NodeFlags::Using
}

fn is_var_await_using_node(node: &Node) -> bool {
    block_scoped_flags_of(node) == tsox_frontend::ast::node_flags::NodeFlags::AwaitUsing
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModeK04 {
    None,
    Modifiers,
    Decorators,
}

const TEF_INDENT_LEADING_COMMENTS: TokenEmitFlags =
    tsox_frontend::format::mig::m4o::TokenEmitFlags::INDENT_LEADING_COMMENTS.0 as i32;
const LF_SINGLE_LINE_BLOCK_STATEMENTS: ListFormat =
    crate::mig::m4q::r33k12_defs::LF_SINGLE_LINE_BLOCK_STATEMENTS;
const LF_MULTI_LINE_BLOCK_STATEMENTS: ListFormat =
    crate::mig::m4q::r33k12_defs::LF_MULTI_LINE_BLOCK_STATEMENTS;

impl Printer {
    pub fn get_literal_kind_of_binary_plus_operand(&self, node: &Arc<Node>) -> SyntaxKind {
        let node = skip_partially_emitted_expressions(node);

        if is_literal_kind(node.kind) {
            return node.kind;
        }

        if node.kind == SyntaxKind::BinaryExpression
            && node.operator_token().kind == SyntaxKind::PlusToken
        {
            let left_kind = self.get_literal_kind_of_binary_plus_operand(node.left());
            let mut literal_kind = SyntaxKind::Unknown;
            if is_literal_kind(left_kind)
                && left_kind == self.get_literal_kind_of_binary_plus_operand(node.right())
            {
                literal_kind = left_kind;
            }
            return literal_kind;
        }

        SyntaxKind::Unknown
    }

    pub fn get_binary_expression_precedence(&self, node: &Arc<Node>) -> (OperatorPrecedence, OperatorPrecedence) {
        let precedence = get_expression_precedence(node);
        let mut left_prec = precedence;
        let mut right_prec = precedence;
        match precedence {
            OPERATOR_PRECEDENCE_COMMA => {}
            OPERATOR_PRECEDENCE_ASSIGNMENT => {
                left_prec = OPERATOR_PRECEDENCE_CONDITIONAL;
                right_prec = OPERATOR_PRECEDENCE_YIELD;
            }
            OPERATOR_PRECEDENCE_LOGICAL_OR => {
                right_prec = OPERATOR_PRECEDENCE_LOGICAL_AND;
            }
            OPERATOR_PRECEDENCE_LOGICAL_AND => {
                right_prec = OPERATOR_PRECEDENCE_BITWISE_OR;
            }
            OPERATOR_PRECEDENCE_BITWISE_OR => {}
            OPERATOR_PRECEDENCE_BITWISE_XOR => {}
            OPERATOR_PRECEDENCE_BITWISE_AND => {}
            OPERATOR_PRECEDENCE_EQUALITY => {
                right_prec = OPERATOR_PRECEDENCE_RELATIONAL;
            }
            OPERATOR_PRECEDENCE_RELATIONAL => {
                right_prec = OPERATOR_PRECEDENCE_SHIFT;
            }
            OPERATOR_PRECEDENCE_SHIFT => {
                right_prec = OPERATOR_PRECEDENCE_ADDITIVE;
            }
            OPERATOR_PRECEDENCE_ADDITIVE => {
                if node.operator_token().kind == SyntaxKind::PlusToken
                    && is_binary_operation(node.right(), SyntaxKind::PlusToken)
                {
                    let left_kind = self.get_literal_kind_of_binary_plus_operand(node.left());
                    if is_literal_kind(left_kind)
                        && left_kind == self.get_literal_kind_of_binary_plus_operand(node.right())
                    {
                        return (left_prec, right_prec);
                    }
                }
                right_prec = OPERATOR_PRECEDENCE_MULTIPLICATIVE;
            }
            OPERATOR_PRECEDENCE_MULTIPLICATIVE => {
                if node.operator_token().kind == SyntaxKind::AsteriskToken
                    && is_binary_operation(node.right(), SyntaxKind::AsteriskToken)
                {
                    return (left_prec, right_prec);
                }
                right_prec = OPERATOR_PRECEDENCE_EXPONENTIATION;
            }
            OPERATOR_PRECEDENCE_EXPONENTIATION => {
                left_prec = OPERATOR_PRECEDENCE_UPDATE;
            }
            _ => panic!("unhandled precedence: {:?}", precedence),
        }
        (left_prec, right_prec)
    }

    pub fn emit_yield_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::YieldKeyword, node.pos(), WriteKind::Keyword, node);
        self.emit_punctuation_node(node.asterisk_token());
        if let Some(expression) = node.expression() {
            self.write_space();
            self.emit_expression_no_asi(expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        }
        self.exit_node(node, state);
    }

    pub fn synthetic_comment_will_emit_new_line(&self, comment: &SynthesizedComment) -> bool {
        comment.kind == SyntaxKind::SingleLineCommentTrivia || comment.has_trailing_new_line
    }

    pub fn will_emit_leading_new_line(&self, node: &Arc<Node>) -> bool {
        let Some(source_file) = self.current_source_file.as_deref() else {
            return false;
        };
        let mut has_leading_comment_ranges = false;
        let mut has_new_line_comment = false;
        for comment in get_leading_comment_ranges(source_file.text(), node.pos()) {
            has_leading_comment_ranges = true;
            if self.comment_will_emit_new_line(comment) {
                has_new_line_comment = true;
            }
        }
        if has_leading_comment_ranges {
            let parse_node = self.emit_context.parse_node(node);
            if let Some(parse_node) = parse_node {
                if let Some(parent) = parse_node.parent() {
                    if is_parenthesized_expression(&parent) {
                        return true;
                    }
                }
            }
        }
        if has_new_line_comment {
            return true;
        }
        if self
            .emit_context
            .get_synthetic_leading_comments(node)
            .iter()
            .any(|c| self.synthetic_comment_will_emit_new_line(c))
        {
            return true;
        }
        if is_partially_emitted_expression(node) {
            let Some(expression) = node.expression() else {
                return false;
            };
            if node.pos() != expression.pos() {
                for comment in get_trailing_comment_ranges(source_file.text(), expression.pos()) {
                    if self.comment_will_emit_new_line(comment) {
                        return true;
                    }
                }
            }
            return self.will_emit_leading_new_line(expression);
        }
        false
    }

    pub fn parenthesize_expression_for_no_asi(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !self.comments_disabled {
            match node.kind {
                SyntaxKind::PartiallyEmittedExpression => {
                    if self.will_emit_leading_new_line(node) {
                        let parse_node = self.emit_context.parse_node(node);
                        if let Some(parse_node) = parse_node {
                            if is_parenthesized_expression(&parse_node) {
                                let Some(pee_expression) = node.expression() else {
                                    panic!("PartiallyEmittedExpression without expression");
                                };
                                let mut parens = self
                                    .emit_context
                                    .factory()
                                    .new_parenthesized_expression(pee_expression);
                                self.emit_context.set_original(&parens, node);
                                if let Some(parens_node) = Arc::get_mut(&mut parens) {
                                    parens_node.loc = parse_node.loc;
                                }
                                return Some(parens);
                            }
                        }
                        return Some(
                            self.emit_context
                                .factory()
                                .new_parenthesized_expression(node),
                        );
                    }
                    let expression = self
                        .parenthesize_expression_for_no_asi(node.expression().expect("PartiallyEmittedExpression expression"));
                    let parenthesized = expression.expect("parenthesized expression");
                    Some(
                        self.emit_context
                            .factory()
                            .update_partially_emitted_expression(node, &parenthesized),
                    )
                }
                SyntaxKind::PropertyAccessExpression => Some(
                    self.emit_context.factory().update_property_access_expression(
                        node,
                        node.expression().and_then(|e| self.parenthesize_expression_for_no_asi(e)),
                        node.question_dot_token(),
                        node.name().expect("property access name"),
                    ),
                ),
                SyntaxKind::ElementAccessExpression => {
                    let argument_expression = match &node.data {
                        NodeData::ElementAccessExpression(d) => &d.argument_expression,
                        _ => panic!("unexpected ElementAccessExpression: {:?}", node.kind),
                    };
                    Some(
                        self.emit_context.factory().update_element_access_expression(
                            node,
                            node.expression().and_then(|e| self.parenthesize_expression_for_no_asi(e)),
                            node.question_dot_token(),
                            argument_expression,
                        ),
                    )
                }
                SyntaxKind::CallExpression => Some(
                    self.emit_context.factory().update_call_expression(
                        node,
                        node.expression().and_then(|e| self.parenthesize_expression_for_no_asi(e)),
                        node.question_dot_token(),
                        node.type_arguments(),
                        node.arguments().expect("call arguments"),
                    ),
                ),
                SyntaxKind::TaggedTemplateExpression => {
                    let (tag, template) = match &node.data {
                        NodeData::TaggedTemplateExpression(d) => (&d.tag, &d.template),
                        _ => panic!("unexpected TaggedTemplateExpression: {:?}", node.kind),
                    };
                    Some(
                        self.emit_context.factory().update_tagged_template_expression(
                            node,
                            self.parenthesize_expression_for_no_asi(tag),
                            node.question_dot_token(),
                            node.type_arguments(),
                            template,
                        ),
                    )
                }
                _ => Some(Arc::clone(node)),
            }
        } else {
            Some(Arc::clone(node))
        }
    }

    pub fn is_empty_block(&self, block: &Node, statements: &NodeList) -> bool {
        statements.nodes.is_empty()
            && (self.current_source_file.is_none()
                || range_end_is_on_same_line_as_range_start(
                    block.loc,
                    block.loc,
                    self.current_source_file.as_deref().unwrap(),
                ))
    }

    pub fn emit_variable_statement(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers().map(|m| &**m), false);
        self.emit_variable_declaration_list(node.declaration_list());
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_while_clause(&mut self, node: &Node, expression: &Arc<Node>, start_pos: usize) {
        let pos = self.emit_token(SyntaxKind::WhileKeyword, start_pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(SyntaxKind::CloseParenToken, expression.end(), WriteKind::Punctuation, node);
    }

    pub fn emit_while_statement(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_while_clause(node, node.expression().expect("while expression"), node.pos());
        self.emit_embedded_statement(node, node.statement());
        self.exit_node(node, state);
    }

    pub fn emit_with_statement(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let pos = self.emit_token(SyntaxKind::WithKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_expression(node.expression().expect("with expression"), OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(
            SyntaxKind::CloseParenToken,
            node.expression().expect("with expression").end(),
            WriteKind::Punctuation,
            node,
        );
        self.emit_embedded_statement(node, node.statement());
        self.exit_node(node, state);
    }

    pub fn emit_try_statement(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::TryKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_block(node.try_block());
        if let Some(catch_clause) = node.catch_clause() {
            self.write_line_or_space(node, node.try_block(), catch_clause);
            self.emit_catch_clause(catch_clause);
        }
        if let Some(finally_block) = node.finally_block() {
            let anchor = node.catch_clause().unwrap_or(node.try_block());
            self.write_line_or_space(node, anchor, finally_block);
            self.emit_token(SyntaxKind::FinallyKeyword, anchor.end(), WriteKind::Keyword, node);
            self.write_space();
            self.emit_block(finally_block);
        }
        self.exit_node(node, state);
    }

    pub fn emit_variable_declaration(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_binding_name(node.name());
        self.emit_punctuation_node(node.exclamation_token());
        self.emit_type_annotation(node.ty());
        let name = node.name().expect("variable declaration name");
        let type_node = self.emit_context.get_type_node(name);
        let mut initializer_pos = name.end() as i64;
        if let Some(ty) = node.ty() {
            initializer_pos = initializer_pos.max(ty.end() as i64);
        }
        if let Some(type_node) = &type_node {
            initializer_pos = initializer_pos.max(type_node.end() as i64);
        }
        self.emit_initializer(node.initializer(), initializer_pos as usize, node);
        self.exit_node(node, state);
    }

    pub fn emit_variable_declaration_node(&mut self, node: &Node) {
        self.emit_variable_declaration(node);
    }

    pub fn emit_variable_declaration_list(&mut self, node: &Node) {
        let state = self.enter_node(node);
        if is_var_let_node(node) {
            self.write_keyword("let");
        } else if is_var_const_node(node) {
            self.write_keyword("const");
        } else if is_var_using_node(node) {
            self.write_keyword("using");
        } else if is_var_await_using_node(node) {
            self.write_keyword("await");
            self.write_space();
            self.write_keyword("using");
        } else {
            self.write_keyword("var");
        }
        self.write_space();
        self.emit_list(
            |p, n| p.emit_variable_declaration_node(n),
            node,
            Some(node.declarations()),
            LF_VARIABLE_DECLARATION_LIST,
        );
        self.exit_node(node, state);
    }

    pub fn emit_type_alias_declaration(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_modifier_list(node, node.modifiers().map(|m| &**m), false);
        self.write_keyword("type");
        self.write_space();
        self.emit_binding_identifier(node.name().expect("TypeAliasDeclaration name"));
        self.emit_type_parameters(node, node.type_parameters().map(|v| &**v));
        self.write_space();
        self.write_punctuation("=");
        self.write_space();
        self.emit_type_node_outside_extends(node.ty().expect("type alias type node"));
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_triple_slash_directives(&mut self, source_file: &SourceFile) {
        self.emit_directive("path", &source_file.referenced_files);
        self.emit_directive("types", &source_file.type_reference_directives);
        self.emit_directive("lib", &source_file.lib_reference_directives);
    }

    pub fn emit_directive(&mut self, kind: &str, refs: &[FileReference]) {
        for r in refs {
            let mut resolution_mode = String::new();
            if r.resolution_mode != tsox_core::core::compiler_options_kinds::ResolutionMode::None {
                resolution_mode = format!(
                    "resolution-mode=\"{}\" ",
                    if r.resolution_mode == tsox_core::core::compiler_options_kinds::ResolutionMode::ESNext {
                        "import"
                    } else {
                        "require"
                    }
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

    pub fn has_trailing_comma(&self, _parent_node: &Node, children: &NodeList) -> bool {
        children.has_trailing_comma()
    }

    pub fn set_source_file(&mut self, source_file: Option<&Arc<SourceFile>>) {
        self.current_source_file = source_file.cloned();
        self.unique_helper_names = HashMap::new();
        self.external_helpers_module_name = None;
        if let Some(source_file) = source_file {
            if self
                .emit_context
                .emit_flags(&self.emit_context.most_original(&source_file.node))
                & (EF_EXTERNAL_HELPERS as u32)
                != 0
            {
                self.unique_helper_names = HashMap::new();
            }
            self.external_helpers_module_name =
                self.emit_context.get_external_helpers_module_name(&source_file.node);
            self.set_source_map_source(source_file);
        }
    }
}

pub type EmitFn = fn(&mut Printer, &Node);

const LF_OPTIONAL_IF_NIL: ListFormat = tsox_frontend::format::mig::m4o::ListFormat::OPTIONAL_IF_NIL.0;
const LF_DELIMITERS_MASK: ListFormat = tsox_frontend::format::mig::m4o::ListFormat::DELIMITERS_MASK.0;
const LF_ASTERISK_DELIMITED: ListFormat = tsox_frontend::format::mig::m4o::ListFormat::ASTERISK_DELIMITED.0;
const LF_LINES_MASK: ListFormat = tsox_frontend::format::mig::m4o::ListFormat::LINES_MASK.0;
const LF_NO_INTERVENING_COMMENTS: ListFormat = tsox_frontend::format::mig::m4o::ListFormat::NO_INTERVENING_COMMENTS.0;
const LF_SPACE_AFTER_LIST: ListFormat = tsox_frontend::format::mig::m4o::ListFormat::SPACE_AFTER_LIST.0;
const LF_PARAMETERS: ListFormat = crate::mig::m4q::r33k12_defs::LF_PARAMETERS;
const LF_MODIFIERS: ListFormat = crate::mig::m4q::r33k12_defs::LF_MODIFIERS;
const LF_DECORATORS: ListFormat = crate::mig::m4q::r33k12_defs::LF_DECORATORS;
const LF_OBJECT_BINDING_PATTERN_ELEMENTS: ListFormat =
    crate::mig::m4q::r33k12_defs::LF_OBJECT_BINDING_PATTERN_ELEMENTS;
const LF_ARRAY_BINDING_PATTERN_ELEMENTS: ListFormat =
    crate::mig::m4q::r33k12_defs::LF_ARRAY_BINDING_PATTERN_ELEMENTS;

impl Printer {
    pub fn emit_expression_no_asi(&mut self, node: &Arc<Node>, precedence: OperatorPrecedence) {
        let parenthesized = self
            .parenthesize_expression_for_no_asi(node)
            .expect("parenthesized expression");
        self.emit_expression(&parenthesized, precedence);
    }

    pub fn emit_binding_identifier(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let text = self.get_text_of_node(node, false);
        self.write(&text);
        self.exit_node(node, state);
    }

    pub fn emit_binding_name(&mut self, node: Option<&Arc<Node>>) {
        let Some(node) = node else {
            return;
        };
        match node.kind {
            SyntaxKind::Identifier => self.emit_binding_identifier(node),
            SyntaxKind::ObjectBindingPattern => self.emit_object_binding_pattern(node),
            SyntaxKind::ArrayBindingPattern => self.emit_array_binding_pattern(node),
            _ => panic!("unexpected BindingName: {:?}", node.kind),
        }
    }

    pub fn emit_object_binding_pattern(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let elements = match &node.data {
            NodeData::BindingPattern(d) => &d.elements,
            _ => panic!("unexpected ObjectBindingPattern: {:?}", node.kind),
        };
        self.write_punctuation("{");
        self.emit_list(
            |p, n| p.emit_binding_element_node(n),
            node,
            Some(&**elements),
            LF_OBJECT_BINDING_PATTERN_ELEMENTS,
        );
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_array_binding_pattern(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let elements = match &node.data {
            NodeData::BindingPattern(d) => &d.elements,
            _ => panic!("unexpected ArrayBindingPattern: {:?}", node.kind),
        };
        self.write_punctuation("[");
        self.emit_list(
            |p, n| p.emit_binding_element_node(n),
            node,
            Some(&**elements),
            LF_ARRAY_BINDING_PATTERN_ELEMENTS,
        );
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub fn emit_binding_element_node(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (dot_dot_dot_token, property_name, name, initializer) = match &node.data {
            NodeData::BindingElement(d) => (
                d.dot_dot_dot_token.as_deref(),
                d.property_name.clone(),
                d.name.clone(),
                d.initializer.as_ref(),
            ),
            _ => panic!("unexpected BindingElement: {:?}", node.kind),
        };
        self.emit_token_node(dot_dot_dot_token);
        if let Some(property_name) = property_name {
            self.emit_property_name(Some(&property_name));
            self.write_punctuation(":");
            self.write_space();
        }
        if let Some(name) = name {
            let name_end = name.end();
            self.emit_binding_name(Some(&name));
            self.emit_initializer(initializer, name_end, node);
        }
        self.exit_node(node, state);
    }

    pub fn emit_property_name(&mut self, node: Option<&Arc<Node>>) {
        let Some(node) = node else {
            return;
        };
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(node),
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                self.emit_no_substitution_template_literal(node)
            }
            SyntaxKind::NumericLiteral => self.emit_numeric_literal(node),
            SyntaxKind::BigIntLiteral => self.emit_big_int_literal(node),
            SyntaxKind::ComputedPropertyName => self.emit_computed_property_name(node),
            _ => panic!("unexpected PropertyName: {:?}", node.kind),
        }
    }

    pub fn emit_computed_property_name(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let expression = match &node.data {
            NodeData::ComputedPropertyName(d) => &d.expression,
            _ => panic!("unexpected ComputedPropertyName: {:?}", node.kind),
        };
        self.write_punctuation("[");
        self.emit_expression(expression, OperatorPrecedence::Comma);
        self.write_punctuation("]");
        self.exit_node(node, state);
    }

    pub fn emit_literal(&mut self, node: &Arc<Node>, flags: GetLiteralTextFlags) {
        let text = self.get_literal_text_of_node(node, None, flags);
        self.writer.write_string_literal(&text);
    }

    pub fn emit_string_literal(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_no_substitution_template_literal(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_numeric_literal(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_big_int_literal(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_initializer(
        &mut self,
        node: Option<&Arc<Node>>,
        equal_token_pos: usize,
        context_node: &Node,
    ) {
        let Some(node) = node else {
            return;
        };
        self.write_space();
        self.emit_token(SyntaxKind::EqualsToken, equal_token_pos, WriteKind::Operator, context_node);
        self.write_space();
        self.emit_expression(node, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
    }

    pub fn emit_decorator(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let expression = match &node.data {
            NodeData::Decorator(d) => &d.expression,
            _ => panic!("unexpected Decorator: {:?}", node.kind),
        };
        self.write_punctuation("@");
        self.emit_expression(expression, OperatorPrecedence::LeftHandSide);
        self.exit_node(node, state);
    }

    pub fn emit_modifier_like(&mut self, node: &Node) {
        if is_decorator(node) {
            self.emit_decorator(node);
        } else if is_modifier(node) {
            self.emit_keyword_node(Some(node));
        } else {
            panic!("unhandled ModifierLike: {:?}", node.kind);
        }
    }

    pub fn emit_modifier_list(
        &mut self,
        parent_node: &Node,
        modifiers: Option<&ModifierList>,
        allow_decorators: bool,
    ) -> usize {
        let Some(modifiers) = modifiers else {
            return parent_node.pos();
        };
        if modifiers.nodes.is_empty() {
            return parent_node.pos();
        }

        if modifiers.nodes.iter().all(|n| is_modifier(n)) {
            self.emit_list(
                |p, n| p.emit_keyword_node(Some(n)),
                parent_node,
                Some(&modifiers.list),
                LF_MODIFIERS,
            );
        } else if modifiers.nodes.iter().all(|n| is_decorator(n)) {
            if !allow_decorators {
                return parent_node.pos();
            }
            self.emit_list(
                Self::emit_modifier_like,
                parent_node,
                Some(&modifiers.list),
                LF_DECORATORS,
            );
        } else {
            let mut last_mode = ModeK04::None;
            let mut mode = ModeK04::None;
            let mut start = 0;
            let mut pos = 0;
            let mut last_modifier: &Node = &modifiers.nodes[0];
            while start < modifiers.nodes.len() {
                while pos < modifiers.nodes.len() {
                    last_modifier = &modifiers.nodes[pos];
                    if is_decorator(last_modifier) {
                        mode = ModeK04::Decorators;
                    } else {
                        mode = ModeK04::Modifiers;
                    }
                    if last_mode == ModeK04::None {
                        last_mode = mode;
                    } else if mode != last_mode {
                        break;
                    }
                    pos += 1;
                }

                let mut text_range = TextRange::undefined();
                if start == 0 {
                    text_range = TextRange::new(modifiers.pos(), text_range.end());
                }
                if pos == modifiers.nodes.len() - 1 {
                    text_range = TextRange::new(text_range.pos(), modifiers.end());
                }
                if allow_decorators || last_mode == ModeK04::Modifiers {
                    self.emit_list_items(
                        Self::emit_modifier_like,
                        Some(parent_node),
                        &modifiers.nodes[start..pos],
                        if last_mode == ModeK04::Modifiers {
                            LF_MODIFIERS
                        } else {
                            LF_DECORATORS
                        },
                        false,
                        text_range,
                    );
                }
                start = pos;
                last_mode = mode;
                pos += 1;
            }
        }

        let mut end = parent_node.pos() as i64;
        if let Some(last) = modifiers.nodes.last() {
            let last_end = last.end() as i64;
            if end < last_end {
                end = last_end;
            }
        }
        end as usize
    }

    pub fn emit_parameter_declaration_node(&mut self, node: &Node) {
        if node.kind == SyntaxKind::Parameter {
            self.emit_parameter(node);
        }
    }

    pub fn emit_parameter(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (modifiers, dot_dot_dot_token, name, question_token, type_node, initializer) =
            match &node.data {
                NodeData::ParameterDeclaration(d) => (
                    d.modifiers.as_deref(),
                    d.dot_dot_dot_token.as_deref(),
                    &d.name,
                    d.question_token.as_deref(),
                    d.type_node.as_deref(),
                    d.initializer.as_ref(),
                ),
                _ => panic!("unexpected ParameterDeclaration: {:?}", node.kind),
            };
        self.emit_modifier_list(node, modifiers, true);
        self.emit_token_node(dot_dot_dot_token);
        self.emit_binding_name(Some(name));
        self.emit_token_node(question_token);
        self.emit_type_annotation(type_node);
        let mut initializer_pos = node.pos() as i64;
        if let Some(part) = type_node {
            initializer_pos = initializer_pos.max(part.end() as i64);
        }
        if let Some(part) = question_token {
            initializer_pos = initializer_pos.max(part.end() as i64);
        }
        initializer_pos = initializer_pos.max(name.end() as i64);
        if let Some(part) = modifiers {
            initializer_pos = initializer_pos.max(part.end() as i64);
        }
        self.emit_initializer(initializer, initializer_pos as usize, node);
        self.exit_node(node, state);
    }

    pub fn emit_parameters(&mut self, parent_node: &Node, parameters: &NodeList) {
        self.generate_all_names(Some(parameters));
        self.emit_list(
            |p, n| p.emit_parameter_declaration_node(n),
            parent_node,
            Some(parameters),
            LF_PARAMETERS,
        );
    }

    pub fn emit_parameters_for_index_signature(&mut self, parent_node: &Node, parameters: &NodeList) {
        self.emit_list(
            |p, n| p.emit_parameter_declaration_node(n),
            parent_node,
            Some(parameters),
            LF_PARAMETERS,
        );
    }

    pub fn emit_signature(&mut self, node: &Node) {
        let (type_parameters, parameters, type_node) = match &node.data {
            NodeData::MethodSignatureDeclaration(d) => {
                (d.type_parameters.as_ref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::CallSignatureDeclaration(d) => {
                (d.type_parameters.as_ref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::ConstructSignatureDeclaration(d) => {
                (d.type_parameters.as_ref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::GetAccessorDeclaration(d) => {
                (d.type_parameters.as_ref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::SetAccessorDeclaration(d) => {
                (d.type_parameters.as_ref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::FunctionTypeNode(d) => {
                (d.type_parameters.as_ref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::ConstructorTypeNode(d) => {
                (d.type_parameters.as_ref(), &d.parameters, d.type_node.as_deref())
            }
            _ => panic!("unexpected signature-bearing node: {:?}", node.kind),
        };
        self.emit_type_parameters(node, type_parameters.map(|l| &**l));
        self.emit_parameters(node, parameters);
        self.emit_type_annotation(type_node);
    }

    pub fn emit_return_type(&mut self, node: Option<&Node>) {
        let Some(node) = node else {
            return;
        };
        self.write_punctuation("=>");
        self.write_space();
        let infer_with_constraint = node.kind == SyntaxKind::InferType
            && match &node.data {
                NodeData::InferTypeNode(d) => match &d.type_parameter.data {
                    NodeData::TypeParameterDeclaration(tp) => tp.constraint.is_some(),
                    _ => false,
                },
                _ => false,
            };
        if self.in_extends && infer_with_constraint {
            self.emit_type_node_preserving_extends(node, TYPE_PRECEDENCE_HIGHEST);
        } else {
            self.emit_type_node_preserving_extends(node, TYPE_PRECEDENCE_LOWEST);
        }
    }

    pub fn write_delimiter(&mut self, format: ListFormat) {
        match format & LF_DELIMITERS_MASK {
            LF_NONE => {}
            LF_COMMA_DELIMITED => self.write_punctuation(","),
            LF_BAR_DELIMITED => {
                self.write_space();
                self.write_punctuation("|");
            }
            LF_ASTERISK_DELIMITED => {
                self.write_space();
                self.write_punctuation("*");
                self.write_space();
            }
            _ => {
                self.write_space();
                self.write_punctuation("&");
            }
        }
    }

    pub fn decrease_indent(&mut self) {
        self.writer.decrease_indent();
    }

    pub fn emit_list(
        &mut self,
        emit: EmitFn,
        parent_node: &Node,
        children: Option<&NodeList>,
        mut format: ListFormat,
    ) {
        if self.should_emit_on_multiple_lines(parent_node) {
            format |= LF_PREFER_NEW_LINE | LF_INDENTED;
        }
        self.emit_list_range(emit, parent_node, children, format, -1, -1);
    }

    pub fn emit_list_range(
        &mut self,
        emit: EmitFn,
        parent_node: &Node,
        children: Option<&NodeList>,
        mut format: ListFormat,
        mut start: i64,
        mut count: i64,
    ) {
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
        if is_nil && format & LF_OPTIONAL_IF_NIL != 0 {
            return;
        }
        let is_empty = is_nil || start >= length || count <= 0;
        if is_empty && format & LF_OPTIONAL_IF_EMPTY != 0 {
            return;
        }
        if format & LF_BRACKETS_MASK != 0 {
            self.write_punctuation(get_opening_bracket(format));
        }
        if is_empty {
            if format & LF_MULTI_LINE != 0
                && !(self.options.preserve_source_newlines
                    && self.current_source_file.is_some()
                    && range_is_on_single_line(
                        parent_node.loc,
                        self.current_source_file.as_deref().unwrap(),
                    ))
            {
                self.write_line();
            } else if format & LF_SPACE_BETWEEN_BRACES != 0 && format & LF_NO_SPACE_IF_EMPTY == 0 {
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
        if format & LF_BRACKETS_MASK != 0 {
            self.write_punctuation(get_closing_bracket(format));
        }
    }

    pub fn emit_list_items(
        &mut self,
        emit: EmitFn,
        parent_node: Option<&Node>,
        children: &[Arc<Node>],
        format: ListFormat,
        has_trailing_comma: bool,
        children_text_range: TextRange,
    ) {
        let may_emit_intervening_comments = format & LF_NO_INTERVENING_COMMENTS == 0;
        let mut should_emit_intervening_comments = may_emit_intervening_comments;

        let mut leading_line_terminator_count = 0;
        if let Some(first) = children.first().map(|f| &**f) {
            leading_line_terminator_count =
                self.get_leading_line_terminator_count_node(parent_node, Some(first), format);
        }
        if leading_line_terminator_count > 0 {
            for _ in 0..leading_line_terminator_count {
                self.write_line();
            }
            should_emit_intervening_comments = false;
        } else if format & LF_SPACE_BETWEEN_BRACES != 0 {
            self.write_space();
        }

        if format & LF_INDENTED != 0 {
            self.increase_indent();
        }

        let parent_end: i64 = parent_node.map_or(-1, |p| p.end() as i64);

        let mut previous_sibling: Option<&Node> = None;
        let mut should_decrease_indent_after_emit = false;
        for child in children {
            if format & LF_ASTERISK_DELIMITED != 0 {
                self.write_line();
                self.write_delimiter(format);
            } else if let Some(prev) = previous_sibling {
                self.write_delimiter(format);
                let separating_line_terminator_count =
                    self.get_separating_line_terminator_count_node(previous_sibling, Some(&**child), format);
                if separating_line_terminator_count > 0 {
                    if format & LF_LINES_MASK == LF_NONE && format & LF_INDENTED != 0 {
                        self.increase_indent();
                        should_decrease_indent_after_emit = true;
                    }
                    for _ in 0..separating_line_terminator_count {
                        self.write_line();
                    }
                    should_emit_intervening_comments = false;
                } else if format & LF_SPACE_BETWEEN_SIBLINGS != 0 {
                    self.write_space();
                }
            }

            if should_emit_intervening_comments && !self.should_emit_leading_comments(child) {
                should_emit_intervening_comments = may_emit_intervening_comments;
            }

            self.next_list_element_pos = child.pos();
            emit(self, child);

            if should_decrease_indent_after_emit {
                self.decrease_indent();
                should_decrease_indent_after_emit = false;
            }
            previous_sibling = Some(&**child);
        }

        let skip_trailing_comments = self.comments_disabled
            || !previous_sibling.is_some_and(|prev| self.should_emit_trailing_comments(prev));
        let emit_trailing_comma = has_trailing_comma
            && format & LF_ALLOW_TRAILING_COMMA != 0
            && format & LF_COMMA_DELIMITED != 0;
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

        if format & LF_INDENTED != 0 {
            self.decrease_indent();
        }

        let closing_line_terminator_count = self.get_closing_line_terminator_count_node(
            parent_node,
            children.last().map(|c| &**c),
            format,
            children_text_range,
        );
        if closing_line_terminator_count > 0 {
            for _ in 0..closing_line_terminator_count {
                self.write_line();
            }
        } else if format & (LF_SPACE_AFTER_LIST | LF_SPACE_BETWEEN_BRACES) != 0 {
            self.write_space();
        }
    }
}

impl Printer {
    fn get_leading_line_terminator_count_node(
        &mut self,
        parent_node: Option<&Node>,
        first_child: Option<&Node>,
        format: ListFormat,
    ) -> i32 {
        if format & LF_PRESERVE_LINES != 0 || self.options.preserve_source_newlines {
            if format & LF_PREFER_NEW_LINE != 0 {
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
        if format & LF_MULTI_LINE != 0 {
            1
        } else {
            0
        }
    }

    fn get_separating_line_terminator_count_node(
        &mut self,
        previous_node: Option<&Node>,
        next_node: Option<&Node>,
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
                    && sibling_node_positions_are_comparable_node(previous_node, next_node)
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
                    && original_nodes_have_same_parent_node(previous_node, next_node)
                {
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

    fn get_closing_line_terminator_count_node(
        &mut self,
        parent_node: Option<&Node>,
        last_child: Option<&Node>,
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
                        .map_or(true, |p| std::ptr::eq(&*p, parent_node))
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
        if format & LF_MULTI_LINE != 0 && format & LF_NO_TRAILING_NEW_LINE == 0 {
            1
        } else {
            0
        }
    }
}

impl Printer {
    pub fn emit_statement(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::Block => self.emit_block(node),
            SyntaxKind::EmptyStatement => self.emit_empty_statement(node, false),
            SyntaxKind::VariableStatement => self.emit_variable_statement(node),
            SyntaxKind::WhileStatement => self.emit_while_statement(node),
            SyntaxKind::WithStatement => self.emit_with_statement(node),
            SyntaxKind::TryStatement => self.emit_try_statement(node),
            _ => panic!("emitStatement: statement kind not yet ported: {:?}", node.kind),
        }
    }

    pub fn emit_empty_statement(&mut self, node: &Node, is_embedded_statement: bool) {
        let state = self.enter_node(node);
        if is_embedded_statement {
            self.write_punctuation(";");
        } else {
            self.write_trailing_semicolon();
        }
        self.exit_node(node, state);
    }

    pub fn emit_block(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.generate_names(Some(node));
        self.emit_token(SyntaxKind::OpenBraceToken, node.pos(), WriteKind::Punctuation, node);
        let (multi_line, statements) = match &node.data {
            NodeData::Block(d) => (d.multi_line, &d.statements),
            _ => panic!("unexpected Block: {:?}", node.kind),
        };
        let format = if !multi_line && self.is_empty_block(node, &**statements)
            || self.should_emit_on_single_line(node)
        {
            LF_SINGLE_LINE_BLOCK_STATEMENTS
        } else {
            LF_MULTI_LINE_BLOCK_STATEMENTS
        };
        self.emit_list(Self::emit_statement, node, Some(&**statements), format);
        self.emit_token_ex(
            SyntaxKind::CloseBraceToken,
            statements.end(),
            WriteKind::Punctuation,
            node,
            if format & LF_MULTI_LINE != 0 {
                TEF_INDENT_LEADING_COMMENTS
            } else {
                TEF_NONE
            },
        );
        self.exit_node(node, state);
    }

    pub fn emit_embedded_statement(&mut self, parent_node: &Node, node: &Node) {
        if is_block(node)
            || self.should_emit_on_single_line(parent_node)
            || (self.options.preserve_source_newlines
                && self.get_leading_line_terminator_count_node(Some(parent_node), Some(node), LF_NONE) == 0)
        {
            self.write_space();
            self.emit_statement(node);
        } else {
            self.write_line();
            self.increase_indent();
            if node.kind == SyntaxKind::EmptyStatement {
                self.emit_empty_statement(node, true);
            } else {
                self.emit_statement(node);
            }
            self.decrease_indent();
        }
    }

    pub fn emit_catch_clause(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let open_paren_pos = self.emit_token(SyntaxKind::CatchKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        if let Some(variable_declaration) = node.k06_variable_declaration() {
            self.emit_token(SyntaxKind::OpenParenToken, open_paren_pos, WriteKind::Punctuation, node);
            self.emit_variable_declaration(variable_declaration);
            self.emit_token(
                SyntaxKind::CloseParenToken,
                variable_declaration.end(),
                WriteKind::Punctuation,
                node,
            );
            self.write_space();
        }
        self.emit_block(node.k06_block());
        self.exit_node(node, state);
    }
}

fn original_nodes_have_same_parent_node(node_a: &Node, node_b: &Node) -> bool {
    match (node_a.parent(), node_b.parent()) {
        (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
        _ => false,
    }
}

fn sibling_node_positions_are_comparable_node(previous_node: &Node, next_node: &Node) -> bool {
    if next_node.pos() < previous_node.end() {
        return false;
    }
    let Some(parent) = previous_node.parent() else {
        return false;
    };
    next_node
        .parent()
        .as_ref()
        .is_some_and(|p| Arc::ptr_eq(p, &parent))
}

