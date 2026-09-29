#![allow(dead_code)]

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    BinaryExpressionData, BindingElementData, BindingPatternData, CallExpressionData,
    ElementAccessExpressionData, ExportAssignmentData, ExportDeclarationData,
    GetAccessorDeclarationData, HeritageClauseData, ImportDeclarationData,
    ImportEqualsDeclarationData, ImportTypeNodeData, IndexSignatureDeclarationData,
    InterfaceDeclarationData, JSDocNonNullableTypeData, JSDocNullableTypeData,
    JSDocOptionalTypeData, JSDocParameterOrPropertyTagData, JSDocTypeExpressionData,
    JSDocTypeLiteralData, JSDocVariadicTypeData, MethodDeclarationData,
    MethodSignatureDeclarationData, MappedTypeNodeData, ModuleDeclarationData,
    ParameterDeclarationData, PropertyDeclarationData, PropertySignatureDeclarationData,
    SetAccessorDeclarationData, SourceFileData, SyntaxListData, TypeAliasDeclarationData,
    TypeParameterDeclarationData, TypeQueryNodeData, TypeReferenceNodeData,
    VariableStatementData,
};

pub trait R37K1DataExt {
    fn as_source_file(&self) -> &SourceFileData;
    fn as_statement_list(&self) -> &SyntaxListData;
    fn as_syntax_list(&self) -> &SyntaxListData;
    fn as_heritage_clause(&self) -> &HeritageClauseData;
    fn as_module_declaration(&self) -> &ModuleDeclarationData;
    fn as_binary_expression(&self) -> &BinaryExpressionData;
    fn as_binding_element(&self) -> &BindingElementData;
    fn as_binding_pattern(&self) -> &BindingPatternData;
    fn as_index_signature_declaration(&self) -> &IndexSignatureDeclarationData;
    fn as_set_accessor_declaration(&self) -> &SetAccessorDeclarationData;
    fn as_get_accessor_declaration(&self) -> &GetAccessorDeclarationData;
    fn as_method_declaration(&self) -> &MethodDeclarationData;
    fn as_method_signature_declaration(&self) -> &MethodSignatureDeclarationData;
    fn as_property_signature_declaration(&self) -> &PropertySignatureDeclarationData;
    fn as_property_declaration(&self) -> &PropertyDeclarationData;
    fn as_parameter_declaration(&self) -> &ParameterDeclarationData;
    fn as_type_parameter_declaration(&self) -> &TypeParameterDeclarationData;
    fn as_variable_statement(&self) -> &VariableStatementData;
    fn as_type_reference_node(&self) -> &TypeReferenceNodeData;
    fn as_type_query_node(&self) -> &TypeQueryNodeData;
    fn as_type_alias_declaration(&self) -> &TypeAliasDeclarationData;
    fn as_mapped_type_node(&self) -> &MappedTypeNodeData;
    fn as_interface_declaration(&self) -> &InterfaceDeclarationData;
    fn as_import_type_node(&self) -> &ImportTypeNodeData;
    fn as_import_equals_declaration(&self) -> &ImportEqualsDeclarationData;
    fn as_import_declaration(&self) -> &ImportDeclarationData;
    fn as_export_declaration(&self) -> &ExportDeclarationData;
    fn as_export_assignment(&self) -> &ExportAssignmentData;
    fn as_element_access_expression(&self) -> &ElementAccessExpressionData;
    fn as_call_expression(&self) -> &CallExpressionData;
    fn as_jsdoc_variadic_type(&self) -> &JSDocVariadicTypeData;
    fn as_jsdoc_optional_type(&self) -> &JSDocOptionalTypeData;
    fn as_jsdoc_non_nullable_type(&self) -> &JSDocNonNullableTypeData;
    fn as_jsdoc_nullable_type(&self) -> &JSDocNullableTypeData;
    fn as_jsdoc_type_literal(&self) -> &JSDocTypeLiteralData;
    fn as_jsdoc_type_expression(&self) -> &JSDocTypeExpressionData;
    fn as_jsdoc_parameter_or_property_tag(&self) -> &JSDocParameterOrPropertyTagData;
}

macro_rules! as_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                tsox_frontend::ast::NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl R37K1DataExt for Node {
    as_data!(as_source_file, SourceFile, SourceFileData);
    as_data!(as_statement_list, SyntaxList, SyntaxListData);
    as_data!(as_syntax_list, SyntaxList, SyntaxListData);
    as_data!(as_heritage_clause, HeritageClause, HeritageClauseData);
    as_data!(as_module_declaration, ModuleDeclaration, ModuleDeclarationData);
    as_data!(as_binary_expression, BinaryExpression, BinaryExpressionData);
    as_data!(as_binding_element, BindingElement, BindingElementData);
    as_data!(as_binding_pattern, BindingPattern, BindingPatternData);
    as_data!(
        as_index_signature_declaration,
        IndexSignatureDeclaration,
        IndexSignatureDeclarationData
    );
    as_data!(
        as_set_accessor_declaration,
        SetAccessorDeclaration,
        SetAccessorDeclarationData
    );
    as_data!(
        as_get_accessor_declaration,
        GetAccessorDeclaration,
        GetAccessorDeclarationData
    );
    as_data!(as_method_declaration, MethodDeclaration, MethodDeclarationData);
    as_data!(
        as_method_signature_declaration,
        MethodSignatureDeclaration,
        MethodSignatureDeclarationData
    );
    as_data!(
        as_property_signature_declaration,
        PropertySignatureDeclaration,
        PropertySignatureDeclarationData
    );
    as_data!(as_property_declaration, PropertyDeclaration, PropertyDeclarationData);
    as_data!(
        as_parameter_declaration,
        ParameterDeclaration,
        ParameterDeclarationData
    );
    as_data!(
        as_type_parameter_declaration,
        TypeParameterDeclaration,
        TypeParameterDeclarationData
    );
    as_data!(as_variable_statement, VariableStatement, VariableStatementData);
    as_data!(as_type_reference_node, TypeReferenceNode, TypeReferenceNodeData);
    as_data!(as_type_query_node, TypeQueryNode, TypeQueryNodeData);
    as_data!(as_type_alias_declaration, TypeAliasDeclaration, TypeAliasDeclarationData);
    as_data!(as_mapped_type_node, MappedTypeNode, MappedTypeNodeData);
    as_data!(as_interface_declaration, InterfaceDeclaration, InterfaceDeclarationData);
    as_data!(as_import_type_node, ImportTypeNode, ImportTypeNodeData);
    as_data!(
        as_import_equals_declaration,
        ImportEqualsDeclaration,
        ImportEqualsDeclarationData
    );
    as_data!(as_import_declaration, ImportDeclaration, ImportDeclarationData);
    as_data!(as_export_declaration, ExportDeclaration, ExportDeclarationData);
    as_data!(as_export_assignment, ExportAssignment, ExportAssignmentData);
    as_data!(
        as_element_access_expression,
        ElementAccessExpression,
        ElementAccessExpressionData
    );
    as_data!(as_call_expression, CallExpression, CallExpressionData);
    as_data!(as_jsdoc_variadic_type, JSDocVariadicType, JSDocVariadicTypeData);
    as_data!(as_jsdoc_optional_type, JSDocOptionalType, JSDocOptionalTypeData);
    as_data!(
        as_jsdoc_non_nullable_type,
        JSDocNonNullableType,
        JSDocNonNullableTypeData
    );
    as_data!(as_jsdoc_nullable_type, JSDocNullableType, JSDocNullableTypeData);
    as_data!(as_jsdoc_type_literal, JSDocTypeLiteral, JSDocTypeLiteralData);
    as_data!(as_jsdoc_type_expression, JSDocTypeExpression, JSDocTypeExpressionData);
    as_data!(
        as_jsdoc_parameter_or_property_tag,
        JSDocParameterOrPropertyTag,
        JSDocParameterOrPropertyTagData
    );
}
