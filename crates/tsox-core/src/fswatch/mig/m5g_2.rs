#![allow(dead_code, unused_imports, unused_variables)]

use std::fmt;
use std::sync::Arc;

use super::m5f::canonicalize_path;
use super::m5f_2::{Event, EventKind, FswatchError};
use super::m5g_3::DirWatch;
use crate::tspath::{get_directory_path, normalize_path, path_is_absolute};

pub const ERR_NIL_CALLBACK: &str = "fswatch: callback must not be nil";
pub const ERR_ROOT_PATH: &str = "fswatch: cannot watch a root path";
pub const ERR_NOT_ABSOLUTE: &str = "fswatch: path must be absolute";
pub const ERR_OVERFLOW: &str = "fswatch: event overflow; some changes were missed";
pub const ERR_WATCH_TERMINATED: &str = "fswatch: watch terminated";
pub const ERR_UNAVAILABLE: &str = "fswatch: watcher not available on this platform";
pub const ERR_FILESYSTEM_UNSUPPORTED: &str =
    "fswatch: watcher backend unsupported on this filesystem";

pub type WatchCallback = Arc<dyn Fn(&[Event], Option<&FswatchError>) + Send + Sync>;
pub type IgnoreFn = Arc<dyn Fn(&str) -> bool + Send + Sync>;
pub type SequenceFn = Arc<dyn Fn() -> u64 + Send + Sync>;

#[derive(Default)]
pub struct WatchOptions {
    pub ignore: Option<IgnoreFn>,
    pub recursive: bool,
}

#[derive(Clone)]
pub struct IgnoreOption {
    pub fn_: IgnoreFn,
}

impl IgnoreOption {
    pub fn apply_watch_option(&self, opts: &mut WatchOptions) {
        opts.ignore = Some(Arc::clone(&self.fn_));
    }
}

#[derive(Clone, Copy, Default)]
pub struct RecursiveOption;

impl RecursiveOption {
    pub fn apply_watch_option(&self, opts: &mut WatchOptions) {
        opts.recursive = true;
    }
}

#[derive(Clone)]
pub enum WatchOption {
    Ignore(IgnoreOption),
    Recursive(RecursiveOption),
}

impl WatchOption {
    pub fn apply_watch_option(&self, opts: &mut WatchOptions) {
        match self {
            WatchOption::Ignore(o) => o.apply_watch_option(opts),
            WatchOption::Recursive(o) => o.apply_watch_option(opts),
        }
    }
}

pub fn with_ignore(fn_: IgnoreFn) -> WatchOption {
    WatchOption::Ignore(IgnoreOption { fn_ })
}

pub fn with_recursive() -> WatchOption {
    WatchOption::Recursive(RecursiveOption)
}

#[derive(Clone)]
pub struct WatchDirectoryRequest {
    pub dir: String,
    pub callback: Option<WatchCallback>,
    pub options: Vec<WatchOption>,
}

pub trait Watcher: Send + Sync {
    fn name(&self) -> &str;
    fn available(&self) -> bool;
    fn has_fast_recursive_backend(&self) -> bool;
    fn watch_directory(
        &self,
        dir: &str,
        callback: WatchCallback,
        options: &[WatchOption],
    ) -> Result<Arc<dyn Watch>, FswatchError>;
    fn watch_directories(
        &self,
        requests: &[WatchDirectoryRequest],
    ) -> Result<Vec<Arc<dyn Watch>>, FswatchError>;
    fn watch_file(&self, path: &str, callback: WatchCallback)
        -> Result<Arc<dyn Watch>, FswatchError>;
}

pub trait Watch: Send + Sync {
    fn close(&self) -> Result<(), FswatchError>;
}

pub trait WatcherBackend: Send + Sync {
    fn start(&self) -> Result<(), FswatchError>;
    fn run(&self) -> Result<(), FswatchError>;
    fn shutdown(&self);
    fn watch_add(&self, w: &Arc<DirWatch>) -> Result<(), FswatchError>;
    fn watch_add_many(&self, watches: &[Arc<DirWatch>]) -> Result<(), FswatchError>;
    fn watch_remove(&self, w: &Arc<DirWatch>);
    fn handle_watcher_error(&self, werr: &DirWatchError);
    fn subscribe(&self, w: &Arc<DirWatch>) -> Result<(), FswatchError>;
    fn close_watch(&self, w: &Arc<DirWatch>) -> Result<(), FswatchError>;
}

#[derive(Clone)]
pub struct Callback {
    pub id: u64,
    pub dir: String,
    pub physical_dir: String,
    pub watch_dir: String,
    pub watch_physical_dir: String,
    pub recursive: bool,
    pub fn_: WatchCallback,
    pub ignore: Option<IgnoreFn>,
    pub since_seq: u64,
    pub terminal: Option<FswatchError>,
    pub delivered: bool,
}

impl Callback {
    pub fn map_event(&self, mut e: Event) -> Event {
        if !self.physical_dir.is_empty() && self.physical_dir != self.dir {
            let physical_path = self.event_physical_path(&e.path);
            if is_in_directory_or_self(&self.physical_dir, &physical_path) {
                e.path = rebase_path(&physical_path, &self.physical_dir, &self.dir);
            }
        }
        e
    }

    pub fn event_physical_path(&self, path: &str) -> String {
        if !self.watch_physical_dir.is_empty()
            && !self.watch_dir.is_empty()
            && self.watch_physical_dir != self.watch_dir
            && is_in_directory_or_self(&self.watch_dir, path)
        {
            return rebase_path(path, &self.watch_dir, &self.watch_physical_dir);
        }
        path.to_string()
    }
}

pub struct DirWatchError {
    pub err: FswatchError,
    pub dir_watch: Arc<DirWatch>,
}

impl fmt::Display for DirWatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.err)
    }
}

impl DirWatchError {
    pub fn unwrap(&self) -> &FswatchError {
        &self.err
    }
}

pub fn is_fswatch_err(err: &FswatchError, target: &str) -> bool {
    err == target || err.contains(target)
}

pub fn is_in_directory_or_self(dir: &str, path: &str) -> bool {
    if dir.is_empty() {
        return false;
    }
    if path == dir {
        return true;
    }
    if !path.starts_with(dir) {
        return false;
    }
    let rest = &path[dir.len()..];
    if rest.is_empty() {
        return false;
    }
    if std::path::is_separator(dir.as_bytes()[dir.len() - 1] as char) {
        return true;
    }
    std::path::is_separator(rest.as_bytes()[0] as char)
}

pub fn is_direct_child(dir: &str, path: &str) -> bool {
    if !path.starts_with(dir) {
        return false;
    }
    let rest = &path[dir.len()..];
    if rest.is_empty() {
        return false;
    }
    if !std::path::is_separator(rest.as_bytes()[0] as char) {
        return false;
    }
    let rest = &rest[1..];
    !rest.is_empty() && !rest.chars().any(std::path::is_separator)
}

pub fn rebase_path(path: &str, from: &str, to: &str) -> String {
    if from == to {
        return path.to_string();
    }
    if path == from {
        return to.to_string();
    }
    if !path.starts_with(from) {
        return path.to_string();
    }
    let suffix = &path[from.len()..];
    if !from.is_empty() && std::path::is_separator(from.as_bytes()[from.len() - 1] as char) {
        return join_path_suffix(to, suffix);
    }
    if suffix.is_empty() || !std::path::is_separator(suffix.as_bytes()[0] as char) {
        return path.to_string();
    }
    join_path_suffix(to, suffix)
}

pub fn join_path_suffix(root: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        return root.to_string();
    }
    let root_ends_in_sep =
        !root.is_empty() && std::path::is_separator(root.as_bytes()[root.len() - 1] as char);
    if std::path::is_separator(suffix.as_bytes()[0] as char) {
        if root_ends_in_sep {
            return format!("{}{}", root, &suffix[1..]);
        }
        return format!("{}{}", root, suffix);
    }
    if root_ends_in_sep {
        return format!("{}{}", root, suffix);
    }
    format!("{}{}{}", root, std::path::MAIN_SEPARATOR, suffix)
}

pub fn file_callback(target: &str, fn_: WatchCallback) -> WatchCallback {
    let target = target.to_string();
    Arc::new(move |events: &[Event], err: Option<&FswatchError>| {
        let filtered: Vec<Event> = events
            .iter()
            .filter(|e| e.path == target)
            .cloned()
            .collect();
        if !filtered.is_empty() || err.is_some() {
            fn_(&filtered, err);
        }
    })
}

pub fn physical_dir_for(dir: &str) -> String {
    match crate::nativepath::mig::m6a::realpath(dir) {
        Ok(realpath) => {
            if realpath == dir {
                return dir.to_string();
            }
            canonicalize_path(&normalize_path(&realpath))
        }
        Err(_) => dir.to_string(),
    }
}

pub fn validate_watch_directory(dir: &str) -> Result<(), FswatchError> {
    match std::fs::metadata(dir) {
        Ok(info) => {
            if !info.is_dir() {
                Err("not a directory".to_string())
            } else {
                Ok(())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
