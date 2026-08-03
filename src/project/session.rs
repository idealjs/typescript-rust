//! Session — central LSP state (1:1 port of Go's `internal/project/session.go`).

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use crate::core::compiler_options::CompilerOptions;
use crate::lsp::lsproto;
use crate::tspath::Path;
use crate::vfs::FS;

use super::background;
use super::client::Client;
use super::compiler_host::SessionOptions;
use super::extended_config_cache::ExtendedConfigCache;
use super::file_change::{FileChange, FileChangeSummary};
use super::overlay_fs::OverlayFS;
use super::parse_cache::ParseCache;
use super::program_counter::ProgramCounter;
use super::snapshot::{Snapshot, SnapshotChange, UpdateReason};
use super::watch::WatchRegistry;

/// Watch request timeout for client calls.
pub const WATCH_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

/// Idle cache clean delay.
pub const IDLE_CACHE_CLEAN_DELAY: Duration = Duration::from_secs(30);

/// Session initialization parameters.
///
/// Go: `type SessionInit struct { ... }`.
pub struct SessionInit {
    pub options: SessionOptions,
    pub fs: Arc<dyn FS>,
    pub client: Option<Arc<dyn Client>>,
    pub parse_cache: Option<Arc<ParseCache>>,
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

    pub fs: Option<Arc<OverlayFS>>,
    pub parse_cache: Option<Arc<ParseCache>>,
    pub extended_config_cache: Option<Arc<ExtendedConfigCache>>,
    pub program_counter: Option<Arc<ProgramCounter>>,
    pub background_queue: Arc<background::Queue>,

    pub snapshot_id: AtomicU64,
    snapshot: RwLock<Option<Box<Snapshot>>>,

    pub pending_file_changes: Mutex<Vec<FileChange>>,
    pub pending_ata_changes: Mutex<HashMap<Path, super::snapshot::ATAStateChange>>,

    pub watches: Arc<WatchRegistry>,
    pub seen_projects: Mutex<HashSet<Path>>,
    pub global_diag_publish_pending: AtomicBool,
}

impl Session {
    /// Creates a new session.
    ///
    /// Go: `func NewSession(init *SessionInit) *Session`.
    pub fn new(init: SessionInit) -> Self {
        let current_directory = init.options.current_directory.clone();
        let use_case_sensitive = init.fs.use_case_sensitive_file_names();
        let to_path: Box<dyn Fn(&str) -> Path + Send + Sync> = Box::new(move |file_name: &str| {
            crate::tspath::to_path(file_name, &current_directory, use_case_sensitive)
        });

        let parse_cache = init.parse_cache.or_else(|| {
            Some(Arc::new(ParseCache::new(
                super::refcount_cache::RefCountCacheOptions::default(),
            )))
        });

        let snapshot = Box::new(Snapshot::new(0));
        let snapshot_lock = RwLock::new(Some(snapshot));

        Session {
            options: init.options,
            start_time: Instant::now(),
            to_path: Box::new(|_: &str| Path::default()),
            client: init.client,
            fs: None,
            parse_cache,
            extended_config_cache: Some(Arc::new(ExtendedConfigCache::new())),
            program_counter: Some(Arc::new(ProgramCounter::new())),
            background_queue: Arc::new(background::Queue::new()),
            snapshot_id: AtomicU64::new(0),
            snapshot: snapshot_lock,
            pending_file_changes: Mutex::new(Vec::new()),
            pending_ata_changes: Mutex::new(HashMap::new()),
            watches: Arc::new(WatchRegistry::new()),
            seen_projects: Mutex::new(HashSet::new()),
            global_diag_publish_pending: AtomicBool::new(false),
        }
    }

    pub fn fs(&self) -> Option<&Arc<dyn FS>> {
        todo!("Session::fs requires overlayFS backing store")
    }

    pub fn current_directory(&self) -> &str {
        &self.options.current_directory
    }

    pub fn snapshot(&self) -> Option<Box<Snapshot>> {
        // Cannot truly clone a Snapshot; this is a placeholder.
        None
    }

    /// Records a didOpen event.
    ///
    /// Go: `func (s *Session) DidOpenFile(...)`.
    pub fn did_open_file(
        &self,
        _uri: &lsproto::DocumentUri,
        _version: i32,
        _content: &str,
        _language_kind: &lsproto::LanguageKind,
    ) {
        todo!("Session::did_open_file requires snapshot update integration")
    }

    /// Records a didClose event.
    pub fn did_close_file(&self, _uri: &lsproto::DocumentUri) {
        self.pending_file_changes.lock().unwrap().push(FileChange {
            kind: super::file_change::FileChangeKind::Close,
            uri: _uri.clone(),
            ..Default::default()
        });
    }

    /// Records a didChange event.
    pub fn did_change_file(
        &self,
        _uri: &lsproto::DocumentUri,
        _version: i32,
        _changes: &[lsproto::TextDocumentContentChangePartialOrWholeDocument],
    ) {
        todo!("Session::did_change_file requires overlay processing")
    }

    /// Records a didSave event.
    pub fn did_save_file(&self, uri: &lsproto::DocumentUri) {
        self.pending_file_changes.lock().unwrap().push(FileChange {
            kind: super::file_change::FileChangeKind::Save,
            uri: uri.clone(),
            ..Default::default()
        });
    }

    /// Schedules a debounced snapshot update.
    pub fn schedule_snapshot_update(&self, _reason: UpdateReason) {
        // Stub: full implementation uses background queue with debounce.
    }

    /// Schedules a debounced diagnostics refresh.
    pub fn schedule_diagnostics_refresh(&self) {
        // Stub.
    }

    /// Closes the session, cancelling pending work.
    pub fn close(&self) {
        self.background_queue.close();
    }

    /// Flushes pending file changes.
    ///
    /// Go: `func (s *Session) flushChanges(...)`.
    pub fn flush_changes(
        &self,
    ) -> (
        FileChangeSummary,
        HashMap<Path, Arc<super::overlay_fs::Overlay>>,
    ) {
        let changes = self
            .pending_file_changes
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<_>>();
        if changes.is_empty() {
            return (FileChangeSummary::default(), HashMap::new());
        }
        // Stub: full implementation calls overlayFS.process_changes.
        (FileChangeSummary::default(), HashMap::new())
    }
}
