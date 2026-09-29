#![allow(unused_imports)]

use crate::parser::expressions::*;

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

        let element_modifiers = self.parse_object_literal_element_modifiers();

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            if Self::token_can_follow_get_or_set(s.token()) {
                let accessor_kind = self.token;
                self.next_token();
                let name = self.parse_property_name();
                let saved_yield = self.yield_context;
                let saved_await = self.await_context;
                self.yield_context = false;
                self.await_context = false;
                let type_parameters = self.parse_optional_type_parameters();
                let parameters = self.parse_parameter_list();
                let type_node = self.parse_optional_return_type();

                let body = if self.token == SyntaxKind::OpenBraceToken {
                    Some(self.parse_block_ex(true))
                } else {
                    self.parse_semicolon();
                    None
                };
                self.yield_context = saved_yield;
                self.await_context = saved_await;

                let end = body
                    .as_ref()
                    .map_or(self.scanner.full_start_pos(), |b| b.end());
                let range = TextRange::new(pos, end);
                return match accessor_kind {
                    SyntaxKind::GetKeyword => Arc::new(Node::with_loc(
                        SyntaxKind::GetAccessor,
                        NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                            modifiers: element_modifiers,
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
                            modifiers: element_modifiers,
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

        let is_async = element_modifiers
            .as_ref()
            .is_some_and(|m| {
                m.flags()
                    .contains(crate::ast::node_flags::ModifierFlags::Async)
            })
            || self.token == SyntaxKind::AsyncKeyword;
        if self.token == SyntaxKind::AsyncKeyword {
            self.next_token();
        }

        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);

        let name_was_identifier = self.is_identifier();
        let name = self.parse_property_name();
        let obj_postfix_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let obj_postfix_token = match obj_postfix_token {
            Some(t) => Some(t),
            None => self.parse_optional_token(SyntaxKind::ExclamationToken),
        };
        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || asterisk_token.is_some()
            || is_async
        {
            let type_parameters = self.parse_optional_type_parameters();
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();

            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_function_block(asterisk_token.is_some(), is_async))
            } else {
                self.expect(SyntaxKind::OpenBraceToken);
                None
            };
            let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers: element_modifiers,
                    asterisk_token,
                    name,
                    postfix_token: obj_postfix_token,
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
            let initializer = self.allow_in(|p| p.parse_assignment_expression());
            let end = initializer.end();
            Arc::new(Node::with_loc(
                SyntaxKind::PropertyAssignment,
                NodeData::PropertyAssignment(PropertyAssignmentData {
                    modifiers: element_modifiers,
                    name,
                    postfix_token: obj_postfix_token,
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
            if !name_was_identifier {
                self.expect(SyntaxKind::ColonToken);
                let initializer = self.allow_in(|p| p.parse_assignment_expression());
                let end = initializer.end();
                return Arc::new(Node::with_loc(
                    SyntaxKind::PropertyAssignment,
                    NodeData::PropertyAssignment(PropertyAssignmentData {
                        modifiers: element_modifiers,
                        name,
                        postfix_token: obj_postfix_token,
                        type_node: Arc::new(Node::with_loc(
                            SyntaxKind::Unknown,
                            NodeData::Token,
                            TextRange::new(end, end),
                        )),
                        initializer,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            let equals_token = self.parse_optional_token(SyntaxKind::EqualsToken);
            let object_assignment_initializer = if equals_token.is_some() {
                Some(self.allow_in(|p| p.parse_assignment_expression()))
            } else {
                None
            };
            let end = object_assignment_initializer
                .as_ref()
                .map(|e| e.end())
                .unwrap_or_else(|| name.end());
            Arc::new(Node::with_loc(
                SyntaxKind::ShorthandPropertyAssignment,
                NodeData::ShorthandPropertyAssignment(ShorthandPropertyAssignmentData {
                    modifiers: element_modifiers,
                    name,
                    postfix_token: obj_postfix_token,
                    type_node: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(end, end),
                    )),
                    equals_token,
                    object_assignment_initializer,
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
        let body = self.parse_block_ex(true);
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
            Some(self.parse_block_ex(true))
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
