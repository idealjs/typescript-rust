#![allow(dead_code, unused_imports, unused_variables)]

#[path = "r36k21_defs.rs"]
pub mod r36k21_defs;

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use super::m4q::r33k12_defs::{OperatorPrecedence, TypePrecedence, is_keyword_kind, is_punctuation_kind};

use super::m4q::Printer;
use super::m4q::r33k12_defs::EmitFlags;
use crate::printer::GetLiteralTextFlags;
use super::m4q::r33k12_defs::{ListFlags, TokenEmitFlags, WriteKind, LF_TEMPLATE_EXPRESSION_SPANS};
use super::m4q_3::r36k15_defs::NodeDataExt15;
use self::r36k21_defs::NodeAsExt21;
use super::m4q::r39k08_defs::{
    EmitContextExtK08, NodeSpanExtK08, PrinterExtK08, TypedNode21ExtK08,
};
use tsox_frontend::ast::mig::m3e_3::OPERATOR_PRECEDENCE_LOWEST;

impl Printer {
    pub(crate) fn emit_template_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let template = node.as_template_expression();
        self.emit_template_head(&template.head().as_template_head());
        self.emit_list(
            Self::emit_template_span_node,
            template.as_node(),
            template.template_spans(),
            LF_TEMPLATE_EXPRESSION_SPANS,
        );
        self.exit_node(template.as_node(), state);
    }

    pub(crate) fn emit_template_head(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_template_literal(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                self.emit_no_substitution_template_literal(node)
            }
            SyntaxKind::TemplateExpression => self.emit_template_expression(node),
            _ => panic!("unhandled TemplateLiteral: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_template_middle(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_template_middle_tail(&mut self, node: &Node) {
        match node.kind {
            SyntaxKind::TemplateMiddle => self.emit_template_middle(&node.as_template_middle()),
            SyntaxKind::TemplateTail => self.emit_template_tail(&node.as_template_tail()),
            _ => {}
        }
    }

    pub(crate) fn emit_template_span(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let span = node.as_template_span();
        self.emit_expression(span.expression(), OperatorPrecedence::Comma);
        self.emit_template_middle_tail(span.literal());
        self.exit_node(span.as_node(), state);
    }

    pub(crate) fn emit_template_span_node(&mut self, node: &Node) {
        self.emit_template_span(node.as_template_span().as_node());
    }

    pub(crate) fn emit_template_tail(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_template_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let template = node.as_template_literal_type_node();
        self.emit_template_head(&template.head().as_template_head());
        self.emit_list(
            Self::emit_template_type_span_node,
            template.as_node(),
            template.template_spans(),
            LF_TEMPLATE_EXPRESSION_SPANS,
        );
        self.exit_node(template.as_node(), state);
    }

    pub(crate) fn emit_template_type_span(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let (span_type, literal) = match &node.data {
            NodeData::TemplateLiteralTypeSpan(d) => (&d.type_node, &d.literal),
            _ => panic!("unexpected TemplateLiteralTypeSpan: {:?}", node.kind),
        };
        self.emit_type_node_outside_extends(span_type);
        self.emit_template_middle_tail(literal);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_template_type_span_node(&mut self, node: &Node) {
        self.emit_template_type_span(&node.as_template_literal_type_span());
    }

    pub(crate) fn emit_tagged_template_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let tagged = node.as_tagged_template_expression();
        self.emit_callee(tagged.tag(), tagged.as_node());
        self.emit_type_arguments(tagged.as_node(), tagged.type_arguments());
        self.writer.write_space(" ");
        self.emit_template_literal(tagged.template());
        self.exit_node(tagged.as_node(), state);
    }

    pub(crate) fn emit_this_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.write_keyword("this");
        self.exit_node(node, state);
    }

    pub(crate) fn emit_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        write_kind: WriteKind,
        context_node: &Node,
    ) -> usize {
        self.emit_token_ex(token, pos, write_kind, context_node, TokenEmitFlags::NONE)
    }

    pub(crate) fn emit_token_ex(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        write_kind: WriteKind,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> usize {
        let (state, pos) = self.enter_token(token, pos, context_node, flags);
        let pos = self.write_token_text(token, write_kind, pos);
        self.exit_token(token, pos, context_node, state);
        pos
    }

    pub(crate) fn emit_token_node(&mut self, node: Option<&Node>) {
        self.emit_token_node_ex(node, TokenEmitFlags::NONE);
    }

    pub(crate) fn emit_token_node_ex(&mut self, node: Option<&Node>, flags: TokenEmitFlags) {
        let Some(node) = node else { return };
        if is_keyword_kind(node.kind) {
            self.emit_keyword_node_ex(node, flags);
        } else if is_punctuation_kind(node.kind) {
            self.emit_punctuation_node_ex(Some(node), flags);
        } else {
            panic!("unexpected TokenNode: {:?}", node.kind);
        }
    }

    pub(crate) fn emit_switch_statement(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let switch = node.as_switch_statement();
        let pos = self.emit_token(SyntaxKind::SwitchKeyword, switch.as_node().pos(), WriteKind::Keyword, switch.as_node());
        self.writer.write_space(" ");
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, switch.as_node());
        self.emit_expression(switch.expression(), OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(SyntaxKind::CloseParenToken, switch.expression().end(), WriteKind::Punctuation, switch.as_node());
        self.writer.write_space(" ");
        self.emit_case_block(&switch.case_block().as_case_block());
        self.exit_node(switch.as_node(), state);
    }

    pub(crate) fn emit_throw_statement(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let statement = node.as_throw_statement();
        self.emit_token(SyntaxKind::ThrowKeyword, statement.as_node().pos(), WriteKind::Keyword, statement.as_node());
        self.writer.write_space(" ");
        self.emit_expression_no_asi(statement.expression(), OPERATOR_PRECEDENCE_LOWEST);
        self.write_trailing_semicolon();
        self.exit_node(statement.as_node(), state);
    }

    pub(crate) fn emit_prologue_directives(&mut self, statements: &NodeList) -> usize {
        for (i, statement) in statements.nodes.iter().enumerate() {
            if is_prologue_directive(statement) {
                self.writer.write_line();
                self.emit_statement(statement);
            } else {
                return i;
            }
        }
        statements.nodes.len()
    }

    pub(crate) fn emit_property_access_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let access = node.as_property_access_expression();
        self.emit_expression(
            access.expression(),
            if is_optional_chain(access.as_node()) {
                OperatorPrecedence::OptionalChain
            } else {
                OperatorPrecedence::Member
            },
        );
        let has_question_dot_token =
            tsox_frontend::ast::mig::m3b::question_dot_token(access.as_node()).is_some();
        let mut token = tsox_frontend::ast::mig::m3b::question_dot_token(access.as_node()).cloned();
        if token.is_none() {
            let mut synthesized = self
                .emit_context
                .factory()
                .new_token(SyntaxKind::DotToken);
            if let Some(s) = Arc::get_mut(&mut synthesized) {
                s.loc = TextRange::new(
                    access.expression().end(),
                    property_access_name13(access.as_node()).pos(),
                );
            }
            self.emit_context.add_emit_flags(&synthesized, EmitFlags::NO_SOURCE_MAP);
            token = Some(synthesized);
        }
        let token = token.as_ref().unwrap();
        let lines_before_dot = self.get_lines_between_nodes(access.as_node(), access.expression(), token);
        self.write_line_repeat(lines_before_dot);
        self.increase_indent_if(lines_before_dot > 0);
        let should_emit_dot_dot = token.kind != SyntaxKind::QuestionDotToken
            && self.may_need_dot_dot_for_property_access(access.expression())
            && !self.writer.has_trailing_comment()
            && !self.writer.has_trailing_whitespace();
        if should_emit_dot_dot {
            self.write_punctuation(".");
        }
        if has_question_dot_token {
            self.emit_token_node(Some(token.as_ref()));
        } else {
            self.emit_token(SyntaxKind::DotToken, access.expression().end(), WriteKind::Punctuation, access.as_node());
        }
        let lines_after_dot = self.get_lines_between_nodes(
            access.as_node(),
            token,
            property_access_name13(access.as_node()),
        );
        self.write_line_repeat(lines_after_dot);
        self.increase_indent_if(lines_after_dot > 0);
        self.emit_member_name(Some(property_access_name13(access.as_node())));
        self.decrease_indent_if(lines_after_dot > 0);
        self.decrease_indent_if(lines_before_dot > 0);
        self.exit_node(access.as_node(), state);
    }
}

fn property_access_name13(node: &Node) -> &Node {
    match &node.data {
        NodeData::PropertyAccessExpression(d) => &d.name,
        _ => panic!("unexpected PropertyAccessExpression: {:?}", node.kind),
    }
}
