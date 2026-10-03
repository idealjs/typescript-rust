use crate::ast::*;
use std::sync::Arc;

pub fn is_compound_assignment(token: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_compound_assignment"); 
    (token as i16) >= (SyntaxKind::PlusEqualsToken as i16)
        && (token as i16) <= (SyntaxKind::CaretEqualsToken as i16)
}

pub fn is_logical_binary_operator(token: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_logical_binary_operator"); 
    token == SyntaxKind::BarBarToken || token == SyntaxKind::AmpersandAmpersandToken
}

pub fn is_logical_or_coalescing_binary_operator(token: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_logical_or_coalescing_binary_operator"); 
    is_logical_binary_operator(token) || token == SyntaxKind::QuestionQuestionToken
}

pub fn skip_partially_emitted_expressions_arc(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("skip_partially_emitted_expressions_arc"); 
    let mut current = Arc::clone(node);
    while is_partially_emitted_expression(&current) {
        if let Some(inner) = current.expression() {
            current = Arc::clone(inner);
        } else {
            break;
        }
    }
    current
}

fn is_left_hand_side_expression_kind(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_left_hand_side_expression_kind"); 
    matches!(
        kind,
        SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionExpression
            | SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateExpression
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::NonNullExpression
            | SyntaxKind::ExpressionWithTypeArguments
            | SyntaxKind::MetaProperty
            | SyntaxKind::ImportKeyword
            | SyntaxKind::MissingDeclaration
    )
}

pub fn is_left_hand_side_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_left_hand_side_expression"); 
    is_left_hand_side_expression_kind(skip_partially_emitted_expressions_kind(node))
}

fn is_unary_expression_kind(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_unary_expression_kind"); 
    matches!(
        kind,
        SyntaxKind::PrefixUnaryExpression
            | SyntaxKind::PostfixUnaryExpression
            | SyntaxKind::DeleteExpression
            | SyntaxKind::TypeOfExpression
            | SyntaxKind::VoidExpression
            | SyntaxKind::AwaitExpression
            | SyntaxKind::TypeAssertionExpression
    ) || is_left_hand_side_expression_kind(kind)
}

pub fn is_unary_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_unary_expression"); 
    is_unary_expression_kind(skip_partially_emitted_expressions_kind(node))
}

fn skip_partially_emitted_expressions_kind(node: &Node) -> SyntaxKind { ::tsox_core::fntrace::enter("skip_partially_emitted_expressions_kind"); 
    let mut current = node;
    loop {
        if !is_partially_emitted_expression(current) {
            return current.kind;
        }
        match current.expression() {
            Some(inner) => current = inner,
            None => return current.kind,
        }
    }
}

fn is_expression_kind(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_expression_kind"); 
    matches!(
        kind,
        SyntaxKind::ConditionalExpression
            | SyntaxKind::YieldExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::BinaryExpression
            | SyntaxKind::SpreadElement
            | SyntaxKind::AsExpression
            | SyntaxKind::OmittedExpression
            | SyntaxKind::PartiallyEmittedExpression
            | SyntaxKind::SatisfiesExpression
    ) || is_unary_expression_kind(kind)
}

pub fn is_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_expression"); 
    is_expression_kind(skip_partially_emitted_expressions_kind(node))
}

pub fn is_comma_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_comma_expression"); 
    if let NodeData::BinaryExpression(d) = &node.data {
        return d.operator_token.kind == SyntaxKind::CommaToken;
    }
    false
}

pub fn is_nullish_coalesce(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_nullish_coalesce"); 
    if let NodeData::BinaryExpression(d) = &node.data {
        return d.operator_token.kind == SyntaxKind::QuestionQuestionToken;
    }
    false
}

pub fn is_assertion_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_assertion_expression"); 
    matches!(
        node.kind,
        SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression
    )
}

pub fn is_boolean_literal(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_boolean_literal"); 
    matches!(
        node.kind,
        SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword
    )
}

pub fn is_literal_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_literal_expression"); 
    is_literal_kind(node.kind)
}

pub fn is_string_literal_like(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_string_literal_like"); 
    matches!(
        node.kind,
        SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
    )
}

pub fn is_string_or_numeric_literal_like(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_string_or_numeric_literal_like"); 
    is_string_literal_like(node) || is_numeric_literal(node)
}

pub fn is_access_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_access_expression"); 
    matches!(
        node.kind,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
    )
}

pub fn is_optional_chain(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_optional_chain"); 
    if node.flags.contains(NodeFlags::OptionalChain) {
        matches!(
            node.kind,
            SyntaxKind::PropertyAccessExpression
                | SyntaxKind::ElementAccessExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::NonNullExpression
        )
    } else {
        false
    }
}

pub fn is_assignment_expression(node: &Node, exclude_compound_assignment: bool) -> bool { ::tsox_core::fntrace::enter("is_assignment_expression"); 
    if let NodeData::BinaryExpression(d) = &node.data {
        return (d.operator_token.kind == SyntaxKind::EqualsToken
            || (!exclude_compound_assignment && is_assignment_operator(d.operator_token.kind)))
            && is_left_hand_side_expression(&d.left);
    }
    false
}
