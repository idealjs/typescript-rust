#![allow(unused_imports)]

use crate::parser::impl_chunk::*;

impl Parser {
    pub(crate) fn scan_jsx_identifier(&mut self) -> SyntaxKind { ::tsox_core::fntrace::enter("scan_jsx_identifier"); 
        self.token = self.scanner.scan_jsx_identifier();
        self.drain_scanner_errors();
        self.token
    }

    #[allow(dead_code)]
    pub(crate) fn scan_jsx_attribute_value(&mut self) -> SyntaxKind { ::tsox_core::fntrace::enter("scan_jsx_attribute_value"); 
        self.token = self.scanner.scan_jsx_attribute_value();
        self.drain_scanner_errors();
        self.token
    }

    pub(crate) fn token_range(&self) -> TextRange { ::tsox_core::fntrace::enter("token_range"); 
        TextRange::new(self.token_pos(), self.token_end())
    }

    pub(crate) fn parse_error_at_range(
        &mut self,
        range: TextRange,
        message: Message,
        args: &[&str],
    ) { ::tsox_core::fntrace::enter("parse_error_at_range"); 
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
    ) { ::tsox_core::fntrace::enter("parse_error_at"); 
        self.parse_error_at_range(TextRange::new(pos, end), message, args);
    }

    pub(crate) fn parse_error_at_current_token(&mut self, message: Message, args: &[&str]) { ::tsox_core::fntrace::enter("parse_error_at_current_token"); 
        self.parse_error_at_range(self.token_range(), message, args);
    }

    /// Go parseExpected：匹配则消费返回 true；不匹配报错返回 false，不消费
    pub(crate) fn expect(&mut self, expected: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("expect"); 
        if self.token == expected {
            self.next_token();
            true
        } else {
            self.parse_error_at_current_token(
                tsox_core::diagnostics::X_0_EXPECTED,
                &[token_to_string(expected)],
            );
            false
        }
    }

    pub(crate) fn expect_without_advancing(&mut self, expected: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("expect_without_advancing"); 
        if self.token == expected {
            true
        } else {
            self.parse_error_at_current_token(
                tsox_core::diagnostics::X_0_EXPECTED,
                &[token_to_string(expected)],
            );
            false
        }
    }

    pub(crate) fn parse_optional(&mut self, kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("parse_optional"); 
        if self.token == kind {
            self.next_token();
            true
        } else {
            false
        }
    }

    pub(crate) fn parse_optional_token(&mut self, kind: SyntaxKind) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parse_optional_token"); 
        if self.token == kind {
            let node = self.create_token_node();
            self.next_token();
            Some(node)
        } else {
            None
        }
    }

    pub(crate) fn create_token_node(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("create_token_node"); 
        Arc::new(Node::with_loc(
            self.token,
            NodeData::Token,
            TextRange::new(self.token_pos(), self.token_end()),
        ))
    }

    pub(crate) fn create_template_token_node(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("create_template_token_node"); 
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
        let template_flags = self.scanner.token_flags();
        let data = match self.token {
            SyntaxKind::TemplateHead => NodeData::TemplateHead(TemplateHeadData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags,
            }),
            SyntaxKind::TemplateMiddle => NodeData::TemplateMiddle(TemplateMiddleData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags,
            }),
            SyntaxKind::TemplateTail => NodeData::TemplateTail(TemplateTailData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags,
            }),
            _ => NodeData::Token,
        };
        Arc::new(Node::with_loc(
            self.token,
            data,
            TextRange::new(self.token_pos(), self.token_end()),
        ))
    }

    pub(crate) fn missing_node(&self, pos: usize) -> Arc<Node> { ::tsox_core::fntrace::enter("missing_node"); 
        Arc::new(Node::with_loc(
            SyntaxKind::MissingDeclaration,
            NodeData::MissingDeclaration(MissingDeclarationData { modifiers: None }),
            TextRange::new(pos, pos),
        ))
    }

    pub(crate) fn can_parse_semicolon(&self) -> bool { ::tsox_core::fntrace::enter("can_parse_semicolon"); 
        self.token == SyntaxKind::SemicolonToken
            || self.token == SyntaxKind::CloseBraceToken
            || self.token == SyntaxKind::EndOfFile
            || self.has_preceding_line_break()
    }

    pub(crate) fn try_parse_semicolon(&mut self) -> bool { ::tsox_core::fntrace::enter("try_parse_semicolon"); 
        if !self.can_parse_semicolon() {
            return false;
        }
        if self.token == SyntaxKind::SemicolonToken {
            self.next_token();
        }
        true
    }

    pub(crate) fn parse_semicolon(&mut self) -> bool { ::tsox_core::fntrace::enter("parse_semicolon"); 
        self.try_parse_semicolon() || {
            self.expect(SyntaxKind::SemicolonToken);
            false
        }
    }
}

impl Parser {
    /// Go parseExpectedToken：匹配时消费并给 token 节点；失败时报错并给
    /// 零宽缺失 token（不消费当前 token）
    pub(crate) fn parse_expected_token_colon(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_expected_token_colon"); 
        if self.token == SyntaxKind::ColonToken {
            let node = self.create_token_node();
            self.next_token();
            node
        } else {
            self.parse_error_at_current_token(
                tsox_core::diagnostics::X_0_EXPECTED,
                &[token_to_string(SyntaxKind::ColonToken)],
            );
            let pos = self.token_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::ColonToken,
                NodeData::Token,
                TextRange::new(pos, pos),
            ))
        }
    }

    /// Go createMissingIdentifier：零宽空名标识符
    pub(crate) fn missing_identifier_expression(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("missing_identifier_expression"); 
        let pos = self.node_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: String::new(),
            }),
            TextRange::new(pos, pos),
        ))
    }
}

impl Parser {
    /// Go parseExpected：匹配时消费；失败时报错返回 false（不消费）
    pub(crate) fn expect_report(&mut self, expected: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("expect_report"); 
        if self.token == expected {
            self.next_token();
            true
        } else {
            self.parse_error_at_current_token(
                tsox_core::diagnostics::X_0_EXPECTED,
                &[token_to_string(expected)],
            );
            false
        }
    }
}
