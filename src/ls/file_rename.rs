//! File rename edits (1:1 port of Go's `internal/ls/file_rename.go`).

#![allow(dead_code)]

use crate::compiler::Program;
use crate::lsp::lsproto::lsp::DocumentUri;

use super::language_service::LanguageService;
use super::types::RenameFile;

/// A path updater function.
pub type PathUpdater = Box<dyn Fn(&str) -> (String, bool)>;

/// An import-update target.
pub struct ToImport {
    pub new_file_name: String,
    pub updated: bool,
}

impl LanguageService {
    /// Get edits for a file rename (update imports and tsconfig).
    ///
    /// Mirrors `GetEditsForFileRename`.
    pub fn get_edits_for_file_rename(
        &self,
        _old_uri: &DocumentUri,
        _new_uri: &DocumentUri,
    ) -> Vec<RenameFile> {
        // TODO: requires change.Tracker + module specifier updates
        Vec::new()
    }

    /// Create a path updater for old → new file path.
    ///
    /// Mirrors `createPathUpdater`.
    pub fn create_path_updater(&self, _old_path: &str, _new_path: &str) -> PathUpdater {
        // TODO: requires tspath comparison
        Box::new(|path: &str| (path.to_string(), false))
    }

    /// Update tsconfig files for a file rename.
    pub fn update_tsconfig_files(
        &self,
        _program: &Program,
        _old_to_new: &PathUpdater,
        _old_path: &str,
        _new_path: &str,
    ) {
        // TODO: requires tsconfig parsing + change tracker
    }

    /// Update imports for a file rename.
    pub fn update_imports_for_file_rename(&self, _program: &Program, _old_to_new: &PathUpdater) {
        // TODO: requires module specifier analysis + change tracker
    }
}
