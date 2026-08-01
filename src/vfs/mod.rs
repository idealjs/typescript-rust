//! Virtual file system abstraction, ported from `internal/vfs/`.
//!
//! Provides a trait-based file system interface that can be backed by
//! the real OS file system or an in-memory implementation for testing.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// A file system entry listing.
#[derive(Clone, Debug, Default)]
pub struct Entries {
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub symlinks: Vec<String>,
}

/// File system metadata.
#[derive(Clone, Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub modified: std::time::SystemTime,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            name: String::new(),
            size: 0,
            is_dir: false,
            is_symlink: false,
            modified: std::time::SystemTime::UNIX_EPOCH,
        }
    }
}

/// The file system trait.
pub trait FS: Send + Sync {
    fn use_case_sensitive_file_names(&self) -> bool;
    fn file_exists(&self, path: &str) -> bool;
    fn read_file(&self, path: &str) -> Option<String>;
    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()>;
    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()>;
    fn remove(&self, path: &str) -> std::io::Result<()>;
    fn directory_exists(&self, path: &str) -> bool;
    fn get_accessible_entries(&self, path: &str) -> Entries;
    fn stat(&self, path: &str) -> Option<FileInfo>;
    fn realpath(&self, path: &str) -> String;
}

/// OS-backed file system implementation.
pub struct OsFS;

impl FS for OsFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        cfg!(not(target_os = "windows"))
    }

    fn file_exists(&self, path: &str) -> bool {
        Path::new(path).is_file()
    }

    fn read_file(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        std::fs::write(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(data.as_bytes())
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        if Path::new(path).is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }

    fn directory_exists(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        let mut entries = Entries::default();
        if let Ok(read_dir) = std::fs::read_dir(path) {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let file_type = entry.file_type();
                if file_type.as_ref().map(|t| t.is_dir()).unwrap_or(false) {
                    entries.directories.push(name);
                } else if file_type.as_ref().map(|t| t.is_symlink()).unwrap_or(false) {
                    entries.symlinks.push(name.clone());
                    // Use std::fs::metadata (not entry.metadata()) to follow symlinks.
                    // On macOS, DirEntry::metadata() does not follow symlinks.
                    if let Ok(meta) = std::fs::metadata(entry.path()) {
                        if meta.is_dir() {
                            entries.directories.push(name);
                        } else {
                            entries.files.push(name);
                        }
                    }
                } else {
                    entries.files.push(name);
                }
            }
        }
        entries.files.sort();
        entries.directories.sort();
        entries
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        let p = Path::new(path);
        let meta = std::fs::metadata(p).ok()?;
        let symlink_meta = std::fs::symlink_metadata(p).ok()?;
        Some(FileInfo {
            name: p.file_name()?.to_string_lossy().to_string(),
            size: meta.len(),
            is_dir: meta.is_dir(),
            is_symlink: symlink_meta.file_type().is_symlink(),
            modified: meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
        })
    }

    fn realpath(&self, path: &str) -> String {
        std::fs::canonicalize(path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string())
    }
}

/// In-memory file system for testing.
pub struct InMemoryFS {
    case_sensitive: bool,
    files: RwLock<HashMap<String, String>>,
    dirs: RwLock<std::collections::HashSet<String>>,
}

impl InMemoryFS {
    pub fn new() -> Self {
        Self::with_case_sensitivity(true)
    }

    pub fn with_case_sensitivity(case_sensitive: bool) -> Self {
        InMemoryFS {
            case_sensitive,
            files: RwLock::new(HashMap::new()),
            dirs: RwLock::new(std::collections::HashSet::new()),
        }
    }

    pub fn insert_file(&self, path: &str, content: &str) {
        self.files
            .write()
            .unwrap()
            .insert(path.to_string(), content.to_string());
    }

    pub fn insert_dir(&self, path: &str) {
        self.dirs.write().unwrap().insert(path.to_string());
    }
}

impl Default for InMemoryFS {
    fn default() -> Self {
        Self::new()
    }
}

impl FS for InMemoryFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.case_sensitive
    }

    fn file_exists(&self, path: &str) -> bool {
        self.files.read().unwrap().contains_key(path)
    }

    fn read_file(&self, path: &str) -> Option<String> {
        self.files.read().unwrap().get(path).cloned()
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        self.files
            .write()
            .unwrap()
            .insert(path.to_string(), data.to_string());
        Ok(())
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        let mut files = self.files.write().unwrap();
        let entry = files.entry(path.to_string()).or_default();
        entry.push_str(data);
        Ok(())
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        self.files.write().unwrap().remove(path);
        self.dirs.write().unwrap().remove(path);
        Ok(())
    }

    fn directory_exists(&self, path: &str) -> bool {
        self.dirs.read().unwrap().contains(path)
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        let mut entries = Entries::default();
        let prefix = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };

        for key in self.files.read().unwrap().keys() {
            if let Some(rest) = key.strip_prefix(&prefix) {
                if !rest.contains('/') {
                    entries.files.push(rest.to_string());
                }
            }
        }

        for dir in self.dirs.read().unwrap().iter() {
            if let Some(rest) = dir.strip_prefix(&prefix) {
                if !rest.contains('/') {
                    entries.directories.push(rest.to_string());
                }
            }
        }

        entries.files.sort();
        entries.directories.sort();
        entries
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        let files = self.files.read().unwrap();
        if let Some(content) = files.get(path) {
            return Some(FileInfo {
                name: path.rsplit('/').next()?.to_string(),
                size: content.len() as u64,
                is_dir: false,
                is_symlink: false,
                modified: std::time::SystemTime::now(),
            });
        }
        let dirs = self.dirs.read().unwrap();
        if dirs.contains(path) {
            return Some(FileInfo {
                name: path.rsplit('/').next()?.to_string(),
                size: 0,
                is_dir: true,
                is_symlink: false,
                modified: std::time::SystemTime::now(),
            });
        }
        None
    }

    fn realpath(&self, path: &str) -> String {
        path.to_string()
    }
}

/// A shared file system handle.
pub type SharedFS = Arc<dyn FS>;

#[cfg(test)]
mod cachedvfs_tests;
#[cfg(test)]
mod iovfs_tests;
#[cfg(test)]
mod osvfs_tests;
#[cfg(test)]
mod vfsmatch_tests;
#[cfg(test)]
mod vfsmock_tests;
#[cfg(test)]
mod vfstest_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_fs_basic() {
        let fs = InMemoryFS::new();
        fs.insert_file("/test.txt", "hello");
        assert!(fs.file_exists("/test.txt"));
        assert_eq!(fs.read_file("/test.txt"), Some("hello".to_string()));
        assert!(!fs.file_exists("/missing.txt"));
    }

    #[test]
    fn in_memory_fs_dirs() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        assert!(fs.directory_exists("/src"));
        fs.insert_file("/src/a.ts", "export {}");
        let entries = fs.get_accessible_entries("/src");
        assert_eq!(entries.files, vec!["a.ts"]);
    }

    #[test]
    fn in_memory_fs_append() {
        let fs = InMemoryFS::new();
        fs.insert_file("/log.txt", "line1\n");
        fs.append_file("/log.txt", "line2\n").unwrap();
        assert_eq!(fs.read_file("/log.txt"), Some("line1\nline2\n".to_string()));
    }

    #[test]
    fn in_memory_fs_write_overwrites() {
        let fs = InMemoryFS::new();
        fs.write_file("/foo.txt", "hello").unwrap();
        assert_eq!(fs.read_file("/foo.txt"), Some("hello".to_string()));
        fs.write_file("/foo.txt", "goodbye").unwrap();
        assert_eq!(fs.read_file("/foo.txt"), Some("goodbye".to_string()));
    }

    #[test]
    fn in_memory_fs_remove_file() {
        let fs = InMemoryFS::new();
        fs.insert_file("/foo/bar/file.ts", "remove");
        assert!(fs.file_exists("/foo/bar/file.ts"));
        fs.remove("/foo/bar/file.ts").unwrap();
        assert!(!fs.file_exists("/foo/bar/file.ts"));
    }

    #[test]
    fn in_memory_fs_remove_dir() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/foo/bar/test");
        assert!(fs.directory_exists("/foo/bar/test"));
        fs.remove("/foo/bar/test").unwrap();
        assert!(!fs.directory_exists("/foo/bar/test"));
    }

    #[test]
    fn in_memory_fs_remove_nonexistent() {
        let fs = InMemoryFS::new();
        // Should not error when removing nonexistent paths
        assert!(fs.remove("/nonexistent").is_ok());
        assert!(fs.remove("/nonexistent/file.ts").is_ok());
    }

    #[test]
    fn in_memory_fs_stat_file() {
        let fs = InMemoryFS::new();
        fs.insert_file("/test.ts", "export const x = 1;");
        let info = fs.stat("/test.ts").unwrap();
        assert!(!info.is_dir);
        assert!(!info.is_symlink);
        assert_eq!(info.size, 19); // "export const x = 1;" is 19 bytes
    }

    #[test]
    fn in_memory_fs_stat_dir() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        let info = fs.stat("/src").unwrap();
        assert!(info.is_dir);
        assert!(!info.is_symlink);
    }

    #[test]
    fn in_memory_fs_stat_nonexistent() {
        let fs = InMemoryFS::new();
        assert!(fs.stat("/missing").is_none());
    }

    #[test]
    fn in_memory_fs_realpath() {
        let fs = InMemoryFS::new();
        fs.insert_file("/foo.ts", "hello");
        assert_eq!(fs.realpath("/foo.ts"), "/foo.ts");
        assert_eq!(fs.realpath("/missing.ts"), "/missing.ts");
    }

    #[test]
    fn in_memory_fs_accessible_entries_multiple() {
        let fs = InMemoryFS::new();
        fs.insert_file("/src/a.ts", "a");
        fs.insert_file("/src/b.ts", "b");
        fs.insert_file("/src/sub/c.ts", "c");
        fs.insert_dir("/src/sub");
        let entries = fs.get_accessible_entries("/src");
        assert_eq!(entries.files, vec!["a.ts", "b.ts"]);
        assert_eq!(entries.directories, vec!["sub"]);
    }

    #[test]
    fn in_memory_fs_accessible_entries_empty() {
        let fs = InMemoryFS::new();
        let entries = fs.get_accessible_entries("/empty");
        assert!(entries.files.is_empty());
        assert!(entries.directories.is_empty());
    }

    #[test]
    fn in_memory_fs_case_sensitive() {
        let fs = InMemoryFS::with_case_sensitivity(true);
        assert!(fs.use_case_sensitive_file_names());
        fs.insert_file("/foo.ts", "hello");
        assert!(fs.file_exists("/foo.ts"));
        assert!(!fs.file_exists("/Foo.ts"));
    }

    #[test]
    fn in_memory_fs_case_insensitive_read() {
        let fs = InMemoryFS::with_case_sensitivity(false);
        assert!(!fs.use_case_sensitive_file_names());
        // Note: our simple InMemoryFS doesn't do case-insensitive lookup,
        // but the flag is stored correctly
        fs.insert_file("/foo.ts", "hello");
        assert!(fs.file_exists("/foo.ts"));
    }

    #[test]
    fn in_memory_fs_trailing_slash_dir() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/src/");
        // get_accessible_entries should handle trailing slash
        fs.insert_file("/src/a.ts", "a");
        let entries = fs.get_accessible_entries("/src/");
        assert_eq!(entries.files, vec!["a.ts"]);
    }

    #[test]
    fn os_fs_basic_exists() {
        let fs = OsFS;
        // Test that OsFS can check file existence
        assert!(!fs.file_exists("/nonexistent_file_12345.ts"));
    }

    #[test]
    fn os_fs_directory_exists() {
        let fs = OsFS;
        assert!(!fs.directory_exists("/nonexistent_dir_12345"));
    }

    #[test]
    fn os_fs_use_case_sensitive() {
        let fs = OsFS;
        // On Linux/Mac, case sensitive; on Windows, not
        #[cfg(target_os = "windows")]
        assert!(!fs.use_case_sensitive_file_names());
        #[cfg(not(target_os = "windows"))]
        assert!(fs.use_case_sensitive_file_names());
    }
}
