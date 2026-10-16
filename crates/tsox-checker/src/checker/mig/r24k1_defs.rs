use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::{FlowFlags, FlowNode};
use tsox_core::core::compiler_options::ModuleKind;

impl Checker {
    pub(crate) fn emit_module_format_of_node_source_file(&self, node: &Arc<Node>) -> ModuleKind { ::tsox_core::fntrace::enter("emit_module_format_of_node_source_file"); 
        let file = tsox_frontend::ast::get_source_file_of_node(node);
        let file_name = file.as_ref().and_then(|f| {
            self.program
                .source_files()
                .iter()
                .find(|sf| Arc::ptr_eq(&sf.node, f))
                .map(|sf| sf.file_name.clone())
        });
        match file_name {
            Some(name) => self.program.get_emit_module_format_of_file(&name),
            None => ModuleKind::None,
        }
    }
}

pub(crate) fn new_start_flow_node() -> Arc<FlowNode> { ::tsox_core::fntrace::enter("new_start_flow_node"); 
    Arc::new(FlowNode::new(FlowFlags::START))
}

pub(crate) fn new_condition_flow_node(
    flags: FlowFlags,
    node: &Arc<Node>,
    antecedent: &Arc<FlowNode>,
) -> Arc<FlowNode> { ::tsox_core::fntrace::enter("new_condition_flow_node"); 
    let mut flow = FlowNode::new(flags);
    flow.node = Some(Arc::clone(node));
    flow.antecedent = Some(Arc::clone(antecedent));
    Arc::new(flow)
}
