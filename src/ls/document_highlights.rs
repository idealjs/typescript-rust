//! Document highlights provider (1:1 port of Go's `internal/ls/documenthighlights.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position};

use super::language_service::LanguageService;
use super::types::{DocumentHighlight, DocumentHighlightKind, MultiDocumentHighlight};

impl LanguageService {
    /// Provide document highlights for a position in a document.
    ///
    /// Mirrors `ProvideDocumentHighlights`.
    pub fn provide_document_highlights(
        &self,
        _document_uri: &DocumentUri,
        _document_position: Position,
    ) -> Vec<DocumentHighlight> {
        // TODO: requires astnav, checker, reference entry logic
        Vec::new()
    }

    /// Provide multi-document highlights.
    ///
    /// Mirrors `ProvideMultiDocumentHighlights`.
    pub fn provide_multi_document_highlights(
        &self,
        _document_uri: &DocumentUri,
        _document_position: Position,
        _files_to_search: &[DocumentUri],
    ) -> Vec<MultiDocumentHighlight> {
        // TODO: requires full reference-entry machinery
        Vec::new()
    }

    /// Get semantic document highlights.
    ///
    /// Mirrors `getSemanticDocumentHighlights`.
    pub fn get_semantic_document_highlights(
        &self,
        _position: usize,
        _node: &Arc<Node>,
        _program: &Program,
        _source_files: &[Arc<SourceFile>],
    ) -> Vec<MultiDocumentHighlight> {
        // TODO: requires getReferencedSymbolsForNode
        Vec::new()
    }

    /// Get syntactic document highlights (keyword-based).
    ///
    /// Mirrors `getSyntacticDocumentHighlights`.
    pub fn get_syntactic_document_highlights(
        &self,
        _node: &Arc<Node>,
        _source_file: &Arc<SourceFile>,
    ) -> Vec<DocumentHighlight> {
        // TODO: requires keyword-based occurrence logic
        Vec::new()
    }
}
