#![allow(dead_code, unused_imports, unused_variables)]

use super::super::rule::TokenRange;
use crate::ast::SyntaxKind;

pub fn token_range_from(tokens: Vec<SyntaxKind>) -> TokenRange {
    TokenRange {
        tokens,
        is_specific: true,
    }
}

pub fn to_token_range(kind: SyntaxKind) -> TokenRange {
    token_range_from(vec![kind])
}

pub fn to_token_range_many(tokens: Vec<SyntaxKind>) -> TokenRange {
    token_range_from(tokens)
}
