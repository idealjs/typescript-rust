#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::ModifierList;
use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub struct TypedNode<'a> {
    pub node: &'a Node,
}

impl<'a> TypedNode<'a> {
    pub fn as_node(&self) -> &'a Node {
        self.node
    }

    pub fn name(&self) -> &'a Node {
        match &self.node.data {
            NodeData::NamedTupleMember(d) => &d.name,
            NodeData::NamespaceExport(d) => &d.name,
            NodeData::NamespaceExportDeclaration(d) => &d.name,
            NodeData::NamespaceImport(d) => &d.name,
            NodeData::ParameterDeclaration(d) => &d.name,
            _ => panic!("name() on {:?}", self.node.kind),
        }
    }

    pub fn elements(&self) -> &'a NodeList {
        match &self.node.data {
            NodeData::NamedImports(d) => &d.elements,
            NodeData::BindingPattern(d) => &d.elements,
            _ => panic!("elements() on {:?}", self.node.kind),
        }
    }

    pub fn properties(&self) -> &'a NodeList {
        match &self.node.data {
            NodeData::ObjectLiteralExpression(d) => &d.properties,
            _ => panic!("properties() on {:?}", self.node.kind),
        }
    }

    pub fn multi_line(&self) -> bool {
        match &self.node.data {
            NodeData::ObjectLiteralExpression(d) => d.multi_line,
            _ => panic!("multi_line() on {:?}", self.node.kind),
        }
    }

    pub fn modifiers(&self) -> Option<&'a ModifierList> {
        self.node.modifiers().map(|m| &**m)
    }

    pub fn dot_dot_dot_token(&self) -> Option<&'a Node> {
        match &self.node.data {
            NodeData::NamedTupleMember(d) => d.dot_dot_dot_token.as_deref(),
            NodeData::ParameterDeclaration(d) => d.dot_dot_dot_token.as_deref(),
            _ => panic!("dotDotDotToken() on {:?}", self.node.kind),
        }
    }

    pub fn question_token(&self) -> Option<&'a Node> {
        match &self.node.data {
            NodeData::NamedTupleMember(d) => d.question_token.as_deref(),
            NodeData::ParameterDeclaration(d) => d.question_token.as_deref(),
            _ => panic!("questionToken() on {:?}", self.node.kind),
        }
    }

    pub fn type_(&self) -> Option<&'a Node> {
        match &self.node.data {
            NodeData::NamedTupleMember(d) => Some(&d.type_node),
            NodeData::ParameterDeclaration(d) => d.type_node.as_deref(),
            NodeData::OptionalTypeNode(d) => Some(&d.type_node),
            NodeData::ParenthesizedTypeNode(d) => Some(&d.type_node),
            _ => panic!("type() on {:?}", self.node.kind),
        }
    }

    pub fn initializer(&self) -> Option<&'a Node> {
        match &self.node.data {
            NodeData::ParameterDeclaration(d) => d.initializer.as_deref(),
            _ => panic!("initializer() on {:?}", self.node.kind),
        }
    }

    pub fn expression(&self) -> Option<&'a Arc<Node>> {
        match &self.node.data {
            NodeData::NewExpression(d) => Some(&d.expression),
            NodeData::NonNullExpression(d) => Some(&d.expression),
            NodeData::ParenthesizedExpression(d) => Some(&d.expression),
            NodeData::SpreadAssignment(d) => Some(&d.expression),
            _ => self.node.expression(),
        }
    }

    pub fn type_arguments(&self) -> Option<&'a NodeList> {
        match &self.node.data {
            NodeData::NewExpression(d) => d.type_arguments.as_deref(),
            _ => panic!("typeArguments() on {:?}", self.node.kind),
        }
    }

    pub fn arguments(&self) -> Option<&'a NodeList> {
        match &self.node.data {
            NodeData::NewExpression(d) => d.arguments.as_deref(),
            _ => panic!("arguments() on {:?}", self.node.kind),
        }
    }
}

pub trait NodeAsExt {
    fn as_typed(&self) -> TypedNode<'_>;

    fn as_identifier(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::Identifier);
        self.as_typed()
    }

    fn as_string_literal(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::StringLiteral);
        self.as_typed()
    }

    fn as_namespace_import(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::NamespaceImport);
        self.as_typed()
    }

    fn as_named_imports(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::NamedImports);
        self.as_typed()
    }

    fn as_named_tuple_member(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::NamedTupleMember);
        self.as_typed()
    }

    fn as_namespace_export(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::NamespaceExport);
        self.as_typed()
    }

    fn as_namespace_export_declaration(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::NamespaceExportDeclaration);
        self.as_typed()
    }

    fn as_parameter_declaration(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::Parameter);
        self.as_typed()
    }

    fn as_new_expression(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::NewExpression);
        self.as_typed()
    }

    fn as_non_null_expression(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::NonNullExpression);
        self.as_typed()
    }

    fn as_binding_pattern(&self) -> TypedNode<'_> {
        self.as_typed()
    }

    fn as_object_literal_expression(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::ObjectLiteralExpression);
        self.as_typed()
    }

    fn as_property_assignment(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::PropertyAssignment);
        self.as_typed()
    }

    fn as_shorthand_property_assignment(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::ShorthandPropertyAssignment);
        self.as_typed()
    }

    fn as_spread_assignment(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::SpreadAssignment);
        self.as_typed()
    }

    fn as_method_declaration(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::MethodDeclaration);
        self.as_typed()
    }

    fn as_get_accessor_declaration(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::GetAccessor);
        self.as_typed()
    }

    fn as_set_accessor_declaration(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::SetAccessor);
        self.as_typed()
    }

    fn as_optional_type_node(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::OptionalType);
        self.as_typed()
    }

    fn as_parenthesized_expression(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::ParenthesizedExpression);
        self.as_typed()
    }

    fn as_parenthesized_type_node(&self) -> TypedNode<'_> {
        self.assert_kind(SyntaxKind::ParenthesizedType);
        self.as_typed()
    }

    fn assert_kind(&self, kind: SyntaxKind);
}

impl NodeAsExt for Node {
    fn as_typed(&self) -> TypedNode<'_> {
        TypedNode { node: self }
    }

    fn assert_kind(&self, kind: SyntaxKind) {
        if self.kind != kind {
            panic!("expected {:?}, got {:?}", kind, self.kind);
        }
    }
}
