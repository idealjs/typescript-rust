#![allow(unused_imports)]

use super::*;

impl crate::parser::Parser {
    pub(crate) fn parse_jsdoc_type_expression(&mut self, may_omit_braces: bool) -> Arc<Node> {
        let pos = self.token_pos();
        let has_brace = if may_omit_braces {
            self.parse_optional(SyntaxKind::OpenBraceToken)
        } else {
            if self.token == SyntaxKind::OpenBraceToken {
                self.next_token();
                true
            } else {
                self.parse_error_at_current_token(
                    diagnostics::X_0_EXPECTED,
                    &[token_to_string(SyntaxKind::OpenBraceToken)],
                );
                false
            }
        };

        let t = self.parse_jsdoc_type();

        if has_brace {
            self.parse_expected_jsdoc(SyntaxKind::CloseBraceToken);
        }

        Arc::new(Node::with_loc(
            SyntaxKind::JSDocTypeExpression,
            NodeData::JSDocTypeExpression(JSDocTypeExpressionData { type_node: t }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    pub(crate) fn parse_jsdoc_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let t = match self.token {
            SyntaxKind::AsteriskToken => {
                self.next_token_jsdoc();
                Arc::new(Node::with_loc(
                    SyntaxKind::JSDocAllType,
                    NodeData::JSDocAllType,
                    TextRange::new(pos, pos + 1),
                ))
            }
            SyntaxKind::QuestionToken => {
                self.next_token_jsdoc();
                let inner = self.parse_type();
                let end = inner.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::JSDocNullableType,
                    NodeData::JSDocNullableType(JSDocNullableTypeData { type_node: inner }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::ExclamationToken => {
                self.next_token_jsdoc();
                let inner = self.parse_type();
                let end = inner.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::JSDocNonNullableType,
                    NodeData::JSDocNonNullableType(JSDocNonNullableTypeData { type_node: inner }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::DotDotDotToken => {
                self.next_token_jsdoc();
                let inner = self.parse_type();
                let end = inner.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::JSDocVariadicType,
                    NodeData::JSDocVariadicType(JSDocVariadicTypeData { type_node: inner }),
                    TextRange::new(pos, end),
                ))
            }
            _ => self.parse_type(),
        };

        if self.token == SyntaxKind::EqualsToken {
            self.next_token_jsdoc();
            let end = self.token_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::JSDocOptionalType,
                NodeData::JSDocOptionalType(JSDocOptionalTypeData { type_node: t }),
                TextRange::new(pos, end),
            ))
        } else {
            t
        }
    }

    pub(crate) fn try_parse_type_expression(&mut self) -> Option<Arc<Node>> {
        self.skip_whitespace_or_asterisk();
        if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_jsdoc_type_expression(false))
        } else {
            None
        }
    }

    pub(crate) fn parse_type_arguments_of_type_node(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        self.parse_expected_jsdoc(SyntaxKind::LessThanToken);
        let mut types: Vec<Arc<Node>> = Vec::new();
        loop {
            self.skip_whitespace();
            types.push(self.parse_jsdoc_type());
            self.skip_whitespace();
            if !self.parse_optional_jsdoc(SyntaxKind::CommaToken) {
                break;
            }
        }
        self.parse_expected_jsdoc(SyntaxKind::GreaterThanToken);
        let end = self.token_pos();
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: types,
        })
    }

    pub(crate) fn parse_expression_with_type_arguments_for_augments(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let has_brace = self.parse_optional_jsdoc(SyntaxKind::OpenBraceToken);
        let expression = self.parse_property_access_entity_name_expression();

        let saved_skip = self.scanner.skip_jsdoc_leading_asterisks_raw();
        self.scanner.set_skip_jsdoc_leading_asterisks(true);

        let type_arguments = if self.token == SyntaxKind::LessThanToken {
            Some(self.parse_type_arguments_of_type_node())
        } else {
            None
        };

        self.scanner
            .set_skip_jsdoc_leading_asterisks_raw(saved_skip);

        let end = if has_brace {
            self.skip_whitespace();
            self.parse_expected_jsdoc(SyntaxKind::CloseBraceToken);
            self.token_pos()
        } else {
            type_arguments
                .as_ref()
                .map(|ta| ta.end())
                .unwrap_or(expression.end())
        };

        Arc::new(Node::with_loc(
            SyntaxKind::ExpressionWithTypeArguments,
            NodeData::ExpressionWithTypeArguments(ExpressionWithTypeArgumentsData {
                expression,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_property_access_entity_name_expression(&mut self) -> Arc<Node> {
        let mut node = self.parse_jsdoc_identifier_name(None);
        while self.parse_optional_jsdoc(SyntaxKind::DotToken) {
            let name = self.parse_jsdoc_identifier_name(None);
            let end = name.end();
            let node_pos = node.pos();
            node = Arc::new(Node::with_loc(
                SyntaxKind::PropertyAccessExpression,
                NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                    expression: node,
                    question_dot_token: None,
                    name,
                }),
                TextRange::new(node_pos, end),
            ));
        }
        node
    }
}
