#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    ArrowFunctionData, NodeData, PropertyAssignmentData, VariableStatementData,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{ModifierList, NodeList};

use crate::printer::generated_identifier_flags::NodeFactory;

impl<'a> NodeFactory<'a> {
    pub fn new_this_expression(&self) -> Arc<Node> {
        self.new_keyword_expression(SyntaxKind::ThisKeyword)
    }

    pub fn new_arrow_function(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: &NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        equals_greater_than_token: &Arc<Node>,
        body: &Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers,
                type_parameters,
                parameters: Arc::new(NodeList {
                    loc: parameters.loc,
                    nodes: parameters.nodes.clone(),
                }),
                type_node,
                full_signature,
                equals_greater_than_token: equals_greater_than_token.clone(),
                body: body.clone(),
            }),
        ))
    }

    pub fn new_property_assignment(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: &Arc<Node>,
    ) -> Arc<Node> {
        let end = initializer.loc.end();
        let type_node = type_node.cloned().unwrap_or_else(|| {
            Arc::new(Node::with_loc(
                SyntaxKind::Unknown,
                NodeData::Token,
                TextRange::new(end, end),
            ))
        });
        Arc::new(Node::new(
            SyntaxKind::PropertyAssignment,
            NodeData::PropertyAssignment(PropertyAssignmentData {
                modifiers,
                name: name.clone(),
                postfix_token: postfix_token.cloned(),
                type_node,
                initializer: initializer.clone(),
            }),
        ))
    }

    pub fn new_variable_statement(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        declaration_list: &Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::VariableStatement,
            NodeData::VariableStatement(VariableStatementData {
                modifiers,
                declaration_list: declaration_list.clone(),
            }),
        ))
    }
}
