#![allow(unused_imports)]

use crate::parser::*;
use tsox_core::core::compiler_options_kinds::ModuleKind;

impl Parser {
    pub(crate) fn parse_optional_token_jsdoc(&mut self, kind: SyntaxKind) -> Option<Arc<Node>> {
        if self.token == kind {
            Some(self.create_token_node())
        } else {
            None
        }
    }

    pub(crate) fn parse_initializer(&mut self) -> Option<Arc<Node>> {
        if self.parse_optional(SyntaxKind::EqualsToken) {
            Some(self.parse_assignment_expression())
        } else {
            None
        }
    }

    pub(crate) fn re_scan_less_than_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.re_scan_less_than();
        self.drain_scanner_errors();
        self.token
    }

    pub(crate) fn parse_function_block_or_semicolon(
        &mut self,
        is_type_member: bool,
        is_generator: bool,
        is_async: bool,
    ) -> Option<Arc<Node>> {
        if self.token != SyntaxKind::OpenBraceToken {
            if is_type_member {
                self.parse_type_member_semicolon();
                return None;
            }
            if self.can_parse_semicolon() {
                self.parse_semicolon();
                return None;
            }
        }
        Some(self.parse_function_block(is_generator, is_async))
    }

    pub(crate) fn parse_resolution_mode(
        &mut self,
        mode: &str,
        pos: usize,
        end: usize,
    ) -> ModuleKind {
        if mode == "import" {
            return ModuleKind::ESNext;
        }
        if mode == "require" {
            return ModuleKind::CommonJS;
        }
        self.parse_error_at(
            pos,
            end,
            tsox_core::diagnostics::X_RESOLUTION_MODE_SHOULD_BE_EITHER_REQUIRE_OR_IMPORT,
            &[],
        );
        ModuleKind::None
    }
}

pub(crate) fn skip_blanks(text: &str, pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut pos = pos;
    while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos
}

pub(crate) fn skip_non_blanks(text: &str, pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut pos = pos;
    while pos < bytes.len()
        && bytes[pos] != b' '
        && bytes[pos] != b'\t'
        && bytes[pos] != b'\r'
        && bytes[pos] != b'\n'
    {
        pos += 1;
    }
    pos
}
