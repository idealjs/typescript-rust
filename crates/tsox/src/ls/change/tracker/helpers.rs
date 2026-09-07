use crate::ast::Node;
use std::sync::Arc;

#[allow(dead_code)]
pub fn need_semicolon_between(_a: &Arc<Node>, _b: &Arc<Node>) -> bool {
    false
}

#[allow(dead_code)]
pub fn is_separator(_node: &Arc<Node>, _candidate: Option<&Arc<Node>>) -> bool {
    false
}

pub fn range_contains_range_exclusive(outer: &Arc<Node>, inner: &Arc<Node>) -> bool {
    outer.pos() < inner.pos() && inner.end() < outer.end()
}

#[allow(dead_code)]
pub fn get_members_or_properties(_node: &Arc<Node>) -> Option<crate::ast::NodeList> {
    None
}

#[allow(dead_code)]
fn find_indentation_column(
    _text: &str,
    _line_start: usize,
    _member_start: usize,
    _tab_size: i32,
) -> i32 {
    0
}

#[allow(dead_code)]
fn advance_indentation_column(column: i32, ch: char, tab_size: i32) -> i32 {
    if ch == '\t' {
        column + tab_size - (column % tab_size)
    } else {
        column + 1
    }
}

#[allow(dead_code)]
pub fn has_comments_before_line_break(text: &str, start: usize) -> bool {
    for ch in text[start..].chars() {
        if !crate::stringutil::is_white_space_single_line(ch) {
            return ch == '/';
        }
    }
    false
}
