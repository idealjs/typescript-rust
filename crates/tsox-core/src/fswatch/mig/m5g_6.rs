#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::{Arc, Mutex};

use super::m5f::canonicalize_path;
use super::m5f_2::FswatchError;
use super::m5g_2::{
    file_callback, physical_dir_for, validate_watch_directory, Watch, WatchCallback,
    WatchDirectoryRequest, WatchOption, WatchOptions, Watcher, WatcherBackend, ERR_NIL_CALLBACK,
    ERR_NOT_ABSOLUTE, ERR_ROOT_PATH, ERR_UNAVAILABLE,
};
use super::m5g_3::DirWatch;
use super::m5g_5::{PreparedWatch, WatcherImpl};
use crate::tspath::{get_directory_path, normalize_path, path_is_absolute};

pub struct WatchImpl {
    mu: Mutex<bool>,
    w: *const WatcherImpl,
    dw: Arc<DirWatch>,
    backend: Arc<dyn WatcherBackend>,
    id: u64,
}

unsafe impl Send for WatchImpl {}
unsafe impl Sync for WatchImpl {}

impl Watch for WatchImpl {
    fn close(&self) -> Result<(), FswatchError> { crate::fntrace::enter("close"); 
        let mut cancelled = self.mu.lock().unwrap();
        if *cancelled {
            return Ok(());
        }
        *cancelled = true;
        drop(cancelled);
        let last = self.dw.unwatch(self.id);
        if last {
            self.backend.watch_remove(&self.dw);
            let watcher = unsafe { &*self.w };
            DirWatch::unref(&self.dw, watcher);
        }
        Ok(())
    }
}

impl Watcher for WatcherImpl {
    fn name(&self) -> &str { crate::fntrace::enter("name"); 
        &self.name
    }

    fn available(&self) -> bool { crate::fntrace::enter("available"); 
        self.factory.is_some()
    }

    fn has_fast_recursive_backend(&self) -> bool { crate::fntrace::enter("has_fast_recursive_backend"); 
        matches!(self.name.as_str(), "windows" | "fsevents")
    }

    fn watch_directory(
        &self,
        dir: &str,
        callback: WatchCallback,
        options: &[WatchOption],
    ) -> Result<Arc<dyn Watch>, FswatchError> { crate::fntrace::enter("watch_directory"); 
        let requests = [WatchDirectoryRequest {
            dir: dir.to_string(),
            callback: Some(callback),
            options: options.to_vec(),
        }];
        let watches = self.watch_directories(&requests)?;
        Ok(watches.into_iter().next().unwrap())
    }

    fn watch_directories(
        &self,
        requests: &[WatchDirectoryRequest],
    ) -> Result<Vec<Arc<dyn Watch>>, FswatchError> { crate::fntrace::enter("watch_directories"); 
        if !self.available() {
            return Err(ERR_UNAVAILABLE.to_string());
        }
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        let mut prepared: Vec<PreparedWatch> = Vec::with_capacity(requests.len());
        let mut unique_dir_watches: Vec<Arc<DirWatch>> = Vec::with_capacity(requests.len());
        let mut seen: Vec<usize> = Vec::with_capacity(requests.len());
        for request in requests {
            let fn_ = match &request.callback {
                Some(fn_) => Arc::clone(fn_),
                None => {
                    self.rollback_prepared(&prepared);
                    return Err(ERR_NIL_CALLBACK.to_string());
                }
            };
            let mut dir = normalize_path(&request.dir);
            if !path_is_absolute(&dir) {
                self.rollback_prepared(&prepared);
                return Err(ERR_NOT_ABSOLUTE.to_string());
            }
            dir = canonicalize_path(&dir);
            if self.can_share_recursive_dir_watches() {
                if let Err(err) = validate_watch_directory(&dir) {
                    self.rollback_prepared(&prepared);
                    return Err(err);
                }
            }
            let physical_dir = physical_dir_for(&dir);
            let mut sopts = WatchOptions::default();
            for o in &request.options {
                o.apply_watch_option(&mut sopts);
            }
            let dw = self.get_or_create_dir_watch(&dir, &physical_dir, sopts.recursive);
            let id = dw.watch(&dir, &physical_dir, sopts.recursive, fn_, sopts.ignore);
            let addr = Arc::as_ptr(&dw) as usize;
            if !seen.contains(&addr) {
                seen.push(addr);
                unique_dir_watches.push(Arc::clone(&dw));
            }
            prepared.push(PreparedWatch {
                dw,
                id,
                recursive: sopts.recursive,
                dir,
            });
        }
        let backend = match self.get_impl() {
            Ok(backend) => backend,
            Err(err) => {
                self.rollback_prepared(&prepared);
                return Err(err);
            }
        };
        if let Err(err) = backend.watch_add_many(&unique_dir_watches) {
            self.rollback_prepared(&prepared);
            return Err(err);
        }
        let watches: Vec<Arc<dyn Watch>> = prepared
            .into_iter()
            .map(|p| {
                Arc::new(WatchImpl {
                    mu: Mutex::new(false),
                    w: self as *const WatcherImpl,
                    dw: p.dw,
                    backend: Arc::clone(&backend),
                    id: p.id,
                }) as Arc<dyn Watch>
            })
            .collect();
        Ok(watches)
    }

    fn watch_file(
        &self,
        path: &str,
        callback: WatchCallback,
    ) -> Result<Arc<dyn Watch>, FswatchError> { crate::fntrace::enter("watch_file"); 
        if !self.available() {
            return Err(ERR_UNAVAILABLE.to_string());
        }
        let mut path = normalize_path(path);
        if !path_is_absolute(&path) {
            return Err(ERR_NOT_ABSOLUTE.to_string());
        }
        path = canonicalize_path(&path);
        let dir = get_directory_path(&path);
        if dir == path {
            return Err(ERR_ROOT_PATH.to_string());
        }
        self.watch_directory(&dir, file_callback(&path, callback), &[])
    }
}
