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

    pub(crate) fn is_thenable_type(&mut self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::Object) {
            return false;
        }
        let Some(then_fn) = self.get_property_of_type(t, "then") else {
            return false;
        };
        let then_type = self.get_type_of_symbol(&then_fn);
        !self
            .get_signatures_of_type(&then_type, SignatureKind::Call)
            .is_empty()
    }

    pub(crate) fn check_awaited_type_no_alias(
        &mut self,
        t: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
        message: tsox_core::diagnostics::Message,
    ) -> Option<Arc<Type>> {
        if self.get_promised_type_of_promise(t).is_none() && self.is_thenable_type(t) {
            if let Some(node) = error_node {
                let file = self.get_source_file_of_node(node);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    message,
                    Vec::new(),
                ));
            }
            return None;
        }
        self.get_awaited_type(t)
    }
    pub(crate) fn check_async_function_return_type(
        &mut self,
        node: &Arc<Node>,
        return_type_node: &Arc<Node>,
    ) {
        let return_type = self.get_type_from_type_node(return_type_node);
        if crate::checker::utilities::is_type_error(&return_type) {
            return;
        }
        let Some(promise_sym) = self.globals.get("Promise").cloned() else {
            return;
        };
        let is_global_promise_ref = return_type
            .symbol
            .as_ref()
            .is_some_and(|s| Arc::ptr_eq(s, &promise_sym));
        if !is_global_promise_ref {
            let awaited = self
                .check_awaited_type_no_alias(
                    &return_type,
                    None,
                    tsox_core::diagnostics::messages_generated::
                        THE_RETURN_TYPE_OF_AN_ASYNC_FUNCTION_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER,
                )
                .unwrap_or_else(|| self.void_type());
            let arg = self.type_to_string(&awaited);
            let file = self.get_source_file_of_node(return_type_node);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                return_type_node.loc,
                tsox_core::diagnostics::messages_generated::
                    THE_RETURN_TYPE_OF_AN_ASYNC_FUNCTION_OR_METHOD_MUST_BE_THE_GLOBAL_PROMISE_T_TYPE_DID_YOU_MEAN_TO_WRITE_PROMISE_0,
                vec![arg],
            ));
            return;
        }
        self.check_awaited_type_no_alias(
            &return_type,
            Some(node),
            tsox_core::diagnostics::messages_generated::
                THE_RETURN_TYPE_OF_AN_ASYNC_FUNCTION_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER,
        );
    }
}
