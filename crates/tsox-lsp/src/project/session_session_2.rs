#![allow(unused_imports)]

use crate::project::session::*;

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

    pub fn get_language_service(
        &self,
        _uri: &lsproto::DocumentUri,
    ) -> Option<crate::ls::language_service::LanguageService> {
        let _snapshot = self.get_snapshot(
            crate::project::snapshot::ResourceRequest {
                documents: vec![_uri.clone()],
                ..Default::default()
            },
            false,
        );

        None
    }

    pub fn start_performance_telemetry(&self) {
        if !self.options.telemetry_enabled {
            return;
        }
    }

    pub fn stop_performance_telemetry(&self) {}

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
