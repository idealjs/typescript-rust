#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::Arc;

use tsox_core::collections::set::Set;
use tsox_core::core::core::append_if_unique;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::utilities::is_prologue_directive;
use tsox_frontend::ast::{is_block, is_function_declaration, is_identifier, is_variable_statement};

use crate::printer::EmitContext;

use super::m4m_3::{EmitFlags, EmitHelper};

impl EmitContext {
    fn find_span_end_with_emit_context(
        &self,
        statements: &[Arc<Node>],
        pred: fn(&EmitContext, &Arc<Node>) -> bool,
        start: usize,
    ) -> usize {
        let mut i = start;
        while i < statements.len() && pred(self, &statements[i]) {
            i += 1;
        }
        i
    }

    pub fn add_initialization_statement(&mut self, node: &Arc<Node>) {
        self.set_emit_flags(node, EmitFlags::CUSTOM_PROLOGUE);
        panic!(
            "EmitContext varScopeStack not yet ported, Go addInitializationStatement needs it"
        );
    }

    pub fn add_emit_helper(&mut self, node: &Arc<Node>, helpers: &[Arc<EmitHelper>]) {
        let mut entry = self.emit_nodes_get_mut(node);
        for h in helpers {
            if !entry.helpers.iter().any(|e| Arc::ptr_eq(e, h)) {
                entry.helpers.push(h.clone());
            }
        }
    }

    pub fn convert_to_function_block(&mut self, node: &Arc<Node>, multi_line: bool) -> Arc<Node> {
        if is_block(node) {
            return node.clone();
        }
        let return_statement = self.factory().new_return_statement(Some(node));
        let mut return_statement = return_statement;
        if let Some(n) = Arc::get_mut(&mut return_statement) {
            n.loc = node.loc;
        }
        let statements = self.factory().new_node_list(vec![return_statement]);
        let mut statements = statements;
        if let Some(l) = Arc::get_mut(&mut statements) {
            l.loc = node.loc;
        }
        let block = self.factory().new_block(&statements, multi_line);
        let mut block = block;
        if let Some(n) = Arc::get_mut(&mut block) {
            n.loc = node.loc;
        }
        block
    }
}

fn find_span_end(
    statements: &[Arc<Node>],
    pred: fn(&Arc<Node>) -> bool,
    start: usize,
) -> usize {
    let mut i = start;
    while i < statements.len() && pred(&statements[i]) {
        i += 1;
    }
    i
}

pub fn is_hoisted_variable(node: &Arc<Node>) -> bool {
    is_identifier(&node.name().unwrap()) && node.initializer().is_none()
}
