#![allow(unused_imports)]

#[path = "r38k6_defs.rs"]
pub mod r38k6_defs;

use std::collections::HashMap;
use std::sync::Arc;
use r38k6_defs::R38K6DataExt;
use tsox_frontend::ast::{Node, NodeFlags, SyntaxKind};
use tsox_frontend::ast::{is_block, is_identifier};
use super::m4q_4::r36k21_defs::NodeAsExt21;
use crate::mig::m4j::r36k3_defs::R36K3NodeAccessExt;
use crate::mig::m4n_5::r36k29_defs::R36K29NodeExt;
use tsox_frontend::ast::node_data_generated::ExportAssignmentData;
use tsox_frontend::ast::mig::m3g_3::{OuterExpressionKinds, skip_outer_expressions};
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};
use super::m4i_8::{
    UsingDeclarationTransformer, UsingKind, get_using_kind,
    get_using_kind_of_variable_declaration_list, is_using_variable_declaration_list,
};
use tsox_core::core::mig::m3j::first_or_nil;
use super::m4h_2::{is_named_evaluation, transform_named_evaluation};
use crate::mig::m4e::r39k01_defs::R39K01DataExt;
impl UsingDeclarationTransformer {
    pub(crate) fn visit_for_of_statement(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_for_of_statement"); 
        let data = node.as_for_in_or_of_statement();
        let initializer = data.initializer.clone();
        if is_using_variable_declaration_list(&initializer) {
            let for_initializer = initializer.as_variable_declaration_list();
            let mut for_decl = first_or_nil(&for_initializer.declarations.nodes);
            if for_decl.is_none() {
                for_decl = Some(self.factory().new_variable_declaration(
                    &self.factory().generated_name_node(&self.factory().new_temp_variable()),
                    None,
                    None,
                    None,
                ));
            }
            let for_decl = for_decl.unwrap();

            let is_await_using =
                get_using_kind_of_variable_declaration_list(&initializer) == UsingKind::Async;
            let temp = self.factory().generated_name_node(&self.factory().new_generated_name_for_node(for_decl.name().unwrap()));
            let using_var = self.factory().update_variable_declaration_r39k13(
                &for_decl,
                for_decl.name().unwrap(),
                None,
                None,
                Some(&temp),
            );
            let using_var_list = self.factory().new_variable_declaration_list(
                &self.factory().new_node_list(vec![using_var]),
                if is_await_using {
                    NodeFlags::AwaitUsing
                } else {
                    NodeFlags::Using
                },
            );
            let using_var_statement = self.factory().new_variable_statement(None, &using_var_list);
            let statement = if is_block(&data.statement) {
                let mut statements: Vec<Arc<Node>> = vec![using_var_statement];
                statements.extend(data.statement.statements().iter().cloned());
                self.factory().update_block(
                    &data.statement,
                    &self.factory().new_node_list(statements),
                    data.statement.as_block().multi_line,
                )
            } else {
                self.factory().new_block(
                    &self
                        .factory()
                        .new_node_list(vec![using_var_statement, data.statement.clone()]),
                    true,
                )
            };
            return self
                .visit_node(
                    self.factory().update_for_in_or_of_statement(
                        &node,
                        data.await_modifier.clone(),
                        Some(self.factory().new_variable_declaration_list(
                            &self.factory().new_node_list(vec![
                                self.factory().new_variable_declaration(&temp, None, None, None),
                            ]),
                            NodeFlags::Const,
                        )),
                        Some(data.expression.clone()),
                        Some(statement),
                    ),
                )
                .unwrap();
        }
        self.visit_each_child(node)
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn transform_using_declarations(
        &mut self,
        statements_in: &[Arc<Node>],
        env_binding: &Arc<Node>,
        mut top_level_statements: Option<&mut Vec<Arc<Node>>>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("transform_using_declarations"); 
        let mut statements: Vec<Arc<Node>> = Vec::new();

        for statement in statements_in {
            let using_kind = get_using_kind(statement);
            if using_kind != UsingKind::None {
                let var_statement = statement.as_variable_statement();
                let declaration_list = &var_statement.declaration_list;
                let mut declarations: Vec<Arc<Node>> = Vec::new();
                let mut valid = true;
                for declaration in &declaration_list.as_variable_declaration_list().declarations.nodes {
                    if !declaration.name().is_some_and(|n| is_identifier(n)) {
                        valid = false;
                        break;
                    }

                    let declaration = if is_named_evaluation(&self.emit_context, declaration) {
                        transform_named_evaluation(&self.emit_context, declaration, false, "")
                    } else {
                        declaration.clone()
                    };

                    let initializer = match declaration.initializer() {
                        Some(init) => self
                            .visit_node(Arc::clone(init))
                            .unwrap_or_else(|| self.factory().new_void_zero_expression()),
                        None => self.factory().new_void_zero_expression(),
                    };
                    let disposable = self.factory().new_add_disposable_resource_helper(
                        env_binding,
                        &initializer,
                        using_kind == UsingKind::Async,
                    );
                    declarations.push(self.factory().update_variable_declaration_r39k13(
                        &declaration,
                        declaration.name().unwrap(),
                        None,
                        None,
                        Some(&disposable),
                    ));
                }

                if valid && !declarations.is_empty() {
                    let mut var_list = self.factory().new_variable_declaration_list(
                        &self.factory().new_node_list(declarations),
                        NodeFlags::Const,
                    );
                    if let Some(l) = Arc::get_mut(&mut var_list) {
                        l.loc = declaration_list.loc;
                    }
                    self.emit_context.set_original(&var_list, declaration_list);
                    let updated = self
                        .factory()
                        .update_variable_statement(statement, None, var_list);
                    self.hoist_or_append(&mut statements, &mut top_level_statements, updated);
                    continue;
                }
            }

            let result = self.visit(statement.clone());
            if result.kind == SyntaxKind::SyntaxList {
                for node in &R38K6DataExt::as_syntax_list(&*result).children {
                    self.hoist_or_append(&mut statements, &mut top_level_statements, node.clone());
                }
            } else {
                self.hoist_or_append(&mut statements, &mut top_level_statements, result);
            }
        }
        statements
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn hoist_or_append(
        &mut self,
        statements: &mut Vec<Arc<Node>>,
        top_level_statements: &mut Option<&mut Vec<Arc<Node>>>,
        node: Arc<Node>,
    ) { ::tsox_core::fntrace::enter("hoist_or_append"); 
        let hoisted = match self.hoist(node, top_level_statements) {
            Some(n) => n,
            None => return,
        };
        statements.push(hoisted);
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn hoist(
        &mut self,
        node: Arc<Node>,
        top_level_statements: &mut Option<&mut Vec<Arc<Node>>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("hoist"); 
        let top_level = match top_level_statements {
            None => return Some(node),
            Some(tl) => tl,
        };

        match node.kind {
            SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::FunctionDeclaration => {
                self.hoist_import_or_export_or_hoisted_declaration(node, top_level);
                None
            }
            SyntaxKind::ExportAssignment => Some(self.hoist_export_assignment(node.clone())),
            SyntaxKind::ClassDeclaration => Some(self.hoist_class_declaration(&node)),
            SyntaxKind::VariableStatement => self.hoist_variable_statement(node.clone()),
            _ => Some(node),
        }
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn visit_each_child(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child"); 
        let mut changed = false;
        tsox_frontend::ast::node_data_generated::for_each_child(&node, |child| {
            let visited = self.visit(child.clone());
            changed |= !Arc::ptr_eq(&visited, child);
            true
        });
        if !changed {
            return node;
        }
        panic!(
            "UsingDeclarationTransformer visit_each_child rebuild for kind {:?} pending factory update-function port (progress_notes_r39k09.md)",
            node.kind
        )
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn hoist_import_or_export_or_hoisted_declaration(
        &mut self,
        node: Arc<Node>,
        top_level_statements: &mut Vec<Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("hoist_import_or_export_or_hoisted_declaration"); 
        top_level_statements.push(node);
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn hoist_export_assignment(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("hoist_export_assignment"); 
        if node.as_export_assignment().is_export_equals {
            self.hoist_export_equals(node)
        } else {
            self.hoist_export_default(node)
        }
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn hoist_export_default(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("hoist_export_default"); 
        let data = node.as_export_assignment();
        if self.default_export_binding.is_some() {
            return node.clone();
        }

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
            &node,
        );

        let mut expression = data.expression.clone();
        let inner_expression = skip_outer_expressions(&expression, OuterExpressionKinds::ALL);
        if is_named_evaluation(&self.emit_context, &inner_expression) {
            let updated =
                transform_named_evaluation(&self.emit_context, &inner_expression, false, "default");
            expression = self
                .factory()
                .restore_outer_expressions(&expression, &updated, OuterExpressionKinds::ALL);
        }

        let assignment = self.factory().new_assignment_expression(&binding, &expression);
        self.factory().new_expression_statement(&assignment)
    }

}
impl UsingDeclarationTransformer {
    pub(crate) fn hoist_export_equals(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("hoist_export_equals"); 
        let data = node.as_export_assignment();
        if self.export_equals_binding.is_some() {
            return node.clone();
        }

        let binding = self.factory().generated_name_node(&self.factory().new_unique_name_ex(
            "_default",
            AutoGenerateOptions {
                flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES
                    | GeneratedIdentifierFlags::FILE_LEVEL
                    | GeneratedIdentifierFlags::OPTIMISTIC,
                ..Default::default()
            },
        ));
        self.export_equals_binding = Some(binding.clone());
        self.emit_context.add_variable_declaration(&binding);

        let assignment = self.factory().new_assignment_expression(&binding, &data.expression);
        self.factory().new_expression_statement(&assignment)
    }

}
