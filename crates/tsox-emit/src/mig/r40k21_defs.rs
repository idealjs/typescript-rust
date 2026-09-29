use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{self as ndg, NodeData};
use tsox_frontend::ast::{ModifierList, SyntaxKind};

use crate::printer::NodeFactory;

pub trait R40K21NodeFactoryExt {
    fn new_keyword_type_node(&self, kind: SyntaxKind) -> Arc<Node>;
    fn update_parameter_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        dot_dot_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        question_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node>;
}

impl R40K21NodeFactoryExt for NodeFactory<'_> {
    fn new_keyword_type_node(&self, kind: SyntaxKind) -> Arc<Node> {
        Arc::new(Node::new(kind, NodeData::Token))
    }

    fn update_parameter_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        dot_dot_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        question_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ndg::ParameterDeclarationData {
                modifiers,
                dot_dot_dot_token: dot_dot_dot_token.cloned(),
                name: name.clone(),
                question_token: question_token.cloned(),
                type_node: type_node.cloned(),
                initializer: initializer.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }
}
