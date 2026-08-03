//! Hover / quick-info provider (1:1 port of Go's `internal/ls/hover.go`).
//!
//! Provides `ProvideHover` and the supporting display-parts logic for building
//! hover content. Depends on checker, nodebuilder, printer, scanner which are
//! not fully wired; method bodies are stubbed.

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol};
use crate::checker::Checker;
use crate::ls::display_parts_writer::{DisplayPartsWriter, new_display_parts_writer};
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range};

use super::language_service::LanguageService;
use super::types::Hover;

/// Symbol format flags used by hover.
pub const SYMBOL_FORMAT_FLAGS: u32 = 0;

/// Type format flags used by hover.
pub const TYPE_FORMAT_FLAGS: u32 = 0;

/// Holds the result of `get_quick_info_and_declaration_at_location`.
pub struct SymbolDisplayInfo {
    pub display_parts: DisplayPartsWriter,
    pub declaration: Option<Arc<Node>>,
}

impl LanguageService {
    /// Provide hover information for a position.
    ///
    /// Mirrors `ProvideHover`.
    pub fn provide_hover(&self, _document_uri: &DocumentUri, _position: Position) -> Option<Hover> {
        // TODO: requires astnav, checker, nodebuilder, printer
        None
    }

    /// Get quickInfo and documentation for a symbol.
    ///
    /// Mirrors `getQuickInfoAndDocumentationForSymbol`.
    pub fn get_quick_info_and_documentation_for_symbol(
        &self,
        _checker: &Checker,
        _symbol: Option<&Arc<Symbol>>,
        _node: &Arc<Node>,
        _content_format: &str,
    ) -> (String, String) {
        // TODO: requires checker display-part logic
        (String::new(), String::new())
    }

    /// Get documentation from a declaration.
    ///
    /// Mirrors `getDocumentationFromDeclaration`.
    pub fn get_documentation_from_declaration(
        &self,
        _checker: &Checker,
        _symbol: Option<&Arc<Symbol>>,
        _declaration: &Arc<Node>,
        _location: &Arc<Node>,
        _content_format: &str,
        _comment_only: bool,
    ) -> String {
        // TODO: requires JSDoc traversal
        String::new()
    }

    /// Get the LSP range of a node.
    ///
    /// Mirrors `getLspRangeOfNode`.
    pub fn get_lsp_range_of_node(
        &self,
        _node: &Arc<Node>,
        _file: Option<&Arc<SourceFile>>,
        _context_node: Option<&Arc<Node>>,
    ) -> Range {
        // TODO: requires scanner.GetTokenPosOfNode + converters
        Range::default()
    }
}

/// Format quickInfo text as a markdown code block.
pub fn format_quick_info(quick_info: &str) -> String {
    if quick_info.is_empty() {
        return String::new();
    }
    write_code("typescript", quick_info)
}

/// Write a fenced code block.
pub fn write_code(lang: &str, code: &str) -> String {
    if code.is_empty() {
        return String::new();
    }
    let mut ticks = 3;
    let tick_str = |n: usize| "`".repeat(n);
    while code.contains(&tick_str(ticks)) {
        ticks += 1;
    }
    let mut result = tick_str(ticks);
    result.push_str(lang);
    result.push('\n');
    result.push_str(code);
    result.push('\n');
    result.push_str(&tick_str(ticks));
    result.push('\n');
    result
}

/// Get the quickInfo and declaration at a location.
///
/// Mirrors `getQuickInfoAndDeclarationAtLocation`.
pub fn get_quick_info_and_declaration_at_location(
    _checker: &Checker,
    _symbol: Option<&Arc<Symbol>>,
    _node: &Arc<Node>,
) -> SymbolDisplayInfo {
    // TODO: requires full checker type/symbol formatting
    SymbolDisplayInfo {
        display_parts: crate::ls::display_parts_writer::new_display_parts_writer(false),
        declaration: None,
    }
}
