#![allow(unused_imports)]

use super::*;

impl<'a> RegExpParser<'a> {
    pub(super) fn scan_identifier_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.body_end {
            let (c, size) = decode_rune_at(self.text, self.pos);
            if !is_identifier_part(c) {
                break;
            }
            self.pos += size;
        }
        self.text[start..self.pos].to_string()
    }
}
