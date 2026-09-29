#![allow(dead_code)]

use crate::stringutil;
use crate::tspath::directory_separator::{
    combine_paths, get_directory_path, get_path_components, get_path_from_path_components,
    get_root_length, reduce_path_components, Path,
};
use crate::tspath::get_normalized_absolute_path::{
    ensure_path_is_non_module_name, file_extension_is, for_each_ancestor_directory,
    get_normalized_path_components_from_combined, has_relative_path_segment,
    remove_trailing_directory_separator, EXTENSION_CJS, EXTENSION_CTS, EXTENSION_DCTS,
    EXTENSION_DMTS, EXTENSION_DTS, EXTENSION_JS, EXTENSION_JSX, EXTENSION_MJS, EXTENSION_MTS,
    EXTENSION_TS, EXTENSION_TSX,
};
use crate::tspath::supported_ts_extensions_flat::{
    change_extension, file_extension_is_one_of, get_any_extension_from_path,
    get_declaration_file_extension, get_path_components_relative_to, is_declaration_file_name,
    remove_extension, remove_file_extension, ComparePathsOptions,
    SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS,
};

pub fn compare_number_of_directory_separators(path1: &str, path2: &str) -> i32 {
    match path1.matches('/').count().cmp(&path2.matches('/').count()) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

impl ComparePathsOptions {
    pub fn get_comparer(&self) -> impl Fn(&str, &str) -> i32 {
        let case_sensitive = self.use_case_sensitive_file_names;
        move |a: &str, b: &str| {
            if case_sensitive {
                stringutil::compare_strings_case_sensitive(a, b)
            } else {
                stringutil::compare_strings_case_insensitive(a, b)
            }
        }
    }
}

pub fn compare_paths(a: &str, b: &str, options: &ComparePathsOptions) -> i32 {
    let a = combine_paths(&options.current_directory, &[a]);
    let b = combine_paths(&options.current_directory, &[b]);

    if a == b {
        return 0;
    }
    if a.is_empty() {
        return -1;
    }
    if b.is_empty() {
        return 1;
    }

    let a_root_length = get_root_length(&a);
    let b_root_length = get_root_length(&b);
    let a_root = &a[..a_root_length];
    let b_root = &b[..b_root_length];
    let result = stringutil::compare_strings_case_insensitive(a_root, b_root);
    if result != 0 {
        return result;
    }

    let a_rest = &a[a_root_length..];
    let b_rest = &b[b_root_length..];
    if !has_relative_path_segment(a_rest) && !has_relative_path_segment(b_rest) {
        return options.get_comparer()(a_rest, b_rest);
    }

    let a_components = reduce_path_components(&get_path_components(&a, ""));
    let b_components = reduce_path_components(&get_path_components(&b, ""));
    let shared_length = a_components.len().min(b_components.len());
    let comparer = options.get_comparer();
    for i in 1..shared_length {
        let result = comparer(&a_components[i], &b_components[i]);
        if result != 0 {
            return result;
        }
    }
    match a_components.len().cmp(&b_components.len()) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

pub fn compare_paths_case_sensitive(a: &str, b: &str, current_directory: &str) -> i32 {
    compare_paths(
        a,
        b,
        &ComparePathsOptions {
            use_case_sensitive_file_names: true,
            current_directory: current_directory.to_string(),
        },
    )
}

pub fn compare_paths_case_insensitive(a: &str, b: &str, current_directory: &str) -> i32 {
    compare_paths(
        a,
        b,
        &ComparePathsOptions {
            use_case_sensitive_file_names: false,
            current_directory: current_directory.to_string(),
        },
    )
}

pub fn for_each_ancestor_directory_path<T, F>(directory: &Path, mut callback: F) -> Option<T>
where
    F: FnMut(&Path) -> Option<T>,
{
    let mut directory = directory.clone();
    loop {
        if let Some(result) = callback(&directory) {
            return Some(result);
        }
        let parent_path = directory.get_directory_path();
        if parent_path == directory {
            return None;
        }
        directory = parent_path;
    }
}

pub fn for_each_ancestor_directory_stopping_at_global_cache<T, F>(
    global_cache_location: &str,
    directory: &str,
    mut callback: F,
) -> Option<T>
where
    F: FnMut(&str) -> Option<T>,
{
    let mut result: Option<T> = None;
    for_each_ancestor_directory(directory, |ancestor_directory| {
        if let Some(value) = callback(ancestor_directory) {
            result = Some(value);
            return true;
        }
        ancestor_directory == global_cache_location
    });
    result
}

fn try_get_extension_from_path(
    path: &str,
    extension: &str,
    ignore_case: bool,
) -> String {
    let extension = if extension.starts_with('.') {
        extension.to_string()
    } else {
        format!(".{}", extension)
    };
    if path.len() >= extension.len()
        && path.as_bytes()[path.len() - extension.len()] == b'.'
    {
        let path_extension = &path[path.len() - extension.len()..];
        if stringutil::equate_string_case_insensitive(path_extension, &extension)
            || (!ignore_case && path_extension == extension)
        {
            return path_extension.to_string();
        }
    }
    String::new()
}

pub fn get_longest_extension_from_path(
    path: &str,
    extensions: &[&str],
    ignore_case: bool,
) -> String {
    let path = remove_trailing_directory_separator(path);
    let mut longest = String::new();
    for extension in extensions {
        if extension.len() > longest.len() {
            let matched = try_get_extension_from_path(&path, extension, ignore_case);
            if !matched.is_empty() {
                longest = matched;
            }
        }
    }
    longest
}

pub fn get_normalized_path_components(path: &str, current_directory: &str) -> Vec<String> {
    let combined = combine_paths(current_directory, &[path]);
    get_normalized_path_components_from_combined(&combined)
}

pub fn get_relative_path_from_directory(
    from_directory: &str,
    to: &str,
    options: &ComparePathsOptions,
) -> String {
    if (get_root_length(from_directory) > 0) != (get_root_length(to) > 0) {
        panic!("paths must either both be absolute or both be relative");
    }
    let path_components = get_path_components_relative_to(from_directory, to, options);
    get_path_from_path_components(&path_components)
}

pub fn get_relative_path_from_file(
    from: &str,
    to: &str,
    options: &ComparePathsOptions,
) -> String {
    ensure_path_is_non_module_name(&get_relative_path_from_directory(
        &get_directory_path(from),
        to,
        options,
    ))
}

const SUPPORTED_TS_EXTENSIONS_FOR_EXTRACT_EXTENSION: &[&str] = &[
    EXTENSION_DTS,
    EXTENSION_DCTS,
    EXTENSION_DMTS,
    EXTENSION_TS,
    EXTENSION_TSX,
    EXTENSION_MTS,
    EXTENSION_CTS,
];

pub fn try_extract_ts_extension(file_name: &str) -> &'static str {
    for ext in SUPPORTED_TS_EXTENSIONS_FOR_EXTRACT_EXTENSION {
        if file_extension_is(file_name, ext) {
            return ext;
        }
    }
    ""
}

pub fn remove_any_file_extension(path: &str) -> String {
    let without_extension = remove_file_extension(path);
    if without_extension != path {
        return without_extension;
    }
    let extension = get_any_extension_from_path(path, &[], false);
    if !extension.is_empty() {
        return remove_extension(path, &extension);
    }
    path.to_string()
}

pub fn has_implementation_ts_file_extension(path: &str) -> bool {
    file_extension_is_one_of(path, SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS)
        && !is_declaration_file_name(path)
}

pub fn extension_is_one_of(ext: &str, extensions: &[&str]) -> bool {
    extensions.contains(&ext)
}

pub fn get_declaration_emit_extension_for_path(path: &str) -> String {
    if file_extension_is_one_of(path, &[EXTENSION_MJS, EXTENSION_MTS]) {
        return EXTENSION_DMTS.to_string();
    }
    if file_extension_is_one_of(path, &[EXTENSION_CJS, EXTENSION_CTS]) {
        return EXTENSION_DCTS.to_string();
    }
    if file_extension_is_one_of(path, &[EXTENSION_TS, EXTENSION_TSX, EXTENSION_JS, EXTENSION_JSX])
    {
        return EXTENSION_DTS.to_string();
    }
    let ext = get_any_extension_from_path(path, &[], false);
    if !ext.is_empty() {
        return format!(".d{}.ts", ext);
    }
    EXTENSION_DTS.to_string()
}

pub fn change_any_extension(
    path: &str,
    ext: &str,
    extensions: &[&str],
    ignore_case: bool,
) -> String {
    let pathext = get_any_extension_from_path(path, extensions, ignore_case);
    if !pathext.is_empty() {
        let result = &path[..path.len() - pathext.len()];
        if ext.is_empty() {
            return result.to_string();
        }
        if ext.starts_with('.') {
            return format!("{}{}", result, ext);
        }
        return format!("{}.{}", result, ext);
    }
    path.to_string()
}

pub fn change_full_extension(path: &str, new_extension: &str) -> String {
    let declaration_extension = get_declaration_file_extension(path);
    if !declaration_extension.is_empty() {
        let ext = if new_extension.starts_with('.') {
            new_extension
        } else {
            &format!(".{}", new_extension)
        };
        return format!(
            "{}{}",
            &path[..path.len() - declaration_extension.len()],
            ext
        );
    }
    change_extension(path, new_extension)
}

pub fn get_possible_original_input_extension_for_extension(path: &str) -> Vec<String> {
    if file_extension_is_one_of(path, &[EXTENSION_DMTS, EXTENSION_MJS, EXTENSION_MTS]) {
        return vec![EXTENSION_MTS.to_string(), EXTENSION_MJS.to_string()];
    }
    if file_extension_is_one_of(path, &[EXTENSION_DCTS, EXTENSION_CJS, EXTENSION_CTS]) {
        return vec![EXTENSION_CTS.to_string(), EXTENSION_CJS.to_string()];
    }
    let ext = get_declaration_file_extension(path);
    if !ext.is_empty() && ext != EXTENSION_DTS {
        let inner = &ext[".d.".len()..ext.len() - ".ts".len()];
        return vec![format!(".{}", inner)];
    }
    vec![
        EXTENSION_TSX.to_string(),
        EXTENSION_TS.to_string(),
        EXTENSION_JSX.to_string(),
        EXTENSION_JS.to_string(),
    ]
}
