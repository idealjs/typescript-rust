#![allow(unused_imports)]

use super::*;

pub const DIRECTORY_SEPARATOR: char = '/';
pub(crate) const URL_SCHEME_SEPARATOR: &str = "://";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Path(pub String);

impl Path {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn get_directory_path(&self) -> Path {
        Path(get_directory_path(&self.0))
    }

    pub fn remove_trailing_directory_separator(&self) -> Path {
        Path(remove_trailing_directory_separator(&self.0))
    }

    pub fn ensure_trailing_directory_separator(&self) -> Path {
        Path(ensure_trailing_directory_separator(&self.0))
    }

    pub fn contains_path(&self, child: &Path) -> bool {
        if self.0.is_empty() {
            return false;
        }
        self.0 == child.0
            || (child.0.len() > self.0.len()
                && child.0.starts_with(&self.0)
                && (self.0.ends_with('/') || child.0.as_bytes()[self.0.len()] == b'/'))
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Path {
    fn from(s: &str) -> Path {
        Path(s.to_string())
    }
}

impl From<String> for Path {
    fn from(s: String) -> Path {
        Path(s)
    }
}

pub(crate) fn is_any_directory_separator(char: u8) -> bool {
    char == b'/' || char == b'\\'
}

pub fn is_url(path: &str) -> bool {
    get_encoded_root_length(path) < 0
}

pub fn is_rooted_disk_path(path: &str) -> bool {
    get_encoded_root_length(path) > 0
}

pub fn is_disk_path_root(path: &str) -> bool {
    let root_length = get_encoded_root_length(path);
    root_length > 0 && root_length as usize == path.len()
}

pub fn is_dynamic_file_name(file_name: &str) -> bool {
    file_name.starts_with("^/")
}

pub fn path_is_absolute(path: &str) -> bool {
    get_encoded_root_length(path) != 0
}

pub fn has_trailing_directory_separator(path: &str) -> bool {
    !path.is_empty() && is_any_directory_separator(*path.as_bytes().last().unwrap())
}

pub fn combine_paths(first_path: &str, paths: &[&str]) -> String {
    let first_path = normalize_slashes(first_path);
    let mut result = first_path;

    for trailing_path in paths {
        if trailing_path.is_empty() {
            continue;
        }
        let trailing_path = normalize_slashes(trailing_path);
        if result.is_empty() || get_root_length(&trailing_path) != 0 {
            result = trailing_path;
        } else {
            if !has_trailing_directory_separator(&result) {
                result.push(DIRECTORY_SEPARATOR);
            }
            result.push_str(&trailing_path);
        }
    }
    result
}

pub fn get_path_components(path: &str, current_directory: &str) -> Vec<String> {
    let combined = combine_paths(current_directory, &[path]);
    let root_length = get_root_length(&combined);
    path_components(&combined, root_length)
}

pub(crate) fn path_components(path: &str, root_length: usize) -> Vec<String> {
    let root = &path[..root_length];
    let rest: Vec<&str> = path[root_length..].split('/').collect();
    let mut components = vec![root.to_string()];
    for part in &rest {
        if !part.is_empty() {
            components.push(part.to_string());
        }
    }
    components
}

pub fn is_volume_character(char: u8) -> bool {
    char.is_ascii_alphabetic()
}

pub fn get_encoded_root_length(path: &str) -> i32 {
    let bytes = path.as_bytes();
    let ln = bytes.len();
    if ln == 0 {
        return 0;
    }
    let ch0 = bytes[0];

    if ch0 == b'/' || ch0 == b'\\' {
        if ln == 1 || bytes[1] != ch0 {
            return 1;
        }
        let offset = 2;
        if let Some(p1) = path[offset..].find(|c| c == ch0 as char) {
            return (p1 + offset + 1) as i32;
        }
        return ln as i32;
    }

    if is_volume_character(ch0) && ln > 1 && bytes[1] == b':' {
        if ln == 2 {
            return 2;
        }
        let ch2 = bytes[2];
        if ch2 == b'/' || ch2 == b'\\' {
            return 3;
        }
    }

    if ch0 == b'^' && ln > 1 && bytes[1] == b'/' {
        return 2;
    }

    if let Some(scheme_end) = path.find(URL_SCHEME_SEPARATOR) {
        let authority_start = scheme_end + URL_SCHEME_SEPARATOR.len();
        if let Some(authority_length) = path[authority_start..].find('/') {
            let authority_end = authority_start + authority_length;
            let scheme = &path[..scheme_end];
            let authority = &path[authority_start..authority_end];

            if scheme == "file"
                && (authority.is_empty() || authority == "localhost")
                && path.len() > authority_end + 2
                && is_volume_character(bytes[authority_end + 1])
            {
                let volume_separator_end =
                    get_file_url_volume_separator_end(path, authority_end + 2);
                if volume_separator_end != -1 {
                    if volume_separator_end as usize == path.len() {
                        return !(volume_separator_end);
                    }
                    if path.as_bytes()[volume_separator_end as usize] == b'/' {
                        return !(volume_separator_end + 1);
                    }
                }
            }
            return !(authority_end as i32 + 1);
        }
        return !(ln as i32);
    }

    0
}

pub(crate) fn get_file_url_volume_separator_end(url: &str, start: usize) -> i32 {
    if url.len() <= start {
        return -1;
    }
    let ch0 = url.as_bytes()[start];
    if ch0 == b':' {
        return (start + 1) as i32;
    }
    if ch0 == b'%' && url.len() > start + 2 && url.as_bytes()[start + 1] == b'3' {
        let ch2 = url.as_bytes()[start + 2];
        if ch2 == b'a' || ch2 == b'A' {
            return (start + 3) as i32;
        }
    }
    -1
}

pub fn get_root_length(path: &str) -> usize {
    let root_length = get_encoded_root_length(path);
    if root_length < 0 {
        (!root_length) as usize
    } else {
        root_length as usize
    }
}

pub fn get_directory_path(path: &str) -> String {
    let path = normalize_slashes(path);
    let root_length = get_root_length(&path);
    if root_length == path.len() {
        return path;
    }
    let path = remove_trailing_directory_separator(&path);
    let last_slash =
        path.rfind('/').map_or(
            root_length,
            |i| {
                if i < root_length { root_length } else { i }
            },
        );
    path[..last_slash].to_string()
}

pub fn get_path_from_path_components(components: &[String]) -> String {
    if components.is_empty() {
        return String::new();
    }
    let root = &components[0];
    let root = if root.is_empty() {
        String::new()
    } else {
        ensure_trailing_directory_separator(root)
    };
    if components.len() == 1 {
        return root;
    }
    format!("{}{}", root, components[1..].join("/"))
}

pub fn normalize_slashes(path: &str) -> String {
    path.replace('\\', "/")
}

pub fn reduce_path_components(components: &[String]) -> Vec<String> {
    if components.is_empty() {
        return vec![];
    }
    let mut reduced = vec![components[0].clone()];
    for component in &components[1..] {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            if reduced.len() > 1 {
                if reduced.last().unwrap() != ".." {
                    reduced.pop();
                    continue;
                }
            } else if !reduced[0].is_empty() {
                continue;
            }
        }
        reduced.push(component.clone());
    }
    reduced
}

pub fn resolve_path(path: &str, paths: &[&str]) -> String {
    let combined = if !paths.is_empty() {
        combine_paths(path, paths)
    } else {
        normalize_slashes(path)
    };
    normalize_path(&combined)
}
