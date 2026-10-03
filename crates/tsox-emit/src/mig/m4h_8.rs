use std::sync::Arc;
use tsox_frontend::ast::*;

#[path = "r36k26_defs.rs"]
pub mod r36k26_defs;

use crate::mig::m4h::r37k18_defs::R37K18NodeVisitorExt;
use crate::mig::m4m_5::r38k9_defs::R38K9NodeCastExt;
use crate::mig::m4h_2::{is_named_evaluation_and, transform_named_evaluation};
use crate::mig::m4h_5::EsDecoratorTransformer;
use crate::mig::m4m_2::is_simple_inlineable_expression;
use crate::mig::m4m_4::move_range_past_modifiers;
use crate::mig::x6a::is_anonymous_class_needing_assigned_name;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::ast::mig::m3f_3::is_array_binding_or_assignment_element;
use tsox_frontend::ast::mig::m3g_2::{is_object_binding_or_assignment_element, is_super_property};
use tsox_frontend::ast::mig::m3g_3::{
    node_is_decorated, node_or_child_is_decorated, skip_outer_expressions, OuterExpressionKinds,
};
use tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body;
use tsox_frontend::ast::node_data_generated::is_class_expression;

fn class_or_constructor_parameter_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("class_or_constructor_parameter_is_decorated"); 
    if node_is_decorated(use_legacy_decorators, node, None, None) {
        return true;
    }
    match get_first_constructor_with_body(node) {
        Some(constructor) => {
            node_or_child_is_decorated(use_legacy_decorators, &constructor, Some(node), None)
        }
        None => false,
    }
}

fn can_ignore_empty_string_literal_in_assigned_name(node: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("can_ignore_empty_string_literal_in_assigned_name"); 
    let Some(node) = node else {
        return false;
    };
    let inner_expression = skip_outer_expressions(node, OuterExpressionKinds::ALL);
    is_class_expression(&inner_expression)
        && inner_expression.name().is_none()
        && !class_or_constructor_parameter_is_decorated(false, &inner_expression)
}

impl EsDecoratorTransformer {
    pub fn visit_this_expression(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_this_expression"); 
        if let Some(class_this) = &self.class_this {
            return class_this.clone();
        }
        node.clone()
    }

    pub fn visit_call_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_call_expression"); 
        let (call_expression, call_arguments) = match &node.data {
            NodeData::CallExpression(d) => (d.expression.clone(), d.arguments.clone()),
            _ => unreachable!(),
        };
        if is_super_property(&call_expression) && self.class_this.is_some() {
            let expression = self.transformer.visitor().visit_node(&call_expression);
            let arguments_list = self.transformer.visitor().visit_node_list(Some(&call_arguments));
            let mut invocation = self.transformer.factory().new_function_call_call(
                &expression,
                self.class_this.as_ref().unwrap(),
                &arguments_list,
            );
            self.transformer.emit_context().set_original(&invocation, node);
            if let Some(invocation) = Arc::get_mut(&mut invocation) {
                invocation.loc = node.loc;
            }
            return invocation;
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_tagged_template_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_tagged_template_expression"); 
        let (tag, template, flags) = match &node.data {
            NodeData::TaggedTemplateExpression(d) => (d.tag.clone(), d.template.clone(), node.flags),
            _ => unreachable!(),
        };
        if is_super_property(&tag) && self.class_this.is_some() {
            let visited_tag = self.transformer.visitor().visit_node(&tag);
            let mut bound_tag = self
                .transformer
                .factory()
                .new_function_bind_call(&visited_tag, self.class_this.as_ref().unwrap(), &[]);
            self.transformer.emit_context().set_original(&bound_tag, node);
            if let Some(bound_tag) = Arc::get_mut(&mut bound_tag) {
                bound_tag.loc = node.loc;
            }
            let visited_template = self.transformer.visitor().visit_node(&template);
            let f = self.transformer.factory();
            return f.update_tagged_template_expression(node, &bound_tag, None, None, &visited_template, flags);
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_property_access_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_property_access_expression"); 
        let (pa_expression, pa_name) = match &node.data {
            NodeData::PropertyAccessExpression(d) => (d.expression.clone(), d.name.clone()),
            _ => unreachable!(),
        };
        if is_super_property(node) && is_identifier(&pa_name) && self.class_this.is_some() && self.class_super.is_some() {
            let f = self.transformer.factory();
            let property_name = f.new_string_literal_from_node(&pa_name);
            let mut super_property = f.new_reflect_get_call(
                self.class_super.as_ref().unwrap(),
                &property_name,
                self.class_this.as_ref().unwrap(),
            );
            self.transformer.emit_context().set_original(&super_property, &pa_expression);
            if let Some(super_property) = Arc::get_mut(&mut super_property) {
                super_property.loc = pa_expression.loc;
            }
            return super_property;
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_element_access_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_element_access_expression"); 
        let (ea_expression, ea_argument) = match &node.data {
            NodeData::ElementAccessExpression(d) => (d.expression.clone(), d.argument_expression.clone()),
            _ => unreachable!(),
        };
        if is_super_property(node) && self.class_this.is_some() && self.class_super.is_some() {
            let property_name = self.transformer.visitor().visit_node(&ea_argument);
            let f = self.transformer.factory();
            let mut super_property = f.new_reflect_get_call(
                self.class_super.as_ref().unwrap(),
                &property_name,
                self.class_this.as_ref().unwrap(),
            );
            self.transformer.emit_context().set_original(&super_property, &ea_expression);
            if let Some(super_property) = Arc::get_mut(&mut super_property) {
                super_property.loc = ea_expression.loc;
            }
            return super_property;
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_parameter_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_parameter_declaration"); 
        let mut param_node = node.clone();
        let initializer = param_node.initializer().cloned();
        if is_named_evaluation_and(
            &self.transformer.emit_context(),
            &param_node,
            Some(&is_anonymous_class_needing_assigned_name),
        ) {
            param_node = transform_named_evaluation(
                &self.transformer.emit_context(),
                &param_node,
                can_ignore_empty_string_literal_in_assigned_name(initializer.as_ref()),
                "",
            );
        }
        let (dot_dot_dot_token, name, initializer) = match &param_node.data {
            NodeData::ParameterDeclaration(d) => {
                (d.dot_dot_dot_token.clone(), d.name.clone(), d.initializer.clone())
            }
            _ => unreachable!(),
        };
        let visited_name = self.transformer.visitor().visit_node(&name);
        let visited_initializer = initializer
            .as_ref()
            .map(|i| self.transformer.visitor().visit_node(i));
        let mut updated = self.transformer.factory().update_parameter_declaration(
            &param_node,
            None,
            dot_dot_dot_token.as_ref(),
            &visited_name,
            None,
            None,
            visited_initializer.as_ref(),
        );
        if updated.id() != param_node.id() {
            let mut ec = self.transformer.emit_context();
            ec.set_comment_range(&updated, param_node.loc);
            let new_loc = move_range_past_modifiers(&param_node);
            if let Some(updated) = Arc::get_mut(&mut updated) {
                updated.loc = new_loc;
            }
            ec.set_source_map_range(&updated, new_loc);
            ec.add_emit_flags(&updated.name().unwrap(), EmitFlags::NO_TRAILING_SOURCE_MAP);
        }
        updated
    }

    pub fn visit_named_evaluation_site(&mut self, node: &Arc<Node>, class_expr: Option<&Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_named_evaluation_site"); 
        if is_named_evaluation_and(
            &self.transformer.emit_context(),
            node,
            Some(&is_anonymous_class_needing_assigned_name),
        ) {
            let node = transform_named_evaluation(
                &self.transformer.emit_context(),
                node,
                can_ignore_empty_string_literal_in_assigned_name(class_expr),
                "",
            );
            return self.transformer.visitor().visit_each_child(&node);
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_for_statement(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_for_statement"); 
        let (initializer, condition, incrementor, statement) = match &node.data {
            NodeData::ForStatement(d) => (
                d.initializer.clone(),
                d.condition.clone(),
                d.incrementor.clone(),
                d.statement.clone(),
            ),
            _ => unreachable!(),
        };
        let visited_statement = self
            .transformer
            .emit_context()
            .visit_iteration_body(Some(Arc::clone(&statement)), self.transformer.visitor());
        let visited_initializer = initializer
            .as_ref()
            .map(|i| self.discarded_visitor.visit_node(i));
        let visited_condition = condition
            .as_ref()
            .map(|c| self.transformer.visitor().visit_node(c));
        let visited_incrementor = incrementor
            .as_ref()
            .map(|i| self.discarded_visitor.visit_node(i));
        let f = self.transformer.factory();
        f.update_for_statement(
            node,
            visited_initializer.as_ref(),
            visited_condition.as_ref(),
            visited_incrementor.as_ref(),
            &visited_statement.unwrap(),
        )
    }

    pub fn visit_expression_statement(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_expression_statement"); 
        self.discarded_visitor.visit_each_child(node)
    }

    pub fn visit_referenced_property_name(&mut self, node: &Arc<Node>) -> (Arc<Node>, Arc<Node>) { ::tsox_core::fntrace::enter("visit_referenced_property_name"); 
        if is_property_name_literal(node) || is_private_identifier(node) {
            let referenced = self.transformer.factory().new_string_literal_from_node(node);
            let visited = self.transformer.visitor().visit_node(node);
            return (referenced, visited);
        }

        let cpn_expression = match &node.data {
            NodeData::ComputedPropertyName(d) => d.expression.clone(),
            _ => unreachable!(),
        };
        if is_property_name_literal(&cpn_expression) && !is_identifier(&cpn_expression) {
            let referenced = self
                .transformer
                .factory()
                .new_string_literal_from_node(&cpn_expression);
            let visited = self.transformer.visitor().visit_node(node);
            return (referenced, visited);
        }

        let referenced_name = {
            let f = self.transformer.factory();
            f.generated_name_node(&f.new_generated_name_for_node(node))
        };
        self.transformer
            .emit_context()
            .add_variable_declaration(&referenced_name);

        let visited_expression = self.transformer.visitor().visit_node(&cpn_expression);
        let assignment = {
            let f = self.transformer.factory();
            let key = f.new_prop_key_helper(&visited_expression);
            f.new_assignment_expression(&referenced_name, &key)
        };
        let injected = self.inject_pending_expressions(&assignment);
        let updated_name = self
            .transformer
            .factory()
            .update_computed_property_name(node, &injected);
        (referenced_name, updated_name)
    }

    pub fn visit_property_name(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_property_name"); 
        if is_computed_property_name(node) {
            return self.visit_computed_property_name(node);
        }
        self.transformer.visitor().visit_node(node)
    }

    pub fn visit_computed_property_name(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_computed_property_name"); 
        let cpn_expression = match &node.data {
            NodeData::ComputedPropertyName(d) => d.expression.clone(),
            _ => unreachable!(),
        };
        let mut expression = self.transformer.visitor().visit_node(&cpn_expression);
        if !is_simple_inlineable_expression(&expression) {
            expression = self.inject_pending_expressions(&expression);
        }
        let f = self.transformer.factory();
        f.update_computed_property_name(node, &expression)
    }

    pub fn visit_destructuring_assignment_target(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_destructuring_assignment_target"); 
        if is_object_literal_expression(node) || is_array_literal_expression(node) {
            return self.visit_assignment_pattern(node);
        }

        if is_super_property(node) && self.class_this.is_some() && self.class_super.is_some() {
            let mut property_name: Option<Arc<Node>> = None;
            if is_element_access_expression(node) {
                let argument = match &node.data {
                    NodeData::ElementAccessExpression(d) => d.argument_expression.clone(),
                    _ => unreachable!(),
                };
                property_name = Some(self.transformer.visitor().visit_node(&argument));
            } else if is_property_access_expression(node) {
                let name = node.name().unwrap();
                if is_identifier(name) {
                    property_name = Some(
                        self.transformer
                            .factory()
                            .new_string_literal_from_node(name),
                    );
                }
            }
            if let Some(property_name) = property_name {
                let ec = self.transformer.emit_context();
                let f = self.transformer.factory();
                let param_name = f.generated_name_node(&f.new_temp_variable());
                let reflect_set = f.new_reflect_set_call(
                    self.class_super.as_ref().unwrap(),
                    &property_name,
                    &param_name,
                    self.class_this.as_ref().unwrap(),
                );
                let mut expression = f.new_assignment_target_wrapper(&param_name, &reflect_set);
                ec.set_original(&expression, node);
                if let Some(expression) = Arc::get_mut(&mut expression) {
                    expression.loc = node.loc;
                }
                return expression;
            }
        }

        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_assignment_rest_element(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_assignment_rest_element"); 
        let se_expression = match &node.data {
            NodeData::SpreadElement(d) => d.expression.clone(),
            _ => unreachable!(),
        };
        if is_left_hand_side_expression(&se_expression) {
            let expression = self.visit_destructuring_assignment_target(&se_expression);
            let f = self.transformer.factory();
            return f.update_spread_element(node, &expression);
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_array_assignment_element(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_array_assignment_element"); 
        assert!(is_array_binding_or_assignment_element(node));
        if is_spread_element(node) {
            return self.visit_assignment_rest_element(node);
        }
        if !is_omitted_expression(node) {
            return self.visit_assignment_element(node);
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_assignment_rest_property(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_assignment_rest_property"); 
        let sa_expression = match &node.data {
            NodeData::SpreadAssignment(d) => d.expression.clone(),
            _ => unreachable!(),
        };
        if is_left_hand_side_expression(&sa_expression) {
            let expression = self.visit_destructuring_assignment_target(&sa_expression);
            let f = self.transformer.factory();
            return f.update_spread_assignment(node, &expression);
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_assignment_element(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_assignment_element"); 
        if is_assignment_expression(node, true) {
            let mut node = node.clone();
            if is_named_evaluation_and(
                &self.transformer.emit_context(),
                &node,
                Some(&is_anonymous_class_needing_assigned_name),
            ) {
                let right = node.as_binary_expression().right.clone();
                let transformed = transform_named_evaluation(
                    &self.transformer.emit_context(),
                    &node,
                    can_ignore_empty_string_literal_in_assigned_name(Some(&right)),
                    "",
                );
                node = transformed;
            }
            let bin = node.as_binary_expression();
            let assignment_target = self.visit_destructuring_assignment_target(&bin.left);
            let initializer = self.transformer.visitor().visit_node(&bin.right);
            let f = self.transformer.factory();
            return f.update_binary_expression_r39k13(
                &node,
                &assignment_target,
                &bin.operator_token,
                &initializer,
            );
        }
        self.visit_destructuring_assignment_target(node)
    }

    pub fn visit_assignment_property_node(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_assignment_property_node"); 
        let (name, initializer) = match &node.data {
            NodeData::PropertyAssignment(d) => (d.name.clone(), d.initializer.clone()),
            _ => unreachable!(),
        };
        let visited_name = self.transformer.visitor().visit_node(&name);
        if is_assignment_expression(&initializer, true) {
            let assignment_element = self.visit_assignment_element(&initializer);
            let f = self.transformer.factory();
            return f.update_property_assignment_r39k13(
                node,
                None,
                &visited_name,
                None,
                None,
                Some(&assignment_element),
            );
        }
        if is_left_hand_side_expression(&initializer) {
            let assignment_element = self.visit_destructuring_assignment_target(&initializer);
            let f = self.transformer.factory();
            return f.update_property_assignment_r39k13(
                node,
                None,
                &visited_name,
                None,
                None,
                Some(&assignment_element),
            );
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_shorthand_assignment_property(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_shorthand_assignment_property"); 
        if is_named_evaluation_and(
            &self.transformer.emit_context(),
            node,
            Some(&is_anonymous_class_needing_assigned_name),
        ) {
            let object_assignment_initializer = node
                .as_shorthand_property_assignment()
                .object_assignment_initializer
                .clone();
            let transformed = transform_named_evaluation(
                &self.transformer.emit_context(),
                node,
                can_ignore_empty_string_literal_in_assigned_name(
                    object_assignment_initializer.as_ref(),
                ),
                "",
            );
            return self.transformer.visitor().visit_each_child(&transformed);
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_object_assignment_element(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_object_assignment_element"); 
        assert!(is_object_binding_or_assignment_element(node));
        if is_spread_assignment(node) {
            return self.visit_assignment_rest_property(node);
        }
        if is_shorthand_property_assignment(node) {
            return self.visit_shorthand_assignment_property(node);
        }
        if is_property_assignment(node) {
            return self.visit_assignment_property_node(node);
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_assignment_pattern(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_assignment_pattern"); 
        let f = self.transformer.factory();
        if is_array_literal_expression(node) {
            let (elements, multi_line) = match &node.data {
                NodeData::ArrayLiteralExpression(d) => (d.elements.clone(), d.multi_line),
                _ => unreachable!(),
            };
            let visited = self.array_assignment_visitor.visit_node_list(Some(&elements));
            return f.update_array_literal_expression(node, &visited, multi_line);
        }
        let (properties, multi_line) = match &node.data {
            NodeData::ObjectLiteralExpression(d) => (d.properties.clone(), d.multi_line),
            _ => unreachable!(),
        };
        let visited = self.object_assignment_visitor.visit_node_list(Some(&properties));
        f.update_object_literal_expression(node, &visited, multi_line)
    }

    pub fn visit_export_assignment(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_export_assignment"); 
        self.visit_named_evaluation_site(node, node.expression())
    }

    pub fn visit_parenthesized_expression(&mut self, node: &Arc<Node>, discarded: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_parenthesized_expression"); 
        let pe_expression = match &node.data {
            NodeData::ParenthesizedExpression(d) => d.expression.clone(),
            _ => unreachable!(),
        };
        let expression = if discarded {
            self.discarded_visitor.visit_node(&pe_expression)
        } else {
            self.transformer.visitor().visit_node(&pe_expression)
        };
        let f = self.transformer.factory();
        f.update_parenthesized_expression(node, &expression)
    }

    pub fn visit_partially_emitted_expression(&mut self, node: &Arc<Node>, discarded: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_partially_emitted_expression"); 
        let pe_expression = match &node.data {
            NodeData::PartiallyEmittedExpression(d) => d.expression.clone(),
            _ => unreachable!(),
        };
        let expression = if discarded {
            self.discarded_visitor.visit_node(&pe_expression)
        } else {
            self.transformer.visitor().visit_node(&pe_expression)
        };
        let f = self.transformer.factory();
        f.update_partially_emitted_expression(node, &expression)
    }

    pub fn prepend_expressions(
        &mut self,
        pending: &[Arc<Node>],
        expression: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("prepend_expressions"); 
        let f = self.transformer.factory();
        if pending.is_empty() {
            return expression.cloned();
        }
        let expression = expression?;
        if is_parenthesized_expression(expression) {
            let pe_expression = match &expression.data {
                NodeData::ParenthesizedExpression(d) => d.expression.clone(),
                _ => unreachable!(),
            };
            let mut exprs = pending.to_vec();
            exprs.push(pe_expression);
            let inlined = f.inline_expressions(exprs).unwrap();
            return Some(f.update_parenthesized_expression(expression, &inlined));
        }
        let mut exprs = pending.to_vec();
        exprs.push(expression.clone());
        f.inline_expressions(exprs)
    }

    pub fn inject_pending_expressions(&mut self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("inject_pending_expressions"); 
        let pending = std::mem::take(&mut self.pending_expressions);
        let result = self
            .prepend_expressions(&pending, Some(expression))
            .unwrap();
        if result.id() != expression.id() {
            self.pending_expressions.clear();
        } else {
            self.pending_expressions = pending;
        }
        result
    }

    pub fn transform_all_decorators_of_declaration(&mut self, decorators: &[Arc<Node>]) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("transform_all_decorators_of_declaration"); 
        if decorators.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(decorators.len());
        for d in decorators {
            result.push(self.transform_decorator(d));
        }
        result
    }

    pub fn transform_decorator(&mut self, decorator: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_decorator"); 
        let decorator_expression = match &decorator.data {
            NodeData::Decorator(d) => d.expression.clone(),
            _ => unreachable!(),
        };
        let expression = self.transformer.visitor().visit_node(&decorator_expression);
        self.transformer
            .emit_context()
            .add_emit_flags(&expression, EmitFlags::NO_COMMENTS);

        let inner_expression = skip_outer_expressions(&expression, OuterExpressionKinds::ALL);
        if is_access_expression(&inner_expression) {
            let f = self.transformer.factory();
            let (target, this_arg) = self.create_call_binding(&expression);
            let bind_call = f.new_function_bind_call(&target, &this_arg, &[]);
            return f.restore_outer_expressions(&expression, &bind_call, OuterExpressionKinds::ALL);
        }
        expression
    }

    fn create_call_binding(&self, expression: &Arc<Node>) -> (Arc<Node>, Arc<Node>) { ::tsox_core::fntrace::enter("create_call_binding"); 
        let f = self.transformer.factory();
        let callee = skip_outer_expressions(expression, OuterExpressionKinds::ALL);
        if is_super_property(&callee) || callee.kind == SyntaxKind::SuperKeyword {
            return (callee, f.new_this_expression());
        }
        if self
            .transformer
            .emit_context()
            .emit_flags(&callee)
            .intersects(EmitFlags::HELPER_NAME)
        {
            return (callee, f.new_void_zero_expression());
        }
        if is_property_access_expression(&callee) {
            let (pa_expression, pa_name) = match &callee.data {
                NodeData::PropertyAccessExpression(d) => (d.expression.clone(), d.name.clone()),
                _ => unreachable!(),
            };
            if self.should_be_captured_in_temp_variable(&pa_expression) {
                let this_arg = f.generated_name_node(&f.new_temp_variable());
                self.transformer.emit_context().add_variable_declaration(&this_arg);
                let mut assign = f.new_assignment_expression(&this_arg, &pa_expression);
                if let Some(assign) = Arc::get_mut(&mut assign) {
                    assign.loc = pa_expression.loc;
                }
                let mut target = f.new_property_access_expression(&assign, None, &pa_name, NodeFlags::empty());
                if let Some(target) = Arc::get_mut(&mut target) {
                    target.loc = callee.loc;
                }
                return (target, this_arg);
            }
            return (callee, pa_expression);
        }
        if is_element_access_expression(&callee) {
            let (ea_expression, ea_argument) = match &callee.data {
                NodeData::ElementAccessExpression(d) => (d.expression.clone(), d.argument_expression.clone()),
                _ => unreachable!(),
            };
            if self.should_be_captured_in_temp_variable(&ea_expression) {
                let this_arg = f.generated_name_node(&f.new_temp_variable());
                self.transformer.emit_context().add_variable_declaration(&this_arg);
                let mut assign = f.new_assignment_expression(&this_arg, &ea_expression);
                if let Some(assign) = Arc::get_mut(&mut assign) {
                    assign.loc = ea_expression.loc;
                }
                let mut target = f.new_element_access_expression(&assign, None, &ea_argument, NodeFlags::empty());
                if let Some(target) = Arc::get_mut(&mut target) {
                    target.loc = callee.loc;
                }
                return (target, this_arg);
            }
            return (callee, ea_expression);
        }
        (expression.clone(), f.new_void_zero_expression())
    }

    pub fn should_be_captured_in_temp_variable(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_be_captured_in_temp_variable"); 
        let target = skip_parentheses(node);
        match target.kind {
            SyntaxKind::Identifier => true,
            SyntaxKind::ThisKeyword
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral => false,
            _ => true,
        }
    }
}
