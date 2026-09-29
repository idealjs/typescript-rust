#![allow(dead_code, unused_imports, unused_variables)]

use tsox_frontend::scanner::{CommentRange, CommentRangeKind};

pub fn is_jsdoc_like_text(text: &str, comment: &CommentRange) -> bool {
    let b = text.as_bytes();
    comment.kind == CommentRangeKind::MultiLine
        && comment.end - comment.pos >= 5
        && b[comment.pos + 2] == b'*'
        && b[comment.pos + 3] != b'/'
}
