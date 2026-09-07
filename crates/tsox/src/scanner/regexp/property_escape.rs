use super::RegExpParser;
use crate::diagnostics;
use crate::scanner::unicode_properties;

impl<'a> RegExpParser<'a> {
    pub(super) fn scan_character_class_escape(&mut self) -> bool {
        let mut is_character_complement = false;
        let start = self.pos - 1;
        let ch = self.char();
        match ch {
            'd' | 'D' | 's' | 'S' | 'w' | 'W' => {
                self.inc_pos(1);
                return true;
            }
            'P' => {
                is_character_complement = true;
            }
            'p' => {}
            _ => return false,
        }

        self.inc_pos(1);
        if self.char() == '{' {
            self.inc_pos(1);
            let property_name_or_value_start = self.pos;
            let property_name_or_value = self.scan_word_characters();
            if self.char() == '=' {
                let property_name =
                    unicode_properties::non_binary_property_canonical(&property_name_or_value)
                        .unwrap_or("");
                if self.pos == property_name_or_value_start {
                    self.error(diagnostics::EXPECTED_A_UNICODE_PROPERTY_NAME, self.pos, 0);
                } else if property_name.is_empty() {
                    self.error(
                        diagnostics::UNKNOWN_UNICODE_PROPERTY_NAME,
                        property_name_or_value_start,
                        self.pos - property_name_or_value_start,
                    );
                }
                self.inc_pos(1);
                let property_value_start = self.pos;
                let property_value = self.scan_word_characters();
                if self.pos == property_value_start {
                    self.error(diagnostics::EXPECTED_A_UNICODE_PROPERTY_VALUE, self.pos, 0);
                } else if !property_name.is_empty() {
                    if !unicode_properties::is_valid_unicode_property_value(
                        property_name,
                        &property_value,
                    ) {
                        self.error(
                            diagnostics::UNKNOWN_UNICODE_PROPERTY_VALUE,
                            property_value_start,
                            self.pos - property_value_start,
                        );
                    }
                }
            } else {
                if self.pos == property_name_or_value_start {
                    self.error(
                        diagnostics::EXPECTED_A_UNICODE_PROPERTY_NAME_OR_VALUE,
                        self.pos,
                        0,
                    );
                } else if unicode_properties::is_binary_unicode_property_of_strings(
                    &property_name_or_value,
                ) {
                    if !self.unicode_sets_mode {
                        self.error(
                            diagnostics::ANY_UNICODE_PROPERTY_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_ONLY_AVAILABLE_WHEN_THE_UNICODE_SETS_V_FLAG_IS_SET,
                            property_name_or_value_start,
                            self.pos - property_name_or_value_start,
                        );
                    } else if is_character_complement {
                        self.error(
                            diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                            property_name_or_value_start,
                            self.pos - property_name_or_value_start,
                        );
                    } else {
                        self.may_contain_strings = true;
                    }
                } else if !unicode_properties::is_valid_unicode_property_value(
                    "General_Category",
                    &property_name_or_value,
                ) && !unicode_properties::is_binary_unicode_property(
                    &property_name_or_value,
                ) {
                    self.error(
                        diagnostics::UNKNOWN_UNICODE_PROPERTY_NAME_OR_VALUE,
                        property_name_or_value_start,
                        self.pos - property_name_or_value_start,
                    );
                }
            }
            self.scan_expected_char('}');
            if !self.any_unicode_mode {
                self.error(
                    diagnostics::UNICODE_PROPERTY_VALUE_EXPRESSIONS_ARE_ONLY_AVAILABLE_WHEN_THE_UNICODE_U_FLAG_OR_THE_UNICODE_SETS_V_FLAG_IS_SET,
                    start,
                    self.pos - start,
                );
            }
        } else if self.any_unicode_mode_or_non_annex_b {
            self.error(
                diagnostics::X_0_MUST_BE_FOLLOWED_BY_A_UNICODE_PROPERTY_VALUE_EXPRESSION_ENCLOSED_IN_BRACES,
                self.pos - 2,
                2,
            );
        } else {
            self.inc_pos(-1);
            return false;
        }
        true
    }
}
