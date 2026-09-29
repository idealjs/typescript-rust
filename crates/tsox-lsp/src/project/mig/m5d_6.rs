#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use tsox_core::tspath::Path;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use crate::lsp::lsproto;
use crate::ls::lsutil::UserPreferences;
use crate::project::overlay_fs::FileHandle;
use std::sync::Arc;
use tsox_compile::compiler::Program;
use crate::project::dirty_map_::MapEntry;
use crate::project::file_change::FileChangeSummary;
use crate::project::logging_log_tree::LogTree;
use crate::project::mig::m5d::LogTreeMigExt;
use crate::project::mig::m5d_5::{
    ChangeFileResult, DirtyMapMigExt, ProjectCollectionBuilderMig, ProjectLoadKind, SearchResult,
};
use crate::project::project::{Kind, Project};
use crate::project::snapshot::ProjectTreeRequest;

impl ProjectCollectionBuilderMig<'_> {
    pub fn did_change_user_preferences(
        &mut self,
        old_preferences: &UserPreferences,
        new_preferences: &UserPreferences,
        logger: Option<&LogTree>,
    ) {
        if user_preferences_locale_equal(old_preferences, new_preferences) {
            return;
        }
        let mut entries: Vec<MapEntry<Path, Box<Project>>> = Vec::new();
        self.for_each_project(|entry| {
            entries.push(entry.clone());
            true
        });
        for entry in &entries {
            let reference_path = entry.key().clone();
            self.change_entry(entry, |project: &mut Project| {
                project.dirty = true;
                project.dirty_file_path = Path::default();
                if let Some(logger) = logger {
                    let reference_display: &dyn std::fmt::Display = &reference_path.as_str();
                    logger.logf(
                        "Marking project as dirty due to locale change: {}",
                        &[reference_display],
                    );
                }
            });
        }
    }

    pub fn did_request_project_trees(
        &mut self,
        project_tree_request: &ProjectTreeRequest,
        logger: Option<&LogTree>,
    ) {
        let current_projects: Vec<Path> = self.configured_projects.keys();
        let mut seen_projects: HashSet<Path> = HashSet::new();
        for project_id in current_projects {
            if let Some(entry) = self.configured_projects.load(&project_id) {
                let should_update = {
                    let project = entry.value();
                    project_tree_request.is_all_projects()
                        || project.has_potential_project_reference(project_tree_request)
                };
                if should_update {
                    self.update_program(&entry, logger);
                }
                self.ensure_project_tree(&entry, project_tree_request, &mut seen_projects, logger);
            }
        }
        if let Some(logger) = logger {
            logger.log(&format!(
                "Completed project tree request for {:?}",
                project_tree_request.projects()
            ));
        }
    }

    pub fn ensure_project_tree(
        &mut self,
        entry: &MapEntry<Path, Box<Project>>,
        project_tree_request: &ProjectTreeRequest,
        seen_projects: &mut HashSet<Path>,
        logger: Option<&LogTree>,
    ) {
        if !seen_projects.insert(entry.key().clone()) {
            return;
        }
        let project = match entry.value().program.clone() {
            Some(program) => program,
            None => return,
        };
        let command_line = project.command_line();
        if command_line.compiler_options().disable_referenced_project_load
            == tsox_core::core::tristate::Tristate::True
        {
            return;
        }
        for child_config in project.get_resolved_project_references().into_iter().flatten() {
            let child_entry = self.find_or_create_project(
                &child_config.config_name(),
                &Path(child_config.compiler_options().config_file_path.clone()),
                ProjectLoadKind::Create,
                logger,
            );
            if let Some(child_entry) = child_entry {
                self.update_program(&child_entry, logger);
                self.ensure_project_tree(&child_entry, project_tree_request, seen_projects, logger);
            }
        }
    }

    fn retain_project_and_references(&self, to_remove: &mut HashSet<Path>, project: &Project) {
        to_remove.remove(&project.config_file_path);
        if let Some(program) = project.get_program() {
            program.range_resolved_project_reference(
                &mut |reference_path: &Path, _config: Option<&Arc<ParsedCommandLine>>, _base: Option<&Arc<ParsedCommandLine>>, _index: usize| {
                    if self.configured_projects.load(reference_path).is_some() {
                        to_remove.remove(reference_path);
                    }
                    true
                },
            );
        }
    }

    pub fn cleanup_configured_projects(
        &mut self,
        retain: &HashSet<Path>,
        logger: Option<&LogTree>,
    ) {
        let mut to_remove_projects: HashSet<Path> = HashSet::new();
        self.configured_projects.range(|entry: &MapEntry<Path, Box<Project>>| {
            to_remove_projects.insert(entry.key().clone());
            true
        });

        let mut inferred_project_files: Vec<String> = Vec::new();
        let overlay_entries: Vec<(Path, String)> = self
            .fs
            .overlays
            .iter()
            .map(|(path, overlay)| (path.clone(), overlay.file_name().to_string()))
            .collect();
        for (open_file_path, open_file) in &overlay_entries {
            if let Some(p) = self.find_default_configured_project(open_file, open_file_path) {
                let mut keep = to_remove_projects.clone();
                self.retain_project_and_references(&mut keep, p.value().as_ref());
                to_remove_projects = keep;
            } else {
                inferred_project_files.push(open_file.clone());
            }
        }
        let open_file_entries: Vec<(Path, String)> = self
            .api_state
            .open_files
            .iter()
            .map(|(path, file)| (path.clone(), file.file_name.clone()))
            .collect();
        for (path, file_name) in &open_file_entries {
            if self.fs.is_open_file(path) {
                continue;
            }
            if let Some(p) = self.find_default_configured_project(file_name, path) {
                let mut keep = to_remove_projects.clone();
                self.retain_project_and_references(&mut keep, p.value().as_ref());
                to_remove_projects = keep;
            } else {
                inferred_project_files.push(file_name.clone());
            }
        }

        for project_path in to_remove_projects {
            if retain.contains(&project_path) {
                continue;
            }
            if self.api_state.open_projects.contains_key(&project_path) {
                continue;
            }
            if let Some(p) = self.configured_projects.load(&project_path) {
                self.delete_configured_project(p, logger);
            }
        }
        self.update_inferred_project_roots(inferred_project_files, logger);
        self.config_file_registry_builder.cleanup();
    }

    pub fn delete_configured_project(
        &mut self,
        project: MapEntry<Path, Box<Project>>,
        logger: Option<&LogTree>,
    ) {
        let project_path = project.value().config_file_path.clone();
        let project_entry = project.value();
        if let Some(logger) = logger {
            logger.log(&format!(
                "Deleting configured project: {}",
                project_entry.config_file_name
            ));
        }
        if let Some(program) = &project_entry.program {
            program.range_resolved_project_reference(
                &mut |reference_path: &Path, _config: Option<&Arc<ParsedCommandLine>>, _base: Option<&Arc<ParsedCommandLine>>, _index: usize| {
                    self.config_file_registry_builder
                        .release_config_for_project(reference_path, &project_path);
                    true
                },
            );
        }
        self.config_file_registry_builder
            .release_config_for_project(&project_path, &project_path);
        self.configured_projects.delete(&project_path);
    }

    pub fn mark_files_changed(
        &mut self,
        entry: &MapEntry<Path, Box<Project>>,
        paths: &[Path],
        change_type: lsproto::FileChangeType,
        logger: Option<&LogTree>,
    ) {
        let mut dirty = false;
        let mut dirty_file_path = Path::default();
        let should_change = {
            let p = entry.value();
            let p = p.as_ref();
            if p.program.is_none() || (p.dirty && p.dirty_file_path.as_str().is_empty()) {
                false
            } else {
                dirty_file_path = p.dirty_file_path.clone();
                for path in paths {
                    if p.contains_file_in_program(path) {
                        dirty = true;
                        if change_type == lsproto::FILE_CHANGE_TYPE_DELETED {
                            dirty_file_path = Path::default();
                            break;
                        }
                        if tsox_core::tspath::get_base_file_name(path.as_str()) == "package.json" {
                            dirty_file_path = Path::default();
                            break;
                        }
                        if dirty_file_path.as_str().is_empty() {
                            dirty_file_path = path.clone();
                        } else if dirty_file_path != *path {
                            dirty_file_path = Path::default();
                            break;
                        }
                    } else if self.host_seen_file(p, path, change_type) {
                        dirty = true;
                        dirty_file_path = Path::default();
                        break;
                    }
                }
                dirty || p.dirty_file_path != dirty_file_path
            }
        };
        if !should_change {
            return;
        }
        self.change_entry(entry, |p: &mut Project| {
            p.dirty = true;
            p.dirty_file_path = dirty_file_path.clone();
            if let Some(logger) = logger {
                if !dirty_file_path.as_str().is_empty() {
                    let config_display: &dyn std::fmt::Display = &p.config_file_name.as_str();
                    let dirty_display: &dyn std::fmt::Display = &dirty_file_path.as_str();
                    logger.logf(
                        "Marking project {} as dirty due to changes in {}",
                        &[config_display, dirty_display],
                    );
                } else {
                    let config_display: &dyn std::fmt::Display = &p.config_file_name.as_str();
                    logger.logf(
                        "Marking project {} as dirty",
                        &[config_display],
                    );
                }
            }
        });
    }

    fn host_seen_file(
        &self,
        _p: &Project,
        _path: &Path,
        _change_type: lsproto::FileChangeType,
    ) -> bool {
        todo!("requires project.host.sourceFS seen-file queries")
    }

    pub fn mark_projects_affected_by_config_changes(
        &mut self,
        config_change_result: &ChangeFileResult,
        logger: Option<&LogTree>,
    ) -> bool {
        for project_path in &config_change_result.affected_projects {
            let entry = if project_path.as_str() == crate::project::project::INFERRED_PROJECT_NAME {
                None
            } else {
                self.configured_projects.load(project_path)
            };
            match entry {
                Some(entry) => {
                    let should = {
                        let p = entry.value();
                        let p = p.as_ref();
                        !p.dirty || !p.dirty_file_path.as_str().is_empty()
                    };
                    if should {
                        self.change_entry(&entry, |p: &mut Project| {
                            p.dirty = true;
                            p.dirty_file_path = Path::default();
                            if let Some(logger) = logger {
                                let project_display: &dyn std::fmt::Display = &project_path.as_str();
                                logger.logf(
                                    "Marking project {} as dirty due to change affecting config",
                                    &[project_display],
                                );
                            }
                        });
                    }
                }
                None => panic!(
                    "project {} affected by config change not found",
                    project_path
                ),
            }
        }

        let mut has_changes = false;
        for path in &config_change_result.affected_files {
            let file_name = self
                .fs
                .overlays
                .get(path)
                .map(|o| o.file_name().to_string())
                .unwrap_or_default();
            let _ = self.ensure_configured_project_and_ancestors_for_file(&file_name, path, logger);
            has_changes = true;
        }
        has_changes
    }

    pub fn refresh_content_mapper_project_for_changes(
        &mut self,
        entry: &MapEntry<Path, Box<Project>>,
        paths: &[Path],
        refresh_all: bool,
        logger: Option<&LogTree>,
    ) {
        let project = entry.value();
        let project = project.as_ref();
        if project.program.is_none() || project.content_mapper_watched_files.is_none() {
            return;
        }
        let mut affected = refresh_all;
        if !affected {
            let watched = project.content_mapper_watched_files.as_ref().expect("watched");
            affected = paths.iter().any(|p| watched.contains(p));
        }
        if !affected {
            return;
        }
        if let Some(program) = &project.program {
            if let Some(content_mapper_project) = program.content_mapper_project() {
                let _ = content_mapper_project.refresh();
            }
        }
        self.change_entry(entry, |project: &mut Project| {
            project.dirty = true;
            project.dirty_file_path = Path::default();
            if let Some(logger) = logger {
                let config_display: &dyn std::fmt::Display = &project.config_file_path.as_str();
                logger.logf(
                    "Marking project as dirty due to content mapper configuration changes: {}",
                    &[config_display],
                );
            }
        });
    }

    pub fn release_dropped_project_references(
        &mut self,
        old_program: Option<&Program>,
        new_program: Option<&Program>,
        project_path: &Path,
    ) {
        let old_program = match old_program {
            Some(p) => p,
            None => return,
        };
        let mut new_references: HashSet<Path> = HashSet::new();
        if let Some(new_program) = new_program {
            new_program.range_resolved_project_reference(
                &mut |reference_path: &Path, _config: Option<&Arc<ParsedCommandLine>>, _base: Option<&Arc<ParsedCommandLine>>, _index: usize| {
                    new_references.insert(reference_path.clone());
                    true
                },
            );
        }
        old_program.range_resolved_project_reference(
            &mut |reference_path: &Path, _config: Option<&Arc<ParsedCommandLine>>, _base: Option<&Arc<ParsedCommandLine>>, _index: usize| {
                if !new_references.contains(reference_path) {
                    self.config_file_registry_builder
                        .release_config_for_project(reference_path, project_path);
                }
                true
            },
        );
    }

    pub fn update_program(
        &mut self,
        entry: &MapEntry<Path, Box<Project>>,
        logger: Option<&LogTree>,
    ) -> bool {
        let config_file_name = entry
            .value()
            .as_ref()
            .config_file_name
            .clone();
        let mut update_program = false;
        let mut delete_project = false;
        let mut files_changed = false;

        let is_configured = entry
            .value()
            .as_ref()
            .kind
            == Kind::Configured;
        if is_configured {
            let project = entry.value();
            let project = project.as_ref();
            let command_line = self.config_file_registry_builder.acquire_config_for_project(
                &project.config_file_name,
                &project.config_file_path,
                project,
                logger,
            );
            match command_line {
                None => {
                    delete_project = true;
                    files_changed = true;
                }
                Some(command_line) => {
                    let command_line_changed = match project.command_line.as_ref() {
                        Some(existing) => {
                            existing.file_names() != command_line.file_names()
                                || !crate::project::mig::m5d_5::debug_structural_equal(
                                    existing.compiler_options(),
                                    command_line.compiler_options(),
                                )
                        }
                        None => true,
                    };
                    if command_line_changed {
                        update_program = true;
                        self.change_entry(entry, |p: &mut Project| {
                            p.set_command_line(command_line.clone());
                        });
                    }
                }
            }
        }
        if !update_program {
            update_program = entry.value().as_ref().dirty;
        }

        if delete_project {
            self.delete_configured_project(entry.clone(), logger);
            return files_changed;
        }

        if update_program {
            let new_snapshot_id = self.new_snapshot_id;
            let result = {
                let mut project = entry.value().as_ref().clone_project();
                let old_program = project.program.clone();
                let old_checker_pool = project.checker_pool();
                let created = project.create_program();
                if created.program.is_some() {
                    self.release_dropped_project_references(
                        old_program.as_deref(),
                        created.program.as_deref(),
                        &project.config_file_path,
                    );
                    if let Some(old_checker_pool) = old_checker_pool {
                        old_checker_pool.discard();
                    }
                }
                self.change_entry(entry, |p: &mut Project| {
                    p.program = created.program.clone();
                    p.program_update_kind = created.update_kind;
                    p.program_last_update = new_snapshot_id;
                    if created.update_kind == crate::project::project::ProgramUpdateKind::NewFiles {
                        files_changed = true;
                    }
                    p.dirty = false;
                    p.dirty_file_path = Path::default();
                });
                created
            };
            let _ = result;
        }

        if update_program {
            if let Some(logger) = logger {
                logger.log(&format!(
                    "Program update for {} completed",
                    config_file_name
                ));
            }
        }
        files_changed
    }
}

fn user_preferences_locale_equal(_old: &UserPreferences, _new: &UserPreferences) -> bool {
    todo!("UserPreferences.locale \u{5b57}\u{6bb5}\u{672a}\u{79fb}\u{690d}\u{ff08}Go: oldPreferences.Locale == newPreferences.Locale\u{ff09}")
}
