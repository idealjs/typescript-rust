use crate::ast::diagnostic::Diagnostic;
use crate::ast::node::Node;
use crate::ast::symbol_flow::FlowNode;
use crate::ast::symbol_symbol::Symbol;
use crate::ast::symbol_symbol::SymbolTable;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct NodeSymbolMap {
    pub symbols: HashMap<u64, Arc<Symbol>>,

    pub locals: HashMap<u64, SymbolTable>,

    pub flow_nodes: HashMap<u64, Arc<FlowNode>>,

    pub end_flow_nodes: HashMap<u64, Arc<FlowNode>>,

    pub binder_diagnostics: Vec<Diagnostic>,
}

impl NodeSymbolMap {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self::default()
    }

    pub fn symbol_of(&self, node: &Node) -> Option<&Arc<Symbol>> { ::tsox_core::fntrace::enter("symbol_of"); 
        self.symbols.get(&node.id())
    }

    pub fn locals_of(&self, node: &Node) -> Option<&SymbolTable> { ::tsox_core::fntrace::enter("locals_of"); 
        self.locals.get(&node.id())
    }

    pub fn flow_node_of(&self, node: &Node) -> Option<&Arc<FlowNode>> { ::tsox_core::fntrace::enter("flow_node_of"); 
        self.flow_nodes.get(&node.id())
    }

    pub fn set_symbol(&mut self, node: &Node, symbol: Arc<Symbol>) { ::tsox_core::fntrace::enter("set_symbol"); 
        self.symbols.insert(node.id(), symbol);
    }

    pub fn set_locals(&mut self, node: &Node, locals: SymbolTable) { ::tsox_core::fntrace::enter("set_locals"); 
        self.locals.insert(node.id(), locals);
    }

    pub fn set_flow_node(&mut self, node: &Node, flow: Arc<FlowNode>) { ::tsox_core::fntrace::enter("set_flow_node"); 
        self.flow_nodes.insert(node.id(), flow);
    }

    pub fn end_flow_node_of(&self, node: &Node) -> Option<&Arc<FlowNode>> { ::tsox_core::fntrace::enter("end_flow_node_of"); 
        self.end_flow_nodes.get(&node.id())
    }

    pub fn set_end_flow_node(&mut self, node: &Node, flow: Arc<FlowNode>) { ::tsox_core::fntrace::enter("set_end_flow_node"); 
        self.end_flow_nodes.insert(node.id(), flow);
    }
}
