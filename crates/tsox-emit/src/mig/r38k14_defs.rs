#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::NodeList;

use crate::printer::NodeFactory;

impl<'a> NodeFactory<'a> {
    pub fn update_expression_with_type_arguments(
        &self,
        node: &Arc<Node>,
        expression: Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_expression_with_type_arguments"); 
        tsox_frontend::format::mig::m4o::new_node_factory(Default::default())
            .update_expression_with_type_arguments(node, &expression, type_arguments)
    }
}
