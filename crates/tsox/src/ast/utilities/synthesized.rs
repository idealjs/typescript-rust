use crate::ast::*;
use crate::core::text::TextRange;
use std::sync::Arc;

pub fn node_is_missing(node: Option<&Arc<Node>>) -> bool {
    match node {
        None => true,
        Some(n) => n.pos() == n.end() && (n.pos() as i32) >= 0 && n.kind != SyntaxKind::EndOfFile,
    }
}

pub fn node_is_present(node: Option<&Arc<Node>>) -> bool {
    !node_is_missing(node)
}

pub fn node_is_synthesized(node: &Node) -> bool {
    position_is_synthesized(node.pos()) || position_is_synthesized(node.end())
}

pub fn position_is_synthesized(pos: usize) -> bool {
    (pos as i32) < 0
}

pub fn range_is_synthesized(loc: TextRange) -> bool {
    position_is_synthesized(loc.pos()) || position_is_synthesized(loc.end())
}
