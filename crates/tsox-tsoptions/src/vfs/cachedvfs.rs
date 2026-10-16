use super::{Entries, FS, FileInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct CachedFS {
    pub(crate) fs: Arc<dyn FS>,
    enabled: Mutex<bool>,
    directory_exists_cache: Mutex<HashMap<String, bool>>,
    file_exists_cache: Mutex<HashMap<String, bool>>,
    get_accessible_entries_cache: Mutex<HashMap<String, Entries>>,
    realpath_cache: Mutex<HashMap<String, String>>,
    stat_cache: Mutex<HashMap<String, Option<FileInfo>>>,
}

impl CachedFS {
    pub fn new(fs: Arc<dyn FS>) -> Self { ::tsox_core::fntrace::enter("new"); 
        CachedFS {
            fs,
            enabled: Mutex::new(true),
            directory_exists_cache: Mutex::new(HashMap::new()),
            file_exists_cache: Mutex::new(HashMap::new()),
            get_accessible_entries_cache: Mutex::new(HashMap::new()),
            realpath_cache: Mutex::new(HashMap::new()),
            stat_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn disable_and_clear_cache(&self) { ::tsox_core::fntrace::enter("disable_and_clear_cache"); 
        let mut enabled = self.enabled.lock().unwrap();
        if *enabled {
            *enabled = false;
            self.clear_caches();
        }
    }

    pub fn enable(&self) { ::tsox_core::fntrace::enter("enable"); 
        *self.enabled.lock().unwrap() = true;
    }

    pub fn clear_cache(&self) { ::tsox_core::fntrace::enter("clear_cache"); 
        self.clear_caches();
    }

    fn clear_caches(&self) { ::tsox_core::fntrace::enter("clear_caches"); 
        self.directory_exists_cache.lock().unwrap().clear();
        self.file_exists_cache.lock().unwrap().clear();
        self.get_accessible_entries_cache.lock().unwrap().clear();
        self.realpath_cache.lock().unwrap().clear();
        self.stat_cache.lock().unwrap().clear();
    }

    fn is_enabled(&self) -> bool { ::tsox_core::fntrace::enter("is_enabled"); 
        *self.enabled.lock().unwrap()
    }
}

impl FS for CachedFS {
    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        self.fs.use_case_sensitive_file_names()
    }

    fn file_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("file_exists"); 
        if self.is_enabled() {
            if let Some(&ret) = self.file_exists_cache.lock().unwrap().get(path) {
                return ret;
            }
        }
        let ret = self.fs.file_exists(path);
        if self.is_enabled() {
            self.file_exists_cache
                .lock()
                .unwrap()
                .insert(path.to_string(), ret);
        }
        ret
    }

    fn read_file(&self, path: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        self.fs.read_file(path)
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_file"); 
        self.fs.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("append_file"); 
        self.fs.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("remove"); 
        self.fs.remove(path)
    }

    fn directory_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("directory_exists"); 
        if self.is_enabled() {
            if let Some(&ret) = self.directory_exists_cache.lock().unwrap().get(path) {
                return ret;
            }
        }
        let ret = self.fs.directory_exists(path);
        if self.is_enabled() {
            self.directory_exists_cache
                .lock()
                .unwrap()
                .insert(path.to_string(), ret);
        }
        ret
    }

    fn get_accessible_entries(&self, path: &str) -> Entries { ::tsox_core::fntrace::enter("get_accessible_entries"); 
        if self.is_enabled() {
            if let Some(ret) = self.get_accessible_entries_cache.lock().unwrap().get(path) {
                return ret.clone();
            }
        }
        let ret = self.fs.get_accessible_entries(path);
        if self.is_enabled() {
            self.get_accessible_entries_cache
                .lock()
                .unwrap()
                .insert(path.to_string(), ret.clone());
        }
        ret
    }

    fn stat(&self, path: &str) -> Option<FileInfo> { ::tsox_core::fntrace::enter("stat"); 
        if self.is_enabled() {
            if let Some(ret) = self.stat_cache.lock().unwrap().get(path) {
                return ret.clone();
            }
        }
        let ret = self.fs.stat(path);
        if self.is_enabled() {
            self.stat_cache
                .lock()
                .unwrap()
                .insert(path.to_string(), ret.clone());
        }
        ret
    }

    fn realpath(&self, path: &str) -> String { ::tsox_core::fntrace::enter("realpath"); 
        if self.is_enabled() {
            if let Some(ret) = self.realpath_cache.lock().unwrap().get(path) {
                return ret.clone();
            }
        }
        let ret = self.fs.realpath(path);
        if self.is_enabled() {
            self.realpath_cache
                .lock()
                .unwrap()
                .insert(path.to_string(), ret.clone());
        }
        ret
    }

    fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &FileInfo),
    ) -> std::io::Result<()> { ::tsox_core::fntrace::enter("walk_dir"); 
        self.fs.walk_dir(root, walk_fn)
    }
}
