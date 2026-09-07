#![allow(dead_code)]

use std::collections::HashSet;

use crate::lsp::lsproto;

const EXCESSIVE_CHANGE_THRESHOLD: usize = 1000;

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
    pub fn is_watch_kind(&self) -> bool {
        matches!(
            self,
            FileChangeKind::WatchCreate | FileChangeKind::WatchChange | FileChangeKind::WatchDelete
        )
    }
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub kind: FileChangeKind,
    pub uri: lsproto::DocumentUri,

    pub version: i32,

    pub content: String,

    pub language_kind: lsproto::LanguageKind,

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

#[derive(Debug, Clone, Default)]
pub struct FileChangeSummary {
    pub opened: lsproto::DocumentUri,

    pub reopened: lsproto::DocumentUri,
    pub closed: HashSet<lsproto::DocumentUri>,
    pub changed: HashSet<lsproto::DocumentUri>,

    pub created: HashSet<lsproto::DocumentUri>,

    pub deleted: HashSet<lsproto::DocumentUri>,

    pub includes_watch_change_outside_node_modules: bool,

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
