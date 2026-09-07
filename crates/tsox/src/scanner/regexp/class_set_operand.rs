use super::RegExpParser;
use crate::diagnostics;

impl<'a> RegExpParser<'a> {
    pub(super) fn scan_class_set_operand(&mut self) -> String {
        self.may_contain_strings = false;
        let ch = self.char();
        match ch {
            '[' => {
                self.inc_pos(1);
                self.scan_class_set_expression();
                self.scan_expected_char(']');
                String::new()
            }
            '\\' => {
                self.inc_pos(1);
                if self.scan_character_class_escape() {
                    return String::new();
                } else if self.char() == 'q' {
                    self.inc_pos(1);
                    if self.char() == '{' {
                        self.inc_pos(1);
                        self.scan_class_string_disjunction_contents();
                        self.scan_expected_char('}');
                        return String::new();
                    } else {
                        self.error(
                            diagnostics::X_Q_MUST_BE_FOLLOWED_BY_STRING_ALTERNATIVES_ENCLOSED_IN_BRACES,
                            self.pos - 2,
                            2,
                        );
                        return "q".to_string();
                    }
                }
                self.inc_pos(-1);

                self.scan_class_set_character()
            }
            _ => self.scan_class_set_character(),
        }
    }

    pub(super) fn scan_class_string_disjunction_contents(&mut self) {
        let mut character_count = 0;
        while self.pos < self.body_end {
            let ch = self.char();
            match ch {
                '}' => {
                    if character_count != 1 {
                        self.may_contain_strings = true;
                    }
                    return;
                }
                '|' => {
                    if character_count != 1 {
                        self.may_contain_strings = true;
                    }
                    self.inc_pos(1);
                    character_count = 0;
                }
                _ => {
                    self.scan_class_set_character();
                    character_count += 1;
                }
            }
        }
    }

    pub(super) fn scan_class_set_character(&mut self) -> String {
        let ch = self.char();
        if ch == '\\' {
            self.inc_pos(1);
            let inner_ch = self.char();
            match inner_ch {
                'b' => {
                    self.inc_pos(1);
                    return "\u{0008}".to_string();
                }
                '&' | '-' | '!' | '#' | '%' | ',' | ':' | ';' | '<' | '=' | '>' | '@' | '`'
                | '~' => {
                    self.inc_pos(1);
                    return inner_ch.to_string();
                }
                _ => {
                    return self.scan_character_escape(false);
                }
            }
        } else if self.pos + 1 < self.body_end && ch == self.char_at(1) {
            match ch {
                '&' | '!' | '#' | '%' | '*' | '+' | ',' | '.' | ':' | ';' | '<' | '=' | '>'
                | '?' | '@' | '`' | '~' => {
                    self.error(
                        diagnostics::A_CHARACTER_CLASS_MUST_NOT_CONTAIN_A_RESERVED_DOUBLE_PUNCTUATOR_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                        self.pos,
                        2,
                    );
                    self.inc_pos(2);
                    return self.text[self.pos - 2..self.pos].to_string();
                }
                _ => {}
            }
        }
        match ch {
            '/' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '|' => {
                self.error(
                    diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                    self.pos,
                    1,
                );
                self.inc_pos(1);
                ch.to_string()
            }
            _ => self.scan_source_character(),
        }
    }
}
