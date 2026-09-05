use std::sync::Arc;

use crate::ast::{
    Node, Symbol, SymbolFlags, SyntaxKind,
};







use super::*;


impl Checker {
    fn op_display(kind: crate::ast::SyntaxKind) -> &'static str {
        use crate::ast::SyntaxKind::*;
        match kind {
            AsteriskToken => "*",
            AsteriskAsteriskToken => "**",
            AsteriskEqualsToken => "*=",
            AsteriskAsteriskEqualsToken => "**=",
            SlashToken => "/",
            SlashEqualsToken => "/=",
            PercentToken => "%",
            PercentEqualsToken => "%=",
            MinusToken => "-",
            MinusEqualsToken => "-=",
            PlusToken => "+",
            PlusEqualsToken => "+=",
            LessThanLessThanToken => "<<",
            LessThanLessThanEqualsToken => "<<=",
            GreaterThanGreaterThanToken => ">>",
            GreaterThanGreaterThanEqualsToken => ">>=",
            GreaterThanGreaterThanGreaterThanToken => ">>>",
            GreaterThanGreaterThanGreaterThanEqualsToken => ">>>=",
            BarToken => "|",
            BarEqualsToken => "|=",
            CaretToken => "^",
            CaretEqualsToken => "^=",
            AmpersandToken => "&",
            AmpersandEqualsToken => "&=",
            _ => "?",
        }
    }

    fn nonvariable_assignment_target_type(
        &mut self,
        operand: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        if operand.kind != SyntaxKind::Identifier {
            return None;
        }

        if !Self::is_definite_assignment_target(operand) {
            return None;
        }
        let sym = self.resolve_identifier(operand)?;
        let base = self.resolve_alias_base(sym);
        if base.flags.intersects(SymbolFlags::VARIABLE) {
            return None;
        }
        Some(self.error_type())
    }

    pub(crate) fn check_binary_arith_pre(
        &mut self,
        node: &Arc<Node>,
        data: &crate::ast::node_data_generated::BinaryExpressionData,
    ) {
        use crate::ast::SyntaxKind::*;
        let op = data.operator_token.kind;
        let arith_nonplus = matches!(
            op,
            AsteriskToken
                | AsteriskAsteriskToken
                | AsteriskEqualsToken
                | AsteriskAsteriskEqualsToken
                | SlashToken
                | SlashEqualsToken
                | PercentToken
                | PercentEqualsToken
                | MinusToken
                | MinusEqualsToken
                | LessThanLessThanToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | BarToken
                | BarEqualsToken
                | CaretToken
                | CaretEqualsToken
                | AmpersandToken
                | AmpersandEqualsToken
        );
        let plus = op == PlusToken || op == PlusEqualsToken;
        if !arith_nonplus && !plus {
            return;
        }
        for operand in [&data.left, &data.right] {
            if matches!(operand.kind, NullKeyword | UndefinedKeyword) {
                let word = if operand.kind == NullKeyword { "null" } else { "undefined" };
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    operand.loc,
                    crate::diagnostics::messages_generated::THE_VALUE_0_CANNOT_BE_USED_HERE,
                    vec![word.to_string()],
                ));
            }
        }
        if !arith_nonplus {
            return;
        }

        let lt = self
            .nonvariable_assignment_target_type(&data.left)
            .unwrap_or_else(|| self.get_type_of_node(&data.left));
        let rt = self.get_type_of_node(&data.right);
        let boolean_like =
            |t: &Arc<Type>| t.flags.intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral);
        if boolean_like(&lt) && boolean_like(&rt) {
            let suggested = match op {
                AmpersandToken | AmpersandEqualsToken => Some("&&"),
                BarToken | BarEqualsToken => Some("||"),
                CaretToken | CaretEqualsToken => Some("!=="),
                _ => None,
            };
            if let Some(sugg) = suggested {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::
                        THE_0_OPERATOR_IS_NOT_ALLOWED_FOR_BOOLEAN_TYPES_CONSIDER_USING_1_INSTEAD,
                    vec![Self::op_display(op).to_string(), sugg.to_string()],
                ));
            }
            return;
        }
        fn ok_number(c: &mut Checker, t: &Arc<Type>) -> bool {
            let n = c.number_type();
            if c.is_type_assignable_to(t, &n) {
                return true;
            }
            let b = c.bigint_type();
            c.is_type_assignable_to(t, &b)
        }

        let left_is_literal = matches!(data.left.kind, NullKeyword | UndefinedKeyword);
        let right_is_literal = matches!(data.right.kind, NullKeyword | UndefinedKeyword);
        if !left_is_literal && !ok_number(self, &lt) {
            self.arith_operand_error_nodes
                .insert(Arc::as_ptr(node) as *const crate::ast::Node);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                data.left.loc,
                crate::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ARITHMETIC_OPERATION_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE,
                Vec::new(),
            ));
        }
        if !right_is_literal && !ok_number(self, &rt) {
            self.arith_operand_error_nodes
                .insert(Arc::as_ptr(node) as *const crate::ast::Node);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                data.right.loc,
                crate::diagnostics::messages_generated::
                    THE_RIGHT_HAND_SIDE_OF_AN_ARITHMETIC_OPERATION_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE,
                Vec::new(),
            ));
        }
    }

    pub(crate) fn check_binary_plus_operator_error(
        &mut self,
        node: &Arc<Node>,
        data: &crate::ast::node_data_generated::BinaryExpressionData,
    ) {
        use crate::ast::SyntaxKind::*;
        let op = data.operator_token.kind;
        if op != PlusToken && op != PlusEqualsToken {
            return;
        }

        let lt = self
            .nonvariable_assignment_target_type(&data.left)
            .unwrap_or_else(|| self.get_type_of_node(&data.left));
        let rt = self.get_type_of_node(&data.right);
        let number_like = |t: &Arc<Type>| {

            (!self.strict_null_checks
                && t.flags.intersects(
                    TypeFlags::Undefined | TypeFlags::Null,
                ))
                || t.flags.contains(TypeFlags::Never)
                || t.flags.intersects(
                    TypeFlags::Number
                        | TypeFlags::NumberLiteral
                        | TypeFlags::EnumLiteral
                        | TypeFlags::Union,
                )
        };
        let bigint_like = |t: &Arc<Type>| {
            t.flags.intersects(
                TypeFlags::BigInt | TypeFlags::BigIntLiteral | TypeFlags::Union,
            )
        };
        let string_like =
            |t: &Arc<Type>| t.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral);
        let valid = (number_like(&lt) && number_like(&rt))
            || (bigint_like(&lt) && bigint_like(&rt))
            || string_like(&lt)
            || string_like(&rt)
            || lt.flags.contains(TypeFlags::Any)
            || rt.flags.contains(TypeFlags::Any);
        if !valid {
            let lt_str = self.type_to_string(&lt);
            let rt_str = self.type_to_string(&rt);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPES_1_AND_2,
                vec!["+".to_string(), lt_str, rt_str],
            ));
        }
    }

    pub(crate) fn logical_rhs_frame(
        &mut self,
        operator: crate::ast::SyntaxKind,
        target: &Arc<Node>,
    ) -> Option<(Arc<Symbol>, Arc<Type>)> {
        use crate::ast::SyntaxKind::*;
        if !matches!(target.data, crate::ast::NodeData::Identifier(_)) {
            return None;
        }
        let left_type = self.assignment_target_type(target)?;
        let frame = match operator {
            QuestionQuestionEqualsToken => {

                let parts: Vec<Arc<Type>> = self
                    .flow_constituents_public(&left_type)
                    .into_iter()
                    .filter(|c| c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(self.build_union_from_types(parts))
                }
            }
            BarBarEqualsToken => {

                let parts: Vec<Arc<Type>> = self
                    .flow_constituents_public(&left_type)
                    .into_iter()
                    .filter(|c| self.flow_constituent_definitely_falsy(c))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(self.build_union_from_types(parts))
                }
            }
            AmpersandAmpersandEqualsToken => {

                let parts: Vec<Arc<Type>> = self
                    .flow_constituents_public(&left_type)
                    .into_iter()
                    .filter(|c| !self.flow_constituent_definitely_falsy(c))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(self.build_union_from_types(parts))
                }
            }
            _ => None,
        }?;
        let sym = self.resolve_identifier(target)?;
        Some((sym, frame))
    }
}
