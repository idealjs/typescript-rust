#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::{DeleteExpressionData, SyntheticReferenceExpressionData};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::printer::NodeFactory;

use crate::mig::m4i_12::r36k17_defs::NodeAsR36k17Ext;

pub trait R39K14NodeExt {
    fn as_synthetic_reference_expression(&self) -> &SyntheticReferenceExpressionData;
    fn as_delete_expression(&self) -> &DeleteExpressionData;
}

impl R39K14NodeExt for Node {
    fn as_synthetic_reference_expression(&self) -> &SyntheticReferenceExpressionData {
        match &self.data {
            ndg::NodeData::SyntheticReferenceExpression(d) => d,
            _ => panic!("AsSyntheticReferenceExpression on wrong node kind"),
        }
    }

    fn as_delete_expression(&self) -> &DeleteExpressionData {
        match &self.data {
            ndg::NodeData::DeleteExpression(d) => d,
            _ => panic!("AsDeleteExpression on wrong node kind"),
        }
    }
}

pub trait R39K14NodeFactoryExt {
    fn new_synthetic_reference_expression(
        &self,
        expression: &Arc<Node>,
        this_arg: &Arc<Node>,
    ) -> Arc<Node>;
    fn new_delete_expression(&self, expression: &Arc<Node>) -> Arc<Node>;
    fn update_property_access_expression(
        &self,
        node: &Arc<Node>,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        flags: NodeFlags,
    ) -> Arc<Node>;
    fn update_element_access_expression(
        &self,
        node: &Arc<Node>,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<Arc<Node>>,
        argument_expression: Option<Arc<Node>>,
        flags: NodeFlags,
    ) -> Arc<Node>;
}

impl R39K14NodeFactoryExt for NodeFactory<'_> {
    fn new_synthetic_reference_expression(
        &self,
        expression: &Arc<Node>,
        this_arg: &Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::SyntheticReferenceExpression,
            ndg::NodeData::SyntheticReferenceExpression(SyntheticReferenceExpressionData {
                expression: expression.clone(),
                this_arg: this_arg.clone(),
            }),
        ))
    }

    fn new_delete_expression(&self, expression: &Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::DeleteExpression,
            ndg::NodeData::DeleteExpression(DeleteExpressionData {
                expression: expression.clone(),
            }),
        ))
    }

    fn update_property_access_expression(
        &self,
        node: &Arc<Node>,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        flags: NodeFlags,
    ) -> Arc<Node> {
        let original = node.as_property_access_expression();
        let mut updated = Node::new(
            SyntaxKind::PropertyAccessExpression,
            ndg::NodeData::PropertyAccessExpression(ndg::PropertyAccessExpressionData {
                expression: expression.unwrap_or_else(|| original.expression.clone()),
                question_dot_token: question_dot_token.or_else(|| original.question_dot_token.clone()),
                name: name.unwrap_or_else(|| original.name.clone()),
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }

    fn update_element_access_expression(
        &self,
        node: &Arc<Node>,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<Arc<Node>>,
        argument_expression: Option<Arc<Node>>,
        flags: NodeFlags,
    ) -> Arc<Node> {
        let original = node.as_element_access_expression();
        let mut updated = Node::new(
            SyntaxKind::ElementAccessExpression,
            ndg::NodeData::ElementAccessExpression(ndg::ElementAccessExpressionData {
                expression: expression.unwrap_or_else(|| original.expression.clone()),
                question_dot_token: question_dot_token.or_else(|| original.question_dot_token.clone()),
                argument_expression: argument_expression
                    .unwrap_or_else(|| original.argument_expression.clone()),
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }
}
