#![allow(dead_code)]

use super::file_change::FileChangeSummary;
use super::session::Session;
use super::snapshot::{APISnapshotRequest, Snapshot};

impl Session {

    pub fn api_update(
        &self,
        _api_file_changes: &FileChangeSummary,
        _api_request: &APISnapshotRequest,
    ) -> Result<Box<Snapshot>, String> {
        todo!("Session::api_update requires full session/snapshot integration")
    }
}
