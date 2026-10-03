#![allow(unused_imports)]

use super::*;

pub fn deep_clone_node(node: &std::sync::Arc<node::Node>) -> std::sync::Arc<node::Node> { ::tsox_core::fntrace::enter("deep_clone_node"); 
    std::sync::Arc::clone(node)
}
