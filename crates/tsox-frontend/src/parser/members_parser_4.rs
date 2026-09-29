#![allow(unused_imports)]

use crate::parser::members::*;

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
        let mut has_static_modifier = false;
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
                    | SyntaxKind::InKeyword
                    | SyntaxKind::OutKeyword
            ) {
                break;
            }

            let mut s = self.scanner.clone();
            s.scan();
            // Go nextTokenCanFollowModifier：static 分支无同行要求
            if s.has_preceding_line_break() && self.token != SyntaxKind::StaticKeyword {
                break;
            }
            if self.token == SyntaxKind::StaticKeyword && s.token() == SyntaxKind::OpenBraceToken {
                break;
            }
            // Go tryParseModifier：已见 static 后第二个 static 不再作修饰符
            //（落为成员名，交由缺分号路径报 TS1434）
            if self.token == SyntaxKind::StaticKeyword && has_static_modifier {
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
            if kind == SyntaxKind::StaticKeyword {
                has_static_modifier = true;
            }
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
                // Go parseClassStaticBlockBody：静态块体 Yield 关、Await 开
                let prev_yield = self.yield_context;
                let prev_await = self.await_context;
                self.yield_context = false;
                self.await_context = true;
                let body = self.parse_block_ex(true);
                self.yield_context = prev_yield;
                self.await_context = prev_await;
                let end = body.end();
                return Arc::new(Node::with_loc(
                    SyntaxKind::ClassStaticBlockDeclaration,
                    NodeData::ClassStaticBlockDeclaration(ClassStaticBlockDeclarationData {
                        modifiers,
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

        if !(is_identifier_or_keyword(self.token)
            || matches!(
                self.token,
                SyntaxKind::StringLiteral
                    | SyntaxKind::NumericLiteral
                    | SyntaxKind::BigIntLiteral
                    | SyntaxKind::AsteriskToken
                    | SyntaxKind::OpenBracketToken
            )) && modifiers.is_some()
        {
            // Go parseClassElementWorker：modifiers 后成员无法开始，
            // 按 missing-name 属性声明恢复（TS1146 + TS1005 链）
            let name_pos = self.node_pos();
            self.parse_error_at(
                name_pos,
                name_pos,
                tsox_core::diagnostics::DECLARATION_EXPECTED,
                &[],
            );
            let name = Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(name_pos, name_pos),
            ));
            let postfix_token = if !self.has_preceding_line_break() {
                self.parse_optional_token(SyntaxKind::ExclamationToken)
            } else {
                None
            };
            let type_node = self.parse_optional_type_annotation();
            let initializer = if self.token == SyntaxKind::EqualsToken {
                self.next_token();
                let saved_yield = self.yield_context;
                let saved_await = self.await_context;
                self.yield_context = false;
                self.await_context = false;
                let init = self.parse_assignment_expression();
                self.yield_context = saved_yield;
                self.await_context = saved_await;
                Some(init)
            } else {
                None
            };
            self.parse_semicolon_after_property_name(&name, type_node.as_ref(), initializer.as_ref());
            let end = self.node_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::PropertyDeclaration,
                NodeData::PropertyDeclaration(PropertyDeclarationData {
                    modifiers,
                    name,
                    postfix_token,
                    type_node,
                    initializer,
                }),
                TextRange::new(pos, end),
            ));
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
            // Go parseMethodDeclaration：整个签名+体在 generator/async 上下文
            let method_is_async = modifiers.as_ref().is_some_and(|ml| {
                ml.list.nodes.iter().any(|m| m.kind == SyntaxKind::AsyncKeyword)
            });
            self.yield_context = asterisk_token.is_some();
            self.await_context = method_is_async;

            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();
            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_block_ex(true))
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
            // Go parsePropertyDeclaration：初始化器有隐式函数边界，yield/await 关
            let saved_yield = self.yield_context;
            let saved_await = self.await_context;
            self.yield_context = false;
            self.await_context = false;
            let init = self.parse_assignment_expression();
            self.yield_context = saved_yield;
            self.await_context = saved_await;
            Some(init)
        } else {
            None
        };
        self.parse_semicolon_after_property_name(&name, type_node.as_ref(), initializer.as_ref());
        let end = self.node_pos();
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
                tsox_core::diagnostics::DECORATORS_MUST_PRECEDE_THE_NAME_AND_ALL_KEYWORDS_OF_PROPERTY_DECLARATIONS,
                &[],
            );
            return;
        }
        if self.token == SyntaxKind::OpenParenToken {
            self.parse_error_at_current_token(
                tsox_core::diagnostics::CANNOT_START_A_FUNCTION_CALL_IN_A_TYPE_ANNOTATION,
                &[],
            );
            self.next_token();
            return;
        }
        if type_node.is_some() && !self.can_parse_semicolon() {
            if initializer.is_some() {
                self.parse_error_at_current_token(tsox_core::diagnostics::X_0_EXPECTED, &[";"]);
            } else {
                self.parse_error_at_current_token(
                    tsox_core::diagnostics::EXPECTED_FOR_PROPERTY_INITIALIZER,
                    &[],
                );
            }
            return;
        }
        if self.try_parse_semicolon() {
            return;
        }
        if initializer.is_some() {
            self.parse_error_at_current_token(tsox_core::diagnostics::X_0_EXPECTED, &[";"]);
            return;
        }
        self.parse_error_for_missing_semicolon_after(name);
    }
}
