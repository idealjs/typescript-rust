use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options::ModuleKind;
use tsox_core::core::compiler_options::ResolutionMode;
use tsox_frontend::ast::SourceFile;
use tsox_tsoptions::module::ResolvedModule;
use tsox_tsoptions::module::Resolver;

use crate::ls::autoimport::ModeAwareCacheKey;
use crate::ls::autoimport::RegistryCloneHost;
use crate::ls::autoimport_util::PathAndFileName;

pub struct AliasResolver {
    pub to_path: Box<dyn Fn(&str) -> tsox_core::tspath::Path + Send + Sync>,
    pub host: Box<dyn RegistryCloneHost>,
    pub module_resolver: Option<Arc<Resolver>>,

    pub root_files: Vec<Arc<SourceFile>>,

    pub symlinks: HashMap<tsox_core::tspath::Path, PathAndFileName>,
    pub on_failed_ambient_module_lookup: Box<dyn Fn(&dyn HasFileName, &str) + Send + Sync>,
    pub resolved_modules:
        SyncMap<tsox_core::tspath::Path, Arc<SyncMap<ModeAwareCacheKey, Arc<ResolvedModule>>>>,
}

pub trait HasFileName {
    fn file_name(&self) -> &str;
    fn path(&self) -> tsox_core::tspath::Path;
}

impl AliasResolver {
    pub fn new(
        root_files: Vec<Arc<SourceFile>>,
        symlinks: HashMap<tsox_core::tspath::Path, PathAndFileName>,
        host: Box<dyn RegistryCloneHost>,
        module_resolver: Option<Arc<Resolver>>,
        to_path: Box<dyn Fn(&str) -> tsox_core::tspath::Path + Send + Sync>,
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

    pub fn bind_source_files(&self) {}

    pub fn source_files(&self) -> &[Arc<SourceFile>] {
        &self.root_files
    }

    pub fn options(&self) -> CompilerOptions {
        let mut opts = CompilerOptions::default();
        opts.no_check = true.into();
        opts
    }

    pub fn get_current_directory(&self) -> &str {
        self.host.get_current_directory()
    }

    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.host.fs().use_case_sensitive_file_names()
    }

    pub fn get_source_file(&self, _file_name: &str) -> Option<Arc<SourceFile>> {
        todo!("AliasResolver::get_source_file requires binder integration")
    }

    pub fn get_default_resolution_mode_for_file(&self, _file: &dyn HasFileName) -> ResolutionMode {
        ModuleKind::ESNext
    }

    pub fn get_emit_module_format_of_file(&self, _source_file: &dyn HasFileName) -> ModuleKind {
        ModuleKind::ESNext
    }

    pub fn get_resolved_module(
        &self,
        _current_source_file: &dyn HasFileName,
        _module_reference: &str,
        _mode: ResolutionMode,
    ) -> Arc<ResolvedModule> {
        todo!("AliasResolver::get_resolved_module requires module resolver integration")
    }

    pub fn is_source_file_default_library(&self, _path: &tsox_core::tspath::Path) -> bool {
        false
    }

    pub fn get_packages_map(&self) -> Option<HashMap<String, bool>> {
        None
    }
}
