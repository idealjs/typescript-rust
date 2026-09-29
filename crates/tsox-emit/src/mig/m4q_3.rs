#![allow(dead_code, unused_imports, unused_variables)]

#[path = "r36k15_defs.rs"]
pub mod r36k15_defs;

#[path = "r37k6_defs.rs"]
pub mod r37k6_defs;

use tsox_frontend::ast::node::{Node, NodeList, SourceFile};
use tsox_frontend::scanner::TokenFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use super::m4q::r33k12_defs::{OperatorPrecedence, TypePrecedence, greatest_end, is_partially_emitted_expression, is_binary_operation, skip_partially_emitted_expressions, SnippetElement, SnippetKind, CommentSeparator};
use tsox_frontend::ast::mig::m3e_3::{
    OPERATOR_PRECEDENCE_COALESCE, OPERATOR_PRECEDENCE_DISALLOW_COMMA,
    OPERATOR_PRECEDENCE_LOWEST, TYPE_PRECEDENCE_HIGHEST, TYPE_PRECEDENCE_LOWEST,
};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::scanner;

use super::m4q::Printer;
use crate::mig::m4q_2::r39k07_defs::{greatest_end07, is_binary_operation07, skip_partially_emitted_expressions07};
use super::m4q::r39k08_defs::EmitContextExtK08;
use crate::mig::m4s::EmitTextWriter;
use super::m4q::r33k12_defs::EmitFlags;
use crate::printer::GetLiteralTextFlags;
use super::m4q::r33k12_defs::{ListFlags, PrinterState, TokenEmitFlags, WriteKind};

use self::r36k15_defs::NodeDataExt15;

impl Printer {
    pub(crate) fn emit_partially_emitted_expression(&mut self, node: &Node) {
        let mut node = node;
        let mut stack: Vec<(&Node, PrinterState)> = Vec::new();
        loop {
            let state = self.enter_node(node);
            let partially = node.as_partially_emitted_expression();
            let emit_flags = self.emit_context.emit_flags_of(node);
            if emit_flags & EmitFlags::NO_LEADING_COMMENTS.0 == 0 && node.pos() != partially.expression.pos()
            {
                self.emit_trailing_comments_of_position(partially.expression.pos(), false, false);
            }
            stack.push((node, state));
            if !is_partially_emitted_expression(&partially.expression) {
                break;
            }
            node = partially.expression.as_ref();
        }

        let partially = node.as_partially_emitted_expression();
        self.emit_expression(&partially.expression, OPERATOR_PRECEDENCE_LOWEST);

        while let Some((entry_node, entry_state)) = stack.pop() {
            let partially = entry_node.as_partially_emitted_expression();
            let emit_flags = self.emit_context.emit_flags_of(entry_node);
            if emit_flags & EmitFlags::NO_TRAILING_COMMENTS.0 == 0
                && node.end() != partially.expression.end()
            {
                self.emit_leading_comments_of_position(partially.expression.end());
            }
            self.exit_node(node, entry_state);
            node = entry_node;
        }
    }

    pub(crate) fn emit_postfix_unary_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let unary = node.as_postfix_unary_expression();
        self.emit_expression(&unary.operand, OperatorPrecedence::LeftHandSide);
        self.emit_token(unary.operator, unary.operand.end(), WriteKind::Operator, node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_prefix_unary_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let unary = node.as_prefix_unary_expression();
        let operator = unary.operator;
        let operand = &unary.operand;
        self.emit_token(operator, node.pos(), WriteKind::Operator, node);

        if operand.kind == SyntaxKind::PrefixUnaryExpression {
            let inner = operand.as_prefix_unary_expression().operator;
            if (operator == SyntaxKind::PlusToken && (inner == SyntaxKind::PlusToken || inner == SyntaxKind::PlusPlusToken))
                || (operator == SyntaxKind::MinusToken && (inner == SyntaxKind::MinusToken || inner == SyntaxKind::MinusMinusToken))
            {
                self.writer.write_space(" ");
            }
        }

        self.emit_expression(operand, OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_private_identifier(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let text = self.get_text_of_node(node, false);
        self.write(&text);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_property_assignment(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let assignment = node.as_property_assignment();
        self.emit_property_name(Some(&assignment.name));
        self.write_punctuation(":");
        self.writer.write_space(" ");
        let initializer = &assignment.initializer;
        if self.emit_context.emit_flags(initializer) & EmitFlags::NO_LEADING_COMMENTS.0 == 0 {
            let comment_range = self.emit_context.comment_range(initializer);
            self.emit_trailing_comments(comment_range.pos(), CommentSeparator::After);
        }
        self.emit_expression(initializer, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_property_declaration(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let declaration = node.as_property_declaration();
        self.emit_modifier_list(node, declaration.modifiers.as_deref(), true);
        self.emit_property_name(Some(&declaration.name));
        self.emit_token_node(declaration.postfix_token.as_deref());
        self.emit_type_annotation(declaration.type_node.as_deref());
        if let Some(initializer) = declaration.initializer.as_deref() {
            self.emit_initializer(
                Some(initializer),
                greatest_end07(declaration.name.end(), &[declaration.type_node.as_deref()]),
                node,
            );
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_property_name(&mut self, node: Option<&Node>) {
        let Some(node) = node else { return };
        let saved_write_kind = self.write_kind;
        self.write_kind = WriteKind::Property;
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
                self.emit_computed_property_name(node)
            }
            _ => panic!("unexpected PropertyName: {:?}", node.kind),
        }
        self.write_kind = saved_write_kind;
    }

    pub(crate) fn emit_property_signature(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let signature = node.as_property_signature_declaration();
        self.emit_modifier_list(node, signature.modifiers.as_deref(), false);
        self.emit_property_name(Some(&signature.name));
        self.emit_token_node(signature.postfix_token.as_deref());
        self.emit_type_annotation(Some(&signature.type_node));
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_punctuation_node(&mut self, node: Option<&Node>) {
        self.emit_punctuation_node_ex(node, TokenEmitFlags::NONE);
    }

    pub(crate) fn emit_punctuation_node_ex(&mut self, node: Option<&Node>, flags: TokenEmitFlags) {
        let Some(node) = node else { return };
        let state = self.enter_token_node(node, flags);
        self.write_token_text(node.kind, WriteKind::Punctuation, node.pos());
        self.exit_token_node(node, state);
    }

    pub(crate) fn emit_qualified_name(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let qualified = node.as_qualified_name();
        self.emit_entity_name(&qualified.left);
        self.write_punctuation(".");
        self.emit_member_name(Some(&qualified.right));
        self.exit_node(node, state);
    }

    pub(crate) fn emit_regular_expression_literal(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_rest_type(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let rest = node.as_rest_type_node();
        self.write_punctuation("...");
        self.emit_type_node_outside_extends(&rest.type_node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_return_statement(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let statement = node.as_return_statement();
        self.emit_token(SyntaxKind::ReturnKeyword, node.pos(), WriteKind::Keyword, node);
        if let Some(expression) = statement.expression.as_deref() {
            self.writer.write_space(" ");
            self.emit_expression_no_asi(expression, OPERATOR_PRECEDENCE_LOWEST);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_return_type(&mut self, node: Option<&Node>) {
        let Some(node) = node else { return };
        self.write_punctuation("=>");
        self.writer.write_space(" ");
        if self.in_extends
            && node.kind == SyntaxKind::InferType
            && node.as_infer_type_node()
                .type_parameter
                .as_type_parameter_declaration()
                .constraint
                .is_some()
        {
            self.emit_type_node_preserving_extends(node, TYPE_PRECEDENCE_HIGHEST);
        } else {
            self.emit_type_node_preserving_extends(node, TYPE_PRECEDENCE_LOWEST);
        }
    }

    pub(crate) fn emit_satisfies_expression(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let satisfies = node.as_satisfies_expression();
        self.emit_expression(&satisfies.expression, OperatorPrecedence::Relational);
        self.writer.write_space(" ");
        self.write_keyword("satisfies");
        self.writer.write_space(" ");
        self.emit_type_node_outside_extends(&satisfies.type_node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_semicolon_class_element(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub(crate) fn emit_set_accessor_declaration(&mut self, node: &Node) {
        self.emit_accessor_declaration(SyntaxKind::SetKeyword, node);
    }

    pub(crate) fn emit_shebang_if_needed(&mut self, node: &SourceFile) {
        if node_is_synthesized(&node.node) {
            return;
        }
        let shebang = scanner::get_shebang(&node.text);
        if !shebang.is_empty() {
            self.write_comment(&shebang);
            self.writer.write_line();
        }
    }

    pub(crate) fn emit_short_circuit_expression(&mut self, node: &Node) {
        if is_binary_operation07(skip_partially_emitted_expressions07(node), SyntaxKind::QuestionQuestionToken) {
            self.emit_expression(node, OPERATOR_PRECEDENCE_COALESCE);
        } else {
            self.emit_expression(node, OperatorPrecedence::LogicalOr);
        }
    }

    pub(crate) fn emit_shorthand_property_assignment(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let assignment = node.as_shorthand_property_assignment();
        self.emit_property_name(Some(&assignment.name));
        if let Some(initializer) = assignment.object_assignment_initializer.as_deref() {
            self.writer.write_space(" ");
            self.write_punctuation("=");
            self.writer.write_space(" ");
            self.emit_expression(initializer, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        }
        self.exit_node(node, state);
    }

    pub(crate) fn emit_signature(&mut self, node: &Node) {
        let (type_parameters, parameters, type_node) = match &node.data {
            NodeData::MethodDeclaration(d) => {
                (d.type_parameters.as_deref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::MethodSignatureDeclaration(d) => {
                (d.type_parameters.as_deref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::FunctionDeclaration(d) => {
                (d.type_parameters.as_deref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::FunctionExpression(d) => {
                (d.type_parameters.as_deref(), &d.parameters, d.type_node.as_deref())
            }
            NodeData::ArrowFunction(d) => {
                (d.type_parameters.as_deref(), &d.parameters, d.type_node.as_deref())
            }
            _ => panic!("unhandled signature node: {:?}", node.kind),
        };
        self.emit_type_parameters(node, type_parameters);
        self.emit_parameters(node, parameters);
        self.emit_type_annotation(type_node);
    }

    pub(crate) fn emit_snippet_node(&mut self, node: &Node, snippet_element: &SnippetElement) {
        if snippet_element.kind == SnippetKind::TabStop as u32 {
            self.emit_tab_stop(node, snippet_element);
        } else {
            panic!("Unhandled snippet element kind: {:?}", snippet_element.kind);
        }
    }

    pub(crate) fn emit_spread_assignment(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let assignment = node.as_spread_assignment();
        self.emit_token(SyntaxKind::DotDotDotToken, node.pos(), WriteKind::Punctuation, node);
        self.emit_expression(&assignment.expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_spread_element(&mut self, node: &Node) {
        let state = self.enter_node(node);
        let spread = node.as_spread_element();
        self.emit_token(SyntaxKind::DotDotDotToken, node.pos(), WriteKind::Punctuation, node);
        self.emit_expression(&spread.expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_string_literal(&mut self, node: &Node) {
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_tab_stop(&mut self, node: &Node, snippet_element: &SnippetElement) {
        debug_assert!(
            node.kind == SyntaxKind::EmptyStatement,
            "Snippet tab stops can only be emitted on empty statements"
        );
        self.writer.raw_write(&format!("${}", snippet_element.order));
    }
}
