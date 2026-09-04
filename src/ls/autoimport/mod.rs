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

use std::collections::HashMap;

use crate::collections::set::Set;
use crate::tspath;

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

#[derive(Debug, Clone, Default)]
pub struct ResolverOptions;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeAwareCacheKey {
    pub name: String,
    pub mode: crate::core::compiler_options::ResolutionMode,
}

#[derive(Debug, Clone, Default)]
pub struct ModeAwareCache<T: Clone> {
    pub map: HashMap<ModeAwareCacheKey, T>,
}

#[derive(Debug, Clone, Default)]
pub struct DirtyMap<K: std::hash::Hash + Eq + Clone, V: Clone> {
    pub entries: HashMap<K, V>,
}

#[derive(Debug)]
pub struct DirtyMapEntry<K: Clone, V: Clone> {
    pub key: K,
    pub value: V,
}

#[derive(Debug, Clone, Default)]
pub struct DirtyMapBuilder<K: std::hash::Hash + Eq + Clone, V: Clone, B: Clone> {
    pub entries: HashMap<K, V>,
    _builder: std::marker::PhantomData<B>,
}

#[derive(Debug, Clone, Default)]
pub struct LogTree;

impl LogTree {
    pub fn fork(&self, _label: &str) -> LogTree {
        LogTree
    }
    pub fn logf(&self, _args: std::fmt::Arguments<'_>) {}
}

pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

#[derive(Debug, Clone, Default)]
pub struct SpecMatcher;

impl SpecMatcher {
    pub fn match_string(&self, _path: &str) -> bool {
        false
    }
}

pub const INTERNAL_SYMBOL_NAME_EXPORT_EQUALS: &str = "=export";
pub const INTERNAL_SYMBOL_NAME_DEFAULT: &str = "default";
pub const INTERNAL_SYMBOL_NAME_EXPORT_STAR: &str = "*";

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

pub type ModuleSpecifierEnding = String;

pub use export::{Export, ExportID, ExportSyntax, ModuleID};
