use std::sync::Arc;
use tsox_frontend::ast::Node;

#[allow(dead_code)]
pub fn need_semicolon_between(_a: &Arc<Node>, _b: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("need_semicolon_between"); 
    false
}

#[allow(dead_code)]
pub fn is_separator(_node: &Arc<Node>, _candidate: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_separator"); 
    false
}

pub fn range_contains_range_exclusive(outer: &Arc<Node>, inner: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("range_contains_range_exclusive"); 
    outer.pos() < inner.pos() && inner.end() < outer.end()
}

#[allow(dead_code)]
pub fn get_members_or_properties(_node: &Arc<Node>) -> Option<tsox_frontend::ast::NodeList> { ::tsox_core::fntrace::enter("get_members_or_properties"); 
    None
}

#[allow(dead_code)]
fn find_indentation_column(
    _text: &str,
    _line_start: usize,
    _member_start: usize,
    _tab_size: i32,
) -> i32 { ::tsox_core::fntrace::enter("find_indentation_column"); 
    0
}

#[allow(dead_code)]
fn advance_indentation_column(column: i32, ch: char, tab_size: i32) -> i32 { ::tsox_core::fntrace::enter("advance_indentation_column"); 
    if ch == '\t' {
        column + tab_size - (column % tab_size)
    } else {
        column + 1
    }
}

#[allow(dead_code)]
pub fn has_comments_before_line_break(text: &str, start: usize) -> bool { ::tsox_core::fntrace::enter("has_comments_before_line_break"); 
    for ch in text[start..].chars() {
        if !tsox_core::stringutil::is_white_space_single_line(ch) {
            return ch == '/';
        }
    }
    false
}
