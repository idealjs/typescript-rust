#![allow(unused_imports)]

use super::*;

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
                crate::project::refcount_cache::RefCountCacheOptions::default(),
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
                resource_request: crate::project::snapshot::ResourceRequest {
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
}
