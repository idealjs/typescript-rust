#![allow(unused_imports)]
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeFlags};
use tsox_frontend::ast::{has_syntactic_modifier, is_binding_pattern, is_identifier};
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::node_data_generated::VariableStatementData;
use tsox_frontend::format::mig::m4o::EmitFlags;
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};
use super::m4i_9::r38k6_defs::R38K6DataExt;
use crate::mig::m4m_5::r38k9_defs::R38K9NodeCastExt;
use super::m4i_8::UsingDeclarationTransformer;
use super::m4i_11::convert_class_declaration_to_class_expression;
use super::m4h_2::{is_named_evaluation, transform_named_evaluation};
use crate::mig::m4m_2::{is_generated_identifier, is_local_name};
use crate::mig::m4m_5::convert_binding_pattern_to_assignment_pattern;
use crate::mig::m4e::r39k01_defs::R39K01DataExt;
impl UsingDeclarationTransformer {
    pub(crate) fn hoist_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("hoist_class_declaration"); 
        if node.name().is_none() && self.default_export_binding.is_some() {
            return node.clone();
        }

        let is_exported = has_syntactic_modifier(node, ModifierFlags::Export);
        let is_default = has_syntactic_modifier(node, ModifierFlags::Default);

        let mut expression = convert_class_declaration_to_class_expression(&self.emit_context, node);
        if let Some(name) = node.name() {
            self.hoist_binding_identifier(
                self.factory().get_local_name(node),
                is_exported && !is_default,
                None,
                node,
            );
            expression = self
                .factory()
                .new_assignment_expression(&self.factory().get_declaration_name(node), &expression);
            self.emit_context.set_original(&expression, node);
            self.emit_context.set_source_map_range(&expression, node.loc);
            self.emit_context.set_comment_range(&expression, node.loc);
            if is_named_evaluation(&self.emit_context, &expression) {
                expression =
                    transform_named_evaluation(&self.emit_context, &expression, false, "");
            }
        }

        if is_default && self.default_export_binding.is_none() {
            let binding = self.factory().generated_name_node(&self.factory().new_unique_name_ex(
                "_default",
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES
                        | GeneratedIdentifierFlags::FILE_LEVEL
                        | GeneratedIdentifierFlags::OPTIMISTIC,
                    ..Default::default()
                },
            ));
            self.default_export_binding = Some(binding.clone());
            self.hoist_binding_identifier(
                binding.clone(),
                true,
                Some(self.factory().new_identifier("default")),
                node,
            );
            expression = self.factory().new_assignment_expression(&binding, &expression);
            self.emit_context.set_original(&expression, node);
            if is_named_evaluation(&self.emit_context, &expression) {
                expression = transform_named_evaluation(&self.emit_context, &expression, false, "default");
            }
        }

        self.factory().new_expression_statement(&expression)
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn hoist_variable_statement(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("hoist_variable_statement"); 
        let data = node.as_variable_statement();
        let mut expressions: Vec<Arc<Node>> = Vec::new();
        let is_exported = has_syntactic_modifier(&node, ModifierFlags::Export);
        for variable in &data.declaration_list.as_variable_declaration_list().declarations.nodes {
            self.hoist_binding_element(variable, is_exported, variable);
            if variable.initializer().is_some() {
                expressions.push(self.hoist_initialized_variable(variable));
            }
        }
        if !expressions.is_empty() {
            let expression = self
                .factory()
                .inline_expressions(expressions)
                .unwrap();
            let statement = self
                .factory()
                .new_expression_statement(&expression);
            self.emit_context.set_original(&statement, &node);
            self.emit_context.set_comment_range(&statement, node.loc);
            self.emit_context.set_source_map_range(&statement, node.loc);
            return Some(statement);
        }
        None
    }

}
impl UsingDeclarationTransformer {
    fn hoist_initialized_variable(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("hoist_initialized_variable"); 
        let initializer = node
            .initializer()
            .expect("Expected initializer")
            .clone();
        let target = if is_identifier(node.name().unwrap()) {
            let target = node.name().unwrap().clone();
            self.emit_context.set_emit_flags(
                &target,
                self.emit_context.emit_flags(&target) & !(EmitFlags::LOCAL_NAME | EmitFlags::EXPORT_NAME),
            );
            target
        } else {
            convert_binding_pattern_to_assignment_pattern(&self.emit_context, node.name().unwrap())
        };

        let assignment = self.factory().new_assignment_expression(&target, &initializer);
        self.emit_context.set_original(&assignment, node);
        self.emit_context.set_comment_range(&assignment, node.loc);
        self.emit_context.set_source_map_range(&assignment, node.loc);
        assignment
    }

}
impl UsingDeclarationTransformer {
    fn hoist_binding_element(
        &mut self,
        node: &Arc<Node>,
        is_exported_declaration: bool,
        original: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("hoist_binding_element"); 
        if is_binding_pattern(node.name().unwrap()) {
            for element in &node.name().unwrap().as_binding_pattern().elements.nodes {
                if element.name().is_some() {
                    self.hoist_binding_element(element, is_exported_declaration, original);
                }
            }
        } else {
            self.hoist_binding_identifier(node.name().unwrap().clone(), is_exported_declaration, None, original);
        }
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn hoist_binding_identifier(
        &mut self,
        node: Arc<Node>,
        is_export: bool,
        export_alias: Option<Arc<Node>>,
        original: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("hoist_binding_identifier"); 
        let mut name = node;
        if !is_generated_identifier(&self.emit_context, &name) {
            name = Arc::clone(&name);
        }
        if is_export {
            if export_alias.is_none() && !is_local_name(&self.emit_context, &name) {
                let var_decl =
                    self.factory().new_variable_declaration(&name, None, None, None);
                self.emit_context.set_original(&var_decl, original);
                self.export_vars.push(var_decl);
                return;
            }

            let (local_name, export_name) = match export_alias {
                Some(alias) => (Some(name.clone()), alias),
                None => (None, name.clone()),
            };
            let specifier = self.factory().new_export_specifier(false, local_name.as_ref(), &export_name);
            self.emit_context.set_original(&specifier, original);
            let bindings = self.export_bindings.get_or_insert_with(HashMap::new);
            if !bindings.contains_key(name.text()) {
                self.export_binding_names.push(name.text().to_string());
            }
            bindings.insert(name.text().to_string(), specifier);
        }
        self.emit_context.add_variable_declaration(&name);
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn create_env_binding(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("create_env_binding"); 
        self.factory().generated_name_node(&self.factory().new_unique_name("env"))
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn create_downlevel_using_statements(
        &mut self,
        body_statements: Vec<Arc<Node>>,
        env_binding: &Arc<Node>,
        is_async: bool,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("create_downlevel_using_statements"); 
        let f = self.factory();
        let mut statements: Vec<Arc<Node>> = Vec::with_capacity(2);

        let env_object = f.new_object_literal_expression(
            &f.new_node_list(vec![
                f.new_property_assignment(
                    None,
                    &f.new_identifier("stack"),
                    None,
                    None,
                    &f.new_array_literal_expression(&f.new_node_list(Vec::new()), false),
                ),
                f.new_property_assignment(
                    None,
                    &f.new_identifier("error"),
                    None,
                    None,
                    &f.new_void_zero_expression(),
                ),
                f.new_property_assignment(
                    None,
                    &f.new_identifier("hasError"),
                    None,
                    None,
                    &f.new_false_expression(),
                ),
            ]),
            false,
        );
        let env_var = f.new_variable_declaration(env_binding, None, None, Some(&env_object));
        let env_var_list = f.new_variable_declaration_list(
            &f.new_node_list(vec![env_var]),
            NodeFlags::Const,
        );
        let env_var_statement = f.new_variable_statement(None, &env_var_list);
        statements.push(env_var_statement);

        let try_block = f.new_block(&f.new_node_list(body_statements), true);
        let body_catch_binding = f.generated_name_node(&f.new_unique_name("e"));
        let catch_clause = f.new_catch_clause(
            Some(&f.new_variable_declaration(&body_catch_binding, None, None, None)),
            &f.new_block(
                &f.new_node_list(vec![
                    f.new_expression_statement(&f.new_assignment_expression(
                        &f.new_property_access_expression(
                            env_binding,
                            None,
                            &f.new_identifier("error"),
                            NodeFlags::empty(),
                        ),
                        &body_catch_binding,
                    )),
                    f.new_expression_statement(&f.new_assignment_expression(
                        &f.new_property_access_expression(
                            env_binding,
                            None,
                            &f.new_identifier("hasError"),
                            NodeFlags::empty(),
                        ),
                        &f.new_true_expression(),
                    )),
                ]),
                true,
            ),
        );

        let finally_block = if is_async {
            let result = f.generated_name_node(&f.new_unique_name("result"));
            f.new_block(
                &f.new_node_list(vec![
                    f.new_variable_statement(
                        None,
                        &f.new_variable_declaration_list(
                            &f.new_node_list(vec![f.new_variable_declaration(
                                &result,
                                None,
                                None,
                                Some(&f.new_dispose_resources_helper(env_binding)),
                            )]),
                            NodeFlags::Const,
                        ),
                    ),
                    f.new_if_statement(
                        &result,
                        &f.new_expression_statement(&f.new_await_expression(&result)),
                        None,
                    ),
                ]),
                true,
            )
        } else {
            f.new_block(
                &f.new_node_list(vec![f.new_expression_statement(
                    &f.new_dispose_resources_helper(env_binding),
                )]),
                true,
            )
        };

        let try_statement = f.new_try_statement(&try_block, Some(&catch_clause), Some(&finally_block));
        statements.push(try_statement);
        statements
    }
}
