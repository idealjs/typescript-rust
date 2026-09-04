use std::collections::HashMap;

use crate::collections::set::Set;
use crate::core::compiler_options::CompilerOptions;
use crate::semver;
use crate::tspath;
use crate::vfs::FS;

use super::types_map::lookup_type_name;

#[derive(Debug, Clone, Default)]
pub struct TypingsInfo {
    pub type_acquisition: Option<TypeAcquisition>,
    pub compiler_options: CompilerOptions,
    pub unresolved_imports: Option<Set<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct TypeAcquisition {
    pub enable: bool,
    pub include: Option<Vec<String>>,
    pub exclude: Vec<String>,
    pub disable_filename_based_type_acquisition: crate::core::tristate::Tristate,
}

impl TypeAcquisition {
    pub fn disable_filename_based_type_acquisition_is_true(&self) -> bool {
        self.disable_filename_based_type_acquisition.is_true()
    }
}

#[derive(Debug, Clone)]
pub struct CachedTyping {
    pub typings_location: String,
    pub version: semver::Version,
}

pub trait AtaLogger: Send + Sync {
    fn log(&self, message: &str);
}

pub fn is_typing_up_to_date(
    cached_typing: &CachedTyping,
    available_typing_versions: &HashMap<String, String>,
) -> bool {
    let _use_version = available_typing_versions.get("latest");
    let use_version = match available_typing_versions.get("latest") {
        Some(v) => v.as_str(),
        None => return true,
    };
    let available_version = semver::must_parse(use_version);
    available_version.compare(&cached_typing.version) <= std::cmp::Ordering::Equal
}

pub fn discover_typings(
    _fs: &dyn FS,
    _logger: Option<&dyn AtaLogger>,
    _typings_info: &TypingsInfo,
    file_names: &[String],
    project_root_path: &str,
    _package_name_to_typing_location: &HashMap<String, CachedTyping>,
    _types_registry: &HashMap<String, HashMap<String, String>>,
) -> (Vec<String>, Vec<String>, Vec<String>) {

    let mut inferred_typings: HashMap<String, String> = HashMap::new();

    let js_file_names: Vec<&String> = file_names
        .iter()
        .filter(|f| tspath::has_js_file_extension(f))
        .collect();

    let files_to_watch: Vec<String> = Vec::new();

    if let Some(ref ta) = _typings_info.type_acquisition {
        if let Some(ref include) = ta.include {
            add_inferred_typings(&mut inferred_typings, include);
        }
    }

    let exclude: Vec<String> = _typings_info
        .type_acquisition
        .as_ref()
        .map(|ta| ta.exclude.clone())
        .unwrap_or_default();

    if _typings_info
        .type_acquisition
        .as_ref()
        .map(|ta| !ta.disable_filename_based_type_acquisition_is_true())
        .unwrap_or(true)
    {
        get_typing_names_from_source_file_names(&mut inferred_typings, &js_file_names);
    }

    for exclude_typing_name in &exclude {
        inferred_typings.remove(exclude_typing_name);
    }

    let mut cached_typing_paths = Vec::new();
    let mut new_typing_names = Vec::new();
    for (typing, inferred) in &inferred_typings {
        if !inferred.is_empty() {
            cached_typing_paths.push(inferred.clone());
        } else {
            new_typing_names.push(typing.clone());
        }
    }

    let _ = project_root_path;
    (cached_typing_paths, new_typing_names, files_to_watch)
}

fn add_inferred_typing(inferred_typings: &mut HashMap<String, String>, typing_name: &str) {
    inferred_typings.entry(typing_name.to_string()).or_default();
}

fn add_inferred_typings(inferred_typings: &mut HashMap<String, String>, typing_names: &[String]) {
    for typing_name in typing_names {
        add_inferred_typing(inferred_typings, typing_name);
    }
}

fn get_typing_names_from_source_file_names(
    inferred_typings: &mut HashMap<String, String>,
    file_names: &[&String],
) {
    let mut has_jsx_file = false;
    let mut from_file_names: Vec<String> = Vec::new();
    for file_name in file_names {
        has_jsx_file = has_jsx_file || tspath::file_extension_is(file_name, tspath::EXTENSION_JSX);
        let inferred_typing_name = tspath::remove_file_extension(&tspath::to_file_name_lower_case(
            &tspath::get_base_file_name(file_name),
        ));
        let cleaned_typing_name = remove_min_and_version_numbers(&inferred_typing_name);
        if let Some(type_name) = lookup_type_name(&cleaned_typing_name) {
            from_file_names.push(type_name.to_string());
        }
    }
    if !from_file_names.is_empty() {
        add_inferred_typings(inferred_typings, &from_file_names);
    }
    if has_jsx_file {
        add_inferred_typing(inferred_typings, "react");
    }
}

pub fn remove_min_and_version_numbers(file_name: &str) -> String {
    let bytes = file_name.as_bytes();
    let mut end = file_name.len();
    let mut pos = end;

    while pos > 0 {
        let ch = bytes[pos - 1];
        if ch >= b'0' && ch <= b'9' {

            loop {
                pos -= 1;
                if pos == 0 || !(bytes[pos - 1] >= b'0' && bytes[pos - 1] <= b'9') {
                    break;
                }
            }
        } else if pos > 4 && (ch == b'n' || ch == b'N') {

            pos -= 1;
            if pos == 0 || (bytes[pos - 1] != b'i' && bytes[pos - 1] != b'I') {
                break;
            }
            pos -= 1;
            if pos == 0 || (bytes[pos - 1] != b'm' && bytes[pos - 1] != b'M') {
                break;
            }
            pos -= 1;
        } else {
            break;
        }

        if pos == 0 {
            break;
        }
        let sep = bytes[pos - 1];
        if sep != b'-' && sep != b'.' {
            break;
        }
        pos -= 1;
        end = pos;
    }
    file_name[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_min_and_version_numbers() {
        assert_eq!(remove_min_and_version_numbers("jquery-min.4.2.3"), "jquery");
        assert_eq!(
            remove_min_and_version_numbers("angular-route.1.2.3"),
            "angular-route"
        );
        assert_eq!(remove_min_and_version_numbers("jquery"), "jquery");
        assert_eq!(remove_min_and_version_numbers("jquery.min"), "jquery");
    }
}
