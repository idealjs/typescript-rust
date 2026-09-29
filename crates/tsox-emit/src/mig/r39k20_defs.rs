#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::{ModifierList, Node, NodeList};
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::mig::m4k_2::Transformer;
use crate::printer::NodeFactory;

pub fn type_eraser_visit_entry(_tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> {
    Some(node)
}

pub fn metadata_transformer_visit_entry(_tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> {
    Some(node)
}

pub trait R39K20NodeVisitorExt {
    fn visit_nodes(&mut self, nodes: Option<&Arc<NodeList>>) -> Option<NodeList>;
    fn visit_slice(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>>;
}

impl R39K20NodeVisitorExt for NodeVisitor {
    fn visit_nodes(&mut self, nodes: Option<&Arc<NodeList>>) -> Option<NodeList> {
        nodes.map(|list| {
            let mut new_list = NodeList::new(list.nodes.clone());
            new_list.loc = list.loc;
            new_list
        })
    }

    fn visit_slice(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>> {
        nodes.to_vec()
    }
}

pub trait R39K20NodeFactoryExt {
    fn new_decorator(&self, expression: Arc<Node>) -> Arc<Node>;
    fn new_metadata_helper(&self, metadata_key: &str, metadata_value: &Arc<Node>) -> Arc<Node>;
    fn new_param_helper(
        &self,
        expression: &Arc<Node>,
        parameter_offset: usize,
        location: tsox_core::core::text::TextRange,
    ) -> Arc<Node>;
    fn new_generated_name_node(&self, node: &Arc<Node>) -> Arc<Node>;
    fn r39k20_update_property_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn r39k20_update_parameter_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        dot_dot_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        question_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node>;
    fn r39k20_update_tagged_template_expression(
        &self,
        node: &Arc<Node>,
        tag: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
        question_dot_token: Option<Arc<Node>>,
        template: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node>;
}

impl R39K20NodeFactoryExt for NodeFactory<'_> {
    fn new_decorator(&self, expression: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Decorator,
            NodeData::Decorator(ndg::DecoratorData { expression }),
        ))
    }

    fn new_metadata_helper(&self, metadata_key: &str, metadata_value: &Arc<Node>) -> Arc<Node> {
        tsox_frontend::format::mig::m4o::new_node_factory(Default::default())
            .new_metadata_helper(metadata_key, metadata_value)
    }

    fn new_param_helper(
        &self,
        expression: &Arc<Node>,
        parameter_offset: usize,
        location: tsox_core::core::text::TextRange,
    ) -> Arc<Node> {
        tsox_frontend::format::mig::m4o::new_node_factory(Default::default())
            .new_param_helper(expression, parameter_offset, location)
    }

    fn new_generated_name_node(&self, node: &Arc<Node>) -> Arc<Node> {
        let generated = self.new_generated_name_for_node(node);
        self.new_identifier(&generated.text)
    }

    fn r39k20_update_property_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(ndg::PropertyDeclarationData {
                modifiers,
                name: Arc::clone(name),
                postfix_token,
                type_node,
                initializer,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn r39k20_update_parameter_declaration(
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
                name: Arc::clone(name),
                question_token: question_token.cloned(),
                type_node: type_node.cloned(),
                initializer: initializer.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn r39k20_update_tagged_template_expression(
        &self,
        node: &Arc<Node>,
        tag: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
        question_dot_token: Option<Arc<Node>>,
        template: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::TaggedTemplateExpression,
            NodeData::TaggedTemplateExpression(ndg::TaggedTemplateExpressionData {
                tag: Arc::clone(tag),
                question_dot_token,
                type_arguments,
                template: Arc::clone(template),
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }
}
