use super::flow::FlowNode;
use super::symbol::{Symbol, SymbolTable};
use crate::ast::diagnostic::Diagnostic;
use crate::ast::node::Node;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct NodeSymbolMap {
    pub symbols: HashMap<u64, Arc<Symbol>>,

    pub locals: HashMap<u64, SymbolTable>,

    pub flow_nodes: HashMap<u64, Arc<FlowNode>>,

    pub binder_diagnostics: Vec<Diagnostic>,
}

impl NodeSymbolMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol_of(&self, node: &Node) -> Option<&Arc<Symbol>> {
        self.symbols.get(&node.id())
    }

    pub fn locals_of(&self, node: &Node) -> Option<&SymbolTable> {
        self.locals.get(&node.id())
    }

    pub fn flow_node_of(&self, node: &Node) -> Option<&Arc<FlowNode>> {
        self.flow_nodes.get(&node.id())
    }

    pub fn set_symbol(&mut self, node: &Node, symbol: Arc<Symbol>) {
        self.symbols.insert(node.id(), symbol);
    }

    pub fn set_locals(&mut self, node: &Node, locals: SymbolTable) {
        self.locals.insert(node.id(), locals);
    }

    pub fn set_flow_node(&mut self, node: &Node, flow: Arc<FlowNode>) {
        self.flow_nodes.insert(node.id(), flow);
    }
}
