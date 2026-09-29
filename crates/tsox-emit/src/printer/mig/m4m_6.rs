#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::node_flags::NodeFlags;

use crate::printer::EmitContext;

use super::m4m_3::{
    EmitFlags, ENVIRONMENT_FLAGS_IN_PARAMETERS, ENVIRONMENT_FLAGS_VARIABLES_HOISTED_IN_PARAMETERS,
    VarScope,
};

fn clone_node_list(list: &NodeList) -> NodeList {
    let mut cloned = NodeList::new(list.nodes.clone());
    cloned.loc = list.loc;
    cloned
}

impl EmitContext {
    pub fn end_variable_environment(&mut self) -> Vec<Arc<Node>> {
        let scope = self.var_scope_stack.pop().unwrap_or_default();
        let mut statements: Vec<Arc<Node>> = vec![];
        if !scope.functions.is_empty() {
            statements = scope.functions.clone();
        }
        if !scope.variables.is_empty() {
            let node_list = self.factory().new_node_list(scope.variables.clone());
            let decl_list =
                self.factory()
                    .new_variable_declaration_list(&node_list, NodeFlags::empty());
            let var_statement = self.factory().new_variable_statement(None, &decl_list);
            self.set_emit_flags(&var_statement, EmitFlags::CUSTOM_PROLOGUE);
            statements.push(var_statement);
        }
        if !scope.initialization_statements.is_empty() {
            statements.extend(scope.initialization_statements.iter().cloned());
        }
        let mut lexical = self.end_lexical_environment();
        statements.append(&mut lexical);
        statements
    }

    pub fn end_and_merge_variable_environment_list(
        &mut self,
        statements: Option<&NodeList>,
    ) -> Option<NodeList> {
        let nodes: Vec<Arc<Node>> = match statements {
            Some(list) => list.nodes.clone(),
            None => vec![],
        };
        let declarations = self.end_variable_environment();
        let (result, changed) = self.merge_environment_inner(&nodes, &declarations);
        if changed {
            let mut list = self.factory().new_node_list(result);
            if let Some(l) = Arc::get_mut(&mut list) {
                l.loc = statements.unwrap().loc;
            }
            return Some(clone_node_list(&list));
        }
        statements.map(clone_node_list)
    }

    pub fn add_variable_declaration(&mut self, name: &Arc<Node>) {
        let var_decl = self
            .factory()
            .new_variable_declaration(name, None, None, None);
        self.set_emit_flags(&var_decl, EmitFlags::NO_NESTED_SOURCE_MAPS);
        let scope = self.var_scope_stack.last_mut().unwrap();
        scope.variables.push(var_decl);
        if scope.flags & ENVIRONMENT_FLAGS_IN_PARAMETERS != 0 {
            scope.flags |= ENVIRONMENT_FLAGS_VARIABLES_HOISTED_IN_PARAMETERS;
        }
    }

    pub fn add_hoisted_function_declaration(&mut self, node: &Arc<Node>) {
        self.set_emit_flags(node, EmitFlags::CUSTOM_PROLOGUE);
        self.var_scope_stack
            .last_mut()
            .unwrap()
            .functions
            .push(node.clone());
    }

    pub fn end_lexical_environment(&mut self) -> Vec<Arc<Node>> {
        let scope = self.let_scope_stack.pop().unwrap_or_default();
        let mut statements: Vec<Arc<Node>> = vec![];
        if !scope.variables.is_empty() {
            let node_list = self.factory().new_node_list(scope.variables.clone());
            let decl_list =
                self.factory()
                    .new_variable_declaration_list(&node_list, NodeFlags::Let);
            let var_statement = self.factory().new_variable_statement(None, &decl_list);
            self.set_emit_flags(&var_statement, EmitFlags::CUSTOM_PROLOGUE);
            statements.push(var_statement);
        }
        statements
    }

    pub fn end_and_merge_lexical_environment_list(
        &mut self,
        statements: Option<&NodeList>,
    ) -> Option<NodeList> {
        let nodes: Vec<Arc<Node>> = match statements {
            Some(list) => list.nodes.clone(),
            None => vec![],
        };
        let declarations = self.end_lexical_environment();
        let (result, changed) = self.merge_environment_inner(&nodes, &declarations);
        if changed {
            let mut list = self.factory().new_node_list(result);
            if let Some(l) = Arc::get_mut(&mut list) {
                l.loc = statements.unwrap().loc;
            }
            return Some(clone_node_list(&list));
        }
        statements.map(clone_node_list)
    }

    pub fn add_lexical_declaration(&mut self, name: &Arc<Node>) {
        let var_decl = self
            .factory()
            .new_variable_declaration(name, None, None, None);
        self.set_emit_flags(&var_decl, EmitFlags::NO_NESTED_SOURCE_MAPS);
        self.let_scope_stack
            .last_mut()
            .unwrap()
            .variables
            .push(var_decl);
    }
}
