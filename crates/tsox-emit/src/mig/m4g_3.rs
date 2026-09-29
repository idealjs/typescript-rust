#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::{Node, node_modifiers};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated::is_identifier;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::{is_private_identifier, node_name};
use tsox_frontend::ast::mig::m3f_2::node_initializer;
use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
use tsox_frontend::ast::mig::m3g_2::{is_private_identifier_class_element_declaration, is_super_property};
use tsox_frontend::format::mig::m4o::EmitFlags;
use crate::mig::m4g::{ClassFieldsTransformer, ClassFacts, PrivateIdentifierInfo};
use crate::mig::m4g::r33k7_defs::{
    binary_left, class_members, computed_property_name_expression, element_access_argument,
    has_decorators, node_body, node_body_data_asterisk_token, node_parameter_list,
    property_access_expression, property_access_name, property_access_question_dot_token,
};
use crate::mig::m4g_2::r37k13_defs::{ClassFieldsTransformerR37k13, EmitContextR37k13};
use crate::mig::m4g::r39k15_defs::{
    k15_m3c_visitor, ClassFieldsTransformerR39k15, K13VisitorR39k15, NodeFactoryR39k15,
};
use crate::mig::m4h_2::{is_named_evaluation_and, transform_named_evaluation};
use crate::mig::m4i_12::create_accessor_property_backing_field;
use crate::mig::m4m_2::is_simple_inlineable_expression;
use crate::mig::wt1b::r39k05_defs::new_get_accessor_declaration_full_r39k05;
use crate::mig::wt1b_4::flatten_comma_list;
use crate::mig::w7t::is_static_property_declaration_or_class_static_block;
use crate::printer::{AutoGenerateOptions, NodeFactory};
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags;
use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::utilities::is_assignment_expression;
use tsox_frontend::ast::is_computed_property_name;

fn with_loc(mut node: Arc<Node>, loc: TextRange) -> Arc<Node> {
    if let Some(n) = Arc::get_mut(&mut node) {
        n.loc = loc;
    }
    node
}

fn create_accessor_property_get_redirector_m4g3(
    factory: &NodeFactory<'_>,
    node: &Arc<Node>,
    modifiers: Option<Arc<ModifierList>>,
    name: &Arc<Node>,
    receiver: &Arc<Node>,
) -> Arc<Node> {
    let backing_field_name = factory.generated_name_node(
        &factory.new_generated_private_name_for_node_ex(
            node_name(node).expect("property declaration requires a name"),
            AutoGenerateOptions {
                suffix: "_accessor_storage".to_string(),
                ..Default::default()
            },
        ),
    );
    let return_expression = factory.new_property_access_expression(
        receiver,
        None,
        &backing_field_name,
        NodeFlags::empty(),
    );
    let return_statement = factory.new_return_statement(Some(&return_expression));
    let body = factory.new_block(&factory.new_node_list(vec![return_statement]), false);
    let parameters = factory.new_node_list(vec![]);
    new_get_accessor_declaration_full_r39k05(
        factory,
        modifiers,
        name,
        None,
        parameters,
        None,
        None,
        body,
    )
}

fn create_accessor_property_set_redirector_m4g3(
    factory: &NodeFactory<'_>,
    node: &Arc<Node>,
    modifiers: Option<Arc<ModifierList>>,
    name: &Arc<Node>,
    receiver: &Arc<Node>,
) -> Arc<Node> {
    let backing_field_name = factory.generated_name_node(
        &factory.new_generated_private_name_for_node_ex(
            node_name(node).expect("property declaration requires a name"),
            AutoGenerateOptions {
                suffix: "_accessor_storage".to_string(),
                ..Default::default()
            },
        ),
    );
    let value_param = factory.new_parameter_declaration(
        None,
        None,
        &factory.new_identifier("value"),
        None,
        None,
        None,
    );
    let assignment = factory.new_assignment_expression(
        &factory.new_property_access_expression(
            receiver,
            None,
            &backing_field_name,
            NodeFlags::empty(),
        ),
        &factory.new_identifier("value"),
    );
    let expression_statement = factory.new_expression_statement(&assignment);
    let body = factory.new_block(
        &factory.new_node_list(vec![expression_statement]),
        false,
    );
    let parameters = factory.new_node_list(vec![value_param]);
    factory.new_set_accessor_declaration(modifiers, name, None, parameters, None, None, body)
}

fn find_computed_property_name_cache_assignment_m4g3(name: &Arc<Node>) -> Option<Arc<Node>> {
    let mut node = Arc::clone(name.expression().expect("computed property name requires expression"));
    loop {
        node = skip_outer_expressions(&node, OuterExpressionKinds::empty());
        if node.kind == SyntaxKind::BinaryExpression
            && binary_operator_token_kind_m4g3(&node) == SyntaxKind::CommaToken
        {
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

fn binary_operator_token_kind_m4g3(node: &Node) -> SyntaxKind {
    match &node.data {
        NodeData::BinaryExpression(d) => d.operator_token.kind,
        _ => panic!("expected BinaryExpression"),
    }
}

impl ClassFieldsTransformer {
    pub fn visit_method_or_accessor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        debug_assert!(!has_decorators(node));

        if !is_private_identifier_class_element_declaration(node)
            || !self.should_transform_class_element_to_weak_map(node)
        {
            return Some(self.class_element_visitor().visit_each_child(node));
        }

        let info = self.access_private_identifier(node_name(node).unwrap());
        let info = info.expect("Undeclared private name for property declaration.");
        if !info.is_valid {
            return Some(node.clone());
        }

        if let Some(function_name) = self.get_hoisted_function_name(node) {
            let modifiers = self.extract_non_static_non_accessor_modifiers(node);
            self.emit_context().start_variable_environment();
            let saved = self.in_iteration_statement;
            self.in_iteration_statement = false;
            let body = self.emit_context().visit_function_body(
                node_body(node).cloned(),
                &mut tsox_frontend::ast::visitor::NodeVisitor::default(),
            );
            let params = self
                .visitor()
                .visit_nodes_list(node_parameter_list(node));
            self.in_iteration_statement = saved;

            let func_expr = self.factory().new_function_expression(
                modifiers,
                node_body_data_asterisk_token(node).cloned(),
                Some(function_name.clone()),
                None,
                params,
                None,
                None,
                body,
            );
            let assignment = self
                .factory()
                .new_assignment_expression(&function_name, &func_expr);
            self.add_pending_expressions(vec![assignment]);
        }

        None
    }

    pub fn visit_function_expression_or_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if self.current_class_element.is_some() {
            let original = self.emit_context().most_original(node);
            if !Arc::ptr_eq(&original, node) && self.current_class_container.is_some() {
                for member in class_members(self.current_class_container.as_ref().unwrap()) {
                    let member_original = self.emit_context().most_original(member);
                    if Arc::ptr_eq(&member_original, &original) && is_static(member) {
                        return self.visit_each_child_of_node(node);
                    }
                }
            }
        }
        let saved = self.current_class_element.take();
        let result = self.visit_each_child_of_node(node);
        self.current_class_element = saved;
        result
    }

    pub fn visit_property_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_auto_accessor_property_declaration(node)
            && (self.should_transform_auto_accessors_in_current_class()
                || has_static_modifier(node)
                    && self.should_always_transform_private_static_elements(node))
        {
            return self.transform_auto_accessor(node);
        }
        self.transform_field_initializer(node)
    }

    pub fn transform_auto_accessor(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let emit_context = self.emit_context();
        let comment_range = emit_context.comment_range(node);
        let source_map_range = emit_context.source_map_range(node);
        let factory = NodeFactory::new(&emit_context);

        let name = node_name(node)
            .expect("property declaration requires a name")
            .clone();
        let mut getter_name = name.clone();
        let mut setter_name = name.clone();
        if is_computed_property_name(&name)
            && !is_simple_inlineable_expression(computed_property_name_expression(&name))
        {
            let expression = computed_property_name_expression(&name).clone();
            match find_computed_property_name_cache_assignment_m4g3(&name) {
                Some(cache_assignment) => {
                    let visited = self.visitor().visit_node(&expression);
                    getter_name = factory.update_computed_property_name(&name, &visited);
                    setter_name =
                        factory.update_computed_property_name(&name, binary_left(&cache_assignment));
                }
                None => {
                    let temp = factory
                        .generated_name_node(&factory.new_temp_variable_ex(AutoGenerateOptions::default()));
                    self.emit_context().set_source_map_range(&temp, expression.loc);
                    self.emit_context().add_variable_declaration(&temp);
                    let visited = self.visitor().visit_node(&expression);
                    let assignment = factory.new_assignment_expression(&temp, &visited);
                    self.emit_context()
                        .set_source_map_range(&assignment, expression.loc);
                    getter_name = factory.update_computed_property_name(&name, &assignment);
                    setter_name = factory.update_computed_property_name(&name, &temp);
                }
            }
        }

        let modifiers = self
            .modifier_visitor()
            .visit_modifiers_list(node_modifiers(node));
        let initializer = node_initializer(node).cloned();
        let backing_field = create_accessor_property_backing_field(
            &factory,
            node,
            modifiers.clone(),
            initializer,
        );
        self.emit_context().set_original(&backing_field, node);
        self.emit_context()
            .add_emit_flags(&backing_field, EmitFlags::NO_COMMENTS);
        self.emit_context()
            .set_source_map_range(&backing_field, source_map_range);

        let receiver: Arc<Node> = if is_static(node) {
            match self.try_get_class_this() {
                Some(class_this) => class_this,
                None => self.factory().new_this_expression(),
            }
        } else {
            self.factory().new_this_expression()
        };

        let getter = create_accessor_property_get_redirector_m4g3(
            &factory,
            node,
            modifiers.clone(),
            &getter_name,
            &receiver,
        );
        self.emit_context().set_original(&getter, node);
        self.emit_context()
            .set_comment_range(&getter, comment_range);
        self.emit_context()
            .set_source_map_range(&getter, source_map_range);

        let setter_modifiers = modifiers.as_ref().map(|modifiers| {
            factory.new_modifier_list(create_modifiers_from_modifier_flags(
                modifiers.modifier_flags,
                |kind| factory.new_modifier(kind),
            ))
        });
        let setter = create_accessor_property_set_redirector_m4g3(
            &factory,
            node,
            setter_modifiers,
            &setter_name,
            &receiver,
        );
        self.emit_context().set_original(&setter, node);
        self.emit_context()
            .add_emit_flags(&setter, EmitFlags::NO_COMMENTS);
        self.emit_context()
            .set_source_map_range(&setter, source_map_range);

        let backing_field_visited = self
            .accessor_field_result_visitor()
            .visit_each_child(&backing_field);
        let getter_visited = self
            .accessor_field_result_visitor()
            .visit_each_child(&getter);
        let setter_visited = self
            .accessor_field_result_visitor()
            .visit_each_child(&setter);
        Some(factory.new_syntax_list(vec![
            backing_field_visited,
            getter_visited,
            setter_visited,
        ]))
    }

    pub fn transform_private_field_initializer(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if self.should_transform_class_element_to_weak_map(node) {
            let info = self.access_private_identifier(node_name(node).unwrap());
            let info = info.expect("Undeclared private name for property declaration.");

            if !info.is_valid {
                return Some(node.clone());
            }

            if info.is_static && !self.should_transform_private_elements_or_class_static_blocks {
                let this_expr = self.factory().new_this_expression();
                let statement = self.transform_property_or_class_static_block(node, &this_expr);
                if let Some(statement) = statement {
                    let block = self.factory().new_block(
                        self.factory().new_node_list(&[statement]),
                        true,
                    );
                    return Some(
                        self.factory()
                            .new_class_static_block_declaration(None, &block),
                    );
                }
            }

            return None;
        }

        if self.should_transform_initializers_using_set
            && !has_static_modifier(node)
            && self
                .lexical_environment
                .as_ref()
                .and_then(|env| env.data.as_ref())
                .map(|data| data.facts.contains(ClassFacts::WillHoistInitializersToConstructor))
                .unwrap_or(false)
        {
            return Some(self.factory().update_property_declaration(
                node,
                self.visitor().visit_modifiers_list(node_modifiers(node)),
                node_name(node).cloned(),
                None,
                None,
                None,
            ));
        }

        let mut node = node.clone();
        let this = &*self;
        let cb = |n: &Arc<Node>| (this.is_anonymous_class_needing_assigned_name)(this, n);
        if is_named_evaluation_and(&self.emit_context(), &node, Some(&cb)) {
            node = transform_named_evaluation(&self.emit_context(), &node, false, "");
        }

        Some(self.factory().update_property_declaration(
            &node,
            self.modifier_visitor().visit_modifiers_list(node_modifiers(&node)),
            Some(self.visit_property_name(node_name(&node).unwrap())),
            None,
            None,
            self.visitor().visit_node_opt(node_initializer(&node)),
        ))
    }

    pub fn transform_public_field_initializer(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if self.should_transform_initializers && !is_auto_accessor_property_declaration(node) {
            let initializer_present = node_initializer(node).is_some();
            let expr = self.get_property_name_expression_if_needed(
                node_name(node).unwrap(),
                initializer_present || self.compiler_options.get_use_define_for_class_fields(),
            );
            if let Some(expr) = expr {
                for e in flatten_comma_list(&expr) {
                    self.add_pending_expressions(vec![e]);
                }
            }

            if is_static(node) && !self.should_transform_private_elements_or_class_static_blocks {
                let this_expr = self.factory().new_this_expression();
                let initializer_statement =
                    self.transform_property_or_class_static_block(node, &this_expr);
                if let Some(initializer_statement) = initializer_statement {
                    let block = self.factory().new_block(
                        self.factory().new_node_list(&[initializer_statement.clone()]),
                        false,
                    );
                    let static_block =
                        self.factory().new_class_static_block_declaration(None, &block);
                    self.emit_context().set_original_r37k13(&static_block, node);
                    self.emit_context().set_comment_range(&static_block, node.loc);
                    self.emit_context()
                        .add_emit_flags(&initializer_statement, EmitFlags::NO_COMMENTS);
                    return Some(static_block);
                }
            }

            return None;
        }

        Some(self.factory().update_property_declaration(
            node,
            self.modifier_visitor().visit_modifiers_list(node_modifiers(node)),
            Some(self.visit_property_name(node_name(node).unwrap())),
            None,
            None,
            self.visitor().visit_node_opt(node_initializer(node)),
        ))
    }

    pub fn visit_property_access_expression(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if is_private_identifier(property_access_name(node)) {
            let info = self.access_private_identifier(property_access_name(node));
            if let Some(info) = info {
                let result = self.create_private_identifier_access(
                    &info,
                    property_access_expression(node),
                );
                self.emit_context().set_original_r37k13(&result, node);
                return with_loc(result, node.loc);
            }
        }
        if self.should_transform_super_in_static_initializers
            && self.current_class_element.is_some()
            && is_super_property(node)
            && is_identifier(property_access_name(node))
            && is_static_property_declaration_or_class_static_block(
                self.current_class_element.as_ref().unwrap(),
            )
        {
            if let Some(lex) = &self.lexical_environment {
                if let Some(data) = &lex.data {
                    if data.facts.contains(ClassFacts::ClassWasDecorated) {
                        return self.visit_invalid_super_property(node);
                    }
                    if data.class_constructor.is_some() && data.super_class_reference.is_some() {
                        let super_property = self.factory().new_reflect_get_call(
                            &data.super_class_reference.clone().unwrap(),
                            &self.factory().new_string_literal_from_node(property_access_name(node)),
                            &data.class_constructor.clone().unwrap(),
                        );
                        let expression = property_access_expression(node);
                        self.emit_context().set_original_r37k13(&super_property, expression);
                        return with_loc(super_property, expression.loc);
                    }
                }
            }
        }
        if is_identifier(property_access_name(node)) {
            return self.visit_property_access_expression_for_substitution(node);
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_property_access_expression_for_substitution(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Node> {
        let expression = property_access_expression(node);
        let visited = self.visitor().visit_node(expression);
        if !Arc::ptr_eq(&visited, expression) {
            return self.factory().update_property_access_expression(
                node,
                &visited,
                property_access_question_dot_token(node).cloned(),
                node_name(node).cloned(),
                node.flags,
            );
        }
        node.clone()
    }

    pub fn visit_element_access_expression(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if self.should_transform_super_in_static_initializers
            && self.current_class_element.is_some()
            && is_super_property(node)
            && is_static_property_declaration_or_class_static_block(
                self.current_class_element.as_ref().unwrap(),
            )
        {
            let lex_data = self
                .lexical_environment
                .as_ref()
                .and_then(|lex| lex.data.clone());
            if let Some(data) = &lex_data {
                if data.facts.contains(ClassFacts::ClassWasDecorated) {
                    return self.visit_invalid_super_property(node);
                }
                if data.class_constructor.is_some() && data.super_class_reference.is_some() {
                    let argument = self
                        .visitor()
                        .visit_node(element_access_argument(node).unwrap());
                    let super_property = self.factory().new_reflect_get_call(
                        &data.super_class_reference.clone().unwrap(),
                        &argument,
                        &data.class_constructor.clone().unwrap(),
                    );
                    let expression = property_access_expression(node);
                    self.emit_context()
                        .set_original_r37k13(&super_property, expression);
                    return with_loc(super_property, expression.loc);
                }
            }
        }
        self.visitor().visit_each_child(node)
    }
}
