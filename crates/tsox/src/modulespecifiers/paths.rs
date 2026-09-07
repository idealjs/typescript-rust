use super::types::{ModulePath, ModuleSpecifierEnding, NodeModulePathParts};
use crate::tspath::{self, ComparePathsOptions};

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

    let bytes = full_path_bytes(full_path);
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

#[allow(non_snake_case)]
fn full_path_bytes(s: &str) -> Vec<u8> {
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
