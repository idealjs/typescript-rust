#![allow(unused_imports)]

use super::*;

impl<'a> RegExpParser<'a> {
    pub(crate) fn scan_atom_escape(&mut self) {
        let ch = self.char();
        if ch == 'k' {
            self.inc_pos(1);
            if self.char() == '<' {
                self.inc_pos(1);
                self.scan_group_name(true);
                self.scan_expected_char('>');
            } else if self.any_unicode_mode_or_non_annex_b || self.named_capture_groups {
                self.error(
                    diagnostics::X_K_MUST_BE_FOLLOWED_BY_A_CAPTURING_GROUP_NAME_ENCLOSED_IN_ANGLE_BRACKETS,
                    self.pos - 2,
                    2,
                );
            }
            return;
        }
        if ch == 'q' && self.unicode_sets_mode {
            self.inc_pos(1);
            self.error(
                diagnostics::X_Q_IS_ONLY_AVAILABLE_INSIDE_CHARACTER_CLASS,
                self.pos - 2,
                2,
            );
            return;
        }

        if !self.scan_character_class_escape() && !self.scan_decimal_escape() {
            let _ = self.scan_character_escape(true);
        }
    }

    pub(super) fn scan_decimal_escape(&mut self) -> bool {
        let ch = self.char();
        if ('1'..='9').contains(&ch) {
            let start = self.pos;
            let digits = self.scan_digits();
            let val = digits.parse::<i32>().unwrap_or(i32::MAX);
            self.decimal_escapes.push(DecimalEscapeValue {
                pos: start,
                end: self.pos,
                value: val,
            });
            return true;
        }
        false
    }

    pub(crate) fn scan_character_escape(&mut self, atom_escape: bool) -> String {
        if self.pos >= self.body_end {
            self.error(diagnostics::UNDETERMINED_CHARACTER_ESCAPE, self.pos - 1, 1);
            return "\\".to_string();
        }
        let ch = self.char();
        match ch {
            'c' => {
                self.inc_pos(1);
                let c2 = self.char();
                if is_ascii_letter(c2) {
                    self.inc_pos(1);
                    let code = (c2 as u32) & 0x1f;
                    return char::from_u32(code)
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| code.to_string());
                }
                if self.any_unicode_mode_or_non_annex_b {
                    self.error(
                        diagnostics::X_C_MUST_BE_FOLLOWED_BY_AN_ASCII_LETTER,
                        self.pos - 2,
                        2,
                    );
                } else if atom_escape {
                    self.inc_pos(-1);
                    return "\\".to_string();
                }
                c2.to_string()
            }
            '^' | '$' | '/' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
            | '|' => {
                self.inc_pos(1);
                ch.to_string()
            }
            _ => {
                self.inc_pos(-1);
                self.scan_escape_sequence(self.annex_b, self.any_unicode_mode, atom_escape)
            }
        }
    }

    pub(super) fn scan_escape_sequence(
        &mut self,
        annex_b: bool,
        any_unicode_mode: bool,
        atom_escape: bool,
    ) -> String {
        let start = self.pos;
        self.inc_pos(1);
        if self.pos >= self.body_end {
            self.error(diagnostics::UNDETERMINED_CHARACTER_ESCAPE, start, 1);
            return "\\".to_string();
        }
        let ch = self.char();
        self.inc_pos(1);
        match ch {
            '0' => {
                if !is_digit(self.char()) {
                    return "\0".to_string();
                }

                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                self.report_octal_escape(start, '0', atom_escape);
                self.text[start..self.pos].to_string()
            }
            '1' | '2' | '3' => {
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                self.report_octal_escape(start, ch, atom_escape);
                self.text[start..self.pos].to_string()
            }
            '4' | '5' | '6' | '7' => {
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                self.report_octal_escape(start, ch, atom_escape);
                self.text[start..self.pos].to_string()
            }
            '8' | '9' => {
                if !atom_escape {
                    self.error(
                        diagnostics::DECIMAL_ESCAPE_SEQUENCES_AND_BACKREFERENCES_ARE_NOT_ALLOWED_IN_A_CHARACTER_CLASS,
                        start,
                        self.pos - start,
                    );
                }
                ch.to_string()
            }
            'b' => "\u{0008}".to_string(),
            't' => "\t".to_string(),
            'n' => "\n".to_string(),
            'v' => "\u{000B}".to_string(),
            'f' => "\u{000C}".to_string(),
            'r' => "\r".to_string(),
            '\'' => "'".to_string(),
            '"' => "\"".to_string(),
            'x' => {
                let hex_start = self.pos;
                for _ in 0..2 {
                    if is_hex_digit(self.char()) {
                        self.inc_pos(1);
                    } else {
                        break;
                    }
                }
                let hex = &self.text[hex_start..self.pos];
                if let Ok(n) = u32::from_str_radix(hex, 16) {
                    if let Some(c) = char::from_u32(n) {
                        return c.to_string();
                    }
                }
                self.text[start..self.pos].to_string()
            }
            'u' => {
                if self.char() == '{' {
                    self.inc_pos(1);
                    let hex_start = self.pos;
                    while is_hex_digit(self.char()) {
                        self.inc_pos(1);
                    }
                    let hex = &self.text[hex_start..self.pos];
                    if self.char() == '}' {
                        self.inc_pos(1);
                    }
                    if !any_unicode_mode {
                        self.error(
                            diagnostics::UNICODE_ESCAPE_SEQUENCES_ARE_ONLY_AVAILABLE_WHEN_THE_UNICODE_U_FLAG_OR_THE_UNICODE_SETS_V_FLAG_IS_SET,
                            start,
                            self.pos - start,
                        );
                    }
                    if let Ok(n) = u32::from_str_radix(hex, 16) {
                        if let Some(c) = char::from_u32(n) {
                            return c.to_string();
                        }
                    }
                    self.text[start..self.pos].to_string()
                } else {
                    let hex_start = self.pos;
                    for _ in 0..4 {
                        if is_hex_digit(self.char()) {
                            self.inc_pos(1);
                        } else {
                            break;
                        }
                    }
                    let hex = &self.text[hex_start..self.pos];
                    if hex.len() == 4 {
                        if let Ok(n) = u32::from_str_radix(hex, 16) {
                            if let Some(c) = char::from_u32(n) {
                                return c.to_string();
                            }
                        }
                    }
                    self.text[start..self.pos].to_string()
                }
            }
            '\r' => {
                if self.char() == '\n' {
                    self.inc_pos(1);
                }
                String::new()
            }
            '\n' => String::new(),
            _ => {
                let byte_pos = start + 1;
                self.set_pos(byte_pos);
                let (c, size) = decode_rune_at(self.text, byte_pos);
                self.set_pos(byte_pos + size);
                if c == '\u{2028}' || c == '\u{2029}' {
                    return String::new();
                }
                if any_unicode_mode || (!annex_b && is_identifier_part(c)) {
                    self.error(
                        diagnostics::THIS_CHARACTER_CANNOT_BE_ESCAPED_IN_A_REGULAR_EXPRESSION,
                        start,
                        self.pos - start,
                    );
                }
                c.to_string()
            }
        }
    }

    pub(super) fn report_octal_escape(&mut self, start: usize, ch: char, atom_escape: bool) {
        if !atom_escape && ch != '0' {
            self.error(
                diagnostics::OCTAL_ESCAPE_SEQUENCES_AND_BACKREFERENCES_ARE_NOT_ALLOWED_IN_A_CHARACTER_CLASS_IF_THIS_WAS_INTENDED_AS_AN_ESCAPE_SEQUENCE_USE_THE_SYNTAX_0_INSTEAD,
                start,
                self.pos - start,
            );
        }
    }

    pub(crate) fn scan_group_name(&mut self, is_reference: bool) {
        let token_start = self.pos;
        let name = self.scan_identifier_name();
        if self.pos == token_start {
            self.error(diagnostics::EXPECTED_A_CAPTURING_GROUP_NAME, self.pos, 0);
        } else if is_reference {
            self.group_name_references.push(GroupNameReference {
                pos: token_start,
                end: self.pos,
                name,
            });
        } else if self.named_capturing_groups_contains(&name) {
            self.error(
                diagnostics::NAMED_CAPTURING_GROUPS_WITH_THE_SAME_NAME_MUST_BE_MUTUALLY_EXCLUSIVE_TO_EACH_OTHER,
                token_start,
                self.pos - token_start,
            );
        } else {
            if let Some(last) = self.named_capturing_groups.last_mut() {
                last.insert(name.clone());
            }
            self.group_specifiers.insert(name);
        }
    }

    pub(super) fn named_capturing_groups_contains(&self, name: &str) -> bool {
        self.named_capturing_groups.iter().any(|g| g.contains(name))
    }
}
