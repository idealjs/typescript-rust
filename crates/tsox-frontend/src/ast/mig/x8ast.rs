#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{self, Node, SyntaxKind};


const INTERNAL_SYMBOL_NAME_DEFAULT: &str = "default";

pub fn module_export_name_is_default(node: &Node) -> bool { ::tsox_core::fntrace::enter("module_export_name_is_default"); 
    node.text() == INTERNAL_SYMBOL_NAME_DEFAULT
}

fn is_argument_of_element_access_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_argument_of_element_access_expression"); 
    node.parent()
        .map(|parent| {
            parent.kind == SyntaxKind::ElementAccessExpression
                && match &parent.data {
                    NodeData::ElementAccessExpression(d) => Arc::ptr_eq(&d.argument_expression, node),
                    _ => false,
                }
        })
        .unwrap_or(false)
}

pub fn literal_is_name(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("literal_is_name"); 
    crate::ast::mig::m3f_4::is_declaration_name(node)
        || node
            .parent()
            .map(|p| p.kind == SyntaxKind::ExternalModuleReference)
            .unwrap_or(false)
        || is_argument_of_element_access_expression(node)
        || crate::ast::mig::m3g::is_literal_computed_property_declaration_name(node)
}

pub fn needs_parenthesized_expression_for_assertion(node: &Node) -> bool { ::tsox_core::fntrace::enter("needs_parenthesized_expression_for_assertion"); 
    !ast::is_entity_name_expression(node)
        && !ast::is_call_expression(node)
        && !ast::is_object_literal_expression(node)
        && !ast::is_array_literal_expression(node)
}
