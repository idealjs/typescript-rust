#![allow(unused_imports)]

use super::*;

impl crate::parser::Parser {
    pub(crate) fn parse_jsdoc_identifier_name(&mut self, diagnostic: Option<Message>) -> Arc<Node> {
        if !self.is_identifier() {
            if let Some(msg) = diagnostic {
                self.parse_error_at_current_token(msg, &[]);
            }

            return Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(self.token_pos(), self.token_pos()),
            ));
        }
        let text = self.scanner.token_text().to_string();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token_jsdoc();
        Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData { text }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_jsdoc_entity_name(&mut self, diagnostic: Option<Message>) -> Arc<Node> {
        let mut node = self.parse_jsdoc_identifier_name(diagnostic);
        while self.parse_optional_jsdoc(SyntaxKind::DotToken) {
            let right = self.parse_jsdoc_identifier_name(diagnostic);
            let end = right.end();
            let node_pos = node.pos();
            node = Arc::new(Node::with_loc(
                SyntaxKind::QualifiedName,
                NodeData::QualifiedName(QualifiedNameData { left: node, right }),
                TextRange::new(node_pos, end),
            ));

            self.parse_optional_jsdoc(SyntaxKind::OpenBracketToken);
            self.parse_optional_jsdoc(SyntaxKind::CloseBracketToken);
        }
        node
    }

    pub(crate) fn parse_jsdoc_name_reference(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let has_brace = self.parse_optional_jsdoc(SyntaxKind::OpenBraceToken);
        let entity_name = self.parse_jsdoc_link_name();
        if has_brace {
            self.parse_expected_jsdoc(SyntaxKind::CloseBraceToken);
        }

        self.scanner
            .set_range(self.scanner.full_start_pos(), self.scanner.end());
        self.next_token_jsdoc();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocNameReference,
            NodeData::JSDocNameReference(JSDocNameReferenceData { name: entity_name }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_jsdoc_link_name(&mut self) -> Arc<Node> {
        if !is_identifier_or_keyword_token(self.token) {
            return self.create_missing_node(
                SyntaxKind::Identifier,
                self.token_pos(),
                self.token_pos(),
            );
        }
        let mut node = self.parse_jsdoc_identifier_name(None);
        loop {
            if self.parse_optional_jsdoc(SyntaxKind::DotToken) {
                let right = if is_identifier_or_keyword_token(self.token) {
                    self.parse_jsdoc_identifier_name(None)
                } else {
                    self.create_missing_node(
                        SyntaxKind::Identifier,
                        self.token_pos(),
                        self.token_pos(),
                    )
                };
                let end = right.end();
                let node_pos = node.pos();
                node = Arc::new(Node::with_loc(
                    SyntaxKind::QualifiedName,
                    NodeData::QualifiedName(QualifiedNameData { left: node, right }),
                    TextRange::new(node_pos, end),
                ));
            } else if self.token == SyntaxKind::PrivateIdentifier {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token_jsdoc();
                let right = Arc::new(Node::with_loc(
                    SyntaxKind::PrivateIdentifier,
                    NodeData::PrivateIdentifier(PrivateIdentifierData { text }),
                    TextRange::new(pos, end),
                ));
                let end = right.end();
                let node_pos = node.pos();
                node = Arc::new(Node::with_loc(
                    SyntaxKind::QualifiedName,
                    NodeData::QualifiedName(QualifiedNameData { left: node, right }),
                    TextRange::new(node_pos, end),
                ));
            } else {
                break;
            }
        }
        node
    }

    pub(crate) fn parse_jsdoc_type_name_with_namespace(
        &mut self,
        nested: bool,
    ) -> Option<Arc<Node>> {
        if !is_identifier_or_keyword_token(self.token) {
            return None;
        }
        let pos = self.token_pos();
        let name = self.parse_jsdoc_identifier_name(None);
        let mut node = name;
        if self.parse_optional_jsdoc(SyntaxKind::DotToken) {
            if let Some(inner) = self.parse_jsdoc_type_name_with_namespace(true) {
                let end = inner.end();
                node = Arc::new(Node::with_loc(
                    SyntaxKind::ModuleDeclaration,
                    NodeData::ModuleDeclaration(ModuleDeclarationData {
                        modifiers: None,
                        keyword: SyntaxKind::NamespaceKeyword,
                        name: node,
                        body: Some(inner),
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }

        let _ = nested;
        Some(node)
    }

    pub(crate) fn parse_bracket_name_in_property_and_param_tag(
        &mut self,
        target: PropertyLikeParse,
    ) -> (Arc<Node>, bool) {
        let is_bracketed = self.parse_optional_jsdoc(SyntaxKind::OpenBracketToken);
        if is_bracketed {
            self.skip_whitespace();
        }

        let backquoted = self.parse_optional_jsdoc(SyntaxKind::BacktickToken);

        let diagnostic = if target.contains(PropertyLikeParse::PARAMETER) {
            None
        } else {
            Some(diagnostics::IDENTIFIER_EXPECTED)
        };
        let name = self.parse_jsdoc_entity_name(diagnostic);

        if backquoted {
            self.parse_expected_token_jsdoc(SyntaxKind::BacktickToken);
        }

        if is_bracketed {
            self.skip_whitespace();

            if self.parse_optional_jsdoc(SyntaxKind::EqualsToken) {
                let _default = self.parse_type();
            }
            let _close = self.parse_expected_token_jsdoc(SyntaxKind::CloseBracketToken);
        }

        (name, is_bracketed)
    }
}

impl crate::parser::Parser {
    pub(crate) fn parse_jsdoc_link(&mut self, start: usize) -> Option<Arc<Node>> {
        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;

        let (link_kind, is_link) = self.parse_jsdoc_link_prefix();
        if !is_link {
            self.scanner = saved_scanner;
            self.token = saved_token;
            return None;
        }

        self.next_token_jsdoc();
        self.skip_whitespace();

        let name = if is_identifier_or_keyword_token(self.token) {
            Some(self.parse_jsdoc_link_name())
        } else {
            None
        };

        let mut text_parts: Vec<String> = Vec::new();
        loop {
            match self.token {
                SyntaxKind::CloseBraceToken | SyntaxKind::NewLineTrivia | SyntaxKind::EndOfFile => {
                    break;
                }
                SyntaxKind::WhitespaceTrivia => {
                    text_parts.push(self.scanner.token_text().to_string());
                    self.next_token_jsdoc();
                }
                _ => {
                    text_parts.push(self.scanner.token_text().to_string());
                    self.next_jsdoc_comment_text_token(false);
                }
            }
        }

        if let Some(last) = text_parts.last_mut() {
            *last = trim_end(last);
        }

        let end = self.token_end();
        let (kind, data) = match link_kind.as_str() {
            "linkcode" => (
                SyntaxKind::JSDocLinkCode,
                NodeData::JSDocLinkCode(JSDocLinkCodeData {
                    name,
                    text: text_parts,
                }),
            ),
            "linkplain" => (
                SyntaxKind::JSDocLinkPlain,
                NodeData::JSDocLinkPlain(JSDocLinkPlainData {
                    name,
                    text: text_parts,
                }),
            ),
            _ => (
                SyntaxKind::JSDocLink,
                NodeData::JSDocLink(JSDocLinkData {
                    name,
                    text: text_parts,
                }),
            ),
        };

        Some(Arc::new(Node::with_loc(
            kind,
            data,
            TextRange::new(start, end),
        )))
    }

    pub(crate) fn parse_jsdoc_link_prefix(&mut self) -> (String, bool) {
        self.skip_whitespace_or_asterisk();
        if self.token != SyntaxKind::OpenBraceToken {
            return ("NONE".to_string(), false);
        }
        let mut sc = self.scanner.clone();
        sc.scan_jsdoc_token();
        if sc.token() != SyntaxKind::AtToken {
            return ("NONE".to_string(), false);
        }
        sc.scan_jsdoc_token();
        if !is_identifier_or_keyword_token(sc.token()) {
            return ("NONE".to_string(), false);
        }
        let kind = sc.token_text().to_string();
        if is_jsdoc_link_tag(&kind) {
            (kind, true)
        } else {
            ("NONE".to_string(), false)
        }
    }
}
