use crate::tspath::directory_separator::{
    combine_paths, get_directory_path, is_rooted_disk_path, is_volume_character,
};
use crate::tspath::get_normalized_absolute_path::{
    get_canonical_file_name, has_relative_path_segment, normalize_path,
};

pub fn resolve_tripleslash_reference(module_name: &str, containing_file: &str) -> String {
    let base_path = get_directory_path(containing_file);
    if is_rooted_disk_path(module_name) {
        return normalize_path(module_name);
    }
    normalize_path(&combine_paths(&base_path, &[module_name]))
}

pub fn simple_normalize_path(path: &str) -> Option<String> {
    if !has_relative_path_segment(path) {
        return Some(path.to_string());
    }
    let simplified = path.replace("/./", "/");
    let trimmed = simplified.strip_prefix("./").unwrap_or(&simplified);
    if trimmed != path && !has_relative_path_segment(trimmed) && !(trimmed != simplified && trimmed.starts_with('/'))
    {
        return Some(trimmed.to_string());
    }
    None
}

pub fn trim_rune_count(s: &str, rune_count: usize) -> &str {
    let mut char_indices = s.char_indices();
    for _ in 0..rune_count {
        match char_indices.next() {
            Some((_, _)) => {}
            None => break,
        }
    }
    let offset = char_indices.next().map_or(s.len(), |(idx, _)| idx);
    &s[offset..]
}

pub fn trim_file_path_prefix(path: &str, prefix: &str, use_case_sensitive_file_names: bool) -> (String, bool) {
    if use_case_sensitive_file_names {
        return match path.strip_prefix(prefix) {
            Some(rest) => (rest.to_string(), true),
            None => (path.to_string(), false),
        };
    }
    let canonical_prefix = get_canonical_file_name(prefix, false);
    if !get_canonical_file_name(path, false).starts_with(&canonical_prefix) {
        return (path.to_string(), false);
    }
    (
        trim_rune_count(path, canonical_prefix.chars().count()).to_string(),
        true,
    )
}

pub fn split_volume_path(path: &str) -> Option<(String, &str)> {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && is_volume_character(bytes[0]) && bytes[1] == b':' {
        let volume = path[..2].to_lowercase();
        return Some((volume, &path[2..]));
    }
    None
}
