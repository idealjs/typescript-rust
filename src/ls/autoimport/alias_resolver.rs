//! Alias resolution (1:1 port of Go's `internal/ls/autoimport/aliasresolver.go`).

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::SourceFile;
use crate::collections::syncmap::SyncMap;
use crate::core::compiler_options::{CompilerOptions, ModuleKind, ResolutionMode};
use crate::module::{ResolvedModule, Resolver};
use crate::tspath;

use super::util::PathAndFileName;
use super::{ModeAwareCacheKey, RegistryCloneHost};

/// Resolves aliases during export extraction by implementing a minimal
/// `checker.Program` interface.
///
/// Mirrors `autoimport.aliasResolver` in Go.
pub struct AliasResolver {
    pub to_path: Box<dyn Fn(&str) -> tspath::Path + Send + Sync>,
    pub host: Box<dyn RegistryCloneHost>,
    pub module_resolver: Option<Arc<Resolver>>,

    pub root_files: Vec<Arc<SourceFile>>,
    /// Maps from realpath to symlinked path and file name.
    pub symlinks: HashMap<tspath::Path, PathAndFileName>,
    pub on_failed_ambient_module_lookup: Box<dyn Fn(&dyn HasFileName, &str) + Send + Sync>,
    pub resolved_modules:
        SyncMap<tspath::Path, Arc<SyncMap<ModeAwareCacheKey, Arc<ResolvedModule>>>>,
}

/// A trait for types that have a file name (mirrors `ast.HasFileName`).
pub trait HasFileName {
    fn file_name(&self) -> &str;
    fn path(&self) -> tspath::Path;
}

impl AliasResolver {
    /// Creates a new alias resolver.
    ///
    /// Mirrors `newAliasResolver` in Go.
    pub fn new(
        root_files: Vec<Arc<SourceFile>>,
        symlinks: HashMap<tspath::Path, PathAndFileName>,
        host: Box<dyn RegistryCloneHost>,
        module_resolver: Option<Arc<Resolver>>,
        to_path: Box<dyn Fn(&str) -> tspath::Path + Send + Sync>,
        on_failed_ambient_module_lookup: Box<dyn Fn(&dyn HasFileName, &str) + Send + Sync>,
    ) -> AliasResolver {
        AliasResolver {
            to_path,
            host,
            module_resolver,
            root_files,
            symlinks,
            on_failed_ambient_module_lookup,
            resolved_modules: SyncMap::new(),
        }
    }

    // --- checker.Program implementation stubs ---

    /// Mirrors `BindSourceFiles`.
    pub fn bind_source_files(&self) {
        // We will bind as we parse
    }

    /// Mirrors `SourceFiles`.
    pub fn source_files(&self) -> &[Arc<SourceFile>] {
        &self.root_files
    }

    /// Mirrors `Options`.
    pub fn options(&self) -> CompilerOptions {
        let mut opts = CompilerOptions::default();
        opts.no_check = true.into();
        opts
    }

    /// Mirrors `GetCurrentDirectory`.
    pub fn get_current_directory(&self) -> &str {
        self.host.get_current_directory()
    }

    /// Mirrors `UseCaseSensitiveFileNames`.
    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.host.fs().use_case_sensitive_file_names()
    }

    /// Mirrors `GetSourceFile`.
    pub fn get_source_file(&self, _file_name: &str) -> Option<Arc<SourceFile>> {
        // Requires binder.BindSourceFile and host.GetSourceFile — stubbed
        todo!("AliasResolver::get_source_file requires binder integration")
    }

    /// Mirrors `GetDefaultResolutionModeForFile`.
    pub fn get_default_resolution_mode_for_file(&self, _file: &dyn HasFileName) -> ResolutionMode {
        ModuleKind::ESNext
    }

    /// Mirrors `GetEmitModuleFormatOfFile`.
    pub fn get_emit_module_format_of_file(&self, _source_file: &dyn HasFileName) -> ModuleKind {
        ModuleKind::ESNext
    }

    /// Mirrors `GetResolvedModule`.
    pub fn get_resolved_module(
        &self,
        _current_source_file: &dyn HasFileName,
        _module_reference: &str,
        _mode: ResolutionMode,
    ) -> Arc<ResolvedModule> {
        todo!("AliasResolver::get_resolved_module requires module resolver integration")
    }

    /// Mirrors `IsSourceFileDefaultLibrary`.
    pub fn is_source_file_default_library(&self, _path: &tspath::Path) -> bool {
        false
    }

    /// Mirrors `GetPackagesMap`.
    pub fn get_packages_map(&self) -> Option<HashMap<String, bool>> {
        None
    }
}
