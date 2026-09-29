#![allow(unused_imports, dead_code)]

use std::fmt;
use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    BindingElementData, BlockData, ExportAssignmentData, ForStatementData, NodeData,
    PropertyAssignmentData, ShorthandPropertyAssignmentData, SourceFileData,
    TemplateExpressionData, TemplateSpanData, VariableDeclarationData, VariableStatementData,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::scanner::TokenFlags;
use tsox_frontend::ast::is_prologue_directive;

use crate::mig::m4g::r33k7_defs::PrivateIdentifierKind;
use crate::printer::NodeFactory;

impl fmt::Display for PrivateIdentifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrivateIdentifierKind::Field => write!(f, "f"),
            PrivateIdentifierKind::Method => write!(f, "m"),
            PrivateIdentifierKind::Accessor => write!(f, "a"),
            PrivateIdentifierKind::Untransformed => Ok(()),
        }
    }
}

pub trait R36K29NodeExt {
    fn template_literal_flags(&self) -> Option<TokenFlags>;
    fn as_template_expression(&self) -> &TemplateExpressionData;
    fn as_template_span(&self) -> &TemplateSpanData;
    fn as_variable_statement(&self) -> &VariableStatementData;
    fn as_source_file_data(&self) -> &SourceFileData;
    fn as_block(&self) -> &BlockData;
    fn as_for_statement(&self) -> &ForStatementData;
}

impl R36K29NodeExt for Arc<Node> {
    fn template_literal_flags(&self) -> Option<TokenFlags> {
        match &self.data {
            NodeData::NoSubstitutionTemplateLiteral(d) => Some(d.template_flags),
            NodeData::TemplateHead(d) => Some(d.template_flags),
            NodeData::TemplateMiddle(d) => Some(d.template_flags),
            NodeData::TemplateTail(d) => Some(d.template_flags),
            _ => None,
        }
    }

    fn as_template_expression(&self) -> &TemplateExpressionData {
        match &self.data {
            NodeData::TemplateExpression(d) => d,
            _ => panic!("unexpected node"),
        }
    }

    fn as_template_span(&self) -> &TemplateSpanData {
        match &self.data {
            NodeData::TemplateSpan(d) => d,
            _ => panic!("unexpected node"),
        }
    }

    fn as_variable_statement(&self) -> &VariableStatementData {
        match &self.data {
            NodeData::VariableStatement(d) => d,
            _ => panic!("unexpected node"),
        }
    }

    fn as_source_file_data(&self) -> &SourceFileData {
        match &self.data {
            NodeData::SourceFile(d) => d,
            _ => panic!("unexpected node"),
        }
    }

    fn as_block(&self) -> &BlockData {
        match &self.data {
            NodeData::Block(d) => d,
            _ => panic!("unexpected node"),
        }
    }

    fn as_for_statement(&self) -> &ForStatementData {
        match &self.data {
            NodeData::ForStatement(d) => d,
            _ => panic!("unexpected node"),
        }
    }
}

impl<'a> NodeFactory<'a> {
    pub fn new_array_literal_expression(
        &self,
        elements: &NodeList,
        multi_line: bool,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ArrayLiteralExpression,
            NodeData::ArrayLiteralExpression(tsox_frontend::ast::node_data_generated::ArrayLiteralExpressionData {
                elements: Arc::new(NodeList {
                    loc: elements.loc,
                    nodes: elements.nodes.clone(),
                }),
                multi_line,
            }),
        ))
    }

    pub fn update_property_assignment(
        &self,
        node: &Arc<PropertyAssignmentData>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::PropertyAssignment,
            NodeData::PropertyAssignment(PropertyAssignmentData {
                modifiers: modifiers.or_else(|| node.modifiers.clone()),
                name: name.clone(),
                postfix_token: postfix_token.cloned().or_else(|| node.postfix_token.clone()),
                type_node: type_node.cloned().unwrap_or_else(|| node.type_node.clone()),
                initializer: initializer.cloned().unwrap_or_else(|| node.initializer.clone()),
            }),
        ))
    }

    pub fn update_shorthand_property_assignment(
        &self,
        node: &Arc<ShorthandPropertyAssignmentData>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        equals_token: Option<&Arc<Node>>,
        object_assignment_initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ShorthandPropertyAssignment,
            NodeData::ShorthandPropertyAssignment(ShorthandPropertyAssignmentData {
                modifiers: modifiers.or_else(|| node.modifiers.clone()),
                name: name.clone(),
                postfix_token: postfix_token.cloned().or_else(|| node.postfix_token.clone()),
                type_node: type_node.cloned().unwrap_or_else(|| node.type_node.clone()),
                equals_token: equals_token.cloned().or_else(|| node.equals_token.clone()),
                object_assignment_initializer: object_assignment_initializer
                    .cloned()
                    .or_else(|| node.object_assignment_initializer.clone()),
            }),
        ))
    }

    pub fn update_variable_declaration(
        &self,
        node: &Arc<VariableDeclarationData>,
        name: &Arc<Node>,
        exclamation_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(VariableDeclarationData {
                name: name.clone(),
                exclamation_token: exclamation_token.cloned().or_else(|| node.exclamation_token.clone()),
                type_node: type_node.cloned().or_else(|| node.type_node.clone()),
                initializer: initializer.cloned().or_else(|| node.initializer.clone()),
            }),
        ))
    }

    pub fn update_binding_element(
        &self,
        node: &Arc<BindingElementData>,
        dot_dot_dot_token: Option<&Arc<Node>>,
        property_name: Option<&Arc<Node>>,
        name: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(BindingElementData {
                dot_dot_dot_token: dot_dot_dot_token.cloned().or_else(|| node.dot_dot_dot_token.clone()),
                property_name: property_name.cloned().or_else(|| node.property_name.clone()),
                name: name.cloned().or_else(|| node.name.clone()),
                initializer: initializer.cloned().or_else(|| node.initializer.clone()),
            }),
        ))
    }

    pub fn update_property_declaration_data(
        &self,
        node: &Arc<tsox_frontend::ast::node_data_generated::PropertyDeclarationData>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(tsox_frontend::ast::node_data_generated::PropertyDeclarationData {
                modifiers: modifiers.or_else(|| node.modifiers.clone()),
                name: name.clone(),
                postfix_token: postfix_token.cloned().or_else(|| node.postfix_token.clone()),
                type_node: type_node.cloned().or_else(|| node.type_node.clone()),
                initializer: initializer.cloned().or_else(|| node.initializer.clone()),
            }),
        ))
    }

    pub fn update_export_assignment(
        &self,
        node: &Arc<ExportAssignmentData>,
        modifiers: Option<Arc<ModifierList>>,
        is_export_equals: bool,
        type_node: Option<&Arc<Node>>,
        expression: &Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ExportAssignment,
            NodeData::ExportAssignment(ExportAssignmentData {
                modifiers: modifiers.or_else(|| node.modifiers.clone()),
                is_export_equals,
                type_node: type_node.cloned().unwrap_or_else(|| node.type_node.clone()),
                expression: expression.clone(),
            }),
        ))
    }

    pub fn update_source_file(&self, node: &Arc<Node>, statements: Arc<NodeList>) -> Arc<Node> {
        let data = node.as_source_file_data();
        let mut updated = Node::new(
            SyntaxKind::SourceFile,
            NodeData::SourceFile(SourceFileData {
                statements,
                end_of_file_token: data.end_of_file_token.clone(),
                global_exports: data.global_exports.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn new_export_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        is_type_only: bool,
        export_clause: &Arc<Node>,
        module_specifier: Option<Arc<Node>>,
        attributes: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ExportDeclaration,
            NodeData::ExportDeclaration(ndg29_export_declaration(
                modifiers,
                is_type_only,
                Some(export_clause.clone()),
                module_specifier,
                attributes,
            )),
        ))
    }

    pub fn new_modifier_list(&self, modifiers: Vec<Arc<Node>>) -> Arc<ModifierList> {
        Arc::new(ModifierList::new(modifiers, ModifierFlags::default()))
    }

    pub fn new_modifier(&self, kind: SyntaxKind) -> Arc<Node> {
        Arc::new(Node::new(kind, NodeData::Token))
    }
}

fn ndg29_export_declaration(
    modifiers: Option<Arc<ModifierList>>,
    is_type_only: bool,
    export_clause: Option<Arc<Node>>,
    module_specifier: Option<Arc<Node>>,
    attributes: Option<Arc<Node>>,
) -> tsox_frontend::ast::node_data_generated::ExportDeclarationData {
    tsox_frontend::ast::node_data_generated::ExportDeclarationData {
        modifiers,
        is_type_only,
        export_clause,
        module_specifier,
        attributes,
    }
}
