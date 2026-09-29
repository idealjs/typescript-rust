#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{
    is_class_static_block_declaration, is_computed_property_name, is_identifier,
    is_numeric_literal, is_parenthesized_expression, is_private_identifier, is_void_expression,
    NodeData,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::mig::m3g_3::skip_parentheses;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::utilities::is_assignment_expression;
use tsox_frontend::ast::utilities::{has_static_modifier, has_syntactic_modifier};
use tsox_frontend::ast::mig::m3f_2::has_abstract_modifier;
use tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::{TokenFlags, TOKEN_FLAGS_NONE};

use crate::printer::{AutoGenerateOptions, NodeFactory};
use super::m4h_2::{is_named_evaluation_and, transform_named_evaluation};
use super::m4m_2::is_simple_inlineable_expression;

use super::m4f_3::ClassFieldsTransformer;
use super::m4g::r33k7_defs::{computed_property_name_expression, PrivateIdentifierKind};
use crate::mig::m4h_2::is_class_named_evaluation_helper_block;
use crate::mig::x6a::is_class_this_assignment_block;

impl ClassFieldsTransformer<'_> {
    pub fn generate_initialized_property_expressions_or_class_static_block(
        &mut self,
        properties_or_class_static_blocks: &[Arc<Node>],
        receiver: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        let mut expressions: Vec<Arc<Node>> = Vec::new();
        for property in properties_or_class_static_blocks {
            let expression = if is_class_static_block_declaration(property) {
                self.set_current_class_element_and_opt(
                    Some(Arc::clone(property)),
                    Self::transform_class_static_block_declaration,
                    property,
                )
            } else {
                self.transform_property(property, receiver)
            };
            let Some(expression) = expression else {
                continue;
            };
            self.emit_context()
                .set_original_ex(&expression, property, true);
            self.emit_context()
                .assign_comment_and_source_map_ranges(&expression, property);
            expressions.push(expression);
        }
        expressions
    }

    pub fn transform_class_static_block_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !self.should_transform_private_elements_or_class_static_blocks {
            return None;
        }
        if is_class_this_assignment_block(&self.emit_context, node) {
            let result = self
                .visitor()
                .visit_node(static_block_first_expression(node)?);
            if is_assignment_expression(&result, true) {
                if let NodeData::BinaryExpression(binary) = &result.data {
                    if Arc::ptr_eq(&binary.left, &binary.right) {
                        return None;
                    }
                }
            }
            return Some(result);
        }
        if is_class_named_evaluation_helper_block(&self.emit_context, node) {
            return Some(self.visitor().visit_node(static_block_first_expression(node)?));
        }

        self.emit_context.start_variable_environment();
        let visited: Vec<Arc<Node>> = {
            let statements = static_block_statements(node);
            let saved = self.current_class_element.take();
            self.current_class_element = Some(Arc::clone(node));
            let visitor = self.visitor();
            let visited = statements.iter().map(|s| visitor.visit_node(s)).collect();
            self.current_class_element = saved;
            visited
        };
        let (statements, _) = self
            .emit_context
            .end_and_merge_variable_environment(&visited);

        let mut iife = tsox_frontend::format::mig::m4o::new_node_factory(
            tsox_frontend::format::mig::m4o::EmitContext::default(),
        )
        .new_immediately_invoked_arrow_function(&statements);
        let arrow_function = skip_parentheses(iife.expression().expect("iife expression"));
        self.emit_context.set_original(&arrow_function, node);
        self.emit_context
            .add_emit_flags(&arrow_function, EmitFlags::NO_LEXICAL_ARGUMENTS);
        self.emit_context.set_original(&iife, node);
        self.emit_context.assign_source_map_range(&iife, node);
        self.emit_context
            .add_emit_flags(&arrow_function, EmitFlags::NO_LEXICAL_THIS);
        let statements_loc = static_block_statements_loc(node);
        drop(arrow_function);
        if let Some(iife_node) = Arc::get_mut(&mut iife) {
            if let NodeData::CallExpression(call) = &mut iife_node.data {
                if let Some(paren) = Arc::get_mut(&mut call.expression) {
                    if let NodeData::ParenthesizedExpression(paren_data) = &mut paren.data {
                        if let Some(arrow) = Arc::get_mut(&mut paren_data.expression) {
                            if let NodeData::ArrowFunction(arrow_data) = &mut arrow.data {
                                if let Some(body) = Arc::get_mut(&mut arrow_data.body) {
                                    if let NodeData::Block(block) = &mut body.data {
                                        block.statements = Arc::new(NodeList {
                                            loc: statements_loc,
                                            nodes: block.statements.nodes.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(iife)
    }

    pub fn transform_property(&mut self, property: &Arc<Node>, receiver: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_current_class_element = self.current_class_element.clone();
        let transformed = self.transform_property_worker(property, receiver);
        if let Some(transformed) = transformed.as_ref() {
            if has_static_modifier(property) {
                self.emit_context
                    .add_emit_flags(transformed, EmitFlags::NO_LEXICAL_THIS);
            }
            let captured_lexical_environment = self
                .lexical_environment
                .as_ref()
                .and_then(|env| env.data.as_ref())
                .is_some_and(|data| data.facts != 0);
            if has_static_modifier(property) && captured_lexical_environment {
                self.emit_context.set_original(transformed, property);
                if let Some(name) = property.name() {
                    self.emit_context
                        .set_source_map_range(transformed, name.loc.clone());
                }
            }
        }
        self.current_class_element = saved_current_class_element;
        transformed
    }

    fn transform_property_worker(&mut self, property: &Arc<Node>, receiver: &Arc<Node>) -> Option<Arc<Node>> {
        let emit_assignment = !self.compiler_options.get_use_define_for_class_fields();

        let mut property = Arc::clone(property);
        if is_named_evaluation_and(&self.emit_context, &property, None) {
            property = transform_named_evaluation(&self.emit_context, &property, false, "");
        }

        let mut property_name = property.name().expect("property declaration requires a name").clone();
        if has_syntactic_modifier(&property, ModifierFlags::Accessor) {
            property_name = {
                let factory = self.factory();
                factory.generated_name_node(&factory.new_generated_private_name_for_node_ex(
                    property.name().as_ref().expect("property declaration requires a name"),
                    AutoGenerateOptions {
                        suffix: "_accessor_storage".to_string(),
                        ..Default::default()
                    },
                ))
            };
        } else if is_computed_property_name(&property_name)
            && !is_simple_inlineable_expression(computed_property_name_expression(&property_name))
        {
            property_name = {
                let factory = self.factory();
                let generated_name = factory.new_generated_name_for_node(&property_name);
                factory.update_computed_property_name(
                    &property_name,
                    &factory.generated_name_node(&generated_name),
                )
            };
        }

        if has_static_modifier(&property) {
            self.current_class_element = Some(Arc::clone(&property));
        }

        if is_private_identifier(&property_name) && self.should_transform_class_element_to_weak_map_wt2b(&property) {
            if let Some(info) = self.access_private_identifier(&property_name) {
                if matches!(info.kind, PrivateIdentifierKind::Field) {
                    let visited_initializer = property
                        .initializer()
                        .map(|initializer| self.visitor().visit_node(initializer));
                    return if !info.is_static {
                        Some(self.create_private_instance_field_initializer(
                            receiver,
                            visited_initializer,
                            info.brand_check_identifier.as_ref().expect("field brand check identifier"),
                        ))
                    } else {
                        Some(self.create_private_static_field_initializer(
                            info.variable_name.as_ref().expect("static field variable name"),
                            visited_initializer,
                        ))
                    };
                }
                return None;
            }
            debug_assert!(false, "Undeclared private name for property declaration.");
        }

        if (is_private_identifier(&property_name) || has_static_modifier(&property))
            && property.initializer().is_none()
        {
            return None;
        }

        let property_original = self.emit_context.most_original(&property);
        if has_abstract_modifier(&property_original) {
            return None;
        }

        let mut initializer = property
            .initializer()
            .map(|initializer| self.visitor().visit_node(initializer));
        if is_parameter_property_declaration(&property_original, &property_original.parent().expect("property parent"))
            && is_identifier(&property_name)
        {
            let factory = self.factory();
            let local_name = factory.new_identifier(&property_name.text());
            if let Some(current_initializer) = initializer {
                let mut unwrapped = current_initializer;
                if is_parenthesized_expression(&unwrapped) {
                    if let Some(expression) = unwrapped.expression() {
                        if is_comma_like_expression(&expression) {
                            if let NodeData::BinaryExpression(binary) = &expression.data {
                                let is_run_initializers =
                                    self.emit_context.is_call_to_helper(&binary.left, "__runInitializers");
                                if is_run_initializers && is_void_zero_literal(&binary.right) {
                                    unwrapped = Arc::clone(&binary.left);
                                }
                            }
                        }
                    }
                }
                initializer = factory.inline_expressions(vec![unwrapped, local_name.clone()]);
            } else {
                initializer = Some(local_name.clone());
            }
            let property_name_node = property.name().expect("property declaration requires a name");
            self.emit_context
                .add_emit_flags(property_name_node, EmitFlags::NO_COMMENTS | EmitFlags::NO_SOURCE_MAP);
            let original_name = property_original.name().expect("property declaration requires a name");
            self.emit_context
                .set_source_map_range(&local_name, original_name.loc.clone());
            self.emit_context.add_emit_flags(&local_name, EmitFlags::NO_COMMENTS);
        } else if initializer.is_none() {
            initializer = Some(self.factory().new_void_zero_expression());
        }
        let initializer = initializer.expect("initializer should be set");

        if emit_assignment || is_private_identifier(&property_name) {
            let member_access = self.create_member_access_for_property_name(
                receiver,
                &property_name,
                Some(&property_name),
            );
            self.emit_context
                .add_emit_flags(&member_access, EmitFlags::NO_LEADING_COMMENTS);
            return Some(
                self.factory()
                    .new_assignment_expression(&member_access, &initializer),
            );
        }

        let name: Arc<Node> = if is_computed_property_name(&property_name) {
            Arc::clone(computed_property_name_expression(&property_name))
        } else if is_identifier(&property_name) {
            self.factory()
                .new_string_literal(&property_name.text(), TOKEN_FLAGS_NONE)
        } else {
            Arc::clone(&property_name)
        };
        let factory = self.factory();
        let descriptor = factory.new_object_literal_expression(
            &factory.new_node_list(vec![
                factory.new_property_assignment(
                    None,
                    &factory.new_identifier("enumerable"),
                    None,
                    None,
                    &factory.new_true_expression(),
                ),
                factory.new_property_assignment(
                    None,
                    &factory.new_identifier("configurable"),
                    None,
                    None,
                    &factory.new_true_expression(),
                ),
                factory.new_property_assignment(
                    None,
                    &factory.new_identifier("writable"),
                    None,
                    None,
                    &factory.new_true_expression(),
                ),
                factory.new_property_assignment(
                    None,
                    &factory.new_identifier("value"),
                    None,
                    None,
                    &initializer,
                ),
            ]),
            true,
        );
        Some(new_object_define_property_call_wt2b(&factory, receiver, &name, &descriptor))
    }

    fn should_transform_class_element_to_weak_map_wt2b(&self, node: &Arc<Node>) -> bool {
        if self.should_transform_private_elements_or_class_static_blocks {
            return true;
        }
        has_static_modifier(node)
            && self
                .emit_context
                .emit_flags(node)
                .contains(EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS)
    }
}

fn static_block_statements(node: &Arc<Node>) -> &[Arc<Node>] {
    match &node.data {
        NodeData::ClassStaticBlockDeclaration(static_block) => match &static_block.body.data {
            NodeData::Block(block) => &block.statements.nodes,
            _ => &[],
        },
        _ => &[],
    }
}

fn static_block_first_expression(node: &Arc<Node>) -> Option<&Arc<Node>> {
    static_block_statements(node).first().and_then(|s| s.expression())
}

fn static_block_statements_loc(node: &Arc<Node>) -> TextRange {
    match &node.data {
        NodeData::ClassStaticBlockDeclaration(static_block) => match &static_block.body.data {
            NodeData::Block(block) => block.statements.loc,
            _ => node.loc,
        },
        _ => node.loc,
    }
}

fn new_object_define_property_call_wt2b(
    factory: &NodeFactory,
    target: &Arc<Node>,
    name: &Arc<Node>,
    descriptor: &Arc<Node>,
) -> Arc<Node> {
    factory.new_call_expression(
        &factory.new_property_access_expression(
            &factory.new_identifier("Object"),
            None,
            &factory.new_identifier("defineProperty"),
            NodeFlags::empty(),
        ),
        None,
        None,
        factory.new_node_list(vec![target.clone(), name.clone(), descriptor.clone()]),
        NodeFlags::empty(),
    )
}

fn is_comma_like_expression(node: &Arc<Node>) -> bool {
    match &node.data {
        NodeData::BinaryExpression(binary) => binary.operator_token.kind == SyntaxKind::CommaToken,
        _ => false,
    }
}

fn is_void_zero_literal(node: &Arc<Node>) -> bool {
    is_void_expression(node)
        && node
            .expression()
            .is_some_and(|expression| is_numeric_literal(&expression))
}
