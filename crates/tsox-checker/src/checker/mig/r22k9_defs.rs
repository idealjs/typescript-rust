#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::r21k9_defs::NodeFactoryExt21;
use crate::checker::mig::r27k_defs::{new_pseudo_type_direct, PseudoType, PseudoTypeKind};
use crate::checker::checker::*;
use crate::checker::nodecopy_builder::{NodeBuilderImpl, NodeFactoryStub, PseudoCheckerStub};
use crate::checker::symboltracker::{NodeBuilderContext, TrackedSymbolArgs};
use crate::checker::types::OBJECT_FLAGS_REQUIRES_WIDENING;
use crate::checker::types_impl_chunk::TypeMapper;
use tsox_frontend::ast::{Node, NodeData, NodeFlags, Symbol, SymbolFlags, SyntaxKind};

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

pub fn reset_nodebuilder_side_state() {
    SIDE_STATE.with(|m| m.borrow_mut().clear());
    NODE_SERIALIZED_TYPES.with(|m| m.borrow_mut().clear());
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
    fn get_return_type_of_signature(&self, declaration: &Arc<Node>) -> Option<Arc<PseudoType>>;
}

fn pseudo_function_like_return_type_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        tsox_frontend::ast::NodeData::MethodDeclaration(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::FunctionDeclaration(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::MethodSignatureDeclaration(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::FunctionExpression(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::ArrowFunction(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::CallSignatureDeclaration(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::ConstructSignatureDeclaration(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::IndexSignatureDeclaration(d) => Some(Arc::clone(&d.type_node)),
        tsox_frontend::ast::NodeData::FunctionTypeNode(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::ConstructorTypeNode(d) => d.type_node.clone(),
        tsox_frontend::ast::NodeData::JSDocSignature(d) => d.type_node.clone(),
        _ => None,
    }
}

fn pseudo_is_value_signature_declaration(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::Constructor
    )
}

fn pseudo_for_each_return_statement(
    body: &Arc<Node>,
    visitor: &mut impl FnMut(&Arc<Node>) -> bool,
) -> bool {
    match body.kind {
        SyntaxKind::ReturnStatement => visitor(body),
        SyntaxKind::CaseBlock
        | SyntaxKind::Block
        | SyntaxKind::IfStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::WithStatement
        | SyntaxKind::SwitchStatement
        | SyntaxKind::CaseClause
        | SyntaxKind::DefaultClause
        | SyntaxKind::LabeledStatement
        | SyntaxKind::TryStatement
        | SyntaxKind::CatchClause => {
            let mut stopped = false;
            tsox_frontend::ast::for_each_child(body, |child| {
                if !pseudo_for_each_return_statement(child, &mut *visitor) {
                    stopped = true;
                    return true;
                }
                false
            });
            !stopped
        }
        _ => true,
    }
}

fn pseudo_return_expression(stmt: &Arc<Node>) -> Option<Arc<Node>> {
    match &stmt.data {
        tsox_frontend::ast::NodeData::ReturnStatement(d) => d.expression.clone(),
        _ => None,
    }
}

fn pseudo_direct_from_assertion(candidate: &Arc<Node>) -> Option<Arc<PseudoType>> {
    let type_node = match &candidate.data {
        tsox_frontend::ast::NodeData::AsExpression(d) => &d.type_node,
        tsox_frontend::ast::NodeData::TypeAssertion(d) => &d.type_node,
        _ => return None,
    };
    if crate::checker::utilities_get_assignment_target::is_const_type_reference(type_node) {
        return None;
    }
    Some(new_pseudo_type_direct(Arc::clone(type_node)))
}

fn pseudo_type_from_single_return_expression(fn_node: &Arc<Node>) -> Option<Arc<PseudoType>> {
    let body = match &fn_node.data {
        tsox_frontend::ast::NodeData::MethodDeclaration(d) => d.body.clone()?,
        tsox_frontend::ast::NodeData::FunctionDeclaration(d) => d.body.clone()?,
        tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => d.body.clone()?,
        tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => d.body.clone()?,
        tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => d.body.clone()?,
        tsox_frontend::ast::NodeData::FunctionExpression(d) => d.body.clone(),
        tsox_frontend::ast::NodeData::ArrowFunction(d) => d.body.clone(),
        _ => return None,
    };
    if tsox_frontend::ast::node_is_missing(Some(&body)) {
        return None;
    }
    let is_async = fn_node
        .has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Async);
    let is_generator = match &fn_node.data {
        tsox_frontend::ast::NodeData::MethodDeclaration(d) => d.asterisk_token.is_some(),
        tsox_frontend::ast::NodeData::FunctionDeclaration(d) => d.asterisk_token.is_some(),
        tsox_frontend::ast::NodeData::FunctionExpression(d) => d.asterisk_token.is_some(),
        _ => false,
    };
    if is_async && is_generator {
        return None;
    }
    let mut candidate: Option<Arc<Node>> = None;
    pseudo_for_each_return_statement(&body, &mut |stmt| {
        let in_body = stmt.parent().is_some_and(|p| Arc::ptr_eq(&p, &body));
        if !in_body {
            candidate = None;
            return false;
        }
        if candidate.is_some() {
            candidate = None;
            return false;
        }
        candidate = pseudo_return_expression(stmt);
        true
    });
    let candidate = candidate?;
    pseudo_direct_from_assertion(&candidate)
}

impl PseudoCheckerExt22 for PseudoCheckerStub {
    fn get_return_type_of_signature(&self, declaration: &Arc<Node>) -> Option<Arc<PseudoType>> { ::tsox_core::fntrace::enter("get_return_type_of_signature");
        match declaration.kind {
            SyntaxKind::MethodDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::MethodSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::SetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::JSDocSignature => {}
            _ => return None,
        }
        if let Some(r) = pseudo_function_like_return_type_node(declaration) {
            return Some(new_pseudo_type_direct(r));
        }
        if !pseudo_is_value_signature_declaration(declaration) {
            return None;
        }
        pseudo_type_from_single_return_expression(declaration)
    }
}

pub trait NodeBuilderPseudoExt22 {
    fn pseudo_type_equivalent_to_type(
        &mut self,
        pseudo: Option<&Arc<PseudoType>>,
        t: &Arc<Type>,
        assume_pseudo_valid: bool,
        report_fallback: bool,
    ) -> bool;
    fn pseudo_return_type_matches_predicate(
        &mut self,
        pseudo: Option<&Arc<PseudoType>>,
        predicate: &TypePredicate,
    ) -> bool;
    fn pseudo_type_to_node_with_checker_fallback(
        &mut self,
        pseudo: Option<&Arc<PseudoType>>,
        fallback: &Arc<Type>,
    ) -> Arc<Node>;
    fn serialize_type_for_declaration(
        &mut self,
        declaration: Option<&Arc<Node>>,
        t: &Arc<Type>,
        symbol: Option<&Arc<Symbol>>,
        try_reuse: bool,
    ) -> Arc<Node>;
    fn type_to_type_node_ex(&mut self, t: &Arc<Type>) -> Arc<Node>;
    fn deep_clone_node(&mut self, node: &Arc<Node>) -> Arc<Node>;
}

impl<'a> NodeBuilderPseudoExt22 for NodeBuilderImpl<'a> {
    fn pseudo_type_equivalent_to_type(
        &mut self,
        pseudo: Option<&Arc<PseudoType>>,
        t: &Arc<Type>,
        _assume_pseudo_valid: bool,
        _report_fallback: bool,
    ) -> bool { ::tsox_core::fntrace::enter("pseudo_type_equivalent_to_type");
        let ch = unsafe {
            &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr()
        };
        if ch.is_error_type(t) {
            return true;
        }
        let Some(p) = pseudo else {
            return false;
        };
        if p.kind != PseudoTypeKind::Direct {
            return false;
        }
        let Some(node) = &p.direct_node else {
            return false;
        };
        let from_node = ch.get_type_from_type_node(node);
        Arc::ptr_eq(&from_node, t)
    }

    fn pseudo_return_type_matches_predicate(
        &mut self,
        pseudo: Option<&Arc<PseudoType>>,
        predicate: &TypePredicate,
    ) -> bool { ::tsox_core::fntrace::enter("pseudo_return_type_matches_predicate");
        let Some(p) = pseudo else {
            return false;
        };
        if p.kind != PseudoTypeKind::Direct {
            return false;
        }
        let Some(node) = &p.direct_node else {
            return false;
        };
        if node.kind != SyntaxKind::TypePredicate {
            return false;
        }
        let tsox_frontend::ast::NodeData::TypePredicateNode(tp) = &node.data else {
            return false;
        };
        let is_asserts = tp.asserts_modifier.is_some();
        let predicate_is_asserts = matches!(
            predicate.kind,
            crate::checker::types_impl_chunk_3::TypePredicateKind::AssertsThis
                | crate::checker::types_impl_chunk_3::TypePredicateKind::AssertsIdentifier
        );
        if is_asserts != predicate_is_asserts {
            return false;
        }
        let is_this = tp.parameter_name.kind == SyntaxKind::ThisKeyword;
        let predicate_is_this = matches!(
            predicate.kind,
            crate::checker::types_impl_chunk_3::TypePredicateKind::This
                | crate::checker::types_impl_chunk_3::TypePredicateKind::AssertsThis
        );
        if is_this != predicate_is_this {
            return false;
        }
        if !is_this && tp.parameter_name.text() != predicate.parameter_name {
            return false;
        }
        match (&predicate.t, &tp.type_node) {
            (Some(pt), Some(tn)) => {
                let ch = unsafe {
                    &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr()
                };
                let from_node = ch.get_type_from_type_node(tn);
                Arc::ptr_eq(&from_node, pt)
            }
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }

    fn pseudo_type_to_node_with_checker_fallback(
        &mut self,
        pseudo: Option<&Arc<PseudoType>>,
        fallback: &Arc<Type>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("pseudo_type_to_node_with_checker_fallback");
        if let Some(p) = pseudo {
            if p.kind == PseudoTypeKind::Direct {
                if let Some(node) = &p.direct_node {
                    if self.can_reuse_existing_js_type_node(node, Some(fallback)) {
                        let bound = self.create_recovery_boundary();
                        let mut v = crate::checker::nodecopy_property_name::ExistingNodeTreeVisitor::new(self, &bound);
                        let transformed = v.visit_node(node);
                        if self.finalize_boundary(&bound) {
                            if let Some(t) = transformed {
                                return t;
                            }
                        }
                    }
                }
            }
        }
        self.type_to_type_node_ex(fallback)
    }

    fn serialize_type_for_declaration(
        &mut self,
        declaration: Option<&Arc<Node>>,
        t: &Arc<Type>,
        _symbol: Option<&Arc<Symbol>>,
        try_reuse: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("serialize_type_for_declaration");
        if let Some(node) = crate::checker::mig::m2f_3::r25k8_defs::serialize_type_for_declaration_with_pseudo_reuse(
            self, declaration, t, try_reuse,
        ) {
            return node;
        }
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

pub trait CheckerPseudoReturnDisplayExt {
    fn pseudo_signature_return_display_text(
        &mut self,
        sig: &Arc<Signature>,
        return_type: &Arc<Type>,
        flags: crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags,
    ) -> Option<String>;
}

impl CheckerPseudoReturnDisplayExt for Checker {
    fn pseudo_signature_return_display_text(
        &mut self,
        sig: &Arc<Signature>,
        return_type: &Arc<Type>,
        flags: crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("pseudo_signature_return_display_text");
        if self.type_render_enclosing.is_none() {
            return None;
        }
        let declaration = sig.declaration.as_ref()?;
        if tsox_frontend::ast::node_is_synthesized(declaration) {
            return None;
        }
        if pseudo_function_like_return_type_node(declaration).is_some() {
            return None;
        }
        let pt = (PseudoCheckerStub).get_return_type_of_signature(declaration)?;
        if pt.kind != PseudoTypeKind::Direct {
            return None;
        }
        let node = pt.direct_node.as_ref()?;
        if !self.is_error_type(return_type) {
            let from_node = self.get_type_from_type_node(node);
            if !Arc::ptr_eq(&from_node, return_type) {
                return None;
            }
        }
        // Go serializeReturnTypeForSignature 复用门（nodebuilderimpl.go:2329+
        // nodecopy visitEntityName，d05b0aa8365 描述的「不可及引用一律回退原
        // 结构渲染」）：注解子树内实体名对 enclosingDeclaration 不可及——如
        // index.ts 引用 createApi.ts 非导出别名 Id——整个复用拒绝，回落结构
        // 渲染（成员形）；同文件可及（createApi.ts）重放保留
        let enclosing = self.type_render_enclosing.clone();
        if self.annotation_has_inaccessible_reference(node, &enclosing) {
            return None;
        }
        if return_type.object_flags.contains(ObjectFlags::Reference)
            && node.kind == SyntaxKind::TypeReference
        {
            return None;
        }
        self.print_annotation_type_node(node, flags)
    }
}
