//! Completions provider (1:1 port of Go's `internal/ls/completions.go`).
//!
//! This is a large file (~6K lines in Go). This port includes the main types
//! (`CompletionItem`, `CompletionList`) and key entry points. Internal helpers
//! that depend on checker/printer/scanner/nodebuilder are stubbed.
//!
//! Mirrors Go's `internal/ls/completions.go`.

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol};
use crate::checker::Checker;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position};

use super::language_service::LanguageService;
use super::types::{
    CompletionContext, CompletionItem, CompletionItemApplyKinds, CompletionItemData,
    CompletionItemDefaults, CompletionList,
};

/// Error indicating that completions need auto-imports to be prepared.
pub const ERR_NEEDS_AUTO_IMPORTS: &str = "completion list needs auto imports";

/// Completion kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    None,
    Global,
    PropertyAccess,
    Member,
    String,
    Import,
    ObjectLiteralMember,
    JsDocTagName,
    JsDocTag,
    JsDocParameterName,
}

/// A completion data wrapper (holds symbols, auto-imports, and metadata).
pub struct CompletionDataData {
    pub symbols: Vec<Arc<Symbol>>,
    pub completion_kind: CompletionKind,
    pub is_in_snippet_scope: bool,
}

impl LanguageService {
    /// Provide completions for a position.
    ///
    /// Mirrors `ProvideCompletion`.
    pub fn provide_completion(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
        _context: &CompletionContext,
    ) -> CompletionList {
        // TODO: requires getCompletionsAtPosition
        CompletionList::default()
    }

    /// Get completions at a position.
    ///
    /// Mirrors `GetCompletionsAtPosition`.
    pub fn get_completions_at_position(
        &self,
        _file: &Arc<SourceFile>,
        _position: usize,
        _trigger_character: Option<&str>,
        _include_symbols: bool,
    ) -> Result<CompletionList, String> {
        // TODO: requires checker + scanner + nodebuilder
        Ok(CompletionList::default())
    }

    /// Resolve a completion item's details.
    ///
    /// Mirrors `GetCompletionEntryDetails`.
    pub fn get_completion_entry_details(
        &self,
        _file: &Arc<SourceFile>,
        _position: usize,
        _name: &str,
    ) -> Option<CompletionItem> {
        // TODO: requires checker symbol resolution
        None
    }
}

/// Ensure each item in a completion list has `data` populated.
///
/// Mirrors `ensureItemData`.
pub fn ensure_item_data(file_name: &str, pos: usize, mut list: CompletionList) -> CompletionList {
    for item in &mut list.items {
        if item.data.is_none() {
            item.data = Some(CompletionItemData {
                file_name: file_name.to_string(),
                position: pos as i32,
                name: item.label.clone(),
            });
        }
    }
    list
}
