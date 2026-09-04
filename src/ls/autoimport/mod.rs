//! Auto-import (1:1 port of Go's `internal/ls/autoimport/`).
//!
//! This module provides the auto-import registry, export extraction, indexing,
//! module specifier generation, and code-fix logic for automatically importing
//! symbols that are used but not yet imported.

#![allow(dead_code)]

pub mod alias_resolver;
pub mod export;
pub mod extract;
pub mod fix;
pub mod import_adder;
pub mod index;
pub mod registry;
pub mod specifiers;
pub mod util;
pub mod view;

// ============================================================================
// Stub types for dependencies not yet ported from other modules.
// These mirror the Go type shapes so the auto-import code compiles 1:1.
// ============================================================================

use std::collections::HashMap;

use crate::collections::set::Set;
use crate::tspath;

// --- lsproto stubs (AutoImportFix and related enums) ---

/// The kind of auto-import code fix.
///
/// Mirrors `lsproto.AutoImportFixKind` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AutoImportFixKind {
    UseNamespace,
    JsdocTypeImport,
    AddToExisting,
    AddNew,
    PromoteTypeOnly,
}

impl Default for AutoImportFixKind {
    fn default() -> Self {
        AutoImportFixKind::AddNew
    }
}

/// The kind of import binding.
///
/// Mirrors `lsproto.ImportKind` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportKind {
    Default,
    Named,
    CommonJS,
    Namespace,
}

impl Default for ImportKind {
    fn default() -> Self {
        ImportKind::Named
    }
}

/// Whether a type-only import is allowed, required, or not allowed.
///
/// Mirrors `lsproto.AddAsTypeOnly` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AddAsTypeOnly {
    Allowed,
    NotAllowed,
    Required,
}

impl Default for AddAsTypeOnly {
    fn default() -> Self {
        AddAsTypeOnly::Allowed
    }
}

/// The auto-import fix data (mirrors `lsproto.AutoImportFix` in Go).
#[derive(Debug, Clone, Default)]
pub struct AutoImportFix {
    pub kind: AutoImportFixKind,
    pub import_kind: ImportKind,
    pub module_specifier: String,
    pub name: String,
    pub use_require: bool,
    pub add_as_type_only: AddAsTypeOnly,
    pub import_index: i32,
    pub usage_position: Option<crate::lsp::lsproto::Position>,
    pub namespace_prefix: String,
}

// --- module stubs ---

/// A resolved entrypoint of a package.
///
/// Mirrors `module.ResolvedEntrypoint` in Go (stub — full type is in the
/// not-yet-ported resolver).
#[derive(Debug, Clone, Default)]
pub struct ResolvedEntrypoint {
    pub resolved_file_name: String,
    pub symlink_or_realpath: String,
    pub include_conditions: Set<String>,
    pub exclude_conditions: Set<String>,
}

impl ResolvedEntrypoint {
    pub fn symlink_or_realpath(&self) -> &str {
        &self.symlink_or_realpath
    }
}

/// Options for constructing a module resolver.
///
/// Mirrors `module.ResolverOptions` in Go (stub).
#[derive(Debug, Clone, Default)]
pub struct ResolverOptions;

/// A cache key combining module name and resolution mode.
///
/// Mirrors `module.ModeAwareCacheKey` in Go (stub).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeAwareCacheKey {
    pub name: String,
    pub mode: crate::core::compiler_options::ResolutionMode,
}

/// A mode-aware cache mapping `ModeAwareCacheKey` to a value.
///
/// Mirrors `module.ModeAwareCache[T]` in Go (stub).
#[derive(Debug, Clone, Default)]
pub struct ModeAwareCache<T: Clone> {
    pub map: HashMap<ModeAwareCacheKey, T>,
}

// --- project dirty/logging stubs ---

/// A dirty-tracking map (stub for `dirty.Map[K,V]`).
#[derive(Debug, Clone, Default)]
pub struct DirtyMap<K: std::hash::Hash + Eq + Clone, V: Clone> {
    pub entries: HashMap<K, V>,
}

/// An entry in a dirty map (stub for `dirty.MapEntry[K,V]`).
#[derive(Debug)]
pub struct DirtyMapEntry<K: Clone, V: Clone> {
    pub key: K,
    pub value: V,
}

/// A dirty-tracking map builder (stub for `dirty.MapBuilder`).
#[derive(Debug, Clone, Default)]
pub struct DirtyMapBuilder<K: std::hash::Hash + Eq + Clone, V: Clone, B: Clone> {
    pub entries: HashMap<K, V>,
    _builder: std::marker::PhantomData<B>,
}

/// A logging tree (stub for `logging.LogTree`).
#[derive(Debug, Clone, Default)]
pub struct LogTree;

impl LogTree {
    pub fn fork(&self, _label: &str) -> LogTree {
        LogTree
    }
    pub fn logf(&self, _args: std::fmt::Arguments<'_>) {}
}

/// A logger trait (stub for `logging.Logger`).
pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

// --- vfsmatch stubs ---

/// A spec matcher for VFS path matching (stub for `vfsmatch.SpecMatcher`).
#[derive(Debug, Clone, Default)]
pub struct SpecMatcher;

impl SpecMatcher {
    pub fn match_string(&self, _path: &str) -> bool {
        false
    }
}

// --- Internal symbol name constants ---
// These mirror Go's `ast.InternalSymbolName*` constants.

pub const INTERNAL_SYMBOL_NAME_EXPORT_EQUALS: &str = "=export";
pub const INTERNAL_SYMBOL_NAME_DEFAULT: &str = "default";
pub const INTERNAL_SYMBOL_NAME_EXPORT_STAR: &str = "*";

// --- RegistryCloneHost trait ---

/// Host interface for cloning the auto-import registry.
///
/// Mirrors `autoimport.RegistryCloneHost` in Go.
pub trait RegistryCloneHost: Send + Sync {
    fn fs(&self) -> &dyn crate::vfs::FS;
    fn get_current_directory(&self) -> &str;
    fn get_default_project(
        &self,
        path: &tspath::Path,
    ) -> (
        tspath::Path,
        Option<std::sync::Arc<crate::compiler::Program>>,
    );
    fn get_program_for_project(
        &self,
        project_path: &tspath::Path,
    ) -> Option<std::sync::Arc<crate::compiler::Program>>;
    fn get_source_file(
        &self,
        file_name: &str,
        path: &tspath::Path,
    ) -> Option<std::sync::Arc<crate::ast::SourceFile>>;
    fn dispose(&self);
}

// --- Module specifier stubs ---

/// A module specifier ending preference.
///
/// Mirrors `modulespecifiers.ModuleSpecifierEnding` in Go (stub).
pub type ModuleSpecifierEnding = String;

// --- Shared forward declarations ---

/// Re-export of `export::Export` for convenience.
pub use export::{Export, ExportID, ExportSyntax, ModuleID};
