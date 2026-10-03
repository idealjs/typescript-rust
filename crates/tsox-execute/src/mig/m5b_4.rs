#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard};

use tsox_core::tspath::{self, ComparePathsOptions};
use tsox_tsoptions::vfs::FS;

mod fswatch {
    pub use tsox_core::fswatch::mig::m5f_2::{Event, EventKind, FswatchError as Error};
    pub use tsox_core::fswatch::mig::m5g_2::{
        is_fswatch_err, with_ignore, with_recursive, IgnoreFn, Watch, WatchCallback,
        WatchDirectoryRequest, WatchOption, Watcher, ERR_OVERFLOW, ERR_WATCH_TERMINATED,
    };

    use std::sync::Arc;

    pub fn default() -> Arc<dyn Watcher> { ::tsox_core::fntrace::enter("default"); 
        use tsox_core::fswatch::mig::m5g_7;
        if cfg!(target_os = "linux") {
            let fanotify = m5g_7::fanotify();
            if fanotify.available() {
                return fanotify;
            }
            m5g_7::inotify()
        } else if cfg!(target_os = "android") {
            m5g_7::inotify()
        } else if cfg!(target_os = "macos") {
            let fsevents = m5g_7::fsevents();
            if fsevents.available() {
                return fsevents;
            }
            m5g_7::kqueue()
        } else if cfg!(target_os = "windows") {
            m5g_7::windows()
        } else {
            m5g_7::kqueue()
        }
    }

    pub fn normalize_ignore(ignore: IgnoreFn) -> IgnoreFn { ::tsox_core::fntrace::enter("normalize_ignore"); 
        ignore
    }
}

pub struct Context {
    pub done: Arc<dyn Fn() -> bool + Send + Sync>,
}

pub trait WatchBackend: Send + Sync {
    fn watch_directory(
        &self,
        dir: &str,
        callback: fswatch::WatchCallback,
        recursive: bool,
        ignore: Option<fswatch::IgnoreFn>,
    ) -> Result<Box<dyn WatchCloser>, String>;
    fn watch_directories(
        &self,
        requests: &[WatchDirectoryRequest],
    ) -> Result<Vec<Box<dyn WatchCloser>>, String>;
}

pub trait WatchCloser: Send + Sync {
    fn close(&mut self);
}

pub type IgnoreFn = Arc<dyn Fn(&str) -> bool + Send + Sync>;

pub struct WatchDirectoryRequest {
    pub dir: String,
    pub callback: fswatch::WatchCallback,
    pub recursive: bool,
    pub ignore: Option<IgnoreFn>,
}

pub struct FSWatchBackend {
    pub inner: Arc<dyn fswatch::Watcher>,
}

impl WatchBackend for FSWatchBackend {
    fn watch_directory(
        &self,
        dir: &str,
        callback: fswatch::WatchCallback,
        recursive: bool,
        ignore: Option<fswatch::IgnoreFn>,
    ) -> Result<Box<dyn WatchCloser>, String> { ::tsox_core::fntrace::enter("watch_directory"); 
        let mut closers = self.watch_directories(&[WatchDirectoryRequest {
            dir: dir.to_string(),
            callback,
            recursive,
            ignore: ignore.map(fswatch::normalize_ignore),
        }])?;
        Ok(closers.remove(0))
    }

    fn watch_directories(
        &self,
        requests: &[WatchDirectoryRequest],
    ) -> Result<Vec<Box<dyn WatchCloser>>, String> { ::tsox_core::fntrace::enter("watch_directories"); 
        let mut fswatch_requests = Vec::with_capacity(requests.len());
        for request in requests {
            let mut opts: Vec<fswatch::WatchOption> = Vec::new();
            if request.recursive {
                opts.push(fswatch::with_recursive());
            }
            if let Some(ignore) = &request.ignore {
                opts.push(fswatch::with_ignore(Arc::clone(ignore)));
            }
            fswatch_requests.push(fswatch::WatchDirectoryRequest {
                dir: request.dir.clone(),
                callback: Some(request.callback.clone()),
                options: opts,
            });
        }
        let watches = self.inner.watch_directories(&fswatch_requests)?;
        Ok(watches
            .into_iter()
            .map(|watch| {
                Box::new(WatchCloserImpl(watch)) as Box<dyn WatchCloser>
            })
            .collect())
    }
}

struct WatchCloserImpl(Arc<dyn fswatch::Watch>);

impl WatchCloser for WatchCloserImpl {
    fn close(&mut self) { ::tsox_core::fntrace::enter("close"); 
        let _ = self.0.close();
    }
}

pub fn should_ignore_watch_path(path: &str) -> bool { ::tsox_core::fntrace::enter("should_ignore_watch_path"); 
    let p = tspath::normalize_slashes(path);
    p.ends_with("/.git")
        || p.contains("/.git/")
        || p.contains("/node_modules/.")
        || p.contains("/.#")
}

pub fn can_watch_directory(dir: &str) -> bool { ::tsox_core::fntrace::enter("can_watch_directory"); 
    let components = tspath::get_path_components(dir, "");
    let length = components.len();
    if length <= 2 {
        return false;
    }
    let root_length = perceived_os_root_length_for_watching(&components);
    length > root_length + 1
}

pub fn perceived_os_root_length_for_watching(components: &[String]) -> usize { ::tsox_core::fntrace::enter("perceived_os_root_length_for_watching"); 
    let length = components.len();
    if length <= 1 {
        return 1;
    }
    let root = &components[0];
    let mut index_after_os_root = 1;
    let mut is_dos_style = root.len() >= 2 && tspath::is_volume_character(root.as_bytes()[0]) && root.as_bytes()[1] == b':';

    if root != "/" && !is_dos_style && components.len() > 1 {
        if components[1].len() >= 2
            && tspath::is_volume_character(components[1].as_bytes()[0])
            && components[1].ends_with('$')
        {
            if length == 2 {
                return 2;
            }
            index_after_os_root = 2;
            is_dos_style = true;
        }
    }

    if is_dos_style
        && (index_after_os_root >= length || !components[index_after_os_root].eq_ignore_ascii_case("users"))
    {
        return index_after_os_root;
    }

    if index_after_os_root < length && components[index_after_os_root].eq_ignore_ascii_case("workspaces") {
        return index_after_os_root + 1;
    }

    index_after_os_root + 2
}

struct WatchedDir {
    closer: Box<dyn WatchCloser>,
    recursive: bool,
}

struct DirWatchUpdate {
    dir: String,
    recursive: bool,
}

pub struct WatchManager {
    mu: Mutex<()>,
    backend: Option<Arc<dyn WatchBackend>>,
    watched_dirs: Mutex<HashMap<String, WatchedDir>>,
    do_cycle_ch: std::sync::mpsc::SyncSender<()>,

    pub debug_log: Option<Arc<Mutex<Box<dyn Write + Send>>>>,

    warn_writer: Arc<Mutex<Box<dyn Write + Send>>>,
    dir_exists: Arc<dyn Fn(&str) -> bool + Send + Sync>,

    changed_state: Mutex<ChangedState>,
}

#[derive(Default)]
struct ChangedState {
    changed_paths: Option<HashMap<String, fswatch::EventKind>>,
    changed_overflow: bool,
}

impl WatchManager {
    pub fn new(warn_writer: Box<dyn Write + Send>, dir_exists: Arc<dyn Fn(&str) -> bool + Send + Sync>) -> Self { ::tsox_core::fntrace::enter("new"); 
        let (tx, _) = std::sync::mpsc::sync_channel(1);
        Self {
            mu: Mutex::new(()),
            backend: None,
            watched_dirs: Mutex::new(HashMap::new()),
            do_cycle_ch: tx,
            debug_log: None,
            warn_writer: Arc::new(Mutex::new(warn_writer)),
            dir_exists,
            changed_state: Mutex::new(ChangedState::default()),
        }
    }

    pub fn set_backend(&mut self, backend: Arc<dyn WatchBackend>) { ::tsox_core::fntrace::enter("set_backend"); 
        self.backend = Some(backend);
    }

    pub fn backend(&self) -> Option<Arc<dyn WatchBackend>> { ::tsox_core::fntrace::enter("backend"); 
        self.backend.clone()
    }

    pub fn ensure_default_backend(&mut self) { ::tsox_core::fntrace::enter("ensure_default_backend"); 
        if self.backend.is_none() {
            let fsw = fswatch::default();
            self.backend = Some(Arc::new(FSWatchBackend { inner: fsw.clone() }));
            if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                let _ = writeln!(debug_log, "[watch] using {} backend", fsw.name());
            }
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, ()> { ::tsox_core::fntrace::enter("lock"); 
        self.mu.lock().unwrap()
    }

    pub fn unlock(&self) { ::tsox_core::fntrace::enter("unlock"); }

    pub fn do_cycle_ch(&self) -> &std::sync::mpsc::SyncSender<()> { ::tsox_core::fntrace::enter("do_cycle_ch"); 
        &self.do_cycle_ch
    }

    pub fn drain_events(&self) -> (HashMap<String, fswatch::EventKind>, bool) { ::tsox_core::fntrace::enter("drain_events"); 
        let mut state = self.changed_state.lock().unwrap();
        let changed = state.changed_paths.take().unwrap_or_default();
        let overflow = state.changed_overflow;
        state.changed_overflow = false;
        (changed, overflow)
    }

    pub fn force_overflow(&self) { ::tsox_core::fntrace::enter("force_overflow"); 
        self.changed_state.lock().unwrap().changed_overflow = true;
    }

    pub fn signal_do_cycle(&self) { ::tsox_core::fntrace::enter("signal_do_cycle"); 
        let _ = self.do_cycle_ch.try_send(());
    }

    pub fn on_watch_events(&self, events: &[fswatch::Event], err: Option<&fswatch::Error>) { ::tsox_core::fntrace::enter("on_watch_events"); 
        if let Some(err) = err {
            if fswatch::is_fswatch_err(err, fswatch::ERR_OVERFLOW) {
                if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                    let _ = writeln!(debug_log, "[watch] event overflow, triggering rebuild");
                }
                self.changed_state.lock().unwrap().changed_overflow = true;
                self.signal_do_cycle();
                return;
            }
            let _ = writeln!(self.warn_writer.lock().unwrap(), "Warning: File watch error: {}", err);
            return;
        }

        if !events.is_empty() {
            if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                let _ = write!(debug_log, "[watch] {} event(s): ", events.len());
                for (i, e) in events.iter().enumerate() {
                    if i > 0 {
                        let _ = write!(debug_log, ", ");
                    }
                    if i >= 5 {
                        let _ = write!(debug_log, "... and {} more", events.len() - i);
                        break;
                    }
                    let _ = write!(debug_log, "{} {}", e.kind, e.path);
                }
                let _ = writeln!(debug_log);
            }
            {
                let mut state = self.changed_state.lock().unwrap();
                let paths = state.changed_paths.get_or_insert_with(HashMap::new);
                for e in events {
                    paths.insert(e.path.clone(), e.kind);
                }
            }
            self.signal_do_cycle();
        }
    }

    pub fn handle_watch_terminated(&self, dir: &str, identity: *const WatchedDir) { ::tsox_core::fntrace::enter("handle_watch_terminated"); 
        if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
            let _ = writeln!(debug_log, "[watch] watch terminated: {}", dir);
        }
        let mut stale_closer: Option<Box<dyn WatchCloser>> = None;
        {
            let mut watched = self.watched_dirs.lock().unwrap();
            let identity_matches = watched
                .get(dir)
                .map_or(false, |wd| std::ptr::eq(wd as *const WatchedDir, identity));
            if identity_matches {
                if let Some(wd) = watched.remove(dir) {
                    stale_closer = Some(wd.closer);
                }
            }
        }
        if let Some(closer) = &mut stale_closer {
            closer.close();
        }
        self.changed_state.lock().unwrap().changed_overflow = true;
        self.signal_do_cycle();
    }

    pub fn close_all_watches(&self) { ::tsox_core::fntrace::enter("close_all_watches"); 
        let mut closers: Vec<Box<dyn WatchCloser>> = Vec::new();
        {
            let mut watched = self.watched_dirs.lock().unwrap();
            for (_, wd) in watched.drain() {
                closers.push(wd.closer);
            }
        }
        for mut closer in closers {
            closer.close();
        }
    }

    fn create_dir_watch_request(&self, dir: &str, entry: *const WatchedDir) -> WatchDirectoryRequest { ::tsox_core::fntrace::enter("create_dir_watch_request"); 
        let dir_owned = dir.to_string();
        WatchDirectoryRequest {
            dir: dir_owned.clone(),
            recursive: false,
            ignore: None,
            callback: Arc::new(move |events, err| {
                if let Some(err) = &err {
                    if fswatch::is_fswatch_err(err, fswatch::ERR_WATCH_TERMINATED) {
                        return;
                    }
                }
            }),
        }
    }

    pub fn resolve_desired_dirs(&self, desired_dirs: &HashMap<String, bool>) -> HashMap<String, bool> { ::tsox_core::fntrace::enter("resolve_desired_dirs"); 
        let mut resolved: HashMap<String, bool> = HashMap::with_capacity(desired_dirs.len());
        for (dir, recursive) in desired_dirs {
            let mut watch_dir = dir.clone();
            let mut watch_recursive = *recursive;
            while !(self.dir_exists)(&watch_dir) {
                let parent = tspath::get_directory_path(&watch_dir);
                if parent == watch_dir {
                    break;
                }
                watch_dir = parent;
                watch_recursive = false;
            }
            if !(self.dir_exists)(&watch_dir) || !can_watch_directory(&watch_dir) {
                if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                    let _ = writeln!(debug_log, "[watch] no watchable ancestor for {}", dir);
                }
                continue;
            }
            if watch_dir != *dir {
                if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                    let _ = writeln!(debug_log, "[watch] resolved {} to ancestor {}", dir, watch_dir);
                }
            }
            resolved
                .entry(watch_dir)
                .and_modify(|existing| *existing = *existing || watch_recursive)
                .or_insert(watch_recursive);
        }
        resolved
    }

    pub fn reconcile_watches(&self, desired_dirs: &HashMap<String, bool>) -> Result<(), String> { ::tsox_core::fntrace::enter("reconcile_watches"); 
        let backend = match &self.backend {
            Some(b) => b,
            None => return Ok(()),
        };

        let mut additions: Vec<DirWatchUpdate> = Vec::new();
        let mut changes: Vec<DirWatchUpdate> = Vec::new();
        let mut stale_closers: Vec<Box<dyn WatchCloser>> = Vec::new();

        {
            let mut watched = self.watched_dirs.lock().unwrap();
            for (dir, wd) in watched.iter() {
                match desired_dirs.get(dir) {
                    None => {
                        if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                            let _ = writeln!(debug_log, "[watch] closing stale dir watch: {}", dir);
                        }
                    }
                    Some(recursive) if *recursive != wd.recursive => {
                        if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                            let _ = writeln!(
                                debug_log,
                                "[watch] recreating dir watch {} (recursive {}→{})",
                                dir, wd.recursive, recursive
                            );
                        }
                    }
                    _ => {}
                }
            }
            let to_remove: Vec<String> = watched
                .iter()
                .filter_map(|(dir, wd)| match desired_dirs.get(dir) {
                    None => Some(dir.clone()),
                    Some(recursive) if *recursive != wd.recursive => Some(dir.clone()),
                    _ => None,
                })
                .collect();
            for dir in to_remove {
                if let Some(recursive) = desired_dirs.get(&dir) {
                    changes.push(DirWatchUpdate {
                        dir: dir.clone(),
                        recursive: *recursive,
                    });
                }
                if let Some(wd) = watched.remove(&dir) {
                    stale_closers.push(wd.closer);
                }
            }
            for (dir, recursive) in desired_dirs {
                if !watched.contains_key(dir) {
                    if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                        let _ = writeln!(debug_log, "[watch] watching directory {} (recursive={})", dir, recursive);
                    }
                    additions.push(DirWatchUpdate {
                        dir: dir.clone(),
                        recursive: *recursive,
                    });
                }
            }
        }
        for mut closer in stale_closers {
            closer.close();
        }
        additions.append(&mut changes);
        self.create_dir_watches(&additions)
    }

    fn create_dir_watches(&self, updates: &[DirWatchUpdate]) -> Result<(), String> { ::tsox_core::fntrace::enter("create_dir_watches"); 
        if updates.is_empty() {
            return Ok(());
        }
        let backend = self.backend.as_ref().unwrap();
        let mut requests = Vec::with_capacity(updates.len());
        for update in updates {
            requests.push(WatchDirectoryRequest {
                dir: update.dir.clone(),
                callback: Arc::new(|_events, _err| {}),
                recursive: update.recursive,
                ignore: None,
            });
        }
        match backend.watch_directories(&requests) {
            Ok(closers) => {
                let mut watched = self.watched_dirs.lock().unwrap();
                for (update, closer) in updates.iter().zip(closers.into_iter()) {
                    watched.insert(
                        update.dir.clone(),
                        WatchedDir {
                            closer,
                            recursive: update.recursive,
                        },
                    );
                }
                Ok(())
            }
            Err(err) => {
                if let Some(mut debug_log) = self.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                    for update in updates {
                        let _ = writeln!(
                            debug_log,
                            "[watch] failed to watch directory {}: {}",
                            update.dir, err
                        );
                    }
                }
                Err(err)
            }
        }
    }

    pub fn is_path_under_watch(&self, path: &str, opts: &ComparePathsOptions) -> bool { ::tsox_core::fntrace::enter("is_path_under_watch"); 
        let watched = self.watched_dirs.lock().unwrap();
        for dir in watched.keys() {
            if contains_path(dir, path, opts) {
                return true;
            }
        }
        false
    }

    pub fn run_loop(&self, ctx: &Context, do_cycle: &mut dyn FnMut()) { ::tsox_core::fntrace::enter("run_loop"); 
        loop {
            if (ctx.done)() {
                self.close_all_watches();
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
            do_cycle();
        }
    }
}

pub struct DirWatchSet {
    opts: ComparePathsOptions,
    dirs: HashMap<String, bool>,
}

impl DirWatchSet {
    pub fn new(opts: ComparePathsOptions) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            opts,
            dirs: HashMap::new(),
        }
    }

    pub fn canonical(&self, dir: &str) -> String { ::tsox_core::fntrace::enter("canonical"); 
        tspath::get_canonical_file_name(dir, self.opts.use_case_sensitive_file_names)
    }

    pub fn set(&mut self, dir: &str, recursive: bool) { ::tsox_core::fntrace::enter("set"); 
        let dir = self.canonical(dir);
        self.dirs.entry(dir).and_modify(|existing| *existing |= recursive).or_insert(recursive);
    }

    pub fn covered(&self, dir: &str) -> bool { ::tsox_core::fntrace::enter("covered"); 
        let mut dir = self.canonical(dir);
        if self.dirs.contains_key(&dir) {
            return true;
        }
        let root_length = tspath::get_root_length(&dir);
        while dir.len() > root_length {
            dir = tspath::get_directory_path(&dir);
            if self.dirs.get(&dir).copied().unwrap_or(false) {
                return true;
            }
        }
        false
    }

    pub fn dirs(&self) -> HashMap<String, bool> { ::tsox_core::fntrace::enter("dirs"); 
        self.dirs.clone()
    }
}

fn contains_path(parent: &str, child: &str, options: &ComparePathsOptions) -> bool { ::tsox_core::fntrace::enter("contains_path"); 
    let parent = tspath::combine_paths(&options.current_directory, &[parent]);
    let child = tspath::combine_paths(&options.current_directory, &[child]);
    if parent.is_empty() || child.is_empty() {
        return false;
    }
    if parent == child {
        return true;
    }
    let parent_components =
        tspath::reduce_path_components(&tspath::get_path_components(&parent, ""));
    let child_components =
        tspath::reduce_path_components(&tspath::get_path_components(&child, ""));
    if child_components.len() < parent_components.len() {
        return false;
    }
    let case_sensitive = options.use_case_sensitive_file_names;
    for (i, parent_component) in parent_components.iter().enumerate() {
        let child_component = &child_components[i];
        let equal = if i == 0 {
            parent_component.eq_ignore_ascii_case(child_component)
        } else if case_sensitive {
            parent_component == child_component
        } else {
            parent_component.eq_ignore_ascii_case(child_component)
        };
        if !equal {
            return false;
        }
    }
    true
}
