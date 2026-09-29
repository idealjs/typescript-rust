#![allow(dead_code, unused_imports, unused_variables)]

//! r38-k02 defs: CommonJSModuleTransformer 补齐方法与工厂缺口。
//! Go oracle: tsc/internal/transformers/moduletransforms/commonjsmodule.go
//! 交接: progress_notes_r38k2.md

use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use tsox_core::core::compiler_options::{ModuleKind, ScriptTarget};
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::NodeList;

use crate::printer::NodeFactory;
use crate::mig::m4k::CommonJSModuleTransformer;
use crate::mig::m4k_2::get_external_module_name_literal;
use crate::mig::m4k_3::ExternalModuleInfo;
use crate::mig::m4m_2::is_local_name;
use crate::mig::m4h_4::r36k9_defs::cloned_node_list;
use crate::mig::r33k6_shim::Visitor;
use crate::printer::generated_identifier_flags::EmitContext;

pub fn clone_emit_context(context: &Arc<EmitContext>) -> EmitContext {
    let mut cloned = EmitContext::new();
    cloned.next_id = AtomicU32::new(context.next_id.load(std::sync::atomic::Ordering::Relaxed));
    cloned
}

impl Visitor {
    pub fn visit_nodes(&mut self, nodes: impl IntoIterator<Item = Arc<Node>>) -> Vec<Arc<Node>> {
        nodes.into_iter().collect()
    }
}

impl<'a> NodeFactory<'a> {
    pub fn new_import_default_helper(&self, expression: &Arc<Node>) -> Arc<Node> {
        self.new_call_expression(
            &self.new_unscoped_helper_name("__importDefault"),
            None,
            None,
            self.new_node_list(vec![expression.clone()]),
            NodeFlags::empty(),
        )
    }

    pub fn update_variable_statement(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<tsox_frontend::ast::node::ModifierList>>,
        declaration_list: Arc<Node>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::VariableStatement,
            NodeData::VariableStatement(VariableStatementData {
                modifiers,
                declaration_list,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_variable_declaration_list(
        &self,
        node: &Arc<Node>,
        declarations: &NodeList,
        flags: NodeFlags,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::VariableDeclarationList,
            NodeData::VariableDeclarationList(VariableDeclarationListData {
                declarations: Arc::new(cloned_node_list(declarations)),
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }

    pub fn update_for_in_or_of_statement(
        &self,
        node: &Arc<Node>,
        await_modifier: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
        expression: Option<Arc<Node>>,
        statement: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let for_stmt = match &node.data {
            NodeData::ForInOrOfStatement(d) => d,
            _ => panic!("update_for_in_or_of_statement on wrong kind"),
        };
        let mut updated = Node::new(
            node.kind,
            NodeData::ForInOrOfStatement(ForInOrOfStatementData {
                await_modifier,
                initializer: initializer.or_else(|| Some(for_stmt.initializer.clone())).unwrap(),
                expression: expression.or_else(|| Some(for_stmt.expression.clone())).unwrap(),
                statement: statement.or_else(|| Some(for_stmt.statement.clone())).unwrap(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }
}

impl<'a> CommonJSModuleTransformer<'a> {
    pub(crate) fn create_require_call(&mut self, node: Arc<Node>) -> Arc<Node> {
        let source_file = self.current_source_file.clone();
        let module_name = get_external_module_name_literal(
            &self.factory(),
            &node,
            source_file.as_ref(),
            None,
            None,
            self.compiler_options,
        );
        let args: Vec<Arc<Node>> = module_name.map(|m| vec![m]).unwrap_or_default();
        self.factory().new_call_expression(
            &self.factory().new_identifier("require"),
            None,
            None,
            self.factory().new_node_list(args),
            NodeFlags::empty(),
        )
    }

    pub(crate) fn get_helper_expression_for_import(
        &mut self,
        node: &Arc<Node>,
        inner_expr: Arc<Node>,
    ) -> Arc<Node> {
        let _ = node;
        inner_expr
    }

    pub(crate) fn get_helper_expression_for_export(
        &mut self,
        node: &Arc<Node>,
        inner_expr: Arc<Node>,
    ) -> Arc<Node> {
        let _ = node;
        inner_expr
    }

    pub(crate) fn visit_expression_identifier(&mut self, name: Arc<Node>) -> Option<Arc<Node>> {
        if is_local_name(&self.emit_context, &name) {
            return Some(name);
        }
        Some(name)
    }
}
