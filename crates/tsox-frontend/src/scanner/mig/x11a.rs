#![allow(dead_code, unused_imports, unused_variables)]

use tsox_core::stringutil::is_white_space_like;

pub fn strip_leading_js_doc_comment(line: &str) -> String {
    let line = line.trim_start_matches(is_white_space_like);
    let line = if let Some(rest) = line.strip_prefix('*') {
        rest
    } else {
        line
    };
    line.trim_start_matches(is_white_space_like).to_string()
}
