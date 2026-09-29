#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node::{ModifierList, NodeList};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::mig::m4k_2::Transformer;
use crate::printer::generated_identifier_flags::NodeFactory;

pub(crate) fn set_loc(node: &mut Arc<Node>, loc: TextRange) {
    if let Some(n) = Arc::get_mut(node) {
        n.loc = loc;
    }
}

pub(crate) fn placeholder_transformer() -> Transformer {
    Transformer::new(|_, node| Some(node), None)
}

pub(crate) fn type_parameter_list(node: &Arc<Node>) -> Option<Arc<NodeList>> {
    match &node.data {
        NodeData::ClassDeclaration(d) => d.type_parameters.clone(),
        NodeData::ClassExpression(d) => d.type_parameters.clone(),
        NodeData::MethodDeclaration(d) => d.type_parameters.clone(),
        NodeData::FunctionDeclaration(d) => d.type_parameters.clone(),
        NodeData::FunctionExpression(d) => d.type_parameters.clone(),
        NodeData::ArrowFunction(d) => d.type_parameters.clone(),
        _ => None,
    }
}

impl<'a> NodeFactory<'a> {
    pub(crate) fn update_property_assignment_r39k13(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::PropertyAssignment,
            NodeData::PropertyAssignment(ndg::PropertyAssignmentData {
                modifiers,
                name: name.clone(),
                postfix_token: postfix_token.cloned(),
                type_node: type_node
                    .cloned()
                    .unwrap_or_else(|| Arc::new(Node::new(SyntaxKind::Unknown, NodeData::Token))),
                initializer: initializer
                    .cloned()
                    .unwrap_or_else(|| Arc::new(Node::new(SyntaxKind::Unknown, NodeData::Token))),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub(crate) fn update_shorthand_property_assignment_r39k13(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        equals_token: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ShorthandPropertyAssignment,
            NodeData::ShorthandPropertyAssignment(ndg::ShorthandPropertyAssignmentData {
                modifiers,
                name: name.clone(),
                postfix_token: None,
                type_node: Arc::new(Node::new(SyntaxKind::Unknown, NodeData::Token)),
                equals_token: equals_token.cloned(),
                object_assignment_initializer: initializer.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub(crate) fn update_variable_declaration_r39k13(
        &self,
        node: &Arc<Node>,
        name: &Arc<Node>,
        exclamation_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(ndg::VariableDeclarationData {
                name: name.clone(),
                exclamation_token: exclamation_token.cloned(),
                type_node: type_node.cloned(),
                initializer: initializer.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub(crate) fn update_binding_element_r39k13(
        &self,
        node: &Arc<Node>,
        dot_dot_dot_token: Option<&Arc<Node>>,
        property_name: Option<&Arc<Node>>,
        name: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(ndg::BindingElementData {
                dot_dot_dot_token: dot_dot_dot_token.cloned(),
                property_name: property_name.cloned(),
                name: name.cloned(),
                initializer: initializer.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub(crate) fn update_property_declaration_r39k13(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(ndg::PropertyDeclarationData {
                modifiers,
                name: name.clone(),
                postfix_token: postfix_token.cloned(),
                type_node: type_node.cloned(),
                initializer: initializer.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub(crate) fn update_export_assignment_r39k13(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        is_export_equals: bool,
        type_node: Option<&Arc<Node>>,
        expression: &Arc<Node>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ExportAssignment,
            NodeData::ExportAssignment(ndg::ExportAssignmentData {
                modifiers,
                is_export_equals,
                type_node: type_node
                    .cloned()
                    .unwrap_or_else(|| Arc::new(Node::new(SyntaxKind::Unknown, NodeData::Token))),
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub(crate) fn update_try_statement_r39k13(
        &self,
        node: &Arc<Node>,
        try_block: &Arc<Node>,
        catch_clause: Option<&Arc<Node>>,
        finally_block: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::TryStatement,
            NodeData::TryStatement(ndg::TryStatementData {
                try_block: try_block.clone(),
                catch_clause: catch_clause.cloned(),
                finally_block: finally_block.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub(crate) fn update_binary_expression_r39k13(
        &self,
        node: &Arc<Node>,
        left: &Arc<Node>,
        operator_token: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::BinaryExpression,
            NodeData::BinaryExpression(ndg::BinaryExpressionData {
                modifiers: None,
                left: left.clone(),
                type_node: None,
                operator_token: operator_token.clone(),
                right: right.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }
}

pub trait R39K13NodeVisitorExt {
    fn visit_modifiers(
        &mut self,
        modifiers: &Option<Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>>;
    fn visit_nodes_r39k13(&mut self, nodes: Option<&NodeList>) -> Option<NodeList>;
    fn visit_embedded_statement(&mut self, node: &Arc<Node>) -> Arc<Node>;
}

impl R39K13NodeVisitorExt for NodeVisitor {
    fn visit_modifiers(
        &mut self,
        modifiers: &Option<Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>> {
        let modifiers = modifiers.as_ref()?;
        let mut changed = false;
        let mut visited_nodes: Vec<Arc<Node>> = Vec::with_capacity(modifiers.list.nodes.len());
        for m in &modifiers.list.nodes {
            let visited = self.visit_node(m);
            if !Arc::ptr_eq(&visited, m) {
                changed = true;
            }
            visited_nodes.push(visited);
        }
        if !changed {
            return Some(Arc::clone(modifiers));
        }
        Some(Arc::new(ModifierList {
            list: NodeList {
                loc: modifiers.list.loc,
                nodes: visited_nodes,
            },
            modifier_flags: modifiers.modifier_flags,
        }))
    }

    fn visit_nodes_r39k13(&mut self, nodes: Option<&NodeList>) -> Option<NodeList> {
        let nodes = nodes?;
        let mut changed = false;
        let mut visited_nodes: Vec<Arc<Node>> = Vec::with_capacity(nodes.nodes.len());
        for n in &nodes.nodes {
            let visited = self.visit_node(n);
            if !Arc::ptr_eq(&visited, n) {
                changed = true;
            }
            visited_nodes.push(visited);
        }
        if !changed {
            return Some(NodeList {
                loc: nodes.loc,
                nodes: nodes.nodes.clone(),
            });
        }
        Some(NodeList {
            loc: nodes.loc,
            nodes: visited_nodes,
        })
    }

    fn visit_embedded_statement(&mut self, node: &Arc<Node>) -> Arc<Node> {
        self.visit_node(node)
    }
}
