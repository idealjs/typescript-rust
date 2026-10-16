#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_checker::binder::referenceresolver::ReferenceResolver;
use tsox_core::core::compiler_options::{CompilerOptions, ModuleKind, ScriptTarget};
use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::utilities::has_syntactic_modifier;
use tsox_frontend::ast::mig::m3e_4::get_namespace_declaration_node;
use tsox_frontend::ast::mig::m3f_4::is_default_import;
use tsox_frontend::ast::mig::m3g_3::module_export_name_is_default;
use tsox_frontend::ast::mig::w7a::is_external_module_import_equals_declaration;
use tsox_frontend::ast::visitor::NodeVisitor;

#[path = "r39k02_defs.rs"]
pub mod r39k02_defs;
pub use r39k02_defs::R39K02NodeExt;
#[path = "r37k2_defs.rs"]
pub mod r37k2_defs;
pub use r37k2_defs::{R37K2NodeExt, R37K2NodeVisitorExt};
#[path = "r38k2_defs.rs"]
pub mod r38k2_defs;
use crate::mig::m4e::r37k1_defs::R37K1DataExt;
use crate::mig::m4q_4::r36k21_defs::NodeAsExt21;

use crate::mig::m4j_2::extract_modifiers;
use crate::mig::m4k_3::{collect_external_module_info, ExternalModuleInfo};
use crate::mig::m4m_2::{
    convert_variable_declaration_to_assignment_expression, is_helper_name, is_local_name,
    single_or_many,
};

pub use crate::mig::r33k6_shim::{HasFileName, Visitor};
pub use crate::printer::{EmitContext, NodeFactory};
pub use tsox_frontend::format::mig::m4o::EmitFlags;
pub use tsox_frontend::ast::ModifierList;

pub struct CommonJSModuleTransformer<'a> {
    pub emit_context: Arc<EmitContext>,
    pub compiler_options: &'a CompilerOptions,
    pub resolver: Arc<dyn ReferenceResolver>,
    pub get_emit_module_format_of_file: Arc<dyn Fn(&dyn HasFileName) -> ModuleKind + Send + Sync>,
    pub module_kind: ModuleKind,
    pub language_version: ScriptTarget,
    pub current_source_file: Option<Arc<Node>>,
    pub current_module_info: Option<ExternalModuleInfo>,
    pub parent_node: Option<Arc<Node>>,
    pub current_node: Option<Arc<Node>>,
}

impl<'a> CommonJSModuleTransformer<'a> {
    pub(crate) fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(&self.emit_context)
    }

    pub(crate) fn visitor(&self) -> Visitor { ::tsox_core::fntrace::enter("visitor"); 
        Visitor
    }

    pub(crate) fn discarded_value_visitor(&self) -> NodeVisitor { ::tsox_core::fntrace::enter("discarded_value_visitor"); 
        NodeVisitor::default()
    }

    pub(crate) fn top_level_nested_visitor(&self) -> NodeVisitor { ::tsox_core::fntrace::enter("top_level_nested_visitor"); 
        self.emit_context.new_node_visitor(Self::visit_top_level_nested)
    }

    pub(crate) fn push_node(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("push_node"); 
        let grandparent_node = self.parent_node.take();
        self.parent_node = self.current_node.take();
        self.current_node = Some(node);
        grandparent_node
    }

    pub(crate) fn pop_node(&mut self, grandparent_node: Option<Arc<Node>>) { ::tsox_core::fntrace::enter("pop_node"); 
        self.current_node = self.parent_node.take();
        self.parent_node = grandparent_node;
    }

    pub fn visit_top_level(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level"); 
        let grandparent_node = self.push_node(node.clone());
        let result = self.visit_top_level_inner(node);
        self.pop_node(grandparent_node);
        result
    }

    fn visit_top_level_inner(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_inner"); 
        match node.kind {
            SyntaxKind::ImportDeclaration => self.visit_top_level_import_declaration(node),
            SyntaxKind::ImportEqualsDeclaration => {
                self.visit_top_level_import_equals_declaration(node)
            }
            SyntaxKind::ExportDeclaration => self.visit_top_level_export_declaration(node),
            SyntaxKind::ExportAssignment => self.visit_top_level_export_assignment(node),
            SyntaxKind::FunctionDeclaration => self.visit_top_level_function_declaration(node),
            SyntaxKind::ClassDeclaration => self.visit_top_level_class_declaration(node),
            SyntaxKind::VariableStatement => self.visit_top_level_variable_statement(node),
            _ => self.visit_top_level_nested_no_stack(node),
        }
    }

    pub fn visit_top_level_nested(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested"); 
        let grandparent_node = self.push_node(node.clone());
        let result = self.visit_top_level_nested_no_stack(node);
        self.pop_node(grandparent_node);
        result
    }

    pub fn visit_top_level_nested_no_stack(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_no_stack"); 
        match node.kind {
            SyntaxKind::VariableStatement => self.visit_top_level_variable_statement(node),
            SyntaxKind::ForStatement => self.visit_top_level_nested_for_statement(node),
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.visit_top_level_nested_for_in_or_of_statement(node)
            }
            SyntaxKind::DoStatement => self.visit_top_level_nested_do_statement(node),
            SyntaxKind::WhileStatement => self.visit_top_level_nested_while_statement(node),
            SyntaxKind::LabeledStatement => self.visit_top_level_nested_labeled_statement(node),
            SyntaxKind::WithStatement => self.visit_top_level_nested_with_statement(node),
            SyntaxKind::IfStatement => self.visit_top_level_nested_if_statement(node),
            SyntaxKind::SwitchStatement => self.visit_top_level_nested_switch_statement(node),
            SyntaxKind::CaseBlock => self.visit_top_level_nested_case_block(node),
            SyntaxKind::CaseClause | SyntaxKind::DefaultClause => {
                self.visit_top_level_nested_case_or_default_clause(node)
            }
            SyntaxKind::TryStatement => self.visit_top_level_nested_try_statement(node),
            SyntaxKind::CatchClause => self.visit_top_level_nested_catch_clause(node),
            SyntaxKind::Block => self.visit_top_level_nested_block(node),
            _ => self.visit_no_stack(node, false),
        }
    }

    pub fn visit_source_file(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_source_file"); 
        if node.is_declaration_file_node()
            || !(node.is_effective_external_module_node(self.compiler_options)
                || node.subtree_facts().intersects(SubtreeFacts::DynamicImport))
        {
            return Some(node);
        }

        self.current_source_file = Some(node.clone());
        self.current_module_info = Some(collect_external_module_info(
            &node,
            self.compiler_options,
            r38k2_defs::clone_emit_context(&self.emit_context),
            self.resolver.clone(),
        ));
        let updated = self.transform_common_js_module(node);
        self.current_source_file = None;
        self.current_module_info = None;
        updated
    }

    pub fn visit_top_level_import_declaration(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_import_declaration"); 
        let import_clause = node.as_import_declaration().import_clause.clone();
        if import_clause.is_none() {
            let require_call = self.create_require_call(node.clone());
            let statement = self.factory().new_expression_statement(&require_call);
            self.emit_context.set_original(&statement, &node);
            self.emit_context
                .assign_comment_and_source_map_ranges(&statement, &node);
            return Some(statement);
        }

        let mut statements: Vec<Arc<Node>> = Vec::new();
        let mut variables: Vec<Arc<Node>> = Vec::new();
        let namespace_declaration = get_namespace_declaration_node(&node);
        if namespace_declaration.is_some() && !is_default_import(&node) {
            let name = namespace_declaration
                .as_ref()
                .unwrap()
                .node_name_r39k02()
                .unwrap();
            let require_call = self.create_require_call(node.clone());
            let helper = self.get_helper_expression_for_import(&node, require_call);
            variables.push(
                self.factory()
                    .new_variable_declaration(&name, None, None, Some(&helper)),
            );
        } else {
            let generated_name = self
                .factory()
                .generated_name_node(&self.factory().new_generated_name_for_node(&node));
            let require_call = self.create_require_call(node.clone());
            let helper = self.get_helper_expression_for_import(&node, require_call);
            variables.push(
                self.factory()
                    .new_variable_declaration(&generated_name, None, None, Some(&helper)),
            );

            if namespace_declaration.is_some() && is_default_import(&node) {
                let name = namespace_declaration
                    .as_ref()
                    .unwrap()
                    .node_name_r39k02()
                    .unwrap();
                let generated_name = self
                    .factory()
                    .generated_name_node(&self.factory().new_generated_name_for_node(&node));
                variables.push(
                    self.factory()
                        .new_variable_declaration(&name, None, None, Some(&generated_name)),
                );
            }
        }

        let var_statement = self.factory().new_variable_statement(
            None,
            &self.factory().new_variable_declaration_list(
                &self.factory().new_node_list(variables),
                NodeFlags::Const,
            ),
        );

        self.emit_context.set_original(&var_statement, &node);
        self.emit_context
            .assign_comment_and_source_map_ranges(&var_statement, &node);
        statements.push(var_statement);
        statements = self.append_exports_of_import_declaration(statements, &node);
        single_or_many(Some(statements.as_slice()), &self.factory())
    }

    pub fn visit_top_level_import_equals_declaration(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_import_equals_declaration"); 
        if !is_external_module_import_equals_declaration(&node) {
            panic!("import= for internal module references should be handled in an earlier transformer.");
        }

        let mut statements: Vec<Arc<Node>> = Vec::new();
        if has_syntactic_modifier(&node, ModifierFlags::Export) {
            let name = node.node_name_r39k02().unwrap();
            let require_call = self.create_require_call(node.clone());
            let export_expression =
                self.create_export_expression(&name, require_call, Some(node.loc), false);
            let statement = self.factory().new_expression_statement(&export_expression);

            self.emit_context.set_original(&statement, &node);
            self.emit_context
                .assign_comment_and_source_map_ranges(&statement, &node);
            statements.push(statement);
        } else {
            let name = deep_clone_node(&node.node_name_r39k02().unwrap());
            let require_call = self.create_require_call(node.clone());
            let var_decl =
                self.factory()
                    .new_variable_declaration(&name, None, None, Some(&require_call));
            let statement = self.factory().new_variable_statement(
                None,
                &self.factory().new_variable_declaration_list(
                    &self.factory().new_node_list(vec![var_decl]),
                    NodeFlags::Const,
                ),
            );
            self.emit_context.set_original(&statement, &node);
            self.emit_context
                .assign_comment_and_source_map_ranges(&statement, &node);
            statements.push(statement);
        }

        statements = self.append_exports_of_declaration(statements, &node, None, false);
        single_or_many(Some(statements.as_slice()), &self.factory())
    }

    pub fn visit_top_level_export_declaration(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_export_declaration"); 
        let export_clause = node.as_export_declaration().export_clause.clone();
        if node.as_export_declaration().module_specifier.is_none() {
            return None;
        }

        let generated_name = self
            .factory()
            .generated_name_node(&self.factory().new_generated_name_for_node(&node));
        if export_clause.as_ref().is_some_and(|c| is_named_exports(c)) {
            let mut statements: Vec<Arc<Node>> = Vec::new();
            let require_call = self.create_require_call(node.clone());
            let var_decl = self
                .factory()
                .new_variable_declaration(&generated_name, None, None, Some(&require_call));
            let var_statement = self.factory().new_variable_statement(
                None,
                &self.factory().new_variable_declaration_list(
                    &self.factory().new_node_list(vec![var_decl]),
                    NodeFlags::empty(),
                ),
            );
            self.emit_context.set_original(&var_statement, &node);
            self.emit_context
                .assign_comment_and_source_map_ranges(&var_statement, &node);
            statements.push(var_statement);

            for specifier in export_clause
                .as_ref()
                .unwrap()
                .elements_list_r39k02()
                .nodes
                .clone()
            {
                let specifier_name = specifier.property_name_or_name_r39k02();
                let export_needs_import_default =
                    module_export_name_is_default(&specifier_name);

                let target = if export_needs_import_default {
                    self.factory().new_import_default_helper(&generated_name)
                } else {
                    generated_name.clone()
                };

                let specifier_node_name = specifier.node_name_r39k02().unwrap();
                let export_name = if specifier_node_name.kind == SyntaxKind::StringLiteral {
                    self.factory().new_string_literal_from_node(&specifier_node_name)
                } else {
                    self.factory().get_export_name(&specifier)
                };

                let exported_value = if specifier_name.kind == SyntaxKind::StringLiteral {
                    self.factory().new_element_access_expression(
                        &target,
                        None,
                        &specifier_name,
                        NodeFlags::empty(),
                    )
                } else {
                    self.factory().new_property_access_expression(
                        &target,
                        None,
                        &specifier_name,
                        NodeFlags::empty(),
                    )
                };
                let exported_value_expression =
                    self.create_export_expression(&export_name, exported_value, None, true);
                let statement = self
                    .factory()
                    .new_expression_statement(&exported_value_expression);
                self.emit_context.set_original(&statement, &specifier);
                self.emit_context
                    .assign_comment_and_source_map_ranges(&statement, &specifier);
                statements.push(statement);
            }

            return single_or_many(Some(statements.as_slice()), &self.factory());
        }

        if let Some(export_clause) = export_clause {
            let clause_name = export_clause.node_name_r39k02().unwrap();
            let export_name = if clause_name.kind == SyntaxKind::StringLiteral {
                self.factory().new_string_literal_from_node(&clause_name)
            } else {
                clause_name.clone()
            };
            let require_call = self.create_require_call(node.clone());
            let helper = self.get_helper_expression_for_export(&node, require_call);
            let export_expression = self.create_export_expression(&export_name, helper, None, false);
            let statement = self
                .factory()
                .new_expression_statement(&export_expression);
            self.emit_context.set_original(&statement, &node);
            self.emit_context
                .assign_comment_and_source_map_ranges(&statement, &node);
            return Some(statement);
        }

        let require_call = self.create_require_call(node.clone());
        let helper = self
            .factory()
            .new_export_star_helper(&require_call, &self.factory().new_identifier("exports"));
        let visited = self.visitor().visit_node(helper)?;
        let statement = self.factory().new_expression_statement(&visited);
        self.emit_context.set_original(&statement, &node);
        self.emit_context
            .assign_comment_and_source_map_ranges(&statement, &node);
        Some(statement)
    }

    pub fn visit_top_level_export_assignment(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_export_assignment"); 
        if node.as_export_assignment().is_export_equals {
            return None;
        }

        let expression = self
            .visitor()
            .visit_node(node.as_export_assignment().expression.clone())?;
        Some(self.create_export_statement(
            self.factory().new_identifier("default"),
            expression,
            Some(node.loc),
            true,
            false,
        ))
    }

    pub fn visit_top_level_function_declaration(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_function_declaration"); 
        if has_syntactic_modifier(&node, ModifierFlags::Export) {
            let func = node.as_function_declaration();
            let modifiers = self.visitor().visit_modifiers(extract_modifiers(
                &self.emit_context,
                node.modifiers().map(|m| &**m),
                !ModifierFlags::ExportDefault,
            ));
            let asterisk_token = func.asterisk_token.clone();
            let name = self.factory().get_declaration_name(&node);
            let visited_parameters = self.visitor().visit_nodes(func.parameters.nodes.clone());
            let mut parameters = NodeList::new(visited_parameters);
            parameters.loc = func.parameters.loc;
            let body = self.visitor().visit_node(func.body.clone());
            Some(self.factory().update_function_declaration(
                &node,
                modifiers,
                asterisk_token.as_ref(),
                Some(&name),
                None,
                &parameters,
                None,
                None,
                body.as_ref(),
            ))
        } else {
            self.visitor().visit_each_child(node)
        }
    }

    pub fn visit_top_level_class_declaration(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_class_declaration"); 
        let mut statements: Vec<Arc<Node>> = Vec::new();
        if has_syntactic_modifier(&node, ModifierFlags::Export) {
            let cls = node.as_class_declaration();
            let modifiers = self.visitor().visit_modifiers(extract_modifiers(
                &self.emit_context,
                node.modifiers().map(|m| &**m),
                !ModifierFlags::ExportDefault,
            ));
            let name = self.factory().get_declaration_name(&node);
            let heritage_clauses = cls.heritage_clauses.as_ref().map(|l| {
                let mut list = NodeList::new(self.visitor().visit_nodes(l.nodes.clone()));
                list.loc = l.loc;
                list
            });
            let mut members = NodeList::new(self.visitor().visit_nodes(cls.members.nodes.clone()));
            members.loc = cls.members.loc;
            statements.push(self.factory().update_class_declaration(
                &node,
                modifiers,
                Some(&name),
                None,
                heritage_clauses.as_ref(),
                &members,
            ));
        } else {
            statements.push(self.visitor().visit_each_child(node.clone())?);
        }
        statements = self.append_exports_of_class_or_function_declaration(statements, &node);
        single_or_many(Some(statements.as_slice()), &self.factory())
    }

    pub fn visit_top_level_variable_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_variable_statement"); 
        let mut statements: Vec<Arc<Node>> = Vec::new();
        if has_syntactic_modifier(&node, ModifierFlags::Export) {
            let mut variables: Vec<Arc<Node>> = Vec::new();
            let mut expressions: Vec<Arc<Node>> = Vec::new();
            let mut modifiers: Option<Arc<ModifierList>> = None;

            macro_rules! commit_pending_variables {
                ($self:expr) => {
                    if !variables.is_empty() {
                        let variable_list = $self.factory().new_node_list(std::mem::take(&mut variables));
                        let declaration_list = &node.as_variable_statement().declaration_list;
                        let statement = $self.factory().update_variable_statement(
                            &node,
                            modifiers.clone(),
                            $self.factory().update_variable_declaration_list(
                                declaration_list,
                                &variable_list,
                                declaration_list.flags,
                            ),
                        );
                        if !statements.is_empty() {
                            $self.emit_context.add_emit_flags(&statement, EmitFlags::NO_COMMENTS);
                        }
                        statements.push(statement);
                    }
                };
            }

            macro_rules! commit_pending_expressions {
                ($self:expr) => {
                    if !expressions.is_empty() {
                        let statement = $self
                            .factory()
                            .new_expression_statement(&$self.factory().inline_expressions(std::mem::take(&mut expressions)).unwrap());
                        $self.emit_context.assign_comment_and_source_map_ranges(&statement, &node);
                        if !statements.is_empty() {
                            $self.emit_context.add_emit_flags(&statement, EmitFlags::NO_COMMENTS);
                        }
                        statements.push(statement);
                    }
                };
            }

            let declaration_list = node
                .as_variable_statement()
                .declaration_list
                .as_variable_declaration_list_r39k02();
            for variable in declaration_list.declarations.nodes.clone() {
                let v = variable.clone();
                let v_data = v.as_variable_declaration_r39k02();
                let name = v_data.name.clone();
                let exclamation_token = v_data.exclamation_token.clone();
                let type_node = v_data.type_node.clone();
                let initializer = v_data.initializer.clone();

                if is_identifier(&name) && is_local_name(&self.emit_context, &name) {
                    if modifiers.is_none() {
                        modifiers = self.visitor().visit_modifiers(extract_modifiers(
                            &self.emit_context,
                            node.modifiers().map(|m| &**m),
                            !ModifierFlags::ExportDefault,
                        ));
                    }

                    let variable = if initializer.is_some() {
                        let visited_initializer = self.visitor().visit_node(initializer.clone())?;
                        let export_expression = self.create_export_expression(
                            &name,
                            visited_initializer,
                            None,
                            false,
                        );
                        self.factory().new_variable_declaration(
                            &name,
                            exclamation_token.as_ref(),
                            type_node.as_ref(),
                            Some(&export_expression),
                        )
                    } else {
                        variable
                    };

                    commit_pending_expressions!(self);
                    variables.push(variable);
                } else if initializer.is_some()
                    && !is_binding_pattern(&name)
                    && (is_arrow_function(initializer.as_ref().unwrap())
                        || is_function_expression(initializer.as_ref().unwrap())
                        || is_class_expression(initializer.as_ref().unwrap()))
                {
                    commit_pending_expressions!(self);
                    let visited_initializer = self.visitor().visit_node(initializer.clone())?;
                    variables.push(self.factory().new_variable_declaration(
                        &name,
                        exclamation_token.as_ref(),
                        type_node.as_ref(),
                        Some(&visited_initializer),
                    ));

                    let property_access = self.factory().new_property_access_expression(
                        &self.factory().new_identifier("exports"),
                        None,
                        &name,
                        NodeFlags::empty(),
                    );
                    self.emit_context
                        .assign_comment_and_source_map_ranges(&property_access, &name);

                    commit_pending_variables!(self);
                    expressions.push(self.factory().new_assignment_expression(
                        &property_access,
                        &deep_clone_node(&name),
                    ));
                } else if is_identifier(&name) {
                    if let Some(expression) = self.transform_initialized_variable(&v) {
                        commit_pending_variables!(self);
                        expressions.push(self.visitor().visit_node(expression)?);
                    }
                } else if is_binding_pattern(&name) {
                    if let Some(expression) = self.transform_initialized_variable(&v) {
                        commit_pending_variables!(self);
                        expressions.push(expression);
                    }
                } else if let Some(expression) = convert_variable_declaration_to_assignment_expression(
                    &self.emit_context,
                    &v,
                ) {
                    commit_pending_variables!(self);
                    expressions.push(self.visitor().visit_node(expression)?);
                }
            }

            commit_pending_variables!(self);
            commit_pending_expressions!(self);
            statements = self.append_exports_of_variable_statement(statements, &node);
            return single_or_many(Some(statements.as_slice()), &self.factory());
        }
        self.visit_top_level_nested_variable_statement(node)
    }

    pub fn visit_top_level_nested_variable_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_variable_statement"); 
        let mut statements: Vec<Arc<Node>> = Vec::new();
        statements.push(self.visitor().visit_each_child(node.clone())?);
        statements = self.append_exports_of_variable_statement(statements, &node);
        single_or_many(Some(statements.as_slice()), &self.factory())
    }

    pub fn visit_top_level_nested_for_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_for_statement"); 
        let for_stmt = node.as_for_statement();
        let initializer = for_stmt.initializer.clone();
        if initializer.is_some()
            && is_variable_declaration_list(initializer.as_ref().unwrap())
            && initializer.as_ref().unwrap().flags & NodeFlags::BlockScoped == NodeFlags::empty()
        {
            let init_node = initializer.clone().unwrap();
            let export_statements = self.append_exports_of_variable_declaration_list(
                Vec::new(),
                &init_node,
                false,
            );
            if !export_statements.is_empty() {
                let mut statements: Vec<Arc<Node>> = Vec::new();
                let var_decl_list = self
                    .discarded_value_visitor()
                    .visit_node(&initializer.clone().unwrap());
                let var_statement = self.factory().new_variable_statement(None, &var_decl_list);
                statements.push(var_statement);
                statements.extend(export_statements);

                let condition = self.visitor().visit_node(for_stmt.condition.clone());
                let incrementor = self
                    .discarded_value_visitor()
                    .visit_node(&for_stmt.incrementor.clone().unwrap());
                let mut visitor = self.top_level_nested_visitor();
                let body = Arc::get_mut(&mut self.emit_context)
                    .expect("emit context uniquely owned")
                    .visit_iteration_body(Some(for_stmt.statement.clone()), &mut visitor);
                statements.push(self.factory().update_for_statement(
                    &node,
                    None,
                    condition.as_ref(),
                    Some(&incrementor),
                    body.as_ref().unwrap_or(&for_stmt.statement),
                ));
                return single_or_many(Some(statements.as_slice()), &self.factory());
            }
        }
        let initializer = self
            .discarded_value_visitor()
            .visit_node(&node.as_for_statement().initializer.clone().unwrap());
        let condition = self
            .visitor()
            .visit_node(node.as_for_statement().condition.clone());
        let incrementor = self
            .discarded_value_visitor()
            .visit_node(&node.as_for_statement().incrementor.clone().unwrap());
        let mut visitor = self.top_level_nested_visitor();
        let body = Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .visit_iteration_body(
                Some(node.as_for_statement().statement.clone()),
                &mut visitor,
            );
        Some(self.factory().update_for_statement(
            &node,
            Some(&initializer),
            condition.as_ref(),
            Some(&incrementor),
            body.as_ref().unwrap_or(&node.as_for_statement().statement),
        ))
    }

    pub fn visit_top_level_nested_for_in_or_of_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_for_in_or_of_statement"); 
        let for_stmt = node.as_for_in_or_of_statement_r39k02();
        let initializer = for_stmt.initializer.clone();
        if is_variable_declaration_list(&initializer)
            && initializer.flags & NodeFlags::BlockScoped == NodeFlags::empty()
        {
            let export_statements =
                self.append_exports_of_variable_declaration_list(Vec::new(), &initializer, true);
            if !export_statements.is_empty() {
                let initializer = self
                    .discarded_value_visitor()
                    .visit_node(&for_stmt.initializer.clone());
                let expression = self
                    .visitor()
                    .visit_node(Some(for_stmt.expression.clone()));
                let mut visitor = self.top_level_nested_visitor();
                let body_node = Arc::get_mut(&mut self.emit_context)
                    .expect("emit context uniquely owned")
                    .visit_iteration_body(
                        Some(for_stmt.statement.clone()),
                        &mut visitor,
                    );
                let body_node = body_node.unwrap_or_else(|| for_stmt.statement.clone());
                let body = if is_block(&body_node) {
                    let block = body_node.as_block();
                    let mut body_statements = export_statements;
                    body_statements.extend(block.statements.nodes.clone());
                    let mut body_statement_list = NodeList::new(body_statements);
                    body_statement_list.loc = block.statements.loc;
                    self.factory()
                        .update_block(&body_node, &body_statement_list, block.multi_line)
                } else {
                    let mut body_statements = export_statements;
                    body_statements.push(body_node);
                    self.factory()
                        .new_block(&NodeList::new(body_statements), true)
                };
                return Some(self.factory().update_for_in_or_of_statement(
                    &node,
                    for_stmt.await_modifier.clone(),
                    Some(initializer),
                    expression,
                    Some(body),
                ));
            }
        }
        let initializer = self
            .discarded_value_visitor()
            .visit_node(&node.as_for_in_or_of_statement_r39k02().initializer.clone());
        let expression = self
            .visitor()
            .visit_node(Some(node.as_for_in_or_of_statement_r39k02().expression.clone()));
        let mut visitor = self.top_level_nested_visitor();
        let statement = Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .visit_iteration_body(
                Some(node.as_for_in_or_of_statement_r39k02().statement.clone()),
                &mut visitor,
            );
        Some(self.factory().update_for_in_or_of_statement(
            &node,
            node.as_for_in_or_of_statement_r39k02().await_modifier.clone(),
            Some(initializer),
            expression,
            statement,
        ))
    }

    pub fn visit_top_level_nested_do_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_do_statement"); 
        let do_stmt = node.as_do_statement();
        let mut visitor = self.top_level_nested_visitor();
        let body = Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .visit_iteration_body(Some(do_stmt.statement.clone()), &mut visitor)
            .unwrap();
        let expression = self.visitor().visit_node(do_stmt.expression.clone()).unwrap();
        Some(self.factory().update_do_statement(&node, &body, &expression))
    }

    pub fn visit_top_level_nested_while_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_while_statement"); 
        let while_stmt = node.as_while_statement();
        let expression = self
            .visitor()
            .visit_node(while_stmt.expression.clone())
            .unwrap();
        let mut visitor = self.top_level_nested_visitor();
        let statement = Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .visit_iteration_body(Some(while_stmt.statement.clone()), &mut visitor)
            .unwrap();
        Some(self
            .factory()
            .update_while_statement(&node, &expression, &statement))
    }

    pub fn visit_top_level_nested_labeled_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_labeled_statement"); 
        let mut statement = self.top_level_nested_visitor().visit_embedded_statement(
            node.as_labeled_statement().statement.clone(),
        );
        if statement.is_none() {
            statement = Some(self.factory().new_empty_statement());
        }
        let label = node.as_labeled_statement().label.clone();
        Some(self.factory().update_labeled_statement(
            &node,
            &label,
            statement.as_ref().unwrap(),
        ))
    }

    pub fn visit_top_level_nested_with_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_with_statement"); 
        let with_stmt = node.as_with_statement();
        let expression = self.visitor().visit_node(with_stmt.expression.clone()).unwrap();
        let statement = self
            .top_level_nested_visitor()
            .visit_embedded_statement(with_stmt.statement.clone())
            .unwrap();
        Some(self
            .factory()
            .update_with_statement(&node, &expression, &statement))
    }

    pub fn visit_top_level_nested_if_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_if_statement"); 
        let if_stmt = node.as_if_statement();
        let expression = self.visitor().visit_node(if_stmt.expression.clone()).unwrap();
        let mut then_statement = self
            .top_level_nested_visitor()
            .visit_embedded_statement(if_stmt.then_statement.clone());
        if then_statement.is_none() {
            then_statement = Some(
                self.factory()
                    .new_block(&NodeList::new(Vec::new()), false),
            );
        }
        let else_statement = if_stmt
            .else_statement
            .clone()
            .and_then(|e| self.top_level_nested_visitor().visit_embedded_statement(e));
        Some(self.factory().update_if_statement(
            &node,
            &expression,
            then_statement.as_ref().unwrap(),
            else_statement.as_ref(),
        ))
    }

    pub fn visit_top_level_nested_switch_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_switch_statement"); 
        let switch_stmt = node.as_switch_statement_r39k02();
        let expression = self
            .visitor()
            .visit_node(switch_stmt.expression.clone())
            .unwrap();
        let case_block = self
            .top_level_nested_visitor()
            .visit_node(&switch_stmt.case_block);
        Some(self
            .factory()
            .update_switch_statement(&node, &expression, &case_block))
    }

    pub fn visit_top_level_nested_case_block(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_case_block"); 
        self.top_level_nested_visitor().visit_each_child(node)
    }

    pub fn visit_top_level_nested_case_or_default_clause(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_case_or_default_clause"); 
        let clause = node.as_case_or_default_clause_r39k02();
        let expression = if clause.expression.kind == SyntaxKind::Unknown {
            None
        } else {
            self.visitor().visit_node(clause.expression.clone())
        };
        let visited_statements = self
            .top_level_nested_visitor()
            .visit_node_list(Some(clause.statements.as_ref()));
        let mut statements = NodeList::new(visited_statements);
        statements.loc = clause.statements.loc;
        Some(self.factory().update_case_or_default_clause(
            &node,
            expression.as_ref(),
            &statements,
        ))
    }

    pub fn visit_top_level_nested_try_statement(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_try_statement"); 
        self.top_level_nested_visitor().visit_each_child(node)
    }

    pub fn visit_top_level_nested_catch_clause(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_catch_clause"); 
        let catch_clause = node.as_catch_clause();
        let variable_declaration = catch_clause.variable_declaration.clone();
        let block = self
            .top_level_nested_visitor()
            .visit_node(&catch_clause.block);
        Some(self.factory().update_catch_clause(
            &node,
            variable_declaration.as_ref(),
            &block,
        ))
    }

    pub fn visit_top_level_nested_block(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_top_level_nested_block"); 
        self.top_level_nested_visitor().visit_each_child(node)
    }

    pub fn visit_void_expression(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_void_expression"); 
        self.discarded_value_visitor().visit_each_child(node)
    }

    pub fn visit_tagged_template_expression(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_tagged_template_expression"); 
        let tagged = node.as_tagged_template_expression_r39k02();
        let tag = tagged.tag.clone();
        if is_identifier(&tag) {
            let expression = self.visit_expression_identifier(tag.clone());
            let updated_tag = expression.clone().unwrap_or_else(|| tag.clone());
            let template = tagged.template.clone();
            let visited_template = self.visitor().visit_node(template.clone());
            let updated = self.factory().update_tagged_template_expression(
                &node,
                &updated_tag,
                None,
                None,
                &visited_template.unwrap_or(template),
                node.flags,
            );
            if expression.is_some()
                && !is_identifier(&updated_tag)
                && !is_helper_name(&self.emit_context, &tag)
            {
                self.emit_context.add_emit_flags(&updated, EmitFlags::INDIRECT_CALL);
            }
            return Some(updated);
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_shorthand_property_assignment(
        &mut self,
        node: Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_shorthand_property_assignment"); 
        let shorthand = node.as_shorthand_property_assignment();
        let name = shorthand.name.clone();
        let exported_or_imported_name = self.visit_expression_identifier(name.clone());
        if !exported_or_imported_name
            .as_ref()
            .is_some_and(|n| Arc::ptr_eq(n, &name))
        {
            let mut expression = exported_or_imported_name.unwrap();
            if let Some(init) = shorthand.object_assignment_initializer.clone() {
                let visited_init = self.visitor().visit_node(init).unwrap();
                expression = self.factory().new_assignment_expression(&expression, &visited_init);
            }
            let mut assignment = self
                .factory()
                .new_property_assignment(None, &name, None, None, &expression);
            Arc::get_mut(&mut assignment).unwrap().loc = node.loc;
            self.emit_context
                .assign_comment_and_source_map_ranges(&assignment, &node);
            return Some(assignment);
        }
        let visited_initializer = shorthand
            .object_assignment_initializer
            .clone()
            .and_then(|i| self.visitor().visit_node(i));
        let mut updated = Node::new(
            SyntaxKind::ShorthandPropertyAssignment,
            NodeData::ShorthandPropertyAssignment(ShorthandPropertyAssignmentData {
                modifiers: shorthand.modifiers.clone(),
                name: name.clone(),
                postfix_token: shorthand.postfix_token.clone(),
                type_node: shorthand.type_node.clone(),
                equals_token: shorthand.equals_token.clone(),
                object_assignment_initializer: visited_initializer
                    .or_else(|| shorthand.object_assignment_initializer.clone()),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Some(Arc::new(updated))
    }
}
