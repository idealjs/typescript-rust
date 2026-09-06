use super::lsp::{Position, Range, StringOrMarkupContent};

pub fn compare_positions(pos: &Position, other: &Position) -> std::cmp::Ordering {
    match pos.line.cmp(&other.line) {
        std::cmp::Ordering::Equal => pos.character.cmp(&other.character),
        ord => ord,
    }
}

pub fn compare_ranges(ls_range: &Range, other: &Range) -> std::cmp::Ordering {
    match compare_positions(&ls_range.start, &other.start) {
        std::cmp::Ordering::Equal => compare_positions(&ls_range.end, &other.end),
        ord => ord,
    }
}

pub fn string_or_markup_content_as_string(m: &StringOrMarkupContent) -> String {
    m.as_string()
}
