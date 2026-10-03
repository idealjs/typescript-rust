#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::mig::m3g_2::is_super_property;
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_binary_expression, is_computed_property_name, is_identifier, is_parenthesized_expression,
    is_private_identifier, is_property_access_expression,
};
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::utilities::{
    is_assignment_expression, is_compound_assignment, node_is_synthesized,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{deep_clone_node, NodeList};
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::printer::NodeFactory;

use super::m4f_3::{ClassFieldsTransformer, PrivateIdentifierInfo};
use super::m4g_2::r37k13_defs::{
    node_members_r37k13, EF_NO_NESTED_SOURCE_MAPS_R37K13, M4f3ClassFieldsTransformerR37k13,
};
use super::wt1b::r39k05_defs::R39k05NodeVisitorExt;
use super::m4g::r33k7_defs::{
    binary_left, binary_right, computed_property_name_expression, PrivateIdentifierKind,
};
use super::m4j_2::extract_modifiers;
use super::m4m_2::is_simple_inlineable_expression;
use super::m4m_4::get_non_assignment_operator_for_compound_assignment;
use super::w11b::should_be_captured_in_temp_variable;
use super::x6a::is_class_this_assignment_block;

fn with_loc(mut node: Arc<Node>, loc: TextRange) -> Arc<Node> { ::tsox_core::fntrace::enter("with_loc"); 
    if let Some(n) = Arc::get_mut(&mut node) {
        n.loc = loc;
    }
    node
}

fn binary_operator_token_kind(node: &Node) -> SyntaxKind { ::tsox_core::fntrace::enter("binary_operator_token_kind"); 
    match &node.data {
        tsox_frontend::ast::node_data_generated::NodeData::BinaryExpression(d) => d.operator_token.kind,
        _ => panic!("expected BinaryExpression"),
    }
}

impl ClassFieldsTransformer<'_> {
    pub fn extract_non_static_non_accessor_modifiers(&self, node: &Arc<Node>) -> Option<NodeList> { ::tsox_core::fntrace::enter("extract_non_static_non_accessor_modifiers"); 
        let modifiers = node.modifiers()?;
        let nodes: Vec<Arc<Node>> = modifiers
            .list
            .nodes
            .iter()
            .filter(|n| {
                n.kind != SyntaxKind::StaticKeyword && n.kind != SyntaxKind::AccessorKeyword
            })
            .cloned()
            .collect();
        Some(NodeList::new(nodes))
    }

    pub fn clear_class_element_and_visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("clear_class_element_and_visit_each_child"); 
        self.set_current_class_element_and(None, node)
    }

    fn set_current_class_element_and(
        &mut self,
        class_element: Option<Arc<Node>>,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("set_current_class_element_and"); 
        let same = match (&class_element, &self.current_class_element) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if !same {
            let saved = self.current_class_element.take();
            self.current_class_element = class_element;
            let result = self.visitor().visit_each_child(node);
            self.current_class_element = saved;
            result
        } else {
            self.visitor().visit_each_child(node)
        }
    }

    pub fn create_call_binding(&mut self, node: &Arc<Node>) -> (Arc<Node>, Arc<Node>) { ::tsox_core::fntrace::enter("create_call_binding"); 
        if is_super_property(node) {
            return (self.factory().new_this_expression(), node.clone());
        }
        if is_property_access_expression(node) {
            let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
            if should_be_captured_in_temp_variable(&expression) {
                let this_arg = self
                    .factory()
                    .generated_name_node(&self.factory().new_temp_variable());
                self.emit_context.add_variable_declaration(&this_arg);
                let target = self.factory().new_property_access_expression(
                    &self.factory().new_parenthesized_expression(
                        &self.factory()
                            .new_assignment_expression(&this_arg, &expression),
                    ),
                    None,
                    node.name().unwrap(),
                    node.flags,
                );
                return (this_arg, target);
            }
            return (
                expression.clone(),
                self.factory().new_property_access_expression(
                    &expression,
                    None,
                    node.name().unwrap(),
                    node.flags,
                ),
            );
        }
        (self.factory().new_this_expression(), node.clone())
    }

    pub fn create_copiable_receiver_expr(&mut self, receiver: &Arc<Node>) -> (Arc<Node>, Option<Arc<Node>>) { ::tsox_core::fntrace::enter("create_copiable_receiver_expr"); 
        let mut clone = receiver.clone();
        if !node_is_synthesized(receiver) {
            clone = deep_clone_node(receiver);
        }
        if is_simple_inlineable_expression(receiver) {
            return (clone, None);
        }
        let read_expression = self
            .factory()
            .generated_name_node(&self.factory().new_temp_variable());
        self.emit_context.add_variable_declaration(&read_expression);
        let initialize_expression = self
            .factory()
            .new_assignment_expression(&read_expression, &clone);
        (read_expression, Some(initialize_expression))
    }

    pub fn create_member_access_for_property_name(
        &mut self,
        receiver: &Arc<Node>,
        name: &Arc<Node>,
        location: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_member_access_for_property_name"); 
        if is_computed_property_name(name) {
            let expression = self.factory().new_element_access_expression(
                receiver,
                None,
                computed_property_name_expression(name),
                NodeFlags::empty(),
            );
            let loc = location.map(|l| l.loc.clone()).unwrap_or_else(|| name.loc.clone());
            return with_loc(expression, loc);
        }
        let expression = if is_identifier(name) || is_private_identifier(name) {
            self.factory().new_property_access_expression(receiver, None, name, NodeFlags::empty())
        } else {
            self.factory().new_element_access_expression(receiver, None, name, NodeFlags::empty())
        };
        self.emit_context.set_comment_range(&expression, name.loc.clone());
        self.emit_context.set_source_map_range(&expression, name.loc.clone());
        self.emit_context()
            .add_emit_flags(&expression, EF_NO_NESTED_SOURCE_MAPS_R37K13);
        expression
    }

    pub fn create_private_identifier_access(&mut self, info: &PrivateIdentifierInfo, receiver: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_identifier_access"); 
        let receiver = self.visitor().visit_node(receiver);
        self.create_private_identifier_access_helper(info, &receiver)
    }

    pub fn create_private_identifier_access_helper(
        &mut self,
        info: &PrivateIdentifierInfo,
        receiver: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_identifier_access_helper"); 
        self.emit_context
            .set_comment_range(receiver, TextRange::new(0, receiver.end()));
        match info.kind {
            PrivateIdentifierKind::Accessor => self.factory().new_class_private_field_get_helper(
                &receiver,
                &info.brand_check_identifier.clone().unwrap(),
                info.kind,
                info.getter_name.as_ref(),
            ),
            PrivateIdentifierKind::Method => self.factory().new_class_private_field_get_helper(
                &receiver,
                &info.brand_check_identifier.clone().unwrap(),
                info.kind,
                info.method_name.as_ref(),
            ),
            PrivateIdentifierKind::Field => {
                let f = if info.is_static {
                    info.variable_name.clone()
                } else {
                    None
                };
                self.factory().new_class_private_field_get_helper(
                    &receiver,
                    &info.brand_check_identifier.clone().unwrap(),
                    info.kind,
                    f.as_ref(),
                )
            }
            PrivateIdentifierKind::Untransformed => {
                debug_assert!(false, "Access helpers should not be created for untransformed private elements");
                self.factory().new_identifier("")
            }
        }
    }

    pub fn create_private_identifier_assignment(
        &mut self,
        info: &PrivateIdentifierInfo,
        receiver: &Arc<Node>,
        right: &Arc<Node>,
        operator: SyntaxKind,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_identifier_assignment"); 
        let mut receiver = self.visitor().visit_node(receiver);
        let mut right = self.visitor().visit_node(right);

        if is_compound_assignment(operator) {
            let (read_expression, initialize_expression) = self.create_copiable_receiver_expr(&receiver);
            receiver = match initialize_expression {
                Some(initialize_expression) => initialize_expression,
                None => read_expression.clone(),
            };
            let access_expression = self.create_private_identifier_access_helper(info, &read_expression);
            right = self.factory().new_binary_expression(
                None,
                &access_expression,
                None,
                &self.factory()
                    .new_token(get_non_assignment_operator_for_compound_assignment(operator)),
                &right,
            );
        }

        self.emit_context
            .set_comment_range(&receiver, TextRange::new(0, receiver.end()));

        match info.kind {
            PrivateIdentifierKind::Accessor => self.factory().new_class_private_field_set_helper(
                &receiver,
                &info.brand_check_identifier.clone().unwrap(),
                &right,
                info.kind,
                info.setter_name.as_ref(),
            ),
            _ => self.factory().new_class_private_field_set_helper(
                &receiver,
                &info.brand_check_identifier.clone().unwrap(),
                &right,
                info.kind,
                info.setter_name.as_ref(),
            ),
        }
    }

    pub fn class_contains_constructor_reference(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("class_contains_constructor_reference"); 
        node_members_r37k13(node)
            .iter()
            .any(|member| self.member_contains_constructor_reference(member, node))
    }

    pub fn create_private_instance_field_initializer(
        &self,
        receiver: &Arc<Node>,
        initializer: Option<Arc<Node>>,
        weak_map_name: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_instance_field_initializer"); 
        let initializer = initializer.unwrap_or_else(|| self.factory().new_void_zero_expression());
        self.factory()
            .new_method_call(weak_map_name, &self.factory().new_identifier("set"), vec![receiver.clone(), initializer])
    }

    pub fn create_private_instance_method_initializer(
        &self,
        receiver: &Arc<Node>,
        weak_set_name: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_instance_method_initializer"); 
        self.factory()
            .new_method_call(weak_set_name, &self.factory().new_identifier("add"), vec![receiver.clone()])
    }

    pub fn create_private_static_field_initializer(
        &self,
        variable_name: &Arc<Node>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_static_field_initializer"); 
        let initializer = initializer.unwrap_or_else(|| self.factory().new_void_zero_expression());
        self.factory().new_assignment_expression(
            variable_name,
            &self.factory().new_object_literal_expression(
                &self.factory().new_node_list(vec![self.factory().new_property_assignment(
                    None,
                    &self.factory().new_identifier("value"),
                    None,
                    None,
                    &initializer,
                )]),
                false,
            ),
        )
    }

    pub fn find_computed_property_name_cache_assignment(&self, name: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_computed_property_name_cache_assignment"); 
        let mut node = name.expression().cloned().unwrap_or_else(|| Arc::clone(name));
        loop {
            node = skip_outer_expressions(&node, OuterExpressionKinds::empty());
            if is_binary_expression(&node) && binary_operator_token_kind(&node) == SyntaxKind::CommaToken {
                node = binary_left(&node).clone();
                continue;
            }
            if is_assignment_expression(&node, true) && is_identifier(binary_left(&node)) {
                return Some(node);
            }
            break;
        }
        None
    }
}

pub fn class_has_class_this_assignment(emit_context: &crate::printer::EmitContext, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("class_has_class_this_assignment"); 
    node_members_r37k13(node)
        .iter()
        .any(|member| is_class_this_assignment_block(emit_context, member))
}

pub fn flatten_comma_list(node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("flatten_comma_list"); 
    let mut result = Vec::new();
    flatten_comma_list_worker(node, &mut result);
    result
}

fn flatten_comma_list_worker(node: &Arc<Node>, out: &mut Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("flatten_comma_list_worker"); 
    if is_parenthesized_expression(node) && node_is_synthesized(node) {
        if let Some(expression) = node.expression() {
            flatten_comma_list_worker(&expression, out);
        }
    } else if is_binary_expression(node) && binary_operator_token_kind(node) == SyntaxKind::CommaToken {
        flatten_comma_list_worker(&binary_left(node), out);
        flatten_comma_list_worker(&binary_right(node), out);
    } else {
        out.push(node.clone());
    }
}
