#![allow(unused_imports, dead_code, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{self as ndg, NodeData};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::printer::generated_identifier_flags::NodeFactory;

impl<'a> NodeFactory<'a> {
    pub fn new_new_expression(
        &self,
        expression: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
        arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NewExpression,
            NodeData::NewExpression(ndg::NewExpressionData {
                expression: expression.clone(),
                type_arguments,
                arguments,
            }),
        ))
    }
}
