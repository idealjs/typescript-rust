use std::sync::Arc;
use tsox_frontend::ast::*;

use crate::mig::m4e_3::is_simple_parameter_list;
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags, NodeFactory as Factory};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_core::collections::ordered_set::OrderedSet;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::mig::m3e::{get_function_flags, FunctionFlags};
use tsox_frontend::format::mig::m4o_2::EmitHelper;

#[path = "r36k9_defs.rs"]
pub mod r36k9_defs;
pub use r36k9_defs::*;

use crate::mig::m4f::r33k11_defs::{advanced_async_super_helper, async_super_helper};
use crate::mig::m4h::r37k18_defs::R37K18NodeVisitorExt;
use crate::mig::m4h::r39k13_defs::R39K13NodeVisitorExt;
use crate::mig::m4h_3::{ForAwaitHierarchyFacts, ForAwaitTransformer};

impl ForAwaitTransformer {
    pub fn convert_for_of_statement_head(
        &mut self,
        node: &Arc<Node>,
        bound_value: &Arc<Node>,
        non_user_code: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("convert_for_of_statement_head"); 
        let mut emit_context = self.transformer.emit_context();
        let (initializer, expression, statement) = match &node.data {
            NodeData::ForInOrOfStatement(d) => {
                (d.initializer.clone(), d.expression.clone(), d.statement.clone())
            }
            _ => unreachable!(),
        };
        let binding;
        let mut statements;
        {
            let f = self.transformer.factory();
            let value = f.generated_name_node(&f.new_temp_variable());
            emit_context.add_variable_declaration(&value);
            let iterator_value_expression = f.new_assignment_expression(&value, bound_value);
            let iterator_value_statement = f.new_expression_statement(&iterator_value_expression);
            emit_context.set_source_map_range(&iterator_value_statement, expression.loc);

            let false_keyword = f.new_keyword_expression(SyntaxKind::FalseKeyword);
            let exit_non_user_code_expression = f.new_assignment_expression(non_user_code, &false_keyword);
            let exit_non_user_code_statement = f.new_expression_statement(&exit_non_user_code_expression);
            emit_context.set_source_map_range(&exit_non_user_code_statement, expression.loc);

            statements = vec![iterator_value_statement, exit_non_user_code_statement];
            binding = f.create_for_of_binding_statement(&initializer, &value);
        }
        statements.push(self.transformer.visitor().visit_node(&binding));

        let mut body_location = TextRange::default();
        let mut statements_location = TextRange::default();
        let statement = self.transformer.visitor().visit_embedded_statement(&statement);
        if is_block(&statement) {
            statements.extend(block_statements(&statement));
            body_location = statement.loc;
            statements_location = statement_list(&statement).loc;
        } else {
            statements.push(statement);
        }

        let f = self.transformer.factory();
        let mut stmt_list = f.new_node_list(statements);
        if let Some(l) = Arc::get_mut(&mut stmt_list) {
            l.loc = statements_location;
        }
        let mut block = f.new_block(&stmt_list, true);
        if let Some(b) = Arc::get_mut(&mut block) {
            b.loc = body_location;
        }
        block
    }

    pub fn create_downlevel_await(&mut self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("create_downlevel_await"); 
        if self.enclosing_function_flags.intersects(FunctionFlags::GENERATOR) {
            return self
                .transformer
                .factory()
                .new_yield_expression(None, &self.transformer.factory().new_await_helper(expression));
        }
        self.transformer.factory().new_await_expression(expression)
    }

    pub fn transform_for_await_of_statement(
        &mut self,
        node: &Arc<Node>,
        outermost_labeled_statement: Option<&Arc<Node>>,
        ancestor_facts: ForAwaitHierarchyFacts,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_for_await_of_statement"); 
        let mut emit_context = self.transformer.emit_context();
        let (node_expression, node_statement, initializer) = match &node.data {
            NodeData::ForInOrOfStatement(d) => {
                (d.expression.clone(), d.statement.clone(), d.initializer.clone())
            }
            _ => unreachable!(),
        };
        let expression = self.transformer.visitor().visit_node(&node_expression);

        let (iterator, result, non_user_code, done, error_record, catch_variable, return_method, mut call_values) = {
            let f = self.transformer.factory();
            let iterator = if is_identifier(&expression) {
                f.generated_name_node(&f.new_generated_name_for_node(&expression))
            } else {
                f.generated_name_node(&f.new_temp_variable())
            };

            let result = if is_identifier(&expression) {
                f.generated_name_node(&f.new_generated_name_for_node(&iterator))
            } else {
                f.generated_name_node(&f.new_temp_variable())
            };

            let non_user_code = f.generated_name_node(&f.new_temp_variable());
            let done = f.generated_name_node(&f.new_temp_variable());
            emit_context.add_variable_declaration(&done);
            let error_record = f.generated_name_node(&f.new_unique_name("e"));
            let catch_variable = f.generated_name_node(&f.new_generated_name_for_node(&error_record));
            let return_method = f.generated_name_node(&f.new_temp_variable());
            let mut call_values = f.new_async_values_helper(&expression);
            if let Some(v) = Arc::get_mut(&mut call_values) {
                v.loc = node_expression.loc;
            }
            (iterator, result, non_user_code, done, error_record, catch_variable, return_method, call_values)
        };
        let (next_access, call_next, get_done, get_value, call_return) = {
            let f = self.transformer.factory();
            let next_identifier = f.new_identifier("next");
            let next_access = f.new_property_access_expression(&iterator, None, &next_identifier, NodeFlags::empty());
            let empty_arguments = f.new_node_list(vec![]);
            let call_next = f.new_call_expression(&next_access, None, None, empty_arguments, NodeFlags::empty());
            let done_identifier = f.new_identifier("done");
            let get_done = f.new_property_access_expression(&result, None, &done_identifier, NodeFlags::empty());
            let value_identifier = f.new_identifier("value");
            let get_value = f.new_property_access_expression(&result, None, &value_identifier, NodeFlags::empty());
            let empty_call_arguments = f.new_node_list(vec![]);
            let call_return = f.new_function_call_call(&return_method, &iterator, &empty_call_arguments.nodes);
            (next_access, call_next, get_done, get_value, call_return)
        };

        emit_context.add_variable_declaration(&error_record);
        emit_context.add_variable_declaration(&return_method);

        let initializer_expr = if ancestor_facts.intersects(ForAwaitHierarchyFacts::ITERATION_CONTAINER) {
            let f = self.transformer.factory();
            let void_zero = f.new_void_zero_expression();
            let reset_error_record = f.new_assignment_expression(&error_record, &void_zero);
            f.inline_expressions(vec![reset_error_record, call_values])
        } else {
            Some(call_values)
        };

        let var_decl_list = {
            let f = self.transformer.factory();
            let mut iterator_decl = f.new_variable_declaration(&iterator, None, None, initializer_expr.as_ref());
            if let Some(d) = Arc::get_mut(&mut iterator_decl) {
                d.loc = node_expression.loc;
            }
            let true_keyword = f.new_keyword_expression(SyntaxKind::TrueKeyword);
            let non_user_code_decl = f.new_variable_declaration(&non_user_code, None, None, Some(&true_keyword));
            let result_decl = f.new_variable_declaration(&result, None, None, None);
            let decl_list_nodes = f.new_node_list(vec![non_user_code_decl, iterator_decl, result_decl]);
            let mut var_decl_list = f.new_variable_declaration_list(&decl_list_nodes, NodeFlags::empty());
            if let Some(l) = Arc::get_mut(&mut var_decl_list) {
                l.loc = node_expression.loc;
            }
            var_decl_list
        };

        let awaited_call_next = self.create_downlevel_await(&call_next);
        let (condition, incrementor) = {
            let f = self.transformer.factory();
            let assign_result = f.new_assignment_expression(&result, &awaited_call_next);
            let assign_done = f.new_assignment_expression(&done, &get_done);
            let not_done = f.new_prefix_unary_expression(SyntaxKind::ExclamationToken, &done);
            let condition = f.inline_expressions(vec![assign_result, assign_done, not_done]);

            let true_keyword2 = f.new_keyword_expression(SyntaxKind::TrueKeyword);
            let incrementor = f.new_assignment_expression(&non_user_code, &true_keyword2);
            (condition, incrementor)
        };

        let head = self.convert_for_of_statement_head(node, &get_value, &non_user_code);
        let labeled_for = {
            let f = self.transformer.factory();
            let mut for_statement = f.new_for_statement(
                Some(&var_decl_list),
                condition.as_ref(),
                Some(&incrementor),
                &head,
            );
            if let Some(s) = Arc::get_mut(&mut for_statement) {
                s.loc = node.loc;
            }
            emit_context.add_emit_flags(&for_statement, EmitFlags::NO_TOKEN_TRAILING_SOURCE_MAPS);
            emit_context.set_original(&for_statement, node);

            f.restore_enclosing_label(&for_statement, outermost_labeled_statement)
        };
        let (try_block, catch_clause, inner_condition) = {
            let f = self.transformer.factory();
            let try_statements = f.new_node_list(vec![labeled_for]);
            let try_block = f.new_block(&try_statements, true);

            let error_identifier = f.new_identifier("error");
            let error_property = f.new_property_assignment(None, &error_identifier, None, None, &catch_variable);
            let error_properties = f.new_node_list(vec![error_property]);
            let error_object = f.new_object_literal_expression(&error_properties, false);
            let error_assignment = f.new_assignment_expression(&error_record, &error_object);
            let error_statement = f.new_expression_statement(&error_assignment);
            let catch_statements = f.new_node_list(vec![error_statement]);
            let mut catch_body = f.new_block(&catch_statements, false);
            emit_context.add_emit_flags(&catch_body, EmitFlags::SINGLE_LINE);
            let catch_variable_decl = f.new_variable_declaration(&catch_variable, None, None, None);
            let catch_clause = f.new_catch_clause(Some(&catch_variable_decl), &catch_body);

            let not_non_user_code = f.new_prefix_unary_expression(SyntaxKind::ExclamationToken, &non_user_code);
            let not_done2 = f.new_prefix_unary_expression(SyntaxKind::ExclamationToken, &done);
            let and_token1 = f.new_token(SyntaxKind::AmpersandAmpersandToken);
            let inner_condition =
                f.new_binary_expression(None, &not_non_user_code, None, &and_token1, &not_done2);
            let return_identifier = f.new_identifier("return");
            let return_access =
                f.new_property_access_expression(&iterator, None, &return_identifier, NodeFlags::empty());
            let return_assignment = f.new_assignment_expression(&return_method, &return_access);
            let and_token2 = f.new_token(SyntaxKind::AmpersandAmpersandToken);
            let inner_if_condition =
                f.new_binary_expression(None, &inner_condition, None, &and_token2, &return_assignment);
            (try_block, catch_clause, inner_if_condition)
        };

        let awaited_call_return = self.create_downlevel_await(&call_return);
        let f = self.transformer.factory();
        let return_statement_expr = f.new_expression_statement(&awaited_call_return);
        let mut inner_if_statement = f.new_if_statement(&inner_condition, &return_statement_expr, None);
        emit_context.add_emit_flags(&inner_if_statement, EmitFlags::SINGLE_LINE);

        let inner_try_statements = f.new_node_list(vec![inner_if_statement]);
        let inner_try_block = f.new_block(&inner_try_statements, false);

        let inner_finally_if = f.new_if_statement(
            &error_record,
            &f.new_throw_statement(&error_property_access(&f, &error_record)),
            None,
        );
        emit_context.add_emit_flags(&inner_finally_if, EmitFlags::SINGLE_LINE);
        let inner_finally_statements = f.new_node_list(vec![inner_finally_if]);
        let mut inner_finally_block = f.new_block(&inner_finally_statements, false);
        emit_context.add_emit_flags(&inner_finally_block, EmitFlags::SINGLE_LINE);

        let inner_try_statement = f.new_try_statement(&inner_try_block, None, Some(&inner_finally_block));
        let finally_statements = f.new_node_list(vec![inner_try_statement]);
        let finally_block = f.new_block(&finally_statements, true);

        f.new_try_statement(&try_block, Some(&catch_clause), Some(&finally_block))
    }

    pub fn visit_constructor_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_constructor_declaration"); 
        let saved_enclosing_function_flags = self.enclosing_function_flags;
        self.enclosing_function_flags = get_function_flags(Some(node));
        let (modifiers, parameters, body) = match &node.data {
            NodeData::ConstructorDeclaration(d) => {
                (d.modifiers.clone(), d.parameters.clone(), d.body.clone())
            }
            _ => unreachable!(),
        };
        let visited_parameters = self
            .transformer
            .emit_context()
            .visit_parameters(&parameters, self.transformer.visitor());
        let visited_body = self.transformer.emit_context().visit_function_body(body.clone(), self.transformer.visitor());
        let updated = self.transformer.factory().update_constructor_declaration(
            node,
            modifiers.clone(),
            None,
            &visited_parameters,
            None,
            None,
            visited_body.as_ref(),
        );
        self.enclosing_function_flags = saved_enclosing_function_flags;
        updated
    }

    pub fn visit_get_accessor_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_get_accessor_declaration"); 
        let saved_enclosing_function_flags = self.enclosing_function_flags;
        self.enclosing_function_flags = get_function_flags(Some(node));
        let (modifiers, name, parameters, body) = match &node.data {
            NodeData::GetAccessorDeclaration(d) => {
                (d.modifiers.clone(), d.name.clone(), d.parameters.clone(), d.body.clone())
            }
            _ => unreachable!(),
        };
        let visited_name = self.transformer.visitor().visit_node(&name);
        let visited_parameters = self
            .transformer
            .emit_context()
            .visit_parameters(&parameters, self.transformer.visitor());
        let visited_body = self.transformer.emit_context().visit_function_body(body.clone(), self.transformer.visitor());
        let updated = self.transformer.factory().update_get_accessor_declaration(
            node,
            modifiers.clone(),
            &visited_name,
            None,
            &visited_parameters,
            None,
            None,
            visited_body.as_ref(),
        );
        self.enclosing_function_flags = saved_enclosing_function_flags;
        updated
    }

    pub fn visit_set_accessor_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_set_accessor_declaration"); 
        let saved_enclosing_function_flags = self.enclosing_function_flags;
        self.enclosing_function_flags = get_function_flags(Some(node));
        let (modifiers, name, parameters, body) = match &node.data {
            NodeData::SetAccessorDeclaration(d) => {
                (d.modifiers.clone(), d.name.clone(), d.parameters.clone(), d.body.clone())
            }
            _ => unreachable!(),
        };
        let visited_name = self.transformer.visitor().visit_node(&name);
        let visited_parameters = self
            .transformer
            .emit_context()
            .visit_parameters(&parameters, self.transformer.visitor());
        let visited_body = self.transformer.emit_context().visit_function_body(body.clone(), self.transformer.visitor());
        let updated = self.transformer.factory().update_set_accessor_declaration(
            node,
            modifiers.clone(),
            &visited_name,
            None,
            &visited_parameters,
            None,
            None,
            visited_body.as_ref(),
        );
        self.enclosing_function_flags = saved_enclosing_function_flags;
        updated
    }

    fn update_function_like(
        &mut self,
        node: &Arc<Node>,
        visit_name: bool,
        name_node: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_function_like"); 
        let saved_enclosing_function_flags = self.enclosing_function_flags;
        self.enclosing_function_flags = get_function_flags(Some(node));
        let mut emit_context = self.transformer.emit_context();
        let (modifiers, asterisk_token, type_parameters, parameters, body) = match &node.data {
            NodeData::MethodDeclaration(d) => (
                d.modifiers.clone(),
                d.asterisk_token.clone(),
                d.type_parameters.clone(),
                d.parameters.clone(),
                d.body.clone(),
            ),
            NodeData::FunctionDeclaration(d) => (
                d.modifiers.clone(),
                d.asterisk_token.clone(),
                d.type_parameters.clone(),
                d.parameters.clone(),
                d.body.clone(),
            ),
            NodeData::FunctionExpression(d) => (
                d.modifiers.clone(),
                d.asterisk_token.clone(),
                d.type_parameters.clone(),
                d.parameters.clone(),
                Some(d.body.clone()),
            ),
            _ => unreachable!(),
        };

        let modifiers = if self.enclosing_function_flags.intersects(FunctionFlags::GENERATOR) {
            self.visit_modifiers_no_async(&modifiers)
        } else {
            modifiers
        };

        let asterisk_token = if self.enclosing_function_flags.intersects(FunctionFlags::ASYNC) {
            None
        } else {
            asterisk_token
        };

        let mut parameters_node: Arc<NodeList>;
        let body_node: Option<Arc<Node>>;
        if self
            .enclosing_function_flags
            .intersects(FunctionFlags::ASYNC_GENERATOR)
            && self.enclosing_function_flags.intersects(FunctionFlags::ASYNC)
            && self.enclosing_function_flags.intersects(FunctionFlags::GENERATOR)
        {
            parameters_node = self.transform_async_generator_function_parameter_list(node);
            body_node = Some(self.transform_async_generator_function_body(node));
        } else {
            parameters_node = Arc::new(emit_context.visit_parameters(
                parameters.as_ref(),
                self.transformer.visitor(),
            ));
            body_node = emit_context.visit_function_body(body.clone(), self.transformer.visitor());
        }

        let visited_name = if visit_name {
            self.transformer.visitor().visit_node(name_node.unwrap())
        } else {
            name_node.cloned().unwrap()
        };

        let factory = self.transformer.factory();
        let updated = match node.kind {
            SyntaxKind::MethodDeclaration => factory.update_method_declaration(
                node,
                modifiers.clone(),
                asterisk_token.as_ref(),
                &visited_name,
                None,
                type_parameters.clone(),
                &parameters_node,
                None,
                None,
                body_node.as_ref(),
            ),
            SyntaxKind::FunctionDeclaration => factory.update_function_declaration(
                node,
                modifiers.clone(),
                asterisk_token.as_ref(),
                Some(&visited_name),
                type_parameters.clone(),
                &parameters_node,
                None,
                None,
                body_node.as_ref(),
            ),
            _ => factory.update_function_expression(
                node,
                modifiers.clone(),
                asterisk_token.as_ref(),
                Some(&visited_name),
                type_parameters.clone(),
                &parameters_node,
                None,
                None,
                body_node.as_ref(),
            ),
        };
        self.enclosing_function_flags = saved_enclosing_function_flags;
        updated
    }

    pub fn visit_method_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_method_declaration"); 
        let name = match &node.data {
            NodeData::MethodDeclaration(d) => Some(d.name.clone()),
            _ => unreachable!(),
        };
        self.update_function_like(node, true, name.as_ref())
    }

    pub fn visit_function_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_function_declaration"); 
        let name = match &node.data {
            NodeData::FunctionDeclaration(d) => d.name.clone(),
            _ => unreachable!(),
        };
        self.update_function_like(node, false, name.as_ref())
    }

    pub fn visit_function_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_function_expression"); 
        let name = match &node.data {
            NodeData::FunctionExpression(d) => d.name.clone(),
            _ => unreachable!(),
        };
        self.update_function_like(node, false, name.as_ref())
    }

    pub fn visit_arrow_function(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_arrow_function"); 
        let saved_enclosing_function_flags = self.enclosing_function_flags;
        self.enclosing_function_flags = get_function_flags(Some(node));
        let (modifiers, parameters, equals_greater_than_token, body) = match &node.data {
            NodeData::ArrowFunction(d) => (
                d.modifiers.clone(),
                d.parameters.clone(),
                d.equals_greater_than_token.clone(),
                d.body.clone(),
            ),
            _ => unreachable!(),
        };
        let visited_parameters = self
            .transformer
            .emit_context()
            .visit_parameters(&parameters, self.transformer.visitor());
        let visited_body = self
            .transformer
            .emit_context()
            .visit_function_body(Some(body.clone()), self.transformer.visitor());
        let updated = self.transformer.factory().update_arrow_function(
            node,
            modifiers.clone(),
            None,
            &visited_parameters,
            None,
            None,
            &equals_greater_than_token,
            visited_body.as_ref().unwrap(),
        );
        self.enclosing_function_flags = saved_enclosing_function_flags;
        updated
    }

    pub fn transform_async_generator_function_parameter_list(&mut self, node: &Arc<Node>) -> Arc<NodeList> { ::tsox_core::fntrace::enter("transform_async_generator_function_parameter_list"); 
        let parameters = node
            .parameters()
            .map(|l| &l.nodes[..])
            .unwrap_or(&[]);
        if is_simple_parameter_list(parameters) {
            let parameter_list = parameter_list_of(node).unwrap();
            return Arc::new(
                self.transformer
                    .emit_context()
                    .visit_parameters(parameter_list.as_ref(), self.transformer.visitor()),
            );
        }
        let f = self.transformer.factory();
        let mut new_parameters: Vec<Arc<Node>> = Vec::new();
        for parameter in parameters {
            let (name, initializer, dot_dot_dot_token) = match &parameter.data {
                NodeData::ParameterDeclaration(d) => {
                    (d.name.clone(), d.initializer.clone(), d.dot_dot_dot_token.clone())
                }
                _ => continue,
            };
            if initializer.is_some() || dot_dot_dot_token.is_some() {
                break;
            }
            let generated_name = f.generated_name_node(&f.new_generated_name_for_node_ex(
                &name,
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                    ..Default::default()
                },
            ));
            let new_parameter = f.new_parameter_declaration(None, None, &generated_name, None, None, None);
            new_parameters.push(new_parameter);
        }
        let mut new_parameters_array = f.new_node_list(new_parameters);
        if let Some(l) = parameter_list_of(node) {
            if let Some(np) = Arc::get_mut(&mut new_parameters_array) {
                np.loc = l.loc;
            }
        }
        new_parameters_array
    }

    pub fn transform_async_generator_function_body(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_async_generator_function_body"); 
        let mut emit_context = self.transformer.emit_context();
        let parameters = node
            .parameters()
            .map(|l| &l.nodes[..])
            .unwrap_or(&[]);
        let body = node.body().unwrap().clone();
        let mut inner_parameters: Option<Arc<NodeList>> = None;
        if !is_simple_parameter_list(parameters) {
            let parameter_list = parameter_list_of(node).unwrap();
            inner_parameters = Some(Arc::new(
                emit_context.visit_parameters(parameter_list.as_ref(), self.transformer.visitor()),
            ));
        }

        let saved_captured_super_properties = self.super_access_state.captured_super_properties.clone();
        let saved_has_super_element_access = self.super_access_state.has_super_element_access;
        let saved_has_super_property_assignment = self.super_access_state.has_super_property_assignment;
        let saved_super_binding = self.super_access_state.super_binding.clone();
        let saved_super_index_binding = self.super_access_state.super_index_binding.clone();
        self.super_access_state.captured_super_properties = Some(OrderedSet::new());
        self.super_access_state.has_super_element_access = false;
        self.super_access_state.has_super_property_assignment = false;
        {
            let f = self.transformer.factory();
            self.super_access_state.super_binding = Some(f.generated_name_node(&f.new_unique_name_ex(
                "_super",
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL,
                    ..Default::default()
                })));
            self.super_access_state.super_index_binding = Some(f.generated_name_node(&f.new_unique_name_ex(
                "_superIndex",
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL,
                    ..Default::default()
                })));
        }

        let body_statement_list = statement_list(&body);
        let visited_statements = self
            .transformer
            .visitor()
            .visit_nodes_r39k13(Some(body_statement_list.as_ref()))
            .unwrap_or_else(|| NodeList {
                loc: body_statement_list.loc,
                nodes: Vec::new(),
            });
        let mut async_body = {
            let f = self.transformer.factory();
            let mut async_body = f.update_block(&body, &visited_statements, block_multi_line(&body));
            let merged_list =
                emit_context.end_and_merge_variable_environment_list(Some(statement_list(&async_body).as_ref()));
            async_body = f.update_block(&async_body, merged_list.as_ref().unwrap(), block_multi_line(&async_body));
            async_body
        };

        let captured_size = self
            .super_access_state
            .captured_super_properties
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(0);
        let emit_super_helpers =
            captured_size > 0 || self.super_access_state.has_super_element_access;
        if emit_super_helpers {
            async_body = self.substitute_super_accesses_in_body(&async_body);
        }

        let return_statement = {
            let f = self.transformer.factory();
            let inner_params = match inner_parameters {
                Some(params) => params,
                None => f.new_node_list(vec![]),
            };

            let name = node
                .name()
                .map(|n| f.generated_name_node(&f.new_generated_name_for_node(n)));

            let asterisk_token = f.new_token(SyntaxKind::AsteriskToken);
            let generator_func = f.new_function_expression(
                None,
                Some(&asterisk_token),
                name.as_ref(),
                None,
                &inner_params,
                None,
                None,
                &async_body,
            );

            let has_lexical_this = self
                .for_await_hierarchy_facts
                .intersects(ForAwaitHierarchyFacts::HAS_LEXICAL_THIS);
            let helper = f.new_async_generator_helper(&generator_func, has_lexical_this);
            f.new_return_statement(Some(&helper))
        };

        emit_context.start_variable_environment();
        if emit_super_helpers && captured_size > 0 {
            let statement = self.create_super_access_variable_statement();
            emit_context.add_initialization_statement(&statement);
        }

        let mut block = {
            let f = self.transformer.factory();
            let outer_statements = f.new_node_list(vec![return_statement]);
            let merged_outer =
                emit_context.end_and_merge_variable_environment_list(Some(outer_statements.as_ref()));
            f.update_block(&body, merged_outer.as_ref().unwrap(), block_multi_line(&body))
        };

        if emit_super_helpers && self.super_access_state.has_super_element_access {
            if self.super_access_state.has_super_property_assignment {
                emit_context.add_emit_helper(&block, &[advanced_async_super_helper()]);
            } else {
                emit_context.add_emit_helper(&block, &[async_super_helper()]);
            }
        }

        self.super_access_state.captured_super_properties = saved_captured_super_properties;
        self.super_access_state.has_super_element_access = saved_has_super_element_access;
        self.super_access_state.has_super_property_assignment = saved_has_super_property_assignment;
        self.super_access_state.super_binding = saved_super_binding;
        self.super_access_state.super_index_binding = saved_super_index_binding;

        block
    }
}

fn parameter_list_of(node: &Arc<Node>) -> Option<Arc<NodeList>> { ::tsox_core::fntrace::enter("parameter_list_of"); 
    match &node.data {
        NodeData::MethodDeclaration(d) => Some(d.parameters.clone()),
        NodeData::FunctionDeclaration(d) => Some(d.parameters.clone()),
        NodeData::FunctionExpression(d) => Some(d.parameters.clone()),
        NodeData::ArrowFunction(d) => Some(d.parameters.clone()),
        NodeData::ConstructorDeclaration(d) => Some(d.parameters.clone()),
        NodeData::GetAccessorDeclaration(d) => Some(d.parameters.clone()),
        NodeData::SetAccessorDeclaration(d) => Some(d.parameters.clone()),
        _ => None,
    }
}

fn error_property_access(f: &Factory, error_record: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("error_property_access"); 
    let error_identifier = f.new_identifier("error");
    f.new_property_access_expression(error_record, None, &error_identifier, NodeFlags::empty())
}

fn block_statements(block: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("block_statements"); 
    match &block.data {
        NodeData::Block(d) => d.statements.nodes.clone(),
        _ => vec![],
    }
}

fn block_multi_line(block: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("block_multi_line"); 
    match &block.data {
        NodeData::Block(d) => d.multi_line,
        _ => false,
    }
}

fn statement_list(block: &Arc<Node>) -> Arc<NodeList> { ::tsox_core::fntrace::enter("statement_list"); 
    match &block.data {
        NodeData::Block(d) => d.statements.clone(),
        _ => panic!("statement_list: not a block"),
    }
}
