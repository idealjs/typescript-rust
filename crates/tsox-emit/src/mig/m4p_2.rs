#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node_source_file::ScriptKind;
use tsox_frontend::ast::mig::m3h::get_question_dot_token;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::mig::m3e_3::{
    get_expression_precedence, get_leftmost_expression, OperatorPrecedence,
    OPERATOR_PRECEDENCE_LOWEST,
};
use tsox_frontend::ast::node_data_generated::is_block;
use tsox_frontend::scanner;
use tsox_frontend::ast::utilities::{
    node_is_synthesized, skip_partially_emitted_expressions_arc as skip_partially_emitted_expressions,
};
use tsox_frontend::format::mig::m4t_4::is_immediately_invoked_function_expression_or_arrow_function;
use tsox_frontend::format::mig::m4t_2::{GetLiteralTextFlags, GET_LITERAL_TEXT_FLAGS_NONE};

use tsox_frontend::format::mig::m4o::ListFormat;
use tsox_frontend::format::mig::m4o_2::WriteKind;

use super::m4p::Printer;

const CALL_EXPRESSION_ARGUMENTS: ListFormat = ListFormat(
    ListFormat::COMMA_DELIMITED.0 | ListFormat::SPACE_BETWEEN_SIBLINGS.0 | ListFormat::PARENTHESIS.0,
);

fn if_statement_then_statement(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("if_statement_then_statement"); 
    match &node.data {
        NodeData::IfStatement(d) => &d.then_statement,
        _ => panic!("unexpected IfStatement: {:?}", node.kind),
    }
}

fn if_statement_else_statement(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("if_statement_else_statement"); 
    match &node.data {
        NodeData::IfStatement(d) => d.else_statement.as_ref(),
        _ => None,
    }
}

fn node_statement(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("node_statement"); 
    match &node.data {
        NodeData::DoStatement(d) => &d.statement,
        NodeData::ForStatement(d) => &d.statement,
        NodeData::ForInOrOfStatement(d) => &d.statement,
        NodeData::LabeledStatement(d) => &d.statement,
        _ => panic!("unexpected statement holder: {:?}", node.kind),
    }
}

fn node_initializer(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("node_initializer"); 
    match &node.data {
        NodeData::ForStatement(d) => d.initializer.as_ref(),
        NodeData::ForInOrOfStatement(d) => Some(&d.initializer),
        _ => None,
    }
}

fn for_statement_condition(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("for_statement_condition"); 
    match &node.data {
        NodeData::ForStatement(d) => d.condition.as_ref(),
        _ => None,
    }
}

fn for_statement_incrementor(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("for_statement_incrementor"); 
    match &node.data {
        NodeData::ForStatement(d) => d.incrementor.as_ref(),
        _ => None,
    }
}

fn for_in_or_of_await_modifier(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("for_in_or_of_await_modifier"); 
    match &node.data {
        NodeData::ForInOrOfStatement(d) => d.await_modifier.as_ref(),
        _ => None,
    }
}

fn labeled_statement_label(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("labeled_statement_label"); 
    match &node.data {
        NodeData::LabeledStatement(d) => &d.label,
        _ => panic!("unexpected LabeledStatement: {:?}", node.kind),
    }
}

fn call_expression_question_dot_token(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("call_expression_question_dot_token"); 
    match &node.data {
        NodeData::CallExpression(d) => d.question_dot_token.as_ref(),
        _ => None,
    }
}

fn node_arguments(node: &Arc<Node>) -> &Arc<tsox_frontend::ast::node::NodeList> { ::tsox_core::fntrace::enter("node_arguments"); 
    match &node.data {
        NodeData::CallExpression(d) => &d.arguments,
        _ => panic!("unexpected CallExpression: {:?}", node.kind),
    }
}

fn node_type_arguments(node: &Arc<Node>) -> Option<&Arc<tsox_frontend::ast::node::NodeList>> { ::tsox_core::fntrace::enter("node_type_arguments"); 
    match &node.data {
        NodeData::CallExpression(d) => d.type_arguments.as_ref(),
        NodeData::NewExpression(d) => d.type_arguments.as_ref(),
        NodeData::TaggedTemplateExpression(d) => d.type_arguments.as_ref(),
        _ => None,
    }
}

fn unary_expression_expression(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("unary_expression_expression"); 
    match &node.data {
        NodeData::DeleteExpression(d) => &d.expression,
        NodeData::TypeOfExpression(d) => &d.expression,
        NodeData::VoidExpression(d) => &d.expression,
        NodeData::AwaitExpression(d) => &d.expression,
        NodeData::SpreadElement(d) => &d.expression,
        NodeData::NonNullExpression(d) => &d.expression,
        NodeData::PrefixUnaryExpression(d) => &d.operand,
        NodeData::PostfixUnaryExpression(d) => &d.operand,
        _ => panic!("unexpected unary-like expression: {:?}", node.kind),
    }
}

impl Printer {
    pub fn emit_expression_no_asi(&mut self, node: &Arc<Node>, precedence: OperatorPrecedence) { ::tsox_core::fntrace::enter("emit_expression_no_asi"); 
        if self.parenthesize_expression_for_no_asi(node) {
            self.write_punctuation("(");
            self.emit_expression(node, precedence);
            self.write_punctuation(")");
        } else {
            self.emit_expression(node, precedence);
        }
    }

    pub fn emit_expression(&mut self, node: &Arc<Node>, precedence: OperatorPrecedence) { ::tsox_core::fntrace::enter("emit_expression"); 
        let parens =
            get_expression_precedence(&skip_partially_emitted_expressions(node)) < precedence;
        if parens {
            self.write_punctuation("(");
        }
        match node.kind {
            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword | SyntaxKind::NullKeyword => {
                self.emit_token_node(Some(node));
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword | SyntaxKind::ImportKeyword => {
                self.emit_keyword_expression(node);
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
            SyntaxKind::ExpressionWithTypeArguments => self.emit_expression_with_type_arguments(node),
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

    pub fn emit_empty_statement(&mut self, node: &Arc<Node>, is_embedded_statement: bool) { ::tsox_core::fntrace::enter("emit_empty_statement"); 
        let state = self.enter_node(node);
        if is_embedded_statement {
            self.write_punctuation(";");
        } else {
            self.write_trailing_semicolon();
        }
        self.exit_node(node, state);
    }

    pub fn emit_expression_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_expression_statement"); 
        let state = self.enter_node(node);
        let expression = node.expression().unwrap();
        if self.current_source_file.as_ref().is_some_and(|f| f.script_kind == ScriptKind::Json) {
            self.emit_expression(expression, OperatorPrecedence::Comma);
        } else if is_immediately_invoked_function_expression_or_arrow_function(expression) {
            self.emit_iife_with_parenthesized_callee(expression);
        } else {
            match get_leftmost_expression(expression, false).kind {
                SyntaxKind::FunctionExpression | SyntaxKind::ObjectLiteralExpression => {
                    self.emit_expression(expression, OperatorPrecedence::Parentheses);
                }
                _ => self.emit_expression(expression, OperatorPrecedence::Comma),
            }
        }
        if self.current_source_file.is_none()
            || self
                .current_source_file
                .as_ref()
                .is_some_and(|f| f.script_kind != ScriptKind::Json)
            || node_is_synthesized(expression)
        {
            self.write_trailing_semicolon();
        }
        self.exit_node(node, state);
    }

    pub fn emit_iife_with_parenthesized_callee(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_iife_with_parenthesized_callee"); 
        let call = skip_partially_emitted_expressions(node);
        let state = self.enter_node(&call);
        self.write_punctuation("(");
        self.emit_expression(call.expression().unwrap(), OPERATOR_PRECEDENCE_LOWEST);
        self.write_punctuation(")");
        self.emit_token_node(get_question_dot_token(&call));
        self.emit_type_arguments(&call, node_type_arguments(&call));
        self.emit_list(
            Self::emit_argument,
            &call,
            node_arguments(&call),
            CALL_EXPRESSION_ARGUMENTS,
        );
        self.exit_node(&call, state);
    }

    pub fn emit_if_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_if_statement"); 
        let state = self.enter_node(node);
        let expression = node.expression().unwrap();
        let then_statement = if_statement_then_statement(node);
        let mut pos = self.emit_token(SyntaxKind::IfKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        pos = self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(SyntaxKind::CloseParenToken, expression.end(), WriteKind::Punctuation, node);
        self.emit_embedded_statement(node, then_statement);
        if let Some(else_statement) = if_statement_else_statement(node) {
            self.write_line_or_space(node, then_statement, else_statement);
            self.emit_token(SyntaxKind::ElseKeyword, then_statement.end(), WriteKind::Keyword, node);
            if else_statement.kind == SyntaxKind::IfStatement {
                self.write_space();
                self.emit_if_statement(else_statement);
            } else {
                self.emit_embedded_statement(node, else_statement);
            }
        }
        self.exit_node(node, state);
    }

    pub fn emit_do_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_do_statement"); 
        let state = self.enter_node(node);
        let expression = node.expression().unwrap();
        let statement = node_statement(node);
        self.emit_token(SyntaxKind::DoKeyword, node.pos(), WriteKind::Keyword, node);
        self.emit_embedded_statement(node, statement);
        if is_block(statement) && !self.options.preserve_source_newlines {
            self.write_space();
        } else {
            self.write_line_or_space(node, statement, expression);
        }
        self.emit_while_clause(node, expression, statement.end());
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_for_initializer(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_for_initializer"); 
        if node.kind == SyntaxKind::VariableDeclarationList {
            self.emit_variable_declaration_list(node);
        } else {
            self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST);
        }
    }

    pub fn emit_for_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_for_statement"); 
        let state = self.enter_node(node);
        let mut pos = self.emit_token(SyntaxKind::ForKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        pos = self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        if let Some(initializer) = node_initializer(node) {
            self.emit_for_initializer(initializer);
            pos = initializer.end();
        }
        pos = self.emit_token(SyntaxKind::SemicolonToken, pos, WriteKind::Punctuation, node);
        if let Some(condition) = for_statement_condition(node) {
            self.write_space();
            self.emit_expression(condition, OPERATOR_PRECEDENCE_LOWEST);
            pos = condition.end();
        }
        pos = self.emit_token(SyntaxKind::SemicolonToken, pos, WriteKind::Punctuation, node);
        if let Some(incrementor) = for_statement_incrementor(node) {
            self.write_space();
            self.emit_expression(incrementor, OPERATOR_PRECEDENCE_LOWEST);
            pos = incrementor.end();
        }
        self.emit_token(SyntaxKind::CloseParenToken, pos, WriteKind::Punctuation, node);
        self.emit_embedded_statement(node, node_statement(node));
        self.exit_node(node, state);
    }

    pub fn emit_for_in_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_for_in_statement"); 
        let state = self.enter_node(node);
        let initializer = node_initializer(node).unwrap();
        let expression = node.expression().unwrap();
        let pos = self.emit_token(SyntaxKind::ForKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_token(SyntaxKind::OpenParenToken, pos, WriteKind::Punctuation, node);
        self.emit_for_initializer(initializer);
        self.write_space();
        self.emit_token(SyntaxKind::InKeyword, initializer.end(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(SyntaxKind::CloseParenToken, expression.end(), WriteKind::Punctuation, node);
        self.emit_embedded_statement(node, node_statement(node));
        self.exit_node(node, state);
    }

    pub fn emit_for_of_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_for_of_statement"); 
        let state = self.enter_node(node);
        let initializer = node_initializer(node).unwrap();
        let expression = node.expression().unwrap();
        let open_paren_pos =
            self.emit_token(SyntaxKind::ForKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        if let Some(await_modifier) = for_in_or_of_await_modifier(node) {
            self.emit_keyword_node(await_modifier);
            self.write_space();
        }
        self.emit_token(SyntaxKind::OpenParenToken, open_paren_pos, WriteKind::Punctuation, node);
        self.emit_for_initializer(initializer);
        self.write_space();
        self.emit_token(SyntaxKind::OfKeyword, initializer.end(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(expression, OPERATOR_PRECEDENCE_LOWEST);
        self.emit_token(SyntaxKind::CloseParenToken, expression.end(), WriteKind::Punctuation, node);
        self.emit_embedded_statement(node, node_statement(node));
        self.exit_node(node, state);
    }

    pub fn emit_labeled_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_labeled_statement"); 
        let state = self.enter_node(node);
        let label = labeled_statement_label(node);
        self.emit_label_identifier(label);
        self.emit_token(SyntaxKind::ColonToken, label.end(), WriteKind::Punctuation, node);
        self.write_space();
        self.emit_statement(node_statement(node));
        self.exit_node(node, state);
    }

    pub fn emit_argument(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_argument"); 
        self.emit_expression(node, OperatorPrecedence::Spread);
    }

    pub fn emit_numeric_literal(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_numeric_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_big_int_literal(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_big_int_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_string_literal(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_string_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_no_substitution_template_literal(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_no_substitution_template_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_regular_expression_literal(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_regular_expression_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GET_LITERAL_TEXT_FLAGS_NONE);
        self.exit_node(node, state);
    }

    pub fn emit_private_identifier(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_private_identifier"); 
        let state = self.enter_node(node);
        let text = node.text().to_string();
        self.write_as(&text, WriteKind::None);
        self.exit_node(node, state);
    }

    pub fn emit_omitted_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_omitted_expression"); 
        let state = self.enter_node(node);
        self.exit_node(node, state);
    }

    pub fn emit_spread_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_spread_element"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::DotDotDotToken, node.pos(), WriteKind::Punctuation, node);
        self.emit_expression(unary_expression_expression(node), OperatorPrecedence::Yield);
        self.exit_node(node, state);
    }

    pub fn emit_delete_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_delete_expression"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::DeleteKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(unary_expression_expression(node), OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub fn emit_type_of_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_type_of_expression"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::TypeOfKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(unary_expression_expression(node), OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub fn emit_void_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_void_expression"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::VoidKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(unary_expression_expression(node), OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub fn emit_await_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_await_expression"); 
        let state = self.enter_node(node);
        self.emit_token(SyntaxKind::AwaitKeyword, node.pos(), WriteKind::Keyword, node);
        self.write_space();
        self.emit_expression(unary_expression_expression(node), OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub fn emit_prefix_unary_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_prefix_unary_expression"); 
        let state = self.enter_node(node);
        let (operator, operand) = match &node.data {
            NodeData::PrefixUnaryExpression(d) => (d.operator, &d.operand),
            _ => panic!("unexpected PrefixUnaryExpression: {:?}", node.kind),
        };
        self.emit_token(operator, node.pos(), WriteKind::Operator, node);
        if operand.kind == SyntaxKind::PrefixUnaryExpression {
            let inner = match &operand.data {
                NodeData::PrefixUnaryExpression(d) => d.operator,
                _ => unreachable!(),
            };
            if (operator == SyntaxKind::PlusToken
                && (inner == SyntaxKind::PlusToken || inner == SyntaxKind::PlusPlusToken))
                || (operator == SyntaxKind::MinusToken
                    && (inner == SyntaxKind::MinusToken || inner == SyntaxKind::MinusMinusToken))
            {
                self.write_space();
            }
        }
        self.emit_expression(operand, OperatorPrecedence::Unary);
        self.exit_node(node, state);
    }

    pub fn emit_postfix_unary_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_postfix_unary_expression"); 
        let state = self.enter_node(node);
        let (operator, operand) = match &node.data {
            NodeData::PostfixUnaryExpression(d) => (d.operator, &d.operand),
            _ => panic!("unexpected PostfixUnaryExpression: {:?}", node.kind),
        };
        self.emit_expression(operand, OperatorPrecedence::LeftHandSide);
        self.emit_token(operator, operand.end(), WriteKind::Operator, node);
        self.exit_node(node, state);
    }

    pub fn emit_yield_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_yield_expression"); 
        let state = self.enter_node(node);
        let (asterisk_token, expression) = match &node.data {
            NodeData::YieldExpression(d) => (d.asterisk_token.as_ref(), d.expression.as_ref()),
            _ => panic!("unexpected YieldExpression: {:?}", node.kind),
        };
        self.emit_token(SyntaxKind::YieldKeyword, node.pos(), WriteKind::Keyword, node);
        if let Some(asterisk) = asterisk_token {
            self.emit_punctuation_node(asterisk);
        }
        if let Some(expression) = expression {
            self.write_space();
            self.emit_expression_no_asi(expression, OperatorPrecedence::Yield);
        }
        self.exit_node(node, state);
    }

    pub fn emit_as_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_as_expression"); 
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

    pub fn emit_satisfies_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_satisfies_expression"); 
        let state = self.enter_node(node);
        let (expression, type_node) = match &node.data {
            NodeData::SatisfiesExpression(d) => (&d.expression, &d.type_node),
            _ => panic!("unexpected SatisfiesExpression: {:?}", node.kind),
        };
        self.emit_expression(expression, OperatorPrecedence::Relational);
        self.write_space();
        self.write_keyword("satisfies");
        self.write_space();
        self.emit_type_node_outside_extends(type_node);
        self.exit_node(node, state);
    }

    pub fn emit_non_null_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_non_null_expression"); 
        let state = self.enter_node(node);
        self.emit_expression(unary_expression_expression(node), OperatorPrecedence::Member);
        self.write_as("!", WriteKind::Operator);
        self.exit_node(node, state);
    }

    pub fn emit_meta_property(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_meta_property"); 
        let state = self.enter_node(node);
        let (keyword_token, name) = match &node.data {
            NodeData::MetaProperty(d) => (d.keyword_token, &d.name),
            _ => panic!("unexpected MetaProperty: {:?}", node.kind),
        };
        self.emit_token(keyword_token, node.pos(), WriteKind::Punctuation, node);
        self.write_punctuation(".");
        self.emit_identifier_name(name);
        self.exit_node(node, state);
    }

    pub fn parenthesize_expression_for_no_asi(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("parenthesize_expression_for_no_asi"); 
        if self.comments_disabled {
            return false;
        }
        self.will_emit_leading_new_line(node)
    }

    fn will_emit_leading_new_line(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("will_emit_leading_new_line"); 
        use tsox_frontend::ast::node_data_generated::is_partially_emitted_expression;
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
        if has_leading_comment_ranges || has_new_line_comment {
            return true;
        }
        if is_partially_emitted_expression(node) {
            let pee = match &node.data {
                NodeData::PartiallyEmittedExpression(d) => d,
                _ => panic!("unexpected PartiallyEmittedExpression: {:?}", node.kind),
            };
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

    pub fn emit_arrow_function(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_arrow_function"); 
        let state = self.enter_node(node);
        let (modifiers, type_parameters, parameters, type_node, equals_greater_than_token, body) =
            match &node.data {
                NodeData::ArrowFunction(d) => (
                    d.modifiers.as_ref(),
                    d.type_parameters.as_ref(),
                    &d.parameters,
                    d.type_node.as_ref(),
                    &d.equals_greater_than_token,
                    &d.body,
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
        self.emit_concise_body(Some(body));
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_parameters_for_arrow(&mut self, parent_node: &Arc<Node>, parameters: &Arc<tsox_frontend::ast::node::NodeList>) { ::tsox_core::fntrace::enter("emit_parameters_for_arrow"); 
        self.emit_parameters(parent_node, parameters);
    }

    pub fn emit_concise_body(&mut self, body: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_concise_body"); 
        let Some(body) = body else { return };
        if is_block(body) {
            self.emit_function_body(body);
        } else {
            self.emit_expression(body, OperatorPrecedence::Yield);
        }
    }

    pub fn emit_binary_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_binary_expression"); 
        let (left, operator_token, right) = match &node.data {
            NodeData::BinaryExpression(d) => (&d.left, &d.operator_token, &d.right),
            _ => panic!("unexpected BinaryExpression: {:?}", node.kind),
        };
        let state = self.enter_node(node);
        self.emit_expression(left, OperatorPrecedence::Comma);
        self.write_as(" ", WriteKind::None);
        self.emit_token_node(Some(operator_token));
        self.write_space();
        self.emit_expression(right, OperatorPrecedence::Comma);
        self.exit_node(node, state);
    }

    pub fn emit_class_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_class_expression"); 
        let state = self.enter_node(node);
        let (name, modifiers, type_parameters, heritage_clauses, members) = match &node.data {
            NodeData::ClassExpression(d) => (
                d.name.as_ref(),
                d.modifiers.as_ref(),
                d.type_parameters.as_ref(),
                d.heritage_clauses.as_ref(),
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
                super::m4p::r39k22_defs::LF_HERITAGE_CLAUSE_TYPES,
            );
        }
        self.write_space();
        self.write_punctuation("{");
        self.push_name_generation_scope(node);
        self.generate_all_member_names13(members);
        self.emit_list(Self::emit_class_element, node, members, ListFormat(ListFormat::INDENTED.0 | ListFormat::MULTI_LINE.0));
        self.pop_name_generation_scope(node);
        self.write_punctuation("}");

        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    fn generate_all_member_names13(&mut self, members: &Arc<tsox_frontend::ast::node::NodeList>) { ::tsox_core::fntrace::enter("generate_all_member_names13"); 
        for member in &members.nodes {
            self.generate_name_if_needed(Some(member));
        }
    }

    pub fn emit_class_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_class_element"); 
        match node.kind {
            SyntaxKind::PropertyDeclaration => self.emit_property_declaration(node),
            SyntaxKind::MethodDeclaration => self.emit_method_declaration(node),
            SyntaxKind::GetAccessor => self.emit_get_accessor_declaration(node),
            SyntaxKind::SetAccessor => self.emit_set_accessor_declaration(node),
            SyntaxKind::IndexSignature => self.emit_index_signature(node),
            SyntaxKind::Constructor => self.emit_constructor(node),
            SyntaxKind::SemicolonClassElement => {
                let state = self.enter_node(node);
                self.write_trailing_semicolon();
                self.exit_node(node, state);
            }
            _ => panic!("unhandled ClassElement: {:?}", node.kind),
        }
    }

    pub fn emit_property_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_property_declaration"); 
        let state = self.enter_node(node);
        let (modifiers, name, postfix_token, type_node, initializer) = match &node.data {
            NodeData::PropertyDeclaration(d) => (
                d.modifiers.as_ref(),
                &d.name,
                d.postfix_token.as_ref(),
                d.type_node.as_ref(),
                d.initializer.as_ref(),
            ),
            _ => panic!("unexpected PropertyDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, true);
        self.emit_property_name(name);
        self.emit_token_node(postfix_token);
        self.emit_type_annotation(type_node);
        if let Some(initializer) = initializer {
            self.emit_initializer(Some(initializer), name.end(), node);
        }
        self.write_trailing_semicolon();
        self.exit_node(node, state);
    }

    pub fn emit_method_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_method_declaration"); 
        let state = self.enter_node(node);
        let (modifiers, postfix_token, name, body) = match &node.data {
            NodeData::MethodDeclaration(d) => (
                d.modifiers.as_ref(),
                d.postfix_token.as_ref(),
                &d.name,
                d.body.as_ref(),
            ),
            _ => panic!("unexpected MethodDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, true);
        self.emit_token_node(postfix_token);
        self.emit_property_name(name);
        let indented = self.should_emit_indented(node);
        self.increase_indent_if(indented);
        self.push_name_generation_scope(node);
        self.emit_signature(node);
        self.emit_function_body_node(body);
        self.pop_name_generation_scope(node);
        self.decrease_indent_if(indented);
        self.exit_node(node, state);
    }

    pub fn emit_set_accessor_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_set_accessor_declaration"); 
        self.emit_accessor_declaration(SyntaxKind::SetKeyword, node);
    }

    pub fn emit_constructor(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_constructor"); 
        let state = self.enter_node(node);
        let (modifiers, type_parameters, parameters, body) = match &node.data {
            NodeData::ConstructorDeclaration(d) => (
                d.modifiers.as_ref(),
                d.type_parameters.as_ref(),
                &d.parameters,
                d.body.as_ref(),
            ),
            _ => panic!("unexpected ConstructorDeclaration: {:?}", node.kind),
        };
        self.emit_modifier_list(node, modifiers, true);
        self.emit_token(SyntaxKind::ConstructorKeyword, node.pos(), WriteKind::Keyword, node);
        self.emit_type_parameters(node, type_parameters);
        self.emit_parameters(node, parameters);
        self.emit_function_body_node(body);
        self.exit_node(node, state);
    }
}
