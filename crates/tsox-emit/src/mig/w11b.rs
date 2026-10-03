#![allow(invalid_reference_casting)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_computed_property_name, is_identifier, is_numeric_literal, is_parenthesized_expression,
    is_private_identifier, is_void_expression, NodeData,
};
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::mig::m3f_2::has_abstract_modifier;
use tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;
use crate::mig::m4g::ClassFieldsTransformer;
use crate::mig::m4g::r33k7_defs::{computed_property_name_expression, PrivateIdentifierKind};
use crate::mig::m4g::r39k15_defs::{ClassFieldsTransformerR39k15, NodeFactoryR39k15};
use crate::mig::m4q::r33k12_defs::{EmitFlags, has_decorators, is_private_identifier_class_element_declaration, skip_parentheses};
use crate::mig::m4g_2::r37k13_defs::ClassFieldsTransformerR37k13;
use crate::mig::m4h_2::{is_named_evaluation_and, transform_named_evaluation};
use crate::mig::m4m_2::is_simple_inlineable_expression;
use crate::printer::{AutoGenerateOptions, EmitContext, NodeFactory};

impl ClassFieldsTransformer {
    pub(crate) fn should_always_transform_private_static_elements(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_always_transform_private_static_elements"); 
        has_static_modifier(node)
            && self
                .emit_context()
                .emit_flags(node)
                .contains(EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS)
    }

    pub(crate) fn should_transform_class_element_to_weak_map(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_transform_class_element_to_weak_map"); 
        if self.should_transform_private_elements_or_class_static_blocks {
            return true;
        }
        self.should_always_transform_private_static_elements(node)
    }

    pub(crate) fn should_transform_auto_accessors_in_current_class(&self) -> bool { ::tsox_core::fntrace::enter("should_transform_auto_accessors_in_current_class"); 
        if self.should_transform_auto_accessors {
            return true;
        }
        // When targeting ESNext with useDefineForClassFields: false, auto-accessors are only
        // transformed if the current class will hoist initializers to the constructor.
        self.lexical_environment
            .as_ref()
            .and_then(|env| env.data.as_ref())
            .map(|data| data.facts.contains(crate::mig::m4g::ClassFacts::WillHoistInitializersToConstructor))
            .unwrap_or(false)
    }

    pub(crate) fn start_class_lexical_environment(&mut self) { ::tsox_core::fntrace::enter("start_class_lexical_environment"); 
        self.lexical_environment = Some(crate::mig::m4g::ClassLexicalEnv {
            previous: self.lexical_environment.take().map(Box::new),
            data: None,
            private_env: None,
        });
    }

    pub(crate) fn transform_property(
        &mut self,
        property: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_property"); 
        let saved_current_class_element = self.current_class_element.clone();
        let transformed = self.transform_property_worker(property, receiver);
        if let Some(transformed) = transformed.as_ref() {
            if has_static_modifier(property) {
                self.emit_context()
                    .add_emit_flags(transformed, EmitFlags::NO_LEXICAL_THIS);
            }
            if has_static_modifier(property)
                && self.lexical_environment.is_some()
                && self
                    .lexical_environment
                    .as_ref()
                    .and_then(|env| env.data.as_ref())
                    .map(|data| data.facts != crate::mig::m4g::ClassFacts::empty())
                    .unwrap_or(false)
            {
                // capture the lexical environment for the member
                self.emit_context().set_original(transformed, property);
                let name = property.name();
                if let Some(name) = name {
                    let range = self.emit_context().source_map_range(&name);
                    self.emit_context().set_source_map_range(transformed, range);
                }
            }
        }
        self.current_class_element = saved_current_class_element;
        transformed
    }

    pub(crate) fn transform_field_initializer(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_field_initializer"); 
        debug_assert!(
            !has_decorators(node),
            "Decorators should already have been transformed and elided."
        );
        if is_private_identifier_class_element_declaration(node) {
            return self.transform_private_field_initializer(node);
        }
        self.transform_public_field_initializer(node)
    }

    pub(crate) fn transform_property_worker(
        &mut self,
        property: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_property_worker"); 
        let emit_assignment = !self.compiler_options.get_use_define_for_class_fields();
        let emit_context = self.emit_context();
        let factory = NodeFactory::new(&emit_context);

        let mut property = Arc::clone(property);
        let named_evaluation = {
            let this = &*self;
            let cb = |n: &Arc<Node>| (this.is_anonymous_class_needing_assigned_name)(this, n);
            is_named_evaluation_and(&self.emit_context(), &property, Some(&cb))
        };
        if named_evaluation {
            property = transform_named_evaluation(&self.emit_context(), &property, false, "");
        }

        let mut property_name = property
            .name()
            .expect("property declaration requires a name")
            .clone();
        if has_syntactic_modifier(&property, ModifierFlags::Accessor) {
            property_name = factory.generated_name_node(&factory.new_generated_private_name_for_node_ex(
                property
                    .name()
                    .as_ref()
                    .expect("property declaration requires a name"),
                AutoGenerateOptions {
                    suffix: "_accessor_storage".to_string(),
                    ..Default::default()
                },
            ));
        } else if is_computed_property_name(&property_name)
            && !is_simple_inlineable_expression(computed_property_name_expression(&property_name))
        {
            let generated_name = factory.new_generated_name_for_node(&property_name);
            property_name = factory.update_computed_property_name(
                &property_name,
                &factory.generated_name_node(&generated_name),
            );
        }

        if has_static_modifier(&property) {
            self.current_class_element = Some(Arc::clone(&property));
        }

        if is_private_identifier(&property_name)
            && self.should_transform_class_element_to_weak_map(&property)
        {
            if let Some(info) = self.access_private_identifier(&property_name) {
                if matches!(info.kind, PrivateIdentifierKind::Field) {
                    let visited_initializer = property
                        .initializer()
                        .map(|initializer| self.visitor().visit_node(initializer));
                    return if !info.is_static {
                        Some(create_private_instance_field_initializer_w11b(
                            &factory,
                            receiver,
                            visited_initializer,
                            info.brand_check_identifier
                                .as_ref()
                                .expect("field brand check identifier"),
                        ))
                    } else {
                        Some(create_private_static_field_initializer_w11b(
                            &factory,
                            info.variable_name
                                .as_ref()
                                .expect("static field variable name"),
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

        let property_original = self.emit_context().most_original(&property);
        if has_abstract_modifier(&property_original) {
            return None;
        }

        let mut initializer = property
            .initializer()
            .map(|initializer| self.visitor().visit_node(initializer));
        if is_parameter_property_declaration(
            &property_original,
            &property_original.parent().expect("property parent"),
        ) && is_identifier(&property_name)
        {
            let local_name = factory.new_identifier(&property_name.text());
            if let Some(current_initializer) = initializer {
                let mut unwrapped = current_initializer;
                if is_parenthesized_expression(&unwrapped) {
                    if let Some(expression) = unwrapped.expression() {
                        if is_comma_like_expression_w11b(&expression) {
                            if let NodeData::BinaryExpression(binary) = &expression.data {
                                let is_run_initializers = self
                                    .emit_context()
                                    .is_call_to_helper(&binary.left, "__runInitializers");
                                if is_run_initializers && is_void_zero_literal_w11b(&binary.right) {
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
            self.emit_context().add_emit_flags(
                property_name_node,
                EmitFlags::NO_COMMENTS | EmitFlags::NO_SOURCE_MAP,
            );
            let original_name = property_original
                .name()
                .expect("property declaration requires a name");
            self.emit_context()
                .set_source_map_range(&local_name, original_name.loc);
            self.emit_context()
                .add_emit_flags(&local_name, EmitFlags::NO_COMMENTS);
        } else if initializer.is_none() {
            initializer = Some(factory.new_void_zero_expression());
        }
        let initializer = initializer.expect("initializer should be set");

        if emit_assignment || is_private_identifier(&property_name) {
            let member_access = create_member_access_for_property_name_w11b(
                &factory,
                &emit_context,
                receiver,
                &property_name,
                &property_name,
            );
            self.emit_context()
                .add_emit_flags(&member_access, EmitFlags::NO_LEADING_COMMENTS);
            return Some(factory.new_assignment_expression(&member_access, &initializer));
        }

        let name: Arc<Node> = if is_computed_property_name(&property_name) {
            Arc::clone(computed_property_name_expression(&property_name))
        } else if is_identifier(&property_name) {
            factory.new_string_literal(&property_name.text(), TOKEN_FLAGS_NONE)
        } else {
            Arc::clone(&property_name)
        };
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
        Some(new_object_define_property_call_w11b(&factory, receiver, &name, &descriptor))
    }
}

fn create_private_static_field_initializer_w11b(
    factory: &NodeFactory,
    variable_name: &Arc<Node>,
    initializer: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_static_field_initializer_w11b"); 
    let initializer = initializer.unwrap_or_else(|| factory.new_void_zero_expression());
    factory.new_assignment_expression(
        variable_name,
        &factory.new_object_literal_expression(
            &factory.new_node_list(vec![factory.new_property_assignment(
                None,
                &factory.new_identifier("value"),
                None,
                None,
                &initializer,
            )]),
            false,
        ),
    )
}

fn create_private_instance_field_initializer_w11b(
    factory: &NodeFactory,
    receiver: &Arc<Node>,
    initializer: Option<Arc<Node>>,
    weak_map_name: &Arc<Node>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_instance_field_initializer_w11b"); 
    let initializer = initializer.unwrap_or_else(|| factory.new_void_zero_expression());
    factory.new_method_call(
        weak_map_name,
        &factory.new_identifier("set"),
        vec![receiver.clone(), initializer],
    )
}

fn emit_context_mut(emit_context: &EmitContext) -> &mut EmitContext { ::tsox_core::fntrace::enter("emit_context_mut"); 
    unsafe { &mut *(emit_context as *const EmitContext as *mut EmitContext) }
}

fn create_member_access_for_property_name_w11b(
    factory: &NodeFactory,
    emit_context: &EmitContext,
    receiver: &Arc<Node>,
    name: &Arc<Node>,
    location: &Arc<Node>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("create_member_access_for_property_name_w11b"); 
    let expression = if is_computed_property_name(name) {
        let mut expression = factory.new_element_access_expression(
            receiver,
            None,
            computed_property_name_expression(name),
            NodeFlags::empty(),
        );
        if let Some(e) = Arc::get_mut(&mut expression) {
            e.loc = location.loc;
        }
        return expression;
    } else if is_identifier(name) || is_private_identifier(name) {
        factory.new_property_access_expression(receiver, None, name, NodeFlags::empty())
    } else {
        factory.new_element_access_expression(receiver, None, name, NodeFlags::empty())
    };
    let emit_context = emit_context_mut(emit_context);
    emit_context.set_comment_range(&expression, name.loc);
    emit_context.set_source_map_range(&expression, name.loc);
    emit_context.add_emit_flags(&expression, EmitFlags::NO_NESTED_SOURCE_MAPS);
    expression
}

fn new_object_define_property_call_w11b(
    factory: &NodeFactory,
    target: &Arc<Node>,
    name: &Arc<Node>,
    descriptor: &Arc<Node>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_object_define_property_call_w11b"); 
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

fn is_comma_like_expression_w11b(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_comma_like_expression_w11b"); 
    match &node.data {
        NodeData::BinaryExpression(binary) => binary.operator_token.kind == SyntaxKind::CommaToken,
        _ => false,
    }
}

fn is_void_zero_literal_w11b(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_void_zero_literal_w11b"); 
    is_void_expression(node)
        && node
            .expression()
            .is_some_and(|expression| is_numeric_literal(&expression))
}

pub(crate) fn should_be_captured_in_temp_variable(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_be_captured_in_temp_variable"); 
    let target = skip_parentheses(node);
    match target.kind {
        SyntaxKind::Identifier
        | SyntaxKind::ThisKeyword
        | SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::StringLiteral => false,
        _ => true,
    }
}
