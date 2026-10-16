#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ast::{self, Node};

pub fn get_leftmost_access_expression(expr: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_leftmost_access_expression"); 
    let mut expr = expr.clone();
    while ast::is_access_expression(&expr) {
        match expr.expression() {
            Some(e) => expr = e.clone(),
            None => break,
        }
    }
    expr
}

pub fn get_text_of_property_name(name: &Node) -> String { ::tsox_core::fntrace::enter("get_text_of_property_name"); 
    super::m3h::try_get_text_of_property_name(name).unwrap_or_default()
}
