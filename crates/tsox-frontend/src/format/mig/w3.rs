#![allow(unused_imports)]

use crate::ast::Node;
use crate::format::lists::is_list_element;
use std::sync::Arc;

pub fn find_outermost_node_within_list_level(
    node: &Arc<Node>,
    file: &crate::ast::SourceFile,
) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node));
    while let Some(cur) = current.clone() {
        let Some(parent) = cur.parent() else {
            break;
        };
        if parent.end() != node.end() || is_list_element(&parent, &cur, file) {
            break;
        }
        current = Some(parent);
    }
    current
}
