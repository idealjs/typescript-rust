#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{NodeData, PropertyAccessExpressionData};
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::is_optional_chain;
use tsox_frontend::ast::mig::m3e_3::{get_expression_precedence, OperatorPrecedence};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::format::mig::m4o_2::WriteKind;
use crate::mig::m4r::{EmitTextWriter, Printer};
use crate::mig::m4q::r39k08_defs::EmitContextExtK08;

pub trait JsxNodeExt {
    fn opening_element(&self) -> &Arc<Node>;
    fn children(&self) -> &Arc<NodeList>;
    fn closing_element(&self) -> &Arc<Node>;
    fn opening_fragment(&self) -> &Arc<Node>;
    fn closing_fragment(&self) -> &Arc<Node>;
    fn tag_name(&self) -> &Arc<Node>;
    fn attributes(&self) -> &Arc<Node>;
    fn type_arguments(&self) -> Option<&Arc<NodeList>>;
    fn properties(&self) -> &Arc<NodeList>;
    fn namespace(&self) -> &Arc<Node>;
    fn jsx_attribute_initializer(&self) -> Option<&Arc<Node>>;
    fn dot_dot_dot_token(&self) -> Option<&Arc<Node>>;
    fn property_access_expression(&self) -> &PropertyAccessExpressionData;
}

impl JsxNodeExt for Node {
    fn opening_element(&self) -> &Arc<Node> {
        match &self.data {
            NodeData::JsxElement(d) => &d.opening_element,
            _ => panic!("openingElement on {:?}", self.kind),
        }
    }

    fn children(&self) -> &Arc<NodeList> {
        match &self.data {
            NodeData::JsxElement(d) => &d.children,
            NodeData::JsxFragment(d) => &d.children,
            _ => panic!("children on {:?}", self.kind),
        }
    }

    fn closing_element(&self) -> &Arc<Node> {
        match &self.data {
            NodeData::JsxElement(d) => &d.closing_element,
            _ => panic!("closingElement on {:?}", self.kind),
        }
    }

    fn opening_fragment(&self) -> &Arc<Node> {
        match &self.data {
            NodeData::JsxFragment(d) => &d.opening_fragment,
            _ => panic!("openingFragment on {:?}", self.kind),
        }
    }

    fn closing_fragment(&self) -> &Arc<Node> {
        match &self.data {
            NodeData::JsxFragment(d) => &d.closing_fragment,
            _ => panic!("closingFragment on {:?}", self.kind),
        }
    }

    fn tag_name(&self) -> &Arc<Node> {
        match &self.data {
            NodeData::JsxOpeningElement(d) => &d.tag_name,
            NodeData::JsxSelfClosingElement(d) => &d.tag_name,
            NodeData::JsxClosingElement(d) => &d.tag_name,
            _ => panic!("tagName on {:?}", self.kind),
        }
    }

    fn attributes(&self) -> &Arc<Node> {
        match &self.data {
            NodeData::JsxOpeningElement(d) => &d.attributes,
            NodeData::JsxSelfClosingElement(d) => &d.attributes,
            _ => panic!("attributes on {:?}", self.kind),
        }
    }

    fn type_arguments(&self) -> Option<&Arc<NodeList>> {
        match &self.data {
            NodeData::JsxOpeningElement(d) => d.type_arguments.as_ref(),
            NodeData::JsxSelfClosingElement(d) => d.type_arguments.as_ref(),
            _ => panic!("typeArguments on {:?}", self.kind),
        }
    }

    fn properties(&self) -> &Arc<NodeList> {
        match &self.data {
            NodeData::JsxAttributes(d) => &d.properties,
            _ => panic!("properties on {:?}", self.kind),
        }
    }

    fn namespace(&self) -> &Arc<Node> {
        match &self.data {
            NodeData::JsxNamespacedName(d) => &d.namespace,
            _ => panic!("namespace on {:?}", self.kind),
        }
    }

    fn jsx_attribute_initializer(&self) -> Option<&Arc<Node>> {
        match &self.data {
            NodeData::JsxAttribute(d) => d.initializer.as_ref(),
            _ => panic!("initializer on {:?}", self.kind),
        }
    }

    fn dot_dot_dot_token(&self) -> Option<&Arc<Node>> {
        match &self.data {
            NodeData::JsxExpression(d) => d.dot_dot_dot_token.as_ref(),
            _ => panic!("dotDotDotToken on {:?}", self.kind),
        }
    }

    fn property_access_expression(&self) -> &PropertyAccessExpressionData {
        match &self.data {
            NodeData::PropertyAccessExpression(d) => d,
            _ => panic!("PropertyAccessExpression on wrong node kind"),
        }
    }
}

impl Printer {
    pub fn emit_identifier_name(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let text = self.get_text_of_node(node, false);
        self.write(&text);
        self.exit_node(node, state);
    }

    pub fn emit_private_identifier(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let text = self.get_text_of_node(node, false);
        self.write(&text);
        self.exit_node(node, state);
    }

    pub fn decrease_indent_if(&mut self, indent_requested: bool) {
        if indent_requested {
            self.writer.decrease_indent();
        }
    }

    pub fn emit_expression(&mut self, expression: &Arc<Node>, precedence: OperatorPrecedence) {
        let parens = get_expression_precedence(expression) < precedence;
        if parens {
            self.write_punctuation("(");
        }
        match expression.kind {
            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword | SyntaxKind::NullKeyword => {
                self.emit_token_node(Some(expression));
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword | SyntaxKind::ImportKeyword => {
                self.emit_keyword_node(Some(expression));
            }
            SyntaxKind::Identifier => self.emit_identifier_name(expression),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(expression),
            SyntaxKind::PropertyAccessExpression => {
                self.emit_property_access_expression(expression);
            }
            SyntaxKind::YieldExpression => self.emit_yield_expression(expression),
            SyntaxKind::TypeAssertionExpression => {
                self.emit_type_assertion_expression(expression);
            }
            SyntaxKind::TypeOfExpression => self.emit_type_of_expression(expression),
            SyntaxKind::VoidExpression => self.emit_void_expression(expression),
            _ => panic!(
                "unhandled expression kind in emit_expression: {:?}",
                expression.kind
            ),
        }
        if parens {
            self.write_punctuation(")");
        }
    }

    pub fn emit_member_name(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(node),
            _ => panic!("unexpected MemberName: {:?}", node.kind),
        }
    }

    pub fn emit_property_access_expression(&mut self, node: &Arc<Node>) {
        let state = self.enter_node(node);
        let data = node.property_access_expression();
        let precedence = if is_optional_chain(node) {
            OperatorPrecedence::OptionalChain
        } else {
            OperatorPrecedence::Member
        };
        self.emit_expression(&data.expression, precedence);
        let token = match &data.question_dot_token {
            Some(token) => token.clone(),
            None => {
                let token = Arc::new(Node::with_loc(
                    SyntaxKind::DotToken,
                    NodeData::Token,
                    tsox_core::core::text::TextRange::new(
                        data.expression.end(),
                        data.name.pos(),
                    ),
                ));
                self.emit_context
                    .add_emit_flags(&token, EmitFlags::NO_SOURCE_MAP);
                token
            }
        };
        let lines_before_dot =
            self.get_lines_between_nodes(node, &data.expression, &token);
        self.write_line_repeat(lines_before_dot);
        self.increase_indent_if(lines_before_dot > 0);
        let should_emit_dot_dot = token.kind != SyntaxKind::QuestionDotToken
            && self.may_need_dot_dot_for_property_access(&data.expression)
            && !self.writer.has_trailing_comment()
            && !self.writer.has_trailing_whitespace();
        if should_emit_dot_dot {
            self.write_punctuation(".");
        }
        if data.question_dot_token.is_some() {
            self.emit_token_node(Some(&token));
        } else {
            self.emit_token(SyntaxKind::DotToken, data.expression.end(), WriteKind::Punctuation, node);
        }
        let lines_after_dot = self.get_lines_between_nodes(node, &token, &data.name);
        self.write_line_repeat(lines_after_dot);
        self.increase_indent_if(lines_after_dot > 0);
        self.emit_member_name(&data.name);
        self.decrease_indent_if(lines_after_dot > 0);
        self.decrease_indent_if(lines_before_dot > 0);
        self.exit_node(node, state);
    }
}
