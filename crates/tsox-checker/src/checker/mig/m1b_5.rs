#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::NodeFlags;

impl Checker {
    pub fn check_property_access_chain(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_property_access_chain"); 
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let left_type = self.check_expression_ex(&expression, CheckMode::Normal);
        let non_optional_type = self.get_optional_expression_type(&left_type, &expression);
        let non_null = self.check_non_null_type(&non_optional_type, &expression);
        let name = node.name().cloned().unwrap_or_else(|| Arc::clone(node));
        let checked = self.check_property_access_expression_or_qualified_name(
            node,
            &expression,
            &non_null,
            &name,
            check_mode,
            false,
        );
        self.propagate_optional_type_marker(&checked, node, !Arc::ptr_eq(&non_optional_type, &left_type))
    }

    pub fn check_property_access_expression(&mut self, node: &Arc<Node>, check_mode: CheckMode, write_only: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("check_property_access_expression"); 
        if node.flags.contains(NodeFlags::OptionalChain) {
            return self.check_property_access_chain(node, check_mode);
        }
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let left_type = self.check_non_null_expression(&expression);
        let name = node.name().cloned().unwrap_or_else(|| Arc::clone(&expression));
        self.check_property_access_expression_or_qualified_name(node, &expression, &left_type, &name, check_mode, write_only)
    }
}
