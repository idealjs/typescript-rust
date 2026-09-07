#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let mut type_node = self.parse_union_type_or_higher();
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            let extends_type = self.parse_type();
            self.expect(SyntaxKind::QuestionToken);
            let true_type = self.parse_type();
            self.expect(SyntaxKind::ColonToken);
            let false_type = self.parse_type();
            let end = false_type.end();
            type_node = Arc::new(Node::with_loc(
                SyntaxKind::ConditionalType,
                NodeData::ConditionalTypeNode(ConditionalTypeNodeData {
                    check_type: type_node,
                    extends_type,
                    true_type,
                    false_type,
                }),
                TextRange::new(pos, end),
            ));
        }
        type_node
    }

    pub(crate) fn parse_type_or_type_predicate(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::Identifier
            || self.token == SyntaxKind::ObjectKeyword
            || self.token == SyntaxKind::ThisKeyword
        {
            let mut scanner = self.scanner.clone();
            scanner.scan();
            if scanner.token() == SyntaxKind::IsKeyword && !scanner.has_preceding_line_break() {
                let pos = self.token_pos();
                let parameter_name = self.parse_identifier();
                self.expect(SyntaxKind::IsKeyword);
                let type_node = self.parse_type();
                let end = type_node.end();
                return Arc::new(Node::with_loc(
                    SyntaxKind::TypePredicate,
                    NodeData::TypePredicateNode(TypePredicateNodeData {
                        asserts_modifier: None,
                        parameter_name,
                        type_node: Some(type_node),
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }
        self.parse_type()
    }

    pub(crate) fn parse_union_type_or_higher(&mut self) -> Arc<Node> {
        self.parse_union_or_intersection_type(
            SyntaxKind::BarToken,
            SyntaxKind::UnionType,
            Parser::parse_intersection_type_or_higher,
        )
    }

    pub(crate) fn parse_intersection_type_or_higher(&mut self) -> Arc<Node> {
        self.parse_union_or_intersection_type(
            SyntaxKind::AmpersandToken,
            SyntaxKind::IntersectionType,
            Parser::parse_type_operator_or_higher,
        )
    }

    pub(crate) fn parse_union_or_intersection_type(
        &mut self,
        operator: SyntaxKind,
        node_kind: SyntaxKind,
        parse_constituent: fn(&mut Self) -> Arc<Node>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        let has_leading_operator = self.parse_optional(operator);
        let mut types = vec![parse_constituent(self)];
        while self.parse_optional(operator) {
            types.push(parse_constituent(self));
        }
        if types.len() == 1 && !has_leading_operator {
            return types.pop().unwrap();
        }
        let end = types.last().map_or(pos, |n| n.end());
        let list = Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: types,
        });
        let data = if node_kind == SyntaxKind::UnionType {
            NodeData::UnionTypeNode(UnionTypeNodeData { types: list })
        } else {
            NodeData::IntersectionTypeNode(IntersectionTypeNodeData { types: list })
        };
        Arc::new(Node::with_loc(node_kind, data, TextRange::new(pos, end)))
    }

    pub(crate) fn parse_type_operator_or_higher(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::KeyOfKeyword | SyntaxKind::UniqueKeyword | SyntaxKind::ReadonlyKeyword => {
                let pos = self.token_pos();
                let operator = self.token;
                self.next_token();
                let type_node = self.parse_type_operator_or_higher();
                let end = type_node.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::TypeOperator,
                    NodeData::TypeOperatorNode(TypeOperatorNodeData {
                        operator,
                        type_node,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            _ => self.parse_postfix_type_or_higher(),
        }
    }

    pub(crate) fn parse_postfix_type_or_higher(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let mut type_node = self.parse_non_array_type();
        loop {
            if self.token == SyntaxKind::OpenBracketToken {
                self.next_token();
                if self.token == SyntaxKind::CloseBracketToken {
                    self.next_token();
                    let end = self.token_pos();
                    type_node = Arc::new(Node::with_loc(
                        SyntaxKind::ArrayType,
                        NodeData::ArrayTypeNode(ArrayTypeNodeData {
                            element_type: type_node,
                        }),
                        TextRange::new(pos, end),
                    ));
                    continue;
                }
                let index_type = self.parse_type();
                self.expect(SyntaxKind::CloseBracketToken);
                let end = self.token_pos();
                type_node = Arc::new(Node::with_loc(
                    SyntaxKind::IndexedAccessType,
                    NodeData::IndexedAccessTypeNode(IndexedAccessTypeNodeData {
                        object_type: type_node,
                        index_type,
                    }),
                    TextRange::new(pos, end),
                ));
                continue;
            }
            break;
        }
        type_node
    }

    pub(crate) fn parse_non_array_type(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::ObjectKeyword => {
                if self.look_ahead_token() == SyntaxKind::DotToken {
                    return self.parse_type_reference();
                }
                let pos = self.token_pos();
                let end = self.token_end();
                let kind = self.token;
                self.next_token();
                Arc::new(Node::with_loc(
                    kind,
                    NodeData::KeywordTypeNode,
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::VoidKeyword => {
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::VoidKeyword,
                    NodeData::KeywordTypeNode,
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NullKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral => self.parse_literal_type_node(),
            SyntaxKind::MinusToken => self.parse_literal_type_node_with_negative(true),
            SyntaxKind::ThisKeyword => {
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                let this_keyword = Arc::new(Node::with_loc(
                    SyntaxKind::ThisType,
                    NodeData::ThisTypeNode,
                    TextRange::new(pos, end),
                ));

                if self.token == SyntaxKind::IsKeyword && !self.has_preceding_line_break() {
                    return self.parse_this_type_predicate(this_keyword);
                }
                this_keyword
            }
            SyntaxKind::TypeOfKeyword => {
                if self.look_ahead_token() == SyntaxKind::ImportKeyword {
                    self.parse_import_type()
                } else {
                    self.parse_type_query()
                }
            }
            SyntaxKind::ImportKeyword => self.parse_import_type(),
            SyntaxKind::AssertsKeyword => {
                if is_identifier_or_keyword(self.look_ahead_token()) {
                    self.parse_asserts_type_predicate()
                } else {
                    self.parse_type_reference()
                }
            }
            SyntaxKind::InferKeyword => self.parse_infer_type(),
            SyntaxKind::TemplateHead => self.parse_template_type(),
            SyntaxKind::OpenBraceToken => {
                if self.next_is_start_of_mapped_type() {
                    self.parse_mapped_type()
                } else {
                    self.parse_type_literal()
                }
            }
            SyntaxKind::OpenBracketToken => self.parse_tuple_type(),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_or_function_type(),
            SyntaxKind::LessThanToken => self.parse_function_type(),
            SyntaxKind::NewKeyword | SyntaxKind::AbstractKeyword => self.parse_constructor_type(),
            _ => self.parse_type_reference(),
        }
    }

    pub(crate) fn parse_literal_type_node(&mut self) -> Arc<Node> {
        self.parse_literal_type_node_with_negative(false)
    }

    pub(crate) fn parse_literal_type_node_with_negative(&mut self, negative: bool) -> Arc<Node> {
        let pos = self.token_pos();
        if negative {
            self.next_token();
        }
        let literal = match self.token {
            SyntaxKind::NullKeyword | SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword => {
                self.parse_keyword_expression(self.token)
            }
            _ => self.parse_primary_expression(),
        };
        let end = literal.end();
        Arc::new(Node::with_loc(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(LiteralTypeNodeData { literal }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_type_reference(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let type_name = self.parse_entity_name();
        let type_arguments = self.parse_optional_type_arguments();
        let end = type_arguments
            .as_ref()
            .map_or_else(|| type_name.end(), |args| args.end());
        Arc::new(Node::with_loc(
            SyntaxKind::TypeReference,
            NodeData::TypeReferenceNode(TypeReferenceNodeData {
                type_name,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }
}
