#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    CallExpressionData, ElementAccessExpressionData, ExportAssignmentData, ExportDeclarationData,
    ImportClauseData, ImportDeclarationData, ImportEqualsDeclarationData,
    ExternalModuleReferenceData, NamedExportsData, NamedImportsData,
    NamespaceExportData, NodeData, PostfixUnaryExpressionData, PropertyAccessExpressionData,
    PropertyDeclarationData, VariableDeclarationListData, VariableStatementData,
    BindingElementData, ImportSpecifierData,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{ModifierList, NodeList, TokenFlags};

use crate::printer::generated_identifier_flags::{
    AutoGenerateOptions, GeneratedIdentifierFlags, NodeFactory,
};

pub const TOKEN_FLAGS_NONE: TokenFlags = 0;

pub trait NodeAsR36k17Ext {
    fn as_call_expression(&self) -> &CallExpressionData;
    fn as_property_access_expression(&self) -> &PropertyAccessExpressionData;
    fn as_element_access_expression(&self) -> &ElementAccessExpressionData;
    fn as_postfix_unary_expression(&self) -> &PostfixUnaryExpressionData;
    fn as_export_assignment(&self) -> &ExportAssignmentData;
    fn as_import_equals_declaration(&self) -> &ImportEqualsDeclarationData;
    fn as_export_declaration(&self) -> &ExportDeclarationData;
    fn as_variable_statement(&self) -> &VariableStatementData;
    fn as_variable_declaration_list(&self) -> &VariableDeclarationListData;
    fn as_binding_element(&self) -> &BindingElementData;
    fn as_import_declaration(&self) -> &ImportDeclarationData;
    fn as_import_clause(&self) -> &ImportClauseData;
    fn as_named_imports(&self) -> &NamedImportsData;
    fn as_named_exports(&self) -> &NamedExportsData;
    fn as_namespace_export(&self) -> &NamespaceExportData;
    fn as_property_declaration(&self) -> &PropertyDeclarationData;
    fn elements(&self) -> &NodeList;
    fn property_name_or_name(&self) -> Arc<Node>;
}

macro_rules! as_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl NodeAsR36k17Ext for Node {
    as_data!(as_call_expression, CallExpression, CallExpressionData);
    as_data!(
        as_property_access_expression,
        PropertyAccessExpression,
        PropertyAccessExpressionData
    );
    as_data!(
        as_element_access_expression,
        ElementAccessExpression,
        ElementAccessExpressionData
    );
    as_data!(
        as_postfix_unary_expression,
        PostfixUnaryExpression,
        PostfixUnaryExpressionData
    );
    as_data!(as_export_assignment, ExportAssignment, ExportAssignmentData);
    as_data!(
        as_import_equals_declaration,
        ImportEqualsDeclaration,
        ImportEqualsDeclarationData
    );
    as_data!(
        as_export_declaration,
        ExportDeclaration,
        ExportDeclarationData
    );
    as_data!(as_variable_statement, VariableStatement, VariableStatementData);
    as_data!(
        as_variable_declaration_list,
        VariableDeclarationList,
        VariableDeclarationListData
    );
    as_data!(as_binding_element, BindingElement, BindingElementData);
    as_data!(
        as_import_declaration,
        ImportDeclaration,
        ImportDeclarationData
    );
    as_data!(as_import_clause, ImportClause, ImportClauseData);
    as_data!(as_named_imports, NamedImports, NamedImportsData);
    as_data!(as_named_exports, NamedExports, NamedExportsData);
    as_data!(as_namespace_export, NamespaceExport, NamespaceExportData);
    as_data!(
        as_property_declaration,
        PropertyDeclaration,
        PropertyDeclarationData
    );

    fn elements(&self) -> &NodeList {
        match &self.data {
            NodeData::NamedImports(d) => &d.elements,
            NodeData::NamedExports(d) => &d.elements,
            NodeData::BindingPattern(d) => &d.elements,
            _ => panic!("Elements on wrong node kind"),
        }
    }

    fn property_name_or_name(&self) -> Arc<Node> {
        match &self.data {
            NodeData::ExportSpecifier(d) => {
                d.property_name.clone().unwrap_or_else(|| d.name.clone())
            }
            NodeData::ImportSpecifier(d) => {
                d.property_name.clone().unwrap_or_else(|| d.name.clone())
            }
            _ => self.name().unwrap().clone(),
        }
    }
}

impl<'a> NodeFactory<'a> {
    pub fn new_unscoped_helper_name(&self, text: &str) -> Arc<Node> {
        let generated = self.new_unique_name_ex(
            text,
            AutoGenerateOptions {
                flags: GeneratedIdentifierFlags::UNIQUE
                    | GeneratedIdentifierFlags::ALLOW_NAME_SUBSTITUTION,
                ..Default::default()
            },
        );
        self.generated_name_node(&generated)
    }

    pub fn new_external_module_reference(&self, expression: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ExternalModuleReference,
            NodeData::ExternalModuleReference(ExternalModuleReferenceData { expression }),
        ))
    }

    pub fn new_import_equals_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        is_type_only: bool,
        name: Arc<Node>,
        module_reference: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ImportEqualsDeclaration,
            NodeData::ImportEqualsDeclaration(ImportEqualsDeclarationData {
                modifiers,
                is_type_only,
                name,
                module_reference,
            }),
        ))
    }

    pub fn new_named_imports(&self, elements: Arc<NodeList>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NamedImports,
            NodeData::NamedImports(NamedImportsData { elements }),
        ))
    }

    pub fn new_import_specifier(
        &self,
        is_type_only: bool,
        property_name: Option<Arc<Node>>,
        name: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ImportSpecifier,
            NodeData::ImportSpecifier(ImportSpecifierData {
                is_type_only,
                property_name,
                name,
            }),
        ))
    }

    pub fn new_named_exports(&self, elements: &NodeList) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NamedExports,
            NodeData::NamedExports(NamedExportsData {
                elements: Arc::new(NodeList {
                    loc: elements.loc,
                    nodes: elements.nodes.clone(),
                }),
            }),
        ))
    }

    pub fn update_import_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        import_clause: Option<Arc<Node>>,
        module_specifier: Arc<Node>,
        attributes: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ImportDeclaration,
            NodeData::ImportDeclaration(ImportDeclarationData {
                modifiers,
                import_clause,
                module_specifier,
                attributes,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_import_clause(
        &self,
        node: &Arc<Node>,
        phase_modifier: Option<SyntaxKind>,
        name: Option<Arc<Node>>,
        named_bindings: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ImportClause,
            NodeData::ImportClause(ImportClauseData {
                phase_modifier,
                name,
                named_bindings,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_named_imports(&self, node: &Arc<Node>, elements: &NodeList) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::NamedImports,
            NodeData::NamedImports(NamedImportsData {
                elements: Arc::new(NodeList {
                    loc: elements.loc,
                    nodes: elements.nodes.clone(),
                }),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_named_exports(&self, node: &Arc<Node>, elements: &NodeList) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::NamedExports,
            NodeData::NamedExports(NamedExportsData {
                elements: Arc::new(NodeList {
                    loc: elements.loc,
                    nodes: elements.nodes.clone(),
                }),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_export_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        is_type_only: bool,
        export_clause: Option<Arc<Node>>,
        module_specifier: Option<Arc<Node>>,
        attributes: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ExportDeclaration,
            NodeData::ExportDeclaration(ExportDeclarationData {
                modifiers,
                is_type_only,
                export_clause,
                module_specifier,
                attributes,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

}
