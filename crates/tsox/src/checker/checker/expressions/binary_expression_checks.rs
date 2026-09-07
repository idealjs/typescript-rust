#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_binary_expression(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
            self.check_binary_arith_pre(node, data);

            if data.operator_token.kind == SyntaxKind::CommaToken
                && !self.is_indirect_call_comma(node)
                && !self.expression_has_side_effects(&data.left)
                && !self.diagnostics.get_all().iter().any(|d| {
                    d.code == 2695
                        && d.file
                            .as_ref()
                            .map(|f| Arc::ptr_eq(f, self.current_file.as_ref().unwrap_or(&f)))
                            .unwrap_or(false)
                        && d.loc == data.left.loc
                })
            {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        data.left.loc,
                        crate::diagnostics::messages_generated::
                            LEFT_SIDE_OF_COMMA_OPERATOR_IS_UNUSED_AND_HAS_NO_SIDE_EFFECTS,
                        Vec::new(),
                    ));
            }
            self.check_expression(&data.left);

            if matches!(
                data.operator_token.kind,
                crate::ast::SyntaxKind::AmpersandAmpersandToken
                    | crate::ast::SyntaxKind::BarBarToken
            ) {
                self.check_truthiness_of_type(&data.left);
            }

            let rhs_frame = {
                let mut lhs: &Arc<Node> = &data.left;
                loop {
                    match &lhs.data {
                        crate::ast::NodeData::ParenthesizedExpression(p) => {
                            lhs = &p.expression;
                        }
                        crate::ast::NodeData::NonNullExpression(n) => {
                            lhs = &n.expression;
                        }
                        _ => break,
                    }
                }
                if matches!(
                    data.operator_token.kind,
                    crate::ast::SyntaxKind::QuestionQuestionEqualsToken
                        | crate::ast::SyntaxKind::BarBarEqualsToken
                        | crate::ast::SyntaxKind::AmpersandAmpersandEqualsToken
                ) {
                    self.logical_rhs_frame(data.operator_token.kind, lhs)
                } else {
                    None
                }
            };
            match rhs_frame {
                Some((sym, t)) => {
                    self.logical_rhs_narrowing_frames.push((sym, t));
                    self.check_expression(&data.right);
                    self.logical_rhs_narrowing_frames.pop();
                }
                None => self.check_expression(&data.right),
            }
            self.check_binary_plus_operator_error(node, data);
            use crate::ast::SyntaxKind::*;

            if data.operator_token.kind == EqualsToken
                && data.left.kind == SyntaxKind::PropertyAccessExpression
            {
                if let crate::ast::NodeData::PropertyAccessExpression(pa) = &data.left.data {
                    let obj_type = self.get_type_of_node(&pa.expression);
                    let name_text = pa.name.text();
                    if self.is_property_readonly(&obj_type, name_text) {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            pa.name.loc,
                            CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                            vec![name_text.to_string()],
                        ));
                    }
                }
            }
            let mut assigned_target_blocks_type_check = false;

            if Self::is_assignment_operator(data.operator_token.kind)
                && data.left.kind == SyntaxKind::PropertyAccessExpression
                && let crate::ast::NodeData::PropertyAccessExpression(pa) = &data.left.data
                && pa.expression.kind == SyntaxKind::Identifier
                && let Some(enum_sym) = self.resolve_identifier(&pa.expression)
                && self
                    .resolve_alias_base(enum_sym)
                    .flags
                    .intersects(SymbolFlags::ENUM)
            {
                let name_text = pa.name.text();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    pa.name.loc,
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                    vec![name_text.to_string()],
                ));

                assigned_target_blocks_type_check = true;
            }

            if Self::is_assignment_operator(data.operator_token.kind)
                && data.left.kind == SyntaxKind::Identifier
            {
                let name_text = data.left.text().to_string();
                if let Some(sym) = self.resolve_identifier(&data.left)
                    && let base = self.resolve_alias_base(sym)
                {
                    let msg = if base.flags.contains(SymbolFlags::Class) {
                        Some(crate::diagnostics::messages_generated::
                                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CLASS)
                    } else if base.flags.intersects(SymbolFlags::ENUM) {
                        Some(crate::diagnostics::messages_generated::
                                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_AN_ENUM)
                    } else if base.flags.contains(SymbolFlags::Function) {
                        Some(crate::diagnostics::messages_generated::
                                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_FUNCTION)
                    } else {
                        None
                    };
                    if let Some(msg) = msg {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.left.loc,
                            msg,
                            vec![name_text],
                        ));

                        assigned_target_blocks_type_check = true;
                    }
                }
            }

            if Self::is_assignment_operator(data.operator_token.kind)
                && data.left.kind == SyntaxKind::Identifier
            {
                if let Some(symbol) = self.resolve_identifier(&data.left) {
                    if self.symbol_is_const_variable(&symbol) {
                        let name_text = data.left.text();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.left.loc,
                            CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CONSTANT,
                            vec![name_text.to_string()],
                        ));
                    }
                }
            }

            if data.operator_token.kind == EqualsToken && data.left.kind == SyntaxKind::Identifier {
                if let Some(target) = self.declared_annotation_type_of(&data.left) {
                    if matches!(
                        data.right.kind,
                        SyntaxKind::ObjectLiteralExpression
                            | SyntaxKind::ArrayLiteralExpression
                            | SyntaxKind::TypeAssertionExpression
                            | SyntaxKind::AsExpression
                    ) {
                        self.check_contextual_elements(&data.right, &target, data.right.loc);
                    }
                }
            }

            if Self::is_assignment_operator(data.operator_token.kind)
                && matches!(
                    data.left.kind,
                    SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                )
            {
                self.check_const_property_assignment(&data.left);
            }

            if Self::is_assignment_operator(data.operator_token.kind)
                && !assigned_target_blocks_type_check
            {
                self.check_assignment_compat(node, data);
            }

            let is_equality_op = matches!(
                data.operator_token.kind,
                EqualsEqualsToken
                    | ExclamationEqualsToken
                    | EqualsEqualsEqualsToken
                    | ExclamationEqualsEqualsToken
            );
            if is_equality_op {
                let left_type = self.get_type_of_node(&data.left);
                let right_type = self.get_type_of_node(&data.right);

                let skip_flags = TypeFlags::Any
                    .union(TypeFlags::Unknown)
                    .union(TypeFlags::Never)
                    .union(TypeFlags::Null)
                    .union(TypeFlags::Undefined);
                if !left_type.flags.intersects(skip_flags)
                    && !right_type.flags.intersects(skip_flags)
                    && !self.are_types_comparable(&left_type, &right_type)
                {
                    let left_str = self.type_to_string(&left_type);
                    let right_str = self.type_to_string(&right_type);
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            node.loc,
                            THIS_COMPARISON_APPEARS_TO_BE_UNINTENTIONAL_BECAUSE_THE_TYPES_0_AND_1_HAVE_NO_OVERLAP,
                            vec![left_str, right_str],
                        ));
                }
            }
        }
    }
}
