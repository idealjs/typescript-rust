#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use super::m5f::{new_debounce, Debounce};
use super::m5f_2::FswatchError;
use super::m5g_2::{
    is_in_directory_or_self, physical_dir_for, SequenceFn, WatcherBackend, ERR_UNAVAILABLE,
};
use super::m5g_3::{new_dir_watch, DirWatch};
use crate::tspath::get_directory_path;

pub const RECURSIVE_CONSOLIDATE_THRESHOLD: usize = 10;

pub type WatchImplFactory = Arc<dyn Fn() -> Arc<dyn WatcherBackend> + Send + Sync>;

struct WatcherMu {
    backend: Option<Arc<dyn WatcherBackend>>,
    dir_watches: HashMap<String, Arc<DirWatch>>,
    debounce: Option<Arc<Debounce>>,
}

pub struct WatcherImpl {
    pub name: String,
    pub mu: Mutex<WatcherMu>,
    pub factory: Option<WatchImplFactory>,
    pub sequence: Option<SequenceFn>,
}

impl WatcherImpl {
    pub fn new(name: &str) -> WatcherImpl {
        WatcherImpl {
            name: name.to_string(),
            mu: Mutex::new(WatcherMu {
                backend: None,
                dir_watches: HashMap::new(),
                debounce: None,
            }),
            factory: None,
            sequence: None,
        }
    }

    pub fn can_share_recursive_dir_watches(&self) -> bool {
        self.name == "fsevents"
    }

    pub fn get_impl(&self) -> Result<Arc<dyn WatcherBackend>, FswatchError> {
        {
            let mu = self.mu.lock().unwrap();
            if let Some(backend) = &mu.backend {
                return Ok(Arc::clone(backend));
            }
        }
        let factory = match &self.factory {
            Some(factory) => Arc::clone(factory),
            None => return Err(ERR_UNAVAILABLE.to_string()),
        };
        let backend = factory();
        backend.run()?;
        let mut mu = self.mu.lock().unwrap();
        if let Some(existing) = &mu.backend {
            let existing = Arc::clone(existing);
            drop(mu);
            backend.shutdown();
            return Ok(existing);
        }
        mu.backend = Some(Arc::clone(&backend));
        drop(mu);
        Ok(backend)
    }

    pub fn key_for_dir_watch(&self, dir: &str, recursive: bool) -> String {
        if recursive {
            format!("{}\x00recursive", dir)
        } else {
            dir.to_string()
        }
    }

    fn find_covering_recursive_watch_locked(
        &self,
        mu: &WatcherMu,
        dir: &str,
        physical_dir: &str,
    ) -> Option<Arc<DirWatch>> {
        let mut best: Option<Arc<DirWatch>> = None;
        for dw in mu.dir_watches.values() {
            if !dw.recursive_value()
                || !is_in_directory_or_self(&dw.dir, dir)
                || !is_in_directory_or_self(&dw.physical_dir, physical_dir)
            {
                continue;
            }
            let better = match &best {
                None => true,
                Some(b) => dw.dir.len() > b.dir.len(),
            };
            if better {
                best = Some(Arc::clone(dw));
            }
        }
        best
    }

    fn find_consolidation_dir_locked(
        &self,
        mu: &WatcherMu,
        dir: &str,
        physical_dir: &str,
    ) -> String {
        if !self.can_share_recursive_dir_watches() {
            return String::new();
        }
        let mut dir = dir.to_string();
        let mut parent = get_directory_path(&dir);
        while parent != dir && parent != "." {
            if get_directory_path(&parent) == parent {
                break;
            }
            let physical_parent = physical_dir_for(&parent);
            if !is_in_directory_or_self(&physical_parent, physical_dir) {
                return String::new();
            }
            let mut count = 1;
            for dw in mu.dir_watches.values() {
                if is_in_directory_or_self(&parent, &dw.dir)
                    && is_in_directory_or_self(&physical_parent, &dw.physical_dir)
                {
                    count += 1;
                    if count >= RECURSIVE_CONSOLIDATE_THRESHOLD {
                        return parent;
                    }
                }
            }
            let next = get_directory_path(&parent);
            if next == parent {
                break;
            }
            dir = parent;
            parent = next;
        }
        String::new()
    }

    pub fn get_or_create_dir_watch(
        &self,
        dir: &str,
        physical_dir: &str,
        recursive: bool,
    ) -> Arc<DirWatch> {
        let mut mu = self.mu.lock().unwrap();
        if mu.debounce.is_none() {
            mu.debounce = Some(new_debounce());
        }
        let mut dir = dir.to_string();
        let mut physical_dir = physical_dir.to_string();
        let mut recursive = recursive;
        if self.can_share_recursive_dir_watches() {
            if let Some(dw) = self.find_covering_recursive_watch_locked(&mu, &dir, &physical_dir) {
                return dw;
            }
            let consolidation_dir = self.find_consolidation_dir_locked(&mu, &dir, &physical_dir);
            if !consolidation_dir.is_empty() {
                dir = consolidation_dir;
                physical_dir = physical_dir_for(&dir);
                recursive = true;
                if let Some(dw) =
                    self.find_covering_recursive_watch_locked(&mu, &dir, &physical_dir)
                {
                    return dw;
                }
            }
        }
        let key = self.key_for_dir_watch(&dir, recursive);
        if let Some(dw) = mu.dir_watches.get(&key) {
            return Arc::clone(dw);
        }
        let db = Arc::clone(mu.debounce.as_ref().unwrap());
        let dw = new_dir_watch(&dir, &physical_dir, recursive, &db);
        dw.set_sequence(self.sequence.clone());
        mu.dir_watches.insert(key, Arc::clone(&dw));
        dw
    }

    pub fn remove_dir_watch(&self, dw: &Arc<DirWatch>) {
        let mut mu = self.mu.lock().unwrap();
        let key = self.key_for_dir_watch(&dw.dir, dw.recursive_value());
        if let Some(existing) = mu.dir_watches.get(&key) {
            if Arc::ptr_eq(existing, dw) {
                mu.dir_watches.remove(&key);
                dw.destroy_debounce();
            }
        }
    }

    pub fn rollback_prepared(&self, prepared: &[PreparedWatch]) {
        for p in prepared.iter().rev() {
            p.dw.unwatch(p.id);
            p.dw.unref(self);
        }
    }
}

impl fmt::Display for WatcherImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

pub struct PreparedWatch {
    pub dw: Arc<DirWatch>,
    pub id: u64,
    pub recursive: bool,
    pub dir: String,
}
