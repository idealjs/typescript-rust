use super::base64::base64_format_encode;
use super::generator::Generator;
use super::mapping::{NameIndex, SourceIndex};

const SOURCE_INDEX_NOT_SET: SourceIndex = -1;
const NAME_INDEX_NOT_SET: NameIndex = -1;
const NOT_SET: i32 = -1;
const NOT_SET_UTF16: i32 = -1;

impl Generator {
    pub(super) fn is_new_generated_position(
        &self,
        generated_line: i32,
        generated_character: i32,
    ) -> bool {
        !self.has_pending
            || self.pending_generated_line != generated_line
            || self.pending_generated_character != generated_character
    }

    pub(super) fn is_backtracking_source_position(
        &self,
        source_index: SourceIndex,
        source_line: i32,
        source_character: i32,
    ) -> bool {
        source_index != SOURCE_INDEX_NOT_SET
            && source_line != NOT_SET
            && source_character != NOT_SET_UTF16
            && self.pending_source_index == source_index
            && (self.pending_source_line > source_line
                || (self.pending_source_line == source_line
                    && self.pending_source_character > source_character))
    }

    pub(super) fn should_commit_mapping(&self) -> bool {
        if !self.has_pending {
            return false;
        }
        if !self.has_last {
            return true;
        }
        self.last_generated_line != self.pending_generated_line
            || self.last_generated_character != self.pending_generated_character
            || self.last_source_index != self.pending_source_index
            || self.last_source_line != self.pending_source_line
            || self.last_source_character != self.pending_source_character
            || self.last_name_index != self.pending_name_index
    }

    fn append_base64_vlq(&mut self, in_value: i32) {
        let mut in_value = if in_value < 0 {
            ((-in_value) << 1) + 1
        } else {
            in_value << 1
        };
        loop {
            let current_digit = in_value & 31;
            in_value >>= 5;
            let digit = if in_value > 0 {
                current_digit | 32
            } else {
                current_digit
            };
            self.mappings.push(base64_format_encode(digit));
            if in_value <= 0 {
                break;
            }
        }
    }

    pub(super) fn commit_pending_mapping(&mut self) {
        if !self.should_commit_mapping() {
            return;
        }

        if self.last_generated_line < self.pending_generated_line {
            loop {
                self.mappings.push(';');
                self.last_generated_line += 1;
                if self.last_generated_line >= self.pending_generated_line {
                    break;
                }
            }
            self.last_generated_character = 0;
        } else {
            assert_eq!(
                self.last_generated_line, self.pending_generated_line,
                "generatedLine cannot backtrack"
            );
            if self.has_last {
                self.mappings.push(',');
            }
        }

        self.append_base64_vlq(self.pending_generated_character - self.last_generated_character);
        self.last_generated_character = self.pending_generated_character;

        if self.has_pending_source {
            self.append_base64_vlq(self.pending_source_index - self.last_source_index);
            self.last_source_index = self.pending_source_index;

            self.append_base64_vlq(self.pending_source_line - self.last_source_line);
            self.last_source_line = self.pending_source_line;

            self.append_base64_vlq(self.pending_source_character - self.last_source_character);
            self.last_source_character = self.pending_source_character;

            if self.has_pending_name {
                self.append_base64_vlq(self.pending_name_index - self.last_name_index);
                self.last_name_index = self.pending_name_index;
            }
        }

        self.has_last = true;
    }

    fn add_mapping(
        &mut self,
        generated_line: i32,
        generated_character: i32,
        source_index: SourceIndex,
        source_line: i32,
        source_character: i32,
        name_index: NameIndex,
    ) {
        if self.is_new_generated_position(generated_line, generated_character)
            || self.is_backtracking_source_position(source_index, source_line, source_character)
        {
            self.commit_pending_mapping();
            self.pending_generated_line = generated_line;
            self.pending_generated_character = generated_character;
            self.has_pending_source = false;
            self.has_pending_name = false;
            self.has_pending = true;
        }

        if source_index != SOURCE_INDEX_NOT_SET
            && source_line != NOT_SET
            && source_character != NOT_SET_UTF16
        {
            self.pending_source_index = source_index;
            self.pending_source_line = source_line;
            self.pending_source_character = source_character;
            self.has_pending_source = true;
            if name_index != NAME_INDEX_NOT_SET {
                self.pending_name_index = name_index;
                self.has_pending_name = true;
            }
        }
    }

    pub fn add_generated_mapping(
        &mut self,
        generated_line: i32,
        generated_character: i32,
    ) -> Result<(), String> {
        if generated_line < self.pending_generated_line {
            return Err("generatedLine cannot backtrack".to_string());
        }
        if generated_character < 0 {
            return Err("generatedCharacter cannot be negative".to_string());
        }
        self.add_mapping(
            generated_line,
            generated_character,
            SOURCE_INDEX_NOT_SET,
            NOT_SET,
            NOT_SET_UTF16,
            NAME_INDEX_NOT_SET,
        );
        Ok(())
    }

    pub fn add_source_mapping(
        &mut self,
        generated_line: i32,
        generated_character: i32,
        source_index: SourceIndex,
        source_line: i32,
        source_character: i32,
    ) -> Result<(), String> {
        if generated_line < self.pending_generated_line {
            return Err("generatedLine cannot backtrack".to_string());
        }
        if generated_character < 0 {
            return Err("generatedCharacter cannot be negative".to_string());
        }
        if source_index < 0 || source_index as usize >= self.sources.len() {
            return Err("sourceIndex is out of range".to_string());
        }
        if source_line < 0 {
            return Err("sourceLine cannot be negative".to_string());
        }
        if source_character < 0 {
            return Err("sourceCharacter cannot be negative".to_string());
        }
        self.add_mapping(
            generated_line,
            generated_character,
            source_index,
            source_line,
            source_character,
            NAME_INDEX_NOT_SET,
        );
        Ok(())
    }

    pub fn add_named_source_mapping(
        &mut self,
        generated_line: i32,
        generated_character: i32,
        source_index: SourceIndex,
        source_line: i32,
        source_character: i32,
        name_index: NameIndex,
    ) -> Result<(), String> {
        if generated_line < self.pending_generated_line {
            return Err("generatedLine cannot backtrack".to_string());
        }
        if generated_character < 0 {
            return Err("generatedCharacter cannot be negative".to_string());
        }
        if source_index < 0 || source_index as usize >= self.sources.len() {
            return Err("sourceIndex is out of range".to_string());
        }
        if source_line < 0 {
            return Err("sourceLine cannot be negative".to_string());
        }
        if source_character < 0 {
            return Err("sourceCharacter cannot be negative".to_string());
        }
        if name_index < 0 || name_index as usize >= self.names.len() {
            return Err("nameIndex is out of range".to_string());
        }
        self.add_mapping(
            generated_line,
            generated_character,
            source_index,
            source_line,
            source_character,
            name_index,
        );
        Ok(())
    }
}
