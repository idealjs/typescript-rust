use crate::scanner::impl_chunk::*;

impl Scanner {
    pub(crate) fn scan_bigint_suffix(&mut self) -> SyntaxKind {
        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == 'n' {
            self.pos += 1;
            SyntaxKind::BigIntLiteral
        } else {
            SyntaxKind::NumericLiteral
        }
    }

    pub(crate) fn scan_number(&mut self) -> SyntaxKind {
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
                self.token_end = self.pos;
                self.token = self.scan_bigint_suffix();
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
                self.token_end = self.pos;
                self.token = self.scan_bigint_suffix();
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
                self.token_end = self.pos;
                self.token = self.scan_bigint_suffix();
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
                self.scan_number_fragment_with_sep(false);
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
            self.scan_number_fragment_with_sep(false);
        }

        let fixed_part_end = self.pos;
        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '.' {
            self.pos += 1;
            self.scan_number_fragment_with_sep(false);
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
                let exp_start = self.pos;
                self.scan_number_fragment_with_sep(false);
                if self.pos == exp_start {
                    self.report_error(DiagnosticKind::DigitExpected, self.pos, 0);
                }
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

        let mut result = SyntaxKind::NumericLiteral;
        if fixed_part_end == self.pos {
            result = self.scan_bigint_suffix();
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

    pub(crate) fn scan_number_fragment_with_sep(&mut self, is_hex: bool) {
        let mut allow_separator = false;
        let mut is_prev_separator = false;
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

    pub(crate) fn scan_binary_or_octal_digits(&mut self, base: u8) {
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
