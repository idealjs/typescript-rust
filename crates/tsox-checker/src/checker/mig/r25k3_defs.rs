#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types_impl_chunk_2::ConditionalRoot;
use crate::checker::types::Type;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::evaluator::EvalResult;
use tsox_frontend::ast::Node;

thread_local! {
    static CONDITIONAL_INSTANTIATIONS: RefCell<HashMap<usize, HashMap<u64, Arc<Type>>>> =
        RefCell::new(HashMap::new());
}

pub(crate) fn conditional_type_instantiations_get(root_key: usize, key: u64) -> Option<Arc<Type>> {
    CONDITIONAL_INSTANTIATIONS.with(|cache| {
        cache
            .borrow()
            .get(&root_key)
            .and_then(|entries| entries.get(&key))
            .cloned()
    })
}

pub(crate) fn conditional_type_instantiations_insert(root_key: usize, key: u64, t: Arc<Type>) {
    CONDITIONAL_INSTANTIATIONS.with(|cache| {
        cache
            .borrow_mut()
            .entry(root_key)
            .or_default()
            .insert(key, t);
    });
}

pub(crate) fn type_resolution_has_property_shared(checker: &Checker, r: &TypeResolution) -> bool {
    match r.property_name {
        TypeSystemPropertyName::Type => r
            .target
            .as_symbol()
            .and_then(|s| checker.value_symbol_links.get(s.as_ref()))
            .map(|l| l.resolved_type.is_some())
            .unwrap_or(false),
        TypeSystemPropertyName::DeclaredType => r
            .target
            .as_symbol()
            .and_then(|s| checker.type_alias_links.get(s.as_ref()))
            .map(|l| l.declared_type.is_some())
            .unwrap_or(false),
        TypeSystemPropertyName::ResolvedBaseTypes => r
            .target
            .as_type()
            .is_some_and(|t| t.as_interface_type().is_some_and(|i| i.base_types_resolved)),
        TypeSystemPropertyName::ResolvedBaseConstructorType => r
            .target
            .as_type()
            .is_some_and(|t| {
                t.as_interface_type()
                    .is_some_and(|i| i.resolved_base_constructor_type.get().is_some())
            }),
        TypeSystemPropertyName::ResolvedReturnType => r
            .target
            .as_signature()
            .is_some_and(|s| s.resolved_return_type.get().is_some()),
        TypeSystemPropertyName::ResolvedBaseConstraint => r
            .target
            .as_type()
            .is_some_and(|t| {
                t.as_constrained_type()
                    .is_some_and(|c| c.resolved_base_constraint.get().is_some())
            }),
        TypeSystemPropertyName::WriteType => r
            .target
            .as_symbol()
            .and_then(|s| checker.value_symbol_links.get(s.as_ref()))
            .map(|l| l.write_type.is_some())
            .unwrap_or(false),
        TypeSystemPropertyName::AliasTarget => r
            .target
            .as_symbol()
            .and_then(|s| checker.alias_symbol_links.get(s.as_ref()))
            .map(|l| l.alias_target.is_some())
            .unwrap_or(false),
        _ => panic!("Unhandled case in typeResolutionHasProperty"),
    }
}

impl Checker {
    pub fn evaluate(&mut self, expr: &Arc<Node>, location: Option<&Arc<Node>>) -> EvalResult {
        let mut entity_fn =
            |expr: &Arc<Node>, loc: Option<&Arc<Node>>| self.evaluate_entity(expr, loc);
        tsox_frontend::evaluator::evaluate_expression(expr, location, &mut entity_fn)
    }
}
