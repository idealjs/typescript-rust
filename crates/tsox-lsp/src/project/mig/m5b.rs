#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::project::ata_discover_typings::TypingsInfo;
use crate::project::snapshot::{ResourceRequest, SnapshotChange};
use crate::project::mig::m5d::new_overlay;
use crate::project::overlay_fs::FileHandle;

use crate::project::file_change::FileChangeSummary;
use crate::project::session::Session;
use crate::project::snapshot::{APISnapshotRequest, Snapshot};
use tsox_core::collections::set::Set;

impl TypingsInfo {
    pub fn equals(&self, other: &TypingsInfo) -> bool {
        let type_acquisition_equal = match (&self.type_acquisition, &other.type_acquisition) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.enable == b.enable
                    && a.include == b.include
                    && a.exclude == b.exclude
                    && a.disable_filename_based_type_acquisition
                        == b.disable_filename_based_type_acquisition
            }
            _ => false,
        };
        let unresolved_imports_equal =
            match (&self.unresolved_imports, &other.unresolved_imports) {
                (None, None) => true,
                (Some(a), Some(b)) => a.equals(b),
                _ => false,
            };
        type_acquisition_equal
            && self.compiler_options.get_allow_js() == other.compiler_options.get_allow_js()
            && unresolved_imports_equal
    }
}

impl Session {
    pub fn api_create_program(
        &self,
        root_file_names: &[String],
        options: &tsox_core::core::compiler_options::CompilerOptions,
        project_references: &[tsox_core::core::project_reference::ProjectReference],
        config_file_parsing_diagnostics: &[tsox_frontend::ast::diagnostic::Diagnostic],
        old_snapshot: Option<&Snapshot>,
        old_project: Option<&crate::project::project::Project>,
        file_changes: FileChangeSummary,
    ) -> Arc<Snapshot> {
        if let Some(old_snapshot) = old_snapshot {
            return old_snapshot.clone_for_program(
                root_file_names.to_vec(),
                options.clone(),
                project_references.to_vec(),
                config_file_parsing_diagnostics.to_vec(),
                old_project,
                file_changes,
                self,
            );
        }

        let snapshot = self
            .api_update(&file_changes, &APISnapshotRequest::default())
            .expect("api update failed");
        snapshot.clone_for_program(
            root_file_names.to_vec(),
            options.clone(),
            project_references.to_vec(),
            config_file_parsing_diagnostics.to_vec(),
            None,
            file_changes,
            self,
        )
    }

    pub fn api_update_temporary(
        &self,
        base_snapshot: &Snapshot,
        uri: crate::lsp::lsproto::DocumentUri,
        new_text: &str,
    ) -> Result<Arc<Snapshot>, String> {
        let path = uri.path(base_snapshot.use_case_sensitive_file_names());

        let mut overlays = base_snapshot
            .fs
            .as_ref()
            .map(|fs| fs.overlays.clone())
            .unwrap_or_default();
        let mut version: i32 = 0;
        let mut file_changes = FileChangeSummary::default();
        let existing = overlays.get(&path).cloned();
        let script_kind = match existing {
            Some(existing) => {
                version = existing.version() + 1;
                file_changes.changed.insert(uri.clone());
                existing.kind()
            }
            None => {
                let script_kind =
                    crate::project::overlay_fs::script_kind_from_file_name(&uri.file_name());
                if script_kind == 0 {
                    return Err(format!("unsupported file extension: {}", uri.file_name()));
                }
                file_changes.opened = uri.clone();
                script_kind
            }
        };
        overlays.insert(
            path,
            Arc::new(new_overlay(
                uri.file_name(),
                new_text.to_string(),
                version,
                script_kind,
            )),
        );

        Ok(base_snapshot.clone_snapshot(
            SnapshotChange {
                file_changes,
                resource_request: ResourceRequest {
                    documents: vec![uri],
                    ..Default::default()
                },
                ..Default::default()
            },
            Some(&overlays),
            self,
        ))
    }
}
