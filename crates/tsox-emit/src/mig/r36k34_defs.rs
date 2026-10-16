#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{ClassDeclarationData, NodeData};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::ast::node_data_generated as ndg;

use crate::printer::generated_identifier_flags::{EmitContext, NodeFactory};

pub trait NodeR36k34Accessors {
    fn as_class_declaration(&self) -> &ClassDeclarationData;
}

impl NodeR36k34Accessors for Node {
    fn as_class_declaration(&self) -> &ClassDeclarationData { ::tsox_core::fntrace::enter("as_class_declaration"); 
        match &self.data {
            NodeData::ClassDeclaration(d) => d,
            _ => panic!("expected ClassDeclaration, got {:?}", self.kind),
        }
    }
}

impl<'a> NodeFactory<'a> {
    pub fn new_parenthesized_expression(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_parenthesized_expression"); 
        Arc::new(Node::new(
            SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedExpression(ndg::ParenthesizedExpressionData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn new_export_specifier(
        &self,
        is_type_only: bool,
        property_name: Option<&Arc<Node>>,
        name: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_export_specifier"); 
        Arc::new(Node::new(
            SyntaxKind::ExportSpecifier,
            NodeData::ExportSpecifier(ndg::ExportSpecifierData {
                is_type_only,
                property_name: property_name.cloned(),
                name: name.clone(),
            }),
        ))
    }

    pub fn new_export_assignment(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        is_export_equals: bool,
        type_node: Option<&Arc<Node>>,
        expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_export_assignment"); 
        Arc::new(Node::new(
            SyntaxKind::ExportAssignment,
            NodeData::ExportAssignment(ndg::ExportAssignmentData {
                modifiers,
                is_export_equals,
                type_node: type_node.cloned().unwrap_or_else(|| {
                    Arc::new(Node::new(
                        SyntaxKind::MissingDeclaration,
                        NodeData::MissingDeclaration(ndg::MissingDeclarationData {
                            modifiers: None,
                        }),
                    ))
                }),
                expression: expression.clone(),
            }),
        ))
    }

    pub fn new_set_accessor_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_set_accessor_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::SetAccessor,
            NodeData::SetAccessorDeclaration(ndg::SetAccessorDeclarationData {
                modifiers,
                name: name.clone(),
                type_parameters,
                parameters,
                type_node,
                full_signature,
                body: Some(body),
            }),
        ))
    }

    pub fn new_class_expression(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        heritage_clauses: Option<Arc<NodeList>>,
        members: Arc<NodeList>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_class_expression"); 
        Arc::new(Node::new(
            SyntaxKind::ClassExpression,
            NodeData::ClassExpression(ndg::ClassExpressionData {
                modifiers,
                name,
                type_parameters,
                heritage_clauses,
                members,
            }),
        ))
    }

    pub fn new_method_call(
        &self,
        object: &Arc<Node>,
        method_name: &Arc<Node>,
        arguments_list: Vec<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_method_call"); 
        let property_access =
            self.new_property_access_expression(object, None, method_name, NodeFlags::empty());
        self.new_call_expression(
            &property_access,
            None,
            None,
            self.new_node_list(arguments_list),
            NodeFlags::empty(),
        )
    }
}
