//! Auto-insert (VS-specific) (1:1 port of Go's `internal/ls/autoinsert.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};

use super::language_service::LanguageService;
use super::types::{TextDocumentIdentifier, VsOnAutoInsertParams};

/// VS OnAutoInsert response item.
#[derive(Debug, Clone, Default)]
pub struct VsOnAutoInsertResponseItem {
    pub text_edit_format: u32,
    pub text_edit: crate::lsp::lsproto::lsp::TextEdit,
}

impl LanguageService {
    /// Handle VS-specific on-auto-insert.
    ///
    /// Mirrors `ProvideOnAutoInsert`.
    pub fn provide_on_auto_insert(
        &self,
        _params: &VsOnAutoInsertParams,
    ) -> Option<VsOnAutoInsertResponseItem> {
        // TODO: requires astnav + JSX closing-tag logic
        None
    }
}

/// Check if a JSX element has an unclosed tag.
///
/// Mirrors `isUnclosedTag`.
pub fn is_unclosed_tag(_element: &Arc<Node>) -> bool {
    // TODO: requires JSX AST analysis
    false
}

/// Check if a JSX fragment is unclosed.
///
/// Mirrors `isUnclosedFragment`.
pub fn is_unclosed_fragment(_fragment: &Arc<Node>) -> bool {
    // TODO: requires JSX AST analysis
    false
}
