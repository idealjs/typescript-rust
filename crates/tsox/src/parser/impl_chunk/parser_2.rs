#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn scan_jsx_identifier(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_identifier();
        self.drain_scanner_errors();
        self.token
    }

    #[allow(dead_code)]
    pub(crate) fn scan_jsx_attribute_value(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_attribute_value();
        self.drain_scanner_errors();
        self.token
    }

    pub(crate) fn token_range(&self) -> TextRange {
        TextRange::new(self.token_pos(), self.token_end())
    }

    pub(crate) fn parse_error_at_range(
        &mut self,
        range: TextRange,
        message: Message,
        args: &[&str],
    ) {
        if let Some(last) = self.diagnostics.last() {
            if last.range.pos() == range.pos() {
                return;
            }
        }
        self.diagnostics.push(ParserDiagnostic {
            message,
            message_args: args.iter().map(|s| s.to_string()).collect(),
            range,
        });
    }

    pub(crate) fn parse_error_at(
        &mut self,
        pos: usize,
        end: usize,
        message: Message,
        args: &[&str],
    ) {
        self.parse_error_at_range(TextRange::new(pos, end), message, args);
    }

    pub(crate) fn parse_error_at_current_token(&mut self, message: Message, args: &[&str]) {
        self.parse_error_at_range(self.token_range(), message, args);
    }

    pub(crate) fn expect(&mut self, expected: SyntaxKind) {
        if self.token == expected {
            self.next_token();
        } else {
            self.parse_error_at_current_token(
                diagnostics::X_0_EXPECTED,
                &[token_to_string(expected)],
            );
        }
    }

    pub(crate) fn expect_without_advancing(&mut self, expected: SyntaxKind) -> bool {
        if self.token == expected {
            true
        } else {
            self.parse_error_at_current_token(
                diagnostics::X_0_EXPECTED,
                &[token_to_string(expected)],
            );
            false
        }
    }

    pub(crate) fn parse_optional(&mut self, kind: SyntaxKind) -> bool {
        if self.token == kind {
            self.next_token();
            true
        } else {
            false
        }
    }

    pub(crate) fn parse_optional_token(&mut self, kind: SyntaxKind) -> Option<Arc<Node>> {
        if self.token == kind {
            let node = self.create_token_node();
            self.next_token();
            Some(node)
        } else {
            None
        }
    }

    pub(crate) fn create_token_node(&self) -> Arc<Node> {
        Arc::new(Node::with_loc(
            self.token,
            NodeData::Token,
            TextRange::new(self.token_pos(), self.token_end()),
        ))
    }

    pub(crate) fn create_template_token_node(&self) -> Arc<Node> {
        let raw = self.scanner.token_text();
        let cooked = match self.token {
            SyntaxKind::TemplateHead => {
                let s = raw.strip_prefix('`').unwrap_or(raw);
                s.strip_suffix("${").unwrap_or(s).to_string()
            }
            SyntaxKind::TemplateMiddle => {
                let s = raw.strip_prefix('}').unwrap_or(raw);
                s.strip_suffix("${").unwrap_or(s).to_string()
            }
            SyntaxKind::TemplateTail => {
                let s = raw.strip_prefix('}').unwrap_or(raw);
                s.strip_suffix('`').unwrap_or(s).to_string()
            }
            _ => raw.to_string(),
        };
        let data = match self.token {
            SyntaxKind::TemplateHead => NodeData::TemplateHead(TemplateHeadData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags: 0,
            }),
            SyntaxKind::TemplateMiddle => NodeData::TemplateMiddle(TemplateMiddleData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags: 0,
            }),
            SyntaxKind::TemplateTail => NodeData::TemplateTail(TemplateTailData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags: 0,
            }),
            _ => NodeData::Token,
        };
        Arc::new(Node::with_loc(
            self.token,
            data,
            TextRange::new(self.token_pos(), self.token_end()),
        ))
    }

    pub(crate) fn missing_node(&self, pos: usize) -> Arc<Node> {
        Arc::new(Node::with_loc(
            SyntaxKind::MissingDeclaration,
            NodeData::MissingDeclaration(MissingDeclarationData { modifiers: None }),
            TextRange::new(pos, pos),
        ))
    }

    pub(crate) fn can_parse_semicolon(&self) -> bool {
        self.token == SyntaxKind::SemicolonToken
            || self.token == SyntaxKind::CloseBraceToken
            || self.token == SyntaxKind::EndOfFile
            || self.has_preceding_line_break()
    }

    pub(crate) fn try_parse_semicolon(&mut self) -> bool {
        if !self.can_parse_semicolon() {
            return false;
        }
        if self.token == SyntaxKind::SemicolonToken {
            self.next_token();
        }
        true
    }

    pub(crate) fn parse_semicolon(&mut self) -> bool {
        self.try_parse_semicolon() || {
            self.expect(SyntaxKind::SemicolonToken);
            false
        }
    }
}
