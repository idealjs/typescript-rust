#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;
use tsox_frontend::ast::Node;
use crate::mig::m4i_3::ObjectRestSpreadTransformer;
use crate::mig::m4k_2::Transformer;

fn r39k09_identity_visit(_tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> {
    Some(node)
}

pub trait R39K09ObjectRestSpreadTxExt {
    fn flatten_tx(&mut self) -> Transformer;
}

impl R39K09ObjectRestSpreadTxExt for ObjectRestSpreadTransformer {
    fn flatten_tx(&mut self) -> Transformer {
        Transformer::new(r39k09_identity_visit, Some(self.emit_context.clone()))
    }
}
