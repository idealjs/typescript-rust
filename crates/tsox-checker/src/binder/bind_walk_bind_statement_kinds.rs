#![allow(unused_imports)]

use crate::binder::bind_walk::*;

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
                // Go checkStrictModeLabeledStatement：标签修饰声明/变量语句报 TS1344
                //（tsgo oracle 无条件分发，sloppy 同报）
                if let NodeData::LabeledStatement(d) = &node.data
                    && (is_declaration_statement(&d.statement)
                        || is_variable_statement(&d.statement))
                {
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        d.label.loc,
                        tsox_core::diagnostics::messages_generated::A_LABEL_IS_NOT_ALLOWED_HERE,
                        vec![],
                    ));
                }
                self.bind_labeled_statement(node);
                return true;
            }
            SyntaxKind::CallExpression => {
                if matches!(
                    crate::binder::get_assignment_declaration_kind(node),
                    crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ObjectDefinePropertyExports
                ) {
                    self.bind_exports_or_object_define_property(node);
                }
                self.bind_call_expression_flow(node);
            }
            SyntaxKind::BinaryExpression => {
                // Go checkStrictModeBinaryExpression：赋值左侧 eval/arguments
                // 无条件报 TS1100/1101（tsgo 行为，sloppy 同报）
                if let NodeData::BinaryExpression(bin) = &node.data
                    && is_assignment_operator(bin.operator_token.kind)
                    && bin.left.kind == SyntaxKind::Identifier
                    && matches!(bin.left.text(), "eval" | "arguments")
                    && !crate::binder::bind_walk::Binder::is_in_for_in_or_of_head(&bin.left)
                    && !self.strict_eval_diag_exists(
                        bin.left.loc,
                        bin.left.text().to_string(),
                    )
                {
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        bin.left.loc,
                        tsox_core::diagnostics::messages_generated::
                            INVALID_USE_OF_0_IN_STRICT_MODE,
                        vec![bin.left.text().to_string()],
                    ));
                }
                match crate::binder::get_assignment_declaration_kind(node) {
                    crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ModuleExports => {
                        self.bind_module_exports_assignment(node);
                    }
                    crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ExportsProperty => {
                        self.bind_exports_or_object_define_property(node);
                    }
                    _ => {}
                }

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
                        .parent()
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
            SyntaxKind::ConditionalExpression => {
                // Go bindConditionalExpression：条件真/假分支各自挂条件流，
                // 分支内引用经真/假守卫收窄（x ? x : [] 的真分支 x 去 null）
                let NodeData::ConditionalExpression(ce) = &node.data else {
                    return false;
                };
                let condition = Arc::clone(&ce.condition);
                let when_true = Arc::clone(&ce.when_true);
                let when_false = Arc::clone(&ce.when_false);
                self.bind(&condition);
                if let Some(current) = self.current_flow.take() {
                    let true_flow =
                        self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &condition);
                    let false_flow = self.create_flow_condition(
                        FlowFlags::FALSE_CONDITION,
                        &current,
                        &condition,
                    );
                    self.current_flow = Some(true_flow);
                    self.bind(&when_true);
                    let after_true = self.current_flow.take();
                    self.current_flow = Some(false_flow);
                    self.bind(&when_false);
                    let after_false = self.current_flow.take();
                    let mut label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
                    if let Some(at) = after_true {
                        label.add_antecedent(at);
                    }
                    if let Some(af) = after_false {
                        label.add_antecedent(af);
                    }
                    self.current_flow =
                        Some(label.finish(self.unreachable_flow.as_ref().unwrap()));
                } else {
                    self.bind(&when_true);
                    self.bind(&when_false);
                }
                return true;
            }
            SyntaxKind::PostfixUnaryExpression => {
                if let NodeData::PostfixUnaryExpression(post) = &node.data
                    && !self.strict_eval_diag_exists(
                        post.operand.loc,
                        post.operand.text().to_string(),
                    )
                    && matches!(
                        post.operator,
                        SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                    )
                    && post.operand.kind == SyntaxKind::Identifier
                    && matches!(post.operand.text(), "eval" | "arguments")
                {
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        post.operand.loc,
                        tsox_core::diagnostics::messages_generated::
                            INVALID_USE_OF_0_IN_STRICT_MODE,
                        vec![post.operand.text().to_string()],
                    ));
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                if let NodeData::PrefixUnaryExpression(pre) = &node.data
                    && !self.strict_eval_diag_exists(
                        pre.operand.loc,
                        pre.operand.text().to_string(),
                    )
                    && matches!(pre.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken)
                    && pre.operand.kind == SyntaxKind::Identifier
                    && matches!(pre.operand.text(), "eval" | "arguments")
                {
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        pre.operand.loc,
                        tsox_core::diagnostics::messages_generated::
                            INVALID_USE_OF_0_IN_STRICT_MODE,
                        vec![pre.operand.text().to_string()],
                    ));
                }
            }
            SyntaxKind::WithStatement => {
                // Go checkStrictModeWithStatement：with 无条件报 TS1101
                //（tsgo oracle 实证 sloppy 同报）；span 取首 token，
                // 与 checker 的 TS2410（宽 span）保持参考基线排序
                let loc = tsox_core::core::text::TextRange::new(node.loc.pos(), node.loc.pos() + 4);
                self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                    self.current_source_file.clone(),
                    loc,
                    tsox_core::diagnostics::messages_generated::
                        X_WITH_STATEMENTS_ARE_NOT_ALLOWED_IN_STRICT_MODE,
                    vec![],
                ));
            }
            _ => {}
        }
        false
    }
}

impl Binder {
    fn strict_eval_diag_exists(&self, loc: tsox_core::core::text::TextRange, text: String) -> bool {
        self.symbol_map
            .binder_diagnostics
            .iter()
            .any(|d| d.code == 1100 && d.loc == loc && d.message_args.first() == Some(&text))
    }
}
