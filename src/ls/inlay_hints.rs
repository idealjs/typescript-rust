//! Inlay hints provider (1:1 port of Go's `internal/ls/inlay_hints.go`).

#![allow(dead_code)]

use crate::core::text::TextRange;
use crate::ls::lsutil::{InlayHintsPreferences, QuotePreference};
use crate::lsp::lsproto::lsp::DocumentUri;

use super::language_service::LanguageService;
use super::types::InlayHint;

impl LanguageService {
    /// Provide inlay hints for a range in a document.
    ///
    /// Mirrors `ProvideInlayHint`.
    pub fn provide_inlay_hint(&self, _document_uri: &DocumentUri) -> Vec<InlayHint> {
        // TODO: requires checker + AST traversal with preferences
        Vec::new()
    }
}

/// State used during inlay hint collection.
pub struct InlayHintState<'a> {
    pub span: TextRange,
    pub preferences: &'a InlayHintsPreferences,
    pub quote_preference: QuotePreference,
    pub result: Vec<InlayHint>,
}

/// Check if any inlay hint is enabled.
pub fn is_any_inlay_hint_enabled(prefs: &InlayHintsPreferences) -> bool {
    // TODO: check all preference flags
    let _ = prefs;
    false
}
