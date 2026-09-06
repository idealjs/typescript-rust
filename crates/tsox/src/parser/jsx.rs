use super::*;

impl Parser {
    pub(crate) fn parse_jsx_element_or_fragment(&mut self, in_expression_context: bool) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::LessThanToken);

        if self.token == SyntaxKind::GreaterThanToken {
            let opening = Arc::new(Node::with_loc(
                SyntaxKind::JsxOpeningFragment,
                NodeData::JsxOpeningFragment,
                TextRange::new(pos, self.token_end()),
            ));

            self.scan_jsx_text();
            let children = self.parse_jsx_children();
            let closing_pos = self.token_pos();
            self.expect(SyntaxKind::LessThanSlashToken);
            let closing_end = self.token_end();
            self.expect_without_advancing(SyntaxKind::GreaterThanToken);
            if in_expression_context {
                self.next_token();
            } else {
                self.scan_jsx_text();
            }
            let closing = Arc::new(Node::with_loc(
                SyntaxKind::JsxClosingFragment,
                NodeData::JsxClosingFragment,
                TextRange::new(closing_pos, closing_end),
            ));
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxFragment,
                NodeData::JsxFragment(JsxFragmentData {
                    opening_fragment: opening,
                    children,
                    closing_fragment: closing,
                }),
                TextRange::new(pos, closing_end),
            ));
        }

        let tag_name = self.parse_jsx_name();
        let attributes = self.parse_jsx_attributes();
        if self.parse_optional(SyntaxKind::SlashToken) {
            let end = self.token_end();
            self.expect_without_advancing(SyntaxKind::GreaterThanToken);

            if in_expression_context {
                self.next_token();
            } else {
                self.scan_jsx_text();
            }
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxSelfClosingElement,
                NodeData::JsxSelfClosingElement(JsxSelfClosingElementData {
                    tag_name,
                    type_arguments: None,
                    attributes,
                }),
                TextRange::new(pos, end),
            ));
        }

        let opening_end = self.token_end();
        self.expect_without_advancing(SyntaxKind::GreaterThanToken);

        self.scan_jsx_text();
        let opening = Arc::new(Node::with_loc(
            SyntaxKind::JsxOpeningElement,
            NodeData::JsxOpeningElement(JsxOpeningElementData {
                tag_name,
                type_arguments: None,
                attributes,
            }),
            TextRange::new(pos, opening_end),
        ));
        let children = self.parse_jsx_children();
        let closing = self.parse_jsx_closing_element(in_expression_context);
        Arc::new(Node::with_loc(
            SyntaxKind::JsxElement,
            NodeData::JsxElement(JsxElementData {
                opening_element: opening,
                children,
                closing_element: closing,
            }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    pub(crate) fn parse_jsx_name(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        self.scan_jsx_identifier();
        let mut name = self.parse_identifier_name_or_keyword();
        while self.parse_optional(SyntaxKind::DotToken) {
            self.scan_jsx_identifier();
            let right = self.parse_identifier_name_or_keyword();
            let end = right.end();
            name = Arc::new(Node::with_loc(
                SyntaxKind::PropertyAccessExpression,
                NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                    expression: name,
                    question_dot_token: None,
                    name: right,
                }),
                TextRange::new(pos, end),
            ));
        }
        name
    }

    pub(crate) fn parse_jsx_attributes(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let mut properties = Vec::new();
        while self.token != SyntaxKind::GreaterThanToken
            && self.token != SyntaxKind::SlashToken
            && self.token != SyntaxKind::EndOfFile
        {
            properties.push(self.parse_jsx_attribute());
        }
        Arc::new(Node::with_loc(
            SyntaxKind::JsxAttributes,
            NodeData::JsxAttributes(JsxAttributesData {
                properties: Arc::new(NodeList {
                    loc: TextRange::new(pos, self.token_pos()),
                    nodes: properties,
                }),
            }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    pub(crate) fn parse_jsx_attribute(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        if self.token == SyntaxKind::OpenBraceToken {

            self.next_token();
            self.expect(SyntaxKind::DotDotDotToken);
            let expression = self.parse_expression();
            self.expect(SyntaxKind::CloseBraceToken);
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxSpreadAttribute,
                NodeData::JsxSpreadAttribute(JsxSpreadAttributeData { expression }),
                TextRange::new(pos, self.token_pos()),
            ));
        }

        self.scan_jsx_identifier();
        let name = self.parse_identifier_name_or_keyword();
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            if self.token == SyntaxKind::StringLiteral {
                Some(self.parse_string_literal_node())
            } else if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_jsx_expression(true))
            } else if self.token == SyntaxKind::LessThanToken {
                Some(self.parse_jsx_element_or_fragment(true))
            } else {
                None
            }
        } else {
            None
        };
        Arc::new(Node::with_loc(
            SyntaxKind::JsxAttribute,
            NodeData::JsxAttribute(JsxAttributeData { name, initializer }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    pub(crate) fn parse_jsx_expression(&mut self, in_expression_context: bool) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let dot_dot_dot_token =
            if !in_expression_context && self.token == SyntaxKind::DotDotDotToken {
                self.parse_optional_token(SyntaxKind::DotDotDotToken)
            } else {
                None
            };
        let expression = if self.token == SyntaxKind::CloseBraceToken {
            None
        } else {
            Some(self.parse_expression())
        };
        if in_expression_context {
            self.expect(SyntaxKind::CloseBraceToken);
        } else {
            let end = self.token_end();
            self.expect_without_advancing(SyntaxKind::CloseBraceToken);
            self.scan_jsx_text();
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxExpression,
                NodeData::JsxExpression(JsxExpressionData {
                    dot_dot_dot_token,
                    expression,
                }),
                TextRange::new(pos, end),
            ));
        }
        Arc::new(Node::with_loc(
            SyntaxKind::JsxExpression,
            NodeData::JsxExpression(JsxExpressionData {
                dot_dot_dot_token,
                expression,
            }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    pub(crate) fn parse_jsx_children(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        let mut children = Vec::new();
        loop {
            match self.token {
                SyntaxKind::EndOfFile | SyntaxKind::LessThanSlashToken => break,
                SyntaxKind::JsxText | SyntaxKind::JsxTextAllWhiteSpaces => {
                    children.push(self.parse_jsx_text());
                }
                SyntaxKind::OpenBraceToken => {
                    children.push(self.parse_jsx_expression(false));
                }
                SyntaxKind::LessThanToken => {
                    children.push(self.parse_jsx_element_or_fragment(false));
                }
                _ => break,
            }
        }
        Arc::new(NodeList {
            loc: TextRange::new(pos, self.token_pos()),
            nodes: children,
        })
    }

    pub(crate) fn parse_jsx_text(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let text = self.scanner.token_text().to_string();
        let end = self.token_end();
        let is_all_whitespace = self.token == SyntaxKind::JsxTextAllWhiteSpaces;
        self.scan_jsx_text();
        Arc::new(Node::with_loc(
            SyntaxKind::JsxText,
            NodeData::JsxText(JsxTextData {
                text,
                contains_only_trivia_white_spaces: is_all_whitespace,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_jsx_closing_element(&mut self, in_expression_context: bool) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::LessThanSlashToken);
        let tag_name = self.parse_jsx_name();
        let end = self.token_end();
        self.expect_without_advancing(SyntaxKind::GreaterThanToken);

        if in_expression_context {
            self.next_token();
        } else {
            self.scan_jsx_text();
        }
        Arc::new(Node::with_loc(
            SyntaxKind::JsxClosingElement,
            NodeData::JsxClosingElement(JsxClosingElementData { tag_name }),
            TextRange::new(pos, end),
        ))
    }
}
