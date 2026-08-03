//! Linked editing ranges (1:1 port of Go's `internal/ls/linkedediting.go`).

#![allow(dead_code)]

use crate::lsp::lsproto::lsp::DocumentUri;

use super::language_service::LanguageService;
use super::types::{LinkedEditingRangeParams, LinkedEditingRanges};

/// JSX tag word pattern for linked editing.
pub const JSX_TAG_WORD_PATTERN: &str = "[a-zA-Z0-9:\\-\\._$]*";

impl LanguageService {
    /// Provide linked editing ranges for a position.
    ///
    /// Mirrors `ProvideLinkedEditingRange`.
    pub fn provide_linked_editing_range(
        &self,
        _params: &LinkedEditingRangeParams,
    ) -> Option<LinkedEditingRanges> {
        // TODO: requires astnav.FindPrecedingToken + JSX tag matching
        None
    }
}
