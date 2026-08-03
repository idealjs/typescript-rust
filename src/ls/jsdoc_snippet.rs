//! JSDoc snippet completions (1:1 port of Go's `internal/ls/jsdoc_snippet.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};

use super::language_service::LanguageService;
use super::types::CompletionItem;

/// A doc-comment template result.
pub struct DocCommentTemplate {
    pub new_text: String,
}

/// Information about the owner of a comment.
pub struct CommentOwnerInfo {
    pub comment_owner: Option<Arc<Node>>,
    pub has_return: bool,
}

impl LanguageService {
    /// Get JSDoc snippet completion for a position.
    ///
    /// Mirrors `getJSDocSnippetCompletion`.
    pub fn get_jsdoc_snippet_completion(
        &self,
        _file: &Arc<SourceFile>,
        _position: usize,
    ) -> Option<CompletionItem> {
        // TODO: requires parser/scanner position analysis
        None
    }
}

/// Check if a position is a potentially valid JSDoc snippet completion position.
///
/// Mirrors `isPotentiallyValidJSDocSnippetCompletionPosition`.
pub fn is_potentially_valid_jsdoc_snippet_completion_position(
    _file: &Arc<SourceFile>,
    _position: usize,
) -> bool {
    // TODO: requires scanner position analysis
    false
}

/// Get the doc-comment template at a position.
///
/// Mirrors `getDocCommentTemplateAtPosition`.
pub fn get_doc_comment_template_at_position(
    _file: &Arc<SourceFile>,
    _position: usize,
    _generate_return: bool,
    _new_line: &str,
) -> Option<DocCommentTemplate> {
    // TODO: requires scanner + AST analysis
    None
}

/// Convert a template string to a snippet.
///
/// Mirrors `templateToSnippet`.
pub fn template_to_snippet(template: &str, _new_line: &str) -> String {
    // TODO: requires snippet conversion (replacing placeholders)
    template.to_string()
}
