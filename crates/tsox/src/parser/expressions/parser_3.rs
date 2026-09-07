#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn try_parse_generic_arrow_function(&mut self) -> Option<Arc<Node>> {
        let starts_with_async = self.token == SyntaxKind::AsyncKeyword;
        if !starts_with_async
            && (self.token != SyntaxKind::LessThanToken
                || self.language_variant == LanguageVariant::Jsx)
        {
            return None;
        }

        {
            let mut s = self.scanner.clone();
            if starts_with_async {
                let after = s.scan();
                if s.has_preceding_line_break() || after != SyntaxKind::LessThanToken {
                    return None;
                }
            }
            let t1 = s.scan();
            if !(t1 == SyntaxKind::Identifier
                || t1 == SyntaxKind::ConstKeyword
                || (t1 as i16) > (SyntaxKind::WithKeyword as i16))
            {
                return None;
            }
        }

        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;
        let diag_len = self.diagnostics.len();
        let pos = self.token_pos();

        if starts_with_async {
            self.next_token();
        }

        let type_parameters = self.parse_optional_type_parameters();

        if type_parameters.is_none()
            || self.token != SyntaxKind::OpenParenToken
            || self.diagnostics.len() != diag_len
        {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();

        if self.token != SyntaxKind::EqualsGreaterThanToken || self.diagnostics.len() != diag_len {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }

        let equals_greater_than_token = self.create_token_node();
        self.next_token();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Some(Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
                type_parameters,
                parameters,
                type_node,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        )))
    }

    pub(crate) fn parse_simple_arrow_function_with_async(
        &mut self,
        identifier: Arc<Node>,
        async_modifier: Arc<Node>,
    ) -> Arc<Node> {
        let modifiers = self.make_async_modifier_list(async_modifier);
        let pos = identifier.pos();
        let parameter = Arc::new(Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name: identifier,
                question_token: None,
                type_node: None,
                initializer: None,
            }),
            TextRange::new(pos, self.token_pos()),
        ));
        let parameters = Arc::new(NodeList {
            loc: TextRange::new(pos, self.token_pos()),
            nodes: vec![parameter],
        });
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let saved_await = self.await_context;
        self.await_context = true;
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        self.await_context = saved_await;
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers,
                type_parameters: None,
                parameters,
                type_node: None,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_parenthesized_arrow_function(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
                type_parameters: None,
                parameters,
                type_node,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_simple_arrow_function(&mut self, identifier: Arc<Node>) -> Arc<Node> {
        let pos = identifier.pos();
        let parameter = Arc::new(Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name: identifier,
                question_token: None,
                type_node: None,
                initializer: None,
            }),
            TextRange::new(pos, self.token_pos()),
        ));
        let parameters = Arc::new(NodeList {
            loc: TextRange::new(pos, self.token_pos()),
            nodes: vec![parameter],
        });
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
                type_parameters: None,
                parameters,
                type_node: None,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_binary_expression(&mut self, min_precedence: u8) -> Arc<Node> {
        let mut left = self.parse_unary_expression();

        loop {
            let precedence = binary_precedence(self.token);
            if precedence == 0 || precedence < min_precedence {
                break;
            }
            let operator_token = self.create_token_node();
            self.next_token();
            let right = self.parse_binary_expression(precedence + 1);
            let loc = TextRange::new(left.pos(), right.end());
            left = Arc::new(Node::with_loc(
                SyntaxKind::BinaryExpression,
                NodeData::BinaryExpression(BinaryExpressionData {
                    modifiers: None,
                    left,
                    type_node: None,
                    operator_token,
                    right,
                }),
                loc,
            ));
        }

        left
    }
}
