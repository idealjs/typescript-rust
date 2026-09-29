#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::checker_classes::{class_decl_extends_null, class_extends_heritage_element};
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::{FlowFlags, FlowNode, Node, SyntaxKind};

impl Checker {
    // Go checkThisBeforeSuper：派生类构造器中 this/super 属性访问
    // 依据 flow 图判断是否已越过 super() 调用
    pub(crate) fn check_this_before_super(
        &mut self,
        node: &Arc<Node>,
        container: &Arc<Node>,
        message: Message,
    ) {
        let Some(class_decl) = container.parent() else {
            return;
        };
        if class_extends_heritage_element(&class_decl).is_none()
            || class_decl_extends_null(&class_decl)
        {
            return;
        }
        let Some(flow) = self.program.symbol_map().flow_node_of(node) else {
            return;
        };
        if !self.is_post_super_flow_node(flow) {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                message,
                vec![],
            ));
        }
    }

    // Go isPostSuperFlowNodeWorker
    pub(crate) fn is_post_super_flow_node(&self, flow: &Arc<FlowNode>) -> bool {
        self.is_post_super_worker(flow, &mut Vec::new(), 0)
    }

    fn is_post_super_worker(
        &self,
        flow: &Arc<FlowNode>,
        reduce_labels: &mut Vec<(Arc<FlowNode>, Vec<Arc<FlowNode>>)>,
        depth: u32,
    ) -> bool {
        if depth > FLOW_POST_SUPER_MAX_DEPTH {
            return true;
        }
        let flags = flow.flags;
        if flags.intersects(
            FlowFlags::ASSIGNMENT
                | FlowFlags::CONDITION
                | FlowFlags::ARRAY_MUTATION
                | FlowFlags::SWITCH_CLAUSE,
        ) {
            let Some(antecedent) = &flow.antecedent else {
                return flags.contains(FlowFlags::UNREACHABLE);
            };
            return self.is_post_super_worker(antecedent, reduce_labels, depth + 1);
        }
        if flags.contains(FlowFlags::CALL) {
            if let Some(node) = &flow.node
                && let Some(expr) = node.expression()
                && expr.kind == SyntaxKind::SuperKeyword
            {
                return true;
            }
            let Some(antecedent) = &flow.antecedent else {
                return flags.contains(FlowFlags::UNREACHABLE);
            };
            return self.is_post_super_worker(antecedent, reduce_labels, depth + 1);
        }
        if flags.contains(FlowFlags::BRANCH_LABEL) {
            let effective: Vec<Arc<FlowNode>> = reduce_labels
                .iter()
                .rev()
                .find(|(target, _)| Arc::ptr_eq(target, flow))
                .map(|(_, ants)| ants.clone())
                .unwrap_or_else(|| flow.antecedents.clone());
            return effective
                .iter()
                .all(|ant| self.is_post_super_worker(ant, reduce_labels, depth + 1));
        }
        if flags.contains(FlowFlags::LOOP_LABEL) {
            let Some(entry) = flow.antecedents.first() else {
                return flags.contains(FlowFlags::UNREACHABLE);
            };
            return self.is_post_super_worker(entry, reduce_labels, depth + 1);
        }
        if flags.contains(FlowFlags::REDUCE_LABEL) {
            if let Some(target) = &flow.reduce_target {
                reduce_labels.push((Arc::clone(target), flow.antecedents.clone()));
                let result = match &flow.antecedent {
                    Some(antecedent) => {
                        self.is_post_super_worker(antecedent, reduce_labels, depth + 1)
                    }
                    None => flags.contains(FlowFlags::UNREACHABLE),
                };
                reduce_labels.pop();
                return result;
            }
            let Some(antecedent) = &flow.antecedent else {
                return flags.contains(FlowFlags::UNREACHABLE);
            };
            return self.is_post_super_worker(antecedent, reduce_labels, depth + 1);
        }
        flags.contains(FlowFlags::UNREACHABLE)
    }
}

const FLOW_POST_SUPER_MAX_DEPTH: u32 = 5000;
