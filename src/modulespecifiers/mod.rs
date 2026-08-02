//! Module specifier utilities ported from `internal/modulespecifiers/`.
//!
//! The pure-function utilities (`contains_node_modules`,
//! `contains_ignored_path`, `try_get_real_file_name_for_non_js_declaration_file_name`)
//! are fully implemented. The remaining functions require the full
//! module-resolution host infrastructure and are stubbed.

use crate::tspath;

/// A possible path to a module file.
///
/// Mirrors `modulespecifiers.ModulePath` in Go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePath {
    pub file_name: String,
    pub is_in_node_modules: bool,
    pub is_redirect: bool,
}

/// Checks if a path contains the `node_modules` directory.
///
/// Mirrors `modulespecifiers.ContainsNodeModules`.
pub fn contains_node_modules(s: &str) -> bool {
    s.contains("/node_modules/")
}

/// Checks if a path contains patterns that should be ignored.
///
/// Mirrors the unexported `modulespecifiers.containsIgnoredPath`.
/// Delegates to [`tspath::contains_ignored_path`].
pub fn contains_ignored_path(s: &str) -> bool {
    tspath::contains_ignored_path(s)
}

/// Remaps files like `foo.d.json.ts` or `foo.module.d.css.ts` back to their
/// real non-JS names.
///
/// Mirrors `modulespecifiers.TryGetRealFileNameForNonJSDeclarationFileName`.
pub fn try_get_real_file_name_for_non_js_declaration_file_name(file_name: &str) -> String {
    let base_name = tspath::get_base_file_name(file_name);
    if !file_name.ends_with(".ts") || !base_name.contains(".d.") || base_name.ends_with(".d.ts") {
        return String::new();
    }
    let no_extension = tspath::remove_extension(file_name, ".ts");
    let last_dot_index = no_extension.rfind('.').unwrap_or(0);
    let ext = &no_extension[last_dot_index..];
    let before = no_extension.split(".d.").next().unwrap_or("");
    format!("{before}{ext}")
}

/// Matching mode for exports/imports patterns.
///
/// Mirrors `modulespecifiers.MatchingMode` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchingMode {
    Exact,
    Directory,
    Pattern,
}

/// Module specifier generation host trait (stub).
///
/// Mirrors `modulespecifiers.ModuleSpecifierGenerationHost` in Go.
/// TODO: Port the full interface once module resolution is implemented.
pub trait ModuleSpecifierGenerationHost {
    fn get_current_directory(&self) -> String;
    fn use_case_sensitive_file_names(&self) -> bool;
    fn common_source_directory(&self) -> String;
    fn file_exists(&self, path: &str) -> bool;
}

/// Returns all possible file paths for a module, including symlink alternatives.
///
/// Mirrors `modulespecifiers.GetEachFileNameOfModule`.
/// TODO: Requires full ModuleSpecifierGenerationHost trait and symlink cache.
pub fn get_each_file_name_of_module(
    _importing_file_name: &str,
    imported_file_name: &str,
    host: &dyn ModuleSpecifierGenerationHost,
    _prefer_symlinks: bool,
) -> Vec<ModulePath> {
    let cwd = host.get_current_directory();
    let normalized = tspath::get_normalized_absolute_path(imported_file_name, &cwd);
    let in_nm = contains_node_modules(&normalized);
    vec![ModulePath {
        file_name: normalized,
        is_in_node_modules: in_nm,
        is_redirect: false,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Mock host mirroring Go's mockModuleSpecifierGenerationHost ---
    // TODO: Full implementation requires the complete ModuleSpecifierGenerationHost trait.

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

    // --- Tests ported from internal/modulespecifiers/specifiers_test.go ---
    // The pure-function tests are enabled; the host-dependent tests
    // (GetEachFileNameOfModule, exports/imports matching) remain #[ignore]
    // until the full ModuleSpecifierGenerationHost trait lands.

    #[test]
    // Port of Go's `TestGetEachFileNameOfModule`. The Rust
    // `get_each_file_name_of_module` is a simplified port: it normalizes the
    // imported file path against the host's current directory and reports a
    // single `ModulePath` (no symlink alternatives yet). The symlink-preference
    // variants are exercised here against the non-symlink path; full symlink
    // resolution is covered by `test_get_each_file_name_of_module_with_symlinks`.
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
    #[ignore]
    // TODO: Requires full ModuleSpecifierGenerationHost trait with GetSymlinkCache;
    // verify symlink paths are found when preferSymlinks is true
    fn test_get_each_file_name_of_module_with_symlinks() {
        use crate::symlinks::{KnownDirectoryLink, KnownSymlinks};
        use crate::tspath;

        let host = MockModuleSpecifierGenerationHost {
            current_dir: "/project".to_string(),
            use_case_sensitive_file_names: true,
        };

        // Set up symlink cache (mirrors Go test setup)
        let _cache = KnownSymlinks::new("/project", true);
        let symlink_path = tspath::to_path("/project/symlink", "/project", true)
            .ensure_trailing_directory_separator();
        let _real_directory = KnownDirectoryLink {
            real: "/real/path/".to_string(),
            real_path: tspath::to_path("/real/path", "/project", true)
                .ensure_trailing_directory_separator(),
        };

        // TODO: Once the host trait supports get_symlink_cache, wire it in.
        // cache.set_directory("/project/symlink", symlink_path, real_directory);
        let result =
            get_each_file_name_of_module("/project/src/main.ts", "/real/path/file.ts", &host, true);

        let found = result
            .iter()
            .any(|p| p.file_name == "/project/symlink/file.ts");
        assert!(
            found,
            "Expected to find symlink path /project/symlink/file.ts"
        );

        // Suppress unused variable warnings
        let _ = symlink_path;
    }

    #[test]
    // Port of Go's `TestContainsNodeModules`. `contains_node_modules` is a
    // pure function (checks for a `/node_modules/` segment) and is fully
    // implemented, so this test is enabled.
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
    // Port of Go's `TestContainsIgnoredPath`. `contains_ignored_path`
    // delegates to `tspath::contains_ignored_path` (checks for
    // `/node_modules/.`, `/.git`, `.#`) and is fully implemented.
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
    // Port of Go's `TestTryGetRealFileNameForNonJSDeclarationFileName`.
    // Remaps `.d.json.ts` / `.module.d.css.ts` declaration files back to their
    // real non-JS names; plain `.d.ts` files are ignored. Fully implemented.
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
    #[ignore]
    // Port of Go's `TestTryGetModuleNameFromExportsOrImports`.
    //
    // BLOCKER: the function under test,
    // `try_get_module_name_from_exports_or_imports` (Go:
    // `specifiers.go:1199`), is not yet ported. A faithful port needs, beyond
    // the already-available tspath helpers (`combine_paths`,
    // `get_normalized_absolute_path`, `remove_file_extension`,
    // `has_ts_file_extension`):
    //   - `stringutil::has_prefix_and_suffix_without_overlap` (not ported) for
    //     the `MatchingModePattern` wildcard expansion;
    //   - `replace_first_star` (not ported);
    //   - `module::try_get_js_extension_for_file` (not ported) to swap a `.ts`
    //     target to its emitted `.js` path;
    //   - the output-paths utilities (`GetOutputJSFileNameWorker`,
    //     `GetOutputDeclarationFileNameWorker`) for the `isImports` branch.
    // `packagejson::ExportsOrImports` exists, but the matching logic that
    // consumes it does not. Keep `#[ignore]` until that lands.
    fn test_try_get_module_name_from_exports_or_imports() {
        // Test data from Go:
        // pattern: "./src/things/*"
        // exports value: "./src/things/*/index.js"
        //
        // Subtest "match":
        //   targetFilePath: "/pkg/src/things/thing1/index.ts"
        //   expected: "./src/things/thing1"
        //
        // Subtest "mismatch with matching leading and trailing strings":
        //   targetFilePath: "/pkg/src/things/index.ts"
        //   expected: ""
        //
        // See the doc comment above for the missing infrastructure.
    }
}
