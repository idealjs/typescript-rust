#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_object_literal_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        if dot_dot_dot_token.is_some() {
            let expression = self.parse_assignment_expression();
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            if Self::token_can_follow_get_or_set(s.token()) {
                let accessor_kind = self.token;
                self.next_token();
                let name = self.parse_property_name();
                let type_parameters = self.parse_optional_type_parameters();
                let parameters = self.parse_parameter_list();
                let type_node = self.parse_optional_return_type();

                let body = if self.token == SyntaxKind::OpenBraceToken {
                    Some(self.parse_block())
                } else {
                    self.parse_semicolon();
                    None
                };

                let end = body
                    .as_ref()
                    .map_or(self.scanner.full_start_pos(), |b| b.end());
                let range = TextRange::new(pos, end);
                return match accessor_kind {
                    SyntaxKind::GetKeyword => Arc::new(Node::with_loc(
                        SyntaxKind::GetAccessor,
                        NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                            modifiers: None,
                            name,
                            type_parameters,
                            parameters,
                            type_node,
                            full_signature: None,
                            body,
                        }),
                        range,
                    )),
                    _ => Arc::new(Node::with_loc(
                        SyntaxKind::SetAccessor,
                        NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                            modifiers: None,
                            name,
                            type_parameters,
                            parameters,
                            type_node,
                            full_signature: None,
                            body,
                        }),
                        range,
                    )),
                };
            }
        }

        let is_async = self.token == SyntaxKind::AsyncKeyword;
        if is_async {
            self.next_token();
        }

        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);

        let name = self.parse_property_name();
        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || asterisk_token.is_some()
            || is_async
        {
            let type_parameters = self.parse_optional_type_parameters();
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();

            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_block())
            } else {
                self.expect(SyntaxKind::OpenBraceToken);
                None
            };
            let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers: None,
                    asterisk_token,
                    name,
                    postfix_token: None,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            let initializer = self.parse_assignment_expression();
            let end = initializer.end();
            Arc::new(Node::with_loc(
                SyntaxKind::PropertyAssignment,
                NodeData::PropertyAssignment(PropertyAssignmentData {
                    modifiers: None,
                    name,
                    postfix_token: None,
                    type_node: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(end, end),
                    )),
                    initializer,
                }),
                TextRange::new(pos, end),
            ))
        } else {
            let end = name.end();
            Arc::new(Node::with_loc(
                SyntaxKind::ShorthandPropertyAssignment,
                NodeData::ShorthandPropertyAssignment(ShorthandPropertyAssignmentData {
                    modifiers: None,
                    name,
                    postfix_token: None,
                    type_node: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(end, end),
                    )),
                    equals_token: None,
                    object_assignment_initializer: None,
                }),
                TextRange::new(pos, end),
            ))
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_get_or_set_accessor(&self) -> bool {
        let mut scanner = self.scanner.clone();
        let next = scanner.scan();

        matches!(
            next,
            SyntaxKind::Identifier
                | SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::PrivateIdentifier
        )
    }

    #[allow(dead_code)]
    pub(crate) fn parse_object_accessor(&mut self, pos: usize, is_get: bool) -> Arc<Node> {
        self.next_token();
        let name = self.parse_property_name();
        let body = self.parse_block();
        let end = body.end();
        let kind = if is_get {
            SyntaxKind::GetAccessor
        } else {
            SyntaxKind::SetAccessor
        };
        let data = if is_get {
            NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: Arc::new(NodeList::default()),
                type_node: None,
                full_signature: None,
                body: Some(body),
            })
        } else {
            NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: Arc::new(NodeList::default()),
                type_node: None,
                full_signature: None,
                body: Some(body),
            })
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(pos, end)))
    }

    #[allow(dead_code)]
    pub(crate) fn parse_class_accessor(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        is_get: bool,
    ) -> Arc<Node> {
        self.next_token();
        let name = self.parse_property_name();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_block())
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
        let kind = if is_get {
            SyntaxKind::GetAccessor
        } else {
            SyntaxKind::SetAccessor
        };
        let data = if is_get {
            NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                modifiers,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            })
        } else {
            NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                modifiers,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            })
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(pos, end)))
    }
}
