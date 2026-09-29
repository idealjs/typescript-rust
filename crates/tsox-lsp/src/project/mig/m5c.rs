use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options_kinds::ModuleResolutionKind;
use tsox_core::semver::Version;
use tsox_core::tspath::{combine_paths, get_base_file_name, get_directory_path};
use tsox_core::tspath::Path;
use tsox_frontend::ast::SourceFile;
use tsox_tsoptions::module::Resolver;
use tsox_tsoptions::vfs::FS;

use super::m5d_2::{new_parse_cache_key, SourceFileParseOptions};
use super::m5e_7::{new_source_fs, SourceFS, SnapshotFSBuilder};
use super::super::ata_ata::{
    NpmConfig, NpmLock, TS_VERSION_TO_USE, TypingsInstallRequest, TypingsInstallResult,
    TypingsInstaller, install_npm_packages,
};
use super::super::ata_discover_typings::{AtaLogger, CachedTyping, discover_typings};
use super::super::overlay_fs::{DiskFile, FileHandle};
use super::super::parse_cache::ParseCacheKey;
use super::super::snapshot_fs::{FileSource, SnapshotFS};

impl TypingsInstaller {
    pub fn discover_and_install_typings(
        &mut self,
        request: &TypingsInstallRequest,
        logger: Option<&dyn AtaLogger>,
    ) -> Result<TypingsInstallResult, String> {
        let fs = request.fs.as_deref().expect("request fs");
        self.init(&request.project_id.0, fs, logger);

        let package_name_to_typing_location = self
            .package_name_to_typing_location
            .to_hash_map()
            .into_iter()
            .map(|(k, v)| (k, (*v).clone()))
            .collect::<HashMap<String, CachedTyping>>();
        let types_registry = self.types_registry.lock().unwrap().clone();
        let (cached_typing_paths, new_typing_names, files_to_watch) = discover_typings(
            fs,
            logger,
            &request.typings_info,
            &request.file_names,
            &request.project_root_path,
            &package_name_to_typing_location,
            &types_registry,
        );

        let request_id = self.install_run_count.fetch_add(1, Ordering::SeqCst) + 1;
        if !new_typing_names.is_empty() {
            let filtered_typings =
                self.filter_typings(&request.project_id, logger, &new_typing_names);
            if !filtered_typings.is_empty() {
                let _ = request_id;
                let result = self.install_typings(request)?;
                return Ok(TypingsInstallResult {
                    typings_files: result.typings_files,
                    files_to_watch,
                });
            }
            if let Some(logger) = logger {
                logger.log(
                    "ATA:: All typings are known to be missing or invalid - no need to install more typings",
                );
            }
        } else if let Some(logger) = logger {
            logger.log("ATA:: No new typings were requested as a result of typings discovery");
        }

        Ok(TypingsInstallResult {
            typings_files: cached_typing_paths,
            files_to_watch,
        })
    }

    pub fn install_worker(
        &self,
        _project_id: &Path,
        request_id: i32,
        package_names: &[String],
        logger: &dyn AtaLogger,
    ) -> (Vec<String>, bool) {
        logger.log(&format!(
            "ATA:: #{} with cwd: {} arguments: {:?}",
            request_id, self.typings_location, package_names
        ));
        let package_names = package_names.to_vec();
        let result = install_npm_packages(&package_names, 1, &|names: &[String]| {
            let mut npm_args: Vec<String> = Vec::new();
            npm_args.push("install".to_string());
            npm_args.push("--ignore-scripts".to_string());
            npm_args.extend(names.iter().cloned());
            npm_args.push("--save-dev".to_string());
            npm_args.push(format!("\"typesInstaller/{}\"", TS_VERSION_TO_USE));
            match self.host.npm_install(&self.typings_location, &npm_args) {
                Ok(_output) => Ok(()),
                Err(err) => Err(err),
            }
        });
        logger.log(&format!("TI:: npm install #{} completed", request_id));
        (package_names, result.is_ok())
    }

    pub fn process_cache_location(
        &self,
        _project_id: &str,
        fs: &Arc<dyn FS>,
        logger: Option<&dyn AtaLogger>,
    ) {
        let logger = logger;
        if let Some(logger) = logger {
            logger.log(&format!("ATA:: Processing cache location {}", self.typings_location));
        }
        let package_json = combine_paths(&self.typings_location, &["package.json"]);
        let package_lock_json = combine_paths(&self.typings_location, &["package-lock.json"]);
        if let Some(logger) = logger {
            logger.log(&format!("ATA:: Trying to find '{}'...", package_json));
        }
        if fs.file_exists(&package_json) && fs.file_exists(&package_lock_json) {
            let mut npm_config = NpmConfig::default();
            let npm_config_contents =
                parse_npm_config_or_lock(fs.as_ref(), &package_json, &mut npm_config);
            let mut npm_lock = NpmLock::default();
            let npm_lock_contents =
                parse_npm_config_or_lock(fs.as_ref(), &package_lock_json, &mut npm_lock);

            if let Some(logger) = logger {
                logger.log(&format!("ATA:: Loaded content of {}: {}", package_json, npm_config_contents));
                logger.log(&format!("ATA:: Loaded content of {}: {}", package_lock_json, npm_lock_contents));
            }

            let resolver = Resolver::new(
                Arc::new(TypingsResolutionHost { fs: Arc::clone(fs) }),
                Arc::new(CompilerOptions {
                    module_resolution: ModuleResolutionKind::Node10,
                    ..CompilerOptions::default()
                }),
                String::new(),
                String::new(),
            );
            if !npm_config.dev_dependencies.is_empty()
                && (!npm_lock.packages.is_empty() || !npm_lock.dependencies.is_empty())
            {
                for key in npm_config.dev_dependencies.keys() {
                    let npm_lock_value = npm_lock
                        .packages
                        .get(&format!("node_modules/{}", key))
                        .or_else(|| npm_lock.dependencies.get(key));
                    let Some(npm_lock_value) = npm_lock_value else {
                        continue;
                    };
                    let package_name = get_base_file_name(key);
                    if package_name.is_empty() {
                        continue;
                    }
                    let typing_file = self.typing_to_file_name(&resolver, &package_name);
                    if typing_file.is_empty() {
                        self.missing_typings_set.store(package_name.clone(), true);
                        continue;
                    }
                    if let Some(existing_typing_file) =
                        self.package_name_to_typing_location.load(&package_name)
                    {
                        if existing_typing_file.typings_location == typing_file {
                            continue;
                        }
                        if let Some(logger) = logger {
                            logger.log(&format!(
                                "ATA:: New typing for package {} from {} conflicts with existing typing file {}",
                                package_name, typing_file, existing_typing_file.typings_location
                            ));
                        }
                    }
                    if let Some(logger) = logger {
                        logger.log(&format!(
                            "ATA:: Adding entry into typings cache: {} => {}",
                            package_name, typing_file
                        ));
                    }
                    let version = npm_lock_value.version.clone();
                    if version.is_empty() {
                        continue;
                    }
                    let new_version =
                        tsox_core::semver::try_parse_version(&version).expect("semver parse");
                    let new_typing = Arc::new(CachedTyping {
                        typings_location: typing_file,
                        version: new_version,
                    });
                    self.package_name_to_typing_location
                        .store(package_name.clone(), new_typing);
                }
            }
        }
        if let Some(logger) = logger {
            logger.log(&format!("ATA:: Finished processing cache location {}", self.typings_location));
        }
    }

    pub fn ensure_typings_location_exists(&self, fs: &dyn FS, logger: Option<&dyn AtaLogger>) {
        let npm_config_path = combine_paths(&self.typings_location, &["package.json"]);
        if let Some(logger) = logger {
            logger.log(&format!("ATA:: Npm config file: {}", npm_config_path));
        }
        if !fs.file_exists(&npm_config_path) {
            if let Some(logger) = logger {
                logger.log(&format!(
                    "ATA:: Npm config file: '{}' is missing, creating new one...",
                    npm_config_path
                ));
            }
            if let Err(err) = fs.write_file(&npm_config_path, "{ \"private\": true }") {
                if let Some(logger) = logger {
                    logger.log(&format!("ATA:: Npm config file write failed: {}", err));
                }
            }
        }
    }

    pub fn load_types_registry_file(
        &self,
        fs: &dyn FS,
        logger: Option<&dyn AtaLogger>,
    ) -> HashMap<String, HashMap<String, String>> {
        let types_registry_file = combine_paths(
            &self.typings_location,
            &["node_modules/types-registry/index.json"],
        );
        if let Some(types_registry_file_contents) = fs.read_file(&types_registry_file) {
            let parsed: Result<HashMap<String, HashMap<String, HashMap<String, String>>>, _> =
                serde_json::from_str(&types_registry_file_contents);
            match parsed {
                Ok(entries) => {
                    if let Some(types_registry) = entries.get("entries") {
                        return types_registry.clone();
                    }
                    if let Some(logger) = logger {
                        logger.log(&format!(
                            "ATA:: Error when loading types registry file '{}': missing entries",
                            types_registry_file
                        ));
                    }
                }
                Err(err) => {
                    if let Some(logger) = logger {
                        logger.log(&format!(
                            "ATA:: Error when loading types registry file '{}': {}",
                            types_registry_file, err
                        ));
                    }
                }
            }
        } else if let Some(logger) = logger {
            logger.log(&format!(
                "ATA:: Error reading types registry file '{}'",
                types_registry_file
            ));
        }
        HashMap::new()
    }
}

pub fn parse_npm_config_or_lock<T: serde::de::DeserializeOwned>(
    fs: &dyn FS,
    location: &str,
    config: &mut T,
) -> String {
    let contents = fs.read_file(location).unwrap_or_default();
    if let Ok(parsed) = serde_json::from_str::<T>(&contents) {
        *config = parsed;
    }
    contents
}

pub fn add_typing_names_and_get_files_to_watch(
    fs: &dyn FS,
    logger: Option<&dyn AtaLogger>,
    inferred_typings: &mut HashMap<String, String>,
    files_to_watch: Vec<String>,
    project_root_path: &str,
    manifest_name: &str,
    modules_dir_name: &str,
) -> Vec<String> {
    let mut files_to_watch = files_to_watch;
    let manifest_path = combine_paths(project_root_path, &[manifest_name]);
    if let Some(contents) = fs.read_file(&manifest_path) {
        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(dependencies) = manifest.get("dependencies").and_then(|d| d.as_object()) {
                for name in dependencies.keys() {
                    add_inferred_typing(inferred_typings, name);
                }
            }
        }
    }

    let modules_path = combine_paths(project_root_path, &[modules_dir_name]);
    if fs.directory_exists(&modules_path) {
        for entry in &fs.get_accessible_entries(&modules_path).files {
            if let Some(typing_name) = super::super::ata_types_map::get_typing_name_from_directory_name(&entry) {
                add_inferred_typing(inferred_typings, &typing_name);
            }
        }
        if !files_to_watch.iter().any(|f| f == &modules_path) {
            files_to_watch.push(modules_path.clone());
        }
    }
    if let Some(logger) = logger {
        logger.log(&format!(
            "ATA::addTypingNamesAndGetFilesToWatch: ProjectRootPath: {}, ManifestName: {}, ModulesDirName: {}",
            project_root_path, manifest_name, modules_dir_name
        ));
    }
    files_to_watch
}

fn add_inferred_typing(inferred_typings: &mut HashMap<String, String>, typing_name: &str) {
    inferred_typings
        .entry(typing_name.to_string())
        .or_default();
}

struct TypingsResolutionHost {
    fs: Arc<dyn FS>,
}

impl tsox_tsoptions::module::resolver::ResolutionHost for TypingsResolutionHost {
    fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }

    fn get_current_directory(&self) -> &str {
        ""
    }
}

pub struct AutoImportBuilderFS {
    pub snapshot_fs_builder: Arc<SnapshotFSBuilder>,
    pub untracked_files: SyncMap<Path, Option<Arc<dyn FileHandle>>>,
}

impl AutoImportBuilderFS {
    pub fn fs(&self) -> &dyn FS {
        self.snapshot_fs_builder.fs.as_ref()
    }

    pub fn get_file(&self, file_name: &str) -> Option<Arc<dyn FileHandle>> {
        let path = (self.snapshot_fs_builder.to_path)(file_name);
        self.get_file_by_path(file_name, &path)
    }

    pub fn get_file_by_path(
        &self,
        file_name: &str,
        path: &Path,
    ) -> Option<Arc<dyn FileHandle>> {
        if let Some(overlay) = self.snapshot_fs_builder.overlays.get(path) {
            return Some(overlay.clone());
        }
        if let Some(disk_file) = self.snapshot_fs_builder.disk_files.get(path) {
            return self.snapshot_fs_builder.reload_entry_if_needed(path, disk_file);
        }
        if let Some(fh) = self.untracked_files.load(path) {
            return fh;
        }
        let mut fh: Option<Arc<dyn FileHandle>> = None;
        if let Some(content) = self.snapshot_fs_builder.fs.read_file(file_name) {
            fh = Some(Arc::new(DiskFile::new(file_name.to_string(), content)));
        }
        let (fh, _) = self
            .untracked_files
            .load_or_store(path.clone(), fh.clone());
        fh
    }

    pub fn get_accessible_entries(&self, path: &str) -> tsox_tsoptions::vfs::Entries {
        self.snapshot_fs_builder.get_accessible_entries(path)
    }

    pub fn file_exists(&self, file_name: &str, path: &Path) -> bool {
        self.snapshot_fs_builder.file_exists(file_name, path)
    }
}

impl FileSource for AutoImportBuilderFS {
    fn fs(&self) -> &dyn FS {
        AutoImportBuilderFS::fs(self)
    }

    fn get_file(&self, file_name: &str) -> Option<Arc<dyn FileHandle>> {
        AutoImportBuilderFS::get_file(self, file_name)
    }

    fn get_file_by_path(
        &self,
        file_name: &str,
        path: &Path,
    ) -> Option<Arc<dyn FileHandle>> {
        AutoImportBuilderFS::get_file_by_path(self, file_name, path)
    }

    fn file_exists(&self, file_name: &str, path: &Path) -> bool {
        AutoImportBuilderFS::file_exists(self, file_name, path)
    }

    fn get_accessible_entries(&self, path: &str) -> tsox_tsoptions::vfs::Entries {
        AutoImportBuilderFS::get_accessible_entries(self, path)
    }
}

pub struct AutoImportRegistryCloneHostReal {
    pub project_collection: Arc<super::super::project_collection::ProjectCollection>,
    pub parse_cache: Arc<super::super::parse_cache::ParseCache>,
    pub fs: Arc<SourceFS>,
    pub current_directory: String,
    pub files_mu: Mutex<Vec<ParseCacheKey>>,
}

pub fn new_auto_import_registry_clone_host(
    project_collection: Arc<super::super::project_collection::ProjectCollection>,
    parse_cache: Arc<super::super::parse_cache::ParseCache>,
    snapshot_fs_builder: Arc<SnapshotFSBuilder>,
    current_directory: String,
    to_path: Arc<dyn Fn(&str) -> Path + Send + Sync>,
) -> AutoImportRegistryCloneHostReal {
    let builder_fs = Arc::new(AutoImportBuilderFS {
        snapshot_fs_builder,
        untracked_files: SyncMap::new(),
    });
    AutoImportRegistryCloneHostReal {
        project_collection,
        parse_cache,
        fs: Arc::new(new_source_fs(false, builder_fs, to_path)),
        current_directory,
        files_mu: Mutex::new(Vec::new()),
    }
}

impl AutoImportRegistryCloneHostReal {
    pub fn fs(&self) -> &dyn FS {
        self.fs.source.fs()
    }

    pub fn get_current_directory(&self) -> &str {
        &self.current_directory
    }
}

impl super::super::auto_import::RegistryCloneHost for AutoImportRegistryCloneHostReal {
    fn fs(&self) -> &dyn FS {
        AutoImportRegistryCloneHostReal::fs(self)
    }

    fn get_current_directory(&self) -> &str {
        AutoImportRegistryCloneHostReal::get_current_directory(self)
    }

    fn get_default_project(
        &self,
        path: &Path,
    ) -> (Path, Option<Arc<tsox_compile::compiler::Program>>) {
        match self.project_collection.get_default_project(path) {
            Some(project) => (
                project.config_file_path().clone(),
                project.get_program().cloned(),
            ),
            None => (Path(String::new()), None),
        }
    }

    fn get_package_json(
        &self,
        _file_name: &str,
    ) -> Option<super::super::auto_import::PackageJsonInfoCacheEntry> {
        None
    }

    fn get_program_for_project(
        &self,
        project_path: &Path,
    ) -> Option<Arc<tsox_compile::compiler::Program>> {
        self.project_collection
            .get_project_by_path(project_path)
            .and_then(|project| project.get_program().cloned())
    }

    fn get_source_file(
        &self,
        file_name: &str,
        path: &Path,
    ) -> Option<Arc<SourceFile>> {
        let fh = self.fs.get_file(file_name)?;
        let options = SourceFileParseOptions {
            file_name: file_name.to_string(),
            path: path.clone(),
        };
        let key = new_parse_cache_key(options, fh.hash(), fh.kind());
        let content = fh.content().to_string();
        let result = self.parse_cache.acquire(&key, fh.as_ref(), move |key| {
            Arc::new(tsox_frontend::parser::Parser::parse_source_file_text(
                &key.file_name,
                content,
            ))
        });
        let mut files = self.files_mu.lock().unwrap();
        files.push(key);
        drop(files);
        Some(result)
    }

    fn dispose(&self) {
        let files = self.files_mu.lock().unwrap();
        for key in files.iter() {
            self.parse_cache.deref(key);
        }
    }
}
