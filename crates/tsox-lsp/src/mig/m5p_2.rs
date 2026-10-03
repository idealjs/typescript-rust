#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Node, SourceFile};

use crate::ls::change_tracker_tracker::Tracker;
use crate::lsp::lsproto;
use crate::mig::m5u_conv::M5uFidelityExt;

pub const FEATURE_ALL: u32 = (1 << 20) - 1;

fn m5u_converters() -> crate::mig::m5u_conv::M5uConverters { ::tsox_core::fntrace::enter("m5u_converters"); 
    crate::mig::m5u_conv::new_converters(
        crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    )
}

impl Tracker {
    pub fn mig_from_lsp_edit_range(&mut self, source_file: &Arc<SourceFile>, lsproto_range: &lsproto::Range) -> TextRange { ::tsox_core::fntrace::enter("mig_from_lsp_edit_range"); 
        let spans = crate::mig::m5u_conv::from_lsp_range_for_source_file(&m5u_converters(), source_file, lsproto_range.clone(), FEATURE_ALL);
        for span in &spans {
            if span.fidelity.is_exact() && Arc::ptr_eq(&span.file, source_file) {
                return span.span;
            }
        }
        self.unmappable_files.insert(crate::mig::m5u_conv::source_file_original_file_name(source_file).to_string());
        if !spans.is_empty() {
            return spans[0].span;
        }
        TextRange::new(0, 0)
    }

    pub fn mig_to_lsp_edit_range(&mut self, source_file: &Arc<SourceFile>, text_range: TextRange) -> lsproto::Range { ::tsox_core::fntrace::enter("mig_to_lsp_edit_range"); 
        let script_view = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(source_file),
        };
        let (r, fidelity) = m5u_converters().to_lsp_range(&script_view, text_range);
        if !fidelity.is_exact() {
            self.unmappable_files.insert(crate::mig::m5u_conv::source_file_original_file_name(source_file).to_string());
        }
        r
    }

    pub fn mig_replace_text_range_with_text(&mut self, source_file: &Arc<SourceFile>, text_range: TextRange, text: &str) { ::tsox_core::fntrace::enter("mig_replace_text_range_with_text"); 
        let lsp_range = self.mig_to_lsp_edit_range(source_file, text_range);
        self.replace_range_with_text(source_file, lsp_range, text.to_string());
    }

    pub fn mig_insert_text_at(&mut self, source_file: &Arc<SourceFile>, pos: usize, text: &str) { ::tsox_core::fntrace::enter("mig_insert_text_at"); 
        self.mig_replace_text_range_with_text(source_file, TextRange::new(pos, pos), text);
    }

    pub fn mig_reindent_inserted_lines(&self, source_file: &Arc<SourceFile>, change: &super::m5p_2_support::TrackerEdit, text: &str) -> String { ::tsox_core::fntrace::enter("mig_reindent_inserted_lines"); 
        if text.is_empty() || change.text_range.pos() != change.text_range.end() || change.options.indentation.is_some() {
            return text.to_string();
        }
        if !text.ends_with(self.new_line()) {
            return text.to_string();
        }
        let original = crate::mig::m5u_conv::source_file_original_text(source_file);
        let mut pos = change.text_range.pos();
        if let Some(spans) = crate::mig::m5u_conv::source_file_span_map(source_file) {
            let (mapped, fidelity) = spans.virtual_to_original_position(pos);
            if !fidelity.is_exact() {
                return text.to_string();
            }
            pos = mapped;
        }
        if pos > original.len() {
            return text.to_string();
        }
        let line_start = original[..pos]
            .rfind(|c: char| c == '\r' || c == '\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let before_point = &original[line_start..pos];
        if before_point.is_empty() {
            if !mig_leading_indentation(text).is_empty() {
                return text.to_string();
            }
            return format!("{}{}", mig_leading_indentation(&original[line_start..]), text);
        }
        if mig_leading_indentation(before_point) != before_point {
            return text.to_string();
        }
        format!("{}{}", text, before_point)
    }
}

pub fn mig_dedupe_identical_edits(edits: Vec<lsproto::TextEdit>) -> Vec<lsproto::TextEdit> { ::tsox_core::fntrace::enter("mig_dedupe_identical_edits"); 
    let mut deduped: Vec<lsproto::TextEdit> = Vec::new();
    for edit in edits {
        if let Some(last) = deduped.last() {
            if last.range == edit.range && last.new_text == edit.new_text {
                continue;
            }
        }
        deduped.push(edit);
    }
    deduped
}

pub fn mig_text_edits_conflict(a: &lsproto::TextEdit, b: &lsproto::TextEdit, multiple_projections: bool) -> bool { ::tsox_core::fntrace::enter("mig_text_edits_conflict"); 
    if crate::lsp::lsproto_util::compare_positions(&a.range.end, &b.range.start) == std::cmp::Ordering::Greater {
        return true;
    }
    multiple_projections && a.range.start == a.range.end && a.range == b.range && a.new_text != b.new_text
}

pub fn mig_leading_indentation(text: &str) -> &str { ::tsox_core::fntrace::enter("mig_leading_indentation"); 
    let end = text.bytes().take_while(|&b| b == b' ' || b == b'\t').count();
    &text[..end]
}

pub fn mig_has_comments_before_line_break(text: &str, start: usize) -> bool { ::tsox_core::fntrace::enter("mig_has_comments_before_line_break"); 
    for ch in text[start..].chars() {
        if !tsox_core::stringutil::is_white_space_single_line(ch) {
            return ch == '/';
        }
    }
    false
}

pub fn mig_need_semicolon_between(a: &Arc<Node>, b: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("mig_need_semicolon_between"); 
    (tsox_frontend::ast::is_property_signature_declaration(a) || tsox_frontend::ast::is_property_declaration(a))
        && tsox_frontend::ast::mig::m3f_4::is_class_or_type_element(b)
        && b.name().map(|n| n.kind == tsox_frontend::ast::SyntaxKind::ComputedPropertyName).unwrap_or(false)
        || tsox_frontend::ast::is_statement_but_not_declaration(a)
            && tsox_frontend::ast::is_statement_but_not_declaration(b)
}
