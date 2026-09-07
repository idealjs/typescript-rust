#![allow(unused_imports)]

use super::*;

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
            self.identifier_value = Some(cooked);
            self.token = SyntaxKind::Identifier;
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

    pub(crate) fn scan_number(&mut self) -> SyntaxKind {
        let start = self.pos;
        if self.text.as_bytes()[self.pos] as char == '0' && self.pos + 1 < self.end {
            let next = self.text.as_bytes()[self.pos + 1] as char;
            if next == 'x' || next == 'X' {
                self.pos += 2;
                self.scan_number_fragment_with_sep(true, true);
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_HEX_SPECIFIER;
                return self.token;
            }
            if next == 'b' || next == 'B' {
                self.pos += 2;
                self.scan_binary_fragment_with_sep();
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_BINARY_SPECIFIER;
                return self.token;
            }
            if next == 'o' || next == 'O' {
                self.pos += 2;
                self.scan_octal_specifier_fragment_with_sep();
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_OCTAL_SPECIFIER;
                return self.token;
            }
        }

        if self.text.as_bytes()[self.pos] as char == '0' {
            self.pos += 1;
            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {
                self.token_flags |=
                    TOKEN_FLAGS_CONTAINS_SEPARATOR | TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                self.report_error(DiagnosticKind::NumericSeparatorNotAllowed, self.pos, 1);
                self.pos = start;
                self.scan_number_fragment_with_sep(false, false);
            } else {
                let digits_start = self.pos;
                let mut is_octal = true;
                while self.pos < self.end {
                    let c = self.text.as_bytes()[self.pos] as char;
                    if is_digit(c) {
                        if !is_octal_digit(c) {
                            is_octal = false;
                        }
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                if self.pos > digits_start && is_octal {
                    self.token_flags |= TOKEN_FLAGS_OCTAL;
                    let with_minus = self.token == SyntaxKind::MinusToken;
                    let err_start = if with_minus { start - 1 } else { start };
                    self.report_error(
                        DiagnosticKind::OctalLiteralNotAllowed,
                        err_start,
                        self.pos - err_start,
                    );
                    self.token_end = self.pos;
                    self.token = SyntaxKind::NumericLiteral;
                    return self.token;
                } else if self.pos > digits_start {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_LEADING_ZERO;
                }
            }
        } else {
            self.scan_number_fragment_with_sep(false, false);
        }

        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '.' {
            self.pos += 1;
            self.scan_number_fragment_with_sep(false, false);
        }

        if self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == 'e' || c == 'E' {
                self.pos += 1;
                if self.pos < self.end {
                    let sign = self.text.as_bytes()[self.pos] as char;
                    if sign == '+' || sign == '-' {
                        self.pos += 1;
                    }
                }
                self.scan_number_fragment_with_sep(false, false);
                self.token_flags |= TOKEN_FLAGS_SCIENTIFIC;
            }
        }

        if token_flags_contains(self.token_flags, TOKEN_FLAGS_CONTAINS_LEADING_ZERO) {
            self.report_error(
                DiagnosticKind::DecimalWithLeadingZero,
                start,
                self.pos - start,
            );
        }

        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == 'n' {
            self.pos += 1;
            self.token_end = self.pos;
            self.token = SyntaxKind::BigIntLiteral;
            return self.token;
        }

        let _ = start;
        self.token_end = self.pos;
        self.token = SyntaxKind::NumericLiteral;
        self.token
    }
}
