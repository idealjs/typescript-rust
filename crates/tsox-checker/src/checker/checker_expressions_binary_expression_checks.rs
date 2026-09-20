#![allow(unused_imports)]

use crate::checker::checker_expressions::*;
use crate::checker::utilities_is_optional_symbol::is_literal_expression_of_object;

impl Checker {
    pub fn check_binary_expression(&mut self, node: &Arc<Node>) {
        if let tsox_frontend::ast::NodeData::BinaryExpression(data) = &node.data {
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
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        data.left.loc,
                        tsox_core::diagnostics::messages_generated::
                            LEFT_SIDE_OF_COMMA_OPERATOR_IS_UNUSED_AND_HAS_NO_SIDE_EFFECTS,
                        Vec::new(),
                    ));
            }
            self.check_expression(&data.left);

            if matches!(
                data.operator_token.kind,
                tsox_frontend::ast::SyntaxKind::AmpersandAmpersandToken
                    | tsox_frontend::ast::SyntaxKind::BarBarToken
                    | tsox_frontend::ast::SyntaxKind::QuestionQuestionToken
            ) {
                self.check_truthiness_of_type(&data.left);
                let mut parent = node.parent();
                while parent.as_ref().is_some_and(|p| {
                    matches!(&p.data, tsox_frontend::ast::NodeData::ParenthesizedExpression(_))
                        || matches!(&p.data, tsox_frontend::ast::NodeData::BinaryExpression(pb)
                            if matches!(
                                pb.operator_token.kind,
                                tsox_frontend::ast::SyntaxKind::AmpersandAmpersandToken
                                    | tsox_frontend::ast::SyntaxKind::BarBarToken
                                    | tsox_frontend::ast::SyntaxKind::QuestionQuestionToken
                            ))
                }) {
                    parent = parent.unwrap().parent();
                }
                let parent_is_if = parent
                    .as_ref()
                    .is_some_and(|p| p.kind == tsox_frontend::ast::SyntaxKind::IfStatement);
                if data.operator_token.kind == tsox_frontend::ast::SyntaxKind::AmpersandAmpersandToken
                    || parent_is_if
                {
                    let body = parent.and_then(|p| match &p.data {
                        tsox_frontend::ast::NodeData::IfStatement(d) => Some(Arc::clone(&d.then_statement)),
                        _ => None,
                    });
                    let left_type = self.get_type_of_node(&data.left);
                    self.check_testing_known_truthy_callable_or_awaitable(
                        &data.left,
                        &left_type,
                        body.as_ref(),
                    );
                }
            }

            let rhs_frame = {
                let mut lhs: &Arc<Node> = &data.left;
                loop {
                    match &lhs.data {
                        tsox_frontend::ast::NodeData::ParenthesizedExpression(p) => {
                            lhs = &p.expression;
                        }
                        tsox_frontend::ast::NodeData::NonNullExpression(n) => {
                            lhs = &n.expression;
                        }
                        _ => break,
                    }
                }
                if matches!(
                    data.operator_token.kind,
                    tsox_frontend::ast::SyntaxKind::QuestionQuestionEqualsToken
                        | tsox_frontend::ast::SyntaxKind::BarBarEqualsToken
                        | tsox_frontend::ast::SyntaxKind::AmpersandAmpersandEqualsToken
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
            self.check_binary_relational_operator_error(node, data);
            use tsox_frontend::ast::SyntaxKind::*;

            let mut readonly_index_reported = false;
            if data.operator_token.kind == EqualsToken
                && data.left.kind == SyntaxKind::PropertyAccessExpression
            {
                if let tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) = &data.left.data
                {
                    let obj_type = self.get_type_of_node(&pa.expression);
                    let name_text = pa.name.text();
                    if self.is_readonly_property_write(&data.left, &obj_type, name_text) {
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            pa.name.loc,
                            CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                            vec![name_text.to_string()],
                        ));
                    } else if self.is_readonly_index_write(&obj_type, name_text) {
                        // Go errorIfWritingToReadonlyIndex（点访问落 string 索引）：
                        // 只报 2542，写类型检查照 Go checkReferenceExpression 阻断
                        let type_name = self.type_to_string(&obj_type);
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            data.left.loc,
                            tsox_core::diagnostics::messages_generated::
                                INDEX_SIGNATURE_IN_TYPE_0_ONLY_PERMITS_READING,
                            vec![type_name],
                        ));
                        readonly_index_reported = true;
                    }
                }
            }
            if Self::is_assignment_operator(data.operator_token.kind)
                && data.left.kind == SyntaxKind::ElementAccessExpression
            {
                if let tsox_frontend::ast::NodeData::ElementAccessExpression(ea) = &data.left.data
                {
                    let obj_type = self.get_type_of_node(&ea.expression);
                    let (arg_name, key_flags) = match &ea.argument_expression.data {
                        tsox_frontend::ast::NodeData::StringLiteral(sl) => {
                            (Some(sl.text.clone()), TypeFlags::String)
                        }
                        tsox_frontend::ast::NodeData::Identifier(id) => {
                            (Some(id.text.clone()), TypeFlags::String)
                        }
                        tsox_frontend::ast::NodeData::NumericLiteral(nl) => {
                            (Some(nl.text.clone()), TypeFlags::Number)
                        }
                        _ => (None, TypeFlags::String),
                    };
                    if let Some(arg_name) = arg_name
                        && self.is_readonly_index_write_kind(&obj_type, &arg_name, key_flags)
                    {
                        let type_name = self.type_to_string(&obj_type);
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            data.left.loc,
                            tsox_core::diagnostics::messages_generated::
                                INDEX_SIGNATURE_IN_TYPE_0_ONLY_PERMITS_READING,
                            vec![type_name],
                        ));
                        readonly_index_reported = true;
                    }
                }
            }
            let mut assigned_target_blocks_type_check = false;

            if Self::is_assignment_operator(data.operator_token.kind)
                && data.left.kind == SyntaxKind::PropertyAccessExpression
                && let tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) = &data.left.data
                && pa.expression.kind == SyntaxKind::Identifier
                && let Some(enum_sym) = self.resolve_identifier(&pa.expression)
                && self
                    .resolve_alias_base(enum_sym)
                    .flags
                    .intersects(SymbolFlags::ENUM)
            {
                let name_text = pa.name.text();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    pa.name.loc,
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                    vec![name_text.to_string()],
                ));

                assigned_target_blocks_type_check = true;
            }

            if Self::is_assignment_operator(data.operator_token.kind) && {
                let mut target: &Arc<Node> = &data.left;
                while target.kind == SyntaxKind::ParenthesizedExpression {
                    target = match &target.data {
                        tsox_frontend::ast::NodeData::ParenthesizedExpression(p) => {
                            &p.expression
                        }
                        _ => break,
                    };
                }
                target.kind == SyntaxKind::Identifier
            } {
                let mut ident_node: &Arc<Node> = &data.left;
                while ident_node.kind == SyntaxKind::ParenthesizedExpression {
                    ident_node = match &ident_node.data {
                        tsox_frontend::ast::NodeData::ParenthesizedExpression(p) => {
                            &p.expression
                        }
                        _ => break,
                    };
                }
                let name_text = ident_node.text().to_string();
                if let Some(sym) = self.resolve_identifier(ident_node)
                    && let base = self.resolve_alias_base(sym)
                {
                    let msg = if base.flags.contains(SymbolFlags::Class) {
                        Some(tsox_core::diagnostics::messages_generated::
                                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CLASS)
                    } else if base.flags.intersects(SymbolFlags::ENUM) {
                        Some(tsox_core::diagnostics::messages_generated::
                                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_AN_ENUM)
                    } else if base.flags.intersects(SymbolFlags::MODULE) {
                        Some(tsox_core::diagnostics::messages_generated::
                                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_NAMESPACE)
                    } else if base.flags.contains(SymbolFlags::Function) {
                        Some(tsox_core::diagnostics::messages_generated::
                                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_FUNCTION)
                    } else {
                        None
                    };
                    if let Some(msg) = msg {
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            ident_node.loc,
                            msg,
                            vec![name_text],
                        ));

                        assigned_target_blocks_type_check = true;
                    }
                }
            }

            if data.operator_token.kind == EqualsToken
                && matches!(
                    data.left.kind,
                    SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
                )
            {
                let rhs_type = self.get_type_of_node(&data.right);
                self.check_destructuring_assignment(&data.left, &rhs_type);
                assigned_target_blocks_type_check = true;
            }

            if Self::is_assignment_operator(data.operator_token.kind)
                && data.left.kind == SyntaxKind::Identifier
            {
                if let Some(symbol) = self.resolve_identifier(&data.left) {
                    if self.symbol_is_const_variable(&symbol) {
                        let name_text = data.left.text();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
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
                        SyntaxKind::ArrayLiteralExpression
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
                && !readonly_index_reported
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
                let in_js = self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.file_name.ends_with(".js") || f.file_name.ends_with(".jsx"));
                if !in_js
                    || matches!(
                        data.operator_token.kind,
                        EqualsEqualsEqualsToken | ExclamationEqualsEqualsToken
                    )
                {
                    let object_literal_operand = is_literal_expression_of_object(&data.left)
                        || is_literal_expression_of_object(&data.right);
                    if object_literal_operand {
                        let result = if matches!(
                            data.operator_token.kind,
                            EqualsEqualsToken | EqualsEqualsEqualsToken
                        ) {
                            "false"
                        } else {
                            "true"
                        };
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            node.loc,
                            THIS_CONDITION_WILL_ALWAYS_RETURN_0_SINCE_JAVASCRIPT_COMPARES_OBJECTS_BY_REFERENCE_NOT_VALUE,
                            vec![result.to_string()],
                        ));
                    }
                }

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
                    // Go：同名不同型时用全限定名消歧
                    let (left_str, right_str) =
                        self.get_type_names_for_error_display(&left_type, &right_type);
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
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
