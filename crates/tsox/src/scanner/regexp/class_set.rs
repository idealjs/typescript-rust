use super::ClassSetExpressionType;
use super::RegExpParser;
use super::decode_first_rune;
use crate::diagnostics;

impl<'a> RegExpParser<'a> {
    pub(super) fn scan_class_set_expression(&mut self) {
        let mut is_character_complement = false;
        if self.char() == '^' {
            self.inc_pos(1);
            is_character_complement = true;
        }
        let mut expression_may_contain_strings = false;
        let ch = self.char();
        if self.is_class_content_exit(ch) {
            return;
        }
        let mut start = self.pos;
        let operand: String;
        let two = self.two_chars_at(self.pos);
        if matches!(two, Some([b'-', b'-']) | Some([b'&', b'&'])) {
            self.error(diagnostics::EXPECTED_A_CLASS_SET_OPERAND, self.pos, 0);
            self.may_contain_strings = false;
            operand = String::new();
        } else {
            operand = self.scan_class_set_operand();
        }

        match self.char() {
            '-' => {
                if self.pos + 1 < self.body_end && self.char_at(1) == '-' {
                    if is_character_complement && self.may_contain_strings {
                        self.error(
                            diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                            start,
                            self.pos - start,
                        );
                    }
                    expression_may_contain_strings = self.may_contain_strings;
                    self.scan_class_set_sub_expression(ClassSetExpressionType::ClassSubtraction);
                    self.may_contain_strings =
                        !is_character_complement && expression_may_contain_strings;
                    return;
                }
            }
            '&' => {
                if self.pos + 1 < self.body_end && self.char_at(1) == '&' {
                    self.scan_class_set_sub_expression(ClassSetExpressionType::ClassIntersection);
                    if is_character_complement && self.may_contain_strings {
                        self.error(
                            diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                            start,
                            self.pos - start,
                        );
                    }
                    expression_may_contain_strings = self.may_contain_strings;
                    self.may_contain_strings =
                        !is_character_complement && expression_may_contain_strings;
                    return;
                } else {
                    self.error(
                        diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                        self.pos,
                        1,
                    );
                }
            }
            _ => {
                if is_character_complement && self.may_contain_strings {
                    self.error(
                        diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                        start,
                        self.pos - start,
                    );
                }
                expression_may_contain_strings = self.may_contain_strings;
            }
        }

        let mut operand = operand;
        while self.pos < self.body_end {
            let ch = self.char();
            match ch {
                '-' => {
                    self.inc_pos(1);
                    let ch2 = self.char();
                    if self.is_class_content_exit(ch2) {
                        self.may_contain_strings =
                            !is_character_complement && expression_may_contain_strings;
                        return;
                    }
                    if ch2 == '-' {
                        self.inc_pos(1);
                        self.error(
                            diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                            self.pos - 2,
                            2,
                        );
                        start = self.pos - 2;
                        operand = self.text[start..self.pos].to_string();
                        continue;
                    } else {
                        if operand.is_empty() {
                            self.error(
                                diagnostics::A_CHARACTER_CLASS_RANGE_MUST_NOT_BE_BOUNDED_BY_ANOTHER_CHARACTER_CLASS,
                                start,
                                self.pos - 1 - start,
                            );
                        }
                        let second_start = self.pos;
                        let second_operand = self.scan_class_set_operand();
                        if is_character_complement && self.may_contain_strings {
                            self.error(
                                diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                                second_start,
                                self.pos - second_start,
                            );
                        }
                        expression_may_contain_strings =
                            expression_may_contain_strings || self.may_contain_strings;
                        if second_operand.is_empty() {
                            self.error(
                                diagnostics::A_CHARACTER_CLASS_RANGE_MUST_NOT_BE_BOUNDED_BY_ANOTHER_CHARACTER_CLASS,
                                second_start,
                                self.pos - second_start,
                            );
                        } else if !operand.is_empty() {
                            if let (Some((min_c, min_size)), Some((max_c, max_size))) = (
                                decode_first_rune(&operand),
                                decode_first_rune(&second_operand),
                            ) {
                                if operand.len() == min_size
                                    && second_operand.len() == max_size
                                    && (min_c as u32) > (max_c as u32)
                                {
                                    self.error(
                                        diagnostics::RANGE_OUT_OF_ORDER_IN_CHARACTER_CLASS,
                                        start,
                                        self.pos - start,
                                    );
                                }
                            }
                        }
                    }
                }
                '&' => {
                    start = self.pos;
                    self.inc_pos(1);
                    if self.char() == '&' {
                        self.inc_pos(1);
                        self.error(
                            diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                            self.pos - 2,
                            2,
                        );
                        if self.char() == '&' {
                            self.error(
                                diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                                self.pos,
                                1,
                            );
                            self.inc_pos(1);
                        }
                    } else {
                        self.error(
                            diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                            self.pos - 1,
                            1,
                        );
                    }
                    operand = self.text[start..self.pos].to_string();
                    continue;
                }
                _ => {}
            }
            if self.is_class_content_exit(self.char()) {
                break;
            }
            start = self.pos;
            let two = self.two_chars_at(self.pos);
            if matches!(two, Some([b'-', b'-']) | Some([b'&', b'&'])) {
                self.error(
                    diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                    self.pos,
                    2,
                );
                self.inc_pos(2);
                operand = self.text[start..self.pos].to_string();
            } else {
                operand = self.scan_class_set_operand();
            }
        }
        self.may_contain_strings = !is_character_complement && expression_may_contain_strings;
    }

    pub(super) fn scan_class_set_sub_expression(
        &mut self,
        expression_type: ClassSetExpressionType,
    ) {
        let mut expression_may_contain_strings = self.may_contain_strings;
        while self.pos < self.body_end {
            let ch = self.char();
            if self.is_class_content_exit(ch) {
                break;
            }
            match ch {
                '-' => {
                    self.inc_pos(1);
                    if self.char() == '-' {
                        self.inc_pos(1);
                        if expression_type != ClassSetExpressionType::ClassSubtraction {
                            self.error(
                                diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                                self.pos - 2,
                                2,
                            );
                        }
                    } else {
                        self.error(
                            diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                            self.pos - 1,
                            1,
                        );
                    }
                }
                '&' => {
                    self.inc_pos(1);
                    if self.char() == '&' {
                        self.inc_pos(1);
                        if expression_type != ClassSetExpressionType::ClassIntersection {
                            self.error(
                                diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                                self.pos - 2,
                                2,
                            );
                        }
                        if self.char() == '&' {
                            self.error(
                                diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                                self.pos,
                                1,
                            );
                            self.inc_pos(1);
                        }
                    } else {
                        self.error(
                            diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                            self.pos - 1,
                            1,
                        );
                    }
                }
                _ => {
                    if expression_type == ClassSetExpressionType::ClassSubtraction
                        || expression_type == ClassSetExpressionType::ClassIntersection
                    {
                        self.error(diagnostics::X_0_EXPECTED, self.pos, 0);
                    }
                }
            }
            let ch2 = self.char();
            if self.is_class_content_exit(ch2) {
                self.error(diagnostics::EXPECTED_A_CLASS_SET_OPERAND, self.pos, 0);
                break;
            }
            self.scan_class_set_operand();
            if expression_type == ClassSetExpressionType::ClassIntersection {
                expression_may_contain_strings =
                    expression_may_contain_strings && self.may_contain_strings;
            }
        }
        self.may_contain_strings = expression_may_contain_strings;
    }
}
