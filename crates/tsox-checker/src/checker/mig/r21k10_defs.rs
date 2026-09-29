#![allow(unused_imports)]

use crate::checker::checker_checker_checker::Checker;
use crate::checker::relater_relation::{ChainRelated, RelaterChainEntry, RelationComparisonResult, RelationKind};
use crate::checker::types_cached_type_kind::CacheHashKey;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::{Node, SyntaxKind};

pub(crate) struct RelaterErrorState {
    pub error_chain: Vec<RelaterChainEntry>,
}

impl Checker {
    pub(crate) fn get_error_state(&self) -> RelaterErrorState {
        RelaterErrorState {
            error_chain: self.relater_error_chain.clone(),
        }
    }

    pub(crate) fn restore_error_state(&mut self, e: RelaterErrorState) {
        self.relater_error_chain = e.error_chain;
    }

    pub(crate) fn clear_error_chain(&mut self) {
        self.relater_error_chain = Vec::new();
    }

    pub(crate) fn get_chain_message(&self, index: usize) -> Option<Message> {
        self.relater_error_chain
            .iter()
            .rev()
            .nth(index)
            .map(|e| e.message)
    }

    pub(crate) fn chain_args_match(&self, args: &[String]) -> bool {
        match self.relater_error_chain.last() {
            Some(entry) => {
                for (i, a) in args.iter().enumerate() {
                    if entry.args.get(i) != Some(a) {
                        return false;
                    }
                }
                true
            }
            None => false,
        }
    }
}

thread_local! {
    static MAYBE_KEYS: RefCell<Vec<CacheHashKey>> = const { RefCell::new(Vec::new()) };
    static MAYBE_KEYS_SET: RefCell<HashSet<CacheHashKey>> = RefCell::new(HashSet::new());
    static MAYBE_RESULTS: RefCell<HashMap<CacheHashKey, RelationComparisonResult>> =
        RefCell::new(HashMap::new());
    static SKIP_DIRECT_INFERENCE_NODES: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());
    static INFERENCE_PARTIALLY_BLOCKED: RefCell<bool> = const { RefCell::new(false) };
    static PENDING_RELATED: RefCell<Vec<ChainRelated>> = RefCell::new(Vec::new());
    static CURRENT_RELATION: RefCell<RelationKind> = RefCell::new(RelationKind::Assignable);
}

pub(crate) fn pending_related_push(related: ChainRelated) {
    PENDING_RELATED.with(|p| p.borrow_mut().push(related));
}

pub(crate) fn pending_related_take_all() -> Vec<ChainRelated> {
    PENDING_RELATED.with(|p| std::mem::take(&mut *p.borrow_mut()))
}

pub(crate) fn current_relation_kind() -> RelationKind {
    CURRENT_RELATION.with(|r| *r.borrow())
}

pub(crate) fn set_current_relation_kind(kind: RelationKind) {
    CURRENT_RELATION.with(|r| {
        *r.borrow_mut() = kind;
    });
}

impl Checker {
    pub(crate) fn global_string_type(&mut self) -> Arc<crate::checker::types::Type> {
        if let Some(t) = self.global_string_type.get() {
            return Arc::clone(t);
        }
        let t = self.get_global_type("String", 0, true);
        let _ = self.global_string_type.set(Arc::clone(&t));
        t
    }

    pub(crate) fn global_number_type(&mut self) -> Arc<crate::checker::types::Type> {
        if let Some(t) = self.global_number_type.get() {
            return Arc::clone(t);
        }
        let t = self.get_global_type("Number", 0, true);
        let _ = self.global_number_type.set(Arc::clone(&t));
        t
    }

    pub(crate) fn global_boolean_type(&mut self) -> Arc<crate::checker::types::Type> {
        if let Some(t) = self.global_boolean_type.get() {
            return Arc::clone(t);
        }
        let t = self.get_global_type("Boolean", 0, true);
        let _ = self.global_boolean_type.set(Arc::clone(&t));
        t
    }

    pub(crate) fn get_global_es_symbol_type(&mut self) -> Arc<crate::checker::types::Type> {
        if let Some(t) = self.es_symbol_type.get() {
            return Arc::clone(t);
        }
        let t = self.get_global_type("ESSymbol", 0, true);
        let _ = self.es_symbol_type.set(Arc::clone(&t));
        t
    }
}

pub(crate) fn maybe_keys_len() -> usize {
    MAYBE_KEYS.with(|k| k.borrow().len())
}

pub(crate) fn maybe_keys_get(index: usize) -> Option<CacheHashKey> {
    MAYBE_KEYS.with(|k| k.borrow().get(index).copied())
}

pub(crate) fn maybe_keys_truncate(maybe_start: usize) {
    MAYBE_KEYS.with(|k| k.borrow_mut().truncate(maybe_start));
}

pub(crate) fn maybe_keys_set_remove(key: &CacheHashKey) {
    MAYBE_KEYS_SET.with(|s| {
        s.borrow_mut().remove(key);
    });
}

pub(crate) fn maybe_result_mark_succeeded(key: CacheHashKey, extra: RelationComparisonResult) {
    MAYBE_RESULTS.with(|m| {
        m.borrow_mut().insert(key, RelationComparisonResult::Succeeded | extra);
    });
}

pub(crate) fn maybe_keys_set_clear() {
    MAYBE_KEYS_SET.with(|s| s.borrow_mut().clear());
}

pub(crate) fn skip_direct_inference_nodes_insert(key: usize) {
    SKIP_DIRECT_INFERENCE_NODES.with(|s| {
        s.borrow_mut().insert(key);
    });
}

pub(crate) fn skip_direct_inference_nodes_clear() {
    SKIP_DIRECT_INFERENCE_NODES.with(|s| s.borrow_mut().clear());
}

pub(crate) fn skip_direct_inference_nodes_has(key: usize) -> bool {
    SKIP_DIRECT_INFERENCE_NODES.with(|s| s.borrow().contains(&key))
}

pub(crate) fn inference_partially_blocked_get() -> bool {
    INFERENCE_PARTIALLY_BLOCKED.with(|b| *b.borrow())
}

pub(crate) fn inference_partially_blocked_set(value: bool) {
    INFERENCE_PARTIALLY_BLOCKED.with(|b| {
        *b.borrow_mut() = value;
    });
}

pub fn is_call_like_expression(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::JsxOpeningElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxOpeningFragment
            | SyntaxKind::CallExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::Decorator
    )
}

pub fn is_call_like_or_function_like_expression(node: &Arc<Node>) -> bool {
    is_call_like_expression(node)
        || tsox_frontend::ast::is_function_expression_or_arrow_function(node)
}

pub fn get_class_like_declaration_of_symbol(symbol: &tsox_frontend::ast::Symbol) -> Option<Arc<Node>> {
    symbol
        .declarations
        .iter()
        .find(|d| {
            matches!(
                d.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            )
        })
        .cloned()
}

pub fn find_constructor_declaration(class_decl: &Arc<Node>) -> Option<Arc<Node>> {
    let members = match &class_decl.data {
        tsox_frontend::ast::NodeData::ClassDeclaration(d) => &d.members,
        tsox_frontend::ast::NodeData::ClassExpression(d) => &d.members,
        _ => return None,
    };
    members.nodes.iter().find(|m| match &m.data {
        tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => d.body.is_some(),
        _ => false,
    }).cloned()
}
