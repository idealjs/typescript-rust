#![allow(unused_imports)]
#[path = "r39k09_defs.rs"]
pub mod r39k09_defs;

use std::collections::HashSet;
use std::sync::Arc;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Node, NodeFlags, SyntaxKind};
use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::mig::m3c_2::subtree_facts;
use tsox_frontend::ast::{is_binding_pattern, is_block, is_prologue_directive, has_syntactic_modifier};
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::format::mig::m4o::EmitFlags;
use super::m4i_3::ObjectRestSpreadTransformer;
use super::m4q_4::r36k21_defs::NodeAsExt21;
use crate::mig::m4j::r36k3_defs::R36K3NodeAccessExt;
use crate::mig::m4m_5::r38k9_defs::R38K9NodeCastExt;
use crate::mig::m4n_5::r36k29_defs::R36K29NodeExt;
use crate::mig::m4e_2::{FlattenLevel, flatten_destructuring_binding};
use r39k09_defs::R39K09ObjectRestSpreadTxExt;
impl ObjectRestSpreadTransformer {
    pub(crate) fn transform_function_body(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_function_body"); 
        self.emit_context.start_variable_environment();
        let mut body = self.visit_node(node.body());
        let mut extras = self.emit_context.end_variable_environment();
        self.emit_context.start_variable_environment();
        let new_statements = self.collect_object_rest_assignments(node);
        let (merged_extras, _) = self.emit_context.end_and_merge_variable_environment(&extras);
        extras = merged_extras;
        if new_statements.is_empty() && extras.is_empty() {
            return body;
        }

        let mut suffix: Vec<Arc<Node>> = Vec::new();
        let body = match body {
            Some(b) if is_block(&b) => b,
            Some(b) => {
                let mut ret = self.factory().new_return_statement(Some(&b));
                if let Some(ret) = Arc::get_mut(&mut ret) {
                    ret.loc = b.loc;
                }
                let mut list = self.factory().new_node_list(Vec::new());
                if let Some(list) = Arc::get_mut(&mut list) {
                    list.loc = b.loc;
                }
                let block = self.factory().new_block(&list, true);
                suffix.push(ret);
                block
            }
            None => self
                .factory()
                .new_block(&self.factory().new_node_list(Vec::new()), true),
        };

        let mut prefix: Vec<Arc<Node>> = Vec::new();
        {
            let mut custom = false;
            for (i, statement) in body.statements().iter().enumerate() {
                if !custom && is_prologue_directive(statement) {
                    prefix.push(Arc::clone(statement));
                } else if self
                    .emit_context
                    .emit_flags(statement)
                    .intersects(EmitFlags::CUSTOM_PROLOGUE)
                {
                    custom = true;
                    prefix.push(Arc::clone(statement));
                } else {
                    suffix = body.statements()[i..].to_vec();
                    break;
                }
            }
        }

        let mut statements = prefix;
        statements.extend(extras);
        statements.extend(new_statements);
        statements.extend(suffix);
        let mut new_statement_list = self.factory().new_node_list(statements);
        if let Some(new_statement_list) = Arc::get_mut(&mut new_statement_list) {
            new_statement_list.loc = body.statement_list().loc;
        }
        Some(
            self.factory()
                .update_block(&body, &new_statement_list, body.as_block().multi_line),
        )
    }

}
impl ObjectRestSpreadTransformer {
    fn collect_object_rest_assignments(&mut self, node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("collect_object_rest_assignments"); 
        let mut contains_preceding_object_rest_or_spread = false;
        let mut results: Vec<Arc<Node>> = Vec::new();
        for parameter in tsox_frontend::ast::mig::m3b::parameters(node) {
            if contains_preceding_object_rest_or_spread {
                if is_binding_pattern(parameter.name().unwrap()) {
                    if !parameter
                        .name()
                        .unwrap()
                        .elements()
                        .map_or(false, |e| !e.nodes.is_empty())
                    {
                        let mut tx = self.flatten_tx();
                        let declarations = flatten_destructuring_binding(
                            &mut tx,
                            parameter.clone(),
                            Some(self.factory().generated_name_node(&self.factory().new_generated_name_for_node(parameter))),
                            FlattenLevel::All,
                            false,
                            false,
                        );
                        if let Some(declarations) = declarations {
                            let statement = self.new_flattened_variable_statement(&declarations);
                            results.push(statement);
                        }
                    } else if parameter.initializer().is_some() {
                        let name = self.factory().generated_name_node(&self.factory().new_generated_name_for_node(parameter));
                        let initializer = self.visit_node(parameter.initializer()).unwrap();
                        let assignment = self.factory().new_assignment_expression(&name, &initializer);
                        let statement = self.factory().new_expression_statement(&assignment);
                        self.emit_context
                            .add_emit_flags(&statement, EmitFlags::CUSTOM_PROLOGUE);
                        results.push(statement);
                    }
                } else if parameter.initializer().is_some() {
                    let mut name = deep_clone_node(parameter.name().unwrap());
                    if let Some(name) = Arc::get_mut(&mut name) {
                        name.loc = parameter.name().unwrap().loc;
                    }
                    self.emit_context.add_emit_flags(&name, EmitFlags::NO_SOURCE_MAP);

                    let initializer = self.visit_node(parameter.initializer()).unwrap();
                    self.emit_context
                        .add_emit_flags(&initializer, EmitFlags::NO_SOURCE_MAP | EmitFlags::NO_COMMENTS);

                    let mut assignment = self
                        .factory()
                        .new_assignment_expression(&name, &initializer);
                    if let Some(assignment) = Arc::get_mut(&mut assignment) {
                        assignment.loc = parameter.loc;
                    }
                    self.emit_context.add_emit_flags(&assignment, EmitFlags::NO_COMMENTS);

                    let mut block = self
                        .factory()
                        .new_block(
                            &self.factory()
                                .new_node_list(vec![self.factory().new_expression_statement(&assignment)]),
                            false,
                        );
                    if let Some(block) = Arc::get_mut(&mut block) {
                        block.loc = parameter.loc;
                    }
                    self.emit_context.add_emit_flags(
                        &block,
                        EmitFlags::SINGLE_LINE
                            | EmitFlags::NO_TRAILING_SOURCE_MAP
                            | EmitFlags::NO_TOKEN_SOURCE_MAPS
                            | EmitFlags::NO_COMMENTS,
                    );

                    let type_check = self
                        .factory()
                        .new_type_check(&name, "undefined");
                    let mut statement = self.factory().new_if_statement(&type_check, &block, None);
                    if let Some(statement) = Arc::get_mut(&mut statement) {
                        statement.loc = parameter.loc;
                    }
                    self.emit_context.add_emit_flags(
                        &statement,
                        EmitFlags::NO_TOKEN_SOURCE_MAPS
                            | EmitFlags::NO_TRAILING_SOURCE_MAP
                            | EmitFlags::CUSTOM_PROLOGUE
                            | EmitFlags::NO_COMMENTS
                            | EmitFlags::START_ON_NEW_LINE,
                    );
                    results.push(statement);
                }
            } else if subtree_facts(parameter).intersects(SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD) {
                contains_preceding_object_rest_or_spread = true;
                let mut tx = self.flatten_tx();
                let declarations = flatten_destructuring_binding(
                    &mut tx,
                    parameter.clone(),
                    Some(self.factory().generated_name_node(&self.factory().new_generated_name_for_node(parameter))),
                    FlattenLevel::ObjectRest,
                    false,
                    true,
                );
                if let Some(declarations) = declarations {
                    let statement = self.new_flattened_variable_statement(&declarations);
                    results.push(statement);
                }
            }
        }

        results
    }

}
impl ObjectRestSpreadTransformer {
    fn new_flattened_variable_statement(&mut self, declarations: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_flattened_variable_statement"); 
        let decls: Vec<Arc<Node>> = if declarations.kind == SyntaxKind::SyntaxList {
            declarations.as_syntax_list().children.clone()
        } else {
            vec![declarations.clone()]
        };
        let declaration_list = self.factory().new_variable_declaration_list(
            &self.factory().new_node_list(decls),
            NodeFlags::default(),
        );
        let statement = self.factory().new_variable_statement(None, &declaration_list);
        self.emit_context
            .add_emit_flags(&statement, EmitFlags::CUSTOM_PROLOGUE);
        statement
    }

}
impl ObjectRestSpreadTransformer {
    pub(crate) fn visit_catch_clause(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_catch_clause"); 
        let data = node.as_catch_clause();
        let variable_declaration = data.variable_declaration.clone();
        if let Some(vd) = &variable_declaration {
            if is_binding_pattern(vd.name().unwrap())
                && subtree_facts(vd.name().unwrap())
                    .intersects(SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD)
            {
                let name = self
                    .factory()
                    .generated_name_node(&self.factory().new_generated_name_for_node(vd.name().unwrap()));
                let updated_decl = self.factory().update_variable_declaration_r39k13(
                    vd,
                    vd.name().unwrap(),
                    None,
                    None,
                    Some(&name),
                );
                let mut tx = self.flatten_tx();
                let visited_bindings = flatten_destructuring_binding(
                    &mut tx,
                    updated_decl,
                    None,
                    FlattenLevel::ObjectRest,
                    false,
                    false,
                );
                let mut block = self.visit_node(Some(&data.block));
                if let Some(visited_bindings) = visited_bindings {
                    let decls: Vec<Arc<Node>> = if visited_bindings.kind == SyntaxKind::SyntaxList {
                        visited_bindings.as_syntax_list().children.clone()
                    } else {
                        vec![visited_bindings]
                    };
                    let new_statement = self.factory().new_variable_statement(
                        None,
                        &self.factory().new_variable_declaration_list(
                            &self.factory().new_node_list(decls),
                            NodeFlags::default(),
                        ),
                    );
                    if let Some(b) = block.take() {
                        let mut statements = vec![new_statement];
                        statements.extend(b.statements().iter().cloned());
                        let mut statement_list = self.factory().new_node_list(statements);
                        if let Some(statement_list) = Arc::get_mut(&mut statement_list) {
                            statement_list.loc = b.statement_list().loc;
                        }
                        block = Some(
                            self.factory()
                                .update_block(&b, &statement_list, b.as_block().multi_line),
                        );
                    }
                }
                return self.factory().update_catch_clause(
                    &node,
                    Some(&self.factory().update_variable_declaration_r39k13(
                        vd,
                        &name,
                        None,
                        None,
                        None,
                    )),
                    block.as_ref().unwrap(),
                );
            }
        }
        self.visit_each_child(&node)
    }

}
impl ObjectRestSpreadTransformer {
    pub(crate) fn visit_variable_statement(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_variable_statement"); 
        if has_syntactic_modifier(&node, ModifierFlags::Export) {
            let old_in_exported_variable_statement = self.in_exported_variable_statement;
            self.in_exported_variable_statement = true;
            let result = self.visit_each_child(&node);
            self.in_exported_variable_statement = old_in_exported_variable_statement;
            return result;
        }
        self.visit_each_child(&node)
    }

}
impl ObjectRestSpreadTransformer {
    pub(crate) fn visit_variable_declaration(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_variable_declaration"); 
        if self.in_exported_variable_statement {
            self.in_exported_variable_statement = false;
            let result = self.visit_variable_declaration_worker(&node, true);
            self.in_exported_variable_statement = true;
            return result;
        }
        self.visit_variable_declaration_worker(&node, false)
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_variable_declaration_worker(&mut self, node: &Arc<Node>, exported: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_variable_declaration_worker"); 
        if is_binding_pattern(node.name().unwrap())
            && subtree_facts(node).intersects(SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD)
        {
            let mut tx = self.flatten_tx();
            return flatten_destructuring_binding(
                &mut tx,
                node.clone(),
                None,
                FlattenLevel::ObjectRest,
                exported,
                false,
            )
            .unwrap_or_else(|| node.clone());
        }
        self.visit_each_child(node)
    }

}

impl ObjectRestSpreadTransformer {
    pub(crate) fn visit_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_node"); 
        let node = node?;
        Some(self.visit(node.clone()))
    }

    pub(crate) fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child"); 
        if node.kind == SyntaxKind::SourceFile {
            return self.visit(node.clone());
        }
        let mut changed = false;
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            let visited = self.visit(child.clone());
            changed |= !Arc::ptr_eq(&visited, child);
            true
        });
        if !changed {
            return node.clone();
        }
        panic!(
            "ObjectRestSpreadTransformer visit_each_child rebuild for kind {:?} pending factory update-function port (progress_notes_r36k21.md)",
            node.kind
        )
    }
}
