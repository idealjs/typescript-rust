#![allow(unused_imports)]

use super::*;

impl Parser {
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
                self.parse_error_at_current_token(crate::diagnostics::X_0_EXPECTED, &["{"]);
                return None;
            }
            Some(self.parse_import_attributes(self.token, false))
        } else {
            None
        }
    }

    pub(crate) fn parse_import_attributes(
        &mut self,
        token: SyntaxKind,
        skip_keyword: bool,
    ) -> Arc<Node> {
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
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
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
}
