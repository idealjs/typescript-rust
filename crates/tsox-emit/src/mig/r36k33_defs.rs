#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{BindingPatternData, NodeData};

pub trait R36K33NodeAsExt {
    fn as_binding_pattern(&self) -> &BindingPatternData;
}

impl R36K33NodeAsExt for Node {
    fn as_binding_pattern(&self) -> &BindingPatternData {
        match &self.data {
            NodeData::BindingPattern(d) => d,
            _ => panic!("AsBindingPattern on wrong node kind"),
        }
    }
}
