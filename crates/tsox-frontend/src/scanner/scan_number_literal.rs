use crate::scanner::impl_chunk::*;
use tsox_core::jsnum::Number;

impl Scanner {
    pub(crate) fn scan_bigint_suffix(&mut self) -> SyntaxKind { ::tsox_core::fntrace::enter("scan_bigint_suffix"); 
        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == 'n' {
            self.pos += 1;
            SyntaxKind::BigIntLiteral
        } else {
            SyntaxKind::NumericLiteral
        }
    }

    pub(crate) fn scan_number(&mut self) -> SyntaxKind { ::tsox_core::fntrace::enter("scan_number"); 
        let start = self.pos;
        if self.text.as_bytes()[self.pos] as char == '0' && self.pos + 1 < self.end {
            let next = self.text.as_bytes()[self.pos + 1] as char;
            if next == 'x' || next == 'X' {
                self.pos += 2;
                let digits_start = self.pos;
                self.scan_number_fragment_with_sep(true);
                if self.pos == digits_start {
                    self.report_error(DiagnosticKind::HexadecimalDigitExpected, self.pos, 0);
                }
                self.token_flags |= TOKEN_FLAGS_HEX_SPECIFIER;
                self.token = self.scan_bigint_suffix();
                self.token_end = self.pos;
                return self.token;
            }
            if next == 'b' || next == 'B' {
                self.pos += 2;
                let digits_start = self.pos;
                self.scan_binary_or_octal_digits(2);
                if self.pos == digits_start {
                    self.report_error(DiagnosticKind::BinaryDigitExpected, self.pos, 0);
                }
                self.token_flags |= TOKEN_FLAGS_BINARY_SPECIFIER;
                self.token = self.scan_bigint_suffix();
                self.token_end = self.pos;
                return self.token;
            }
            if next == 'o' || next == 'O' {
                self.pos += 2;
                let digits_start = self.pos;
                self.scan_binary_or_octal_digits(8);
                if self.pos == digits_start {
                    self.report_error(DiagnosticKind::OctalDigitExpected, self.pos, 0);
                }
                self.token_flags |= TOKEN_FLAGS_OCTAL_SPECIFIER;
                self.token = self.scan_bigint_suffix();
                self.token_end = self.pos;
                return self.token;
            }
        }

        let fixed_part;
        if self.text.as_bytes()[self.pos] as char == '0' {
            self.pos += 1;
            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {
                self.token_flags |=
                    TOKEN_FLAGS_CONTAINS_SEPARATOR | TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                self.report_error(DiagnosticKind::NumericSeparatorNotAllowed, self.pos, 1);
                self.pos = start;
                fixed_part = self.scan_number_fragment_with_sep(false);
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
                    // Go scanNumber：legacy 八进制 tokenValue 为八进制值的
                    // 十进制串（ParseInt 越界钳到 MaxInt64），节点文本取
                    // TokenValue，字面量类型随之呈十进制值
                    let digits = &self.text[digits_start..self.pos];
                    let val = i64::from_str_radix(digits, 8).unwrap_or(i64::MAX);
                    self.token_value_override = Some(val.to_string());
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
                let digits = self.text[digits_start..self.pos].to_string();
                fixed_part = if digits.is_empty() {
                    "0".to_string()
                } else {
                    digits
                };
            }
        } else {
            fixed_part = self.scan_number_fragment_with_sep(false);
        }

        let fixed_part_end = self.pos;
        let mut fractional_part = String::new();
        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '.' {
            self.pos += 1;
            fractional_part = self.scan_number_fragment_with_sep(false);
        }

        let mut end = self.pos;
        let mut exponent_part = String::new();
        let mut exponent_preamble = String::new();
        if self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == 'e' || c == 'E' {
                self.pos += 1;
                self.token_flags |= TOKEN_FLAGS_SCIENTIFIC;
                if self.pos < self.end {
                    let sign = self.text.as_bytes()[self.pos] as char;
                    if sign == '+' || sign == '-' {
                        self.pos += 1;
                    }
                }
                let start_numeric_part = self.pos;
                exponent_part = self.scan_number_fragment_with_sep(false);
                if exponent_part.is_empty() {
                    self.report_error(DiagnosticKind::DigitExpected, self.pos, 0);
                } else {
                    exponent_preamble = self.text[end..start_numeric_part].to_string();
                    end = self.pos;
                }
            }
        }

        let token_value = if token_flags_contains(self.token_flags, TOKEN_FLAGS_CONTAINS_SEPARATOR)
        {
            let mut v = fixed_part;
            if !fractional_part.is_empty() {
                v.push('.');
                v.push_str(&fractional_part);
            }
            if !exponent_part.is_empty() {
                v.push_str(&exponent_preamble);
                v.push_str(&exponent_part);
            }
            v
        } else {
            self.text[start..end].to_string()
        };

        if token_flags_contains(self.token_flags, TOKEN_FLAGS_CONTAINS_LEADING_ZERO) {
            self.report_error(
                DiagnosticKind::DecimalWithLeadingZero,
                start,
                self.pos - start,
            );
            self.token_value_override = Some(Number::from_string(&token_value).to_string());
            self.token_end = self.pos;
            self.token = SyntaxKind::NumericLiteral;
            return self.token;
        }

        let mut result = SyntaxKind::NumericLiteral;
        if fixed_part_end == self.pos {
            result = self.scan_bigint_suffix();
            if result == SyntaxKind::NumericLiteral {
                let token_text = &self.text[start..self.pos];
                if token_text != token_value {
                    self.token_value_override = Some(token_value);
                }
            }
        } else {
            self.token_value_override = Some(Number::from_string(&token_value).to_string());
        }

        if self.pos < self.end {
            let c = self.text[self.pos..].chars().next().unwrap();
            if is_identifier_start(c) {
                let id_start = self.pos;
                while self.pos < self.end {
                    let ch = self.text[self.pos..].chars().next().unwrap();
                    if is_identifier_part(ch) {
                        self.pos += ch.len_utf8();
                    } else {
                        break;
                    }
                }
                let id = &self.text[id_start..self.pos];
                if result != SyntaxKind::BigIntLiteral && id == "n" {
                    if token_flags_contains(self.token_flags, TOKEN_FLAGS_SCIENTIFIC) {
                        self.report_error(
                            DiagnosticKind::BigIntExponentialNotation,
                            start,
                            self.pos - start,
                        );
                        self.token_end = self.pos;
                        self.token = result;
                        return self.token;
                    }
                    if fixed_part_end < id_start {
                        self.report_error(
                            DiagnosticKind::BigIntMustBeInteger,
                            start,
                            self.pos - start,
                        );
                        self.token_end = self.pos;
                        self.token = result;
                        return self.token;
                    }
                }
                self.report_error(
                    DiagnosticKind::IdentifierFollowsNumeric,
                    id_start,
                    self.pos - id_start,
                );
                self.pos = id_start;
            }
        }

        self.token_end = self.pos;
        self.token = result;
        self.token
    }

    pub(crate) fn scan_number_fragment_with_sep(&mut self, is_hex: bool) -> String { ::tsox_core::fntrace::enter("scan_number_fragment_with_sep");
        let mut allow_separator = false;
        let mut is_prev_separator = false;
        let mut result = String::new();
        let mut seg_start = self.pos;
        loop {
            let before = self.pos;

            while self.pos < self.end {
                let c = self.text.as_bytes()[self.pos] as char;
                if is_digit(c) || (is_hex && c.is_ascii_hexdigit()) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.pos > before {
                allow_separator = true;
                is_prev_separator = false;
            }

            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {
                self.token_flags |= TOKEN_FLAGS_CONTAINS_SEPARATOR;
                if allow_separator {
                    allow_separator = false;
                    is_prev_separator = true;
                    result.push_str(&self.text[seg_start..self.pos]);
                } else {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                    if is_prev_separator {
                        self.report_error(
                            DiagnosticKind::MultipleConsecutiveNumericSeparators,
                            self.pos,
                            1,
                        );
                    } else {
                        self.report_error(DiagnosticKind::NumericSeparatorNotAllowed, self.pos, 1);
                    }
                }
                self.pos += 1;
                seg_start = self.pos;
                continue;
            }
            break;
        }
        if is_prev_separator {
            self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
            self.report_error(DiagnosticKind::NumericSeparatorNotAllowed, self.pos - 1, 1);
        }
        if result.is_empty() {
            return self.text[seg_start..self.pos].to_string();
        }
        result.push_str(&self.text[seg_start..self.pos]);
        result
    }

    pub(crate) fn scan_binary_or_octal_digits(&mut self, base: u8) { ::tsox_core::fntrace::enter("scan_binary_or_octal_digits"); 
        let mut allow_separator = false;
        let mut is_prev_separator = false;
        loop {
            let before = self.pos;
            while self.pos < self.end {
                let b = self.text.as_bytes()[self.pos];
                if b.is_ascii_digit() && b - b'0' < base {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.pos > before {
                allow_separator = true;
                is_prev_separator = false;
            }
            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {
                self.token_flags |= TOKEN_FLAGS_CONTAINS_SEPARATOR;
                if allow_separator {
                    allow_separator = false;
                    is_prev_separator = true;
                } else {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                    if is_prev_separator {
                        self.report_error(
                            DiagnosticKind::MultipleConsecutiveNumericSeparators,
                            self.pos,
                            1,
                        );
                    } else {
                        self.report_error(DiagnosticKind::NumericSeparatorNotAllowed, self.pos, 1);
                    }
                }
                self.pos += 1;
                continue;
            }
            break;
        }
        if is_prev_separator {
            self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
            self.report_error(DiagnosticKind::NumericSeparatorNotAllowed, self.pos - 1, 1);
        }
    }
}
