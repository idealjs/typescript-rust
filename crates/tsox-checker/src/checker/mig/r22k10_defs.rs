use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3c::NodeFactory;
use tsox_frontend::ast::node_data_generated::{
    IdentifierData, NodeData, PrivateIdentifierData, PropertyAccessExpressionData,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::{FlowNode, Node, SyntaxKind};

pub trait R22K10FactoryExt {
    fn new_identifier(&self, text: &str) -> Arc<Node>;
    fn new_private_identifier(&self, text: &str) -> Arc<Node>;
    fn new_keyword_expression(&self, kind: SyntaxKind) -> Arc<Node>;
    fn new_property_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node>;
}

impl R22K10FactoryExt for NodeFactory {
    fn new_identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_identifier"); 
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    fn new_private_identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_private_identifier"); 
        Arc::new(Node::new(
            SyntaxKind::PrivateIdentifier,
            NodeData::PrivateIdentifier(PrivateIdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    fn new_keyword_expression(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_keyword_expression"); 
        Arc::new(Node::new(kind, NodeData::KeywordExpression))
    }

    fn new_property_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_property_access_expression"); 
        let mut node = Node::new(
            SyntaxKind::PropertyAccessExpression,
            NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                expression: Arc::clone(expression),
                question_dot_token: question_dot_token.cloned(),
                name: Arc::clone(name),
            }),
        );
        node.flags |= flags & NodeFlags::OptionalChain;
        Arc::new(node)
    }
}

pub fn is_access_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_access_expression"); 
    matches!(
        node.kind,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
    )
}

thread_local! {
    static NODE_FLOW_NODES: RefCell<HashMap<u64, Arc<FlowNode>>> = RefCell::new(HashMap::new());
}

pub fn flow_node_of(node: &Arc<Node>) -> Option<Arc<FlowNode>> { ::tsox_core::fntrace::enter("flow_node_of"); 
    NODE_FLOW_NODES.with(|m| m.borrow().get(&node.id()).cloned())
}

pub fn set_flow_node_of(node: &Arc<Node>, flow: Option<Arc<FlowNode>>) { ::tsox_core::fntrace::enter("set_flow_node_of"); 
    NODE_FLOW_NODES.with(|m| {
        let mut m = m.borrow_mut();
        match flow {
            Some(f) => {
                m.insert(node.id(), f);
            }
            None => {
                m.remove(&node.id());
            }
        }
    });
}
