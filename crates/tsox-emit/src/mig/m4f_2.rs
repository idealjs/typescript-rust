#![allow(unused_imports)]

use std::sync::Arc;

use tsox_core::collections::set::Set;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{
    is_binding_pattern, is_block, is_identifier, is_omitted_expression, NodeData,
};
use tsox_frontend::ast::mig::m3b::{parameter_list, parameters};
use tsox_frontend::ast::mig::m3c::statement_list;
use tsox_frontend::ast::mig::m3e::{get_function_flags, FunctionFlags};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node_name;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::is_function_like_declaration;
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::mig::m4e_3::is_simple_parameter_list;
use crate::mig::m4m_2::convert_binding_pattern_to_assignment_pattern;
use crate::mig::m4f::r33k11_defs::{advanced_async_super_helper, async_super_helper};
use crate::mig::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};

#[path = "r36k13_defs.rs"]
pub mod r36k13_defs;

#[path = "r38k5_defs.rs"]
pub mod r38k5_defs;

use super::m4f::{
    restore_super_access_state, reset_super_access_state, save_super_access_state,
    AsyncTransformer, LexicalArgumentsInfo,
};
use crate::mig::m4n::r37k19_defs::R37K19ArcNodeExt;
use self::r38k5_defs::R38K5NodeVisitorExt;

impl AsyncTransformer {
    pub fn visit_constructor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_lexical_arguments = std::mem::take(&mut self.lexical_arguments);
        let updated = match &node.data {
            NodeData::ConstructorDeclaration(d) => {
                let mut visitor = self.visitor();
                let modifiers = visitor.visit_modifiers(&d.modifiers);
                let parameters = visitor.visit_nodes_list(&d.parameters);
                let body = self.transform_method_body(node);
                self.factory().update_constructor_declaration(
                    node,
                    modifiers,
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                )
            }
            _ => {
                let mut visitor = self.visitor();
                match visitor.visit_each_child(node) {
                    Some(visited) => visited,
                    None => Arc::clone(node),
                }
            }
        };
        self.lexical_arguments = saved_lexical_arguments;
        Some(updated)
    }

    pub fn visit_method_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_lexical_arguments = std::mem::take(&mut self.lexical_arguments);
        let function_flags = get_function_flags(Some(node));

        let (parameters, body) = if function_flags.contains(FunctionFlags::ASYNC) {
            let parameters = self.transform_async_function_parameter_list(node);
            let body = self.transform_async_function_body(node, &parameters);
            (parameters, body)
        } else {
            match &node.data {
                NodeData::MethodDeclaration(d) => {
                    let mut visitor = self.visitor();
                    let parameters = visitor.visit_nodes_list(&d.parameters);
                    let body = self.transform_method_body(node);
                    (parameters, body)
                }
                _ => {
                    let mut visitor = self.visitor();
                    return match visitor.visit_each_child(node) {
                        Some(visited) => Some(visited),
                        None => Some(Arc::clone(node)),
                    };
                }
            }
        };

        let updated = match &node.data {
            NodeData::MethodDeclaration(d) => {
                let mut visitor = self.visitor();
                let modifiers = visitor.visit_modifiers(&d.modifiers);
                self.factory().update_method_declaration(
                    node,
                    modifiers,
                    d.asterisk_token.as_ref(),
                    &d.name,
                    None,
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                )
            }
            _ => {
                let mut visitor = self.visitor();
                return match visitor.visit_each_child(node) {
                    Some(visited) => Some(visited),
                    None => Some(Arc::clone(node)),
                };
            }
        };
        self.lexical_arguments = saved_lexical_arguments;
        Some(updated)
    }

    pub fn visit_get_accessor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_lexical_arguments = std::mem::take(&mut self.lexical_arguments);
        let updated = match &node.data {
            NodeData::GetAccessorDeclaration(d) => {
                let mut visitor = self.visitor();
                let modifiers = visitor.visit_modifiers(&d.modifiers);
                let parameters = visitor.visit_nodes_list(&d.parameters);
                let body = self.transform_method_body(node);
                self.factory().update_get_accessor_declaration(
                    node,
                    modifiers,
                    &d.name,
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                )
            }
            _ => {
                let mut visitor = self.visitor();
                match visitor.visit_each_child(node) {
                    Some(visited) => visited,
                    None => Arc::clone(node),
                }
            }
        };
        self.lexical_arguments = saved_lexical_arguments;
        Some(updated)
    }

    pub fn visit_set_accessor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_lexical_arguments = std::mem::take(&mut self.lexical_arguments);
        let updated = match &node.data {
            NodeData::SetAccessorDeclaration(d) => {
                let mut visitor = self.visitor();
                let modifiers = visitor.visit_modifiers(&d.modifiers);
                let parameters = visitor.visit_nodes_list(&d.parameters);
                let body = self.transform_method_body(node);
                self.factory().update_set_accessor_declaration(
                    node,
                    modifiers,
                    &d.name,
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                )
            }
            _ => {
                let mut visitor = self.visitor();
                match visitor.visit_each_child(node) {
                    Some(visited) => visited,
                    None => Arc::clone(node),
                }
            }
        };
        self.lexical_arguments = saved_lexical_arguments;
        Some(updated)
    }

    pub fn visit_function_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_lexical_arguments = std::mem::take(&mut self.lexical_arguments);
        let function_flags = get_function_flags(Some(node));

        let (parameters, body) = if function_flags.contains(FunctionFlags::ASYNC) {
            let parameters = self.transform_async_function_parameter_list(node);
            let body = self.transform_async_function_body(node, &parameters);
            (parameters, body)
        } else {
            match &node.data {
                NodeData::FunctionDeclaration(d) => {
                    let mut visitor = self.visitor();
                    let parameters = visitor.visit_nodes_list(&d.parameters);
                    let body = self.emit_context_mut().visit_function_body(d.body.clone(), &mut visitor);
                    (parameters, body)
                }
                _ => {
                    let mut visitor = self.visitor();
                    return match visitor.visit_each_child(node) {
                        Some(visited) => Some(visited),
                        None => Some(Arc::clone(node)),
                    };
                }
            }
        };

        let updated = match &node.data {
            NodeData::FunctionDeclaration(d) => {
                let mut visitor = self.visitor();
                let modifiers = visitor.visit_modifiers(&d.modifiers);
                let name = d.name.as_ref().map(|n| visitor.visit_node(n));
                self.factory().update_function_declaration(
                    node,
                    modifiers,
                    d.asterisk_token.as_ref(),
                    name.as_ref(),
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                )
            }
            _ => {
                let mut visitor = self.visitor();
                return match visitor.visit_each_child(node) {
                    Some(visited) => Some(visited),
                    None => Some(Arc::clone(node)),
                };
            }
        };
        self.lexical_arguments = saved_lexical_arguments;
        Some(updated)
    }

    pub fn visit_function_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved_lexical_arguments = std::mem::take(&mut self.lexical_arguments);
        let function_flags = get_function_flags(Some(node));

        let (parameters, body) = if function_flags.contains(FunctionFlags::ASYNC) {
            let parameters = self.transform_async_function_parameter_list(node);
            let body = self.transform_async_function_body(node, &parameters);
            (parameters, body)
        } else {
            match &node.data {
                NodeData::FunctionExpression(d) => {
                    let mut visitor = self.visitor();
                    let parameters = visitor.visit_nodes_list(&d.parameters);
                    let body = self.emit_context_mut().visit_function_body(Some(Arc::clone(&d.body)), &mut visitor);
                    (parameters, body)
                }
                _ => {
                    let mut visitor = self.visitor();
                    return match visitor.visit_each_child(node) {
                        Some(visited) => Some(visited),
                        None => Some(Arc::clone(node)),
                    };
                }
            }
        };

        let updated = match &node.data {
            NodeData::FunctionExpression(d) => {
                let mut visitor = self.visitor();
                let modifiers = visitor.visit_modifiers(&d.modifiers);
                let name = d.name.as_ref().map(|n| visitor.visit_node(n));
                self.factory().update_function_expression(
                    node,
                    modifiers,
                    d.asterisk_token.as_ref(),
                    name.as_ref(),
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                )
            }
            _ => {
                let mut visitor = self.visitor();
                return match visitor.visit_each_child(node) {
                    Some(visited) => Some(visited),
                    None => Some(Arc::clone(node)),
                };
            }
        };
        self.lexical_arguments = saved_lexical_arguments;
        Some(updated)
    }

    pub fn visit_arrow_function(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let no_lexical_arguments = self
            .emit_context_mut()
            .emit_flags(node)
            .contains(EmitFlags::NO_LEXICAL_ARGUMENTS);
        let saved_lexical_arguments = if no_lexical_arguments {
            Some(std::mem::take(&mut self.lexical_arguments))
        } else {
            None
        };

        let function_flags = get_function_flags(Some(node));
        let (parameters, body) = if function_flags.contains(FunctionFlags::ASYNC) {
            let parameters = self.transform_async_function_parameter_list(node);
            let body = self.transform_async_function_body(node, &parameters);
            (Some(parameters), body)
        } else {
            match &node.data {
                NodeData::ArrowFunction(d) => {
                    let mut visitor = self.visitor();
                    let parameters = visitor.visit_nodes_list(&d.parameters);
                    let body = self.emit_context_mut().visit_function_body(Some(Arc::clone(&d.body)), &mut visitor);
                    (Some(parameters), body)
                }
                _ => {
                    let mut visitor = self.visitor();
                    return match visitor.visit_each_child(node) {
                        Some(visited) => Some(visited),
                        None => Some(Arc::clone(node)),
                    };
                }
            }
        };

        let result = match &node.data {
            NodeData::ArrowFunction(d) => {
                let mut visitor = self.visitor();
                let modifiers = visitor.visit_modifiers(&d.modifiers);
                self.factory().update_arrow_function(
                    node,
                    modifiers,
                    None,
                    parameters.as_deref().unwrap_or(&d.parameters),
                    None,
                    None,
                    &d.equals_greater_than_token,
                    body.as_ref().unwrap_or(&d.body),
                )
            }
            _ => {
                let mut visitor = self.visitor();
                return match visitor.visit_each_child(node) {
                    Some(visited) => Some(visited),
                    None => Some(Arc::clone(node)),
                };
            }
        };
        if let Some(saved) = saved_lexical_arguments {
            self.lexical_arguments = saved;
        }
        Some(result)
    }

    pub fn visit_variable_declaration_list_with_colliding_names(
        &mut self,
        node: &Arc<Node>,
        has_receiver: bool,
    ) -> Option<Arc<Node>> {
        self.hoist_variable_declaration_list(node);

        let declarations = match &node.data {
            NodeData::VariableDeclarationList(d) => Arc::clone(&d.declarations),
            _ => return None,
        };
        let variables: Vec<&Arc<Node>> = declarations
            .nodes
            .iter()
            .filter(|decl| match &decl.data {
                NodeData::VariableDeclaration(d) => d.initializer.is_some(),
                _ => false,
            })
            .collect();

        if variables.is_empty() {
            if has_receiver {
                let name: Option<Arc<Node>> = node_name(&declarations.nodes[0]).cloned();
                let target: Option<Arc<Node>> = match &name {
                    Some(n) if is_binding_pattern(n) => Some(
                        convert_binding_pattern_to_assignment_pattern(self.emit_context_mut(), n),
                    ),
                    other => other.clone(),
                };
                return self.visitor().visit_node_option(target.as_ref());
            }
            return None;
        }

        let expressions: Vec<Arc<Node>> = variables
            .iter()
            .map(|variable| self.transform_initialized_variable(variable))
            .collect();
        self.factory().inline_expressions(expressions)
    }

    pub fn transform_initialized_variable(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let (name, initializer, loc) = match &node.data {
            NodeData::VariableDeclaration(d) => (
                Arc::clone(&d.name),
                d.initializer.clone(),
                node.loc,
            ),
            _ => return Arc::clone(node),
        };
        let target = if is_binding_pattern(&name) {
            convert_binding_pattern_to_assignment_pattern(self.emit_context_mut(), &name)
        } else {
            name
        };
        let initializer = initializer.expect("initialized variable initializer");
        let converted = self
            .factory()
            .new_assignment_expression(&target, &initializer);
        self.emit_context_mut().set_source_map_range(&converted, loc);
        let mut visitor = self.visitor();
        visitor.visit_node(&converted)
    }

    pub fn transform_method_body(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let saved = save_super_access_state(&mut self.super_access);
        reset_super_access_state(self, true);

        self.emit_context_mut().start_variable_environment();
        let body = node.body().cloned();
        let mut visitor = self.visitor();
        let mut updated = match self
            .emit_context_mut()
            .visit_function_body(body, &mut visitor)
        {
            Some(body) => body,
            None => return None,
        };

        let emit_super_helpers = (self.super_access.captured_super_properties.as_ref().map(|p| p.len()).unwrap_or(0) > 0
            || self.super_access.has_super_element_access)
            && !get_function_flags(Some(&self.get_original_if_function_like(node)))
                .contains(FunctionFlags::ASYNC_GENERATOR);

        if emit_super_helpers {
            if self.super_access.captured_super_properties.as_ref().map(|p| p.len()).unwrap_or(0) > 0 {
                let statement = self.create_super_access_variable_statement();
                self.emit_context_mut().add_initialization_statement(&statement);
            }
        }

        let merged_statements = self
            .emit_context_mut()
            .end_and_merge_variable_environment_list(statement_list(&updated).map(|l| &**l));
        let merged_list = merged_statements.unwrap_or_default();
        let multi_line = match &updated.data {
            NodeData::Block(d) => d.multi_line,
            _ => false,
        };
        if emit_super_helpers && self.super_access.has_super_element_access && !multi_line {
            let mut new_block = self.factory().new_block(&merged_list, true);
            if let Some(b) = Arc::get_mut(&mut new_block) {
                b.loc = updated.loc;
            }
            updated = new_block;
        } else {
            updated = self.factory().update_block(&updated, &merged_list, multi_line);
        }

        if emit_super_helpers && self.super_access.has_super_element_access {
            if self.super_access.has_super_property_assignment {
                self.emit_context_mut().add_emit_helper(&updated, &[advanced_async_super_helper()]);
            } else {
                self.emit_context_mut().add_emit_helper(&updated, &[async_super_helper()]);
            }
        }

        restore_super_access_state(&mut self.super_access, saved);
        Some(updated)
    }

    pub fn transform_async_function_parameter_list(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<NodeList> {
        if is_simple_parameter_list(parameters(node)) {
            let mut visitor = self.visitor();
            return match parameter_list(node) {
                Some(list) => visitor.visit_nodes_list(list),
                None => self.factory().new_node_list(Vec::new()),
            };
        }

        let mut new_parameters: Vec<Arc<Node>> = Vec::new();
        for parameter in parameters(node) {
            let (initializer, dot_dot_dot_token, name) = match &parameter.data {
                NodeData::ParameterDeclaration(d) => {
                    (d.initializer.is_some(), d.dot_dot_dot_token.is_some(), d.name.clone())
                }
                _ => continue,
            };
            if initializer || dot_dot_dot_token {
                if node.kind == SyntaxKind::ArrowFunction {
                    let dot_dot_dot_token = self.factory().new_token(SyntaxKind::DotDotDotToken);
                    let args_name = self.factory().generated_name_node(&self.factory().new_unique_name_ex(
                        "args",
                        AutoGenerateOptions {
                            flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                            prefix: String::new(),
                            suffix: String::new(),
                        },
                    ));
                    let rest_parameter = self.factory().new_parameter_declaration(
                        None,
                        Some(&dot_dot_dot_token),
                        &args_name,
                        None,
                        None,
                        None,
                    );
                    new_parameters.push(rest_parameter);
                }
                break;
            }
            let generated_name = self.factory().generated_name_node(
                &self.factory().new_generated_name_for_node_ex(
                    &name,
                    AutoGenerateOptions {
                        flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                        prefix: String::new(),
                        suffix: String::new(),
                    },
                ),
            );
            let new_parameter = self.factory().new_parameter_declaration(
                None,
                None,
                &generated_name,
                None,
                None,
                None,
            );
            new_parameters.push(new_parameter);
        }
        let mut new_parameters_array = self.factory().new_node_list(new_parameters);
        if let Some(list) = parameter_list(node) {
            if let Some(l) = Arc::get_mut(&mut new_parameters_array) {
                l.loc = list.loc;
            }
        }
        new_parameters_array
    }

    pub fn transform_async_function_body(
        &mut self,
        node: &Arc<Node>,
        outer_parameters: &Arc<NodeList>,
    ) -> Option<Arc<Node>> {
        let is_arrow = node.kind == SyntaxKind::ArrowFunction;
        let saved = save_super_access_state(&mut self.super_access);
        if !is_arrow {
            reset_super_access_state(self, true);
        }

        let mut inner_parameters: Option<Arc<NodeList>> = None;
        if !is_simple_parameter_list(parameters(node)) {
            let mut visitor = self.visitor();
            inner_parameters = Some(match parameter_list(node) {
                Some(list) => visitor.visit_nodes_list(list),
                None => self.factory().new_node_list(Vec::new()),
            });
        }

        let saved_lexical_arguments = LexicalArgumentsInfo {
            binding: self.lexical_arguments.binding.clone(),
            used: self.lexical_arguments.used,
        };
        let capture_lexical_arguments = self.lexical_arguments.binding.is_none();
        if capture_lexical_arguments {
            let binding = self
                .factory()
                .generated_name_node(&self.factory().new_unique_name("arguments"));
            self.lexical_arguments = LexicalArgumentsInfo {
                binding: Some(binding),
                used: false,
            };
        }

        let mut arguments_expression: Option<Arc<Node>> = None;
        if inner_parameters.is_some() {
            if is_arrow {
                let mut parameter_bindings: Vec<Arc<Node>> = Vec::new();
                let outer_len = outer_parameters.nodes.len();
                for (i, param) in parameters(node).iter().enumerate() {
                    if i >= outer_len {
                        break;
                    }
                    let (initializer, dot_dot_dot_token, name) = match &param.data {
                        NodeData::ParameterDeclaration(d) => {
                            (d.initializer.is_some(), d.dot_dot_dot_token.is_some(), d.name.clone())
                        }
                        _ => continue,
                    };
                    let outer_name = match &outer_parameters.nodes[i].data {
                        NodeData::ParameterDeclaration(d) => d.name.clone(),
                        _ => continue,
                    };
                    if initializer || dot_dot_dot_token {
                        parameter_bindings.push(self.factory().new_spread_element(outer_name));
                        break;
                    }
                    parameter_bindings.push(outer_name);
                }
                let bindings_list = self.factory().new_node_list(parameter_bindings);
                arguments_expression =
                    Some(self.factory().new_array_literal_expression(&bindings_list, false));
            } else {
                arguments_expression = Some(self.factory().new_identifier("arguments"));
            }
        }

        let saved_enclosing_function_parameter_names = self.enclosing_function_parameter_names.take();
        let mut enclosing = Set::<String>::new();
        for parameter in parameters(node) {
            self.record_declaration_name(parameter, &mut enclosing);
        }
        self.enclosing_function_parameter_names = Some(enclosing);

        let has_lexical_this = self.in_has_lexical_this_context();

        let mut async_body = self.transform_async_function_body_worker(&node.body().unwrap());
        let async_multi_line = match &async_body.data {
            NodeData::Block(d) => d.multi_line,
            _ => false,
        };
        let merged_statements = self
            .emit_context_mut()
            .end_and_merge_variable_environment_list(statement_list(&async_body).map(|l| &**l));
        let merged_list = merged_statements.unwrap_or_default();
        async_body = self.factory().update_block(&async_body, &merged_list, async_multi_line);

        let emit_super_helpers = self.super_access.captured_super_properties.is_some()
            && (self.super_access.captured_super_properties.as_ref().map(|p| p.len()).unwrap_or(0) > 0
                || self.super_access.has_super_element_access);
        if emit_super_helpers {
            let visited = self
                .super_access_visitor
                .as_mut()
                .map(|v| v.visit_node_list(inner_parameters.as_deref()))
                .unwrap_or_default();
            let mut visited_list = self.factory().new_node_list(visited);
            if let Some(original) = inner_parameters.as_ref() {
                if let Some(l) = Arc::get_mut(&mut visited_list) {
                    l.loc = original.loc;
                }
            }
            inner_parameters = Some(visited_list);
            async_body = self.substitute_super_accesses_in_body(async_body);
        }

        let result: Arc<Node>;
        if !is_arrow {
            self.emit_context_mut().start_variable_environment();

            if emit_super_helpers
                && self.super_access.captured_super_properties.as_ref().map(|p| p.len()).unwrap_or(0) > 0
            {
                let statement = self.create_super_access_variable_statement();
                self.emit_context_mut().add_initialization_statement(&statement);
            }

            if capture_lexical_arguments && self.lexical_arguments.used {
                let statement = self.create_capture_arguments_statement();
                self.emit_context_mut().add_initialization_statement(&statement);
            }

            let awaiter = self.factory().new_awaiter_helper(
                has_lexical_this,
                arguments_expression.as_ref(),
                inner_parameters,
                &async_body,
            );
            let statements = vec![self.factory().new_return_statement(Some(&awaiter))];

            let statements_list = self.factory().new_node_list(statements);
            let merged = self
                .emit_context_mut()
                .end_and_merge_variable_environment_list(Some(statements_list.as_ref()));
            let merged_list = merged.unwrap_or_default();
            let mut block = self.factory().new_block(&merged_list, true);
            if let Some(b) = Arc::get_mut(&mut block) {
                b.loc = node.body().unwrap().loc;
            }

            if emit_super_helpers && self.super_access.has_super_element_access {
                if self.super_access.has_super_property_assignment {
                    self.emit_context_mut().add_emit_helper(&block, &[advanced_async_super_helper()]);
                } else {
                    self.emit_context_mut().add_emit_helper(&block, &[async_super_helper()]);
                }
            }

            result = block;
        } else {
            let mut awaiter = self.factory().new_awaiter_helper(
                has_lexical_this,
                arguments_expression.as_ref(),
                inner_parameters,
                &async_body,
            );

            if capture_lexical_arguments && self.lexical_arguments.used {
                let block = self
                    .emit_context_mut()
                    .convert_to_function_block(&awaiter, true);
                if !is_block(&awaiter) {
                    let first = &statement_list(&block).unwrap().nodes[0];
                    self.emit_context_mut().set_original(first, &awaiter);
                }
                let block_multi_line = match &block.data {
                    NodeData::Block(d) => d.multi_line,
                    _ => false,
                };
                let capture_statement = self.create_capture_arguments_statement();
                let merged = self.emit_context_mut().merge_environment_list(
                    statement_list(&block).unwrap(),
                    vec![capture_statement],
                );
                awaiter = self.factory().update_block(&block, &merged, block_multi_line);
            }
            result = awaiter;
        }

        self.enclosing_function_parameter_names = saved_enclosing_function_parameter_names;
        if !is_arrow {
            restore_super_access_state(&mut self.super_access, saved);
            self.lexical_arguments = saved_lexical_arguments;
        } else if capture_lexical_arguments && !self.lexical_arguments.used {
            self.lexical_arguments = saved_lexical_arguments;
        } else if capture_lexical_arguments {
            self.lexical_arguments.used = false;
        }
        Some(result)
    }

    pub fn transform_async_function_body_worker(
        &mut self,
        body: &Arc<Node>,
    ) -> Arc<Node> {
        if is_block(body) {
            let multi_line = match &body.data {
                NodeData::Block(d) => d.multi_line,
                _ => false,
            };
            let visited = self
                .async_body_visitor
                .as_mut()
                .map(|v| v.visit_node_list(statement_list(body).map(|l| &**l)))
                .unwrap_or_default();
            let list = self.factory().new_node_list(visited);
            return self.factory().update_block(body, &list, multi_line);
        }
        let visited = self
            .async_body_visitor
            .as_mut()
            .map(|v| v.visit_node(body))
            .unwrap_or_else(|| Arc::clone(body));
        let mut ret = self.factory().new_return_statement(Some(&visited));
        ret.set_loc(body.loc);
        let mut list = self.factory().new_node_list(vec![ret]);
        if let Some(l) = Arc::get_mut(&mut list) {
            l.loc = body.loc;
        }
        let mut block = self.factory().new_block(&list, false);
        block.set_loc(body.loc);
        block
    }

    pub fn hoist_variable_declaration_list(&mut self, node: &Arc<Node>) {
        if let NodeData::VariableDeclarationList(d) = &node.data {
            for decl in &d.declarations.nodes {
                self.hoist_variable(decl);
            }
        }
    }

    pub fn hoist_variable(&mut self, node: &Arc<Node>) {
        let name = match node.name() {
            None => return,
            Some(name) => name,
        };
        if is_identifier(&name) {
            self.emit_context_mut().add_variable_declaration(&name);
        } else if is_binding_pattern(&name) {
            if let NodeData::BindingPattern(d) = &name.data {
                for element in &d.elements.nodes {
                    if !is_omitted_expression(element) {
                        self.hoist_variable(element);
                    }
                }
            }
        }
    }

    pub fn get_original_if_function_like(&self, node: &Arc<Node>) -> Arc<Node> {
        let original = self.emit_context().most_original(node);
        if is_function_like_declaration(&original) {
            original
        } else {
            Arc::clone(node)
        }
    }

    pub fn create_capture_arguments_statement(&mut self) -> Arc<Node> {
        let binding = self.lexical_arguments.binding.clone().unwrap();
        let initializer = self.factory().new_identifier("arguments");
        let variable = self
            .factory()
            .new_variable_declaration(&binding, None, None, Some(&initializer));
        let list = self.factory().new_node_list(vec![variable]);
        let decl_list = self
            .factory()
            .new_variable_declaration_list(&list, NodeFlags::empty());
        let statement = self.factory().new_variable_statement(None, &decl_list);
        self.emit_context_mut()
            .add_emit_flags(&statement, EmitFlags::START_ON_NEW_LINE | EmitFlags::CUSTOM_PROLOGUE);
        statement
    }

    pub fn create_super_access_variable_statement(&mut self) -> Arc<Node> {
        let f = self.factory();
        let mut accessors: Vec<Arc<Node>> = Vec::new();

        for name in self
            .super_access
            .captured_super_properties
            .as_ref()
            .unwrap()
            .iter()
        {
            let mut descriptor_properties: Vec<Arc<Node>> = Vec::new();

            let getter_body = f.new_property_access_expression(
                &f.new_keyword_expression(SyntaxKind::SuperKeyword),
                None,
                &f.new_identifier(name),
                NodeFlags::empty(),
            );
            let getter_parameters = f.new_node_list(Vec::new());
            let getter_arrow = f.new_arrow_function(
                None,
                None,
                &getter_parameters,
                None,
                None,
                &f.new_token(SyntaxKind::EqualsGreaterThanToken),
                &getter_body,
            );
            let getter = f.new_property_assignment(
                None,
                &f.new_identifier("get"),
                None,
                None,
                &getter_arrow,
            );
            descriptor_properties.push(getter);

            if self.super_access.has_super_property_assignment {
                let v_param = f.new_parameter_declaration(
                    None,
                    None,
                    &f.new_identifier("v"),
                    None,
                    None,
                    None,
                );
                let super_prop = f.new_property_access_expression(
                    &f.new_keyword_expression(SyntaxKind::SuperKeyword),
                    None,
                    &f.new_identifier(name),
                    NodeFlags::empty(),
                );
                let assign_expr = f.new_assignment_expression(&super_prop, &f.new_identifier("v"));
                let setter_parameters = f.new_node_list(vec![v_param]);
                let setter_arrow = f.new_arrow_function(
                    None,
                    None,
                    &setter_parameters,
                    None,
                    None,
                    &f.new_token(SyntaxKind::EqualsGreaterThanToken),
                    &assign_expr,
                );
                let setter = f.new_property_assignment(
                    None,
                    &f.new_identifier("set"),
                    None,
                    None,
                    &setter_arrow,
                );
                descriptor_properties.push(setter);
            }

            let descriptor =
                f.new_object_literal_expression(&f.new_node_list(descriptor_properties), false);
            let accessor =
                f.new_property_assignment(None, &f.new_identifier(name), None, None, &descriptor);
            accessors.push(accessor);
        }

        let descriptors_object = f.new_object_literal_expression(&f.new_node_list(accessors), true);

        let object_create_call = f.new_call_expression(
            &f.new_property_access_expression(
                &f.new_identifier("Object"),
                None,
                &f.new_identifier("create"),
                NodeFlags::empty(),
            ),
            None,
            None,
            f.new_node_list(vec![
                f.new_keyword_expression(SyntaxKind::NullKeyword),
                descriptors_object,
            ]),
            NodeFlags::empty(),
        );

        let super_binding = self.super_access.super_binding.clone().unwrap();
        let decl = f.new_variable_declaration(&super_binding, None, None, Some(&object_create_call));
        let list = f.new_node_list(vec![decl]);
        let decl_list = f.new_variable_declaration_list(&list, NodeFlags::Const);
        f.new_variable_statement(None, &decl_list)
    }

    pub fn substitute_super_accesses_in_body(&mut self, body: Arc<Node>) -> Arc<Node> {
        self.super_access_visitor
            .as_mut()
            .map(|v| v.visit_node(&body))
            .unwrap_or_else(|| Arc::clone(&body))
    }
}
