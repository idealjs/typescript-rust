#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::project_reference::ProjectReference;
use tsox_core::tspath::Path;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use crate::lsp::lsproto;
use crate::project::config_file_registry::ConfigFileRegistry;
use crate::project::config_file_registry_builder::ConfigFileRegistryBuilder;
use crate::project::dirty_box_::DirtyBox;
use crate::project::dirty_map_::{DirtyMap, MapEntry};
use crate::project::file_change::FileChangeSummary;
use crate::project::logging_log_tree::LogTree;
use crate::project::mig::m5d::LogTreeMigExt;
use crate::project::mig::m5d_3::{
    new_configured_project, new_inferred_project, new_inferred_project_command_line,
    new_inferred_project_from_project,
};
use crate::project::mig::m5e_7::SnapshotFSBuilder;
use crate::project::overlay_fs::FileHandle;
use crate::project::compiler_host::SessionOptions;
use crate::project::extended_config_cache::ExtendedConfigCache;
use crate::project::parse_cache::ParseCache;
use crate::project::project::{Kind, Project, INFERRED_PROJECT_NAME};
use crate::project::project_collection::{APIState, ApiOpenedFile, ProjectCollection};
use crate::project::snapshot::{ATAStateChange, APISnapshotRequest};
use crate::ls::lsutil::UserPreferences;
use crate::mig::m5m::ResolvedClientCapabilitiesContext;

pub const PROJECT_LOAD_KIND_FIND: ProjectLoadKind = ProjectLoadKind::Find;
pub const PROJECT_LOAD_KIND_CREATE: ProjectLoadKind = ProjectLoadKind::Create;

pub struct ProjectCollectionBuilderMig<'a> {
    pub session_options: SessionOptions,
    pub new_snapshot_id: u64,
    pub program_structure_changed: bool,
    pub default_projects_invalidated: bool,
    pub open_files_changed: bool,
    pub file_default_projects: HashMap<Path, Path>,
    pub api_state: APIState,
    pub parse_cache: std::sync::Arc<ParseCache>,
    pub content_mapped_parse_cache: std::sync::Arc<ParseCache>,
    pub extended_config_cache: std::sync::Arc<ExtendedConfigCache>,
    pub to_path: std::sync::Arc<dyn Fn(&str) -> Path + Send + Sync>,
    pub ctx: ResolvedClientCapabilitiesContext,
    pub fs: &'a mut SnapshotFSBuilder,
    pub base: ProjectCollection,
    pub compiler_options_for_inferred_projects: Option<CompilerOptions>,
    pub inferred_content_mappers: Vec<tsox_compile::mig::m3l_cm::Mapper>,
    pub inferred_content_mapper_extensions: Vec<String>,
    pub config_file_registry_builder: ConfigFileRegistryBuilder,
    pub configured_projects: DirtyMap<Path, Box<Project>>,
    pub inferred_project: DirtyBox<Option<Box<Project>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectLoadKind {
    Find,
    Create,
}

pub struct SearchResult {
    pub project: Option<MapEntry<Path, Box<Project>>>,
    pub retain: HashSet<Path>,
}

pub fn new_project_collection_builder<'a>(
    ctx: ResolvedClientCapabilitiesContext,
    new_snapshot_id: u64,
    fs: &'a mut SnapshotFSBuilder,
    old_project_collection: &ProjectCollection,
    old_config_file_registry: &ConfigFileRegistry,
    old_api_state: &APIState,
    compiler_options_for_inferred_projects: Option<CompilerOptions>,
    inferred_content_mappers: Vec<tsox_compile::mig::m3l_cm::Mapper>,
    inferred_content_mapper_extensions: Vec<String>,
    session_options: SessionOptions,
    custom_config_file_name: String,
    parse_cache: std::sync::Arc<ParseCache>,
    content_mapped_parse_cache: std::sync::Arc<ParseCache>,
    extended_config_cache: std::sync::Arc<ExtendedConfigCache>,
) -> ProjectCollectionBuilderMig<'a> {
    let to_path = fs.to_path.clone();
    let config_file_registry_builder = ConfigFileRegistryBuilder::new(
        client_capabilities_relative_pattern_support(&ctx),
        old_config_file_registry.clone_shallow(),
        new_snapshot_id,
        session_options.clone(),
        custom_config_file_name.clone(),
    );
    let configured_projects = DirtyMap::new(old_project_collection.configured_projects.clone());
    let inferred_project = DirtyBox::new(old_project_collection.inferred_project.clone());
    let base_to_path = to_path.clone();
    ProjectCollectionBuilderMig {
        ctx,
        fs,
        to_path,
        compiler_options_for_inferred_projects,
        inferred_content_mappers,
        inferred_content_mapper_extensions,
        session_options,
        parse_cache,
        content_mapped_parse_cache,
        extended_config_cache,
        base: ProjectCollection {
            to_path: Box::new(move |file_name: &str| (base_to_path)(file_name)),
            config_file_registry: old_project_collection
                .config_file_registry
                .as_ref()
                .map(|r| r.clone_shallow()),
            file_default_projects: old_project_collection.file_default_projects.clone(),
            configured_projects: old_project_collection.configured_projects.clone(),
            open_files: old_project_collection.open_files.clone(),
            inferred_project: old_project_collection.inferred_project.clone(),
            api_state: old_api_state.clone(),
        },
        config_file_registry_builder,
        new_snapshot_id,
        configured_projects,
        inferred_project,
        api_state: old_api_state.clone(),
        file_default_projects: HashMap::new(),
        program_structure_changed: false,
        default_projects_invalidated: false,
        open_files_changed: false,
    }
}

impl ProjectCollectionBuilderMig<'_> {
    pub fn to_path_of(&self, file_name: &str) -> Path { ::tsox_core::fntrace::enter("to_path_of"); 
        (self.to_path)(file_name)
    }

    pub fn for_each_project(&self, mut f: impl FnMut(&MapEntry<Path, Box<Project>>) -> bool) { ::tsox_core::fntrace::enter("for_each_project"); 
        let mut keep_going = true;
        self.configured_projects.range(|entry: &MapEntry<Path, Box<Project>>| {
            keep_going = f(entry);
            keep_going
        });
        if !keep_going {
            return;
        }
        if self.inferred_project.value().is_some() {
            f(self.inferred_project_entry());
        }
    }

    pub fn inferred_project_entry(&self) -> &MapEntry<Path, Box<Project>> { ::tsox_core::fntrace::enter("inferred_project_entry"); 
        todo!("DirtyBox is not a MapEntry; integration must unify dirty.Value semantics")
    }

    pub fn cleanup_inferred_project(&mut self, logger: Option<&LogTree>) { ::tsox_core::fntrace::enter("cleanup_inferred_project"); 
        let roots = self.collect_inferred_project_roots();
        self.update_inferred_project_roots(roots, logger);
    }

    pub fn did_change_content_mapper_contributions(&mut self, logger: Option<&LogTree>) { ::tsox_core::fntrace::enter("did_change_content_mapper_contributions"); 
        self.cleanup_inferred_project(logger);
        if self.inferred_project.value().is_some() {
            let entry = self.inferred_project_entry().clone();
            self.update_program(&entry, logger);
        }
    }

    pub fn ensure_inferred_project_includes_closed_file(
        &mut self,
        file_name: &str,
        logger: Option<&LogTree>,
    ) { ::tsox_core::fntrace::enter("ensure_inferred_project_includes_closed_file"); 
        let mut inferred_project_files = self.collect_inferred_project_roots();
        inferred_project_files.push(file_name.to_string());
        self.update_inferred_project_roots(inferred_project_files, logger);
        if self.inferred_project.value().is_some() {
            let entry = self.inferred_project_entry().clone();
            self.update_program(&entry, logger);
        }
    }

    pub fn collect_inferred_project_roots(&mut self) -> Vec<String> { ::tsox_core::fntrace::enter("collect_inferred_project_roots"); 
        let mut inferred_project_files: Vec<String> = Vec::new();
        let overlays: Vec<(Path, String)> = self
            .fs
            .overlays
            .iter()
            .map(|(path, overlay)| (path.clone(), overlay.file_name().to_string()))
            .collect();
        for (path, overlay_file_name) in &overlays {
            if self
                .find_default_configured_project(overlay_file_name, path)
                .is_none()
            {
                inferred_project_files.push(overlay_file_name.clone());
            }
        }
        self.append_api_opened_inferred_roots(inferred_project_files)
    }

    pub fn append_api_opened_inferred_roots(
        &mut self,
        mut inferred_project_files: Vec<String>,
    ) -> Vec<String> { ::tsox_core::fntrace::enter("append_api_opened_inferred_roots"); 
        let open_files: Vec<(Path, String)> = self
            .api_state
            .open_files
            .iter()
            .map(|(path, file)| (path.clone(), file.file_name.clone()))
            .collect();
        for (path, file_name) in &open_files {
            if self.fs.is_open_file(path) {
                continue;
            }
            if self
                .find_default_configured_project(file_name, path)
                .is_none()
            {
                inferred_project_files.push(file_name.clone());
            }
        }
        inferred_project_files
    }

    pub fn cleanup_all_configured_projects(&mut self, logger: Option<&LogTree>) { ::tsox_core::fntrace::enter("cleanup_all_configured_projects"); 
        let keys: Vec<Path> = self.configured_projects.keys();
        for key in keys {
            if let Some(entry) = self.configured_projects.load(&key) {
                self.delete_configured_project(entry, logger);
            }
        }
        self.config_file_registry_builder.cleanup();
    }

    pub fn find_default_project(
        &mut self,
        _file_name: &str,
        path: &Path,
    ) -> Option<MapEntry<Path, Box<Project>>> { ::tsox_core::fntrace::enter("find_default_project"); 
        if let Some(configured_project) = self.find_default_configured_project_entry(path) {
            return Some(configured_project);
        }
        if let Some(key) = self.file_default_projects.get(path) {
            if key.as_str() == INFERRED_PROJECT_NAME {
                return Some(self.inferred_project_entry().clone());
            }
        }
        let inferred_contains = self
            .inferred_project
            .value()
            .as_ref()
            .map(|p| p.contains_file_in_program(path))
            .unwrap_or(false);
        if inferred_contains {
            self.file_default_projects
                .insert(path.clone(), Path(INFERRED_PROJECT_NAME.to_string()));
            return Some(self.inferred_project_entry().clone());
        }
        None
    }

    pub fn find_default_configured_project_entry(
        &self,
        path: &Path,
    ) -> Option<MapEntry<Path, Box<Project>>> { ::tsox_core::fntrace::enter("find_default_configured_project_entry"); 
        None
    }

    pub fn find_default_configured_project(
        &mut self,
        file_name: &str,
        path: &Path,
    ) -> Option<MapEntry<Path, Box<Project>>> { ::tsox_core::fntrace::enter("find_default_configured_project"); 
        if let Some(key) = self.file_default_projects.get(path) {
            if key.as_str() != INFERRED_PROJECT_NAME {
                if let Some(entry) = self.configured_projects.load(key) {
                    return Some(entry);
                }
            }
        }
        let mut configured_project_paths: Vec<Path> = Vec::new();
        let mut configured_projects: HashMap<Path, MapEntry<Path, Box<Project>>> = HashMap::new();
        self.configured_projects.range(|entry: &MapEntry<Path, Box<Project>>| {
            configured_project_paths.push(entry.key().clone());
            configured_projects.insert(entry.key().clone(), entry.clone());
            true
        });
        configured_project_paths.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        let (project, multiple_candidates) =
            crate::project::mig::m5d_4::find_default_configured_project_from_program_inclusion(
                file_name,
                path,
                &configured_project_paths,
                &|p: &Path| configured_projects.get(p).map(|e| e.value.as_ref()),
            );

        if multiple_candidates {
            if let Some(p) = self
                .find_or_create_default_configured_project_for_file(
                    file_name,
                    path,
                    ProjectLoadKind::Find,
                    None,
                )
                .project
            {
                return Some(p);
            }
        }
        configured_projects.get(&project).cloned()
    }

    pub fn find_or_create_default_configured_project_for_file(
        &mut self,
        file_name: &str,
        path: &Path,
        load_kind: ProjectLoadKind,
        logger: Option<&LogTree>,
    ) -> SearchResult { ::tsox_core::fntrace::enter("find_or_create_default_configured_project_for_file"); 
        if let Some(key) = self.file_default_projects.get(path).cloned() {
            if key.as_str() == INFERRED_PROJECT_NAME {
                return SearchResult {
                    project: None,
                    retain: HashSet::new(),
                };
            }
            let entry = self.configured_projects.load(&key);
            return SearchResult {
                project: entry,
                retain: HashSet::new(),
            };
        }
        let config_file_name = self
            .config_file_registry_builder
            .get_config_file_name_for_file(file_name, path, logger);
        if config_file_name.is_empty() {
            return SearchResult {
                project: None,
                retain: HashSet::new(),
            };
        }
        let mut result = self.find_or_create_default_configured_project_worker(
            file_name,
            path,
            &config_file_name,
            load_kind,
            &mut HashSet::new(),
            None,
            logger,
        );
        if let Some(project) = &result.project {
            self.file_default_projects.insert(
                path.clone(),
                project.value().config_file_path.clone(),
            );
        }
        result
    }

    pub fn find_or_create_default_configured_project_worker(
        &mut self,
        file_name: &str,
        path: &Path,
        config_file_name: &str,
        load_kind: ProjectLoadKind,
        visited: &mut HashSet<SearchNodeKey>,
        fallback: Option<SearchResult>,
        logger: Option<&LogTree>,
    ) -> SearchResult { ::tsox_core::fntrace::enter("find_or_create_default_configured_project_worker"); 
        let _ = (file_name, path, config_file_name, load_kind, visited, fallback, logger);
        todo!("BreadthFirstSearchParallelEx over searchNode pending core BFS helper alignment")
    }

    pub fn ensure_configured_project_and_ancestors_for_file(
        &mut self,
        file_name: &str,
        path: &Path,
        logger: Option<&LogTree>,
    ) -> SearchResult { ::tsox_core::fntrace::enter("ensure_configured_project_and_ancestors_for_file"); 
        let mut result =
            self.find_or_create_default_configured_project_for_file(file_name, path, ProjectLoadKind::Create, logger);
        if result.project.is_some() && self.fs.is_open_file(path) {
            self.create_ancestor_tree(file_name, path, &mut result, logger);
        }
        result
    }

    pub fn create_ancestor_tree(
        &mut self,
        file_name: &str,
        path: &Path,
        open_result: &mut SearchResult,
        logger: Option<&LogTree>,
    ) { ::tsox_core::fntrace::enter("create_ancestor_tree"); 
        let mut next_project = open_result
            .project
            .as_ref()
            .map(|e| e.value())
            .map(|p| p.clone_project());
        loop {
            let project = match next_project {
                Some(p) => p,
                None => return,
            };
            if let Some(command_line) = &project.command_line {
                let options = command_line.compiler_options();
                if options.composite != tsox_core::core::tristate::Tristate::True
                    || options.disable_solution_searching == tsox_core::core::tristate::Tristate::True
                {
                    return;
                }
            }
            let ancestor_config_name = self
                .config_file_registry_builder
                .get_ancestor_config_file_name(file_name, path, &project.config_file_name, logger);
            if ancestor_config_name.is_empty() {
                return;
            }
            let ancestor_path = self.to_path_of(&ancestor_config_name);
            let ancestor = self.find_or_create_project(
                &ancestor_config_name,
                &ancestor_path,
                ProjectLoadKind::Create,
                logger,
            );
            let ancestor = match ancestor {
                Some(a) => a,
                None => return,
            };
            open_result.retain.insert(ancestor_path);
            let ancestor_command_line_missing = {
                let ancestor_project = ancestor.value();
                ancestor_project.command_line.is_none()
            };
            if ancestor_command_line_missing && project.command_line.is_some() {
                let reference = project.config_file_path.clone();
                self.change_entry(&ancestor, |ancestor_project: &mut Project| {
                    ancestor_project.set_potential_project_reference(reference.clone());
                });
            }
            next_project = Some(ancestor.value().clone_project());
        }
    }

    pub(crate) fn change_entry(
        &mut self,
        entry: &MapEntry<Path, Box<Project>>,
        apply: impl FnOnce(&mut Project),
    ) { ::tsox_core::fntrace::enter("change_entry"); 
        let key = entry.key().clone();
        self.configured_projects
            .change(&key, |value: &mut Box<Project>| apply(value.as_mut()));
    }

    fn project_collection_builder_seed(&self) -> crate::project::project_collection_builder::ProjectCollectionBuilder { ::tsox_core::fntrace::enter("project_collection_builder_seed"); 
        crate::project::project_collection_builder::ProjectCollectionBuilder::new(
            self.new_snapshot_id,
            self.compiler_options_for_inferred_projects.as_ref(),
            self.session_options.clone(),
        )
    }

    pub fn find_or_create_project(
        &mut self,
        config_file_name: &str,
        config_file_path: &Path,
        load_kind: ProjectLoadKind,
        logger: Option<&LogTree>,
    ) -> Option<MapEntry<Path, Box<Project>>> { ::tsox_core::fntrace::enter("find_or_create_project"); 
        if load_kind == ProjectLoadKind::Find {
            return self.configured_projects.load(config_file_path);
        }
        let project = new_configured_project(
            config_file_name.to_string(),
            config_file_path.clone(),
            &self.project_collection_builder_seed(),
            logger,
        );
        self.configured_projects
            .load_or_store(config_file_path.clone(), Box::new(project))
    }

    pub fn update_inferred_project_roots(
        &mut self,
        root_file_names: Vec<String>,
        logger: Option<&LogTree>,
    ) -> bool { ::tsox_core::fntrace::enter("update_inferred_project_roots"); 
        let root_file_names: Vec<String> = root_file_names
            .into_iter()
            .filter(|f| self.is_supported_in_inferred_project(f))
            .collect();
        let mut project_references = Vec::new();
        let mut config_file_parsing_diagnostics = Vec::new();
        if let Some(project) = self.inferred_project.value().as_ref() {
            if let Some(command_line) = &project.command_line {
                project_references = command_line.project_references().to_vec();
                config_file_parsing_diagnostics = command_line.errors.clone();
            }
        }
        self.update_inferred_project(
            root_file_names,
            self.compiler_options_for_inferred_projects.clone(),
            project_references,
            config_file_parsing_diagnostics,
            self.inferred_content_mappers.clone(),
            logger,
        )
    }

    pub fn seed_inferred_project_for_program(
        &mut self,
        project: &Project,
        logger: Option<&LogTree>,
    ) { ::tsox_core::fntrace::enter("seed_inferred_project_for_program"); 
        if project.program.is_none() {
            return;
        }
        let inferred_project = new_inferred_project_from_project(
            project,
            &self.project_collection_builder_seed(),
            logger,
        );
        if let Some(program) = &project.program {
            program.range_resolved_project_reference(&mut |reference_path: &Path,
                                                          _config: Option<&std::sync::Arc<ParsedCommandLine>>,
                                                          _base: Option<&std::sync::Arc<ParsedCommandLine>>,
                                                          _index: usize| {
                self.config_file_registry_builder.retain_config_for_project(
                    reference_path,
                    &inferred_project.config_file_path,
                );
                true
            });
        }
        self.inferred_project.set(Some(Box::new(inferred_project)));
    }

    pub fn update_inferred_project(
        &mut self,
        root_file_names: Vec<String>,
        compiler_options: Option<CompilerOptions>,
        project_references: Vec<ProjectReference>,
        config_file_parsing_diagnostics: Vec<tsox_frontend::ast::Diagnostic>,
        content_mappers: Vec<tsox_compile::mig::m3l_cm::Mapper>,
        logger: Option<&LogTree>,
    ) -> bool { ::tsox_core::fntrace::enter("update_inferred_project"); 
        if root_file_names.is_empty() {
            if self.inferred_project.value().is_some() {
                if let Some(logger) = logger {
                    logger.log("Deleting inferred project");
                }
                self.inferred_project.delete();
                return true;
            }
            return false;
        }
        let mut root_file_names = root_file_names;
        root_file_names.sort();
        self.update_or_create_inferred_project(
            root_file_names,
            compiler_options,
            project_references,
            config_file_parsing_diagnostics,
            content_mappers,
            logger,
        )
    }

    pub fn update_or_create_inferred_project(
        &mut self,
        root_file_names: Vec<String>,
        compiler_options: Option<CompilerOptions>,
        project_references: Vec<ProjectReference>,
        config_file_parsing_diagnostics: Vec<tsox_frontend::ast::Diagnostic>,
        content_mappers: Vec<tsox_compile::mig::m3l_cm::Mapper>,
        logger: Option<&LogTree>,
    ) -> bool { ::tsox_core::fntrace::enter("update_or_create_inferred_project"); 
        let project = match self.inferred_project.value().as_ref() {
            None => {
                let mut project = new_inferred_project(
                    self.session_options.current_directory.clone(),
                    compiler_options,
                    root_file_names,
                    project_references,
                    content_mappers,
                    &self.project_collection_builder_seed(),
                    logger,
                );
                if let Some(command_line) = &mut project.command_line {
                    command_line.errors = config_file_parsing_diagnostics;
                }
                self.inferred_project.set(Some(Box::new(project)));
                return true;
            }
            Some(project) => project.clone_project(),
        };
        let compiler_options = compiler_options.unwrap_or_else(|| {
            project
                .command_line
                .as_ref()
                .map(|cl| cl.compiler_options().clone())
                .expect("command line required")
        });
        let mut new_command_line = new_inferred_project_command_line(
            compiler_options,
            &root_file_names,
            &project_references,
            &content_mappers,
            tsox_core::tspath::ComparePathsOptions {
                use_case_sensitive_file_names: self.fs.fs.use_case_sensitive_file_names(),
                current_directory: project.current_directory.clone(),
                ..Default::default()
            },
        );
        new_command_line.errors = config_file_parsing_diagnostics.clone();

        let changed = self.inferred_project.change_if(
            |p: &Option<Box<Project>>| {
                let p = p.as_ref().expect("inferred project");
                let cl = p.command_line.as_ref().expect("command line");
                cl.file_names() != new_command_line.file_names()
                    || !debug_structural_equal(cl.compiler_options(), new_command_line.compiler_options())
                    || !project_references_equal(&cl.project_references(), &project_references)
                    || !debug_structural_equal(&cl.errors, &new_command_line.errors)
                    || !debug_structural_equal(&cl.content_mappers(), &new_command_line.content_mappers())
            },
            |p: &mut Option<Box<Project>>| {
                let p = p.as_mut().expect("inferred project");
                if let Some(logger) = logger {
                    logger.log(&format!(
                        "Updating inferred project config with {} root files",
                        root_file_names.len()
                    ));
                }
                p.set_command_line(new_command_line.clone());
            },
        );
        changed
    }

    pub fn is_supported_in_inferred_project(&self, file_name: &str) -> bool { ::tsox_core::fntrace::enter("is_supported_in_inferred_project"); 
        if tsox_core::tspath::is_dynamic_file_name(file_name)
            || tsox_core::core::mig::m3j::get_script_kind_from_file_name(file_name)
                != tsox_core::core::mig::m3j::ScriptKind::Unknown
        {
            return true;
        }
        if let Some(file) = self.fs.get_file(file_name) {
            if file.is_overlay() && tsox_core::tspath::get_any_extension_from_path(file_name, &[], false).is_empty()
            {
                return true;
            }
        }
        let extensions: Vec<&str> = self
            .inferred_content_mapper_extensions
            .iter()
            .map(|e| e.as_str())
            .collect();
        tsox_core::tspath::file_extension_is_one_of(file_name, &extensions)
    }
}

pub fn project_references_equal(
    a: &[ProjectReference],
    b: &[ProjectReference],
) -> bool { ::tsox_core::fntrace::enter("project_references_equal"); 
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(a, b)| a.path == b.path && a.circular == b.circular)
}

pub struct SearchNode {
    pub config_file_name: String,
    pub load_kind: ProjectLoadKind,
    pub logger: Option<&'static LogTree>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchNodeKey {
    pub config_file_name: String,
    pub load_kind: ProjectLoadKind,
}

pub fn log_change_file_result(_result: &ChangeFileResult, _logger: Option<&LogTree>) { ::tsox_core::fntrace::enter("log_change_file_result"); }

pub struct ChangeFileResult {
    pub affected_projects: HashSet<Path>,
    pub affected_files: HashSet<Path>,
}

impl Clone for Project {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        self.clone_shallow()
    }
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> Clone for MapEntry<K, V> {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        MapEntry {
            key: self.key.clone(),
            original: self.original.clone(),
            value: self.value.clone(),
            dirty: self.dirty,
            delete: self.delete,
        }
    }
}

pub trait DirtyMapMigExt<K: Clone + Eq + std::hash::Hash, V: Clone> {
    fn load(&self, key: &K) -> Option<MapEntry<K, V>>;
    fn keys(&self) -> Vec<K>;
    fn load_or_store(&mut self, key: K, value: V) -> Option<MapEntry<K, V>>;
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> DirtyMapMigExt<K, V> for DirtyMap<K, V> {
    fn load(&self, key: &K) -> Option<MapEntry<K, V>> { ::tsox_core::fntrace::enter("load"); 
        self.get(key)
    }

    fn keys(&self) -> Vec<K> { ::tsox_core::fntrace::enter("keys"); 
        let mut keys = Vec::new();
        self.range(|entry| {
            keys.push(entry.key().clone());
            true
        });
        keys
    }

    fn load_or_store(&mut self, key: K, value: V) -> Option<MapEntry<K, V>> { ::tsox_core::fntrace::enter("load_or_store"); 
        if let Some(existing) = self.get(&key) {
            return Some(existing);
        }
        let stored_key = key.clone();
        self.add(key, value);
        self.get(&stored_key)
    }
}

impl ConfigFileRegistryBuilder {
    pub fn get_config_file_name_for_file(
        &self,
        _file_name: &str,
        _path: &Path,
        _logger: Option<&LogTree>,
    ) -> String { ::tsox_core::fntrace::enter("get_config_file_name_for_file"); 
        todo!("getConfigFileNameForFile requires computeConfigFileName fs walk pending port")
    }

    pub fn get_ancestor_config_file_name(
        &self,
        _file_name: &str,
        _path: &Path,
        _config_file_name: &str,
        _logger: Option<&LogTree>,
    ) -> String { ::tsox_core::fntrace::enter("get_ancestor_config_file_name"); 
        todo!("getAncestorConfigFileName requires forEachConfigFileNameFor pending port")
    }

    pub fn retain_config_for_project(&self, _config_file_path: &Path, _project_path: &Path) { ::tsox_core::fntrace::enter("retain_config_for_project"); 
        todo!("retainConfigForProject requires configFileEntry refcount cache pending port")
    }

    pub fn release_config_for_project(&self, _config_file_path: &Path, _project_path: &Path) { ::tsox_core::fntrace::enter("release_config_for_project"); 
        todo!("releaseConfigForProject requires configFileEntry refcount cache pending port")
    }

    pub fn acquire_config_for_project(
        &self,
        _file_name: &str,
        _path: &Path,
        _project: &Project,
        _logger: Option<&LogTree>,
    ) -> Option<ParsedCommandLine> { ::tsox_core::fntrace::enter("acquire_config_for_project"); 
        todo!("acquireConfigForProject requires findOrAcquireConfigForFile pending port")
    }
}

pub fn client_capabilities_relative_pattern_support(ctx: &ResolvedClientCapabilitiesContext) -> bool { ::tsox_core::fntrace::enter("client_capabilities_relative_pattern_support"); 
    crate::mig::m5m::get_client_capabilities(ctx)
        .raw
        .as_object()
        .map(|raw| {
            serde_json::from_value::<lsproto::ClientCapabilities>(serde_json::Value::Object(
                raw.clone(),
            ))
            .map(|caps| caps.workspace.did_change_watched_files.relative_pattern_support)
            .unwrap_or(false)
        })
        .unwrap_or(false)
}

pub fn debug_structural_equal<T: std::fmt::Debug>(a: &T, b: &T) -> bool { ::tsox_core::fntrace::enter("debug_structural_equal"); 
    format!("{a:?}") == format!("{b:?}")
}
