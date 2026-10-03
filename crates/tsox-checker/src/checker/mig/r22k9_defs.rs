#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::r21k9_defs::NodeFactoryExt21;
use crate::checker::checker::*;
use crate::checker::nodecopy_builder::{NodeBuilderImpl, NodeFactoryStub, PseudoCheckerStub};
use crate::checker::symboltracker::{NodeBuilderContext, TrackedSymbolArgs};
use crate::checker::types_impl_chunk::TypeMapper;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct CompositeSymbolIdentity {
    pub is_constructor_object: bool,
    pub symbol_id: usize,
    pub node_id: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CompositeTypeCacheIdentity {
    pub type_id: u32,
    pub flags_bits: u32,
    pub internal_flags_bits: u32,
}

#[derive(Clone)]
pub struct SerializedTypeEntry {
    pub node: Arc<Node>,
    pub truncating: bool,
    pub added_length: usize,
    pub tracked_symbols: Vec<TrackedSymbolArgs>,
}

#[derive(Default)]
pub struct VisitedTypeSet22(pub HashSet<u32>);

impl VisitedTypeSet22 {
    pub fn has(&self, id: u32) -> bool { ::tsox_core::fntrace::enter("has"); 
        self.0.contains(&id)
    }
    pub fn add(&mut self, id: u32) { ::tsox_core::fntrace::enter("add"); 
        self.0.insert(id);
    }
    pub fn delete(&mut self, id: u32) { ::tsox_core::fntrace::enter("delete"); 
        self.0.remove(&id);
    }
}

#[derive(Default)]
pub struct ScopeStack22 {
    pub stack: Vec<u8>,
}

impl ScopeStack22 {
    pub fn enter_scope(&mut self) -> Box<dyn FnOnce(&mut NodeBuilderContext)> { ::tsox_core::fntrace::enter("enter_scope"); 
        self.stack.push(1);
        let depth = self.stack.len();
        Box::new(move |ctx: &mut NodeBuilderContext| {
            side_with(ctx, |s| s.type_parameter_names.stack.truncate(depth - 1));
        })
    }
}

#[derive(Default)]
pub struct NodeBuilderCtxSideState22 {
    pub mapper: Option<Arc<TypeMapper>>,
    pub enclosing_symbol_types: HashMap<usize, Arc<Type>>,
    pub visited_types: VisitedTypeSet22,
    pub symbol_depth: HashMap<CompositeSymbolIdentity, usize>,
    pub type_parameter_names: ScopeStack22,
    pub type_parameter_names_by_text: ScopeStack22,
    pub type_parameter_names_by_text_next_name_count: ScopeStack22,
    pub type_parameter_symbol_list: ScopeStack22,
}

thread_local! {
    static SIDE_STATE: RefCell<HashMap<usize, NodeBuilderCtxSideState22>> =
        RefCell::new(HashMap::new());
    static NODE_SERIALIZED_TYPES: RefCell<HashMap<usize, HashMap<CompositeTypeCacheIdentity, SerializedTypeEntry>>> =
        RefCell::new(HashMap::new());
}

pub fn ctx_side_key(ctx: &NodeBuilderContext) -> usize { ::tsox_core::fntrace::enter("ctx_side_key"); 
    ctx as *const NodeBuilderContext as usize
}

pub fn side_with<R>(
    ctx: &NodeBuilderContext,
    f: impl FnOnce(&mut NodeBuilderCtxSideState22) -> R,
) -> R { ::tsox_core::fntrace::enter("side_with"); 
    SIDE_STATE.with(|m| f(m.borrow_mut().entry(ctx_side_key(ctx)).or_default()))
}

pub fn links_has(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("links_has"); 
    NODE_SERIALIZED_TYPES.with(|m| m.borrow().contains_key(&(Arc::as_ptr(node) as usize)))
}

pub fn links_with<R>(
    node: &Arc<Node>,
    f: impl FnOnce(&mut HashMap<CompositeTypeCacheIdentity, SerializedTypeEntry>) -> R,
) -> R { ::tsox_core::fntrace::enter("links_with"); 
    NODE_SERIALIZED_TYPES.with(|m| f(m.borrow_mut().entry(Arc::as_ptr(node) as usize).or_default()))
}

pub trait NodeBuilderCtxTrackExt22 {
    fn track_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool;
}

impl NodeBuilderCtxTrackExt22 for NodeBuilderContext {
    fn track_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("track_symbol"); 
        match self.tracker.as_mut() {
            Some(tracker) => tracker.track_symbol(symbol, enclosing_declaration, meaning),
            None => false,
        }
    }
}

pub trait CheckerNodeBuilderExt22 {
    fn get_regular_type_of_expression(&self, expr: &Arc<Node>) -> Arc<Type>;
    fn is_type_any(&self, t: &Arc<Type>) -> bool;
}

pub trait PseudoCheckerExt22 {
    fn get_return_type_of_signature(&self, declaration: &Arc<Node>) -> Option<Arc<Type>>;
}

impl PseudoCheckerExt22 for PseudoCheckerStub {
    fn get_return_type_of_signature(&self, _declaration: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_return_type_of_signature"); 
        None
    }
}

pub trait NodeBuilderPseudoExt22 {
    fn pseudo_type_equivalent_to_type(
        &mut self,
        pseudo: Option<&Arc<Type>>,
        t: &Arc<Type>,
        assume_pseudo_valid: bool,
        report_fallback: bool,
    ) -> bool;
    fn pseudo_return_type_matches_predicate(
        &mut self,
        pseudo: Option<&Arc<Type>>,
        predicate: &TypePredicate,
    ) -> bool;
    fn pseudo_type_to_node_with_checker_fallback(
        &mut self,
        pseudo: Option<&Arc<Type>>,
        fallback: &Arc<Type>,
    ) -> Arc<Node>;
    fn serialize_type_for_declaration(
        &mut self,
        declaration: Option<&Arc<Node>>,
        t: &Arc<Type>,
        symbol: Option<&Arc<Symbol>>,
        preserve_modifier_flags: bool,
    ) -> Arc<Node>;
    fn type_to_type_node_ex(&mut self, t: &Arc<Type>) -> Arc<Node>;
    fn deep_clone_node(&mut self, node: &Arc<Node>) -> Arc<Node>;
}

impl<'a> NodeBuilderPseudoExt22 for NodeBuilderImpl<'a> {
    fn pseudo_type_equivalent_to_type(
        &mut self,
        pseudo: Option<&Arc<Type>>,
        t: &Arc<Type>,
        _assume_pseudo_valid: bool,
        _report_fallback: bool,
    ) -> bool { ::tsox_core::fntrace::enter("pseudo_type_equivalent_to_type"); 
        matches!(pseudo, Some(p) if Arc::ptr_eq(p, t))
    }

    fn pseudo_return_type_matches_predicate(
        &mut self,
        pseudo: Option<&Arc<Type>>,
        _predicate: &TypePredicate,
    ) -> bool { ::tsox_core::fntrace::enter("pseudo_return_type_matches_predicate"); 
        pseudo.is_some()
    }

    fn pseudo_type_to_node_with_checker_fallback(
        &mut self,
        pseudo: Option<&Arc<Type>>,
        fallback: &Arc<Type>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("pseudo_type_to_node_with_checker_fallback"); 
        match pseudo {
            Some(p) => self.type_to_type_node_ex(p),
            None => self.type_to_type_node_ex(fallback),
        }
    }

    fn serialize_type_for_declaration(
        &mut self,
        _declaration: Option<&Arc<Node>>,
        t: &Arc<Type>,
        _symbol: Option<&Arc<Symbol>>,
        _preserve_modifier_flags: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("serialize_type_for_declaration"); 
        self.type_to_type_node_ex(t)
    }

    fn type_to_type_node_ex(&mut self, t: &Arc<Type>) -> Arc<Node> { ::tsox_core::fntrace::enter("type_to_type_node_ex"); 
        match self.type_to_type_node(t) {
            Some(n) => n,
            None => NodeFactoryStub.new_keyword_type_node(SyntaxKind::UnknownKeyword),
        }
    }

    fn deep_clone_node(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("deep_clone_node"); 
        Arc::clone(node)
    }
}
