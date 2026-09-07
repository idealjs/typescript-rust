#![allow(unused_imports)]

use super::*;

pub fn deep_clone_node(node: &std::sync::Arc<node::Node>) -> std::sync::Arc<node::Node> {
    std::sync::Arc::clone(node)
}
