#![allow(unused_imports)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::Node;

use crate::checker::checker_checker_checker::Checker;

pub enum TypeReferenceSerializationKind {
    Unknown,
    TypeWithConstructSignatureAndValue,
    VoidNullableOrNeverType,
    BigIntLikeType,
    BooleanType,
    NumberLikeType,
    StringLikeType,
    ArrayLikeType,
    ESSymbolType,
    Promise,
    TypeWithCallSignature,
    ObjectType,
}

#[derive(Default, Clone)]
pub struct JsxLinks {
    pub import_ref: Option<Arc<Node>>,
}

thread_local! {
    static JSX_LINKS: RefCell<HashMap<u64, JsxLinks>> = RefCell::new(HashMap::new());
}

pub fn jsx_links(node: &Arc<Node>) -> JsxLinks {
    JSX_LINKS.with(|l| {
        l.borrow()
            .get(&node.id())
            .cloned()
            .unwrap_or_default()
    })
}

pub fn set_jsx_links_import_ref(node: &Arc<Node>, import_ref: Option<Arc<Node>>) {
    JSX_LINKS.with(|l| {
        l.borrow_mut()
            .entry(node.id())
            .or_default()
            .import_ref = import_ref;
    });
}

thread_local! {
    static BUILDER_CHECKER: Cell<*mut Checker> = const { Cell::new(std::ptr::null_mut()) };
}

pub fn set_builder_checker(ch: &Checker) {
    BUILDER_CHECKER.with(|c| c.set(ch as *const Checker as *mut Checker));
}

pub fn builder_checker_ptr() -> *mut Checker {
    BUILDER_CHECKER.with(|c| c.get())
}
