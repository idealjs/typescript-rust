//! API session helpers (1:1 port of Go's `internal/project/api.go`).

#![allow(dead_code)]

use crate::lsp::lsproto;

use super::file_change::FileChangeSummary;
use super::session::Session;
use super::snapshot::{APISnapshotRequest, Snapshot};

impl Session {
    /// Creates a new snapshot incorporating the given file changes and the
    /// supplied API open/close request.
    ///
    /// Go: `func (s *Session) APIUpdate(...) (*Snapshot, error)`.
    pub fn api_update(
        &self,
        _api_file_changes: &FileChangeSummary,
        _api_request: &APISnapshotRequest,
    ) -> Result<Box<Snapshot>, String> {
        todo!("Session::api_update requires full session/snapshot integration")
    }
}
