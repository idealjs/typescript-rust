#![allow(dead_code)]
#![allow(unused_variables)]

//! Symbol tracker for deferred type checking.
//!
//! Ported 1:1 from `internal/checker/symboltracker.go` (129 lines).
//! Provides `SymbolTrackerImpl`, which wraps a `nodebuilder.SymbolTracker`
//! and records symbol references for late visibility painting during
//! declaration emit.
//!
//! Also defines the shared node-builder infrastructure types needed by
//! both this module and `nodecopy.rs`:
//! - `SymbolTracker` trait (Go: `nodebuilder.SymbolTracker`)
//! - `NodeBuilderFlags` / `NodeBuilderInternalFlags` (Go: `nodebuilder.Flags` / `InternalFlags`)
//! - `TrackedSymbolArgs`
//! - `NodeBuilderContext`

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol, SymbolFlags};

// ────────────────────────────────────────────────────────────────────────────
// nodebuilder.Flags / InternalFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    /// Flags controlling node-builder behaviour.
    ///
    /// Mirrors Go's `nodebuilder.Flags` (nodebuilder/types.go).
    /// NOTE: If modifying this, must also modify `TypeFormatFlags`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
    pub struct NodeBuilderFlags: u32 {
        const None                               = 0;
        // Options
        const NoTruncation                        = 1 << 0;
        const WriteArrayAsGenericType             = 1 << 1;
        const GenerateNamesForShadowedTypeParams  = 1 << 2;
        const UseStructuralFallback               = 1 << 3;
        const ForbidIndexedAccessSymbolReferences = 1 << 4;
        const WriteTypeArgumentsOfSignature       = 1 << 5;
        const UseFullyQualifiedType               = 1 << 6;
        const UseOnlyExternalAliasing             = 1 << 7;
        const SuppressAnyReturnType               = 1 << 8;
        const WriteTypeParametersInQualifiedName  = 1 << 9;
        const MultilineObjectLiterals             = 1 << 10;
        const WriteClassExpressionAsTypeLiteral   = 1 << 11;
        const UseTypeOfFunction                   = 1 << 12;
        const OmitParameterModifiers              = 1 << 13;
        const UseAliasDefinedOutsideCurrentScope  = 1 << 14;
        const UseSingleQuotesForStringLiteralType = 1 << 28;
        const NoTypeReduction                     = 1 << 29;
        const UseInstantiationExpressions         = 1 << 30;
        const OmitThisParameter                   = 1 << 25;
        const WriteCallStyleSignature             = 1 << 27;
        // Error handling
        const AllowThisInObjectLiteral              = 1 << 15;
        const AllowQualifiedNameInPlaceOfIdentifier = 1 << 16;
        const AllowAnonymousIdentifier              = 1 << 17;
        const AllowEmptyUnionOrIntersection         = 1 << 18;
        const AllowEmptyTuple                       = 1 << 19;
        const AllowUniqueESSymbolType               = 1 << 20;
        const AllowEmptyIndexInfoType               = 1 << 21;
        // Errors (cont.)
        const AllowNodeModulesRelativePaths = 1 << 26;
        // State
        const InObjectTypeLiteral = 1 << 22;
        const InTypeAlias         = 1 << 23;
        const InInitialEntityName = 1 << 24;
    }
}

impl NodeBuilderFlags {
    /// Composite: `FlagsIgnoreErrors`.
    pub const IGNORE_ERRORS: Self = Self::AllowThisInObjectLiteral
        .union(Self::AllowQualifiedNameInPlaceOfIdentifier)
        .union(Self::AllowAnonymousIdentifier)
        .union(Self::AllowEmptyUnionOrIntersection)
        .union(Self::AllowEmptyTuple)
        .union(Self::AllowEmptyIndexInfoType)
        .union(Self::AllowNodeModulesRelativePaths);
}

bitflags::bitflags! {
    /// Internal node-builder flags.
    ///
    /// Mirrors Go's `nodebuilder.InternalFlags` (nodebuilder/types.go).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
    pub struct NodeBuilderInternalFlags: i32 {
        const None                    = 0;
        const WriteComputedProps      = 1 << 0;
        const NoSyntacticPrinter      = 1 << 1;
        const DoNotIncludeSymbolChain = 1 << 2;
        const AllowUnresolvedNames    = 1 << 3;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// SymbolTracker trait
// ────────────────────────────────────────────────────────────────────────────

/// Trait for tracking symbol references during type-to-node serialization.
///
/// Mirrors Go's `nodebuilder.SymbolTracker` interface (nodebuilder/types.go).
/// The concrete implementation is `SymbolTrackerImpl` (in this module).
/// A wrapping decorator is `WrappingTracker` (in `nodecopy.rs`).
pub trait SymbolTracker {
    /// Record a symbol reference. Returns `true` if the symbol was already
    /// tracked (and thus an error/diagnostic should be reported).
    fn track_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool;

    fn report_inaccessible_this_error(&mut self);
    fn report_private_in_base_of_class_expression(&mut self, property_name: &str);
    fn report_inaccessible_unique_symbol_error(&mut self);
    fn report_cyclic_structure_error(&mut self);
    fn report_likely_unsafe_import_required_error(&mut self, specifier: &str, symbol_name: &str);
    fn report_truncation_error(&mut self);
    fn report_nonlocal_augmentation(
        &mut self,
        containing_file: Option<&Arc<SourceFile>>,
        parent_symbol: &Arc<Symbol>,
        augmenting_symbol: &Arc<Symbol>,
    );
    fn report_non_serializable_property(&mut self, property_name: &str);

    fn report_inference_fallback(&mut self, node: &Arc<Node>);
    fn push_error_fallback_node(&mut self, node: &Arc<Node>);
    fn pop_error_fallback_node(&mut self);
}

// ────────────────────────────────────────────────────────────────────────────
// TrackedSymbolArgs
// ────────────────────────────────────────────────────────────────────────────

/// Arguments for a deferred `TrackSymbol` call.
///
/// Mirrors Go's `TrackedSymbolArgs` (nodebuilderimpl.go).
#[derive(Clone)]
pub struct TrackedSymbolArgs {
    pub symbol: Arc<Symbol>,
    pub enclosing_declaration: Option<Arc<Node>>,
    pub meaning: SymbolFlags,
}

// ────────────────────────────────────────────────────────────────────────────
// NodeBuilderContext
// ────────────────────────────────────────────────────────────────────────────

/// Context for a node-builder session.
///
/// Mirrors Go's `NodeBuilderContext` (nodebuilderimpl.go). Only the fields
/// needed by the ported modules are defined; the full Go struct has ~25
/// fields. Fields are added as they are needed by ported code.
#[derive(Default)]
pub struct NodeBuilderContext {
    pub tracker: Option<Box<dyn SymbolTracker>>,
    pub approximate_length: usize,
    pub max_truncation_length: usize,
    pub encountered_error: bool,
    pub truncating: bool,
    pub reported_diagnostic: bool,
    pub flags: NodeBuilderFlags,
    pub internal_flags: NodeBuilderInternalFlags,
    pub depth: usize,
    /// -1 means no expansion, 0+ = verbosity levels.
    pub max_expansion_depth: i32,
    pub can_increase_expansion_depth: bool,
    pub expansion_truncated: bool,
    pub enclosing_declaration: Option<Arc<Node>>,
    pub enclosing_file: Option<Arc<SourceFile>>,
    pub tracked_symbols: Vec<TrackedSymbolArgs>,
    /// Whether to suppress `ReportInferenceFallback` calls.
    pub suppress_report_inference_fallback: bool,
}

/// Shared, mutable reference to a `NodeBuilderContext`.
pub type SharedNodeBuilderContext = Rc<RefCell<NodeBuilderContext>>;

// Default constants
pub const DEFAULT_MAXIMUM_TRUNCATION_LENGTH: usize = 160;
pub const NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH: usize = 1_000_000;

// ────────────────────────────────────────────────────────────────────────────
// SymbolTrackerImpl
// ────────────────────────────────────────────────────────────────────────────

/// Concrete `SymbolTracker` implementation used by the checker's node builder.
///
/// Mirrors Go's `SymbolTrackerImpl` (symboltracker.go). Wraps an inner
/// tracker (which may be another `SymbolTrackerImpl` or a `WrappingTracker`),
/// records tracked symbols for late visibility painting, and forwards
/// diagnostic reports.
pub struct SymbolTrackerImpl {
    pub context: SharedNodeBuilderContext,
    pub inner: Option<Box<dyn SymbolTracker>>,
    pub disable_track_symbol: bool,
}

impl SymbolTrackerImpl {
    /// Create a new `SymbolTrackerImpl`.
    ///
    /// Mirrors Go's `NewSymbolTrackerImpl`. If `tracker` is itself a
    /// `SymbolTrackerImpl`, unwraps to its inner tracker to avoid
    /// double-wrapping.
    pub fn new(context: SharedNodeBuilderContext, tracker: Option<Box<dyn SymbolTracker>>) -> Self {
        // Unwrap nested SymbolTrackerImpl instances.
        // In Go, this is done via type assertion `tracker.(*SymbolTrackerImpl)`.
        // In Rust with trait objects, we can't easily downcast, so we keep
        // the tracker as-is. The wrapping is handled at a higher level.
        // TODO: Implement unwrapping of nested SymbolTrackerImpl via downcast
        SymbolTrackerImpl {
            context,
            inner: tracker,
            disable_track_symbol: false,
        }
    }

    /// Mark that a diagnostic was reported in the context.
    ///
    /// Mirrors Go's `onDiagnosticReported`.
    fn on_diagnostic_reported(&self) {
        self.context.borrow_mut().reported_diagnostic = true;
    }
}

impl SymbolTracker for SymbolTrackerImpl {
    fn track_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool {
        if !self.disable_track_symbol {
            if let Some(ref mut inner) = self.inner {
                if inner.track_symbol(symbol, enclosing_declaration, meaning) {
                    self.on_diagnostic_reported();
                    return true;
                }
            }
            // Skip recording type parameters as they dont contribute to late painted statements
            if !symbol.flags.intersects(SymbolFlags::TypeParameter) {
                self.context
                    .borrow_mut()
                    .tracked_symbols
                    .push(TrackedSymbolArgs {
                        symbol: Arc::clone(symbol),
                        enclosing_declaration: enclosing_declaration.cloned(),
                        meaning,
                    });
            }
        }
        false
    }

    fn report_inaccessible_this_error(&mut self) {
        self.on_diagnostic_reported();
        if let Some(ref mut inner) = self.inner {
            inner.report_inaccessible_this_error();
        }
    }

    fn report_private_in_base_of_class_expression(&mut self, property_name: &str) {
        self.on_diagnostic_reported();
        if let Some(ref mut inner) = self.inner {
            inner.report_private_in_base_of_class_expression(property_name);
        }
    }

    fn report_inaccessible_unique_symbol_error(&mut self) {
        self.on_diagnostic_reported();
        if let Some(ref mut inner) = self.inner {
            inner.report_inaccessible_unique_symbol_error();
        }
    }

    fn report_cyclic_structure_error(&mut self) {
        self.on_diagnostic_reported();
        if let Some(ref mut inner) = self.inner {
            inner.report_cyclic_structure_error();
        }
    }

    fn report_likely_unsafe_import_required_error(&mut self, specifier: &str, symbol_name: &str) {
        self.on_diagnostic_reported();
        if let Some(ref mut inner) = self.inner {
            inner.report_likely_unsafe_import_required_error(specifier, symbol_name);
        }
    }

    fn report_truncation_error(&mut self) {
        self.on_diagnostic_reported();
        if let Some(ref mut inner) = self.inner {
            inner.report_truncation_error();
        }
    }

    fn report_nonlocal_augmentation(
        &mut self,
        containing_file: Option<&Arc<SourceFile>>,
        parent_symbol: &Arc<Symbol>,
        augmenting_symbol: &Arc<Symbol>,
    ) {
        self.on_diagnostic_reported();
        if let Some(ref mut inner) = self.inner {
            inner.report_nonlocal_augmentation(containing_file, parent_symbol, augmenting_symbol);
        }
    }

    fn report_non_serializable_property(&mut self, property_name: &str) {
        self.on_diagnostic_reported();
        if let Some(ref mut inner) = self.inner {
            inner.report_non_serializable_property(property_name);
        }
    }

    fn report_inference_fallback(&mut self, node: &Arc<Node>) {
        if let Some(ref mut inner) = self.inner {
            inner.report_inference_fallback(node);
        }
    }

    fn push_error_fallback_node(&mut self, node: &Arc<Node>) {
        if let Some(ref mut inner) = self.inner {
            inner.push_error_fallback_node(node);
        }
    }

    fn pop_error_fallback_node(&mut self) {
        if let Some(ref mut inner) = self.inner {
            inner.pop_error_fallback_node();
        }
    }
}
