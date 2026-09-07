#![allow(unused_imports)]

use super::*;

impl crate::parser::Parser {
    pub(crate) fn next_token_jsdoc(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsdoc_token();
        self.token
    }

    pub(crate) fn next_jsdoc_comment_text_token(&mut self, in_backticks: bool) -> SyntaxKind {
        self.token = self.scanner.scan_jsdoc_comment_text_token(in_backticks);
        self.token
    }

    pub(crate) fn parse_optional_jsdoc(&mut self, kind: SyntaxKind) -> bool {
        if self.token == kind {
            self.next_token_jsdoc();
            true
        } else {
            false
        }
    }

    pub(crate) fn parse_expected_jsdoc(&mut self, kind: SyntaxKind) {
        if !self.parse_optional_jsdoc(kind) {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[token_to_string(kind)]);
        }
    }

    pub(crate) fn parse_expected_token_jsdoc(&mut self, kind: SyntaxKind) -> Arc<Node> {
        if self.token == kind {
            let node = self.create_token_node_jsdoc();
            self.next_token_jsdoc();
            node
        } else {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[token_to_string(kind)]);
            self.create_missing_node(kind, self.token_pos(), self.token_pos())
        }
    }

    pub(crate) fn create_token_node_jsdoc(&self) -> Arc<Node> {
        Arc::new(Node::with_loc(
            self.token,
            NodeData::Token,
            TextRange::new(self.token_pos(), self.token_end()),
        ))
    }

    pub(crate) fn create_missing_node(
        &self,
        kind: SyntaxKind,
        pos: usize,
        end: usize,
    ) -> Arc<Node> {
        Arc::new(Node::with_loc(
            kind,
            NodeData::Token,
            TextRange::new(pos, end),
        ))
    }
}
