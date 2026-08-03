//! LSP utility functions.
//!
//! Ported from Go's `internal/lsp/lsproto/util.go`.

use super::lsp::{Position, Range, StringOrMarkupContent};

/// Implements a cmp::Ord-like comparison for two Positions.
pub fn compare_positions(pos: &Position, other: &Position) -> std::cmp::Ordering {
    match pos.line.cmp(&other.line) {
        std::cmp::Ordering::Equal => pos.character.cmp(&other.character),
        ord => ord,
    }
}

/// Implements a cmp::Ord-like comparison for two Ranges.
/// Range.Start is compared before Range.End.
pub fn compare_ranges(ls_range: &Range, other: &Range) -> std::cmp::Ordering {
    match compare_positions(&ls_range.start, &other.start) {
        std::cmp::Ordering::Equal => compare_positions(&ls_range.end, &other.end),
        ord => ord,
    }
}

/// Returns the plain text of a StringOrMarkupContent, reading the
/// MarkupContent value when the message is not a plain string.
pub fn string_or_markup_content_as_string(m: &StringOrMarkupContent) -> String {
    m.as_string()
}
