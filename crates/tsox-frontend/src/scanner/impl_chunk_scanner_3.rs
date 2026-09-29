#![allow(unused_imports)]

use crate::scanner::impl_chunk::*;

impl Scanner {
    pub(crate) fn scan_identifier(&mut self) -> SyntaxKind {
        let start = self.pos;

        let bytes = self.text.as_bytes();
        let first_b = bytes[self.pos];
        let escaped_first = first_b == b'\\';
        if !escaped_first {
            if first_b < 128 {
                self.pos += 1;
                while self.pos < self.end {
                    let b = bytes[self.pos];
                    if b.is_ascii_alphanumeric() || b == b'_' || b == b'$' {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
            } else {
                let first_c = self.text[self.pos..].chars().next().unwrap();
                self.pos += first_c.len_utf8();
            }
        }

        let mut cooked = String::new();
        let mut has_escape = false;
        let mut segment_start = start;
        while self.pos < self.end {
            let c = self.text[self.pos..].chars().next().unwrap();
            if c != '\\' && is_identifier_part(c) {
                self.pos += c.len_utf8();
                continue;
            }
            if c != '\\' {
                break;
            }
            let at_start = !has_escape && escaped_first;
            match self.scan_identifier_escape_part(at_start) {
                Some((escape_start, escaped)) => {
                    cooked.push_str(&self.text[segment_start..escape_start]);
                    cooked.push(escaped);
                    segment_start = self.pos;
                    has_escape = true;
                }
                None => break,
            }
        }
        self.token_end = self.pos;
        if has_escape {
            cooked.push_str(&self.text[segment_start..self.pos]);
            self.identifier_value = Some(cooked.clone());
            self.token = string_to_keyword(&cooked).unwrap_or(SyntaxKind::Identifier);
        } else {
            let text = &self.text[start..self.pos];
            self.token = string_to_keyword(text).unwrap_or(SyntaxKind::Identifier);
        }
        self.token
    }

    pub(crate) fn scan_identifier_escape_part(
        &mut self,
        at_identifier_start: bool,
    ) -> Option<(usize, char)> {
        let escape_start = self.pos;
        let next = self.text.as_bytes().get(self.pos + 1).copied();
        if next != Some(b'u') {
            return None;
        }
        let escaped = self.scan_unicode_escape()?;
        let valid = if at_identifier_start {
            is_identifier_start(escaped)
        } else {
            is_identifier_part(escaped)
        };
        if !valid {
            self.pos = escape_start;
            return None;
        }
        Some((escape_start, escaped))
    }

    pub(crate) fn scan_unicode_escape(&mut self) -> Option<char> {
        let escape_start = self.pos;
        let bytes = self.text.as_bytes();
        if bytes.get(self.pos) != Some(&b'\\') || bytes.get(self.pos + 1) != Some(&b'u') {
            return None;
        }
        self.pos += 2;
        let mut digits = String::new();
        if bytes.get(self.pos) == Some(&b'{') {
            self.pos += 1;
            while self.pos < self.end {
                let b = bytes[self.pos];
                if b == b'}' {
                    break;
                }
                digits.push(b as char);
                self.pos += 1;
            }
            if bytes.get(self.pos) != Some(&b'}')
                || digits.is_empty()
                || digits.len() > 6
                || !digits.chars().all(|d| d.is_ascii_hexdigit())
            {
                self.pos = escape_start;
                return None;
            }
            self.pos += 1;
            self.token_flags |= TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE;
        } else {
            for _ in 0..4 {
                match bytes.get(self.pos) {
                    Some(b) if b.is_ascii_hexdigit() => {
                        digits.push(*b as char);
                        self.pos += 1;
                    }
                    _ => {
                        self.pos = escape_start;
                        return None;
                    }
                }
            }
            self.token_flags |= TOKEN_FLAGS_UNICODE_ESCAPE;
        }
        let code = u32::from_str_radix(&digits, 16).ok()?;
        char::from_u32(code)
    }

    pub(crate) fn scan_private_identifier(&mut self) -> SyntaxKind {
        self.pos += 1;

        if self.pos < self.end {
            let next_c = self.text[self.pos..].chars().next().unwrap();
            if is_identifier_start(next_c) {
                self.pos += next_c.len_utf8();
                while self.pos < self.end {
                    let c = self.text[self.pos..].chars().next().unwrap();
                    if !is_identifier_part(c) {
                        break;
                    }
                    self.pos += c.len_utf8();
                }
            } else {
                self.report_error(DiagnosticKind::InvalidCharacter, self.pos - 1, 1);
            }
        }
        self.token_end = self.pos;
        self.token = SyntaxKind::PrivateIdentifier;
        self.token
    }
}
