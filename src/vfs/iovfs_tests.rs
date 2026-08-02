//! Test adapted from `internal/vfs/iovfs/iofs_test.go`.
//!
//! The Go `iovfs` package adapts Go's `io/fs.FS` interface (e.g.
//! `testing/fstest.MapFS`) to the `vfs.FS` interface. In Rust the equivalent
//! role is played by [`OsFS`], which wraps `std::fs` operations behind the
//! [`FS`] trait. This test exercises `OsFS` through the trait interface by
//! creating, reading, and listing temp files and verifying the content.

use super::*;

/// Port of Go `TestIOFS`.
///
/// Exercises `OsFS` through the `FS` trait: writes a file via the trait, reads
/// it back, checks `file_exists` / `directory_exists`, lists entries with
/// `get_accessible_entries`, and checks `use_case_sensitive_file_names`.
#[test]
fn test_iofs() {
    use std::path::PathBuf;

    let fs = OsFS;

    // Unique temp directory per run.
    let mut tmp = std::env::temp_dir();
    tmp.push(format!("tsox_iovfs_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Create a file via the FS trait (this is what Go's IOFS adapter does:
    // surface underlying stdlib FS ops behind vfs.FS).
    let file_path: PathBuf = tmp.join("hello.txt");
    let file_str = file_path.to_str().unwrap();
    fs.write_file(file_str, "hello world").unwrap();

    // file_exists should reflect the newly written file.
    assert!(fs.file_exists(file_str));
    assert!(!fs.directory_exists(file_str));

    // Read it back through the trait and verify content matches.
    let content = fs.read_file(file_str).expect("read_file returned None");
    assert_eq!(content, "hello world");

    // use_case_sensitive_file_names matches the platform default.
    assert_eq!(
        fs.use_case_sensitive_file_names(),
        cfg!(not(target_os = "windows"))
    );

    // get_accessible_entries should list the temp directory and contain the
    // file we created.
    let entries = fs.get_accessible_entries(tmp.to_str().unwrap());
    assert!(
        entries.files.iter().any(|f| f == "hello.txt"),
        "expected hello.txt in entries.files: {entries:?}"
    );

    // realpath should canonicalize to an absolute path containing the file.
    let real = fs.realpath(file_str);
    assert!(real.ends_with("hello.txt"), "realpath was {real:?}");

    // Cleanup via the trait's remove.
    fs.remove(file_str).unwrap();
    assert!(!fs.file_exists(file_str));

    let _ = std::fs::remove_dir_all(&tmp);
}
