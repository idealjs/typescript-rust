#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tsox_core::tspath::Path;

use crate::lsp::lsproto;
use crate::project::overlay_fs::Overlay;
use crate::project::project::INFERRED_PROJECT_NAME;
use crate::project::project::Project;
use crate::project::project_collection::{APIState, ProjectCollection};
use crate::project::config_file_registry::ConfigFileRegistry;
use crate::project::logging_log_tree::LogTree;
use crate::project::mig::m5d::LogTreeMigExt;
use crate::project::session_watch_request_timeout::Session;

pub fn api_state_clone(s: &APIState) -> APIState {
    APIState {
        open_projects: s.open_projects.clone(),
        open_files: s.open_files.clone(),
    }
}

impl APIState {
    pub fn clone_state(&self) -> APIState {
        api_state_clone(self)
    }
}

fn clone_config_file_registry(r: &ConfigFileRegistry) -> ConfigFileRegistry {
    ConfigFileRegistry {
        configs: r.configs.clone(),
        config_file_names: r.config_file_names.clone(),
        custom_config_file_name: r.custom_config_file_name.clone(),
    }
}

impl ProjectCollection {
    pub fn clone_collection(
        &self,
        current_directory: String,
        use_case_sensitive_file_names: bool,
    ) -> ProjectCollection {
        ProjectCollection {
            to_path: Box::new(move |file_name: &str| {
                tsox_core::tspath::to_path(file_name, &current_directory, use_case_sensitive_file_names)
            }),
            config_file_registry: self.config_file_registry.as_ref().map(clone_config_file_registry),
            configured_projects: self
                .configured_projects
                .iter()
                .map(|(k, v)| (k.clone(), Box::new(v.clone_shallow())))
                .collect(),
            open_files: self.open_files.clone(),
            inferred_project: self
                .inferred_project
                .as_ref()
                .map(|p| Box::new(p.clone_shallow())),
            file_default_projects: self.file_default_projects.clone(),
            api_state: self.api_state.clone_shallow(),
        }
    }

    fn to_path_ref(&self) -> &Box<dyn Fn(&str) -> Path + Send + Sync> {
        &self.to_path
    }

    pub fn fill_configured_projects<'a>(&'a self, projects: &mut Vec<&'a Project>) {
        let mut collected: Vec<&Project> = self
            .configured_projects
            .values()
            .map(|p| p.as_ref())
            .collect();
        collected.sort_by(|a, b| a.name().cmp(b.name()));
        projects.extend(collected);
    }

    pub fn projects_by_path(&self) -> Vec<(Path, &Project)> {
        let mut projects: Vec<(Path, &Project)> = Vec::with_capacity(
            self.configured_projects.len() + usize::from(self.inferred_project.is_some()),
        );
        let mut configured: Vec<&Project> = Vec::new();
        self.fill_configured_projects(&mut configured);
        for project in configured {
            projects.push((project.config_file_path.clone(), project));
        }
        if let Some(inferred) = &self.inferred_project {
            projects.push((
                Path(INFERRED_PROJECT_NAME.to_string()),
                inferred.as_ref(),
            ));
        }
        projects
    }

    pub fn get_projects_containing_file(&self, path: &Path) -> Vec<&Project> {
        let mut projects = Vec::new();
        let mut configured: Vec<&Project> = Vec::new();
        self.fill_configured_projects(&mut configured);
        for project in configured {
            if project.contains_file_in_program(path) {
                projects.push(project);
            }
        }
        if let Some(inferred) = &self.inferred_project {
            if inferred.contains_file_in_program(path) {
                projects.push(inferred.as_ref());
            }
        }
        projects
    }

    pub fn get_open_configured_projects(&self) -> HashSet<Path> {
        let mut open_projects = HashSet::with_capacity(self.configured_projects.len());
        for path in &self.open_files {
            if let Some(project_path) = self.file_default_projects.get(path) {
                if project_path.as_str() != INFERRED_PROJECT_NAME
                    && self.configured_projects.contains_key(project_path)
                {
                    open_projects.insert(project_path.clone());
                    continue;
                }
            }
            for project in self.configured_projects.values() {
                if project.contains_file_in_program(path) {
                    open_projects.insert(project.config_file_path.clone());
                }
            }
        }
        open_projects
    }

    pub fn find_default_configured_project(&self, path: &Path) -> Option<&Project> {
        let config_file_name = self
            .config_file_registry
            .as_ref()?
            .get_config_file_name(path);
        if config_file_name.is_empty() {
            return None;
        }
        self.find_default_configured_project_worker(path, &config_file_name, &mut HashSet::new(), None)
    }

    pub fn find_default_configured_project_worker<'a>(
        &'a self,
        path: &Path,
        config_file_name: &str,
        visited: &mut HashSet<*const Project>,
        fallback: Option<&'a Project>,
    ) -> Option<&'a Project> {
        let config_file_path = (self.to_path)(config_file_name);
        let project = self.configured_projects.get(&config_file_path)?;

        let neighbors = |project: &Project| -> Vec<&Project> {
            let Some(command_line) = &project.command_line else {
                return Vec::new();
            };
            let mut command_line = command_line.clone();
            command_line
                .resolved_project_reference_paths()
                .into_iter()
                .filter_map(|config_file_name| {
                    let reference_path = (self.to_path)(&config_file_name);
                    self.configured_projects.get(&reference_path).map(|p| p.as_ref())
                })
                .collect()
        };
        let visit = |project: &Project| -> (bool, bool) {
            if project.contains_file_in_program(path) {
                return (
                    true,
                    !project.is_source_from_project_reference_in_program(path),
                );
            }
            (false, false)
        };

        let mut stopped = false;
        let mut goal: Option<&Project> = None;
        let mut level: Vec<&Project> = vec![project];
        while !level.is_empty() {
            let mut next: Vec<&Project> = Vec::new();
            for node in &level {
                if !visited.insert(*node as *const Project) {
                    continue;
                }
                let (is_result, stop) = visit(node);
                if is_result && goal.is_none() {
                    goal = Some(*node);
                    stopped = stop;
                }
                if stopped {
                    break;
                }
                next.extend(neighbors(*node));
            }
            if stopped {
                break;
            }
            let mut seen = HashSet::new();
            next.retain(|p| seen.insert(*p as *const Project));
            level = next;
        }

        if stopped {
            return goal;
        }
        let mut fallback = fallback;
        if goal.is_some() && fallback.is_none() {
            fallback = goal;
        }

        if let Some(config) = self
            .config_file_registry
            .as_ref()
            .and_then(|registry| registry.get_config(path))
        {
            if config.compiler_options().disable_solution_searching == tsox_core::core::tristate::Tristate::True {
                return fallback;
            }
        }
        let ancestor_config_name = self
            .config_file_registry
            .as_ref()
            .map(|registry| registry.get_ancestor_config_file_name(path, config_file_name))
            .unwrap_or_default();
        if !ancestor_config_name.is_empty() {
            return self.find_default_configured_project_worker(
                path,
                &ancestor_config_name,
                visited,
                fallback,
            );
        }
        fallback
    }
}

pub fn find_default_configured_project_from_program_inclusion<'a>(
    file_name: &str,
    path: &Path,
    project_paths: &[Path],
    get_project: &dyn Fn(&Path) -> Option<&'a Project>,
) -> (Path, bool) {
    let _ = file_name;
    let mut containing_projects: Vec<Path> = Vec::new();
    let mut first_configured_project: Option<Path> = None;
    let mut first_non_source_of_project_reference_redirect: Option<Path> = None;
    let mut multiple_direct_inclusions = false;

    for project_path in project_paths {
        let p = match get_project(project_path) {
            Some(p) => p,
            None => continue,
        };
        if p.contains_file_in_program(path) {
            containing_projects.push(project_path.clone());
            if !multiple_direct_inclusions && !p.is_source_from_project_reference_in_program(path) {
                if first_non_source_of_project_reference_redirect.is_none() {
                    first_non_source_of_project_reference_redirect =
                        Some(project_path.clone());
                } else {
                    multiple_direct_inclusions = true;
                }
            }
            if first_configured_project.is_none() {
                first_configured_project = Some(project_path.clone());
            }
        }
    }

    if containing_projects.len() == 1 {
        return (containing_projects[0].clone(), false);
    }
    if !multiple_direct_inclusions {
        if let Some(redirect) = first_non_source_of_project_reference_redirect {
            return (redirect, false);
        }
        return (
            first_configured_project.unwrap_or_else(|| Path::default()),
            false,
        );
    }
    (
        first_configured_project.unwrap_or_else(|| Path::default()),
        true,
    )
}

pub fn open_file_paths_mig(overlays: &HashMap<Path, Arc<Overlay>>) -> HashSet<Path> {
    overlays.keys().cloned().collect()
}

impl Session {
    pub fn get_current_directory(&self) -> String {
        self.options.current_directory.clone()
    }

    pub fn get_language_service_and_projects_for_file(
        &self,
        uri: &lsproto::DocumentUri,
    ) -> Result<(Arc<Project>, Arc<LanguageService>, Vec<Project>), String> {
        let (snapshot, project, default_ls) =
            self.get_snapshot_and_default_project(uri, false)?;
        let all_projects = snapshot
            .project_collection
            .as_ref()
            .map(|pc| {
                let path = (pc.to_path)(&uri.file_name());
                pc.get_projects_containing_file(&path)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<Project>>()
            })
            .unwrap_or_default();
        Ok((project, default_ls, all_projects))
    }

    pub fn get_language_service_for_project_with_file(
        &self,
        project: &Project,
        uri: &lsproto::DocumentUri,
    ) -> Option<LanguageService> {
        let snapshot = self.snapshot().expect("snapshot");
        let project = snapshot
            .project_collection
            .as_ref()?
            .get_project_by_path(project.id())?;
        if !project.has_file_in_program(&uri.file_name()) {
            return None;
        }
        Some(crate::ls::language_service::LanguageService::new(
            project.config_file_path.clone(),
            project.get_program()?.clone(),
            Box::new(Arc::clone(&snapshot)),
            &uri.file_name(),
        ))
    }

    pub fn get_current_language_service_with_auto_imports(
        &self,
        uri: &lsproto::DocumentUri,
    ) -> Result<LanguageService, String> {
        let base_snapshot = self.snapshot().expect("snapshot");
        let snapshot = self.get_snapshot_with_auto_imports(&base_snapshot, uri);
        let project = snapshot.get_default_project(uri);
        match project {
            None => Err(format!("no project found for URI {}", uri)),
            Some(project) => Ok(crate::ls::language_service::LanguageService::new(
                project.config_file_path.clone(),
                project
                    .get_program()
                    .cloned()
                    .ok_or_else(|| "project has no program".to_string())?,
                Box::new(Arc::clone(&snapshot)),
                &uri.file_name(),
            )),
        }
    }

}

use crate::ls::language_service::LanguageService;
use crate::project::snapshot::Snapshot;
use std::sync::atomic::Ordering;
