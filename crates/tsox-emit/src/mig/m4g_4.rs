#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::{is_private_identifier, is_property_access_expression};
use tsox_frontend::ast::mig::m3f_4::is_destructuring_assignment;
use tsox_frontend::ast::mig::m3g_2::is_super_property;
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use crate::mig::m4g::{ClassFieldsTransformer, ClassFacts};
use crate::mig::m4g_2::r37k13_defs::ClassFieldsTransformerR37k13;
use crate::mig::m4g::r39k15_defs::{
    ClassFieldsTransformerR39k15, K13VisitorR39k15, NodeFactoryR39k15,
};
use crate::mig::m4g::r40k14_defs::{ClassFieldsTransformerR40k14, K13VisitorR40k14};
use crate::mig::m4g::r33k7_defs::{
    binary_left, binary_operator_token, binary_right, call_expression_arguments,
    call_expression_expression, parenthesized_expression, property_access_expression,
    property_access_name, property_access_question_dot_token, tagged_template_tag,
    tagged_template_template,
};
use crate::mig::m4h_2::{is_named_evaluation_and, transform_named_evaluation};
use crate::mig::w7t::is_static_property_declaration_or_class_static_block;

fn with_loc(mut node: Arc<Node>, loc: TextRange) -> Arc<Node> {
    if let Some(n) = Arc::get_mut(&mut node) {
        n.loc = loc;
    }
    node
}

impl ClassFieldsTransformer {
    pub fn visit_call_expression(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let call_target = call_expression_expression(node);
        if is_property_access_expression(call_target)
            && is_private_identifier(property_access_name(call_target))
            && self
                .access_private_identifier(property_access_name(call_target))
                .is_some()
        {
            let (this_arg, target) = self.create_call_binding(call_target);
            let visited_target = self.visitor().visit_node(&target);
            let visited_this_arg = self.visitor().visit_node(&this_arg);
            let visited_args = self.visitor().visit_nodes(call_expression_arguments(node));
            let mut all_args = Vec::with_capacity(1 + visited_args.len());
            all_args.push(visited_this_arg);
            all_args.extend(visited_args.iter().cloned());
            let flags = node.flags;
            if flags.contains(NodeFlags::OptionalChain) {
                let call = self.factory().new_property_access_expression(
                    &visited_target,
                    property_access_question_dot_token(call_target).cloned(),
                    &self.factory().new_identifier("call"),
                    NodeFlags::OptionalChain,
                );
                return self.factory().update_call_expression(
                    node,
                    &call,
                    None,
                    None,
                    self.factory().new_node_list(&all_args),
                    flags,
                );
            }
            let call = self.factory().new_property_access_expression(
                &visited_target,
                None,
                &self.factory().new_identifier("call"),
                NodeFlags::empty(),
            );
            return self.factory().update_call_expression(
                node,
                &call,
                None,
                None,
                self.factory().new_node_list(&all_args),
                flags,
            );
        }

        if self.should_transform_super_in_static_initializers
            && self.current_class_element.is_some()
            && is_super_property(call_target)
            && is_static_property_declaration_or_class_static_block(
                self.current_class_element.as_ref().unwrap(),
            )
            && self
                .lexical_environment
                .as_ref()
                .and_then(|env| env.data.as_ref())
                .map(|data| data.class_constructor.is_some())
                .unwrap_or(false)
        {
            let visited_target = self.visitor().visit_node(call_target);
            let class_constructor = self
                .lexical_environment
                .as_ref()
                .and_then(|env| env.data.as_ref())
                .and_then(|data| data.class_constructor.clone())
                .unwrap();
            let visited_args = self.visitor().visit_nodes(call_expression_arguments(node));
            let invocation = self.factory().new_function_call_call(
                &visited_target,
                &class_constructor,
                &visited_args.nodes,
            );
            self.emit_context().set_original(&invocation, node);
            return with_loc(invocation, node.loc);
        }

        self.visitor().visit_each_child(node)
    }

    pub fn visit_tagged_template_expression(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let tag = tagged_template_tag(node);
        if is_property_access_expression(tag)
            && is_private_identifier(property_access_name(tag))
            && self
                .access_private_identifier(property_access_name(tag))
                .is_some()
        {
            let (this_arg, target) = self.create_call_binding(tag);
            let visited_target = self.visitor().visit_node(&target);
            let bind = self.factory().new_property_access_expression(
                &visited_target,
                None,
                &self.factory().new_identifier("bind"),
                NodeFlags::empty(),
            );
            let bind_expr = self.factory().new_call_expression(
                &bind,
                None,
                None,
                self.factory()
                    .new_node_list(&[self.visitor().visit_node(&this_arg)]),
                NodeFlags::empty(),
            );
            let visited_template = self.visitor().visit_node(tagged_template_template(node));
            return self.factory().update_tagged_template_expression(
                node,
                &bind_expr,
                None,
                None,
                &visited_template,
                node.flags,
            );
        }

        if self.should_transform_super_in_static_initializers
            && self.current_class_element.is_some()
            && is_super_property(tag)
            && is_static_property_declaration_or_class_static_block(
                self.current_class_element.as_ref().unwrap(),
            )
            && self
                .lexical_environment
                .as_ref()
                .and_then(|env| env.data.as_ref())
                .map(|data| data.class_constructor.is_some())
                .unwrap_or(false)
        {
            let visited_tag = self.visitor().visit_node(tag);
            let class_constructor = self
                .lexical_environment
                .as_ref()
                .and_then(|env| env.data.as_ref())
                .and_then(|data| data.class_constructor.clone())
                .unwrap();
            let invocation = self
                .factory()
                .new_function_bind_call(&visited_tag, &class_constructor, None);
            self.emit_context().set_original(&invocation, node);
            let located = with_loc(invocation, node.loc);
            let visited_template = self.visitor().visit_node(tagged_template_template(node));
            return self.factory().update_tagged_template_expression(
                node,
                &located,
                None,
                None,
                &visited_template,
                node.flags,
            );
        }

        self.visitor().visit_each_child(node)
    }

    pub fn visit_binary_expression(&mut self, node: &Arc<Node>, discarded: bool) -> Arc<Node> {
        if is_destructuring_assignment(node) {
            let saved_pending_expressions = std::mem::take(&mut self.pending_expressions);
            let updated = self.factory().update_binary_expression(
                node,
                None,
                self.assignment_target_visitor().visit_node(binary_left(node)),
                None,
                binary_operator_token(node).clone(),
                self.visitor().visit_node(binary_right(node)),
            );
            let result = if !self.pending_expressions.is_empty() {
                let mut exprs = std::mem::take(&mut self.pending_expressions);
                exprs.push(updated);
                self.factory().inline_expressions(exprs)
            } else {
                updated
            };
            self.pending_expressions = saved_pending_expressions;
            return result;
        }

        if is_assignment_expression(node, false) {
            let mut node = node.clone();
            let this = &*self;
            let cb = |n: &Arc<Node>| (this.is_anonymous_class_needing_assigned_name)(this, n);
            if is_named_evaluation_and(&self.emit_context(), &node, Some(&cb)) {
                node = transform_named_evaluation(&self.emit_context(), &node, false, "");
                debug_assert!(is_assignment_expression(&node, false));
            }

            let left = &skip_outer_expressions(
                binary_left(&node),
                OuterExpressionKinds::PARTIALLY_EMITTED_EXPRESSIONS | OuterExpressionKinds::PARENS,
            );
            if is_property_access_expression(left) && is_private_identifier(property_access_name(left)) {
                let info = self.access_private_identifier(property_access_name(left));
                if let Some(info) = info {
                    let result = self.create_private_identifier_assignment(
                        &info,
                        property_access_expression(left),
                        binary_right(&node),
                        binary_operator_token(&node).kind,
                    );
                    self.emit_context().set_original(&result, &node);
                    return with_loc(result, node.loc);
                }
            } else if self.should_transform_super_in_static_initializers
                && self.current_class_element.is_some()
                && is_super_property(binary_left(&node))
                && is_static_property_declaration_or_class_static_block(
                    self.current_class_element.as_ref().unwrap(),
                )
            {
                let data = self
                    .lexical_environment
                    .as_ref()
                    .and_then(|lex| lex.data.clone());
                if let Some(data) = data {
                    let result = self.visit_binary_super_assignment(&node, discarded, &data, left);
                    if let Some(result) = result {
                        return result;
                    }
                }
            }
        }

        if binary_operator_token(node).kind == SyntaxKind::InKeyword
            && is_private_identifier(binary_left(node))
        {
            return self.transform_private_identifier_in_in_expression(node);
        }

        self.visitor().visit_each_child(node)
    }

    pub fn visit_parenthesized_expression(&mut self, node: &Arc<Node>, discarded: bool) -> Arc<Node> {
        let expression = parenthesized_expression(node);
        let visited = if discarded {
            self.discarded_value_visitor().visit_node(expression)
        } else {
            self.visitor().visit_node(expression)
        };
        self.factory()
            .update_parenthesized_expression(node, &visited)
    }
}

use crate::printer::{
    AutoGenerateOptions, GeneratedIdentifierFlags, NodeFactory as PrinterNodeFactory,
};
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::{
    is_class_static_block_declaration,
    is_computed_property_name, is_constructor_declaration, is_get_accessor_declaration,
    is_method_declaration, is_property_declaration, is_set_accessor_declaration,
    for_each_child,
};
use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
use tsox_frontend::ast::subtree_facts::{
    SubtreeContainsLexicalSuper, SubtreeContainsLexicalThis,
};
use tsox_frontend::ast::{has_syntactic_modifier, is_assignment_expression, is_class_like, ModifierFlags};
use tsox_frontend::ast::mig::m3b::members;
use tsox_frontend::ast::mig::m3f_2::has_abstract_modifier;
use tsox_frontend::ast::node_source_file::LanguageVariant;
use tsox_frontend::scanner::mig::m3i::is_identifier_text;
use tsox_checker::checker::mig::wc1b::{class_or_constructor_parameter_is_decorated, is_initialized_property};
use crate::mig::m4j_2::extract_modifiers;
use crate::mig::m4h_2::class_has_explicitly_assigned_name;
use crate::mig::m4q::r33k12_defs::{is_private_identifier_class_element_declaration, skip_parentheses};
use tsox_frontend::format::mig::m4o::EmitFlags;
use crate::mig::wt1b_4::class_has_class_this_assignment;
use crate::mig::x6a::is_class_this_assignment_block;
use tsox_frontend::format::mig::m4o;

impl ClassFieldsTransformer {
    pub fn visit_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> {
        self.visit_in_new_class_lexical_environment(
            node,
            Self::visit_class_declaration_in_new_class_lexical_environment,
        )
    }

    pub fn visit_class_expression(&mut self, node: &Arc<Node>) -> Arc<Node> {
        self.visit_in_new_class_lexical_environment(
            node,
            Self::visit_class_expression_in_new_class_lexical_environment,
        )
    }

    fn new_reserved_temp_variable_m4g4(&self) -> Arc<Node> {
        let emit_context = self.emit_context();
        let factory = PrinterNodeFactory::new(&emit_context);
        factory.generated_name_node(&factory.new_temp_variable_ex(AutoGenerateOptions {
            flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
            ..Default::default()
        }))
    }

    fn get_local_name_m4g4(&self, node: &Arc<Node>) -> Arc<Node> {
        let emit_context = self.emit_context();
        let factory = PrinterNodeFactory::new(&emit_context);
        factory.get_local_name(node)
    }

    fn visit_in_new_class_lexical_environment(
        &mut self,
        node: &Arc<Node>,
        visitor: fn(&mut Self, &Arc<Node>, ClassFacts) -> Arc<Node>,
    ) -> Arc<Node> {
        let saved_current_class_container = self.current_class_container.take();
        let saved_pending_expressions = std::mem::take(&mut self.pending_expressions);
        let saved_lexical_environment = self.lexical_environment.take();
        self.current_class_container = Some(Arc::clone(node));
        self.start_class_lexical_environment();
        let original = self.emit_context().most_original(node);
        let original_key = Arc::as_ptr(&original) as usize;
        self.enclosing_class_declarations.insert(original_key);

        if self.should_transform_private_elements_or_class_static_blocks
            || self.node_has_transform_private_static_elements_flag(node)
        {
            let name = node.name();
            if let Some(name) = name.filter(|n| tsox_frontend::ast::is_identifier(n)) {
                self.get_private_identifier_environment().data.class_name = Some(name.clone());
            } else if let Some(assigned_name) = self.emit_context().assigned_name(node) {
                if assigned_name.kind == SyntaxKind::StringLiteral {
                    let text_source = self.emit_context().text_source(&assigned_name);
                    match text_source {
                        Some(text_source) if tsox_frontend::ast::is_identifier(&text_source) => {
                            self.get_private_identifier_environment().data.class_name = Some(text_source);
                        }
                        _ => {
                            if is_identifier_text(&assigned_name.text(), LanguageVariant::Standard) {
                                let prefix_name = self.factory().new_identifier(&assigned_name.text());
                                self.get_private_identifier_environment().data.class_name = Some(prefix_name);
                            }
                        }
                    }
                }
            }
        }

        if self.should_transform_private_elements_or_class_static_blocks {
            let private_instance_methods_and_accessors =
                get_private_instance_methods_and_accessors_m4g4(node);
            if let Some(first) = private_instance_methods_and_accessors.first() {
                let first_name = first
                    .name()
                    .expect("private method or accessor requires a name");
                let weak_set_name =
                    self.create_hoisted_variable_for_class("instances", &first_name, "");
                self.get_private_identifier_environment().data.weak_set_name = Some(weak_set_name);
            }
        }

        let facts = self.get_class_facts(node);
        if facts != ClassFacts::empty() {
            self.get_class_lexical_environment().facts = facts;
        }

        let result = visitor(self, node, facts);
        self.enclosing_class_declarations.remove(&original_key);
        self.end_class_lexical_environment();
        self.current_class_container = saved_current_class_container;
        self.pending_expressions = saved_pending_expressions;
        self.lexical_environment = saved_lexical_environment;
        result
    }

    fn visit_class_declaration_in_new_class_lexical_environment(
        &mut self,
        node: &Arc<Node>,
        facts: ClassFacts,
    ) -> Arc<Node> {
        let mut pending_class_reference_assignment: Option<Arc<Node>> = None;
        if facts.contains(ClassFacts::NeedsClassConstructorReference) {
            let class_this = self.emit_context().class_this(node);
            if self.should_transform_private_elements_or_class_static_blocks && class_this.is_some()
            {
                let class_this = class_this.unwrap();
                self.get_class_lexical_environment().class_constructor = Some(class_this.clone());
                let local_name = self.get_local_name_m4g4(node);
                pending_class_reference_assignment =
                    Some(self.factory().new_assignment_expression(&class_this, &local_name));
            } else {
                let temp = self.new_reserved_temp_variable_m4g4();
                self.emit_context().add_variable_declaration(&temp);
                self.get_class_lexical_environment().class_constructor = Some(temp.clone());
                let local_name = self.get_local_name_m4g4(node);
                pending_class_reference_assignment =
                    Some(self.factory().new_assignment_expression(&temp, &local_name));
            }
        }

        let class_this = self.emit_context().class_this(node);
        if let Some(class_this) = class_this {
            self.get_class_lexical_environment().class_this = Some(class_this);
        }

        let is_class_with_constructor_reference = self.class_contains_constructor_reference(node);

        let alias = self.get_class_lexical_environment().class_constructor.clone();
        if let Some(alias) = alias {
            if is_class_with_constructor_reference {
                let original = self.emit_context().most_original(node);
                self.class_aliases
                    .insert(Arc::as_ptr(&original) as usize, alias);
            }
        }

        let mut modifiers = self
            .modifier_visitor()
            .visit_modifiers_list(node.modifiers());
        let heritage_clauses = get_heritage_clauses(node)
            .map(|hc| self.heritage_clause_visitor().visit_nodes_list(hc))
            .flatten();
        let (members_list, members_prologue) = self.transform_class_members(node);

        let mut statements: Vec<Arc<Node>> = Vec::new();
        if let Some(pending) = pending_class_reference_assignment {
            let mut exprs = vec![pending];
            exprs.extend(self.pending_expressions.drain(..));
            self.pending_expressions = exprs;
        }

        if !self.pending_expressions.is_empty() {
            let exprs = std::mem::take(&mut self.pending_expressions);
            statements.push(
                self.factory()
                    .new_expression_statement(&self.factory().inline_expressions(exprs)),
            );
        }

        let mut name = node.name();
        let mut generated_name: Option<Arc<Node>> = None;
        if self.should_transform_initializers_using_set
            || self.should_transform_private_elements_or_class_static_blocks
        {
            let static_properties = self.get_static_properties_and_class_static_block(node);
            if !static_properties.is_empty() {
                if name.is_none() {
                    let emit_context = self.emit_context();
                    let factory = PrinterNodeFactory::new(&emit_context);
                    generated_name = Some(
                        factory.generated_name_node(&factory.new_generated_name_for_node(node)),
                    );
                    name = generated_name.as_ref();
                }
                let local_name = self.get_local_name_m4g4(node);
                statements = self.add_property_or_class_static_block_statements(
                    statements,
                    &static_properties,
                    &local_name,
                );
            }
        }

        let is_export = has_syntactic_modifier(node, ModifierFlags::Export);
        let is_default = has_syntactic_modifier(node, ModifierFlags::Default);
        if !statements.is_empty() && is_export && is_default {
            let emit_context = Arc::new(self.emit_context());
            modifiers = extract_modifiers(
                &emit_context,
                modifiers.as_ref().map(|m| m.as_ref()),
                !(ModifierFlags::Export | ModifierFlags::Default),
            )
            .map(Arc::new);
            let local_name = self.get_local_name_m4g4(node);
            let emit_context_for_export = self.emit_context();
            let factory = PrinterNodeFactory::new(&emit_context_for_export);
            statements.push(factory.new_export_assignment(None, false, None, &local_name));
        }

        let updated_class = {
            let emit_context = self.emit_context();
            let factory = PrinterNodeFactory::new(&emit_context);
            factory.update_class_declaration(
                node,
                modifiers,
                name,
                None,
                heritage_clauses.as_deref(),
                &members_list,
            )
        };

        let mut result: Vec<Arc<Node>> = Vec::with_capacity(statements.len() + 2);
        if let Some(prologue) = members_prologue {
            result.push(self.factory().new_expression_statement(&prologue));
        }
        result.push(updated_class);
        result.extend(statements);
        self.factory().new_syntax_list(result)
    }

    fn visit_class_expression_in_new_class_lexical_environment(
        &mut self,
        node: &Arc<Node>,
        facts: ClassFacts,
    ) -> Arc<Node> {
        let is_decorated_class_declaration = facts.contains(ClassFacts::ClassWasDecorated);

        let class_this = self.emit_context().class_this(node);
        if let Some(class_this) = &class_this {
            self.get_class_lexical_environment().class_this = Some(class_this.clone());
        }

        let mut temp: Option<Arc<Node>> = None;
        if facts.contains(ClassFacts::NeedsClassConstructorReference) {
            if (self.should_transform_private_elements_or_class_static_blocks
                || self.node_has_transform_private_static_elements_flag(node))
                && class_this.is_some()
            {
                let class_this = class_this.clone().expect("class this checked above");
                self.get_class_lexical_environment().class_constructor = Some(class_this.clone());
                temp = Some(class_this);
            } else {
                let new_temp = self.new_reserved_temp_variable_m4g4();
                if self.class_expression_needs_block_scoped_temp() {
                    self.emit_context().add_lexical_declaration(&new_temp);
                } else {
                    self.emit_context().add_variable_declaration(&new_temp);
                }
                self.get_class_lexical_environment().class_constructor = Some(new_temp.clone());
                temp = Some(new_temp);
            }
        }

        let static_properties_or_class_static_blocks =
            self.get_static_properties_and_class_static_block(node);

        let mut is_class_with_constructor_reference = false;
        let mut has_transformable_statics = false;
        let mut defer_temp_declaration = false;
        if !is_decorated_class_declaration {
            is_class_with_constructor_reference = self.class_contains_constructor_reference(node);
            has_transformable_statics =
                (self.should_transform_private_elements_or_class_static_blocks
                    || self.node_has_transform_private_static_elements_flag(node))
                    && static_properties_or_class_static_blocks.iter().any(|n| {
                        is_class_static_block_declaration(n)
                            || is_private_identifier_class_element_declaration(n)
                            || (self.should_transform_initializers && is_initialized_property(n))
                    });

            let will_have_private_pending_expressions =
                self.should_transform_private_elements_or_class_static_blocks
                    && members(node).iter().any(|n| {
                        is_private_identifier_class_element_declaration(n)
                            && !has_static_modifier(n)
                            && self.should_transform_class_element_to_weak_map(n)
                    });
            let will_need_temp_wrapper =
                has_transformable_statics || will_have_private_pending_expressions;

            if will_need_temp_wrapper
                && self
                    .get_class_lexical_environment()
                    .class_constructor
                    .is_none()
            {
                let new_temp = self.new_reserved_temp_variable_m4g4();
                defer_temp_declaration = true;
                self.get_class_lexical_environment().class_constructor = Some(new_temp.clone());
                temp = Some(new_temp);
            }
            if will_need_temp_wrapper {
                let alias = self
                    .get_class_lexical_environment()
                    .class_constructor
                    .clone();
                if let Some(alias) = alias {
                    if is_class_with_constructor_reference {
                        let original = self.emit_context().most_original(node);
                        self.class_aliases
                            .insert(Arc::as_ptr(&original) as usize, alias);
                    }
                }
            }
        }

        let modifiers = self
            .modifier_visitor()
            .visit_modifiers_list(node.modifiers());
        let heritage_clauses = get_heritage_clauses(node)
            .map(|hc| self.heritage_clause_visitor().visit_nodes_list(hc))
            .flatten();
        let (members_list, members_prologue) = self.transform_class_members(node);

        if defer_temp_declaration {
            if let Some(temp) = &temp {
                if self.class_expression_needs_block_scoped_temp() {
                    self.emit_context().add_lexical_declaration(temp);
                } else {
                    self.emit_context().add_variable_declaration(temp);
                }
            }
        }

        let class_expression = {
            let emit_context = self.emit_context();
            let factory = PrinterNodeFactory::new(&emit_context);
            factory.update_class_expression(
                node,
                modifiers,
                node.name(),
                None,
                heritage_clauses.as_deref(),
                &members_list,
            )
        };

        let class_expression_for_flags = class_expression.clone();
        let mut expressions: Vec<Arc<Node>> = Vec::new();
        if let Some(prologue) = members_prologue {
            expressions.push(prologue);
        }

        if !is_decorated_class_declaration {
            if has_transformable_statics || !self.pending_expressions.is_empty() {
                let temp = match temp {
                    Some(temp) => Some(temp),
                    None => {
                        let new_temp = self.new_reserved_temp_variable_m4g4();
                        if self.class_expression_needs_block_scoped_temp() {
                            self.emit_context().add_lexical_declaration(&new_temp);
                        } else {
                            self.emit_context().add_variable_declaration(&new_temp);
                        }
                        self.get_class_lexical_environment().class_constructor = Some(new_temp.clone());
                        if is_class_with_constructor_reference {
                            let original = self.emit_context().most_original(node);
                            self.class_aliases
                                .insert(Arc::as_ptr(&original) as usize, new_temp.clone());
                        }
                        Some(new_temp)
                    }
                };
                let temp = temp.expect("temp should be set");

                expressions.push(self.factory().new_assignment_expression(&temp, &class_expression));
                expressions.extend(self.pending_expressions.drain(..));
                expressions.extend(self.generate_initialized_property_expressions_or_class_static_block(
                    &static_properties_or_class_static_blocks,
                    &temp,
                ));
                expressions.push(temp);
            } else {
                expressions.push(class_expression);
            }
        } else {
            if !self.pending_expressions.is_empty() {
                let exprs = std::mem::take(&mut self.pending_expressions);
                for expr in exprs {
                    self.pending_statements
                        .push(self.factory().new_expression_statement(&expr));
                }
            }

            if !static_properties_or_class_static_blocks.is_empty() {
                let class_this_or_name = match &class_this {
                    Some(class_this) => class_this.clone(),
                    None => self.get_local_name_m4g4(node),
                };
                let pending = std::mem::take(&mut self.pending_statements);
                self.pending_statements = self.add_property_or_class_static_block_statements(
                    pending,
                    &static_properties_or_class_static_blocks,
                    &class_this_or_name,
                );
            }

            if let Some(temp) = &temp {
                expressions.push(self.factory().new_assignment_expression(temp, &class_expression));
            } else if self.should_transform_private_elements_or_class_static_blocks
                && class_this.is_some()
            {
                expressions.push(self.factory().new_assignment_expression(
                    class_this.as_ref().unwrap(),
                    &class_expression,
                ));
            } else {
                expressions.push(class_expression);
            }
        }

        if expressions.len() > 1 {
            self.emit_context()
                .add_emit_flags(&class_expression_for_flags, EmitFlags::INDENTED);
            for expr in &expressions {
                self.emit_context().add_emit_flags(expr, EmitFlags::START_ON_NEW_LINE);
            }
        }
        self.factory().inline_expressions(expressions)
    }

    fn get_class_facts(&mut self, node: &Arc<Node>) -> ClassFacts {
        let mut facts = ClassFacts::empty();

        let original = self.emit_context().most_original(node);
        if is_class_like(&original)
            && class_or_constructor_parameter_is_decorated(false, &original)
        {
            facts |= ClassFacts::ClassWasDecorated;
        }

        if self.should_transform_private_elements_or_class_static_blocks
            && (class_has_class_this_assignment(&self.emit_context(), node)
                || class_has_explicitly_assigned_name(&self.emit_context(), node))
        {
            facts |= ClassFacts::NeedsClassConstructorReference;
        }

        let mut contains_public_instance_fields = false;
        let mut contains_initialized_public_instance_fields = false;
        let mut contains_instance_private_elements = false;
        let mut contains_instance_auto_accessors = false;

        for member in members(node) {
            if has_static_modifier(member) {
                let member_name = member.name();
                if member_name.is_some()
                    && (is_private_identifier(member_name.as_ref().unwrap())
                        || is_auto_accessor_property_declaration(member))
                    && self.should_transform_private_elements_or_class_static_blocks
                {
                    facts |= ClassFacts::NeedsClassConstructorReference;
                } else if is_auto_accessor_property_declaration(member)
                    && self.should_transform_auto_accessors
                    && node.name().is_none()
                    && self.emit_context().class_this(node).is_none()
                {
                    facts |= ClassFacts::NeedsClassConstructorReference;
                }
                if is_property_declaration(member) || is_class_static_block_declaration(member) {
                    if self.should_transform_this_in_static_initializers
                        && member.subtree_facts().intersects(SubtreeContainsLexicalThis)
                    {
                        facts |= ClassFacts::NeedsSubstitutionForThisInClassStaticField;
                        if !facts.contains(ClassFacts::ClassWasDecorated) {
                            facts |= ClassFacts::NeedsClassConstructorReference;
                        }
                    }
                    if self.should_transform_super_in_static_initializers
                        && member.subtree_facts().intersects(SubtreeContainsLexicalSuper)
                    {
                        if !facts.contains(ClassFacts::ClassWasDecorated) {
                            facts |= ClassFacts::NeedsClassConstructorReference
                                | ClassFacts::NeedsClassSuperReference;
                        }
                    }
                }
            } else if !has_abstract_modifier(&self.emit_context().most_original(member)) {
                if is_auto_accessor_property_declaration(member) {
                    contains_instance_auto_accessors = true;
                    contains_instance_private_elements = contains_instance_private_elements
                        || is_private_identifier_class_element_declaration(member);
                } else if is_private_identifier_class_element_declaration(member) {
                    contains_instance_private_elements = true;
                    if self.member_contains_constructor_reference(member, node) {
                        facts |= ClassFacts::NeedsClassConstructorReference;
                    }
                } else if is_property_declaration(member) {
                    contains_public_instance_fields = true;
                    contains_initialized_public_instance_fields =
                        contains_initialized_public_instance_fields
                            || member.initializer().is_some();
                }
            }
        }

        let will_hoist_initializers_to_constructor =
            (self.should_transform_initializers_using_define && contains_public_instance_fields)
                || (self.should_transform_initializers_using_set
                    && contains_initialized_public_instance_fields)
                || (self.should_transform_private_elements_or_class_static_blocks
                    && contains_instance_private_elements)
                || (self.should_transform_private_elements_or_class_static_blocks
                    && contains_instance_auto_accessors
                    && self.should_transform_auto_accessors);

        if will_hoist_initializers_to_constructor {
            facts |= ClassFacts::WillHoistInitializersToConstructor;
        }

        facts
    }

    fn class_contains_constructor_reference(&self, node: &Arc<Node>) -> bool {
        for member in members(node) {
            if self.member_contains_constructor_reference(member, node) {
                return true;
            }
        }
        false
    }

    fn member_contains_constructor_reference(
        &self,
        member: &Arc<Node>,
        class_decl: &Arc<Node>,
    ) -> bool {
        let class_original = self.emit_context().most_original(class_decl);
        let class_name = class_decl.name();
        let check =
            |n: &Arc<Node>| -> bool {
                self.contains_constructor_reference_worker(n, &class_original, class_name)
            };
        if is_class_static_block_declaration(member) {
            if let Some(body) = class_static_block_body(member) {
                for stmt in block_statement_nodes(&body) {
                    if check(&stmt) {
                        return true;
                    }
                }
            }
        } else if let Some(body) = function_like_body(member) {
            if check(&body) {
                return true;
            }
        }
        if is_property_declaration(member) {
            if let Some(init) = member.initializer() {
                if check(&init) {
                    return true;
                }
            }
        }
        false
    }

    fn contains_constructor_reference_worker(
        &self,
        n: &Arc<Node>,
        class_original: &Arc<Node>,
        class_name: Option<&Arc<Node>>,
    ) -> bool {
        let not_class_name = match class_name {
            Some(class_name) => !Arc::ptr_eq(class_name, n),
            None => true,
        };
        if tsox_frontend::ast::is_identifier(n) && not_class_name {
            let decl = self.resolver.get_referenced_value_declaration(n);
            if let Some(decl) = decl {
                if Arc::ptr_eq(&decl, class_original) {
                    return true;
                }
            }
        }
        if is_property_access_expression(n) {
            if let Some(expr) = n.expression() {
                return self.contains_constructor_reference_worker(&expr, class_original, class_name);
            }
        }
        for_each_child(n, |child: &Arc<Node>| {
            self.contains_constructor_reference_worker(child, class_original, class_name)
        })
    }

    fn class_expression_needs_block_scoped_temp(&self) -> bool {
        if !self.requires_block_scoped_var() {
            return false;
        }
        let container = self
            .current_class_container
            .as_ref()
            .expect("current class container should be set");
        for member in members(container) {
            if is_property_declaration(member)
                && !has_static_modifier(member)
                && member
                    .name()
                    .map(|n| is_computed_property_name(&n))
                    .unwrap_or(false)
            {
                return true;
            }
        }
        false
    }
}

fn get_private_instance_methods_and_accessors_m4g4(node: &Arc<Node>) -> Vec<Arc<Node>> {
    members(node)
        .iter()
        .filter(|m| is_non_static_method_or_accessor_with_private_name(m))
        .cloned()
        .collect()
}

fn is_non_static_method_or_accessor_with_private_name(member: &Arc<Node>) -> bool {
    !has_static_modifier(member)
        && (is_method_declaration(member)
            || is_get_accessor_declaration(member)
            || is_set_accessor_declaration(member)
            || is_auto_accessor_property_declaration(member))
        && member
            .name()
            .map(|n| is_private_identifier(&n))
            .unwrap_or(false)
}

fn class_static_block_body(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        ndg::NodeData::ClassStaticBlockDeclaration(d) => Some(d.body.clone()),
        _ => None,
    }
}

fn block_statement_nodes(block: &Arc<Node>) -> Vec<Arc<Node>> {
    match &block.data {
        ndg::NodeData::Block(d) => d.statements.nodes.clone(),
        _ => Vec::new(),
    }
}

fn function_like_body(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        ndg::NodeData::MethodDeclaration(d) => d.body.clone(),
        ndg::NodeData::GetAccessorDeclaration(d) => d.body.clone(),
        ndg::NodeData::SetAccessorDeclaration(d) => d.body.clone(),
        ndg::NodeData::ConstructorDeclaration(d) => d.body.clone(),
        ndg::NodeData::FunctionDeclaration(d) => d.body.clone(),
        ndg::NodeData::FunctionExpression(d) => Some(d.body.clone()),
        _ => None,
    }
}
