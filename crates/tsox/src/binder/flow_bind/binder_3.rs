#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn bind_switch_statement(&mut self, node: &Arc<Node>) {
        let mut post_switch_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expression, case_block) = match &node.data {
            NodeData::SwitchStatement(data) => (data.expression.clone(), data.case_block.clone()),
            _ => return,
        };

        self.bind(&expression);

        let prev_break = self.current_break_target.take();
        let break_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));

        let clauses = match &case_block.data {
            NodeData::CaseBlock(data) => data.clauses.clone(),
            _ => {
                self.current_break_target = prev_break;
                return;
            }
        };

        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(&case_block));
        self.symbol_map
            .locals
            .entry(case_block.id())
            .or_insert_with(SymbolTable::new);

        let entry_flow = self.current_flow.clone();
        let is_narrowing_switch =
            expression.kind == SyntaxKind::TrueKeyword || self.is_narrowing_expression(&expression);
        let mut fallthrough_flow: Option<Arc<FlowNode>> = None;
        let mut has_default = false;
        let clause_nodes = &clauses.nodes;
        let mut i = 0;
        while i < clause_nodes.len() {
            let clause_start = i;

            while clause_statements_empty(&clause_nodes[i]) && i + 1 < clause_nodes.len() {
                self.bind_case_clause(&clause_nodes[i], &entry_flow);
                i += 1;
            }
            let mut pre_case_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
            let pre_case_flow = if is_narrowing_switch {
                entry_flow.as_ref().map(|entry| {
                    self.create_flow_switch_clause(
                        entry,
                        Some(&clause_nodes[i]),
                        node,
                        clause_start,
                        i + 1,
                    )
                })
            } else {
                entry_flow.clone()
            };
            if let Some(f) = &pre_case_flow {
                pre_case_label.add_antecedent(Arc::clone(f));
            }
            if let Some(f) = &fallthrough_flow {
                pre_case_label.add_antecedent(Arc::clone(f));
            }
            self.current_flow =
                Some(pre_case_label.finish(self.unreachable_flow.as_ref().unwrap()));
            let clause = &clause_nodes[i];
            if clause.kind == SyntaxKind::DefaultClause {
                has_default = true;
            }
            self.bind_case_clause(clause, &entry_flow);
            fallthrough_flow = self.current_flow.clone();
            i += 1;
        }

        if let Some(current) = &self.current_flow {
            post_switch_label.add_antecedent(Arc::clone(current));
        }

        for ant in &break_acc.antecedents {
            post_switch_label.add_antecedent(Arc::clone(ant));
        }

        if !has_default {
            if let Some(entry) = &entry_flow {
                let bypass = self.create_flow_switch_clause(entry, None, node, 0, 0);
                post_switch_label.add_antecedent(bypass);
            }
        }

        self.current_flow = Some(post_switch_label.finish(self.unreachable_flow.as_ref().unwrap()));

        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;

        self.current_break_target = prev_break;
    }

    pub(crate) fn bind_case_clause(
        &mut self,
        clause: &Arc<Node>,
        entry_flow: &Option<Arc<FlowNode>>,
    ) {
        let NodeData::CaseOrDefaultClause(data) = &clause.data else {
            return;
        };
        if clause.kind == SyntaxKind::CaseClause {
            let saved = self.current_flow.take();
            self.current_flow = entry_flow.clone();
            self.bind(&data.expression);
            self.current_flow = saved;
        }
        for stmt in &data.statements.nodes {
            self.bind(stmt);
        }
    }

    pub(crate) fn is_narrowing_expression(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::Identifier | SyntaxKind::ThisKeyword => true,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                self.contains_narrowable_reference(expr)
            }
            SyntaxKind::CallExpression => self.has_narrowable_argument(expr),
            SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression
            | SyntaxKind::TypeOfExpression => expr
                .expression()
                .map(|inner| self.is_narrowing_expression(inner))
                .unwrap_or(false),
            SyntaxKind::BinaryExpression => {
                let NodeData::BinaryExpression(bin) = &expr.data else {
                    return false;
                };
                self.is_narrowing_binary_expression(&bin.left, &bin.operator_token, &bin.right)
            }
            SyntaxKind::PrefixUnaryExpression => {
                let NodeData::PrefixUnaryExpression(un) = &expr.data else {
                    return false;
                };
                un.operator == SyntaxKind::ExclamationToken
                    && self.is_narrowing_expression(&un.operand)
            }
            _ => false,
        }
    }

    pub(crate) fn is_narrowing_binary_expression(
        &self,
        left: &Arc<Node>,
        operator: &Arc<Node>,
        right: &Arc<Node>,
    ) -> bool {
        match operator.kind {
            SyntaxKind::EqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken => self.contains_narrowable_reference(left),
            SyntaxKind::EqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
            | SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken => {
                self.is_narrowable_operand(left)
                    || self.is_narrowable_operand(right)
                    || self.is_narrowing_typeof_operands(right, left)
                    || self.is_narrowing_typeof_operands(left, right)
                    || (Self::is_boolean_literal(right) && self.is_narrowing_expression(left))
                    || (Self::is_boolean_literal(left) && self.is_narrowing_expression(right))
            }
            SyntaxKind::InstanceOfKeyword => self.is_narrowable_operand(left),
            SyntaxKind::InKeyword => self.is_narrowing_expression(right),
            SyntaxKind::CommaToken => self.is_narrowing_expression(right),
            _ => false,
        }
    }

    pub(crate) fn is_boolean_literal(node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword
        )
    }

    pub(crate) fn is_narrowable_operand(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::ParenthesizedExpression => expr
                .expression()
                .map(|e| self.is_narrowable_operand(e))
                .unwrap_or(false),
            SyntaxKind::BinaryExpression => {
                let NodeData::BinaryExpression(bin) = &expr.data else {
                    return false;
                };
                match bin.operator_token.kind {
                    SyntaxKind::EqualsToken => self.is_narrowable_operand(&bin.left),
                    SyntaxKind::CommaToken => self.is_narrowable_operand(&bin.right),
                    _ => self.contains_narrowable_reference(expr),
                }
            }
            _ => self.contains_narrowable_reference(expr),
        }
    }

    pub(crate) fn is_narrowing_typeof_operands(
        &self,
        expr1: &Arc<Node>,
        expr2: &Arc<Node>,
    ) -> bool {
        expr1.kind == SyntaxKind::TypeOfExpression
            && expr1
                .expression()
                .map(|e| self.is_narrowable_operand(e))
                .unwrap_or(false)
            && matches!(
                expr2.kind,
                SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
            )
    }

    pub(crate) fn contains_narrowable_reference(&self, expr: &Arc<Node>) -> bool {
        if self.is_narrowable_reference(expr) {
            return true;
        }
        if expr.flags.contains(NodeFlags::OptionalChain) {
            if let Some(inner) = expr.expression() {
                if matches!(
                    expr.kind,
                    SyntaxKind::PropertyAccessExpression
                        | SyntaxKind::ElementAccessExpression
                        | SyntaxKind::CallExpression
                        | SyntaxKind::NonNullExpression
                ) {
                    return self.contains_narrowable_reference(inner);
                }
            }
        }
        false
    }

    pub(crate) fn is_narrowable_reference(&self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty => true,
            SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression => node
                .expression()
                .map(|e| self.is_narrowable_reference(e))
                .unwrap_or(false),
            SyntaxKind::ElementAccessExpression => {
                let NodeData::ElementAccessExpression(el) = &node.data else {
                    return false;
                };
                self.is_string_or_numeric_literal_like(&el.argument_expression)
                    || (self.is_entity_name_expression(&el.argument_expression)
                        && self.is_narrowable_reference(&el.expression))
            }
            SyntaxKind::BinaryExpression => {
                let NodeData::BinaryExpression(bin) = &node.data else {
                    return false;
                };
                (bin.operator_token.kind == SyntaxKind::CommaToken
                    && self.is_narrowable_reference(&bin.right))
                    || (is_assignment_operator(bin.operator_token.kind)
                        && crate::ast::utilities::is_left_hand_side_expression(&bin.left))
            }
            _ => false,
        }
    }

    pub(crate) fn has_narrowable_argument(&self, expr: &Arc<Node>) -> bool {
        let NodeData::CallExpression(call) = &expr.data else {
            return false;
        };
        call.arguments
            .nodes
            .iter()
            .any(|arg| self.contains_narrowable_reference(arg))
    }

    pub(crate) fn bind_return_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ReturnStatement(data) = &node.data {
            if let Some(expr) = &data.expression {
                self.bind(expr);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
        self.has_explicit_return = true;
        self.has_flow_effects = true;
    }
}
