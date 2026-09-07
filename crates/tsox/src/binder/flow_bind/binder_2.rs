#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn bind_do_statement(&mut self, node: &Arc<Node>) {
        let mut pre_do_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_condition_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_do_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, stmt) = match &node.data {
            NodeData::DoStatement(data) => (data.expression.clone(), data.statement.clone()),
            _ => return,
        };

        if let Some(current) = &self.current_flow {
            pre_do_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_do_label.finish(self.unreachable_flow.as_ref().unwrap()));

        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));

        self.set_continue_target(node, &continue_acc);

        self.bind(&stmt);
        if let Some(current) = &self.current_flow {
            pre_condition_label.add_antecedent(Arc::clone(current));
        }

        for ant in &continue_acc.antecedents {
            pre_condition_label.add_antecedent(Arc::clone(ant));
        }

        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow =
            Some(pre_condition_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            pre_do_label.add_antecedent(true_flow);
            post_do_label.add_antecedent(false_flow);
        }

        for ant in &break_acc.antecedents {
            post_do_label.add_antecedent(Arc::clone(ant));
        }

        self.current_flow = Some(post_do_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    pub(crate) fn bind_for_statement(&mut self, node: &Arc<Node>) {
        let mut pre_loop_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_body_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut pre_incr_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_loop_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (initializer, condition, incrementor, statement) = match &node.data {
            NodeData::ForStatement(data) => (
                data.initializer.clone(),
                data.condition.clone(),
                data.incrementor.clone(),
                data.statement.clone(),
            ),
            _ => return,
        };

        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(node));
        self.symbol_map
            .locals
            .entry(node.id())
            .or_insert_with(SymbolTable::new);

        if let Some(init) = initializer {
            self.bind(&init);
        }

        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        if let Some(cond) = condition {
            self.bind(&cond);
            if let Some(current) = self.current_flow.take() {
                let true_flow =
                    self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &cond);
                let false_flow =
                    self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &cond);
                pre_body_label.add_antecedent(true_flow);
                post_loop_label.add_antecedent(false_flow);
            }
        } else {
            if let Some(current) = &self.current_flow {
                pre_body_label.add_antecedent(Arc::clone(current));
            }
        }

        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));

        self.set_continue_target(node, &continue_acc);

        self.current_flow = Some(pre_body_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&statement);
        if let Some(current) = &self.current_flow {
            pre_incr_label.add_antecedent(Arc::clone(current));
        }

        for ant in &continue_acc.antecedents {
            pre_incr_label.add_antecedent(Arc::clone(ant));
        }

        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(pre_incr_label.finish(self.unreachable_flow.as_ref().unwrap()));
        if let Some(inc) = incrementor {
            self.bind(&inc);
        }
        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }

        for ant in &break_acc.antecedents {
            post_loop_label.add_antecedent(Arc::clone(ant));
        }

        self.current_flow = Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;
    }

    pub(crate) fn bind_for_in_or_of_statement(&mut self, node: &Arc<Node>) {
        let mut pre_loop_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut post_loop_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expression, initializer, statement) = match &node.data {
            NodeData::ForInOrOfStatement(data) => (
                data.expression.clone(),
                data.initializer.clone(),
                data.statement.clone(),
            ),
            _ => return,
        };

        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(node));
        self.symbol_map
            .locals
            .entry(node.id())
            .or_insert_with(SymbolTable::new);

        self.bind(&expression);

        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        post_loop_label.add_antecedent(Arc::clone(self.current_flow.as_ref().unwrap()));

        self.bind(&initializer);

        if initializer.kind != SyntaxKind::VariableDeclarationList {
            self.bind_assignment_target_flow(&initializer);
        }

        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));

        self.set_continue_target(node, &continue_acc);

        self.bind(&statement);
        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }

        for ant in &continue_acc.antecedents {
            pre_loop_label.add_antecedent(Arc::clone(ant));
        }

        for ant in &break_acc.antecedents {
            post_loop_label.add_antecedent(Arc::clone(ant));
        }

        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;
    }
}
