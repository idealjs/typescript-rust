#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_computed_property_name, is_get_accessor_declaration, is_set_accessor_declaration,
    is_private_identifier,
};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::has_static_modifier;
use tsox_frontend::ast::mig::m3f_2::node_initializer;
use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
use tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration;
use tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags;
use tsox_frontend::ast::ModifierList;

use super::ClassFieldsTransformer;
use super::m4f_4::visit_modifiers_list_m4f4;
use super::m4f_7::{
    create_accessor_property_get_redirector_m4f5, create_accessor_property_set_redirector_m4f5,
};
use crate::mig::m4f_3::r38k10_defs::R38K10NodeVisitorExt;
use crate::mig::m4g::r33k7_defs::{
    binary_left, computed_property_name_expression, has_decorators, node_body,
    node_body_data_asterisk_token, node_parameter_list, PrivateIdentifierKind,
};
use crate::mig::m4i_12::create_accessor_property_backing_field;
use crate::mig::m4j_2::extract_modifiers;
use crate::mig::m4m_2::is_simple_inlineable_expression;
use crate::printer::{AutoGenerateOptions, NodeFactory};

impl ClassFieldsTransformer<'_> {
    pub fn visit_method_or_accessor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_method_or_accessor_declaration"); 
        debug_assert!(!has_decorators(node));

        if !is_private_identifier_class_element_declaration(node)
            || !self.should_transform_class_element_to_weak_map(node)
        {
            return self
                .class_element_visitor
                .as_mut()
                .unwrap()
                .visit_each_child(node);
        }

        let name = node.name().expect("method or accessor requires a name");
        let info = self
            .access_private_identifier(name)
            .expect("Undeclared private name for property declaration.");
        if !info.is_valid {
            return Some(Arc::clone(node));
        }

        if let Some(function_name) = self.get_hoisted_function_name(node) {
            let modifiers = self.extract_modifiers_excluding_static_and_accessor(node);
            self.emit_context.start_variable_environment();
            let saved = self.in_iteration_statement;
            self.in_iteration_statement = false;
            let visitor = self.substitution_visitor.as_mut().unwrap();
            let body = self
                .emit_context
                .visit_function_body(node_body(node).cloned(), visitor);
            let params = self.visitor().visit_nodes(node_parameter_list(node));
            self.in_iteration_statement = saved;

            if let Some(body) = body {
                let func_expr = self.factory().new_function_expression(
                    modifiers,
                    node_body_data_asterisk_token(node),
                    Some(&function_name),
                    None,
                    &params,
                    None,
                    None,
                    &body,
                );
                let assignment = self
                    .factory()
                    .new_assignment_expression(&function_name, &func_expr);
                self.pending_expressions.push(assignment);
            }
        }

        None
    }

    pub fn get_hoisted_function_name(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_hoisted_function_name"); 
        let name = node.name();
        debug_assert!(name.is_some() && is_private_identifier(name.unwrap()));
        let info = self.access_private_identifier(name?)?;
        match info.kind {
            PrivateIdentifierKind::Method => info.method_name,
            PrivateIdentifierKind::Accessor => {
                if is_get_accessor_declaration(node) {
                    info.getter_name
                } else if is_set_accessor_declaration(node) {
                    info.setter_name
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn extract_modifiers_excluding_static_and_accessor(
        &self,
        node: &Arc<Node>,
    ) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("extract_modifiers_excluding_static_and_accessor"); 
        let ec = Arc::new(self.emit_context.clone());
        extract_modifiers(
            &ec,
            node.modifiers().map(|m| m.as_ref()),
            !(ModifierFlags::Static | ModifierFlags::Accessor),
        )
        .map(Arc::new)
    }

    pub fn visit_property_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_property_declaration"); 
        if is_auto_accessor_property_declaration(node)
            && (self.should_transform_auto_accessors_in_current_class()
                || has_static_modifier(node)
                    && self.should_always_transform_private_static_elements(node))
        {
            return self.transform_auto_accessor(node);
        }
        self.transform_field_initializer(node)
    }

    pub fn transform_auto_accessor(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_auto_accessor"); 
        let comment_range = self.emit_context.comment_range(node);
        let source_map_range = self.emit_context.source_map_range(node);

        let name = node
            .name()
            .expect("property declaration requires a name")
            .clone();
        let mut getter_name = Arc::clone(&name);
        let mut setter_name = Arc::clone(&name);
        if is_computed_property_name(&name)
            && !is_simple_inlineable_expression(computed_property_name_expression(&name))
        {
            let expression = computed_property_name_expression(&name).clone();
            match self.find_computed_property_name_cache_assignment(&name) {
                Some(cache_assignment) => {
                    let visited = self.visitor().visit_node(&expression);
                    getter_name = self.factory().update_computed_property_name(&name, &visited);
                    setter_name = self
                        .factory()
                        .update_computed_property_name(&name, binary_left(&cache_assignment));
                }
                None => {
                    let temp = self.factory().generated_name_node(
                        &self.factory().new_temp_variable_ex(AutoGenerateOptions::default()),
                    );
                    self.emit_context.set_source_map_range(&temp, expression.loc);
                    self.emit_context.add_variable_declaration(&temp);
                    let visited = self.visitor().visit_node(&expression);
                    let assignment = self.factory().new_assignment_expression(&temp, &visited);
                    self.emit_context
                        .set_source_map_range(&assignment, expression.loc);
                    getter_name = self.factory().update_computed_property_name(&name, &assignment);
                    setter_name = self.factory().update_computed_property_name(&name, &temp);
                }
            }
        }

        let modifiers = visit_modifiers_list_m4f4(
            self.modifier_visitor.as_mut().unwrap(),
            node.modifiers(),
        );
        let initializer = node_initializer(node).cloned();
        let backing_field = create_accessor_property_backing_field(
            &self.factory(),
            node,
            modifiers.clone(),
            initializer,
        );
        self.emit_context.set_original(&backing_field, node);
        self.emit_context
            .add_emit_flags(&backing_field, EmitFlags::NO_COMMENTS);
        self.emit_context
            .set_source_map_range(&backing_field, source_map_range);

        let receiver: Arc<Node> = if has_static_modifier(node) {
            match self.try_get_class_this() {
                Some(class_this) => class_this,
                None => self.factory().new_this_expression(),
            }
        } else {
            self.factory().new_this_expression()
        };

        let getter = create_accessor_property_get_redirector_m4f5(
            &self.factory(),
            node,
            modifiers.clone(),
            &getter_name,
            &receiver,
        );
        self.emit_context.set_original(&getter, node);
        self.emit_context.set_comment_range(&getter, comment_range);
        self.emit_context
            .set_source_map_range(&getter, source_map_range);

        let setter_modifiers = modifiers.as_ref().map(|mods| {
            let factory = self.factory();
            factory.new_modifier_list(create_modifiers_from_modifier_flags(
                mods.modifier_flags,
                |kind| factory.new_modifier(kind),
            ))
        });
        let setter = create_accessor_property_set_redirector_m4f5(
            &self.factory(),
            node,
            setter_modifiers,
            &setter_name,
            &receiver,
        );
        self.emit_context.set_original(&setter, node);
        self.emit_context
            .add_emit_flags(&setter, EmitFlags::NO_COMMENTS);
        self.emit_context
            .set_source_map_range(&setter, source_map_range);

        let backing_field_visited = self
            .accessor_field_result_visitor
            .as_mut()
            .unwrap()
            .visit_each_child(&backing_field)
            .unwrap_or_else(|| backing_field.clone());
        let getter_visited = self
            .accessor_field_result_visitor
            .as_mut()
            .unwrap()
            .visit_each_child(&getter)
            .unwrap_or_else(|| getter.clone());
        let setter_visited = self
            .accessor_field_result_visitor
            .as_mut()
            .unwrap()
            .visit_each_child(&setter)
            .unwrap_or_else(|| setter.clone());
        Some(self.factory().new_syntax_list(vec![
            backing_field_visited,
            getter_visited,
            setter_visited,
        ]))
    }
}
