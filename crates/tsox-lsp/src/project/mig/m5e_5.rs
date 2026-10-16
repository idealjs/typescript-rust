#![allow(dead_code, unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use tsox_core::tspath::Path;

use crate::project::session::*;
use crate::project::watch::{WatcherID, WatchedFiles, Watchers, file_system_watcher_glob_string};

pub(crate) use super::m5e::{ata, unsafe_clone_session};

mod logging {
    pub use crate::project::logging_log_tree::new_log_tree;
}

use tsox_core::diagnostics;

use crate::project::snapshot::{ATAStateChange, ResourceRequest};

const WATCH_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

fn sync_project_watch<T: Clone + Send + Sync + Default>(
    session: &Session,
    logger: Option<&dyn Logger>,
    errors: &mut Vec<String>,
    old_watcher: Option<&WatchedFiles<T>>,
    new_watcher: Option<&WatchedFiles<T>>,
) { ::tsox_core::fntrace::enter("sync_project_watch"); 
    let Some(new_watcher) = new_watcher else {
        if let Some(old_watcher) = old_watcher {
            errors.extend(update_watch(session, logger, Some(old_watcher), None));
        }
        return;
    };
    if old_watcher.is_some_and(|old| old.id() == new_watcher.id()) {
        if session.watches.is_pending(&new_watcher.id()) {
            errors.extend(update_watch(session, logger, None, Some(new_watcher)));
        }
        return;
    }
    errors.extend(update_watch(session, logger, old_watcher, Some(new_watcher)));
}

pub fn update_watch<T: Clone + Send + Sync + Default>(
    session: &Session,
    logger: Option<&dyn Logger>,
    old_watcher: Option<&WatchedFiles<T>>,
    new_watcher: Option<&WatchedFiles<T>>,
) -> Vec<String> { ::tsox_core::fntrace::enter("update_watch"); 
    let mut errors = Vec::new();
    if let Some(new_watcher) = new_watcher {
        let w = new_watcher.watchers();
        let mut watchers = w.workspace_watchers.clone();
        watchers.extend(w.outside_workspace_watchers.iter().cloned());
        if !watchers.is_empty() {
            let mut new_watchers: Vec<(WatcherID, lsproto::FileSystemWatcher)> = Vec::new();
            for (i, watcher) in watchers.iter().enumerate() {
                let glob_id = format!("{}.{}", w.watcher_id, i);
                if session.watches.acquire(watcher, glob_id.clone()) {
                    new_watchers.push((glob_id, watcher.clone()));
                }
            }
            let mut watch_errors = Vec::new();
            for (id, watcher) in &new_watchers {
                if let Err(err) = session
                    .client
                    .as_ref()
                    .expect("client")
                    .watch_files(id, &[watcher.clone()])
                {
                    watch_errors.push(err.to_string());
                } else if let Some(logger) = logger {
                    if old_watcher.is_none() {
                        logger.log(&format!("Added new watch: {id}"));
                    } else {
                        logger.log(&format!("Updated watch: {id}"));
                    }
                    logger.log(&format!("\t{}", file_system_watcher_glob_string(watcher)));
                    logger.log("");
                }
            }
            if !watch_errors.is_empty() {
                for (id, watcher) in &new_watchers {
                    session.watches.release(watcher);
                }
                session.watches.mark_pending(&w.watcher_id);
                errors.extend(watch_errors);
            } else {
                session.watches.clear_pending(&w.watcher_id);
            }
            if !w.ignored_paths.is_empty() {
                if let Some(logger) = logger {
                    logger.log(&format!("{} paths ineligible for watching", w.ignored_paths.len()));
                    if logger.is_verbose() {
                        for path in &w.ignored_paths {
                            logger.log(&format!("\t{path}"));
                        }
                    }
                }
            }
        }
    }
    if let Some(old_watcher) = old_watcher {
        let w = old_watcher.watchers();
        let mut watchers = w.workspace_watchers.clone();
        watchers.extend(w.outside_workspace_watchers.iter().cloned());
        if !watchers.is_empty() {
            let mut removed_ids = Vec::new();
            for watcher in &watchers {
                let (id, removed) = session.watches.release(watcher);
                if removed {
                    removed_ids.push(id);
                }
            }
            for id in removed_ids {
                if let Err(err) = session
                    .client
                    .as_ref()
                    .expect("client")
                    .unwatch_files(&id)
                {
                    errors.push(err.to_string());
                } else if let Some(logger) = logger
                    && new_watcher.is_none()
                {
                    logger.log(&format!("Removed watch: {id}"));
                }
            }
        }
    }
    errors
}

impl Session {
    pub fn update_watches(
        &self,
        old_snapshot: &Snapshot,
        new_snapshot: &Snapshot,
    ) -> Result<(), String> { ::tsox_core::fntrace::enter("update_watches"); 
        let start = std::time::Instant::now();
        let mut errors: Vec<String> = Vec::new();
        let old_configs = old_snapshot.config_file_registry_configs();
        let new_configs = new_snapshot.config_file_registry_configs();
        for (path, new_entry) in new_configs {
            let Some(new_watch) = new_entry.root_files_watch.as_ref() else { continue };
            match old_configs.get(path) {
                None => {
                    errors.extend(update_watch(self, self.logger.as_deref(), None, Some(new_watch)));
                }
                Some(old_entry) => {
                    if let Some(old_watch) = old_entry.root_files_watch.as_ref() {
                        if old_watch.id() != new_watch.id() {
                            errors.extend(update_watch(
                                self,
                                self.logger.as_deref(),
                                Some(old_watch),
                                Some(new_watch),
                            ));
                        } else if self.watches.is_pending(&new_watch.id()) {
                            errors.extend(update_watch(self, self.logger.as_deref(), None, Some(new_watch)));
                        }
                    }
                }
            }
        }
        for (path, old_entry) in old_configs {
            if !new_configs.contains_key(path) {
                if let Some(old_watch) = old_entry.root_files_watch.as_ref() {
                    errors.extend(update_watch(self, self.logger.as_deref(), Some(old_watch), None));
                }
            }
        }
        let old_projects = old_snapshot.projects_by_path();
        let new_projects = new_snapshot.projects_by_path();
        for (path, project) in &new_projects {
            match old_projects.iter().find(|(p, _)| p == path) {
                None => {
                    let new_ext = project.ext();
                    sync_project_watch(self, self.logger.as_deref(), &mut errors, None, new_ext.program_files_watch.as_deref());
                    sync_project_watch(self, self.logger.as_deref(), &mut errors, None, new_ext.typings_watch.as_deref());
                    sync_project_watch(self, self.logger.as_deref(), &mut errors, None, new_ext.content_mapper_watch.as_deref());
                }
                Some(old_project) => {
                    let old_ext = old_project.1.ext();
                    let new_ext = project.ext();
                    sync_project_watch(self, self.logger.as_deref(), &mut errors, old_ext.program_files_watch.as_deref(), new_ext.program_files_watch.as_deref());
                    sync_project_watch(self, self.logger.as_deref(), &mut errors, old_ext.typings_watch.as_deref(), new_ext.typings_watch.as_deref());
                    sync_project_watch(self, self.logger.as_deref(), &mut errors, old_ext.content_mapper_watch.as_deref(), new_ext.content_mapper_watch.as_deref());
                }
            }
        }
        for (path, old_project) in &old_projects {
            if !new_projects.iter().any(|(p, _)| p == path) {
                let old_ext = old_project.ext();
                sync_project_watch(self, self.logger.as_deref(), &mut errors, old_ext.program_files_watch.as_deref(), None);
                sync_project_watch(self, self.logger.as_deref(), &mut errors, old_ext.typings_watch.as_deref(), None);
                sync_project_watch(self, self.logger.as_deref(), &mut errors, old_ext.content_mapper_watch.as_deref(), None);
            }
        }
        let old_watch = old_snapshot.auto_imports_watch.as_deref();
        let new_watch = new_snapshot.auto_imports_watch.as_deref();
        match (old_watch, new_watch) {
            (Some(old_w), Some(new_w)) => {
                if old_w.id() != new_w.id() {
                    errors.extend(update_watch(self, self.logger.as_deref(), Some(old_w), Some(new_w)));
                } else if self.watches.is_pending(&new_w.id()) {
                    errors.extend(update_watch(self, self.logger.as_deref(), None, Some(new_w)));
                }
            }
            (None, Some(new_w)) => {
                errors.extend(update_watch(self, self.logger.as_deref(), None, Some(new_w)));
            }
            (Some(old_w), None) => {
                errors.extend(update_watch(self, self.logger.as_deref(), Some(old_w), None));
            }
            (None, None) => {}
        }
        if !errors.is_empty() {
            Err(format!("errors updating watches: {errors:?}"))
        } else {
        if self.options.logging_enabled {
            self.logger.as_ref().expect("logger").log(&format!("Updated watches in {:?}", start.elapsed()));
        }
            Ok(())
        }
    }

    pub fn enqueue_publish_global_diagnostics(&self) { ::tsox_core::fntrace::enter("enqueue_publish_global_diagnostics"); 
        if !self.options.push_diagnostics_enabled || self.config().enable_validation.is_false() {
            return;
        }
        if !self
            .global_diag_publish_pending
            .compare_exchange(false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
            .is_ok()
        {
            return;
        }
        let session = unsafe_clone_session(self);
        self.background_queue.enqueue(move || {
            session.publish_global_diagnostics();
        });
    }

    pub fn trigger_ata_for_updated_projects(&self, new_snapshot: &Arc<Snapshot>) { ::tsox_core::fntrace::enter("trigger_ata_for_updated_projects"); 
        for project in new_snapshot.projects() {
            if !project.should_trigger_ata(new_snapshot.id()) {
                continue;
            }
            let session = unsafe_clone_session(self);
            let new_snapshot = Arc::clone(new_snapshot);
            let config_file_path = project.config_file_path.clone();
            self.background_queue.enqueue(move || {
                let project = new_snapshot.project_by_path(&config_file_path);
                let Some(project) = project else { return };
                let mut log_tree = None;
                if session.options.logging_enabled {
                    log_tree = Some(logging::new_log_tree(&format!(
                        "Triggering ATA for project {}",
                        project.name()
                    )));
                }
                let typings_info = project.compute_typings_info();
                let request = ata::TypingsInstallRequest {
                    project_id: project.config_file_path.clone(),
                    typings_info: typings_info.clone(),
                    file_names: project
                        .program
                        .as_ref()
                        .map(|p| p.get_source_files().iter().map(|f| f.file_name.clone()).collect())
                        .unwrap_or_default(),
                    project_root_path: project.current_directory.clone(),
                    compiler_options: project
                        .command_line
                        .as_ref()
                        .expect("command line")
                        .compiler_options()
                        .clone(),
                    current_directory: session.options.current_directory.clone(),
                    fs: Some(session.fs_fs()),
                };
                let project_display_name = project.display_name(&session.options.current_directory);
                if let Some(client) = &session.client {
                    client.progress_start(
                        &diagnostics::INSTALLING_TYPES_FOR_0,
                        &[Box::new(project_display_name.clone()) as Box<dyn std::fmt::Debug>],
                    );
                }
                let result = session
                    .typings_installer
                    .as_ref()
                    .expect("typings installer")
                    .install_typings(&request);
                if let Some(client) = &session.client {
                    client.progress_finish(
                        &diagnostics::INSTALLING_TYPES_FOR_0,
                        &[Box::new(project_display_name.clone()) as Box<dyn std::fmt::Debug>],
                    );
                }
                match result {
                    Err(err) => {
                        if log_tree.is_some() {
                            session.logger.as_ref().expect("logger").log(&format!(
                                "ATA installation failed for project {}: {err}",
                                project.name()
                            ));
                            session.logger.as_ref().expect("logger").log(&log_tree.unwrap().to_string());
                        }
                    }
                    Ok(result) => {
                        if result.typings_files != project.typings_files {
                            session.pending_ata_changes.lock().unwrap().insert(
                                project.config_file_path.clone(),
                                ATAStateChange {
                                    project_id: project.config_file_path.clone(),
                                    typings_files: result.typings_files.clone(),
                                    typings_files_to_watch: result.files_to_watch.clone(),
                                },
                            );
                            session.schedule_diagnostics_refresh();
                        }
                    }
                }
            });
        }
    }

    pub fn warm_auto_import_cache(
        &self,
        change: &SnapshotChange,
        old_snapshot: &Arc<Snapshot>,
        new_snapshot: &Arc<Snapshot>,
    ) { ::tsox_core::fntrace::enter("warm_auto_import_cache"); 
        if change.file_changes.changed.len() != 1 {
            return;
        }
        let changed_file = change.file_changes.changed.iter().next().unwrap().clone();
        if !new_snapshot.fs_is_open_file(&changed_file.file_name()) {
            return;
        }
        let prefs = &new_snapshot.user_preferences;
        if prefs.include_completions_for_module_exports.is_false() {
            return;
        }
        let Some(project) = new_snapshot.get_default_project(&changed_file) else {
            return;
        };
        if new_snapshot
            .auto_imports
            .as_deref()
            .is_some_and(|registry| {
                registry.is_prepared_for_importing_file(
                    &changed_file.file_name(),
                    &project.config_file_path,
                    prefs,
                )
            })
        {
            return;
        }
        if self
            .warm_auto_import_active
            .compare_exchange(false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
            .is_err()
        {
            return;
        }
        if !new_snapshot.try_ref() {
            self.warm_auto_import_active.store(false, std::sync::atomic::Ordering::SeqCst);
            return;
        }
        let session = unsafe_clone_session(self);
        let new_snapshot = Arc::clone(new_snapshot);
        let old_snapshot = Arc::clone(old_snapshot);
        let changed_file = changed_file.clone();
        self.background_queue.enqueue(move || {
            let warm_change = SnapshotChange {
                reason: UpdateReason::RequestedLanguageServiceWithAutoImports,
                resource_request: ResourceRequest {
                    documents: vec![changed_file.clone()],
                    auto_imports: changed_file.clone(),
                    ..Default::default()
                },
                ..Default::default()
            };
            let cloned_snapshot =
                new_snapshot.clone_snapshot(warm_change, new_snapshot.overlays(), &session);
            session.adopt_snapshot_change(&new_snapshot, &cloned_snapshot);
            new_snapshot.deref(&session);
            session.warm_auto_import_active.store(false, std::sync::atomic::Ordering::SeqCst);
        });
    }
}

impl Snapshot {
    pub fn projects(&self) -> Vec<&crate::project::project::Project> { ::tsox_core::fntrace::enter("projects"); 
        self.project_collection
            .as_deref()
            .expect("project collection")
            .projects()
    }

    pub fn projects_by_path(&self) -> Vec<(Path, &crate::project::project::Project)> { ::tsox_core::fntrace::enter("projects_by_path"); 
        self.project_collection
            .as_deref()
            .expect("project collection")
            .projects_by_path()
    }

    pub fn project_by_path(&self, path: &Path) -> Option<&crate::project::project::Project> { ::tsox_core::fntrace::enter("project_by_path"); 
        self.project_collection
            .as_deref()
            .expect("project collection")
            .projects_by_path()
            .into_iter()
            .find(|(p, _)| p == path)
            .map(|(_, project)| project)
    }

    pub fn config_file_registry_configs(
        &self,
    ) -> &HashMap<Path, crate::project::config_file_registry::ConfigFileEntry> { ::tsox_core::fntrace::enter("config_file_registry_configs"); 
        &self
            .config_file_registry
            .as_deref()
            .expect("config file registry")
            .configs
    }

    pub fn fs_is_open_file(&self, file_name: &str) -> bool { ::tsox_core::fntrace::enter("fs_is_open_file"); 
        self.fs.as_ref().expect("fs").is_open_file(file_name)
    }

    pub fn overlays(&self) -> Option<&HashMap<Path, Arc<crate::project::overlay_fs::Overlay>>> { ::tsox_core::fntrace::enter("overlays"); 
        self.fs.as_ref().map(|fs| &fs.overlays)
    }
}
