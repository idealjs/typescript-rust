use crate::vfs::fs::FS;
use crate::vfs::types::{Entries, FileInfo, SharedFS};
use std::sync::Arc;
use std::time::SystemTime;

pub type ChtimesFn =
    Box<dyn Fn(&str, SystemTime, SystemTime) -> std::io::Result<()> + Send + Sync>;
pub type FileExistsFn = Box<dyn Fn(&str) -> bool + Send + Sync>;
pub type ReadFileFn = Box<dyn Fn(&str) -> Option<String> + Send + Sync>;
pub type WriteFileFn =
    Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>;
pub type AppendFileFn = Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>;
pub type RemoveFn = Box<dyn Fn(&str) -> std::io::Result<()> + Send + Sync>;
pub type UseCaseSensitiveFileNamesFn = Box<dyn Fn() -> bool + Send + Sync>;
pub type DirectoryExistsFn = Box<dyn Fn(&str) -> bool + Send + Sync>;
pub type GetAccessibleEntriesFn = Box<dyn Fn(&str) -> Entries + Send + Sync>;
pub type StatFn = Box<dyn Fn(&str) -> Option<FileInfo> + Send + Sync>;
pub type WalkDirFn<'a> = &'a mut dyn FnMut(&str, &FileInfo);
pub type WalkDirReplacement =
    Box<dyn Fn(&str, WalkDirFn<'_>) -> std::io::Result<()> + Send + Sync>;
pub type RealpathFn = Box<dyn Fn(&str) -> String + Send + Sync>;

pub struct Replacements {
    pub use_case_sensitive_file_names: Option<UseCaseSensitiveFileNamesFn>,
    pub file_exists: Option<FileExistsFn>,
    pub read_file: Option<ReadFileFn>,
    pub write_file: Option<WriteFileFn>,
    pub append_file: Option<AppendFileFn>,
    pub remove: Option<RemoveFn>,
    pub chtimes: Option<ChtimesFn>,
    pub directory_exists: Option<DirectoryExistsFn>,
    pub get_accessible_entries: Option<GetAccessibleEntriesFn>,
    pub stat: Option<StatFn>,
    pub walk_dir: Option<WalkDirReplacement>,
    pub realpath: Option<RealpathFn>,
}

impl Default for Replacements {
    fn default() -> Self {
        Replacements {
            use_case_sensitive_file_names: None,
            file_exists: None,
            read_file: None,
            write_file: None,
            append_file: None,
            remove: None,
            chtimes: None,
            directory_exists: None,
            get_accessible_entries: None,
            stat: None,
            walk_dir: None,
            realpath: None,
        }
    }
}

pub fn wrap(fs: SharedFS, replacements: Replacements) -> SharedFS {
    Arc::new(WrappedFS {
        fs,
        replacements,
    })
}

pub struct WrappedFS {
    fs: SharedFS,
    replacements: Replacements,
}

impl FS for WrappedFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        if let Some(replacement) = &self.replacements.use_case_sensitive_file_names {
            return replacement();
        }
        self.fs.use_case_sensitive_file_names()
    }

    fn file_exists(&self, path: &str) -> bool {
        if let Some(replacement) = &self.replacements.file_exists {
            return replacement(path);
        }
        self.fs.file_exists(path)
    }

    fn read_file(&self, path: &str) -> Option<String> {
        if let Some(replacement) = &self.replacements.read_file {
            return replacement(path);
        }
        self.fs.read_file(path)
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        if let Some(replacement) = &self.replacements.write_file {
            return replacement(path, data);
        }
        self.fs.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        if let Some(replacement) = &self.replacements.append_file {
            return replacement(path, data);
        }
        self.fs.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        if let Some(replacement) = &self.replacements.remove {
            return replacement(path);
        }
        self.fs.remove(path)
    }

    fn directory_exists(&self, path: &str) -> bool {
        if let Some(replacement) = &self.replacements.directory_exists {
            return replacement(path);
        }
        self.fs.directory_exists(path)
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        if let Some(replacement) = &self.replacements.get_accessible_entries {
            return replacement(path);
        }
        self.fs.get_accessible_entries(path)
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        if let Some(replacement) = &self.replacements.stat {
            return replacement(path);
        }
        self.fs.stat(path)
    }

    fn realpath(&self, path: &str) -> String {
        if let Some(replacement) = &self.replacements.realpath {
            return replacement(path);
        }
        self.fs.realpath(path)
    }

    fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &FileInfo),
    ) -> std::io::Result<()> {
        if let Some(replacement) = &self.replacements.walk_dir {
            return replacement(root, walk_fn);
        }
        self.fs.walk_dir(root, walk_fn)
    }
}

impl WrappedFS {
    pub fn chtimes(
        &self,
        path: &str,
        a_time: SystemTime,
        m_time: SystemTime,
    ) -> std::io::Result<()> {
        if let Some(replacement) = &self.replacements.chtimes {
            return replacement(path, a_time, m_time);
        }
        self.fs.chtimes(path, a_time, m_time)
    }
}
