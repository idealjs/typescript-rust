#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_declaration_with_modifiers(
        &mut self,
        mut modifiers: Vec<(SyntaxKind, usize, usize)>,
    ) -> Arc<Node> {
        let mut decorators: Vec<Arc<Node>> = Vec::new();
        loop {
            if self.token == SyntaxKind::AtToken {
                decorators.push(self.parse_decorator());
                continue;
            }
            if !matches!(
                self.token,
                SyntaxKind::ExportKeyword
                    | SyntaxKind::DeclareKeyword
                    | SyntaxKind::DefaultKeyword
                    | SyntaxKind::AbstractKeyword
                    | SyntaxKind::AsyncKeyword
                    | SyntaxKind::ReadonlyKeyword
                    | SyntaxKind::PublicKeyword
                    | SyntaxKind::PrivateKeyword
                    | SyntaxKind::ProtectedKeyword
                    | SyntaxKind::StaticKeyword
                    | SyntaxKind::ConstKeyword
                    | SyntaxKind::AccessorKeyword
                    | SyntaxKind::OverrideKeyword
            ) {
                break;
            }

            let mut s = self.scanner.clone();
            s.scan();
            let can_follow = if self.token == SyntaxKind::ConstKeyword {
                s.token() == SyntaxKind::EnumKeyword
            } else {
                !s.has_preceding_line_break() && Self::token_can_follow_modifier(s.token())
            };
            if !can_follow {
                break;
            }
            let kind = self.token;
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            modifiers.push((kind, pos, end));
        }

        let modifiers = Some(if decorators.is_empty() {
            self.make_modifier_list(modifiers)
        } else {
            self.make_modifier_list_with_decorators(modifiers, decorators)
        });
        match self.token {
            SyntaxKind::FunctionKeyword => {
                self.parse_function_declaration_with_modifiers(modifiers)
            }
            SyntaxKind::ClassKeyword => self.parse_class_declaration_with_modifiers(modifiers),
            SyntaxKind::InterfaceKeyword => {
                self.parse_interface_declaration_with_modifiers(modifiers)
            }
            SyntaxKind::TypeKeyword => self.parse_type_alias_declaration_with_modifiers(modifiers),
            SyntaxKind::EnumKeyword => self.parse_enum_declaration_with_modifiers(modifiers),
            SyntaxKind::NamespaceKeyword | SyntaxKind::ModuleKeyword => {
                self.parse_namespace_declaration_with_modifiers(modifiers)
            }
            SyntaxKind::GlobalKeyword => self.parse_namespace_declaration_with_modifiers(modifiers),
            SyntaxKind::VarKeyword | SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword => {
                self.parse_variable_statement_with_modifiers(modifiers)
            }
            SyntaxKind::ImportKeyword => {
                self.parse_import_equals_declaration_with_modifiers(modifiers)
            }
            _ => self.parse_expression_statement(),
        }
    }

    pub(crate) fn parse_import_equals_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = self.parse_identifier();
        self.parse_import_equals_tail(pos, modifiers, name, false)
    }

    pub(crate) fn parse_function_declaration(&mut self) -> Arc<Node> {
        self.parse_function_declaration_with_modifiers(None)
    }

    pub(crate) fn parse_function_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let is_generator = asterisk_token.is_some();
        let is_async = modifiers
            .as_ref()
            .map(|m| m.flags().contains(ModifierFlags::Async))
            .unwrap_or(false);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_function_block(is_generator, is_async))
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionDeclaration,
            NodeData::FunctionDeclaration(FunctionDeclarationData {
                modifiers,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_function_block(&mut self, is_generator: bool, is_async: bool) -> Arc<Node> {
        let saved_yield = self.yield_context;
        let saved_await = self.await_context;
        self.yield_context = is_generator;
        self.await_context = is_async;
        let block = self.parse_block();
        self.yield_context = saved_yield;
        self.await_context = saved_await;
        block
    }

    pub(crate) fn parse_class_declaration(&mut self) -> Arc<Node> {
        self.parse_class_declaration_with_modifiers(None)
    }

    pub(crate) fn parse_class_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let heritage_clauses = self.parse_heritage_clauses();
        let members = self.parse_class_members();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ClassDeclaration,
            NodeData::ClassDeclaration(ClassDeclarationData {
                modifiers,
                name,
                type_parameters,
                heritage_clauses,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_interface_declaration(&mut self) -> Arc<Node> {
        self.parse_interface_declaration_with_modifiers(None)
    }

    pub(crate) fn parse_interface_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = self.parse_identifier();
        let type_parameters = self.parse_optional_type_parameters();
        let heritage_clauses = self.parse_heritage_clauses();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_list(ParsingContext::TypeMembers, Parser::parse_type_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::InterfaceDeclaration,
            NodeData::InterfaceDeclaration(InterfaceDeclarationData {
                modifiers,
                name,
                type_parameters,
                heritage_clauses,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_type_alias_declaration(&mut self) -> Arc<Node> {
        self.parse_type_alias_declaration_with_modifiers(None)
    }

    pub(crate) fn parse_type_alias_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = self.parse_identifier();
        let type_parameters = self.parse_optional_type_parameters();
        self.expect(SyntaxKind::EqualsToken);
        let type_node = self.parse_type();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeAliasDeclaration,
            NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
                modifiers,
                name,
                type_parameters,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_enum_declaration(&mut self) -> Arc<Node> {
        self.parse_enum_declaration_with_modifiers(None)
    }

    pub(crate) fn parse_enum_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = self.parse_identifier();
        self.expect(SyntaxKind::OpenBraceToken);
        let members =
            self.parse_delimited_list(ParsingContext::EnumMembers, Self::parse_enum_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::EnumDeclaration,
            NodeData::EnumDeclaration(EnumDeclarationData {
                modifiers,
                name,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_namespace_declaration(&mut self) -> Arc<Node> {
        self.parse_namespace_declaration_with_modifiers(None)
    }
}
