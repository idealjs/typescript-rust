#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::is_identifier;
use crate::mig::m4g::ClassFieldsTransformer;
use crate::mig::m4g::r39k15_defs::{ClassFieldsTransformerR39k15, NodeFactoryR39k15};
use crate::mig::m4g_2::r37k13_defs::ClassFieldsTransformerR37k13;
use crate::mig::m4g::r33k7_defs::{
    class_like_name, computed_property_name_expression, expression_statement_expression,
    for_statement_condition, for_statement_incrementor, for_statement_initializer,
    for_statement_statement,
};

impl ClassFieldsTransformer {

    pub fn visit_constructor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if let Some(container) = self.current_class_container.clone() {
            return self.transform_constructor(Some(node), &container);
        }
        Some(self.visitor().visit_each_child(node))
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
        let lex = env.data.as_ref();
        if let Some(class_this) = lex.and_then(|d| d.class_this.clone()) {
            return Some(class_this);
        }
        if let Some(class_constructor) = lex.and_then(|d| d.class_constructor.clone()) {
            return Some(class_constructor);
        }
        None
    }

    pub fn visit_for_statement(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let initializer = self
            .discarded_value_visitor()
            .visit_node_opt(for_statement_initializer(node));
        let condition = self.visitor().visit_node_opt(for_statement_condition(node));
        let incrementor = self
            .discarded_value_visitor()
            .visit_node_opt(for_statement_incrementor(node));
        let saved = self.in_iteration_statement;
        self.in_iteration_statement = true;
        let body = self
            .emit_context()
            .visit_iteration_body(
                Some(for_statement_statement(node).clone()),
                &mut tsox_frontend::ast::visitor::NodeVisitor::default(),
            )
            .expect("Expected visitor to return a statement.");
        self.in_iteration_statement = saved;
        self.factory().update_for_statement(node, initializer, condition, incrementor, &body)
    }

    pub fn visit_expression_statement(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if is_private_identifier(expression_statement_expression(node).unwrap())
            && self.should_transform_private_elements_or_class_static_blocks
        {
            return node.clone();
        }
        let visited = self
            .discarded_value_visitor()
            .visit_node(expression_statement_expression(node).unwrap());
        self.factory().update_expression_statement(node, &visited)

    }

    pub fn visit_computed_property_name(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let saved_lexical_environment = self.lexical_environment.take();
        let saved_inside_computed_property_name = self.inside_computed_property_name;
        self.inside_computed_property_name = true;
        if let Some(env) = &saved_lexical_environment {
            if env.previous.is_some() {
                self.lexical_environment = env.previous.clone().map(|b| *b);
            }
        }
        let expression = self.visitor().visit_node(computed_property_name_expression(node));
        self.lexical_environment = saved_lexical_environment;
        self.inside_computed_property_name = saved_inside_computed_property_name;
        let injected = self.inject_pending_expressions(&expression);
        self.factory().update_computed_property_name(node, &injected)
    }
}

use crate::mig::m4g::{ClassFacts, PrivateIdentifierInfo};
use crate::mig::m4g::r33k7_defs::PrivateIdentifierKind;
use crate::mig::m4g::r39k15_defs::{ClassFieldsTransformerR39k15 as _, K13VisitorR39k15};
use crate::mig::m4q::r33k12_defs::{
    is_private_identifier_class_element_declaration, skip_parentheses, EmitFlags,
};
use crate::mig::m4m_2::find_super_statement_index_path;
use crate::mig::m4h_2::is_class_named_evaluation_helper_block;
use crate::mig::x6a::{is_class_this_assignment_block, is_decorated_class_like};
use crate::printer::{
    AutoGenerateOptions, GeneratedIdentifierFlags, NodeFactory as PrinterNodeFactory,
};
use tsox_frontend::ast::mig::m3b::{member_list, members};
use tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration;
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::{
    is_class_static_block_declaration, is_constructor_declaration, is_property_declaration,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::subtree_facts::{
    SubtreeContainsLexicalSuper, SubtreeContainsLexicalThis,
};
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::{has_accessor_modifier, has_static_modifier, is_assignment_expression, is_private_identifier};
use tsox_frontend::ast::node::NodeList;

impl ClassFieldsTransformer {
    pub fn transform_constructor(
        &mut self,
        constructor: Option<&Arc<Node>>,
        container: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let will_hoist_initializers = self
            .lexical_environment
            .as_ref()
            .and_then(|env| env.data.as_ref())
            .map(|data| data.facts.contains(ClassFacts::WillHoistInitializersToConstructor))
            .unwrap_or(false);
        if !will_hoist_initializers {
            return match constructor {
                Some(constructor) => Some(self.visitor().visit_each_child(constructor)),
                None => None,
            };
        }

        let extends_clause_element = get_class_extends_heritage_element_m4g5(container);
        let is_derived_class = extends_clause_element
            .as_ref()
            .and_then(|element| element.expression())
            .map(|expression| {
                skip_outer_expressions(&expression, OuterExpressionKinds::all()).kind
                    != SyntaxKind::NullKeyword
            })
            .unwrap_or(false);

        let mut parameters: Option<Arc<NodeList>> = None;
        if let Some(constructor) = constructor {
            let constructor_parameters = match &constructor.data {
                ndg::NodeData::ConstructorDeclaration(d) => d.parameters.clone(),
                _ => panic!("expected ConstructorDeclaration"),
            };
            parameters = Some(
                self.visitor()
                    .visit_nodes_list(&constructor_parameters)
                    .unwrap_or(constructor_parameters),
            );
        }

        let body = self.transform_constructor_body(container, constructor, is_derived_class);
        let Some(body) = body else {
            return match constructor {
                Some(constructor) => Some(self.visitor().visit_each_child(constructor)),
                None => None,
            };
        };

        if let Some(constructor) = constructor {
            let parameters = parameters.expect("constructor parameters should be visited");
            let emit_context = self.emit_context();
            let factory = PrinterNodeFactory::new(&emit_context);
            return Some(factory.update_constructor_declaration(
                constructor,
                None,
                None,
                &parameters,
                None,
                None,
                Some(&body),
            ));
        }

        let parameters = parameters.unwrap_or_else(|| {
            let emit_context = self.emit_context();
            let factory = PrinterNodeFactory::new(&emit_context);
            factory.new_node_list(Vec::new())
        });
        let mut result = Node::new(
            SyntaxKind::Constructor,
            ndg::NodeData::ConstructorDeclaration(ndg::ConstructorDeclarationData {
                modifiers: None,
                type_parameters: None,
                parameters,
                type_node: None,
                full_signature: None,
                body: Some(body),
            }),
        );
        result.loc = container.loc;
        Some(Arc::new(result))
    }

    fn transform_constructor_body_worker(
        &mut self,
        mut statements_out: Vec<Arc<Node>>,
        statements_in: &[Arc<Node>],
        statement_offset: usize,
        super_path: &[usize],
        super_path_depth: usize,
        initializer_statements: &[Arc<Node>],
        constructor: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        let super_statement_index = super_path[super_path_depth];
        for statement in &statements_in[statement_offset..super_statement_index] {
            statements_out.push(self.visitor().visit_node(statement));
        }
        let mut statement_offset = super_statement_index + 1;

        let super_statement = &statements_in[super_statement_index];
        if super_statement.kind == SyntaxKind::TryStatement {
            let (try_block, catch_clause, finally_block) = match &super_statement.data {
                ndg::NodeData::TryStatement(d) => {
                    (d.try_block.clone(), d.catch_clause.clone(), d.finally_block.clone())
                }
                _ => unreachable!(),
            };
            let try_block_statements = self.transform_constructor_body_worker(
                Vec::new(),
                &block_statement_nodes_m4g5(&try_block),
                0,
                super_path,
                super_path_depth + 1,
                initializer_statements,
                constructor,
            );
            let try_statement_list = {
                let emit_context = self.emit_context();
                let factory = PrinterNodeFactory::new(&emit_context);
                let mut list = factory.new_node_list(try_block_statements);
                if let Some(list) = Arc::get_mut(&mut list) {
                    list.loc = block_statements_loc(&try_block);
                }
                list
            };
            let updated = {
                let emit_context = self.emit_context();
                let factory = PrinterNodeFactory::new(&emit_context);
                let new_try_block = factory.update_block(
                    &try_block,
                    &try_statement_list,
                    block_is_multiline(&try_block),
                );
                let catch_clause = catch_clause.map(|c| self.visitor().visit_node(&c));
                let finally_block = finally_block.map(|b| self.visitor().visit_node(&b));
                factory_update_try_statement_m4g5(
                    super_statement,
                    &new_try_block,
                    catch_clause.as_ref(),
                    finally_block.as_ref(),
                )
            };
            statements_out.push(updated);
        } else {
            statements_out.push(self.visitor().visit_node(super_statement));

            while statement_offset < statements_in.len() {
                let original = self
                    .emit_context()
                    .most_original(&statements_in[statement_offset]);
                if is_parameter_property_declaration(&original, constructor) {
                    statement_offset += 1;
                } else {
                    break;
                }
            }

            statements_out.extend(initializer_statements.iter().cloned());
        }

        for statement in &statements_in[statement_offset..] {
            statements_out.push(self.visitor().visit_node(statement));
        }
        statements_out
    }

    fn transform_constructor_body(
        &mut self,
        container: &Arc<Node>,
        constructor: Option<&Arc<Node>>,
        is_derived_class: bool,
    ) -> Option<Arc<Node>> {
        let instance_properties = get_properties_m4g5(container, false, false);
        let use_define_for_class_fields = self.compiler_options.get_use_define_for_class_fields();
        let properties: Vec<Arc<Node>> = if !use_define_for_class_fields {
            instance_properties
                .iter()
                .filter(|prop| {
                    prop.initializer().is_some()
                        || prop
                            .name()
                            .map(|name| is_private_identifier(&name))
                            .unwrap_or(false)
                        || has_accessor_modifier(prop)
                })
                .cloned()
                .collect()
        } else {
            instance_properties.clone()
        };

        let private_methods_and_accessors = get_private_instance_methods_and_accessors_m4g5(container);
        let needs_constructor_body =
            !properties.is_empty() || !private_methods_and_accessors.is_empty();

        if constructor.is_none() && !needs_constructor_body {
            let mut visitor = NodeVisitor::default();
            return self.emit_context().visit_function_body(None, &mut visitor);
        }

        let mut emit_context = self.emit_context();
        emit_context.start_variable_environment();

        let needs_synthetic_constructor = constructor.is_none() && is_derived_class;
        let mut statements: Vec<Arc<Node>> = Vec::new();

        let receiver = self.factory().new_this_expression();

        let mut initializer_statements: Vec<Arc<Node>> = Vec::new();
        initializer_statements = self.add_instance_method_statements(
            initializer_statements,
            &private_methods_and_accessors,
            &receiver,
        );

        if let Some(constructor) = constructor {
            let parameter_properties: Vec<Arc<Node>> = instance_properties
                .iter()
                .filter(|prop| {
                    let original = self.emit_context().most_original(prop);
                    is_parameter_property_declaration(&original, constructor)
                })
                .cloned()
                .collect();
            let non_parameter_properties: Vec<Arc<Node>> = properties
                .iter()
                .filter(|prop| {
                    let original = self.emit_context().most_original(prop);
                    !is_parameter_property_declaration(&original, constructor)
                })
                .cloned()
                .collect();
            initializer_statements = self.add_property_or_class_static_block_statements(
                initializer_statements,
                &parameter_properties,
                &receiver,
            );
            initializer_statements = self.add_property_or_class_static_block_statements(
                initializer_statements,
                &non_parameter_properties,
                &receiver,
            );
        } else {
            initializer_statements = self.add_property_or_class_static_block_statements(
                initializer_statements,
                &properties,
                &receiver,
            );
        }

        let constructor_body: Option<Arc<Node>> = constructor.and_then(|c| match &c.data {
            ndg::NodeData::ConstructorDeclaration(d) => d.body.clone(),
            _ => None,
        });

        if let (Some(constructor), Some(constructor_body)) = (constructor, constructor_body.clone())
        {
            let body_statements = block_statement_nodes_m4g5(&constructor_body);
            let (prologue, _) = {
                let emit_context = self.emit_context();
                let factory = PrinterNodeFactory::new(&emit_context);
                factory.split_standard_prologue(&body_statements)
            };
            statements.extend(prologue);
            let statement_offset = statements.len();

            let super_path = find_super_statement_index_path(&body_statements, statement_offset);
            if !super_path.is_empty() {
                statements = self.transform_constructor_body_worker(
                    statements,
                    &body_statements,
                    statement_offset,
                    &super_path,
                    0,
                    &initializer_statements,
                    constructor,
                );
            } else {
                let mut statement_offset = statement_offset;
                while statement_offset < body_statements.len() {
                    let original = self
                        .emit_context()
                        .most_original(&body_statements[statement_offset]);
                    if is_parameter_property_declaration(&original, constructor) {
                        statement_offset += 1;
                    } else {
                        break;
                    }
                }
                statements.extend(initializer_statements.iter().cloned());
                for statement in &body_statements[statement_offset..] {
                    statements.push(self.visitor().visit_node(statement));
                }
            }
        } else {
            if needs_synthetic_constructor {
                let emit_context = self.emit_context();
                let factory = PrinterNodeFactory::new(&emit_context);
                let spread = Arc::new(Node::new(
                    SyntaxKind::SpreadElement,
                    ndg::NodeData::SpreadElement(ndg::SpreadElementData {
                        expression: factory.new_identifier("arguments"),
                    }),
                ));
                let super_call = factory.new_expression_statement(&factory.new_call_expression(
                    &factory.new_keyword_expression(SyntaxKind::SuperKeyword),
                    None,
                    None,
                    factory.new_node_list(vec![spread]),
                    NodeFlags::empty(),
                ));
                statements.push(super_call);
            }
            statements.extend(initializer_statements.iter().cloned());
        }

        let declarations = emit_context.end_variable_environment();
        statements = emit_context.merge_environment(statements, declarations);

        if statements.is_empty() && constructor.is_none() {
            return None;
        }

        let multi_line = match (&constructor, &constructor_body) {
            (Some(_), Some(constructor_body))
                if block_statement_nodes_m4g5(constructor_body).len() >= statements.len() =>
            {
                block_is_multiline(constructor_body)
            }
            _ => !statements.is_empty(),
        };

        let emit_context = self.emit_context();
        let factory = PrinterNodeFactory::new(&emit_context);
        let mut statement_list = factory.new_node_list(statements);
        let statements_loc = match &constructor_body {
            Some(constructor_body) => block_statements_loc(constructor_body),
            None => {
                let member_list = member_list(container).expect("class requires member list");
                member_list.loc
            }
        };
        if let Some(list) = Arc::get_mut(&mut statement_list) {
            list.loc = statements_loc;
        }

        let mut block = factory.new_block(&statement_list, multi_line);
        if let Some(constructor_body) = &constructor_body {
            if let Some(b) = Arc::get_mut(&mut block) {
                b.loc = constructor_body.loc;
            }
        }
        Some(block)
    }

    pub(crate) fn add_property_or_class_static_block_statements(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        properties: &[Arc<Node>],
        receiver: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        for property in properties {
            if has_static_modifier(property)
                && !self.should_transform_private_elements_or_class_static_blocks
            {
                continue;
            }
            if let Some(statement) = self.transform_property_or_class_static_block(property, receiver)
            {
                statements.push(statement);
            }
        }
        statements
    }

    pub fn transform_class_members(&mut self, node: &Arc<Node>) -> (Arc<NodeList>, Option<Arc<Node>>) {
        let should_transform_private_static_elements_in_class = self
            .emit_context()
            .emit_flags(node)
            .contains(EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS);

        if self.should_transform_private_elements_or_class_static_blocks
            || self.should_transform_private_static_elements_in_file
        {
            for member in members(node) {
                if is_private_identifier_class_element_declaration(member) {
                    if self.should_transform_class_element_to_weak_map(member) {
                        self.add_private_identifier_to_environment(member);
                    } else {
                        let name = member
                            .name()
                            .expect("private class element requires a name");
                        self.set_private_identifier(&name, untransformed_private_identifier_info());
                    }
                }
            }

            if self.should_transform_private_elements_or_class_static_blocks
                && !get_private_instance_methods_and_accessors_m4g5(node).is_empty()
            {
                self.create_brand_check_weak_set_for_private_methods();
            }

            if self.should_transform_auto_accessors_in_current_class() {
                for member in members(node) {
                    if is_auto_accessor_property_declaration_m4g5(member) {
                        let member_name =
                            member.name().expect("auto accessor requires a name");
                        let emit_context = self.emit_context();
                        let factory = PrinterNodeFactory::new(&emit_context);
                        let storage_name = factory.generated_name_node(
                            &factory.new_generated_private_name_for_node_ex(
                                &member_name,
                                AutoGenerateOptions {
                                    suffix: "_accessor_storage".to_string(),
                                    ..Default::default()
                                },
                            ),
                        );
                        if self.should_transform_private_elements_or_class_static_blocks
                            || (should_transform_private_static_elements_in_class
                                && has_static_modifier(member))
                        {
                            self.add_private_identifier_property_declaration_to_environment(
                                member,
                                &storage_name,
                            );
                        } else {
                            let emit_context = self.emit_context();
                            let already_registered = {
                                let env = self.get_private_identifier_environment();
                                get_private_identifier_in_m4g5(&emit_context, env, &storage_name)
                                    .is_some()
                            };
                            if !already_registered {
                                self.set_private_identifier(
                                    &storage_name,
                                    untransformed_private_identifier_info(),
                                );
                            }
                        }
                    }
                }
            }
        }

        let members_list_loc = member_list(node).map(|list| list.loc);
        let mut members_list = self
            .class_element_visitor()
            .visit_nodes_list(member_list(node).expect("class requires member list"))
            .expect("member list visit should produce a list");

        let mut synthetic_constructor: Option<Arc<Node>> = None;
        if !members_list.nodes.iter().any(|m| is_constructor_declaration(m)) {
            synthetic_constructor = self.transform_constructor(None, node);
        }

        let mut members_prologue: Option<Arc<Node>> = None;
        let mut synthetic_static_block: Option<Arc<Node>> = None;
        if !self.should_transform_private_elements_or_class_static_blocks
            && !self.pending_expressions.is_empty()
        {
            let mut statement = {
                let expressions = std::mem::take(&mut self.pending_expressions);
                let expression = self.factory().inline_expressions(expressions);
                self.factory().new_expression_statement(&expression)
            };
            if statement
                .subtree_facts()
                .intersects(SubtreeContainsLexicalThis | SubtreeContainsLexicalSuper)
            {
                let emit_context = self.emit_context();
                let factory = PrinterNodeFactory::new(&emit_context);
                let temp = factory.generated_name_node(&factory.new_temp_variable());
                self.emit_context().add_variable_declaration(&temp);
                let arrow = self.factory().new_arrow_function(
                    None,
                    None,
                    self.factory().new_node_list(&[]),
                    None,
                    None,
                    self.factory().new_token(SyntaxKind::EqualsGreaterThanToken),
                    self.factory()
                        .new_block(self.factory().new_node_list(&[statement.clone()]), false),
                );
                members_prologue = Some(self.factory().new_assignment_expression(&temp, &arrow));
                let call = self.factory().new_call_expression(
                    &temp,
                    None,
                    None,
                    self.factory().new_node_list(&[]),
                    NodeFlags::empty(),
                );
                statement = self.factory().new_expression_statement(&call);
            }

            let block = {
                let emit_context = self.emit_context();
                let factory = PrinterNodeFactory::new(&emit_context);
                factory.new_block(
                    &NodeList {
                        loc: statement.loc,
                        nodes: vec![statement],
                    },
                    false,
                )
            };
            synthetic_static_block = Some(
                self.factory()
                    .new_class_static_block_declaration(None, &block),
            );
            self.pending_expressions = Vec::new();
        }

        if synthetic_constructor.is_some() || synthetic_static_block.is_some() {
            let emit_context = self.emit_context();
            let class_this_idx = members_list
                .nodes
                .iter()
                .position(|n| is_class_this_assignment_block(&emit_context, n));
            let named_eval_idx = members_list
                .nodes
                .iter()
                .position(|n| is_class_named_evaluation_helper_block(&emit_context, n));

            let mut members_array: Vec<Arc<Node>> =
                Vec::with_capacity(members_list.nodes.len() + 2);
            if let Some(idx) = class_this_idx {
                members_array.push(members_list.nodes[idx].clone());
            }
            if let Some(idx) = named_eval_idx {
                members_array.push(members_list.nodes[idx].clone());
            }
            if let Some(constructor) = &synthetic_constructor {
                members_array.push(constructor.clone());
            }
            if let Some(static_block) = &synthetic_static_block {
                members_array.push(static_block.clone());
            }
            for (i, member) in members_list.nodes.iter().enumerate() {
                if Some(i) != class_this_idx && Some(i) != named_eval_idx {
                    members_array.push(member.clone());
                }
            }
            let mut new_list = NodeList::new(members_array);
            if let Some(loc) = members_list_loc {
                new_list.loc = loc;
            }
            members_list = Arc::new(new_list);
        }

        (members_list, members_prologue)
    }

    fn create_brand_check_weak_set_for_private_methods(&mut self) {
        let weak_set_name = self
            .get_private_identifier_environment()
            .data
            .weak_set_name
            .clone()
            .expect("weakSetName should be set in private identifier environment");
        let emit_context = self.emit_context();
        let factory = PrinterNodeFactory::new(&emit_context);
        let new_expression = Arc::new(Node::new(
            SyntaxKind::NewExpression,
            ndg::NodeData::NewExpression(ndg::NewExpressionData {
                expression: factory.new_identifier("WeakSet"),
                type_arguments: None,
                arguments: Some(factory.new_node_list(Vec::new())),
            }),
        ));
        let assignment =
            factory.new_assignment_expression(&weak_set_name, &new_expression);
        self.add_pending_expressions(vec![assignment]);
    }

    pub fn generate_initialized_property_expressions_or_class_static_block(
        &mut self,
        properties_or_class_static_blocks: &[Arc<Node>],
        receiver: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        let mut expressions: Vec<Arc<Node>> = Vec::new();
        for property in properties_or_class_static_blocks {
            let expression = if is_class_static_block_declaration(property) {
                let saved = self.current_class_element.replace(property.clone());
                let visited = self.transform_class_static_block_declaration_m4g5(property);
                self.current_class_element = saved;
                visited
            } else {
                self.transform_property(property, receiver)
            };
            let Some(expression) = expression else {
                continue;
            };
            self.emit_context().set_original_ex(&expression, property, true);
            self.emit_context()
                .assign_comment_and_source_map_ranges(&expression, property);
            expressions.push(expression);
        }
        expressions
    }

    fn transform_class_static_block_declaration_m4g5(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        if !self.should_transform_private_elements_or_class_static_blocks {
            return None;
        }
        let emit_context = self.emit_context();
        if is_class_this_assignment_block(&emit_context, node) {
            let first_expression = static_block_first_expression_m4g5(node)?;
            let result = self.visitor().visit_node(&first_expression);
            if is_assignment_expression(&result, true) {
                if let ndg::NodeData::BinaryExpression(binary) = &result.data {
                    if Arc::ptr_eq(&binary.left, &binary.right) {
                        return None;
                    }
                }
            }
            return Some(result);
        }
        if is_class_named_evaluation_helper_block(&emit_context, node) {
            let first_expression = static_block_first_expression_m4g5(node)?;
            return Some(self.visitor().visit_node(&first_expression));
        }

        let mut emit_context = self.emit_context();
        emit_context.start_variable_environment();
        let visited: Vec<Arc<Node>> = {
            let statements = static_block_statements_m4g5(node);
            let saved = self.current_class_element.replace(node.clone());
            let mut visitor = self.visitor();
            let visited = statements.iter().map(|s| visitor.visit_node(s)).collect();
            self.current_class_element = saved;
            visited
        };
        let (statements, _) = emit_context.end_and_merge_variable_environment(&visited);

        let iife = {
            let factory = tsox_frontend::format::mig::m4o::new_node_factory(
                tsox_frontend::format::mig::m4o::EmitContext::default(),
            );
            factory.new_immediately_invoked_arrow_function(&statements)
        };
        let arrow_function = skip_parentheses(iife.expression().expect("iife requires expression"));
        self.emit_context().set_original(&arrow_function, node);
        self.emit_context()
            .add_emit_flags(&arrow_function, EmitFlags::NO_LEXICAL_ARGUMENTS);
        self.emit_context().set_original(&iife, node);
        self.emit_context().assign_source_map_range(&iife, node);
        self.emit_context()
            .add_emit_flags(&arrow_function, EmitFlags::NO_LEXICAL_THIS);
        let statements_loc = static_block_statements_loc_m4g5(node);
        let mut iife = iife;
        if let Some(iife_node) = Arc::get_mut(&mut iife) {
            if let ndg::NodeData::CallExpression(call) = &mut iife_node.data {
                if let Some(paren) = Arc::get_mut(&mut call.expression) {
                    if let ndg::NodeData::ParenthesizedExpression(paren_data) = &mut paren.data {
                        if let Some(arrow) = Arc::get_mut(&mut paren_data.expression) {
                            if let ndg::NodeData::ArrowFunction(arrow_data) = &mut arrow.data {
                                if let Some(body) = Arc::get_mut(&mut arrow_data.body) {
                                    if let ndg::NodeData::Block(block) = &mut body.data {
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
}

fn untransformed_private_identifier_info() -> PrivateIdentifierInfo {
    PrivateIdentifierInfo {
        kind: PrivateIdentifierKind::Untransformed,
        brand_check_identifier: None,
        is_static: false,
        is_valid: false,
        variable_name: None,
        method_name: None,
        getter_name: None,
        setter_name: None,
    }
}

fn get_private_instance_methods_and_accessors_m4g5(node: &Arc<Node>) -> Vec<Arc<Node>> {
    members(node)
        .iter()
        .filter(|m| is_non_static_method_or_accessor_with_private_name_m4g5(m))
        .cloned()
        .collect()
}

fn is_non_static_method_or_accessor_with_private_name_m4g5(member: &Arc<Node>) -> bool {
    !has_static_modifier(member)
        && (tsox_frontend::ast::node_data_generated::is_method_declaration(member)
            || tsox_frontend::ast::node_data_generated::is_get_accessor_declaration(member)
            || tsox_frontend::ast::node_data_generated::is_set_accessor_declaration(member)
            || is_auto_accessor_property_declaration_m4g5(member))
        && member
            .name()
            .map(|n| is_private_identifier(&n))
            .unwrap_or(false)
}

fn is_auto_accessor_property_declaration_m4g5(node: &Arc<Node>) -> bool {
    tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration(node)
}

fn get_private_identifier_in_m4g5<'a>(
    emit_context: &crate::printer::EmitContext,
    env: &'a crate::mig::m4g::PrivateEnvironment,
    name: &Arc<Node>,
) -> Option<&'a crate::mig::m4g::PrivateIdentifierInfo> {
    if emit_context.has_auto_generate_info(name) {
        let key = Arc::as_ptr(name);
        return env.generated_identifiers.get(&key);
    }
    env.members.get(name.text())
}

fn get_properties_m4g5(node: &Arc<Node>, require_initializer: bool, is_static: bool) -> Vec<Arc<Node>> {
    let mut result = Vec::new();
    for member in members(node) {
        if is_property_declaration(member)
            && (!require_initializer || member.initializer().is_some())
            && has_static_modifier(member) == is_static
        {
            result.push(member.clone());
        }
    }
    result
}

fn get_class_extends_heritage_element_m4g5(node: &Arc<Node>) -> Option<Arc<Node>> {
    let heritage_clauses = get_heritage_clauses(node)?;
    for clause in heritage_clauses.nodes.iter() {
        if let ndg::NodeData::HeritageClause(data) = &clause.data {
            if data.token == SyntaxKind::ExtendsKeyword {
                return data.types.nodes.first().cloned();
            }
        }
    }
    None
}

fn block_statement_nodes_m4g5(block: &Arc<Node>) -> Vec<Arc<Node>> {
    match &block.data {
        ndg::NodeData::Block(d) => d.statements.nodes.clone(),
        _ => Vec::new(),
    }
}

fn block_is_multiline(block: &Arc<Node>) -> bool {
    match &block.data {
        ndg::NodeData::Block(d) => d.multi_line,
        _ => false,
    }
}

fn block_statements_loc(block: &Arc<Node>) -> tsox_core::core::text::TextRange {
    match &block.data {
        ndg::NodeData::Block(d) => d.statements.loc,
        _ => block.loc,
    }
}

fn static_block_statements_loc_m4g5(node: &Arc<Node>) -> tsox_core::core::text::TextRange {
    if let Some(body) = class_static_block_body_m4g5(node) {
        return block_statements_loc(&body);
    }
    node.loc
}

fn class_static_block_body_m4g5(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        ndg::NodeData::ClassStaticBlockDeclaration(d) => Some(d.body.clone()),
        _ => None,
    }
}

fn static_block_statements_m4g5(node: &Arc<Node>) -> Vec<Arc<Node>> {
    class_static_block_body_m4g5(node)
        .map(|body| block_statement_nodes_m4g5(&body))
        .unwrap_or_default()
}

fn static_block_first_expression_m4g5(node: &Arc<Node>) -> Option<Arc<Node>> {
    let statement = static_block_statements_m4g5(node).into_iter().next()?;
    match &statement.data {
        ndg::NodeData::ExpressionStatement(d) => Some(d.expression.clone()),
        _ => None,
    }
}

fn factory_update_try_statement_m4g5(
    node: &Arc<Node>,
    try_block: &Arc<Node>,
    catch_clause: Option<&Arc<Node>>,
    finally_block: Option<&Arc<Node>>,
) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::TryStatement,
        ndg::NodeData::TryStatement(ndg::TryStatementData {
            try_block: try_block.clone(),
            catch_clause: catch_clause.cloned(),
            finally_block: finally_block.cloned(),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}
