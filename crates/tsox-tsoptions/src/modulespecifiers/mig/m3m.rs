use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

use tsox_core::core::mig::m3j_2::compare_booleans;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath;
use tsox_core::tspath::mig::m3i as tspath_mig;
use tsox_core::tspath::ComparePathsOptions;

use crate::modulespecifiers::types::ModulePath;
use crate::modulespecifiers::types::ModuleSpecifierEnding;
use crate::modulespecifiers::types::ModuleSpecifierGenerationHost;
use crate::modulespecifiers::types::SourceFileForSpecifierGeneration;
use crate::modulespecifiers::types::UserPreferences;
use crate::modulespecifiers::get_allowed_endings_in_preferred_order;
use crate::modulespecifiers::types::ModuleSpecifierOptions;
use tsox_frontend::ast::SourceFile;

#[derive(Hash, PartialEq, Eq, Clone)]
struct RegexPatternCacheKey {
    pattern: String,
    case_insensitive: bool,
}

fn regex_pattern_cache() -> &'static Mutex<HashMap<RegexPatternCacheKey, Option<regex::Regex>>> {
    static CACHE: OnceLock<Mutex<HashMap<RegexPatternCacheKey, Option<regex::Regex>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn compare_paths_by_redirect(
    a: &ModulePath,
    b: &ModulePath,
    use_case_sensitive_file_names: bool,
) -> i32 {
    let c = compare_booleans(b.is_redirect, a.is_redirect);
    if c != 0 {
        return c;
    }
    let c = tspath_mig::compare_number_of_directory_separators(&a.file_name, &b.file_name);
    if c != 0 {
        return c;
    }
    tspath_mig::compare_paths(
        &a.file_name,
        &b.file_name,
        &ComparePathsOptions {
            use_case_sensitive_file_names,
            ..Default::default()
        },
    )
}

pub fn path_is_bare_specifier(path: &str) -> bool {
    !tspath::path_is_absolute(path) && !tspath::path_is_relative(path)
}

pub fn is_excluded_by_regex(module_specifier: &str, excludes: &[String]) -> bool {
    for pattern in excludes {
        let re = match string_to_regex(pattern) {
            Some(re) => re,
            None => continue,
        };
        if re.is_match(module_specifier) {
            return true;
        }
    }
    false
}

pub fn string_to_regex(pattern: &str) -> Option<regex::Regex> {
    let mut case_insensitive = false;
    let mut pattern = pattern.to_string();

    if pattern.len() > 2 && pattern.starts_with('/') {
        let last_slash = pattern.rfind('/');
        if let Some(last_slash) = last_slash {
            if last_slash > 0 {
                let bytes = pattern.as_bytes();
                let mut has_unescaped_middle_slash = false;
                for i in 1..last_slash {
                    if bytes[i] == b'/' && (i == 0 || bytes[i - 1] != b'\\') {
                        has_unescaped_middle_slash = true;
                        break;
                    }
                }
                if !has_unescaped_middle_slash {
                    let flags = pattern[last_slash + 1..].to_string();
                    pattern = pattern[1..last_slash].to_string();
                    for flag in flags.chars() {
                        if flag == 'i' {
                            case_insensitive = true;
                        }
                    }
                }
            }
        }
    }
    let key = RegexPatternCacheKey {
        pattern: pattern.clone(),
        case_insensitive,
    };

    let mut cache = regex_pattern_cache().lock().unwrap();
    if let Some(re) = cache.get(&key) {
        return re.clone();
    }

    if cache.len() > 1000 {
        cache.clear();
    }

    let compile_pattern = if case_insensitive {
        format!("(?i:{})", pattern)
    } else {
        pattern.clone()
    };

    let compiled = regex::Regex::new(&compile_pattern).ok();
    cache.insert(key, compiled.clone());
    compiled
}

pub fn ensure_path_is_non_module_name(path: &str) -> String {
    if path_is_bare_specifier(path) {
        return format!("./{}", path);
    }
    path.to_string()
}

pub fn get_js_extension_for_declaration_file_extension(ext: &str) -> String {
    match ext {
        tspath::EXTENSION_DTS => tspath::EXTENSION_JS.to_string(),
        tspath::EXTENSION_DMTS => tspath::EXTENSION_MJS.to_string(),
        tspath::EXTENSION_DCTS => tspath::EXTENSION_CJS.to_string(),
        _ => ext[".d".len()..ext.len() - tspath::EXTENSION_TS.len()].to_string(),
    }
}

pub fn try_get_real_file_name_for_non_js_declaration_file_name(file_name: &str) -> String {
    let base_name = tspath::get_base_file_name(file_name);
    if !file_name.ends_with(tspath::EXTENSION_TS)
        || !base_name.contains(".d.")
        || base_name.ends_with(tspath::EXTENSION_DTS)
    {
        return String::new();
    }
    let no_extension = tspath::remove_extension(file_name, tspath::EXTENSION_TS);
    let last_dot_index = no_extension.rfind('.').unwrap_or(0);
    let ext = &no_extension[last_dot_index..];
    let before = no_extension.split(".d.").next().unwrap_or("");
    format!("{}{}", before, ext)
}

pub fn get_js_extension_for_file(file_name: &str, options: &CompilerOptions) -> String {
    let result = crate::module::mig::m3i::try_get_js_extension_for_file(file_name, options);
    if result.is_empty() {
        panic!(
            "Extension {} is unsupported:: FileName:: {}",
            extension_from_path(file_name),
            file_name
        );
    }
    result.to_string()
}

pub fn extension_from_path(path: &str) -> &str {
    let ext = tspath::try_get_extension_from_path(path);
    if ext.is_empty() {
        panic!("File {} has unknown extension.", path);
    }
    ext
}

pub fn try_get_any_file_from_path(host: &dyn ModuleSpecifierGenerationHost, path: &str) -> bool {
    let ext_groups = crate::mig::m5i_3::get_supported_extensions(
        &CompilerOptions {
            allow_js: Tristate::True,
            ..Default::default()
        },
        &[".node", ".json"],
    );
    for exts in &ext_groups {
        for e in exts {
            let full_path = format!("{}{}", path, e);
            if host.file_exists(&tspath::get_normalized_absolute_path(
                &full_path,
                &host.get_current_directory(),
            )) {
                return true;
            }
        }
    }
    false
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
            ..Default::default()
        },
    );
    if tspath::is_rooted_disk_path(&relative_path) {
        return String::new();
    }
    relative_path
}

pub fn package_json_paths_are_equal(a: &str, b: &str, options: ComparePathsOptions) -> bool {
    if a == b {
        return true;
    }
    if a.is_empty() || b.is_empty() {
        return false;
    }
    tspath_mig::compare_paths(a, b, &options) == 0
}

pub fn prefers_ts_extension(allowed_endings: &[ModuleSpecifierEnding]) -> bool {
    let js_priority = allowed_endings
        .iter()
        .position(|e| *e == ModuleSpecifierEnding::JsExtension);
    let ts_priority = allowed_endings
        .iter()
        .position(|e| *e == ModuleSpecifierEnding::TsExtension);
    if let Some(ts_priority) = ts_priority {
        return ts_priority < js_priority.unwrap_or(usize::MAX);
    }
    false
}

pub fn replace_first_star(s: &str, replacement: &str) -> String {
    match s.find('*') {
        Some(idx) => format!("{}{}{}", &s[..idx], replacement, &s[idx + 1..]),
        None => s.to_string(),
    }
}
