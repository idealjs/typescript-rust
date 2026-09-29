#![allow(unused_imports, ambiguous_glob_reexports)]

use std::sync::Arc;

use crate::checker::*;
use crate::checker::checker::*;
use crate::checker::checker_checker_checker::Checker;
use crate::checker::nodecopy_builder::{NodeBuilderImpl, NodeFactoryStub};
use crate::checker::types::{SymbolTable, Type, TypeData};
use tsox_frontend::ast::{Diagnostic, Node, Symbol, SymbolFlags};

pub struct NameResolver {
    pub compiler_options: Arc<CompilerOptions>,
    pub get_symbol_of_declaration: fn(&Checker, &Arc<Node>) -> Option<Arc<Symbol>>,
    pub error: fn(&mut Checker, &Arc<Node>, &'static str, &[String]) -> Option<Arc<Diagnostic>>,
    pub globals: SymbolTable,
    pub arguments_symbol: Option<Arc<Symbol>>,
    pub require_symbol: Option<Arc<Symbol>>,
    pub lookup: fn(&mut Checker, &SymbolTable, &str, SymbolFlags) -> Option<Arc<Symbol>>,
    pub symbol_referenced: fn(&mut Checker, &Arc<Symbol>, SymbolFlags),
    pub set_requires_scope_change_cache: fn(&mut Checker, &Arc<Node>, Tristate),
    pub get_requires_scope_change_cache: fn(&Checker, &Node) -> Tristate,
    pub on_property_with_invalid_initializer: Option<
        fn(&mut Checker, Option<&Arc<Node>>, &str, &Arc<Node>, Option<&Arc<Symbol>>) -> bool,
    >,
    pub on_failed_to_resolve_symbol:
        Option<fn(&mut Checker, &Arc<Node>, &str, SymbolFlags, &'static tsox_core::diagnostics::Message)>,
    pub on_successfully_resolved_symbol: Option<
        fn(
            &mut Checker,
            &Arc<Node>,
            &Arc<Symbol>,
            SymbolFlags,
            Option<&Arc<Node>>,
            Option<&Arc<Node>>,
            bool,
        ),
    >,
}

impl NameResolver {
    pub fn resolve(
        &mut self,
        _location: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
        _name_not_found_message: Option<&'static str>,
        _use_outer_name: bool,
        _is_found_as_alias: bool,
    ) -> Option<Arc<Symbol>> {
        let symbol = self.globals.get(name)?.clone();
        if !symbol.flags.intersects(meaning) {
            return None;
        }
        Some(symbol)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PseudoTypeKind {
    Undefined,
    Null,
    Any,
    String,
    Number,
    BigInt,
    Boolean,
    False,
    True,
    SingleCallSignature,
    Tuple,
    ObjectLiteral,
    StringLiteral,
    NumericLiteral,
    BigIntLiteral,
    Direct,
    Inferred,
    MaybeConstLocation,
    NoResult,
}

pub struct PseudoType {
    pub kind: PseudoTypeKind,
    pub const_type: Option<Arc<PseudoType>>,
    pub regular_type: Option<Arc<PseudoType>>,
}

impl PseudoType {
    pub fn as_pseudo_type_maybe_const_location(&self) -> &Self {
        self
    }
}

pub struct MappedType {
    pub ty: Arc<Type>,
    pub target: Option<Arc<Type>>,
}

impl MappedType {
    pub fn as_type(&self) -> &Type {
        &self.ty
    }
}

impl<'a> NodeBuilderImpl<'a> {
    pub fn mapped_type_target(&self, mapped: &MappedType) -> Option<Arc<Type>> {
        mapped.target.clone()
    }
}

pub struct NodeVisitorHooks;

impl Default for NodeVisitorHooks {
    fn default() -> Self {
        NodeVisitorHooks
    }
}

pub type NodeVisitHook = Box<dyn Fn(&Arc<Node>) -> Option<Arc<Node>>>;

pub struct NodeVisitor {
    pub visit: NodeVisitHook,
    pub hooks: NodeVisitorHooks,
}

pub fn new_node_visitor(
    visit: NodeVisitHook,
    _factory: &NodeFactoryStub,
    hooks: NodeVisitorHooks,
) -> NodeVisitor {
    NodeVisitor { visit, hooks }
}

pub fn clone_binding_name_visitor_hook(_b: &NodeBuilderImpl) -> NodeVisitHook {
    Box::new(|node| Some(Arc::clone(node)))
}

pub(crate) fn set_literal_fresh_type(regular_type: &Arc<Type>, fresh_type: &Arc<Type>) {
    if let Some(t) = Arc::get_mut(&mut Arc::clone(regular_type)) {
        if let TypeData::Literal(d) = &mut t.data {
            let _ = d.fresh_type.set(Arc::clone(fresh_type));
        }
    }
    if let Some(t) = Arc::get_mut(&mut Arc::clone(fresh_type)) {
        if let TypeData::Literal(d) = &mut t.data {
            let _ = d.fresh_type.set(Arc::clone(fresh_type));
        }
    }
}
