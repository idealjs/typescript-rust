#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::mig::m3c::visit_each_child;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::mig::m4f::AsyncTransformer;
use crate::printer::EmitContext;

pub trait R38K5NodeVisitorExt {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>>;
    fn visit_node_option(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>>;
    fn visit_modifiers(&mut self, modifiers: &Option<Arc<ModifierList>>) -> Option<Arc<ModifierList>>;
    fn visit_nodes_list(&mut self, nodes: &NodeList) -> Arc<NodeList>;
}

fn r38k5_m3c_visitor() -> tsox_frontend::ast::mig::m3c::NodeVisitor {
    tsox_frontend::ast::mig::m3c::NodeVisitor {
        factory: tsox_frontend::ast::mig::m3c::NodeFactory {
            hooks: tsox_frontend::ast::mig::m3c::NodeFactoryHooks::default(),
            text_count: 0,
            node_count: 0,
        },
    }
}

impl R38K5NodeVisitorExt for NodeVisitor {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        Some(visit_each_child(node, &mut r38k5_m3c_visitor()))
    }

    fn visit_node_option(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        self.visit_node_opt(node)
    }

    fn visit_modifiers(
        &mut self,
        modifiers: &Option<Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>> {
        let modifiers = modifiers.as_ref()?;
        let nodes = self.visit_nodes_list(&modifiers.list);
        Some(Arc::new(ModifierList {
            list: NodeList {
                loc: nodes.loc,
                nodes: nodes.nodes.to_vec(),
            },
            modifier_flags: modifiers.modifier_flags,
        }))
    }

    fn visit_nodes_list(&mut self, nodes: &NodeList) -> Arc<NodeList> {
        let visited = nodes
            .nodes
            .iter()
            .map(|n| visit_each_child(n, &mut r38k5_m3c_visitor()))
            .collect();
        Arc::new(NodeList {
            loc: nodes.loc,
            nodes: visited,
        })
    }
}

pub trait R38K5ArcNodeExt {
    fn heritage_clauses(&self) -> Option<&Arc<NodeList>>;
    fn members(&self) -> &Arc<NodeList>;
    fn postfix_token(&self) -> Option<&Arc<Node>>;
    fn asterisk_token(&self) -> Option<&Arc<Node>>;
    fn full_signature(&self) -> Option<&Arc<Node>>;
}

impl R38K5ArcNodeExt for Arc<Node> {
    fn heritage_clauses(&self) -> Option<&Arc<NodeList>> {
        match &self.data {
            NodeData::ClassDeclaration(d) => d.heritage_clauses.as_ref(),
            NodeData::ClassExpression(d) => d.heritage_clauses.as_ref(),
            NodeData::InterfaceDeclaration(d) => d.heritage_clauses.as_ref(),
            _ => None,
        }
    }

    fn members(&self) -> &Arc<NodeList> {
        match &self.data {
            NodeData::ClassDeclaration(d) => &d.members,
            NodeData::ClassExpression(d) => &d.members,
            NodeData::InterfaceDeclaration(d) => &d.members,
            NodeData::ObjectLiteralExpression(d) => &d.properties,
            _ => panic!("unexpected node for members: {:?}", self.kind),
        }
    }

    fn postfix_token(&self) -> Option<&Arc<Node>> {
        match &self.data {
            NodeData::MethodDeclaration(d) => d.postfix_token.as_ref(),
            NodeData::PropertyDeclaration(d) => d.postfix_token.as_ref(),
            _ => None,
        }
    }

    fn asterisk_token(&self) -> Option<&Arc<Node>> {
        match &self.data {
            NodeData::MethodDeclaration(d) => d.asterisk_token.as_ref(),
            NodeData::FunctionDeclaration(d) => d.asterisk_token.as_ref(),
            NodeData::FunctionExpression(d) => d.asterisk_token.as_ref(),
            _ => None,
        }
    }

    fn full_signature(&self) -> Option<&Arc<Node>> {
        match &self.data {
            NodeData::MethodDeclaration(d) => d.full_signature.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.full_signature.as_ref(),
            NodeData::SetAccessorDeclaration(d) => d.full_signature.as_ref(),
            NodeData::FunctionDeclaration(d) => d.full_signature.as_ref(),
            NodeData::FunctionExpression(d) => d.full_signature.as_ref(),
            _ => None,
        }
    }
}

impl AsyncTransformer {
    pub fn visitor(&self) -> NodeVisitor {
        NodeVisitor::default()
    }

    pub fn emit_context_mut(&mut self) -> &mut EmitContext {
        self.emit_context.as_mut().unwrap()
    }
}
