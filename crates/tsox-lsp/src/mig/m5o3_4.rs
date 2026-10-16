#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::collections::set::Set;
use tsox_core::tspath::Path;
use tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry;

use crate::ls::autoimport::ResolvedEntrypoint;
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_registry_bucket::{
    BucketBuildPreferences, BucketState, RegistryBucket,
};
use crate::ls::autoimport_registry_registry_impl::Directory;

pub struct FailedAmbientModuleLookupSource {
    pub file_name: String,
    pub package_name: String,
}

pub struct DiscoveredPackage {
    pub package_name: String,
    pub package_json: Option<InfoCacheEntry>,
    pub realpath: String,
    pub types_package_json: Option<InfoCacheEntry>,
    pub types_realpath: String,
    pub dir_path: Path,
    pub is_local: bool,
}

pub struct PerPackageExtractionResult {
    pub package_files: HashMap<Path, String>,
    pub entrypoints: Vec<Arc<ResolvedEntrypoint>>,
    pub exports: HashMap<Path, Vec<Export>>,
    pub ambient_modules: HashMap<String, Vec<String>>,
    pub stats_exports: i32,
    pub stats_used_checker: i32,
    pub skipped_entrypoints: usize,
    pub is_symlinked: bool,
    pub failed_ambient_module_lookup_sources: HashMap<Path, FailedAmbientModuleLookupSource>,
    pub failed_ambient_module_lookup_targets: Set<String>,
}

pub struct PackageExtractionResult {
    pub exports: HashMap<Path, Vec<Export>>,
    pub package_files: HashMap<String, HashMap<Path, String>>,
    pub ambient_module_names: HashMap<String, Vec<String>>,
    pub entrypoints: Vec<Vec<Arc<ResolvedEntrypoint>>>,
    pub workspace_packages: Set<String>,
    pub possible_failed_ambient_module_lookup_sources: HashMap<Path, FailedAmbientModuleLookupSource>,
    pub possible_failed_ambient_module_lookup_targets: Set<String>,
    pub stats_exports: i32,
    pub stats_used_checker: i32,
    pub skipped_entrypoints_count: usize,
}

pub struct BucketBuildResult {
    pub key: Path,
    pub err: Option<String>,
    pub bucket: Option<RegistryBucket>,
    pub entrypoints: HashMap<Path, Vec<Arc<ResolvedEntrypoint>>>,
    pub removed_entrypoint_paths: Vec<Path>,
    pub possible_failed_ambient_module_lookup_sources: HashMap<Path, FailedAmbientModuleLookupSource>,
    pub possible_failed_ambient_module_lookup_targets: Set<String>,
}

pub struct NodeModulesBucketTask {
    pub dir_path: Path,
    pub dir_name: String,
    pub dependency_names: Option<Set<String>>,
    pub is_update: bool,
    pub existing_bucket: Option<RegistryBucket>,
    pub dirty_packages: Option<Set<String>>,
    pub package_names: Option<Set<String>>,
    pub directory_package_names: Option<Set<String>>,
    pub discovered: Vec<DiscoveredPackage>,
}

impl BucketBuildPreferences {
    pub fn clone_bucket_build_preferences(&self) -> BucketBuildPreferences { ::tsox_core::fntrace::enter("clone_bucket_build_preferences"); 
        BucketBuildPreferences {
            file_exclude_patterns: self.file_exclude_patterns.clone(),
            auto_import_entrypoint_directory_search: self.auto_import_entrypoint_directory_search,
        }
    }
}

impl BucketState {
    pub fn clone_state(&self) -> BucketState { ::tsox_core::fntrace::enter("clone_state"); 
        BucketState {
            dirty_file: self.dirty_file.clone(),
            multiple_files_dirty: self.multiple_files_dirty,
            new_program_structure: self.new_program_structure,
            build_preferences: self.build_preferences.clone_bucket_build_preferences(),
            dirty_packages: self.dirty_packages.clone(),
            recursive_search_packages: self.recursive_search_packages.clone(),
        }
    }

    pub fn dirty_file(&self) -> Path { ::tsox_core::fntrace::enter("dirty_file"); 
        if self.multiple_files_dirty {
            Path(String::new())
        } else {
            self.dirty_file.clone()
        }
    }
}

impl RegistryBucket {
    pub fn clone_bucket(&self) -> RegistryBucket { ::tsox_core::fntrace::enter("clone_bucket"); 
        RegistryBucket {
            state: self.state.clone_state(),
            paths: self.paths.clone(),
            package_files: self.package_files.clone(),
            resolved_package_names: self.resolved_package_names.clone(),
            dependency_names: self.dependency_names.clone(),
            ambient_module_names: self.ambient_module_names.clone(),
            index: self.index.clone(),
        }
    }
}

impl Directory {
    pub fn clone_directory(&self) -> Directory { ::tsox_core::fntrace::enter("clone_directory"); 
        Directory {
            name: self.name.clone(),
            package_json: self.package_json.clone(),
            has_node_modules: self.has_node_modules,
        }
    }
}

