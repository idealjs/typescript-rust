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
            SyntaxKind::MinusToken => {

                self.parse_literal_type_node_with_negative(true)
            }
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

                let with_str =
                    crate::scanner::token_to_string(SyntaxKind::WithKeyword).to_string();
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
                qualifier.as_ref().map_or_else(
                    || argument.end(),
                    |q| q.end(),
                )
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
            SyntaxKind::NullKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword => {
                let text = self.scanner.token_text().to_string();
                let text_str = text.as_str();
                self.parse_error_at_current_token(
                    crate::diagnostics::messages_generated::
                        IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                    &[text_str],
                );
            }
            SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral => {
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

    pub(crate) fn parse_mapped_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);

        let readonly_token = match self.token {
            SyntaxKind::ReadonlyKeyword | SyntaxKind::PlusToken | SyntaxKind::MinusToken => {
                let token = self.create_token_node();
                self.next_token();
                if token.kind != SyntaxKind::ReadonlyKeyword {
                    self.expect(SyntaxKind::ReadonlyKeyword);
                }
                Some(token)
            }
            _ => None,
        };

        self.expect(SyntaxKind::OpenBracketToken);
        let type_parameter = self.parse_mapped_type_parameter();
        let name_type = if self.parse_optional(SyntaxKind::AsKeyword) {
            Some(self.parse_type())
        } else {
            None
        };
        self.expect(SyntaxKind::CloseBracketToken);

        let question_token = match self.token {
            SyntaxKind::QuestionToken | SyntaxKind::PlusToken | SyntaxKind::MinusToken => {
                let token = self.create_token_node();
                self.next_token();
                if token.kind != SyntaxKind::QuestionToken {
                    self.expect(SyntaxKind::QuestionToken);
                }
                Some(token)
            }
            _ => None,
        };

        let type_node = self.parse_optional_type_annotation();
        self.parse_semicolon();
        let members = self.parse_list(ParsingContext::TypeMembers, Parser::parse_type_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();

        Arc::new(Node::with_loc(
            SyntaxKind::MappedType,
            NodeData::MappedTypeNode(MappedTypeNodeData {
                readonly_token,
                type_parameter,
                name_type,
                question_token,
                type_node,
                members: Some(Arc::new(members)),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_mapped_type_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let name = self.parse_identifier();
        self.expect(SyntaxKind::InKeyword);
        let constraint = self.parse_type();
        let end = constraint.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                modifiers: None,
                name,
                constraint: Some(constraint),
                expression: None,
                default_type: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_type_literal(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_list(ParsingContext::TypeMembers, Parser::parse_type_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeLiteral,
            NodeData::TypeLiteralNode(TypeLiteralNodeData {
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_tuple_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBracketToken);
        let elements = self.parse_delimited_list(
            ParsingContext::TupleElementTypes,
            Parser::parse_tuple_element_type,
        );
        self.expect(SyntaxKind::CloseBracketToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::TupleType,
            NodeData::TupleTypeNode(TupleTypeNodeData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_tuple_element_type(&mut self) -> Arc<Node> {

        if self.is_start_of_named_tuple_element() {
            return self.parse_named_tuple_member();
        }

        let pos = self.token_pos();
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let type_node = self.parse_type();
            let end = type_node.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::RestType,
                NodeData::RestTypeNode(RestTypeNodeData { type_node }),
                TextRange::new(pos, end),
            ));
        }
        let type_node = self.parse_type();

        if self.parse_optional(SyntaxKind::QuestionToken) {
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::OptionalType,
                NodeData::OptionalTypeNode(OptionalTypeNodeData { type_node }),
                TextRange::new(pos, end),
            ));
        }
        type_node
    }

    pub(crate) fn is_start_of_named_tuple_element(&self) -> bool {
        if self.token == SyntaxKind::DotDotDotToken {

            let next = self.look_ahead_token();
            if !is_identifier_or_keyword(next) {
                return false;
            }

            let after = self.look_ahead_2_tokens();
            return after == SyntaxKind::ColonToken
                || (after == SyntaxKind::QuestionToken
                    && self.look_ahead_3_tokens() == SyntaxKind::ColonToken);
        }
        if is_identifier_or_keyword(self.token) {
            let next = self.look_ahead_token();

            if next == SyntaxKind::ColonToken {
                return true;
            }

            if next == SyntaxKind::QuestionToken {
                return self.look_ahead_2_tokens() == SyntaxKind::ColonToken;
            }
        }
        false
    }

    pub(crate) fn parse_named_tuple_member(&mut self) -> Arc<Node> {

        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let name = self.parse_identifier();
        let question_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        self.expect(SyntaxKind::ColonToken);
        let type_node = self.parse_tuple_element_type();
        let end = type_node.end();
        Arc::new(Node::with_loc(
            SyntaxKind::NamedTupleMember,
            NodeData::NamedTupleMember(NamedTupleMemberData {
                dot_dot_dot_token,
                name,
                question_token,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_parenthesized_or_function_type(&mut self) -> Arc<Node> {

        let pos = self.token_pos();
        if self.is_start_of_function_type_with_open_paren() {

            let parameters = self.parse_parameter_list();
            self.expect(SyntaxKind::EqualsGreaterThanToken);
            let type_node = if self.is_start_of_type() {
                Some(self.parse_type_or_type_predicate())
            } else {
                None
            };
            let end = type_node.as_ref().map_or(self.token_pos(), |n| n.end());
            return Arc::new(Node::with_loc(
                SyntaxKind::FunctionType,
                NodeData::FunctionTypeNode(FunctionTypeNodeData {
                    type_parameters: None,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ));
        }

        self.expect(SyntaxKind::OpenParenToken);
        let type_node = self.parse_type();
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ParenthesizedType,
            NodeData::ParenthesizedTypeNode(ParenthesizedTypeNodeData { type_node }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn is_start_of_function_type_with_open_paren(&self) -> bool {

        let mut scanner = self.scanner.clone();
        let t1 = scanner.scan();

        if t1 == SyntaxKind::CloseParenToken || t1 == SyntaxKind::DotDotDotToken {
            return true;
        }

        let mut t = t1;
        while is_modifier_kind(t) {
            let mut probe = scanner.clone();
            let next = probe.scan();
            if probe.has_preceding_line_break() || !Self::token_can_follow_modifier(next) {
                break;
            }
            t = scanner.scan();
        }
        if t == SyntaxKind::DotDotDotToken {
            t = scanner.scan();
        }

        if t == SyntaxKind::OpenBraceToken || t == SyntaxKind::OpenBracketToken {
            let mut depth = 1usize;
            loop {
                match scanner.scan() {
                    SyntaxKind::OpenBraceToken
                    | SyntaxKind::OpenBracketToken
                    | SyntaxKind::OpenParenToken => depth += 1,
                    SyntaxKind::CloseBraceToken
                    | SyntaxKind::CloseBracketToken
                    | SyntaxKind::CloseParenToken => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            break;
                        }
                    }
                    SyntaxKind::EndOfFile => return false,
                    _ => {}
                }
            }
            let t2 = scanner.scan();
            return matches!(
                t2,
                SyntaxKind::ColonToken
                    | SyntaxKind::CommaToken
                    | SyntaxKind::QuestionToken
                    | SyntaxKind::EqualsToken
            ) || (t2 == SyntaxKind::CloseParenToken
                && scanner.scan() == SyntaxKind::EqualsGreaterThanToken);
        }

        if !is_identifier_or_keyword(t) {
            return false;
        }
        let t2 = scanner.scan();
        matches!(
            t2,
            SyntaxKind::ColonToken
                | SyntaxKind::CommaToken
                | SyntaxKind::QuestionToken
                | SyntaxKind::EqualsToken
        ) || (t2 == SyntaxKind::CloseParenToken
            && scanner.scan() == SyntaxKind::EqualsGreaterThanToken)
    }

    pub(crate) fn parse_function_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let type_node = if self.is_start_of_type() {
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        };
        let end = type_node.as_ref().map_or(self.token_pos(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(FunctionTypeNodeData {
                type_parameters,
                parameters,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_constructor_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let modifiers = if self.token == SyntaxKind::AbstractKeyword {
            let modifier_pos = self.token_pos();
            let modifier_end = self.token_end();
            self.next_token();
            Some(self.make_modifier_list(vec![(
                SyntaxKind::AbstractKeyword,
                modifier_pos,
                modifier_end,
            )]))
        } else {
            None
        };
        self.expect(SyntaxKind::NewKeyword);
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let type_node = if self.is_start_of_type() {
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        };
        let end = type_node.as_ref().map_or(self.token_pos(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ConstructorType,
            NodeData::ConstructorTypeNode(ConstructorTypeNodeData {
                modifiers,
                type_parameters,
                parameters,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }
}
