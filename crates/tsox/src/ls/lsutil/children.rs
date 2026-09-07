use std::sync::Arc;

use crate::ast::{Node, SourceFile};

pub fn get_last_child(_node: &Arc<Node>, _source_file: &SourceFile) -> Option<Arc<Node>> {
    None
}

pub fn get_last_token(node: Option<&Arc<Node>>, _source_file: &SourceFile) -> Option<Arc<Node>> {
    node.cloned()
}

pub fn get_last_visited_child(_node: &Arc<Node>, _source_file: &SourceFile) -> Option<Arc<Node>> {
    None
}

pub fn get_first_token(_node: &Arc<Node>, _source_file: &SourceFile) -> Option<Arc<Node>> {
    None
}

pub fn assert_has_real_position(node: &Arc<Node>) {
    if node.loc.pos < 0 || node.loc.end < 0 {
        panic!("Node must have a real position for this operation.");
    }
}
