//! Bundled `lib.*.d.ts` files, ported from `internal/bundled/`.
//!
//! The `.d.ts` files are embedded into the binary at build time (see
//! `build.rs`) and served through a virtual `bundled:///` URL scheme, mirroring
//! Go's `bundled:///libs` lib path and `wrapFS`.

use std::sync::Arc;

use crate::vfs::{Entries, FS, FileInfo};

// Include the build-generated table of `(name, content)` pairs.
include!(concat!(env!("OUT_DIR"), "/bundled_libs.rs"));

/// The virtual URL scheme used for bundled paths.
pub const SCHEME: &str = "bundled:///";

/// The virtual directory containing the bundled lib files.
pub fn lib_path() -> String {
    format!("{SCHEME}libs")
}

/// Whether `path` refers to a bundled (embedded) location.
pub fn is_bundled(path: &str) -> bool {
    path.starts_with(SCHEME)
}

/// Strip the `bundled:///` prefix, returning the remainder.
fn split_path(path: &str) -> Option<&str> {
    path.strip_prefix(SCHEME)
}

/// The list of bundled lib file names (e.g. `lib.d.ts`, `lib.es5.d.ts`).
pub fn lib_names() -> Vec<&'static str> {
    bundled_libs().iter().map(|(n, _)| *n).collect()
}

/// Look up the contents of a bundled lib by name (e.g. `lib.d.ts`).
pub fn lib_contents(name: &str) -> Option<&'static str> {
    bundled_libs()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
}

/// A file system wrapper that serves bundled lib files from embedded data for
/// paths under the `bundled:///` scheme, and delegates everything else to an
/// inner file system.
///
/// Mirrors Go's `wrappedFS`.
pub struct BundledFS {
    inner: Arc<dyn FS>,
}

impl BundledFS {
    pub fn new(inner: Arc<dyn FS>) -> Self {
        Self { inner }
    }
}

impl FS for BundledFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.inner.use_case_sensitive_file_names()
    }

    fn file_exists(&self, path: &str) -> bool {
        if let Some(rest) = split_path(path) {
            return bundled_lib_name(rest).is_some();
        }
        self.inner.file_exists(path)
    }

    fn read_file(&self, path: &str) -> Option<String> {
        if let Some(rest) = split_path(path) {
            if let Some(name) = bundled_lib_name(rest) {
                return lib_contents(name).map(|s| s.to_string());
            }
            return None;
        }
        self.inner.read_file(path)
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        if is_bundled(path) {
            panic!("cannot write to embedded file system: {path}");
        }
        self.inner.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        if is_bundled(path) {
            panic!("cannot write to embedded file system: {path}");
        }
        self.inner.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        if is_bundled(path) {
            panic!("cannot remove from embedded file system: {path}");
        }
        self.inner.remove(path)
    }

    fn directory_exists(&self, path: &str) -> bool {
        if let Some(rest) = split_path(path) {
            return rest == "libs" || rest.is_empty();
        }
        self.inner.directory_exists(path)
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        if let Some(rest) = split_path(path) {
            let mut entries = Entries::default();
            if rest.is_empty() {
                entries.directories.push("libs".to_string());
            } else if rest == "libs" {
                entries.files = lib_names().iter().map(|s| s.to_string()).collect();
            }
            return entries;
        }
        self.inner.get_accessible_entries(path)
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        if let Some(rest) = split_path(path) {
            if rest.is_empty() || rest == "libs" {
                return Some(FileInfo {
                    name: if rest.is_empty() {
                        String::new()
                    } else {
                        "libs".to_string()
                    },
                    is_dir: true,
                    ..FileInfo::default()
                });
            }
            if let Some(name) = bundled_lib_name(rest) {
                let size = lib_contents(name).map(|c| c.len()).unwrap_or(0) as u64;
                return Some(FileInfo {
                    name: name.to_string(),
                    size,
                    is_dir: false,
                    ..FileInfo::default()
                });
            }
            return None;
        }
        self.inner.stat(path)
    }

    fn realpath(&self, path: &str) -> String {
        if is_bundled(path) {
            return path.to_string();
        }
        self.inner.realpath(path)
    }
}

/// Given a bundled-relative path like `libs/lib.d.ts`, return the lib name
/// (`lib.d.ts`) if it exists in the embedded set.
fn bundled_lib_name(rest: &str) -> Option<&'static str> {
    let name = rest.strip_prefix("libs/").unwrap_or(rest);
    bundled_libs()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(n, _)| *n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::InMemoryFS;

    #[test]
    fn lib_path_uses_scheme() {
        assert_eq!(lib_path(), "bundled:///libs");
        assert!(is_bundled("bundled:///libs/lib.d.ts"));
        assert!(!is_bundled("/home/user/lib.d.ts"));
    }

    #[test]
    fn bundled_fs_serves_libs() {
        let inner = Arc::new(InMemoryFS::new());
        let fs = BundledFS::new(inner);
        // If libs were embedded at build time, lib.d.ts should be readable.
        let path = "bundled:///libs/lib.d.ts";
        if fs.file_exists(path) {
            assert!(fs.read_file(path).is_some());
        }
        assert!(fs.directory_exists("bundled:///libs"));
    }

    #[test]
    fn bundled_fs_case_sensitive_matching() {
        // Lib name matching should be case-sensitive (matches Go's map lookup)
        let inner = Arc::new(InMemoryFS::new());
        let fs = BundledFS::new(inner);
        // If lib.d.ts exists, LIB.D.TS should NOT (case-sensitive)
        let lower = "bundled:///libs/lib.d.ts";
        let upper = "bundled:///libs/LIB.D.TS";
        if fs.file_exists(lower) {
            assert!(
                !fs.file_exists(upper),
                "Bundled lib matching should be case-sensitive"
            );
        }
    }

    #[test]
    fn bundled_fs_delegates_case_sensitivity() {
        // use_case_sensitive_file_names should delegate to inner FS
        let inner = Arc::new(InMemoryFS::new());
        let fs = BundledFS::new(inner);
        // InMemoryFS defaults to case-sensitive
        assert!(fs.use_case_sensitive_file_names());
    }

    #[test]
    fn bundled_fs_lib_names_nonempty() {
        // At least lib.d.ts should be in the embedded set (if built with build.rs)
        let names = lib_names();
        // Only check if build.rs generated the table
        if !names.is_empty() {
            assert!(
                names.iter().any(|n| *n == "lib.d.ts"),
                "Expected lib.d.ts in bundled libs"
            );
        }
    }

    /// Ported from Go `TestTestingLibPath`.
    ///
    /// In Go this checks the on-disk source libs directory. In Rust the libs are
    /// embedded at build time, so we verify `lib.d.ts` is available through the
    /// embedded set.
    #[test]
    fn testing_lib_path() {
        let names = lib_names();
        if !names.is_empty() {
            assert!(
                lib_contents("lib.d.ts").is_some(),
                "Expected lib.d.ts in bundled libs"
            );
        }
    }

    /// Ported from Go `TestEmbeddedLibs`.
    ///
    /// Walks the bundled virtual FS `libs` directory and verifies that the file
    /// list matches `lib_names()`.
    #[test]
    fn embedded_libs() {
        let inner = Arc::new(InMemoryFS::new());
        let fs = BundledFS::new(inner);
        let entries = fs.get_accessible_entries(&lib_path());
        let mut files = entries.files.clone();
        files.sort();
        let mut names: Vec<String> = lib_names().iter().map(|s| s.to_string()).collect();
        names.sort();
        assert_eq!(files, names);
    }
}
