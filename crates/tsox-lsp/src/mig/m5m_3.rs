use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tsox_core::fswatch::mig::m5f_2::Event as FswatchEvent;
use tsox_core::fswatch::mig::m5f_2::EventKind as FswatchEventKind;
use tsox_core::fswatch::mig::m5g_2::WatchCallback;
use tsox_core::fswatch::mig::m5g_2::WatchOption;
use tsox_core::fswatch::mig::m5g_2::RecursiveOption;
use tsox_core::fswatch::mig::m5g_2::Watcher as FswatchWatcher;
use tsox_core::fswatch::mig::m5g_2::ERR_WATCH_TERMINATED;
use crate::lsp::lsproto_lsp_protocol::FileEvent;
use crate::lsp::lsproto_lsp_protocol::WatchKind;
use crate::lsp::lsproto_lsp_uri::DocumentUri;
use crate::project::logging_logger::Logger;
use tsox_tsoptions::vfs::FileInfo as DirEntry;
use tsox_tsoptions::vfs::FS as VfsFs;

pub const WATCH_KIND_CREATE: WatchKind = 1;
pub const WATCH_KIND_CHANGE: WatchKind = 2;
pub const WATCH_KIND_DELETE: WatchKind = 4;

pub const FILE_CHANGE_TYPE_CREATED: u32 = 1;
pub const FILE_CHANGE_TYPE_CHANGED: u32 = 2;
pub const FILE_CHANGE_TYPE_DELETED: u32 = 3;

const THROTTLE_WINDOW: Duration = Duration::from_millis(75);

pub trait WatcherBackend: Send + Sync {
    fn watch_directory(
        &self,
        dir: &str,
        callback: WatchCallback,
        opts: &[WatchOption],
    ) -> Result<Box<dyn FnOnce() + Send>, String>;
}

pub struct DefaultWatcherBackend {
    pub watcher: Arc<dyn FswatchWatcher>,
}

impl WatcherBackend for DefaultWatcherBackend {
    fn watch_directory(
        &self,
        dir: &str,
        callback: WatchCallback,
        opts: &[WatchOption],
    ) -> Result<Box<dyn FnOnce() + Send>, String> { ::tsox_core::fntrace::enter("watch_directory"); 
        let watch = self
            .watcher
            .watch_directory(dir, callback, opts)
            .map_err(|err| err.to_string())?;
        Ok(Box::new(move || {
            let _ = watch.close();
        }))
    }
}

pub type OnChanges = Arc<dyn Fn(Vec<FileEvent>) + Send + Sync>;

pub struct Watcher {
    pub fs: Arc<dyn VfsFs>,
    pub backend: Box<dyn WatcherBackend>,
    pub on_changes: OnChanges,
    pub logger: Arc<dyn Logger>,

    pub mu: std::sync::Mutex<WatcherState>,
}

pub struct WatcherState {
    pub watches: HashMap<String, Vec<Arc<Watch>>>,
    pub closed: bool,
    pub pending: Option<HashMap<String, FileEvent>>,
    pub flush_timer: Option<std::thread::JoinHandle<()>>,
}

pub struct Watch {
    pub watcher: Arc<Watcher>,
    pub requested_directory: String,
    pub kind: WatchKind,
    pub recursive: bool,

    pub mu: std::sync::Mutex<WatchState>,
}

pub struct WatchState {
    pub subscription: Option<Box<dyn FnOnce() + Send>>,
    pub watched_directory: String,
    pub watching_target: bool,
    pub closed: bool,
}

pub fn new_with_fs_watcher(
    fs: Arc<dyn VfsFs>,
    watcher: Arc<dyn FswatchWatcher>,
    on_changes: OnChanges,
    logger: Arc<dyn Logger>,
) -> Arc<Watcher> { ::tsox_core::fntrace::enter("new_with_fs_watcher"); 
    new_with_backend(
        fs,
        Box::new(DefaultWatcherBackend { watcher }),
        on_changes,
        logger,
    )
}

pub fn new_with_backend(
    fs: Arc<dyn VfsFs>,
    backend: Box<dyn WatcherBackend>,
    on_changes: OnChanges,
    logger: Arc<dyn Logger>,
) -> Arc<Watcher> { ::tsox_core::fntrace::enter("new_with_backend"); 
    Arc::new(Watcher {
        fs,
        backend,
        on_changes,
        logger,
        mu: std::sync::Mutex::new(WatcherState {
            watches: HashMap::new(),
            closed: false,
            pending: None,
            flush_timer: None,
        }),
    })
}

impl Watch {
    pub fn target_callback(self: &Arc<Self>, watched_directory: String) -> WatchCallback { ::tsox_core::fntrace::enter("target_callback"); 
        let watcher = self.watcher.clone();
        let watch = Arc::clone(self);
        let kind = self.kind;
        Arc::new(move |events: &[FswatchEvent], err: Option<&String>| {
            let mut terminated = false;
            if let Some(err) = err {
                watcher.logger.logf(
                    &format!(
                        "lspwatcher: watch error in {watched_directory:?}: {err}"
                    ),
                    &[],
                );
                if err.contains(ERR_WATCH_TERMINATED) {
                    terminated = true;
                }
            }
            if !events.is_empty() {
                watcher.forward_events(kind, events);
            }
            if terminated {
                watch.handle_terminated();
            }
        })
    }

    pub fn ancestor_callback(self: &Arc<Self>) -> WatchCallback { ::tsox_core::fntrace::enter("ancestor_callback"); 
        let watch = Arc::clone(self);
        Arc::new(move |_events: &[FswatchEvent], _err: Option<&String>| {
            let _ = watch.reconcile(true);
        })
    }

    pub fn handle_terminated(self: &Arc<Self>) { ::tsox_core::fntrace::enter("handle_terminated"); 
        let mut state = self.mu.lock().unwrap();
        if state.closed {
            return;
        }
        let previous = state.subscription.take();
        state.watched_directory = String::new();
        state.watching_target = false;
        drop(state);
        if let Some(previous) = previous {
            previous();
        }
        let _ = self.reconcile(true);
    }

    pub fn reconcile(self: &Arc<Self>, emit_synthetic_creates: bool) -> Result<(), String> { ::tsox_core::fntrace::enter("reconcile"); 
        let mut state = self.mu.lock().unwrap();
        loop {
            if state.closed {
                return Ok(());
            }
            let watcher = &self.watcher;
            if watcher.fs.directory_exists(&self.requested_directory) {
                if state.watching_target && state.subscription.is_some() {
                    return Ok(());
                }
                let target_directory = self.requested_directory.clone();
                let mut options: Vec<WatchOption> = Vec::new();
                if self.recursive {
                    options.push(WatchOption::Recursive(RecursiveOption));
                }
                let subscription = watcher
                    .backend
                    .watch_directory(&target_directory, self.target_callback(target_directory.clone()), &options)?;
                let previous = state.subscription.take();
                state.subscription = Some(subscription);
                state.watched_directory = target_directory.clone();
                state.watching_target = true;
                if let Some(previous) = previous {
                    previous();
                }
                if emit_synthetic_creates {
                    watcher.emit_synthetic_creates(&target_directory, self.kind, self.recursive);
                }
                return Ok(());
            }

            let ancestor = nearest_existing_ancestor(watcher.fs.as_ref(), &self.requested_directory);
            let ancestor_directory = match ancestor {
                Some(directory) => directory,
                None => {
                    if state.subscription.is_some() {
                        let previous = state.subscription.take();
                        state.watched_directory = String::new();
                        state.watching_target = false;
                        if let Some(previous) = previous {
                            previous();
                        }
                    }
                    return Ok(());
                }
            };
            if !state.watching_target
                && state.subscription.is_some()
                && state.watched_directory == ancestor_directory
            {
                return Ok(());
            }
            let subscription = watcher
                .backend
                .watch_directory(&ancestor_directory, self.ancestor_callback(), &[])?;
            let previous = state.subscription.take();
            state.subscription = Some(subscription);
            state.watched_directory = ancestor_directory;
            state.watching_target = false;
            if let Some(previous) = previous {
                previous();
            }
        }
    }
}

pub fn nearest_existing_ancestor(fs: &dyn VfsFs, dir: &str) -> Option<String> { ::tsox_core::fntrace::enter("nearest_existing_ancestor"); 
    let mut dir = dir.to_string();
    loop {
        if fs.directory_exists(&dir) {
            return Some(dir);
        }
        let parent = tsox_core::tspath::get_directory_path(&dir);
        if parent == dir {
            return None;
        }
        dir = parent;
    }
}

impl Watcher {
    pub fn forward_events(&self, kind: WatchKind, events: &[FswatchEvent]) { ::tsox_core::fntrace::enter("forward_events"); 
        let mut state = self.mu.lock().unwrap();
        if state.closed {
            return;
        }
        let pending = state.pending.get_or_insert_with(HashMap::new);
        for event in events {
            let change_type = match event.kind {
                FswatchEventKind::Update => {
                    if kind & (WATCH_KIND_CREATE | WATCH_KIND_CHANGE) == 0 {
                        continue;
                    }
                    FILE_CHANGE_TYPE_CHANGED
                }
                FswatchEventKind::Delete => {
                    if kind & WATCH_KIND_DELETE == 0 {
                        continue;
                    }
                    FILE_CHANGE_TYPE_DELETED
                }
            };
            let path = tsox_core::tspath::normalize_slashes(&event.path);
            let uri = DocumentUri(crate::ls::lsconv_converters::file_name_to_document_uri(
                &path,
            ));
            pending.insert(uri.0.clone(), FileEvent { uri, change_type });
        }
        schedule_flush_locked(&mut state, self);
    }

    pub fn emit_synthetic_creates(&self, directory: &str, kind: WatchKind, recursive: bool) { ::tsox_core::fntrace::enter("emit_synthetic_creates"); 
        if kind & WATCH_KIND_CREATE == 0 {
            return;
        }
        let mut paths = vec![directory.to_string()];
        if recursive {
            let _ = self.fs.walk_dir(directory, &mut |path: &str, _entry: &DirEntry| {
                let normalized_path = tsox_core::tspath::normalize_slashes(path);
                if normalized_path != directory {
                    paths.push(normalized_path);
                }
            });
        } else {
            let entries = self.fs.get_accessible_entries(directory);
            for name in entries.files.iter().chain(entries.directories.iter()) {
                paths.push(tsox_core::tspath::combine_paths(directory, &[name]));
            }
        }
        self.enqueue_synthetic_creates(&paths);
    }

    pub fn enqueue_synthetic_creates(&self, paths: &[String]) { ::tsox_core::fntrace::enter("enqueue_synthetic_creates"); 
        let mut state = self.mu.lock().unwrap();
        if state.closed {
            return;
        }
        let pending = state.pending.get_or_insert_with(HashMap::new);
        for path in paths {
            let uri = DocumentUri(crate::ls::lsconv_converters::file_name_to_document_uri(path));
            if pending.contains_key(&uri.0) {
                continue;
            }
            pending.insert(
                uri.0.clone(),
                FileEvent {
                    uri,
                    change_type: FILE_CHANGE_TYPE_CREATED,
                },
            );
        }
        schedule_flush_locked(&mut state, self);
    }

    pub fn flush(&self) { ::tsox_core::fntrace::enter("flush"); 
        let mut state = self.mu.lock().unwrap();
        if state.closed {
            return;
        }
        let pending = state.pending.take();
        state.flush_timer = None;
        drop(state);

        let pending = match pending {
            Some(pending) if !pending.is_empty() => pending,
            _ => return,
        };
        let changes: Vec<FileEvent> = pending.into_values().collect();
        (self.on_changes)(changes);
    }
}

fn schedule_flush_locked(state: &mut WatcherState, watcher: &Watcher) { ::tsox_core::fntrace::enter("schedule_flush_locked"); 
    if state.flush_timer.is_none() {
        let watcher = unsafe { &*(watcher as *const Watcher) };
        state.flush_timer = Some(std::thread::spawn(move || {
            std::thread::sleep(THROTTLE_WINDOW);
            watcher.flush();
        }));
    }
}

pub fn watch_root(file_system_watcher: &crate::lsp::lsproto_lsp_protocol::FileSystemWatcher) -> Option<String> { ::tsox_core::fntrace::enter("watch_root"); 
    if let Some(pattern) = &file_system_watcher.glob_pattern.pattern {
        return Some(root_from_glob(pattern));
    }
    if let Some(relative_pattern) = &file_system_watcher.glob_pattern.relative_pattern {
        let base = match &relative_pattern.base_uri.uri {
            Some(uri) => crate::mig::m5m::document_uri_file_name(&DocumentUri(uri.clone())),
            None => return None,
        };
        let pattern =
            tsox_core::tspath::combine_paths(&base, &[relative_pattern.pattern.as_str()]);
        return Some(root_from_glob(&pattern));
    }
    None
}

pub fn root_from_glob(pattern: &str) -> String { ::tsox_core::fntrace::enter("root_from_glob"); 
    let pattern = tsox_core::tspath::normalize_slashes(pattern);
    let mut meta_index: Option<usize> = None;
    for (i, ch) in pattern.char_indices() {
        match ch {
            '*' | '?' | '[' | '{' => meta_index = Some(i),
            _ => {}
        }
        if meta_index.is_some() {
            break;
        }
    }
    let meta_index = match meta_index {
        Some(index) => index,
        None => {
            return tsox_core::tspath::normalize_path(pattern.trim_end_matches('/'));
        }
    };
    let directory = pattern[..meta_index].trim_end_matches('/');
    if directory.is_empty() {
        return String::new();
    }
    tsox_core::tspath::normalize_path(directory)
}

pub fn watch_pattern_string(file_system_watcher: &crate::lsp::lsproto_lsp_protocol::FileSystemWatcher) -> String { ::tsox_core::fntrace::enter("watch_pattern_string"); 
    if let Some(pattern) = &file_system_watcher.glob_pattern.pattern {
        return pattern.clone();
    }
    if let Some(relative_pattern) = &file_system_watcher.glob_pattern.relative_pattern {
        let base = relative_pattern
            .base_uri
            .uri
            .clone()
            .unwrap_or_default();
        return format!("{base}/{}", relative_pattern.pattern);
    }
    String::new()
}

pub fn is_recursive_glob(file_system_watcher: &crate::lsp::lsproto_lsp_protocol::FileSystemWatcher) -> bool { ::tsox_core::fntrace::enter("is_recursive_glob"); 
    watch_pattern_string(file_system_watcher).contains("**")
}

pub fn effective_kind(file_system_watcher: &crate::lsp::lsproto_lsp_protocol::FileSystemWatcher) -> WatchKind { ::tsox_core::fntrace::enter("effective_kind"); 
    match &file_system_watcher.kind {
        Some(kind) => *kind,
        None => WATCH_KIND_CREATE | WATCH_KIND_CHANGE | WATCH_KIND_DELETE,
    }
}

pub fn server_progress_reporter_done(server: &crate::mig::m5n::Server) -> bool { ::tsox_core::fntrace::enter("server_progress_reporter_done"); 
    server.background_ctx.is_done()
}
