#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_unary_expression(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::ExclamationToken
            | SyntaxKind::TildeToken
            | SyntaxKind::PlusPlusToken
            | SyntaxKind::MinusMinusToken => {
                let operator = self.token;
                let op_pos = self.token_pos();
                self.next_token();
                let operand = self.parse_unary_expression();
                let loc = TextRange::new(op_pos, operand.end());
                Arc::new(Node::with_loc(
                    SyntaxKind::PrefixUnaryExpression,
                    NodeData::PrefixUnaryExpression(PrefixUnaryExpressionData {
                        operator,
                        operand,
                    }),
                    loc,
                ))
            }
            SyntaxKind::TypeOfKeyword => {
                let pos = self.token_pos();
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::TypeOfExpression,
                    NodeData::TypeOfExpression(TypeOfExpressionData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::VoidKeyword | SyntaxKind::DeleteKeyword => {
                let pos = self.token_pos();
                let is_delete = self.token == SyntaxKind::DeleteKeyword;
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                if is_delete {
                    Arc::new(Node::with_loc(
                        SyntaxKind::DeleteExpression,
                        NodeData::DeleteExpression(DeleteExpressionData { expression }),
                        TextRange::new(pos, end),
                    ))
                } else {
                    Arc::new(Node::with_loc(
                        SyntaxKind::VoidExpression,
                        NodeData::VoidExpression(VoidExpressionData { expression }),
                        TextRange::new(pos, end),
                    ))
                }
            }
            SyntaxKind::AwaitKeyword if self.is_await_expression() => {
                let pos = self.token_pos();
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::AwaitExpression,
                    NodeData::AwaitExpression(AwaitExpressionData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::LessThanToken if self.language_variant != LanguageVariant::Jsx => {
                self.parse_type_assertion()
            }
            _ => self.parse_postfix_expression(),
        }
    }

    pub(crate) fn parse_type_assertion(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let type_node = self.parse_type();
        self.expect(SyntaxKind::GreaterThanToken);
        let expression = self.parse_unary_expression();
        let end = expression.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeAssertionExpression,
            NodeData::TypeAssertion(TypeAssertionData {
                type_node,
                expression,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_postfix_expression(&mut self) -> Arc<Node> {
        let operand = self.parse_left_hand_side_expression();

        if !self.has_preceding_line_break()
            && (self.token == SyntaxKind::PlusPlusToken
                || self.token == SyntaxKind::MinusMinusToken)
        {
            let pos = operand.pos();
            let operator = self.token;
            let op_end = self.token_end();
            self.next_token();
            return Arc::new(Node::with_loc(
                SyntaxKind::PostfixUnaryExpression,
                NodeData::PostfixUnaryExpression(PostfixUnaryExpressionData { operand, operator }),
                TextRange::new(pos, op_end),
            ));
        }
        operand
    }

    pub(crate) fn parse_left_hand_side_expression(&mut self) -> Arc<Node> {
        let expr = if self.token == SyntaxKind::NewKeyword {
            self.parse_new_expression()
        } else {
            self.parse_primary_expression()
        };
        self.parse_call_and_member_chain(expr, false)
    }

    pub(crate) fn parse_member_chain(&mut self, expr: Arc<Node>) -> Arc<Node> {
        self.parse_call_and_member_chain(expr, true)
    }

    pub(crate) fn parse_new_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let expression = if self.token == SyntaxKind::DotToken {
            self.next_token();
            let name = self.parse_identifier();
            let end = name.end();
            Arc::new(Node::with_loc(
                SyntaxKind::PropertyAccessExpression,
                NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                    expression: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(pos, pos),
                    )),
                    question_dot_token: None,
                    name,
                }),
                TextRange::new(pos, end),
            ))
        } else {
            let primary = if self.token == SyntaxKind::NewKeyword {
                self.parse_new_expression()
            } else {
                self.parse_primary_expression()
            };
            self.parse_member_chain(primary)
        };
        let type_arguments = self.parse_optional_type_arguments();
        let arguments = if self.token == SyntaxKind::OpenParenToken {
            Some(self.parse_argument_list())
        } else {
            None
        };
        let end = arguments.as_ref().map_or(expression.end(), |a| a.end());
        Arc::new(Node::with_loc(
            SyntaxKind::NewExpression,
            NodeData::NewExpression(NewExpressionData {
                expression,
                type_arguments,
                arguments,
            }),
            TextRange::new(pos, end),
        ))
    }
}
