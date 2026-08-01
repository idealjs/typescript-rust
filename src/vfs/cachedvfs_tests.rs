//! Tests ported from `internal/vfs/cachedvfs/cachedvfs_test.go` (10 tests).
//!
//! Each test wraps a call-recording [`CountingFS`] in a [`CachedFS`] and
//! verifies that:
//! - repeated calls to a cached method hit the cache (underlying call count
//!   stays the same),
//! - `clear_cache` / `disable_and_clear_cache` / `enable` behave as in Go.

use super::cachedvfs::CachedFS;
use super::*;
use std::sync::{Arc, Mutex};

/// A call-recording wrapper around an [`InMemoryFS`], mirroring Go's
/// `vfsmock.FSMock`. Every method records its arguments so the cachedvfs
/// tests can assert on underlying call counts.
struct CountingFS {
    inner: InMemoryFS,
    directory_exists_calls: Mutex<Vec<String>>,
    file_exists_calls: Mutex<Vec<String>>,
    get_accessible_entries_calls: Mutex<Vec<String>>,
    realpath_calls: Mutex<Vec<String>>,
    stat_calls: Mutex<Vec<String>>,
    read_file_calls: Mutex<Vec<String>>,
    use_case_sensitive_calls: Mutex<u32>,
    walk_dir_calls: Mutex<Vec<String>>,
    remove_calls: Mutex<Vec<String>>,
    write_file_calls: Mutex<Vec<(String, String)>>,
}

impl CountingFS {
    fn new(inner: InMemoryFS) -> Self {
        CountingFS {
            inner,
            directory_exists_calls: Mutex::new(Vec::new()),
            file_exists_calls: Mutex::new(Vec::new()),
            get_accessible_entries_calls: Mutex::new(Vec::new()),
            realpath_calls: Mutex::new(Vec::new()),
            stat_calls: Mutex::new(Vec::new()),
            read_file_calls: Mutex::new(Vec::new()),
            use_case_sensitive_calls: Mutex::new(0),
            walk_dir_calls: Mutex::new(Vec::new()),
            remove_calls: Mutex::new(Vec::new()),
            write_file_calls: Mutex::new(Vec::new()),
        }
    }

    fn directory_exists_calls(&self) -> usize {
        self.directory_exists_calls.lock().unwrap().len()
    }

    fn file_exists_calls(&self) -> usize {
        self.file_exists_calls.lock().unwrap().len()
    }

    fn get_accessible_entries_calls(&self) -> usize {
        self.get_accessible_entries_calls.lock().unwrap().len()
    }

    fn realpath_calls(&self) -> usize {
        self.realpath_calls.lock().unwrap().len()
    }

    fn stat_calls(&self) -> usize {
        self.stat_calls.lock().unwrap().len()
    }

    fn read_file_calls(&self) -> usize {
        self.read_file_calls.lock().unwrap().len()
    }

    fn use_case_sensitive_calls(&self) -> u32 {
        *self.use_case_sensitive_calls.lock().unwrap()
    }

    fn walk_dir_calls(&self) -> usize {
        self.walk_dir_calls.lock().unwrap().len()
    }

    fn remove_calls(&self) -> usize {
        self.remove_calls.lock().unwrap().len()
    }

    fn write_file_calls(&self) -> Vec<(String, String)> {
        self.write_file_calls.lock().unwrap().clone()
    }
}

impl FS for CountingFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        *self.use_case_sensitive_calls.lock().unwrap() += 1;
        self.inner.use_case_sensitive_file_names()
    }

    fn file_exists(&self, path: &str) -> bool {
        self.file_exists_calls
            .lock()
            .unwrap()
            .push(path.to_string());
        self.inner.file_exists(path)
    }

    fn read_file(&self, path: &str) -> Option<String> {
        self.read_file_calls.lock().unwrap().push(path.to_string());
        self.inner.read_file(path)
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        self.write_file_calls
            .lock()
            .unwrap()
            .push((path.to_string(), data.to_string()));
        self.inner.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        self.inner.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        self.remove_calls.lock().unwrap().push(path.to_string());
        self.inner.remove(path)
    }

    fn directory_exists(&self, path: &str) -> bool {
        self.directory_exists_calls
            .lock()
            .unwrap()
            .push(path.to_string());
        self.inner.directory_exists(path)
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        self.get_accessible_entries_calls
            .lock()
            .unwrap()
            .push(path.to_string());
        self.inner.get_accessible_entries(path)
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        self.stat_calls.lock().unwrap().push(path.to_string());
        self.inner.stat(path)
    }

    fn realpath(&self, path: &str) -> String {
        self.realpath_calls.lock().unwrap().push(path.to_string());
        self.inner.realpath(path)
    }

    fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &FileInfo),
    ) -> std::io::Result<()> {
        self.walk_dir_calls.lock().unwrap().push(root.to_string());
        self.inner.walk_dir(root, walk_fn)
    }
}

/// Build a `CountingFS` seeded with `"/some/path/file.txt"` (mirrors Go's
/// `createMockFS`, which uses `vfstest.FromMap` and infers parent dirs).
fn create_mock_fs() -> CountingFS {
    let inner = InMemoryFS::with_case_sensitivity(true);
    inner.insert_dir("/some");
    inner.insert_dir("/some/path");
    inner.insert_file("/some/path/file.txt", "hello world");
    CountingFS::new(inner)
}

// ---------------------------------------------------------------------------
// Cached methods: directory_exists / file_exists / get_accessible_entries /
// realpath / stat.
// Expected underlying call sequence: 1, 1 (cached), clear -> 2, 3, disable ->
// 4, 5 (not cached), enable -> 6, 6 (cached again).
// ---------------------------------------------------------------------------

#[test]
fn test_cached_directory_exists() {
    // Port of Go TestDirectoryExists.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    cached.directory_exists("/some/path");
    assert_eq!(1, underlying.directory_exists_calls());

    cached.directory_exists("/some/path");
    assert_eq!(1, underlying.directory_exists_calls());

    cached.clear_cache();
    cached.directory_exists("/some/path");
    assert_eq!(2, underlying.directory_exists_calls());

    cached.directory_exists("/other/path");
    assert_eq!(3, underlying.directory_exists_calls());

    cached.disable_and_clear_cache();
    cached.directory_exists("/some/path");
    assert_eq!(4, underlying.directory_exists_calls());

    cached.directory_exists("/some/path");
    assert_eq!(5, underlying.directory_exists_calls());

    cached.enable();
    cached.directory_exists("/some/path");
    assert_eq!(6, underlying.directory_exists_calls());

    cached.directory_exists("/some/path");
    assert_eq!(6, underlying.directory_exists_calls());
}

#[test]
fn test_cached_file_exists() {
    // Port of Go TestFileExists.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    cached.file_exists("/some/path/file.txt");
    assert_eq!(1, underlying.file_exists_calls());

    cached.file_exists("/some/path/file.txt");
    assert_eq!(1, underlying.file_exists_calls());

    cached.clear_cache();
    cached.file_exists("/some/path/file.txt");
    assert_eq!(2, underlying.file_exists_calls());

    cached.file_exists("/other/path/file.txt");
    assert_eq!(3, underlying.file_exists_calls());

    cached.disable_and_clear_cache();
    cached.file_exists("/some/path/file.txt");
    assert_eq!(4, underlying.file_exists_calls());

    cached.file_exists("/some/path/file.txt");
    assert_eq!(5, underlying.file_exists_calls());

    cached.enable();
    cached.file_exists("/some/path/file.txt");
    assert_eq!(6, underlying.file_exists_calls());

    cached.file_exists("/some/path/file.txt");
    assert_eq!(6, underlying.file_exists_calls());
}

#[test]
fn test_cached_get_accessible_entries() {
    // Port of Go TestGetAccessibleEntries.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    cached.get_accessible_entries("/some/path");
    assert_eq!(1, underlying.get_accessible_entries_calls());

    cached.get_accessible_entries("/some/path");
    assert_eq!(1, underlying.get_accessible_entries_calls());

    cached.clear_cache();
    cached.get_accessible_entries("/some/path");
    assert_eq!(2, underlying.get_accessible_entries_calls());

    cached.get_accessible_entries("/other/path");
    assert_eq!(3, underlying.get_accessible_entries_calls());

    cached.disable_and_clear_cache();
    cached.get_accessible_entries("/some/path");
    assert_eq!(4, underlying.get_accessible_entries_calls());

    cached.get_accessible_entries("/some/path");
    assert_eq!(5, underlying.get_accessible_entries_calls());

    cached.enable();
    cached.get_accessible_entries("/some/path");
    assert_eq!(6, underlying.get_accessible_entries_calls());

    cached.get_accessible_entries("/some/path");
    assert_eq!(6, underlying.get_accessible_entries_calls());
}

#[test]
fn test_cached_realpath() {
    // Port of Go TestRealpath.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    cached.realpath("/some/path");
    assert_eq!(1, underlying.realpath_calls());

    cached.realpath("/some/path");
    assert_eq!(1, underlying.realpath_calls());

    cached.clear_cache();
    cached.realpath("/some/path");
    assert_eq!(2, underlying.realpath_calls());

    cached.realpath("/other/path");
    assert_eq!(3, underlying.realpath_calls());

    cached.disable_and_clear_cache();
    cached.realpath("/some/path");
    assert_eq!(4, underlying.realpath_calls());

    cached.realpath("/some/path");
    assert_eq!(5, underlying.realpath_calls());

    cached.enable();
    cached.realpath("/some/path");
    assert_eq!(6, underlying.realpath_calls());

    cached.realpath("/some/path");
    assert_eq!(6, underlying.realpath_calls());
}

#[test]
fn test_cached_stat() {
    // Port of Go TestStat.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    cached.stat("/some/path");
    assert_eq!(1, underlying.stat_calls());

    cached.stat("/some/path");
    assert_eq!(1, underlying.stat_calls());

    cached.clear_cache();
    cached.stat("/some/path");
    assert_eq!(2, underlying.stat_calls());

    cached.stat("/other/path");
    assert_eq!(3, underlying.stat_calls());

    cached.disable_and_clear_cache();
    cached.stat("/some/path");
    assert_eq!(4, underlying.stat_calls());

    cached.stat("/some/path");
    assert_eq!(5, underlying.stat_calls());

    cached.enable();
    cached.stat("/some/path");
    assert_eq!(6, underlying.stat_calls());

    cached.stat("/some/path");
    assert_eq!(6, underlying.stat_calls());
}

// ---------------------------------------------------------------------------
// Pass-through (never cached) methods.
// Expected underlying call sequence: 1, 2, clear -> 3, disable -> 4, 5, enable
// -> 6, 7 (every call reaches the underlying FS).
// ---------------------------------------------------------------------------

#[test]
fn test_cached_read_file() {
    // Port of Go TestReadFile.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    cached.read_file("/some/path/file.txt");
    assert_eq!(1, underlying.read_file_calls());

    cached.read_file("/some/path/file.txt");
    assert_eq!(2, underlying.read_file_calls());

    cached.clear_cache();
    cached.read_file("/some/path/file.txt");
    assert_eq!(3, underlying.read_file_calls());

    cached.disable_and_clear_cache();
    cached.read_file("/some/path/file.txt");
    assert_eq!(4, underlying.read_file_calls());

    cached.read_file("/some/path/file.txt");
    assert_eq!(5, underlying.read_file_calls());

    cached.enable();
    cached.read_file("/some/path/file.txt");
    assert_eq!(6, underlying.read_file_calls());

    cached.read_file("/some/path/file.txt");
    assert_eq!(7, underlying.read_file_calls());
}

#[test]
fn test_cached_use_case_sensitive_file_names() {
    // Port of Go TestUseCaseSensitiveFileNames.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    cached.use_case_sensitive_file_names();
    assert_eq!(1, underlying.use_case_sensitive_calls());

    cached.use_case_sensitive_file_names();
    assert_eq!(2, underlying.use_case_sensitive_calls());

    cached.clear_cache();
    cached.use_case_sensitive_file_names();
    assert_eq!(3, underlying.use_case_sensitive_calls());

    cached.disable_and_clear_cache();
    cached.use_case_sensitive_file_names();
    assert_eq!(4, underlying.use_case_sensitive_calls());

    cached.use_case_sensitive_file_names();
    assert_eq!(5, underlying.use_case_sensitive_calls());

    cached.enable();
    cached.use_case_sensitive_file_names();
    assert_eq!(6, underlying.use_case_sensitive_calls());

    cached.use_case_sensitive_file_names();
    assert_eq!(7, underlying.use_case_sensitive_calls());
}

#[test]
fn test_cached_walk_dir() {
    // Port of Go TestWalkDir. WalkDir is not cached — always passed through.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    let mut walk_fn = |_path: &str, _info: &FileInfo| {};

    cached.walk_dir("/some/path", &mut walk_fn).unwrap();
    assert_eq!(1, underlying.walk_dir_calls());

    cached.walk_dir("/some/path", &mut walk_fn).unwrap();
    assert_eq!(2, underlying.walk_dir_calls());

    cached.clear_cache();
    cached.walk_dir("/some/path", &mut walk_fn).unwrap();
    assert_eq!(3, underlying.walk_dir_calls());

    cached.disable_and_clear_cache();
    cached.walk_dir("/some/path", &mut walk_fn).unwrap();
    assert_eq!(4, underlying.walk_dir_calls());

    cached.walk_dir("/some/path", &mut walk_fn).unwrap();
    assert_eq!(5, underlying.walk_dir_calls());

    cached.enable();
    cached.walk_dir("/some/path", &mut walk_fn).unwrap();
    assert_eq!(6, underlying.walk_dir_calls());

    cached.walk_dir("/some/path", &mut walk_fn).unwrap();
    assert_eq!(7, underlying.walk_dir_calls());
}

#[test]
fn test_cached_remove() {
    // Port of Go TestRemove. Remove is not cached — always passed through.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    let _ = cached.remove("/some/path/file.txt");
    assert_eq!(1, underlying.remove_calls());

    let _ = cached.remove("/some/path/file.txt");
    assert_eq!(2, underlying.remove_calls());

    cached.clear_cache();
    let _ = cached.remove("/some/path/file.txt");
    assert_eq!(3, underlying.remove_calls());

    cached.disable_and_clear_cache();
    let _ = cached.remove("/some/path/file.txt");
    assert_eq!(4, underlying.remove_calls());

    let _ = cached.remove("/some/path/file.txt");
    assert_eq!(5, underlying.remove_calls());

    cached.enable();
    let _ = cached.remove("/some/path/file.txt");
    assert_eq!(6, underlying.remove_calls());

    let _ = cached.remove("/some/path/file.txt");
    assert_eq!(7, underlying.remove_calls());
}

#[test]
fn test_cached_write_file() {
    // Port of Go TestWriteFile. WriteFile is not cached — always passed
    // through, and call args are verifiable on the mock.
    let underlying = Arc::new(create_mock_fs());
    let cached = CachedFS::new(underlying.clone());

    let _ = cached.write_file("/some/path/file.txt", "new content");
    assert_eq!(1, underlying.write_file_calls().len());

    let _ = cached.write_file("/some/path/file.txt", "another content");
    assert_eq!(2, underlying.write_file_calls().len());

    cached.clear_cache();
    let _ = cached.write_file("/some/path/file.txt", "third content");
    assert_eq!(3, underlying.write_file_calls().len());

    let call = &underlying.write_file_calls()[2];
    assert_eq!(call.0, "/some/path/file.txt");
    assert_eq!(call.1, "third content");

    cached.disable_and_clear_cache();
    let _ = cached.write_file("/some/path/file.txt", "fourth content");
    assert_eq!(4, underlying.write_file_calls().len());

    let _ = cached.write_file("/some/path/file.txt", "fifth content");
    assert_eq!(5, underlying.write_file_calls().len());

    cached.enable();
    let _ = cached.write_file("/some/path/file.txt", "sixth content");
    assert_eq!(6, underlying.write_file_calls().len());

    let _ = cached.write_file("/some/path/file.txt", "seventh content");
    assert_eq!(7, underlying.write_file_calls().len());
}
