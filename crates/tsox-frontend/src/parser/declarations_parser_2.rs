#![allow(unused_imports)]

use crate::parser::declarations::*;

impl Parser {
    pub(crate) fn parse_namespace_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = Self::declaration_start(&modifiers, self.token_pos());
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
            let end = self.node_pos();
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

        loop {
            let decl_pos = if segments.is_empty() { pos } else { name.pos() };
            // Go parser parseNamespaceDeclaration：点分名展开为嵌套声明，
            // 仅内层段补 export，最外层保留用户修饰符
            let mods = if segments.is_empty() {
                modifiers.clone()
            } else {
                let phantom_pos = name.pos();
                let export_tok = Arc::new(Node::with_loc_flags(
                    SyntaxKind::ExportKeyword,
                    NodeData::Token,
                    TextRange::new(phantom_pos, phantom_pos),
                    crate::ast::NodeFlags::Reparsed,
                ));
                let mut ml = ModifierList::new(vec![export_tok], ModifierFlags::Export);
                ml.list.loc = TextRange::new(phantom_pos, phantom_pos);
                Some(Arc::new(ml))
            };
            let decl = Arc::new(Node::with_loc(
                SyntaxKind::ModuleDeclaration,
                NodeData::ModuleDeclaration(ModuleDeclarationData {
                    modifiers: mods,
                    keyword,
                    name: Arc::clone(&name),
                    attributes: None,
                    body: inner_body,
                }),
                TextRange::new(decl_pos, end),
            ));
            match segments.pop() {
                Some(seg) => {
                    name = seg;
                    inner_body = Some(decl);
                }
                None => return decl,
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
        let attributes = if self.token == SyntaxKind::WithKeyword {
            Some(self.parse_import_attributes(self.token, false))
        } else {
            None
        };
        let body = if self.token == SyntaxKind::OpenBraceToken {
            let body_pos = self.token_pos();
            self.next_token();
            let statements =
                self.parse_list(ParsingContext::BlockStatements, Parser::parse_statement);
            self.expect(SyntaxKind::CloseBraceToken);
            let end = self.node_pos();
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
                attributes,
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
        self.parse_import_declaration_with_modifiers(pos, None)
    }

    pub(crate) fn parse_import_declaration_with_modifiers(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
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
                return self.parse_import_equals_with_modifiers(
                    pos,
                    modifiers,
                    id.clone(),
                    is_type_only,
                );
            }
        }

        let import_clause =
            self.try_parse_import_clause(identifier, after_import_pos, phase_modifier);
        let module_specifier = self.parse_module_specifier();
        let attributes = self.try_parse_import_attributes();
        self.parse_semicolon();
        let end = self.node_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportDeclaration,
            NodeData::ImportDeclaration(ImportDeclarationData {
                modifiers,
                import_clause,
                module_specifier,
                attributes,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_import_equals_with_modifiers(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        is_type_only: bool,
    ) -> Arc<Node> {
        self.parse_import_equals_tail(pos, modifiers, name, is_type_only)
    }

    pub(crate) fn token_after_import_definitely_produces_import_declaration(&self) -> bool {
        self.token == SyntaxKind::AsteriskToken || self.token == SyntaxKind::OpenBraceToken
    }

    pub(crate) fn token_after_imported_identifier_definitely_produces_import_declaration(
        &self,
    ) -> bool {
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
}

impl Parser {
    /// 浅消费一个平衡的 `{ ... }` 块（module attributes 等不求值场景）
    pub(crate) fn skip_balanced_brace_block(&mut self) {
        if self.token != SyntaxKind::OpenBraceToken {
            return;
        }
        let mut depth = 0usize;
        loop {
            let kind = self.token;
            if kind == SyntaxKind::OpenBraceToken {
                depth += 1;
            } else if kind == SyntaxKind::CloseBraceToken {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    self.next_token();
                    return;
                }
            } else if kind == SyntaxKind::EndOfFile {
                return;
            }
            self.next_token();
        }
    }
}
