use super::*;

impl Parser {
    pub(crate) fn parse_jsx_element_or_fragment(
        &mut self,
        in_expression_context: bool,
    ) -> Arc<Node> {
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
            // span 先记 </ 的结尾；> 匹配成功再延伸
            let slash_end = self.token_end();
            self.expect(SyntaxKind::LessThanSlashToken);
            let mut closing_end = slash_end;
            // Go parseJsxClosingFragment：缺失 > 时不推进（保留 Foo 供后续语句解析）
            if self.expect_without_advancing(SyntaxKind::GreaterThanToken) {
                closing_end = self.token_end();
                if in_expression_context {
                    self.next_token();
                } else {
                    self.scan_jsx_text();
                }
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
        // Go parseJsxOpeningOrSelfClosingElementOrOpeningFragment：TS 文件中
        // 标签名后为类型实参（<Component<T> .../>），JS 文件不解析
        let type_arguments = if !self.javascript_file && self.token == SyntaxKind::LessThanToken {
            Some(self.parse_jsx_type_arguments())
        } else {
            None
        };
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
                    type_arguments,
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
                type_arguments,
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

    pub(crate) fn parse_jsx_type_arguments(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::LessThanToken);
        let args = self.parse_delimited_list(ParsingContext::TypeArguments, Parser::parse_type);
        self.re_scan_greater_than();
        self.expect(SyntaxKind::GreaterThanToken);
        let end = self.node_pos();
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: args.nodes,
        })
    }

    pub(crate) fn parse_jsx_name(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        self.scan_jsx_identifier();
        let mut name = self.parse_identifier_name_or_keyword();
        // Go parseJsxTagName：冒号后为命名空间名（foo:bar），优先于点号限定
        if self.parse_optional(SyntaxKind::ColonToken) {
            self.scan_jsx_identifier();
            let right = self.parse_identifier_name_or_keyword();
            let end = right.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxNamespacedName,
                NodeData::JsxNamespacedName(JsxNamespacedNameData {
                    namespace: name,
                    name: right,
                }),
                TextRange::new(pos, end),
            ));
        }
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
        let list = self.parse_list(ParsingContext::JsxAttributes, Parser::parse_jsx_attribute);
        Arc::new(Node::with_loc(
            SyntaxKind::JsxAttributes,
            NodeData::JsxAttributes(JsxAttributesData {
                properties: Arc::new(list),
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
        // Go parseJsxAttributeName：属性名支持命名空间（prop:foo）
        let name = if self.parse_optional(SyntaxKind::ColonToken) {
            self.scan_jsx_identifier();
            let right = self.parse_identifier_name_or_keyword();
            let end = right.end();
            Arc::new(Node::with_loc(
                SyntaxKind::JsxNamespacedName,
                NodeData::JsxNamespacedName(JsxNamespacedNameData {
                    namespace: name,
                    name: right,
                }),
                TextRange::new(pos, end),
            ))
        } else {
            name
        };
        let initializer = if self.token == SyntaxKind::EqualsToken {
            // Go parseJsxAttributeValue：token 停在 = 时经 scanJsxAttributeValue
            // 取值（引号串跨行合法，普通扫描会在换行处截断；不能先 next_token
            // 预扫——那会把 { 表达式开括号吞进 jsx 语义扫描）
            if self.scan_jsx_attribute_value() == SyntaxKind::StringLiteral {
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
        // Go parseJsxTagName 对非标识符 token 产零宽 missing 名且不消费，
        // 由下方 expect `>` + 手动推进保证前进（本语境无列表循环，安全；
        // 共享 parse_jsx_name 的消费语义会把 `>` 吞成标识符并拉长 span）
        let tag_name = if self.token == SyntaxKind::Identifier
            || is_keyword(self.token) && !is_reserved_word_kind(self.token)
        {
            self.parse_jsx_name()
        } else {
            let p = self.token_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(p, p),
            ))
        };
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
