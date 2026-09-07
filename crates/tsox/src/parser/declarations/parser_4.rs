#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn can_parse_module_export_name(&self) -> bool {
        is_identifier_or_keyword(self.token) || self.token == SyntaxKind::StringLiteral
    }

    pub(crate) fn parse_module_export_name(
        &mut self,
        disallow_keywords: bool,
    ) -> (Arc<Node>, bool) {
        if self.token == SyntaxKind::StringLiteral {
            return (self.parse_string_literal_node(), true);
        }
        let name_ok =
            !(disallow_keywords && is_keyword(self.token) && is_reserved_word_kind(self.token));
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

    pub(crate) fn parse_export_declaration_tail(
        &mut self,
        pos: usize,
        is_type_only: bool,
    ) -> Arc<Node> {
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
}
