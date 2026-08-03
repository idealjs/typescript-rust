//! Organize imports provider (1:1 port of Go's `internal/ls/organizeimports.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::TextEdit;

use super::language_service::LanguageService;

impl LanguageService {
    /// Organize imports: remove unused, coalesce, and sort.
    ///
    /// Mirrors `OrganizeImports`.
    pub fn organize_imports(
        &self,
        _source_file: &Arc<SourceFile>,
        _program: &Program,
        _kind: &str,
    ) -> std::collections::HashMap<String, Vec<TextEdit>> {
        // TODO: requires change.Tracker + lsutil import grouping
        std::collections::HashMap::new()
    }
}

/// Group contiguous import declarations by newline gaps.
///
/// Mirrors `groupByNewlineContiguous`.
pub fn group_by_newline_contiguous(
    _source_file: &Arc<SourceFile>,
    _imports: &[Arc<crate::ast::Node>],
) -> Vec<Vec<Arc<crate::ast::Node>>> {
    // TODO: requires AST position / newline analysis
    Vec::new()
}
