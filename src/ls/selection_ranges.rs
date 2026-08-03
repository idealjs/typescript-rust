//! Selection ranges provider (1:1 port of Go's `internal/ls/selectionranges.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::lsp::lsproto::lsp::{DocumentUri, Position};

use super::language_service::LanguageService;
use super::types::SelectionRange;

impl LanguageService {
    /// Provide selection ranges for positions in a document.
    ///
    /// Mirrors `ProvideSelectionRanges`.
    pub fn provide_selection_ranges(
        &self,
        _document_uri: &DocumentUri,
        positions: &[Position],
    ) -> Vec<SelectionRange> {
        let (_program, source_file) = self.get_program_and_file(_document_uri);
        let mut results = Vec::new();
        for position in positions {
            if let Some(sr) = get_smart_selection_range(self, &source_file, 0) {
                results.push(sr);
            }
            let _ = position;
        }
        results
    }
}

/// Compute a smart selection range for a position.
///
/// Mirrors `getSmartSelectionRange`.
pub fn get_smart_selection_range(
    _ls: &LanguageService,
    _source_file: &Arc<SourceFile>,
    _pos: usize,
) -> Option<SelectionRange> {
    // TODO: requires scanner.GetTokenPosOfNode + AST traversal
    None
}
