#![allow(unused_imports)]
#[path = "r39k14_defs.rs"]
pub mod r39k14_defs;

#[path = "r40k18_defs.rs"]
pub mod r40k18_defs;

use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeFlags, NodeList, SyntaxKind};
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::mig::m3c_2::subtree_facts;
use tsox_frontend::ast::{is_variable_declaration_list, is_variable_statement};
use tsox_frontend::ast::node_data_generated::VariableStatementData;
use crate::printer::{EmitContext, NodeFactory};
use crate::mig::m3m::TransformOptions;
use crate::mig::m4n_5::r36k29_defs::R36K29NodeExt;
use tsox_core::core::mig::m3j::first_result;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsingKind {
    None = 0,
    Sync = 1,
    Async = 2,
}

pub fn is_using_variable_declaration_list(node: &Arc<Node>) -> bool {
    is_variable_declaration_list(node)
        && get_using_kind_of_variable_declaration_list(node) != UsingKind::None
}

pub fn get_using_kind_of_variable_declaration_list(node: &Arc<Node>) -> UsingKind {
    let block_scoped = node.flags & NodeFlags::BlockScoped;
    if block_scoped == NodeFlags::AwaitUsing {
        UsingKind::Async
    } else if block_scoped == NodeFlags::Using {
        UsingKind::Sync
    } else {
        UsingKind::None
    }
}

pub fn get_using_kind_of_variable_statement(node: &Arc<Node>) -> UsingKind {
    get_using_kind_of_variable_declaration_list(&node.as_variable_statement().declaration_list)
}

pub fn get_using_kind(statement: &Arc<Node>) -> UsingKind {
    if is_variable_statement(statement) {
        return get_using_kind_of_variable_statement(statement);
    }
    UsingKind::None
}

pub struct UsingDeclarationTransformer {
    pub(crate) emit_context: EmitContext,
    pub(crate) export_bindings: Option<HashMap<String, Arc<Node>>>,
    pub(crate) export_binding_names: Vec<String>,
    pub(crate) export_vars: Vec<Arc<Node>>,
    pub(crate) default_export_binding: Option<Arc<Node>>,
    pub(crate) export_equals_binding: Option<Arc<Node>>,
}

pub fn new_using_declaration_transformer(opts: &TransformOptions) -> UsingDeclarationTransformer {
    UsingDeclarationTransformer {
        emit_context: opts.context.clone(),
        export_bindings: None,
        export_binding_names: Vec::new(),
        export_vars: Vec::new(),
        default_export_binding: None,
        export_equals_binding: None,
    }
}


pub fn get_using_kind_of_statements(statements: &[Arc<Node>]) -> UsingKind {
    let mut result = UsingKind::None;
    for statement in statements {
        let using_kind = get_using_kind(statement);
        if using_kind == UsingKind::Async {
            return UsingKind::Async;
        }
        if using_kind > result {
            result = using_kind;
        }
    }
    result
}

impl UsingDeclarationTransformer {
    pub(crate) fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    pub(crate) fn visit_slice(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>> {
        nodes.iter().map(|node| self.visit(node.clone())).collect()
    }

    pub(crate) fn visit_node(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        Some(self.visit(node))
    }
}
impl UsingDeclarationTransformer {
    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> {
        if !subtree_facts(&node).intersects(SubtreeFacts::CONTAINS_USING) {
            return node;
        }
        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::Block => self.visit_block(node),
            SyntaxKind::ForStatement => self.visit_for_statement(node),
            SyntaxKind::ForOfStatement => self.visit_for_of_statement(node),
            _ => self.visit_each_child(node),
        }
    }

}
impl UsingDeclarationTransformer {
    fn visit_source_file(&mut self, node: Arc<Node>) -> Arc<Node> {
        let sf = node.as_source_file_data();
        let using_kind = get_using_kind_of_statements(&sf.statements.nodes);
        let visited: Arc<Node> = if using_kind != UsingKind::None {
            self.emit_context.start_variable_environment();

            self.export_bindings = Some(HashMap::new());
            self.export_vars.clear();

            let (prologue, rest) = self.factory().split_standard_prologue(sf.statements.nodes.clone());

            let mut top_level_statements: Vec<Arc<Node>> =
                first_result(self.visit_slice(&prologue), &[]);

            let mut pos = 0;
            while pos < rest.len() {
                let statement = &rest[pos];
                if get_using_kind(statement) != UsingKind::None {
                    if pos > 0 {
                        top_level_statements
                            .extend(first_result(self.visit_slice(&rest[..pos]), &[]));
                    }
                    break;
                }
                pos += 1;
            }

            if pos >= rest.len() {
                panic!("Should have encountered at least one 'using' statement.");
            }

            let env_binding = self.create_env_binding();
            let body_statements =
                self.transform_using_declarations(&rest[pos..], &env_binding, Some(&mut top_level_statements));

            if let Some(export_bindings) = &self.export_bindings {
                if !export_bindings.is_empty() {
                    let export_specifiers: Vec<Arc<Node>> = self
                        .export_binding_names
                        .iter()
                        .map(|name| {
                            export_bindings[name].clone()
                        })
                        .collect();
                    let named_exports = self
                        .factory()
                        .new_named_exports(&self.factory().new_node_list(export_specifiers));
                    top_level_statements.push(self.factory().new_export_declaration(
                        None,
                        false,
                        &named_exports,
                        None,
                        None,
                    ));
                }
            }

            top_level_statements.extend(self.emit_context.end_variable_environment());
            if !self.export_vars.is_empty() {
                let declaration_list = self.factory().new_variable_declaration_list(
                    &self.factory().new_node_list(self.export_vars.clone()),
                    NodeFlags::Let,
                );
                top_level_statements.push(self.factory().new_variable_statement(
                    Some(self.factory().new_modifier_list(vec![self
                        .factory()
                        .new_modifier(SyntaxKind::ExportKeyword)])),
                    &declaration_list,
                ));
            }
            top_level_statements.extend(self.create_downlevel_using_statements(
                body_statements,
                &env_binding,
                using_kind == UsingKind::Async,
            ));

            if let Some(export_equals_binding) = self.export_equals_binding.clone() {
                top_level_statements.push(self.factory().new_export_assignment(
                    None,
                    true,
                    None,
                    &export_equals_binding,
                ));
            }

            self.factory()
                .update_source_file(&node, self.factory().new_node_list(top_level_statements))
        } else {
            self.visit_each_child(node)
        };
        let helpers = self.emit_context.read_emit_helpers();
        self.emit_context.add_emit_helper(&visited, &helpers);
        self.export_vars.clear();
        self.export_bindings = None;
        self.export_binding_names.clear();
        self.default_export_binding = None;
        self.export_equals_binding = None;
        visited
    }

}
impl UsingDeclarationTransformer {
    fn visit_block(&mut self, node: Arc<Node>) -> Arc<Node> {
        let data = node.as_block();
        let using_kind = get_using_kind_of_statements(&data.statements.nodes);
        if using_kind != UsingKind::None {
            let (prologue, rest) = self.factory().split_standard_prologue(data.statements.nodes.clone());
            let env_binding = self.create_env_binding();
            let mut statements: Vec<Arc<Node>> = first_result(self.visit_slice(&prologue), &[]);
            let transformed = self.transform_using_declarations(&rest, &env_binding, None);
            statements.extend(self.create_downlevel_using_statements(
                transformed,
                &env_binding,
                using_kind == UsingKind::Async,
            ));
            let mut statement_list = NodeList::new(statements);
            statement_list.loc = data.statements.loc;
            return self.factory().update_block(&node, &statement_list, data.multi_line);
        }
        self.visit_each_child(node)
    }

}
impl UsingDeclarationTransformer {
    fn visit_for_statement(&mut self, node: Arc<Node>) -> Arc<Node> {
        let data = node.as_for_statement();
        if let Some(initializer) = data.initializer.clone() {
            if is_using_variable_declaration_list(&initializer) {
                return self
                    .visit_node(
                        self.factory().new_block(
                            &self.factory().new_node_list(vec![
                                self.factory().new_variable_statement(None, &initializer),
                                self.factory().update_for_statement(
                                    &node,
                                    None,
                                    data.condition.as_ref(),
                                    data.incrementor.as_ref(),
                                    &data.statement,
                                ),
                            ]),
                            false,
                        ),
                    )
                    .unwrap();
            }
        }
        self.visit_each_child(node)
    }

}
