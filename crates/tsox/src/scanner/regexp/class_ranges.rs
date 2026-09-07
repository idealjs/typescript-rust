use super::RegExpParser;
use super::decode_first_rune;
use crate::diagnostics;

impl<'a> RegExpParser<'a> {
    pub(super) fn scan_class_ranges(&mut self) {
        if self.char() == '^' {
            self.inc_pos(1);
        }
        while self.pos < self.body_end {
            let ch = self.char();
            if self.is_class_content_exit(ch) {
                return;
            }
            let min_start = self.pos;
            let min_character = self.scan_class_atom();
            if self.char() == '-' {
                self.inc_pos(1);
                let ch2 = self.char();
                if self.is_class_content_exit(ch2) {
                    return;
                }
                if min_character.is_empty() && self.any_unicode_mode_or_non_annex_b {
                    self.error(
                        diagnostics::A_CHARACTER_CLASS_RANGE_MUST_NOT_BE_BOUNDED_BY_ANOTHER_CHARACTER_CLASS,
                        min_start,
                        self.pos - 1 - min_start,
                    );
                }
                let max_start = self.pos;
                let max_character = self.scan_class_atom();
                if max_character.is_empty() && self.any_unicode_mode_or_non_annex_b {
                    self.error(
                        diagnostics::A_CHARACTER_CLASS_RANGE_MUST_NOT_BE_BOUNDED_BY_ANOTHER_CHARACTER_CLASS,
                        max_start,
                        self.pos - max_start,
                    );
                    continue;
                }
                if min_character.is_empty() {
                    continue;
                }
                if let (Some((min_c, min_size)), Some((max_c, max_size))) = (
                    decode_first_rune(&min_character),
                    decode_first_rune(&max_character),
                ) {
                    if min_character.len() == min_size
                        && max_character.len() == max_size
                        && (min_c as u32) > (max_c as u32)
                    {
                        self.error(
                            diagnostics::RANGE_OUT_OF_ORDER_IN_CHARACTER_CLASS,
                            min_start,
                            self.pos - min_start,
                        );
                    }
                }
            }
        }
    }

    pub(super) fn scan_class_atom(&mut self) -> String {
        let ch = self.char();
        if ch == '\\' {
            self.inc_pos(1);
            let ch2 = self.char();
            match ch2 {
                'b' => {
                    self.inc_pos(1);
                    "\u{0008}".to_string()
                }
                '-' => {
                    self.inc_pos(1);
                    ch2.to_string()
                }
                _ => {
                    if self.scan_character_class_escape() {
                        return String::new();
                    }
                    self.scan_character_escape(false)
                }
            }
        } else {
            self.scan_source_character()
        }
    }
}
