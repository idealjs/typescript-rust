#![allow(unused_imports)]

use super::*;

impl Session {
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

    pub fn get_snapshot(
        &self,
        request: crate::project::snapshot::ResourceRequest,
    ) -> Arc<Snapshot> {
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
        let _snapshot = self.get_snapshot(crate::project::snapshot::ResourceRequest {
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

    pub fn stop_performance_telemetry(&self) {}

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

    pub(crate) fn refresh_inlay_hints_if_needed(&self, old_prefs: &UserPreferences) {
        if old_prefs.inlay_hints != self.config().inlay_hints {
            if let Some(client) = &self.client {
                let _ = client.refresh_inlay_hints();
            }
        }
    }

    pub(crate) fn refresh_code_lens_if_needed(&self, old_prefs: &UserPreferences) {
        if old_prefs.code_lens != self.config().code_lens {
            if let Some(client) = &self.client {
                let _ = client.refresh_code_lens();
            }
        }
    }

    pub(crate) fn refresh_diagnostics_if_needed(&self, old_prefs: &UserPreferences) {
        let new_prefs = self.config();
        if old_prefs.custom_config_file_name != new_prefs.custom_config_file_name
            || old_prefs.report_style_checks_as_warnings
                != new_prefs.report_style_checks_as_warnings
            || old_prefs.enable_validation != new_prefs.enable_validation
        {
            self.schedule_diagnostics_refresh();
        }
    }

    pub(crate) fn refresh_ata_if_needed(&self, old_prefs: &UserPreferences) {
        if old_prefs.is_ata_disabled() && !self.config().is_ata_disabled() {
            self.schedule_diagnostics_refresh();
        }
    }
}
