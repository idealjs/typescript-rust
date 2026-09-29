#![allow(dead_code, unused_imports, unused_variables)]

use tsox_frontend::ast::{Node, SyntaxKind};

pub fn mixing_binary_operators_requires_parentheses(a: SyntaxKind, b: SyntaxKind) -> bool {
    if a == SyntaxKind::QuestionQuestionToken {
        return b == SyntaxKind::AmpersandAmpersandToken || b == SyntaxKind::BarBarToken;
    }
    if b == SyntaxKind::QuestionQuestionToken {
        return a == SyntaxKind::AmpersandAmpersandToken || a == SyntaxKind::BarBarToken;
    }
    false
}
