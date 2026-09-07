use super::*;

fn test_temp_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tsox_vfs_test_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(&dir).unwrap_or(dir)
}

#[test]
fn test_os_read_file() {
    let fs = OsFS;
    let cargo_toml = format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"));
    let expected = std::fs::read_to_string(&cargo_toml).unwrap();
    let contents = fs.read_file(&cargo_toml);
    assert_eq!(contents, Some(expected));
}

#[test]
fn test_os_realpath() {
    let fs = OsFS;

    if let Ok(home) = std::env::var("HOME") {
        let realpath = fs.realpath(&home);

        assert!(realpath.starts_with('/'));
    }
}

#[test]
fn test_os_use_case_sensitive_file_names() {
    let fs = OsFS;
    #[cfg(target_os = "windows")]
    assert!(!fs.use_case_sensitive_file_names());
    #[cfg(not(target_os = "windows"))]
    assert!(fs.use_case_sensitive_file_names());
}

#[cfg(unix)]
#[test]
fn test_symlink_realpath() {
    use std::os::unix::fs::symlink;

    let tmp = test_temp_dir("symlink_rp");
    let target = tmp.join("target");
    let link = tmp.join("link");

    let target_file = target.join("file");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(&target_file, "hello").unwrap();

    symlink(&target, &link).unwrap();

    let link_file = link.join("file");

    let contents = std::fs::read_to_string(&link_file).unwrap();
    assert_eq!(contents, "hello");

    let fs = OsFS;
    let target_realpath = fs.realpath(target_file.to_str().unwrap());
    let link_realpath = fs.realpath(link_file.to_str().unwrap());

    assert_eq!(link_realpath, target_realpath);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[cfg(unix)]
#[test]
fn test_get_accessible_entries() {
    use std::os::unix::fs::symlink;

    let tmp = test_temp_dir("gae");
    let target = tmp.join("target");
    let link = tmp.join("link");

    std::fs::create_dir_all(&target).unwrap();
    std::fs::create_dir_all(&link).unwrap();

    let target_file1 = target.join("file1");
    let target_file2 = target.join("file2");
    std::fs::write(&target_file1, "hello").unwrap();
    std::fs::write(&target_file2, "world").unwrap();

    let target_dir1 = target.join("dir1");
    let target_dir2 = target.join("dir2");
    std::fs::create_dir_all(&target_dir1).unwrap();
    std::fs::create_dir_all(&target_dir2).unwrap();

    symlink(&target_file1, link.join("file1")).unwrap();
    symlink(&target_file2, link.join("file2")).unwrap();
    symlink(&target_dir1, link.join("dir1")).unwrap();
    symlink(&target_dir2, link.join("dir2")).unwrap();

    let fs = OsFS;
    let entries = fs.get_accessible_entries(link.to_str().unwrap());

    assert_eq!(entries.directories, vec!["dir1", "dir2"]);
    assert_eq!(entries.files, vec!["file1", "file2"]);

    assert_eq!(entries.symlinks.len(), 4);
    for name in &["file1", "file2", "dir1", "dir2"] {
        assert!(
            entries.symlinks.iter().any(|s| s == name),
            "expected '{}' in symlinks",
            name
        );
    }

    let entries2 = fs.get_accessible_entries(target.to_str().unwrap());
    assert_eq!(entries2.directories, vec!["dir1", "dir2"]);
    assert_eq!(entries2.files, vec!["file1", "file2"]);
    assert!(entries2.symlinks.is_empty());

    let _ = std::fs::remove_dir_all(&tmp);
}
