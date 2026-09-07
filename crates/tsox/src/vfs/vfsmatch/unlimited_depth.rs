use crate::tspath;
use crate::vfs::FS;

use super::*;

pub const UNLIMITED_DEPTH: i32 = i32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Usage {
    Files,
    Directories,
    Exclude,
}

pub fn read_directory(
    host: &dyn FS,
    current_dir: &str,
    path: &str,
    extensions: &[&str],
    excludes: &[&str],
    includes: &[&str],
    depth: i32,
) -> Vec<String> {
    match_files(
        path,
        extensions,
        excludes,
        includes,
        host.use_case_sensitive_file_names(),
        current_dir,
        depth,
        host,
    )
}

pub fn is_implicit_glob(last_path_component: &str) -> bool {
    !last_path_component.contains('.')
        && !last_path_component.contains('*')
        && !last_path_component.contains('?')
}

pub(crate) const WILDCARD_CHARS: &[char] = &['*', '?'];

pub(crate) fn get_include_base_path(absolute: &str) -> String {
    let wildcard_offset = absolute.find(|c: char| WILDCARD_CHARS.contains(&c));
    match wildcard_offset {
        None => {
            if !tspath::has_extension(absolute) {
                absolute.to_string()
            } else {
                tspath::remove_trailing_directory_separator(&tspath::get_directory_path(absolute))
            }
        }
        Some(woff) => {
            let prefix = &absolute[..woff];
            let last_slash = prefix.rfind('/').map_or(0, |i| i);
            absolute[..last_slash].to_string()
        }
    }
}

pub fn get_base_paths(
    path: &str,
    includes: &[&str],
    use_case_sensitive_file_names: bool,
) -> Vec<String> {
    let mut base_paths: Vec<String> = vec![path.to_string()];

    if !includes.is_empty() {
        let options = tspath::ComparePathsOptions {
            current_directory: path.to_string(),
            use_case_sensitive_file_names,
        };

        let mut include_base_paths: Vec<String> = Vec::new();
        for include in includes {
            let absolute = if tspath::is_rooted_disk_path(include) {
                include.to_string()
            } else {
                tspath::normalize_path(&tspath::combine_paths(path, &[include]))
            };
            include_base_paths.push(get_include_base_path(&absolute));
        }

        let case_sensitive = use_case_sensitive_file_names;
        include_base_paths.sort_by(|a, b| {
            if case_sensitive {
                a.cmp(b)
            } else {
                a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase())
            }
        });

        for include_base_path in &include_base_paths {
            let is_new = base_paths
                .iter()
                .all(|bp| !contains_path(bp, include_base_path, &options));
            if is_new {
                base_paths.push(include_base_path.clone());
            }
        }
    }

    base_paths
}

pub(crate) fn contains_path(
    parent: &str,
    child: &str,
    options: &tspath::ComparePathsOptions,
) -> bool {
    let parent_components = tspath::reduce_path_components(&tspath::get_path_components(
        parent,
        &options.current_directory,
    ));
    let child_components = tspath::reduce_path_components(&tspath::get_path_components(
        child,
        &options.current_directory,
    ));

    if child_components.len() < parent_components.len() {
        return false;
    }

    let case_sensitive = options.use_case_sensitive_file_names;
    for (i, pc) in parent_components.iter().enumerate() {
        let cc = &child_components[i];
        if i == 0 {
            if !pc.eq_ignore_ascii_case(cc) {
                return false;
            }
        } else if case_sensitive {
            if pc != cc {
                return false;
            }
        } else if !pc.eq_ignore_ascii_case(cc) {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComponentKind {
    Literal,
    Wildcard,
    DoubleAsterisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SegmentKind {
    Literal,
    Star,
    Question,
}

#[derive(Debug, Clone)]
pub(crate) struct Segment {
    pub(crate) kind: SegmentKind,
    pub(crate) literal: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Component {
    pub(crate) kind: ComponentKind,
    pub(crate) literal: String,
    pub(crate) segments: Vec<Segment>,
    pub(crate) skip_package_folders: bool,
}

#[derive(Debug, Clone)]
pub struct GlobPattern {
    pub(crate) components: Vec<Component>,
    pub(crate) is_exclude: bool,
    pub(crate) case_sensitive: bool,
    pub(crate) exclude_min_js: bool,
}

pub fn compile_glob_pattern(
    spec: &str,
    base_path: &str,
    usage: Usage,
    case_sensitive: bool,
) -> Option<GlobPattern> {
    let mut parts = get_normalized_path_components(spec, base_path);

    if usage != Usage::Exclude {
        if let Some(last) = parts.last() {
            if last == "**" {
                return None;
            }
        }
    }

    if let Some(first) = parts.first_mut() {
        *first = tspath::remove_trailing_directory_separator(first);
    }

    if let Some(last) = parts.last() {
        if is_implicit_glob(last) {
            parts.push("**".to_string());
            parts.push("*".to_string());
        }
    }

    let is_include = usage != Usage::Exclude;
    let mut components = Vec::with_capacity(parts.len());
    for part in &parts {
        components.push(parse_component(part, is_include));
    }

    Some(GlobPattern {
        components,
        is_exclude: usage == Usage::Exclude,
        case_sensitive,
        exclude_min_js: usage == Usage::Files,
    })
}

pub(crate) fn get_normalized_path_components(path: &str, current_directory: &str) -> Vec<String> {
    let combined = tspath::combine_paths(current_directory, &[path]);
    let normalized = tspath::normalize_path(&combined);
    tspath::reduce_path_components(&tspath::get_path_components(&normalized, ""))
}

pub(crate) fn parse_component(s: &str, is_include: bool) -> Component {
    if s == "**" {
        return Component {
            kind: ComponentKind::DoubleAsterisk,
            literal: String::new(),
            segments: Vec::new(),
            skip_package_folders: false,
        };
    }
    if !s.contains('*') && !s.contains('?') {
        return Component {
            kind: ComponentKind::Literal,
            literal: s.to_string(),
            segments: Vec::new(),
            skip_package_folders: false,
        };
    }
    Component {
        kind: ComponentKind::Wildcard,
        literal: String::new(),
        segments: parse_segments(s),
        skip_package_folders: is_include,
    }
}

pub(crate) fn parse_segments(s: &str) -> Vec<Segment> {
    let wildcards = s.bytes().filter(|&b| b == b'*' || b == b'?').count();
    let mut result = Vec::with_capacity(2 * wildcards + 1);
    let mut start = 0usize;
    for (i, b) in s.bytes().enumerate() {
        match b {
            b'*' | b'?' => {
                if i > start {
                    result.push(Segment {
                        kind: SegmentKind::Literal,
                        literal: s[start..i].to_string(),
                    });
                }
                result.push(Segment {
                    kind: if b == b'*' {
                        SegmentKind::Star
                    } else {
                        SegmentKind::Question
                    },
                    literal: String::new(),
                });
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        result.push(Segment {
            kind: SegmentKind::Literal,
            literal: s[start..].to_string(),
        });
    }
    result
}
