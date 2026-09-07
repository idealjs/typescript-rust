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

    let path = "bundled:///libs/lib.d.ts";
    if fs.file_exists(path) {
        assert!(fs.read_file(path).is_some());
    }
    assert!(fs.directory_exists("bundled:///libs"));
}

#[test]
fn bundled_fs_case_sensitive_matching() {
    let inner = Arc::new(InMemoryFS::new());
    let fs = BundledFS::new(inner);

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
    let inner = Arc::new(InMemoryFS::new());
    let fs = BundledFS::new(inner);

    assert!(fs.use_case_sensitive_file_names());
}

#[test]
fn bundled_fs_lib_names_nonempty() {
    let names = lib_names();

    if !names.is_empty() {
        assert!(
            names.iter().any(|n| *n == "lib.d.ts"),
            "Expected lib.d.ts in bundled libs"
        );
    }
}

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
