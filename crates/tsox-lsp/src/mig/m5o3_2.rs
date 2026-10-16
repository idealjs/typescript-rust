#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath::Path;
use tsox_frontend::ast::SourceFile;

use crate::ls::autoimport::DirtyMap;
use crate::ls::autoimport::DirtyMapBuilder;
use crate::ls::autoimport::LogTree;
use crate::ls::autoimport_registry_registry_impl::RegistryChange;
use crate::ls::autoimport::RegistryCloneHost;
use crate::ls::autoimport::ResolvedEntrypoint;
use crate::ls::autoimport::ResolverOptions;
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_registry::BucketBuildPreferences;
use crate::ls::autoimport_registry::BucketState;
use crate::ls::autoimport_registry::Directory;
use crate::ls::autoimport_registry::NewProgramStructure;
use crate::ls::autoimport_registry::Registry;
use crate::ls::autoimport_registry::RegistryBucket;
use crate::ls::lsutil_user_preferences::UserPreferences;
use tsox_compile::compiler::Program;

pub use crate::mig::m5o3_4::*;

pub fn bucket_build_preferences_from_user_preferences(prefs: &UserPreferences) -> BucketBuildPreferences { ::tsox_core::fntrace::enter("bucket_build_preferences_from_user_preferences"); 
    BucketBuildPreferences {
        file_exclude_patterns: prefs.auto_import_file_exclude_patterns.clone(),
        auto_import_entrypoint_directory_search: prefs.auto_import_entrypoint_directory_search,
    }
}

pub fn new_registry_bucket() -> RegistryBucket { ::tsox_core::fntrace::enter("new_registry_bucket"); 
    RegistryBucket {
        state: BucketState {
            multiple_files_dirty: true,
            new_program_structure: NewProgramStructure::DifferentFileNames,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn recursive_search_subset(target: Option<&Set<String>>, current: Option<&Set<String>>) -> bool { ::tsox_core::fntrace::enter("recursive_search_subset"); 
    match (target, current) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(_), None) => true,
        (Some(target), Some(current)) => target.is_subset_of(current),
    }
}

pub fn is_ignored_file(program: &Program, file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("is_ignored_file"); 
    program.is_source_file_default_library(&file.file_name)
        || program.is_global_typings_file(&file.file_name)
}

pub fn has_new_non_node_modules_files(program: &Program, bucket: &RegistryBucket) -> bool { ::tsox_core::fntrace::enter("has_new_non_node_modules_files"); 
    if bucket.state.new_program_structure != NewProgramStructure::DifferentFileNames {
        return false;
    }
    for file in program.get_source_files() {
        if tsox_frontend::ast::mig::m3b_2::is_content_mapper_supplemental(&file)
            || file.file_name.contains("/node_modules/")
            || is_ignored_file(program, &file)
        {
            continue;
        }
        if !bucket.paths.contains_key(&tsox_core::tspath::Path(file.file_name.clone())) {
            return true;
        }
    }
    false
}

pub fn has_symlink_to_node_modules(
    file_path: &Path,
    project_root_path: &Path,
    symlink_cache: Option<&tsox_core::symlinks::KnownSymlinks>,
) -> bool { ::tsox_core::fntrace::enter("has_symlink_to_node_modules"); 
    let Some(symlink_cache) = symlink_cache else {
        return false;
    };
    if project_root_path.contains_path(file_path) {
        return false;
    }

    if let Some(symlink_paths) = symlink_cache.files_by_realpath().load(file_path) {
        let hit = symlink_paths
            .lock()
            .unwrap()
            .iter()
            .any(|symlink_path| symlink_path.contains("/node_modules/"));
        if hit {
            return true;
        }
    }

    let directories_by_realpath = symlink_cache.directories_by_realpath();
    let mut found = false;
    tsox_core::tspath::mig::m3i::for_each_ancestor_directory_path(file_path, &mut |dir_path: &Path| {
        let Some(symlink_paths) = directories_by_realpath.load(&dir_path.ensure_trailing_directory_separator()) else {
            return None;
        };
        let hit = symlink_paths
            .lock()
            .unwrap()
            .iter()
            .any(|symlink_path| symlink_path.contains("/node_modules/"));
        if hit {
            found = true;
        }
        if found {
            return Some(());
        }
        None
    });
    found
}

pub fn install_extractions(
    discovered: &[DiscoveredPackage],
    extraction_cache: &HashMap<String, PerPackageExtractionResult>,
) -> PackageExtractionResult { ::tsox_core::fntrace::enter("install_extractions"); 
    let mut result = PackageExtractionResult {
        exports: HashMap::new(),
        package_files: HashMap::new(),
        ambient_module_names: HashMap::new(),
        entrypoints: Vec::new(),
        workspace_packages: Set::new(),
        possible_failed_ambient_module_lookup_sources: HashMap::new(),
        possible_failed_ambient_module_lookup_targets: Set::new(),
        stats_exports: 0,
        stats_used_checker: 0,
        skipped_entrypoints_count: 0,
    };

    for pkg in discovered {
        let extraction = extraction_cache
            .get(&pkg.realpath)
            .or_else(|| extraction_cache.get(&pkg.types_realpath));
        let Some(extraction) = extraction else {
            continue;
        };
        result.exports.extend(extraction.exports.clone());
        result
            .package_files
            .entry(pkg.package_name.clone())
            .or_default()
            .extend(extraction.package_files.clone());
        for (name, file_names) in &extraction.ambient_modules {
            result
                .ambient_module_names
                .entry(name.clone())
                .or_default()
                .extend(file_names.clone());
        }
        if !extraction.entrypoints.is_empty() {
            result.entrypoints.push(extraction.entrypoints.clone());
        }
        for (path, source) in &extraction.failed_ambient_module_lookup_sources {
            result
                .possible_failed_ambient_module_lookup_sources
                .entry(path.clone())
                .or_insert_with(|| FailedAmbientModuleLookupSource {
                    file_name: source.file_name.clone(),
                    package_name: source.package_name.clone(),
                });
        }
        for target in extraction.failed_ambient_module_lookup_targets.iter() {
            result.possible_failed_ambient_module_lookup_targets.add(target.clone());
        }
        if extraction.is_symlinked && pkg.is_local {
            result.workspace_packages.add(pkg.package_name.clone());
        }
        result.stats_exports += extraction.stats_exports;
        result.stats_used_checker += extraction.stats_used_checker;
        result.skipped_entrypoints_count += extraction.skipped_entrypoints;
    }

    result
}

fn package_json_info_stub_from_host_entry(
    entry: crate::project::auto_import::PackageJsonInfoCacheEntry,
) -> crate::ls::autoimport_registry_registry_impl::PackageJsonInfoStub { ::tsox_core::fntrace::enter("package_json_info_stub_from_host_entry"); 
    crate::ls::autoimport_registry_registry_impl::PackageJsonInfoStub {
        exists: entry.directory_exists && !entry.package_directory.is_empty(),
        parseable: false,
    }
}

pub struct RegistryBuilder {
    pub host: Arc<dyn RegistryCloneHost>,
    pub base: Arc<Registry>,
    pub user_preferences: UserPreferences,
    pub directories: DirtyMap<Path, Directory>,
    pub node_modules: DirtyMap<Path, RegistryBucket>,
    pub projects: DirtyMap<Path, RegistryBucket>,
    pub specifier_cache: DirtyMapBuilder<Path, SyncMap<Path, String>, SyncMap<Path, String>>,
    pub resolver_options: ResolverOptions,
    pub unique_package_count: usize,
    pub entrypoints: DirtyMapBuilder<Path, Vec<Arc<ResolvedEntrypoint>>, Vec<Arc<ResolvedEntrypoint>>>,
}

pub fn new_registry_builder(registry: &Arc<Registry>, host: Arc<dyn RegistryCloneHost>) -> RegistryBuilder { ::tsox_core::fntrace::enter("new_registry_builder"); 
    RegistryBuilder {
        host,
        base: std::sync::Arc::clone(registry),
        user_preferences: registry.user_preferences.clone(),
        directories: DirtyMap {
            entries: registry.directories.clone(),
        },
        node_modules: DirtyMap {
            entries: registry.node_modules.clone(),
        },
        projects: DirtyMap {
            entries: registry.projects.clone(),
        },
        specifier_cache: DirtyMapBuilder::new(registry.specifier_cache.clone()),
        resolver_options: ResolverOptions,
        unique_package_count: registry.unique_package_count,
        entrypoints: DirtyMapBuilder::new(registry.entrypoints.clone()),
    }
}

impl RegistryBuilder {
    fn update_directory(&mut self, dir_path: &Path, dir_name: &str, package_json_changed: bool) { ::tsox_core::fntrace::enter("update_directory"); 
        let package_json_file_name = tsox_core::tspath::combine_paths(dir_name, &["package.json"]);
        let has_node_modules = self
            .host
            .fs()
            .directory_exists(&tsox_core::tspath::combine_paths(dir_name, &["node_modules"]));
        match self.directories.entries.get_mut(dir_path) {
            Some(dir) => {
                if package_json_changed || dir.has_node_modules != has_node_modules {
                    dir.package_json = self
                        .host
                        .get_package_json(&package_json_file_name)
                        .map(package_json_info_stub_from_host_entry);
                    dir.has_node_modules = has_node_modules;
                }
            }
            None => {
                self.directories.entries.insert(
                    dir_path.clone(),
                    Directory {
                        name: dir_name.to_string(),
                        package_json: self
                            .host
                            .get_package_json(&package_json_file_name)
                            .map(package_json_info_stub_from_host_entry),
                        has_node_modules,
                    },
                );
            }
        }
        if has_node_modules {
            self.node_modules
                .entries
                .entry(dir_path.clone())
                .or_insert_with(new_registry_bucket);
        } else {
            self.node_modules.entries.remove(dir_path);
        }
    }

    pub fn update_bucket_and_directory_existence(&mut self, change: &RegistryChange, _logger: Option<&LogTree>) { ::tsox_core::fntrace::enter("update_bucket_and_directory_existence"); 
        let mut needed_projects: Set<Path> = Set::new();
        let mut needed_directories: HashMap<Path, String> = HashMap::new();
        for (path, file_name) in &change.open_files {
            needed_projects.add(self.host.get_default_project(path).0);
            if !tsox_core::tspath::is_dynamic_file_name(file_name) {
                let mut dir = file_name.clone();
                let mut dir_path = path.clone();
                loop {
                    dir = tsox_core::tspath::get_directory_path(&dir);
                    let last_dir_path = dir_path.clone();
                    dir_path = dir_path.get_directory_path();
                    if dir_path == last_dir_path {
                        break;
                    }
                    if needed_directories.contains_key(&dir_path) {
                        break;
                    }
                    needed_directories.insert(dir_path.clone(), dir.clone());
                }
            }
            self.specifier_cache
                .entries
                .entry(path.clone())
                .or_insert_with(SyncMap::new);
        }

        if !change.requested_file.0.is_empty() {
            needed_projects.add(self.host.get_default_project(&change.requested_file).0);
            self.specifier_cache
                .entries
                .entry(change.requested_file.clone())
                .or_insert_with(SyncMap::new);
        }

        let base_specifier_paths: Vec<Path> = self.base.specifier_cache.keys().cloned().collect();
        for path in &base_specifier_paths {
            if !change.open_files.contains_key(path) && *path != change.requested_file {
                self.specifier_cache.entries.remove(path);
            }
        }

        for project_path in needed_projects.iter() {
            if !self.projects.entries.contains_key(project_path) {
                self.projects
                    .entries
                    .insert(project_path.clone(), new_registry_bucket());
            }
        }
        let base_project_paths: Vec<Path> = self.base.projects.keys().cloned().collect();
        for project_path in &base_project_paths {
            if !needed_projects.has(project_path) {
                self.projects.entries.remove(project_path);
            }
        }

        let package_json_changed = |dir_name: &str| -> bool {
            let uri = crate::lsp::lsproto::DocumentUri(crate::ls::lsconv_converters::file_name_to_document_uri(
                &tsox_core::tspath::combine_paths(dir_name, &["package.json"]),
            ));
            change.changed.has(&uri) || change.deleted.has(&uri) || change.created.has(&uri)
        };

        for (dir_path, dir_name) in &needed_directories {
            match self.base.directories.get(dir_path) {
                Some(base_dir) => {
                    let unchanged = !package_json_changed(dir_name)
                        && base_dir.has_node_modules
                            == self
                                .host
                                .fs()
                                .directory_exists(&tsox_core::tspath::combine_paths(dir_name, &["node_modules"]));
                    if !unchanged {
                        self.update_directory(dir_path, dir_name, package_json_changed(dir_name));
                    }
                }
                None => {
                    self.update_directory(dir_path, dir_name, false);
                }
            }
        }
        let base_directory_paths: Vec<Path> = self.base.directories.keys().cloned().collect();
        for dir_path in &base_directory_paths {
            if !needed_directories.contains_key(dir_path) {
                self.directories.entries.remove(dir_path);
                self.node_modules.entries.remove(dir_path);
            }
        }
    }

    pub fn mark_buckets_dirty(&mut self, change: &RegistryChange, _logger: Option<&LogTree>) { ::tsox_core::fntrace::enter("mark_buckets_dirty"); 
        for (project_path, new_file_names) in &change.rebuilt_programs {
            if let Some(bucket) = self.projects.entries.get_mut(project_path) {
                bucket.state.new_program_structure = if *new_file_names {
                    NewProgramStructure::DifferentFileNames
                } else {
                    NewProgramStructure::SameFileNames
                };
            }
        }

        let mut clean_node_modules_buckets: Set<Path> = Set::new();
        for (path, bucket) in &self.node_modules.entries {
            if !bucket.state.multiple_files_dirty {
                clean_node_modules_buckets.add(path.clone());
            }
        }
        let mut clean_project_buckets: Set<Path> = Set::new();
        for (path, bucket) in &self.projects.entries {
            if !bucket.state.multiple_files_dirty {
                clean_project_buckets.add(path.clone());
            }
        }

        let mut mark_files_dirty = |builder: &mut RegistryBuilder, uris: &Set<crate::lsp::lsproto::DocumentUri>| {
            if clean_node_modules_buckets.is_empty() && clean_project_buckets.is_empty() {
                return;
            }
            for uri in uris.iter() {
                let path = (builder.base.to_path)(&uri.file_name());

                if !clean_node_modules_buckets.is_empty() {
                    match path.0.find("/node_modules/") {
                        Some(node_modules_index) => {
                            let dir_path = Path(path.0[..node_modules_index].to_string());
                            if clean_node_modules_buckets.has(&dir_path) {
                                if let Some(entry) = builder.node_modules.entries.get_mut(&dir_path) {
                                    let package_name = entry.paths.get(&path).cloned().unwrap_or_default();
                                    entry.mark_node_modules_dirty(&package_name);
                                    if !entry.state.multiple_files_dirty {
                                        clean_node_modules_buckets.delete(&dir_path);
                                    }
                                }
                            }
                        }
                        None => {
                            let bucket_dir_paths: Vec<Path> =
                                clean_node_modules_buckets.iter().cloned().collect();
                            for bucket_dir_path in bucket_dir_paths {
                                if let Some(entry) = builder.node_modules.entries.get_mut(&bucket_dir_path) {
                                    if let Some(package_name) = entry.paths.get(&path) {
                                        let package_name = package_name.clone();
                                        entry.mark_node_modules_dirty(&package_name);
                                        if !entry.state.multiple_files_dirty {
                                            clean_node_modules_buckets.delete(&bucket_dir_path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let project_dir_paths: Vec<Path> = clean_project_buckets.iter().cloned().collect();
                for project_dir_path in project_dir_paths {
                    if let Some(entry) = builder.projects.entries.get_mut(&project_dir_path) {
                        if entry.paths.contains_key(&path) {
                            entry.mark_project_file_dirty(path.clone());
                            if !entry.state.multiple_files_dirty {
                                clean_project_buckets.delete(&project_dir_path);
                            }
                        }
                    }
                }
            }
        };

        mark_files_dirty(self, &change.created);
        mark_files_dirty(self, &change.deleted);
        mark_files_dirty(self, &change.changed);
    }

    pub fn update_indexes(&mut self, change: &RegistryChange, _logger: Option<&LogTree>) { ::tsox_core::fntrace::enter("update_indexes"); 
        let (project_path, _) = self.host.get_default_project(&change.requested_file);
        if project_path.0.is_empty() {
            return;
        }

        let mut project_reference_outputs: HashMap<Path, String> = HashMap::new();
        let project_paths: Vec<Path> = self.projects.entries.keys().cloned().collect();
        for entry_path in &project_paths {
            if let Some(program) = self.host.get_program_for_project(entry_path) {
                crate::ls::autoimport_util::add_project_reference_output_mappings(&program, &mut project_reference_outputs);
            }
        }

        todo!(
            "updateIndexes phases 2-3: node_modules task collection, extraction cache, \
             buildNodeModulesBucket/buildProjectBucket/updateNodeModulesBucket (Go registry.go 782-1810)"
        );
    }

    pub fn build(&self) -> Registry { ::tsox_core::fntrace::enter("build"); 
        let base = std::sync::Arc::clone(&self.base);
        Registry {
            to_path: Box::new(move |file_name: &str| (base.to_path)(file_name)),
            user_preferences: self.user_preferences.clone(),
            directories: self.directories.entries.clone(),
            node_modules: self.node_modules.entries.clone(),
            projects: self.projects.entries.clone(),
            specifier_cache: self.specifier_cache.entries.clone(),
            unique_package_count: self.unique_package_count,
            entrypoints: self.entrypoints.entries.clone(),
        }
    }
}

impl Registry {
    pub fn clone(
        self: &Arc<Self>,
        change: &RegistryChange,
        host: Arc<dyn RegistryCloneHost>,
        logger: Option<&LogTree>,
    ) -> Result<Registry, String> { ::tsox_core::fntrace::enter("clone"); 
        let logger = logger.map(|l| l.fork("Building autoimport registry"));
        let mut builder = new_registry_builder(self, host);
        if let Some(prefs) = &change.user_preferences {
            builder.user_preferences = prefs.clone();
            if builder.user_preferences.auto_import_specifier_exclude_regexes
                != self.user_preferences.auto_import_specifier_exclude_regexes
            {
                builder.specifier_cache.entries.clear();
            }
        }
        builder.update_bucket_and_directory_existence(change, logger.as_ref());
        builder.mark_buckets_dirty(change, logger.as_ref());
        if !change.requested_file.0.is_empty() {
            builder.update_indexes(change, logger.as_ref());
        }
        Ok(builder.build())
    }
}
