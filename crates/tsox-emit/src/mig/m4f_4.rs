#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated::{is_computed_property_name, is_identifier};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::is_modifier_like;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::utilities::has_static_modifier;
use tsox_frontend::format::mig::m4o::EmitFlags;

use super::ClassFieldsTransformer;
use crate::mig::m4f_3::r38k10_defs::R38K10NodeVisitorExt;
use crate::mig::m4g::r33k7_defs::{class_like_name, computed_property_name_expression};
use crate::printer::NodeFactory;
use tsox_frontend::ast::ModifierList;

pub(crate) fn visit_modifiers_list_m4f4(
    visitor: &mut NodeVisitor,
    modifiers: Option<&Arc<ModifierList>>,
) -> Option<Arc<ModifierList>> {
    let modifiers = modifiers?;
    let visited: Vec<Arc<Node>> = modifiers
        .list
        .nodes
        .iter()
        .map(|m| visitor.visit_node(m))
        .collect();
    Some(Arc::new(ModifierList::new(
        visited,
        modifiers.modifier_flags,
    )))
}

impl ClassFieldsTransformer<'_> {
    pub fn visit_class_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::Constructor => self.set_current_class_element_and_opt(
                Some(Arc::clone(node)),
                Self::visit_constructor_declaration,
                node,
            ),
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor | SyntaxKind::MethodDeclaration => {
                self.set_current_class_element_and_opt(
                    Some(Arc::clone(node)),
                    Self::visit_method_or_accessor_declaration,
                    node,
                )
            }
            SyntaxKind::PropertyDeclaration => self.set_current_class_element_and_opt(
                Some(Arc::clone(node)),
                Self::visit_property_declaration,
                node,
            ),
            SyntaxKind::ClassStaticBlockDeclaration => self.set_current_class_element_and_opt(
                Some(Arc::clone(node)),
                Self::visit_class_static_block_declaration,
                node,
            ),
            SyntaxKind::ComputedPropertyName => self.visit_computed_property_name(node),
            SyntaxKind::SemicolonClassElement => Some(Arc::clone(node)),
            _ => {
                if is_modifier_like(node) {
                    return self.visit_modifier(node);
                }
                self.visit(node)
            }
        }
    }

    pub fn visit_property_name(&mut self, name: &Arc<Node>) -> Arc<Node> {
        if is_computed_property_name(name) {
            return self
                .visit_computed_property_name(name)
                .expect("computed property name visit should produce a node");
        }
        self.visitor().visit_node(name)
    }

    pub fn visit_accessor_field_result(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::PropertyDeclaration => self.transform_field_initializer(node),
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => self.visit_class_element(node),
            _ => panic!(
                "Expected node to either be a PropertyDeclaration, GetAccessorDeclaration, or SetAccessorDeclaration"
            ),
        }
    }

    pub fn visit_constructor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.visit_each_child_of_node(node)
    }

    pub fn visit_class_static_block_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !self.should_transform_private_elements_or_class_static_blocks {
            return self.visitor().visit_each_child(node);
        }
        None
    }

    pub fn visit_computed_property_name(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_inside_computed_property_name = self.inside_computed_property_name;
        self.inside_computed_property_name = true;
        let mut restored = self.lexical_environment.take();
        if let Some(env) = restored.as_mut() {
            if env.previous.is_some() {
                self.lexical_environment = env.previous.take();
            }
        }
        let expression = self.visitor().visit_node(computed_property_name_expression(node));
        self.lexical_environment = restored;
        self.inside_computed_property_name = saved_inside_computed_property_name;
        let injected = self.inject_pending_expressions(&expression);
        Some(self.factory().update_computed_property_name(node, &injected))
    }

    pub fn inject_pending_expressions(&mut self, expression: &Arc<Node>) -> Arc<Node> {
        if self.pending_expressions.is_empty() {
            return Arc::clone(expression);
        }
        let mut exprs = std::mem::take(&mut self.pending_expressions);
        let factory = self.factory();
        if let NodeData::ParenthesizedExpression(d) = &expression.data {
            exprs.push(Arc::clone(&d.expression));
            let inlined = factory
                .inline_expressions(exprs)
                .expect("expected at least one expression");
            factory.update_parenthesized_expression(expression, &inlined)
        } else {
            exprs.push(Arc::clone(expression));
            factory
                .inline_expressions(exprs)
                .expect("expected at least one expression")
        }
    }

    pub fn should_transform_class_element_to_weak_map(&self, node: &Arc<Node>) -> bool {
        if self.should_transform_private_elements_or_class_static_blocks {
            return true;
        }
        self.should_always_transform_private_static_elements(node)
    }

    pub fn should_always_transform_private_static_elements(&self, node: &Arc<Node>) -> bool {
        has_static_modifier(node)
            && self
                .emit_context
                .emit_flags(node)
                .contains(EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS)
    }

    pub fn try_get_class_this(&self) -> Option<Arc<Node>> {
        if let Some(class_this) = self.try_get_class_this_no_container() {
            return Some(class_this);
        }
        if let Some(container) = &self.current_class_container {
            return class_like_name(container).cloned();
        }
        None
    }

    pub fn try_get_class_this_no_container(&self) -> Option<Arc<Node>> {
        let env = self
            .lexical_environment
            .as_ref()
            .expect("lexicalEnvironment should be set");
        if let Some(class_this) = env.data.as_ref().and_then(|d| d.class_this.clone()) {
            return Some(class_this);
        }
        env.data
            .as_ref()
            .and_then(|d| d.class_constructor.clone())
    }
}
