//! Tests ported from `internal/vfs/vfstest/vfstest_test.go` (18 tests).
//!
//! All `InMemoryFS` functionality — case-insensitive lookup, symlink support,
//! BOM handling, and path validation on construction — is implemented and
//! exercised below.

use super::*;
use std::collections::HashSet;
use std::sync::Arc;

/// Returns `true` if `path` is a Windows drive-rooted path (`c:/…`).
fn is_windows_rooted(path: &str) -> bool {
    let b = path.as_bytes();
    b.len() >= 3 && b[1] == b':' && (b[2] == b'/' || b[2] == b'\\') && b[0].is_ascii_alphabetic()
}

/// Returns `true` if `path` is normalized: no trailing slash (other than a
/// bare root), and no "." or ".." path segments.
fn is_normalized(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    // A bare POSIX root or Windows drive root is normalized.
    if path == "/" {
        return true;
    }
    if is_windows_rooted(path) && path.len() == 3 {
        return true;
    }
    if path.ends_with('/') {
        return false;
    }
    for seg in path.split('/') {
        if seg == "." || seg == ".." {
            return false;
        }
    }
    true
}

/// Create an `InMemoryFS` from a list of `(path, content)` pairs, inferring
/// intermediate directories from file paths (matching Go's `FromMap`).
fn from_map(files: &[(&str, &str)], case_sensitive: bool) -> InMemoryFS {
    let fs = InMemoryFS::with_case_sensitivity(case_sensitive);

    // Validate each path: must be rooted (POSIX "/" or Windows drive "c:/"),
    // must not mix POSIX and Windows roots, and must be normalized (no
    // trailing slash, no "." or ".." segments).
    let mut seen_posix = false;
    let mut seen_windows = false;
    for (path, _) in files {
        let is_windows = is_windows_rooted(path);
        let is_posix = path.starts_with('/');
        if !is_posix && !is_windows {
            panic!("non-rooted path {path:?}");
        }
        if is_posix {
            seen_posix = true;
        }
        if is_windows {
            seen_windows = true;
        }
        if seen_posix && seen_windows {
            panic!("mixed posix and windows paths");
        }
        if !is_normalized(path) {
            panic!("non-normalized path {path:?}");
        }
    }

    // Detect duplicate canonical paths on a case-insensitive FS.
    if !case_sensitive {
        let mut seen: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
        for (path, _) in files {
            let canonical = path.to_ascii_lowercase();
            if let Some(existing) = seen.get(&canonical) {
                if *existing != *path {
                    panic!(
                        "duplicate path: {:?} and {:?} have the same canonical path",
                        path, existing
                    );
                }
            }
            seen.insert(canonical, path);
        }
    }

    // Detect parent-is-file conflicts: a file path cannot be used as a
    // directory prefix of another file path.
    let file_paths: HashSet<&str> = files.iter().map(|(p, _)| *p).collect();
    for (path, _) in files {
        let mut current = *path;
        while let Some(idx) = current.rfind('/') {
            current = &current[..idx];
            if current.is_empty() {
                break;
            }
            if file_paths.contains(current) {
                panic!(
                    "failed to create intermediate directories for {:?}: mkdir {:?}: path exists but is not a directory",
                    path, current
                );
            }
        }
    }

    let mut dirs = HashSet::new();
    for (path, _) in files {
        let mut current = *path;
        while let Some(idx) = current.rfind('/') {
            current = &current[..idx];
            if !current.is_empty() {
                dirs.insert(current.to_string());
            }
        }
    }
    for dir in &dirs {
        fs.insert_dir(dir);
    }
    for (path, content) in files {
        fs.insert_file(path, content);
    }
    fs
}

// ---------------------------------------------------------------------------
// TestInsensitive / TestInsensitiveUpper — case-insensitive lookup
// ---------------------------------------------------------------------------

#[test]
fn test_insensitive() {
    // Port of Go TestInsensitive.
    // On a case-insensitive FS, reading "Foo/Bar/Baz" should resolve to "foo/bar/baz".
    let contents = "bar";
    let fs = from_map(
        &[
            ("/foo/bar/baz", contents),
            ("/foo/bar2/baz2", contents),
            ("/foo/bar3/baz3", contents),
        ],
        false,
    );

    // Sensitive (exact) lookups work.
    assert_eq!(fs.read_file("/foo/bar/baz"), Some(contents.to_string()));
    assert!(fs.stat("/foo/bar/baz").is_some());
    assert_eq!(fs.realpath("/foo/bar/baz"), "/foo/bar/baz");

    let entries = fs.get_accessible_entries("/foo");
    assert_eq!(entries.directories, vec!["bar", "bar2", "bar3"]);

    // Case-insensitive lookups resolve to the stored (canonical) path.
    assert_eq!(fs.read_file("/Foo/Bar/Baz"), Some(contents.to_string()));
    assert_eq!(fs.realpath("/Foo/Bar/Baz"), "/foo/bar/baz");
}

#[test]
fn test_insensitive_upper() {
    // Port of Go TestInsensitiveUpper.
    // Files stored with uppercase names should be accessible via lowercase on
    // a case-insensitive FS.
    let contents = "bar";
    let fs = from_map(
        &[
            ("/Foo/Bar/Baz", contents),
            ("/Foo/Bar2/Baz2", contents),
            ("/Foo/Bar3/Baz3", contents),
        ],
        false,
    );

    assert_eq!(fs.read_file("/foo/bar/baz"), Some(contents.to_string()));
    let entries = fs.get_accessible_entries("/foo");
    assert_eq!(entries.directories, vec!["Bar", "Bar2", "Bar3"]);
}

// ---------------------------------------------------------------------------
// TestSensitive — case-sensitive FS, exact match required
// ---------------------------------------------------------------------------

#[test]
fn test_sensitive() {
    let contents = "bar";
    let fs = from_map(
        &[
            ("/foo/bar/baz", contents),
            ("/foo/bar2/baz2", contents),
            ("/foo/bar3/baz3", contents),
        ],
        true,
    );

    // Exact case works.
    assert_eq!(fs.read_file("/foo/bar/baz"), Some(contents.to_string()));
    assert!(fs.stat("/foo/bar/baz").is_some());
    assert_eq!(fs.realpath("/foo/bar/baz"), "/foo/bar/baz");

    let entries = fs.get_accessible_entries("/foo");
    assert_eq!(entries.directories, vec!["bar", "bar2", "bar3"]);

    // Wrong case should not find the file on a case-sensitive FS.
    assert_eq!(fs.read_file("/Foo/Bar/Baz"), None);

    // Nonexistent paths.
    assert_eq!(fs.realpath("/does/not/exist"), "/does/not/exist");
    assert!(fs.stat("/does/not/exist").is_none());
}

// ---------------------------------------------------------------------------
// Duplicate path detection (not implemented in Rust InMemoryFS)
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "duplicate path")]
fn test_sensitive_duplicate_path() {
    // Port of Go TestSensitiveDuplicatePath.
    // On a case-insensitive FS, "foo" and "Foo" have the same canonical path
    // and construction should panic.
    let _fs = from_map(&[("/foo", "bar"), ("/Foo", "baz")], false);
}

#[test]
fn test_insensitive_duplicate_path() {
    // Port of Go TestInsensitiveDuplicatePath.
    // On a case-sensitive FS, "foo" and "Foo" are distinct and should coexist.
    let fs = from_map(&[("/foo", "bar"), ("/Foo", "baz")], true);
    assert_eq!(fs.read_file("/foo"), Some("bar".to_string()));
    assert_eq!(fs.read_file("/Foo"), Some("baz".to_string()));
}

// ---------------------------------------------------------------------------
// TestWritableFS — write / read / overwrite
// ---------------------------------------------------------------------------

#[test]
fn test_writable_fs() {
    let fs = InMemoryFS::with_case_sensitivity(false);

    fs.write_file("/foo/bar/baz", "hello, world").unwrap();
    assert_eq!(
        fs.read_file("/foo/bar/baz"),
        Some("hello, world".to_string())
    );

    fs.write_file("/foo/bar/baz", "goodbye, world").unwrap();
    assert_eq!(
        fs.read_file("/foo/bar/baz"),
        Some("goodbye, world".to_string())
    );
}

#[test]
fn test_writable_fs_write_under_file() {
    // Port of Go TestWritableFS — error part.
    // Writing "/foo/bar/baz/oops" should error because "/foo/bar/baz" is a file.
    let fs = InMemoryFS::with_case_sensitivity(false);
    fs.write_file("/foo/bar/baz", "hello, world").unwrap();
    // mkdir "foo/bar/baz": path exists but is not a directory
    let err = fs.write_file("/foo/bar/baz/oops", "goodbye, world");
    assert!(
        err.is_err(),
        "writing under a file path should fail, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// TestWritableFSDelete
// ---------------------------------------------------------------------------

#[test]
fn test_writable_fs_delete() {
    let fs = InMemoryFS::with_case_sensitivity(false);

    // Delete a file.
    fs.write_file("/foo/bar/file.ts", "remove").unwrap();
    fs.insert_dir("/foo/bar");
    assert!(fs.file_exists("/foo/bar/file.ts"));
    fs.remove("/foo/bar/file.ts").unwrap();
    assert!(!fs.file_exists("/foo/bar/file.ts"));

    // No errors when removing file/dir that does not exist.
    fs.remove("/foo/bar/test").unwrap();
    fs.remove("/foo/bar/file.ts").unwrap();

    // Removing "/foo/bar" should not affect "/foo/barbar".
    fs.write_file("/foo/barbar", "remove2").unwrap();
    fs.remove("/foo/bar").unwrap();
    assert!(fs.file_exists("/foo/barbar"));
}

#[test]
fn test_writable_fs_delete_directory_recursive() {
    // Port of Go TestWritableFSDelete — recursive directory removal part.
    // Go's Remove recursively removes all children under a directory.
    let fs = InMemoryFS::with_case_sensitivity(false);
    fs.write_file("/foo/bar/test/remove2.ts", "remove2")
        .unwrap();
    fs.insert_dir("/foo/bar/test");
    assert!(fs.directory_exists("/foo/bar/test"));
    fs.remove("/foo/bar/test").unwrap();
    assert!(!fs.directory_exists("/foo/bar/test"));
    assert!(!fs.file_exists("/foo/bar/test/remove2.ts"));
}

// ---------------------------------------------------------------------------
// TestStress — concurrent access
// ---------------------------------------------------------------------------

#[test]
fn test_stress() {
    let fs = Arc::new(InMemoryFS::with_case_sensitivity(false));
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let fs = Arc::clone(&fs);
        handles.push(std::thread::spawn(move || {
            for i in 0..10_000 {
                match i % 6 {
                    0 => {
                        let _ = fs.write_file("/foo/bar/baz.txt", "hello, world");
                    }
                    1 => {
                        fs.read_file("/foo/bar/baz.txt");
                    }
                    2 => {
                        fs.directory_exists("/foo/bar");
                    }
                    3 => {
                        fs.file_exists("/foo/bar");
                    }
                    4 => {
                        fs.file_exists("/foo/bar/baz.txt");
                    }
                    5 => {
                        fs.get_accessible_entries("/foo/bar");
                    }
                    _ => {}
                }
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}

// ---------------------------------------------------------------------------
// TestParentDirFile — parent-is-file validation
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not a directory")]
fn test_parent_dir_file() {
    // Port of Go TestParentDirFile.
    // "foo" is a file but "foo/oops" tries to use it as a directory.
    // Go panics: failed to create intermediate directories for "foo/oops"
    let _fs = from_map(&[("/foo", "bar"), ("/foo/oops", "baz")], false);
}

// ---------------------------------------------------------------------------
// TestFromMap — sub-tests for construction validation
// ---------------------------------------------------------------------------

#[test]
fn test_from_map_posix() {
    let fs = from_map(
        &[
            ("/string", "hello, world"),
            ("/bytes", "hello, world"),
            ("/mapfile", "hello, world"),
        ],
        false,
    );
    assert_eq!(fs.read_file("/string"), Some("hello, world".to_string()));
    assert_eq!(fs.read_file("/bytes"), Some("hello, world".to_string()));
    assert_eq!(fs.read_file("/mapfile"), Some("hello, world".to_string()));
}

#[test]
fn test_from_map_windows() {
    let fs = from_map(
        &[
            ("c:/string", "hello, world"),
            ("d:/bytes", "hello, world"),
            ("e:/mapfile", "hello, world"),
        ],
        false,
    );
    assert_eq!(fs.read_file("c:/string"), Some("hello, world".to_string()));
    assert_eq!(fs.read_file("d:/bytes"), Some("hello, world".to_string()));
    assert_eq!(fs.read_file("e:/mapfile"), Some("hello, world".to_string()));
}

#[test]
#[should_panic(expected = "mixed posix and windows paths")]
fn test_from_map_mixed() {
    // Port of Go TestFromMap "Mixed".
    // Mixing "/" and "c:/" roots should panic: 'mixed posix and windows paths'
    let _fs = from_map(&[("/string", "x"), ("c:/bytes", "x")], false);
}

#[test]
#[should_panic(expected = "non-rooted path")]
fn test_from_map_non_rooted() {
    // Port of Go TestFromMap "NonRooted".
    // Non-rooted path "string" should panic: 'non-rooted path "string"'
    let _fs = from_map(&[("string", "x")], false);
}

#[test]
#[should_panic(expected = "non-normalized path")]
fn test_from_map_non_normalized() {
    // Port of Go TestFromMap "NonNormalized".
    // Trailing slash should panic: 'non-normalized path "/string/"'
    let _fs = from_map(&[("/string/", "x")], false);
}

#[test]
#[should_panic(expected = "non-normalized path")]
fn test_from_map_non_normalized2() {
    // Port of Go TestFromMap "NonNormalized2".
    // ".." in path should panic: 'non-normalized path "/string/../foo"'
    let _fs = from_map(&[("/string/../foo", "x")], false);
}

#[test]
fn test_from_map_invalid_file() {
    // Port of Go TestFromMap "InvalidFile".
    // Go's FromMap panics on a non-string/bytes value ('invalid file type int').
    // The Rust `from_map` helper takes `&[(&str, &str)]`, so file contents are
    // always strings — non-string values cannot be expressed in the API. This
    // test documents that contract: every provided value is a valid file.
    let fs = from_map(&[("/a", "1"), ("/b", "text")], true);
    assert_eq!(fs.read_file("/a"), Some("1".to_string()));
    assert_eq!(fs.read_file("/b"), Some("text".to_string()));
}

// ---------------------------------------------------------------------------
// TestVFSTestMapFS — ReadFile, Realpath, UseCaseSensitiveFileNames
// ---------------------------------------------------------------------------

#[test]
fn test_vfs_test_map_fs() {
    let fs = from_map(
        &[
            ("/foo.ts", "hello, world"),
            ("/dir1/file1.ts", "export const foo = 42;"),
            ("/dir1/file2.ts", "export const foo = 42;"),
            ("/dir2/file1.ts", "export const foo = 42;"),
        ],
        false,
    );

    // ReadFile
    assert_eq!(fs.read_file("/foo.ts"), Some("hello, world".to_string()));
    assert_eq!(fs.read_file("/does/not/exist.ts"), None);

    // Realpath returns the path itself (no symlinks in InMemoryFS).
    assert_eq!(fs.realpath("/foo.ts"), "/foo.ts");
    // On case-insensitive FS, Go would canonicalize; Rust returns input as-is.
    assert_eq!(fs.realpath("/does/not/exist.ts"), "/does/not/exist.ts");

    // UseCaseSensitiveFileNames
    assert!(!fs.use_case_sensitive_file_names());
}

#[test]
fn test_vfs_test_map_fs_windows() {
    let fs = from_map(
        &[
            ("c:/foo.ts", "hello, world"),
            ("c:/dir1/file1.ts", "export const foo = 42;"),
            ("c:/dir1/file2.ts", "export const foo = 42;"),
            ("c:/dir2/file1.ts", "export const foo = 42;"),
        ],
        false,
    );

    assert_eq!(fs.read_file("c:/foo.ts"), Some("hello, world".to_string()));
    assert_eq!(fs.read_file("c:/does/not/exist.ts"), None);

    assert_eq!(fs.realpath("c:/foo.ts"), "c:/foo.ts");
    assert_eq!(fs.realpath("c:/does/not/exist.ts"), "c:/does/not/exist.ts");
}

// ---------------------------------------------------------------------------
// TestBOM — BOM stripping
// ---------------------------------------------------------------------------

#[test]
fn test_bom() {
    // Port of Go TestBOM (UTF-8 sub-test).
    // A UTF-8 BOM (U+FEFF) prefix should be stripped on read.
    // (UTF-16 BOMs are not exercised here because `InMemoryFS` stores UTF-8
    // `String`s and cannot hold the non-UTF-8 bytes of a UTF-16 BE BOM.)
    let expected = "hello, world";
    let fs = from_map(&[("/foo.ts", "\u{FEFF}hello, world")], true);
    assert_eq!(fs.read_file("/foo.ts"), Some(expected.to_string()));
}

// ---------------------------------------------------------------------------
// Symlink tests — symlink support not implemented in InMemoryFS
// ---------------------------------------------------------------------------

#[test]
fn test_symlink() {
    // Port of Go TestSymlink.
    // Tests ReadFile, Realpath, FileExists, DirectoryExists through symlinks.
    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.insert_file("/foo.ts", "hello, world");
    fs.insert_dir("/dir");
    fs.insert_file("/dir/file.ts", "export const x = 1;");
    fs.create_symlink("/link.ts", "/foo.ts");
    fs.create_symlink("/dirlink", "/dir");

    // ReadFile through a file symlink.
    assert_eq!(fs.read_file("/link.ts"), Some("hello, world".to_string()));
    // Realpath resolves to the target.
    assert_eq!(fs.realpath("/link.ts"), "/foo.ts");
    // FileExists follows symlinks.
    assert!(fs.file_exists("/link.ts"));

    // DirectoryExists through a directory symlink.
    assert!(fs.directory_exists("/dirlink"));
    assert_eq!(fs.realpath("/dirlink"), "/dir");

    // Listing entries through a directory symlink resolves children via
    // prefix resolution of the symlink.
    let entries = fs.get_accessible_entries("/dirlink");
    assert!(entries.files.contains(&"file.ts".to_string()));

    // Reading a file under a directory symlink (symlink in the middle).
    assert_eq!(
        fs.read_file("/dirlink/file.ts"),
        Some("export const x = 1;".to_string())
    );
}

#[test]
fn test_writable_fs_symlink() {
    // Port of Go TestWritableFSSymlink.
    // Tests writing through symlinks, broken symlinks, and error cases.
    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.write_file("/foo", "hello").unwrap();
    fs.create_symlink("/link", "/foo");

    // Writing through the symlink updates the target file.
    fs.write_file("/link", "goodbye").unwrap();
    assert_eq!(fs.read_file("/foo"), Some("goodbye".to_string()));
    assert_eq!(fs.read_file("/link"), Some("goodbye".to_string()));

    // A broken symlink (target missing) reads as absent but realpath still
    // reports the dangling target.
    fs.create_symlink("/broken", "/missing");
    assert_eq!(fs.read_file("/broken"), None);
    assert!(!fs.file_exists("/broken"));
    assert_eq!(fs.realpath("/broken"), "/missing");
}

#[test]
fn test_writable_fs_symlink_chain() {
    // Port of Go TestWritableFSSymlinkChain.
    // Tests writing through a chain of symlinks (a→b→c→d).
    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.write_file("/d", "x").unwrap();
    fs.create_symlink("/a", "/b");
    fs.create_symlink("/b", "/c");
    fs.create_symlink("/c", "/d");

    // Writing through the chain resolves to the final target /d.
    fs.write_file("/a", "hello").unwrap();
    assert_eq!(fs.read_file("/d"), Some("hello".to_string()));
    assert_eq!(fs.realpath("/a"), "/d");
    assert!(fs.file_exists("/a"));
}

#[test]
fn test_writable_fs_symlink_chain_not_dir() {
    // Port of Go TestWritableFSSymlinkChainNotDir.
    // Tests that writing under a symlink chain ending in a file produces an
    // error (the parent resolves to a file, not a directory).
    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.write_file("/d", "x").unwrap();
    fs.create_symlink("/a", "/b");
    fs.create_symlink("/b", "/c");
    fs.create_symlink("/c", "/d");

    let err = fs.write_file("/a/oops", "y");
    assert!(
        err.is_err(),
        "writing under a symlink chain ending in a file should fail, got {err:?}"
    );
}

#[test]
fn test_writable_fs_symlink_delete() {
    // Port of Go TestWritableFSSymlinkDelete.
    // Tests deleting symlinks, re-creating targets, and broken symlink behavior.
    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.write_file("/foo", "hello").unwrap();
    fs.create_symlink("/link", "/foo");

    // Removing the symlink leaves the target intact.
    fs.remove("/link").unwrap();
    assert_eq!(fs.read_symlink("/link"), None);
    assert!(fs.file_exists("/foo"));
    assert_eq!(fs.read_file("/foo"), Some("hello".to_string()));

    // After the target is removed, the link (if recreated) is dangling.
    fs.create_symlink("/link", "/foo");
    fs.remove("/foo").unwrap();
    assert_eq!(fs.read_file("/link"), None);
    assert!(!fs.file_exists("/link"));
}
