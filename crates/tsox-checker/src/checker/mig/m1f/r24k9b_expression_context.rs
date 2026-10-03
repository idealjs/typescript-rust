//! Go ast.IsExpressionNode / IsInExpressionContext 全量移植（utilities.go:2211-2281），
//! 供 .types 基线发射过滤、get_type_of_symbol_at_location 与
//! get_symbol_of_name_or_property_access_expression 共用。
#![allow(unused_imports)]

use std::sync::Arc;
use tsox_frontend::ast::{is_import_call, Node, NodeData, SyntaxKind};

pub(crate) fn is_expression_node(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_expression_node");
    match node.kind {
        SyntaxKind::SuperKeyword
        | SyntaxKind::NullKeyword
        | SyntaxKind::TrueKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::RegularExpressionLiteral
        | SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::PropertyAccessExpression
        | SyntaxKind::ElementAccessExpression
        | SyntaxKind::CallExpression
        | SyntaxKind::NewExpression
        | SyntaxKind::TaggedTemplateExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::SatisfiesExpression
        | SyntaxKind::NonNullExpression
        | SyntaxKind::ParenthesizedExpression
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ClassExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::VoidExpression
        | SyntaxKind::DeleteExpression
        | SyntaxKind::TypeOfExpression
        | SyntaxKind::PrefixUnaryExpression
        | SyntaxKind::PostfixUnaryExpression
        | SyntaxKind::BinaryExpression
        | SyntaxKind::ConditionalExpression
        | SyntaxKind::SpreadElement
        | SyntaxKind::TemplateExpression
        | SyntaxKind::OmittedExpression
        | SyntaxKind::JsxElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxFragment
        | SyntaxKind::YieldExpression
        | SyntaxKind::AwaitExpression => true,
        SyntaxKind::MetaProperty => node.parent().map_or(true, |parent| {
            !(is_import_call(&parent)
                && parent
                    .expression()
                    .is_some_and(|e| std::ptr::eq(e.as_ref(), node)))
        }),
        SyntaxKind::ExpressionWithTypeArguments => !node
            .parent()
            .is_some_and(|p| p.kind == SyntaxKind::HeritageClause),
        SyntaxKind::QualifiedName => {
            let Some(mut container) = node.parent() else {
                return false;
            };
            let mut tail: Option<Arc<Node>> = None;
            while container.kind == SyntaxKind::QualifiedName {
                let Some(next) = container.parent() else {
                    return false;
                };
                let moved = std::mem::replace(&mut container, next);
                tail = Some(moved);
            }
            is_type_query_node(&container)
                || is_jsdoc_link_like(&container)
                || container.kind == SyntaxKind::JSDocNameReference
                || is_jsx_tag_name(tail.as_deref().unwrap_or(node))
        }
        SyntaxKind::PrivateIdentifier => node.parent().is_some_and(|p| {
            matches!(&p.data, NodeData::BinaryExpression(b)
                if std::ptr::eq(b.left.as_ref(), node) && b.operator_token.kind == SyntaxKind::InKeyword)
        }),
        SyntaxKind::Identifier => {
            let Some(p) = node.parent() else {
                return false;
            };
            is_type_query_node(&p)
                || is_jsdoc_link_like(&p)
                || p.kind == SyntaxKind::JSDocNameReference
                || is_jsx_tag_name(node)
                || is_in_expression_context(node)
        }
        SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::StringLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::ThisKeyword => is_in_expression_context(node),
        _ => false,
    }
}

pub(crate) fn is_in_expression_context(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_in_expression_context");
    let Some(parent) = node.parent() else {
        return false;
    };
    let is = |child: &Arc<Node>| std::ptr::eq(child.as_ref(), node);
    match parent.kind {
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::EnumMember
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::BindingElement => initializer_of(&parent).is_some_and(|i| is(&i)),
        SyntaxKind::ExpressionStatement
        | SyntaxKind::IfStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::ReturnStatement
        | SyntaxKind::WithStatement
        | SyntaxKind::SwitchStatement
        | SyntaxKind::CaseClause
        | SyntaxKind::DefaultClause
        | SyntaxKind::ThrowStatement
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::TemplateSpan
        | SyntaxKind::ComputedPropertyName
        | SyntaxKind::SatisfiesExpression => parent.expression().is_some_and(|e| is(&e)),
        SyntaxKind::ForStatement => match &parent.data {
            NodeData::ForStatement(d) => {
                d.initializer.as_ref().is_some_and(|i| is(i) && i.kind != SyntaxKind::VariableDeclarationList)
                    || d.condition.as_ref().is_some_and(|c| is(c))
                    || d.incrementor.as_ref().is_some_and(|i| is(i))
            }
            _ => false,
        },
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => match &parent.data {
            NodeData::ForInOrOfStatement(d) => {
                (is(&d.initializer) && d.initializer.kind != SyntaxKind::VariableDeclarationList)
                    || is(&d.expression)
            }
            _ => false,
        },
        SyntaxKind::Decorator
        | SyntaxKind::JsxExpression
        | SyntaxKind::JsxSpreadAttribute
        | SyntaxKind::SpreadAssignment => true,
        SyntaxKind::ExpressionWithTypeArguments => {
            parent.expression().is_some_and(|e| is(&e))
                && !tsox_frontend::ast::mig::m3g_3::is_part_of_type_node(&parent)
        }
        SyntaxKind::ShorthandPropertyAssignment => {
            matches!(&parent.data, NodeData::ShorthandPropertyAssignment(sd)
                if sd.object_assignment_initializer.as_ref().is_some_and(|i| is(i)))
        }
        _ => is_expression_node(&parent),
    }
}

fn initializer_of(parent: &Arc<Node>) -> Option<Arc<Node>> {
    match &parent.data {
        NodeData::VariableDeclaration(d) => d.initializer.clone(),
        NodeData::ParameterDeclaration(d) => d.initializer.clone(),
        NodeData::PropertyDeclaration(d) => d.initializer.clone(),
        NodeData::PropertySignatureDeclaration(d) => Some(Arc::clone(&d.initializer)),
        NodeData::EnumMember(d) => d.initializer.clone(),
        NodeData::PropertyAssignment(d) => Some(Arc::clone(&d.initializer)),
        NodeData::BindingElement(d) => d.initializer.clone(),
        _ => None,
    }
}

fn is_type_query_node(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::TypeQuery
}

fn is_jsdoc_link_like(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::JSDocLink | SyntaxKind::JSDocLinkCode | SyntaxKind::JSDocLinkPlain
    )
}

fn is_jsx_tag_name(node: &Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::JsxOpeningElement => matches!(
            &parent.data,
            NodeData::JsxOpeningElement(d) if std::ptr::eq(d.tag_name.as_ref(), node)
        ),
        SyntaxKind::JsxClosingElement => matches!(
            &parent.data,
            NodeData::JsxClosingElement(d) if std::ptr::eq(d.tag_name.as_ref(), node)
        ),
        SyntaxKind::JsxSelfClosingElement => matches!(
            &parent.data,
            NodeData::JsxSelfClosingElement(d) if std::ptr::eq(d.tag_name.as_ref(), node)
        ),
        _ => false,
    }
}
