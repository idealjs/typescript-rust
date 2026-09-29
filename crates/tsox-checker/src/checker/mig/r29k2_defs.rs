#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{NodeData, StringLiteralData, SyntheticExpressionData};
use tsox_frontend::ast::{Node, SyntaxKind};

use crate::checker::types::Type;

pub trait NodeFactoryExt29 {
    fn new_synthetic_expression(
        &self,
        type_node: &Arc<Type>,
        is_spread: bool,
        tuple_name_source: Option<&Arc<Node>>,
    ) -> Arc<Node>;
}

impl NodeFactoryExt29 for tsox_frontend::ast::mig::m3c::NodeFactory {
    fn new_synthetic_expression(
        &self,
        _type_node: &Arc<Type>,
        is_spread: bool,
        tuple_name_source: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::SyntheticExpression,
            NodeData::SyntheticExpression(SyntheticExpressionData {
                type_node: None,
                is_spread,
                tuple_name_source: tuple_name_source.cloned(),
            }),
        ))
    }
}
