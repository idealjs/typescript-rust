#![allow(dead_code, unused_imports, unused_variables)]

use crate::ast::{Node, NodeFlags};
use crate::ast::utilities_types::is_type_node;
use crate::ast::node_data_generated::is_jsdoc_type_expression;
use std::sync::Arc;

pub fn is_jsdoc_type_expression_or_child(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_type_expression_or_child"); 
    if is_jsdoc_type_expression(node) {
        return true;
    }
    if !node.flags.intersects(NodeFlags::JSDoc.union(NodeFlags::Reparsed)) {
        return false;
    }
    if is_type_node(node) {
        return true;
    }
    let mut current: Option<Arc<Node>> = node.parent();
    while let Some(n) = current {
        if is_type_node(&n) {
            return true;
        }
        current = n.parent();
    }
    false
}
