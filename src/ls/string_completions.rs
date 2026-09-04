//! String-literal completions (1:1 port of Go's `internal/ls/string_completions.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol};
use crate::checker::Checker;
use crate::core::text::TextRange;

use super::language_service::LanguageService;

/// Completions derived from type string-literal members.
pub struct CompletionsFromTypes {
    pub is_new_identifier: bool,
}

/// Completions derived from property symbols.
pub struct CompletionsFromProperties {
    pub symbols: Vec<Arc<Symbol>>,
    pub has_index_signature: bool,
}

/// A path-based completion entry.
pub struct PathCompletion {
    pub name: String,
    pub extension: String,
    pub text_range: TextRange,
}

/// Aggregated string-literal completion results.
pub struct StringLiteralCompletions {
    pub from_types: Option<CompletionsFromTypes>,
    pub from_properties: Option<CompletionsFromProperties>,
    pub from_paths: Vec<PathCompletion>,
}

impl LanguageService {
    /// Get string-literal completions for a position.
    ///
    /// Mirrors `getStringLiteralCompletions`.
    pub fn get_string_literal_completions(
        &self,
        _file: &Arc<SourceFile>,
        _position: usize,
        _context_token: Option<&Arc<Node>>,
        _checker: &Checker,
    ) -> Option<StringLiteralCompletions> {
        // TODO: requires checker type analysis + path enumeration
        None
    }
}
