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
            SyntaxKind::GlobalKeyword => {

                self.parse_namespace_declaration_with_modifiers(modifiers)
            }
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

    pub(crate) fn parse_namespace_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {

        let pos = self.token_pos();
        let keyword = self.token;

        if self.token == SyntaxKind::GlobalKeyword {
            return self.parse_ambient_external_module_declaration(pos, modifiers);
        }

        self.next_token();

        if self.token == SyntaxKind::StringLiteral {
            return self.parse_ambient_external_module_declaration(pos, modifiers);
        }

        let mut segments: Vec<Arc<Node>> = vec![self.parse_identifier()];
        while self.token == SyntaxKind::DotToken {
            self.next_token();
            segments.push(self.parse_identifier());
        }
        let body = if self.token == SyntaxKind::OpenBraceToken {
            let body_pos = self.token_pos();
            self.next_token();
            let statements =
                self.parse_list(ParsingContext::BlockStatements, Parser::parse_statement);
            self.expect(SyntaxKind::CloseBraceToken);
            let end = self.token_pos();
            Some(Arc::new(Node::with_loc(
                SyntaxKind::ModuleBlock,
                NodeData::ModuleBlock(ModuleBlockData {
                    statements: Arc::new(statements),
                }),
                TextRange::new(body_pos, end),
            )))
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());

        let mut name = segments.pop().expect("at least one segment");
        let mut inner_body = body;

        let user_modifiers = modifiers;
        let mut mods = if segments.is_empty() {
            user_modifiers.clone()
        } else {
            None
        };
        let outermost = segments.is_empty();

        let export_only: Option<Arc<ModifierList>> = if segments.is_empty() {
            None
        } else {
            let export_tok = Arc::new(Node::with_loc(
                SyntaxKind::ExportKeyword,
                NodeData::Token,
                TextRange::new(pos, pos + 6),
            ));
            Some(Arc::new(ModifierList::new(
                vec![export_tok],
                ModifierFlags::Export,
            )))
        };
        loop {
            let decl = Arc::new(Node::with_loc(
                SyntaxKind::ModuleDeclaration,
                NodeData::ModuleDeclaration(ModuleDeclarationData {
                    modifiers: mods.clone().or_else(|| export_only.clone()),
                    keyword,
                    name: Arc::clone(&name),
                    body: inner_body,
                }),
                TextRange::new(pos, end),
            ));
            match segments.pop() {
                Some(seg) => {
                    name = seg;
                    inner_body = Some(decl);
                    mods = None;
                }
                None => {

                    if !outermost {
                        let decl_mut = Arc::as_ptr(&decl) as *mut Node;
                        unsafe {
                            if let NodeData::ModuleDeclaration(d) = &mut (*decl_mut).data {
                                d.modifiers = user_modifiers.clone();
                            }
                        }
                    }
                    return decl;
                }
            }
        }
    }

    pub(crate) fn parse_ambient_external_module_declaration(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {

        let keyword = self.token;
        let name = if self.token == SyntaxKind::GlobalKeyword {

            self.parse_identifier()
        } else {

            self.parse_string_literal_name()
        };
        let body = if self.token == SyntaxKind::OpenBraceToken {
            let body_pos = self.token_pos();
            self.next_token();
            let statements =
                self.parse_list(ParsingContext::BlockStatements, Parser::parse_statement);
            self.expect(SyntaxKind::CloseBraceToken);
            let end = self.token_pos();
            Some(Arc::new(Node::with_loc(
                SyntaxKind::ModuleBlock,
                NodeData::ModuleBlock(ModuleBlockData {
                    statements: Arc::new(statements),
                }),
                TextRange::new(body_pos, end),
            )))
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ModuleDeclarationData {
                modifiers,
                keyword,
                name,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_string_literal_name(&mut self) -> Arc<Node> {
        let text = self.scanner.token_text().to_string();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text,
                token_flags: 0,
            }),
            TextRange::new(pos, end),
        ))
    }

    #[allow(dead_code)]
    pub(crate) fn parse_namespace_name(&mut self) -> Arc<Node> {
        let name = self.parse_identifier();

        if self.token == SyntaxKind::DotToken {
            let pos = name.pos();
            let mut left = name;
            while self.token == SyntaxKind::DotToken {
                self.next_token();
                let right = self.parse_identifier();
                let end = right.end();
                left = Arc::new(Node::with_loc(
                    SyntaxKind::QualifiedName,
                    NodeData::QualifiedName(QualifiedNameData { left, right }),
                    TextRange::new(pos, end),
                ));
            }
            return left;
        }
        name
    }

    pub(crate) fn parse_import_declaration(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();

        let after_import_pos = self.token_pos();
        let mut identifier = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };

        let mut phase_modifier = None;
        if let Some(id) = identifier.as_ref() {
            if id.text() == "type"
                && (self.token != SyntaxKind::FromKeyword
                    || (self.is_identifier()
                        && matches!(
                            self.look_ahead_token(),
                            SyntaxKind::FromKeyword | SyntaxKind::EqualsToken
                        )))
                && (self.is_identifier()
                    || self.token_after_import_definitely_produces_import_declaration())
            {
                phase_modifier = Some(SyntaxKind::TypeKeyword);
                identifier = if self.is_identifier() {
                    Some(self.parse_identifier())
                } else {
                    None
                };
            } else if id.text() == "defer" {
                let should_parse_as_defer_modifier = if self.token == SyntaxKind::FromKeyword {
                    self.look_ahead_token() != SyntaxKind::StringLiteral
                } else {
                    self.token != SyntaxKind::CommaToken && self.token != SyntaxKind::EqualsToken
                };
                if should_parse_as_defer_modifier {
                    phase_modifier = Some(SyntaxKind::DeferKeyword);
                    identifier = if self.is_identifier() {
                        Some(self.parse_identifier())
                    } else {
                        None
                    };
                }
            }
        }

        if let Some(id) = identifier.as_ref() {
            if !self.token_after_imported_identifier_definitely_produces_import_declaration()
                && phase_modifier != Some(SyntaxKind::DeferKeyword)
            {
                let is_type_only = phase_modifier == Some(SyntaxKind::TypeKeyword);
                return self.parse_import_equals_declaration(pos, id.clone(), is_type_only);
            }
        }

        let import_clause =
            self.try_parse_import_clause(identifier, after_import_pos, phase_modifier);
        let module_specifier = self.parse_module_specifier();
        let attributes = self.try_parse_import_attributes();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportDeclaration,
            NodeData::ImportDeclaration(ImportDeclarationData {
                modifiers: None,
                import_clause,
                module_specifier,
                attributes,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn token_after_import_definitely_produces_import_declaration(&self) -> bool {
        self.token == SyntaxKind::AsteriskToken || self.token == SyntaxKind::OpenBraceToken
    }

    pub(crate) fn token_after_imported_identifier_definitely_produces_import_declaration(&self) -> bool {
        self.token == SyntaxKind::CommaToken || self.token == SyntaxKind::FromKeyword
    }

    pub(crate) fn parse_import_equals_declaration(
        &mut self,
        pos: usize,
        name: Arc<Node>,
        is_type_only: bool,
    ) -> Arc<Node> {
        self.parse_import_equals_tail(pos, None, name, is_type_only)
    }

    pub(crate) fn parse_import_equals_tail(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        is_type_only: bool,
    ) -> Arc<Node> {
        self.expect(SyntaxKind::EqualsToken);
        let module_reference = self.parse_module_reference();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportEqualsDeclaration,
            NodeData::ImportEqualsDeclaration(ImportEqualsDeclarationData {
                modifiers,
                is_type_only,
                name,
                module_reference,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_module_reference(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::RequireKeyword
            && self.look_ahead_token() == SyntaxKind::OpenParenToken
        {
            return self.parse_external_module_reference();
        }
        self.parse_entity_name()
    }

    pub(crate) fn parse_external_module_reference(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::RequireKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_module_specifier();
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ExternalModuleReference,
            NodeData::ExternalModuleReference(ExternalModuleReferenceData { expression }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_module_specifier(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::StringLiteral {
            return self.parse_string_literal_node();
        }
        self.parse_expression()
    }

    pub(crate) fn try_parse_import_attributes(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::WithKeyword
            || (self.token == SyntaxKind::AssertKeyword && !self.has_preceding_line_break())
        {

            let mut probe = self.scanner.clone();
            probe.scan();
            if probe.token() != SyntaxKind::OpenBraceToken {
                self.next_token();
                self.parse_error_at_current_token(
                    crate::diagnostics::X_0_EXPECTED,
                    &["{"],
                );
                return None;
            }
            Some(self.parse_import_attributes(self.token, false))
        } else {
            None
        }
    }

    pub(crate) fn parse_import_attributes(&mut self, token: SyntaxKind, skip_keyword: bool) -> Arc<Node> {
        let pos = self.token_pos();
        if !skip_keyword {
            self.next_token();
        }
        self.expect(SyntaxKind::OpenBraceToken);
        let multi_line = self.has_preceding_line_break();
        let attributes = self.parse_delimited_list(
            ParsingContext::ImportAttributes,
            Parser::parse_import_attribute,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportAttributes,
            NodeData::ImportAttributes(ImportAttributesData {
                token,
                attributes: Arc::new(attributes),
                multi_line,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_import_attribute(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let name = if is_identifier_or_keyword(self.token) {
            self.parse_identifier_name_or_keyword()
        } else if self.token == SyntaxKind::StringLiteral {
            self.parse_string_literal_node()
        } else {
            self.expect(SyntaxKind::Identifier);
            self.parse_identifier_name_or_keyword()
        };
        self.expect(SyntaxKind::ColonToken);
        let value = self.parse_assignment_expression();
        let end = value.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportAttribute,
            NodeData::ImportAttribute(ImportAttributeData { name, value }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn try_parse_import_clause(
        &mut self,
        identifier: Option<Arc<Node>>,
        pos: usize,
        phase_modifier: Option<SyntaxKind>,
    ) -> Option<Arc<Node>> {
        if identifier.is_some()
            || self.token == SyntaxKind::AsteriskToken
            || self.token == SyntaxKind::OpenBraceToken
        {
            let import_clause = self.parse_import_clause(identifier, pos, phase_modifier);
            self.expect(SyntaxKind::FromKeyword);
            Some(import_clause)
        } else {
            None
        }
    }

    pub(crate) fn parse_import_clause(
        &mut self,
        identifier: Option<Arc<Node>>,
        pos: usize,
        phase_modifier: Option<SyntaxKind>,
    ) -> Arc<Node> {
        let mut named_bindings = None;
        if identifier.is_none() || self.parse_optional(SyntaxKind::CommaToken) {
            named_bindings = if self.token == SyntaxKind::AsteriskToken {
                Some(self.parse_namespace_import())
            } else {
                Some(self.parse_named_imports())
            };
        }
        let end = named_bindings.as_ref().map_or_else(
            || identifier.as_ref().map_or(pos, |id| id.end()),
            |n| n.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::ImportClause,
            NodeData::ImportClause(ImportClauseData {
                phase_modifier,
                name: identifier,
                named_bindings,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_namespace_import(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        self.expect(SyntaxKind::AsKeyword);
        let name = self.parse_identifier();
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::NamespaceImport,
            NodeData::NamespaceImport(NamespaceImportData { name }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_named_imports(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let elements = self.parse_list(
            ParsingContext::ImportOrExportSpecifiers,
            Parser::parse_import_specifier,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::NamedImports,
            NodeData::NamedImports(NamedImportsData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_import_specifier(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let (is_type_only, property_name, name) = self.parse_import_or_export_specifier(true);

        let name = if name.kind == SyntaxKind::Identifier {
            name
        } else {
            self.parse_error_at_range(
                TextRange::new(name.pos(), name.end()),
                diagnostics::IDENTIFIER_EXPECTED,
                &[],
            );
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text: String::new() }),
                TextRange::new(name.pos(), name.pos()),
            ))
        };

        self.parse_optional(SyntaxKind::CommaToken);
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportSpecifier,
            NodeData::ImportSpecifier(ImportSpecifierData {
                is_type_only,
                property_name,
                name,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_import_or_export_specifier(
        &mut self,
        is_import: bool,
    ) -> (bool, Option<Arc<Node>>, Arc<Node>) {
        let mut can_parse_as_keyword = true;
        let disallow_keywords = is_import;
        let (mut name, mut name_ok) = self.parse_module_export_name(disallow_keywords);
        let mut is_type_only = false;
        let mut property_name: Option<Arc<Node>> = None;
        if name.kind == SyntaxKind::Identifier && name.text() == "type" {
            if self.token == SyntaxKind::AsKeyword {

                let first_as = self.parse_identifier_name_or_keyword();
                if self.token == SyntaxKind::AsKeyword {

                    let second_as = self.parse_identifier_name_or_keyword();
                    if self.can_parse_module_export_name() {

                        is_type_only = true;
                        property_name = Some(first_as);
                        let (n, ok) = self.parse_module_export_name(disallow_keywords);
                        name = n;
                        name_ok = ok;
                        can_parse_as_keyword = false;
                    } else {

                        property_name = Some(name);
                        name = second_as;
                        can_parse_as_keyword = false;
                    }
                } else if self.can_parse_module_export_name() {

                    property_name = Some(name);
                    let (n, ok) = self.parse_module_export_name(disallow_keywords);
                    name = n;
                    name_ok = ok;
                    can_parse_as_keyword = false;
                } else {

                    is_type_only = true;
                    name = first_as;
                }
            } else if self.can_parse_module_export_name() {

                is_type_only = true;
                let (n, ok) = self.parse_module_export_name(disallow_keywords);
                name = n;
                name_ok = ok;
            }

        }
        if can_parse_as_keyword && self.token == SyntaxKind::AsKeyword {
            property_name = Some(name);
            self.expect(SyntaxKind::AsKeyword);
            let (n, ok) = self.parse_module_export_name(disallow_keywords);
            name = n;
            name_ok = ok;
        }
        if !name_ok {

            self.parse_error_at_range(
                TextRange::new(name.pos(), name.end()),
                diagnostics::IDENTIFIER_EXPECTED,
                &[],
            );
        }
        (is_type_only, property_name, name)
    }

    pub(crate) fn can_parse_module_export_name(&self) -> bool {
        is_identifier_or_keyword(self.token) || self.token == SyntaxKind::StringLiteral
    }

    pub(crate) fn parse_module_export_name(&mut self, disallow_keywords: bool) -> (Arc<Node>, bool) {
        if self.token == SyntaxKind::StringLiteral {
            return (self.parse_string_literal_node(), true);
        }
        let name_ok = !(disallow_keywords
            && is_keyword(self.token)
            && is_reserved_word_kind(self.token));
        (self.parse_identifier_name_or_keyword(), name_ok)
    }

    pub(crate) fn parse_identifier_name_or_keyword(&mut self) -> Arc<Node> {
        if self.is_identifier() {
            self.parse_identifier()
        } else {

            let text = format!("{:?}", self.token)
                .trim_end_matches("Keyword")
                .to_lowercase();
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text }),
                TextRange::new(pos, end),
            ))
        }
    }

    pub(crate) fn parse_string_literal_node(&mut self) -> Arc<Node> {
        let text = self.scanner.token_value();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text,
                token_flags: 0,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn make_export_modifier(&self, pos: usize, end: usize) -> ModifierList {
        let export_token = Arc::new(Node::with_loc(
            SyntaxKind::ExportKeyword,
            NodeData::Token,
            TextRange::new(pos, end),
        ));
        ModifierList::new(vec![export_token], ModifierFlags::Export)
    }

    pub(crate) fn parse_export_declaration(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let export_end = self.token_end();
        self.next_token();

        if matches!(
            self.token,
            SyntaxKind::DeclareKeyword
                | SyntaxKind::AsyncKeyword
                | SyntaxKind::AbstractKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::StaticKeyword
        ) {
            return self.parse_declaration_with_modifiers(vec![(
                SyntaxKind::ExportKeyword,
                pos,
                export_end,
            )]);
        }

        if self.token == SyntaxKind::DefaultKeyword {
            let default_pos = self.token_pos();
            let default_end = self.token_end();
            self.next_token();
            if self.token == SyntaxKind::FunctionKeyword {

                let modifiers = self.make_modifier_list(vec![
                    (SyntaxKind::ExportKeyword, pos, export_end),
                    (SyntaxKind::DefaultKeyword, default_pos, default_end),
                ]);
                return self.parse_function_declaration_with_modifiers(Some(modifiers));
            }
            if self.token == SyntaxKind::ClassKeyword {
                let modifiers = self.make_modifier_list(vec![
                    (SyntaxKind::ExportKeyword, pos, export_end),
                    (SyntaxKind::DefaultKeyword, default_pos, default_end),
                ]);
                return self.parse_class_declaration_with_modifiers(Some(modifiers));
            }

            if self.token == SyntaxKind::InterfaceKeyword {
                return self.parse_declaration_with_modifiers(vec![
                    (SyntaxKind::ExportKeyword, pos, export_end),
                    (SyntaxKind::DefaultKeyword, default_pos, default_end),
                ]);
            }

            let expr = self.parse_assignment_expression();
            self.parse_semicolon();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::ExportAssignment,
                NodeData::ExportAssignment(ExportAssignmentData {
                    modifiers: None,
                    is_export_equals: false,
                    type_node: expr.clone(),
                    expression: expr,
                }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            let expr = self.parse_assignment_expression();
            self.parse_semicolon();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::ExportAssignment,
                NodeData::ExportAssignment(ExportAssignmentData {
                    modifiers: None,
                    is_export_equals: true,
                    type_node: expr.clone(),
                    expression: expr,
                }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::AsKeyword {
            self.next_token();
            if self.token == SyntaxKind::NamespaceKeyword {
                self.next_token();
                let name = self.parse_identifier_name_or_keyword();
                self.parse_semicolon();
                let end = self.token_pos();
                return Arc::new(Node::with_loc(
                    SyntaxKind::NamespaceExportDeclaration,
                    NodeData::NamespaceExportDeclaration(NamespaceExportDeclarationData {
                        modifiers: None,
                        name,
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }

        match self.token {
            SyntaxKind::FunctionKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::NamespaceKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::ImportKeyword => {
                return self.parse_declaration_with_modifiers(vec![(
                    SyntaxKind::ExportKeyword,
                    pos,
                    export_end,
                )]);
            }
            SyntaxKind::TypeKeyword => {

                let mut s = self.scanner.clone();
                s.scan();
                if !s.has_preceding_line_break() && Self::token_is_identifier(&s) {
                    return self.parse_declaration_with_modifiers(vec![(
                        SyntaxKind::ExportKeyword,
                        pos,
                        export_end,
                    )]);
                }
                self.next_token();
                return self.parse_export_declaration_tail(pos, true);
            }
            SyntaxKind::ConstKeyword | SyntaxKind::LetKeyword | SyntaxKind::VarKeyword => {

                if self.token == SyntaxKind::ConstKeyword {
                    let mut s = self.scanner.clone();
                    if s.scan() == SyntaxKind::EnumKeyword {
                        return self.parse_declaration_with_modifiers(vec![(
                            SyntaxKind::ExportKeyword,
                            pos,
                            export_end,
                        )]);
                    }
                }

                let export_mod = self.make_export_modifier(pos, export_end);
                let declaration_list = self.parse_variable_declaration_list(false);
                self.parse_semicolon();
                let end = self.token_pos();
                return Arc::new(Node::with_loc(
                    SyntaxKind::VariableStatement,
                    NodeData::VariableStatement(VariableStatementData {
                        modifiers: Some(Arc::new(export_mod)),
                        declaration_list,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            _ => {}
        }

        self.parse_export_declaration_tail(pos, false)
    }

    pub(crate) fn parse_export_declaration_tail(&mut self, pos: usize, is_type_only: bool) -> Arc<Node> {
        let export_clause = if self.parse_optional(SyntaxKind::AsteriskToken) {

            if self.parse_optional(SyntaxKind::AsKeyword) {

                let (name, _) = self.parse_module_export_name(false);
                let end = name.end();
                Some(Arc::new(Node::with_loc(
                    SyntaxKind::NamespaceExport,
                    NodeData::NamespaceExport(NamespaceExportData { name }),
                    TextRange::new(pos, end),
                )))
            } else {
                None
            }
        } else {

            Some(self.parse_named_exports())
        };

        let module_specifier = if self.parse_optional(SyntaxKind::FromKeyword) {
            Some(self.parse_string_literal_node())
        } else {
            None
        };
        let attributes = self.try_parse_import_attributes();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ExportDeclaration,
            NodeData::ExportDeclaration(ExportDeclarationData {
                modifiers: None,
                is_type_only,
                export_clause,
                module_specifier,
                attributes,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_named_exports(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let elements = self.parse_list(
            ParsingContext::ImportOrExportSpecifiers,
            Parser::parse_export_specifier,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::NamedExports,
            NodeData::NamedExports(NamedExportsData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_export_specifier(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let (is_type_only, property_name, name) = self.parse_import_or_export_specifier(false);
        self.parse_optional(SyntaxKind::CommaToken);
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ExportSpecifier,
            NodeData::ExportSpecifier(ExportSpecifierData {
                is_type_only,
                property_name,
                name,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_enum_member(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let name = self.parse_property_name();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };

        let end = initializer.as_ref().map_or(name.end(), |i| i.end());
        Arc::new(Node::with_loc(
            SyntaxKind::EnumMember,
            NodeData::EnumMember(EnumMemberData { name, initializer }),
            TextRange::new(pos, end),
        ))
    }
}
