#![allow(unused_imports, dead_code, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::SyntaxKind;

use crate::printer::NodeFactory;
use tsox_checker::checker::mig::m2d::EmitResolver;

impl<'a> NodeFactory<'a> {
    pub fn new_keyword_type_node(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_keyword_type_node"); 
        Arc::new(Node::new(kind, NodeData::KeywordTypeNode))
    }

    pub fn new_literal_type_node(&self, literal: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_literal_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(ndg::LiteralTypeNodeData { literal }),
        ))
    }

    pub fn new_union_type_node(&self, types: Arc<NodeList>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_union_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::UnionType,
            NodeData::UnionTypeNode(ndg::UnionTypeNodeData { types }),
        ))
    }

    pub fn new_array_type_node(&self, element_type: Option<Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_array_type_node"); 
        Arc::new(Node::new(
            SyntaxKind::ArrayType,
            NodeData::ArrayTypeNode(ndg::ArrayTypeNodeData {
                element_type: element_type.unwrap_or_else(|| {
                    Arc::new(Node::new(SyntaxKind::Unknown, NodeData::KeywordTypeNode))
                }),
            }),
        ))
    }

    pub fn new_type_literal_node(&self, members: Arc<NodeList>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_type_literal_node"); 
        Arc::new(Node::new(
            SyntaxKind::TypeLiteral,
            NodeData::TypeLiteralNode(ndg::TypeLiteralNodeData { members }),
        ))
    }

    pub fn new_computed_property_name(&self, expression: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_computed_property_name"); 
        Arc::new(Node::new(
            SyntaxKind::ComputedPropertyName,
            NodeData::ComputedPropertyName(ndg::ComputedPropertyNameData { expression }),
        ))
    }

    pub fn new_property_signature_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_property_signature_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(ndg::PropertySignatureDeclarationData {
                modifiers,
                name: name.unwrap_or_else(|| missing_node()),
                postfix_token,
                type_node: type_node.unwrap_or_else(missing_node),
                initializer: initializer.unwrap_or_else(missing_node),
            }),
        ))
    }

    pub fn update_mapped_type_node(
        &self,
        node: &Arc<Node>,
        readonly_token: Option<Arc<Node>>,
        type_parameter: Option<Arc<Node>>,
        name_type: Option<Arc<Node>>,
        question_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        members: Option<Arc<NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_mapped_type_node"); 
        let mut updated = Node::new(
            SyntaxKind::MappedType,
            NodeData::MappedTypeNode(ndg::MappedTypeNodeData {
                readonly_token,
                type_parameter: type_parameter.unwrap_or_else(missing_node),
                name_type,
                question_token,
                type_node,
                members,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_import_type_node(
        &self,
        node: &Arc<Node>,
        is_type_of: bool,
        argument: Arc<Node>,
        attributes: Option<Arc<Node>>,
        qualifier: Option<Arc<Node>>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_import_type_node"); 
        let mut updated = Node::new(
            SyntaxKind::ImportType,
            NodeData::ImportTypeNode(ndg::ImportTypeNodeData {
                is_type_of,
                argument,
                attributes,
                qualifier,
                type_arguments,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_literal_type_node(&self, node: &Arc<Node>, literal: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("update_literal_type_node"); 
        let mut updated = Node::new(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(ndg::LiteralTypeNodeData { literal }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_type_parameter_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        constraint: Option<Arc<Node>>,
        expression: Option<Arc<Node>>,
        default_type: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_type_parameter_declaration"); 
        let old = node.as_type_parameter_declaration_data();
        let mut updated = Node::new(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(ndg::TypeParameterDeclarationData {
                modifiers: modifiers.or(old.modifiers.clone()),
                name: name.unwrap_or_else(|| old.name.clone()),
                constraint: constraint.or_else(|| old.constraint.clone()),
                expression: expression.or_else(|| old.expression.clone()),
                default_type: default_type.or_else(|| old.default_type.clone()),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_type_alias_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        type_parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_type_alias_declaration"); 
        let old = node.as_type_alias_declaration_data();
        let mut updated = Node::new(
            node.kind,
            NodeData::TypeAliasDeclaration(ndg::TypeAliasDeclarationData {
                modifiers,
                name,
                type_parameters: Some(type_parameters),
                type_node: type_node.unwrap_or_else(|| old.type_node.clone()),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_interface_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        type_parameters: Arc<NodeList>,
        heritage_clauses: Arc<NodeList>,
        members: Arc<NodeList>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_interface_declaration"); 
        let mut updated = Node::new(
            SyntaxKind::InterfaceDeclaration,
            NodeData::InterfaceDeclaration(ndg::InterfaceDeclarationData {
                modifiers,
                name,
                type_parameters: Some(type_parameters),
                heritage_clauses: Some(heritage_clauses),
                members,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_module_block(&self, node: &Arc<Node>, statements: Arc<NodeList>) -> Arc<Node> { ::tsox_core::fntrace::enter("update_module_block"); 
        let mut updated = Node::new(
            SyntaxKind::ModuleBlock,
            NodeData::ModuleBlock(ndg::ModuleBlockData { statements }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_module_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        keyword: SyntaxKind,
        name: Option<Arc<Node>>,
        attributes: Option<Arc<Node>>,
        body: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_module_declaration"); 
        let old = node.as_module_declaration_data();
        let mut updated = Node::new(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ndg::ModuleDeclarationData {
                modifiers,
                keyword,
                name: name.unwrap_or_else(|| old.name.clone()),
                attributes: attributes.or(old.attributes.clone()),
                body,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_import_equals_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        is_type_only: bool,
        name: Arc<Node>,
        module_reference: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_import_equals_declaration"); 
        let mut updated = Node::new(
            SyntaxKind::ImportEqualsDeclaration,
            NodeData::ImportEqualsDeclaration(ndg::ImportEqualsDeclarationData {
                modifiers,
                is_type_only,
                name,
                module_reference,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_external_module_reference(
        &self,
        node: &Arc<Node>,
        expression: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_external_module_reference"); 
        let mut updated = Node::new(
            SyntaxKind::ExternalModuleReference,
            NodeData::ExternalModuleReference(ndg::ExternalModuleReferenceData { expression }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_index_signature_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_index_signature_declaration"); 
        let mut updated = Node::new(
            SyntaxKind::IndexSignature,
            NodeData::IndexSignatureDeclaration(ndg::IndexSignatureDeclarationData {
                modifiers,
                parameters,
                type_node: type_node.unwrap_or_else(missing_node),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_property_signature_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_property_signature_declaration"); 
        let old = node.as_property_signature_declaration_data();
        let mut updated = Node::new(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(ndg::PropertySignatureDeclarationData {
                modifiers,
                name,
                postfix_token: postfix_token.or(old.postfix_token.clone()),
                type_node: type_node.unwrap_or_else(|| old.type_node.clone()),
                initializer: initializer
                    .unwrap_or_else(|| old.initializer.clone()),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_method_signature_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        postfix_token: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_method_signature_declaration"); 
        let mut updated = Node::new(
            SyntaxKind::MethodSignature,
            NodeData::MethodSignatureDeclaration(ndg::MethodSignatureDeclarationData {
                modifiers,
                name,
                postfix_token,
                type_parameters,
                parameters,
                type_node,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }
}

fn missing_node() -> Arc<Node> { ::tsox_core::fntrace::enter("missing_node"); 
    Arc::new(Node::new(SyntaxKind::Unknown, NodeData::Token))
}

pub trait R39K01DataExt {
    fn as_qualified_name(&self) -> &ndg::QualifiedNameData;
    fn as_variable_declaration_list(&self) -> &ndg::VariableDeclarationListData;
    fn as_external_module_reference(&self) -> &ndg::ExternalModuleReferenceData;
    fn as_import_clause(&self) -> &ndg::ImportClauseData;
    fn as_literal_type_node(&self) -> &ndg::LiteralTypeNodeData;
    fn as_expression_with_type_arguments(&self) -> &ndg::ExpressionWithTypeArgumentsData;
    fn as_parenthesized_expression(&self) -> &ndg::ParenthesizedExpressionData;
    fn as_array_literal_expression(&self) -> &ndg::ArrayLiteralExpressionData;
    fn as_object_literal_expression(&self) -> &ndg::ObjectLiteralExpressionData;
    fn as_spread_element(&self) -> &ndg::SpreadElementData;
    fn as_type_parameter_declaration_data(&self) -> &ndg::TypeParameterDeclarationData;
    fn as_type_alias_declaration_data(&self) -> &ndg::TypeAliasDeclarationData;
    fn as_module_declaration_data(&self) -> &ndg::ModuleDeclarationData;
    fn as_property_signature_declaration_data(&self) -> &ndg::PropertySignatureDeclarationData;
}

macro_rules! r39k01_as_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl R39K01DataExt for Node {
    r39k01_as_data!(as_qualified_name, QualifiedName, ndg::QualifiedNameData);
    r39k01_as_data!(
        as_variable_declaration_list,
        VariableDeclarationList,
        ndg::VariableDeclarationListData
    );
    r39k01_as_data!(
        as_external_module_reference,
        ExternalModuleReference,
        ndg::ExternalModuleReferenceData
    );
    r39k01_as_data!(as_import_clause, ImportClause, ndg::ImportClauseData);
    r39k01_as_data!(as_literal_type_node, LiteralTypeNode, ndg::LiteralTypeNodeData);
    r39k01_as_data!(
        as_expression_with_type_arguments,
        ExpressionWithTypeArguments,
        ndg::ExpressionWithTypeArgumentsData
    );
    r39k01_as_data!(
        as_parenthesized_expression,
        ParenthesizedExpression,
        ndg::ParenthesizedExpressionData
    );
    r39k01_as_data!(
        as_array_literal_expression,
        ArrayLiteralExpression,
        ndg::ArrayLiteralExpressionData
    );
    r39k01_as_data!(
        as_object_literal_expression,
        ObjectLiteralExpression,
        ndg::ObjectLiteralExpressionData
    );
    r39k01_as_data!(as_spread_element, SpreadElement, ndg::SpreadElementData);
    r39k01_as_data!(
        as_type_parameter_declaration_data,
        TypeParameterDeclaration,
        ndg::TypeParameterDeclarationData
    );
    r39k01_as_data!(
        as_type_alias_declaration_data,
        TypeAliasDeclaration,
        ndg::TypeAliasDeclarationData
    );
    r39k01_as_data!(
        as_module_declaration_data,
        ModuleDeclaration,
        ndg::ModuleDeclarationData
    );
    r39k01_as_data!(
        as_property_signature_declaration_data,
        PropertySignatureDeclaration,
        ndg::PropertySignatureDeclarationData
    );
}

pub trait R39K01EmitResolverExt {
    fn is_declaration_visible(&self, node: &Arc<Node>) -> bool;
    fn is_name_resolvable(&self, location: Option<&Arc<Node>>, name: &str) -> bool;
    fn is_late_bound(&self, node: Option<&Arc<Node>>) -> bool;
    fn is_implementation_of_overload(&self, node: &Arc<Node>) -> bool;
    fn is_optional_parameter(&self, node: &Arc<Node>) -> bool;
    fn is_import_required_by_augmentation(&self, decl: &Arc<Node>) -> bool;
    fn is_this_property_assignment_declaration_redundant(&self, decl: &Arc<Node>) -> bool;
    fn is_definitely_reference_to_global_symbol_object(&self, node: &Arc<Node>) -> bool;
    fn get_referenced_member_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String;
}

impl R39K01EmitResolverExt for EmitResolver {
    fn is_declaration_visible(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_declaration_visible"); 
        !has_private_modifier(node)
    }

    fn is_name_resolvable(&self, _location: Option<&Arc<Node>>, name: &str) -> bool { ::tsox_core::fntrace::enter("is_name_resolvable"); 
        !name.is_empty()
    }

    fn is_late_bound(&self, node: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_late_bound"); 
        false
    }

    fn is_implementation_of_overload(&self, _node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_implementation_of_overload"); 
        false
    }

    fn is_optional_parameter(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_optional_parameter"); 
        match &node.data {
            NodeData::ParameterDeclaration(d) => d
                .question_token
                .as_ref()
                .map(|t| t.kind == SyntaxKind::QuestionToken)
                .unwrap_or(false),
            _ => false,
        }
    }

    fn is_import_required_by_augmentation(&self, _decl: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_import_required_by_augmentation"); 
        false
    }

    fn is_this_property_assignment_declaration_redundant(&self, _decl: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_this_property_assignment_declaration_redundant"); 
        false
    }

    fn is_definitely_reference_to_global_symbol_object(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_definitely_reference_to_global_symbol_object"); 
        node.kind == SyntaxKind::PropertyAccessExpression
            && node
                .expression()
                .map(|e| e.kind == SyntaxKind::Identifier && e.text() == "Symbol")
                .unwrap_or(false)
    }

    fn get_referenced_member_value_declaration(&self, _node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_referenced_member_value_declaration"); 
        None
    }

    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_element_access_expression_name"); 
        match &expression.data {
            NodeData::ElementAccessExpression(d) => d.argument_expression.text().to_string(),
            _ => String::new(),
        }
    }
}

fn has_private_modifier(node: &Node) -> bool { ::tsox_core::fntrace::enter("has_private_modifier"); 
    node.modifier_nodes()
        .iter()
        .any(|m| m.kind == SyntaxKind::PrivateKeyword)
}
