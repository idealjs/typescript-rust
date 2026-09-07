use super::*;

#[test]
fn test_new_known_symlink() {
    let cache = KnownSymlinks::new("/test/dir", true);
    assert_eq!(cache.cwd, "/test/dir");
    assert!(cache.use_case_sensitive_file_names);
}

#[test]
fn test_set_directory() {
    let cache = KnownSymlinks::new("/test/dir", true);
    let symlink_path =
        tspath::to_path("/test/symlink", "/test/dir", true).ensure_trailing_directory_separator();
    let real_directory = KnownDirectoryLink {
        real: "/real/path/".to_string(),
        real_path: tspath::to_path("/real/path", "/test/dir", true)
            .ensure_trailing_directory_separator(),
    };

    cache.set_directory(
        "/test/symlink",
        symlink_path.clone(),
        real_directory.clone(),
    );

    let stored = cache.directories().load(&symlink_path);
    assert!(stored.is_some(), "Expected directory to be stored");
    let stored = stored.unwrap();
    assert_eq!(stored.real, real_directory.real);
    assert_eq!(stored.real_path, real_directory.real_path);

    let set = cache
        .directories_by_realpath()
        .load(&real_directory.real_path);
    assert!(
        set.is_some() && !set.as_ref().unwrap().lock().unwrap().is_empty(),
        "Expected realpath mapping to be created"
    );
    assert!(
        set.unwrap().lock().unwrap().contains("/test/symlink"),
        "Expected symlink '/test/symlink' to be in set"
    );
}

#[test]
fn test_set_file() {
    let cache = KnownSymlinks::new("/test/dir", true);
    let symlink = "/test/symlink/file.ts";
    let symlink_path = tspath::to_path(symlink, "/test/dir", true);
    let realpath = "/real/path/file.ts";

    cache.set_file(symlink, symlink_path.clone(), realpath);

    let stored = cache.files().load(&symlink_path);
    assert!(stored.is_some(), "Expected file to be stored");
    assert_eq!(stored.unwrap(), realpath);
}

#[test]
fn test_process_resolution() {
    let cache = KnownSymlinks::new("/test/dir", true);

    cache.process_resolution("", "");
    cache.process_resolution("original", "");
    cache.process_resolution("", "resolved");

    let original_path = "/test/original/file.ts";
    let resolved_path = "/test/resolved/file.ts";
    cache.process_resolution(original_path, resolved_path);

    let symlink_path = tspath::to_path(original_path, "/test/dir", true);
    let stored = cache.files().load(&symlink_path);
    assert!(stored.is_some(), "Expected file to be stored");
    assert_eq!(stored.unwrap(), resolved_path);
}

#[test]
fn test_guess_directory_symlink() {
    let cache = KnownSymlinks::new("/test/dir", true);

    let cases: &[(&str, &str, &str, &str, &str, &str)] = &[
        (
            "identical paths",
            "/test/path/file.ts",
            "/test/path/file.ts",
            "/test/dir",
            "/",
            "/",
        ),
        (
            "different files same directory",
            "/test/path/file1.ts",
            "/test/path/file2.ts",
            "/test/dir",
            "",
            "",
        ),
        (
            "different directories",
            "/test/path1/file.ts",
            "/test/path2/file.ts",
            "/test/dir",
            "/test/path1",
            "/test/path2",
        ),
        (
            "node_modules paths",
            "/test/node_modules/pkg/file.ts",
            "/test/node_modules/pkg/file.ts",
            "/test/dir",
            "/test/node_modules/pkg",
            "/test/node_modules/pkg",
        ),
        (
            "scoped package paths",
            "/test/node_modules/@scope/pkg/file.ts",
            "/test/node_modules/@scope/pkg/file.ts",
            "/test/dir",
            "/test/node_modules/@scope/pkg",
            "/test/node_modules/@scope/pkg",
        ),
    ];

    for (name, a, b, cwd, expected_resolved, expected_original) in cases {
        let (common_resolved, common_original) = cache.guess_directory_symlink(a, b, cwd);
        assert_eq!(
            common_resolved, *expected_resolved,
            "{name}: expected common_resolved to be '{expected_resolved}', got '{common_resolved}'"
        );
        assert_eq!(
            common_original, *expected_original,
            "{name}: expected common_original to be '{expected_original}', got '{common_original}'"
        );
    }
}

#[test]
fn test_is_node_modules_or_scoped_package_directory() {
    let cache = KnownSymlinks::new("/test/dir", true);

    let cases: &[(&str, &str, bool)] = &[
        ("node_modules", "node_modules", true),
        ("scoped package", "@scope", true),
        ("regular directory", "src", false),
        ("empty string", "", false),
        ("case insensitive node_modules", "NODE_MODULES", false),
        ("case insensitive scoped", "@SCOPE", true),
    ];

    for (name, dir, expected) in cases {
        let result = cache.is_node_modules_or_scoped_package_directory(dir);
        assert_eq!(
            result, *expected,
            "{name}: expected {expected}, got {result} for directory '{dir}'"
        );
    }
}

#[test]
fn test_set_symlinks_from_resolutions() {
    let cache = KnownSymlinks::new("/test/dir", true);

    let resolved_modules: &[(&str, &str)] = &[
        ("/test/original/file1.ts", "/test/resolved/file1.ts"),
        ("/test/original/file2.ts", "/test/resolved/file2.ts"),
    ];

    cache.set_symlinks_from_resolutions(
        |cb| {
            for &(original, resolved) in resolved_modules {
                cb(original, resolved);
            }
        },
        |_| {},
    );

    for &(original, resolved) in resolved_modules {
        let symlink_path = tspath::to_path(original, "/test/dir", true);
        let stored = cache.files().load(&symlink_path);
        assert!(stored.is_some(), "Expected file '{original}' to be stored");
        assert_eq!(stored.unwrap(), resolved);
    }
}

#[test]
fn test_known_symlinks_thread_safety() {
    use std::thread;

    let cache = KnownSymlinks::new("/test/dir", true);

    thread::scope(|s| {
        for id in 0..10u32 {
            let cache_ref = &cache;
            s.spawn(move || {
                let symlink = format!("/test/symlink{id}");
                let symlink_path = tspath::to_path(&symlink, "/test/dir", true)
                    .ensure_trailing_directory_separator();
                let real_directory = KnownDirectoryLink {
                    real: format!("/real/path{id}/"),
                    real_path: tspath::to_path(&format!("/real/path{id}"), "/test/dir", true)
                        .ensure_trailing_directory_separator(),
                };

                cache_ref.set_directory(&symlink, symlink_path.clone(), real_directory.clone());

                let stored = cache_ref.directories().load(&symlink_path);
                assert!(
                    stored.is_some(),
                    "Goroutine {id}: Expected directory to be stored"
                );
                assert_eq!(
                    stored.unwrap().real,
                    real_directory.real,
                    "Goroutine {id}: Expected Real to be '{}'",
                    real_directory.real
                );
            });
        }
    });

    assert_eq!(
        cache.directories().len(),
        10,
        "Expected 10 directories to be stored"
    );
}
