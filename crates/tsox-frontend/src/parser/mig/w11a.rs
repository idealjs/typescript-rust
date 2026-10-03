use super::super::parsing_context::Parser;
use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, SyntaxKind, is_identifier, is_modifier_kind};
use crate::scanner::token_to_string::token_to_string;
use std::sync::Arc;
use tsox_core::diagnostics;

pub(crate) fn should_consume_binary_operator(
    operator: SyntaxKind,
    operator_precedence: i32,
    current_precedence: i32,
) -> bool { ::tsox_core::fntrace::enter("should_consume_binary_operator"); 
    if operator_precedence > current_precedence {
        return true;
    }
    operator_precedence == current_precedence && operator == SyntaxKind::AsteriskAsteriskToken
}

impl Parser {
    pub(crate) fn should_parse_return_type(&mut self, return_token: SyntaxKind, is_type: bool) -> bool { ::tsox_core::fntrace::enter("should_parse_return_type"); 
        if return_token == SyntaxKind::EqualsGreaterThanToken {
            self.expect(return_token);
            return true;
        } else if self.parse_optional(SyntaxKind::ColonToken) {
            return true;
        } else if is_type && self.token == SyntaxKind::EqualsGreaterThanToken {
            // This is easy to get backward, especially in type contexts, so parse the type anyway
            self.parse_error_at_current_token(
                diagnostics::X_0_EXPECTED,
                &[token_to_string(SyntaxKind::ColonToken)],
            );
            self.next_token();
            return true;
        }
        false
    }

    /// Go parseModifiers 的跳过形态:连续消费 modifier token
    pub(crate) fn parse_modifiers(&mut self) { ::tsox_core::fntrace::enter("parse_modifiers"); 
        while is_modifier_kind(self.token) {
            self.next_token();
        }
    }

    pub(crate) fn skip_parameter_start(&mut self) -> bool { ::tsox_core::fntrace::enter("skip_parameter_start"); 
        if is_modifier_kind(self.token) {
            // Skip modifiers(Go parseModifiers:本调用点仅消费 modifier token)
            self.parse_modifiers();
        }
        self.parse_optional(SyntaxKind::DotDotDotToken);
        if self.is_identifier() || self.token == SyntaxKind::ThisKeyword {
            self.next_token();
            return true;
        }
        if self.token == SyntaxKind::OpenBracketToken || self.token == SyntaxKind::OpenBraceToken {
            // Return true if we can parse an array or object binding pattern with no errors
            let previous_error_count = self.diagnostics.len();
            self.parse_identifier_or_pattern();
            return previous_error_count == self.diagnostics.len();
        }
        false
    }
}

pub(crate) fn texts_equal(a: &Arc<Node>, b: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("texts_equal"); 
    let mut a = Arc::clone(a);
    let mut b = Arc::clone(b);
    while !is_identifier(&a) || !is_identifier(&b) {
        let (NodeData::QualifiedName(a_qualified), NodeData::QualifiedName(b_qualified)) =
            (&a.data, &b.data)
        else {
            return false;
        };
        if a_qualified.right.text() == b_qualified.right.text() {
            let next_a = Arc::clone(&a_qualified.left);
            let next_b = Arc::clone(&b_qualified.left);
            a = next_a;
            b = next_b;
        } else {
            return false;
        }
    }
    a.text() == b.text()
}
