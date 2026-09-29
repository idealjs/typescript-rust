#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::collections::set::Set;
use tsox_core::tspath::Path;
use tsox_frontend::ast::SourceFile;

use crate::ls::autoimport::LogTree;
use crate::ls::autoimport_registry_registry_impl::RegistryChange;
use crate::ls::autoimport::RegistryCloneHost;
use crate::ls::autoimport::ResolvedEntrypoint;
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_registry::BucketState;
use crate::ls::autoimport_registry::Directory;
use crate::ls::autoimport_registry::RegistryBucket;
use crate::ls::autoimport_util::PathAndFileName;
use crate::mig::m5o3_2::BucketBuildResult;
use crate::mig::m5o3_2::DiscoveredPackage;
use crate::mig::m5o3_2::FailedAmbientModuleLookupSource;
use crate::mig::m5o3_2::NodeModulesBucketTask;
use crate::mig::m5o3_2::PackageExtractionResult;
use crate::mig::m5o3_2::PerPackageExtractionResult;
use crate::mig::m5o3_2::RegistryBuilder;
use crate::mig::m5o3_2::bucket_build_preferences_from_user_preferences;
use crate::mig::m5o3_2::has_new_non_node_modules_files;
use crate::mig::m5o3_2::has_symlink_to_node_modules;
use crate::mig::m5o3_2::install_extractions;
use crate::mig::m5o3_2::is_ignored_file;
use crate::mig::m5o3_2::new_registry_bucket;
use crate::mig::m5o3_2::recursive_search_subset;
use crate::ls::lsutil_user_preferences::UserPreferences;
use tsox_compile::compiler::Program;

impl RegistryBuilder {
    pub fn get_nearest_ancestor_directory_with_package_json(&self, file_path: &Path) -> Option<Directory> {
        tsox_core::tspath::mig::m3i::for_each_ancestor_directory_path(&file_path.get_directory_path(), &mut |dir_path: &Path| {
            self.directories
                .entries
                .get(dir_path)
                .filter(|dir| dir.package_json.as_ref().map(|p| p.exists()).unwrap_or(false))
                .map(|dir| dir.clone_directory())
        })
    }

    pub fn resolve_ambient_module_name(&self, module_name: &str, from_path: &Path) -> Vec<String> {
        tsox_core::tspath::mig::m3i::for_each_ancestor_directory_path(from_path, &mut |dir_path: &Path| {
            if let Some(bucket) = self.node_modules.entries.get(dir_path) {
                if let Some(file_names) = bucket.ambient_module_names.get(module_name) {
                    return Some(file_names.clone());
                }
            }
            None
        })
        .unwrap_or_default()
    }

    pub fn compute_dependencies_for_node_modules_directory(
        &self,
        change: &RegistryChange,
        all_resolved_package_names: &HashMap<Path, Set<String>>,
        dir_name: &str,
        dir_path: &Path,
    ) -> Option<Set<String>> {
        for (path, _) in &change.open_files {
            if dir_path.contains_path(path) && self.get_nearest_ancestor_directory_with_package_json(path).is_none() {
                return None;
            }
        }

        let mut dependencies: Set<String> = Set::new();
        for (entry_path, dir) in &self.directories.entries {
            if dir.package_json.as_ref().map(|p| p.exists()).unwrap_or(false) && dir_path.contains_path(entry_path) {
                crate::ls::autoimport_util::add_package_json_dependencies(&mut dependencies);
            }
        }

        for resolved_package_names in all_resolved_package_names.values() {
            for name in resolved_package_names.iter() {
                dependencies.add(name.clone());
            }
        }

        Some(dependencies)
    }

    pub fn discover_bucket_packages(
        &self,
        package_names: &Set<String>,
        dir_name: &str,
        dir_path: &Path,
    ) -> Vec<DiscoveredPackage> {
        let mut result: Vec<DiscoveredPackage> = Vec::with_capacity(package_names.len());
        for package_name in package_names.iter() {
            let types_package_name = tsox_tsoptions::module::get_types_package_name(package_name);
            let package_json = self
                .host
                .get_package_json(&tsox_core::tspath::combine_paths(
                    dir_name,
                    &["node_modules", package_name, "package.json"],
                ))
                .map(info_cache_entry_from_host_entry);
            let mut types_package_json: Option<tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry> = None;
            if package_name != &types_package_name {
                let types_json = self
                    .host
                    .get_package_json(&tsox_core::tspath::combine_paths(
                        dir_name,
                        &["node_modules", &types_package_name, "package.json"],
                    ))
                    .map(info_cache_entry_from_host_entry);
                if types_json.as_ref().map(|t| t.directory_exists).unwrap_or(false) {
                    types_package_json = types_json;
                }
            }
            let mut realpath = String::new();
            if package_json.as_ref().map(|p| p.directory_exists).unwrap_or(false) {
                realpath = self.host.fs().realpath(&package_json.as_ref().unwrap().package_directory);
            }
            let mut types_realpath = String::new();
            if let Some(types_package_json) = &types_package_json {
                types_realpath = self.host.fs().realpath(&types_package_json.package_directory);
            }
            let is_local = !realpath.is_empty()
                && !realpath.contains("/node_modules/")
                && contains_path(
                    self.host.get_current_directory(),
                    &realpath,
                    &tsox_core::tspath::ComparePathsOptions {
                        use_case_sensitive_file_names: self.host.fs().use_case_sensitive_file_names(),
                        current_directory: self.host.get_current_directory().to_string(),
                    },
                );
            result.push(DiscoveredPackage {
                package_name: package_name.clone(),
                package_json,
                realpath,
                types_package_json,
                types_realpath,
                dir_path: dir_path.clone(),
                is_local,
            });
        }
        result
    }

    pub fn extract_package(
        &self,
        package_json: Option<&tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry>,
        package_name: &str,
        project_reference_outputs: &HashMap<Path, String>,
        file_exclude_patterns: Option<&crate::ls::autoimport::SpecMatcher>,
        enable_directory_search: bool,
    ) -> Option<PerPackageExtractionResult> {
        let package_json = package_json?;
        if !package_json.directory_exists {
            return None;
        }
        let (to_realpath, to_symlink) =
            crate::ls::autoimport_util::get_package_realpath_funcs(self.host.fs(), &package_json.package_directory);
        let to_realpath: Arc<dyn Fn(&str) -> String + Send + Sync> =
            Arc::new(move |file_name: &str| to_realpath(file_name));
        let resolver = crate::mig::m5o_2::get_module_resolver(
            Arc::clone(&self.host),
            {
                let to_realpath = Arc::clone(&to_realpath);
                move |file_name: &str| to_realpath(file_name)
            },
            crate::ls::autoimport::ResolverOptions,
        );
        let mut package_entrypoints: Vec<Arc<ResolvedEntrypoint>> = resolver
            .get_entrypoints_from_package_json_info(package_json, package_name, enable_directory_search)?
            .into_iter()
            .map(|entrypoint| {
                let mut include_conditions = Set::new();
                for condition in &entrypoint.include_conditions {
                    include_conditions.add(condition.clone());
                }
                let mut exclude_conditions = Set::new();
                for condition in &entrypoint.exclude_conditions {
                    exclude_conditions.add(condition.clone());
                }
                Arc::new(ResolvedEntrypoint {
                    resolved_file_name: entrypoint.resolved_file_name.clone(),
                    symlink_or_realpath: entrypoint.symlink_or_realpath().to_string(),
                    include_conditions,
                    exclude_conditions,
                })
            })
            .collect();
        let mut skipped_entrypoints = 0;
        if let Some(file_exclude_patterns) = file_exclude_patterns {
            let count = package_entrypoints.len();
            package_entrypoints.retain(|entrypoint| file_exclude_patterns.match_string(&entrypoint.resolved_file_name));
            skipped_entrypoints = count - package_entrypoints.len();
        }
        if package_entrypoints.is_empty() {
            return None;
        }

        let mut result = PerPackageExtractionResult {
            package_files: HashMap::new(),
            entrypoints: package_entrypoints.clone(),
            exports: HashMap::new(),
            ambient_modules: HashMap::new(),
            stats_exports: 0,
            stats_used_checker: 0,
            skipped_entrypoints,
            is_symlinked: false,
            failed_ambient_module_lookup_sources: HashMap::new(),
            failed_ambient_module_lookup_targets: Set::new(),
        };
        let PerPackageExtractionResult {
            mut package_files,
            mut entrypoints,
            mut exports,
            mut ambient_modules,
            mut stats_exports,
            stats_used_checker,
            skipped_entrypoints,
            mut is_symlinked,
            failed_ambient_module_lookup_sources,
            failed_ambient_module_lookup_targets,
        } = result;
        let failed_ambient_module_lookup_sources =
            std::sync::Arc::new(std::sync::Mutex::new(failed_ambient_module_lookup_sources));
        let failed_ambient_module_lookup_targets =
            std::sync::Arc::new(std::sync::Mutex::new(failed_ambient_module_lookup_targets));

        let mut seen_files: Set<Path> = Set::new();
        let mut root_files: Vec<Arc<SourceFile>> = Vec::new();
        let mut symlinks: HashMap<Path, PathAndFileName> = HashMap::new();
        for entrypoint in &package_entrypoints {
            let mut file_name = entrypoint.symlink_or_realpath().to_string();
            let mut realpath_file_name = entrypoint.resolved_file_name.clone();
            let mut realpath_path = (self.base.to_path)(&realpath_file_name);

            if let Some(input_file_name) = project_reference_outputs.get(&realpath_path) {
                file_name = to_symlink(input_file_name);
                realpath_file_name = input_file_name.clone();
                realpath_path = (self.base.to_path)(&realpath_file_name);
            }

            if !seen_files.add_if_absent(realpath_path.clone()) {
                continue;
            }
            if file_name != realpath_file_name {
                let symlink_path = (self.base.to_path)(&file_name);
                symlinks.insert(realpath_path.clone(), PathAndFileName { path: symlink_path, file_name });
                is_symlinked = true;
            }
            if let Some(file) = self.host.get_source_file(&realpath_file_name, &realpath_path) {
                tsox_checker::binder::bind_source_file(&file);
                root_files.push(file);
            }
        }

        let alias_resolver = std::sync::Arc::new(crate::ls::autoimport_alias_resolver::AliasResolver::new(
            root_files.clone(),
            symlinks.clone(),
            Box::new(SharedRegistryCloneHost(Arc::clone(&self.host))),
            Some(resolver.clone()),
            {
                let base = Arc::clone(&self.base);
                Box::new(move |file_name: &str| (base.to_path)(file_name))
            },
            {
                let failed_ambient_module_lookup_targets =
                    std::sync::Arc::clone(&failed_ambient_module_lookup_targets);
                let failed_ambient_module_lookup_sources =
                    std::sync::Arc::clone(&failed_ambient_module_lookup_sources);
                Box::new(
                    move |source: &dyn crate::ls::autoimport_alias_resolver::HasFileName,
                          module_name: &str| {
                        failed_ambient_module_lookup_targets
                            .lock()
                            .unwrap()
                            .add(module_name.to_string());
                        failed_ambient_module_lookup_sources
                            .lock()
                            .unwrap()
                            .entry(source.path())
                            .or_insert_with(|| FailedAmbientModuleLookupSource {
                                file_name: source.file_name().to_string(),
                                package_name: String::new(),
                            });
                    }
                )
            }
        ));

        let program: std::sync::Arc<dyn tsox_checker::checker::Program> = {
            let concrete: std::sync::Arc<crate::ls::autoimport_alias_resolver::AliasResolver> =
                std::sync::Arc::clone(&alias_resolver);
            concrete
        };
        let ch = std::sync::Arc::new(tsox_checker::checker::Checker::new(
            program,
            Arc::new(tsox_checker::checker::Tracer::new()),
        ));
        let extractor = crate::mig::m5o_2::RegistryBuilder {
            to_path: Some({
                let base = Arc::clone(&self.base);
                Arc::new(move |file_name: &str| (base.to_path)(file_name))
            }),
        }
        .new_export_extractor(package_name, ch.clone(), Some(resolver), {
            let to_realpath = Arc::clone(&to_realpath);
            Some(Box::new(move |file_name: &str| to_realpath(file_name)))
        });

        let mut non_module_files: Set<Path> = Set::new();
        for entrypoint in &alias_resolver.root_files {
            let file_exports = extractor.extract_from_file(entrypoint);
            for name in &entrypoint.ambient_module_names {
                ambient_modules
                    .entry(name.clone())
                    .or_default()
                    .push(entrypoint.file_name.clone());
            }
            package_files.insert(tsox_core::tspath::Path(entrypoint.file_name.clone()), entrypoint.file_name.clone());
            let symlink = alias_resolver.symlinks.get(&tsox_core::tspath::Path(entrypoint.file_name.clone())).cloned();
            let has_symlink = symlink.is_some();
            if let Some(symlink) = &symlink {
                package_files.insert(symlink.path.clone(), symlink.file_name.clone());
            }

            let mut has_exports = !file_exports.is_empty() && entrypoint.external_module_indicator.is_some();
            match failed_ambient_module_lookup_sources
                .lock()
                .unwrap()
                .get_mut(&tsox_core::tspath::Path(entrypoint.file_name.clone()))
            {
                None => {
                    exports.insert(tsox_core::tspath::Path(entrypoint.file_name.clone()), file_exports);
                }
                Some(source) => {
                    source.package_name = package_name.to_string();
                    has_exports = entrypoint.external_module_indicator.is_some();
                }
            }

            if !has_exports {
                non_module_files.add(tsox_core::tspath::Path(entrypoint.file_name.clone()));
                if let Some(symlink) = &symlink {
                    non_module_files.add(symlink.path.clone());
                }
            }
        }

        entrypoints.retain(|ep| !non_module_files.has(&(self.base.to_path)(&ep.resolved_file_name)));

        stats_exports = extractor.stats().exports.load(std::sync::atomic::Ordering::SeqCst);
        let stats_used_checker = extractor.stats().used_checker.load(std::sync::atomic::Ordering::SeqCst);
        let failed_ambient_module_lookup_sources =
            std::mem::replace(&mut *failed_ambient_module_lookup_sources.lock().unwrap(), HashMap::new());
        let failed_ambient_module_lookup_targets =
            std::mem::replace(&mut *failed_ambient_module_lookup_targets.lock().unwrap(), Set::new());
        Some(PerPackageExtractionResult {
            package_files,
            entrypoints,
            exports,
            ambient_modules,
            stats_exports,
            stats_used_checker,
            skipped_entrypoints,
            is_symlinked,
            failed_ambient_module_lookup_sources,
            failed_ambient_module_lookup_targets,
        })
    }
}

struct SharedRegistryCloneHost(Arc<dyn RegistryCloneHost>);

impl RegistryCloneHost for SharedRegistryCloneHost {
    fn fs(&self) -> &dyn tsox_tsoptions::vfs::FS {
        self.0.fs()
    }

    fn get_current_directory(&self) -> &str {
        self.0.get_current_directory()
    }

    fn get_default_project(
        &self,
        path: &Path,
    ) -> (
        tsox_core::tspath::Path,
        Option<std::sync::Arc<tsox_compile::compiler::Program>>,
    ) {
        self.0.get_default_project(path)
    }

    fn get_program_for_project(
        &self,
        project_path: &tsox_core::tspath::Path,
    ) -> Option<std::sync::Arc<tsox_compile::compiler::Program>> {
        self.0.get_program_for_project(project_path)
    }

    fn get_source_file(
        &self,
        file_name: &str,
        path: &tsox_core::tspath::Path,
    ) -> Option<std::sync::Arc<tsox_frontend::ast::SourceFile>> {
        self.0.get_source_file(file_name, path)
    }

    fn get_package_json(
        &self,
        file_name: &str,
    ) -> Option<crate::project::auto_import::PackageJsonInfoCacheEntry> {
        self.0.get_package_json(file_name)
    }

    fn dispose(&self) {
        self.0.dispose()
    }
}

fn info_cache_entry_from_host_entry(
    entry: crate::project::auto_import::PackageJsonInfoCacheEntry,
) -> tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry {
    tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry {
        package_directory: entry.package_directory,
        directory_exists: entry.directory_exists,
        contents: None,
    }
}

impl tsox_checker::checker::Program for crate::ls::autoimport_alias_resolver::AliasResolver {
    fn options(&self) -> &tsox_core::core::compiler_options::CompilerOptions {
        static OPTIONS: std::sync::OnceLock<tsox_core::core::compiler_options::CompilerOptions> =
            std::sync::OnceLock::new();
        OPTIONS.get_or_init(|| {
            let mut opts = tsox_core::core::compiler_options::CompilerOptions::default();
            opts.no_check = true.into();
            opts
        })
    }

    fn source_files(&self) -> &[Arc<SourceFile>] {
        &self.root_files
    }

    fn bind_source_files(&self) {}

    fn file_exists(&self, _file_name: &str) -> bool {
        panic!("unimplemented")
    }

    fn get_source_file(&self, file_name: &str) -> Option<Arc<SourceFile>> {
        let file = self.host.get_source_file(file_name, &(self.to_path)(file_name))?;
        tsox_checker::binder::bind_source_file(&file);
        Some(file)
    }

    fn is_source_file_default_library(&self, _path: &str) -> bool {
        false
    }

    fn symbol_map(&self) -> &tsox_frontend::ast::NodeSymbolMap {
        panic!("unimplemented")
    }

    fn current_directory(&self) -> &str {
        self.host.get_current_directory()
    }

    fn use_case_sensitive_file_names(&self) -> bool {
        self.host.fs().use_case_sensitive_file_names()
    }

    fn common_source_directory(&self) -> String {
        panic!("unimplemented")
    }
}

fn contains_path(parent: &str, target: &str, options: &tsox_core::tspath::ComparePathsOptions) -> bool {
    use tsox_core::tspath;
    let parent = tspath::combine_paths(&options.current_directory, &[parent]);
    let target = tspath::combine_paths(&options.current_directory, &[target]);
    if parent.is_empty() || target.is_empty() {
        return false;
    }
    if parent == target {
        return true;
    }
    let parent_components =
        tspath::reduce_path_components(&tspath::get_path_components(&parent, ""));
    let target_components =
        tspath::reduce_path_components(&tspath::get_path_components(&target, ""));
    if target_components.len() < parent_components.len() {
        return false;
    }
    for (i, parent_component) in parent_components.iter().enumerate() {
        let child_component = &target_components[i];
        let equal = if i == 0 || !options.use_case_sensitive_file_names {
            parent_component.eq_ignore_ascii_case(child_component)
        } else {
            parent_component == child_component
        };
        if !equal {
            return false;
        }
    }
    true
}
