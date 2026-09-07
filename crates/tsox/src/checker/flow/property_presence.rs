use std::sync::Arc;

use crate::ast::{Node, NodeData, SyntaxKind};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PropertyPresence {
    Definitely,

    Maybe,

    DefinitelyNot,
}

impl PropertyPresence {
    pub(crate) fn is_definitely(self) -> bool {
        matches!(self, PropertyPresence::Definitely)
    }
    pub(crate) fn is_definitely_not(self) -> bool {
        matches!(self, PropertyPresence::DefinitelyNot)
    }
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
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
    )
}

pub(crate) fn clauses_of_range(
    switch_stmt: &Arc<Node>,
    start: usize,
    end: usize,
) -> Vec<Arc<Node>> {
    let NodeData::SwitchStatement(sd) = &switch_stmt.data else {
        return Vec::new();
    };
    let NodeData::CaseBlock(cb) = &sd.case_block.data else {
        return Vec::new();
    };
    let clauses = &cb.clauses.nodes;
    let start = start.min(clauses.len());
    let end = end.max(start).min(clauses.len());
    clauses[start..end].to_vec()
}
