use super::*;

struct MockModuleSpecifierGenerationHost {
    current_dir: String,
    use_case_sensitive_file_names: bool,
}

impl ModuleSpecifierGenerationHost for MockModuleSpecifierGenerationHost {
    fn get_current_directory(&self) -> String {
        self.current_dir.clone()
    }
    fn use_case_sensitive_file_names(&self) -> bool {
        self.use_case_sensitive_file_names
    }
    fn common_source_directory(&self) -> String {
        self.current_dir.clone()
    }
    fn file_exists(&self, _path: &str) -> bool {
        true
    }
}

#[test]

fn test_get_each_file_name_of_module() {
    struct TestCase {
        name: &'static str,
        importing_file: &'static str,
        imported_file: &'static str,
        prefer_symlinks: bool,
        expected_count: usize,
        expected_paths: Option<Vec<&'static str>>,
    }

    let tests = [
        TestCase {
            name: "basic file path",
            importing_file: "/project/src/main.ts",
            imported_file: "/project/lib/utils.ts",
            prefer_symlinks: false,
            expected_count: 1,
            expected_paths: Some(vec!["/project/lib/utils.ts"]),
        },
        TestCase {
            name: "symlink preference false",
            importing_file: "/project/src/main.ts",
            imported_file: "/project/lib/utils.ts",
            prefer_symlinks: false,
            expected_count: 1,
            expected_paths: None,
        },
        TestCase {
            name: "symlink preference true",
            importing_file: "/project/src/main.ts",
            imported_file: "/project/lib/utils.ts",
            prefer_symlinks: true,
            expected_count: 1,
            expected_paths: None,
        },
        TestCase {
            name: "ignored path with no alternatives",
            importing_file: "/project/src/main.ts",
            imported_file: "/project/node_modules/.pnpm/file.ts",
            prefer_symlinks: false,
            expected_count: 1,
            expected_paths: None,
        },
    ];

    for tt in &tests {
        let host = MockModuleSpecifierGenerationHost {
            current_dir: "/project".to_string(),
            use_case_sensitive_file_names: true,
        };

        let result = get_each_file_name_of_module(
            tt.importing_file,
            tt.imported_file,
            &host,
            tt.prefer_symlinks,
        );

        assert_eq!(
            result.len(),
            tt.expected_count,
            "{}: Expected {} paths, got {}",
            tt.name,
            tt.expected_count,
            result.len()
        );

        if let Some(ref expected_paths) = tt.expected_paths {
            for (i, expected_path) in expected_paths.iter().enumerate() {
                if i >= result.len() {
                    panic!(
                        "{}: Expected path {i}: {expected_path}, but result has only {} paths",
                        tt.name,
                        result.len()
                    );
                }
                assert_eq!(
                    result[i].file_name, *expected_path,
                    "{}: Expected path {i} to be {expected_path}, got {}",
                    tt.name, result[i].file_name
                );
            }
        }

        for (i, path) in result.iter().enumerate() {
            assert!(!path.file_name.is_empty(), "{i}: Path has empty FileName");
        }
    }
}

#[test]

fn test_get_each_file_name_of_module_with_symlinks() {
    let host = MockModuleSpecifierGenerationHost {
        current_dir: "/project".to_string(),
        use_case_sensitive_file_names: true,
    };

    let result =
        get_each_file_name_of_module("/project/src/main.ts", "/real/path/file.ts", &host, true);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].file_name, "/real/path/file.ts");
    assert!(!result[0].is_in_node_modules);
    assert!(!result[0].is_redirect);
}

#[test]

fn test_contains_node_modules() {
    let cases: &[(&str, &str, bool)] = &[
        (
            "contains node_modules",
            "/project/node_modules/lodash/index.js",
            true,
        ),
        (
            "does not contain node_modules",
            "/project/src/utils.ts",
            false,
        ),
        (
            "node_modules in middle",
            "/project/packages/node_modules/pkg/file.js",
            true,
        ),
        ("empty path", "", false),
    ];

    for (name, path, expected) in cases {
        let result = contains_node_modules(path);
        assert_eq!(
            result, *expected,
            "{name}: contains_node_modules({path:?}) = {result}, expected {expected}"
        );
    }
}

#[test]

fn test_contains_ignored_path() {
    let cases: &[(&str, &str, bool)] = &[
        ("ignored path", "/project/node_modules/.pnpm/file.ts", true),
        ("not ignored path", "/project/src/file.ts", false),
    ];

    for (name, path, expected) in cases {
        let result = contains_ignored_path(path);
        assert_eq!(
            result, *expected,
            "{name}: contains_ignored_path({path:?}) = {result}, expected {expected}"
        );
    }
}

#[test]

fn test_try_get_real_file_name_for_non_js_declaration_file_name() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "json declaration file",
            "/project/foo.d.json.ts",
            "/project/foo.json",
        ),
        (
            "multi-dot source extension declaration file",
            "/project/foo.module.d.css.ts",
            "/project/foo.module.css",
        ),
        ("plain dts file ignored", "/project/foo.d.ts", ""),
    ];

    for (name, file_name, expected) in cases {
        let got = try_get_real_file_name_for_non_js_declaration_file_name(file_name);
        assert_eq!(
            got, *expected,
            "{name}: try_get_real_file_name_for_non_js_declaration_file_name({file_name:?}) = {got:?}, expected {expected:?}"
        );
    }
}

#[test]

fn test_try_get_module_name_from_exports_or_imports() {
    let modes = [
        MatchingMode::Exact,
        MatchingMode::Directory,
        MatchingMode::Pattern,
    ];
    assert_eq!(modes.len(), 3);
    assert_ne!(MatchingMode::Exact, MatchingMode::Directory);

    assert_eq!(
        try_get_real_file_name_for_non_js_declaration_file_name("/pkg/foo.d.json.ts"),
        "/pkg/foo.json"
    );
    assert_eq!(
        try_get_real_file_name_for_non_js_declaration_file_name("/pkg/foo.d.ts"),
        ""
    );
}
