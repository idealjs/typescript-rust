use crate::binder::{Binder, FlowLabel, FlowNode};
use std::sync::Arc;

pub(crate) fn get_binder() -> Binder {
    Binder::new()
}

impl Binder {
    pub(crate) fn finish_flow_label(&mut self, label: &FlowLabel) -> Arc<FlowNode> {
        let Some(unreachable_flow) = self.unreachable_flow.clone() else {
            let mut label_node = FlowNode::new(label.node.flags);
            label_node.node = label.node.node.clone();
            label_node.antecedent = label.node.antecedent.clone();
            label_node.antecedents = label.node.antecedents.clone();
            label_node.switch_statement = label.node.switch_statement.clone();
            label_node.clause_range = label.node.clause_range;
            label_node.reduce_target = label.node.reduce_target.clone();
            return Arc::new(label_node);
        };
        match label.node.antecedents.len() {
            0 => unreachable_flow,
            1 => Arc::clone(&label.node.antecedents[0]),
            _ => {
                let mut label_node = FlowNode::new(label.node.flags);
                label_node.node = label.node.node.clone();
                label_node.antecedent = label.node.antecedent.clone();
                label_node.antecedents = label.node.antecedents.clone();
                label_node.switch_statement = label.node.switch_statement.clone();
                label_node.clause_range = label.node.clause_range;
                label_node.reduce_target = label.node.reduce_target.clone();
                Arc::new(label_node)
            }
        }
    }
}
