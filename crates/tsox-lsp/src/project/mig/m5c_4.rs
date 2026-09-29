use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use tsox_core::tspath::{
    self, combine_paths, get_base_file_name, get_directory_path, ComparePathsOptions, Path,
};
use tsox_tsoptions::mig::m5i2_4::get_parsed_command_line_of_config_file_path;
use tsox_tsoptions::mig::m5j_2::ParseConfigHost;
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_tsoptions::vfs::FS;

use super::super::config_file_registry::{ConfigFileRegistry, ConfigFileEntry, ConfigFileNames, PendingReload};
use super::super::dirty_map_::DirtyMap;
use super::super::extended_config_cache::{ExtendedConfigCache, ExtendedConfigCacheEntry, ExtendedConfigParseArgs};
use super::super::compiler_host::SessionOptions;
use super::super::logging_log_tree::LogTree;
use super::super::logging_logger::Logger;
use super::super::project::Project;
use super::super::project_collection_builder::ProjectLoadKind;
use super::super::watch::{get_recursive_glob_pattern, PatternsAndIgnored, WatchedFiles};
use super::m5c_3::{
    collect_configured_content_mappers, new_config_file_entry, new_extended_config_file_entry,
    ConfiguredContentMappers,
};
use super::m5c_5::extended_config_entry_hash;

pub struct ConfigFileEntryFull {
    pub file_name: String,
    pub pending_reload: PendingReload,
    pub command_line: Option<ParsedCommandLine>,
    pub retaining_projects: HashMap<Path, ()>,
    pub retaining_open_files: HashMap<Path, ()>,
    pub retaining_configs: HashMap<Path, ()>,
    pub root_files_watch: Option<WatchedFiles<PatternsAndIgnored>>,
}

pub struct ConfigFileNamesFull {
    pub nearest_config_file_name: String,
    pub ancestors: HashMap<String, String>,
}

pub struct ChangeFileResultFull {
    pub affected_projects: HashMap<Path, ()>,
    pub affected_files: HashMap<Path, ()>,
}

const MIN_WATCH_LOCATION_DEPTH: usize = 2;

pub struct SyncMapEntryShared<K: Clone + Eq + std::hash::Hash, V> {
    key: K,
    value: RwLock<V>,
}

impl<K: Clone + Eq + std::hash::Hash, V> SyncMapEntryShared<K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn value(&self) -> std::sync::RwLockReadGuard<'_, V> {
        self.value.read().unwrap()
    }

    pub fn change_if<C, A>(&self, mut cond: C, mut apply: A) -> bool
    where
        C: FnMut(&V) -> bool,
        A: FnMut(&mut V),
    {
        let mut value = self.value.write().unwrap();
        if !cond(&value) {
            return false;
        }
        apply(&mut value);
        true
    }

    pub fn change<A: FnMut(&mut V)>(&self, mut apply: A) {
        let mut value = self.value.write().unwrap();
        apply(&mut value);
    }
}

pub struct DirtySyncMap<K: Clone + Eq + std::hash::Hash, V> {
    entries: RwLock<HashMap<K, Arc<SyncMapEntryShared<K, V>>>>,
}

impl<K: Clone + Eq + std::hash::Hash, V> DirtySyncMap<K, V> {
    pub fn new(base: HashMap<K, V>) -> Self {
        let mut entries = HashMap::new();
        for (key, value) in base {
            entries.insert(
                key.clone(),
                Arc::new(SyncMapEntryShared {
                    key,
                    value: RwLock::new(value),
                }),
            );
        }
        DirtySyncMap {
            entries: RwLock::new(entries),
        }
    }

    pub fn load(&self, key: &K) -> Option<Arc<SyncMapEntryShared<K, V>>> {
        self.entries.read().unwrap().get(key).cloned()
    }

    pub fn load_or_store(&self, key: K, value: V) -> (Arc<SyncMapEntryShared<K, V>>, bool) {
        let mut entries = self.entries.write().unwrap();
        if let Some(existing) = entries.get(&key) {
            return (existing.clone(), true);
        }
        let entry = Arc::new(SyncMapEntryShared {
            key: key.clone(),
            value: RwLock::new(value),
        });
        entries.insert(key, entry.clone());
        (entry, false)
    }

    pub fn range(&self, mut f: impl FnMut(&Arc<SyncMapEntryShared<K, V>>) -> bool) {
        for entry in self.entries.read().unwrap().values() {
            if !f(entry) {
                break;
            }
        }
    }

    pub fn delete(&self, key: &K) {
        self.entries.write().unwrap().remove(key);
    }
}

fn contains_path(parent: &str, target: &str, options: &ComparePathsOptions) -> bool {
    let parent = combine_paths(&options.current_directory, &[parent]);
    let target = combine_paths(&options.current_directory, &[target]);
    if parent.is_empty() || target.is_empty() {
        return false;
    }
    if parent == target {
        return true;
    }
    let parent_components = tspath::reduce_path_components(&tspath::get_path_components(&parent, ""));
    let target_components = tspath::reduce_path_components(&tspath::get_path_components(&target, ""));
    if target_components.len() < parent_components.len() {
        return false;
    }
    let case_sensitive = options.use_case_sensitive_file_names;
    for (i, parent_component) in parent_components.iter().enumerate() {
        let child_component = &target_components[i];
        let equal = if i == 0 {
            parent_component.eq_ignore_ascii_case(child_component)
        } else if case_sensitive {
            parent_component == child_component
        } else {
            parent_component.eq_ignore_ascii_case(child_component)
        };
        if !equal {
            return false;
        }
    }
    true
}

pub struct ConfigFileRegistryBuilderFull {
    pub has_relative_pattern_capability: bool,
    pub fs: Arc<dyn FS>,
    pub is_open_file: Box<dyn Fn(&Path) -> bool + Send + Sync>,
    pub extended_config_cache: Arc<ExtendedConfigCache>,
    pub snapshot_id: u64,
    pub session_options: SessionOptions,
    pub custom_config_file_name: String,
    pub base: ConfigFileRegistry,
    pub configs: DirtySyncMap<Path, ConfigFileEntryFull>,
    pub config_file_names: Mutex<DirtyMap<Path, ConfigFileNamesShared>>,
    pub custom_config_file_name_changed: bool,
    pub content_mappers_mu: Mutex<()>,
    pub all_configured_content_mappers: Mutex<Option<ConfiguredContentMappers>>,
}

pub type ConfigFileEntryShared = Arc<ConfigFileEntryFull>;
pub type ConfigFileNamesShared = Arc<ConfigFileNamesFull>;

fn convert_config_file_entry(entry: &ConfigFileEntry) -> ConfigFileEntryFull {
    ConfigFileEntryFull {
        file_name: entry.file_name.clone(),
        pending_reload: entry.pending_reload,
        command_line: entry.command_line.clone(),
        retaining_projects: entry.retaining_projects.clone(),
        retaining_open_files: entry.retaining_open_files.clone(),
        retaining_configs: entry.retaining_configs.clone(),
        root_files_watch: None,
    }
}

fn convert_config_file_names(names: &ConfigFileNames) -> ConfigFileNamesFull {
    ConfigFileNamesFull {
        nearest_config_file_name: names.nearest_config_file_name.clone(),
        ancestors: names.ancestors.clone(),
    }
}

pub fn new_config_file_registry_builder(
    has_relative_pattern_capability: bool,
    fs: crate::project::mig::m5e_7::SnapshotFSBuilder,
    old_config_file_registry: &ConfigFileRegistry,
    extended_config_cache: Arc<ExtendedConfigCache>,
    snapshot_id: u64,
    session_options: SessionOptions,
    custom_config_file_name: String,
) -> ConfigFileRegistryBuilderFull {
    let open_files: HashSet<Path> = fs.overlays.keys().cloned().collect();
    let configs: HashMap<Path, ConfigFileEntryFull> = old_config_file_registry
        .configs
        .iter()
        .map(|(path, entry)| (path.clone(), convert_config_file_entry(entry)))
        .collect();
    let config_file_names: HashMap<Path, ConfigFileNamesShared> = old_config_file_registry
        .config_file_names
        .iter()
        .map(|(path, names)| (path.clone(), Arc::new(convert_config_file_names(names))))
        .collect();
    ConfigFileRegistryBuilderFull {
        has_relative_pattern_capability,
        is_open_file: Box::new(move |path| open_files.contains(path)),
        fs: fs.fs.clone(),
        extended_config_cache,
        snapshot_id,
        custom_config_file_name_changed: custom_config_file_name
            != old_config_file_registry.custom_config_file_name,
        custom_config_file_name,
        base: old_config_file_registry.clone_shallow(),
        configs: DirtySyncMap::new(configs),
        config_file_names: Mutex::new(DirtyMap::new(config_file_names)),
        session_options,
        content_mappers_mu: Mutex::new(()),
        all_configured_content_mappers: Mutex::new(None),
    }
}

impl ConfigFileRegistryBuilderFull {
    pub fn fs_handle(&self) -> &dyn FS {
        self.fs.as_ref()
    }

    pub fn get_current_directory(&self) -> &str {
        &self.session_options.current_directory
    }

    pub fn parse_config_host(&self) -> ParseConfigHost {
        ParseConfigHost {
            fs: self.fs.clone(),
            current_directory: self.session_options.current_directory.clone(),
        }
    }

    pub fn get_extended_config(
        &self,
        file_name: &str,
        path: &Path,
        resolution_stack: Vec<Path>,
        host: &ParseConfigHost,
    ) -> ExtendedConfigCacheEntry {
        let content = self
            .fs
            .read_file(file_name)
            .unwrap_or_default();
        let args = ExtendedConfigParseArgs {
            file_name: file_name.to_string(),
            content,
            resolution_stack,
        };
        let fs = self.fs.clone();
        self.extended_config_cache
            .load_and_acquire(path, self.snapshot_id, |path| {
                let resolution_stack: Vec<String> = args
                    .resolution_stack
                    .iter()
                    .map(|p| p.0.clone())
                    .collect();
                let parsed = tsox_tsoptions::mig::m5i2_4::parse_extended_config(
                    &args.file_name,
                    path.as_str(),
                    &resolution_stack,
                    host,
                    None,
                );
                let extended_source_files = parsed
                    .extended_result
                    .as_ref()
                    .map(|result| result.extended_source_files.clone())
                    .unwrap_or_default();
                let (hash_lo, hash_hi) =
                    extended_config_entry_hash(&args, &extended_source_files, fs.as_ref());
                ExtendedConfigCacheEntry {
                    command_line: parsed.extended_config,
                    hash_lo,
                    hash_hi,
                }
            })
    }

    pub fn content_mappers(&self) -> ConfiguredContentMappers {
        let _guard = self.content_mappers_mu.lock().unwrap();
        let mut cached = self.all_configured_content_mappers.lock().unwrap();
        if let Some(all) = cached.as_ref() {
            return all.clone();
        }
        let mut command_line_values: Vec<ParsedCommandLine> = Vec::new();
        self.configs.range(|entry| {
            if let Some(command_line) = entry.value().command_line.clone() {
                command_line_values.push(command_line);
            }
            true
        });
        let command_lines: Vec<&ParsedCommandLine> = command_line_values.iter().collect();
        let all = collect_configured_content_mappers(&command_lines);
        *cached = Some(all.clone());
        all
    }

    pub fn invalidate_content_mappers(&self) {
        let _guard = self.content_mappers_mu.lock().unwrap();
        *self.all_configured_content_mappers.lock().unwrap() = None;
    }

    pub fn find_or_acquire_config_for_file(
        &self,
        config_file_name: &str,
        config_file_path: &Path,
        file_path: &Path,
        load_kind: ProjectLoadKind,
        logger: &dyn Logger,
    ) -> Option<ParsedCommandLine> {
        match load_kind {
            ProjectLoadKind::Find => self
                .configs
                .load(config_file_path)
                .and_then(|entry| entry.value().command_line.clone()),
            ProjectLoadKind::Create => Some(self.acquire_config_for_file(
                config_file_name,
                config_file_path,
                file_path,
                logger,
            )),
        }
    }

    pub fn reload_if_needed(
        &self,
        entry: &mut ConfigFileEntryFull,
        file_name: &str,
        path: &Path,
        logger: &dyn Logger,
    ) -> bool {
        let old_command_line = entry.command_line.clone();
        match entry.pending_reload {
            PendingReload::FileNames => {
                logger.log(&format!("Reloading file names for config: {}", file_name));
                entry.command_line = entry
                    .command_line
                    .as_ref()
                    .map(|cl| cl.reload_file_names_of_parsed_command_line(self.fs.as_ref()));
            }
            PendingReload::Full => {
                logger.log(&format!("Loading config file: {}", file_name));
                let host = self.parse_config_host();
                let (command_line, _) = get_parsed_command_line_of_config_file_path(
                    file_name,
                    path.as_str(),
                    None,
                    None,
                    &host,
                    None,
                );
                entry.command_line = command_line;
                self.update_extending_configs(path, &entry.command_line, &old_command_line);
                self.update_root_files_watch(file_name, entry);
                logger.log("Finished loading config file");
            }
            PendingReload::None => return false,
        }
        entry.pending_reload = PendingReload::None;
        format!("{:?}", old_command_line) != format!("{:?}", entry.command_line)
    }

    pub fn update_extending_configs(
        &self,
        extending_config_path: &Path,
        new_command_line: &Option<ParsedCommandLine>,
        old_command_line: &Option<ParsedCommandLine>,
    ) {
        let mut new_extended_config_paths = HashSet::new();
        if let Some(new_command_line) = new_command_line {
            for extended_config in new_command_line.extended_source_files() {
                let extended_config_path = (self.to_path_fn())(extended_config);
                new_extended_config_paths.insert(extended_config_path.clone());
                let (entry, loaded) = self.configs.load_or_store(
                    extended_config_path.clone(),
                    convert_config_file_entry(&new_extended_config_file_entry(
                        extended_config,
                        extending_config_path.clone(),
                    )),
                );
                if loaded {
                    entry.change_if(
                        |config| !config.retaining_configs.contains_key(extending_config_path),
                        |config| {
                            config
                                .retaining_configs
                                .insert(extending_config_path.clone(), ());
                        },
                    );
                }
            }
        }
        if let Some(old_command_line) = old_command_line {
            for extended_config in old_command_line.extended_source_files() {
                let extended_config_path = (self.to_path_fn())(extended_config);
                if new_extended_config_paths.contains(&extended_config_path) {
                    continue;
                }
                if let Some(entry) = self.configs.load(&extended_config_path) {
                    entry.change_if(
                        |config| config.retaining_configs.contains_key(extending_config_path),
                        |config| {
                            config.retaining_configs.remove(extending_config_path);
                        },
                    );
                }
            }
        }
    }

    fn to_path_fn(&self) -> impl Fn(&str) -> Path + '_ {
        let current_directory = self.session_options.current_directory.clone();
        move |name: &str| Path(tspath::combine_paths(&current_directory, &[name]))
    }

    pub fn update_root_files_watch(&self, file_name: &str, entry: &mut ConfigFileEntryFull) {
        let Some(_watch) = entry.root_files_watch.as_ref() else {
            return;
        };
        let mut ignored: HashSet<String> = HashSet::new();
        let mut globs: Vec<String> = Vec::new();
        let mut external_directories: Vec<String> = Vec::new();
        let mut include_workspace = false;
        let mut include_tsconfig_dir = false;
        let tsconfig_dir = get_directory_path(file_name);
        let command_line = entry.command_line.as_ref().expect("command line");
        let wildcard_directories = command_line.wildcard_directories();
        let compare_paths_options = ComparePathsOptions {
            current_directory: self.session_options.current_directory.clone(),
            use_case_sensitive_file_names: self.fs.use_case_sensitive_file_names(),
        };
        for dir in wildcard_directories.keys() {
            if contains_path(
                &self.session_options.current_directory,
                dir,
                &compare_paths_options,
            ) {
                include_workspace = true;
            } else if contains_path(&tsconfig_dir, dir, &compare_paths_options) {
                include_tsconfig_dir = true;
            } else {
                external_directories.push(dir.clone());
            }
        }
        for literal_file_name in command_line.literal_file_names() {
            if contains_path(
                &self.session_options.current_directory,
                literal_file_name,
                &compare_paths_options,
            ) {
                include_workspace = true;
            } else if contains_path(&tsconfig_dir, literal_file_name, &compare_paths_options) {
                include_tsconfig_dir = true;
            } else {
                external_directories.push(get_directory_path(literal_file_name));
            }
        }
        if include_workspace {
            globs.push(get_recursive_glob_pattern(&self.session_options.current_directory));
        }
        if include_tsconfig_dir {
            globs.push(get_recursive_glob_pattern(&tsconfig_dir));
        }
        for extended_file_name in command_line.extended_source_files() {
            if include_workspace
                && contains_path(
                    &self.session_options.current_directory,
                    extended_file_name,
                    &compare_paths_options,
                )
            {
                continue;
            }
            globs.push(extended_file_name.clone());
        }
        if !external_directories.is_empty() {
            let (common_parents, ignored_external_dirs) = tspath::get_common_parents(
                &external_directories,
                MIN_WATCH_LOCATION_DEPTH,
                &compare_paths_options,
            );
            for parent in common_parents {
                globs.push(get_recursive_glob_pattern(&parent));
            }
            ignored = ignored_external_dirs;
        }
        globs.sort();
        entry.root_files_watch = Some(entry.root_files_watch.as_ref().unwrap().clone_with_input(
            PatternsAndIgnored {
                patterns_inside_workspace: globs,
                ignored,
                ..PatternsAndIgnored::default()
            },
        ));
    }

    pub fn acquire_config_for_project(
        &self,
        file_name: &str,
        path: &Path,
        project: &Project,
        logger: &dyn Logger,
    ) -> Option<ParsedCommandLine> {
        let (entry, _) = self.configs.load_or_store(
            path.clone(),
            convert_config_file_entry(&new_config_file_entry(
                self.has_relative_pattern_capability,
                file_name,
            )),
        );
        let mut needs_retain_project = false;
        {
            let config = entry.value();
            needs_retain_project = !config.retaining_projects.contains_key(&project.config_file_path);
        }
        let mut content_mappers_changed = false;
        if needs_retain_project || entry.value().pending_reload != PendingReload::None {
            entry.change(|config| {
                if needs_retain_project {
                    config
                        .retaining_projects
                        .insert(project.config_file_path.clone(), ());
                }
                content_mappers_changed =
                    self.reload_if_needed(config, file_name, path, logger);
            });
        }
        if content_mappers_changed {
            self.invalidate_content_mappers();
        }
        entry.value().command_line.clone()
    }

    pub fn acquire_config_for_file(
        &self,
        config_file_name: &str,
        config_file_path: &Path,
        file_path: &Path,
        logger: &dyn Logger,
    ) -> ParsedCommandLine {
        let (entry, _) = self.configs.load_or_store(
            config_file_path.clone(),
            convert_config_file_entry(&new_config_file_entry(
                self.has_relative_pattern_capability,
                config_file_name,
            )),
        );
        let mut needs_retain_open_file = false;
        if (self.is_open_file)(file_path) {
            needs_retain_open_file = !entry.value().retaining_open_files.contains_key(file_path);
        }
        let mut content_mappers_changed = false;
        if needs_retain_open_file || entry.value().pending_reload != PendingReload::None {
            entry.change(|config| {
                if needs_retain_open_file {
                    config.retaining_open_files.insert(file_path.clone(), ());
                }
                content_mappers_changed =
                    self.reload_if_needed(config, config_file_name, config_file_path, logger);
            });
        }
        if content_mappers_changed {
            self.invalidate_content_mappers();
        }
        entry.value().command_line.clone().expect("command line")
    }

    pub fn release_config_for_project(&self, config_file_path: &Path, project_path: &Path) {
        if let Some(entry) = self.configs.load(config_file_path) {
            entry.change_if(
                |config| config.retaining_projects.contains_key(project_path),
                |config| {
                    config.retaining_projects.remove(project_path);
                },
            );
        }
    }

    pub fn retain_config_for_project(&self, config_file_path: &Path, project_path: &Path) {
        if let Some(entry) = self.configs.load(config_file_path) {
            entry.change_if(
                |config| !config.retaining_projects.contains_key(project_path),
                |config| {
                    config
                        .retaining_projects
                        .insert(project_path.clone(), ());
                },
            );
        }
    }

    pub fn did_close_file(&self, path: &Path) {
        if tspath::is_dynamic_file_name(&path.0) {
            return;
        }
        self.config_file_names.lock().unwrap().delete(path);
        self.configs.range(|entry| {
            entry.change_if(
                |config| config.retaining_open_files.contains_key(path),
                |config| {
                    config.retaining_open_files.remove(path);
                },
            );
            true
        });
    }

    pub fn handle_config_change(
        &self,
        entry: &SyncMapEntryShared<Path, ConfigFileEntryFull>,
        logger: &dyn Logger,
    ) -> HashMap<Path, ()> {
        let mut affected_projects = HashMap::new();
        let changed = entry.change_if(
            |config| config.pending_reload != PendingReload::Full,
            |config| config.pending_reload = PendingReload::Full,
        );
        if changed {
            logger.log(&format!("Config file {} changed", entry.key().0));
            affected_projects = entry.value().retaining_projects.clone();
        }
        affected_projects
    }

    pub fn invalidate_cache(&self, logger: &dyn Logger) -> ChangeFileResultFull {
        let mut affected_projects: HashMap<Path, ()> = HashMap::new();
        let mut affected_files: HashMap<Path, ()> = HashMap::new();
        logger.log("Too many files changed; marking all configs for reload");
        self.config_file_names.lock().unwrap().range(|entry| {
            affected_files.insert(entry.key().clone(), ());
            true
        });
        self.config_file_names.lock().unwrap().clear();
        self.configs.range(|entry| {
            entry.change(|config| {
                for project_path in config.retaining_projects.keys() {
                    affected_projects.insert(project_path.clone(), ());
                }
                if config.pending_reload != PendingReload::Full {
                    let text = self.fs.read_file(&config.file_name);
                    let config_file_text = config
                        .command_line
                        .as_ref()
                        .and_then(|command_line| command_line.config_file.as_ref())
                        .map(|config_file| config_file.source_file.text.clone());
                    let matches = match (&text, &config_file_text) {
                        (Some(text), Some(config_text)) => text == config_text,
                        _ => false,
                    };
                    config.pending_reload = if text.is_none()
                        || config.command_line.is_none()
                        || !matches
                    {
                        PendingReload::Full
                    } else {
                        PendingReload::FileNames
                    };
                }
            });
            true
        });
        ChangeFileResultFull {
            affected_projects,
            affected_files,
        }
    }
}

pub fn content_mapper_manifest_path(
    command_line: Option<&ParsedCommandLine>,
    to_path: &dyn Fn(&str) -> Path,
    path: &Path,
) -> bool {
    let Some(command_line) = command_line else {
        return false;
    };
    for mapper in command_line.content_mappers() {
        if !mapper.definition.package.is_empty()
            && mapper.contribution_id.is_empty()
            && !mapper.package_directory.is_empty()
            && to_path(&combine_paths(&mapper.package_directory, &["package.json"])) == *path
        {
            return true;
        }
    }
    false
}

impl ConfigFileRegistryBuilderFull {
    pub fn compute_config_file_name(
        &self,
        file_name: &str,
        skip_search_in_directory_of_file: bool,
        logger: &dyn Logger,
    ) -> String {
        let search_path = get_directory_path(file_name);
        if !self.custom_config_file_name.is_empty() {
            let mut skip = skip_search_in_directory_of_file;
            let custom_config = self.custom_config_file_name.clone();
            let mut found: Option<String> = None;
            tspath::for_each_ancestor_directory(&search_path, |directory| {
                if !skip {
                    let custom_path = combine_paths(directory, &[custom_config.as_str()]);
                    if self.fs.file_exists(&custom_path) {
                        found = Some(custom_path);
                        return true;
                    }
                }
                if directory.ends_with("/node_modules") {
                    found = Some(String::new());
                    return true;
                }
                skip = false;
                false
            });
            if let Some(result) = found.filter(|r| !r.is_empty()) {
                logger.log(&format!(
                    "computeConfigFileName:: File: {}:: Result: {}",
                    file_name, result
                ));
                return result;
            }
        }
        let mut skip_tsconfig = skip_search_in_directory_of_file;
        let mut skip_jsconfig =
            skip_search_in_directory_of_file && !file_name.ends_with("/tsconfig.json");
        let mut found: Option<String> = None;
        tspath::for_each_ancestor_directory(&search_path, |directory| {
            if !skip_tsconfig {
                let tsconfig_path = combine_paths(directory, &["tsconfig.json"]);
                if self.fs.file_exists(&tsconfig_path) {
                    found = Some(tsconfig_path);
                    return true;
                }
            }
            if !skip_jsconfig {
                let jsconfig_path = combine_paths(directory, &["jsconfig.json"]);
                if self.fs.file_exists(&jsconfig_path) {
                    found = Some(jsconfig_path);
                    return true;
                }
            }
            if directory.ends_with("/node_modules") {
                found = Some(String::new());
                return true;
            }
            skip_tsconfig = false;
            skip_jsconfig = false;
            false
        });
        let result = found.unwrap_or_default();
        logger.log(&format!(
            "computeConfigFileName:: File: {}:: Result: {}",
            file_name, result
        ));
        result
    }

    pub fn get_config_file_name_for_file(
        &self,
        file_name: &str,
        path: &Path,
        logger: &dyn Logger,
    ) -> String {
        if tspath::is_dynamic_file_name(file_name) {
            return String::new();
        }
        if let Some(entry) = self.config_file_names.lock().unwrap().get(path) {
            return entry.value().nearest_config_file_name.clone();
        }
        let config_name = self.compute_config_file_name(file_name, false, logger);
        if (self.is_open_file)(path) {
            self.config_file_names.lock().unwrap().add(
                path.clone(),
                Arc::new(ConfigFileNamesFull {
                    nearest_config_file_name: config_name.clone(),
                    ancestors: HashMap::new(),
                }),
            );
        }
        config_name
    }

    pub fn for_each_config_file_name_for(&self, path: &Path, mut cb: impl FnMut(&str)) {
        if tspath::is_dynamic_file_name(&path.0) {
            return;
        }
        let entry = self.config_file_names.lock().unwrap().get(path);
        if let Some(entry) = entry {
            let mut config_file_name = entry.value().nearest_config_file_name.clone();
            while !config_file_name.is_empty() {
                cb(&config_file_name);
                match entry.value().ancestors.get(&config_file_name) {
                    Some(ancestor) => config_file_name = ancestor.clone(),
                    None => return,
                }
            }
        }
    }
}
