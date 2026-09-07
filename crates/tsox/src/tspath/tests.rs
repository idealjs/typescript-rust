use super::*;

#[test]
fn test_normalize_slashes() {
    assert_eq!(normalize_slashes("a"), "a");
    assert_eq!(normalize_slashes("a/b"), "a/b");
    assert_eq!(normalize_slashes("a\\b"), "a/b");
    assert_eq!(normalize_slashes("\\\\server\\path"), "//server/path");
    assert_eq!(normalize_slashes("a\\b\\c"), "a/b/c");
}

#[test]
fn test_get_root_length() {
    assert_eq!(get_root_length("a"), 0);
    assert_eq!(get_root_length("/"), 1);
    assert_eq!(get_root_length("/path"), 1);
    assert_eq!(get_root_length("c:"), 2);
    assert_eq!(get_root_length("c:d"), 0);
    assert_eq!(get_root_length("c:/"), 3);
    assert_eq!(get_root_length("c:\\"), 3);
    assert_eq!(get_root_length("//server"), 8);
    assert_eq!(get_root_length("//server/share"), 9);
    assert_eq!(get_root_length("\\\\server"), 8);
    assert_eq!(get_root_length("\\\\server\\share"), 9);
    assert_eq!(get_root_length("file:///"), 8);
    assert_eq!(get_root_length("file:///path"), 8);
    assert_eq!(get_root_length("file:///c:"), 10);
    assert_eq!(get_root_length("file:///c:d"), 8);
    assert_eq!(get_root_length("file:///c:/path"), 11);
    assert_eq!(get_root_length("file://localhost"), 16);
    assert_eq!(get_root_length("file://localhost/"), 17);
    assert_eq!(get_root_length("file://localhost/path"), 17);
    assert_eq!(get_root_length("file://server"), 13);
    assert_eq!(get_root_length("file://server/"), 14);
    assert_eq!(get_root_length("file://server/path"), 14);
    assert_eq!(get_root_length("http://server"), 13);
    assert_eq!(get_root_length("http://server/path"), 14);
}

#[test]
fn test_path_is_absolute() {
    assert!(path_is_absolute("/path/to/file.ext"));
    assert!(path_is_absolute("c:/path/to/file.ext"));
    assert!(path_is_absolute("file:///path/to/file.ext"));
    assert!(!path_is_absolute("path/to/file.ext"));
    assert!(!path_is_absolute("./path/to/file.ext"));
}

#[test]
fn test_is_url() {
    assert!(!is_url("a"));
    assert!(!is_url("/"));
    assert!(!is_url("c:"));
    assert!(!is_url("c:d"));
    assert!(!is_url("c:/"));
    assert!(!is_url("c:\\"));
    assert!(!is_url("//server"));
    assert!(!is_url("//server/share"));
    assert!(!is_url("\\\\server"));
    assert!(!is_url("\\\\server\\share"));

    assert!(is_url("file:///path"));
    assert!(is_url("file:///c:"));
    assert!(is_url("file:///c:d"));
    assert!(is_url("file:///c:/path"));
    assert!(is_url("file://server"));
    assert!(is_url("file://server/path"));
    assert!(is_url("http://server"));
    assert!(is_url("http://server/path"));
}

#[test]
fn test_is_rooted_disk_path() {
    assert!(!is_rooted_disk_path("a"));
    assert!(is_rooted_disk_path("/"));
    assert!(is_rooted_disk_path("c:"));
    assert!(!is_rooted_disk_path("c:d"));
    assert!(is_rooted_disk_path("c:/"));
    assert!(is_rooted_disk_path("c:\\"));
    assert!(is_rooted_disk_path("//server"));
    assert!(is_rooted_disk_path("//server/share"));
    assert!(is_rooted_disk_path("\\\\server"));
    assert!(is_rooted_disk_path("\\\\server\\share"));
    assert!(!is_rooted_disk_path("file:///path"));
    assert!(!is_rooted_disk_path("file:///c:"));
    assert!(!is_rooted_disk_path("file://server"));
    assert!(!is_rooted_disk_path("http://server"));
}

#[test]
fn test_get_directory_path() {
    assert_eq!(get_directory_path(""), "");
    assert_eq!(get_directory_path("a"), "");
    assert_eq!(get_directory_path("a/b"), "a");
    assert_eq!(get_directory_path("/"), "/");
    assert_eq!(get_directory_path("/a"), "/");
    assert_eq!(get_directory_path("/a/"), "/");
    assert_eq!(get_directory_path("/a/b"), "/a");
    assert_eq!(get_directory_path("/a/b/"), "/a");
    assert_eq!(get_directory_path("c:"), "c:");
    assert_eq!(get_directory_path("c:d"), "");
    assert_eq!(get_directory_path("c:/"), "c:/");
    assert_eq!(get_directory_path("c:/path"), "c:/");
    assert_eq!(get_directory_path("c:/path/"), "c:/");
    assert_eq!(get_directory_path("//server"), "//server");
    assert_eq!(get_directory_path("//server/"), "//server/");
    assert_eq!(get_directory_path("//server/share"), "//server/");
    assert_eq!(get_directory_path("//server/share/"), "//server/");
    assert_eq!(get_directory_path("\\\\server"), "//server");
    assert_eq!(get_directory_path("\\\\server\\"), "//server/");
    assert_eq!(get_directory_path("\\\\server\\share"), "//server/");
    assert_eq!(get_directory_path("file:///"), "file:///");
    assert_eq!(get_directory_path("file:///path"), "file:///");
    assert_eq!(get_directory_path("file:///c:"), "file:///c:");
    assert_eq!(get_directory_path("file:///c:d"), "file:///");
    assert_eq!(get_directory_path("file:///c:/"), "file:///c:/");
    assert_eq!(get_directory_path("file:///c:/path"), "file:///c:/");
    assert_eq!(get_directory_path("file://server"), "file://server");
    assert_eq!(get_directory_path("file://server/"), "file://server/");
    assert_eq!(get_directory_path("file://server/path"), "file://server/");
    assert_eq!(get_directory_path("http://server"), "http://server");
    assert_eq!(get_directory_path("http://server/"), "http://server/");
    assert_eq!(get_directory_path("http://server/path"), "http://server/");
}

#[test]
fn test_get_path_components() {
    assert_eq!(get_path_components("", ""), vec![""]);
    assert_eq!(get_path_components("a", ""), vec!["", "a"]);
    assert_eq!(get_path_components("./a", ""), vec!["", ".", "a"]);
    assert_eq!(get_path_components("/", ""), vec!["/"]);
    assert_eq!(get_path_components("/a", ""), vec!["/", "a"]);
    assert_eq!(get_path_components("/a/", ""), vec!["/", "a"]);
    assert_eq!(get_path_components("c:", ""), vec!["c:"]);
    assert_eq!(get_path_components("c:d", ""), vec!["", "c:d"]);
    assert_eq!(get_path_components("c:/", ""), vec!["c:/"]);
    assert_eq!(get_path_components("c:/path", ""), vec!["c:/", "path"]);
    assert_eq!(get_path_components("//server", ""), vec!["//server"]);
    assert_eq!(get_path_components("//server/", ""), vec!["//server/"]);
    assert_eq!(
        get_path_components("//server/share", ""),
        vec!["//server/", "share"]
    );
    assert_eq!(get_path_components("file:///", ""), vec!["file:///"]);
    assert_eq!(
        get_path_components("file:///path", ""),
        vec!["file:///", "path"]
    );
    assert_eq!(get_path_components("file:///c:", ""), vec!["file:///c:"]);
    assert_eq!(
        get_path_components("file:///c:d", ""),
        vec!["file:///", "c:d"]
    );
    assert_eq!(get_path_components("file:///c:/", ""), vec!["file:///c:/"]);
    assert_eq!(
        get_path_components("file:///c:/path", ""),
        vec!["file:///c:/", "path"]
    );
    assert_eq!(
        get_path_components("file://server", ""),
        vec!["file://server"]
    );
    assert_eq!(
        get_path_components("file://server/", ""),
        vec!["file://server/"]
    );
    assert_eq!(
        get_path_components("file://server/path", ""),
        vec!["file://server/", "path"]
    );
    assert_eq!(
        get_path_components("http://server", ""),
        vec!["http://server"]
    );
    assert_eq!(
        get_path_components("http://server/", ""),
        vec!["http://server/"]
    );
    assert_eq!(
        get_path_components("http://server/path", ""),
        vec!["http://server/", "path"]
    );
}

#[test]
fn test_combine_paths() {
    assert_eq!(
        combine_paths("path", &["to", "file.ext"]),
        "path/to/file.ext"
    );
    assert_eq!(
        combine_paths("path", &["dir", "..", "to", "file.ext"]),
        "path/dir/../to/file.ext"
    );

    assert_eq!(
        combine_paths("/path", &["to", "file.ext"]),
        "/path/to/file.ext"
    );
    assert_eq!(combine_paths("/path", &["/to", "file.ext"]), "/to/file.ext");

    assert_eq!(
        combine_paths("c:/path", &["to", "file.ext"]),
        "c:/path/to/file.ext"
    );
    assert_eq!(
        combine_paths("c:/path", &["c:/to", "file.ext"]),
        "c:/to/file.ext"
    );

    assert_eq!(
        combine_paths("file:///path", &["to", "file.ext"]),
        "file:///path/to/file.ext"
    );
    assert_eq!(
        combine_paths("file:///path", &["file:///to", "file.ext"]),
        "file:///to/file.ext"
    );

    assert_eq!(
        combine_paths("/", &["/node_modules/@types"]),
        "/node_modules/@types"
    );
    assert_eq!(combine_paths("/a/..", &[""]), "/a/..");
    assert_eq!(combine_paths("/a/..", &["b"]), "/a/../b");
    assert_eq!(combine_paths("/a/..", &["b/"]), "/a/../b/");
    assert_eq!(combine_paths("/a/..", &["/"]), "/");
    assert_eq!(combine_paths("/a/..", &["/b"]), "/b");
}

#[test]
fn test_resolve_path() {
    assert_eq!(resolve_path("", &[]), "");
    assert_eq!(resolve_path(".", &[]), "");
    assert_eq!(resolve_path("./", &[]), "");
    assert_eq!(resolve_path("..", &[]), "..");
    assert_eq!(resolve_path("../", &[]), "../");
    assert_eq!(resolve_path("/", &[]), "/");
    assert_eq!(resolve_path("/.", &[]), "/");
    assert_eq!(resolve_path("/./", &[]), "/");
    assert_eq!(resolve_path("/../", &[]), "/");
    assert_eq!(resolve_path("/a", &[]), "/a");
    assert_eq!(resolve_path("/a/", &[]), "/a/");
    assert_eq!(resolve_path("/a/.", &[]), "/a");
    assert_eq!(resolve_path("/a/./", &[]), "/a/");
    assert_eq!(resolve_path("/a/./b", &[]), "/a/b");
    assert_eq!(resolve_path("/a/./b/", &[]), "/a/b/");
    assert_eq!(resolve_path("/a/..", &[]), "/");
    assert_eq!(resolve_path("/a/../", &[]), "/");
    assert_eq!(resolve_path("/a/../b", &[]), "/b");
    assert_eq!(resolve_path("/a/../b/", &[]), "/b/");
    assert_eq!(resolve_path("/a/..", &["b"]), "/b");
    assert_eq!(resolve_path("/a/..", &["/"]), "/");
    assert_eq!(resolve_path("/a/..", &["b/"]), "/b/");
    assert_eq!(resolve_path("/a/..", &["/b"]), "/b");
    assert_eq!(resolve_path("/a/.", &["b"]), "/a/b");
    assert_eq!(resolve_path("/a/.", &["."]), "/a");
    assert_eq!(resolve_path("a", &["b", "c"]), "a/b/c");
    assert_eq!(resolve_path("a", &["b", "/c"]), "/c");
    assert_eq!(resolve_path("a", &["b", "../c"]), "a/c");
}

#[test]
fn test_get_normalized_absolute_path() {
    assert_eq!(get_normalized_absolute_path("/", ""), "/");
    assert_eq!(get_normalized_absolute_path("/.", ""), "/");
    assert_eq!(get_normalized_absolute_path("/./", ""), "/");
    assert_eq!(get_normalized_absolute_path("/../", ""), "/");
    assert_eq!(get_normalized_absolute_path("/a", ""), "/a");
    assert_eq!(get_normalized_absolute_path("/a/", ""), "/a");
    assert_eq!(get_normalized_absolute_path("/a/.", ""), "/a");
    assert_eq!(get_normalized_absolute_path("/a/foo.", ""), "/a/foo.");
    assert_eq!(get_normalized_absolute_path("/a/./", ""), "/a");
    assert_eq!(get_normalized_absolute_path("/a/./b", ""), "/a/b");
    assert_eq!(get_normalized_absolute_path("/a/./b/", ""), "/a/b");
    assert_eq!(get_normalized_absolute_path("/a/..", ""), "/");
    assert_eq!(get_normalized_absolute_path("/a/../", ""), "/");
    assert_eq!(get_normalized_absolute_path("/a/../b", ""), "/b");
    assert_eq!(get_normalized_absolute_path("/a/../b/", ""), "/b");
    assert_eq!(get_normalized_absolute_path("/a/..", "/"), "/");
    assert_eq!(get_normalized_absolute_path("/a/..", "b/"), "/");
    assert_eq!(get_normalized_absolute_path("/a/..", "/b"), "/");
    assert_eq!(get_normalized_absolute_path("/a/.", "b"), "/a");
    assert_eq!(get_normalized_absolute_path("/a/.", "."), "/a");

    assert_eq!(get_normalized_absolute_path("\\", ""), "/");
    assert_eq!(get_normalized_absolute_path("\\.", ""), "/");
    assert_eq!(get_normalized_absolute_path("\\.\\", ""), "/");
    assert_eq!(get_normalized_absolute_path("\\..\\", ""), "/");
    assert_eq!(get_normalized_absolute_path("\\a\\.\\", ""), "/a");
    assert_eq!(get_normalized_absolute_path("\\a\\.\\b", ""), "/a/b");
    assert_eq!(get_normalized_absolute_path("\\a\\.\\b\\", ""), "/a/b");
    assert_eq!(get_normalized_absolute_path("\\a\\..", ""), "/");
    assert_eq!(get_normalized_absolute_path("\\a\\..\\", ""), "/");
    assert_eq!(get_normalized_absolute_path("\\a\\..\\b", ""), "/b");
    assert_eq!(get_normalized_absolute_path("\\a\\..\\b\\", ""), "/b");
    assert_eq!(get_normalized_absolute_path("\\a\\..", "\\"), "/");
    assert_eq!(get_normalized_absolute_path("\\a\\..", "b\\"), "/");
    assert_eq!(get_normalized_absolute_path("\\a\\..", "\\b"), "/");
    assert_eq!(get_normalized_absolute_path("\\a\\.", "b"), "/a");
    assert_eq!(get_normalized_absolute_path("\\a\\.", "."), "/a");

    assert_eq!(get_normalized_absolute_path("", ""), "");
    assert_eq!(get_normalized_absolute_path(".", ""), "");
    assert_eq!(get_normalized_absolute_path("./", ""), "");
    assert_eq!(get_normalized_absolute_path("..", ""), "..");
    assert_eq!(get_normalized_absolute_path("../", ""), "..");

    assert_eq!(get_normalized_absolute_path("", "/home"), "/home");
    assert_eq!(get_normalized_absolute_path(".", "/home"), "/home");
    assert_eq!(get_normalized_absolute_path("./", "/home"), "/home");
    assert_eq!(get_normalized_absolute_path("..", "/home"), "/");
    assert_eq!(get_normalized_absolute_path("../", "/home"), "/");
    assert_eq!(get_normalized_absolute_path("a", "b"), "b/a");
    assert_eq!(get_normalized_absolute_path("a", "b/c"), "b/c/a");

    assert_eq!(get_normalized_absolute_path(".a", ""), ".a");
    assert_eq!(get_normalized_absolute_path("..a", ""), "..a");
    assert_eq!(get_normalized_absolute_path("a.", ""), "a.");
    assert_eq!(get_normalized_absolute_path("a..", ""), "a..");

    assert_eq!(get_normalized_absolute_path("/base/./.a", ""), "/base/.a");
    assert_eq!(get_normalized_absolute_path("/base/../.a", ""), "/.a");
    assert_eq!(get_normalized_absolute_path("/base/./..a", ""), "/base/..a");
    assert_eq!(get_normalized_absolute_path("/base/../..a", ""), "/..a");
    assert_eq!(
        get_normalized_absolute_path("/base/./..a/b", ""),
        "/base/..a/b"
    );
    assert_eq!(get_normalized_absolute_path("/base/../..a/b", ""), "/..a/b");
    assert_eq!(get_normalized_absolute_path("/base/./a.", ""), "/base/a.");
    assert_eq!(get_normalized_absolute_path("/base/../a.", ""), "/a.");
    assert_eq!(get_normalized_absolute_path("/base/./a..", ""), "/base/a..");
    assert_eq!(get_normalized_absolute_path("/base/../a..", ""), "/a..");
    assert_eq!(
        get_normalized_absolute_path("/base/./a../b", ""),
        "/base/a../b"
    );
    assert_eq!(get_normalized_absolute_path("/base/../a../b", ""), "/a../b");

    assert_eq!(get_normalized_absolute_path("a/..", ""), "");
    assert_eq!(get_normalized_absolute_path("/a//", ""), "/a");
    assert_eq!(get_normalized_absolute_path("a/..", ""), "");

    assert_eq!(get_normalized_absolute_path("a//b", ""), "a/b");
    assert_eq!(get_normalized_absolute_path("a///b", ""), "a/b");
    assert_eq!(get_normalized_absolute_path("a/b//c", ""), "a/b/c");
    assert_eq!(get_normalized_absolute_path("/a/b//c", ""), "/a/b/c");

    assert_eq!(get_normalized_absolute_path("a\\\\b", ""), "a/b");
    assert_eq!(get_normalized_absolute_path("a\\\\\\b", ""), "a/b");
    assert_eq!(get_normalized_absolute_path("a\\b\\\\c", ""), "a/b/c");
    assert_eq!(get_normalized_absolute_path("\\a\\b\\\\c", ""), "/a/b/c");
}

#[test]
fn test_to_file_name_lower_case() {
    assert_eq!(
        to_file_name_lower_case("/user/UserName/projects/Project/file.ts"),
        "/user/username/projects/project/file.ts"
    );
    assert_eq!(
        to_file_name_lower_case("/user/UserName/projects/projectß/file.ts"),
        "/user/username/projects/projectß/file.ts"
    );
}

#[test]
fn test_to_path() {
    assert_eq!(
        to_path("file.ext", "path/to", false).as_str(),
        "path/to/file.ext"
    );
    assert_eq!(
        to_path("file.ext", "/path/to", true).as_str(),
        "/path/to/file.ext"
    );
    assert_eq!(
        to_path("/path/to/../file.ext", "path/to", true).as_str(),
        "/path/file.ext"
    );
}

#[test]
fn test_path_is_relative() {
    assert!(path_is_relative("."));
    assert!(path_is_relative(".."));
    assert!(path_is_relative("./"));
    assert!(path_is_relative("../"));
    assert!(path_is_relative("./foo/bar"));
    assert!(path_is_relative("../foo/bar"));
    assert!(!path_is_relative(""));
    assert!(!path_is_relative("foo"));
    assert!(!path_is_relative("foo/bar"));
    assert!(!path_is_relative("/foo/bar"));
    assert!(!path_is_relative("c:/foo/bar"));
}

#[test]
fn test_is_dynamic_file_name() {
    assert!(is_dynamic_file_name("^/untitled/foo.ts"));
    assert!(!is_dynamic_file_name("/path/to/file.ts"));
    assert!(!is_dynamic_file_name(""));
}

#[test]
fn test_untitled_path_root_length() {
    assert_eq!(get_encoded_root_length("^/untitled"), 2);
    assert_eq!(get_root_length("^/untitled"), 2);

    assert_ne!(get_encoded_root_length("^"), 2);
}

#[test]
fn test_contains_ignored_path() {
    let tests: &[(&str, &str, bool)] = &[
        (
            "node_modules dot path",
            "/project/node_modules/.pnpm/file.ts",
            true,
        ),
        ("git directory", "/project/.git/hooks/pre-commit", true),
        ("emacs lock file", "/project/src/file.ts.#", true),
        ("regular file path", "/project/src/file.ts", false),
        (
            "node_modules without dot",
            "/project/node_modules/lodash/index.js",
            false,
        ),
        ("empty path", "", false),
        (
            "path with multiple ignored patterns",
            "/project/node_modules/.pnpm/.git/.#file.ts",
            true,
        ),
        (
            "case sensitive test",
            "/project/NODE_MODULES/.PNPM/file.ts",
            false,
        ),
        (
            "path with ignored pattern in middle",
            "/project/src/node_modules/.pnpm/dist/file.js",
            true,
        ),
        (
            "path with ignored pattern at end",
            "/project/src/file.ts.#",
            true,
        ),
    ];

    for &(name, path, expected) in tests {
        let result = contains_ignored_path(path);
        assert_eq!(
            result, expected,
            "ContainsIgnoredPath({:?}) = {}, expected {} ({})",
            path, result, expected, name
        );
    }
}

#[test]
fn test_ignored_paths_patterns() {
    let expected_patterns = ["/node_modules/.", "/.git", ".#"];

    for pattern in expected_patterns {
        let test_path = format!("/test{}/file.ts", pattern);
        assert!(
            contains_ignored_path(&test_path),
            "Expected pattern '{}' to be detected in path '{}'",
            pattern,
            test_path
        );
    }
}

#[test]
fn test_ignored_paths_edge_cases() {
    let tests: &[(&str, &str, bool)] = &[
        ("pattern at start", "/node_modules./file.ts", false),
        ("pattern at end", "/project/file.ts.#", true),
        (
            "multiple occurrences",
            "/project/.git/node_modules./.git/file.ts",
            true,
        ),
        ("no slashes", "node_modules.file.ts", false),
        ("single slash", "/file.ts", false),
    ];

    for &(name, path, expected) in tests {
        let result = contains_ignored_path(path);
        assert_eq!(
            result, expected,
            "ContainsIgnoredPath({:?}) = {}, expected {} ({})",
            path, result, expected, name
        );
    }
}

#[test]
fn test_get_base_file_name() {
    assert_eq!(get_base_file_name("/path/to/file.ext"), "file.ext");
    assert_eq!(get_base_file_name("/path/to/"), "to");
    assert_eq!(get_base_file_name("/"), "");
}

#[test]
fn test_normalize_path() {
    assert_eq!(normalize_path("/path/./to/../file.ext"), "/path/file.ext");
    assert_eq!(normalize_path("./file.ext"), "file.ext");
    assert_eq!(normalize_path("path/to/file.ext"), "path/to/file.ext");
}

#[test]
fn test_extension_functions() {
    assert!(has_ts_file_extension("file.ts"));
    assert!(has_ts_file_extension("file.tsx"));
    assert!(has_ts_file_extension("file.d.ts"));
    assert!(!has_ts_file_extension("file.js"));
    assert!(has_js_file_extension("file.js"));
    assert!(is_declaration_file_name("file.d.ts"));
    assert!(!is_declaration_file_name("file.ts"));
    assert_eq!(remove_file_extension("/path/to/file.ts"), "/path/to/file");
    assert_eq!(remove_file_extension("/path/to/file.d.ts"), "/path/to/file");
    assert_eq!(change_extension("file.ts", ".js"), "file.js");
}

#[test]
fn test_trailing_directory_separator() {
    assert!(has_trailing_directory_separator("path/"));
    assert!(has_trailing_directory_separator("path\\"));
    assert!(!has_trailing_directory_separator("path"));
    assert_eq!(ensure_trailing_directory_separator("path"), "path/");
    assert_eq!(ensure_trailing_directory_separator("path/"), "path/");
    assert_eq!(remove_trailing_directory_separator("path/"), "path");
    assert_eq!(remove_trailing_directory_separator("path"), "path");
}

#[test]
fn test_for_each_ancestor_directory() {
    let mut ancestors = Vec::new();
    for_each_ancestor_directory("/a/b/c", |dir| {
        ancestors.push(dir.to_string());
        false
    });
    assert_eq!(ancestors, vec!["/a/b/c", "/a/b", "/a", "/"]);

    let mut ancestors = Vec::new();
    for_each_ancestor_directory("/a/b/c", |dir| {
        ancestors.push(dir.to_string());
        dir == "/a/b"
    });
    assert_eq!(ancestors, vec!["/a/b/c", "/a/b"]);
}

#[test]
fn test_reduce_path_components() {
    assert_eq!(reduce_path_components(&vec!["".to_string()]), vec![""]);
    assert_eq!(
        reduce_path_components(&vec!["".to_string(), ".".to_string()]),
        vec![""]
    );
    assert_eq!(
        reduce_path_components(&vec!["".to_string(), ".".to_string(), "a".to_string()]),
        vec!["", "a"]
    );
    assert_eq!(
        reduce_path_components(&vec!["".to_string(), "a".to_string(), ".".to_string()]),
        vec!["", "a"]
    );
    assert_eq!(
        reduce_path_components(&vec!["".to_string(), "..".to_string()]),
        vec!["", ".."]
    );
    assert_eq!(
        reduce_path_components(&vec!["".to_string(), "..".to_string(), "..".to_string()]),
        vec!["", "..", ".."]
    );
    assert_eq!(
        reduce_path_components(&vec![
            "".to_string(),
            "..".to_string(),
            ".".to_string(),
            "..".to_string()
        ]),
        vec!["", "..", ".."]
    );
    assert_eq!(
        reduce_path_components(&vec!["".to_string(), "a".to_string(), "..".to_string()]),
        vec![""]
    );
    assert_eq!(
        reduce_path_components(&vec!["".to_string(), "..".to_string(), "a".to_string()]),
        vec!["", "..", "a"]
    );
    assert_eq!(reduce_path_components(&vec!["/".to_string()]), vec!["/"]);
    assert_eq!(
        reduce_path_components(&vec!["/".to_string(), ".".to_string()]),
        vec!["/"]
    );
    assert_eq!(
        reduce_path_components(&vec!["/".to_string(), "..".to_string()]),
        vec!["/"]
    );
    assert_eq!(
        reduce_path_components(&vec!["/".to_string(), "a".to_string(), "..".to_string()]),
        vec!["/"]
    );
}

#[test]
fn test_get_normalized_absolute_path_without_root() {
    assert_eq!(
        get_normalized_absolute_path_without_root("/a/b/c.txt", "/a/b"),
        "a/b/c.txt"
    );
    assert_eq!(
        get_normalized_absolute_path_without_root("c:/work/hello.txt", "c:/work"),
        "work/hello.txt"
    );
    assert_eq!(
        get_normalized_absolute_path_without_root("c:/work/hello.txt", "d:/worspaces"),
        "work/hello.txt"
    );
}

#[test]
fn test_get_relative_path_to_directory_or_url() {
    let opts = ComparePathsOptions::default();

    assert_eq!(
        get_relative_path_to_directory_or_url("/", "/", false, &opts),
        ""
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("/a", "/a", false, &opts),
        ""
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("/a/", "/a", false, &opts),
        ""
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("/a", "/", false, &opts),
        ".."
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("/a", "/b", false, &opts),
        "../b"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("/a/b", "/b", false, &opts),
        "../../b"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("/a/b/c", "/b", false, &opts),
        "../../../b"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("/a/b/c", "/b/c", false, &opts),
        "../../../b/c"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("/a/b/c", "/a/b", false, &opts),
        ".."
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("c:", "d:", false, &opts),
        "d:/"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///", "file:///", false, &opts),
        ""
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///a", "file:///a", false, &opts),
        ""
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///a/", "file:///a", false, &opts),
        ""
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///a", "file:///", false, &opts),
        ".."
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///a", "file:///b", false, &opts),
        "../b"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///a/b", "file:///b", false, &opts),
        "../../b"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///a/b/c", "file:///b", false, &opts),
        "../../../b"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///a/b/c", "file:///b/c", false, &opts),
        "../../../b/c"
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///a/b/c", "file:///a/b", false, &opts),
        ".."
    );
    assert_eq!(
        get_relative_path_to_directory_or_url("file:///c:", "file:///d:", false, &opts),
        "file:///d:/"
    );
}

#[test]
fn test_get_common_parents() {
    let opts = ComparePathsOptions::default();

    let (got, ignored) = get_common_parents(&[], 1, &opts);
    assert!(ignored.is_empty());
    assert!(got.is_empty());

    let paths = vec!["/a/b/c/d".to_string()];
    let (got, ignored) = get_common_parents(&paths, 1, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["/a/b/c/d"]);

    let paths = vec![
        "/a/b/c/d".to_string(),
        "/a/b/c/e".to_string(),
        "/a/b/f/g".to_string(),
        "/x/y".to_string(),
    ];
    let (got, ignored) = get_common_parents(&paths, 4, &opts);
    assert_eq!(ignored.len(), 1);
    assert!(ignored.contains("/x/y"));
    assert_eq!(got, vec!["/a/b/c", "/a/b/f/g"]);

    let paths = vec![
        "/a/b/c/d".to_string(),
        "/a/b/c/e".to_string(),
        "/a/b/f/g".to_string(),
    ];
    let (got, ignored) = get_common_parents(&paths, 1, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["/a/b"]);

    let paths = vec![
        "/a/b/c/d".to_string(),
        "/a/b/c/e".to_string(),
        "/a/b/f/g".to_string(),
        "/x/y/z".to_string(),
    ];
    let (got, ignored) = get_common_parents(&paths, 1, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["/"]);

    let paths = vec![
        "/a/b/c/d".to_string(),
        "/a/b/c/e".to_string(),
        "/a/b/f/g".to_string(),
        "/x/y/z".to_string(),
    ];
    let (got, ignored) = get_common_parents(&paths, 3, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["/a/b", "/x/y/z"]);

    let paths = vec!["c:/a/b/c/d".to_string(), "d:/a/b/c/d".to_string()];
    let (got, ignored) = get_common_parents(&paths, 1, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["c:/a/b/c/d", "d:/a/b/c/d"]);

    let paths = vec!["/a/b/c/d".to_string(), "/a/b/c/d".to_string()];
    let (got, ignored) = get_common_parents(&paths, 1, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["/a/b/c/d"]);

    let paths = vec!["/a/b/c/d".to_string(), "/x/y".to_string()];
    let (got, ignored) = get_common_parents(&paths, 2, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["/a/b/c/d", "/x/y"]);

    let paths = vec![
        "/a/b/c/d".to_string(),
        "/a/z/c/e".to_string(),
        "/a/aaa/f/g".to_string(),
        "/x/y/z".to_string(),
    ];
    let (got, ignored) = get_common_parents(&paths, 2, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["/a", "/x/y/z"]);

    let paths = vec!["/a/b/".to_string(), "/a/b/c".to_string()];
    let (got, ignored) = get_common_parents(&paths, 1, &opts);
    assert!(ignored.is_empty());
    assert_eq!(got, vec!["/a/b"]);
}

#[test]
fn test_untitled_path_handling() {
    let untitled_path = "^/untitled/ts-nul-authority/Untitled-2";

    let root_length = get_encoded_root_length(untitled_path);
    assert_eq!(
        root_length, 2,
        "GetEncodedRootLength should return 2 for untitled paths"
    );

    let is_rooted = is_rooted_disk_path(untitled_path);
    assert!(
        is_rooted,
        "IsRootedDiskPath should return true for untitled paths"
    );

    let current_dir = "/home/user/project";
    let path = to_path(untitled_path, current_dir, true);

    assert_eq!(
        path.as_str(),
        "^/untitled/ts-nul-authority/Untitled-2",
        "ToPath should not resolve untitled paths against current directory"
    );

    let normalized = get_normalized_absolute_path(untitled_path, current_dir);
    assert_eq!(
        normalized, "^/untitled/ts-nul-authority/Untitled-2",
        "GetNormalizedAbsolutePath should not resolve untitled paths"
    );
}

#[test]
fn test_untitled_path_edge_cases() {
    let test_cases: &[(&str, i32, bool)] = &[
        ("^/", 2, true),
        ("^/untitled/ts-nul-authority/test", 2, true),
        ("^", 0, false),
        ("^x", 0, false),
        ("^^/", 0, false),
        ("x^/", 0, false),
        (
            "^/untitled/ts-nul-authority/path/with/deeper/structure",
            2,
            true,
        ),
    ];

    for &(path, expected, is_rooted) in test_cases {
        let root_length = get_encoded_root_length(path);
        assert_eq!(
            root_length, expected,
            "GetEncodedRootLength for path {}",
            path
        );

        let result = is_rooted_disk_path(path);
        assert_eq!(result, is_rooted, "IsRootedDiskPath for path {}", path);
    }
}

#[test]
fn test_starts_with_directory() {
    let tests: &[(&str, &str, &str, bool, bool)] = &[
        (
            "exact match case sensitive",
            "/project/src/file.ts",
            "/project/src",
            true,
            true,
        ),
        (
            "exact match case insensitive",
            "/project/src/file.ts",
            "/PROJECT/SRC",
            false,
            true,
        ),
        (
            "case sensitive mismatch",
            "/project/src/file.ts",
            "/PROJECT/SRC",
            true,
            false,
        ),
        (
            "file not in directory",
            "/project/lib/file.ts",
            "/project/src",
            true,
            false,
        ),
        (
            "file in subdirectory",
            "/project/src/components/Button.tsx",
            "/project/src",
            true,
            true,
        ),
        (
            "file in parent directory",
            "/project/file.ts",
            "/project/src",
            true,
            false,
        ),
        (
            "windows style separators",
            "C:\\project\\src\\file.ts",
            "C:\\project\\src",
            true,
            true,
        ),
        (
            "mixed separators",
            "/project/src/file.ts",
            "\\project\\src",
            true,
            false,
        ),
        (
            "empty directory name",
            "/project/src/file.ts",
            "",
            true,
            false,
        ),
        ("empty file name", "", "/project/src", true, false),
        (
            "identical paths",
            "/project/src",
            "/project/src",
            true,
            false,
        ),
        (
            "directory with trailing separator",
            "/project/src/file.ts",
            "/project/src/",
            true,
            true,
        ),
        (
            "unicode characters",
            "/project/测试/file.ts",
            "/project/测试",
            true,
            true,
        ),
        (
            "unicode case insensitive",
            "/project/测试/file.ts",
            "/PROJECT/测试",
            false,
            true,
        ),
    ];

    for &(name, file_name, directory_name, use_case_sensitive, expected) in tests {
        let result = starts_with_directory(file_name, directory_name, use_case_sensitive);
        assert_eq!(
            result, expected,
            "StartsWithDirectory({:?}, {:?}, {}) = {}, expected {} ({})",
            file_name, directory_name, use_case_sensitive, result, expected, name
        );
    }
}

#[test]
fn test_starts_with_directory_edge_cases() {
    let tests: &[(&str, &str, &str, bool, bool)] = &[
        (
            "file name shorter than directory",
            "/proj",
            "/project",
            true,
            false,
        ),
        (
            "file name starts with directory but no separator",
            "/projectsrc/file.ts",
            "/project",
            true,
            false,
        ),
        ("relative paths", "src/file.ts", "src", true, true),
        (
            "absolute vs relative",
            "/project/src/file.ts",
            "project/src",
            true,
            false,
        ),
    ];

    for &(name, file_name, directory_name, use_case_sensitive, expected) in tests {
        let result = starts_with_directory(file_name, directory_name, use_case_sensitive);
        assert_eq!(
            result, expected,
            "StartsWithDirectory({:?}, {:?}, {}) = {}, expected {} ({})",
            file_name, directory_name, use_case_sensitive, result, expected, name
        );
    }
}
