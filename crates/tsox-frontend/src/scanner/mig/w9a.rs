#![allow(unused_imports)]

//! w9a: scanner utilities 余量收尾批(w9)
//!
//! 交接:stringutil::is_line_break 与 core::NewLineKindLF::get_new_line_character 为缺失依赖,按命名约定调用。

pub(crate) fn normalize_js_doc_type_source_text(text: &str) -> String {
    let line_starts = tsox_core::core::mig::m3j::compute_ecma_line_starts(text);
    if line_starts.len() == 1 {
        return super::m3i::strip_leading_jsdoc_comment(text);
    }

    let mut result = String::with_capacity(text.len());
    let new_line = "\n";
    for (i, &line_start) in line_starts.iter().enumerate() {
        if i > 0 {
            result.push_str(new_line);
        }
        let line_end = if i + 1 < line_starts.len() {
            line_starts[i + 1] as usize
        } else {
            text.len()
        };
        let line = text[line_start as usize..line_end]
            .trim_end_matches(|c| tsox_core::stringutil::is_line_break(c));
        result.push_str(&super::m3i::strip_leading_jsdoc_comment(line));
    }
    result
}
