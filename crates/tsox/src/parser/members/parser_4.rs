#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_class_member(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        if self.token == SyntaxKind::SemicolonToken {
            let end = self.token_end();
            self.next_token();
            return Arc::new(Node::with_loc(
                SyntaxKind::SemicolonClassElement,
                NodeData::Token,
                TextRange::new(pos, end),
            ));
        }

        let mut decorators: Vec<Arc<Node>> = Vec::new();
        let mut modifiers: Vec<(SyntaxKind, usize, usize)> = Vec::new();
        loop {
            if self.token == SyntaxKind::AtToken {
                decorators.push(self.parse_decorator());
                continue;
            }
            if !matches!(
                self.token,
                SyntaxKind::PublicKeyword
                    | SyntaxKind::PrivateKeyword
                    | SyntaxKind::ProtectedKeyword
                    | SyntaxKind::StaticKeyword
                    | SyntaxKind::ReadonlyKeyword
                    | SyntaxKind::AbstractKeyword
                    | SyntaxKind::AsyncKeyword
                    | SyntaxKind::OverrideKeyword
                    | SyntaxKind::AccessorKeyword
                    | SyntaxKind::ConstKeyword
                    | SyntaxKind::ExportKeyword
                    | SyntaxKind::DefaultKeyword
                    | SyntaxKind::DeclareKeyword
            ) {
                break;
            }

            let mut s = self.scanner.clone();
            s.scan();
            if s.has_preceding_line_break() {
                break;
            }
            if self.token == SyntaxKind::StaticKeyword && s.token() == SyntaxKind::OpenBraceToken {
                break;
            }

            let can_follow = match self.token {
                SyntaxKind::ExportKeyword => {
                    let next = s.token();
                    match next {
                        SyntaxKind::DefaultKeyword => {
                            let t = s.scan();
                            Self::token_can_follow_default_keyword(t, &mut s)
                        }
                        SyntaxKind::TypeKeyword => {
                            let t = s.scan();
                            Self::can_follow_export_modifier(t)
                        }
                        _ => Self::can_follow_export_modifier(next),
                    }
                }
                SyntaxKind::DefaultKeyword => {
                    Self::token_can_follow_default_keyword(s.token(), &mut s)
                }
                _ => Self::token_can_follow_modifier(s.token()),
            };
            if !can_follow {
                break;
            }
            let kind = self.token;
            let mpos = self.token_pos();
            let mend = self.token_end();
            self.next_token();
            modifiers.push((kind, mpos, mend));
        }
        let modifiers = if decorators.is_empty() && modifiers.is_empty() {
            None
        } else if decorators.is_empty() {
            Some(self.make_modifier_list(modifiers))
        } else {
            Some(self.make_modifier_list_with_decorators(modifiers, decorators))
        };

        if self.token == SyntaxKind::StaticKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            if !s.has_preceding_line_break() && s.token() == SyntaxKind::OpenBraceToken {
                let pos = self.token_pos();
                self.next_token();
                let body = self.parse_block();
                let end = body.end();
                return Arc::new(Node::with_loc(
                    SyntaxKind::ClassStaticBlockDeclaration,
                    NodeData::ClassStaticBlockDeclaration(ClassStaticBlockDeclarationData {
                        modifiers: None,
                        body,
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }

        if self.is_index_signature_start() {
            return self.parse_index_signature(pos, modifiers);
        }

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            let next = s.token();
            let is_accessor = Self::token_can_follow_get_or_set(next);
            if is_accessor {
                let accessor_kind = self.token;
                return self.parse_accessor_declaration(pos, modifiers, accessor_kind);
            }
        }

        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = self.parse_property_name();
        let postfix_token = self
            .parse_optional_token(SyntaxKind::QuestionToken)
            .or_else(|| self.parse_optional_token(SyntaxKind::ExclamationToken));

        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || asterisk_token.is_some()
        {
            let is_constructor =
                name.kind == SyntaxKind::Identifier && name.text() == "constructor";
            let type_parameters = self.parse_optional_type_parameters();

            let prev_yield = self.yield_context;
            let prev_await = self.await_context;
            if asterisk_token.is_some() {
                self.yield_context = true;
            }

            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();
            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_block())
            } else {
                self.parse_semicolon();
                None
            };
            self.yield_context = prev_yield;
            self.await_context = prev_await;

            let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
            if is_constructor {
                return Arc::new(Node::with_loc(
                    SyntaxKind::Constructor,
                    NodeData::ConstructorDeclaration(ConstructorDeclarationData {
                        modifiers,
                        type_parameters,
                        parameters,
                        type_node,
                        full_signature: None,
                        body,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers,
                    asterisk_token,
                    name,
                    postfix_token,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                TextRange::new(pos, end),
            ));
        }

        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        self.parse_semicolon_after_property_name(&name, type_node.as_ref(), initializer.as_ref());
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(PropertyDeclarationData {
                modifiers,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_semicolon_after_property_name(
        &mut self,
        name: &Arc<Node>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) {
        if self.token == SyntaxKind::AtToken && !self.has_preceding_line_break() {
            self.parse_error_at_current_token(
                diagnostics::DECORATORS_MUST_PRECEDE_THE_NAME_AND_ALL_KEYWORDS_OF_PROPERTY_DECLARATIONS,
                &[],
            );
            return;
        }
        if self.token == SyntaxKind::OpenParenToken {
            self.parse_error_at_current_token(
                diagnostics::CANNOT_START_A_FUNCTION_CALL_IN_A_TYPE_ANNOTATION,
                &[],
            );
            self.next_token();
            return;
        }
        if type_node.is_some() && !self.can_parse_semicolon() {
            if initializer.is_some() {
                self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            } else {
                self.parse_error_at_current_token(
                    diagnostics::EXPECTED_FOR_PROPERTY_INITIALIZER,
                    &[],
                );
            }
            return;
        }
        if self.try_parse_semicolon() {
            return;
        }
        if initializer.is_some() {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            return;
        }
        self.parse_error_for_missing_semicolon_after(name);
    }
}
