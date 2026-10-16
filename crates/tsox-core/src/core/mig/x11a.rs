#![allow(dead_code, unused_imports, unused_variables)]

use super::super::text::TextRange;

pub fn undefined_text_range() -> TextRange { crate::fntrace::enter("undefined_text_range"); 
    TextRange::new(usize::MAX, usize::MAX)
}
