#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use crate::ls::lsutil::{UserPreferences, new_default_user_preferences};
use crate::lsp::lsproto;
use crate::tspath::Path;
use crate::vfs::FS;

use super::background;
use super::client::Client;
use super::compiler_host::SessionOptions;
use super::extended_config_cache::ExtendedConfigCache;
use super::file_change::{FileChange, FileChangeKind, FileChangeSummary};
use super::logging::logger::Logger;
use super::overlay_fs::{Overlay, OverlayFS};
use super::parse_cache::ParseCache;
use super::program_counter::ProgramCounter;
use super::snapshot::{Snapshot, SnapshotChange, UpdateReason};
use super::watch::WatchRegistry;

pub const WATCH_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

pub const IDLE_CACHE_CLEAN_DELAY: Duration = Duration::from_secs(30);

pub const PERFORMANCE_TELEMETRY_INTERVAL: Duration = Duration::from_secs(300);

pub struct SessionInit {
    pub options: SessionOptions,
    pub fs: Arc<dyn FS>,
    pub client: Option<Arc<dyn Client>>,
    pub parse_cache: Option<Arc<ParseCache>>,
    pub logger: Option<Arc<dyn Logger>>,
}

pub struct Session {
    pub options: SessionOptions,
    pub start_time: Instant,
    pub to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
    pub client: Option<Arc<dyn Client>>,
    pub logger: Option<Arc<dyn Logger>>,

    pub fs: Option<Arc<OverlayFS>>,
    pub parse_cache: Option<Arc<ParseCache>>,
    pub extended_config_cache: Option<Arc<ExtendedConfigCache>>,
    pub program_counter: Option<Arc<ProgramCounter>>,
    pub background_queue: Arc<background::Queue>,

    pub snapshot_id: AtomicU64,

    snapshot: RwLock<Option<Arc<Snapshot>>>,

    pub pending_file_changes: Mutex<Vec<FileChange>>,
    pub pending_ata_changes: Mutex<HashMap<Path, super::snapshot::ATAStateChange>>,

    pub watches: Arc<WatchRegistry>,
    pub seen_projects: Mutex<HashSet<Path>>,
    pub global_diag_publish_pending: AtomicBool,

    workspace_user_preferences: Mutex<UserPreferences>,

    pending_user_config_changes: AtomicBool,

    scheduled_snapshot_update_generation: AtomicU64,
    scheduled_snapshot_update_at: Mutex<Option<Instant>>,

    diagnostics_refresh_generation: AtomicU64,
    diagnostics_refresh_at: Mutex<Option<Instant>>,

    idle_cache_clean_at: Mutex<Option<Instant>>,
    warm_auto_import_active: AtomicBool,
}

impl Session {

    pub fn new(init: SessionInit) -> Self {
        let current_directory = init.options.current_directory.clone();
        let use_case_sensitive = init.fs.use_case_sensitive_file_names();

        let cd_for_fs = current_directory.clone();
        let to_path_for_fs: Box<dyn Fn(&str) -> Path + Send + Sync> =
            Box::new(move |file_name: &str| {
                crate::tspath::to_path(file_name, &cd_for_fs, use_case_sensitive)
            });
        let to_path: Box<dyn Fn(&str) -> Path + Send + Sync> = Box::new(move |file_name: &str| {
            crate::tspath::to_path(file_name, &current_directory, use_case_sensitive)
        });

        let position_encoding = init.options.position_encoding.clone();
        let overlay_fs = Arc::new(OverlayFS::new(
            Arc::clone(&init.fs),
            HashMap::new(),
            position_encoding,
            to_path_for_fs,
        ));

        let parse_cache = init.parse_cache.unwrap_or_else(|| {
            Arc::new(ParseCache::new(
                super::refcount_cache::RefCountCacheOptions::default(),
            ))
        });

        let snapshot = Arc::new(Snapshot::new(0));

        Session {
            options: init.options,
            start_time: Instant::now(),
            to_path,
            client: init.client,
            logger: init.logger,
            fs: Some(overlay_fs),
            parse_cache: Some(parse_cache),
            extended_config_cache: Some(Arc::new(ExtendedConfigCache::new())),
            program_counter: Some(Arc::new(ProgramCounter::new())),
            background_queue: Arc::new(background::Queue::new()),
            snapshot_id: AtomicU64::new(0),
            snapshot: RwLock::new(Some(snapshot)),
            pending_file_changes: Mutex::new(Vec::new()),
            pending_ata_changes: Mutex::new(HashMap::new()),
            watches: Arc::new(WatchRegistry::new()),
            seen_projects: Mutex::new(HashSet::new()),
            global_diag_publish_pending: AtomicBool::new(false),
            workspace_user_preferences: Mutex::new(new_default_user_preferences()),
            pending_user_config_changes: AtomicBool::new(false),
            scheduled_snapshot_update_generation: AtomicU64::new(0),
            scheduled_snapshot_update_at: Mutex::new(None),
            diagnostics_refresh_generation: AtomicU64::new(0),
            diagnostics_refresh_at: Mutex::new(None),
            idle_cache_clean_at: Mutex::new(None),
            warm_auto_import_active: AtomicBool::new(false),
        }
    }

    pub fn fs(&self) -> Option<&Arc<dyn FS>> {
        self.fs.as_ref().map(|ofs| &ofs.fs)
    }

    pub fn current_directory(&self) -> &str {
        &self.options.current_directory
    }

    pub fn snapshot(&self) -> Option<Arc<Snapshot>> {
        self.snapshot.read().unwrap().clone()
    }

    pub fn config(&self) -> UserPreferences {
        self.workspace_user_preferences.lock().unwrap().clone()
    }

    pub fn configure(&self, config: UserPreferences) {
        let mut prefs = self.workspace_user_preferences.lock().unwrap();
        let old = prefs.clone();
        self.pending_user_config_changes
            .store(true, Ordering::SeqCst);
        *prefs = config;
        drop(prefs);

        self.refresh_inlay_hints_if_needed(&old);
        self.refresh_code_lens_if_needed(&old);
        self.refresh_diagnostics_if_needed(&old);
        self.refresh_ata_if_needed(&old);
    }

    pub fn initialize_with_user_config(&self, config: UserPreferences) {
        self.configure(config);
    }

    pub fn did_open_file(
        &self,
        uri: &lsproto::DocumentUri,
        version: i32,
        content: &str,
        language_kind: &lsproto::LanguageKind,
    ) {
        self.cancel_warm_auto_import_cache();
        self.schedule_idle_cache_clean();
        self.cancel_scheduled_snapshot_update();

        {
            let mut pending = self.pending_file_changes.lock().unwrap();
            pending.push(FileChange {
                kind: FileChangeKind::Open,
                uri: uri.clone(),
                version,
                content: content.to_string(),
                language_kind: language_kind.clone(),
                ..Default::default()
            });
        }

        let (changes, overlays) = self.flush_changes();
        self.update_snapshot(
            overlays,
            SnapshotChange {
                reason: UpdateReason::DidOpenFile,
                file_changes: changes,
                resource_request: super::snapshot::ResourceRequest {
                    documents: vec![uri.clone()],
                    ..Default::default()
                },
                ..Default::default()
            },
        );
    }

    pub fn did_close_file(&self, uri: &lsproto::DocumentUri) {
        self.cancel_warm_auto_import_cache();
        self.schedule_idle_cache_clean();
        {
            let mut pending = self.pending_file_changes.lock().unwrap();
            pending.push(FileChange {
                kind: FileChangeKind::Close,
                uri: uri.clone(),
                ..Default::default()
            });
        }
        self.schedule_snapshot_update(UpdateReason::DidCloseFile);
    }

    pub fn did_change_file(
        &self,
        uri: &lsproto::DocumentUri,
        version: i32,
        changes: &[lsproto::TextDocumentContentChangePartialOrWholeDocument],
    ) {
        self.cancel_diagnostics_refresh();
        self.cancel_warm_auto_import_cache();
        self.schedule_idle_cache_clean();
        let mut pending = self.pending_file_changes.lock().unwrap();
        pending.push(FileChange {
            kind: FileChangeKind::Change,
            uri: uri.clone(),
            version,
            changes: changes.to_vec(),
            ..Default::default()
        });
    }

    pub fn did_save_file(&self, uri: &lsproto::DocumentUri) {
        self.schedule_idle_cache_clean();
        let mut pending = self.pending_file_changes.lock().unwrap();
        pending.push(FileChange {
            kind: FileChangeKind::Save,
            uri: uri.clone(),
            ..Default::default()
        });
    }

    pub fn did_change_watched_files(&self, changes: &[lsproto::FileEvent]) {
        let mut file_changes: Vec<FileChange> = Vec::with_capacity(changes.len());
        for change in changes {
            let kind = match change.change_type {
                lsproto::FILE_CHANGE_TYPE_CREATED => FileChangeKind::WatchCreate,
                lsproto::FILE_CHANGE_TYPE_CHANGED => FileChangeKind::WatchChange,
                lsproto::FILE_CHANGE_TYPE_DELETED => FileChangeKind::WatchDelete,
                _ => continue,
            };
            file_changes.push(FileChange {
                kind,
                uri: change.uri.clone(),
                ..Default::default()
            });
        }

        if !file_changes.is_empty() {
            let mut pending = self.pending_file_changes.lock().unwrap();
            pending.extend(file_changes);
        }

        self.schedule_diagnostics_refresh();
        self.cancel_warm_auto_import_cache();
        self.schedule_idle_cache_clean();
    }

    pub fn did_change_compiler_options_for_inferred_projects(
        &self,
        options: Option<crate::core::compiler_options::CompilerOptions>,
    ) {
        let overlays = self
            .fs
            .as_ref()
            .map(|ofs| ofs.overlays())
            .unwrap_or_default();
        self.update_snapshot(
            overlays,
            SnapshotChange {
                reason: UpdateReason::DidChangeCompilerOptionsForInferredProjects,
                compiler_options_for_inferred_projects: options,
                ..Default::default()
            },
        );
    }

    pub fn schedule_snapshot_update(&self, reason: UpdateReason) {
        let _ = reason;
        self.scheduled_snapshot_update_generation
            .fetch_add(1, Ordering::SeqCst);
        let delay = self.options.debounce_delay;
        *self.scheduled_snapshot_update_at.lock().unwrap() = Some(Instant::now() + delay);
    }

    pub fn snapshot_update_due(&self) -> bool {
        match *self.scheduled_snapshot_update_at.lock().unwrap() {
            Some(when) => Instant::now() >= when,
            None => false,
        }
    }

    pub fn cancel_scheduled_snapshot_update(&self) {
        self.scheduled_snapshot_update_generation
            .fetch_add(1, Ordering::SeqCst);
        *self.scheduled_snapshot_update_at.lock().unwrap() = None;
    }

    pub fn schedule_diagnostics_refresh(&self) {
        self.diagnostics_refresh_generation
            .fetch_add(1, Ordering::SeqCst);
        let delay = self.options.debounce_delay;
        *self.diagnostics_refresh_at.lock().unwrap() = Some(Instant::now() + delay);

        if let Some(client) = &self.client {
            let client = Arc::clone(client);
            self.background_queue.enqueue(move || {
                std::thread::sleep(delay);
                let _ = client.refresh_diagnostics();
            });
        }
    }

    pub fn diagnostics_refresh_due(&self) -> bool {
        match *self.diagnostics_refresh_at.lock().unwrap() {
            Some(when) => Instant::now() >= when,
            None => false,
        }
    }

    pub fn cancel_diagnostics_refresh(&self) {
        self.diagnostics_refresh_generation
            .fetch_add(1, Ordering::SeqCst);
        *self.diagnostics_refresh_at.lock().unwrap() = None;
    }

    pub fn cancel_warm_auto_import_cache(&self) {
        self.warm_auto_import_active.store(false, Ordering::SeqCst);
    }

    pub fn schedule_idle_cache_clean(&self) {
        *self.idle_cache_clean_at.lock().unwrap() = Some(Instant::now() + IDLE_CACHE_CLEAN_DELAY);
    }

    pub fn cancel_idle_cache_clean(&self) {
        *self.idle_cache_clean_at.lock().unwrap() = None;
    }

    pub fn close(&self) {
        self.cancel_scheduled_snapshot_update();
        self.cancel_diagnostics_refresh();
        self.cancel_warm_auto_import_cache();
        self.cancel_idle_cache_clean();
        self.stop_performance_telemetry();
        self.background_queue.close();
    }

    pub fn wait_for_background_tasks(&self) {
        self.cancel_idle_cache_clean();
        self.background_queue.wait();
    }

    pub fn flush_changes(&self) -> (FileChangeSummary, HashMap<Path, Arc<Overlay>>) {
        let pending = {
            let mut guard = self.pending_file_changes.lock().unwrap();
            std::mem::take(&mut *guard)
        };

        if pending.is_empty() {
            let overlays = self
                .fs
                .as_ref()
                .map(|ofs| ofs.overlays())
                .unwrap_or_default();
            return (FileChangeSummary::default(), overlays);
        }

        match &self.fs {
            Some(fs) => fs.process_changes(&pending),
            None => (FileChangeSummary::default(), HashMap::new()),
        }
    }

    pub fn get_snapshot(&self, request: super::snapshot::ResourceRequest) -> Arc<Snapshot> {
        self.cancel_scheduled_snapshot_update();

        let (file_changes, overlays) = self.flush_changes();
        let ata_changes = self.pending_ata_changes.lock().unwrap().drain().count();
        let new_config = if self
            .pending_user_config_changes
            .swap(false, Ordering::SeqCst)
        {
            Some(self.config())
        } else {
            None
        };

        if !file_changes.is_empty() || ata_changes > 0 || new_config.is_some() {
            return self.update_snapshot(
                overlays,
                SnapshotChange {
                    reason: UpdateReason::RequestedLanguageServicePendingChanges,
                    file_changes,
                    resource_request: request,
                    ..Default::default()
                },
            );
        }

        self.snapshot.read().unwrap().clone().unwrap_or_else(|| {
            self.update_snapshot(
                overlays,
                SnapshotChange {
                    reason: UpdateReason::Unknown,
                    resource_request: request,
                    ..Default::default()
                },
            )
        })
    }

    pub fn update_snapshot(
        &self,
        _overlays: HashMap<Path, Arc<Overlay>>,
        change: SnapshotChange,
    ) -> Arc<Snapshot> {
        let new_id = self.snapshot_id.fetch_add(1, Ordering::SeqCst) + 1;

        let prev = self.snapshot.read().unwrap().clone();
        let parent_id = prev.as_ref().map(|s| s.id).unwrap_or(0);

        let mut new_snapshot = Snapshot::new(new_id);
        new_snapshot.parent_id = parent_id;

        if let Some(prev) = &prev {
            new_snapshot.fs = prev.fs.clone();
            new_snapshot.compiler_options_for_inferred_projects = change
                .compiler_options_for_inferred_projects
                .clone()
                .or_else(|| prev.compiler_options_for_inferred_projects.clone());
        } else {
            new_snapshot.compiler_options_for_inferred_projects =
                change.compiler_options_for_inferred_projects.clone();
        }

        let new_arc = Arc::new(new_snapshot);

        *self.snapshot.write().unwrap() = Some(Arc::clone(&new_arc));

        new_arc
    }

    pub fn get_language_service(
        &self,
        _uri: &lsproto::DocumentUri,
    ) -> Option<crate::ls::language_service::LanguageService> {

        let _snapshot = self.get_snapshot(super::snapshot::ResourceRequest {
            documents: vec![_uri.clone()],
            ..Default::default()
        });

        None
    }

    pub fn start_performance_telemetry(&self) {
        if !self.options.telemetry_enabled {
            return;
        }

    }

    pub fn stop_performance_telemetry(&self) {

    }

    pub fn send_project_info_telemetry(&self, _old_snapshot: &Snapshot, _new_snapshot: &Snapshot) {
        if !self.options.telemetry_enabled {
            return;
        }

    }

    pub fn mark_project_seen(&self, project_path: &Path) {
        self.seen_projects
            .lock()
            .unwrap()
            .insert(project_path.clone());
    }

    pub fn has_seen_project(&self, project_path: &Path) -> bool {
        self.seen_projects.lock().unwrap().contains(project_path)
    }

    fn refresh_inlay_hints_if_needed(&self, old_prefs: &UserPreferences) {
        if old_prefs.inlay_hints != self.config().inlay_hints {
            if let Some(client) = &self.client {
                let _ = client.refresh_inlay_hints();
            }
        }
    }

    fn refresh_code_lens_if_needed(&self, old_prefs: &UserPreferences) {
        if old_prefs.code_lens != self.config().code_lens {
            if let Some(client) = &self.client {
                let _ = client.refresh_code_lens();
            }
        }
    }

    fn refresh_diagnostics_if_needed(&self, old_prefs: &UserPreferences) {
        let new_prefs = self.config();
        if old_prefs.custom_config_file_name != new_prefs.custom_config_file_name
            || old_prefs.report_style_checks_as_warnings
                != new_prefs.report_style_checks_as_warnings
            || old_prefs.enable_validation != new_prefs.enable_validation
        {
            self.schedule_diagnostics_refresh();
        }
    }

    fn refresh_ata_if_needed(&self, old_prefs: &UserPreferences) {
        if old_prefs.is_ata_disabled() && !self.config().is_ata_disabled() {

            self.schedule_diagnostics_refresh();
        }
    }
}
