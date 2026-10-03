#![allow(dead_code)]

use std::collections::HashMap;

use crate::lsp::lsproto;
use tsox_core::tspath::Path;

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
            &tsox_core::core::compiler_options::CompilerOptions,
        >,
        session_options: SessionOptions,
    ) -> Self { ::tsox_core::fntrace::enter("new"); 
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

    pub fn finalize(&self) -> (ProjectCollection, ConfigFileRegistry) { ::tsox_core::fntrace::enter("finalize"); 
        todo!("ProjectCollectionBuilder::finalize requires full integration")
    }

    pub fn handle_api_request(&mut self, _api_request: &APISnapshotRequest) -> Result<(), String> { ::tsox_core::fntrace::enter("handle_api_request"); 
        todo!("ProjectCollectionBuilder::handle_api_request requires full integration")
    }

    pub fn did_change_files(&mut self, _summary: &FileChangeSummary) { ::tsox_core::fntrace::enter("did_change_files"); 
        todo!("ProjectCollectionBuilder::did_change_files requires full integration")
    }

    pub fn did_update_ata_state(&mut self, _ata_changes: &HashMap<Path, ATAStateChange>) { ::tsox_core::fntrace::enter("did_update_ata_state"); }

    pub fn did_change_custom_config_file_name(&mut self) { ::tsox_core::fntrace::enter("did_change_custom_config_file_name"); }

    pub fn did_request_file(
        &mut self,
        _uri: &lsproto::DocumentUri,
        _configured_projects_only: bool,
    ) { ::tsox_core::fntrace::enter("did_request_file"); 
        todo!("ProjectCollectionBuilder::did_request_file requires full integration")
    }

    pub fn did_request_project(&mut self, _project_id: &Path) { ::tsox_core::fntrace::enter("did_request_project"); 
        todo!("ProjectCollectionBuilder::did_request_project requires full integration")
    }
}
