use crate::lsp::lsproto_lsp::Position;
use crate::lsp::lsproto_lsp::Range;
use crate::lsp::lsproto_lsp::StringOrMarkupContent;

pub fn compare_positions(pos: &Position, other: &Position) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("compare_positions"); 
    match pos.line.cmp(&other.line) {
        std::cmp::Ordering::Equal => pos.character.cmp(&other.character),
        ord => ord,
    }
}

pub fn compare_ranges(ls_range: &Range, other: &Range) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("compare_ranges"); 
    match compare_positions(&ls_range.start, &other.start) {
        std::cmp::Ordering::Equal => compare_positions(&ls_range.end, &other.end),
        ord => ord,
    }
}

pub fn string_or_markup_content_as_string(m: &StringOrMarkupContent) -> String { ::tsox_core::fntrace::enter("string_or_markup_content_as_string"); 
    m.as_string()
}
