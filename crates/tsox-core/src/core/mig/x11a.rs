#![allow(dead_code, unused_imports, unused_variables)]

use super::super::text::TextRange;

pub fn undefined_text_range() -> TextRange {
    TextRange::new(usize::MAX, usize::MAX)
}
