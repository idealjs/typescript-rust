#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};

use crate::lsp::lsproto;

const MIN_WATCH_LOCATION_DEPTH: usize = 2;

pub type WatcherID = String;

static WATCHER_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default)]
pub struct PatternsAndIgnored {
    pub directories_outside_workspace: Vec<String>,
    pub patterns_inside_workspace: Vec<String>,
    pub ignored: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileSystemWatcherKey {
    pattern: String,
    kind: lsproto::WatchKind,
}

#[derive(Debug)]
struct FileSystemWatcherValue {
    count: i32,
    id: WatcherID,
}

pub struct WatchRegistry {
    entries: Mutex<HashMap<FileSystemWatcherKey, FileSystemWatcherValue>>,
    pending: Mutex<HashSet<WatcherID>>,
}

impl WatchRegistry {
    pub fn new() -> Self {
        WatchRegistry {
            entries: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashSet::new()),
        }
    }

    pub fn acquire(&self, watcher: &lsproto::FileSystemWatcher, id: WatcherID) -> bool {
        let key = to_file_system_watcher_key(watcher);
        let mut entries = self.entries.lock().unwrap();
        let value = entries
            .entry(key)
            .or_insert_with(|| FileSystemWatcherValue {
                count: 0,
                id: id.clone(),
            });
        value.count += 1;
        value.count == 1
    }

    pub fn release(&self, watcher: &lsproto::FileSystemWatcher) -> (WatcherID, bool) {
        let key = to_file_system_watcher_key(watcher);
        let mut entries = self.entries.lock().unwrap();
        match entries.get_mut(&key) {
            None => (String::new(), false),
            Some(value) => {
                if value.count <= 1 {
                    let id = value.id.clone();
                    entries.remove(&key);
                    (id, true)
                } else {
                    value.count -= 1;
                    (String::new(), false)
                }
            }
        }
    }

    pub fn mark_pending(&self, id: &WatcherID) {
        self.pending.lock().unwrap().insert(id.clone());
    }

    pub fn clear_pending(&self, id: &WatcherID) {
        self.pending.lock().unwrap().remove(id);
    }

    pub fn is_pending(&self, id: &WatcherID) -> bool {
        self.pending.lock().unwrap().contains(id)
    }
}

impl Default for WatchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn to_file_system_watcher_key(w: &lsproto::FileSystemWatcher) -> FileSystemWatcherKey {
    let kind = w.kind.unwrap_or(
        lsproto::WATCH_KIND_CREATE | lsproto::WATCH_KIND_CHANGE | lsproto::WATCH_KIND_DELETE,
    );
    let pattern = file_system_watcher_glob_string(w);
    FileSystemWatcherKey { pattern, kind }
}

pub fn file_system_watcher_glob_string(w: &lsproto::FileSystemWatcher) -> String {
    if let Some(pattern) = &w.glob_pattern.pattern {
        return pattern.clone();
    }
    if let Some(rp) = &w.glob_pattern.relative_pattern {
        let base = match &rp.base_uri.uri {
            Some(uri) => uri.clone(),
            None => panic!("workspace folder-based relative patterns not implemented"),
        };
        return format!("{}/{}", base, rp.pattern);
    }
    String::new()
}

pub struct WatchedFiles<T: Clone + Send + Sync> {
    name: String,
    watch_kind: lsproto::WatchKind,
    has_relative_pattern_capability: bool,
    compute_glob_patterns: Box<dyn Fn(&T) -> PatternsAndIgnored + Send + Sync>,

    inner: RwLock<WatchedFilesInner<T>>,
    id: Mutex<u64>,
}

struct WatchedFilesInner<T> {
    input: T,
    workspace_watchers: Vec<lsproto::FileSystemWatcher>,
    outside_workspace_watchers: Vec<lsproto::FileSystemWatcher>,
    ignored: HashSet<String>,
    current_id: u64,
}

#[derive(Debug, Clone)]
pub struct Watchers {
    pub watcher_id: WatcherID,
    pub workspace_watchers: Vec<lsproto::FileSystemWatcher>,
    pub outside_workspace_watchers: Vec<lsproto::FileSystemWatcher>,
    pub ignored_paths: HashSet<String>,
}

impl<T: Clone + Send + Sync + Default> WatchedFiles<T> {
    pub fn new<F>(
        name: &str,
        watch_kind: lsproto::WatchKind,
        has_relative_pattern_capability: bool,
        compute_glob_patterns: F,
    ) -> Self
    where
        F: Fn(&T) -> PatternsAndIgnored + Send + Sync + 'static,
    {
        let id = WATCHER_ID.fetch_add(1, Ordering::SeqCst);
        WatchedFiles {
            name: name.to_string(),
            watch_kind,
            has_relative_pattern_capability,
            compute_glob_patterns: Box::new(compute_glob_patterns),
            inner: RwLock::new(WatchedFilesInner {
                input: T::default(),
                workspace_watchers: Vec::new(),
                outside_workspace_watchers: Vec::new(),
                ignored: HashSet::new(),
                current_id: id,
            }),
            id: Mutex::new(id),
        }
    }

    pub fn watchers(&self) -> Watchers {
        let inner = self.inner.read().unwrap();
        Watchers {
            watcher_id: format!("{} watcher {}", self.name, inner.current_id),
            workspace_watchers: inner.workspace_watchers.clone(),
            outside_workspace_watchers: inner.outside_workspace_watchers.clone(),
            ignored_paths: inner.ignored.clone(),
        }
    }

    pub fn id(&self) -> WatcherID {
        self.watchers().watcher_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn watch_kind(&self) -> lsproto::WatchKind {
        self.watch_kind
    }

    pub fn clone_with_input(&self, input: T) -> WatchedFiles<T> {
        let inner = self.inner.read().unwrap();
        WatchedFiles {
            name: self.name.clone(),
            watch_kind: self.watch_kind,
            has_relative_pattern_capability: self.has_relative_pattern_capability,
            compute_glob_patterns: Box::new(|_| PatternsAndIgnored::default()),
            inner: RwLock::new(WatchedFilesInner {
                input,
                workspace_watchers: inner.workspace_watchers.clone(),
                outside_workspace_watchers: inner.outside_workspace_watchers.clone(),
                ignored: inner.ignored.clone(),
                current_id: inner.current_id,
            }),
            id: Mutex::new(*self.id.lock().unwrap()),
        }
    }
}

pub fn get_recursive_glob_pattern(directory: &str) -> String {
    let dir = crate::tspath::remove_trailing_directory_separator(directory);
    format!("{}/**/*", dir)
}

pub fn recursive_directory_glob_pattern(directory: &str, use_relative_pattern: bool) -> String {
    if use_relative_pattern {
        format!("file://{directory}/**/*")
    } else {
        get_recursive_glob_pattern(directory)
    }
}
