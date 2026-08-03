//! Session — central LSP state (1:1 port of Go's `internal/project/session.go`).

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

/// Watch request timeout for client calls.
pub const WATCH_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

/// Idle cache clean delay.
pub const IDLE_CACHE_CLEAN_DELAY: Duration = Duration::from_secs(30);

/// Periodic performance telemetry interval.
pub const PERFORMANCE_TELEMETRY_INTERVAL: Duration = Duration::from_secs(300);

/// Session initialization parameters.
///
/// Go: `type SessionInit struct { ... }`.
pub struct SessionInit {
    pub options: SessionOptions,
    pub fs: Arc<dyn FS>,
    pub client: Option<Arc<dyn Client>>,
    pub parse_cache: Option<Arc<ParseCache>>,
    pub logger: Option<Arc<dyn Logger>>,
}

/// Session manages the state of an LSP session. It receives textDocument
/// events and requests for LanguageService objects, processing them into
/// immutable snapshots.
///
/// Go: `type Session struct { ... }`.
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

    /// Counter for snapshot IDs (predictable in tests).
    pub snapshot_id: AtomicU64,
    /// The current immutable snapshot, shared via `Arc`.
    ///
    /// Go stores a `*Snapshot`; in Rust the snapshot is reference-counted with
    /// `Arc` so callers can hold onto it without cloning its (non-`Clone`)
    /// interior.
    snapshot: RwLock<Option<Arc<Snapshot>>>,

    pub pending_file_changes: Mutex<Vec<FileChange>>,
    pub pending_ata_changes: Mutex<HashMap<Path, super::snapshot::ATAStateChange>>,

    pub watches: Arc<WatchRegistry>,
    pub seen_projects: Mutex<HashSet<Path>>,
    pub global_diag_publish_pending: AtomicBool,

    /// Current workspace user preferences.
    workspace_user_preferences: Mutex<UserPreferences>,
    /// Set when `configure` is called and cleared when consumed by a flush.
    pending_user_config_changes: AtomicBool,

    // --- Debounce state -------------------------------------------------
    // Go uses cancellable contexts + `time.After` in background goroutines.
    // Here the debounce is tracked with a generation counter and a scheduled
    // `Instant`; a scheduled task is cancelled by bumping the generation and
    // clearing the timestamp. The real refresh fires lazily on the next
    // synchronous update or diagnostics request.
    scheduled_snapshot_update_generation: AtomicU64,
    scheduled_snapshot_update_at: Mutex<Option<Instant>>,

    diagnostics_refresh_generation: AtomicU64,
    diagnostics_refresh_at: Mutex<Option<Instant>>,

    idle_cache_clean_at: Mutex<Option<Instant>>,
    warm_auto_import_active: AtomicBool,
}

impl Session {
    /// Creates a new session.
    ///
    /// Go: `func NewSession(init *SessionInit) *Session`.
    pub fn new(init: SessionInit) -> Self {
        let current_directory = init.options.current_directory.clone();
        let use_case_sensitive = init.fs.use_case_sensitive_file_names();

        // Build the to_path closure used by the overlay FS. Each closure owns
        // its own copy of the captured state so they can live independently.
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

    /// Returns the backing virtual file system.
    ///
    /// Go: `func (s *Session) FS() vfs.FS { return s.fs.fs }`.
    pub fn fs(&self) -> Option<&Arc<dyn FS>> {
        self.fs.as_ref().map(|ofs| &ofs.fs)
    }

    /// Returns the session's current directory.
    ///
    /// Go: `func (s *Session) GetCurrentDirectory() string`.
    pub fn current_directory(&self) -> &str {
        &self.options.current_directory
    }

    /// Returns a shared handle to the current snapshot.
    ///
    /// Go: `func (s *Session) Snapshot() *Snapshot`. In Rust the snapshot is
    /// shared via `Arc`, so this cheaply clones the `Arc` under a read lock.
    pub fn snapshot(&self) -> Option<Arc<Snapshot>> {
        self.snapshot.read().unwrap().clone()
    }

    /// Gets a copy of the current user preferences.
    ///
    /// Go: `func (s *Session) Config() lsutil.UserPreferences`.
    pub fn config(&self) -> UserPreferences {
        self.workspace_user_preferences.lock().unwrap().clone()
    }

    /// Updates the workspace user preferences.
    ///
    /// Go: `func (s *Session) Configure(config lsutil.UserPreferences)`.
    pub fn configure(&self, config: UserPreferences) {
        let mut prefs = self.workspace_user_preferences.lock().unwrap();
        let old = prefs.clone();
        self.pending_user_config_changes
            .store(true, Ordering::SeqCst);
        *prefs = config;
        drop(prefs);

        // Tell the client to re-request certain capabilities depending on
        // preference changes (best-effort; errors are ignored).
        self.refresh_inlay_hints_if_needed(&old);
        self.refresh_code_lens_if_needed(&old);
        self.refresh_diagnostics_if_needed(&old);
        self.refresh_ata_if_needed(&old);
    }

    /// Initializes the session with the user config supplied at startup.
    ///
    /// Go: `func (s *Session) InitializeWithUserConfig(config lsutil.UserPreferences)`.
    pub fn initialize_with_user_config(&self, config: UserPreferences) {
        self.configure(config);
    }

    /// Records a didOpen event.
    ///
    /// Go: `func (s *Session) DidOpenFile(...)`.
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

    /// Records a didClose event.
    ///
    /// Go: `func (s *Session) DidCloseFile(...)`.
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

    /// Records a didChange event.
    ///
    /// Go: `func (s *Session) DidChangeFile(...)`.
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

    /// Records a didSave event.
    ///
    /// Go: `func (s *Session) DidSaveFile(...)`.
    pub fn did_save_file(&self, uri: &lsproto::DocumentUri) {
        self.schedule_idle_cache_clean();
        let mut pending = self.pending_file_changes.lock().unwrap();
        pending.push(FileChange {
            kind: FileChangeKind::Save,
            uri: uri.clone(),
            ..Default::default()
        });
    }

    /// Records a workspace/didChangeWatchedFiles event.
    ///
    /// Go: `func (s *Session) DidChangeWatchedFiles(...)`.
    pub fn did_change_watched_files(&self, changes: &[lsproto::FileEvent]) {
        let mut file_changes: Vec<FileChange> = Vec::with_capacity(changes.len());
        for change in changes {
            let kind = match change.change_type {
                lsproto::FILE_CHANGE_TYPE_CREATED => FileChangeKind::WatchCreate,
                lsproto::FILE_CHANGE_TYPE_CHANGED => FileChangeKind::WatchChange,
                lsproto::FILE_CHANGE_TYPE_DELETED => FileChangeKind::WatchDelete,
                _ => continue, // Ignore unknown change types.
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

        // Schedule a debounced diagnostics refresh.
        self.schedule_diagnostics_refresh();
        self.cancel_warm_auto_import_cache();
        self.schedule_idle_cache_clean();
    }

    /// Updates the compiler options used for inferred projects.
    ///
    /// Go: `func (s *Session) DidChangeCompilerOptionsForInferredProjects(...)`.
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

    /// Schedules a debounced snapshot update.
    ///
    /// Go: `func (s *Session) ScheduleSnapshotUpdate(reason UpdateReason)`.
    /// The debounce is implemented with a generation counter and a scheduled
    /// `Instant`; cancelling bumps the generation so a stale task is ignored.
    pub fn schedule_snapshot_update(&self, reason: UpdateReason) {
        let _ = reason;
        self.scheduled_snapshot_update_generation
            .fetch_add(1, Ordering::SeqCst);
        let delay = self.options.debounce_delay;
        *self.scheduled_snapshot_update_at.lock().unwrap() = Some(Instant::now() + delay);
    }

    /// Returns true if a scheduled snapshot update is pending and its debounce
    /// delay has elapsed.
    pub fn snapshot_update_due(&self) -> bool {
        match *self.scheduled_snapshot_update_at.lock().unwrap() {
            Some(when) => Instant::now() >= when,
            None => false,
        }
    }

    /// Cancels any pending scheduled snapshot update.
    ///
    /// Go: `func (s *Session) cancelScheduledSnapshotUpdate()`.
    pub fn cancel_scheduled_snapshot_update(&self) {
        self.scheduled_snapshot_update_generation
            .fetch_add(1, Ordering::SeqCst);
        *self.scheduled_snapshot_update_at.lock().unwrap() = None;
    }

    /// Schedules a debounced diagnostics refresh.
    ///
    /// Go: `func (s *Session) ScheduleDiagnosticsRefresh()`.
    pub fn schedule_diagnostics_refresh(&self) {
        self.diagnostics_refresh_generation
            .fetch_add(1, Ordering::SeqCst);
        let delay = self.options.debounce_delay;
        *self.diagnostics_refresh_at.lock().unwrap() = Some(Instant::now() + delay);

        // If there is a connected client, enqueue a self-contained background
        // task that waits out the debounce and then asks the client to refresh.
        // (The closure only captures `Arc<dyn Client>`, which is `'static`.)
        if let Some(client) = &self.client {
            let client = Arc::clone(client);
            self.background_queue.enqueue(move || {
                std::thread::sleep(delay);
                let _ = client.refresh_diagnostics();
            });
        }
    }

    /// Returns true if a scheduled diagnostics refresh is pending and its
    /// debounce delay has elapsed.
    pub fn diagnostics_refresh_due(&self) -> bool {
        match *self.diagnostics_refresh_at.lock().unwrap() {
            Some(when) => Instant::now() >= when,
            None => false,
        }
    }

    /// Cancels any pending diagnostics refresh.
    ///
    /// Go: `func (s *Session) cancelDiagnosticsRefresh()`.
    pub fn cancel_diagnostics_refresh(&self) {
        self.diagnostics_refresh_generation
            .fetch_add(1, Ordering::SeqCst);
        *self.diagnostics_refresh_at.lock().unwrap() = None;
    }

    /// Cancels any running auto-import cache warming.
    ///
    /// Go: `func (s *Session) cancelWarmAutoImportCache()`.
    pub fn cancel_warm_auto_import_cache(&self) {
        self.warm_auto_import_active.store(false, Ordering::SeqCst);
    }

    /// Schedules an idle disk-cache clean.
    ///
    /// Go: `func (s *Session) scheduleIdleCacheClean()`.
    pub fn schedule_idle_cache_clean(&self) {
        *self.idle_cache_clean_at.lock().unwrap() = Some(Instant::now() + IDLE_CACHE_CLEAN_DELAY);
    }

    /// Cancels any pending idle cache clean.
    ///
    /// Go: `func (s *Session) cancelIdleCacheClean()`.
    pub fn cancel_idle_cache_clean(&self) {
        *self.idle_cache_clean_at.lock().unwrap() = None;
    }

    /// Closes the session, cancelling all pending work.
    ///
    /// Go: `func (s *Session) Close()`.
    pub fn close(&self) {
        self.cancel_scheduled_snapshot_update();
        self.cancel_diagnostics_refresh();
        self.cancel_warm_auto_import_cache();
        self.cancel_idle_cache_clean();
        self.stop_performance_telemetry();
        self.background_queue.close();
    }

    /// Waits for all background tasks to complete (intended for tests).
    ///
    /// Go: `func (s *Session) WaitForBackgroundTasks()`.
    pub fn wait_for_background_tasks(&self) {
        self.cancel_idle_cache_clean();
        self.background_queue.wait();
    }

    /// Drains pending file changes, processing them through the overlay FS.
    ///
    /// Returns the change summary and the resulting overlay map.
    ///
    /// Go: `func (s *Session) flushChanges(...)`. The ATA/config portions of
    /// the Go return are handled separately here.
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

    /// Flushes pending changes and, if there were any, updates the snapshot.
    /// Returns the (possibly newly created) current snapshot.
    ///
    /// Go: `func (s *Session) getSnapshot(...)`.
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

        // No pending changes: reuse the current snapshot.
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

    /// Builds a new snapshot from `change`, adopting it as the current
    /// snapshot. Returns the new snapshot.
    ///
    /// Go: `func (s *Session) UpdateSnapshot(...)` / `updateSnapshot(...)`.
    /// The full Go clone recomputes the project tree; here we create a new
    /// snapshot with an incremented ID that carries over the previous
    /// snapshot's immutable references (a simplified clone).
    pub fn update_snapshot(
        &self,
        _overlays: HashMap<Path, Arc<Overlay>>,
        change: SnapshotChange,
    ) -> Arc<Snapshot> {
        let new_id = self.snapshot_id.fetch_add(1, Ordering::SeqCst) + 1;

        // Read the previous snapshot (drop the read guard before writing).
        let prev = self.snapshot.read().unwrap().clone();
        let parent_id = prev.as_ref().map(|s| s.id).unwrap_or(0);

        let mut new_snapshot = Snapshot::new(new_id);
        new_snapshot.parent_id = parent_id;

        // Carry over the immutable file source and compiler options from the
        // previous snapshot, applying the change's overrides where present.
        // `ProjectCollection`/`ConfigFileRegistry` are boxed and not yet
        // `Clone`-able, so they are not carried over in this simplified clone.
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

        // Swap under the write lock.
        *self.snapshot.write().unwrap() = Some(Arc::clone(&new_arc));

        new_arc
    }

    /// Returns a `LanguageService` for the given URI.
    ///
    /// Go: `func (s *Session) GetLanguageService(...)`. Because the
    /// project/program lookup is not yet wired through (`Snapshot::
    /// get_default_project` is unimplemented), this returns `None` until that
    /// integration lands.
    pub fn get_language_service(
        &self,
        _uri: &lsproto::DocumentUri,
    ) -> Option<crate::ls::language_service::LanguageService> {
        // Flush any pending changes so the snapshot is current.
        let _snapshot = self.get_snapshot(super::snapshot::ResourceRequest {
            documents: vec![_uri.clone()],
            ..Default::default()
        });
        // The default-project lookup is not yet implemented; no service yet.
        None
    }

    // ====================================================================
    // Telemetry
    // ====================================================================

    /// Begins periodic collection of performance telemetry.
    ///
    /// Go: `func (s *Session) StartPerformanceTelemetry()`. The Go version
    /// reads runtime metrics (Go runtime specific) on a ticker and sends them
    /// via the client. Those runtime metrics have no direct Rust equivalent,
    /// so this simplified version just verifies telemetry is enabled and
    /// records that telemetry collection started.
    pub fn start_performance_telemetry(&self) {
        if !self.options.telemetry_enabled {
            return;
        }
        // A real implementation would enqueue a periodic task that reads
        // process metrics and calls `client.send_telemetry`. Runtime-specific
        // metrics collection is omitted in this simplified port.
    }

    /// Stops the periodic performance telemetry ticker.
    ///
    /// Go: `func (s *Session) stopPerformanceTelemetry()`.
    pub fn stop_performance_telemetry(&self) {
        // No background ticker is running in the simplified port; this is a
        // no-op kept for API parity.
    }

    /// Sends project-info telemetry for any projects added between the two
    /// snapshots.
    ///
    /// Go: `func (s *Session) sendProjectInfoTelemetryForNewProjects(...)`.
    pub fn send_project_info_telemetry(&self, _old_snapshot: &Snapshot, _new_snapshot: &Snapshot) {
        if !self.options.telemetry_enabled {
            return;
        }
        // The full implementation diffs the project collections and emits a
        // telemetry event per newly-seen project. Project collection diffing is
        // not yet wired, so this is a no-op placeholder.
    }

    /// Returns whether a project has already had telemetry sent for it.
    ///
    /// Go uses a `collections.SyncSet`; here a `Mutex<HashSet<Path>>`.
    pub fn mark_project_seen(&self, project_path: &Path) {
        self.seen_projects
            .lock()
            .unwrap()
            .insert(project_path.clone());
    }

    /// Returns true if telemetry has already been sent for `project_path`.
    pub fn has_seen_project(&self, project_path: &Path) -> bool {
        self.seen_projects.lock().unwrap().contains(project_path)
    }

    // ====================================================================
    // Internal preference-change refresh helpers
    // ====================================================================

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
            // ATA was re-enabled; schedule a diagnostics refresh so the next
            // snapshot update re-triggers ATA.
            self.schedule_diagnostics_refresh();
        }
    }
}
