#![allow(unused_imports)]

use super::*;

impl Binder {
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

    pub(crate) fn finish_flow_node(
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

    pub(crate) fn set_continue_target(&mut self, loop_node: &Arc<Node>, target: &Arc<FlowNode>) {
        let mut node = Arc::clone(loop_node);
        let mut cursor = &mut self.active_label_list;
        loop {
            let Some(parent) = node.parent.clone() else {
                break;
            };
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

    pub(crate) fn is_push_or_unshift_identifier(&self, name: &str) -> bool {
        name == "push" || name == "unshift"
    }
}
