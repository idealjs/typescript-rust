#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::Node;
use tsox_frontend::ast::SyntaxKind;

use crate::checker::checker::*;
use tsox_core::diagnostics::messages_generated::TYPE_0_MAY_REPRESENT_A_PRIMITIVE_VALUE_WHICH_IS_NOT_PERMITTED_AS_THE_RIGHT_OPERAND_OF_THE_IN_OPERATOR;

impl Checker {
    pub(crate) fn check_in_expression(
        &mut self,
        data: &tsox_frontend::ast::node_data_generated::BinaryExpressionData,
    ) {
        let left_type = self.get_type_of_node(&data.left);
        let right_type = self.get_type_of_node(&data.right);
        let silent = self.silent_never_type();
        if Arc::ptr_eq(&left_type, &silent) || Arc::ptr_eq(&right_type, &silent) {
            return;
        }
        if data.left.kind != SyntaxKind::PrivateIdentifier {
            let target = self.get_union_type(vec![
                self.string_type(),
                self.number_type(),
                self.es_symbol_type(),
            ]);
            let checked = self.check_non_null_type(&left_type, &data.left);
            self.check_type_assignable_to_and_optionally_elaborate(
                &checked,
                &target,
                Some(&data.left),
                Some(&data.left),
                None,
                None,
            );
        }
        let checked_right = self.check_non_null_type(&right_type, &data.right);
        let assignable = self.check_type_assignable_to_and_optionally_elaborate(
            &checked_right,
            &self.non_primitive_type(),
            Some(&data.right),
            Some(&data.right),
            None,
            None,
        );
        if assignable && self.has_empty_object_intersection(&right_type) {
            let type_name = self.type_to_string(&right_type);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                data.right.loc,
                TYPE_0_MAY_REPRESENT_A_PRIMITIVE_VALUE_WHICH_IS_NOT_PERMITTED_AS_THE_RIGHT_OPERAND_OF_THE_IN_OPERATOR,
                vec![type_name],
            ));
        }
    }

    fn has_empty_object_intersection(&self, t: &Arc<Type>) -> bool {
        fn matches(t: &Arc<Type>, s: &Checker) -> bool {
            (t.symbol.is_none() && s.is_empty_anonymous_object_type(t))
                || (t.flags.contains(TypeFlags::Intersection)
                    && s.is_empty_anonymous_object_type(&s.get_base_constraint_or_type(t)))
        }
        if t.flags.contains(TypeFlags::Union) {
            t.types()
                .map_or(false, |ts| ts.iter().any(|c| matches(c, self)))
        } else {
            matches(t, self)
        }
    }
}
