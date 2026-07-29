//! JSX type checking.
//!
//! Ported from `internal/checker/jsx.go` in the Go implementation. The Go
//! version is ~1500 lines covering JSX namespace resolution, intrinsic
//! element checking, component checking, attribute/children type checking,
//! and JSX factory resolution.
//!
//! This module currently implements a pragmatic subset:
//!
//! - `JsxFlags`, `JsxReferenceKind`, `JsxNames`, `ReactNames` constants
//! - `is_intrinsic_jsx_name` / `is_jsx_intrinsic_tag_name` helpers
//! - JSX namespace resolution (`get_jsx_namespace`, `get_jsx_type`)
//! - JSX precondition check (TS17004: `--jsx` flag required)
//! - Intrinsic element checking (TS7026: missing `JSX.IntrinsicElements`)
//! - Component reference resolution (TS2604: no call/construct signatures)
//! - JSX grammar checks (`check_grammar_jsx_element`,
//!   `check_grammar_jsx_name`, `check_grammar_jsx_expression`)
//!
//! Full attribute type checking, JSX children type checking, JSX factory
//! resolution, and elaborate component constraints (TS2787/2788/2789/18053)
//! are not yet implemented.

use std::sync::Arc;

use crate::ast::{is_identifier, is_jsx_namespaced_name, Node, NodeData, SyntaxKind};
use crate::diagnostics::messages_generated::*;
use crate::diagnostics::Message;

use super::checker::Checker;

// ────────────────────────────────────────────────────────────────────────────
// Constants and helpers
// ────────────────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    /// Flags describing how a JSX element was resolved.
    ///
    /// Mirrors Go's `JsxFlags`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct JsxFlags: u32 {
        /// An element from a named property of the
        /// `JSX.IntrinsicElements` interface.
        const INTRINSIC_NAMED_ELEMENT = 1 << 0;
        /// An element inferred from the string index signature of the
        /// `JSX.IntrinsicElements` interface.
        const INTRINSIC_INDEXED_ELEMENT = 1 << 1;
    }
}

impl JsxFlags {
    /// Combination of both intrinsic element kinds.
    pub const INTRINSIC_ELEMENT: Self = Self::INTRINSIC_NAMED_ELEMENT.union(Self::INTRINSIC_INDEXED_ELEMENT);
}

/// How a JSX tag name resolves to a callable/constructable entity.
///
/// Mirrors Go's `JsxReferenceKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxReferenceKind {
    /// Reference is a class-based component (construct signatures only).
    Component,
    /// Reference is a function component (call signatures only).
    Function,
    /// Reference has both call and construct signatures.
    Mixed,
}

/// Well-known names in the global `JSX` namespace.
pub struct JsxNames;
impl JsxNames {
    pub const JSX: &'static str = "JSX";
    pub const INTRINSIC_ELEMENTS: &'static str = "IntrinsicElements";
    pub const ELEMENT_CLASS: &'static str = "ElementClass";
    pub const ELEMENT_ATTRIBUTES_PROPERTY_NAME_CONTAINER: &'static str = "ElementAttributesProperty";
    pub const ELEMENT_CHILDREN_ATTRIBUTE_NAME_CONTAINER: &'static str = "ElementChildrenAttribute";
    pub const ELEMENT: &'static str = "Element";
    pub const ELEMENT_TYPE: &'static str = "ElementType";
    pub const INTRINSIC_ATTRIBUTES: &'static str = "IntrinsicAttributes";
    pub const INTRINSIC_CLASS_ATTRIBUTES: &'static str = "IntrinsicClassAttributes";
    pub const LIBRARY_MANAGED_ATTRIBUTES: &'static str = "LibraryManagedAttributes";
}

/// Well-known React names.
pub struct ReactNames;
impl ReactNames {
    pub const FRAGMENT: &'static str = "Fragment";
}

/// Whether `name` is a valid JSX intrinsic tag name.
///
/// Mirrors Go's `scanner.IsIntrinsicJsxName`: a name is intrinsic if it
/// starts with a lowercase ASCII letter or contains a hyphen.
pub fn is_intrinsic_jsx_name(name: &str) -> bool {
    !name.is_empty()
        && (name.as_bytes()[0].is_ascii_lowercase() || name.contains('-'))
}

/// Whether a JSX tag name node refers to an intrinsic element.
///
/// Mirrors Go's `isJsxIntrinsicTagName`: an identifier with an intrinsic
/// name, or any `JsxNamespacedName`.
pub fn is_jsx_intrinsic_tag_name(tag_name: &Arc<Node>) -> bool {
    (is_identifier(tag_name) && is_intrinsic_jsx_name(tag_name.text()))
        || is_jsx_namespaced_name(tag_name)
}

/// Get the tag-name node of a JSX opening-like element (opening element or
/// self-closing element). Returns `None` for fragments and unknown kinds.
pub fn jsx_tag_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::JsxOpeningElement(data) => Some(Arc::clone(&data.tag_name)),
        NodeData::JsxSelfClosingElement(data) => Some(Arc::clone(&data.tag_name)),
        NodeData::JsxClosingElement(data) => Some(Arc::clone(&data.tag_name)),
        _ => None,
    }
}

/// Get the attributes node of a JSX opening-like element.
pub fn jsx_attributes(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::JsxOpeningElement(data) => Some(Arc::clone(&data.attributes)),
        NodeData::JsxSelfClosingElement(data) => Some(Arc::clone(&data.attributes)),
        _ => None,
    }
}

/// Whether `node` is a JSX opening-like element (opening element or
/// self-closing element), as opposed to a fragment.
pub fn is_jsx_opening_like_element(node: &Arc<Node>) -> bool {
    matches!(node.kind, SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement)
}

// ────────────────────────────────────────────────────────────────────────────
// JSX namespace resolution
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Look up the `JSX` namespace symbol in the global scope.
    ///
    /// Mirrors Go's `getJsxNamespaceAt`.
    pub fn get_jsx_namespace(&self) -> Option<Arc<crate::ast::Symbol>> {
        self.globals.get(JsxNames::JSX).cloned()
    }

    /// Look up a type within the JSX namespace by name (e.g. `Element`,
    /// `IntrinsicElements`).
    ///
    /// Mirrors Go's `getJsxType`.
    pub fn get_jsx_type(&self, name: &str) -> Option<Arc<crate::ast::Symbol>> {
        let ns = self.get_jsx_namespace()?;
        ns.members.get(name).or_else(|| ns.exports.get(name)).cloned()
    }

    /// Look up the `JSX.Element` symbol.
    pub fn get_jsx_element_type(&self) -> Option<Arc<crate::ast::Symbol>> {
        self.get_jsx_type(JsxNames::ELEMENT)
    }

    /// Look up the `JSX.IntrinsicElements` symbol.
    pub fn get_jsx_intrinsic_elements(&self) -> Option<Arc<crate::ast::Symbol>> {
        self.get_jsx_type(JsxNames::INTRINSIC_ELEMENTS)
    }

    /// Whether `--jsx` is set to anything other than `None`.
    pub fn is_jsx_enabled(&self) -> bool {
        self.compiler_options.jsx != crate::core::compiler_options::JsxEmit::None
    }

    // ─────────────────────────────────────────────────────────────────────
    // JSX preconditions
    // ─────────────────────────────────────────────────────────────────────

    /// Emit precondition errors for using JSX.
    ///
    /// Mirrors Go's `checkJsxPreconditions`:
    /// - TS17004 if `--jsx` is not provided.
    /// - TS2602 if `noImplicitAny` and `JSX.Element` doesn't exist.
    pub fn check_jsx_preconditions(&mut self, error_node: &Arc<Node>) {
        if !self.is_jsx_enabled() {
            self.grammar_error_on_node(error_node, &CANNOT_USE_JSX_UNLESS_THE_JSX_FLAG_IS_PROVIDED);
        }
        if self.no_implicit_any && self.get_jsx_element_type().is_none() {
            self.grammar_error_on_node(
                error_node,
                &JSX_ELEMENT_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_THE_GLOBAL_TYPE_JSX_ELEMENT_DOES_NOT_EXIST,
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Intrinsic element checking
    // ─────────────────────────────────────────────────────────────────────

    /// Check a JSX intrinsic tag name (e.g. `<div>`).
    ///
    /// Mirrors the relevant slice of Go's `resolveJsxOpeningLikeElement`:
    /// - If `JSX.IntrinsicElements` exists, ensure `tag_name` is a member
    ///   or the interface has a string index signature.
    /// - Otherwise emit TS7026 ("JSX element implicitly has type 'any'
    ///   because no interface 'JSX.IntrinsicElements' exists").
    pub fn check_jsx_intrinsic_element(&mut self, opening: &Arc<Node>) {
        let tag_name = match jsx_tag_name(opening) {
            Some(t) => t,
            None => return,
        };
        let tag_text = tag_name.text().to_string();

        let intrinsic_elements = match self.get_jsx_intrinsic_elements() {
            Some(sym) => sym,
            None => {
                // No JSX.IntrinsicElements interface in scope.
                if self.no_implicit_any {
                    self.grammar_error_on_node_with_args(
                        opening,
                        &JSX_ELEMENT_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_NO_INTERFACE_JSX_0_EXISTS,
                        &[JsxNames::INTRINSIC_ELEMENTS.to_string()],
                    );
                }
                return;
            }
        };

        // Try to find the tag as a named member first.
        let member = intrinsic_elements
            .members
            .get(&tag_text)
            .or_else(|| intrinsic_elements.exports.get(&tag_text));

        if member.is_none() {
            // No named member; check for a string index signature on the
            // intrinsic elements type. We don't yet model the type
            // thoroughly enough to look up index signatures here, so emit
            // TS7026 only when there are zero members (i.e. the interface
            // is empty).
            //
            // TODO: once type/index-signature resolution is in place, look
            // up the string index signature and treat the element as
            // valid if one exists.
            if intrinsic_elements.members.is_empty() && intrinsic_elements.exports.is_empty() {
                if self.no_implicit_any {
                    self.grammar_error_on_node_with_args(
                        opening,
                        &JSX_ELEMENT_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_NO_INTERFACE_JSX_0_EXISTS,
                        &[JsxNames::INTRINSIC_ELEMENTS.to_string()],
                    );
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Component (non-intrinsic) element checking
    // ─────────────────────────────────────────────────────────────────────

    /// Check a JSX component reference (e.g. `<Foo />`).
    ///
    /// Mirrors the relevant slice of Go's `resolveJsxOpeningLikeElement`:
    /// resolve the tag name as a value, ensure it has at least one call or
    /// construct signature; otherwise emit TS2604.
    pub fn check_jsx_component(&mut self, opening: &Arc<Node>) {
        let tag_name = match jsx_tag_name(opening) {
            Some(t) => t,
            None => return,
        };

        // Resolve the tag name as a value reference. This will emit TS2304
        // if the name is undefined.
        self.check_expression(&tag_name);

        // Look up the resolved type to check signatures.
        let tag_type = self.get_type_of_node(&tag_name);

        let has_call_sigs = !self.get_signatures_of_type(&tag_type, crate::checker::SignatureKind::Call).is_empty();
        let has_construct_sigs = !self
            .get_signatures_of_type(&tag_type, crate::checker::SignatureKind::Construct)
            .is_empty();

        if !has_call_sigs && !has_construct_sigs {
            let text = tag_name.text().to_string();
            self.grammar_error_on_node_with_args(
                &tag_name,
                &JSX_ELEMENT_TYPE_0_DOES_NOT_HAVE_ANY_CONSTRUCT_OR_CALL_SIGNATURES,
                &[text],
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Top-level JSX entry points
    // ─────────────────────────────────────────────────────────────────────

    /// Check a JSX opening-like element or opening fragment.
    ///
    /// Mirrors Go's `checkJsxOpeningLikeElementOrOpeningFragment` (a
    /// reduced subset):
    /// - Run grammar checks (duplicate attributes, namespace names, etc.)
    /// - Run precondition checks (TS17004, TS2602)
    /// - For intrinsic elements: check `JSX.IntrinsicElements` membership
    /// - For component elements: check tag name has call/construct
    ///   signatures (TS2604)
    pub fn check_jsx_opening_like_element(&mut self, opening: &Arc<Node>) {
        let is_opening_like = is_jsx_opening_like_element(opening);
        if is_opening_like {
            self.check_grammar_jsx_element(opening);
        }
        self.check_jsx_preconditions(opening);
        if !is_opening_like {
            return;
        }
        let tag_name = match jsx_tag_name(opening) {
            Some(t) => t,
            None => return,
        };
        if is_jsx_intrinsic_tag_name(&tag_name) {
            self.check_jsx_intrinsic_element(opening);
        } else {
            self.check_jsx_component(opening);
        }
    }
}
