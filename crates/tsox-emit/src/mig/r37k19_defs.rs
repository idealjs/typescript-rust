#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::{ModifierList, NodeList};

pub trait R37K19ArcNodeExt {
    fn set_loc(&mut self, loc: TextRange);
}

impl R37K19ArcNodeExt for Arc<Node> {
    fn set_loc(&mut self, loc: TextRange) { ::tsox_core::fntrace::enter("set_loc"); 
        if let Some(node) = Arc::get_mut(self) {
            node.loc = loc;
        }
    }
}
