#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use super::m5a2_2::fs_baseline_util;
use super::m5b_4::{WatchBackend, WatchCloser, WatchDirectoryRequest};
use tsox_core::fswatch::mig::m5f_2::{Event, EventKind};
use tsox_core::fswatch::mig::m5g_2::{ERR_OVERFLOW, IgnoreFn, WatchCallback};
use tsox_core::tspath;

pub struct MockWatch {
    pub path: String,
    pub callback: WatchCallback,
    pub recursive: bool,
    pub ignore: Option<IgnoreFn>,
    pub closed: bool,
}

impl WatchCloser for MockWatch {
    fn close(&mut self) { ::tsox_core::fntrace::enter("close"); 
        self.closed = true;
    }
}

pub struct MockWatchBackend {
    pub dirs: Mutex<HashMap<String, Arc<Mutex<MockWatch>>>>,
    pub directory_exists: bool,
    pub use_case_sensitive_file_names: bool,
}

impl MockWatchBackend {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            dirs: Mutex::new(HashMap::new()),
            directory_exists: false,
            use_case_sensitive_file_names: false,
        }
    }

    pub fn has_watches(&self) -> bool { ::tsox_core::fntrace::enter("has_watches"); 
        !self.dirs.lock().unwrap().is_empty()
    }

    pub fn watch_directory(
        &self,
        dir: &str,
        callback: WatchCallback,
        recursive: bool,
        ignore: Option<IgnoreFn>,
    ) -> Result<Box<dyn WatchCloser>, String> { ::tsox_core::fntrace::enter("watch_directory"); 
        let closers = self.watch_directories(&[WatchDirectoryRequest {
            dir: dir.to_string(),
            callback,
            recursive,
            ignore,
        }])?;
        Ok(closers.into_iter().next().unwrap())
    }

    pub fn watch_directories(
        &self,
        requests: &[WatchDirectoryRequest],
    ) -> Result<Vec<Box<dyn WatchCloser>>, String> { ::tsox_core::fntrace::enter("watch_directories"); 
        let mut dirs = self.dirs.lock().unwrap();
        for request in requests {
            if self.directory_exists && !mock_directory_exists(&request.dir) {
                return Err(format!("directory does not exist: {}", request.dir));
            }
        }
        let mut closers: Vec<Box<dyn WatchCloser>> = Vec::with_capacity(requests.len());
        for request in requests {
            let watch = Arc::new(Mutex::new(MockWatch {
                path: request.dir.clone(),
                callback: Arc::clone(&request.callback),
                recursive: request.recursive,
                ignore: request.ignore.clone(),
                closed: false,
            }));
            dirs.insert(request.dir.clone(), Arc::clone(&watch));
            closers.push(Box::new(MockWatchHandle { watch }));
        }
        Ok(closers)
    }

    pub fn send_events(&self, events: &[Event]) { ::tsox_core::fntrace::enter("send_events"); 
        struct Target {
            cb: WatchCallback,
            events: Vec<Event>,
        }
        let mut targets: HashMap<usize, Target> = HashMap::new();

        for e in events {
            let dirs = self.dirs.lock().unwrap();
            for (i, w) in dirs.values().enumerate() {
                let w = w.lock().unwrap();
                if w.closed {
                    continue;
                }
                if let Some(ignore) = &w.ignore {
                    if ignore(&e.path) {
                        continue;
                    }
                }
                if !path_is_under(
                    &e.path,
                    &w.path,
                    w.recursive,
                    self.use_case_sensitive_file_names,
                ) {
                    continue;
                }
                match targets.get_mut(&i) {
                    Some(t) => t.events.push(e.clone()),
                    None => {
                        targets.insert(
                            i,
                            Target {
                                cb: Arc::clone(&w.callback),
                                events: vec![e.clone()],
                            },
                        );
                    }
                }
            }
        }

        for (_, t) in targets {
            (t.cb)(&t.events, None);
        }
    }

    pub fn send_overflow(&self) { ::tsox_core::fntrace::enter("send_overflow"); 
        let mut cbs: Vec<WatchCallback> = Vec::new();
        {
            let dirs = self.dirs.lock().unwrap();
            for w in dirs.values() {
                let w = w.lock().unwrap();
                if !w.closed {
                    cbs.push(Arc::clone(&w.callback));
                }
            }
        }
        for cb in cbs {
            (cb)(&[], Some(&ERR_OVERFLOW.to_string()));
        }
    }

    pub fn send_changed_paths(&self, changes: &[fs_baseline_util::FileChange]) { ::tsox_core::fntrace::enter("send_changed_paths"); 
        let mut events: Vec<Event> = Vec::with_capacity(changes.len() * 2);
        let mut seen_dirs: HashSet<String> = HashSet::new();
        for c in changes {
            let kind = if c.deleted {
                EventKind::Delete
            } else {
                EventKind::Update
            };
            events.push(Event {
                kind,
                path: c.path.clone(),
                included_watch_root: false,
            });
            let mut dir = tspath::get_directory_path(&c.path);
            while dir != "" && dir != "/" && dir != "." {
                if seen_dirs.contains(&dir) {
                    break;
                }
                seen_dirs.insert(dir.clone());
                events.push(Event {
                    kind: EventKind::Update,
                    path: dir.clone(),
                    included_watch_root: false,
                });
                let parent = tspath::get_directory_path(&dir);
                if parent == dir {
                    break;
                }
                dir = parent;
            }
        }
        self.send_events(&events);
    }

    pub fn watch_state(&self) -> String { ::tsox_core::fntrace::enter("watch_state"); 
        let dirs = self.dirs.lock().unwrap();

        let mut b = String::new();
        b.push_str("Watch Registrations::\n");

        let mut active_dirs: Vec<String> = Vec::new();
        for (dir, w) in dirs.iter() {
            if !w.lock().unwrap().closed {
                active_dirs.push(dir.clone());
            }
        }
        active_dirs.sort();

        b.push_str("Directory watches::\n");
        if active_dirs.is_empty() {
            b.push_str("  (none)\n");
        }
        for d in &active_dirs {
            let w = dirs.get(d).unwrap().lock().unwrap();
            if w.recursive {
                b.push_str(&format!("  {} (recursive)\n", d));
            } else {
                b.push_str(&format!("  {}\n", d));
            }
        }

        b
    }
}

pub fn mock_directory_exists(_dir: &str) -> bool { ::tsox_core::fntrace::enter("mock_directory_exists"); 
    false
}

pub struct MockWatchHandle {
    watch: Arc<Mutex<MockWatch>>,
}

impl WatchCloser for MockWatchHandle {
    fn close(&mut self) { ::tsox_core::fntrace::enter("close"); 
        self.watch.lock().unwrap().closed = true;
    }
}

pub fn path_is_under(
    event_path: &str,
    dir: &str,
    recursive: bool,
    use_case_sensitive_file_names: bool,
) -> bool { ::tsox_core::fntrace::enter("path_is_under"); 
    let mut event_path = event_path.to_string();
    let mut dir = dir.to_string();
    if !use_case_sensitive_file_names {
        event_path = tspath::get_canonical_file_name(&event_path, false);
        dir = tspath::get_canonical_file_name(&dir, false);
    }
    if !event_path.starts_with(&dir) {
        return false;
    }
    let rest = &event_path[dir.len()..];
    if rest.is_empty() {
        return false;
    }
    if !rest.starts_with('/') {
        return false;
    }
    if !recursive {
        return !rest[1..].contains('/');
    }
    true
}
