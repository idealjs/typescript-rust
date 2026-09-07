#![allow(unused_imports)]

use super::*;

impl Parser {
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
}
