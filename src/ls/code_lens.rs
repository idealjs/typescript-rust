//! Code lens provider (1:1 port of Go's `internal/ls/codelens.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Node;
use crate::lsp::lsproto::lsp::DocumentUri;

use super::cross_project::CrossProjectOrchestrator;
use super::language_service::LanguageService;
use super::types::CodeLens;

impl LanguageService {
    /// Provide code lenses for a document.
    ///
    /// Mirrors `ProvideCodeLenses`.
    pub fn provide_code_lenses(&self, _document_uri: &DocumentUri) -> Vec<CodeLens> {
        // TODO: requires AST traversal + checker
        Vec::new()
    }

    /// Resolve a code lens.
    ///
    /// Mirrors `ResolveCodeLens`.
    pub fn resolve_code_lens(
        &self,
        _code_lens: &CodeLens,
        _show_locations_command_name: Option<&str>,
        _orchestrator: Option<&dyn CrossProjectOrchestrator>,
    ) -> CodeLens {
        // TODO: requires reference-count resolution
        CodeLens::default()
    }

    /// Create a code lens for a node.
    ///
    /// Mirrors `newCodeLensForNode`.
    pub fn new_code_lens_for_node(
        &self,
        _document_uri: &DocumentUri,
        _file: &Arc<crate::ast::SourceFile>,
        _node: &Arc<Node>,
        _kind: &str,
    ) -> CodeLens {
        // TODO: requires createLspRangeFromNode
        CodeLens::default()
    }
}

/// Check if a node is valid for reference code lens.
///
/// Mirrors `isValidReferenceLensNode`.
pub fn is_valid_reference_lens_node(_node: &Arc<Node>) -> bool {
    // TODO: requires AST kind checks
    false
}

/// Check if a node is valid for implementations code lens.
///
/// Mirrors `isValidImplementationsCodeLensNode`.
pub fn is_valid_implementations_code_lens_node(_node: &Arc<Node>) -> bool {
    // TODO: requires AST kind checks
    false
}
