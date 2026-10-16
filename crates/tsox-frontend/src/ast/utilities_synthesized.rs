use crate::ast::*;
use std::sync::Arc;
use tsox_core::core::text::TextRange;

pub fn node_is_missing(node: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("node_is_missing"); 
    match node {
        None => true,
        Some(n) => n.pos() == n.end() && (n.pos() as i32) >= 0 && n.kind != SyntaxKind::EndOfFile,
    }
}

pub fn node_is_present(node: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("node_is_present"); 
    !node_is_missing(node)
}

pub fn node_is_synthesized(node: &Node) -> bool { ::tsox_core::fntrace::enter("node_is_synthesized"); 
    position_is_synthesized(node.pos()) || position_is_synthesized(node.end())
}

pub fn position_is_synthesized(pos: usize) -> bool { ::tsox_core::fntrace::enter("position_is_synthesized"); 
    (pos as i32) < 0
}

pub fn range_is_synthesized(loc: TextRange) -> bool { ::tsox_core::fntrace::enter("range_is_synthesized"); 
    position_is_synthesized(loc.pos()) || position_is_synthesized(loc.end())
}
