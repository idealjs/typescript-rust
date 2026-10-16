#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tsox_core::tspath::Path;

use crate::project::session::*;

use crate::project::project::Project;
use crate::project::snapshot::{ProjectTreeRequest, ResourceRequest};

pub(crate) use super::m5e::{background, contentmapper, unsafe_clone_session};

pub const ERR_NO_PROJECT_FOR_UNKNOWN_SCRIPT_KIND: &str = "no project for unknown script kind";

use crate::ls as ls;
use crate::ls::language_service::LanguageService;

mod core {
    pub use tsox_frontend::ast::node_source_file::ScriptKind;
}

pub fn has_content_mapper_operation_timings(
    timings: &HashMap<String, contentmapper::MapperTimings>,
) -> bool { ::tsox_core::fntrace::enter("has_content_mapper_operation_timings"); 
    timings.values().any(|t| has_content_mapper_operation_timing(t))
}

pub fn has_content_mapper_operation_timing(timing: &contentmapper::MapperTimings) -> bool { ::tsox_core::fntrace::enter("has_content_mapper_operation_timing"); 
    timing.spawn.count != 0
        || timing.open_project.count != 0
        || timing.close_project.count != 0
        || timing.transform.count != 0
}

impl Session {
    pub fn get_snapshot(&self, request: ResourceRequest, caller_ref: bool) -> Arc<Snapshot> { ::tsox_core::fntrace::enter("get_snapshot"); 
        self.cancel_scheduled_snapshot_update();
        let (file_changes, overlays) = self.flush_changes();
        let update_snapshot = !file_changes.is_empty();
        if update_snapshot {
            return self.update_snapshot_internal(
                overlays,
                SnapshotChange {
                    reason: UpdateReason::RequestedLanguageServicePendingChanges,
                    file_changes,
                    resource_request: request,
                    ..Default::default()
                },
                caller_ref,
            );
        }
        let snapshot = self.snapshot().expect("snapshot");
        let mut update_reason = UpdateReason::Unknown;
        if !request.projects.is_empty() {
            update_reason = UpdateReason::RequestedLanguageServiceProjectDirty;
        } else if request.project_tree.is_some() {
            update_reason = UpdateReason::RequestedLoadProjectTree;
        } else if !request.auto_imports.0.is_empty() {
            update_reason = UpdateReason::RequestedLanguageServiceWithAutoImports;
        } else {
            for document in &request.documents {
                match snapshot.get_default_project(document) {
                    None => update_reason = UpdateReason::RequestedLanguageServiceProjectNotLoaded,
                    Some(project) if project.dirty => {
                        update_reason = UpdateReason::RequestedLanguageServiceProjectDirty
                    }
                    Some(_) => {}
                }
            }
            if update_reason == UpdateReason::Unknown {
                for document in &request.configured_project_documents {
                    if snapshot.fs_is_open_file(&document.file_name()) {
                        match snapshot.get_default_project(document) {
                            None => {
                                update_reason =
                                    UpdateReason::RequestedLanguageServiceProjectNotLoaded
                            }
                            Some(project) if project.dirty => {
                                update_reason = UpdateReason::RequestedLanguageServiceProjectDirty
                            }
                            Some(_) => {}
                        }
                    } else {
                        update_reason = UpdateReason::RequestedLanguageServiceForFileNotOpen;
                    }
                }
            }
        }
        if update_reason == UpdateReason::Unknown {
            if caller_ref {
                snapshot.r#ref();
            }
            return snapshot;
        }
        self.update_snapshot_internal(
            overlays,
            SnapshotChange {
                reason: update_reason,
                resource_request: request,
                ..Default::default()
            },
            caller_ref,
        )
    }

    pub fn get_snapshot_and_default_project(
        &self,
        uri: &lsproto::DocumentUri,
        caller_ref: bool,
    ) -> Result<(Arc<Snapshot>, Arc<Project>, Arc<LanguageService>), String> { ::tsox_core::fntrace::enter("get_snapshot_and_default_project"); 
        let snapshot = self.get_snapshot(
            ResourceRequest {
                documents: vec![uri.clone()],
                ..Default::default()
            },
            caller_ref,
        );
        match snapshot.get_default_project(uri) {
            Some(project) => {
                let ls = LanguageService::new(
                    project.config_file_path.clone(),
                    project.get_program().expect("project program").clone(),
                    Box::new(Arc::clone(&snapshot)),
                    &uri.file_name(),
                );
                let project = Arc::new(project.clone());
                Ok((snapshot, project, Arc::new(ls)))
            }
            None => {
                if caller_ref {
                    snapshot.deref(self);
                }
                let file = snapshot.fs.as_ref().and_then(|fs| fs.get_file(&uri.file_name()));
                if let Some(file) = file
                    && file.kind() == core::ScriptKind::Unknown as i32
                {
                    return Err(format!(
                        "{ERR_NO_PROJECT_FOR_UNKNOWN_SCRIPT_KIND}: no project found for URI {uri}"
                    ));
                }
                Err(format!("no project found for URI {uri}"))
            }
        }
    }

    pub fn get_projects_for_file(
        &self,
        uri: &lsproto::DocumentUri,
    ) -> Vec<Project> { ::tsox_core::fntrace::enter("get_projects_for_file"); 
        let snapshot = self.get_snapshot(
            ResourceRequest {
                configured_project_documents: vec![uri.clone()],
                ..Default::default()
            },
            false,
        );
        snapshot.get_projects_containing_file(uri)
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn get_language_services_for_documents_loading_project_tree(
        &self,
        uris: &[lsproto::DocumentUri],
    ) -> Vec<Arc<LanguageService>> { ::tsox_core::fntrace::enter("get_language_services_for_documents_loading_project_tree"); 
        let snapshot = self.get_snapshot(
            ResourceRequest {
                documents: uris.to_vec(),
                project_tree: Some(ProjectTreeRequest {
                    referenced_projects: None,
                }),
                ..Default::default()
            },
            false,
        );
        let active_file = uris
            .first()
            .map(|u| u.file_name())
            .unwrap_or_default();
        let mut services = Vec::new();
        for project in snapshot.projects() {
            let Some(program) = project.get_program() else {
                continue;
            };
            services.push(Arc::new(LanguageService::new(
                project.config_file_path.clone(),
                program.clone(),
                Box::new(Arc::clone(&snapshot)),
                &active_file,
            )));
        }
        services
    }

    pub fn with_snapshot_loading_project_tree(
        &self,
        requested_project_trees: HashSet<Path>,
        mut f: impl FnMut(&Arc<Snapshot>),
    ) { ::tsox_core::fntrace::enter("with_snapshot_loading_project_tree"); 
        let snapshot = self.get_snapshot(
            ResourceRequest {
                project_tree: Some(ProjectTreeRequest {
                    referenced_projects: Some(requested_project_trees),
                }),
                ..Default::default()
            },
            true,
        );
        f(&snapshot);
        snapshot.deref(self);
    }

    pub fn with_snapshot_for_document(
        &self,
        uri: &lsproto::DocumentUri,
        mut f: impl FnMut(&Arc<Snapshot>),
    ) { ::tsox_core::fntrace::enter("with_snapshot_for_document"); 
        let snapshot = self.get_snapshot(
            ResourceRequest {
                documents: vec![uri.clone()],
                ..Default::default()
            },
            true,
        );
        f(&snapshot);
        snapshot.deref(self);
    }

    pub fn with_language_service_and_snapshot(
        &self,
        uri: &lsproto::DocumentUri,
        f: impl FnOnce(
            &Arc<LanguageService>,
            &Arc<Snapshot>,
        ) -> Result<Option<Box<dyn FnOnce() -> Result<(), String>>>, String>,
    ) -> Result<Option<Box<dyn FnOnce() -> Result<(), String>>>, String> { ::tsox_core::fntrace::enter("with_language_service_and_snapshot"); 
        let (snapshot, _, language_service) = self.get_snapshot_and_default_project(uri, true)?;
        let async_work = f(&language_service, &snapshot)?;
        let Some(async_work) = async_work else {
            snapshot.deref(self);
            return Ok(None);
        };
        let session = unsafe_clone_session(self);
        Ok(Some(Box::new(move || {
            let result = async_work();
            snapshot.deref(session);
            result
        })))
    }

    pub fn get_language_service_with_auto_imports(
        &self,
        base_snapshot: &Arc<Snapshot>,
        uri: &lsproto::DocumentUri,
    ) -> Result<Arc<LanguageService>, String> { ::tsox_core::fntrace::enter("get_language_service_with_auto_imports"); 
        let new_snapshot = self.clone_with_auto_imports(base_snapshot, uri, false);
        let Some(project) = new_snapshot.get_default_project(uri) else {
            new_snapshot.deref(self);
            return Err(format!("no project found for URI {uri}"));
        };
        self.adopt_snapshot_change_in_background(base_snapshot, Arc::clone(&new_snapshot));
        Ok(Arc::new(LanguageService::new(
            project.config_file_path.clone(),
            project.get_program().expect("project program").clone(),
            Box::new(Arc::clone(&new_snapshot)),
            &uri.file_name(),
        )))
    }

    pub fn get_snapshot_with_auto_imports(
        &self,
        base_snapshot: &Arc<Snapshot>,
        uri: &lsproto::DocumentUri,
    ) -> Arc<Snapshot> { ::tsox_core::fntrace::enter("get_snapshot_with_auto_imports"); 
        let new_snapshot = self.clone_with_auto_imports(base_snapshot, uri, true);
        self.adopt_snapshot_change_in_background(base_snapshot, Arc::clone(&new_snapshot));
        new_snapshot
    }

    fn clone_with_auto_imports(
        &self,
        base_snapshot: &Arc<Snapshot>,
        uri: &lsproto::DocumentUri,
        caller_ref: bool,
    ) -> Arc<Snapshot> { ::tsox_core::fntrace::enter("clone_with_auto_imports"); 
        let change = SnapshotChange {
            reason: UpdateReason::RequestedLanguageServiceWithAutoImports,
            resource_request: ResourceRequest {
                documents: vec![uri.clone()],
                auto_imports: uri.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        let new_snapshot = base_snapshot.clone_snapshot(change, base_snapshot.overlays(), self);
        if caller_ref {
            new_snapshot.r#ref();
        }
        new_snapshot
    }

    pub fn adopt_snapshot_change_in_background(
        &self,
        base_snapshot: &Arc<Snapshot>,
        new_snapshot: Arc<Snapshot>,
    ) { ::tsox_core::fntrace::enter("adopt_snapshot_change_in_background"); 
        let base = Arc::clone(base_snapshot);
        let session = unsafe_clone_session(self);
        self.background_queue.enqueue(move || {
            session.adopt_snapshot_change(&base, &new_snapshot);
        });
    }

    pub fn adopt_snapshot_change(&self, base_snapshot: &Arc<Snapshot>, new_snapshot: &Arc<Snapshot>) { ::tsox_core::fntrace::enter("adopt_snapshot_change"); 
        let mut current = self.snapshot.write().unwrap();
        let old_snapshot = current.clone().expect("snapshot");
        if Arc::ptr_eq(&old_snapshot, base_snapshot) {
            *current = Some(Arc::clone(new_snapshot));
            drop(current);
            old_snapshot.deref(self);
            let content_mapper_timings = self.take_content_mapper_timing_delta();
            if self.options.logging_enabled {
                self.sess_log(&format!(
                    "Adopted snapshot {} (parent {}) as current session snapshot (replacing {})",
                    new_snapshot.id, new_snapshot.parent_id, old_snapshot.id
                ));
                self.sess_log(&new_snapshot.builder_logs_string());
                self.log_content_mapper_timings(&content_mapper_timings);
            }
        } else {
            drop(current);
            if self.options.logging_enabled {
                self.sess_log(&format!(
                    "Discarded snapshot {} (parent {}); session has moved on to snapshot {}",
                    new_snapshot.id, new_snapshot.parent_id, old_snapshot.id
                ));
                let logs = new_snapshot.builder_logs_string();
                if !logs.is_empty() {
                    self.sess_log(&format!(
                        "--- Discarded snapshot {} builder logs (NOT adopted) ---",
                        new_snapshot.id
                    ));
                    self.sess_log(&logs);
                    self.sess_log(&format!(
                        "--- End discarded snapshot {} builder logs ---",
                        new_snapshot.id
                    ));
                }
            }
            new_snapshot.deref(self);
        }
    }

    pub fn update_snapshot_ref(
        &self,
        overlays: HashMap<Path, Arc<Overlay>>,
        change: SnapshotChange,
    ) -> Arc<Snapshot> { ::tsox_core::fntrace::enter("update_snapshot_ref"); 
        self.update_snapshot_internal(overlays, change, true)
    }

    pub fn take_content_mapper_timing_delta(&self) -> contentmapper::Timings { ::tsox_core::fntrace::enter("take_content_mapper_timing_delta"); 
        let Some(host) = &self.content_mapper_host else {
            return contentmapper::Timings::default();
        };
        let current = host.timings();
        let mut stored = self.content_mapper_timings.lock().unwrap();
        let delta = current.since(&*stored);
        *stored = current;
        delta
    }

    pub fn log_content_mapper_timings(&self, timings: &contentmapper::Timings) { ::tsox_core::fntrace::enter("log_content_mapper_timings"); 
        if timings.request_wait.is_zero()
            && !has_content_mapper_operation_timings(&timings.mappers)
        {
            return;
        }
        self.sess_log("Content mapper timings since previous snapshot adoption:");
        if !timings.request_wait.is_zero() {
            self.sess_log(&format!("  Request wait time: {:?}", timings.request_wait));
        }
        let mut identities: Vec<&String> = timings.mappers.keys().collect();
        identities.sort();
        for identity in identities {
            let mapper = &timings.mappers[identity];
            if !has_content_mapper_operation_timing(mapper) {
                continue;
            }
            self.sess_log(&format!("  {identity}:"));
            if mapper.spawn.count != 0 {
                self.sess_log(&format!(
                    "    Initializations: {} ({:?})",
                    mapper.spawn.count,
                    mapper.spawn.duration + mapper.initialize.duration
                ));
            }
            if mapper.open_project.count != 0 {
                self.sess_log(&format!(
                    "    openProject requests: {} ({:?})",
                    mapper.open_project.count, mapper.open_project.duration
                ));
            }
            if mapper.close_project.count != 0 {
                self.sess_log(&format!(
                    "    closeProject requests: {} ({:?})",
                    mapper.close_project.count, mapper.close_project.duration
                ));
            }
            if mapper.transform.count != 0 {
                self.sess_log(&format!(
                    "    Transforms: {} ({:?})",
                    mapper.transform.count, mapper.transform.duration
                ));
            }
        }
    }
}

impl crate::ls::host::Host for Arc<Snapshot> {
    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        self.fs
            .as_ref()
            .map_or(true, |fs| fs.fs.use_case_sensitive_file_names())
    }

    fn read_file(&self, path: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        let handle = self.fs.as_ref()?.get_file(path)?;
        Some(handle.content().to_string())
    }

    fn converters(&self) -> crate::ls::lsconv_converters::Converters { ::tsox_core::fntrace::enter("converters"); 
        crate::ls::lsconv_converters::Converters::new(
            self.converters
                .as_ref()
                .map(|c| c.position_encoding.clone())
                .unwrap_or(crate::ls::lsconv_converters::PositionEncodingKind::Utf16),
        )
    }

    fn get_preferences(&self, _active_file: &str) -> crate::ls::lsutil::UserPreferences { ::tsox_core::fntrace::enter("get_preferences"); 
        self.user_preferences.clone()
    }

    fn get_ecma_line_info(&self, file_name: &str) -> Option<crate::ls::host::EcmaLineInfo> { ::tsox_core::fntrace::enter("get_ecma_line_info"); 
        self.fs
            .as_ref()?
            .get_file(file_name)
            .map(|_| crate::ls::host::EcmaLineInfo)
    }

    fn auto_import_registry(&self) -> crate::ls::host::AutoImportRegistry { ::tsox_core::fntrace::enter("auto_import_registry"); 
        crate::ls::host::AutoImportRegistry
    }

    fn read_directory(
        &self,
        current_dir: &str,
        path: &str,
        extensions: &[String],
        excludes: &[String],
        includes: &[String],
        depth: i32,
    ) -> Vec<String> { ::tsox_core::fntrace::enter("read_directory"); 
        let Some(fs) = self.fs.as_ref() else {
            return Vec::new();
        };
        let extensions: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        let excludes: Vec<&str> = excludes.iter().map(|s| s.as_str()).collect();
        let includes: Vec<&str> = includes.iter().map(|s| s.as_str()).collect();
        tsox_tsoptions::vfs::vfsmatch::read_directory(
            fs.fs.as_ref(),
            current_dir,
            path,
            &extensions,
            &excludes,
            &includes,
            depth,
        )
    }

    fn get_directories(&self, path: &str) -> Vec<String> { ::tsox_core::fntrace::enter("get_directories"); 
        self.fs
            .as_ref()
            .map(|fs| fs.fs.get_accessible_entries(path).directories)
            .unwrap_or_default()
    }

    fn directory_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("directory_exists"); 
        self.fs
            .as_ref()
            .map_or(false, |fs| fs.fs.directory_exists(path))
    }

    fn file_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("file_exists"); 
        self.fs
            .as_ref()
            .map_or(false, |fs| fs.fs.file_exists(path))
    }
}
