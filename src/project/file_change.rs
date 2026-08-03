//! File change tracking types (1:1 port of Go's `internal/project/filechange.go`).

#![allow(dead_code)]

use std::collections::HashSet;

use crate::lsp::lsproto;

const EXCESSIVE_CHANGE_THRESHOLD: usize = 1000;

/// FileChangeKind enumerates the kinds of file changes the LSP server
/// processes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileChangeKind {
    Open,
    Close,
    Change,
    Save,
    WatchCreate,
    WatchChange,
    WatchDelete,
}

impl FileChangeKind {
    /// Returns true if this is a watch (file system watcher) change kind.
    pub fn is_watch_kind(&self) -> bool {
        matches!(
            self,
            FileChangeKind::WatchCreate | FileChangeKind::WatchChange | FileChangeKind::WatchDelete
        )
    }
}

/// A single file change event received from the client.
#[derive(Debug, Clone)]
pub struct FileChange {
    pub kind: FileChangeKind,
    pub uri: lsproto::DocumentUri,
    /// Only set for Open/Change.
    pub version: i32,
    /// Only set for Open.
    pub content: String,
    /// Only set for Open.
    pub language_kind: lsproto::LanguageKind,
    /// Only set for Change.
    pub changes: Vec<lsproto::TextDocumentContentChangePartialOrWholeDocument>,
}

impl Default for FileChange {
    fn default() -> Self {
        FileChange {
            kind: FileChangeKind::Change,
            uri: lsproto::DocumentUri::default(),
            version: 0,
            content: String::new(),
            language_kind: String::new(),
            changes: Vec::new(),
        }
    }
}

/// A summary of file changes processed during a snapshot update.
#[derive(Debug, Clone, Default)]
pub struct FileChangeSummary {
    /// Only one file can be opened at a time per request.
    pub opened: lsproto::DocumentUri,
    /// Reopened is set if a close and open occurred for the same file in a
    /// single batch of changes.
    pub reopened: lsproto::DocumentUri,
    pub closed: HashSet<lsproto::DocumentUri>,
    pub changed: HashSet<lsproto::DocumentUri>,
    /// Only set when file watching is enabled.
    pub created: HashSet<lsproto::DocumentUri>,
    /// Only set when file watching is enabled.
    pub deleted: HashSet<lsproto::DocumentUri>,
    /// True if the summary includes a create/change/delete watch event of a
    /// file outside a node_modules directory.
    pub includes_watch_change_outside_node_modules: bool,
    /// Indicates that all cached file state should be discarded.
    pub invalidate_all: bool,
}

impl FileChangeSummary {
    pub fn is_empty(&self) -> bool {
        !self.invalidate_all
            && self.opened.0.is_empty()
            && self.reopened.0.is_empty()
            && self.closed.is_empty()
            && self.changed.is_empty()
            && self.created.is_empty()
            && self.deleted.is_empty()
    }

    pub fn has_excessive_watch_events(&self) -> bool {
        self.invalidate_all
            || (self.created.len() + self.deleted.len() + self.changed.len())
                > EXCESSIVE_CHANGE_THRESHOLD
    }

    pub fn has_excessive_non_create_watch_events(&self) -> bool {
        self.invalidate_all
            || (self.deleted.len() + self.changed.len()) > EXCESSIVE_CHANGE_THRESHOLD
    }
}

/// Merges `src` into `dst`, combining their change sets.
pub fn merge_file_change_summary(dst: &mut FileChangeSummary, src: &FileChangeSummary) {
    if src.is_empty() {
        return;
    }
    if src.invalidate_all {
        dst.invalidate_all = true;
    }
    for uri in &src.changed {
        dst.changed.insert(uri.clone());
    }
    for uri in &src.created {
        dst.created.insert(uri.clone());
    }
    for uri in &src.deleted {
        dst.deleted.insert(uri.clone());
    }
    if src.includes_watch_change_outside_node_modules {
        dst.includes_watch_change_outside_node_modules = true;
    }
}
