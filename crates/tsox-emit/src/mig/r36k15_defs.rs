#![allow(dead_code)]

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated::{
    BigIntLiteralData, ComputedPropertyNameData, IdentifierData, InferTypeNodeData,
    NoSubstitutionTemplateLiteralData, NumericLiteralData, PartiallyEmittedExpressionData,
    PostfixUnaryExpressionData, PrefixUnaryExpressionData, PrivateIdentifierData,
    PropertyAssignmentData, PropertyDeclarationData, PropertySignatureDeclarationData,
    QualifiedNameData, RestTypeNodeData, ReturnStatementData, SatisfiesExpressionData,
    ShorthandPropertyAssignmentData, SpreadAssignmentData, SpreadElementData,
    StringLiteralData, TypeParameterDeclarationData,
};

pub trait NodeDataExt15 {
    fn as_partially_emitted_expression(&self) -> &PartiallyEmittedExpressionData;
    fn as_postfix_unary_expression(&self) -> &PostfixUnaryExpressionData;
    fn as_prefix_unary_expression(&self) -> &PrefixUnaryExpressionData;
    fn as_property_assignment(&self) -> &PropertyAssignmentData;
    fn as_property_declaration(&self) -> &PropertyDeclarationData;
    fn as_property_signature_declaration(&self) -> &PropertySignatureDeclarationData;
    fn as_qualified_name(&self) -> &QualifiedNameData;
    fn as_rest_type_node(&self) -> &RestTypeNodeData;
    fn as_return_statement(&self) -> &ReturnStatementData;
    fn as_satisfies_expression(&self) -> &SatisfiesExpressionData;
    fn as_shorthand_property_assignment(&self) -> &ShorthandPropertyAssignmentData;
    fn as_spread_assignment(&self) -> &SpreadAssignmentData;
    fn as_spread_element(&self) -> &SpreadElementData;
    fn as_identifier(&self) -> &IdentifierData;
    fn as_private_identifier(&self) -> &PrivateIdentifierData;
    fn as_string_literal(&self) -> &StringLiteralData;
    fn as_no_substitution_template_literal(&self) -> &NoSubstitutionTemplateLiteralData;
    fn as_numeric_literal(&self) -> &NumericLiteralData;
    fn as_big_int_literal(&self) -> &BigIntLiteralData;
    fn as_computed_property_name(&self) -> &ComputedPropertyNameData;
    fn as_infer_type_node(&self) -> &InferTypeNodeData;
    fn as_type_parameter_declaration(&self) -> &TypeParameterDeclarationData;
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

impl NodeDataExt15 for Node {
    as_data!(
        as_partially_emitted_expression,
        PartiallyEmittedExpression,
        PartiallyEmittedExpressionData
    );
    as_data!(
        as_postfix_unary_expression,
        PostfixUnaryExpression,
        PostfixUnaryExpressionData
    );
    as_data!(
        as_prefix_unary_expression,
        PrefixUnaryExpression,
        PrefixUnaryExpressionData
    );
    as_data!(
        as_property_assignment,
        PropertyAssignment,
        PropertyAssignmentData
    );
    as_data!(
        as_property_declaration,
        PropertyDeclaration,
        PropertyDeclarationData
    );
    as_data!(
        as_property_signature_declaration,
        PropertySignatureDeclaration,
        PropertySignatureDeclarationData
    );
    as_data!(as_qualified_name, QualifiedName, QualifiedNameData);
    as_data!(as_rest_type_node, RestTypeNode, RestTypeNodeData);
    as_data!(as_return_statement, ReturnStatement, ReturnStatementData);
    as_data!(
        as_satisfies_expression,
        SatisfiesExpression,
        SatisfiesExpressionData
    );
    as_data!(
        as_shorthand_property_assignment,
        ShorthandPropertyAssignment,
        ShorthandPropertyAssignmentData
    );
    as_data!(as_spread_assignment, SpreadAssignment, SpreadAssignmentData);
    as_data!(as_spread_element, SpreadElement, SpreadElementData);
    as_data!(as_identifier, Identifier, IdentifierData);
    as_data!(as_private_identifier, PrivateIdentifier, PrivateIdentifierData);
    as_data!(as_string_literal, StringLiteral, StringLiteralData);
    as_data!(
        as_no_substitution_template_literal,
        NoSubstitutionTemplateLiteral,
        NoSubstitutionTemplateLiteralData
    );
    as_data!(as_numeric_literal, NumericLiteral, NumericLiteralData);
    as_data!(as_big_int_literal, BigIntLiteral, BigIntLiteralData);
    as_data!(
        as_computed_property_name,
        ComputedPropertyName,
        ComputedPropertyNameData
    );
    as_data!(as_infer_type_node, InferTypeNode, InferTypeNodeData);
    as_data!(
        as_type_parameter_declaration,
        TypeParameterDeclaration,
        TypeParameterDeclarationData
    );
}
