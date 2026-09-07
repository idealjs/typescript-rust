use super::*;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct FlowLabel {
    pub(crate) node: FlowNode,
}

impl FlowLabel {
    pub(crate) fn new(flags: FlowFlags) -> Self {
        Self {
            node: FlowNode::new(flags),
        }
    }

    pub(crate) fn add_antecedent(&mut self, antecedent: Arc<FlowNode>) {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }

        for ant in &self.node.antecedents {
            if Arc::ptr_eq(ant, &antecedent) {
                return;
            }
        }
        self.node.antecedents.push(antecedent);
    }

    pub(crate) fn finish_multi(&self, unreachable: &Arc<FlowNode>) -> Arc<FlowNode> {
        if self.node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        Arc::new(FlowNode {
            flags: self.node.flags,
            node: None,
            antecedent: None,
            antecedents: self.node.antecedents.clone(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }

    pub(crate) fn push_antecedent(node: &Arc<FlowNode>, ant: Arc<FlowNode>) {
        if ant.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }
        let ptr = Arc::as_ptr(node) as *mut FlowNode;
        unsafe {
            for existing in &(*ptr).antecedents {
                if Arc::ptr_eq(existing, &ant) {
                    return;
                }
            }
            (*ptr).antecedents.push(ant);
        }
    }

    pub(crate) fn finish(&self, unreachable: &Arc<FlowNode>) -> Arc<FlowNode> {
        if self.node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        if self.node.antecedents.len() == 1 {
            return Arc::clone(&self.node.antecedents[0]);
        }
        Arc::new(FlowNode {
            flags: self.node.flags,
            node: None,
            antecedent: None,
            antecedents: self.node.antecedents.clone(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }
}

#[derive(Debug)]
pub(crate) struct ActiveLabel {
    pub(crate) name: String,
    pub(crate) break_target: Arc<FlowNode>,
    pub(crate) continue_target: Option<Arc<FlowNode>>,
    pub(crate) referenced: bool,
    pub(crate) next: Option<Box<ActiveLabel>>,
}
