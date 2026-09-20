#![allow(unused_imports)]

use crate::parser::declarations::*;

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
        } else if is_keyword(self.token) {
            let text = format!("{:?}", self.token)
                .trim_end_matches("Keyword")
                .to_lowercase();
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            Arc::new(Node::with_loc_flags(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text }),
                TextRange::new(pos, end),
                self.context_flags_now(),
            ))
        } else {
            // Go createIdentifierWithDiagnostic：保留字与普通 token 分别报
            // TS1003 两种变体，给缺失名且不消费当前 token
            self.identifier_expected_error_and_missing()
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

    pub(crate) fn parse_export_declaration(&mut self) -> Arc<Node> {
        self.parse_export_declaration_with_pre(Vec::new())
    }

    pub(crate) fn parse_export_declaration_with_pre(
        &mut self,
        pre: Vec<(SyntaxKind, usize, usize)>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        let export_end = self.token_end();
        self.next_token();
        let with_export =
            |pre: &Vec<(SyntaxKind, usize, usize)>| -> Vec<(SyntaxKind, usize, usize)> {
                let mut v = pre.clone();
                v.push((SyntaxKind::ExportKeyword, pos, export_end));
                v
            };

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
            return self.parse_declaration_with_modifiers(with_export(&pre));
        }

        if self.token == SyntaxKind::UsingKeyword
            || (self.token == SyntaxKind::AwaitKeyword && self.is_await_using_declaration())
        {
            return self.parse_declaration_with_modifiers(with_export(&pre));
        }

        if self.token == SyntaxKind::DefaultKeyword {
            let default_pos = self.token_pos();
            let default_end = self.token_end();
            self.next_token();
            if self.token == SyntaxKind::AsyncKeyword
                && self.look_ahead_token() == SyntaxKind::FunctionKeyword
            {
                let async_pos = self.token_pos();
                let async_end = self.token_end();
                self.next_token();
                let modifiers = self.make_modifier_list(vec![
                    (SyntaxKind::ExportKeyword, pos, export_end),
                    (SyntaxKind::DefaultKeyword, default_pos, default_end),
                    (SyntaxKind::AsyncKeyword, async_pos, async_end),
                ]);
                return self.parse_function_declaration_with_modifiers(Some(modifiers));
            }
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
            let end = self.node_pos();
            let decl_pos = pre.first().map_or(pos, |m| m.1.min(pos));
            return Arc::new(Node::with_loc(
                SyntaxKind::ExportAssignment,
                NodeData::ExportAssignment(ExportAssignmentData {
                    modifiers: self.make_optional_modifier_list(&pre),
                    is_export_equals: false,
                    type_node: expr.clone(),
                    expression: expr,
                }),
                TextRange::new(decl_pos, end),
            ));
        }

        if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            let expr = self.parse_assignment_expression();
            self.parse_semicolon();
            let end = self.node_pos();
            let decl_pos = pre.first().map_or(pos, |m| m.1.min(pos));
            return Arc::new(Node::with_loc(
                SyntaxKind::ExportAssignment,
                NodeData::ExportAssignment(ExportAssignmentData {
                    modifiers: self.make_optional_modifier_list(&pre),
                    is_export_equals: true,
                    type_node: expr.clone(),
                    expression: expr,
                }),
                TextRange::new(decl_pos, end),
            ));
        }

        if self.token == SyntaxKind::AsKeyword {
            self.next_token();
            if self.token == SyntaxKind::NamespaceKeyword {
                self.next_token();
                let name = self.parse_identifier_name_or_keyword();
                self.parse_semicolon();
                let end = self.node_pos();
                return Arc::new(Node::with_loc(
                    SyntaxKind::NamespaceExportDeclaration,
                    NodeData::NamespaceExportDeclaration(NamespaceExportDeclarationData {
                        modifiers: self.make_optional_modifier_list(&pre),
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
            | SyntaxKind::ImportKeyword
            | SyntaxKind::ExportKeyword => {
                return self.parse_declaration_with_modifiers(with_export(&pre));
            }
            SyntaxKind::TypeKeyword => {
                let mut s = self.scanner.clone();
                s.scan();
                if !s.has_preceding_line_break() && Self::token_is_identifier(&s) {
                    return self.parse_declaration_with_modifiers(with_export(&pre));
                }
                self.next_token();
                return self.parse_export_declaration_tail(pos, true);
            }
            SyntaxKind::ConstKeyword | SyntaxKind::LetKeyword | SyntaxKind::VarKeyword => {
                if self.token == SyntaxKind::ConstKeyword {
                    let mut s = self.scanner.clone();
                    if s.scan() == SyntaxKind::EnumKeyword {
                        return self.parse_declaration_with_modifiers(with_export(&pre));
                    }
                }

                let export_mod = self.make_modifier_list(with_export(&pre));
                let declaration_list = self.parse_variable_declaration_list(false);
                self.parse_semicolon();
                let end = self.node_pos();
                return Arc::new(Node::with_loc(
                    SyntaxKind::VariableStatement,
                    NodeData::VariableStatement(VariableStatementData {
                        modifiers: Some(export_mod),
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
        let end = self.node_pos();
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
        let end = self.node_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::NamedExports,
            NodeData::NamedExports(NamedExportsData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }
}
