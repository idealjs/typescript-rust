#![allow(unused_imports)]

use super::*;

pub const SUPPORTED_TS_EXTENSIONS_FLAT: &[&str] = &[
    EXTENSION_TS,
    EXTENSION_TSX,
    EXTENSION_DTS,
    EXTENSION_CTS,
    EXTENSION_DCTS,
    EXTENSION_MTS,
    EXTENSION_DMTS,
];

pub const SUPPORTED_JS_EXTENSIONS_FLAT: &[&str] =
    &[EXTENSION_JS, EXTENSION_JSX, EXTENSION_MJS, EXTENSION_CJS];

pub const SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS: &[&str] =
    &[EXTENSION_TS, EXTENSION_TSX, EXTENSION_MTS, EXTENSION_CTS];

pub const SUPPORTED_DECLARATION_EXTENSIONS: &[&str] =
    &[EXTENSION_DTS, EXTENSION_DCTS, EXTENSION_DMTS];

pub const EXTENSIONS_TO_REMOVE: &[&str] = &[
    EXTENSION_DTS,
    EXTENSION_DMTS,
    EXTENSION_DCTS,
    EXTENSION_MJS,
    EXTENSION_MTS,
    EXTENSION_CJS,
    EXTENSION_CTS,
    EXTENSION_TS,
    EXTENSION_JS,
    EXTENSION_TSX,
    EXTENSION_JSX,
    EXTENSION_JSON,
];

pub fn extension_is_ts(ext: &str) -> bool {
    ext == EXTENSION_TS
        || ext == EXTENSION_TSX
        || ext == EXTENSION_DTS
        || ext == EXTENSION_MTS
        || ext == EXTENSION_DMTS
        || ext == EXTENSION_CTS
        || ext == EXTENSION_DCTS
        || (ext.len() >= 7 && &ext[..3] == ".d." && &ext[ext.len() - 3..] == ".ts")
}

pub fn remove_file_extension(path: &str) -> String {
    for ext in EXTENSIONS_TO_REMOVE {
        if path.ends_with(ext) {
            return path[..path.len() - ext.len()].to_string();
        }
    }
    path.to_string()
}

pub fn try_get_extension_from_path(p: &str) -> &str {
    for ext in EXTENSIONS_TO_REMOVE {
        if file_extension_is(p, ext) {
            return ext;
        }
    }
    ""
}

pub fn remove_extension(path: &str, extension: &str) -> String {
    path[..path.len() - extension.len()].to_string()
}

pub fn file_extension_is_one_of(path: &str, extensions: &[&str]) -> bool {
    extensions.iter().any(|ext| file_extension_is(path, ext))
}

pub fn has_ts_file_extension(path: &str) -> bool {
    file_extension_is_one_of(path, SUPPORTED_TS_EXTENSIONS_FLAT)
}

pub fn has_js_file_extension(path: &str) -> bool {
    file_extension_is_one_of(path, SUPPORTED_JS_EXTENSIONS_FLAT)
}

pub fn has_json_file_extension(path: &str) -> bool {
    file_extension_is(path, EXTENSION_JSON)
}

pub fn is_declaration_file_name(file_name: &str) -> bool {
    !get_declaration_file_extension(file_name).is_empty()
}

pub fn get_declaration_file_extension(file_name: &str) -> String {
    let base = get_base_file_name(file_name);
    for ext in &[EXTENSION_DTS, EXTENSION_DCTS, EXTENSION_DMTS] {
        if base.ends_with(ext) {
            return ext.to_string();
        }
    }
    if base.ends_with(EXTENSION_TS) {
        if let Some(index) = base.find(".d.") {
            return base[index..].to_string();
        }
    }
    String::new()
}

pub fn change_extension(path: &str, new_extension: &str) -> String {
    let pathext = get_any_extension_from_path(path, &[], false);
    if !pathext.is_empty() {
        let result = &path[..path.len() - pathext.len()];
        if new_extension.is_empty() {
            return result.to_string();
        }
        if new_extension.starts_with('.') {
            return format!("{}{}", result, new_extension);
        }
        return format!("{}.{}", result, new_extension);
    }
    path.to_string()
}

pub fn get_any_extension_from_path(path: &str, extensions: &[&str], ignore_case: bool) -> String {
    if !extensions.is_empty() {
        let path = remove_trailing_directory_separator(path);
        for extension in extensions {
            let ext = if extension.starts_with('.') {
                extension.to_string()
            } else {
                format!(".{}", extension)
            };
            if path.len() >= ext.len() && path.as_bytes()[path.len() - ext.len()] == b'.' {
                let path_extension = &path[path.len() - ext.len()..];
                if stringutil::equate_string_case_insensitive(path_extension, &ext)
                    || (!ignore_case && path_extension == ext)
                {
                    return path_extension.to_string();
                }
            }
        }
        return String::new();
    }
    let base = get_base_file_name(path);
    if let Some(idx) = base.rfind('.') {
        return base[idx..].to_string();
    }
    String::new()
}

pub fn contains_ignored_path(path: &str) -> bool {
    let ignored_paths = ["/node_modules/.", "/.git", ".#"];
    ignored_paths.iter().any(|p| path.contains(p))
}

pub fn starts_with_directory(
    file_name: &str,
    directory_name: &str,
    use_case_sensitive_file_names: bool,
) -> bool {
    if directory_name.is_empty() {
        return false;
    }

    let canonical_file_name = get_canonical_file_name(file_name, use_case_sensitive_file_names);
    let mut canonical_directory_name =
        get_canonical_file_name(directory_name, use_case_sensitive_file_names);

    if canonical_directory_name.ends_with('/') {
        canonical_directory_name.pop();
    }
    if canonical_directory_name.ends_with('\\') {
        canonical_directory_name.pop();
    }

    canonical_file_name.starts_with(&format!("{}/", canonical_directory_name))
        || canonical_file_name.starts_with(&format!("{}\\", canonical_directory_name))
}

#[derive(Debug, Clone, Default)]
pub struct ComparePathsOptions {
    pub use_case_sensitive_file_names: bool,
    pub current_directory: String,
}

impl ComparePathsOptions {
    pub(crate) fn equality_comparer(&self) -> impl Fn(&str, &str) -> bool {
        let case_sensitive = self.use_case_sensitive_file_names;
        move |a: &str, b: &str| -> bool {
            if case_sensitive {
                a == b
            } else {
                a.eq_ignore_ascii_case(b)
            }
        }
    }
}

pub fn get_normalized_absolute_path_without_root(
    file_name: &str,
    current_directory: &str,
) -> String {
    let absolute_path = get_normalized_absolute_path(file_name, current_directory);
    let root_length = get_root_length(&absolute_path);
    absolute_path[root_length..].to_string()
}

pub(crate) fn get_path_components_relative_to(
    from: &str,
    to: &str,
    options: &ComparePathsOptions,
) -> Vec<String> {
    let from_components =
        reduce_path_components(&get_path_components(from, &options.current_directory));
    let to_components =
        reduce_path_components(&get_path_components(to, &options.current_directory));

    let max_common = from_components.len().min(to_components.len());
    let equality = options.equality_comparer();
    let mut start = 0;
    while start < max_common {
        let from_component = &from_components[start];
        let to_component = &to_components[start];
        if start == 0 {
            if !from_component.eq_ignore_ascii_case(to_component) {
                break;
            }
        } else if !equality(from_component, to_component) {
            break;
        }
        start += 1;
    }

    if start == 0 {
        return to_components;
    }

    let num_dot_dot = from_components.len() - start;
    let mut result = Vec::with_capacity(1 + num_dot_dot + (to_components.len() - start));
    result.push(String::new());
    for _ in 0..num_dot_dot {
        result.push("..".to_string());
    }
    for component in &to_components[start..] {
        result.push(component.clone());
    }
    result
}

pub fn get_relative_path_to_directory_or_url(
    directory_path_or_url: &str,
    relative_or_absolute_path: &str,
    is_absolute_path_an_url: bool,
    options: &ComparePathsOptions,
) -> String {
    let mut path_components =
        get_path_components_relative_to(directory_path_or_url, relative_or_absolute_path, options);

    if !path_components.is_empty() {
        let first_component = &path_components[0];
        if is_absolute_path_an_url && is_rooted_disk_path(first_component) {
            let prefix = if first_component.starts_with('/') {
                "file://"
            } else {
                "file:///"
            };
            path_components[0] = format!("{prefix}{first_component}");
        }
    }

    get_path_from_path_components(&path_components)
}

pub fn convert_to_relative_path(path: &str, options: &ComparePathsOptions) -> String {
    if !is_rooted_disk_path(path) {
        return path.to_string();
    }
    get_relative_path_to_directory_or_url(&options.current_directory, path, false, options)
}
