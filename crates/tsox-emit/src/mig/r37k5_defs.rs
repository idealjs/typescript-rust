#![allow(dead_code, unused_imports, unused_variables)]

use tsox_frontend::ast::node::Node;

use crate::mig::m4q::r33k12_defs::{PrinterState, NameGenerator};
use crate::mig::m4q::Printer;

impl Printer {
    pub fn enter_node(&mut self, node: &Node) -> PrinterState {
        PrinterState {
            comment_state: None,
            source_map_state: None,
        }
    }

    pub fn exit_node(&mut self, node: &Node, previous_state: PrinterState) {
    }

    pub fn push_name_generation_scope(&mut self, node: &Node) {
        self.name_generator.push_scope(false);
    }

    pub fn pop_name_generation_scope(&mut self, node: &Node) {
        self.name_generator.pop_scope(false);
    }
}
