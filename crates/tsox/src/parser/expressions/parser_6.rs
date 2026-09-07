#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_primary_expression(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::Identifier => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData { text }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NumericLiteral => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::NumericLiteral,
                    NodeData::NumericLiteral(NumericLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::BigIntLiteral => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::BigIntLiteral,
                    NodeData::BigIntLiteral(BigIntLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::StringLiteral => {
                let text = self.scanner.token_value();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::StringLiteral,
                    NodeData::StringLiteral(StringLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                let text = self.scanner.token_value();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::NoSubstitutionTemplateLiteral,
                    NodeData::NoSubstitutionTemplateLiteral(NoSubstitutionTemplateLiteralData {
                        text,
                        template_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NullKeyword => self.parse_keyword_expression(SyntaxKind::NullKeyword),
            SyntaxKind::TrueKeyword => self.parse_keyword_expression(SyntaxKind::TrueKeyword),
            SyntaxKind::FalseKeyword => self.parse_keyword_expression(SyntaxKind::FalseKeyword),
            SyntaxKind::UndefinedKeyword => {
                self.parse_keyword_expression(SyntaxKind::UndefinedKeyword)
            }
            SyntaxKind::ThisKeyword => self.parse_keyword_expression(SyntaxKind::ThisKeyword),
            SyntaxKind::SuperKeyword => self.parse_keyword_expression(SyntaxKind::SuperKeyword),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_or_arrow(),
            SyntaxKind::OpenBracketToken => self.parse_array_literal(),
            SyntaxKind::OpenBraceToken => self.parse_object_literal(),
            SyntaxKind::LessThanToken if self.language_variant == LanguageVariant::Jsx => {
                self.parse_jsx_element_or_fragment(true)
            }
            SyntaxKind::FunctionKeyword => self.parse_function_expression(),
            SyntaxKind::ClassKeyword => self.parse_class_expression(),

            SyntaxKind::ImportKeyword => {
                if matches!(
                    self.look_ahead_token(),
                    SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                ) {
                    self.parse_keyword_expression(SyntaxKind::ImportKeyword)
                } else if self.is_import_meta() {
                    self.parse_import_meta()
                } else {
                    self.parse_fallback_identifier_or_error()
                }
            }

            SyntaxKind::AsyncKeyword if self.is_async_function_expression() => {
                self.parse_async_function_expression()
            }
            SyntaxKind::TemplateHead => self.parse_template_expression(),

            SyntaxKind::PrivateIdentifier => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::PrivateIdentifier,
                    NodeData::PrivateIdentifier(PrivateIdentifierData { text }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::SlashToken | SyntaxKind::SlashEqualsToken => {
                self.re_scan_slash_token();
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::RegularExpressionLiteral,
                    NodeData::RegularExpressionLiteral(RegularExpressionLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            _ => self.parse_fallback_identifier_or_error(),
        }
    }

    pub(crate) fn parse_fallback_identifier_or_error(&mut self) -> Arc<Node> {
        if is_identifier_or_keyword(self.token)
            && self.token != SyntaxKind::InKeyword
            && self.token != SyntaxKind::InstanceOfKeyword
        {
            let text = self.scanner.token_text().to_string();
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text }),
                TextRange::new(pos, end),
            ))
        } else {
            let pos = self.token_pos();
            let end = self.token_end();
            self.parse_error_at(pos, end, diagnostics::EXPRESSION_EXPECTED, &[]);
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(pos, pos),
            ))
        }
    }

    pub(crate) fn parse_keyword_expression(&mut self, kind: SyntaxKind) -> Arc<Node> {
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            kind,
            NodeData::Token,
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_parenthesized_or_arrow(&mut self) -> Arc<Node> {
        if self.is_parenthesized_arrow_function() {
            return self.parse_parenthesized_arrow_function();
        }

        let pos = self.token_pos();
        self.next_token();

        let expr = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();

        if self.token == SyntaxKind::EqualsGreaterThanToken {
            let arrow_token = self.create_token_node();
            self.next_token();
            let body = if self.token == SyntaxKind::OpenBraceToken {
                self.parse_block()
            } else {
                self.parse_assignment_expression()
            };
            let end = body.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::ArrowFunction,
                NodeData::ArrowFunction(ArrowFunctionData {
                    modifiers: None,
                    type_parameters: None,
                    parameters: Arc::new(NodeList::default()),
                    type_node: None,
                    equals_greater_than_token: arrow_token,
                    body,
                    full_signature: None,
                }),
                TextRange::new(pos, end),
            ));
        }

        Arc::new(Node::with_loc(
            SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedExpression(ParenthesizedExpressionData { expression: expr }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_array_literal(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBracketToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ArrayLiteralMembers,
            Parser::parse_array_literal_element,
        );
        self.expect(SyntaxKind::CloseBracketToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrayLiteralExpression,
            NodeData::ArrayLiteralExpression(ArrayLiteralExpressionData {
                elements: Arc::new(elements),
                multi_line: false,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_array_literal_element(&mut self) -> Arc<Node> {
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let pos = self.token_pos();
            let expression = self.parse_assignment_expression();
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }
        if self.token == SyntaxKind::CommaToken {
            let pos = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::OmittedExpression,
                NodeData::OmittedExpression,
                TextRange::new(pos, pos),
            ));
        }
        self.parse_assignment_expression()
    }

    pub(crate) fn parse_object_literal(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_delimited_list(
            ParsingContext::ObjectLiteralMembers,
            Parser::parse_object_literal_element,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ObjectLiteralExpression,
            NodeData::ObjectLiteralExpression(ObjectLiteralExpressionData {
                properties: Arc::new(members),
                multi_line: false,
            }),
            TextRange::new(pos, end),
        ))
    }
}
