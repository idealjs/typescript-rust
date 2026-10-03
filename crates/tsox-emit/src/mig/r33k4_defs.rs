use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub fn syntax_kind_string(kind: &SyntaxKind) -> String { ::tsox_core::fntrace::enter("syntax_kind_string"); 
    format!("{:?}", kind)
}

pub fn node_kind_string(node: &Node) -> String { ::tsox_core::fntrace::enter("node_kind_string"); 
    format!("{:?}", node.kind)
}

pub fn arc_node_kind_string(node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("arc_node_kind_string"); 
    format!("{:?}", node.kind)
}
