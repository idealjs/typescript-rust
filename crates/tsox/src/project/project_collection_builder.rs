#![allow(dead_code)]

use std::collections::HashMap;

use crate::lsp::lsproto;
use crate::tspath::Path;

use super::compiler_host::SessionOptions;
use super::config_file_registry::ConfigFileRegistry;
use super::file_change::FileChangeSummary;
use super::project_collection::{APIState, ProjectCollection};
use super::snapshot::{APISnapshotRequest, ATAStateChange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectLoadKind {

    Find,

    Create,
}

pub struct ProjectCollectionBuilder {
    pub session_options: SessionOptions,
    pub new_snapshot_id: u64,
    pub program_structure_changed: bool,
    pub default_projects_invalidated: bool,
    pub open_files_changed: bool,
    pub file_default_projects: HashMap<Path, Path>,
    pub api_state: APIState,
}

impl ProjectCollectionBuilder {

    pub fn new(
        new_snapshot_id: u64,
        compiler_options_for_inferred_projects: Option<
            &crate::core::compiler_options::CompilerOptions,
        >,
        session_options: SessionOptions,
    ) -> Self {
        let _ = compiler_options_for_inferred_projects;
        ProjectCollectionBuilder {
            session_options,
            new_snapshot_id,
            program_structure_changed: false,
            default_projects_invalidated: false,
            open_files_changed: false,
            file_default_projects: HashMap::new(),
            api_state: APIState::default(),
        }
    }

    pub fn finalize(&self) -> (ProjectCollection, ConfigFileRegistry) {
        todo!("ProjectCollectionBuilder::finalize requires full integration")
    }

    pub fn handle_api_request(&mut self, _api_request: &APISnapshotRequest) -> Result<(), String> {
        todo!("ProjectCollectionBuilder::handle_api_request requires full integration")
    }

    pub fn did_change_files(&mut self, _summary: &FileChangeSummary) {
        todo!("ProjectCollectionBuilder::did_change_files requires full integration")
    }

    pub fn did_update_ata_state(&mut self, _ata_changes: &HashMap<Path, ATAStateChange>) {

    }

    pub fn did_change_custom_config_file_name(&mut self) {

    }

    pub fn did_request_file(
        &mut self,
        _uri: &lsproto::DocumentUri,
        _configured_projects_only: bool,
    ) {
        todo!("ProjectCollectionBuilder::did_request_file requires full integration")
    }

    pub fn did_request_project(&mut self, _project_id: &Path) {
        todo!("ProjectCollectionBuilder::did_request_project requires full integration")
    }
}
