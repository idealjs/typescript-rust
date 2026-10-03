use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{
    BigIntLiteralData, IdentifierData, NumericLiteralData, PrefixUnaryExpressionData,
    StringLiteralData,
};
use tsox_frontend::ast::{Node, NodeData, SyntaxKind};

use crate::checker::types_type_id::TypeFlags;

pub const TYPE_FLAGS_BIGINT_LIKE: TypeFlags =
    TypeFlags::BigInt.union(TypeFlags::BigIntLiteral);

pub const TYPE_FLAGS_ESSYMBOL_LIKE: TypeFlags =
    TypeFlags::ESSymbol.union(TypeFlags::UniqueESSymbol);

pub fn new_identifier(text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_identifier"); 
    Arc::new(Node::new(
        SyntaxKind::Identifier,
        NodeData::Identifier(IdentifierData {
            text: text.to_string(),
        }),
    ))
}

pub fn new_string_literal(text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_string_literal"); 
    Arc::new(Node::new(
        SyntaxKind::StringLiteral,
        NodeData::StringLiteral(StringLiteralData {
            text: text.to_string(),
            token_flags: Default::default(),
        }),
    ))
}

pub fn new_numeric_literal(text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_numeric_literal"); 
    Arc::new(Node::new(
        SyntaxKind::NumericLiteral,
        NodeData::NumericLiteral(NumericLiteralData {
            text: text.to_string(),
            token_flags: Default::default(),
        }),
    ))
}

pub fn new_big_int_literal(text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_big_int_literal"); 
    Arc::new(Node::new(
        SyntaxKind::BigIntLiteral,
        NodeData::BigIntLiteral(BigIntLiteralData {
            text: text.to_string(),
            token_flags: Default::default(),
        }),
    ))
}

pub fn new_keyword_expression(kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_keyword_expression"); 
    Arc::new(Node::new(kind, NodeData::Token))
}

pub fn new_keyword_type_node(kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_keyword_type_node"); 
    Arc::new(Node::new(kind, NodeData::Token))
}

pub fn new_prefix_unary_expression(operator: SyntaxKind, operand: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_prefix_unary_expression"); 
    Arc::new(Node::new(
        SyntaxKind::PrefixUnaryExpression,
        NodeData::PrefixUnaryExpression(PrefixUnaryExpressionData { operator, operand }),
    ))
}

pub fn pseudo_big_int_to_string(value: &tsox_core::jsnum::PseudoBigInt) -> String { ::tsox_core::fntrace::enter("pseudo_big_int_to_string"); 
    value.to_string()
}
