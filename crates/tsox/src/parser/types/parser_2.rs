#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_this_type_predicate(&mut self, lhs: Arc<Node>) -> Arc<Node> {
        let pos = lhs.pos();
        self.expect(SyntaxKind::IsKeyword);
        let type_node = self.parse_type();
        let end = type_node.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypePredicate,
            NodeData::TypePredicateNode(TypePredicateNodeData {
                asserts_modifier: None,
                parameter_name: lhs,
                type_node: Some(type_node),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_asserts_type_predicate(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        let asserts_node = self.create_token_node();
        self.next_token();
        let parameter_name = self.parse_identifier();
        let mut type_node = None;
        if self.token == SyntaxKind::IsKeyword && !self.has_preceding_line_break() {
            self.next_token();
            type_node = Some(self.parse_type());
        }
        let end = type_node.as_ref().map_or(parameter_name.end(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::TypePredicate,
            NodeData::TypePredicateNode(TypePredicateNodeData {
                asserts_modifier: Some(asserts_node),
                parameter_name,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_infer_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::InferKeyword);
        let type_parameter = self.parse_type_parameter();
        let end = type_parameter.end();
        Arc::new(Node::with_loc(
            SyntaxKind::InferType,
            NodeData::InferTypeNode(InferTypeNodeData { type_parameter }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_type_query(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::TypeOfKeyword);
        let expr_name = self.parse_entity_name();

        let type_arguments = if !self.has_preceding_line_break() {
            self.parse_optional_type_arguments()
        } else {
            None
        };
        let end = type_arguments.as_ref().map_or(expr_name.end(), |a| a.end());
        Arc::new(Node::with_loc(
            SyntaxKind::TypeQuery,
            NodeData::TypeQueryNode(TypeQueryNodeData {
                expr_name,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_import_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let is_type_of = self.parse_optional(SyntaxKind::TypeOfKeyword);
        self.expect(SyntaxKind::ImportKeyword);
        self.expect(SyntaxKind::OpenParenToken);

        let argument = self.parse_type();
        let attributes = if self.parse_optional(SyntaxKind::CommaToken) {
            self.expect(SyntaxKind::OpenBraceToken);
            let token = self.token;
            if matches!(token, SyntaxKind::WithKeyword | SyntaxKind::AssertKeyword) {
                self.next_token();
            } else {
                let with_str = crate::scanner::token_to_string(SyntaxKind::WithKeyword).to_string();
                self.parse_error_at_current_token(
                    crate::diagnostics::messages_generated::X_0_EXPECTED,
                    &[&with_str],
                );
            }
            self.expect(SyntaxKind::ColonToken);
            let attrs = self.parse_import_attributes(token, true);
            self.parse_optional(SyntaxKind::CommaToken);
            self.expect(SyntaxKind::CloseBraceToken);
            Some(attrs)
        } else {
            None
        };
        self.expect(SyntaxKind::CloseParenToken);

        let qualifier = if self.parse_optional(SyntaxKind::DotToken) {
            Some(self.parse_entity_name())
        } else {
            None
        };
        let type_arguments = self.parse_optional_type_arguments();
        let end = type_arguments.as_ref().map_or_else(
            || {
                qualifier
                    .as_ref()
                    .map_or_else(|| argument.end(), |q| q.end())
            },
            |a: &Arc<NodeList>| a.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::ImportType,
            NodeData::ImportTypeNode(ImportTypeNodeData {
                is_type_of,
                argument,
                attributes,
                qualifier,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_template_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let head = self.create_template_token_node();
        self.next_token();
        let template_spans = self.parse_template_type_spans();
        let end = template_spans.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TemplateLiteralType,
            NodeData::TemplateLiteralTypeNode(TemplateLiteralTypeNodeData {
                head,
                template_spans,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_template_type_spans(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        let mut spans = Vec::new();
        loop {
            let span = self.parse_template_type_span();

            let is_middle = self.last_template_literal_was_middle;
            spans.push(span);
            if !is_middle {
                break;
            }
        }
        let end = spans.last().map_or(pos, |n| n.end());
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: spans,
        })
    }

    pub(crate) fn parse_template_type_span(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let type_node = self.parse_type();

        let literal = if self.token == SyntaxKind::CloseBraceToken {
            self.next_template_token();
            self.last_template_literal_was_middle = self.token == SyntaxKind::TemplateMiddle;
            let lit = self.create_template_token_node();
            self.next_token();
            lit
        } else {
            self.last_template_literal_was_middle = false;
            self.missing_node(self.token_pos())
        };
        let end = literal.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TemplateLiteralTypeSpan,
            NodeData::TemplateLiteralTypeSpan(TemplateLiteralTypeSpanData { type_node, literal }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_entity_name(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        match self.token {
            SyntaxKind::NullKeyword | SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword => {
                let text = self.scanner.token_text().to_string();
                let text_str = text.as_str();
                self.parse_error_at_current_token(
                    crate::diagnostics::messages_generated::
                        IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                    &[text_str],
                );
            }
            SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral | SyntaxKind::StringLiteral => {
                self.parse_error_at_current_token(
                    crate::diagnostics::messages_generated::IDENTIFIER_EXPECTED,
                    &[],
                );
            }
            _ => {}
        }
        let mut left = self.parse_identifier();
        while self.parse_optional(SyntaxKind::DotToken) {
            let right = self.parse_identifier();
            let end = right.end();
            left = Arc::new(Node::with_loc(
                SyntaxKind::QualifiedName,
                NodeData::QualifiedName(QualifiedNameData { left, right }),
                TextRange::new(pos, end),
            ));
        }
        left
    }

    pub(crate) fn next_is_start_of_mapped_type(&self) -> bool {
        let mut scanner = self.scanner.clone();

        let t1 = scanner.scan();

        if t1 == SyntaxKind::PlusToken || t1 == SyntaxKind::MinusToken {
            return scanner.scan() == SyntaxKind::ReadonlyKeyword;
        }

        let t2 = if t1 == SyntaxKind::ReadonlyKeyword {
            scanner.scan()
        } else {
            t1
        };

        if t2 != SyntaxKind::OpenBracketToken {
            return false;
        }
        let t3 = scanner.scan();
        if !is_identifier_or_keyword(t3) {
            return false;
        }
        scanner.scan() == SyntaxKind::InKeyword
    }
}
