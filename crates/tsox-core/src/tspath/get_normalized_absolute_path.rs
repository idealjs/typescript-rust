#![allow(unused_imports)]

use super::*;

pub fn get_normalized_absolute_path(path: &str, current_directory: &str) -> String {
    let combined = if path_is_absolute(path) {
        normalize_slashes(path)
    } else if current_directory.is_empty() {
        normalize_slashes(path)
    } else {
        combine_paths(current_directory, &[path])
    };
    let root_length = get_root_length(&combined);
    let normalized = normalize_path(&combined);
    let length = normalized.len();
    if length > root_length {
        remove_trailing_directory_separator(&normalized)
    } else if length == root_length && root_length != 0 {
        ensure_trailing_directory_separator(&normalized)
    } else {
        normalized
    }
}

pub fn normalize_path(path: &str) -> String {
    let path = normalize_slashes(path);

    let simplified = path.replace("/./", "/");
    let simplified = simplified
        .strip_prefix("./")
        .unwrap_or(&simplified)
        .to_string();
    if !has_relative_path_segment(&simplified) {
        return simplified;
    }

    let components = get_normalized_path_components_from_combined(&simplified);
    let result = get_path_from_path_components(&components);
    if !result.is_empty() && has_trailing_directory_separator(&path) {
        ensure_trailing_directory_separator(&result)
    } else {
        result
    }
}

pub(crate) fn get_normalized_path_components_from_combined(path: &str) -> Vec<String> {
    let root_length = get_root_length(path);
    let mut components = vec![path[..root_length].to_string()];

    let bytes = path.as_bytes();
    let mut i = root_length;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i] == b'/' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let start = i;
        while i < bytes.len() && bytes[i] != b'/' {
            i += 1;
        }
        let component = &path[start..i];
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            if components.len() > 1 {
                if components.last().unwrap() != ".." {
                    components.pop();
                    continue;
                }
            } else if !components[0].is_empty() {
                continue;
            }
        }
        components.push(component.to_string());
    }
    components
}

pub(crate) fn has_relative_path_segment(p: &str) -> bool {
    let n = p.len();
    if n == 0 {
        return false;
    }
    if p == "." || p == ".." {
        return true;
    }
    let bytes = p.as_bytes();

    if bytes[0] == b'.' {
        if n >= 2 && bytes[1] == b'/' {
            return true;
        }
        if n >= 3 && bytes[1] == b'.' && bytes[2] == b'/' {
            return true;
        }
    }

    if bytes[n - 1] == b'.' {
        if n >= 2 && bytes[n - 2] == b'/' {
            return true;
        }
        if n >= 3 && bytes[n - 2] == b'.' && bytes[n - 3] == b'/' {
            return true;
        }
    }

    let mut prev_slash = false;
    let mut seg_len = 0;
    let mut dot_count: i32 = 0;
    for &c in bytes {
        if c == b'/' {
            if prev_slash {
                return true;
            }
            if (seg_len == 1 && dot_count == 1) || (seg_len == 2 && dot_count == 2) {
                return true;
            }
            prev_slash = true;
            seg_len = 0;
            dot_count = 0;
            continue;
        }
        if c == b'.' {
            if dot_count >= 0 {
                dot_count += 1;
            }
        } else {
            dot_count = -1;
        }
        seg_len += 1;
        prev_slash = false;
    }
    (seg_len == 1 && dot_count == 1) || (seg_len == 2 && dot_count == 2)
}

pub fn get_canonical_file_name(file_name: &str, use_case_sensitive_file_names: bool) -> String {
    if use_case_sensitive_file_names {
        file_name.to_string()
    } else {
        to_file_name_lower_case(file_name)
    }
}

pub fn to_file_name_lower_case(file_name: &str) -> String {
    const I_WITH_DOT: char = '\u{0130}';
    if file_name.is_ascii() {
        return file_name.to_ascii_lowercase();
    }
    file_name
        .chars()
        .map(|r| {
            if r == I_WITH_DOT {
                r
            } else {
                r.to_lowercase()
                    .collect::<String>()
                    .chars()
                    .next()
                    .unwrap_or(r)
            }
        })
        .collect()
}

pub fn to_path(file_name: &str, base_path: &str, use_case_sensitive_file_names: bool) -> Path {
    let non_canonicalized_path = if is_rooted_disk_path(file_name) {
        normalize_path(file_name)
    } else {
        let combined = combine_paths(base_path, &[file_name]);
        normalize_path(&combined)
    };
    Path(get_canonical_file_name(
        &non_canonicalized_path,
        use_case_sensitive_file_names,
    ))
}

pub fn remove_trailing_directory_separator(path: &str) -> String {
    if has_trailing_directory_separator(path) {
        path[..path.len() - 1].to_string()
    } else {
        path.to_string()
    }
}

pub fn remove_trailing_directory_separators(path: &str) -> String {
    let mut result = path.to_string();
    while has_trailing_directory_separator(&result) {
        result.pop();
    }
    result
}

pub fn ensure_trailing_directory_separator(path: &str) -> String {
    if has_trailing_directory_separator(path) {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

pub fn get_base_file_name(path: &str) -> String {
    let path = normalize_slashes(path);
    let root_length = get_root_length(&path);
    if root_length == path.len() {
        return String::new();
    }
    let path = remove_trailing_directory_separator(&path);
    let last_slash = path.rfind('/').map_or(root_length, |i| {
        if i + 1 < root_length {
            root_length
        } else {
            i + 1
        }
    });
    path[last_slash..].to_string()
}

pub fn path_is_relative(path: &str) -> bool {
    if path == "." || path == ".." {
        return true;
    }
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'.' && (bytes[1] == b'/' || bytes[1] == b'\\') {
        return true;
    }
    if bytes.len() >= 3
        && bytes[0] == b'.'
        && bytes[1] == b'.'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
    {
        return true;
    }
    false
}

pub fn ensure_path_is_non_module_name(path: &str) -> String {
    if !path_is_absolute(path) && !path_is_relative(path) {
        format!("./{}", path)
    } else {
        path.to_string()
    }
}

pub fn is_external_module_name_relative(module_name: &str) -> bool {
    path_is_relative(module_name) || is_rooted_disk_path(module_name)
}

pub fn has_extension(file_name: &str) -> bool {
    get_base_file_name(file_name).contains('.')
}

pub fn file_extension_is(path: &str, extension: &str) -> bool {
    path.len() > extension.len() && path.ends_with(extension)
}

pub fn for_each_ancestor_directory<F>(directory: &str, mut callback: F)
where
    F: FnMut(&str) -> bool,
{
    let mut directory = directory.to_string();
    loop {
        if callback(&directory) {
            return;
        }
        let parent_path = get_directory_path(&directory);
        if parent_path == directory {
            return;
        }
        directory = parent_path;
    }
}

pub const EXTENSION_TS: &str = ".ts";
pub const EXTENSION_TSX: &str = ".tsx";
pub const EXTENSION_DTS: &str = ".d.ts";
pub const EXTENSION_JS: &str = ".js";
pub const EXTENSION_JSX: &str = ".jsx";
pub const EXTENSION_JSON: &str = ".json";
pub const EXTENSION_TS_BUILD_INFO: &str = ".tsbuildinfo";
pub const EXTENSION_MJS: &str = ".mjs";
pub const EXTENSION_MTS: &str = ".mts";
pub const EXTENSION_DMTS: &str = ".d.mts";
pub const EXTENSION_CJS: &str = ".cjs";
pub const EXTENSION_CTS: &str = ".cts";
pub const EXTENSION_DCTS: &str = ".d.cts";
