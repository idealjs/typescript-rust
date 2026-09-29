#![allow(unused_imports, dead_code, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::mig::m4i::{NullishCoalescingTransformer, OptionalCatchTransformer, TaggedTemplateTransformer};
use crate::mig::m4i_3::ObjectRestSpreadTransformer;
use crate::printer::NodeFactory;

use tsox_frontend::ast::node_data_generated::{self as ndg, NodeData};

pub trait R39K11FunctionDeclCastExt {
    fn as_function_declaration(&self) -> &ndg::FunctionDeclarationData;
}

impl R39K11FunctionDeclCastExt for Node {
    fn as_function_declaration(&self) -> &ndg::FunctionDeclarationData {
        match &self.data {
            NodeData::FunctionDeclaration(d) => d,
            _ => panic!("AsFunctionDeclaration on wrong node kind"),
        }
    }
}

pub trait R39K11SimpleTxExt {
    fn r39k11_visit(&mut self, node: Arc<Node>) -> Arc<Node>;

    fn visit_node(&mut self, node: Arc<Node>) -> Arc<Node> {
        self.r39k11_visit(node)
    }

    fn visit_each_child(&mut self, node: Arc<Node>) -> Arc<Node> {
        let mut changed = false;
        tsox_frontend::ast::node_data_generated::for_each_child(&node, |child| {
            let visited = self.r39k11_visit(child.clone());
            changed |= !Arc::ptr_eq(&visited, child);
            true
        });
        if !changed {
            return node;
        }
        panic!(
            "R39K11 visit_each_child rebuild for kind {:?} pending factory update-function port (progress_notes_r39k11.md)",
            node.kind
        );
    }
}

impl R39K11SimpleTxExt for NullishCoalescingTransformer {
    fn r39k11_visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        NullishCoalescingTransformer::visit(self, node)
    }
}

impl R39K11SimpleTxExt for OptionalCatchTransformer {
    fn r39k11_visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        OptionalCatchTransformer::visit(self, node)
    }
}

impl R39K11SimpleTxExt for TaggedTemplateTransformer {
    fn r39k11_visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        TaggedTemplateTransformer::visit(self, node)
    }
}

pub trait R39K11ObjectRestExt {
    fn visit_nodes(&mut self, nodes: Option<&Arc<NodeList>>) -> Option<Arc<NodeList>>;
}

impl R39K11ObjectRestExt for ObjectRestSpreadTransformer {
    fn visit_nodes(&mut self, nodes: Option<&Arc<NodeList>>) -> Option<Arc<NodeList>> {
        nodes.map(|list| {
            let visited: Vec<Arc<Node>> =
                list.nodes.iter().map(|n| self.visit(n.clone())).collect();
            Arc::new(NodeList::new(visited))
        })
    }
}

impl<'a> NodeFactory<'a> {
    pub fn new_template_object_helper(&self, cooked: &Arc<Node>, raw: &Arc<Node>) -> Arc<Node> {
        self.new_call_expression(
            &self.new_identifier("__makeTemplateObject"),
            None,
            None,
            self.new_node_list(vec![cooked.clone(), raw.clone()]),
            NodeFlags::empty(),
        )
    }

    pub fn new_logical_or_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> {
        self.new_binary_expression(
            None,
            left,
            None,
            &self.new_token(SyntaxKind::BarBarToken),
            right,
        )
    }
}
