#![allow(dead_code)]

use crate::ast::{Node, Symbol};
use crate::core::compiler_options::{CompilerOptions, ResolutionMode};
use crate::tspath::{self, ComparePathsOptions};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePath {
    pub file_name: String,
    pub is_in_node_modules: bool,
    pub is_redirect: bool,
}

pub fn contains_node_modules(s: &str) -> bool {
    s.contains("/node_modules/")
}

pub fn contains_ignored_path(s: &str) -> bool {
    tspath::contains_ignored_path(s)
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchingMode {
    Exact,
    Directory,
    Pattern,
}

pub trait ModuleSpecifierGenerationHost {
    fn get_current_directory(&self) -> String;
    fn use_case_sensitive_file_names(&self) -> bool;
    fn common_source_directory(&self) -> String;
    fn file_exists(&self, path: &str) -> bool;
}

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

pub type ImportModuleSpecifierPreference = String;

pub const IMPORT_MODULE_SPECIFIER_PREFERENCE_SHORTEST: &str = "shortest";
pub const IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE: &str = "project-relative";
pub const IMPORT_MODULE_SPECIFIER_PREFERENCE_RELATIVE: &str = "relative";
pub const IMPORT_MODULE_SPECIFIER_PREFERENCE_NON_RELATIVE: &str = "non-relative";

pub type ImportModuleSpecifierEndingPreference = String;

pub const IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_AUTO: &str = "auto";
pub const IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_MINIMAL: &str = "minimal";
pub const IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_INDEX: &str = "index";
pub const IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_JS: &str = "js";

#[derive(Debug, Clone, Default)]
pub struct UserPreferences {
    pub import_module_specifier_preference: ImportModuleSpecifierPreference,
    pub import_module_specifier_ending: ImportModuleSpecifierEndingPreference,
    pub auto_import_specifier_exclude_regexes: Vec<String>,
}

#[allow(unused_variables)]
pub fn is_excluded_by_regex(module_specifier: &str, exclude_regexes: &[String]) -> bool {

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ResultKind {
    #[default]
    None = 0,
    NodeModules = 1,
    Paths = 2,
    Redirect = 3,
    Relative = 4,
    Ambient = 5,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModuleSpecifierOptions {
    pub override_import_mode: ResolutionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum RelativePreferenceKind {
    Relative = 0,
    NonRelative = 1,
    #[default]
    Shortest = 2,
    ExternalNonRelative = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ModuleSpecifierEnding {
    #[default]
    Minimal = 0,
    Index = 1,
    JsExtension = 2,
    TsExtension = 3,
}

pub struct ModuleSpecifierPreferences {
    pub relative_preference: RelativePreferenceKind,
    pub exclude_regexes: Vec<String>,
}

pub trait SourceFileForSpecifierGeneration {
    fn path(&self) -> &str;
    fn file_name(&self) -> &str;
    fn is_js(&self) -> bool;
}

pub trait CheckerShape {
    fn get_symbol_at_location(&self, node: &Arc<Node>) -> Option<Arc<Symbol>>;
    fn get_aliased_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>>;
}

#[derive(Debug, Clone, Default)]
pub struct Info {
    pub importing_source_file_file_name: String,
    pub importing_source_file_directory: String,
    pub importing_source_file_is_in_node_modules: bool,
    pub common_source_directory: String,
    pub use_case_sensitive_file_names: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NodeModulePathParts {
    pub top_level_node_modules_index: usize,
    pub top_level_package_name_index: usize,
    pub package_root_index: usize,
    pub file_name_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegexPatternCacheKey {
    pub pattern: String,
    pub case_insensitive: bool,
}

#[derive(Debug, Clone)]
pub struct SpecPair {
    pub ending: ModuleSpecifierEnding,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct PkgJsonDirAttemptResult {
    pub pkg_json_directory: String,
    pub directory_exists: bool,
    pub package_name: String,
    pub version: String,
    pub root_dir_attempt_failed: bool,
}

pub fn path_is_bare_specifier(path: &str) -> bool {
    !tspath::path_is_absolute(path) && !tspath::path_is_relative(path)
}

pub fn ensure_path_is_non_module_name(path: &str) -> String {
    if path_is_bare_specifier(path) {
        format!("./{path}")
    } else {
        path.to_string()
    }
}

pub fn get_js_extension_for_declaration_file_extension(ext: &str) -> String {
    match ext {
        tspath::EXTENSION_DTS => tspath::EXTENSION_JS.to_string(),
        tspath::EXTENSION_DMTS => tspath::EXTENSION_MJS.to_string(),
        tspath::EXTENSION_DCTS => tspath::EXTENSION_CJS.to_string(),
        _ => {

            let start = ".d".len();
            let end = ext.len().saturating_sub(tspath::EXTENSION_TS.len());
            if start <= end {
                ext[start..end].to_string()
            } else {
                ext.to_string()
            }
        }
    }
}

pub fn extension_from_path(path: &str) -> String {
    let ext = tspath::try_get_extension_from_path(path);
    if ext.is_empty() {
        panic!("File {path} has unknown extension.");
    }
    ext.to_string()
}

pub fn is_path_relative_to_parent(path: &str) -> bool {
    path.starts_with("..")
}

pub fn get_relative_path_if_in_same_volume(
    path: &str,
    directory_path: &str,
    use_case_sensitive_file_names: bool,
) -> String {
    let relative_path = tspath::get_relative_path_to_directory_or_url(
        directory_path,
        path,
        false,
        &ComparePathsOptions {
            use_case_sensitive_file_names,
            current_directory: directory_path.to_string(),
        },
    );
    if tspath::is_rooted_disk_path(&relative_path) {
        return String::new();
    }
    relative_path
}

pub fn get_paths_relative_to_root_dirs(
    path: &str,
    root_dirs: &[String],
    use_case_sensitive_file_names: bool,
) -> Vec<String> {
    let mut results = Vec::new();
    for root_dir in root_dirs {
        let relative_path =
            get_relative_path_if_in_same_volume(path, root_dir, use_case_sensitive_file_names);
        if !is_path_relative_to_parent(&relative_path) {
            results.push(relative_path);
        }
    }
    results
}

pub fn package_json_paths_are_equal(a: &str, b: &str, options: ComparePathsOptions) -> bool {
    if a == b {
        return true;
    }
    if a.is_empty() || b.is_empty() {
        return false;
    }

    if options.use_case_sensitive_file_names {
        a == b
    } else {
        a.eq_ignore_ascii_case(b)
    }
}

pub fn prefers_ts_extension(allowed_endings: &[ModuleSpecifierEnding]) -> bool {
    let js_priority = allowed_endings
        .iter()
        .position(|e| *e == ModuleSpecifierEnding::JsExtension);
    let ts_priority = allowed_endings
        .iter()
        .position(|e| *e == ModuleSpecifierEnding::TsExtension);
    if let Some(ts) = ts_priority {
        return js_priority.map_or(true, |js| ts < js);
    }
    false
}

pub fn replace_first_star(s: &str, replacement: &str) -> String {
    s.replacen('*', replacement, 1)
}

pub fn all_keys_start_with_dot<'a, I>(keys: I) -> bool
where
    I: IntoIterator<Item = &'a str>,
{
    keys.into_iter().all(|k| k.starts_with('.'))
}

pub fn get_node_module_path_parts(full_path: &str) -> Option<NodeModulePathParts> {

    let mut top_level_node_modules_index = 0usize;
    let mut top_level_package_name_index = 0usize;
    let mut package_root_index = 0usize;
    let file_name_index;

    let bytes = fullPathBytes(full_path);
    let mut part_start;
    let mut part_end = 0usize;

    let mut state: u8 = 0;

    loop {
        part_start = part_end;
        match index_after_bytes(&bytes, b'/', part_start + 1) {
            Some(idx) => part_end = idx,
            None => break,
        }
        let segment = &full_path[part_start..part_end];
        match state {
            0 => {
                if segment.starts_with("/node_modules/") {
                    top_level_node_modules_index = part_start;
                    top_level_package_name_index = part_end;
                    state = 1;
                }
            }
            1 | 2 => {
                if state == 1 {
                    let inner = if part_start + 1 < part_end {
                        &full_path[part_start + 1..part_start + 2]
                    } else {
                        ""
                    };
                    if inner == "@" {
                        state = 2;
                        continue;
                    }
                }
                package_root_index = part_end;
                state = 3;
            }
            _ => {
                if segment.starts_with("/node_modules/") {
                    state = 1;
                } else {
                    state = 3;
                }
            }
        }
    }

    file_name_index = part_start;

    if state > 1 {
        return Some(NodeModulePathParts {
            top_level_node_modules_index,
            top_level_package_name_index,
            package_root_index,
            file_name_index,
        });
    }
    None
}

pub fn get_package_name_from_directory(file_or_directory_path: &str) -> String {
    let idx = match file_or_directory_path.rfind("/node_modules/") {
        Some(i) => i,
        None => return String::new(),
    };
    let basename = &file_or_directory_path[idx + "/node_modules/".len()..];
    if basename.is_empty() || basename.as_bytes()[0] == b'.' {
        return String::new();
    }
    let next_slash = match basename.find('/') {
        Some(i) => i,
        None => return basename.to_string(),
    };
    if !basename.starts_with('@') || next_slash == basename.len() - 1 {
        return basename[..next_slash].to_string();
    }
    let second_slash = match basename[next_slash + 1..].find('/') {
        Some(i) => next_slash + 1 + i,
        None => return basename.to_string(),
    };
    basename[..second_slash].to_string()
}

pub fn compare_paths_by_redirect(
    a: &ModulePath,
    b: &ModulePath,
    use_case_sensitive_file_names: bool,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match b.is_redirect.cmp(&a.is_redirect) {
        Ordering::Equal => {}
        ord => return ord,
    }

    let a_seps = a.file_name.matches('/').count();
    let b_seps = b.file_name.matches('/').count();
    match a_seps.cmp(&b_seps) {
        Ordering::Equal => {}
        ord => return ord,
    }

    if use_case_sensitive_file_names {
        a.file_name.cmp(&b.file_name)
    } else {
        a.file_name
            .to_ascii_lowercase()
            .cmp(&b.file_name.to_ascii_lowercase())
    }
}

pub fn should_allow_importing_ts_extension(
    compiler_options: &CompilerOptions,
    from_file_name: &str,
) -> bool {
    compiler_options.get_allow_importing_ts_extensions()
        || (!from_file_name.is_empty() && tspath::is_declaration_file_name(from_file_name))
}

pub fn get_allowed_endings_in_preferred_order(
    _prefs: &UserPreferences,
    _host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _old_import_specifier: &str,
    _syntax_implied_node_format: ResolutionMode,
) -> Vec<ModuleSpecifierEnding> {

    vec![ModuleSpecifierEnding::Minimal]
}

pub fn get_module_specifier_preferences(
    prefs: &UserPreferences,
    _host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    old_import_specifier: &str,
) -> ModuleSpecifierPreferences {
    let excludes = prefs.auto_import_specifier_exclude_regexes.clone();
    let relative_preference;
    if !old_import_specifier.is_empty() {
        if tspath::is_external_module_name_relative(old_import_specifier) {
            relative_preference = RelativePreferenceKind::Relative;
        } else {
            relative_preference = RelativePreferenceKind::NonRelative;
        }
    } else {
        relative_preference = match prefs.import_module_specifier_preference.as_str() {
            IMPORT_MODULE_SPECIFIER_PREFERENCE_RELATIVE => RelativePreferenceKind::Relative,
            IMPORT_MODULE_SPECIFIER_PREFERENCE_NON_RELATIVE => RelativePreferenceKind::NonRelative,
            IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE => {
                RelativePreferenceKind::ExternalNonRelative
            }

            _ => RelativePreferenceKind::Shortest,
        };
    }
    ModuleSpecifierPreferences {
        exclude_regexes: excludes,
        relative_preference,
    }
}

pub fn get_module_specifiers(
    _module_symbol: &Arc<Symbol>,
    _checker: &dyn CheckerShape,
    _compiler_options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _host: &dyn ModuleSpecifierGenerationHost,
    _user_preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
    _for_auto_imports: bool,
) -> Vec<String> {

    Vec::new()
}

pub fn get_module_specifiers_with_info(
    _module_symbol: &Arc<Symbol>,
    _checker: &dyn CheckerShape,
    _compiler_options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _host: &dyn ModuleSpecifierGenerationHost,
    _user_preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
    _for_auto_imports: bool,
) -> (Vec<String>, ResultKind) {

    (Vec::new(), ResultKind::None)
}

pub fn get_module_specifiers_for_file_with_info(
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _module_file_name: &str,
    _compiler_options: &CompilerOptions,
    _host: &dyn ModuleSpecifierGenerationHost,
    _user_preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
    _for_auto_imports: bool,
) -> (Vec<String>, ResultKind) {

    (Vec::new(), ResultKind::None)
}

pub fn get_module_specifier(
    _from_file_name: &str,
    _to_file_name: &str,
    _host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
    _preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
) -> Option<String> {

    None
}

pub fn update_module_specifier(
    _from_file_name: &str,
    _to_file_name: &str,
    _host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
    _preferences: &UserPreferences,
    _old_import_specifier: &str,
    _options: &ModuleSpecifierOptions,
) -> Option<String> {

    None
}

pub fn get_node_modules_package_name(
    _compiler_options: &CompilerOptions,
    _importing_source_file_file_name: &str,
    _node_modules_file_name: &str,
    _host: &dyn ModuleSpecifierGenerationHost,
    _preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
) -> String {

    String::new()
}

pub fn process_entrypoint_ending(
    _entrypoint_module_specifier: &str,
    _entrypoint_is_fixed: bool,
    _prefs: &UserPreferences,
    _host: &dyn ModuleSpecifierGenerationHost,
    _options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _allowed_endings: &[ModuleSpecifierEnding],
) -> String {

    String::new()
}

pub fn get_js_extension_for_file(file_name: &str, _options: &CompilerOptions) -> String {

    extension_from_path(file_name)
}

#[allow(non_snake_case)]
fn fullPathBytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

fn index_after_bytes(bytes: &[u8], b: u8, start: usize) -> Option<usize> {
    if start > bytes.len() {
        return None;
    }
    bytes[start..]
        .iter()
        .position(|&c| c == b)
        .map(|i| i + start)
}

#[cfg(test)]
mod tests {
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
}
