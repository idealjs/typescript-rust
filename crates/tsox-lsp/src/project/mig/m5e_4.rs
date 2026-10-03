#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tsox_core::tspath::Path;

use crate::project::session::*;

use crate::project::project::{HR, Kind as ProjectKind, ProgramUpdateKind, Project};
use crate::project::project_collection::ProjectCollection;

pub(crate) use super::m5e::{background, contentmapper, unsafe_clone_session};

mod ast {
    pub use tsox_frontend::ast::Diagnostic;
}

mod lsconv {
    pub use crate::ls::lsconv_converters::{Converters, file_name_to_document_uri};
    pub use crate::mig::m5u_conv::diagnostic_to_lsp_push;
}

fn program_update_kind_rank(kind: &ProgramUpdateKind) -> u8 { ::tsox_core::fntrace::enter("program_update_kind_rank"); 
    match kind {
        ProgramUpdateKind::None => 0,
        ProgramUpdateKind::Cloned => 1,
        ProgramUpdateKind::SameFileNames => 2,
        ProgramUpdateKind::NewFiles => 3,
    }
}

/// Go snapshot.ProjectCollection 非空指针不变式（NewSnapshot 必建），
/// Rust 侧 Snapshot.project_collection 为 Option，按 m5e_5::Snapshot::projects 同规取 expect
fn snapshot_project_collection(snapshot: &Snapshot) -> &ProjectCollection { ::tsox_core::fntrace::enter("snapshot_project_collection"); 
    snapshot.project_collection.as_deref().expect("project collection")
}

pub fn should_publish_program_diagnostics(p: &Project, snapshot_id: u64) -> bool { ::tsox_core::fntrace::enter("should_publish_program_diagnostics"); 
    if p.kind != ProjectKind::Configured || p.program.is_none() || p.program_last_update != snapshot_id
    {
        return false;
    }
    program_update_kind_rank(&p.program_update_kind)
        > program_update_kind_rank(&ProgramUpdateKind::Cloned)
}

impl Snapshot {
    pub fn converters(&self) -> &crate::mig::m5u_conv::M5uConverters { ::tsox_core::fntrace::enter("converters"); 
        self.converters
            .as_ref()
            .expect("snapshot converters")
    }

    pub fn fs_realpath_alias_count(&self) -> usize { ::tsox_core::fntrace::enter("fs_realpath_alias_count"); 
        self.fs
            .as_ref()
            .map_or(0, |fs| fs.node_modules_realpath_aliases.len())
    }

    pub fn config_file_registry_content_mappers(
        &self,
    ) -> super::m5c_3::ConfiguredContentMappers { ::tsox_core::fntrace::enter("config_file_registry_content_mappers"); 
        self.config_file_registry
            .as_deref()
            .map(|registry| registry.content_mappers())
            .unwrap_or_default()
    }
}

impl Session {
    pub(crate) fn sess_log(&self, msg: &str) { ::tsox_core::fntrace::enter("sess_log"); 
        if let Some(logger) = &self.logger {
            logger.log(msg);
        }
    }

    pub(crate) fn logger_is_verbose(&self) -> bool { ::tsox_core::fntrace::enter("logger_is_verbose"); 
        self.logger.as_ref().map(|l| l.is_verbose()).unwrap_or(false)
    }

    pub fn fs_overlays(&self) -> HashMap<Path, Arc<Overlay>> { ::tsox_core::fntrace::enter("fs_overlays"); 
        self.fs.as_ref().map(|fs| fs.overlays()).unwrap_or_default()
    }

    pub fn fs_process_changes(
        &self,
        pending: &[FileChange],
    ) -> (FileChangeSummary, HashMap<Path, Arc<Overlay>>) { ::tsox_core::fntrace::enter("fs_process_changes"); 
        self.fs
            .as_ref()
            .map(|fs| fs.process_changes(pending))
            .unwrap_or_default()
    }

    pub fn parse_cache_len(&self) -> usize { ::tsox_core::fntrace::enter("parse_cache_len"); 
        self.parse_cache.as_ref().map_or(0, |c| c.len())
    }

    pub fn extended_config_cache_len(&self) -> usize { ::tsox_core::fntrace::enter("extended_config_cache_len"); 
        self.extended_config_cache.as_ref().map_or(0, |c| c.len())
    }

    pub fn program_counter_len(&self) -> usize { ::tsox_core::fntrace::enter("program_counter_len"); 
        self.program_counter.as_ref().map(|c| c.len()).unwrap_or(0)
    }

    pub fn update_snapshot(
        &self,
        overlays: HashMap<Path, Arc<Overlay>>,
        change: SnapshotChange,
    ) -> Arc<Snapshot> { ::tsox_core::fntrace::enter("update_snapshot"); 
        self.update_snapshot_internal(overlays, change, false)
    }

    pub(crate) fn update_snapshot_internal(
        &self,
        overlays: HashMap<Path, Arc<Overlay>>,
        change: SnapshotChange,
        caller_ref: bool,
    ) -> Arc<Snapshot> { ::tsox_core::fntrace::enter("update_snapshot_internal"); 
        let old_snapshot = self.snapshot().expect("snapshot");
        let new_snapshot = old_snapshot.clone_snapshot(change.clone(), Some(&overlays), self);
        *self.snapshot.write().unwrap() = Some(Arc::clone(&new_snapshot));
        if caller_ref {
            new_snapshot.r#ref();
        }
        let mut content_mapper_timings = contentmapper::Timings::default();
        if !Arc::ptr_eq(&new_snapshot, &old_snapshot) {
            old_snapshot.deref(self);
            content_mapper_timings = self.take_content_mapper_timing_delta();
        }
        if self.typings_installer.is_some() && !self.config().is_ata_disabled() {
            self.trigger_ata_for_updated_projects(&new_snapshot);
        }
        let old = Arc::clone(&old_snapshot);
        let new = Arc::clone(&new_snapshot);
        let session = unsafe_clone_session(self);
        let change = change.clone();
        self.background_queue.enqueue(move || {
            if session.options.logging_enabled {
                session.sess_log(&format!(
                    "Adopted snapshot {} (parent {}) as current session snapshot (replacing {})",
                    new.id, new.parent_id, old.id
                ));
                session.sess_log(&new.builder_logs_string());
                session.log_project_changes(&old, &new);
                session.log_content_mapper_timings(&content_mapper_timings);
                session.sess_log("");
            }
            if session.options.watch_enabled
                && let Err(err) = session.update_watches(&old, &new)
                && session.options.logging_enabled
            {
                session.sess_log(&format!("{err}"));
            }
            let _ = session.update_content_mapper_registrations(&new);
            session.publish_program_diagnostics(&old, &new);
            session.send_project_info_telemetry_for_new_projects(&old, &new);
            session.warm_auto_import_cache(&change, &old, &new);
        });
        new_snapshot
    }

    pub fn update_content_mapper_registrations(&self, snapshot: &Arc<Snapshot>) -> Result<(), String> { ::tsox_core::fntrace::enter("update_content_mapper_registrations"); 
        let Some(client) = &self.client else { return Ok(()) };
        let content_mappers = snapshot.config_file_registry_content_mappers();
        let mut extensions = content_mappers.extensions.clone();
        extensions.extend(snapshot.inferred_project_content_mapper_extensions.iter().cloned());
        extensions.sort();
        extensions.dedup();
        let mut registered = self.registered_content_mapper_extensions.lock().unwrap();
        if snapshot.id() <= self
            .registered_content_mapper_snapshot_id
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Ok(());
        }
        if extensions == *registered {
            self.registered_content_mapper_snapshot_id
                .store(snapshot.id(), std::sync::atomic::Ordering::SeqCst);
            return Ok(());
        }
        if let Err(err) = client.register_content_mapper_extensions(&extensions) {
            if self.options.logging_enabled {
                self.sess_log(&format!("{err}"));
            }
            return Err(err);
        }
        *registered = extensions;
        self.registered_content_mapper_snapshot_id
            .store(snapshot.id(), std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    pub fn flush_changes_locked(&self) -> (FileChangeSummary, HashMap<Path, Arc<Overlay>>) { ::tsox_core::fntrace::enter("flush_changes_locked"); 
        let pending = self.pending_file_changes.lock().unwrap();
        if pending.is_empty() {
            return (FileChangeSummary::default(), self.fs_overlays());
        }
        let start = std::time::Instant::now();
        let (changes, overlays) = self.fs_process_changes(&pending);
        if self.options.logging_enabled {
            self.sess_log(&format!(
                "Processed {} file changes in {:?}",
                pending.len(),
                start.elapsed()
            ));
        }
        drop(pending);
        *self.pending_file_changes.lock().unwrap() = Vec::new();
        (changes, overlays)
    }

    pub fn log_project_changes(&self, old_snapshot: &Snapshot, new_snapshot: &Snapshot) { ::tsox_core::fntrace::enter("log_project_changes"); 
        let mut logged_project_changes = false;
        let mut log_project = |project: &Project| {
            let mut builder = String::new();
            project.print(self.logger_is_verbose(), self.logger_is_verbose(), &mut builder);
            self.sess_log(&builder);
            logged_project_changes = true;
        };
        let old_projects = snapshot_project_collection(old_snapshot).projects_by_path();
        let new_projects = snapshot_project_collection(new_snapshot).projects_by_path();
        for (_path, project) in &new_projects {
            if !old_projects.iter().any(|(p, _)| p == _path) {
                log_project(project);
            }
        }
        for (_path, project) in &old_projects {
            if !new_projects.iter().any(|(p, _)| p == _path) {
                self.sess_log(&format!(
                    "\nProject '{}' removed\n{}",
                    project.name(),
                    HR
                ));
            }
        }
        for (_path, project) in &new_projects {
            if old_projects.iter().any(|(p, old_project)| {
                p == _path && project.program_update_kind == ProgramUpdateKind::NewFiles
            }) {
                log_project(project);
            }
        }
        if logged_project_changes || self.logger_is_verbose() {
            self.log_cache_stats(new_snapshot);
        }
    }

    pub fn log_cache_stats(&self, snapshot: &Snapshot) { ::tsox_core::fntrace::enter("log_cache_stats"); 
        self.sess_log("\n======== Cache Statistics ========");
        self.sess_log(&format!(
            "Open file count:   {:6}",
            snapshot.fs.as_ref().map_or(0, |fs| fs.overlays.len())
        ));
        self.sess_log(&format!(
            "Cached disk files: {:6}",
            snapshot.fs.as_ref().map_or(0, |fs| fs.disk_files.len())
        ));
        self.sess_log(&format!("Realpath aliases:  {:6}", snapshot.fs_realpath_alias_count()));
        self.sess_log(&format!(
            "Project count:     {:6}",
            snapshot_project_collection(snapshot).projects().len()
        ));
        self.sess_log(&format!(
            "Config count:      {:6}",
            snapshot
                .config_file_registry
                .as_deref()
                .map_or(0, |r| r.configs.len())
        ));
        if self.logger_is_verbose() {
            let parse_cache_size = self.parse_cache_len();
            let extended_config_count = self.extended_config_cache_len();
            self.sess_log(&format!("Parse cache size:           {:6}", parse_cache_size));
            self.sess_log(&format!("Program count:              {:6}", self.program_counter_len()));
            self.sess_log(&format!(
                "Extended config cache size: {:6}",
                extended_config_count
            ));
            self.sess_log("Auto Imports:");
            if let Some(registry) = snapshot.auto_import_registry() {
                let stats = registry.get_cache_stats();
                self.sess_log(&format!("\tUnique packages (by realpath): {}", stats.unique_package_count));
                if !stats.project_buckets.is_empty() {
                    self.sess_log("\tProject buckets:");
                    for bucket in &stats.project_buckets {
                        self.sess_log(&format!(
                            "\t\t{}{}:",
                            bucket.path,
                            if bucket.state.dirty() { " (dirty)" } else { "" }
                        ));
                        self.sess_log(&format!("\t\t\tFiles: {}", bucket.file_count));
                        self.sess_log(&format!("\t\t\tExports: {}", bucket.export_count));
                    }
                }
                if !stats.node_modules_buckets.is_empty() {
                    self.sess_log("\tnode_modules buckets:");
                    for bucket in &stats.node_modules_buckets {
                        self.sess_log(&format!(
                            "\t\t{}{}:",
                            bucket.path,
                            if bucket.state.dirty() { " (dirty)" } else { "" }
                        ));
                        if let Some(dirty) = bucket.state.dirty_packages() {
                            for package_name in dirty.iter() {
                                self.sess_log(&format!("\t\t\tNeeds granular update: {package_name}"));
                            }
                        }
                        match &bucket.dependency_names {
                            Some(names) => self.sess_log(&format!("\t\t\tCollected packages: {}", names.len())),
                            None => self.sess_log(
                                "\t\t\tCollected packages: all, due to no package.json!",
                            ),
                        }
                        self.sess_log(&format!(
                            "\t\t\tTotal packages: {}",
                            bucket.package_names.as_ref().map_or(0, |s| s.len())
                        ));
                        self.sess_log(&format!("\t\t\tFiles: {}", bucket.file_count));
                        self.sess_log(&format!("\t\t\tExports: {}", bucket.export_count));
                        match bucket.state.recursive_search_packages() {
                            None => self.sess_log("\t\t\tRecursive search: all"),
                            Some(packages) if !packages.is_empty() => self.sess_log(&format!(
                                "\t\t\tRecursive search: {} packages",
                                packages.len()
                            )),
                            Some(_) => self.sess_log("\t\t\tRecursive search: none"),
                        }
                    }
                }
            }
        }
    }

    pub fn npm_install(&self, cwd: &str, npm_install_args: &[String]) -> Result<Vec<u8>, String> { ::tsox_core::fntrace::enter("npm_install"); 
        self.npm_executor
            .as_ref()
            .expect("npm executor")
            .npm_install(cwd, npm_install_args)
    }

    pub fn publish_program_diagnostics(&self, old_snapshot: &Snapshot, new_snapshot: &Snapshot) { ::tsox_core::fntrace::enter("publish_program_diagnostics"); 
        if !self.options.push_diagnostics_enabled {
            return;
        }
        if new_snapshot.user_preferences().enable_validation.is_false() {
            if old_snapshot.user_preferences().enable_validation.is_false() {
                return;
            }
            for (config_file_path, old_project) in snapshot_project_collection(old_snapshot).projects_by_path() {
                if old_project.kind == ProjectKind::Configured
                    && snapshot_project_collection(old_snapshot)
                        .get_open_configured_projects()
                        .contains(&config_file_path)
                {
                    self.publish_project_diagnostics(
                        config_file_path.as_str(),
                        Vec::new(),
                        old_snapshot.converters(),
                    );
                }
            }
            return;
        }
        let old_projects = snapshot_project_collection(old_snapshot).projects_by_path();
        let new_projects = snapshot_project_collection(new_snapshot).projects_by_path();
        let old_open_projects = snapshot_project_collection(old_snapshot).get_open_configured_projects();
        let new_open_projects = snapshot_project_collection(new_snapshot).get_open_configured_projects();
        for (config_file_path, added_project) in &new_projects {
            if !old_projects.iter().any(|(p, _)| p == config_file_path) {
                if should_publish_program_diagnostics(added_project, new_snapshot.id())
                    && new_open_projects.contains(config_file_path)
                {
                    self.publish_project_diagnostics(
                        config_file_path.as_str(),
                        added_project.get_project_diagnostics(),
                        new_snapshot.converters(),
                    );
                }
            }
        }
        for (config_file_path, removed_project) in &old_projects {
            if !new_projects.iter().any(|(p, _)| p == config_file_path)
                && removed_project.kind == ProjectKind::Configured
            {
                self.publish_project_diagnostics(
                    config_file_path.as_str(),
                    Vec::new(),
                    old_snapshot.converters(),
                );
            }
        }
        for (config_file_path, new_project) in &new_projects {
            if new_project.kind != ProjectKind::Configured
                || !old_projects.iter().any(|(p, _)| p == config_file_path)
            {
                continue;
            }
            let old_project = old_projects
                .iter()
                .find(|(p, _)| p == config_file_path)
                .map(|(_, project)| *project);
            let new_has_open_files = new_open_projects.contains(config_file_path);
            let old_has_open_files = old_open_projects.contains(config_file_path);
            if new_has_open_files
                && !old_has_open_files
                && (old_project.is_some_and(|old_project| std::ptr::eq(*new_project, old_project))
                    || !should_publish_program_diagnostics(new_project, new_snapshot.id()))
            {
                self.publish_project_diagnostics(
                    config_file_path.as_str(),
                    new_project.get_project_diagnostics(),
                    new_snapshot.converters(),
                );
            } else if !new_has_open_files && old_has_open_files {
                self.publish_project_diagnostics(
                    config_file_path.as_str(),
                    Vec::new(),
                    new_snapshot.converters(),
                );
            }
        }
    }

    pub fn publish_project_diagnostics(
        &self,
        config_file_path: &str,
        diagnostics: Vec<Arc<ast::Diagnostic>>,
        _converters: &crate::mig::m5u_conv::M5uConverters,
    ) { ::tsox_core::fntrace::enter("publish_project_diagnostics"); 
        let mut diagnostics = diagnostics;
        if self.config().enable_validation.is_false() {
            diagnostics = Vec::new();
        }
        let _ctx = self.with_current_locale(self.background_context());
        let mut lsp_diagnostics = Vec::with_capacity(diagnostics.len());
        for diag in &diagnostics {
            lsp_diagnostics.push(ast_diagnostic_to_protocol(diag));
        }
        if let Some(client) = &self.client
            && let Err(err) = client.publish_diagnostics(&lsproto::PublishDiagnosticsParams {
                uri: lsproto::DocumentUri(lsconv::file_name_to_document_uri(config_file_path)),
                version: None,
                diagnostics: lsp_diagnostics,
            })
            && self.options.logging_enabled
        {
            self.sess_log(&format!("Error publishing diagnostics: {err}"));
        }
    }

    pub fn publish_global_diagnostics(&self) { ::tsox_core::fntrace::enter("publish_global_diagnostics"); 
        self.global_diag_publish_pending
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let Some(snapshot) = self.snapshot() else { return };
        snapshot.r#ref();
        for project in snapshot.projects() {
            if project.kind != ProjectKind::Configured {
                continue;
            }
            let Some(checker_pool) = project.checker_pool() else {
                continue;
            };
            if checker_pool.take_new_global_diagnostics() {
                self.publish_project_diagnostics(
                    project.config_file_path.as_str(),
                    project.get_project_diagnostics(),
                    snapshot.converters(),
                );
            }
        }
        snapshot.deref(self);
    }
}

/// Go lsconv.DiagnosticToLSPPush 的最小移植：ast::Diagnostic 转 lsproto 协议结构，
/// 行列号取自诊断自带 SourceFile 的 line_map（无文件时退化为第 0 行偏移）
fn ast_diagnostic_to_protocol(diag: &ast::Diagnostic) -> lsproto::Diagnostic { ::tsox_core::fntrace::enter("ast_diagnostic_to_protocol"); 
    let message = diagnostic_message_text(diag);
    let (start, end) = match &diag.file {
        Some(file) => (
            offset_to_protocol_position(&file.line_map, diag.loc.pos()),
            offset_to_protocol_position(&file.line_map, diag.loc.end()),
        ),
        None => (
            lsproto::Position {
                line: 0,
                character: diag.loc.pos() as u32,
            },
            lsproto::Position {
                line: 0,
                character: diag.loc.end() as u32,
            },
        ),
    };
    lsproto::Diagnostic {
        range: lsproto::Range { start, end },
        severity: Some(diagnostic_category_to_severity(diag.category) as i32),
        code: Some(serde_json::Value::Number(serde_json::Number::from(diag.code))),
        source: Some("typescript".to_string()),
        message,
    }
}

fn diagnostic_message_text(diag: &ast::Diagnostic) -> String { ::tsox_core::fntrace::enter("diagnostic_message_text"); 
    if let Some(msg) = &diag.message {
        let args: Vec<&str> = diag.message_args.iter().map(|s| s.as_str()).collect();
        let text = tsox_core::diagnostics::format_message(msg.text, &args);
        if !text.is_empty() {
            return text;
        }
    }
    format!("TS{}", diag.code)
}

fn diagnostic_category_to_severity(category: tsox_core::diagnostics::Category) -> u32 { ::tsox_core::fntrace::enter("diagnostic_category_to_severity"); 
    match category {
        tsox_core::diagnostics::Category::Error => 1,
        tsox_core::diagnostics::Category::Warning => 2,
        // Go lsconv：Suggestion→Hint(4)、Message→Information(3)
        tsox_core::diagnostics::Category::Suggestion => 4,
        tsox_core::diagnostics::Category::Message => 3,
    }
}

fn offset_to_protocol_position(
    line_map: &tsox_frontend::ast::node::LineMap,
    offset: usize,
) -> lsproto::Position { ::tsox_core::fntrace::enter("offset_to_protocol_position"); 
    let line = match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    lsproto::Position {
        line: line as u32,
        character: offset.saturating_sub(line_start) as u32,
    }
}
