//! Source map generation and decoding, ported from `internal/sourcemap/`.
//!
//! Implements the [Source Map Version 3](https://sourcemaps.info/spec.html)
//! encoding: Base64 VLQ relative-delta mappings for generated ↔ source
//! position associations.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::tspath::{self, ComparePathsOptions};

pub type SourceIndex = i32;
pub type NameIndex = i32;

const SOURCE_INDEX_NOT_SET: SourceIndex = -1;
const NAME_INDEX_NOT_SET: NameIndex = -1;
const NOT_SET: i32 = -1;
const NOT_SET_UTF16: i32 = -1;

pub const MISSING_SOURCE: SourceIndex = -1;
pub const MISSING_NAME: NameIndex = -1;
pub const MISSING_LINE_OR_COLUMN: i32 = -1;
pub const MISSING_UTF16_COLUMN: i32 = -1;

/// A decoded source map mapping entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    pub generated_line: i32,
    pub generated_character: i32,
    pub source_index: SourceIndex,
    pub source_line: i32,
    pub source_character: i32,
    pub name_index: NameIndex,
}

impl Mapping {
    pub fn is_source_mapping(&self) -> bool {
        self.source_index != MISSING_SOURCE
            && self.source_line != MISSING_LINE_OR_COLUMN
            && self.source_character != MISSING_UTF16_COLUMN
    }
}

/// The raw source map JSON structure (version 3).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RawSourceMap {
    pub version: i32,
    pub file: String,
    #[serde(default, rename = "sourceRoot")]
    pub source_root: String,
    pub sources: Vec<String>,
    pub names: Vec<String>,
    pub mappings: String,
    #[serde(default, rename = "sourcesContent", skip_serializing_if = "Vec::is_empty")]
    pub sources_content: Vec<Option<String>>,
}

/// Source map generator. Mirrors `sourcemap.Generator` in Go.
pub struct Generator {
    path_options: ComparePathsOptions,
    file: String,
    source_root: String,
    sources_directory_path: String,
    raw_sources: Vec<String>,
    sources: Vec<String>,
    source_to_source_index_map: HashMap<String, SourceIndex>,
    sources_content: Vec<Option<String>>,
    names: Vec<String>,
    name_to_name_index_map: HashMap<String, NameIndex>,
    mappings: String,
    last_generated_line: i32,
    last_generated_character: i32,
    last_source_index: SourceIndex,
    last_source_line: i32,
    last_source_character: i32,
    last_name_index: NameIndex,
    has_last: bool,
    pending_generated_line: i32,
    pending_generated_character: i32,
    pending_source_index: SourceIndex,
    pending_source_line: i32,
    pending_source_character: i32,
    pending_name_index: NameIndex,
    has_pending: bool,
    has_pending_source: bool,
    has_pending_name: bool,
}

impl Generator {
    pub fn new(
        file: &str,
        source_root: &str,
        sources_directory_path: &str,
        options: ComparePathsOptions,
    ) -> Self {
        Generator {
            path_options: options,
            file: file.to_string(),
            source_root: source_root.to_string(),
            sources_directory_path: sources_directory_path.to_string(),
            raw_sources: Vec::new(),
            sources: Vec::new(),
            source_to_source_index_map: HashMap::new(),
            sources_content: Vec::new(),
            names: Vec::new(),
            name_to_name_index_map: HashMap::new(),
            mappings: String::new(),
            last_generated_line: 0,
            last_generated_character: 0,
            last_source_index: 0,
            last_source_line: 0,
            last_source_character: 0,
            last_name_index: 0,
            has_last: false,
            pending_generated_line: 0,
            pending_generated_character: 0,
            pending_source_index: 0,
            pending_source_line: 0,
            pending_source_character: 0,
            pending_name_index: 0,
            has_pending: false,
            has_pending_source: false,
            has_pending_name: false,
        }
    }

    pub fn sources(&self) -> &[String] {
        &self.raw_sources
    }

    pub fn add_source(&mut self, file_name: &str) -> SourceIndex {
        let source = tspath::get_relative_path_to_directory_or_url(
            &self.sources_directory_path,
            file_name,
            true,
            &self.path_options,
        );
        if let Some(&idx) = self.source_to_source_index_map.get(&source) {
            return idx;
        }
        let idx = self.sources.len() as SourceIndex;
        self.sources.push(source.clone());
        self.raw_sources.push(file_name.to_string());
        self.source_to_source_index_map.insert(source, idx);
        idx
    }

    pub fn set_source_content(
        &mut self,
        source_index: SourceIndex,
        content: &str,
    ) -> Result<(), String> {
        if source_index < 0 || source_index as usize >= self.sources.len() {
            return Err("sourceIndex is out of range".to_string());
        }
        let idx = source_index as usize;
        if self.sources_content.len() <= idx {
            self.sources_content.resize(idx + 1, None);
        }
        self.sources_content[idx] = Some(content.to_string());
        Ok(())
    }

    pub fn add_name(&mut self, name: &str) -> NameIndex {
        if let Some(&idx) = self.name_to_name_index_map.get(name) {
            return idx;
        }
        let idx = self.names.len() as NameIndex;
        self.names.push(name.to_string());
        self.name_to_name_index_map.insert(name.to_string(), idx);
        idx
    }

    fn is_new_generated_position(&self, generated_line: i32, generated_character: i32) -> bool {
        !self.has_pending
            || self.pending_generated_line != generated_line
            || self.pending_generated_character != generated_character
    }

    fn is_backtracking_source_position(
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

    fn should_commit_mapping(&self) -> bool {
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

    fn commit_pending_mapping(&mut self) {
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

    pub fn raw_source_map(&mut self) -> RawSourceMap {
        self.commit_pending_mapping();
        RawSourceMap {
            version: 3,
            file: self.file.clone(),
            source_root: self.source_root.clone(),
            sources: self.sources.clone(),
            names: self.names.clone(),
            mappings: self.mappings.clone(),
            sources_content: self.sources_content.clone(),
        }
    }

    pub fn to_json(&mut self) -> String {
        let map = self.raw_source_map();
        crate::json::marshal(&map).unwrap_or_default()
    }

    pub fn to_base64_data_url(&mut self) -> String {
        let json = self.to_json();
        use base64::{engine::general_purpose, Engine as _};
        let encoded = general_purpose::STANDARD.encode(json.as_bytes());
        format!("data:application/json;base64,{encoded}")
    }
}

fn base64_format_encode(value: i32) -> char {
    match value {
        0..=25 => (b'A' + value as u8) as char,
        26..=51 => (b'a' + (value - 26) as u8) as char,
        52..=61 => (b'0' + (value - 52) as u8) as char,
        62 => '+',
        63 => '/',
        _ => panic!("not a base64 value: {value}"),
    }
}

fn base64_format_decode(ch: u8) -> i32 {
    match ch {
        b'A'..=b'Z' => (ch - b'A') as i32,
        b'a'..=b'z' => (ch - b'a' + 26) as i32,
        b'0'..=b'9' => (ch - b'0' + 52) as i32,
        b'+' => 62,
        b'/' => 63,
        _ => -1,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Decoder
// ────────────────────────────────────────────────────────────────────────────

/// Decodes source map VLQ mappings string into a sequence of `Mapping`s.
/// Mirrors `sourcemap.MappingsDecoder` in Go.
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

    /// Decode the next mapping. Returns `Some(mapping)` or `None` when done.
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
                    self.error = Some("Unsupported Format: No entries after sourceLine".to_string());
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

    /// Collect all decoded mappings into a vector.
    pub fn collect_all(mut self) -> (Vec<Mapping>, Option<String>) {
        let mut result = Vec::new();
        while let Some(m) = self.next() {
            result.push(m);
        }
        (result, self.error)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TryGetSourceMappingURL
// ────────────────────────────────────────────────────────────────────────────

/// Try to find the `//# sourceMappingURL=...` comment at the end of a file.
/// Mirrors `sourcemap.TryGetSourceMappingURL` in Go.
pub fn try_get_source_mapping_url(text: &str, line_starts: &[usize]) -> String {
    if line_starts.is_empty() {
        return String::new();
    }
    for index in (0..line_starts.len()).rev() {
        let pos = line_starts[index];
        let end = if index + 1 < line_starts.len() {
            line_starts[index + 1]
        } else {
            text.len()
        };
        let line = text[pos..end].trim();
        if line.is_empty() {
            continue;
        }
        let bytes = line.as_bytes();
        if bytes.len() < 4
            || !line.starts_with("//")
            || (bytes[2] != b'#' && bytes[2] != b'@')
            || bytes[3] != b' '
        {
            break;
        }
        if let Some(url) = line[4..].strip_prefix("sourceMappingURL=") {
            return url.trim_end().to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tspath::ComparePathsOptions;

    fn gen() -> Generator {
        Generator::new("main.js", "/", "/", ComparePathsOptions::default())
    }

    fn raw_map(g: &mut Generator) -> RawSourceMap {
        g.raw_source_map()
    }

    // ── Empty generator tests ──────────────────────────────────────────────

    #[test]
    fn empty() {
        let mut g = gen();
        let map = raw_map(&mut g);
        assert_eq!(
            map,
            RawSourceMap {
                version: 3,
                file: "main.js".to_string(),
                source_root: "/".to_string(),
                sources: vec![],
                names: vec![],
                mappings: "".to_string(),
                sources_content: vec![],
            }
        );
    }

    #[test]
    fn empty_serialized() {
        let mut g = gen();
        let actual = g.to_json();
        let expected = r#"{"version":3,"file":"main.js","sourceRoot":"/","sources":[],"names":[],"mappings":""}"#;
        assert_eq!(actual, expected);
    }

    // ── AddSource tests ────────────────────────────────────────────────────

    #[test]
    fn add_source() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        let map = raw_map(&mut g);
        assert_eq!(source_index, 0);
        assert_eq!(
            map,
            RawSourceMap {
                version: 3,
                file: "main.js".to_string(),
                source_root: "/".to_string(),
                sources: vec!["main.ts".to_string()],
                names: vec![],
                mappings: "".to_string(),
                sources_content: vec![],
            }
        );
    }

    #[test]
    fn set_source_content() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        g.set_source_content(source_index, "foo").unwrap();
        let map = raw_map(&mut g);
        assert_eq!(source_index, 0);
        assert_eq!(
            map,
            RawSourceMap {
                version: 3,
                file: "main.js".to_string(),
                source_root: "/".to_string(),
                sources: vec!["main.ts".to_string()],
                names: vec![],
                mappings: "".to_string(),
                sources_content: vec![Some("foo".to_string())],
            }
        );
    }

    #[test]
    fn set_source_content_for_second_source_only() {
        let mut g = gen();
        g.add_source("/skipped.ts");
        let source_index = g.add_source("/main.ts");
        g.set_source_content(source_index, "foo").unwrap();
        let map = raw_map(&mut g);
        assert_eq!(source_index, 1);
        assert_eq!(
            map,
            RawSourceMap {
                version: 3,
                file: "main.js".to_string(),
                source_root: "/".to_string(),
                sources: vec!["skipped.ts".to_string(), "main.ts".to_string()],
                names: vec![],
                mappings: "".to_string(),
                sources_content: vec![None, Some("foo".to_string())],
            }
        );
    }

    #[test]
    fn set_source_content_source_index_out_of_range() {
        let mut g = gen();
        assert_eq!(
            g.set_source_content(-1, "").unwrap_err(),
            "sourceIndex is out of range"
        );
        assert_eq!(
            g.set_source_content(0, "").unwrap_err(),
            "sourceIndex is out of range"
        );
    }

    #[test]
    fn set_source_content_for_second_source_only_serialized() {
        let mut g = gen();
        g.add_source("/skipped.ts");
        let source_index = g.add_source("/main.ts");
        g.set_source_content(source_index, "foo").unwrap();
        let actual = g.to_json();
        let expected = r#"{"version":3,"file":"main.js","sourceRoot":"/","sources":["skipped.ts","main.ts"],"names":[],"mappings":"","sourcesContent":[null,"foo"]}"#;
        assert_eq!(actual, expected);
    }

    // ── AddName tests ──────────────────────────────────────────────────────

    #[test]
    fn add_name() {
        let mut g = gen();
        let name_index = g.add_name("foo");
        let map = raw_map(&mut g);
        assert_eq!(name_index, 0);
        assert_eq!(
            map,
            RawSourceMap {
                version: 3,
                file: "main.js".to_string(),
                source_root: "/".to_string(),
                sources: vec![],
                names: vec!["foo".to_string()],
                mappings: "".to_string(),
                sources_content: vec![],
            }
        );
    }

    // ── AddGeneratedMapping tests ──────────────────────────────────────────

    #[test]
    fn add_generated_mapping() {
        let mut g = gen();
        g.add_generated_mapping(0, 0).unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, "A");
    }

    #[test]
    fn add_generated_mapping_on_second_line_only() {
        let mut g = gen();
        g.add_generated_mapping(1, 0).unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, ";A");
    }

    // ── AddSourceMapping tests ─────────────────────────────────────────────

    #[test]
    fn add_source_mapping() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, "AAAA");
    }

    #[test]
    fn add_source_mapping_next_generated_character() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
        g.add_source_mapping(0, 1, source_index, 0, 0).unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, "AAAA,CAAA");
    }

    #[test]
    fn add_source_mapping_next_generated_and_source_character() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
        g.add_source_mapping(0, 1, source_index, 0, 1).unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, "AAAA,CAAC");
    }

    #[test]
    fn add_source_mapping_next_generated_line() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
        g.add_source_mapping(1, 0, source_index, 0, 0).unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, "AAAA;AAAA");
    }

    #[test]
    fn add_source_mapping_previous_source_character() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        g.add_source_mapping(0, 0, source_index, 0, 1).unwrap();
        g.add_source_mapping(0, 1, source_index, 0, 0).unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, "AAAC,CAAD");
    }

    // ── AddNamedSourceMapping tests ────────────────────────────────────────

    #[test]
    fn add_named_source_mapping() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        let name_index = g.add_name("foo");
        g.add_named_source_mapping(0, 0, source_index, 0, 0, name_index)
            .unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, "AAAAA");
        assert_eq!(map.names, vec!["foo".to_string()]);
    }

    #[test]
    fn add_named_source_mapping_with_previous_name() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        let name_index1 = g.add_name("foo");
        let name_index2 = g.add_name("bar");
        g.add_named_source_mapping(0, 0, source_index, 0, 0, name_index2)
            .unwrap();
        g.add_named_source_mapping(0, 1, source_index, 0, 0, name_index1)
            .unwrap();
        let map = raw_map(&mut g);
        assert_eq!(map.mappings, "AAAAC,CAAAD");
        assert_eq!(map.names, vec!["foo".to_string(), "bar".to_string()]);
    }

    // ── Error cases: AddGeneratedMapping ──────────────────────────────────

    #[test]
    fn add_generated_mapping_generated_line_cannot_backtrack() {
        let mut g = gen();
        g.add_generated_mapping(1, 0).unwrap();
        assert_eq!(
            g.add_generated_mapping(0, 0).unwrap_err(),
            "generatedLine cannot backtrack"
        );
    }

    #[test]
    fn add_generated_mapping_generated_character_cannot_be_negative() {
        let mut g = gen();
        g.add_generated_mapping(0, 0).unwrap();
        assert_eq!(
            g.add_generated_mapping(0, -1).unwrap_err(),
            "generatedCharacter cannot be negative"
        );
    }

    // ── Error cases: AddSourceMapping ─────────────────────────────────────

    #[test]
    fn add_source_mapping_generated_line_cannot_backtrack() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        g.add_source_mapping(1, 0, source_index, 0, 0).unwrap();
        assert_eq!(
            g.add_source_mapping(0, 0, source_index, 0, 0).unwrap_err(),
            "generatedLine cannot backtrack"
        );
    }

    #[test]
    fn add_source_mapping_generated_character_cannot_be_negative() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
        assert_eq!(
            g.add_source_mapping(0, -1, source_index, 0, 0).unwrap_err(),
            "generatedCharacter cannot be negative"
        );
    }

    #[test]
    fn add_source_mapping_source_index_is_out_of_range() {
        let mut g = gen();
        assert_eq!(
            g.add_source_mapping(0, 0, -1, 0, 0).unwrap_err(),
            "sourceIndex is out of range"
        );
        assert_eq!(
            g.add_source_mapping(0, 0, 0, 0, 0).unwrap_err(),
            "sourceIndex is out of range"
        );
    }

    #[test]
    fn add_source_mapping_source_line_cannot_be_negative() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        assert_eq!(
            g.add_source_mapping(0, 0, source_index, -1, 0).unwrap_err(),
            "sourceLine cannot be negative"
        );
    }

    #[test]
    fn add_source_mapping_source_character_cannot_be_negative() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        assert_eq!(
            g.add_source_mapping(0, 0, source_index, 0, -1).unwrap_err(),
            "sourceCharacter cannot be negative"
        );
    }

    // ── Error cases: AddNamedSourceMapping ────────────────────────────────

    #[test]
    fn add_named_source_mapping_generated_line_cannot_backtrack() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        let name_index = g.add_name("foo");
        g.add_named_source_mapping(1, 0, source_index, 0, 0, name_index)
            .unwrap();
        assert_eq!(
            g.add_named_source_mapping(0, 0, source_index, 0, 0, name_index)
                .unwrap_err(),
            "generatedLine cannot backtrack"
        );
    }

    #[test]
    fn add_named_source_mapping_generated_character_cannot_be_negative() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        let name_index = g.add_name("foo");
        g.add_named_source_mapping(0, 0, source_index, 0, 0, name_index)
            .unwrap();
        assert_eq!(
            g.add_named_source_mapping(0, -1, source_index, 0, 0, name_index)
                .unwrap_err(),
            "generatedCharacter cannot be negative"
        );
    }

    #[test]
    fn add_named_source_mapping_source_index_is_out_of_range() {
        let mut g = gen();
        let name_index = g.add_name("foo");
        assert_eq!(
            g.add_named_source_mapping(0, 0, -1, 0, 0, name_index)
                .unwrap_err(),
            "sourceIndex is out of range"
        );
        assert_eq!(
            g.add_named_source_mapping(0, 0, 0, 0, 0, name_index)
                .unwrap_err(),
            "sourceIndex is out of range"
        );
    }

    #[test]
    fn add_named_source_mapping_source_line_cannot_be_negative() {
        let mut g = gen();
        let name_index = g.add_name("foo");
        let source_index = g.add_source("/main.ts");
        assert_eq!(
            g.add_named_source_mapping(0, 0, source_index, -1, 0, name_index)
                .unwrap_err(),
            "sourceLine cannot be negative"
        );
    }

    #[test]
    fn add_named_source_mapping_source_character_cannot_be_negative() {
        let mut g = gen();
        let name_index = g.add_name("foo");
        let source_index = g.add_source("/main.ts");
        assert_eq!(
            g.add_named_source_mapping(0, 0, source_index, 0, -1, name_index)
                .unwrap_err(),
            "sourceCharacter cannot be negative"
        );
    }

    #[test]
    fn add_named_source_mapping_name_index_is_out_of_range() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        assert_eq!(
            g.add_named_source_mapping(0, 0, source_index, 0, 0, -1)
                .unwrap_err(),
            "nameIndex is out of range"
        );
        assert_eq!(
            g.add_named_source_mapping(0, 0, source_index, 0, 0, 0)
                .unwrap_err(),
            "nameIndex is out of range"
        );
    }

    // ── Decoder round-trip tests ──────────────────────────────────────────

    #[test]
    fn decoder_empty() {
        let decoder = MappingsDecoder::new("");
        let (mappings, err) = decoder.collect_all();
        assert!(mappings.is_empty());
        assert!(err.is_none());
    }

    #[test]
    fn decoder_single_generated_mapping() {
        let decoder = MappingsDecoder::new("A");
        let (mappings, err) = decoder.collect_all();
        assert!(err.is_none());
        assert_eq!(mappings.len(), 1);
        assert_eq!(
            mappings[0],
            Mapping {
                generated_line: 0,
                generated_character: 0,
                source_index: MISSING_SOURCE,
                source_line: MISSING_LINE_OR_COLUMN,
                source_character: MISSING_UTF16_COLUMN,
                name_index: MISSING_NAME,
            }
        );
    }

    #[test]
    fn decoder_single_source_mapping() {
        let decoder = MappingsDecoder::new("AAAA");
        let (mappings, err) = decoder.collect_all();
        assert!(err.is_none());
        assert_eq!(mappings.len(), 1);
        assert!(mappings[0].is_source_mapping());
        assert_eq!(mappings[0].source_index, 0);
        assert_eq!(mappings[0].source_line, 0);
        assert_eq!(mappings[0].source_character, 0);
    }

    #[test]
    fn decoder_two_lines() {
        let decoder = MappingsDecoder::new("AAAA;AAAA");
        let (mappings, err) = decoder.collect_all();
        assert!(err.is_none());
        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].generated_line, 0);
        assert_eq!(mappings[1].generated_line, 1);
    }

    #[test]
    fn decoder_roundtrip() {
        let mut g = gen();
        let source_index = g.add_source("/main.ts");
        let name_index = g.add_name("foo");
        g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
        g.add_source_mapping(0, 5, source_index, 0, 3).unwrap();
        g.add_named_source_mapping(1, 0, source_index, 1, 0, name_index)
            .unwrap();
        let map = raw_map(&mut g);

        let decoder = MappingsDecoder::new(&map.mappings);
        let (mappings, err) = decoder.collect_all();
        assert!(err.is_none());
        assert_eq!(mappings.len(), 3);

        assert_eq!(mappings[0].generated_line, 0);
        assert_eq!(mappings[0].generated_character, 0);
        assert_eq!(mappings[0].source_line, 0);
        assert_eq!(mappings[0].source_character, 0);

        assert_eq!(mappings[1].generated_line, 0);
        assert_eq!(mappings[1].generated_character, 5);
        assert_eq!(mappings[1].source_line, 0);
        assert_eq!(mappings[1].source_character, 3);

        assert_eq!(mappings[2].generated_line, 1);
        assert_eq!(mappings[2].generated_character, 0);
        assert_eq!(mappings[2].source_line, 1);
        assert_eq!(mappings[2].source_character, 0);
        assert_eq!(mappings[2].name_index, 0);
    }

    // ── try_get_source_mapping_url tests ──────────────────────────────────

    #[test]
    fn try_get_source_mapping_url_finds_comment() {
        let text = "var x = 1;\n//# sourceMappingURL=app.js.map\n";
        // Line 0: "var x = 1;\n" (11 chars), line 1 starts at 11
        // Line 1: "//# sourceMappingURL=app.js.map\n" (31 chars), line 2 starts at 42
        let line_starts = vec![0, 11, 42];
        assert_eq!(
            try_get_source_mapping_url(text, &line_starts),
            "app.js.map"
        );
    }

    #[test]
    fn try_get_source_mapping_url_no_comment() {
        let text = "var x = 1;\n";
        let line_starts = vec![0, 11];
        assert_eq!(try_get_source_mapping_url(text, &line_starts), "");
    }
}
