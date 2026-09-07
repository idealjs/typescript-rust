#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn bind_statement_kinds(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::IfStatement => {
                self.bind_if_statement(node);
                return true;
            }
            SyntaxKind::WhileStatement => {
                self.bind_while_statement(node);
                return true;
            }
            SyntaxKind::DoStatement => {
                self.bind_do_statement(node);
                return true;
            }
            SyntaxKind::ForStatement => {
                self.bind_for_statement(node);
                return true;
            }
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.bind_for_in_or_of_statement(node);
                return true;
            }
            SyntaxKind::SwitchStatement => {
                self.bind_switch_statement(node);
                return true;
            }
            SyntaxKind::ReturnStatement => {
                self.bind_return_statement(node);
                return true;
            }
            SyntaxKind::ThrowStatement => {
                self.bind_throw_statement(node);
                return true;
            }
            SyntaxKind::BreakStatement => {
                self.bind_break_statement(node);
                return true;
            }
            SyntaxKind::ContinueStatement => {
                self.bind_continue_statement(node);
                return true;
            }
            SyntaxKind::ExpressionStatement => {
                self.bind_expression_statement(node);
                return true;
            }
            SyntaxKind::VariableStatement => {
                self.bind_children(node);
                return true;
            }
            SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement => {
                self.bind_children(node);
                let has_initializer = match &node.data {
                    NodeData::VariableDeclaration(d) => d.initializer.is_some(),
                    NodeData::BindingElement(d) => d.initializer.is_some(),
                    _ => false,
                };
                if has_initializer || Self::is_in_for_in_or_of_head(node) {
                    self.bind_initialized_variable_flow(node);
                }
                return true;
            }
            SyntaxKind::TryStatement => {
                self.bind_try_statement(node);
                return true;
            }
            SyntaxKind::LabeledStatement => {
                self.bind_labeled_statement(node);
                return true;
            }
            SyntaxKind::CallExpression => {
                self.bind_call_expression_flow(node);
            }
            SyntaxKind::BinaryExpression => {
                self.bind_this_property_assignment(node);

                self.collect_expando_assignment(node);

                if matches!(&node.data, NodeData::BinaryExpression(bin)
                if bin.operator_token.kind == SyntaxKind::EqualsToken
                    && matches!(
                        bin.left.kind,
                        SyntaxKind::ObjectLiteralExpression
                            | SyntaxKind::ArrayLiteralExpression
                    ))
                {
                    if let NodeData::BinaryExpression(bin) = &node.data {
                        let left = Arc::clone(&bin.left);
                        self.bind_assignment_target_flow(&left);
                    }
                }

                if let NodeData::BinaryExpression(bin) = &node.data {
                    let op = bin.operator_token.kind;

                    let parent_is_expr_stmt = node
                        .parent
                        .as_ref()
                        .is_some_and(|p| p.kind == SyntaxKind::ExpressionStatement);
                    if is_assignment_operator(op)
                        && matches!(bin.left.kind, SyntaxKind::Identifier)
                        && !parent_is_expr_stmt
                    {
                        let left = Arc::clone(&bin.left);
                        let right = Arc::clone(&bin.right);
                        self.bind(&left);
                        self.bind(&right);
                        if let Some(current) = self.current_flow.take() {
                            self.current_flow = Some(self.create_flow_assignment(&current, node));
                        }
                        return true;
                    }
                    if matches!(
                        op,
                        SyntaxKind::AmpersandAmpersandToken | SyntaxKind::BarBarToken
                    ) {
                        let left = Arc::clone(&bin.left);
                        let right = Arc::clone(&bin.right);
                        self.bind(&left);
                        if let Some(current) = self.current_flow.take() {
                            let is_and = op == SyntaxKind::AmpersandAmpersandToken;

                            let rhs_flags = if is_and {
                                FlowFlags::TRUE_CONDITION
                            } else {
                                FlowFlags::FALSE_CONDITION
                            };
                            let keep_flags = if is_and {
                                FlowFlags::FALSE_CONDITION
                            } else {
                                FlowFlags::TRUE_CONDITION
                            };
                            let keep = self.create_flow_condition(keep_flags, &current, &left);
                            let cond = self.create_flow_condition(rhs_flags, &current, &left);
                            self.current_flow = Some(cond);
                            self.bind(&right);

                            let after_right = self.current_flow.take();
                            let mut label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
                            label.add_antecedent(keep);
                            if let Some(ar) = after_right {
                                label.add_antecedent(ar);
                            }
                            self.current_flow =
                                Some(label.finish(self.unreachable_flow.as_ref().unwrap()));
                        } else {
                            self.bind(&right);
                        }
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }
}
