//! Rename provider (1:1 port of Go's `internal/ls/rename.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::checker::Checker;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range, TextEdit};

use super::cross_project::CrossProjectOrchestrator;
use super::find_all_references::{ReferenceEntry, SymbolAndEntriesData};
use super::language_service::LanguageService;
use super::types::{RenameParams, WorkspaceEdit};

/// Rename info (result of prepareRename validation).
#[derive(Debug, Clone, Default)]
pub struct RenameInfo {
    pub can_rename: bool,
    pub localized_error_message: String,
    pub display_name: String,
    pub trigger_span: Range,
    pub file_to_rename: String,
    pub new_file_name: String,
}

impl LanguageService {
    /// Provide a rename workspace edit.
    ///
    /// Mirrors `ProvideRename`.
    pub fn provide_rename(
        &self,
        _params: &RenameParams,
        _orchestrator: Option<&dyn CrossProjectOrchestrator>,
    ) -> Option<WorkspaceEdit> {
        // TODO: requires handleCrossProject + symbolAndEntriesToRename
        None
    }

    /// Get rename info (prepareRename).
    ///
    /// Mirrors `GetRenameInfo`.
    pub fn get_rename_info(
        &self,
        _new_name: &str,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> RenameInfo {
        // TODO: requires astnav + eligibility checks
        RenameInfo::default()
    }

    /// Convert symbol-and-entries to a rename workspace edit.
    ///
    /// Mirrors `symbolAndEntriesToRename`.
    pub fn symbol_and_entries_to_rename(
        &self,
        _data: &SymbolAndEntriesData,
        _new_name: &str,
    ) -> Option<WorkspaceEdit> {
        // TODO: requires text-edit generation + alias handling
        None
    }

    /// Get the text for a rename edit.
    ///
    /// Mirrors `getTextForRename`.
    pub fn get_text_for_rename(
        &self,
        _original_node: &Arc<Node>,
        _entry: &ReferenceEntry,
        _new_name: &str,
        _checker: &Checker,
    ) -> String {
        // TODO: requires checker rename validation
        String::new()
    }
}

/// Check if a node is eligible for rename.
///
/// Mirrors `nodeIsEligibleForRename`.
pub fn node_is_eligible_for_rename(_node: &Arc<Node>) -> bool {
    // TODO: requires AST kind + context checks
    false
}

/// Get rename info for a specific node.
///
/// Mirrors `getRenameInfoForNode`.
pub fn get_rename_info_for_node(
    _ls: &LanguageService,
    _new_name: &str,
    _node: &Arc<Node>,
    _source_file: &Arc<SourceFile>,
    _program: &Program,
) -> Option<RenameInfo> {
    // TODO: requires checker + identifier validation
    None
}

/// Get the adjusted location for rename.
///
/// Mirrors `getAdjustedLocation`.
pub fn get_adjusted_location(
    _node: &Arc<Node>,
    _for_rename: bool,
    _file: &Arc<SourceFile>,
) -> Arc<Node> {
    // TODO: requires AST adjustment logic
    _node.clone()
}
