#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_computed_property_name, is_identifier, NodeData,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{has_accessor_modifier, has_static_modifier, is_assignment_expression};
use tsox_frontend::ast::mig::m3f_2::node_initializer;
use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
use tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration;
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use tsox_frontend::format::mig::m4o::EmitFlags;

use super::{ClassFieldsTransformer, CLASS_FACTS_WILL_HOIST_INITIALIZERS_TO_CONSTRUCTOR};
use super::m4f_4::visit_modifiers_list_m4f4;
use crate::mig::m4g::r33k7_defs::{binary_left, computed_property_name_expression};
use crate::mig::m4h_2::{is_named_evaluation_and, transform_named_evaluation};
use crate::mig::m4m_2::is_simple_inlineable_expression;
use crate::mig::m4m_4::move_range_past_modifiers;
use crate::mig::wt1b_4::flatten_comma_list;
use crate::mig::x6a::is_anonymous_class_needing_assigned_name;
use crate::printer::NodeFactory;

fn new_class_static_block_declaration_m4f6(body: &Arc<Node>) -> Arc<Node> {
    Arc::new(Node::new(
        SyntaxKind::ClassStaticBlockDeclaration,
        NodeData::ClassStaticBlockDeclaration(tsox_frontend::ast::node_data_generated::ClassStaticBlockDeclarationData {
            modifiers: None,
            body: Arc::clone(body),
        }),
    ))
}

impl ClassFieldsTransformer<'_> {
    pub fn transform_field_initializer(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_private_identifier_class_element_declaration(node) {
            return self.transform_private_field_initializer(node);
        }
        self.transform_public_field_initializer(node)
    }

    pub fn transform_private_field_initializer(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if self.should_transform_class_element_to_weak_map(node) {
            let name = node.name().expect("property declaration requires a name");
            let info = self
                .access_private_identifier(name)
                .expect("Undeclared private name for property declaration.");

            if !info.is_valid {
                return Some(Arc::clone(node));
            }

            if info.is_static && !self.should_transform_private_elements_or_class_static_blocks {
                let this_expr = self.factory().new_this_expression();
                if let Some(statement) = self.transform_property_or_class_static_block(node, &this_expr)
                {
                    let block = self
                        .factory()
                        .new_block(&self.factory().new_node_list(vec![statement]), true);
                    return Some(new_class_static_block_declaration_m4f6(&block));
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
                .is_some_and(|data| {
                    data.facts & CLASS_FACTS_WILL_HOIST_INITIALIZERS_TO_CONSTRUCTOR != 0
                })
        {
            let visited_modifiers =
                visit_modifiers_list_m4f4(self.visitor(), node.modifiers());
            return Some(self.factory().update_property_declaration(
                node,
                visited_modifiers,
                node.name().expect("property declaration requires a name"),
                None,
                None,
                None,
            ));
        }

        let mut node = Arc::clone(node);
        if is_named_evaluation_and(
            &self.emit_context,
            &node,
            Some(&is_anonymous_class_needing_assigned_name),
        ) {
            node = transform_named_evaluation(&self.emit_context, &node, false, "");
        }

        let modifiers = visit_modifiers_list_m4f4(
            self.modifier_visitor.as_mut().unwrap(),
            node.modifiers(),
        );
        let visited_name = self.visit_property_name(
            node.name().expect("property declaration requires a name"),
        );
        let initializer = self.visitor().visit_node_opt(node_initializer(&node));
        Some(self.factory().update_property_declaration(
            &node,
            modifiers,
            &visited_name,
            None,
            None,
            initializer,
        ))
    }

    pub fn transform_public_field_initializer(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if self.should_transform_initializers && !is_auto_accessor_property_declaration(node) {
            let initializer_present = node_initializer(node).is_some();
            let expr = self.get_property_name_expression_if_needed(
                node.name().expect("property declaration requires a name"),
                initializer_present || self.compiler_options.get_use_define_for_class_fields(),
            );
            if let Some(expr) = expr {
                for e in flatten_comma_list(&expr) {
                    self.pending_expressions.push(e);
                }
            }

            if has_static_modifier(node)
                && !self.should_transform_private_elements_or_class_static_blocks
            {
                let this_expr = self.factory().new_this_expression();
                if let Some(initializer_statement) =
                    self.transform_property_or_class_static_block(node, &this_expr)
                {
                    let block = self.factory().new_block(
                        &self
                            .factory()
                            .new_node_list(vec![Arc::clone(&initializer_statement)]),
                        false,
                    );
                    let static_block = new_class_static_block_declaration_m4f6(&block);
                    self.emit_context.set_original(&static_block, node);
                    self.emit_context.set_comment_range(&static_block, node.loc);
                    self.emit_context
                        .add_emit_flags(&initializer_statement, EmitFlags::NO_COMMENTS);
                    return Some(static_block);
                }
            }

            return None;
        }

        let modifiers = visit_modifiers_list_m4f4(
            self.modifier_visitor.as_mut().unwrap(),
            node.modifiers(),
        );
        let visited_name = self.visit_property_name(
            node.name().expect("property declaration requires a name"),
        );
        let initializer = self.visitor().visit_node_opt(node_initializer(node));
        Some(self.factory().update_property_declaration(
            node,
            modifiers,
            &visited_name,
            None,
            None,
            initializer,
        ))
    }

    pub fn get_property_name_expression_if_needed(
        &mut self,
        name: &Arc<Node>,
        should_hoist: bool,
    ) -> Option<Arc<Node>> {
        if !is_computed_property_name(name) {
            return None;
        }
        let cache_assignment = self.find_computed_property_name_cache_assignment(name);
        let saved_inside_computed_property_name = self.inside_computed_property_name;
        self.inside_computed_property_name = true;
        let mut restored = self.lexical_environment.take();
        if let Some(env) = restored.as_mut() {
            if env.previous.is_some() {
                self.lexical_environment = env.previous.take();
            }
        }
        let expression = self.visitor().visit_node(computed_property_name_expression(name));
        self.lexical_environment = restored;
        self.inside_computed_property_name = saved_inside_computed_property_name;
        let inner_expression =
            skip_outer_expressions(&expression, OuterExpressionKinds::PARTIALLY_EMITTED_EXPRESSIONS);
        let inlinable = is_simple_inlineable_expression(&inner_expression);
        let already_transformed = cache_assignment.is_some()
            || (is_assignment_expression(&inner_expression, true)
                && is_identifier(binary_left(&inner_expression)));
        if !already_transformed && !inlinable && should_hoist {
            let generated_name =
                self.factory().generated_name_node(&self.factory().new_generated_name_for_node(name));
            if self.requires_block_scoped_var() {
                self.emit_context.add_lexical_declaration(&generated_name);
            } else {
                self.emit_context.add_variable_declaration(&generated_name);
            }
            return Some(self.factory().new_assignment_expression(&generated_name, &expression));
        }
        if inlinable || is_identifier(&inner_expression) {
            return None;
        }
        Some(expression)
    }

    pub fn transform_property_or_class_static_block(
        &mut self,
        property: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let expression = if property.kind == SyntaxKind::ClassStaticBlockDeclaration {
            self.set_current_class_element_and_opt(
                Some(Arc::clone(property)),
                Self::transform_class_static_block_declaration,
                property,
            )
        } else {
            self.transform_property(property, receiver)
        }?;
        let statement = self.factory().new_expression_statement(&expression);
        self.emit_context.set_original(&statement, property);
        let property_flags = self.emit_context.emit_flags(property);
        self.emit_context
            .add_emit_flags(&statement, property_flags & EmitFlags::NO_COMMENTS);
        self.emit_context.set_comment_range(&statement, property.loc);

        let property_original_node = self.emit_context.most_original(property);
        if property_original_node.kind == SyntaxKind::Parameter {
            self.emit_context
                .set_source_map_range(&statement, property_original_node.loc);
            self.emit_context
                .add_emit_flags(&statement, EmitFlags::NO_COMMENTS);
        } else {
            self.emit_context
                .set_source_map_range(&statement, move_range_past_modifiers(property));
        }

        if has_accessor_modifier(&property_original_node) {
            self.emit_context
                .add_emit_flags(&statement, EmitFlags::NO_COMMENTS);
        }

        Some(statement)
    }
}
