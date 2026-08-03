//! Converters between TS compiler types and LSP protocol types.
//! Port of Go's `internal/ls/lsconv/converters.go`.

use super::linemap::{LspLineMap, compute_lsp_line_starts};
use crate::lsp::lsproto::lsp::{DocumentUri, Location, Position, Range};

/// Position encoding kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionEncodingKind {
    Utf8,
    Utf16,
    Utf32,
}

/// Script interface for converting between compiler positions and LSP positions.
pub trait Script {
    fn file_name(&self) -> &str;
    fn text(&self) -> &str;
}

/// Converts between compiler offsets and LSP positions.
pub struct Converters {
    position_encoding: PositionEncodingKind,
}

impl Converters {
    pub fn new(position_encoding: PositionEncodingKind) -> Self {
        Converters { position_encoding }
    }

    /// Convert a compiler text range to an LSP Range.
    pub fn to_lsp_range(&self, script: &dyn Script, pos: usize, end: usize) -> Range {
        Range {
            start: self.position_to_line_and_character(script, pos),
            end: self.position_to_line_and_character(script, end),
        }
    }

    /// Convert an LSP Position to a compiler byte offset.
    pub fn line_and_character_to_position(
        &self,
        script: &dyn Script,
        line_and_character: &Position,
    ) -> usize {
        let text = script.text();
        let line_map = compute_lsp_line_starts(text);
        let line = line_and_character.line as usize;
        let char_pos = line_and_character.character as usize;
        let text_len = text.len();

        // Clamp line to valid range.
        if line >= line_map.line_starts.len() {
            return text_len;
        }

        let start = line_map.line_starts[line];

        // Determine end of this line.
        let line_end = if line + 1 < line_map.line_starts.len() {
            line_map.line_starts[line + 1]
        } else {
            text_len
        };

        if line_map.ascii_only || self.position_encoding == PositionEncodingKind::Utf8 {
            return std::cmp::max(start, std::cmp::min(start + char_pos, line_end));
        }

        // Scan from line start counting UTF-16 code units.
        let mut utf16_char = 0usize;
        let mut pos = start;
        let text_bytes = text.as_bytes();
        while pos < line_end {
            let b = text_bytes[pos];
            let char_len = if b < 0x80 { 1 } else { utf8_char_len(b) };
            let r = text[pos..].chars().next().unwrap_or('\0');
            let u16_len = utf16_len_of_char(r);
            if utf16_char + u16_len > char_pos {
                break;
            }
            utf16_char += u16_len;
            pos += char_len;
        }

        pos
    }

    /// Convert a compiler byte offset to an LSP Position.
    pub fn position_to_line_and_character(&self, script: &dyn Script, position: usize) -> Position {
        let text = script.text();
        let position = std::cmp::min(position, text.len());

        let line_map = compute_lsp_line_starts(text);

        let line = line_map.compute_index_of_line_start(position);
        let start = line_map.line_starts[line];

        let character =
            if line_map.ascii_only || self.position_encoding == PositionEncodingKind::Utf8 {
                position - start
            } else {
                // Count UTF-16 code units from start to position.
                let mut char_count = 0u32;
                for r in text[start..position].chars() {
                    char_count += utf16_len_of_char(r) as u32;
                }
                char_count as usize
            };

        Position {
            line: line as u32,
            character: character as u32,
        }
    }

    /// Convert a compiler range to an LSP Location.
    pub fn to_lsp_location(&self, script: &dyn Script, pos: usize, end: usize) -> Location {
        Location {
            uri: DocumentUri(file_name_to_document_uri(script.file_name())),
            range: self.to_lsp_range(script, pos, end),
        }
    }
}

/// Convert a file name to a document URI.
pub fn file_name_to_document_uri(file_name: &str) -> String {
    if crate::bundled::is_bundled(file_name) {
        return file_name.to_string();
    }

    // Simple implementation: file:// scheme
    if file_name.starts_with("file://") {
        return file_name.to_string();
    }

    // Handle dynamic file names (^/...)
    if file_name.starts_with("^/") {
        let rest = &file_name[2..];
        if let Some(slash) = rest.find('/') {
            let scheme = &rest[..slash];
            let rest2 = &rest[slash + 1..];
            if let Some(slash2) = rest2.find('/') {
                let authority = &rest2[..slash2];
                let path = &rest2[slash2 + 1..];
                if authority == "ts-nul-authority" {
                    return format!("{scheme}:{path}");
                }
                return format!("{scheme}://{authority}/{path}");
            }
        }
    }

    // Standard file:// URI
    format!("file://{file_name}")
}

/// Convert a language ID to a script kind.
pub fn language_kind_to_script_kind(language_id: &str) -> u8 {
    match language_id {
        "typescript" => 3,      // ScriptKindTS
        "typescriptreact" => 4, // ScriptKindTSX
        "javascript" => 1,      // ScriptKindJS
        "javascriptreact" => 2, // ScriptKindJSX
        "json" => 5,            // ScriptKindJSON
        _ => 0,                 // ScriptKindUnknown
    }
}

fn utf8_char_len(first_byte: u8) -> usize {
    if first_byte < 0x80 {
        1
    } else if first_byte < 0xC0 {
        1
    } else if first_byte < 0xE0 {
        2
    } else if first_byte < 0xF0 {
        3
    } else {
        4
    }
}

fn utf16_len_of_char(c: char) -> usize {
    let code = c as u32;
    if code >= 0x10000 {
        2 // Surrogate pair
    } else {
        1
    }
}
