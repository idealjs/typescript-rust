#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn is_import_meta(&self) -> bool {
        if self.token != SyntaxKind::ImportKeyword {
            return false;
        }
        let mut scanner = self.scanner.clone();
        scanner.scan() == SyntaxKind::DotToken && !scanner.has_preceding_line_break()
    }

    pub(crate) fn parse_import_meta(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        self.next_token();
        let name = self.parse_identifier_name_or_keyword();
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::MetaProperty,
            NodeData::MetaProperty(MetaPropertyData {
                keyword_token: SyntaxKind::ImportKeyword,
                name,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn is_async_function_expression(&self) -> bool {
        if self.token != SyntaxKind::AsyncKeyword {
            return false;
        }
        let mut scanner = self.scanner.clone();
        scanner.scan() == SyntaxKind::FunctionKeyword && !scanner.has_preceding_line_break()
    }

    pub(crate) fn parse_async_function_expression(&mut self) -> Arc<Node> {
        let async_modifier = self.create_token_node();
        self.next_token();
        let pos = async_modifier.pos();

        self.next_token();
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let is_generator = asterisk_token.is_some();
        let body = self.parse_function_block(is_generator, true);
        let end = body.end();
        let modifiers = self.make_async_modifier_list(async_modifier);
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(FunctionExpressionData {
                modifiers,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_function_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = self.parse_block();
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(FunctionExpressionData {
                modifiers: None,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_class_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();

        let name = if self.is_identifier()
            && !matches!(
                self.token,
                SyntaxKind::ExtendsKeyword | SyntaxKind::ImplementsKeyword
            ) {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let heritage_clauses = self.parse_heritage_clauses();
        let members = self.parse_class_members();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ClassExpression,
            NodeData::ClassExpression(ClassExpressionData {
                modifiers: None,
                name,
                type_parameters,
                heritage_clauses,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub fn parse_expression(&mut self) -> Arc<Node> {
        let expr = self.parse_assignment_expression();

        if self.token == SyntaxKind::CommaToken {
            let pos = expr.pos();
            let mut left = expr;
            loop {
                let comma_pos = self.token_pos();
                let comma_end = self.token_end();
                if !self.parse_optional(SyntaxKind::CommaToken) {
                    break;
                }
                let right = self.parse_assignment_expression();
                let end = right.end();
                left = Arc::new(Node::with_loc(
                    SyntaxKind::BinaryExpression,
                    NodeData::BinaryExpression(BinaryExpressionData {
                        modifiers: None,
                        left,
                        type_node: None,
                        operator_token: Arc::new(Node::with_loc(
                            SyntaxKind::CommaToken,
                            NodeData::Token,
                            TextRange::new(comma_pos, comma_end),
                        )),
                        right,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            return left;
        }
        expr
    }

    pub(crate) fn parse_assignment_expression(&mut self) -> Arc<Node> {
        if self.is_yield_expression() {
            return self.parse_yield_expression();
        }

        if self.token == SyntaxKind::LessThanToken
            || (self.token == SyntaxKind::AsyncKeyword
                && self.look_ahead_token() == SyntaxKind::LessThanToken)
        {
            if let Some(arrow) = self.try_parse_generic_arrow_function() {
                return arrow;
            }
        }
        if self.token == SyntaxKind::AsyncKeyword && self.is_async_arrow_function() {
            let async_modifier = self.create_token_node();
            self.next_token();
            if self.token == SyntaxKind::OpenParenToken {
                return self.parse_parenthesized_arrow_function_with_async(async_modifier);
            }
            let identifier = self.parse_identifier();
            return self.parse_simple_arrow_function_with_async(identifier, async_modifier);
        }

        if self.token == SyntaxKind::OpenParenToken && self.is_parenthesized_arrow_function() {
            return self.parse_parenthesized_arrow_function();
        }

        let mut expr = self.parse_binary_expression(0);
        if expr.kind == SyntaxKind::Identifier && self.token == SyntaxKind::EqualsGreaterThanToken {
            return self.parse_simple_arrow_function(expr);
        }

        while self.token == SyntaxKind::AsKeyword || self.token == SyntaxKind::SatisfiesKeyword {
            let pos = expr.pos();
            let kind = self.token;
            self.next_token();

            let type_node = if kind == SyntaxKind::AsKeyword
                && self.token == SyntaxKind::ConstKeyword
                && !self.has_preceding_line_break()
            {
                let tp = self.token_pos();
                let te = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::ConstKeyword,
                    NodeData::KeywordTypeNode,
                    TextRange::new(tp, te),
                ))
            } else {
                self.parse_type()
            };
            let end = type_node.end();
            expr = match kind {
                SyntaxKind::AsKeyword => Arc::new(Node::with_loc(
                    SyntaxKind::AsExpression,
                    NodeData::AsExpression(AsExpressionData {
                        expression: expr,
                        type_node,
                    }),
                    TextRange::new(pos, end),
                )),
                _ => Arc::new(Node::with_loc(
                    SyntaxKind::SatisfiesExpression,
                    NodeData::SatisfiesExpression(SatisfiesExpressionData {
                        expression: expr,
                        type_node,
                    }),
                    TextRange::new(pos, end),
                )),
            };
        }

        if self.token == SyntaxKind::QuestionToken {
            let pos = expr.pos();
            let question_token = self.create_token_node();
            self.next_token();
            let when_true = self.parse_expression();
            let colon_token = self.create_token_node();
            self.expect(SyntaxKind::ColonToken);
            let when_false = self.parse_assignment_expression();
            let end = when_false.end();
            expr = Arc::new(Node::with_loc(
                SyntaxKind::ConditionalExpression,
                NodeData::ConditionalExpression(ConditionalExpressionData {
                    condition: expr,
                    question_token,
                    when_true,
                    colon_token,
                    when_false,
                }),
                TextRange::new(pos, end),
            ));
        }

        if is_assignment_operator(self.token) {
            let pos = expr.pos();
            let operator_token = self.create_token_node();
            self.next_token();
            let right = self.parse_assignment_expression();
            let end = right.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::BinaryExpression,
                NodeData::BinaryExpression(BinaryExpressionData {
                    modifiers: None,
                    left: expr,
                    type_node: None,
                    operator_token,
                    right,
                }),
                TextRange::new(pos, end),
            ));
        }
        expr
    }
}
