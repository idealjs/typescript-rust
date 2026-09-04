#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{self};

use crate::lsp::lsproto;
use crate::tspath;
use crate::vfs::FS;

const THROTTLE_WINDOW: Duration = Duration::from_millis(75);

struct Watch {
    requested_directory: String,
    kind: lsproto::WatchKind,
    recursive: bool,
    closed: bool,
}

pub struct Watcher {
    fs: Arc<dyn FS>,
    on_changes: Box<dyn Fn(&[lsproto::FileEvent]) + Send + Sync>,
    inner: Mutex<WatcherInner>,
}

struct WatcherInner {
    watches: HashMap<String, Vec<Watch>>,
    closed: bool,
    pending: HashMap<String, lsproto::FileEvent>,
}

impl Watcher {

    pub fn new(
        fs: Arc<dyn FS>,
        on_changes: impl Fn(&[lsproto::FileEvent]) + Send + Sync + 'static,
    ) -> Arc<Self> {
        Arc::new(Watcher {
            fs,
            on_changes: Box::new(on_changes),
            inner: Mutex::new(WatcherInner {
                watches: HashMap::new(),
                closed: false,
                pending: HashMap::new(),
            }),
        })
    }

    pub fn watch_files(
        &self,
        id: &str,
        file_system_watchers: &[lsproto::FileSystemWatcher],
    ) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            return Err("lspwatcher: closed".to_string());
        }
        if inner.watches.contains_key(id) {
            return Err(format!("lspwatcher: watcher {:?} already exists", id));
        }

        let mut watches = Vec::new();
        for fsw in file_system_watchers {
            let directory = match watch_root(fsw) {
                Some(d) if !d.is_empty() => d,
                _ => continue,
            };
            let kind = effective_kind(fsw);
            let recursive = is_recursive_glob(fsw);
            watches.push(Watch {
                requested_directory: directory,
                kind,
                recursive,
                closed: false,
            });
        }
        inner.watches.insert(id.to_string(), watches);
        Ok(())
    }

    pub fn unwatch_files(&self, id: &str) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap();
        match inner.watches.remove(id) {
            None => Err(format!("lspwatcher: no watcher with id {:?}", id)),
            Some(mut watches) => {
                for watch in &mut watches {
                    watch.closed = true;
                }
                Ok(())
            }
        }
    }

    pub fn close(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            return;
        }
        inner.closed = true;
        let watches = std::mem::take(&mut inner.watches);
        inner.pending.clear();
        drop(inner);
        for (_, mut ws) in watches {
            for w in &mut ws {
                w.closed = true;
            }
        }
    }

    fn forward_events(&self, kind: lsproto::WatchKind, events: &[notify::Event]) {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            return;
        }
        for event in events {
            let change_type = match event.kind {
                notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                    if kind & (lsproto::WATCH_KIND_CREATE | lsproto::WATCH_KIND_CHANGE) == 0 {
                        continue;
                    }
                    lsproto::FILE_CHANGE_TYPE_CHANGED
                }
                notify::EventKind::Remove(_) => {
                    if kind & lsproto::WATCH_KIND_DELETE == 0 {
                        continue;
                    }
                    lsproto::FILE_CHANGE_TYPE_DELETED
                }
                _ => continue,
            };
            for path in &event.paths {
                let path_str = path.to_string_lossy().replace('\\', "/");
                let uri = lsproto::DocumentUri(format!("file://{}", path_str));
                inner.pending.insert(
                    uri.0.clone(),
                    lsproto::FileEvent {
                        uri: uri.clone(),
                        change_type,
                    },
                );
            }
        }

        let pending = std::mem::take(&mut inner.pending);
        drop(inner);
        if !pending.is_empty() {
            let changes: Vec<lsproto::FileEvent> = pending.into_values().collect();
            (self.on_changes)(&changes);
        }
    }
}

fn watch_root(file_system_watcher: &lsproto::FileSystemWatcher) -> Option<String> {
    if let Some(pattern) = &file_system_watcher.glob_pattern.pattern {
        return Some(root_from_glob(pattern));
    }
    if let Some(rp) = &file_system_watcher.glob_pattern.relative_pattern {
        if let Some(uri) = &rp.base_uri.uri {
            let base = lsproto::DocumentUri(uri.clone()).file_name();
            let pattern = format!("{}/{}", base, rp.pattern);
            return Some(root_from_glob(&pattern));
        }
    }
    None
}

fn root_from_glob(pattern: &str) -> String {
    let pattern = tspath::normalize_slashes(pattern);
    let meta_index = pattern
        .char_indices()
        .find(|(_, c)| matches!(*c, '*' | '?' | '[' | '{'))
        .map(|(i, _)| i);

    match meta_index {
        None => {
            let trimmed = pattern.trim_end_matches('/');
            tspath::normalize_path(trimmed)
        }
        Some(idx) => {
            let directory = pattern[..idx].trim_end_matches('/');
            if directory.is_empty() {
                String::new()
            } else {
                tspath::normalize_path(directory)
            }
        }
    }
}

fn watch_pattern_string(file_system_watcher: &lsproto::FileSystemWatcher) -> String {
    if let Some(pattern) = &file_system_watcher.glob_pattern.pattern {
        return pattern.clone();
    }
    if let Some(rp) = &file_system_watcher.glob_pattern.relative_pattern {
        let base = rp
            .base_uri
            .uri
            .as_ref()
            .map(|u| u.clone())
            .unwrap_or_default();
        return format!("{}/{}", base, rp.pattern);
    }
    String::new()
}

fn is_recursive_glob(file_system_watcher: &lsproto::FileSystemWatcher) -> bool {
    watch_pattern_string(file_system_watcher).contains("**")
}

fn effective_kind(file_system_watcher: &lsproto::FileSystemWatcher) -> lsproto::WatchKind {
    file_system_watcher.kind.unwrap_or(
        lsproto::WATCH_KIND_CREATE | lsproto::WATCH_KIND_CHANGE | lsproto::WATCH_KIND_DELETE,
    )
}
