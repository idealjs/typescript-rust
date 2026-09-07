use super::RegExpParser;
use super::{
    REG_EXP_FLAG_MODIFIERS, char_to_reg_exp_flag, compare_decimal_strings, decode_rune_at,
    is_digit, is_word_character,
};
use crate::core::compiler_options::ScriptTarget;
use crate::diagnostics;
use crate::scanner::is_identifier_part;
use std::collections::HashSet;

impl<'a> RegExpParser<'a> {
    pub(super) fn scan_disjunction(&mut self, is_in_group: bool) {
        loop {
            self.named_capturing_groups.push(HashSet::new());
            self.scan_alternative(is_in_group);
            self.named_capturing_groups.pop();
            if self.char() != '|' {
                return;
            }
            self.inc_pos(1);
        }
    }

    pub(super) fn scan_alternative(&mut self, is_in_group: bool) {
        let mut is_previous_term_quantifiable = false;
        while self.pos < self.body_end {
            let start = self.pos;
            let ch = self.char();
            match ch {
                '^' | '$' => {
                    self.inc_pos(1);
                    is_previous_term_quantifiable = false;
                }
                '\\' => {
                    self.inc_pos(1);
                    match self.char() {
                        'b' | 'B' => {
                            self.inc_pos(1);
                            is_previous_term_quantifiable = false;
                        }
                        _ => {
                            self.scan_atom_escape();
                            is_previous_term_quantifiable = true;
                        }
                    }
                }
                '(' => {
                    self.inc_pos(1);
                    if self.char() == '?' {
                        self.inc_pos(1);
                        match self.char() {
                            '=' | '!' => {
                                self.inc_pos(1);

                                is_previous_term_quantifiable =
                                    !self.any_unicode_mode_or_non_annex_b;
                            }
                            '<' => {
                                let group_name_start = self.pos;
                                self.inc_pos(1);
                                match self.char() {
                                    '=' | '!' => {
                                        self.inc_pos(1);
                                        is_previous_term_quantifiable = false;
                                    }
                                    _ => {
                                        self.scan_group_name(false);
                                        self.scan_expected_char('>');
                                        if self.script_target < ScriptTarget::ES2018 {
                                            self.error(
                                                diagnostics::NAMED_CAPTURING_GROUPS_ARE_ONLY_AVAILABLE_WHEN_TARGETING_ES2018_OR_LATER,
                                                group_name_start,
                                                self.pos - group_name_start,
                                            );
                                        }
                                        self.number_of_capturing_groups += 1;
                                        is_previous_term_quantifiable = true;
                                    }
                                }
                            }
                            _ => {
                                let flags_start = self.pos;
                                let set_flags = self.scan_pattern_modifiers(0);
                                if self.char() == '-' {
                                    self.inc_pos(1);
                                    self.scan_pattern_modifiers(set_flags);
                                    if self.pos == flags_start + 1 {
                                        self.error(
                                            diagnostics::SUBPATTERN_FLAGS_MUST_BE_PRESENT_WHEN_THERE_IS_A_MINUS_SIGN,
                                            flags_start,
                                            self.pos - flags_start,
                                        );
                                    }
                                }
                                self.scan_expected_char(':');
                                is_previous_term_quantifiable = true;
                            }
                        }
                    } else {
                        self.number_of_capturing_groups += 1;
                        is_previous_term_quantifiable = true;
                    }
                    self.scan_disjunction(true);
                    self.scan_expected_char(')');
                }
                '{' => {
                    self.inc_pos(1);
                    let digits_start = self.pos;
                    let min_str = self.scan_digits();
                    if !self.any_unicode_mode_or_non_annex_b && min_str.is_empty() {
                        is_previous_term_quantifiable = true;
                        continue;
                    }
                    if self.char() == ',' {
                        self.inc_pos(1);
                        let max_str = self.scan_digits();
                        if min_str.is_empty() {
                            if !max_str.is_empty() || self.char() == '}' {
                                self.error(
                                    diagnostics::INCOMPLETE_QUANTIFIER_DIGIT_EXPECTED,
                                    digits_start,
                                    0,
                                );
                            } else {
                                self.error(
                                    diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                                    start,
                                    1,
                                );
                                is_previous_term_quantifiable = true;
                                continue;
                            }
                        } else if !max_str.is_empty() {
                            if compare_decimal_strings(&min_str, &max_str) > 0
                                && (self.any_unicode_mode_or_non_annex_b || self.char() == '}')
                            {
                                self.error(
                                    diagnostics::NUMBERS_OUT_OF_ORDER_IN_QUANTIFIER,
                                    digits_start,
                                    self.pos - digits_start,
                                );
                            }
                        }
                    } else if min_str.is_empty() {
                        if self.any_unicode_mode_or_non_annex_b {
                            self.error(
                                diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                                start,
                                1,
                            );
                        }
                        is_previous_term_quantifiable = true;
                        continue;
                    }
                    if self.char() != '}' {
                        if self.any_unicode_mode_or_non_annex_b {
                            self.error(diagnostics::X_0_EXPECTED, self.pos, 0);
                            self.inc_pos(-1);
                        } else {
                            is_previous_term_quantifiable = true;
                            continue;
                        }
                    }

                    self.inc_pos(1);
                    if self.char() == '?' {
                        self.inc_pos(1);
                    }
                    if !is_previous_term_quantifiable {
                        self.error(
                            diagnostics::THERE_IS_NOTHING_AVAILABLE_FOR_REPETITION,
                            start,
                            self.pos - start,
                        );
                    }
                    is_previous_term_quantifiable = false;
                }
                '*' | '+' | '?' => {
                    self.inc_pos(1);
                    if self.char() == '?' {
                        self.inc_pos(1);
                    }
                    if !is_previous_term_quantifiable {
                        self.error(
                            diagnostics::THERE_IS_NOTHING_AVAILABLE_FOR_REPETITION,
                            start,
                            self.pos - start,
                        );
                    }
                    is_previous_term_quantifiable = false;
                }
                '.' => {
                    self.inc_pos(1);
                    is_previous_term_quantifiable = true;
                }
                '[' => {
                    self.inc_pos(1);
                    if self.unicode_sets_mode {
                        self.scan_class_set_expression();
                    } else {
                        self.scan_class_ranges();
                    }
                    self.scan_expected_char(']');
                    is_previous_term_quantifiable = true;
                }
                ')' => {
                    if is_in_group {
                        return;
                    }

                    if self.any_unicode_mode_or_non_annex_b || ch == ')' {
                        self.error(
                            diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                            self.pos,
                            1,
                        );
                    }
                    self.inc_pos(1);
                    is_previous_term_quantifiable = true;
                }
                ']' | '}' => {
                    if self.any_unicode_mode_or_non_annex_b || ch == ')' {
                        self.error(
                            diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                            self.pos,
                            1,
                        );
                    }
                    self.inc_pos(1);
                    is_previous_term_quantifiable = true;
                }
                '/' | '|' => return,
                _ => {
                    self.scan_source_character();
                    is_previous_term_quantifiable = true;
                }
            }
        }
    }

    pub(super) fn scan_pattern_modifiers(&mut self, curr_flags: u16) -> u16 {
        let mut curr = curr_flags;
        while self.pos < self.body_end {
            let (ch, size) = decode_rune_at(self.text, self.pos);
            if !is_identifier_part(ch) {
                break;
            }
            let flag = char_to_reg_exp_flag(ch);
            if flag == 0 {
                self.error(diagnostics::UNKNOWN_REGULAR_EXPRESSION_FLAG, self.pos, size);
            } else if curr & flag != 0 {
                self.error(
                    diagnostics::DUPLICATE_REGULAR_EXPRESSION_FLAG,
                    self.pos,
                    size,
                );
            } else if flag & REG_EXP_FLAG_MODIFIERS == 0 {
                self.error(
                    diagnostics::THIS_REGULAR_EXPRESSION_FLAG_CANNOT_BE_TOGGLED_WITHIN_A_SUBPATTERN,
                    self.pos,
                    size,
                );
            } else {
                curr |= flag;
                self.check_regular_expression_flag_availability(flag, self.pos, size);
            }
            self.inc_pos(size as i32);
        }
        curr
    }

    pub(super) fn scan_digits(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.body_end && is_digit(self.char()) {
            self.inc_pos(1);
        }
        self.text[start..self.pos].to_string()
    }

    pub(super) fn scan_word_characters(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.body_end && is_word_character(self.char()) {
            self.inc_pos(1);
        }
        self.text[start..self.pos].to_string()
    }
}
