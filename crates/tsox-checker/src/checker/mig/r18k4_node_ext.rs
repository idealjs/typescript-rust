use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{
    ArrayTypeNodeData, BinaryExpressionData, ConditionalTypeNodeData, ElementAccessExpressionData,
    ExportSpecifierData, ForInOrOfStatementData, ImportEqualsDeclarationData,
    ParameterDeclarationData, PropertyAccessExpressionData, QualifiedNameData,
    ShorthandPropertyAssignmentData, TaggedTemplateExpressionData, TemplateExpressionData,
    TemplateSpanData, TypeQueryNodeData, VariableDeclarationListData,
};
use tsox_frontend::ast::{Node, NodeData, NodeList};

pub trait NodeAccessExt {
    fn as_binary_expression(&self) -> &BinaryExpressionData;
    fn as_element_access_expression(&self) -> &ElementAccessExpressionData;
    fn as_property_access_expression(&self) -> &PropertyAccessExpressionData;
    fn as_qualified_name(&self) -> &QualifiedNameData;
    fn as_tagged_template_expression(&self) -> &TaggedTemplateExpressionData;
    fn as_template_expression(&self) -> &TemplateExpressionData;
    fn as_template_span(&self) -> &TemplateSpanData;
    fn as_import_equals_declaration(&self) -> &ImportEqualsDeclarationData;
    fn as_conditional_type_node(&self) -> &ConditionalTypeNodeData;
    fn as_type_query_node(&self) -> &TypeQueryNodeData;
    fn as_array_type_node(&self) -> &ArrayTypeNodeData;
    fn as_export_specifier(&self) -> &ExportSpecifierData;
    fn as_parameter_declaration(&self) -> &ParameterDeclarationData;
    fn as_for_in_or_of_statement(&self) -> &ForInOrOfStatementData;
    fn as_shorthand_property_assignment(&self) -> &ShorthandPropertyAssignmentData;
    fn as_variable_declaration_list(&self) -> &VariableDeclarationListData;
    fn typ(&self) -> Option<&Arc<Node>>;
    fn attributes(&self) -> &Node;
    fn argument_list(&self) -> Option<&NodeList>;
    fn for_each_child<F: FnMut(&Arc<Node>) -> bool>(&self, visitor: F) -> bool;
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

impl NodeAccessExt for Node {
    as_data!(as_binary_expression, BinaryExpression, BinaryExpressionData);
    as_data!(
        as_element_access_expression,
        ElementAccessExpression,
        ElementAccessExpressionData
    );
    as_data!(
        as_property_access_expression,
        PropertyAccessExpression,
        PropertyAccessExpressionData
    );
    as_data!(as_qualified_name, QualifiedName, QualifiedNameData);
    as_data!(
        as_tagged_template_expression,
        TaggedTemplateExpression,
        TaggedTemplateExpressionData
    );
    as_data!(
        as_template_expression,
        TemplateExpression,
        TemplateExpressionData
    );
    as_data!(as_template_span, TemplateSpan, TemplateSpanData);
    as_data!(
        as_import_equals_declaration,
        ImportEqualsDeclaration,
        ImportEqualsDeclarationData
    );
    as_data!(
        as_conditional_type_node,
        ConditionalTypeNode,
        ConditionalTypeNodeData
    );
    as_data!(as_type_query_node, TypeQueryNode, TypeQueryNodeData);
    as_data!(as_array_type_node, ArrayTypeNode, ArrayTypeNodeData);
    as_data!(as_export_specifier, ExportSpecifier, ExportSpecifierData);
    as_data!(
        as_parameter_declaration,
        ParameterDeclaration,
        ParameterDeclarationData
    );
    as_data!(
        as_for_in_or_of_statement,
        ForInOrOfStatement,
        ForInOrOfStatementData
    );
    as_data!(
        as_shorthand_property_assignment,
        ShorthandPropertyAssignment,
        ShorthandPropertyAssignmentData
    );
    as_data!(
        as_variable_declaration_list,
        VariableDeclarationList,
        VariableDeclarationListData
    );

    fn typ(&self) -> Option<&Arc<Node>> {
        self.type_node()
    }

    fn attributes(&self) -> &Node {
        match &self.data {
            NodeData::JsxOpeningElement(d) => &d.attributes,
            NodeData::JsxSelfClosingElement(d) => &d.attributes,
            _ => panic!("AsJsxOpeningLikeElement on wrong node kind"),
        }
    }

    fn argument_list(&self) -> Option<&NodeList> {
        tsox_frontend::ast::mig::x1a::argument_list(self)
    }

    fn for_each_child<F: FnMut(&Arc<Node>) -> bool>(&self, visitor: F) -> bool {
        tsox_frontend::ast::node_data_generated::for_each_child(self, visitor)
    }
}
