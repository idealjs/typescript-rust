#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use crate::mig::m4h_2::is_class_named_evaluation_helper_block;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::node_data_generated::{
    is_binary_expression, is_class_expression, is_class_static_block_declaration,
    is_expression_statement, is_identifier, is_property_declaration, NodeData,
};
use super::m4q::r33k12_defs::{EmitFlags, is_private_identifier_class_element_declaration};
use tsox_checker::checker::mig::wc1b::{class_or_constructor_parameter_is_decorated, is_initialized_property};
use crate::mig::m4l::r33k9_defs::child_is_decorated;
use crate::mig::m4g_2::r37k13_defs::ClassFieldsTransformerR37k13;
use crate::mig::m4n_2::r39k12_defs::R39K12EmitContextMapsExt;
use crate::mig::m4m_5::r38k9_defs::R38K9NodeCastExt;
use tsox_frontend::ast::mig::m3b::members;
use tsox_frontend::ast::mig::m3c;
use tsox_frontend::ast::symbol::NodeSymbolMap;

use crate::mig::m4g::ClassFieldsTransformer;
use crate::printer::EmitContext;

pub fn is_decorated_class_like(node: &Arc<Node>) -> bool {
    class_or_constructor_parameter_is_decorated(false, node)
        || child_is_decorated(false, node, None)
}

pub fn is_anonymous_class_needing_assigned_name(node: &Arc<Node>) -> bool {
    is_class_expression(node)
        && node.name().is_none()
        && is_decorated_class_like(node)
}

impl ClassFieldsTransformer {
    pub(crate) fn get_static_properties_and_class_static_block(
        &self,
        node: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        let mut result = Vec::new();
        for member in members(node) {
            if is_class_static_block_declaration(member)
                || (is_property_declaration(member) && has_static_modifier(member))
            {
                result.push(member.clone());
            }
        }
        result
    }

    pub(crate) fn node_has_transform_private_static_elements_flag(&self, node: &Arc<Node>) -> bool {
        self.emit_context()
            .emit_flags(node)
            .contains(EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS)
    }

    pub(crate) fn is_anonymous_class_needing_assigned_name_worker(
        &self,
        node: &Arc<Node>,
    ) -> bool {
        if is_class_expression(node) && node.name().is_none() {
            let static_properties_or_class_static_blocks =
                self.get_static_properties_and_class_static_block(node);
            if static_properties_or_class_static_blocks
                .iter()
                .any(|n| is_class_named_evaluation_helper_block(&self.emit_context(), n))
            {
                return false;
            }
            let has_transformable_statics = (self.should_transform_private_elements_or_class_static_blocks
                || self.node_has_transform_private_static_elements_flag(node))
                && static_properties_or_class_static_blocks.iter().any(|n| {
                    is_class_static_block_declaration(n)
                        || is_private_identifier_class_element_declaration(n)
                        || (self.should_transform_initializers && is_initialized_property(n))
                });
            return has_transformable_statics;
        }
        false
    }
}

pub fn is_class_this_assignment_block(emit_context: &EmitContext, node: &Arc<Node>) -> bool {
    if is_class_static_block_declaration(node) {
        if let NodeData::ClassStaticBlockDeclaration(static_block) = &node.data {
            if let NodeData::Block(body) = &static_block.body.data {
                let statements = &body.statements.nodes;
                if statements.len() == 1 {
                    let statement = &statements[0];
                    if is_expression_statement(statement) {
                        if let Some(expression) = statement.expression() {
                            if is_assignment_expression(expression, true) {
                                let binary = expression.as_binary_expression();
                                let class_this = emit_context.r39k12_class_this_get(node);
                                return is_identifier(&binary.left)
                                    && class_this
                                        .as_ref()
                                        .is_some_and(|c| Arc::ptr_eq(c, &binary.left))
                                    && binary.right.kind == SyntaxKind::ThisKeyword;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn is_common_js_alias_export(node: &Arc<Node>, map: &NodeSymbolMap) -> bool {
    if is_binary_expression(node) {
        let binary = node.as_binary_expression();
        if is_identifier(&binary.right) {
            if let Some(symbol) = m3c::symbol(node, map) {
                if symbol.declarations.len() == 1 {
                    return true;
                }
            }
        }
    }
    false
}
