use std::sync::Arc;

use super::m3e::{FlowReduceLabelData, FlowSwitchClauseData};
use crate::ast::node_data_generated::{CallExpressionData, NodeData, NewExpressionData};
use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;

pub fn argument_list(node: &Node) -> Option<&NodeList> { ::tsox_core::fntrace::enter("argument_list"); 
    match &node.data {
        NodeData::CallExpression(CallExpressionData { arguments, .. }) => Some(arguments),
        NodeData::NewExpression(NewExpressionData { arguments, .. }) => arguments.as_deref(),
        _ => panic!("Unhandled case in Node.Arguments: {:?}", node.kind),
    }
}

pub fn arguments(node: &Node) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("arguments"); 
    match argument_list(node) {
        Some(list) => &list.nodes,
        None => &[],
    }
}

pub fn as_flow_reduce_label_data(node: &Node) -> &FlowReduceLabelData { ::tsox_core::fntrace::enter("as_flow_reduce_label_data"); 
    match &node.data {
        NodeData::FlowReduceLabelData(d) => d,
        _ => panic!("unexpected node data: {:?}", node.kind),
    }
}

pub fn as_flow_switch_clause_data(node: &Node) -> &FlowSwitchClauseData { ::tsox_core::fntrace::enter("as_flow_switch_clause_data"); 
    match &node.data {
        NodeData::FlowSwitchClauseData(d) => d,
        _ => panic!("unexpected node data: {:?}", node.kind),
    }
}
