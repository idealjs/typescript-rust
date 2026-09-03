#![allow(dead_code)]
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

use crate::ast::{Node, NodeData, SyntaxKind, is_identifier, is_jsx_namespaced_name};
use crate::diagnostics::Message;
use crate::diagnostics::messages_generated::*;

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
    pub const INTRINSIC_ELEMENT: Self =
        Self::INTRINSIC_NAMED_ELEMENT.union(Self::INTRINSIC_INDEXED_ELEMENT);
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
    pub const ELEMENT_ATTRIBUTES_PROPERTY_NAME_CONTAINER: &'static str =
        "ElementAttributesProperty";
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
    !name.is_empty() && (name.as_bytes()[0].is_ascii_lowercase() || name.contains('-'))
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
    matches!(
        node.kind,
        SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement
    )
}

// ────────────────────────────────────────────────────────────────────────────
// JSX namespace resolution
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Look up the `JSX` namespace symbol in the global scope.
    ///
    /// Mirrors Go's `getJsxNamespaceAt`: under `react-jsx`/`react-jsxdev`
    /// the runtime container's `JSX` export wins (computed lazily by
    /// `ensure_jsx_implicit_container` via `check_jsx_preconditions`);
    /// otherwise — and as fallback when the container yields nothing —
    /// the global `JSX` namespace.
    pub fn get_jsx_namespace(&self) -> Option<Arc<crate::ast::Symbol>> {
        let file_id = self.current_file_id as usize;
        if let Some(cached) = self.jsx_implicit_namespace.get(&file_id)
            && let Some(ns) = cached
        {
            return Some(Arc::clone(ns));
        }
        self.globals.get(JsxNames::JSX).cloned()
    }

    /// Look up a type within the JSX namespace by name (e.g. `Element`,
    /// `IntrinsicElements`).
    ///
    /// Mirrors Go's `getJsxType`. Ambient `declare namespace JSX` bodies
    /// (react.d.ts fixtures) bind their members into the namespace node's
    /// LOCALS rather than the symbol tables — the same fallback
    /// `resolve_qualified_symbol_traced` applies for `JSX.Element` type
    /// references (`ambient_namespace_local`).
    pub fn get_jsx_type(&self, name: &str) -> Option<Arc<crate::ast::Symbol>> {
        let ns = self.get_jsx_namespace()?;
        ns.members
            .get(name)
            .or_else(|| ns.exports.get(name))
            .cloned()
            .or_else(|| self.ambient_namespace_local(&ns, name))
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
    ///
    /// Also the lazy hook for the implicit JSX runtime import (Go
    /// `getJsxNamespaceContainerForImplicitImport`): under
    /// `react-jsx`/`react-jsxdev` the `<importSource>/jsx-runtime`
    /// (or `/jsx-dev-runtime`) module must resolve — TS2875 at the first
    /// JSX tag when it doesn't — and the JSX namespace is then looked up
    /// in the container's exports before the global fallback.
    pub fn check_jsx_preconditions(&mut self, error_node: &Arc<Node>) {
        if !self.is_jsx_enabled() {
            self.grammar_error_on_node(error_node, &CANNOT_USE_JSX_UNLESS_THE_JSX_FLAG_IS_PROVIDED);
        }
        // TS2602 fires when there is no JSX NAMESPACE at all (no React
        // types in scope). A namespace that exists but lacks `Element`
        // (the inline-jsx-factory fixtures declare only
        // IntrinsicElements) types the element as `any` silently —
        // official baselines report nothing there.
        if self.no_implicit_any && self.get_jsx_namespace().is_none() {
            self.grammar_error_on_node(
                error_node,
                &JSX_ELEMENT_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_THE_GLOBAL_TYPE_JSX_ELEMENT_DOES_NOT_EXIST,
            );
        }
    }

    /// Resolve (once per checker) the implicit JSX runtime import container
    /// for `react-jsx`/`react-jsxdev` modes and cache the resulting JSX
    /// namespace. Mirrors Go `getJsxNamespaceContainerForImplicitImport` +
    /// the container-then-global lookup of `getJsxNamespaceAt`: when the
    /// runtime module resolves, its `JSX` export wins; otherwise (module
    /// unresolvable — TS2875 — or no `JSX` export inside it) the global
    /// `JSX` namespace is the fallback.
    fn ensure_jsx_implicit_container(&mut self, error_node: &Arc<Node>) {
        use crate::core::compiler_options::JsxEmit;
        let file_id = self.current_file_id as usize;
        if self.jsx_implicit_namespace.contains_key(&file_id) {
            return;
        }
        let resolved: Option<std::sync::Arc<crate::ast::Symbol>> =
            match self.compiler_options.jsx {
                JsxEmit::ReactJSX | JsxEmit::ReactJSXDev => {
                    let source = if self.compiler_options.jsx_import_source.is_empty() {
                        "react"
                    } else {
                        self.compiler_options.jsx_import_source.as_str()
                    };
                    let module_ref = if self.compiler_options.jsx == JsxEmit::ReactJSXDev {
                        format!("{source}/jsx-dev-runtime")
                    } else {
                        format!("{source}/jsx-runtime")
                    };
                    match self
                        .resolve_module_file_symbol(&module_ref)
                        .or_else(|| self.resolve_jsx_runtime_by_path(&module_ref))
                    {
                        Some(module_sym) => {
                            let ns = module_sym
                                .exports
                                .get(JsxNames::JSX)
                                .or_else(|| module_sym.members.get(JsxNames::JSX))
                                .cloned();
                            ns
                        }
                        None => {
                            // DEFERRED: official orders the element's own
                            // diagnostics (7026) before TS2875 at the same
                            // position — the runtime-module failure is
                            // buffered and flushed at the opening-like
                            // element check's tail.
                            // Span note: official anchors TS2875 at the
                            // OUTERMOST JSX expression containing the first
                            // runtime consult — a fragment `<>...</>` reports
                            // at the FRAGMENT (jsxJsxsCjsTransformCustomImport
                            // (2,11)), element nesting at the outer element
                            // (the longer span also sorts after the element's
                            // own 7026 at the same start).
                            let mut span = error_node.loc;
                            let mut node: &Arc<Node> = error_node;
                            while let Some(parent) = node.parent.as_ref() {
                                match parent.kind {
                                    crate::ast::SyntaxKind::JsxElement
                                    | crate::ast::SyntaxKind::JsxSelfClosingElement
                                    | crate::ast::SyntaxKind::JsxFragment => {
                                        span = parent.loc;
                                        node = parent;
                                    }
                                    _ => break,
                                }
                            }
                            self.pending_jsx_2875 = Some((span, module_ref));
                            None
                        }
                    }
                }
                // Classic/preserve modes never consult a runtime module.
                _ => None,
            };
        self.jsx_implicit_namespace.insert(file_id, resolved);
    }

    /// Resolve the implicit JSX runtime module through the program's REAL
    /// resolver (node_modules walks, @types mangling) and return the LOADED
    /// file's symbol. The runtime file must already be in the program —
    /// the compiler's loading loop preloads it for JSX files (Go's
    /// `GetJSXRuntimeImportSpecifier` feeding the file loader).
    fn resolve_jsx_runtime_by_path(&self, module_ref: &str) -> Option<Arc<crate::ast::Symbol>> {
        let containing = self
            .current_file
            .as_ref()
            .map(|f| f.file_name.clone())
            .unwrap_or_default();
        // The file's implied node format (ESNext/CommonJS carriers) picks
        // the import/require condition under node1x — the EMIT format's
        // ES2020 value would wrongly select `require`.
        let mode = crate::compiler::implied_node_format_of_file(&containing, &|p| {
            self.program.read_file(p)
        });
        let path = self
            .program
            .resolve_external_module_path(module_ref, &containing, mode)?;
        let sf = self.program.get_source_file(&path)?;
        self.program.symbol_map().symbol_of(&sf.node).cloned()
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
            // No named member; accept any tag when the interface declares
            // an index signature (`[e: string]: any` — the inline-jsx
            // fixtures' catch-all IntrinsicElements), scanned directly on
            // the declarations (the ambient symbol may not resolve a type
            // from this lookup context). TS7026 only fires when the
            // interface is genuinely empty (no members AND no index
            // signatures).
            let has_index_signature = intrinsic_elements.declarations.iter().any(|d| {
                matches!(&d.data, crate::ast::NodeData::InterfaceDeclaration(id) if id
                    .members
                    .iter()
                    .any(|m| m.kind == SyntaxKind::IndexSignature))
            });
            if intrinsic_elements.members.is_empty()
                && intrinsic_elements.exports.is_empty()
                && !has_index_signature
            {
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

        // If the type is `any` or unresolved (e.g. import binding couldn't
        // be fully resolved), skip TS2604 — it's better to miss a real error
        // than to emit a false positive. Mirrors Go's pragmatic behavior where
        // unresolved types don't trigger component-signature checks.
        if tag_type
            .flags
            .contains(crate::checker::types::TypeFlags::Any)
        {
            return;
        }

        let has_call_sigs = !self
            .get_signatures_of_type(&tag_type, crate::checker::SignatureKind::Call)
            .is_empty();
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

    /// Whether the JSX factory namespace name (`React`) resolves with
    /// VALUE meaning from the current scope stack (locals, member tables,
    /// module exports, then globals) — the TS2874 gate. Node-free variant
    /// of the scope walk in `resolve_identifier_with_meaning_inner`.
    fn jsx_factory_namespace_in_scope(&self, name: &str) -> bool {
        use crate::ast::SymbolFlags;
        let symbol_map = self.program.symbol_map();
        let value = |sym: &std::sync::Arc<crate::ast::Symbol>| {
            // An import (`import React from "react"`) binds an ALIAS — no
            // VALUE flag of its own; resolve through the alias chain (Go
            // getSymbolFlags).
            if sym.flags.intersects(SymbolFlags::Alias) {
                match self.follow_alias(sym) {
                    // An unresolved/cyclic alias returns itself — the name
                    // IS declared (TS2307 already reports the failed
                    // import); don't add TS2874 on top.
                    Some(t) if std::sync::Arc::ptr_eq(&t, sym) => true,
                    Some(t) => t.flags.intersects(SymbolFlags::VALUE),
                    None => true,
                }
            } else {
                sym.flags.intersects(SymbolFlags::VALUE)
            }
        };
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id)
                && let Some(sym) = locals.get(name)
                && value(sym)
            {
                return true;
            }
            if let Some(cs) = symbol_map.symbols.get(&container_id)
                && (!cs.flags.intersects(SymbolFlags::Class)
                    || cs.flags.intersects(SymbolFlags::Function))
                && let Some(sym) = cs.members.get(name)
                && value(sym)
            {
                return true;
            }
            if let Some(cs) = symbol_map.symbols.get(&container_id)
                && cs.flags.intersects(SymbolFlags::MODULE)
                && !cs.flags.intersects(SymbolFlags::Class)
                && let Some(sym) = cs.exports.get(name)
                && value(sym)
            {
                return true;
            }
        }
        self.globals
            .get(name)
            .is_some_and(|g| g.flags.intersects(SymbolFlags::VALUE))
    }

    /// Go `getLocalJsxNamespace` via `GetPragmaFromSourceFile(file, "jsx")`
    /// (parser.go `getCommentPragmas`): the LAST `@jsx`/`@jsxfrag`/
    /// `@jsximportsource`/`@jsxruntime` pragma in the file's leading
    /// multi-line comments overrides the compiler-option factory —
    /// `/** @jsx dom */` makes `dom` the required entity for that file
    /// even under `--jsxFactory p`. Returns the pragma's factory argument.
    fn local_jsx_pragma_factory(&self, pragma: &str) -> Option<String> {
        let file = self.current_file.as_ref()?;
        let ranges = crate::scanner::get_leading_comment_ranges(&file.text, 0);
        let mut result = None;
        for r in ranges {
            if r.kind != crate::scanner::CommentRangeKind::MultiLine {
                continue;
            }
            let comment = &file.text[r.pos..r.end];
            let comment = comment.strip_suffix("*/").unwrap_or(comment);
            for line in comment.split('\n') {
                // Only the FIRST '@'-token on a line can open a pragma (an
                // earlier unrelated '@token' — e.g. an email — blocks it).
                let Some(at) = line.find('@') else {
                    continue;
                };
                let after = &line[at + 1..];
                let name_end = after
                    .find(char::is_whitespace)
                    .unwrap_or(after.len());
                let name = &after[..name_end];
                if !name.eq_ignore_ascii_case(pragma) {
                    continue;
                }
                let args = after[name_end..].trim_start();
                let arg_end = args
                    .find(char::is_whitespace)
                    .unwrap_or(args.len());
                if arg_end > 0 {
                    result = Some(args[..arg_end].to_string());
                }
            }
        }
        result
    }

    pub fn check_jsx_opening_like_element(&mut self, opening: &Arc<Node>) {
        let is_opening_like = is_jsx_opening_like_element(opening);
        if is_opening_like {
            self.check_grammar_jsx_element(opening);
        }
        self.check_jsx_preconditions(opening);
        // TS2874 (Go markJsxAliasReferenced): classic `jsx: react` requires
        // the JSX factory namespace (`React`) resolvable as a VALUE at each
        // opening-like tag — reported at the tag NAME.
        if is_opening_like
            && matches!(
                self.compiler_options.jsx,
                crate::core::compiler_options::JsxEmit::React
            )
            && let Some(tag) = jsx_tag_name(opening)
        {
            // The required entity is the JSX factory's first identifier —
            // `React` by default (or `--reactNamespace`), the first
            // component of `--jsxFactory a.b.c` when set (Go
            // resolveJsxEntityName). A per-file `/** @jsx dom */` pragma
            // OVERRIDES the option factory for that file (Go
            // getLocalJsxNamespace). With an explicit factory the React
            // namespace is never referenced, so a declared `__make`
            // satisfies the gate (TS2874 must not fire).
            let default_name = self
                .compiler_options
                .react_namespace
                .as_str();
            let default_name = if default_name.is_empty() { "React" } else { default_name };
            let option_factory = self
                .compiler_options
                .jsx_factory
                .split('.')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or(default_name)
                .to_string();
            let factory_name = self
                .local_jsx_pragma_factory("jsx")
                .and_then(|f| f.split('.').next().map(str::to_string))
                .filter(|s| !s.is_empty())
                .unwrap_or(option_factory);
            if !self.jsx_factory_namespace_in_scope(&factory_name) {
                self.grammar_error_on_node_with_args(
                    &tag,
                    &THIS_JSX_TAG_REQUIRES_0_TO_BE_IN_SCOPE_BUT_IT_COULD_NOT_BE_FOUND,
                    &[factory_name.to_string()],
                );
            }
        }
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
        // The implicit JSX runtime import (TS2875) resolves AFTER the
        // element's own resolution in Go — at the same position, official
        // orders 7026 before 2875 (commentsOnJSXExpressionsArePreserved).
        // The buffered failure (if any) flushes here, after the element's
        // own diagnostics.
        self.ensure_jsx_implicit_container(opening);
        if let Some((loc, module_ref)) = self.pending_jsx_2875.take() {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                THIS_JSX_TAG_REQUIRES_THE_MODULE_PATH_0_TO_EXIST_BUT_NONE_COULD_BE_FOUND_MAKE_SURE_YOU_HAVE_TYPES_FOR_THE_APPROPRIATE_PACKAGE_INSTALLED,
                vec![module_ref],
            ));
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Additional JSX checking functions (stubs)
    // ─────────────────────────────────────────────────────────────────────

    pub fn check_jsx_element_deferred(&mut self, _node: &Arc<Node>) {
        // TODO: full implementation
    }

    pub fn check_jsx_expression(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: u32,
    ) -> Arc<super::types::Type> {
        // TODO: full implementation
        self.any_type()
    }

    pub fn check_jsx_self_closing_element(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: u32,
    ) -> Arc<super::types::Type> {
        // TODO: full implementation
        self.any_type()
    }

    pub fn check_jsx_self_closing_element_deferred(&mut self, _node: &Arc<Node>) {
        // TODO: full implementation
    }

    pub fn check_jsx_fragment(&mut self, _node: &Arc<Node>) -> Arc<super::types::Type> {
        // TODO: full implementation
        self.any_type()
    }

    pub fn check_jsx_attributes(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: u32,
    ) -> Arc<super::types::Type> {
        // TODO: full implementation
        self.any_type()
    }

    pub fn check_jsx_return_assignable_to_appropriate_bound(
        &mut self,
        _ref_kind: JsxReferenceKind,
        _elem_instance_type: &Arc<super::types::Type>,
        _opening_like_element: &Arc<Node>,
    ) {
        // TODO: full implementation
    }

    pub fn infer_jsx_type_arguments(
        &mut self,
        _node: &Arc<Node>,
        _signature: &Arc<super::types::Signature>,
        _check_mode: u32,
        _context: &super::inference::InferenceContext,
    ) -> Vec<Arc<super::types::Type>> {
        // TODO: full implementation
        Vec::new()
    }

    pub fn get_contextual_type_for_jsx_expression(
        &mut self,
        _node: &Arc<Node>,
        _context_flags: super::types::ContextFlags,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_contextual_type_for_jsx_attribute(
        &mut self,
        _attribute: &Arc<Node>,
        _context_flags: super::types::ContextFlags,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_contextual_jsx_element_attributes_type(
        &mut self,
        _node: &Arc<Node>,
        _context_flags: super::types::ContextFlags,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_contextual_type_for_child_jsx_expression(
        &mut self,
        _node: &Arc<Node>,
        _child: &Arc<Node>,
        _context_flags: super::types::ContextFlags,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn discriminate_contextual_type_by_jsx_attributes(
        &mut self,
        _node: &Arc<Node>,
        contextual_type: &Arc<super::types::Type>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        let _ = contextual_type;
        None
    }

    pub fn elaborate_jsx_components(
        &mut self,
        _node: &Arc<Node>,
        _source: &Arc<super::types::Type>,
        _target: &Arc<super::types::Type>,
        _relation: super::relater::RelationKind,
        _diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        // TODO: full implementation
        false
    }

    pub fn get_suggested_symbol_for_nonexistent_jsx_attribute(
        &mut self,
        _name: &str,
        _containing_type: &Arc<super::types::Type>,
    ) -> Option<Arc<crate::ast::Symbol>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_fragment_type(&mut self, _node: &Arc<Node>) -> Arc<super::types::Type> {
        // TODO: full implementation
        self.any_type()
    }

    pub fn resolve_jsx_opening_like_element(
        &mut self,
        _node: &Arc<Node>,
        _candidates_out_array: Option<&mut Vec<Arc<super::types::Signature>>>,
        _check_mode: u32,
    ) -> Option<Arc<super::types::Signature>> {
        // TODO: full implementation
        None
    }

    pub fn check_applicable_signature_for_jsx_call_like_element(
        &mut self,
        _node: &Arc<Node>,
        _signature: &Arc<super::types::Signature>,
        _relation: super::relater::RelationKind,
        _check_mode: u32,
        _report_errors: bool,
        _diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        // TODO: full implementation
        false
    }

    pub fn create_jsx_attributes_type_from_attributes_property(
        &mut self,
        _opening_like_element: &Arc<Node>,
        _check_mode: u32,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn check_jsx_children(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: u32,
    ) -> Vec<Arc<super::types::Type>> {
        // TODO: full implementation
        Vec::new()
    }

    pub fn get_uninstantiated_jsx_signatures_of_type(
        &mut self,
        _element_type: &Arc<super::types::Type>,
        _caller: &Arc<Node>,
    ) -> Vec<Arc<super::types::Signature>> {
        // TODO: full implementation
        Vec::new()
    }

    pub fn get_effective_first_argument_for_jsx_signature(
        &mut self,
        _signature: &Arc<super::types::Signature>,
        _node: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_props_type_from_call_signature(
        &mut self,
        _sig: &Arc<super::types::Signature>,
        _context: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_props_type_from_class_type(
        &mut self,
        _sig: &Arc<super::types::Signature>,
        _context: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_props_type_for_signature_from_member(
        &mut self,
        _sig: &Arc<super::types::Signature>,
        _forced_lookup_location: &str,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_managed_attributes_from_located_attributes(
        &mut self,
        _context: &Arc<Node>,
        _ns: &Arc<crate::ast::Symbol>,
        _attributes_type: &Arc<super::types::Type>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn instantiate_alias_or_interface_with_defaults(
        &mut self,
        _managed_sym: &Arc<crate::ast::Symbol>,
        _type_arguments: &[Arc<super::types::Type>],
        _in_java_script: bool,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_library_managed_attributes(
        &self,
        _jsx_namespace: &Arc<crate::ast::Symbol>,
    ) -> Option<Arc<crate::ast::Symbol>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_element_type_symbol(
        &self,
        _jsx_namespace: &Arc<crate::ast::Symbol>,
    ) -> Option<Arc<crate::ast::Symbol>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_element_properties_name(
        &self,
        _jsx_namespace: &Arc<crate::ast::Symbol>,
    ) -> Option<String> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_element_children_property_name(
        &self,
        _jsx_namespace: &Arc<crate::ast::Symbol>,
    ) -> Option<String> {
        // TODO: full implementation
        None
    }

    pub fn get_name_from_jsx_element_attributes_container(
        &self,
        _name_of_attrib_prop_container: &str,
        _jsx_namespace: &Arc<crate::ast::Symbol>,
    ) -> Option<String> {
        // TODO: full implementation
        None
    }

    pub fn get_static_type_of_referenced_jsx_constructor(
        &mut self,
        _context: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_intrinsic_attributes_type_from_string_literal_type(
        &mut self,
        _t: &Arc<super::types::Type>,
        _location: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_reference_kind(&self, _node: &Arc<Node>) -> JsxReferenceKind {
        // TODO: full implementation
        JsxReferenceKind::Function
    }

    pub fn create_signature_for_jsx_intrinsic(
        &mut self,
        _node: &Arc<Node>,
        _result: &Arc<super::types::Type>,
    ) -> Option<Arc<super::types::Signature>> {
        // TODO: full implementation
        None
    }

    pub fn get_intrinsic_attributes_type_from_jsx_opening_like_element(
        &mut self,
        _node: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_intrinsic_tag_symbol(&self, _node: &Arc<Node>) -> Option<Arc<crate::ast::Symbol>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_stateless_element_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_element_class_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_element_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_element_type_type_at(
        &mut self,
        _location: &Arc<Node>,
    ) -> Option<Arc<super::types::Type>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_namespace_str(&self, _location: &Arc<Node>) -> String {
        // TODO: full implementation with location-based resolution
        "jsx".to_string()
    }

    pub fn get_local_jsx_namespace(&self, _file: &Arc<crate::ast::SourceFile>) -> String {
        // TODO: full implementation
        "jsx".to_string()
    }

    pub fn get_jsx_factory_entity(&self, _location: &Arc<Node>) -> Option<Arc<Node>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_fragment_factory_entity(&self, _location: &Arc<Node>) -> Option<Arc<Node>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_namespace_container_for_implicit_import(
        &self,
        _location: &Arc<Node>,
    ) -> Option<Arc<crate::ast::Symbol>> {
        // TODO: full implementation
        None
    }

    pub fn get_jsx_runtime_import_specifier(
        &self,
        _file: &Arc<crate::ast::SourceFile>,
    ) -> (String, Option<Arc<Node>>) {
        // TODO: full implementation
        (String::new(), None)
    }
}

/// Parse an isolated entity name from a string.
///
/// Mirrors Go's `parseIsolatedEntityName`.
pub fn parse_isolated_entity_name(_name: &str) -> Option<Arc<Node>> {
    // TODO: full implementation with parser
    None
}

/// Mark a node as synthetic.
///
/// Mirrors Go's `markAsSynthetic`.
pub fn mark_as_synthetic(node: &Arc<Node>) -> bool {
    // TODO: set node flags to synthesized
    let _ = node;
    false
}
