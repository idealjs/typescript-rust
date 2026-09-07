#![allow(unused_imports)]

use super::*;

impl Scanner {
    pub(crate) fn scan_template(&mut self) -> SyntaxKind {
        self.pos += 1;
        let mut has_substitution = false;
        let mut terminated = false;
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == '`' {
                self.pos += 1;
                terminated = true;
                break;
            }
            if c == '$'
                && self.pos + 1 < self.end
                && self.text.as_bytes()[self.pos + 1] as char == '{'
            {
                self.pos += 2;
                has_substitution = true;
                terminated = true;
                break;
            }
            if c == '\\' {
                self.scan_escape_sequence();
                continue;
            }
            self.pos += 1;
        }
        if !terminated {
            self.token_flags |= TOKEN_FLAGS_UNTERMINATED;
            self.report_error(
                DiagnosticKind::UnterminatedTemplateLiteral,
                self.token_pos,
                self.pos - self.token_pos,
            );
        }
        self.token_end = self.pos;
        self.token = if has_substitution {
            SyntaxKind::TemplateHead
        } else {
            SyntaxKind::NoSubstitutionTemplateLiteral
        };
        self.token
    }

    pub(crate) fn scan_punctuation(&mut self) -> SyntaxKind {
        let start = self.pos;

        let remaining = &self.text[start..];

        if remaining.starts_with('<') {
            if remaining.starts_with("<<=") {
                self.pos = start + 3;
                self.token_end = self.pos;
                self.token = SyntaxKind::LessThanLessThanEqualsToken;
                return self.token;
            }
            if remaining.starts_with("<<") {
                self.pos = start + 2;
                self.token_end = self.pos;
                self.token = SyntaxKind::LessThanLessThanToken;
                return self.token;
            }
            if remaining.starts_with("<=") {
                self.pos = start + 2;
                self.token_end = self.pos;
                self.token = SyntaxKind::LessThanEqualsToken;
                return self.token;
            }
            if self.language_variant == crate::ast::LanguageVariant::Jsx
                && remaining.starts_with("</")
                && !remaining[2..].starts_with('*')
            {
                self.pos = start + 2;
                self.token_end = self.pos;
                self.token = SyntaxKind::LessThanSlashToken;
                return self.token;
            }
            self.pos = start + 1;
            self.token_end = self.pos;
            self.token = SyntaxKind::LessThanToken;
            return self.token;
        }

        let mut best_match: Option<SyntaxKind> = None;
        let mut best_len = 0;

        if remaining.len() >= 4 {
            if let Some(slice) = remaining.get(..4) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 4;
                }
            }
        }

        if best_len == 0 && remaining.len() >= 3 {
            if let Some(slice) = remaining.get(..3) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 3;
                }
            }
        }

        if best_len == 0 && remaining.len() >= 2 {
            if let Some(slice) = remaining.get(..2) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 2;
                }
            }
        }

        if best_len == 0 {
            let first_len = remaining.chars().next().map(char::len_utf8).unwrap_or(0);
            if first_len == 1 {
                if let Some(kind) = string_to_token(&remaining[..first_len]) {
                    best_match = Some(kind);
                    best_len = first_len;
                }
            }
        }

        if let Some(kind) = best_match {
            self.pos += best_len;
            self.token_end = self.pos;
            self.token = kind;
            kind
        } else {
            let c = self.text[start..].chars().next().unwrap();

            if c == '\u{fffd}' {
                self.report_error(DiagnosticKind::FileAppearsToBeBinary, 0, 0);
                if self.binary_marker_pos.is_none() {
                    self.binary_marker_pos = Some(start);
                }
                self.pos = self.text.len();
                self.token_end = self.pos;
                self.token = SyntaxKind::EndOfFile;
                return SyntaxKind::EndOfFile;
            }
            let len = c.len_utf8();
            self.pos += len;
            self.token_end = self.pos;
            self.token = SyntaxKind::Unknown;
            self.report_error(DiagnosticKind::InvalidCharacter, start, len);
            SyntaxKind::Unknown
        }
    }

    pub fn binary_marker_pos(&self) -> Option<usize> {
        self.binary_marker_pos
    }

    pub fn rewind(&mut self) {
        self.pos = self.token_pos;
    }

    pub fn re_scan_greater_than(&mut self) -> SyntaxKind {
        let token = self.token;
        if token == SyntaxKind::GreaterThanToken {
            return token;
        }

        match token {
            SyntaxKind::GreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {}
            _ => return token,
        }

        self.pos = self.token_pos + 1;
        self.token_end = self.pos;
        self.token = SyntaxKind::GreaterThanToken;
        SyntaxKind::GreaterThanToken
    }

    pub fn re_scan_greater_than_remainder(&self) -> Option<SyntaxKind> {
        match self.token {
            SyntaxKind::GreaterThanGreaterThanToken => Some(SyntaxKind::GreaterThanToken),
            SyntaxKind::GreaterThanGreaterThanGreaterThanToken => {
                Some(SyntaxKind::GreaterThanGreaterThanToken)
            }
            SyntaxKind::GreaterThanEqualsToken => Some(SyntaxKind::EqualsToken),
            SyntaxKind::GreaterThanGreaterThanEqualsToken => {
                Some(SyntaxKind::GreaterThanEqualsToken)
            }
            SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {
                Some(SyntaxKind::GreaterThanGreaterThanEqualsToken)
            }
            _ => None,
        }
    }
}
