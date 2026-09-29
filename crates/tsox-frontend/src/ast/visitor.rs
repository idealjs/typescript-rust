use std::sync::Arc;

use crate::ast::mig::m3b_2::NodeFactory;
use crate::ast::node::Node;
use crate::ast::node_node_list::NodeList;

pub struct NodeVisitorHooks;

pub struct NodeVisitor {
    pub factory: NodeFactory,
    pub hooks: NodeVisitorHooks,
}

impl Default for NodeVisitor {
    fn default() -> Self {
        Self {
            factory: NodeFactory::new(),
            hooks: NodeVisitorHooks,
        }
    }
}

impl NodeVisitor {
    pub fn visit_node(&mut self, node: &Arc<Node>) -> Arc<Node> {
        Arc::clone(node)
    }

    pub fn visit_node_opt(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        node.cloned()
    }

    pub fn visit_node_list(&mut self, list: Option<&NodeList>) -> Vec<Arc<Node>> {
        list.map(|l| l.nodes.iter().map(Arc::clone).collect())
            .unwrap_or_default()
    }
}
