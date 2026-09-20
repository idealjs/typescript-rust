#![allow(unused_imports)]

use crate::checker::*;
use tsox_frontend::ast::{Node, NodeData};
use std::sync::Arc;

impl Checker {
    pub(crate) fn check_return_expression_against_type(
        &mut self,
        expected: &Arc<Type>,
        anchor: &Arc<Node>,
        expr: &Arc<Node>,
        in_conditional: bool,
        in_return_statement: bool,
    ) {
        let unwrapped = Checker::skip_parentheses(expr);
        if let NodeData::ConditionalExpression(d) = &unwrapped.data {
            for branch in [&d.when_true, &d.when_false] {
                self.check_expression(branch);
                self.check_return_expression_against_type(
                    expected,
                    anchor,
                    branch,
                    true,
                    in_return_statement,
                );
            }
            return;
        }
        let actual = self.get_type_of_node(expr);
        if !actual.flags.contains(TypeFlags::Any)
            && !self.is_type_assignable_to(&actual, expected)
        {
            let error_node = if in_return_statement && !in_conditional {
                Arc::clone(anchor)
            } else {
                Arc::clone(&unwrapped)
            };
            self.check_type_related_to_and_optionally_elaborate(
                &actual,
                expected,
                crate::checker::relater::RelationKind::Assignable,
                Some(&error_node),
                Some(&unwrapped),
                None,
                None,
            );
        }
    }
}
