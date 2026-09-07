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
    assert_eq!(info.size, 19);
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

    fs.insert_file("/src/a.ts", "a");
    let entries = fs.get_accessible_entries("/src/");
    assert_eq!(entries.files, vec!["a.ts"]);
}

#[test]
fn os_fs_basic_exists() {
    let fs = OsFS;

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

    #[cfg(target_os = "windows")]
    assert!(!fs.use_case_sensitive_file_names());
    #[cfg(not(target_os = "windows"))]
    assert!(fs.use_case_sensitive_file_names());
}
