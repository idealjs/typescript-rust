//! Import tracker (1:1 port of Go's `internal/ls/importTracker.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol};
use crate::checker::Checker;
use crate::compiler::Program;

/// Whether an import-export symbol is an import or export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpExpKind {
    Unknown,
    Import,
    Export,
}

/// An import-export symbol.
pub struct ImportExportSymbol {
    pub kind: ImpExpKind,
    pub symbol: Arc<Symbol>,
}

/// The kind of export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Named,
    Default,
    ExportEquals,
    Umd,
    Module,
}

/// Information about an export.
pub struct ExportInfo {
    pub exporting_module_symbol: Option<Arc<Symbol>>,
    pub export_kind: ExportKind,
}

/// A location and symbol pair for an import search.
pub struct LocationAndSymbol {
    pub import_location: Option<Arc<Node>>,
    pub import_symbol: Option<Arc<Symbol>>,
}

/// Result of an import search.
pub struct ImportsResult {
    pub import_searches: Vec<LocationAndSymbol>,
    pub single_references: Vec<Arc<Node>>,
    pub indirect_users: Vec<Arc<SourceFile>>,
}

/// Function type for tracking imports of an exported symbol.
pub type ImportTracker =
    Box<dyn Fn(&Arc<Symbol>, &ExportInfo, bool) -> ImportsResult + Send + Sync>;

/// Module reference kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleReferenceKind {
    Import,
    Reference,
    Implicit,
}

/// A module reference.
pub struct ModuleReference {
    pub kind: ModuleReferenceKind,
    pub literal: Option<Arc<Node>>,
    pub referencing_file: Option<Arc<SourceFile>>,
}

/// Create an import tracker lazily.
///
/// Mirrors `createImportTracker`.
pub fn create_import_tracker(
    _program: &Program,
    _source_files: &[Arc<SourceFile>],
    _source_files_set: &std::collections::HashSet<String>,
    _checker: &Checker,
) -> ImportTracker {
    // TODO: requires getDirectImportsMap
    Box::new(
        |_export_symbol: &Arc<Symbol>, _export_info: &ExportInfo, _is_for_rename: bool| {
            ImportsResult {
                import_searches: Vec::new(),
                single_references: Vec::new(),
                indirect_users: Vec::new(),
            }
        },
    )
}
