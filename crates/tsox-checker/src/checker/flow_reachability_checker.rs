#![allow(unused_imports)]

use crate::checker::flow_impl_chunk::*;

impl Checker {
    pub fn is_reachable_flow_node(&mut self, flow: &Arc<FlowNode>) -> bool {
        let mut reduce_labels: Vec<(usize, Vec<Arc<FlowNode>>)> = Vec::new();
        self.is_reachable_flow_node_worker(flow, false, &mut reduce_labels)
    }

    fn is_reachable_flow_node_worker(
        &mut self,
        flow: &Arc<FlowNode>,
        no_cache_check: bool,
        reduce_labels: &mut Vec<(usize, Vec<Arc<FlowNode>>)>,
    ) -> bool {
        let mut flow = Arc::clone(flow);
        let mut no_cache_check = no_cache_check;
        loop {
            let flags = flow.flags;
            if flags.contains(FlowFlags::SHARED) {
                if !no_cache_check && reduce_labels.is_empty() {
                    let key = Arc::as_ptr(&flow) as usize as u64;
                    if let Some(&reachable) = self.flow_node_reachable.get(&key) {
                        return reachable;
                    }
                    let reachable =
                        self.is_reachable_flow_node_worker(&flow, true, reduce_labels);
                    self.flow_node_reachable.insert(key, reachable);
                    return reachable;
                }
                no_cache_check = false;
            }
            if flags.intersects(
                FlowFlags::ASSIGNMENT
                    | FlowFlags::CONDITION
                    | FlowFlags::ARRAY_MUTATION
                    | FlowFlags::CALL,
            ) {
                let Some(antecedent) = &flow.antecedent else {
                    return false;
                };
                flow = Arc::clone(antecedent);
                continue;
            }
            if flags.contains(FlowFlags::BRANCH_LABEL) {
                let antecedents: Vec<Arc<FlowNode>> =
                    branch_label_antecedents(&flow, reduce_labels).to_vec();
                for antecedent in &antecedents {
                    if self.is_reachable_flow_node_worker(antecedent, false, reduce_labels) {
                        return true;
                    }
                }
                return false;
            }
            if flags.contains(FlowFlags::LOOP_LABEL) {
                if flow.antecedents.is_empty() {
                    return false;
                }
                flow = Arc::clone(&flow.antecedents[0]);
                continue;
            }
            if flags.contains(FlowFlags::SWITCH_CLAUSE) {
                let bypass = matches!(flow.clause_range, Some((start, end)) if start == end);
                if bypass {
                    let Some(switch_statement) = &flow.switch_statement else {
                        return false;
                    };
                    if self.is_exhaustive_switch_statement(switch_statement) {
                        return false;
                    }
                }
                let Some(antecedent) = &flow.antecedent else {
                    return false;
                };
                flow = Arc::clone(antecedent);
                continue;
            }
            if flags.contains(FlowFlags::REDUCE_LABEL) {
                let Some(target) = &flow.reduce_target else {
                    return false;
                };
                let target_key = Arc::as_ptr(target) as usize;
                let antecedents = flow.antecedents.clone();
                let Some(antecedent) = flow.antecedent.clone() else {
                    return false;
                };
                reduce_labels.push((target_key, antecedents));
                let result =
                    self.is_reachable_flow_node_worker(&antecedent, false, reduce_labels);
                reduce_labels.pop();
                return result;
            }
            return !flags.contains(FlowFlags::UNREACHABLE);
        }
    }

    pub fn function_has_implicit_return(&mut self, fn_node: &Arc<Node>) -> bool {
        let Some(end_flow) = self
            .program
            .symbol_map()
            .flow_node_of(fn_node)
            .map(Arc::clone)
        else {
            return false;
        };
        self.is_reachable_flow_node(&end_flow)
    }
}

fn branch_label_antecedents<'a>(
    flow: &'a Arc<FlowNode>,
    reduce_labels: &'a [(usize, Vec<Arc<FlowNode>>)],
) -> &'a [Arc<FlowNode>] {
    let mut i = reduce_labels.len();
    while i != 0 {
        i -= 1;
        if reduce_labels[i].0 == Arc::as_ptr(flow) as usize {
            return &reduce_labels[i].1;
        }
    }
    &flow.antecedents
}
