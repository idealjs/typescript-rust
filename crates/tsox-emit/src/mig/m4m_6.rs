#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::Arc;

use crate::mig::m4h_4::r36k9_defs::cloned_node_list;
use tsox_frontend::ast::mig::w3::get_all_accessor_declarations;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{
    is_binary_expression, is_conditional_expression, is_identifier,
    is_numeric_literal, is_parenthesized_expression, is_property_access_expression,
    is_string_literal, is_type_of_expression, is_void_expression, NodeList,
};
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

use crate::mig::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::mig::m4m::r36k5_defs::NodeDataExt;
use crate::mig::m4m::MetadataSerializer;
use crate::mig::m4m_2::is_generated_identifier;
use crate::printer::NodeFactory;

impl MetadataSerializer {
    pub(crate) fn create_checked_value(
        &mut self,
        left: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_checked_value"); 
        Some(self.factory().new_logical_and_expression(
            &self.factory().new_strict_inequality_expression(
                &self.factory().new_type_of_expression(left),
                &self.factory().new_string_literal("undefined", TOKEN_FLAGS_NONE),
            ),
            right,
        ))
    }

    pub(crate) fn equate_serialized_type_nodes(&self, left: &Arc<Node>, right: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("equate_serialized_type_nodes"); 
        if is_generated_identifier(&self.emit_context(), left) {
            return is_generated_identifier(&self.emit_context(), right);
        }
        if is_identifier(left) {
            return is_identifier(right) && left.text() == right.text();
        }
        if is_property_access_expression(left) {
            return is_property_access_expression(right)
                && left
                    .expression()
                    .zip(right.expression())
                    .map_or(false, |(l, r)| self.equate_serialized_type_nodes(l, r))
                && left
                    .name()
                    .zip(right.name())
                    .map_or(false, |(l, r)| self.equate_serialized_type_nodes(l, r));
        }
        if is_void_expression(left) {
            return is_void_expression(right)
                && left
                    .expression()
                    .zip(right.expression())
                    .map_or(false, |(l, r)| {
                        is_numeric_literal(l)
                            && is_numeric_literal(r)
                            && l.text() == "0"
                            && r.text() == "0"
                    });
        }
        if is_string_literal(left) {
            return is_string_literal(right) && left.text() == right.text();
        }
        if is_type_of_expression(left) {
            return is_type_of_expression(right)
                && left
                    .expression()
                    .zip(right.expression())
                    .map_or(false, |(l, r)| self.equate_serialized_type_nodes(l, r));
        }
        if is_parenthesized_expression(left) {
            return is_parenthesized_expression(right)
                && left
                    .expression()
                    .zip(right.expression())
                    .map_or(false, |(l, r)| self.equate_serialized_type_nodes(l, r));
        }
        if is_conditional_expression(left) {
            if !is_conditional_expression(right) {
                return false;
            }
            let l = left.as_conditional_expression();
            let r = right.as_conditional_expression();
            return self.equate_serialized_type_nodes(&l.condition, &r.condition)
                && self.equate_serialized_type_nodes(&l.when_true, &r.when_true)
                && self.equate_serialized_type_nodes(&l.when_false, &r.when_false);
        }
        if is_binary_expression(left) {
            if !is_binary_expression(right) {
                return false;
            }
            let l = left.as_binary_expression();
            let r = right.as_binary_expression();
            return l.operator_token.kind == r.operator_token.kind
                && self.equate_serialized_type_nodes(&l.left, &r.left)
                && self.equate_serialized_type_nodes(&l.right, &r.right);
        }
        false
    }
}

pub fn get_parameters_of_decorated_declaration(
    node: &Arc<Node>,
    container: Option<&Arc<Node>>,
) -> NodeList { ::tsox_core::fntrace::enter("get_parameters_of_decorated_declaration"); 
    if let Some(container) = container {
        if node.kind == SyntaxKind::GetAccessor {
            let acc =
                get_all_accessor_declarations(tsox_frontend::ast::mig::m3b::members(container), node);
            if let Some(set_accessor) = acc.set_accessor {
                if let NodeData::SetAccessorDeclaration(d) = &set_accessor.data {
                    return cloned_node_list(&d.parameters);
                }
                if let NodeData::GetAccessorDeclaration(d) = &set_accessor.data {
                    return cloned_node_list(&d.parameters);
                }
            }
        }
    }
    tsox_frontend::ast::mig::m3b::parameter_list(node)
        .map(|l| cloned_node_list(l))
        .unwrap_or_default()
}
