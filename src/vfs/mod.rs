//! Virtual file system abstraction, ported from `internal/vfs/`.
//!
//! Provides a trait-based file system interface that can be backed by
//! the real OS file system or an in-memory implementation for testing.

use std::collections::{HashMap, HashSet};
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

    /// Walk the file tree rooted at `root`, invoking `walk_fn` once for each
    /// file or directory entry. Mirrors Go's `WalkDir`.
    ///
    /// The default implementation is a no-op; concrete implementations may
    /// override it. The callback receives `(path, info)` for every entry.
    fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &FileInfo),
    ) -> std::io::Result<()> {
        let _ = (root, walk_fn);
        Ok(())
    }
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
    /// Maps a link path to its target path. Targets may be absolute or
    /// relative (resolved against the link's parent directory).
    symlinks: RwLock<HashMap<String, String>>,
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
            symlinks: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_file(&self, path: &str, content: &str) {
        let mut files = self.files.write().unwrap();
        let key = if self.case_sensitive {
            path.to_string()
        } else {
            let target = path.to_ascii_lowercase();
            files
                .keys()
                .find(|k| k.to_ascii_lowercase() == target)
                .cloned()
                .unwrap_or_else(|| path.to_string())
        };
        files.insert(key, content.to_string());
    }

    pub fn insert_dir(&self, path: &str) {
        let mut dirs = self.dirs.write().unwrap();
        if !self.case_sensitive {
            let target = path.to_ascii_lowercase();
            if let Some(existing) = dirs
                .iter()
                .find(|d| d.to_ascii_lowercase() == target)
                .cloned()
            {
                dirs.remove(&existing);
            }
        }
        dirs.insert(path.to_string());
    }

    /// Finds the stored file key matching `path`, performing a case-insensitive
    /// match when the FS is not case-sensitive.
    fn lookup_file_key(&self, path: &str) -> Option<String> {
        let files = self.files.read().unwrap();
        if files.contains_key(path) {
            return Some(path.to_string());
        }
        if self.case_sensitive {
            return None;
        }
        let target = path.to_ascii_lowercase();
        files
            .keys()
            .find(|k| k.to_ascii_lowercase() == target)
            .cloned()
    }

    /// Finds the stored directory key matching `path`, performing a
    /// case-insensitive match when the FS is not case-sensitive.
    fn lookup_dir_key(&self, path: &str) -> Option<String> {
        let dirs = self.dirs.read().unwrap();
        if dirs.contains(path) {
            return Some(path.to_string());
        }
        if self.case_sensitive {
            return None;
        }
        let target = path.to_ascii_lowercase();
        dirs.iter()
            .find(|d| d.to_ascii_lowercase() == target)
            .cloned()
    }

    /// Creates a symlink: `link` resolves to `target`.
    pub fn create_symlink(&self, link: &str, target: &str) {
        let mut symlinks = self.symlinks.write().unwrap();
        let key = if self.case_sensitive {
            link.to_string()
        } else {
            let target_lc = link.to_ascii_lowercase();
            symlinks
                .keys()
                .find(|k| k.to_ascii_lowercase() == target_lc)
                .cloned()
                .unwrap_or_else(|| link.to_string())
        };
        symlinks.insert(key, target.to_string());
    }

    /// Reads the target of a symlink. Returns `None` if `path` is not a
    /// symlink.
    pub fn read_symlink(&self, path: &str) -> Option<String> {
        self.lookup_symlink_key(path)
    }

    /// Finds the stored symlink target for `link`, performing a
    /// case-insensitive match when the FS is not case-sensitive.
    fn lookup_symlink_key(&self, link: &str) -> Option<String> {
        let symlinks = self.symlinks.read().unwrap();
        if let Some(t) = symlinks.get(link) {
            return Some(t.clone());
        }
        if self.case_sensitive {
            return None;
        }
        let target = link.to_ascii_lowercase();
        symlinks
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == target)
            .map(|(_, v)| v.clone())
    }

    /// Finds the stored symlink *key* (the link path) matching `link`,
    /// performing a case-insensitive match when the FS is not case-sensitive.
    fn lookup_symlink_stored_key(&self, link: &str) -> Option<String> {
        let symlinks = self.symlinks.read().unwrap();
        if symlinks.contains_key(link) {
            return Some(link.to_string());
        }
        if self.case_sensitive {
            return None;
        }
        let target = link.to_ascii_lowercase();
        symlinks
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == target)
            .map(|(k, _)| k.clone())
    }

    /// Returns `true` if `path` (or its case-insensitive equivalent) is a
    /// stored file.
    fn is_file_path(&self, path: &str) -> bool {
        let files = self.files.read().unwrap();
        if files.contains_key(path) {
            return true;
        }
        if self.case_sensitive {
            return false;
        }
        let target = path.to_ascii_lowercase();
        files.keys().any(|k| k.to_ascii_lowercase() == target)
    }

    /// Resolves `path` by following symlinks at any path component.
    ///
    /// Symlink chains are followed (with a hop limit) and cycles are broken
    /// (the last-resolvable path is returned). With no symlinks present this
    /// is a cheap no-op returning the input verbatim.
    fn resolve_symlinks(&self, path: &str) -> String {
        if path.is_empty() {
            return String::new();
        }
        let symlinks = self.symlinks.read().unwrap();
        if symlinks.is_empty() {
            return path.to_string();
        }
        let is_absolute = path.starts_with('/');
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut resolved = if is_absolute {
            String::from("/")
        } else {
            String::new()
        };
        let mut visited: HashSet<String> = HashSet::new();
        for part in &parts {
            // Append the next component to the running resolved path.
            if resolved.is_empty() {
                resolved.push_str(part);
            } else if resolved.ends_with('/') {
                resolved.push_str(part);
            } else {
                resolved.push('/');
                resolved.push_str(part);
            }
            // Follow the symlink chain rooted at this prefix.
            let mut hops = 0;
            loop {
                hops += 1;
                if hops > MAX_SYMLINK_HOPS {
                    break;
                }
                let target = match self.symlink_target(&symlinks, &resolved) {
                    Some(t) => t,
                    None => break,
                };
                resolved = if is_absolute_path(&target) {
                    target
                } else {
                    // Relative target: resolve against the link's parent dir.
                    match parent_dir(&resolved) {
                        Some(p) if p.ends_with('/') => format!("{p}{target}"),
                        Some(p) => format!("{p}/{target}"),
                        None => target,
                    }
                };
                if !visited.insert(resolved.clone()) {
                    break; // cycle detected
                }
            }
        }
        resolved
    }

    /// Looks up the symlink target for `path` within a borrowed symlink map.
    fn symlink_target<'a>(
        &self,
        symlinks: &'a HashMap<String, String>,
        path: &str,
    ) -> Option<String> {
        if let Some(t) = symlinks.get(path) {
            return Some(t.clone());
        }
        if self.case_sensitive {
            return None;
        }
        let target = path.to_ascii_lowercase();
        symlinks
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == target)
            .map(|(_, v)| v.clone())
    }
}

impl Default for InMemoryFS {
    fn default() -> Self {
        Self::new()
    }
}

/// Maximum number of symlink hops before giving up (cycle/loop protection).
const MAX_SYMLINK_HOPS: usize = 40;

/// Returns the parent directory of `path`, or `None` if `path` has no parent.
fn parent_dir(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(0) => Some(String::from("/")),
        Some(i) => Some(trimmed[..i].to_string()),
        None => None,
    }
}

/// Returns `true` for absolute paths (POSIX `/…` or Windows drive `c:/…`).
fn is_absolute_path(path: &str) -> bool {
    path.starts_with('/')
        || (path.len() >= 3 && path.as_bytes()[1] == b':' && path.as_bytes()[2] == b'/')
}

/// Strips a UTF-8 BOM (`U+FEFF`) from the start of `content` if present.
fn decode_with_bom(content: &str) -> String {
    content
        .strip_prefix('\u{FEFF}')
        .map(|s| s.to_string())
        .unwrap_or_else(|| content.to_string())
}

/// Returns the remainder of `haystack` after `prefix`, comparing
/// case-insensitively when `case_sensitive` is false. Preserves the original
/// casing of the remainder.
fn strip_path_prefix<'a>(haystack: &'a str, prefix: &str, case_sensitive: bool) -> Option<&'a str> {
    if case_sensitive {
        haystack.strip_prefix(prefix)
    } else {
        let h = haystack.as_bytes();
        let p = prefix.as_bytes();
        if h.len() >= p.len() && h[..p.len()].eq_ignore_ascii_case(p) {
            Some(&haystack[p.len()..])
        } else {
            None
        }
    }
}

impl FS for InMemoryFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.case_sensitive
    }

    fn file_exists(&self, path: &str) -> bool {
        let resolved = self.resolve_symlinks(path);
        self.lookup_file_key(&resolved).is_some()
    }

    fn read_file(&self, path: &str) -> Option<String> {
        let resolved = self.resolve_symlinks(path);
        let files = self.files.read().unwrap();
        let content = if let Some(c) = files.get(&resolved) {
            c
        } else if !self.case_sensitive {
            let target = resolved.to_ascii_lowercase();
            files
                .iter()
                .find(|(k, _)| k.to_ascii_lowercase() == target)
                .map(|(_, v)| v.as_str())?
        } else {
            return None;
        };
        Some(decode_with_bom(content))
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        let resolved = self.resolve_symlinks(path);
        // Validate that the parent path is not an existing file.
        if let Some(parent) = parent_dir(&resolved) {
            if self.is_file_path(&parent) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("mkdir {parent:?}: path exists but is not a directory",),
                ));
            }
        }
        let mut files = self.files.write().unwrap();
        let key = if self.case_sensitive {
            resolved.clone()
        } else {
            let target = resolved.to_ascii_lowercase();
            files
                .keys()
                .find(|k| k.to_ascii_lowercase() == target)
                .cloned()
                .unwrap_or_else(|| resolved.clone())
        };
        files.insert(key, data.to_string());
        Ok(())
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        let resolved = self.resolve_symlinks(path);
        let mut files = self.files.write().unwrap();
        let key = if self.case_sensitive {
            resolved.clone()
        } else {
            let target = resolved.to_ascii_lowercase();
            files
                .keys()
                .find(|k| k.to_ascii_lowercase() == target)
                .cloned()
                .unwrap_or_else(|| resolved.clone())
        };
        let entry = files.entry(key).or_default();
        entry.push_str(data);
        Ok(())
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        // A symlink is removed on its own (the target is untouched).
        if let Some(key) = self.lookup_symlink_stored_key(path) {
            self.symlinks.write().unwrap().remove(&key);
            return Ok(());
        }
        // An exact file match is removed on its own.
        if let Some(key) = self.lookup_file_key(path) {
            self.files.write().unwrap().remove(&key);
            return Ok(());
        }
        // A directory is removed recursively (all descendants are cleared).
        if let Some(key) = self.lookup_dir_key(path) {
            let prefix = format!("{key}/");
            let mut files = self.files.write().unwrap();
            let mut dirs = self.dirs.write().unwrap();
            let mut symlinks = self.symlinks.write().unwrap();
            dirs.remove(&key);
            let desc_files: Vec<String> = files
                .keys()
                .filter(|p| p.starts_with(&prefix))
                .cloned()
                .collect();
            for p in desc_files {
                files.remove(&p);
            }
            let desc_dirs: Vec<String> = dirs
                .iter()
                .filter(|p| p.starts_with(&prefix))
                .cloned()
                .collect();
            for p in desc_dirs {
                dirs.remove(&p);
            }
            let desc_syms: Vec<String> = symlinks
                .keys()
                .filter(|p| p.starts_with(&prefix))
                .cloned()
                .collect();
            for p in desc_syms {
                symlinks.remove(&p);
            }
            return Ok(());
        }
        Ok(())
    }

    fn directory_exists(&self, path: &str) -> bool {
        let resolved = self.resolve_symlinks(path);
        self.lookup_dir_key(&resolved).is_some()
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        let mut entries = Entries::default();
        let resolved = self.resolve_symlinks(path);
        let prefix = if resolved.ends_with('/') {
            resolved
        } else {
            format!("{}/", resolved)
        };

        for key in self.files.read().unwrap().keys() {
            if let Some(rest) = strip_path_prefix(key, &prefix, self.case_sensitive) {
                if !rest.contains('/') {
                    entries.files.push(rest.to_string());
                }
            }
        }

        for dir in self.dirs.read().unwrap().iter() {
            if let Some(rest) = strip_path_prefix(dir, &prefix, self.case_sensitive) {
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
        let is_symlink = self.lookup_symlink_key(path).is_some();
        let resolved = self.resolve_symlinks(path);
        if let Some(key) = self.lookup_file_key(&resolved) {
            let files = self.files.read().unwrap();
            let content = files.get(&key)?;
            return Some(FileInfo {
                name: key.rsplit('/').next()?.to_string(),
                size: content.len() as u64,
                is_dir: false,
                is_symlink,
                modified: std::time::SystemTime::now(),
            });
        }
        let key = self.lookup_dir_key(&resolved)?;
        Some(FileInfo {
            name: key.rsplit('/').next()?.to_string(),
            size: 0,
            is_dir: true,
            is_symlink,
            modified: std::time::SystemTime::now(),
        })
    }

    fn realpath(&self, path: &str) -> String {
        let resolved = self.resolve_symlinks(path);
        if let Some(key) = self.lookup_file_key(&resolved) {
            return key;
        }
        if let Some(key) = self.lookup_dir_key(&resolved) {
            return key;
        }
        // A (possibly broken) symlink resolves to its final target.
        if resolved != path {
            return resolved;
        }
        path.to_string()
    }
}

/// A shared file system handle.
pub type SharedFS = Arc<dyn FS>;

pub mod cachedvfs;
pub mod vfsmatch;

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
        fs.insert_file("/foo.ts", "hello");
        assert!(fs.file_exists("/foo.ts"));
        assert!(fs.file_exists("/Foo.ts"));
        assert_eq!(fs.read_file("/FOO.ts"), Some("hello".to_string()));
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
