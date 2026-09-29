use std::sync::Arc;

use super::m3e::{FlowReduceLabelData, FlowSwitchClauseData};
use crate::ast::node_data_generated::{CallExpressionData, NodeData, NewExpressionData};
use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;

pub fn argument_list(node: &Node) -> Option<&NodeList> {
    match &node.data {
        NodeData::CallExpression(CallExpressionData { arguments, .. }) => Some(arguments),
        NodeData::NewExpression(NewExpressionData { arguments, .. }) => arguments.as_deref(),
        _ => panic!("Unhandled case in Node.Arguments: {:?}", node.kind),
    }
}

pub fn arguments(node: &Node) -> &[Arc<Node>] {
    match argument_list(node) {
        Some(list) => &list.nodes,
        None => &[],
    }
}

pub fn as_flow_reduce_label_data(node: &Node) -> &FlowReduceLabelData {
    match &node.data {
        NodeData::FlowReduceLabelData(d) => d,
        _ => panic!("unexpected node data: {:?}", node.kind),
    }
}

pub fn as_flow_switch_clause_data(node: &Node) -> &FlowSwitchClauseData {
    match &node.data {
        NodeData::FlowSwitchClauseData(d) => d,
        _ => panic!("unexpected node data: {:?}", node.kind),
    }
}
