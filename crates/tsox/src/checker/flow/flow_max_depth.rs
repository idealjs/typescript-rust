use std::sync::Arc;

use crate::ast::{FlowNode, Node, Symbol};

use super::*;

pub(crate) const FLOW_MAX_DEPTH: u32 = 2000;

#[derive(Clone)]
pub(crate) enum FlowRef {
    Symbol(Arc<Symbol>),
    Node(Arc<Node>),
}

impl FlowRef {
    pub(crate) fn anchor_node(&self) -> Option<Arc<Node>> {
        match self {
            FlowRef::Node(n) => Some(Arc::clone(n)),
            FlowRef::Symbol(s) => s.declarations.first().map(Arc::clone),
        }
    }
}

#[derive(Default)]
pub(crate) struct FlowQuery {
    pub(crate) memo: std::collections::HashMap<usize, Arc<Type>>,
    pub(crate) on_path: std::collections::HashSet<usize>,

    pub(crate) reduce_labels: Vec<(std::sync::Arc<FlowNode>, Vec<std::sync::Arc<FlowNode>>)>,

    pub(crate) loop_stack: Vec<(usize, Vec<Arc<Type>>)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NarrowKind {
    TrueBranch,

    FalseBranch,
}
