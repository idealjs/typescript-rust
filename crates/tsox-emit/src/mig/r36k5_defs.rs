#![allow(dead_code)]

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    BinaryExpressionData, ConditionalExpressionData, ConditionalTypeNodeData,
    IntersectionTypeNodeData, LiteralTypeNodeData, NodeData, ParameterDeclarationData,
    PrefixUnaryExpressionData, QualifiedNameData, TypeOperatorNodeData, TypePredicateNodeData,
    TypeReferenceNodeData, UnionTypeNodeData,
};

pub trait NodeDataExt {
    fn as_parameter_declaration(&self) -> &ParameterDeclarationData;
    fn as_type_predicate_node(&self) -> &TypePredicateNodeData;
    fn as_literal_type_node(&self) -> &LiteralTypeNodeData;
    fn as_type_reference_node(&self) -> &TypeReferenceNodeData;
    fn as_intersection_type_node(&self) -> &IntersectionTypeNodeData;
    fn as_union_type_node(&self) -> &UnionTypeNodeData;
    fn as_conditional_type_node(&self) -> &ConditionalTypeNodeData;
    fn as_type_operator_node(&self) -> &TypeOperatorNodeData;
    fn as_prefix_unary_expression(&self) -> &PrefixUnaryExpressionData;
    fn as_qualified_name(&self) -> &QualifiedNameData;
    fn as_conditional_expression(&self) -> &ConditionalExpressionData;
    fn as_binary_expression(&self) -> &BinaryExpressionData;
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

impl NodeDataExt for Node {
    as_data!(
        as_parameter_declaration,
        ParameterDeclaration,
        ParameterDeclarationData
    );
    as_data!(as_type_predicate_node, TypePredicateNode, TypePredicateNodeData);
    as_data!(as_literal_type_node, LiteralTypeNode, LiteralTypeNodeData);
    as_data!(as_type_reference_node, TypeReferenceNode, TypeReferenceNodeData);
    as_data!(
        as_intersection_type_node,
        IntersectionTypeNode,
        IntersectionTypeNodeData
    );
    as_data!(as_union_type_node, UnionTypeNode, UnionTypeNodeData);
    as_data!(
        as_conditional_type_node,
        ConditionalTypeNode,
        ConditionalTypeNodeData
    );
    as_data!(as_type_operator_node, TypeOperatorNode, TypeOperatorNodeData);
    as_data!(
        as_prefix_unary_expression,
        PrefixUnaryExpression,
        PrefixUnaryExpressionData
    );
    as_data!(as_qualified_name, QualifiedName, QualifiedNameData);
    as_data!(
        as_conditional_expression,
        ConditionalExpression,
        ConditionalExpressionData
    );
    as_data!(as_binary_expression, BinaryExpression, BinaryExpressionData);
}
