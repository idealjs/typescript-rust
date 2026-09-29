#![allow(unused_imports)]

use std::cell::RefCell;
use std::sync::Arc;

use crate::checker::checker::*;
use crate::checker::checker_checker_checker::Checker;
use tsox_frontend::ast::node_data_generated::{is_parenthesized_type_node, is_type_operator_node};
use tsox_frontend::ast::{Node, Symbol, SyntaxKind};

use crate::checker::mig::m2a::r19k11_defs::R19K11NodeExt;

thread_local! {
    static CONTEXTUAL_BINDING_PATTERNS: RefCell<Vec<Arc<Node>>> = const { RefCell::new(Vec::new()) };
    static PATTERN_FOR_TYPE: RefCell<Vec<(Arc<Type>, Arc<Node>)>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn contextual_binding_patterns_push(node: Arc<Node>) {
    CONTEXTUAL_BINDING_PATTERNS.with(|s| s.borrow_mut().push(node));
}

pub(crate) fn contextual_binding_patterns_pop() {
    CONTEXTUAL_BINDING_PATTERNS.with(|s| {
        s.borrow_mut().pop();
    });
}

pub(crate) fn pattern_for_type_insert(t: Arc<Type>, node: Arc<Node>) {
    PATTERN_FOR_TYPE.with(|s| {
        let mut s = s.borrow_mut();
        match s.iter_mut().find(|(k, _)| Arc::ptr_eq(k, &t)) {
            Some(slot) => slot.1 = node,
            None => s.push((t, node)),
        }
    });
}

pub(crate) fn pattern_for_type_get(t: &Arc<Type>) -> Option<Arc<Node>> {
    PATTERN_FOR_TYPE.with(|s| {
        s.borrow()
            .iter()
            .find(|(k, _)| Arc::ptr_eq(k, t))
            .map(|(_, n)| Arc::clone(n))
    })
}

impl Checker {
    pub(crate) fn is_function_object_type_fwd(&mut self, t: &Arc<Type>) -> bool {
        if let Some(sym) = self.get_global_type_by_name("Function")
            && let Some(target) = sym.target().cloned()
        {
            return self.has_base_type(t, &target) || Arc::ptr_eq(t, &sym);
        }
        false
    }

    pub(crate) fn get_alias_symbol_for_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let mut host = node.parent()?;
        loop {
            let next = if is_parenthesized_type_node(&host)
                || (is_type_operator_node(&host)
                    && matches!(
                        &host.data,
                        tsox_frontend::ast::NodeData::TypeOperatorNode(d)
                            if d.operator == SyntaxKind::ReadonlyKeyword
                    ))
            {
                host.parent()
            } else {
                None
            };
            match next {
                Some(h) => host = h,
                None => break,
            }
        }
        if host.kind == SyntaxKind::TypeAliasDeclaration {
            return self.get_symbol_of_declaration(&host);
        }
        None
    }
}
