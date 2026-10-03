use crate::ast::{Node, SyntaxKind, is_jsdoc_link_like, is_jsdoc_tag, is_jsx_child};
use crate::scanner::Scanner;

pub(crate) fn should_rescan_less_than_less_than_token(
    _s: &Scanner,
    containing_node: &Node,
    token: SyntaxKind,
) -> bool { ::tsox_core::fntrace::enter("should_rescan_less_than_less_than_token"); 
    token == SyntaxKind::LessThanLessThanToken && is_jsx_child(containing_node)
}

pub(crate) fn should_skip_child(node: &Node) -> bool { ::tsox_core::fntrace::enter("should_skip_child"); 
    node.kind == SyntaxKind::JSDoc
        || node.kind == SyntaxKind::JSDocText
        || node.kind == SyntaxKind::JSDocTypeLiteral
        || node.kind == SyntaxKind::JSDocSignature
        || is_jsdoc_link_like(node)
        || is_jsdoc_tag(node)
}
