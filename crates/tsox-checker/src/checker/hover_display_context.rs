//! Go ast.IsInExpressionContext / IsThisInTypeQuery 移植：this 的上下文归属判定。

use std::sync::Arc;

use tsox_frontend::ast::{Node, SyntaxKind};

/// Go ast.IsInExpressionContext：父节点类别 + 表达式归属判定
pub(crate) fn is_in_expression_context(node: &Arc<Node>) -> bool {
    use tsox_frontend::ast::NodeData;
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };
    let is = |child: &Arc<Node>| Arc::ptr_eq(child, node);
    match parent.kind {
        SyntaxKind::VariableDeclaration => {
            matches!(&parent.data, NodeData::VariableDeclaration(d) if d.initializer.as_ref().is_some_and(|i| is(i)))
        }
        SyntaxKind::Parameter
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::EnumMember
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::BindingElement => {
            node_initializer(parent).is_some_and(|i| is(&i))
        }
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
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
            parent.expression().is_some_and(|e| is(&e))
        }
        SyntaxKind::Decorator
        | SyntaxKind::JsxExpression
        | SyntaxKind::JsxSpreadAttribute
        | SyntaxKind::SpreadAssignment => true,
        SyntaxKind::ShorthandPropertyAssignment => {
            matches!(&parent.data, NodeData::ShorthandPropertyAssignment(sd) if sd.object_assignment_initializer.as_ref().is_some_and(|i| is(i)))
        }
        _ => is_expression_node(parent),
    }
}

/// Go ast.IsExpressionNode 的常用子集
fn is_expression_node(node: &Arc<Node>) -> bool {
    use SyntaxKind::*;
    matches!(
        node.kind,
        Identifier
            | StringLiteral
            | NumericLiteral
            | NoSubstitutionTemplateLiteral
            | TemplateExpression
            | ThisKeyword
            | SuperKeyword
            | TrueKeyword
            | FalseKeyword
            | NullKeyword
            | ArrayLiteralExpression
            | ObjectLiteralExpression
            | PropertyAccessExpression
            | ElementAccessExpression
            | CallExpression
            | NewExpression
            | BinaryExpression
            | PrefixUnaryExpression
            | PostfixUnaryExpression
            | ConditionalExpression
            | ArrowFunction
            | FunctionExpression
            | ClassExpression
            | ParenthesizedExpression
            | NonNullExpression
            | AsExpression
            | SatisfiesExpression
            | TypeAssertionExpression
            | AwaitExpression
            | DeleteExpression
            | TypeOfExpression
            | VoidExpression
            | YieldExpression
            | TaggedTemplateExpression
            | JsxElement
            | JsxSelfClosingElement
            | JsxExpression
            | MetaProperty
    )
}

fn node_initializer(parent: &Arc<Node>) -> Option<Arc<Node>> {
    match &parent.data {
        tsox_frontend::ast::NodeData::VariableDeclaration(d) => d.initializer.clone(),
        tsox_frontend::ast::NodeData::ParameterDeclaration(d) => d.initializer.clone(),
        tsox_frontend::ast::NodeData::PropertyDeclaration(d) => d.initializer.clone(),
        tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) => Some(Arc::clone(&d.initializer)),
        tsox_frontend::ast::NodeData::EnumMember(d) => d.initializer.clone(),
        tsox_frontend::ast::NodeData::PropertyAssignment(d) => Some(Arc::clone(&d.initializer)),
        tsox_frontend::ast::NodeData::BindingElement(d) => d.initializer.clone(),
        _ => None,
    }
}

/// Go ast.IsThisInTypeQuery：typeof 查询中的 this
pub(crate) fn is_this_in_type_query(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };
    if parent.kind != SyntaxKind::TypeOfExpression {
        return false;
    }
    matches!(&parent.data, tsox_frontend::ast::NodeData::TypeOfExpression(d) if Arc::ptr_eq(&d.expression, node))
}
