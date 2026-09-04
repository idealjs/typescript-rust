#![allow(dead_code)]
#![allow(unused_variables)]

//! AST node cloning for declaration emit.
//!
//! Ported 1:1 from `internal/checker/nodecopy.go` (900 lines). These
//! functions clone/reuse AST nodes when building type nodes for declaration
//! emit, handling symbol tracking, recovery boundaries, and module specifier
//! rewriting.
//!
//! The Go implementation operates on `NodeBuilderImpl`, which wraps a
//! `NodeFactory`, an `EmitContext`, and a `NodeBuilderContext`. The Rust
//! port defines stub structs for the unported infrastructure (`NodeFactory`,
//! `EmitContext`, `NodeVisitor`) and stubs methods that depend on them.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol, SymbolFlags, SyntaxKind};

use super::checker::Checker;
use super::symboltracker::{
    NodeBuilderContext, SharedNodeBuilderContext, SymbolTracker, TrackedSymbolArgs,
};
use super::types::Type;

// ────────────────────────────────────────────────────────────────────────────
// NodeBuilderLinks / NodeBuilderSymbolLinks
// ────────────────────────────────────────────────────────────────────────────

/// Per-node links for the node builder.
///
/// Mirrors Go's `NodeBuilderLinks` (nodebuilderimpl.go).
#[derive(Default)]
pub struct NodeBuilderLinks {
    /// Collection of types serialized at this location.
    // TODO: serialized_types map
    /// If present, this is a fake scope injected into an enclosing
    /// declaration chain.
    pub fake_scope_for_signature_declaration: Option<String>,
}

/// Per-symbol links for the node builder.
///
/// Mirrors Go's `NodeBuilderSymbolLinks` (nodebuilderimpl.go).
#[derive(Default)]
pub struct NodeBuilderSymbolLinks {
    // TODO: specifier_cache: module.ModeAwareCache<String>
}

// ────────────────────────────────────────────────────────────────────────────
// NodeBuilderImpl
// ────────────────────────────────────────────────────────────────────────────

/// The node builder implementation.
///
/// Mirrors Go's `NodeBuilderImpl` (nodebuilderimpl.go). Builds `TypeNode`s
/// from `Type`s for declaration emit and hover/quick-info. The Go struct
/// embeds a `NodeFactory`, an `EmitContext`, and a `PseudoChecker`; these
/// are stubbed here.
pub struct NodeBuilderImpl<'a> {
    /// Node factory. TODO: Port `ast.NodeFactory`.
    pub f: NodeFactoryStub,
    /// The checker.
    pub ch: &'a Checker,
    /// Emit context. TODO: Port `printer.EmitContext`.
    pub e: EmitContextStub,
    /// Pseudo checker. TODO: Port `pseudochecker.PseudoChecker`.
    pub pc: PseudoCheckerStub,

    /// Per-node builder links.
    // TODO: links: LinkStore<Node, NodeBuilderLinks>,
    /// Per-symbol builder links.
    // TODO: symbol_links: LinkStore<Symbol, NodeBuilderSymbolLinks>,

    /// Current builder context (shared, mutable).
    pub ctx: SharedNodeBuilderContext,

    /// Reusable visitor for binding-name cloning.
    // TODO: clone_binding_name_visitor: NodeVisitor,

    /// Symbols for synthesized identifiers (e.g. inlay hints).
    pub id_to_symbol: HashMap<u64, Arc<Symbol>>,
}

/// Stub for Go's `ast.NodeFactory`.
/// TODO: Port the full NodeFactory.
#[derive(Default)]
pub struct NodeFactoryStub;

/// Stub for Go's `printer.EmitContext`.
/// TODO: Port the full EmitContext.
#[derive(Default)]
pub struct EmitContextStub;

/// Stub for Go's `pseudochecker.PseudoChecker`.
/// TODO: Port the full PseudoChecker.
#[derive(Default)]
pub struct PseudoCheckerStub;

impl EmitContextStub {
    /// Set the original node of `node` to `original`.
    /// TODO: Port `printer.EmitContext.SetOriginal`.
    pub fn set_original(&self, node: &Arc<Node>, original: Option<&Arc<Node>>) {
        // TODO
    }

    /// Add emit flags to a node.
    /// TODO: Port `printer.EmitContext.AddEmitFlags`.
    pub fn add_emit_flags(&self, node: &Arc<Node>, flags: u32) {
        // TODO
    }

    /// Get the "most original" version of a node.
    /// TODO: Port `printer.EmitContext.MostOriginal`.
    pub fn most_original(&self, node: &Arc<Node>) -> Arc<Node> {
        Arc::clone(node)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// RecoveryBoundary
// ────────────────────────────────────────────────────────────────────────────

/// A recovery boundary for error handling during node reuse.
///
/// Mirrors Go's `recoveryBoundary` (nodecopy.go). When an error occurs
/// during node reuse, the boundary captures the state so it can be restored
/// and the reused subtree discarded.
pub struct RecoveryBoundary {
    pub ctx: SharedNodeBuilderContext,
    pub had_error: bool,
    pub deferred_reports: Vec<Box<dyn FnOnce()>>,
    pub old_tracker: Option<Box<dyn SymbolTracker>>,
    pub old_tracked_symbols: Vec<TrackedSymbolArgs>,
    pub tracked_symbols: Vec<TrackedSymbolArgs>,
    pub old_encountered_error: bool,
    pub old_approximate_length: usize,
}

impl RecoveryBoundary {
    /// Mark an error in this boundary, optionally deferring a report.
    ///
    /// Mirrors Go's `recoveryBoundary.markError`.
    pub fn mark_error(&mut self, report: Box<dyn FnOnce()>) {
        self.had_error = true;
        self.deferred_reports.push(report);
    }

    /// Begin a recovery scope, returning the state needed to restore it.
    ///
    /// Mirrors Go's `recoveryBoundary.startRecoveryScope`.
    pub fn start_recovery_scope(&self) -> OriginalRecoveryScopeState {
        let tracked_symbols_top = self.ctx.borrow().tracked_symbols.len();
        let unreported_errors_top = self.deferred_reports.len();
        OriginalRecoveryScopeState {
            tracked_symbols_top,
            unreported_errors_top,
            had_error: self.had_error,
        }
    }

    /// End a recovery scope, restoring the state.
    ///
    /// Mirrors Go's `recoveryBoundary.endRecoveryScope`.
    pub fn end_recovery_scope(&mut self, state: OriginalRecoveryScopeState) {
        self.had_error = state.had_error;
        let mut ctx = self.ctx.borrow_mut();
        ctx.tracked_symbols.truncate(state.tracked_symbols_top);
        drop(ctx);
        self.deferred_reports.truncate(state.unreported_errors_top);
    }
}

/// State captured at the start of a recovery scope.
///
/// Mirrors Go's `originalRecoveryScopeState` (nodecopy.go).
pub struct OriginalRecoveryScopeState {
    pub tracked_symbols_top: usize,
    pub unreported_errors_top: usize,
    pub had_error: bool,
}

// ────────────────────────────────────────────────────────────────────────────
// WrappingTracker
// ────────────────────────────────────────────────────────────────────────────

/// A `SymbolTracker` that wraps another tracker and defers error reports
/// through a `RecoveryBoundary`.
///
/// Mirrors Go's `wrappingTracker` (nodecopy.go).
pub struct WrappingTracker {
    pub wrapped: Rc<RefCell<Box<dyn SymbolTracker>>>,
    pub bound: Rc<RefCell<RecoveryBoundary>>,
}

impl WrappingTracker {
    /// Create a new wrapping tracker.
    ///
    /// Mirrors Go's `newWrappingTracker`.
    pub fn new(inner: Box<dyn SymbolTracker>, bound: Rc<RefCell<RecoveryBoundary>>) -> Self {
        WrappingTracker {
            wrapped: Rc::new(RefCell::new(inner)),
            bound,
        }
    }
}

impl SymbolTracker for WrappingTracker {
    fn track_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool {
        self.bound
            .borrow_mut()
            .tracked_symbols
            .push(TrackedSymbolArgs {
                symbol: Arc::clone(symbol),
                enclosing_declaration: enclosing_declaration.cloned(),
                meaning,
            });
        false
    }

    fn report_inaccessible_this_error(&mut self) {
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped.borrow_mut().report_inaccessible_this_error();
        }));
    }

    fn report_private_in_base_of_class_expression(&mut self, property_name: &str) {
        let pn = property_name.to_string();
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped
                .borrow_mut()
                .report_private_in_base_of_class_expression(&pn);
        }));
    }

    fn report_inaccessible_unique_symbol_error(&mut self) {
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped
                .borrow_mut()
                .report_inaccessible_unique_symbol_error();
        }));
    }

    fn report_cyclic_structure_error(&mut self) {
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped.borrow_mut().report_cyclic_structure_error();
        }));
    }

    fn report_likely_unsafe_import_required_error(&mut self, specifier: &str, symbol_name: &str) {
        let sp = specifier.to_string();
        let sn = symbol_name.to_string();
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped
                .borrow_mut()
                .report_likely_unsafe_import_required_error(&sp, &sn);
        }));
    }

    fn report_truncation_error(&mut self) {
        // Should this also be deferred?
        self.wrapped.borrow_mut().report_truncation_error();
    }

    fn report_nonlocal_augmentation(
        &mut self,
        containing_file: Option<&Arc<SourceFile>>,
        parent_symbol: &Arc<Symbol>,
        augmenting_symbol: &Arc<Symbol>,
    ) {
        // Should this also be deferred?
        self.wrapped.borrow_mut().report_nonlocal_augmentation(
            containing_file,
            parent_symbol,
            augmenting_symbol,
        );
    }

    fn report_non_serializable_property(&mut self, property_name: &str) {
        let pn = property_name.to_string();
        let wrapped = Rc::clone(&self.wrapped);
        self.bound.borrow_mut().mark_error(Box::new(move || {
            wrapped.borrow_mut().report_non_serializable_property(&pn);
        }));
    }

    fn report_inference_fallback(&mut self, node: &Arc<Node>) {
        // Should this also be deferred?
        self.wrapped.borrow_mut().report_inference_fallback(node);
    }

    fn push_error_fallback_node(&mut self, node: &Arc<Node>) {
        self.wrapped.borrow_mut().push_error_fallback_node(node);
    }

    fn pop_error_fallback_node(&mut self) {
        self.wrapped.borrow_mut().pop_error_fallback_node();
    }
}

// ────────────────────────────────────────────────────────────────────────────
// propertyNameNodeKind
// ────────────────────────────────────────────────────────────────────────────

/// The kind of a property name node.
///
/// Mirrors Go's `propertyNameNodeKind` (nodebuilderimpl.go).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyNameNodeKind {
    Identifier,
    NumericLiteral,
    StringLiteral,
}

/// Classify a property name as identifier, numeric literal, or string literal.
///
/// Mirrors Go's `classifyPropertyName` (nodebuilderimpl.go).
pub fn classify_property_name(
    name: &str,
    string_named: bool,
    is_method: bool,
) -> PropertyNameNodeKind {
    if is_method && name == "new" {
        return PropertyNameNodeKind::StringLiteral;
    }
    // TODO: Port scanner.IsIdentifierText
    if is_identifier_text(name) {
        return PropertyNameNodeKind::Identifier;
    }
    if !string_named && is_numeric_literal_name(name) {
        PropertyNameNodeKind::NumericLiteral
    } else {
        PropertyNameNodeKind::StringLiteral
    }
}

/// TODO: Port from Go's `scanner.IsIdentifierText`.
fn is_identifier_text(text: &str) -> bool {
    // Simplified: a valid identifier starts with a letter, _, or $,
    // and contains only letters, digits, _, or $.
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// TODO: Port from Go's `isNumericLiteralName`.
fn is_numeric_literal_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_digit() || c == '.')
}

// ────────────────────────────────────────────────────────────────────────────
// NodeBuilderImpl methods
// ────────────────────────────────────────────────────────────────────────────

impl<'a> NodeBuilderImpl<'a> {
    /// Create a new node builder.
    ///
    /// Mirrors Go's `newNodeBuilderImpl`.
    pub fn new(ch: &'a Checker, id_to_symbol: HashMap<u64, Arc<Symbol>>) -> Self {
        let ctx = Rc::new(RefCell::new(NodeBuilderContext::default()));
        NodeBuilderImpl {
            f: NodeFactoryStub,
            ch,
            e: EmitContextStub,
            pc: PseudoCheckerStub,
            ctx,
            id_to_symbol,
        }
    }

    // ── Node reuse methods ──────────────────────────────────────────────

    /// Reuse an existing node if possible.
    ///
    /// Mirrors Go's `NodeBuilderImpl.reuseNode`.
    pub fn reuse_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        let node = node?;
        self.try_reuse_existing_node_helper(node)
    }

    /// Try to convert a JSDoc type node to a regular type node.
    ///
    /// Mirrors Go's `NodeBuilderImpl.tryJSTypeNodeToTypeNode`.
    pub fn try_js_type_node_to_type_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        self.reuse_node(node)
    }

    /// Reuse a property name node, reclassifying if necessary.
    ///
    /// Mirrors Go's `NodeBuilderImpl.reuseName`.
    pub fn reuse_name(&mut self, node: Option<&Arc<Node>>, is_method: bool) -> Option<Arc<Node>> {
        let res = self.reuse_node(node)?;
        // TODO: Port ast.TryGetTextOfPropertyName, ast.IsStringLiteral,
        // ast.IsIdentifier, factory.NewStringLiteral, etc.
        Some(res)
    }

    /// Reuse a type node, probing for expandability and falling back to
    /// type serialization if reuse fails.
    ///
    /// Mirrors Go's `NodeBuilderImpl.reuseTypeNode`.
    pub fn reuse_type_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        let node = node?;
        let r = self.reuse_node(Some(node));
        if let Some(ref r) = r {
            // After successful reuse during hover, probe the reused AST for
            // expandable type references so canIncreaseExpansionDepth is set
            // even though typeToTypeNode (and shouldExpandType) were never
            // called.
            let ctx = self.ctx.borrow();
            if ctx.max_expansion_depth >= 0 && !ctx.can_increase_expansion_depth {
                drop(ctx);
                self.walk_node_for_expandability(node);
            }
            return r.clone().into();
        }
        // TODO: Port ctx.tracker.ReportInferenceFallback(node),
        // b.getTypeFromTypeNode(node, false), b.typeToTypeNode(t)
        None
    }

    /// Walk a reused AST node tree, calling checkTypeExpandability on each
    /// type reference, type predicate, or import type node.
    ///
    /// Mirrors Go's `NodeBuilderImpl.walkNodeForExpandability`.
    pub fn walk_node_for_expandability(&mut self, node: &Arc<Node>) {
        let can_increase = self.ctx.borrow().can_increase_expansion_depth;
        if can_increase {
            return;
        }
        // TODO: Port full implementation including:
        // - ast.IsTypeReferenceNode, ast.IsExpressionWithTypeArguments,
        //   ast.IsTypePredicateNode, ast.IsImportTypeNode
        // - b.getTypeFromTypeNode(node, false)
        // - b.checkTypeExpandability(t)
        // - node.ForEachChild(...)
    }

    // ── Recovery boundary management ────────────────────────────────────

    /// Create a recovery boundary.
    ///
    /// Mirrors Go's `NodeBuilderImpl.createRecoveryBoundary`.
    pub fn create_recovery_boundary(&mut self) -> Rc<RefCell<RecoveryBoundary>> {
        let ctx = self.ctx.borrow();
        let bound = RecoveryBoundary {
            ctx: Rc::clone(&self.ctx),
            had_error: false,
            deferred_reports: Vec::new(),
            old_tracker: None, // TODO: Save/restore old tracker
            old_tracked_symbols: ctx.tracked_symbols.clone(),
            tracked_symbols: Vec::new(),
            old_encountered_error: ctx.encountered_error,
            old_approximate_length: ctx.approximate_length,
        };
        drop(ctx);

        Rc::new(RefCell::new(bound))
    }

    /// Finalize a recovery boundary, restoring state and replaying deferred
    /// reports if no error occurred.
    ///
    /// Mirrors Go's `NodeBuilderImpl.finalizeBoundary`.
    pub fn finalize_boundary(&mut self, bound: &Rc<RefCell<RecoveryBoundary>>) -> bool {
        let mut ctx = self.ctx.borrow_mut();
        // TODO: Restore old_tracker, old_tracked_symbols, etc.
        ctx.encountered_error = bound.borrow().old_encountered_error;
        ctx.approximate_length = bound.borrow().old_approximate_length;
        drop(ctx);

        // Replay deferred reports
        // TODO: Need to extract deferred_reports from bound without holding borrow
        let had_error = bound.borrow().had_error;
        if had_error {
            return false;
        }

        // Replay tracked symbols
        let tracked = bound.borrow().tracked_symbols.clone();
        let mut ctx = self.ctx.borrow_mut();
        if let Some(ref mut tracker) = ctx.tracker {
            for a in &tracked {
                tracker.track_symbol(&a.symbol, a.enclosing_declaration.as_ref(), a.meaning);
            }
        }
        true
    }

    /// Try to reuse an existing type node, creating a recovery boundary.
    ///
    /// Mirrors Go's `NodeBuilderImpl.tryReuseExistingNodeHelper`.
    pub fn try_reuse_existing_node_helper(&mut self, existing: &Arc<Node>) -> Option<Arc<Node>> {
        let bound = self.create_recovery_boundary();
        // TODO: Port full implementation:
        //   v := getExistingNodeTreeVisitor(self, bound)
        //   transformed = v.VisitNode(existing)
        //   if !self.finalize_boundary(bound) { return None }
        //   self.ctx.approximate_length += existing.loc.end - existing.loc.pos
        //   return transformed
        self.finalize_boundary(&bound);
        None
    }

    // ── Module specifier rewriting ──────────────────────────────────────

    /// Get the overridden module specifier for an import type node.
    ///
    /// Mirrors Go's `NodeBuilderImpl.getModuleSpecifierOverride`.
    pub fn get_module_specifier_override(&mut self, parent: &Arc<Node>, lit: &Arc<Node>) -> String {
        // TODO: Port full implementation including:
        // - b.ctx.enclosingFile != ast.GetSourceFileOfNode(lit)
        // - resolution mode handling
        // - b.tryGetResolvedSymbolFromTypeNode(parent)
        // - b.ch.IsSymbolAccessible(...)
        // - b.getSpecifierForModuleSymbol(...)
        // - node_modules path detection
        String::new()
    }

    /// Rewrite a module specifier if needed.
    ///
    /// Mirrors Go's `NodeBuilderImpl.rewriteModuleSpecifier`.
    pub fn rewrite_module_specifier(&mut self, parent: &Arc<Node>, lit: &Arc<Node>) -> Arc<Node> {
        let new_name = self.get_module_specifier_override(parent, lit);
        if new_name.is_empty() {
            return Arc::clone(lit);
        }
        // TODO: b.f.NewStringLiteral(new_name, ast.TokenFlagsNone)
        // b.e.SetOriginal(res, lit)
        Arc::clone(lit)
    }

    /// Get the enclosing declaration, skipping fake scopes.
    ///
    /// Mirrors Go's `NodeBuilderImpl.getEnclosingDeclarationIgnoringFakeScope`.
    pub fn get_enclosing_declaration_ignoring_fake_scope(&self) -> Option<Arc<Node>> {
        let enc = self.ctx.borrow().enclosing_declaration.clone();
        // TODO: Port fakeScopeForSignatureDeclaration check
        // while enc != nil && b.links.Get(enc).fake_scope_for_signature_declaration != nil {
        //     enc = enc.parent
        // }
        enc
    }

    // ── Text range helper ───────────────────────────────────────────────

    /// Set the text range of `node` to match `range_node`, cloning if
    /// the node originates from a different file.
    ///
    /// Mirrors Go's `NodeBuilderImpl.setTextRange` (inline in Go).
    pub fn set_text_range(&self, node: Arc<Node>, range_node: &Arc<Node>) -> Arc<Node> {
        // TODO: Port full implementation including non-local node detection
        // and core.NewTextRange(-1, -1) for cross-file nodes.
        node
    }

    /// Create a new identifier node with an optional symbol.
    ///
    /// Mirrors Go's `NodeBuilderImpl.newIdentifier`.
    pub fn new_identifier(&mut self, text: &str, symbol: Option<&Arc<Symbol>>) -> Arc<Node> {
        // TODO: Port ast.NodeFactory.NewIdentifier
        // For now, create a minimal node
        let node = Node::new(SyntaxKind::Identifier, crate::ast::NodeData::Token);
        if let Some(sym) = symbol {
            // TODO: Set symbol on node
            let _ = sym;
        }
        Arc::new(node)
    }

    // ── Deep clone helpers ──────────────────────────────────────────────

    /// Get a synthesized deep clone of a node (position info stripped).
    ///
    /// Mirrors Go's `NodeBuilderImpl.getSynthesizedDeepClone`.
    pub fn get_synthesized_deep_clone(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        // TODO: Port full implementation using NodeFactory.DeepCloneNode
        // and stripping position info via set_text_range with synthesized range
        Some(Arc::clone(node))
    }

    /// Get synthesized deep clones of a list of nodes.
    ///
    /// Mirrors Go's `NodeBuilderImpl.getSynthesizedDeepClones`.
    pub fn get_synthesized_deep_clones(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>> {
        nodes
            .iter()
            .filter_map(|n| self.get_synthesized_deep_clone(n))
            .collect()
    }

    /// Deep-clone a node.
    ///
    /// Mirrors Go's `NodeBuilderImpl.deepCloneNode`.
    pub fn deep_clone_node(&self, node: &Arc<Node>) -> Arc<Node> {
        // TODO: Port ast.NodeFactory.DeepCloneNode
        Arc::clone(node)
    }

    // ── Type helpers (stubs) ────────────────────────────────────────────

    /// TODO: Port `NodeBuilderImpl.getTypeFromTypeNode`.
    pub fn get_type_from_type_node(
        &mut self,
        node: &Arc<Node>,
        _ignore_errors: bool,
    ) -> Option<Arc<Type>> {
        // TODO: Port full implementation
        None
    }

    /// TODO: Port `NodeBuilderImpl.typeToTypeNode`.
    pub fn type_to_type_node(&mut self, t: &Arc<Type>) -> Option<Arc<Node>> {
        // TODO: Port full implementation
        None
    }

    /// TODO: Port `NodeBuilderImpl.serializeTypeName`.
    pub fn serialize_type_name(
        &mut self,
        _node: &Arc<Node>,
        _is_type_query: bool,
        _type_arguments: Option<&[Arc<Node>]>,
    ) -> Option<Arc<Node>> {
        // TODO: Port full implementation
        None
    }

    /// TODO: Port `NodeBuilderImpl.canReuseExistingJSTypeNode`.
    pub fn can_reuse_existing_js_type_node(
        &mut self,
        node: &Arc<Node>,
        t: Option<&Arc<Type>>,
    ) -> bool {
        // TODO: Port full implementation
        true
    }

    /// TODO: Port `NodeBuilderImpl.checkTypeExpandability`.
    pub fn check_type_expandability(&mut self, t: &Arc<Type>) {
        // TODO: Port full implementation
    }

    /// TODO: Port `NodeBuilderImpl.enterNewScope`.
    pub fn enter_new_scope(
        &mut self,
        _node: &Arc<Node>,
        _params: Option<Vec<Arc<Symbol>>>,
        _type_params: Option<Vec<Arc<Type>>>,
        _arg1: Option<()>,
        _arg2: Option<()>,
    ) -> impl FnOnce() {
        // TODO: Port full implementation
        || {}
    }

    /// TODO: Port `NodeBuilderImpl.typeParameterToName`.
    pub fn type_parameter_to_name(&mut self, t: &Arc<Type>) -> Arc<Node> {
        // TODO: Port full implementation
        let node = Node::new(SyntaxKind::Identifier, crate::ast::NodeData::Token);
        Arc::new(node)
    }

    /// TODO: Port `NodeBuilderImpl.tryGetResolvedSymbolFromTypeNode`.
    pub fn try_get_resolved_symbol_from_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        // TODO: Port full implementation
        None
    }

    /// TODO: Port `NodeBuilderImpl.lookupSymbolChain`.
    pub fn lookup_symbol_chain(
        &mut self,
        symbol: &Arc<Symbol>,
        _meaning: SymbolFlags,
        _use_only_external_aliasing: bool,
    ) -> Vec<Arc<Symbol>> {
        // TODO: Port full implementation
        vec![Arc::clone(symbol)]
    }

    /// TODO: Port `NodeBuilderImpl.getSpecifierForModuleSymbol`.
    pub fn get_specifier_for_module_symbol(&mut self, symbol: &Arc<Symbol>, _mode: u32) -> String {
        // TODO: Port full implementation
        String::new()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Free functions
// ────────────────────────────────────────────────────────────────────────────

/// Whether a symbol is an external module symbol.
///
/// Mirrors Go's `IsExternalModuleSymbol` (referenced in nodecopy.go).
pub fn is_external_module_symbol(symbol: &Symbol) -> bool {
    symbol.is_external_module()
}

/// Get the meaning of an entity name reference.
///
/// TODO: Port from Go's `getMeaningOfEntityNameReference`.
pub fn get_meaning_of_entity_name_reference(node: &Arc<Node>) -> SymbolFlags {
    // TODO: Port full implementation
    SymbolFlags::TYPE
}

// ────────────────────────────────────────────────────────────────────────────
// getExistingNodeTreeVisitor — stubbed
// ────────────────────────────────────────────────────────────────────────────

/// A node visitor for reuse-existing-node-tree processing.
///
/// TODO: Port from Go's `getExistingNodeTreeVisitor`. The Go implementation
/// is ~600 lines and depends on `ast.NodeVisitor`, `ast.NodeFactory`, and
/// extensive node-update factory methods that are not yet ported.
pub struct ExistingNodeTreeVisitor {
    // TODO: visitor: ast.NodeVisitor,
}

impl ExistingNodeTreeVisitor {
    /// Create the visitor.
    ///
    /// Mirrors Go's `getExistingNodeTreeVisitor`.
    pub fn new(_b: &mut NodeBuilderImpl, _bound: &Rc<RefCell<RecoveryBoundary>>) -> Self {
        // TODO: Port full implementation
        ExistingNodeTreeVisitor {}
    }

    /// Visit a node.
    ///
    /// TODO: Port the visitor logic.
    pub fn visit_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        // TODO: Port full implementation
        Some(Arc::clone(node))
    }

    /// Visit a list of nodes.
    ///
    /// TODO: Port the visitor logic.
    pub fn visit_nodes(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>> {
        // TODO: Port full implementation
        nodes.to_vec()
    }
}
