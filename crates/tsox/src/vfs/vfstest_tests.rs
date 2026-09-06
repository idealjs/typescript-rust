use super::*;
use std::collections::HashSet;
use std::sync::Arc;

fn is_windows_rooted(path: &str) -> bool {
    let b = path.as_bytes();
    b.len() >= 3 && b[1] == b':' && (b[2] == b'/' || b[2] == b'\\') && b[0].is_ascii_alphabetic()
}

fn is_normalized(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }

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

fn from_map(files: &[(&str, &str)], case_sensitive: bool) -> InMemoryFS {
    let fs = InMemoryFS::with_case_sensitivity(case_sensitive);

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

#[test]
fn test_insensitive() {

    let contents = "bar";
    let fs = from_map(
        &[
            ("/foo/bar/baz", contents),
            ("/foo/bar2/baz2", contents),
            ("/foo/bar3/baz3", contents),
        ],
        false,
    );

    assert_eq!(fs.read_file("/foo/bar/baz"), Some(contents.to_string()));
    assert!(fs.stat("/foo/bar/baz").is_some());
    assert_eq!(fs.realpath("/foo/bar/baz"), "/foo/bar/baz");

    let entries = fs.get_accessible_entries("/foo");
    assert_eq!(entries.directories, vec!["bar", "bar2", "bar3"]);

    assert_eq!(fs.read_file("/Foo/Bar/Baz"), Some(contents.to_string()));
    assert_eq!(fs.realpath("/Foo/Bar/Baz"), "/foo/bar/baz");
}

#[test]
fn test_insensitive_upper() {

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

    assert_eq!(fs.read_file("/foo/bar/baz"), Some(contents.to_string()));
    assert!(fs.stat("/foo/bar/baz").is_some());
    assert_eq!(fs.realpath("/foo/bar/baz"), "/foo/bar/baz");

    let entries = fs.get_accessible_entries("/foo");
    assert_eq!(entries.directories, vec!["bar", "bar2", "bar3"]);

    assert_eq!(fs.read_file("/Foo/Bar/Baz"), None);

    assert_eq!(fs.realpath("/does/not/exist"), "/does/not/exist");
    assert!(fs.stat("/does/not/exist").is_none());
}

#[test]
#[should_panic(expected = "duplicate path")]
fn test_sensitive_duplicate_path() {

    let _fs = from_map(&[("/foo", "bar"), ("/Foo", "baz")], false);
}

#[test]
fn test_insensitive_duplicate_path() {

    let fs = from_map(&[("/foo", "bar"), ("/Foo", "baz")], true);
    assert_eq!(fs.read_file("/foo"), Some("bar".to_string()));
    assert_eq!(fs.read_file("/Foo"), Some("baz".to_string()));
}

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

    let fs = InMemoryFS::with_case_sensitivity(false);
    fs.write_file("/foo/bar/baz", "hello, world").unwrap();

    let err = fs.write_file("/foo/bar/baz/oops", "goodbye, world");
    assert!(
        err.is_err(),
        "writing under a file path should fail, got {err:?}"
    );
}

#[test]
fn test_writable_fs_delete() {
    let fs = InMemoryFS::with_case_sensitivity(false);

    fs.write_file("/foo/bar/file.ts", "remove").unwrap();
    fs.insert_dir("/foo/bar");
    assert!(fs.file_exists("/foo/bar/file.ts"));
    fs.remove("/foo/bar/file.ts").unwrap();
    assert!(!fs.file_exists("/foo/bar/file.ts"));

    fs.remove("/foo/bar/test").unwrap();
    fs.remove("/foo/bar/file.ts").unwrap();

    fs.write_file("/foo/barbar", "remove2").unwrap();
    fs.remove("/foo/bar").unwrap();
    assert!(fs.file_exists("/foo/barbar"));
}

#[test]
fn test_writable_fs_delete_directory_recursive() {

    let fs = InMemoryFS::with_case_sensitivity(false);
    fs.write_file("/foo/bar/test/remove2.ts", "remove2")
        .unwrap();
    fs.insert_dir("/foo/bar/test");
    assert!(fs.directory_exists("/foo/bar/test"));
    fs.remove("/foo/bar/test").unwrap();
    assert!(!fs.directory_exists("/foo/bar/test"));
    assert!(!fs.file_exists("/foo/bar/test/remove2.ts"));
}

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

#[test]
#[should_panic(expected = "not a directory")]
fn test_parent_dir_file() {

    let _fs = from_map(&[("/foo", "bar"), ("/foo/oops", "baz")], false);
}

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

    let _fs = from_map(&[("/string", "x"), ("c:/bytes", "x")], false);
}

#[test]
#[should_panic(expected = "non-rooted path")]
fn test_from_map_non_rooted() {

    let _fs = from_map(&[("string", "x")], false);
}

#[test]
#[should_panic(expected = "non-normalized path")]
fn test_from_map_non_normalized() {

    let _fs = from_map(&[("/string/", "x")], false);
}

#[test]
#[should_panic(expected = "non-normalized path")]
fn test_from_map_non_normalized2() {

    let _fs = from_map(&[("/string/../foo", "x")], false);
}

#[test]
fn test_from_map_invalid_file() {

    let fs = from_map(&[("/a", "1"), ("/b", "text")], true);
    assert_eq!(fs.read_file("/a"), Some("1".to_string()));
    assert_eq!(fs.read_file("/b"), Some("text".to_string()));
}

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

    assert_eq!(fs.read_file("/foo.ts"), Some("hello, world".to_string()));
    assert_eq!(fs.read_file("/does/not/exist.ts"), None);

    assert_eq!(fs.realpath("/foo.ts"), "/foo.ts");

    assert_eq!(fs.realpath("/does/not/exist.ts"), "/does/not/exist.ts");

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

#[test]
fn test_bom() {

    let expected = "hello, world";
    let fs = from_map(&[("/foo.ts", "\u{FEFF}hello, world")], true);
    assert_eq!(fs.read_file("/foo.ts"), Some(expected.to_string()));
}

#[test]
fn test_symlink() {

    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.insert_file("/foo.ts", "hello, world");
    fs.insert_dir("/dir");
    fs.insert_file("/dir/file.ts", "export const x = 1;");
    fs.create_symlink("/link.ts", "/foo.ts");
    fs.create_symlink("/dirlink", "/dir");

    assert_eq!(fs.read_file("/link.ts"), Some("hello, world".to_string()));

    assert_eq!(fs.realpath("/link.ts"), "/foo.ts");

    assert!(fs.file_exists("/link.ts"));

    assert!(fs.directory_exists("/dirlink"));
    assert_eq!(fs.realpath("/dirlink"), "/dir");

    let entries = fs.get_accessible_entries("/dirlink");
    assert!(entries.files.contains(&"file.ts".to_string()));

    assert_eq!(
        fs.read_file("/dirlink/file.ts"),
        Some("export const x = 1;".to_string())
    );
}

#[test]
fn test_writable_fs_symlink() {

    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.write_file("/foo", "hello").unwrap();
    fs.create_symlink("/link", "/foo");

    fs.write_file("/link", "goodbye").unwrap();
    assert_eq!(fs.read_file("/foo"), Some("goodbye".to_string()));
    assert_eq!(fs.read_file("/link"), Some("goodbye".to_string()));

    fs.create_symlink("/broken", "/missing");
    assert_eq!(fs.read_file("/broken"), None);
    assert!(!fs.file_exists("/broken"));
    assert_eq!(fs.realpath("/broken"), "/missing");
}

#[test]
fn test_writable_fs_symlink_chain() {

    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.write_file("/d", "x").unwrap();
    fs.create_symlink("/a", "/b");
    fs.create_symlink("/b", "/c");
    fs.create_symlink("/c", "/d");

    fs.write_file("/a", "hello").unwrap();
    assert_eq!(fs.read_file("/d"), Some("hello".to_string()));
    assert_eq!(fs.realpath("/a"), "/d");
    assert!(fs.file_exists("/a"));
}

#[test]
fn test_writable_fs_symlink_chain_not_dir() {

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

    let fs = InMemoryFS::with_case_sensitivity(true);
    fs.write_file("/foo", "hello").unwrap();
    fs.create_symlink("/link", "/foo");

    fs.remove("/link").unwrap();
    assert_eq!(fs.read_symlink("/link"), None);
    assert!(fs.file_exists("/foo"));
    assert_eq!(fs.read_file("/foo"), Some("hello".to_string()));

    fs.create_symlink("/link", "/foo");
    fs.remove("/foo").unwrap();
    assert_eq!(fs.read_file("/link"), None);
    assert!(!fs.file_exists("/link"));
}
