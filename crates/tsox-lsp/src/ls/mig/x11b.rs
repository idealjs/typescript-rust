#![allow(dead_code, unused_imports, unused_variables)]

use std::cmp::Ordering;
use std::sync::Arc;

use crate::lsp::lsproto_lsp_basic::TextEdit;
use crate::lsp::lsproto_util::compare_positions;
use crate::ls::language_service::LanguageService;
use tsox_core::core;

pub fn text_edits_conflict(a: &TextEdit, b: &TextEdit, multiple_projections: bool) -> bool { ::tsox_core::fntrace::enter("text_edits_conflict"); 
    if compare_positions(&a.range.end, &b.range.start) == Ordering::Greater {
        return true;
    }
    multiple_projections && a.range.start == a.range.end && a.range == b.range && a.new_text != b.new_text
}

impl LanguageService {
    pub fn try_get_generated_position(
        &self,
        file_name: &str,
        position: tsox_core::core::text::TextPos,
    ) -> Option<crate::ls::source_map::DocumentPosition> { ::tsox_core::fntrace::enter("try_get_generated_position"); 
        let new_pos = self.m5w_try_get_generated_position_worker(file_name, position);
        if let Some(new_pos) = &new_pos {
            if self.read_file(&new_pos.file_name).is_none() {
                return None;
            }
        }
        new_pos
    }

}
