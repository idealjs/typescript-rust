#![allow(unused_imports)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};

pub fn is_valid_preceding_node(node: &Arc<Node>, source_file: &SourceFile) -> bool {
    if node.kind == crate::ast::SyntaxKind::EndOfFile {
        return !node.jsdoc(source_file).is_empty();
    }
    let start = crate::format::util::token_pos_of_node(source_file, node);
    let width = node.end() - start;
    !(crate::ast::is_whitespace_only_jsx_text(node) || width == 0)
}
