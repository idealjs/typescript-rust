use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::super::cachedvfs::CachedFS;
use super::super::fs::FS;
use super::super::types::{Entries, FileInfo};

pub struct TrackingFS {
    pub inner: Arc<dyn FS>,
    pub seen_files: Mutex<HashSet<String>>,
}

impl TrackingFS {
    pub fn new(inner: Arc<dyn FS>) -> Self {
        TrackingFS {
            inner,
            seen_files: Mutex::new(HashSet::new()),
        }
    }

    fn track(&self, path: &str) {
        self.seen_files.lock().unwrap().insert(path.to_string());
    }
}

impl FS for TrackingFS {
    fn read_file(&self, path: &str) -> Option<String> {
        self.track(path);
        self.inner.read_file(path)
    }

    fn file_exists(&self, path: &str) -> bool {
        self.track(path);
        self.inner.file_exists(path)
    }

    fn use_case_sensitive_file_names(&self) -> bool {
        self.inner.use_case_sensitive_file_names()
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        self.inner.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        self.inner.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        self.inner.remove(path)
    }

    fn chtimes(&self, path: &str, atime: SystemTime, mtime: SystemTime) -> std::io::Result<()> {
        self.inner.chtimes(path, atime, mtime)
    }

    fn directory_exists(&self, path: &str) -> bool {
        self.track(path);
        self.inner.directory_exists(path)
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        self.track(path);
        self.inner.get_accessible_entries(path)
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        self.track(path);
        self.inner.stat(path)
    }

    fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &FileInfo),
    ) -> std::io::Result<()> {
        self.track(root);
        self.inner.walk_dir(root, &mut |path, info| {
            self.track(path);
            walk_fn(path, info);
        })
    }

    fn realpath(&self, path: &str) -> String {
        self.track(path);
        self.inner.realpath(path)
    }
}

impl CachedFS {
    pub fn chtimes(
        &self,
        path: &str,
        atime: SystemTime,
        mtime: SystemTime,
    ) -> std::io::Result<()> {
        self.fs.chtimes(path, atime, mtime)
    }
}

pub fn root_length(p: &str) -> usize {
    let l = tsox_core::tspath::get_encoded_root_length(p);
    if l == 0 {
        panic!("vfs: path {p:?} is not absolute");
    } else if l < 0 {
        !l as usize
    } else {
        l as usize
    }
}

pub fn split_path(p: &str) -> (String, String) {
    let p = tsox_core::tspath::normalize_path(p);
    let l = root_length(&p);
    let (root_name, rest) = p.split_at(l);
    let rest = tsox_core::tspath::remove_trailing_directory_separator(rest);
    (root_name.to_string(), rest)
}

pub fn decode_bytes(s: &str) -> String {
    let b = s.as_bytes();
    if b.len() >= 2 {
        match [b[0], b[1]] {
            [0xFF, 0xFE] => return decode_utf16(&b[2..], true),
            [0xFE, 0xFF] => return decode_utf16(&b[2..], false),
            _ => {}
        }
    }
    if b.len() >= 3 && b[0] == 0xEF && b[1] == 0xBB && b[2] == 0xBF {
        return s[3..].to_string();
    }
    s.to_string()
}

pub fn decode_utf16(b: &[u8], little_endian: bool) -> String {
    let units: Vec<u16> = b
        .chunks_exact(2)
        .map(|c| {
            if little_endian {
                u16::from_le_bytes([c[0], c[1]])
            } else {
                u16::from_be_bytes([c[0], c[1]])
            }
        })
        .collect();
    String::from_utf16_lossy(&units)
}

pub struct FSMock {
    pub directory_exists_func: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    pub file_exists_func: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    pub get_accessible_entries_func: Option<Box<dyn Fn(&str) -> Entries + Send + Sync>>,
    pub read_file_func: Option<Box<dyn Fn(&str) -> Option<String> + Send + Sync>>,
    pub realpath_func: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
    pub remove_func: Option<Box<dyn Fn(&str) -> std::io::Result<()> + Send + Sync>>,
    pub chtimes_func:
        Option<Box<dyn Fn(&str, SystemTime, SystemTime) -> std::io::Result<()> + Send + Sync>>,
    pub stat_func: Option<Box<dyn Fn(&str) -> Option<FileInfo> + Send + Sync>>,
    pub use_case_sensitive_file_names_func: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    pub walk_dir_func: Option<
        Box<dyn Fn(&str, &mut dyn FnMut(&str, &FileInfo)) -> std::io::Result<()> + Send + Sync>,
    >,
    pub write_file_func: Option<Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>>,
    pub append_file_func: Option<Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>>,
}

pub fn wrap_mock(fs: Arc<dyn FS>) -> FSMock {
    let f_directory_exists = Arc::clone(&fs);
    let f_file_exists = Arc::clone(&fs);
    let f_get_accessible_entries = Arc::clone(&fs);
    let f_read_file = Arc::clone(&fs);
    let f_realpath = Arc::clone(&fs);
    let f_remove = Arc::clone(&fs);
    let f_stat = Arc::clone(&fs);
    let f_use_case_sensitive = Arc::clone(&fs);
    let f_write_file = Arc::clone(&fs);
    let f_append_file = Arc::clone(&fs);
    FSMock {
        directory_exists_func: Some(Box::new(move |p| f_directory_exists.directory_exists(p))),
        file_exists_func: Some(Box::new(move |p| f_file_exists.file_exists(p))),
        get_accessible_entries_func: Some(Box::new(move |p| {
            f_get_accessible_entries.get_accessible_entries(p)
        })),
        read_file_func: Some(Box::new(move |p| f_read_file.read_file(p))),
        realpath_func: Some(Box::new(move |p| f_realpath.realpath(p))),
        remove_func: Some(Box::new(move |p| f_remove.remove(p))),
        chtimes_func: None,
        stat_func: Some(Box::new(move |p| f_stat.stat(p))),
        use_case_sensitive_file_names_func: Some(Box::new(move || {
            f_use_case_sensitive.use_case_sensitive_file_names()
        })),
        walk_dir_func: None,
        write_file_func: Some(Box::new(move |p, d| f_write_file.write_file(p, d))),
        append_file_func: Some(Box::new(move |p, d| f_append_file.append_file(p, d))),
    }
}

pub struct Replacements {
    pub use_case_sensitive_file_names: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    pub file_exists: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    pub read_file: Option<Box<dyn Fn(&str) -> Option<String> + Send + Sync>>,
    pub write_file: Option<Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>>,
    pub append_file: Option<Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>>,
    pub remove: Option<Box<dyn Fn(&str) -> std::io::Result<()> + Send + Sync>>,
    pub chtimes: Option<
        Box<dyn Fn(&str, SystemTime, SystemTime) -> std::io::Result<()> + Send + Sync>,
    >,
    pub directory_exists: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    pub get_accessible_entries: Option<Box<dyn Fn(&str) -> Entries + Send + Sync>>,
    pub stat: Option<Box<dyn Fn(&str) -> Option<FileInfo> + Send + Sync>>,
    pub walk_dir: Option<
        Box<dyn Fn(&str, &mut dyn FnMut(&str, &FileInfo)) -> std::io::Result<()> + Send + Sync>,
    >,
    pub realpath: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
}

pub struct WrappedFS {
    pub fs: Arc<dyn FS>,
    pub replacements: Replacements,
}

pub fn wrap(fs: Arc<dyn FS>, replacements: Replacements) -> WrappedFS {
    WrappedFS { fs, replacements }
}

impl WrappedFS {
    pub fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        if let Some(f) = &self.replacements.append_file {
            return f(path, data);
        }
        self.fs.append_file(path, data)
    }
}
