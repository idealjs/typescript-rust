//! Go string_completions.go 路径映射枚举的移植：
//! typesVersions 版本重定向、getCompletionsForPathMapping 与
//! getModulesForPathsPattern 的 `前缀*后缀` 模式枚举

use std::sync::Arc;

use tsox_compile::compiler::Program;
use tsox_core::tspath as tsp;
use tsox_tsoptions::packagejson;
use tsox_tsoptions::packagejson::JsonValueType;

use super::completions_path::{
    completion_file_name, get_fragment_directory, nearest_package_json_dir, read_package_json,
    string_extensions, ModuleCompletionSet,
};


use super::language_service::LanguageService;

/// Go getCompletionEntriesForDirectoryFragment 的版本重定向段：
/// 就近 package.json 的 typesVersions 命中时按映射产生补全（最长匹配覆盖）
pub(super) fn version_redirect(
    service: &LanguageService,
    program: &Arc<Program>,
    base_directory: &str,
    result: &mut ModuleCompletionSet,
) -> bool {
    let Some(package_dir) = nearest_package_json_dir(service, base_directory) else {
        return false;
    };
    let Some(fields) = read_package_json(service, &package_dir) else {
        return false;
    };
    let Some(paths) = version_paths(&fields) else {
        return false;
    };
    let prefix = tsp::ensure_trailing_directory_separator(&package_dir);
    let path_in_package = base_directory
        .strip_prefix(&prefix)
        .unwrap_or("")
        .trim_start_matches('/')
        .to_string();

    let prefix_len = |key: &str| key.find('*').unwrap_or(key.len());
    let mut matched_names: Vec<String> = Vec::new();
    let mut unmatched_names: Vec<String> = Vec::new();
    let mut matched_key: Option<String> = None;
    for (key, patterns) in &paths {
        let normalized_key = key.trim_start_matches("./").to_string();
        let pattern = tsox_core::core::core::try_parse_pattern(&normalized_key);
        if !pattern.is_valid() {
            continue;
        }
        let is_match = pattern.matches(&path_in_package);
        let is_longest =
            is_match && matched_key.as_ref().is_none_or(|m| prefix_len(&normalized_key) > prefix_len(m));
        if is_longest {
            matched_key = Some(normalized_key.clone());
            matched_names.clear();
        }
        if pattern.star_index == -1
            || matched_key.is_none()
            || prefix_len(&normalized_key) >= prefix_len(matched_key.as_deref().unwrap())
        {
            let names = completions_for_path_mapping(
                service,
                program,
                &normalized_key,
                patterns,
                &path_in_package,
                &package_dir,
            );
            if is_match {
                matched_names.extend(names);
            } else {
                unmatched_names.extend(names);
            }
        }
    }
    for name in unmatched_names.into_iter().chain(matched_names) {
        result.add(name);
    }
    matched_key.is_some()
}

/// Go getCompletionsForPathMapping：单个映射键的补全产出
pub(super) fn completions_for_path_mapping(
    service: &LanguageService,
    program: &Arc<Program>,
    path: &str,
    patterns: &[String],
    fragment: &str,
    package_directory: &str,
) -> Vec<String> {
    let parsed = tsox_core::core::core::try_parse_pattern(path);
    let fragment_directory = get_fragment_directory(fragment);
    let just_name = |name: &str| -> Option<String> {
        if !name.starts_with(fragment) {
            return None;
        }
        let mut name = tsp::remove_trailing_directory_separator(name);
        if !fragment_directory.is_empty() {
            name = name.trim_start_matches(&fragment_directory).to_string();
        }
        Some(name)
    };
    if parsed.star_index == -1 {
        return just_name(path).into_iter().collect();
    }
    let star = parsed.star_index as usize;
    let path_prefix = &path[..star];
    let path_suffix = &path[star + 1..];
    if !fragment.starts_with(path_prefix) {
        if !path_prefix.starts_with(fragment) {
            return Vec::new();
        }
        if path.ends_with("/*") {
            return just_name(path_prefix).into_iter().collect();
        }
        let remaining_prefix = &path_prefix[fragment_directory.len()..];
        let mut out = Vec::new();
        for pattern in patterns {
            for (name, is_dir) in
                modules_for_paths_pattern(service, program, "", pattern, package_directory)
            {
                out.push(if is_dir {
                    format!("{remaining_prefix}{name}")
                } else {
                    format!("{remaining_prefix}{name}{path_suffix}")
                });
            }
        }
        return out;
    }
    let remaining_fragment = &fragment[path_prefix.len()..];
    let remaining_directory_fragment = if !fragment_directory.starts_with(path_prefix) {
        &path_prefix[fragment_directory.len()..]
    } else {
        ""
    };
    let mut out = Vec::new();
    for pattern in patterns {
        for (name, is_dir) in modules_for_paths_pattern(
            service,
            program,
            remaining_fragment,
            pattern,
            package_directory,
        ) {
            out.push(if is_dir {
                format!("{remaining_directory_fragment}{name}")
            } else {
                format!("{remaining_directory_fragment}{name}{path_suffix}")
            });
        }
    }
    out
}

/// Go getModulesForPathsPattern：`前缀*后缀` 模式下的目录枚举
fn modules_for_paths_pattern(
    service: &LanguageService,
    program: &Arc<Program>,
    fragment: &str,
    pattern: &str,
    package_directory: &str,
) -> Vec<(String, bool)> {
    let parsed = tsox_core::core::core::try_parse_pattern(pattern);
    if parsed.star_index == -1 {
        return Vec::new();
    }
    let star = parsed.star_index as usize;
    let prefix = &pattern[..star];
    let suffix = pattern[star + 1..].to_string();

    let normalized_prefix = tsp::normalize_path(prefix);
    let (prefix_directory, prefix_base) = if tsp::has_trailing_directory_separator(prefix) {
        (normalized_prefix, String::new())
    } else {
        (
            tsp::get_directory_path(&normalized_prefix),
            tsp::get_base_file_name(&normalized_prefix),
        )
    };
    let fragment_directory = get_fragment_directory(fragment);
    let expanded = if fragment.contains('/') {
        format!("{prefix_base}{fragment_directory}")
    } else {
        prefix_base.clone()
    };
    let joined = tsp::combine_paths(&prefix_directory, &[&expanded]);
    let base_directory = tsp::normalize_path(&tsp::combine_paths(package_directory, &[&joined]));

    let includes = if suffix.is_empty() {
        vec!["./*".to_string()]
    } else {
        vec![format!("**/*{suffix}")]
    };
    let extensions = string_extensions(program);
    let matches = service.read_directory(&base_directory, &extensions, &includes);
    let complete_prefix = base_directory.trim_end_matches('/').to_string();
    let mut out: Vec<(String, bool)> = Vec::new();
    for m in &matches {
        let Some(trimmed) = m
            .strip_prefix(&format!("{complete_prefix}/"))
            .map(str::to_string)
        else {
            continue;
        };
        let mut trimmed = trimmed;
        if !suffix.is_empty() && trimmed.ends_with(&suffix) {
            trimmed = trimmed[..trimmed.len() - suffix.len()].to_string();
        }
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains('/') {
            let first = trimmed.split('/').next().unwrap_or_default();
            out.push((first.to_string(), true));
        } else {
            out.push((completion_file_name(&trimmed), false));
        }
    }
    if suffix.is_empty() && fragment.is_empty() {
        for dir in service.get_directories(&base_directory) {
            let name = tsp::get_base_file_name(&dir);
            if name != "node_modules" {
                out.push((name, true));
            }
        }
    }
    out
}


/// typesVersions 版本键匹配（Go semver.TryParseVersionRange 的常用子集）：
/// 取第一个匹配编译器版本的范围键下的路径映射
fn version_paths(fields: &packagejson::Fields) -> Option<Vec<(String, Vec<String>)>> {
    let tv = &fields.path_fields.types_versions;
    if !tv.is_present() || tv.value_type != JsonValueType::Object {
        return None;
    }
    for (key, value) in tv.as_object() {
        if !version_range_matches(key) {
            continue;
        }
        let mut paths = Vec::new();
        if value.value_type == JsonValueType::Object {
            for (pattern, targets) in value.as_object() {
                let list: Vec<String> = targets
                    .as_array()
                    .iter()
                    .filter(|t| t.value_type == JsonValueType::String)
                    .map(|t| t.as_string().to_string())
                    .collect();
                if !list.is_empty() {
                    paths.push((pattern.clone(), list));
                }
            }
        }
        return (!paths.is_empty()).then_some(paths);
    }
    None
}

fn version_range_matches(range: &str) -> bool {
    if range == "*" {
        return true;
    }
    for op in [">=", "<=", ">", "<", "="] {
        if let Some(rest) = range.strip_prefix(op) {
            return compare_version(rest.trim(), op);
        }
    }
    compare_version(range.trim(), "=")
}

fn compare_version(text: &str, op: &str) -> bool {
    let parse = |s: &str| -> (u64, u64, u64) {
        let mut it = s.split('.');
        (
            it.next().and_then(|p| p.parse().ok()).unwrap_or(0),
            it.next().and_then(|p| p.parse().ok()).unwrap_or(0),
            it.next()
                .and_then(|p| p.split('-').next().and_then(|x| x.parse().ok()))
                .unwrap_or(0),
        )
    };
    let v = parse(text);
    let ts = (7u64, 1u64, 0u64);
    match op {
        ">=" => v <= ts,
        "<=" => v >= ts,
        ">" => v < ts,
        "<" => v > ts,
        _ => v == ts,
    }
}
