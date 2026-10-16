#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, *};
use crate::mig::m4g::ClassFieldsTransformer;
use crate::mig::m4g_2::r37k13_defs::ClassFieldsTransformerR37k13;
use crate::mig::m4q::r33k12_defs::{FindAncestorResult, to_find_ancestor_result};

impl ClassFieldsTransformer {
    pub fn is_reserved_private_name(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_reserved_private_name"); 
        !(is_private_identifier(node)
            && self.emit_context().has_auto_generate_info(node))
            && node.text() == "#constructor"
    }
}

pub fn is_parent_for_idd_diagnostic(node: &Node) -> FindAncestorResult { ::tsox_core::fntrace::enter("is_parent_for_idd_diagnostic"); 
    if is_export_assignment(node) {
        return FindAncestorResult::True;
    }
    if is_statement(node) {
        return FindAncestorResult::Quit;
    }
    to_find_ancestor_result(!is_parenthesized_expression(node) && !is_assertion_expression(node))
}
