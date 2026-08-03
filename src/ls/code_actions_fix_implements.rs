//! Fix class incorrectly implements interface code action
//! (1:1 port of Go's `internal/ls/codeactions_fixclassincorrectlyimplementsinterface.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Node;
use crate::checker::Checker;

use super::code_actions::{CodeFixContext, CodeFixProvider};
use super::language_service::LanguageService;
use super::types::CodeAction;

/// Fix ID for class incorrectly implements interface.
pub const FIX_CLASS_INCORRECTLY_IMPLEMENTS_INTERFACE_FIX_ID: &str =
    "fixClassIncorrectlyImplementsInterface";

/// The `FixClassIncorrectlyImplementsInterfaceProvider`.
pub fn fix_class_incorrectly_implements_interface_provider() -> CodeFixProvider {
    CodeFixProvider {
        error_codes: Vec::new(), // TODO: populate with diagnostic codes
        fix_ids: vec![FIX_CLASS_INCORRECTLY_IMPLEMENTS_INTERFACE_FIX_ID.to_string()],
    }
}

impl LanguageService {
    /// Get code actions to fix a class that incorrectly implements an interface.
    ///
    /// Mirrors `getCodeActionsToFixClassIncorrectlyImplementsInterface`.
    pub fn get_code_actions_to_fix_class_incorrectly_implements_interface(
        &self,
        _context: &CodeFixContext,
    ) -> Vec<CodeAction> {
        // TODO: requires change.Tracker + missing-member fixer
        Vec::new()
    }

    /// Get all code actions (fix-all) for class incorrectly implements interface.
    ///
    /// Mirrors `getAllCodeActionsToFixClassIncorrectlyImplementsInterface`.
    pub fn get_all_code_actions_to_fix_class_incorrectly_implements_interface(
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

/// Add changes for implementing missing members.
///
/// Mirrors `addChanges`.
pub fn add_changes(
    _change_tracker: &crate::ls::change::Tracker,
    _type_checker: &Checker,
    _class_declaration: &Arc<Node>,
    _implemented_type_node: &Arc<Node>,
) {
    // TODO: requires missing-member fixer + change tracker
}

/// Get the changes from a change tracker.
///
/// Mirrors `getChanges`.
pub fn get_changes(
    _change_tracker: &crate::ls::change::Tracker,
    _file: &Arc<crate::ast::SourceFile>,
) -> Vec<crate::lsp::lsproto::lsp::TextEdit> {
    // TODO: requires change tracker
    Vec::new()
}

/// Create an import adder for the fix context.
///
/// Mirrors `createImportAdder`.
pub fn create_import_adder(
    _context: &CodeFixContext,
    _type_checker: &Checker,
) -> Result<(), String> {
    // TODO: requires autoimport.ImportAdder
    Ok(())
}
