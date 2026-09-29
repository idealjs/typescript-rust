#![allow(unused_imports)]
#![allow(dead_code)]

use crate::checker::checker::*;
use crate::checker::types_type_id::TYPE_FLAGS_NULLABLE;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tsox_frontend::ast::{Diagnostic, NodeData};
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::SyntaxKind;

thread_local! {
    static CONTEXT_FREE_TYPES: RefCell<HashMap<usize, Arc<Type>>> = RefCell::new(HashMap::new());
}

pub(crate) fn context_free_types_get(node: &Arc<Node>) -> Option<Arc<Type>> {
    CONTEXT_FREE_TYPES.with(|m| m.borrow().get(&(Arc::as_ptr(node) as *const () as usize)).cloned())
}

pub(crate) fn context_free_types_insert(node: &Arc<Node>, t: &Arc<Type>) {
    CONTEXT_FREE_TYPES.with(|m| {
        m.borrow_mut()
            .insert(Arc::as_ptr(node) as *const () as usize, Arc::clone(t))
    });
}

thread_local! {
    static CONTEXTUAL_BINDING_PATTERNS: RefCell<Vec<Arc<Node>>> = RefCell::new(Vec::new());
}

pub(crate) fn contextual_binding_patterns_contains_declaration(decl: &Arc<Node>) -> bool {
    CONTEXTUAL_BINDING_PATTERNS.with(|v| {
        v.borrow()
            .iter()
            .any(|p| p.parent().map(|dp| Arc::ptr_eq(&dp, decl)).unwrap_or(false))
    })
}

impl Checker {
    pub(crate) fn non_inferrable_any_type(&mut self) -> Arc<Type> {
        let mut t = self.new_object_type(ObjectFlags::Anonymous, None);
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            t_mut.object_flags |= ObjectFlags::NonInferrableType;
        }
        t
    }

    pub(crate) fn resolving_signature(&self) -> Arc<Signature> {
        Arc::clone(self.resolving_signature.get_or_init(|| {
            let mut sig = Signature::new();
            sig.flags = SignatureFlags::empty();
            sig.min_argument_count = 0;
            sig.resolved_min_argument_count = -1;
            sig.resolved_return_type = OnceLock::from(self.any_type());
            Arc::new(sig)
        }))
    }

    pub(crate) fn create_promise_return_type(&mut self, _node: &Arc<Node>, promised: &Arc<Type>) -> Arc<Type> {
        self.create_promise_return_type_for(promised)
    }

    pub(crate) fn get_global_import_type_checked(&mut self, name: &str) -> Arc<Type> {
        match self.get_global_symbol(name, SymbolFlags::TYPE, None) {
            Some(symbol) => self.get_type_of_symbol(&symbol),
            None => self.empty_object_type(),
        }
    }

    pub(crate) fn get_global_import_meta_type(&mut self) -> Arc<Type> {
        self.get_global_import_type_checked("ImportMeta")
    }

    pub(crate) fn get_global_import_call_options_type_checked(&mut self) -> Arc<Type> {
        self.get_global_import_type_checked("ImportCallOptions")
    }

    pub(crate) fn get_global_import_attributes_type_checked(&mut self) -> Arc<Type> {
        self.get_global_import_type_checked("ImportAttributes")
    }

    pub(crate) fn get_flow_type_of_reference_ex(
        &mut self,
        _reference: &Arc<Node>,
        declared_type: &Arc<Type>,
        narrowable_type: Option<&Arc<Type>>,
        _container: Option<&Arc<Node>>,
    ) -> Arc<Type> {
        match narrowable_type {
            Some(t) => Arc::clone(t),
            None => Arc::clone(declared_type),
        }
    }

    pub(crate) fn check_await_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let t = self.check_expression_ex(&expression, CheckMode::Normal);
        self.get_awaited_type(&t).unwrap_or_else(|| Arc::clone(&t))
    }

    pub(crate) fn check_spread_expression(&mut self, node: &Arc<Node>, _check_mode: CheckMode) -> Arc<Type> {
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let t = self.check_expression_ex(&expression, CheckMode::Normal);
        self.get_non_nullable_type_if_needed(&t)
    }

    pub(crate) fn check_return_expression(
        &mut self,
        _node: &Arc<Node>,
        return_type: &Arc<Type>,
        expression: &Arc<Node>,
        _body: &Arc<Node>,
        expression_type: &Arc<Type>,
        _is_async: bool,
    ) -> Arc<Type> {
        self.check_type_assignable_to_and_optionally_elaborate(
            expression_type,
            return_type,
            Some(expression),
            None,
            None,
            None,
        );
        Arc::clone(expression_type)
    }

    pub(crate) fn check_decorators(&mut self, _node: &Arc<Node>) {}

    pub(crate) fn check_if_statement(&mut self, node: &Arc<Node>) {
        if self.check_grammar_statement_in_ambient_context(node) {
            return;
        }
        let (expression, then_statement, else_statement) = match &node.data {
            NodeData::IfStatement(data) => (
                data.expression.clone(),
                data.then_statement.clone(),
                data.else_statement.clone(),
            ),
            _ => return,
        };
        let _expr_type = self.check_truthiness_expression(&expression, CheckMode::Normal);
        self.check_source_element(&then_statement);
        if let Some(else_statement) = else_statement {
            self.check_source_element(&else_statement);
        }
    }

    pub(crate) fn add_related_info(&mut self, _node: &Arc<Node>, _related: Diagnostic) {}

    pub(crate) fn add_related_info_to_diagnostic(&mut self, _diagnostic: &Diagnostic, _related: Diagnostic) {}

    pub(crate) fn check_type_parameters(&mut self, type_parameters: Option<&NodeList>) {
        if let Some(type_parameters) = type_parameters {
            for type_parameter in type_parameters.iter() {
                self.check_node_deferred(&type_parameter);
            }
        }
    }

    pub(crate) fn check_object_type_for_duplicate_declarations(&mut self, _node: &Arc<Node>, _is_interface: bool) {}
}
