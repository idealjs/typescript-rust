use crate::ast::{Node, NodeFlags, SourceFile, SyntaxKind};
use crate::scanner::impl_chunk::*;
use crate::scanner::{
    is_hex_digit, keywords, ErrorCallback, Scanner, ScannerError, DiagnosticKind,
    TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE, TOKEN_FLAGS_NONE, TOKEN_FLAGS_UNICODE_ESCAPE,
};
use std::sync::Arc;
use tsox_core::core::compiler_options::ScriptTarget;
use tsox_core::core::text::TextRange;

impl Scanner {
    pub fn reset(&mut self) { ::tsox_core::fntrace::enter("reset"); 
        let text = std::mem::take(&mut self.text);
        *self = Scanner::new(text.to_string());
    }

    pub fn set_text(&mut self, text: impl Into<String>) { ::tsox_core::fntrace::enter("set_text"); 
        self.text = std::sync::Arc::from(text.into());
        self.end = self.text.len();
        self.pos = 0;
        self.token = SyntaxKind::Unknown;
        self.token_pos = 0;
        self.token_end = 0;
        self.full_start_pos = 0;
        self.token_flags = TOKEN_FLAGS_NONE;
    }

    pub fn set_on_error(&mut self, cb: ErrorCallback) { ::tsox_core::fntrace::enter("set_on_error"); 
        self.error_callback = Some(cb);
    }

    pub fn token_range(&self) -> TextRange { ::tsox_core::fntrace::enter("token_range"); 
        TextRange::new(self.token_pos, self.pos)
    }

    pub fn has_unicode_escape(&self) -> bool { ::tsox_core::fntrace::enter("has_unicode_escape"); 
        self.token_flags & TOKEN_FLAGS_UNICODE_ESCAPE != 0
    }

    pub fn has_extended_unicode_escape(&self) -> bool { ::tsox_core::fntrace::enter("has_extended_unicode_escape"); 
        self.token_flags & TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE != 0
    }

    pub fn reset_pos(&mut self, pos: usize) { ::tsox_core::fntrace::enter("reset_pos"); 
        self.pos = pos;
        self.full_start_pos = pos;
        self.token_pos = pos;
    }

    pub fn reset_token_state(&mut self, pos: usize) { ::tsox_core::fntrace::enter("reset_token_state"); 
        self.reset_pos(pos);
        self.token = SyntaxKind::Unknown;
        self.token_flags = TOKEN_FLAGS_NONE;
    }

    pub(crate) fn language_version(&self) -> ScriptTarget { ::tsox_core::fntrace::enter("language_version"); 
        if self.script_target == ScriptTarget::None {
            return ScriptTarget::LATEST;
        }
        self.script_target
    }

    pub(crate) fn error(&mut self, kind: DiagnosticKind) { ::tsox_core::fntrace::enter("error"); 
        self.error_at(kind, self.pos, 0);
    }

    pub(crate) fn error_at(&mut self, kind: DiagnosticKind, pos: usize, length: usize) { ::tsox_core::fntrace::enter("error_at"); 
        if let Some(cb) = self.error_callback {
            cb(kind, pos, length);
        }
        self.errors.push(ScannerError { kind, pos, length });
    }

    pub(crate) fn char(&self) -> Option<char> { ::tsox_core::fntrace::enter("char"); 
        if self.pos < self.end {
            return Some(self.text.as_bytes()[self.pos] as char);
        }
        None
    }

    pub(crate) fn char_at(&self, offset: usize) -> Option<char> { ::tsox_core::fntrace::enter("char_at"); 
        if self.pos + offset < self.end {
            return Some(self.text.as_bytes()[self.pos + offset] as char);
        }
        None
    }

    pub(crate) fn char_and_size(&self) -> (char, usize) { ::tsox_core::fntrace::enter("char_and_size"); 
        if self.pos < self.end {
            let b = self.text.as_bytes()[self.pos];
            if b < 128 {
                return (b as char, 1);
            }
        }
        match self.text.get(self.pos..).and_then(|s| s.chars().next()) {
            Some(ch) => (ch, ch.len_utf8()),
            None => ('\0', 0),
        }
    }

    pub(crate) fn scan_ascii_while(&mut self, pred: impl Fn(u8) -> bool) { ::tsox_core::fntrace::enter("scan_ascii_while"); 
        let bytes = &self.text.as_bytes()[self.pos..self.end];
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b >= 128 || !pred(b) {
                break;
            }
            i += 1;
        }
        self.pos += i;
    }

    pub fn re_scan_asterisk_equals_token(&mut self) -> SyntaxKind { ::tsox_core::fntrace::enter("re_scan_asterisk_equals_token"); 
        if self.token != SyntaxKind::AsteriskEqualsToken {
            panic!("'ReScanAsteriskEqualsToken' should only be called on a '*='");
        }
        self.pos = self.token_pos + 1;
        self.token = SyntaxKind::EqualsToken;
        self.token
    }

    pub fn re_scan_hash_token(&mut self) -> SyntaxKind { ::tsox_core::fntrace::enter("re_scan_hash_token"); 
        if self.token == SyntaxKind::PrivateIdentifier {
            self.pos = self.token_pos + 1;
            self.token = SyntaxKind::HashToken;
        }
        self.token
    }

    pub fn re_scan_question_token(&mut self) -> SyntaxKind { ::tsox_core::fntrace::enter("re_scan_question_token"); 
        if self.token != SyntaxKind::QuestionQuestionToken {
            panic!("'reScanQuestionToken' should only be called on a '??'");
        }
        self.pos = self.token_pos + 1;
        self.token = SyntaxKind::QuestionToken;
        self.token
    }

    pub(crate) fn scan_low_surrogate_escape(&mut self, high: char) -> Option<char> { ::tsox_core::fntrace::enter("scan_low_surrogate_escape"); 
        if self.char() != Some('\\') || self.char_at(1) != Some('u') {
            return None;
        }
        let saved_pos = self.pos;
        let saved_token_flags = self.token_flags;
        let low = self.scan_unicode_escape();
        if let Some(low) = low {
            if is_low_surrogate(low) {
                return Some(surrogate_pair_to_code_point(high, low));
            }
        }
        self.pos = saved_pos;
        self.token_flags = saved_token_flags;
        None
    }

    pub(crate) fn peek_unicode_escape(&mut self) -> Option<char> { ::tsox_core::fntrace::enter("peek_unicode_escape"); 
        if self.char_at(1) == Some('u') {
            let save_pos = self.pos;
            let save_token_flags = self.token_flags;
            let code_point = self.scan_unicode_escape();
            self.pos = save_pos;
            self.token_flags = save_token_flags;
            return code_point;
        }
        None
    }

    pub(crate) fn scan_digits(&mut self) -> (String, bool) { ::tsox_core::fntrace::enter("scan_digits"); 
        let start = self.pos;
        let mut is_octal = true;
        while let Some(ch) = self.char() {
            if !ch.is_ascii_digit() {
                break;
            }
            if ch > '7' {
                is_octal = false;
            }
            self.pos += 1;
        }
        (self.text[start..self.pos].to_string(), is_octal)
    }

    pub(crate) fn scan_hex_digits(
        &mut self,
        min_count: usize,
        scan_as_many_as_possible: bool,
        can_have_separators: bool,
    ) -> String { ::tsox_core::fntrace::enter("scan_hex_digits"); 
        let mut digit_count = 0usize;
        let start = self.pos;
        let mut allow_separator = false;
        let mut is_previous_token_separator = false;
        while digit_count < min_count || scan_as_many_as_possible {
            let ch = self.char();
            if ch.map_or(false, is_hex_digit) {
                allow_separator = can_have_separators;
                is_previous_token_separator = false;
                digit_count += 1;
            } else if can_have_separators && ch == Some('_') {
                self.token_flags |= crate::scanner::TOKEN_FLAGS_CONTAINS_SEPARATOR;
                if allow_separator {
                    allow_separator = false;
                    is_previous_token_separator = true;
                } else if is_previous_token_separator {
                    self.error_at(
                        DiagnosticKind::MultipleConsecutiveNumericSeparators,
                        self.pos,
                        1,
                    );
                } else {
                    self.error_at(DiagnosticKind::NumericSeparatorNotAllowed, self.pos, 1);
                }
            } else {
                break;
            }
            self.pos += 1;
        }
        if is_previous_token_separator {
            self.error_at(DiagnosticKind::NumericSeparatorNotAllowed, self.pos - 1, 1);
        }
        if digit_count < min_count {
            return String::new();
        }
        let digits = &self.text[start..self.pos];
        if self.token_flags & crate::scanner::TOKEN_FLAGS_CONTAINS_SEPARATOR != 0 {
            return digits.replace('_', "").to_lowercase();
        }
        digits.to_lowercase()
    }

    pub(crate) fn scan_invalid_character(&mut self) { ::tsox_core::fntrace::enter("scan_invalid_character"); 
        let (_, size) = self.char_and_size();
        self.error_at(DiagnosticKind::InvalidCharacter, self.pos, size);
        self.pos += size;
        self.token = SyntaxKind::Unknown;
    }
}


/// Go stringutil.IsLowSurrogate
fn is_low_surrogate(ch: char) -> bool { ::tsox_core::fntrace::enter("is_low_surrogate"); 
    matches!(ch as u32, 0xDC00..=0xDFFF)
}

/// Go stringutil.SurrogatePairToCodePoint(utf16.DecodeRune)
fn surrogate_pair_to_code_point(high: char, low: char) -> char { ::tsox_core::fntrace::enter("surrogate_pair_to_code_point"); 
    let code_point = 0x10000 + (((high as u32 - 0xD800) << 10) | (low as u32 - 0xDC00));
    char::from_u32(code_point).unwrap_or('\u{FFFD}')
}
