#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::{
    is_array_literal_expression, is_computed_property_name, is_element_access_expression,
    is_identifier, is_object_literal_expression, is_private_identifier,
    is_property_access_expression, node_name,
};
use tsox_frontend::ast::mig::m3g_2::is_super_property;
#[path = "r37k13_defs.rs"]
pub mod r37k13_defs;

use r37k13_defs::{ClassFieldsTransformerR37k13, NodeFactoryR37k13};

use crate::mig::m4g::r39k15_defs::{ClassFieldsTransformerR39k15, NodeFactoryR39k15};

use crate::mig::m4g::{ClassFieldsTransformer, ClassFacts};
use crate::mig::m4g::r33k7_defs::{
    binary_left, binary_right, clone_node, element_access_argument, property_access_name,
};
use crate::mig::m4h_2::{is_named_evaluation_and, transform_named_evaluation};
use crate::mig::w7t::is_static_property_declaration_or_class_static_block;
use tsox_checker::binder::mig::r19k5_ext::export_assignment_is_export_equals;

impl ClassFieldsTransformer {
    pub fn visit_discarded_value(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_discarded_value"); 
        match node.kind {
            SyntaxKind::PrefixUnaryExpression | SyntaxKind::PostfixUnaryExpression => {
                self.visit_pre_or_postfix_unary_expression(node, true)
            }
            SyntaxKind::BinaryExpression => self.visit_binary_expression(node, true),
            SyntaxKind::ParenthesizedExpression => self.visit_parenthesized_expression(node, true),
            _ => self.visit(node),
        }
    }

    pub fn visit_heritage_clause(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_heritage_clause"); 
        match node.kind {
            SyntaxKind::HeritageClause => self.heritage_clause_visitor().visit_each_child(node),
            SyntaxKind::ExpressionWithTypeArguments => {
                self.visit_expression_with_type_arguments_in_heritage_clause(node)
            }
            _ => self.visit(node),
        }
    }

    pub fn visit_assignment_target(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_assignment_target"); 
        match node.kind {
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression => {
                self.visit_assignment_pattern(node)
            }
            _ => self.visit(node),
        }
    }

    pub fn visit_destructuring_assignment_target(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_destructuring_assignment_target"); 
        if is_object_literal_expression(node) || is_array_literal_expression(node) {
            return self.visit_assignment_pattern(node);
        }
        if is_property_access_expression(node) && is_private_identifier(property_access_name(node)) {
            return self.wrap_private_identifier_for_destructuring_target(node);
        }
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
                    let mut name: Option<Arc<Node>> = None;
                    if is_element_access_expression(node) {
                        name = Some(
                            self.visitor()
                                .visit_node(element_access_argument(node).unwrap())
                                .clone(),
                        );
                    } else if is_property_access_expression(node)
                        && is_identifier(property_access_name(node))
                    {
                        name = Some(
                            self.factory()
                                .new_string_literal_from_node(property_access_name(node)),
                        );
                    }
                    if let Some(name) = name {
                        let temp = self.factory().new_temp_variable_r37k13();
                        let set_expr = self.factory().new_reflect_set_call(
                            &data.super_class_reference.clone().unwrap(),
                            &name,
                            &temp,
                            &data.class_constructor.clone().unwrap(),
                        );
                        return self
                            .factory()
                            .new_assignment_target_wrapper(&temp, &set_expr);
                    }
                }
            }
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_class_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_class_element"); 
        match node.kind {
            SyntaxKind::Constructor => {
                self.set_current_class_element_and(node, Self::visit_constructor_declaration, node)
            }
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor | SyntaxKind::MethodDeclaration => {
                self.set_current_class_element_and(
                    node,
                    Self::visit_method_or_accessor_declaration,
                    node,
                )
            }
            SyntaxKind::PropertyDeclaration => {
                self.set_current_class_element_and(node, Self::visit_property_declaration, node)
            }
            SyntaxKind::ClassStaticBlockDeclaration => self.set_current_class_element_and(
                node,
                visit_class_static_block_declaration_r37k13,
                node,
            ),
            SyntaxKind::ComputedPropertyName => Some(self.visit_computed_property_name(node)),
            SyntaxKind::SemicolonClassElement => Some(node.clone()),
            _ => {
                if is_modifier_like(node) {
                    return self.visit_modifier(node);
                }
                Some(self.visit(node))
            }
        }
    }

    pub fn visit_property_name(&mut self, name: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_property_name"); 
        if is_computed_property_name(name) {
            return self.visit_computed_property_name(name);
        }
        self.visitor().visit_node(name)
    }

    pub fn visit_accessor_field_result(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_accessor_field_result"); 
        match node.kind {
            SyntaxKind::PropertyDeclaration => self.transform_field_initializer(node),
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => self.visit_class_element(node),
            _ => panic!(
                "Expected node to either be a PropertyDeclaration, GetAccessorDeclaration, or SetAccessorDeclaration"
            ),
        }
    }

    pub fn visit_identifier(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_identifier"); 
        let declaration = self
            .resolver()
            .get_referenced_value_declaration(&self.emit_context().most_original(node));
        if let Some(declaration) = declaration {
            let decl_key = Arc::as_ptr(&declaration) as usize;
            if let Some(alias) = self.class_aliases.get(&decl_key) {
                if self.enclosing_class_declarations.contains(&decl_key) {
                    let clone = clone_node(alias, &self.factory());
                    self.emit_context().set_source_map_range(&clone, node.loc);
                    self.emit_context().set_comment_range(&clone, node.loc);
                    return clone;
                }
            }
        }
        node.clone()
    }

    pub fn visit_private_identifier(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_private_identifier"); 
        if !self.should_transform_private_elements_or_class_static_blocks {
            return node.clone();
        }
        if let Some(parent) = &self.parent_node {
            if is_statement(parent) {
                return node.clone();
            }
        }
        let result = self.factory().new_identifier("");
        self.emit_context().set_original(&result, node);
        result
    }

    pub fn transform_private_identifier_in_in_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_private_identifier_in_in_expression"); 
        let info = self.access_private_identifier(binary_left(node));
        if let Some(info) = info {
            let receiver = self.visitor().visit_node(binary_right(node));
            let result = self
                .factory()
                .new_class_private_field_in_helper(
                    &info.brand_check_identifier.clone().expect("brand check identifier"),
                    &receiver,
                );
            self.emit_context().set_original(&result, node);
            return result;
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_property_assignment(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_property_assignment"); 
        let mut node = node.clone();
        if is_named_evaluation_and(&self.emit_context(), &node, None) {
            node = transform_named_evaluation(&self.emit_context(), &node, false, "");
        }
        self.visitor().visit_each_child(&node)
    }

    pub fn visit_variable_statement(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_variable_statement"); 
        let saved_pending_statements = std::mem::take(&mut self.pending_statements);
        let visited_node = self.visitor().visit_each_child(node);
        if !self.pending_statements.is_empty() {
            let mut result = Vec::with_capacity(1 + self.pending_statements.len());
            result.push(visited_node);
            result.extend(self.pending_statements.drain(..));
            self.pending_statements = saved_pending_statements;
            return self.factory().new_syntax_list(result);
        }
        self.pending_statements = saved_pending_statements;
        visited_node
    }

    pub fn visit_variable_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_variable_declaration"); 
        let mut node = node.clone();
        if is_named_evaluation_and(&self.emit_context(), &node, None) {
            node = transform_named_evaluation(&self.emit_context(), &node, false, "");
        }
        self.visitor().visit_each_child(&node)
    }

    pub fn visit_parameter_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_parameter_declaration"); 
        let mut node = node.clone();
        if is_named_evaluation_and(&self.emit_context(), &node, None) {
            node = transform_named_evaluation(&self.emit_context(), &node, false, "");
        }
        self.visitor().visit_each_child(&node)
    }

    pub fn visit_binding_element(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_binding_element"); 
        let mut node = node.clone();
        if is_named_evaluation_and(&self.emit_context(), &node, None) {
            node = transform_named_evaluation(&self.emit_context(), &node, false, "");
        }
        self.visitor().visit_each_child(&node)
    }

    pub fn visit_export_assignment(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_export_assignment"); 
        let mut node = node.clone();
        if is_named_evaluation_and(&self.emit_context(), &node, None) {
            let mut assigned_name = "";
            if !export_assignment_is_export_equals(&node) {
                assigned_name = "default";
            }
            node = transform_named_evaluation(&self.emit_context(), &node, true, assigned_name);
        }
        self.visitor().visit_each_child(&node)
    }

    fn set_current_class_element_and(
        &mut self,
        class_element: &Arc<Node>,
        visitor: fn(&mut Self, &Arc<Node>) -> Option<Arc<Node>>,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("set_current_class_element_and"); 
        if !self
            .current_class_element
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, class_element))
        {
            let saved = self.current_class_element.replace(class_element.clone());
            let result = visitor(self, node);
            self.current_class_element = saved;
            return result;
        }
        visitor(self, node)
    }

    pub fn visit_each_child_of_node(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child_of_node"); 
        self.visitor().visit_each_child(node)
    }
}

fn visit_class_static_block_declaration_r37k13(
    tx: &mut ClassFieldsTransformer,
    node: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_class_static_block_declaration_r37k13"); 
    if !tx.should_transform_private_elements_or_class_static_blocks {
        return Some(tx.visitor().visit_each_child(node));
    }
    None
}
