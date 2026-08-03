//! Code actions (1:1 port of Go's `internal/ls/codeactions.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::compiler::Program;
use crate::core::text::TextRange;
use crate::lsp::lsproto::lsp::TextEdit;

use super::language_service::LanguageService;
use super::types::{CodeAction, CodeActionParams, Diagnostic};

/// A provider for a specific type of code fix.
pub struct CodeFixProvider {
    pub error_codes: Vec<i32>,
    pub fix_ids: Vec<String>,
}

/// Context for generating code fixes.
pub struct CodeFixContext<'a> {
    pub source_file: &'a Arc<SourceFile>,
    pub span: TextRange,
    pub error_code: i32,
    pub program: &'a Program,
    pub ls: &'a LanguageService,
    pub diagnostic: Option<&'a Diagnostic>,
    pub params: Option<&'a CodeActionParams>,
}

/// Combined code actions for fix-all scenarios.
pub struct CombinedCodeActions {
    pub description: String,
    pub changes: Vec<TextEdit>,
}

impl LanguageService {
    /// Provide code actions for a range and context.
    ///
    /// Mirrors `ProvideCodeActions`.
    pub fn provide_code_actions(&self, _params: &CodeActionParams) -> Vec<CodeAction> {
        // TODO: requires diagnostic → code-fix provider dispatch
        Vec::new()
    }
}

/// Registered code-fix providers (stubs referenced by codeactions_*.rs).
pub fn registered_code_fix_providers() -> Vec<&'static str> {
    vec![
        "ImportFixProvider",
        "IsolatedDeclarationsFixProvider",
        "FixClassIncorrectlyImplementsInterfaceProvider",
    ]
}
