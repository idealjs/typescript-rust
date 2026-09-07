use super::base64::base64_format_decode;
use super::mapping::{
    MISSING_LINE_OR_COLUMN, MISSING_NAME, MISSING_SOURCE, MISSING_UTF16_COLUMN, Mapping, NameIndex,
    SourceIndex,
};

pub struct MappingsDecoder<'a> {
    mappings: &'a str,
    pos: usize,
    done: bool,
    generated_line: i32,
    generated_character: i32,
    source_index: SourceIndex,
    source_line: i32,
    source_character: i32,
    name_index: NameIndex,
    error: Option<String>,
}

impl<'a> MappingsDecoder<'a> {
    pub fn new(mappings: &'a str) -> Self {
        MappingsDecoder {
            mappings,
            pos: 0,
            done: false,
            generated_line: 0,
            generated_character: 0,
            source_index: 0,
            source_line: 0,
            source_character: 0,
            name_index: 0,
            error: None,
        }
    }

    pub fn mappings_string(&self) -> &str {
        self.mappings
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn state(&self) -> Mapping {
        self.capture_mapping(true, true)
    }

    fn is_source_mapping_segment_end(&self) -> bool {
        self.pos == self.mappings.len()
            || self.mappings.as_bytes()[self.pos] == b','
            || self.mappings.as_bytes()[self.pos] == b';'
    }

    fn base64_vlq_format_decode(&mut self) -> i32 {
        let mut shift_count = 0;
        let mut value = 0;
        loop {
            if self.pos >= self.mappings.len() {
                self.error = Some(
                    "Error in decoding base64VLQFormatDecode, past the mapping string".to_string(),
                );
                return -1;
            }
            let current_byte = base64_format_decode(self.mappings.as_bytes()[self.pos]);
            self.pos += 1;
            if current_byte == -1 {
                self.error = Some("Invalid character in VLQ".to_string());
                return -1;
            }
            let more_digits = (current_byte & 32) != 0;
            value |= (current_byte & 31) << shift_count;
            shift_count += 5;
            if !more_digits {
                break;
            }
        }
        if (value & 1) == 0 {
            value >> 1
        } else {
            -(value >> 1)
        }
    }

    fn capture_mapping(&self, has_source: bool, has_name: bool) -> Mapping {
        Mapping {
            generated_line: self.generated_line,
            generated_character: self.generated_character,
            source_index: if has_source {
                self.source_index
            } else {
                MISSING_SOURCE
            },
            source_line: if has_source {
                self.source_line
            } else {
                MISSING_LINE_OR_COLUMN
            },
            source_character: if has_source {
                self.source_character
            } else {
                MISSING_UTF16_COLUMN
            },
            name_index: if has_name {
                self.name_index
            } else {
                MISSING_NAME
            },
        }
    }

    pub fn next(&mut self) -> Option<Mapping> {
        while !self.done && self.pos < self.mappings.len() {
            let ch = self.mappings.as_bytes()[self.pos];
            if ch == b';' {
                self.generated_line += 1;
                self.generated_character = 0;
                self.pos += 1;
                continue;
            }
            if ch == b',' {
                self.pos += 1;
                continue;
            }

            let mut has_source = false;
            let mut has_name = false;

            self.generated_character += self.base64_vlq_format_decode();
            if self.error.is_some() {
                self.done = true;
                return None;
            }
            if self.generated_character < 0 {
                self.error = Some("Invalid generatedCharacter found".to_string());
                self.done = true;
                return None;
            }

            if !self.is_source_mapping_segment_end() {
                has_source = true;

                self.source_index += self.base64_vlq_format_decode();
                if self.error.is_some() {
                    self.done = true;
                    return None;
                }
                if self.source_index < 0 {
                    self.error = Some("Invalid sourceIndex found".to_string());
                    self.done = true;
                    return None;
                }
                if self.is_source_mapping_segment_end() {
                    self.error =
                        Some("Unsupported Format: No entries after sourceIndex".to_string());
                    self.done = true;
                    return None;
                }

                self.source_line += self.base64_vlq_format_decode();
                if self.error.is_some() {
                    self.done = true;
                    return None;
                }
                if self.source_line < 0 {
                    self.error = Some("Invalid sourceLine found".to_string());
                    self.done = true;
                    return None;
                }
                if self.is_source_mapping_segment_end() {
                    self.error =
                        Some("Unsupported Format: No entries after sourceLine".to_string());
                    self.done = true;
                    return None;
                }

                self.source_character += self.base64_vlq_format_decode();
                if self.error.is_some() {
                    self.done = true;
                    return None;
                }
                if self.source_character < 0 {
                    self.error = Some("Invalid sourceCharacter found".to_string());
                    self.done = true;
                    return None;
                }

                if !self.is_source_mapping_segment_end() {
                    has_name = true;
                    self.name_index += self.base64_vlq_format_decode();
                    if self.error.is_some() {
                        self.done = true;
                        return None;
                    }
                    if self.name_index < 0 {
                        self.error = Some("Invalid nameIndex found".to_string());
                        self.done = true;
                        return None;
                    }
                    if !self.is_source_mapping_segment_end() {
                        self.error =
                            Some("Unsupported Error Format: Entries after nameIndex".to_string());
                        self.done = true;
                        return None;
                    }
                }
            }

            return Some(self.capture_mapping(has_source, has_name));
        }
        self.done = true;
        None
    }

    pub fn collect_all(mut self) -> (Vec<Mapping>, Option<String>) {
        let mut result = Vec::new();
        while let Some(m) = self.next() {
            result.push(m);
        }
        (result, self.error)
    }
}
