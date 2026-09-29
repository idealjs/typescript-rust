#![allow(unused_imports)]

use crate::checker::checker_checker_checker::Checker;
use crate::checker::types::{Signature, Type};
use std::cell::RefCell;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol};

thread_local! {
    static CALL_STATE_CANDIDATES_FOR_ARGUMENT_ERROR: RefCell<Vec<Arc<Signature>>> =
        RefCell::new(Vec::new());
    static CALL_STATE_CANDIDATE_FOR_ARGUMENT_ARITY_ERROR: RefCell<Option<Arc<Signature>>> =
        RefCell::new(None);
    static CALL_STATE_CANDIDATE_FOR_TYPE_ARGUMENT_ERROR: RefCell<Option<Arc<Signature>>> =
        RefCell::new(None);
}

pub fn call_state_candidates_for_argument_error() -> Vec<Arc<Signature>> {
    CALL_STATE_CANDIDATES_FOR_ARGUMENT_ERROR.with(|c| c.borrow().clone())
}

pub fn set_call_state_candidates_for_argument_error(candidates: Vec<Arc<Signature>>) {
    CALL_STATE_CANDIDATES_FOR_ARGUMENT_ERROR.with(|c| *c.borrow_mut() = candidates);
}

pub fn call_state_candidate_for_argument_arity_error() -> Option<Arc<Signature>> {
    CALL_STATE_CANDIDATE_FOR_ARGUMENT_ARITY_ERROR.with(|c| c.borrow().clone())
}

pub fn set_call_state_candidate_for_argument_arity_error(candidate: Option<Arc<Signature>>) {
    CALL_STATE_CANDIDATE_FOR_ARGUMENT_ARITY_ERROR.with(|c| *c.borrow_mut() = candidate);
}

pub fn call_state_candidate_for_type_argument_error() -> Option<Arc<Signature>> {
    CALL_STATE_CANDIDATE_FOR_TYPE_ARGUMENT_ERROR.with(|c| c.borrow().clone())
}

pub fn set_call_state_candidate_for_type_argument_error(candidate: Option<Arc<Signature>>) {
    CALL_STATE_CANDIDATE_FOR_TYPE_ARGUMENT_ERROR.with(|c| *c.borrow_mut() = candidate);
}

pub fn get_base_types_if_unrelated_ext(
    c: &Checker,
    left_type: &Arc<Type>,
    right_type: &Arc<Type>,
    is_related: &mut dyn FnMut(&Checker, &Arc<Type>, &Arc<Type>) -> bool,
) -> (Arc<Type>, Arc<Type>) {
    let left_base = c.get_base_type_of_literal_type(left_type);
    let right_base = c.get_base_type_of_literal_type(right_type);
    if !is_related(c, &left_base, &right_base) {
        (left_base, right_base)
    } else {
        (Arc::clone(left_type), Arc::clone(right_type))
    }
}
