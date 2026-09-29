use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub fn syntax_kind_string(kind: &SyntaxKind) -> String {
    format!("{:?}", kind)
}

pub fn node_kind_string(node: &Node) -> String {
    format!("{:?}", node.kind)
}

pub fn arc_node_kind_string(node: &Arc<Node>) -> String {
    format!("{:?}", node.kind)
}
