//! Diagnostics provider (1:1 port of Go's `internal/ls/diagnostics.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Diagnostic, SourceFile};
use crate::compiler;
use crate::lsp::lsproto::lsp::DocumentUri;

use super::language_service::LanguageService;
use super::types::Diagnostic as LspDiagnostic;

impl LanguageService {
    /// Provide diagnostics for a document.
    ///
    /// Mirrors `ProvideDiagnostics`.
    pub fn provide_diagnostics(&self, _document_uri: &DocumentUri) -> Vec<LspDiagnostic> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        // TODO: requires program.GetSyntacticDiagnostics etc.
        let _ = file;
        Vec::new()
    }

    /// Convert compiler diagnostics to LSP diagnostics.
    ///
    /// Mirrors `toLSPDiagnostics`.
    pub fn to_lsp_diagnostics(&self, _diagnostics: &[Vec<Arc<Diagnostic>>]) -> Vec<LspDiagnostic> {
        // TODO: requires lsconv::DiagnosticToLSPPull
        Vec::new()
    }
}

/// Collect all diagnostics for a file: syntactic, semantic, suggestion, and
/// (when declarations are emitted) declaration diagnostics.
///
/// Mirrors `getAllDiagnostics`.
pub fn get_all_diagnostics(
    _program: &compiler::Program,
    _file: &Arc<SourceFile>,
) -> Vec<Arc<Diagnostic>> {
    // TODO: requires program diagnostic methods
    Vec::new()
}
