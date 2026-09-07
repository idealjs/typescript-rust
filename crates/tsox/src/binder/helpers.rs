use crate::ast::*;
use std::sync::Arc;

pub(crate) fn collect_binding_elements<'a>(node: &'a Arc<Node>, out: &mut Vec<&'a Arc<Node>>) {
    if let NodeData::BindingPattern(pattern) = &node.data {
        for el in pattern.elements.iter() {
            out.push(el);
            let name = match &el.data {
                NodeData::BindingElement(be) => &be.name,
                _ => continue,
            };
            if let Some(name_node) = name
                && matches!(name_node.data, NodeData::BindingPattern(_))
            {
                collect_binding_elements(name_node, out);
            }
        }
    }
}

pub(crate) fn fn_like_body_present(parent: &Arc<Node>) -> bool {
    match &parent.data {
        NodeData::FunctionDeclaration(d) => d.body.is_some(),
        NodeData::MethodDeclaration(d) => d.body.is_some(),
        NodeData::ConstructorDeclaration(d) => d.body.is_some(),
        NodeData::GetAccessorDeclaration(d) => d.body.is_some(),
        NodeData::SetAccessorDeclaration(d) => d.body.is_some(),
        NodeData::FunctionExpression(_) | NodeData::ArrowFunction(_) => true,
        _ => false,
    }
}

pub(crate) fn clause_statements_empty(clause: &Arc<Node>) -> bool {
    matches!(&clause.data, NodeData::CaseOrDefaultClause(d) if d.statements.nodes.is_empty())
}

pub(crate) fn is_assignment_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
            | SyntaxKind::CaretEqualsToken
    )
}
