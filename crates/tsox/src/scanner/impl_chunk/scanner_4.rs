#![allow(unused_imports)]

use super::*;

impl Scanner {
    pub(crate) fn scan_number_fragment_with_sep(&mut self, is_hex: bool, _can_have_sep: bool) {
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
                }
                self.pos += 1;
                continue;
            }
            break;
        }
        if is_prev_separator {
            self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
        }
    }

    pub(crate) fn scan_binary_fragment_with_sep(&mut self) {
        let mut allow_separator = false;
        let mut is_prev_separator = false;
        loop {
            let before = self.pos;
            while self.pos < self.end {
                let c = self.text.as_bytes()[self.pos] as char;
                if c == '0' || c == '1' {
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
                }
                self.pos += 1;
                continue;
            }
            break;
        }
        if is_prev_separator {
            self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
        }
    }

    pub(crate) fn scan_octal_specifier_fragment_with_sep(&mut self) {
        let mut allow_separator = false;
        let mut is_prev_separator = false;
        loop {
            let before = self.pos;
            while self.pos < self.end {
                let c = self.text.as_bytes()[self.pos] as char;
                if is_octal_digit(c) {
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
                }
                self.pos += 1;
                continue;
            }
            break;
        }
        if is_prev_separator {
            self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
        }
    }

    pub(crate) fn scan_string(&mut self, quote: char) -> SyntaxKind {
        if quote == '\'' {
            self.token_flags |= TOKEN_FLAGS_SINGLE_QUOTE;
        }
        self.pos += 1;
        let mut terminated = false;
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == quote {
                self.pos += 1;
                terminated = true;
                break;
            }
            if c == '\\' {
                self.scan_escape_sequence();
                continue;
            }
            if c == '\n' || c == '\r' {
                break;
            }
            self.pos += 1;
        }
        if !terminated {
            self.token_flags |= TOKEN_FLAGS_UNTERMINATED;
            self.report_error(
                DiagnosticKind::UnterminatedStringLiteral,
                self.token_pos,
                self.pos - self.token_pos,
            );
        }
        self.token_end = self.pos;
        self.token = SyntaxKind::StringLiteral;
        self.token
    }

    pub(crate) fn scan_escape_sequence(&mut self) {
        self.pos += 1;
        if self.pos >= self.end {
            return;
        }
        let c = self.text.as_bytes()[self.pos] as char;
        self.pos += 1;
        match c {
            '0' => {
                if self.pos < self.end && is_digit(self.text.as_bytes()[self.pos] as char) {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;

                    for _ in 0..2 {
                        if self.pos < self.end
                            && is_octal_digit(self.text.as_bytes()[self.pos] as char)
                        {
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
            '1'..='3' => {
                self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                for _ in 0..2 {
                    if self.pos < self.end && is_octal_digit(self.text.as_bytes()[self.pos] as char)
                    {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
            }
            '4'..='7' => {
                self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                if self.pos < self.end && is_octal_digit(self.text.as_bytes()[self.pos] as char) {
                    self.pos += 1;
                }
            }
            '8' | '9' => {
                self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
            }
            'x' => {
                let mut digit_count = 0;
                for _ in 0..2 {
                    if self.pos < self.end && is_hex_digit(self.text.as_bytes()[self.pos] as char) {
                        self.pos += 1;
                        digit_count += 1;
                    } else {
                        break;
                    }
                }
                if digit_count == 2 {
                    self.token_flags |= TOKEN_FLAGS_HEX_ESCAPE;
                } else {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                }
            }
            'u' => {
                if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '{' {
                    self.pos += 1;
                    let hex_start = self.pos;
                    while self.pos < self.end
                        && is_hex_digit(self.text.as_bytes()[self.pos] as char)
                    {
                        self.pos += 1;
                    }
                    let has_hex = self.pos > hex_start;
                    let closed =
                        self.pos < self.end && self.text.as_bytes()[self.pos] as char == '}';
                    if closed {
                        self.pos += 1;
                    }
                    if has_hex && closed {
                        self.token_flags |= TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE;
                    } else {
                        self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                    }
                } else {
                    let mut digit_count = 0;
                    for _ in 0..4 {
                        if self.pos < self.end
                            && is_hex_digit(self.text.as_bytes()[self.pos] as char)
                        {
                            self.pos += 1;
                            digit_count += 1;
                        } else {
                            break;
                        }
                    }
                    if digit_count == 4 {
                        self.token_flags |= TOKEN_FLAGS_UNICODE_ESCAPE;
                    } else {
                        self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                    }
                }
            }
            '\r' => {
                if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '\n' {
                    self.pos += 1;
                }
            }

            _ => {}
        }
    }
}
