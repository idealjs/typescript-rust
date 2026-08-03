//! Import fixes code action
//! (1:1 port of Go's `internal/ls/codeactions_importfixes.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, Symbol};
use crate::checker::Checker;

use super::code_actions::{CodeFixContext, CodeFixProvider};
use super::language_service::LanguageService;
use super::types::CodeAction;

/// Fix ID for missing import.
pub const IMPORT_FIX_ID: &str = "fixMissingImport";

/// The `ImportFixProvider`.
pub fn import_fix_provider() -> CodeFixProvider {
    CodeFixProvider {
        error_codes: Vec::new(), // TODO: populate with diagnostic codes
        fix_ids: vec![IMPORT_FIX_ID.to_string()],
    }
}

/// Info about an import fix candidate.
pub struct FixInfo {
    pub symbol_name: String,
    pub error_identifier_text: String,
}

impl LanguageService {
    /// Get import code actions for a diagnostic.
    ///
    /// Mirrors `getImportCodeActions`.
    pub fn get_import_code_actions(&self, _context: &CodeFixContext) -> Vec<CodeAction> {
        // TODO: requires autoimport.Registry lookup
        Vec::new()
    }

    /// Get all import code actions (fix-all).
    ///
    /// Mirrors `getAllImportCodeActions`.
    pub fn get_all_import_code_actions(
        &self,
        _context: &CodeFixContext,
    ) -> super::code_actions::CombinedCodeActions {
        // TODO: requires fix-all aggregation
        super::code_actions::CombinedCodeActions {
            description: String::new(),
            changes: Vec::new(),
        }
    }
}

/// Get the symbol name from an error identifier text.
///
/// Mirrors `getSymbolNameFromErrorText`.
pub fn get_symbol_name_from_error_text(text: &str) -> &str {
    text
}
