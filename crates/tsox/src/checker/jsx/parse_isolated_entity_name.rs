#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Node;

bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct JsxFlags: u32 {

        const INTRINSIC_NAMED_ELEMENT = 1 << 0;

        const INTRINSIC_INDEXED_ELEMENT = 1 << 1;
    }
}

pub fn parse_isolated_entity_name(_name: &str) -> Option<Arc<Node>> {
    None
}

pub fn mark_as_synthetic(node: &Arc<Node>) -> bool {
    let _ = node;
    false
}
