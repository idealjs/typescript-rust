#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::tspath::Path;
use crate::vfs::FS;

use super::overlay_fs::{DiskFile, FileHandle, Overlay};

pub trait FileSource: Send + Sync {
    fn fs(&self) -> &dyn FS;
    fn get_file(&self, file_name: &str) -> Option<Arc<dyn FileHandle>>;
    fn get_file_by_path(&self, file_name: &str, path: &Path) -> Option<Arc<dyn FileHandle>>;
    fn file_exists(&self, file_name: &str, path: &Path) -> bool;
    fn get_accessible_entries(&self, path: &str) -> crate::vfs::Entries;
}

pub struct SnapshotFS {
    pub fs: Arc<dyn FS>,
    pub overlays: HashMap<Path, Arc<Overlay>>,
    pub disk_files: HashMap<Path, Arc<DiskFile>>,
    pub to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
}

impl SnapshotFS {
    pub fn new(
        fs: Arc<dyn FS>,
        overlays: HashMap<Path, Arc<Overlay>>,
        to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
    ) -> Self {
        SnapshotFS {
            fs,
            overlays,
            disk_files: HashMap::new(),
            to_path,
        }
    }

    pub fn get_file(&self, file_name: &str) -> Option<Arc<dyn FileHandle>> {
        self.get_file_by_path(file_name, &(self.to_path)(file_name))
    }

    pub fn get_file_by_path(&self, file_name: &str, path: &Path) -> Option<Arc<dyn FileHandle>> {
        if let Some(overlay) = self.overlays.get(path) {
            return Some(Arc::clone(overlay) as Arc<dyn FileHandle>);
        }
        if let Some(disk) = self.disk_files.get(path) {
            return Some(Arc::clone(disk) as Arc<dyn FileHandle>);
        }
        match self.fs.read_file(file_name) {
            Some(content) => {
                let disk = Arc::new(DiskFile::new(file_name.to_string(), content));
                Some(disk as Arc<dyn FileHandle>)
            }
            None => None,
        }
    }

    pub fn file_exists(&self, file_name: &str, path: &Path) -> bool {
        if self.overlays.contains_key(path) {
            return true;
        }
        if self.disk_files.contains_key(path) {
            return true;
        }
        self.fs.file_exists(file_name)
    }

    pub fn is_open_file(&self, file_name: &str) -> bool {
        let path = (self.to_path)(file_name);
        self.overlays.contains_key(&path)
    }
}

impl FileSource for SnapshotFS {
    fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }
    fn get_file(&self, file_name: &str) -> Option<Arc<dyn FileHandle>> {
        SnapshotFS::get_file(self, file_name)
    }
    fn get_file_by_path(&self, file_name: &str, path: &Path) -> Option<Arc<dyn FileHandle>> {
        SnapshotFS::get_file_by_path(self, file_name, path)
    }
    fn file_exists(&self, file_name: &str, path: &Path) -> bool {
        SnapshotFS::file_exists(self, file_name, path)
    }
    fn get_accessible_entries(&self, path: &str) -> crate::vfs::Entries {
        self.fs.get_accessible_entries(path)
    }
}
