use std::sync::Arc;
use tsox_frontend::ast::*;

#[path = "r36k28_defs.rs"]
pub mod r36k28_defs;
use self::r36k28_defs::R36K28NodeVisitorExt;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

use crate::mig::m4h::r39k13_defs;
use crate::mig::m4h::r40k17_defs::clone_class_info;
use crate::mig::m4h_2::inject_class_named_evaluation_helper_block_if_missing;
use crate::mig::m4h_5::{ClassInfo, EsDecoratorTransformer, LexicalEntry, LexicalEntryKind};
use crate::mig::m4m_2::{find_super_statement_index_path, single_or_many};
use crate::mig::m4m_4::move_range_past_decorators;
use crate::mig::m4n_4::AssignedNameOptions;
use crate::mig::x6a::is_decorated_class_like;
use tsox_frontend::ast::mig::m3b::{member_list, members};
use tsox_frontend::ast::utilities::get_heritage_clauses as heritage_clauses;

impl EsDecoratorTransformer {
    pub fn visit_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_class_declaration"); 
        if is_decorated_class_like(node) {
            let f = self.transformer.factory();
            let mut ec = self.transformer.emit_context();
            let mut statements: Vec<Arc<Node>> = Vec::new();

            let mut original_class = ec.most_original(node);
            if !is_class_like(&original_class) {
                original_class = node.clone();
            }
            let class_name = match original_class.name() {
                Some(name) => f.new_string_literal_from_node(name),
                None => f.new_string_literal("default", TOKEN_FLAGS_NONE),
            };

            let is_export = has_syntactic_modifier(node, ModifierFlags::Export);
            let is_default = has_syntactic_modifier(node, ModifierFlags::Default);

            let mut class_node = node.clone();
            if node.name().is_none() {
                class_node = inject_class_named_evaluation_helper_block_if_missing(
                    &ec,
                    &class_node,
                    &class_name,
                    None,
                );
            }

            if is_export && is_default {
                let iife = self.transform_class_like(&class_node);
                let f = self.transformer.factory();
                if class_node.name().is_some() {
                    let local_name = f.get_local_name(&class_node);
                    let var_decl = f.new_variable_declaration(&local_name, None, None, Some(&iife));
                    ec.set_original(&var_decl, &class_node);
                    let decls = f.new_node_list(vec![var_decl]);
                    let var_decls = f.new_variable_declaration_list(&decls, NodeFlags::Let);
                    statements.push(f.new_variable_statement(None, &var_decls));
                    let export_statement = f.new_export_default(&f.get_declaration_name(&class_node));
                    ec.set_original(&export_statement, &class_node);
                    ec.assign_comment_range(&export_statement, &class_node);
                    ec.set_source_map_range(&export_statement, move_range_past_decorators(&class_node));
                    statements.push(export_statement);
                } else {
                    let export_statement = f.new_export_default(&iife);
                    ec.set_original(&export_statement, &class_node);
                    ec.assign_comment_range(&export_statement, &class_node);
                    ec.set_source_map_range(&export_statement, move_range_past_decorators(&class_node));
                    statements.push(export_statement);
                }
            } else {
                assert!(class_node.name().is_some());
                let iife = self.transform_class_like(&class_node);
                let f = self.transformer.factory();
                let modifiers = self
                    .export_stripping_modifier_visitor
                    .visit_modifiers(class_node.modifiers());

                let decl_name = f.get_local_name_ex(
                    &class_node,
                    AssignedNameOptions {
                        allow_comments: false,
                        allow_source_maps: true,
                        ignore_assigned_name: false,
                    },
                );
                let var_decl = f.new_variable_declaration(&decl_name, None, None, Some(&iife));
                ec.set_original(&var_decl, &class_node);
                let decls = f.new_node_list(vec![var_decl]);
                let var_decls = f.new_variable_declaration_list(&decls, NodeFlags::Let);
                let var_statement = f.new_variable_statement(modifiers.clone(), &var_decls);
                ec.set_original(&var_statement, &class_node);
                ec.assign_comment_range(&var_statement, &class_node);
                statements.push(var_statement);

                if is_export {
                    let export_statement = f.new_external_module_export(&decl_name);
                    ec.set_original(&export_statement, &class_node);
                    statements.push(export_statement);
                }
            }

            return single_or_many(Some(statements.as_slice()), &self.transformer.factory())
                .expect("expected statement");
        }

        let modifiers = self.modifier_visitor.visit_modifiers(node.modifiers());
        let heritage_clauses = self.transformer.visitor().visit_nodes_opt(heritage_clauses(node));
        let top = self.top.take();
        self.top = Some(Box::new(LexicalEntry {
            kind: LexicalEntryKind::Class,
            next: top,
            class_info_data: None,
            saved_pending_expressions: std::mem::take(&mut self.pending_expressions),
            class_this_data: None,
            class_super_data: None,
            depth: 0,
        }));
        self.update_state();
        let members = self.class_element_visitor.visit_nodes(&member_list(node).unwrap());
        self.exit_class();
        self.transformer.factory().update_class_declaration(
            node,
            modifiers.clone(),
            node.name(),
            None,
            heritage_clauses.as_deref(),
            &members,
        )
    }

    pub fn visit_class_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_class_expression"); 
        if is_decorated_class_like(node) {
            let iife = self.transform_class_like(node);
            self.transformer.emit_context().set_original(&iife, node);
            return iife;
        }

        let modifiers = self.modifier_visitor.visit_modifiers(node.modifiers());
        let heritage_clauses = self.transformer.visitor().visit_nodes_opt(heritage_clauses(node));
        let top = self.top.take();
        self.top = Some(Box::new(LexicalEntry {
            kind: LexicalEntryKind::Class,
            next: top,
            class_info_data: None,
            saved_pending_expressions: std::mem::take(&mut self.pending_expressions),
            class_this_data: None,
            class_super_data: None,
            depth: 0,
        }));
        self.update_state();
        let members = self.class_element_visitor.visit_nodes(&member_list(node).unwrap());
        self.exit_class();
        self.transformer.factory().update_class_expression(
            node,
            modifiers.clone(),
            node.name(),
            None,
            heritage_clauses.as_deref(),
            &members,
        )
    }

    pub fn prepare_constructor(&mut self, ci: &mut ClassInfo) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("prepare_constructor"); 
        if ci.pending_instance_initializers.is_empty() {
            return Vec::new();
        }
        let f = self.transformer.factory();
        let statements = vec![f.new_expression_statement(
            &f.inline_expressions(ci.pending_instance_initializers.clone())
                .expect("expected inlined expression"),
        )];
        ci.pending_instance_initializers.clear();
        statements
    }

    pub fn transform_constructor_body_worker(
        &mut self,
        mut statements_out: Vec<Arc<Node>>,
        statements_in: &[Arc<Node>],
        statement_offset: usize,
        super_path: &[usize],
        super_path_depth: usize,
        initializer_statements: &[Arc<Node>],
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("transform_constructor_body_worker"); 
        let super_statement_index = super_path[super_path_depth];
        if super_statement_index > statement_offset {
            for s in &statements_in[statement_offset..super_statement_index] {
                statements_out.push(self.transformer.visitor().visit_node(s));
            }
        }

        let super_statement = &statements_in[super_statement_index];
        if is_try_statement(super_statement) {
            let (try_block_node, catch_clause_node, finally_block_node) = match &super_statement.data {
                NodeData::TryStatement(d) => {
                    (d.try_block.clone(), d.catch_clause.clone(), d.finally_block.clone())
                }
                _ => unreachable!(),
            };
            let try_block_statements = self.transform_constructor_body_worker(
                Vec::new(),
                &block_statements(&try_block_node),
                0,
                super_path,
                super_path_depth + 1,
                initializer_statements,
            );

            let new_try_statements = self.transformer.factory().new_node_list(try_block_statements);
            let mut new_try_block = self.transformer.factory().new_block(&new_try_statements, true);
            if let Some(b) = Arc::get_mut(&mut new_try_block) {
                b.loc = try_block_node.loc;
            }

            let catch_clause = catch_clause_node
                .as_ref()
                .map(|c| self.transformer.visitor().visit_node(c));
            let finally_block = finally_block_node
                .as_ref()
                .map(|b| self.transformer.visitor().visit_node(b));
            let updated = self.transformer.factory().update_try_statement_r39k13(
                super_statement,
                &new_try_block,
                catch_clause.as_ref(),
                finally_block.as_ref(),
            );
            statements_out.push(updated);
        } else {
            statements_out.push(self.transformer.visitor().visit_node(super_statement));
            statements_out.extend_from_slice(initializer_statements);
        }

        if super_statement_index + 1 < statements_in.len() {
            for s in &statements_in[super_statement_index + 1..] {
                statements_out.push(self.transformer.visitor().visit_node(s));
            }
        }
        statements_out
    }

    pub fn visit_constructor_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_constructor_declaration"); 
        self.enter_class_element(node);
        let modifiers = self.modifier_visitor.visit_modifiers(node.modifiers());
        let (parameters_list, ctor_body) = match &node.data {
            NodeData::ConstructorDeclaration(d) => (d.parameters.clone(), d.body.clone()),
            _ => unreachable!(),
        };
        let parameters = self.transformer.visitor().visit_nodes(&parameters_list);

        let mut body: Option<Arc<Node>> = None;
        if let Some(ctor_body) = ctor_body.as_ref() {
            if let Some(current_class_info) = self.class_info_stack.as_ref() {
                let mut ci = clone_class_info(current_class_info);
                let initializer_statements = self.prepare_constructor(&mut ci);
                if !initializer_statements.is_empty() {
                    self.store_class_info(ci);
                    let body_statements = block_statements(ctor_body);
                    let mut stmts: Vec<Arc<Node>> = Vec::new();
                    let (prologue, rest) = self
                        .transformer
                        .factory()
                        .split_standard_prologue(&body_statements);
                    stmts.extend_from_slice(&prologue);

                    let super_statement_indices = find_super_statement_index_path(&rest, 0);
                    if !super_statement_indices.is_empty() {
                        stmts = self.transform_constructor_body_worker(
                            stmts,
                            &rest,
                            0,
                            &super_statement_indices,
                            0,
                            &initializer_statements,
                        );
                    } else {
                        stmts.extend_from_slice(&initializer_statements);
                        let (visited, _) = self.transformer.visitor().visit_slice(&rest);
                        stmts.extend_from_slice(&visited);
                    }

                    let f = self.transformer.factory();
                    let ec = self.transformer.emit_context();
                    let new_stmts = f.new_node_list(stmts);
                    let mut new_body = f.new_block(&new_stmts, true);
                    ec.set_original(&new_body, ctor_body);
                    if let Some(b) = Arc::get_mut(&mut new_body) {
                        b.loc = ctor_body.loc;
                    }
                    body = Some(new_body);
                }
            }
        }

        if body.is_none() {
            body = Some(self.transformer.visitor().visit_node(ctor_body.as_ref().unwrap()));
        }
        self.exit_class_element();
        self.transformer.factory().update_constructor_declaration(
            node,
            modifiers.clone(),
            None,
            &parameters,
            None,
            None,
            Some(body.as_ref().unwrap()),
        )
    }
}

fn block_statements(block: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("block_statements"); 
    match &block.data {
        NodeData::Block(d) => d.statements.nodes.clone(),
        _ => vec![],
    }
}
