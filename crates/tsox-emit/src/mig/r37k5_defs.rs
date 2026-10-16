#![allow(dead_code, unused_imports, unused_variables)]

use tsox_frontend::ast::node::Node;

use crate::mig::m4q::r33k12_defs::{PrinterState, NameGenerator};
use crate::mig::m4q::Printer;

impl Printer {
    pub fn enter_node(&mut self, node: &Node) -> PrinterState { ::tsox_core::fntrace::enter("enter_node"); 
        PrinterState {
            comment_state: None,
            source_map_state: None,
        }
    }

    pub fn exit_node(&mut self, node: &Node, previous_state: PrinterState) { ::tsox_core::fntrace::enter("exit_node"); 
    }

    pub fn push_name_generation_scope(&mut self, node: &Node) { ::tsox_core::fntrace::enter("push_name_generation_scope"); 
        self.name_generator.push_scope(false);
    }

    pub fn pop_name_generation_scope(&mut self, node: &Node) { ::tsox_core::fntrace::enter("pop_name_generation_scope"); 
        self.name_generator.pop_scope(false);
    }
}
