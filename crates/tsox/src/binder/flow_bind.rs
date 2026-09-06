use super::*;

impl Binder {
    fn unreachable_flow(&self) -> Arc<FlowNode> {
        Arc::clone(self.unreachable_flow.as_ref().unwrap())
    }

    #[allow(dead_code)]
    fn new_flow_node(&self, flags: FlowFlags) -> FlowNode {
        FlowNode::new(flags)
    }

    pub(crate) fn create_flow_condition(
        &mut self,
        flags: FlowFlags,
        antecedent: &Arc<FlowNode>,
        expression: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags,
            node: Some(Arc::clone(expression)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }

    pub(crate) fn create_flow_assignment(
        &mut self,
        antecedent: &Arc<FlowNode>,
        node: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags: FlowFlags::ASSIGNMENT,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }

    pub(crate) fn create_flow_call(&mut self, antecedent: &Arc<FlowNode>, node: &Arc<Node>) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags: FlowFlags::CALL,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }

    fn create_flow_mutation(
        &mut self,
        antecedent: &Arc<FlowNode>,
        node: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.set_flow_node_referenced(antecedent);
        self.has_flow_effects = true;
        let result = Arc::new(FlowNode {
            flags: FlowFlags::ARRAY_MUTATION,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        });

        if let Some(target) = &self.current_exception_target {
            self.add_antecedent_to_flow(target, &result);
        }
        result
    }

    fn set_flow_node_referenced(&self, flow: &FlowNode) {

        let ptr = flow as *const FlowNode as *mut FlowNode;
        unsafe {
            if (*ptr).flags.contains(FlowFlags::REFERENCED) {
                (*ptr).flags = (*ptr).flags | FlowFlags::SHARED;
            } else {
                (*ptr).flags = (*ptr).flags | FlowFlags::REFERENCED;
            }
        }
    }

    fn create_reduce_label(
        &self,
        target: &Arc<FlowNode>,
        antecedents: &[Arc<FlowNode>],
        antecedent: &Arc<FlowNode>,
    ) -> Arc<FlowNode> {
        Arc::new(FlowNode {
            flags: FlowFlags::REDUCE_LABEL,
            node: None,
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: antecedents.to_vec(),
            switch_statement: None,
            clause_range: None,
            reduce_target: Some(Arc::clone(target)),
        })
    }

    fn new_flow_accumulator() -> Arc<FlowNode> {
        Arc::new(FlowNode {
            flags: FlowFlags::BRANCH_LABEL,
            node: None,
            antecedent: None,
            antecedents: Vec::new(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }

    fn add_antecedent_to_flow(&self, label: &Arc<FlowNode>, antecedent: &Arc<FlowNode>) {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }
        for ant in &label.antecedents {
            if Arc::ptr_eq(ant, antecedent) {
                return;
            }
        }
        let ptr = Arc::as_ptr(label) as *mut FlowNode;
        unsafe {
            (*ptr).antecedents.push(Arc::clone(antecedent));
        }

        self.set_flow_node_referenced(antecedent);
    }

    fn create_flow_switch_clause(
        &mut self,
        antecedent: &Arc<FlowNode>,
        clause: Option<&Arc<Node>>,
        switch_statement: &Arc<Node>,
        clause_start: usize,
        clause_end: usize,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        Arc::new(FlowNode {
            flags: FlowFlags::SWITCH_CLAUSE,
            node: clause.map(Arc::clone),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: Some(Arc::clone(switch_statement)),
            clause_range: Some((clause_start, clause_end)),
            reduce_target: None,
        })
    }

    pub(crate) fn bind_if_statement(&mut self, node: &Arc<Node>) {
        let mut then_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut else_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_if_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, then_stmt, else_stmt) = match &node.data {
            NodeData::IfStatement(data) => (
                data.expression.clone(),
                data.then_statement.clone(),
                data.else_statement.clone(),
            ),
            _ => return,
        };

        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            then_label.add_antecedent(true_flow);
            else_label.add_antecedent(false_flow);
        }

        self.current_flow = Some(then_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&then_stmt);
        if let Some(current) = &self.current_flow {
            post_if_label.add_antecedent(Arc::clone(current));
        }

        self.current_flow = Some(else_label.finish(self.unreachable_flow.as_ref().unwrap()));
        if let Some(else_s) = else_stmt {
            self.bind(&else_s);
        }
        if let Some(current) = &self.current_flow {
            post_if_label.add_antecedent(Arc::clone(current));
        }

        self.current_flow = Some(post_if_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    pub(crate) fn bind_while_statement(&mut self, node: &Arc<Node>) {
        let mut pre_while_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_body_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_while_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, stmt) = match &node.data {
            NodeData::WhileStatement(data) => (data.expression.clone(), data.statement.clone()),
            _ => return,
        };

        if let Some(current) = &self.current_flow {
            pre_while_label.add_antecedent(Arc::clone(current));
        }

        let loop_head = pre_while_label.finish_multi(self.unreachable_flow.as_ref().unwrap());
        self.current_flow = Some(Arc::clone(&loop_head));

        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            pre_body_label.add_antecedent(true_flow);
            post_while_label.add_antecedent(false_flow);
        }

        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));

        self.set_continue_target(node, &continue_acc);

        self.current_flow = Some(pre_body_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&stmt);
        if let Some(current) = &self.current_flow {
            FlowLabel::push_antecedent(&loop_head, Arc::clone(current));
        }

        for ant in &continue_acc.antecedents {
            FlowLabel::push_antecedent(&loop_head, Arc::clone(ant));
        }

        for ant in &break_acc.antecedents {
            post_while_label.add_antecedent(Arc::clone(ant));
        }

        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(post_while_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

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
        let is_narrowing_switch = expression.kind == SyntaxKind::TrueKeyword
            || self.is_narrowing_expression(&expression);
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

    fn bind_case_clause(&mut self, clause: &Arc<Node>, entry_flow: &Option<Arc<FlowNode>>) {
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

    fn is_narrowing_expression(&self, expr: &Arc<Node>) -> bool {
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

    fn is_narrowing_binary_expression(
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

    fn is_boolean_literal(node: &Arc<Node>) -> bool {
        matches!(node.kind, SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword)
    }

    fn is_narrowable_operand(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::ParenthesizedExpression => {
                expr.expression().map(|e| self.is_narrowable_operand(e)).unwrap_or(false)
            }
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

    fn is_narrowing_typeof_operands(&self, expr1: &Arc<Node>, expr2: &Arc<Node>) -> bool {
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

    fn contains_narrowable_reference(&self, expr: &Arc<Node>) -> bool {
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

    fn is_narrowable_reference(&self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty => true,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression => {
                node.expression().map(|e| self.is_narrowable_reference(e)).unwrap_or(false)
            }
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

    fn has_narrowable_argument(&self, expr: &Arc<Node>) -> bool {
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

    pub(crate) fn bind_throw_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ThrowStatement(data) = &node.data {
            self.bind(&data.expression);
        }
        self.current_flow = Some(self.unreachable_flow());
        self.has_flow_effects = true;
    }

    pub(crate) fn bind_try_statement(&mut self, node: &Arc<Node>) {
        let stmt = match &node.data {
            NodeData::TryStatement(data) => data,
            _ => return,
        };

        let save_return_target = self.current_return_target.take();
        let save_exception_target = self.current_exception_target.take();

        let normal_exit_label = Self::new_flow_accumulator();
        let return_label = Self::new_flow_accumulator();
        let mut exception_label = Self::new_flow_accumulator();

        if stmt.finally_block.is_some() {
            self.current_return_target = Some(Arc::clone(&return_label));
        }

        if let Some(current) = &self.current_flow {
            self.add_antecedent_to_flow(&exception_label, current);
        }
        self.current_exception_target = Some(Arc::clone(&exception_label));

        self.bind(&stmt.try_block);
        if let Some(current) = &self.current_flow {
            self.add_antecedent_to_flow(&normal_exit_label, current);
        }

        if let Some(catch_clause) = &stmt.catch_clause {
            self.current_flow = Some(Self::finish_flow_node(
                &exception_label,
                &self.unreachable_flow(),
            ));
            let catch_exception_label = Self::new_flow_accumulator();
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(&catch_exception_label, current);
            }
            self.current_exception_target = Some(Arc::clone(&catch_exception_label));
            exception_label = catch_exception_label;
            self.bind(catch_clause);
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(&normal_exit_label, current);
            }
        }

        self.current_return_target = save_return_target;
        self.current_exception_target = save_exception_target;

        if let Some(finally_block) = &stmt.finally_block {

            let finally_label = Self::new_flow_accumulator();
            for ant in normal_exit_label
                .antecedents
                .iter()
                .chain(exception_label.antecedents.iter())
                .chain(return_label.antecedents.iter())
            {
                self.add_antecedent_to_flow(&finally_label, ant);
            }
            let finally_node = Self::finish_flow_node(&finally_label, &self.unreachable_flow());
            self.current_flow = Some(Arc::clone(&finally_node));
            self.bind(finally_block);

            if self
                .current_flow
                .as_ref()
                .is_some_and(|f| f.flags.contains(FlowFlags::UNREACHABLE))
            {

                self.current_flow = Some(self.unreachable_flow());
            } else {
                let current_flow = self.current_flow.clone().expect("reachable flow");

                if self.current_return_target.is_some()
                    && !return_label.antecedents.is_empty()
                    && let Some(rt) = &self.current_return_target
                {
                    let reduce = self.create_reduce_label(
                        &finally_node,
                        &return_label.antecedents,
                        &current_flow,
                    );
                    self.add_antecedent_to_flow(rt, &reduce);
                }

                if self.current_exception_target.is_some()
                    && !exception_label.antecedents.is_empty()
                    && let Some(et) = &self.current_exception_target
                {
                    let reduce = self.create_reduce_label(
                        &finally_node,
                        &exception_label.antecedents,
                        &current_flow,
                    );
                    self.add_antecedent_to_flow(et, &reduce);
                }

                if !normal_exit_label.antecedents.is_empty() {
                    self.current_flow = Some(self.create_reduce_label(
                        &finally_node,
                        &normal_exit_label.antecedents,
                        &current_flow,
                    ));
                } else {
                    self.current_flow = Some(self.unreachable_flow());
                }
            }
        } else {
            self.current_flow = Some(Self::finish_flow_node(
                &normal_exit_label,
                &self.unreachable_flow(),
            ));
        }
    }

    fn finish_flow_node(
        node: &Arc<FlowNode>,
        unreachable: &Arc<FlowNode>,
    ) -> Arc<FlowNode> {
        if node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        if node.antecedents.len() == 1 {
            return Arc::clone(&node.antecedents[0]);
        }
        Arc::clone(node)
    }

    pub(crate) fn bind_break_statement(&mut self, node: &Arc<Node>) {

        let label_name = if let NodeData::BreakStatement(data) = &node.data {
            data.label.as_ref().map(|l| self.node_text(l))
        } else {
            None
        };

        if let Some(name) = label_name {

            let break_target = {
                let mut current = &self.active_label_list;
                let mut found = None;
                while let Some(label) = current {
                    if label.name == name {
                        found = Some(Arc::clone(&label.break_target));
                        break;
                    }
                    current = &label.next;
                }
                found
            };
            if let Some(target) = break_target {
                if let Some(current_flow) = &self.current_flow {
                    self.add_antecedent_to_flow(&target, current_flow);
                }

                let mut current = &mut self.active_label_list;
                while let Some(label) = current {
                    if label.name == name {
                        label.referenced = true;
                        break;
                    }
                    current = &mut label.next;
                }
            }
        } else if let Some(target) = &self.current_break_target {

            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(target, current);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
    }

    pub(crate) fn bind_continue_statement(&mut self, node: &Arc<Node>) {

        let label_name = if let NodeData::ContinueStatement(data) = &node.data {
            data.label.as_ref().map(|l| self.node_text(l))
        } else {
            None
        };

        if let Some(name) = label_name {

            let continue_target = {
                let mut current = &self.active_label_list;
                let mut found = None;
                while let Some(label) = current {
                    if label.name == name {
                        found = label.continue_target.clone();
                        break;
                    }
                    current = &label.next;
                }
                found
            };
            if let Some(ref target) = continue_target {
                if let Some(current_flow) = &self.current_flow {
                    self.add_antecedent_to_flow(&target, current_flow);
                }

                let mut current = &mut self.active_label_list;
                while let Some(label) = current {
                    if label.name == name {
                        label.referenced = true;
                        break;
                    }
                    current = &mut label.next;
                }
            }
        } else if let Some(target) = &self.current_continue_target {
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(target, current);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
    }

    fn set_continue_target(&mut self, loop_node: &Arc<Node>, target: &Arc<FlowNode>) {
        let mut node = Arc::clone(loop_node);
        let mut cursor = &mut self.active_label_list;
        loop {
            let Some(parent) = node.parent.clone() else { break };
            if parent.kind != SyntaxKind::LabeledStatement {
                break;
            }
            let Some(label) = cursor else { break };
            label.continue_target = Some(Arc::clone(target));
            node = parent;
            cursor = &mut label.next;
        }
    }

    pub(crate) fn bind_labeled_statement(&mut self, node: &Arc<Node>) {
        let stmt = match &node.data {
            NodeData::LabeledStatement(data) => data,
            _ => return,
        };

        let label_name = self.node_text(&stmt.label);

        let break_target = Self::new_flow_accumulator();

        let continue_target: Option<Arc<FlowNode>> = None;

        let active_label = Box::new(ActiveLabel {
            name: label_name,
            break_target: Arc::clone(&break_target),
            continue_target,
            referenced: false,
            next: self.active_label_list.take(),
        });

        self.active_label_list = Some(active_label);

        self.bind(&stmt.statement);

        let was_referenced = self
            .active_label_list
            .as_ref()
            .map_or(false, |l| l.referenced);

        self.active_label_list = self.active_label_list.take().and_then(|l| l.next);

        if !was_referenced {

            let label_ptr = Arc::as_ptr(&stmt.label) as *mut Node;
            unsafe {
                (*label_ptr).flags |= NodeFlags::Unreachable;
            }
        }

        if let Some(current) = &self.current_flow {
            self.add_antecedent_to_flow(&break_target, current);
        }
        self.current_flow = if break_target.antecedents.is_empty() {
            Some(self.unreachable_flow())
        } else {
            Some(break_target)
        };
    }

    fn is_push_or_unshift_identifier(&self, name: &str) -> bool {
        name == "push" || name == "unshift"
    }

    fn is_mutation_tracked_reference(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty => true,
            SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression => {
                if let Some(inner) = expr.expression() {
                    self.is_mutation_tracked_reference(&inner)
                } else {
                    false
                }
            }
            SyntaxKind::ElementAccessExpression => {

                if let NodeData::ElementAccessExpression(ea) = &expr.data {
                    if self.is_string_or_numeric_literal_like(&ea.argument_expression) {
                        return true;
                    }
                    return self.is_entity_name_expression(&ea.argument_expression)
                        && self.is_mutation_tracked_reference(&ea.expression);
                }
                false
            }
            _ => false,
        }
    }

    fn is_string_or_numeric_literal_like(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
        )
    }

    fn is_entity_name_expression(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName
        )
    }

    pub(crate) fn bind_call_expression_flow(&mut self, node: &Arc<Node>) {
        if let NodeData::CallExpression(data) = &node.data {
            let expr = &data.expression;

            if let NodeData::PropertyAccessExpression(prop) = &expr.data {
                let name = self.node_text(&prop.name);
                if self.is_push_or_unshift_identifier(&name)
                    && self.is_mutation_tracked_reference(&prop.expression)
                {

                    let current = self.current_flow.clone();
                    if let Some(current) = current {
                        self.current_flow = Some(self.create_flow_mutation(&current, node));
                    }
                }
            }
        }
    }

    pub(crate) fn bind_this_property_assignment(&mut self, _node: &Arc<Node>) {

    }

    pub(crate) fn collect_expando_assignment(&mut self, node: &Arc<Node>) {
        let NodeData::BinaryExpression(bin) = &node.data else {
            return;
        };
        if bin.operator_token.kind != SyntaxKind::EqualsToken {
            return;
        }
        let base = match &bin.left.data {
            NodeData::PropertyAccessExpression(pae)
                if pae.expression.kind == SyntaxKind::Identifier
                    && pae.name.kind == SyntaxKind::Identifier =>
            {
                &pae.expression
            }
            NodeData::ElementAccessExpression(eae)
                if eae.expression.kind == SyntaxKind::Identifier =>
            {
                &eae.expression
            }
            _ => return,
        };

        let base_name = base.text();
        if matches!(base_name, "exports" | "module" | "globalThis") {
            return;
        }
        self.expando_assignments
            .push((Arc::clone(node), self.block_scope_container.clone()));
    }

    pub(crate) fn process_expando_assignments(&mut self) {
        let assignments = std::mem::take(&mut self.expando_assignments);
        for (node, scope_start) in assignments {
            let NodeData::BinaryExpression(bin) = &node.data else {
                continue;
            };
            let base = match &bin.left.data {
                NodeData::PropertyAccessExpression(pae) => &pae.expression,
                NodeData::ElementAccessExpression(eae) => &eae.expression,
                _ => continue,
            };
            let base_name = base.text();
            let mut target: Option<Arc<Symbol>> = None;
            let mut scope = scope_start;
            while let Some(sc) = scope {
                if let Some(sym) = self
                    .symbol_map
                    .locals
                    .get(&sc.id())
                    .and_then(|l| l.get(base_name))
                {
                    target = Some(Arc::clone(sym));
                    break;
                }

                if matches!(
                    sc.kind,
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
                ) && let Some(sym) = self.symbol_map.symbol_of(&sc)
                {
                    let hit = sym
                        .members
                        .get(base_name)
                        .or_else(|| sym.exports.get(base_name))
                        .cloned();
                    if let Some(h) = hit {
                        target = Some(h);
                        break;
                    }
                }
                scope = sc.parent.clone();
            }
            let Some(sym) = target else { continue };

            if !sym
                .value_declaration
                .as_ref()
                .is_some_and(|d| d.kind == SyntaxKind::FunctionDeclaration)
            {
                continue;
            }
            let member_name: Option<String> = match &bin.left.data {
                NodeData::PropertyAccessExpression(pae) => Some(pae.name.text().to_string()),
                NodeData::ElementAccessExpression(eae) => {
                    match &eae.argument_expression.data {
                        NodeData::StringLiteral(s) => Some(s.text.clone()),
                        NodeData::NumericLiteral(n) => Some(n.text.clone()),
                        _ => None,
                    }
                }
                _ => None,
            };
            match member_name {
                Some(mname) => {

                    let existing = sym
                        .exports
                        .get(&mname)
                        .or_else(|| sym.members.get(&mname))
                        .cloned()
                        .or_else(|| {
                            sym.declarations
                                .iter()
                                .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                                .find_map(|md| {
                                    self.symbol_map
                                        .locals
                                        .get(&md.id())
                                        .and_then(|l| l.get(&mname))
                                        .cloned()
                                })
                        });
                    let eligible = existing.as_ref().map_or(true, |e| {
                        e.declarations
                            .iter()
                            .all(|d| d.kind == SyntaxKind::BinaryExpression)
                    });
                    if !eligible {
                        continue;
                    }
                    match existing {
                        Some(e) => {
                            let e_mut = Arc::as_ptr(&e) as *mut Symbol;
                            unsafe { (*e_mut).declarations.push(Arc::clone(&node)) };
                        }
                        None => {
                            let prop = self.new_symbol(SymbolFlags::Property, mname.clone());
                            let prop_mut = Arc::as_ptr(&prop) as *mut Symbol;
                            unsafe {
                                (*prop_mut).declarations.push(Arc::clone(&node));
                                (*prop_mut).parent = Some(Arc::clone(&sym));
                            }
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut).exports.insert(mname, prop);
                            }
                        }
                    }
                }
                None => {

                    let pseudo = sym
                        .exports
                        .get(crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT)
                        .cloned();
                    match pseudo {
                        Some(p) => {
                            let p_mut = Arc::as_ptr(&p) as *mut Symbol;
                            unsafe { (*p_mut).declarations.push(Arc::clone(&node)) };
                        }
                        None => {
                            let p = self.new_symbol(
                                SymbolFlags::empty(),
                                crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT.to_string(),
                            );
                            let p_mut = Arc::as_ptr(&p) as *mut Symbol;
                            unsafe {
                                (*p_mut).declarations.push(Arc::clone(&node));
                                (*p_mut).parent = Some(Arc::clone(&sym));
                            }
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut)
                                    .exports
                                    .insert(crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT.to_string(), p);
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn bind_expression_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ExpressionStatement(data) = &node.data {
            self.bind(&data.expression);

            if let NodeData::BinaryExpression(bin_data) = &data.expression.data {
                if is_assignment_operator(bin_data.operator_token.kind) {
                    if let Some(current) = self.current_flow.take() {
                        let assign_flow = self.create_flow_assignment(&current, &data.expression);
                        self.symbol_map
                            .set_flow_node(&data.expression, Arc::clone(&assign_flow));
                        self.current_flow = Some(assign_flow);
                    }

                    if let NodeData::ElementAccessExpression(ea) = &bin_data.left.data {
                        if self.is_mutation_tracked_reference(&ea.expression) {
                            let current = self.current_flow.clone();
                            if let Some(current) = current {
                                self.current_flow =
                                    Some(self.create_flow_mutation(&current, &data.expression));
                            }
                        }
                    }
                }
            }

            if let NodeData::CallExpression(_) = &data.expression.data {
                if let Some(current) = self.current_flow.take() {
                    let call_flow = self.create_flow_call(&current, &data.expression);
                    self.symbol_map
                        .set_flow_node(&data.expression, Arc::clone(&call_flow));
                    self.current_flow = Some(call_flow);
                }
            }
        } else {
            self.bind_children(node);
        }
    }

    pub(crate) fn is_in_for_in_or_of_head(node: &Arc<Node>) -> bool {
        let Some(parent) = &node.parent else {
            return false;
        };
        let Some(grandparent) = &parent.parent else {
            return false;
        };
        matches!(
            grandparent.kind,
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
        )
    }

    pub(crate) fn bind_assignment_target_flow(&mut self, node: &Arc<Node>) {
        match &node.data {
            NodeData::ArrayLiteralExpression(arr) => {
                for e in &arr.elements.nodes {
                    if e.kind == SyntaxKind::SpreadElement {
                        if let Some(inner) = e.expression() {
                            self.bind_assignment_target_flow(&inner);
                        }
                    } else {
                        self.bind_destructuring_target_flow(e);
                    }
                }
            }
            NodeData::ObjectLiteralExpression(obj) => {
                for p in &obj.properties.nodes {
                    match &p.data {
                        NodeData::PropertyAssignment(pa) => {
                            self.bind_destructuring_target_flow(&pa.initializer);
                        }
                        NodeData::ShorthandPropertyAssignment(sa) => {
                            self.bind_assignment_target_flow(&sa.name);
                        }
                        NodeData::SpreadAssignment(sp) => {
                            self.bind_assignment_target_flow(&sp.expression);
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                if self.is_mutation_tracked_reference(node)
                    && matches!(
                        node.kind,
                        SyntaxKind::Identifier
                            | SyntaxKind::PropertyAccessExpression
                            | SyntaxKind::ElementAccessExpression
                            | SyntaxKind::ParenthesizedExpression
                            | SyntaxKind::NonNullExpression
                            | SyntaxKind::ThisKeyword
                            | SyntaxKind::SuperKeyword
                            | SyntaxKind::MetaProperty
                    )
                {
                    if let Some(current) = self.current_flow.take() {
                        let assign_flow = self.create_flow_assignment(&current, node);
                        self.current_flow = Some(assign_flow);
                    }
                }
            }
        }
    }

    fn bind_destructuring_target_flow(&mut self, node: &Arc<Node>) {
        if let NodeData::BinaryExpression(bin) = &node.data {
            if bin.operator_token.kind == SyntaxKind::EqualsToken {
                self.bind_assignment_target_flow(&bin.left);
                return;
            }
        }
        self.bind_assignment_target_flow(node);
    }

    pub(crate) fn bind_initialized_variable_flow(&mut self, node: &Arc<Node>) {
        let name = match &node.data {
            NodeData::VariableDeclaration(d) => Some(Arc::clone(&d.name)),
            NodeData::BindingElement(d) => d.name.clone(),
            _ => None,
        };
        let Some(name) = name else { return };
        if matches!(
            name.kind,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        ) {
            if let NodeData::BindingPattern(pattern) = &name.data {
                for child in &pattern.elements.nodes {
                    self.bind_initialized_variable_flow(child);
                }
            }
            return;
        }
        if let Some(current) = self.current_flow.take() {
            let assign_flow = self.create_flow_assignment(&current, node);
            self.symbol_map.set_flow_node(node, Arc::clone(&assign_flow));
            self.current_flow = Some(assign_flow);
        }
    }

    pub(crate) fn check_contextual_identifier(&mut self, node: &Arc<Node>) {
        let Some(file) = self.current_source_file.clone() else {
            return;
        };
        if file.has_parse_diagnostics
            || node.flags.contains(NodeFlags::Ambient)
            || node.flags.contains(NodeFlags::JSDoc)
            || is_identifier_name(node)
            || file.is_declaration_file
        {
            return;
        }

        {
            let mut anc = node.parent.as_ref();
            while let Some(a) = anc {
                if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                    return;
                }
                anc = a.parent.as_ref();
            }
        }
        let Some(kind) = crate::scanner::string_to_keyword(node.text()) else {
            return;
        };
        let is_future_reserved = matches!(
            kind,
            SyntaxKind::ImplementsKeyword
                | SyntaxKind::InterfaceKeyword
                | SyntaxKind::LetKeyword
                | SyntaxKind::PackageKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::YieldKeyword
        );
        let message = if is_future_reserved {
            if crate::ast::utilities::get_containing_class(node).is_some() {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else if file.external_module_indicator.is_some() {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE
            }
        } else if kind == SyntaxKind::AwaitKeyword {
            if file.external_module_indicator.is_some()
                && crate::ast::utilities::is_in_top_level_context(node)
            {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE
            } else if node.flags.contains(NodeFlags::AwaitContext) {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE
            } else {
                return;
            }
        } else if kind == SyntaxKind::YieldKeyword && node.flags.contains(NodeFlags::YieldContext) {
            IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE
        } else {
            return;
        };
        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
            Some(file),
            node.loc,
            message,
            vec![node.text().to_string()],
        ));
    }
}
