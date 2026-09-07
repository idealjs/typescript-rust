#![allow(unused_imports)]

use super::*;

impl Scanner {
    pub fn re_scan_slash_token(&mut self) -> SyntaxKind {
        if self.token != SyntaxKind::SlashToken && self.token != SyntaxKind::SlashEqualsToken {
            return self.token;
        }

        let start_of_regex_body = self.token_pos + 1;
        let mut p = start_of_regex_body;
        let mut in_escape = false;
        let mut in_character_class = false;
        let mut unterminated = false;

        let mut named_capture_groups = false;

        while p < self.end {
            let c = self.text.as_bytes()[p] as char;
            if is_line_break(c) {
                unterminated = true;
                break;
            }
            if in_escape {
                in_escape = false;
                p += 1;
                continue;
            }
            match c {
                '\\' => {
                    in_escape = true;
                }
                '/' if !in_character_class => {
                    break;
                }
                '[' => {
                    in_character_class = true;
                }
                ']' if in_character_class => {
                    in_character_class = false;
                }
                '(' if !in_character_class
                    && p + 3 < self.end
                    && self.text.as_bytes()[p + 1] as char == '?'
                    && self.text.as_bytes()[p + 2] as char == '<'
                    && self.text.as_bytes()[p + 3] as char != '='
                    && self.text.as_bytes()[p + 3] as char != '!' =>
                {
                    named_capture_groups = true;
                }
                _ => {}
            }
            p += 1;
        }

        let end_of_regex_body = p;

        if unterminated || p >= self.end {
            self.token_flags |= TOKEN_FLAGS_UNTERMINATED;
            self.report_error(
                DiagnosticKind::UnterminatedRegularExpression,
                self.token_pos,
                p - self.token_pos,
            );
            self.pos = p;
        } else {
            p += 1;

            let mut seen_flags: u16 = 0;
            while p < self.end {
                let c = self.text[p..].chars().next().unwrap_or('\0');
                let c_len = c.len_utf8();
                if !is_identifier_part(c) {
                    break;
                }
                if let Some(bit) = reg_exp_flag_bit(c) {
                    if seen_flags & bit != 0 {
                        self.report_error(DiagnosticKind::DuplicateRegularExpressionFlag, p, 1);
                    } else if (seen_flags | bit) & (REG_EXP_FLAG_U | REG_EXP_FLAG_V)
                        == (REG_EXP_FLAG_U | REG_EXP_FLAG_V)
                    {
                        self.report_error(DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive, p, 1);
                    } else {
                        seen_flags |= bit;

                        self.check_reg_exp_flag_availability(bit, p);
                    }
                } else {
                    self.report_error(DiagnosticKind::UnknownRegularExpressionFlag, p, c_len);
                }
                p += c_len;
            }
            self.pos = p;

            let mut parser = regexp::RegExpParser::new(
                &self.text,
                start_of_regex_body,
                end_of_regex_body,
                seen_flags,
                named_capture_groups,
                self.script_target,
            );
            parser.run();
            for err in parser.errors() {
                self.errors.push(*err);
            }
        }

        self.token_end = self.pos;
        self.token = SyntaxKind::RegularExpressionLiteral;
        self.token
    }

    pub(crate) fn check_reg_exp_flag_availability(&mut self, flag: u16, pos: usize) {
        let available_from = match flag {
            REG_EXP_FLAG_D => Some(ScriptTarget::ES2022),
            REG_EXP_FLAG_S => Some(ScriptTarget::ES2018),
            REG_EXP_FLAG_V => Some(ScriptTarget::ES2024),
            _ => None,
        };
        if let Some(target) = available_from {
            if self.script_target < target {
                self.report_error(
                    DiagnosticKind::RegexMessage(
                        crate::diagnostics::THIS_REGULAR_EXPRESSION_FLAG_IS_ONLY_AVAILABLE_WHEN_TARGETING_0_OR_LATER,
                    ),
                    pos,
                    1,
                );
            }
        }
    }

    pub fn scan_jsx_token(&mut self) -> SyntaxKind {
        self.scan_jsx_token_ex(true)
    }

    pub fn scan_jsx_token_ex(&mut self, allow_multiline_jsx_text: bool) -> SyntaxKind {
        self.has_preceding_line_break = self.preceding_line_break;
        self.preceding_line_break = false;
        self.token_pos = self.pos;

        if self.pos >= self.end {
            self.token_end = self.pos;
            self.token = SyntaxKind::EndOfFile;
            return self.token;
        }

        let c = self.text.as_bytes()[self.pos] as char;

        if c == '<' {
            if self.pos + 1 < self.end && self.text.as_bytes()[self.pos + 1] == b'/' {
                self.pos += 2;
            } else {
                self.pos += 1;
            }
            self.token_end = self.pos;
            self.token = if c == '<'
                && self.pos > self.token_pos + 1
                && self.text.as_bytes()[self.token_pos + 1] == b'/'
            {
                SyntaxKind::LessThanSlashToken
            } else {
                SyntaxKind::LessThanToken
            };
            return self.token;
        }

        if c == '{' {
            self.pos += 1;
            self.token_end = self.pos;
            self.token = SyntaxKind::OpenBraceToken;
            return self.token;
        }

        let mut first_non_whitespace = 0usize;
        let start = self.pos;

        while self.pos < self.end {
            let ch = self.text[self.pos..].chars().next().unwrap();
            let size = ch.len_utf8();

            if ch == '{' || ch == '<' {
                break;
            }

            if is_jsx_line_break(ch) && first_non_whitespace == 0 {
                first_non_whitespace = usize::MAX;
            } else if !allow_multiline_jsx_text
                && is_jsx_line_break(ch)
                && first_non_whitespace > 0
                && first_non_whitespace != usize::MAX
            {
                break;
            } else if !is_jsx_whitespace_like(ch) {
                first_non_whitespace = self.pos;
            }

            self.pos += size;
        }

        self.token_end = self.pos;
        self.token = if first_non_whitespace == usize::MAX {
            SyntaxKind::JsxTextAllWhiteSpaces
        } else {
            SyntaxKind::JsxText
        };
        let _ = start;
        self.token
    }

    pub fn scan_jsx_identifier(&mut self) -> SyntaxKind {
        if is_identifier_or_keyword_token(self.token) {
            loop {
                if self.pos >= self.end {
                    break;
                }
                let c = self.text.as_bytes()[self.pos] as char;
                if c == '-' {
                    self.pos += 1;
                    continue;
                }

                let old_pos = self.pos;
                if is_identifier_part(c) {
                    self.pos += c.len_utf8();
                    while self.pos < self.end {
                        let next = self.text[self.pos..].chars().next().unwrap();
                        if is_identifier_part(next) {
                            self.pos += next.len_utf8();
                        } else {
                            break;
                        }
                    }
                }
                if self.pos == old_pos {
                    break;
                }
            }
            self.token_end = self.pos;

            let text = self.token_text();
            self.token = string_to_keyword(text).unwrap_or(SyntaxKind::Identifier);
        }
        self.token
    }

    pub fn scan_jsx_attribute_value(&mut self) -> SyntaxKind {
        while self.pos < self.end {
            let c = self.text[self.pos..].chars().next().unwrap();
            if !is_jsx_whitespace_like(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
        self.token_pos = self.pos;

        if self.pos >= self.end {
            self.token_end = self.pos;
            self.token = SyntaxKind::EndOfFile;
            return self.token;
        }

        let c = self.text.as_bytes()[self.pos] as char;
        if c == '"' || c == '\'' {
            return self.scan_string(c);
        }

        self.scan()
    }

    pub fn can_follow_jsdoc_at(&self) -> bool {
        if self.pos >= self.end {
            return true;
        }
        let (ch, _) = decode_char(&self.text, self.pos);
        is_identifier_start(ch) || is_whitespace_single_line(ch) || is_line_break(ch)
    }
}
