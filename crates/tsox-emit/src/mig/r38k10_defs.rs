#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::mig::m4k_2::Transformer;

#[allow(unused_imports)]
use crate::printer::NodeFactory;

#[allow(unused_imports)]
use Transformer as _TransformerKeep;

pub trait R38K10NodeVisitorExt {
    fn visit_each_child(&mut self, node: &Arc<tsox_frontend::ast::node::Node>) -> Option<Arc<tsox_frontend::ast::node::Node>>;
    fn visit_embedded_statement(&mut self, node: &Arc<tsox_frontend::ast::node::Node>) -> Option<Arc<tsox_frontend::ast::node::Node>>;
    fn visit_nodes(&mut self, nodes: &NodeList) -> NodeList;
}

impl R38K10NodeVisitorExt for NodeVisitor {
    fn visit_each_child(&mut self, node: &Arc<tsox_frontend::ast::node::Node>) -> Option<Arc<tsox_frontend::ast::node::Node>> {
        Some(Arc::clone(node))
    }

    fn visit_embedded_statement(&mut self, node: &Arc<tsox_frontend::ast::node::Node>) -> Option<Arc<tsox_frontend::ast::node::Node>> {
        Some(Arc::clone(node))
    }

    fn visit_nodes(&mut self, nodes: &NodeList) -> NodeList {
        let mut new_list = NodeList::new(nodes.nodes.clone());
        new_list.loc = nodes.loc;
        new_list
    }
}

pub trait R38K10NodeExt {
    fn members(&self) -> Arc<NodeList>;
    fn heritage_clauses(&self) -> Arc<NodeList>;
    fn parameter_list(&self) -> Arc<NodeList>;
    fn decorators(&self) -> Vec<Arc<tsox_frontend::ast::node::Node>>;
}

macro_rules! r38k10_required_list_accessor {
    ($name:ident, $field:ident) => {
        fn $name(&self) -> Arc<NodeList> {
            match &self.data {
                NodeData::ClassDeclaration(d) => d.$field.clone(),
                NodeData::ClassExpression(d) => d.$field.clone(),
                _ => panic!(concat!(stringify!($name), "() on {:?}"), self.kind),
            }
        }
    };
}

impl R38K10NodeExt for tsox_frontend::ast::node::Node {
    r38k10_required_list_accessor!(members, members);

    fn heritage_clauses(&self) -> Arc<NodeList> {
        match &self.data {
            NodeData::ClassDeclaration(d) => d
                .heritage_clauses
                .clone()
                .unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
            NodeData::ClassExpression(d) => d
                .heritage_clauses
                .clone()
                .unwrap_or_else(|| Arc::new(NodeList::new(Vec::new()))),
            _ => panic!("heritage_clauses() on {:?}", self.kind),
        }
    }

    fn parameter_list(&self) -> Arc<NodeList> {
        match &self.data {
            NodeData::FunctionDeclaration(d) => d.parameters.clone(),
            _ => panic!("parameter_list() on {:?}", self.kind),
        }
    }

    fn decorators(&self) -> Vec<Arc<tsox_frontend::ast::node::Node>> {
        tsox_frontend::ast::mig::m3b::decorators(self)
    }
}
