//! Path utilities, ported from `internal/tspath/`.
//!
//! Internally, paths are represented as strings with `/` as the directory separator.

use crate::stringutil;

pub const DIRECTORY_SEPARATOR: char = '/';
const URL_SCHEME_SEPARATOR: &str = "://";

/// A canonicalized path used as a key in maps.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

/// Whether a byte is `/` or `\`.
fn is_any_directory_separator(char: u8) -> bool {
    char == b'/' || char == b'\\'
}

/// Whether a path starts with a URL scheme.
pub fn is_url(path: &str) -> bool {
    get_encoded_root_length(path) < 0
}

/// Whether a path is an absolute disk path.
pub fn is_rooted_disk_path(path: &str) -> bool {
    get_encoded_root_length(path) > 0
}

/// Whether a path consists only of a path root.
pub fn is_disk_path_root(path: &str) -> bool {
    let root_length = get_encoded_root_length(path);
    root_length > 0 && root_length as usize == path.len()
}

/// Whether a file name is a dynamic/virtual file (e.g. `^/untitled/...`).
pub fn is_dynamic_file_name(file_name: &str) -> bool {
    file_name.starts_with("^/")
}

/// Whether a path starts with an absolute path component.
pub fn path_is_absolute(path: &str) -> bool {
    get_encoded_root_length(path) != 0
}

/// Whether a path has a trailing directory separator.
pub fn has_trailing_directory_separator(path: &str) -> bool {
    !path.is_empty() && is_any_directory_separator(*path.as_bytes().last().unwrap())
}

/// Combine paths. If a path is absolute, it replaces any previous path.
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

/// Get path components (root + each directory/file segment).
pub fn get_path_components(path: &str, current_directory: &str) -> Vec<String> {
    let combined = combine_paths(current_directory, &[path]);
    let root_length = get_root_length(&combined);
    path_components(&combined, root_length)
}

fn path_components(path: &str, root_length: usize) -> Vec<String> {
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

/// Whether a byte is a volume character (a-z, A-Z).
pub fn is_volume_character(char: u8) -> bool {
    char.is_ascii_alphabetic()
}

/// Get the encoded root length of a path. Negative values indicate URLs.
pub fn get_encoded_root_length(path: &str) -> i32 {
    let bytes = path.as_bytes();
    let ln = bytes.len();
    if ln == 0 {
        return 0;
    }
    let ch0 = bytes[0];

    // POSIX or UNC
    if ch0 == b'/' || ch0 == b'\\' {
        if ln == 1 || bytes[1] != ch0 {
            return 1; // POSIX: "/"
        }
        let offset = 2;
        if let Some(p1) = path[offset..].find(|c| c == ch0 as char) {
            return (p1 + offset + 1) as i32; // UNC: "//server/"
        }
        return ln as i32; // UNC: "//server"
    }

    // DOS
    if is_volume_character(ch0) && ln > 1 && bytes[1] == b':' {
        if ln == 2 {
            return 2; // DOS: "c:"
        }
        let ch2 = bytes[2];
        if ch2 == b'/' || ch2 == b'\\' {
            return 3; // DOS: "c:/" or "c:\"
        }
    }

    // Untitled paths
    if ch0 == b'^' && ln > 1 && bytes[1] == b'/' {
        return 2; // "^/"
    }

    // URL
    if let Some(scheme_end) = path.find(URL_SCHEME_SEPARATOR) {
        let authority_start = scheme_end + URL_SCHEME_SEPARATOR.len();
        if let Some(authority_length) = path[authority_start..].find('/') {
            let authority_end = authority_start + authority_length;
            let scheme = &path[..scheme_end];
            let authority = &path[authority_start..authority_end];

            // For local "file" URLs, include the leading DOS volume (if present).
            if scheme == "file"
                && (authority.is_empty() || authority == "localhost")
                && path.len() > authority_end + 2
                && is_volume_character(bytes[authority_end + 1])
            {
                let volume_separator_end =
                    get_file_url_volume_separator_end(path, authority_end + 2);
                if volume_separator_end != -1 {
                    if volume_separator_end as usize == path.len() {
                        return !(volume_separator_end); // URL: "file:///c:"
                    }
                    if path.as_bytes()[volume_separator_end as usize] == b'/' {
                        return !(volume_separator_end + 1); // URL: "file:///c:/"
                    }
                }
            }
            return !(authority_end as i32 + 1); // URL: "file://server/"
        }
        return !(ln as i32); // URL: "file://server"
    }

    0 // relative
}

fn get_file_url_volume_separator_end(url: &str, start: usize) -> i32 {
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

/// Get the root length of a path (always non-negative).
pub fn get_root_length(path: &str) -> usize {
    let root_length = get_encoded_root_length(path);
    if root_length < 0 {
        (!root_length) as usize
    } else {
        root_length as usize
    }
}

/// Get the directory path (parent directory).
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

/// Build a path from components.
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

/// Replace `\` with `/`.
pub fn normalize_slashes(path: &str) -> String {
    path.replace('\\', "/")
}

/// Reduce path components by resolving `.` and `..`.
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

/// Combine and resolve paths. Resolves `.` and `..` components.
pub fn resolve_path(path: &str, paths: &[&str]) -> String {
    let combined = if !paths.is_empty() {
        combine_paths(path, paths)
    } else {
        normalize_slashes(path)
    };
    normalize_path(&combined)
}

/// Return the normalized absolute path of `path` resolved against `current_directory`.
///
/// Mirrors `tspath.GetNormalizedAbsolutePath` in Go. Unlike `normalize_path`,
/// this strips trailing directory separators when the path length exceeds the
/// root length, and ensures a trailing separator when the path is exactly the root.
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

/// Normalize a path: normalize slashes and resolve `.` / `..`.
pub fn normalize_path(path: &str) -> String {
    let path = normalize_slashes(path);
    // Simple normalization: replace /./ with /, trim leading ./
    let simplified = path.replace("/./", "/");
    let simplified = simplified
        .strip_prefix("./")
        .unwrap_or(&simplified)
        .to_string();
    if !has_relative_path_segment(&simplified) {
        return simplified;
    }
    // Full normalization via components
    let components = get_normalized_path_components_from_combined(&simplified);
    let result = get_path_from_path_components(&components);
    if !result.is_empty() && has_trailing_directory_separator(&path) {
        ensure_trailing_directory_separator(&result)
    } else {
        result
    }
}

fn get_normalized_path_components_from_combined(path: &str) -> Vec<String> {
    let root_length = get_root_length(path);
    let mut components = vec![path[..root_length].to_string()];

    let bytes = path.as_bytes();
    let mut i = root_length;
    while i < bytes.len() {
        // Skip directory separators
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

fn has_relative_path_segment(p: &str) -> bool {
    let n = p.len();
    if n == 0 {
        return false;
    }
    if p == "." || p == ".." {
        return true;
    }
    let bytes = p.as_bytes();
    // Leading "./" or "../"
    if bytes[0] == b'.' {
        if n >= 2 && bytes[1] == b'/' {
            return true;
        }
        if n >= 3 && bytes[1] == b'.' && bytes[2] == b'/' {
            return true;
        }
    }
    // Trailing "/." or "/.."
    if bytes[n - 1] == b'.' {
        if n >= 2 && bytes[n - 2] == b'/' {
            return true;
        }
        if n >= 3 && bytes[n - 2] == b'.' && bytes[n - 3] == b'/' {
            return true;
        }
    }
    // Look for //, /./, /../
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

/// Get the canonical file name (lowercase on case-insensitive systems).
pub fn get_canonical_file_name(file_name: &str, use_case_sensitive_file_names: bool) -> String {
    if use_case_sensitive_file_names {
        file_name.to_string()
    } else {
        to_file_name_lower_case(file_name)
    }
}

/// Convert file name to lowercase, handling special characters.
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

/// Convert a file name to a canonical `Path`.
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

/// Remove a single trailing directory separator.
pub fn remove_trailing_directory_separator(path: &str) -> String {
    if has_trailing_directory_separator(path) {
        path[..path.len() - 1].to_string()
    } else {
        path.to_string()
    }
}

/// Remove all trailing directory separators.
pub fn remove_trailing_directory_separators(path: &str) -> String {
    let mut result = path.to_string();
    while has_trailing_directory_separator(&result) {
        result.pop();
    }
    result
}

/// Ensure a path ends with a directory separator.
pub fn ensure_trailing_directory_separator(path: &str) -> String {
    if has_trailing_directory_separator(path) {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

/// Get the base file name from a path.
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

/// Whether a path is relative (starts with `./` or `../`).
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

/// Ensure a path is either absolute or dot-relative (not confused with a module name).
pub fn ensure_path_is_non_module_name(path: &str) -> String {
    if !path_is_absolute(path) && !path_is_relative(path) {
        format!("./{}", path)
    } else {
        path.to_string()
    }
}

/// Whether an external module name is relative.
pub fn is_external_module_name_relative(module_name: &str) -> bool {
    path_is_relative(module_name) || is_rooted_disk_path(module_name)
}

/// Whether a path has an extension.
pub fn has_extension(file_name: &str) -> bool {
    get_base_file_name(file_name).contains('.')
}

/// Check if a path has a specific extension.
pub fn file_extension_is(path: &str, extension: &str) -> bool {
    path.len() > extension.len() && path.ends_with(extension)
}

/// Iterate over ancestor directories, calling a callback on each.
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

// ─────────────────────────────────────────────────────────────────────
// Extension constants and functions (from extension.go)
// ─────────────────────────────────────────────────────────────────────

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

/// Whether an extension is a TypeScript extension.
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

/// Remove a known file extension from a path.
pub fn remove_file_extension(path: &str) -> String {
    for ext in EXTENSIONS_TO_REMOVE {
        if path.ends_with(ext) {
            return path[..path.len() - ext.len()].to_string();
        }
    }
    path.to_string()
}

/// Try to get a known extension from a path.
pub fn try_get_extension_from_path(p: &str) -> &str {
    for ext in EXTENSIONS_TO_REMOVE {
        if file_extension_is(p, ext) {
            return ext;
        }
    }
    ""
}

/// Remove a specific extension from a path.
pub fn remove_extension(path: &str, extension: &str) -> String {
    path[..path.len() - extension.len()].to_string()
}

/// Whether a path has one of the given extensions.
pub fn file_extension_is_one_of(path: &str, extensions: &[&str]) -> bool {
    extensions.iter().any(|ext| file_extension_is(path, ext))
}

/// Whether a path has a TypeScript file extension.
pub fn has_ts_file_extension(path: &str) -> bool {
    file_extension_is_one_of(path, SUPPORTED_TS_EXTENSIONS_FLAT)
}

/// Whether a path has a JavaScript file extension.
pub fn has_js_file_extension(path: &str) -> bool {
    file_extension_is_one_of(path, SUPPORTED_JS_EXTENSIONS_FLAT)
}

/// Whether a path has a JSON file extension.
pub fn has_json_file_extension(path: &str) -> bool {
    file_extension_is(path, EXTENSION_JSON)
}

/// Whether a file name is a declaration file.
pub fn is_declaration_file_name(file_name: &str) -> bool {
    !get_declaration_file_extension(file_name).is_empty()
}

/// Get the declaration extension from a file name.
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

/// Change the extension of a path.
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

/// Get any extension from a path, optionally matching a list of extensions.
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

/// Contains ignored path patterns.
pub fn contains_ignored_path(path: &str) -> bool {
    let ignored_paths = ["/node_modules/.", "/.git", ".#"];
    ignored_paths.iter().any(|p| path.contains(p))
}

/// Options for comparing paths.
///
/// Mirrors `tspath.ComparePathsOptions` in Go.
#[derive(Debug, Clone, Default)]
pub struct ComparePathsOptions {
    pub use_case_sensitive_file_names: bool,
    pub current_directory: String,
}

impl ComparePathsOptions {
    fn equality_comparer(&self) -> impl Fn(&str, &str) -> bool {
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

/// Return the normalized absolute path of `fileName` without the root prefix.
///
/// Mirrors `tspath.GetNormalizedAbsolutePathWithoutRoot` in Go.
pub fn get_normalized_absolute_path_without_root(
    file_name: &str,
    current_directory: &str,
) -> String {
    let absolute_path = get_normalized_absolute_path(file_name, current_directory);
    let root_length = get_root_length(&absolute_path);
    absolute_path[root_length..].to_string()
}

/// Get path components relative from one path to another.
///
/// Mirrors `tspath.GetPathComponentsRelativeTo` in Go.
fn get_path_components_relative_to(
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
            // First component (root) is always compared case-insensitively.
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

/// Get the relative path from a directory to a file/URL.
///
/// Mirrors `tspath.GetRelativePathToDirectoryOrUrl` in Go.
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

/// Convert an absolute path to a path relative to `options.current_directory`.
/// If the path is already relative, it is returned as-is.
///
/// Mirrors `tspath.ConvertToRelativePath` in Go (path.go:785).
pub fn convert_to_relative_path(path: &str, options: &ComparePathsOptions) -> String {
    if !is_rooted_disk_path(path) {
        return path.to_string();
    }
    get_relative_path_to_directory_or_url(
        &options.current_directory,
        path,
        false, // is_absolute_path_an_url
        options,
    )
}

/// Find the common parent directories for a set of paths.
///
/// Mirrors `tspath.GetCommonParents` in Go. Returns `(parents, ignored)` where
/// `ignored` is a set of paths that had fewer than `min_components` components.
pub fn get_common_parents(
    paths: &[String],
    min_components: usize,
    options: &ComparePathsOptions,
) -> (Vec<String>, std::collections::HashSet<String>) {
    if min_components < 1 {
        panic!("minComponents must be at least 1");
    }
    if paths.is_empty() {
        return (vec![], std::collections::HashSet::new());
    }
    if paths.len() == 1 {
        let components =
            reduce_path_components(&get_path_components(&paths[0], &options.current_directory));
        if components.len() < min_components {
            let mut ignored = std::collections::HashSet::new();
            ignored.insert(paths[0].clone());
            return (vec![], ignored);
        }
        return (vec![paths[0].clone()], std::collections::HashSet::new());
    }

    let mut ignored = std::collections::HashSet::new();
    let mut path_components: Vec<Vec<String>> = Vec::new();
    for path in paths {
        let components =
            reduce_path_components(&get_path_components(path, &options.current_directory));
        if components.len() < min_components {
            ignored.insert(path.clone());
        } else {
            path_components.push(components);
        }
    }

    let results = get_common_parents_worker(&path_components, min_components, options);
    let result_paths: Vec<String> = results
        .iter()
        .map(|comps| get_path_from_path_components(comps))
        .collect();

    (result_paths, ignored)
}

/// Recursive worker for `get_common_parents`.
fn get_common_parents_worker(
    component_groups: &[Vec<String>],
    min_components: usize,
    options: &ComparePathsOptions,
) -> Vec<Vec<String>> {
    if component_groups.is_empty() {
        return vec![];
    }

    let max_depth = component_groups.iter().map(|g| g.len()).min().unwrap_or(0);
    let equality = options.equality_comparer();

    for last_common_index in 0..max_depth {
        let candidate = &component_groups[0][last_common_index];
        for j in 1..component_groups.len() {
            let comps = &component_groups[j];
            if !equality(candidate, &comps[last_common_index]) {
                if last_common_index < min_components {
                    // Not enough components — fan out by grouping on the divergent component.
                    let mut ordered_groups: Vec<String> = Vec::new();
                    let mut new_groups: std::collections::HashMap<
                        String,
                        (Vec<String>, Vec<Vec<String>>),
                    > = std::collections::HashMap::new();

                    for g in component_groups {
                        let key = to_path(
                            &g[last_common_index],
                            &options.current_directory,
                            options.use_case_sensitive_file_names,
                        )
                        .to_string();
                        if !new_groups.contains_key(&key) {
                            ordered_groups.push(key.clone());
                        }
                        let entry = new_groups.entry(key).or_insert_with(|| (vec![], vec![]));
                        if entry.0.is_empty() {
                            entry.0 = g[..=last_common_index].to_vec();
                        }
                        entry.1.push(g[last_common_index + 1..].to_vec());
                    }

                    ordered_groups.sort();
                    let mut result: Vec<Vec<String>> = vec![];
                    for key in &ordered_groups {
                        let group = &new_groups[key];
                        let sub_results = get_common_parents_worker(
                            &group.1,
                            min_components.saturating_sub(last_common_index + 1),
                            options,
                        );
                        for sr in &sub_results {
                            let mut combined = group.0.clone();
                            combined.extend(sr.iter().cloned());
                            result.push(combined);
                        }
                    }
                    return result;
                }
                return vec![component_groups[0][..last_common_index].to_vec()];
            }
        }
    }

    vec![component_groups[0][..max_depth].to_vec()]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── NormalizeSlashes ──

    #[test]
    fn test_normalize_slashes() {
        assert_eq!(normalize_slashes("a"), "a");
        assert_eq!(normalize_slashes("a/b"), "a/b");
        assert_eq!(normalize_slashes("a\\b"), "a/b");
        assert_eq!(normalize_slashes("\\\\server\\path"), "//server/path");
        assert_eq!(normalize_slashes("a\\b\\c"), "a/b/c");
    }

    // ── GetRootLength ──

    #[test]
    fn test_get_root_length() {
        assert_eq!(get_root_length("a"), 0);
        assert_eq!(get_root_length("/"), 1);
        assert_eq!(get_root_length("/path"), 1);
        assert_eq!(get_root_length("c:"), 2);
        assert_eq!(get_root_length("c:d"), 0);
        assert_eq!(get_root_length("c:/"), 3);
        assert_eq!(get_root_length("c:\\"), 3);
        assert_eq!(get_root_length("//server"), 8);
        assert_eq!(get_root_length("//server/share"), 9);
        assert_eq!(get_root_length("\\\\server"), 8);
        assert_eq!(get_root_length("\\\\server\\share"), 9);
        assert_eq!(get_root_length("file:///"), 8);
        assert_eq!(get_root_length("file:///path"), 8);
        assert_eq!(get_root_length("file:///c:"), 10);
        assert_eq!(get_root_length("file:///c:d"), 8);
        assert_eq!(get_root_length("file:///c:/path"), 11);
        assert_eq!(get_root_length("file://localhost"), 16);
        assert_eq!(get_root_length("file://localhost/"), 17);
        assert_eq!(get_root_length("file://localhost/path"), 17);
        assert_eq!(get_root_length("file://server"), 13);
        assert_eq!(get_root_length("file://server/"), 14);
        assert_eq!(get_root_length("file://server/path"), 14);
        assert_eq!(get_root_length("http://server"), 13);
        assert_eq!(get_root_length("http://server/path"), 14);
    }

    // ── PathIsAbsolute ──

    #[test]
    fn test_path_is_absolute() {
        assert!(path_is_absolute("/path/to/file.ext"));
        assert!(path_is_absolute("c:/path/to/file.ext"));
        assert!(path_is_absolute("file:///path/to/file.ext"));
        assert!(!path_is_absolute("path/to/file.ext"));
        assert!(!path_is_absolute("./path/to/file.ext"));
    }

    // ── IsUrl ──

    #[test]
    fn test_is_url() {
        assert!(!is_url("a"));
        assert!(!is_url("/"));
        assert!(!is_url("c:"));
        assert!(!is_url("c:d"));
        assert!(!is_url("c:/"));
        assert!(!is_url("c:\\"));
        assert!(!is_url("//server"));
        assert!(!is_url("//server/share"));
        assert!(!is_url("\\\\server"));
        assert!(!is_url("\\\\server\\share"));

        assert!(is_url("file:///path"));
        assert!(is_url("file:///c:"));
        assert!(is_url("file:///c:d"));
        assert!(is_url("file:///c:/path"));
        assert!(is_url("file://server"));
        assert!(is_url("file://server/path"));
        assert!(is_url("http://server"));
        assert!(is_url("http://server/path"));
    }

    // ── IsRootedDiskPath ──

    #[test]
    fn test_is_rooted_disk_path() {
        assert!(!is_rooted_disk_path("a"));
        assert!(is_rooted_disk_path("/"));
        assert!(is_rooted_disk_path("c:"));
        assert!(!is_rooted_disk_path("c:d"));
        assert!(is_rooted_disk_path("c:/"));
        assert!(is_rooted_disk_path("c:\\"));
        assert!(is_rooted_disk_path("//server"));
        assert!(is_rooted_disk_path("//server/share"));
        assert!(is_rooted_disk_path("\\\\server"));
        assert!(is_rooted_disk_path("\\\\server\\share"));
        assert!(!is_rooted_disk_path("file:///path"));
        assert!(!is_rooted_disk_path("file:///c:"));
        assert!(!is_rooted_disk_path("file://server"));
        assert!(!is_rooted_disk_path("http://server"));
    }

    // ── GetDirectoryPath ──

    #[test]
    fn test_get_directory_path() {
        assert_eq!(get_directory_path(""), "");
        assert_eq!(get_directory_path("a"), "");
        assert_eq!(get_directory_path("a/b"), "a");
        assert_eq!(get_directory_path("/"), "/");
        assert_eq!(get_directory_path("/a"), "/");
        assert_eq!(get_directory_path("/a/"), "/");
        assert_eq!(get_directory_path("/a/b"), "/a");
        assert_eq!(get_directory_path("/a/b/"), "/a");
        assert_eq!(get_directory_path("c:"), "c:");
        assert_eq!(get_directory_path("c:d"), "");
        assert_eq!(get_directory_path("c:/"), "c:/");
        assert_eq!(get_directory_path("c:/path"), "c:/");
        assert_eq!(get_directory_path("c:/path/"), "c:/");
        assert_eq!(get_directory_path("//server"), "//server");
        assert_eq!(get_directory_path("//server/"), "//server/");
        assert_eq!(get_directory_path("//server/share"), "//server/");
        assert_eq!(get_directory_path("//server/share/"), "//server/");
        assert_eq!(get_directory_path("\\\\server"), "//server");
        assert_eq!(get_directory_path("\\\\server\\"), "//server/");
        assert_eq!(get_directory_path("\\\\server\\share"), "//server/");
        assert_eq!(get_directory_path("file:///"), "file:///");
        assert_eq!(get_directory_path("file:///path"), "file:///");
        assert_eq!(get_directory_path("file:///c:"), "file:///c:");
        assert_eq!(get_directory_path("file:///c:d"), "file:///");
        assert_eq!(get_directory_path("file:///c:/"), "file:///c:/");
        assert_eq!(get_directory_path("file:///c:/path"), "file:///c:/");
        assert_eq!(get_directory_path("file://server"), "file://server");
        assert_eq!(get_directory_path("file://server/"), "file://server/");
        assert_eq!(get_directory_path("file://server/path"), "file://server/");
        assert_eq!(get_directory_path("http://server"), "http://server");
        assert_eq!(get_directory_path("http://server/"), "http://server/");
        assert_eq!(get_directory_path("http://server/path"), "http://server/");
    }

    // ── GetPathComponents ──

    #[test]
    fn test_get_path_components() {
        assert_eq!(get_path_components("", ""), vec![""]);
        assert_eq!(get_path_components("a", ""), vec!["", "a"]);
        assert_eq!(get_path_components("./a", ""), vec!["", ".", "a"]);
        assert_eq!(get_path_components("/", ""), vec!["/"]);
        assert_eq!(get_path_components("/a", ""), vec!["/", "a"]);
        assert_eq!(get_path_components("/a/", ""), vec!["/", "a"]);
        assert_eq!(get_path_components("c:", ""), vec!["c:"]);
        assert_eq!(get_path_components("c:d", ""), vec!["", "c:d"]);
        assert_eq!(get_path_components("c:/", ""), vec!["c:/"]);
        assert_eq!(get_path_components("c:/path", ""), vec!["c:/", "path"]);
        assert_eq!(get_path_components("//server", ""), vec!["//server"]);
        assert_eq!(get_path_components("//server/", ""), vec!["//server/"]);
        assert_eq!(
            get_path_components("//server/share", ""),
            vec!["//server/", "share"]
        );
        assert_eq!(get_path_components("file:///", ""), vec!["file:///"]);
        assert_eq!(
            get_path_components("file:///path", ""),
            vec!["file:///", "path"]
        );
        assert_eq!(get_path_components("file:///c:", ""), vec!["file:///c:"]);
        assert_eq!(
            get_path_components("file:///c:d", ""),
            vec!["file:///", "c:d"]
        );
        assert_eq!(get_path_components("file:///c:/", ""), vec!["file:///c:/"]);
        assert_eq!(
            get_path_components("file:///c:/path", ""),
            vec!["file:///c:/", "path"]
        );
        assert_eq!(
            get_path_components("file://server", ""),
            vec!["file://server"]
        );
        assert_eq!(
            get_path_components("file://server/", ""),
            vec!["file://server/"]
        );
        assert_eq!(
            get_path_components("file://server/path", ""),
            vec!["file://server/", "path"]
        );
        assert_eq!(
            get_path_components("http://server", ""),
            vec!["http://server"]
        );
        assert_eq!(
            get_path_components("http://server/", ""),
            vec!["http://server/"]
        );
        assert_eq!(
            get_path_components("http://server/path", ""),
            vec!["http://server/", "path"]
        );
    }

    // ── CombinePaths ──

    #[test]
    fn test_combine_paths() {
        // Non-rooted
        assert_eq!(
            combine_paths("path", &["to", "file.ext"]),
            "path/to/file.ext"
        );
        assert_eq!(
            combine_paths("path", &["dir", "..", "to", "file.ext"]),
            "path/dir/../to/file.ext"
        );
        // POSIX
        assert_eq!(
            combine_paths("/path", &["to", "file.ext"]),
            "/path/to/file.ext"
        );
        assert_eq!(combine_paths("/path", &["/to", "file.ext"]), "/to/file.ext");
        // DOS
        assert_eq!(
            combine_paths("c:/path", &["to", "file.ext"]),
            "c:/path/to/file.ext"
        );
        assert_eq!(
            combine_paths("c:/path", &["c:/to", "file.ext"]),
            "c:/to/file.ext"
        );
        // URL
        assert_eq!(
            combine_paths("file:///path", &["to", "file.ext"]),
            "file:///path/to/file.ext"
        );
        assert_eq!(
            combine_paths("file:///path", &["file:///to", "file.ext"]),
            "file:///to/file.ext"
        );

        assert_eq!(
            combine_paths("/", &["/node_modules/@types"]),
            "/node_modules/@types"
        );
        assert_eq!(combine_paths("/a/..", &[""]), "/a/..");
        assert_eq!(combine_paths("/a/..", &["b"]), "/a/../b");
        assert_eq!(combine_paths("/a/..", &["b/"]), "/a/../b/");
        assert_eq!(combine_paths("/a/..", &["/"]), "/");
        assert_eq!(combine_paths("/a/..", &["/b"]), "/b");
    }

    // ── ResolvePath ──

    #[test]
    fn test_resolve_path() {
        assert_eq!(resolve_path("", &[]), "");
        assert_eq!(resolve_path(".", &[]), "");
        assert_eq!(resolve_path("./", &[]), "");
        assert_eq!(resolve_path("..", &[]), "..");
        assert_eq!(resolve_path("../", &[]), "../");
        assert_eq!(resolve_path("/", &[]), "/");
        assert_eq!(resolve_path("/.", &[]), "/");
        assert_eq!(resolve_path("/./", &[]), "/");
        assert_eq!(resolve_path("/../", &[]), "/");
        assert_eq!(resolve_path("/a", &[]), "/a");
        assert_eq!(resolve_path("/a/", &[]), "/a/");
        assert_eq!(resolve_path("/a/.", &[]), "/a");
        assert_eq!(resolve_path("/a/./", &[]), "/a/");
        assert_eq!(resolve_path("/a/./b", &[]), "/a/b");
        assert_eq!(resolve_path("/a/./b/", &[]), "/a/b/");
        assert_eq!(resolve_path("/a/..", &[]), "/");
        assert_eq!(resolve_path("/a/../", &[]), "/");
        assert_eq!(resolve_path("/a/../b", &[]), "/b");
        assert_eq!(resolve_path("/a/../b/", &[]), "/b/");
        assert_eq!(resolve_path("/a/..", &["b"]), "/b");
        assert_eq!(resolve_path("/a/..", &["/"]), "/");
        assert_eq!(resolve_path("/a/..", &["b/"]), "/b/");
        assert_eq!(resolve_path("/a/..", &["/b"]), "/b");
        assert_eq!(resolve_path("/a/.", &["b"]), "/a/b");
        assert_eq!(resolve_path("/a/.", &["."]), "/a");
        assert_eq!(resolve_path("a", &["b", "c"]), "a/b/c");
        assert_eq!(resolve_path("a", &["b", "/c"]), "/c");
        assert_eq!(resolve_path("a", &["b", "../c"]), "a/c");
    }

    // ── GetNormalizedAbsolutePath ──

    #[test]
    fn test_get_normalized_absolute_path() {
        // Absolute paths (ported from Go path_test.go TestGetNormalizedAbsolutePath)
        assert_eq!(get_normalized_absolute_path("/", ""), "/");
        assert_eq!(get_normalized_absolute_path("/.", ""), "/");
        assert_eq!(get_normalized_absolute_path("/./", ""), "/");
        assert_eq!(get_normalized_absolute_path("/../", ""), "/");
        assert_eq!(get_normalized_absolute_path("/a", ""), "/a");
        assert_eq!(get_normalized_absolute_path("/a/", ""), "/a");
        assert_eq!(get_normalized_absolute_path("/a/.", ""), "/a");
        assert_eq!(get_normalized_absolute_path("/a/foo.", ""), "/a/foo.");
        assert_eq!(get_normalized_absolute_path("/a/./", ""), "/a");
        assert_eq!(get_normalized_absolute_path("/a/./b", ""), "/a/b");
        assert_eq!(get_normalized_absolute_path("/a/./b/", ""), "/a/b");
        assert_eq!(get_normalized_absolute_path("/a/..", ""), "/");
        assert_eq!(get_normalized_absolute_path("/a/../", ""), "/");
        assert_eq!(get_normalized_absolute_path("/a/../b", ""), "/b");
        assert_eq!(get_normalized_absolute_path("/a/../b/", ""), "/b");
        assert_eq!(get_normalized_absolute_path("/a/..", "/"), "/");
        assert_eq!(get_normalized_absolute_path("/a/..", "b/"), "/");
        assert_eq!(get_normalized_absolute_path("/a/..", "/b"), "/");
        assert_eq!(get_normalized_absolute_path("/a/.", "b"), "/a");
        assert_eq!(get_normalized_absolute_path("/a/.", "."), "/a");

        // Backslash normalization
        assert_eq!(get_normalized_absolute_path("\\", ""), "/");
        assert_eq!(get_normalized_absolute_path("\\.", ""), "/");
        assert_eq!(get_normalized_absolute_path("\\.\\", ""), "/");
        assert_eq!(get_normalized_absolute_path("\\..\\", ""), "/");
        assert_eq!(get_normalized_absolute_path("\\a\\.\\", ""), "/a");
        assert_eq!(get_normalized_absolute_path("\\a\\.\\b", ""), "/a/b");
        assert_eq!(get_normalized_absolute_path("\\a\\.\\b\\", ""), "/a/b");
        assert_eq!(get_normalized_absolute_path("\\a\\..", ""), "/");
        assert_eq!(get_normalized_absolute_path("\\a\\..\\", ""), "/");
        assert_eq!(get_normalized_absolute_path("\\a\\..\\b", ""), "/b");
        assert_eq!(get_normalized_absolute_path("\\a\\..\\b\\", ""), "/b");
        assert_eq!(get_normalized_absolute_path("\\a\\..", "\\"), "/");
        assert_eq!(get_normalized_absolute_path("\\a\\..", "b\\"), "/");
        assert_eq!(get_normalized_absolute_path("\\a\\..", "\\b"), "/");
        assert_eq!(get_normalized_absolute_path("\\a\\.", "b"), "/a");
        assert_eq!(get_normalized_absolute_path("\\a\\.", "."), "/a");

        // Relative paths with empty current_directory
        assert_eq!(get_normalized_absolute_path("", ""), "");
        assert_eq!(get_normalized_absolute_path(".", ""), "");
        assert_eq!(get_normalized_absolute_path("./", ""), "");
        assert_eq!(get_normalized_absolute_path("..", ""), "..");
        assert_eq!(get_normalized_absolute_path("../", ""), "..");

        // Relative paths with current_directory
        assert_eq!(get_normalized_absolute_path("", "/home"), "/home");
        assert_eq!(get_normalized_absolute_path(".", "/home"), "/home");
        assert_eq!(get_normalized_absolute_path("./", "/home"), "/home");
        assert_eq!(get_normalized_absolute_path("..", "/home"), "/");
        assert_eq!(get_normalized_absolute_path("../", "/home"), "/");
        assert_eq!(get_normalized_absolute_path("a", "b"), "b/a");
        assert_eq!(get_normalized_absolute_path("a", "b/c"), "b/c/a");

        // Dot-prefixed names (not . or ..)
        assert_eq!(get_normalized_absolute_path(".a", ""), ".a");
        assert_eq!(get_normalized_absolute_path("..a", ""), "..a");
        assert_eq!(get_normalized_absolute_path("a.", ""), "a.");
        assert_eq!(get_normalized_absolute_path("a..", ""), "a..");

        // Dot-prefixed names with paths
        assert_eq!(get_normalized_absolute_path("/base/./.a", ""), "/base/.a");
        assert_eq!(get_normalized_absolute_path("/base/../.a", ""), "/.a");
        assert_eq!(get_normalized_absolute_path("/base/./..a", ""), "/base/..a");
        assert_eq!(get_normalized_absolute_path("/base/../..a", ""), "/..a");
        assert_eq!(
            get_normalized_absolute_path("/base/./..a/b", ""),
            "/base/..a/b"
        );
        assert_eq!(get_normalized_absolute_path("/base/../..a/b", ""), "/..a/b");
        assert_eq!(get_normalized_absolute_path("/base/./a.", ""), "/base/a.");
        assert_eq!(get_normalized_absolute_path("/base/../a.", ""), "/a.");
        assert_eq!(get_normalized_absolute_path("/base/./a..", ""), "/base/a..");
        assert_eq!(get_normalized_absolute_path("/base/../a..", ""), "/a..");
        assert_eq!(
            get_normalized_absolute_path("/base/./a../b", ""),
            "/base/a../b"
        );
        assert_eq!(get_normalized_absolute_path("/base/../a../b", ""), "/a../b");

        // Edge cases
        assert_eq!(get_normalized_absolute_path("a/..", ""), "");
        assert_eq!(get_normalized_absolute_path("/a//", ""), "/a");
        assert_eq!(get_normalized_absolute_path("a/..", ""), "");

        // Consecutive slashes
        assert_eq!(get_normalized_absolute_path("a//b", ""), "a/b");
        assert_eq!(get_normalized_absolute_path("a///b", ""), "a/b");
        assert_eq!(get_normalized_absolute_path("a/b//c", ""), "a/b/c");
        assert_eq!(get_normalized_absolute_path("/a/b//c", ""), "/a/b/c");

        // Consecutive backslashes
        assert_eq!(get_normalized_absolute_path("a\\\\b", ""), "a/b");
        assert_eq!(get_normalized_absolute_path("a\\\\\\b", ""), "a/b");
        assert_eq!(get_normalized_absolute_path("a\\b\\\\c", ""), "a/b/c");
        assert_eq!(get_normalized_absolute_path("\\a\\b\\\\c", ""), "/a/b/c");
    }

    // ── ToFileNameLowerCase ──

    #[test]
    fn test_to_file_name_lower_case() {
        assert_eq!(
            to_file_name_lower_case("/user/UserName/projects/Project/file.ts"),
            "/user/username/projects/project/file.ts"
        );
        assert_eq!(
            to_file_name_lower_case("/user/UserName/projects/projectß/file.ts"),
            "/user/username/projects/projectß/file.ts"
        );
    }

    // ── ToPath ──

    #[test]
    fn test_to_path() {
        assert_eq!(
            to_path("file.ext", "path/to", false).as_str(),
            "path/to/file.ext"
        );
        assert_eq!(
            to_path("file.ext", "/path/to", true).as_str(),
            "/path/to/file.ext"
        );
        assert_eq!(
            to_path("/path/to/../file.ext", "path/to", true).as_str(),
            "/path/file.ext"
        );
    }

    // ── PathIsRelative ──

    #[test]
    fn test_path_is_relative() {
        assert!(path_is_relative("."));
        assert!(path_is_relative(".."));
        assert!(path_is_relative("./"));
        assert!(path_is_relative("../"));
        assert!(path_is_relative("./foo/bar"));
        assert!(path_is_relative("../foo/bar"));
        assert!(!path_is_relative(""));
        assert!(!path_is_relative("foo"));
        assert!(!path_is_relative("foo/bar"));
        assert!(!path_is_relative("/foo/bar"));
        assert!(!path_is_relative("c:/foo/bar"));
    }

    // ── IsDynamicFileName / Untitled paths ──

    #[test]
    fn test_is_dynamic_file_name() {
        assert!(is_dynamic_file_name("^/untitled/foo.ts"));
        assert!(!is_dynamic_file_name("/path/to/file.ts"));
        assert!(!is_dynamic_file_name(""));
    }

    #[test]
    fn test_untitled_path_root_length() {
        // "^/" has root length 2
        assert_eq!(get_encoded_root_length("^/untitled"), 2);
        assert_eq!(get_root_length("^/untitled"), 2);
        // "^" alone is not a dynamic file name root
        assert_ne!(get_encoded_root_length("^"), 2);
    }

    // ── ContainsIgnoredPath (ported 1:1 from Go TestContainsIgnoredPath) ──

    #[test]
    fn test_contains_ignored_path() {
        let tests: &[(&str, &str, bool)] = &[
            // (name, path, expected)
            (
                "node_modules dot path",
                "/project/node_modules/.pnpm/file.ts",
                true,
            ),
            ("git directory", "/project/.git/hooks/pre-commit", true),
            ("emacs lock file", "/project/src/file.ts.#", true),
            ("regular file path", "/project/src/file.ts", false),
            (
                "node_modules without dot",
                "/project/node_modules/lodash/index.js",
                false,
            ),
            ("empty path", "", false),
            (
                "path with multiple ignored patterns",
                "/project/node_modules/.pnpm/.git/.#file.ts",
                true,
            ),
            (
                "case sensitive test",
                "/project/NODE_MODULES/.PNPM/file.ts",
                false, // Should be case sensitive
            ),
            (
                "path with ignored pattern in middle",
                "/project/src/node_modules/.pnpm/dist/file.js",
                true,
            ),
            (
                "path with ignored pattern at end",
                "/project/src/file.ts.#",
                true,
            ),
        ];

        for &(name, path, expected) in tests {
            let result = contains_ignored_path(path);
            assert_eq!(
                result, expected,
                "ContainsIgnoredPath({:?}) = {}, expected {} ({})",
                path, result, expected, name
            );
        }
    }

    // ── IgnoredPaths patterns (ported 1:1 from Go TestIgnoredPathsPatterns) ──

    #[test]
    fn test_ignored_paths_patterns() {
        // Test that all expected patterns are present
        let expected_patterns = ["/node_modules/.", "/.git", ".#"];

        for pattern in expected_patterns {
            let test_path = format!("/test{}/file.ts", pattern);
            assert!(
                contains_ignored_path(&test_path),
                "Expected pattern '{}' to be detected in path '{}'",
                pattern,
                test_path
            );
        }
    }

    // ── IgnoredPaths edge cases (ported 1:1 from Go TestIgnoredPathsEdgeCases) ──

    #[test]
    fn test_ignored_paths_edge_cases() {
        let tests: &[(&str, &str, bool)] = &[
            // (name, path, expected)
            (
                "pattern at start",
                "/node_modules./file.ts",
                false, // Pattern is "/node_modules/." not "/node_modules."
            ),
            ("pattern at end", "/project/file.ts.#", true),
            (
                "multiple occurrences",
                "/project/.git/node_modules./.git/file.ts",
                true,
            ),
            ("no slashes", "node_modules.file.ts", false),
            ("single slash", "/file.ts", false),
        ];

        for &(name, path, expected) in tests {
            let result = contains_ignored_path(path);
            assert_eq!(
                result, expected,
                "ContainsIgnoredPath({:?}) = {}, expected {} ({})",
                path, result, expected, name
            );
        }
    }

    // ── BaseFileName / Extension functions ──

    #[test]
    fn test_get_base_file_name() {
        assert_eq!(get_base_file_name("/path/to/file.ext"), "file.ext");
        assert_eq!(get_base_file_name("/path/to/"), "to");
        assert_eq!(get_base_file_name("/"), "");
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/path/./to/../file.ext"), "/path/file.ext");
        assert_eq!(normalize_path("./file.ext"), "file.ext");
        assert_eq!(normalize_path("path/to/file.ext"), "path/to/file.ext");
    }

    #[test]
    fn test_extension_functions() {
        assert!(has_ts_file_extension("file.ts"));
        assert!(has_ts_file_extension("file.tsx"));
        assert!(has_ts_file_extension("file.d.ts"));
        assert!(!has_ts_file_extension("file.js"));
        assert!(has_js_file_extension("file.js"));
        assert!(is_declaration_file_name("file.d.ts"));
        assert!(!is_declaration_file_name("file.ts"));
        assert_eq!(remove_file_extension("/path/to/file.ts"), "/path/to/file");
        assert_eq!(remove_file_extension("/path/to/file.d.ts"), "/path/to/file");
        assert_eq!(change_extension("file.ts", ".js"), "file.js");
    }

    // ── Trailing separator functions ──

    #[test]
    fn test_trailing_directory_separator() {
        assert!(has_trailing_directory_separator("path/"));
        assert!(has_trailing_directory_separator("path\\"));
        assert!(!has_trailing_directory_separator("path"));
        assert_eq!(ensure_trailing_directory_separator("path"), "path/");
        assert_eq!(ensure_trailing_directory_separator("path/"), "path/");
        assert_eq!(remove_trailing_directory_separator("path/"), "path");
        assert_eq!(remove_trailing_directory_separator("path"), "path");
    }

    // ── ForEachAncestorDirectory ──

    #[test]
    fn test_for_each_ancestor_directory() {
        let mut ancestors = Vec::new();
        for_each_ancestor_directory("/a/b/c", |dir| {
            ancestors.push(dir.to_string());
            false
        });
        assert_eq!(ancestors, vec!["/a/b/c", "/a/b", "/a", "/"]);

        // Stop early
        let mut ancestors = Vec::new();
        for_each_ancestor_directory("/a/b/c", |dir| {
            ancestors.push(dir.to_string());
            dir == "/a/b"
        });
        assert_eq!(ancestors, vec!["/a/b/c", "/a/b"]);
    }

    // ── ReducePathComponents ──

    #[test]
    fn test_reduce_path_components() {
        assert_eq!(reduce_path_components(&vec!["".to_string()]), vec![""]);
        assert_eq!(
            reduce_path_components(&vec!["".to_string(), ".".to_string()]),
            vec![""]
        );
        assert_eq!(
            reduce_path_components(&vec!["".to_string(), ".".to_string(), "a".to_string()]),
            vec!["", "a"]
        );
        assert_eq!(
            reduce_path_components(&vec!["".to_string(), "a".to_string(), ".".to_string()]),
            vec!["", "a"]
        );
        assert_eq!(
            reduce_path_components(&vec!["".to_string(), "..".to_string()]),
            vec!["", ".."]
        );
        assert_eq!(
            reduce_path_components(&vec!["".to_string(), "..".to_string(), "..".to_string()]),
            vec!["", "..", ".."]
        );
        assert_eq!(
            reduce_path_components(&vec![
                "".to_string(),
                "..".to_string(),
                ".".to_string(),
                "..".to_string()
            ]),
            vec!["", "..", ".."]
        );
        assert_eq!(
            reduce_path_components(&vec!["".to_string(), "a".to_string(), "..".to_string()]),
            vec![""]
        );
        assert_eq!(
            reduce_path_components(&vec!["".to_string(), "..".to_string(), "a".to_string()]),
            vec!["", "..", "a"]
        );
        assert_eq!(reduce_path_components(&vec!["/".to_string()]), vec!["/"]);
        assert_eq!(
            reduce_path_components(&vec!["/".to_string(), ".".to_string()]),
            vec!["/"]
        );
        assert_eq!(
            reduce_path_components(&vec!["/".to_string(), "..".to_string()]),
            vec!["/"]
        );
        assert_eq!(
            reduce_path_components(&vec!["/".to_string(), "a".to_string(), "..".to_string()]),
            vec!["/"]
        );
    }

    // ── GetNormalizedAbsolutePathWithoutRoot ──

    #[test]
    fn test_get_normalized_absolute_path_without_root() {
        assert_eq!(
            get_normalized_absolute_path_without_root("/a/b/c.txt", "/a/b"),
            "a/b/c.txt"
        );
        assert_eq!(
            get_normalized_absolute_path_without_root("c:/work/hello.txt", "c:/work"),
            "work/hello.txt"
        );
        assert_eq!(
            get_normalized_absolute_path_without_root("c:/work/hello.txt", "d:/worspaces"),
            "work/hello.txt"
        );
    }

    // ── GetRelativePathToDirectoryOrUrl ──

    #[test]
    fn test_get_relative_path_to_directory_or_url() {
        let opts = ComparePathsOptions::default();

        assert_eq!(
            get_relative_path_to_directory_or_url("/", "/", false, &opts),
            ""
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("/a", "/a", false, &opts),
            ""
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("/a/", "/a", false, &opts),
            ""
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("/a", "/", false, &opts),
            ".."
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("/a", "/b", false, &opts),
            "../b"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("/a/b", "/b", false, &opts),
            "../../b"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("/a/b/c", "/b", false, &opts),
            "../../../b"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("/a/b/c", "/b/c", false, &opts),
            "../../../b/c"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("/a/b/c", "/a/b", false, &opts),
            ".."
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("c:", "d:", false, &opts),
            "d:/"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///", "file:///", false, &opts),
            ""
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///a", "file:///a", false, &opts),
            ""
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///a/", "file:///a", false, &opts),
            ""
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///a", "file:///", false, &opts),
            ".."
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///a", "file:///b", false, &opts),
            "../b"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///a/b", "file:///b", false, &opts),
            "../../b"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///a/b/c", "file:///b", false, &opts),
            "../../../b"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///a/b/c", "file:///b/c", false, &opts),
            "../../../b/c"
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///a/b/c", "file:///a/b", false, &opts),
            ".."
        );
        assert_eq!(
            get_relative_path_to_directory_or_url("file:///c:", "file:///d:", false, &opts),
            "file:///d:/"
        );
    }

    // ── GetCommonParents ──

    #[test]
    fn test_get_common_parents() {
        let opts = ComparePathsOptions::default();

        // empty input
        let (got, ignored) = get_common_parents(&[], 1, &opts);
        assert!(ignored.is_empty());
        assert!(got.is_empty());

        // single path returns itself
        let paths = vec!["/a/b/c/d".to_string()];
        let (got, ignored) = get_common_parents(&paths, 1, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["/a/b/c/d"]);

        // paths shorter than minComponents are ignored
        let paths = vec![
            "/a/b/c/d".to_string(),
            "/a/b/c/e".to_string(),
            "/a/b/f/g".to_string(),
            "/x/y".to_string(),
        ];
        let (got, ignored) = get_common_parents(&paths, 4, &opts);
        assert_eq!(ignored.len(), 1);
        assert!(ignored.contains("/x/y"));
        assert_eq!(got, vec!["/a/b/c", "/a/b/f/g"]);

        // three paths share /a/b
        let paths = vec![
            "/a/b/c/d".to_string(),
            "/a/b/c/e".to_string(),
            "/a/b/f/g".to_string(),
        ];
        let (got, ignored) = get_common_parents(&paths, 1, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["/a/b"]);

        // mixed with short path collapses to root when minComponents=1
        let paths = vec![
            "/a/b/c/d".to_string(),
            "/a/b/c/e".to_string(),
            "/a/b/f/g".to_string(),
            "/x/y/z".to_string(),
        ];
        let (got, ignored) = get_common_parents(&paths, 1, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["/"]);

        // mixed with short path preserves both when minComponents=3
        let paths = vec![
            "/a/b/c/d".to_string(),
            "/a/b/c/e".to_string(),
            "/a/b/f/g".to_string(),
            "/x/y/z".to_string(),
        ];
        let (got, ignored) = get_common_parents(&paths, 3, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["/a/b", "/x/y/z"]);

        // different volumes are returned individually
        let paths = vec!["c:/a/b/c/d".to_string(), "d:/a/b/c/d".to_string()];
        let (got, ignored) = get_common_parents(&paths, 1, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["c:/a/b/c/d", "d:/a/b/c/d"]);

        // duplicate paths deduplicate result
        let paths = vec!["/a/b/c/d".to_string(), "/a/b/c/d".to_string()];
        let (got, ignored) = get_common_parents(&paths, 1, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["/a/b/c/d"]);

        // paths with few components are returned as-is when minComponents met
        let paths = vec!["/a/b/c/d".to_string(), "/x/y".to_string()];
        let (got, ignored) = get_common_parents(&paths, 2, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["/a/b/c/d", "/x/y"]);

        // minComponents=2
        let paths = vec![
            "/a/b/c/d".to_string(),
            "/a/z/c/e".to_string(),
            "/a/aaa/f/g".to_string(),
            "/x/y/z".to_string(),
        ];
        let (got, ignored) = get_common_parents(&paths, 2, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["/a", "/x/y/z"]);

        // trailing separators are handled
        let paths = vec!["/a/b/".to_string(), "/a/b/c".to_string()];
        let (got, ignored) = get_common_parents(&paths, 1, &opts);
        assert!(ignored.is_empty());
        assert_eq!(got, vec!["/a/b"]);
    }

    // ── Untitled path handling (ported 1:1 from Go TestUntitledPathHandling) ──

    #[test]
    fn test_untitled_path_handling() {
        // Test that untitled paths are treated as rooted
        let untitled_path = "^/untitled/ts-nul-authority/Untitled-2";

        // GetEncodedRootLength should return 2 for "^/"
        let root_length = get_encoded_root_length(untitled_path);
        assert_eq!(
            root_length, 2,
            "GetEncodedRootLength should return 2 for untitled paths"
        );

        // IsRootedDiskPath should return true
        let is_rooted = is_rooted_disk_path(untitled_path);
        assert!(
            is_rooted,
            "IsRootedDiskPath should return true for untitled paths"
        );

        // ToPath should not resolve untitled paths against current directory
        let current_dir = "/home/user/project";
        let path = to_path(untitled_path, current_dir, true);
        // The path should be the original untitled path
        assert_eq!(
            path.as_str(),
            "^/untitled/ts-nul-authority/Untitled-2",
            "ToPath should not resolve untitled paths against current directory"
        );

        // Test GetNormalizedAbsolutePath doesn't resolve untitled paths
        let normalized = get_normalized_absolute_path(untitled_path, current_dir);
        assert_eq!(
            normalized, "^/untitled/ts-nul-authority/Untitled-2",
            "GetNormalizedAbsolutePath should not resolve untitled paths"
        );
    }

    // ── Untitled path edge cases (ported 1:1 from Go TestUntitledPathEdgeCases) ──

    #[test]
    fn test_untitled_path_edge_cases() {
        // (path, expected root length, is rooted)
        let test_cases: &[(&str, i32, bool)] = &[
            ("^/", 2, true),                               // Minimal untitled path
            ("^/untitled/ts-nul-authority/test", 2, true), // Normal untitled path
            ("^", 0, false),                               // Just ^ is not rooted
            ("^x", 0, false),                              // ^x is not untitled
            ("^^/", 0, false),                             // ^^/ is not untitled
            ("x^/", 0, false), // x^/ is not untitled (doesn't start with ^)
            (
                "^/untitled/ts-nul-authority/path/with/deeper/structure",
                2,
                true,
            ), // Deeper path
        ];

        for &(path, expected, is_rooted) in test_cases {
            let root_length = get_encoded_root_length(path);
            assert_eq!(
                root_length, expected,
                "GetEncodedRootLength for path {}",
                path
            );

            let result = is_rooted_disk_path(path);
            assert_eq!(result, is_rooted, "IsRootedDiskPath for path {}", path);
        }
    }

    // ── StartsWithDirectory (ported 1:1 from Go TestStartsWithDirectory) ──
    //
    // TODO: `starts_with_directory` is not yet implemented in Rust. The test
    // data below is a 1:1 port of the Go `TestStartsWithDirectory`. Enable this
    // test once `starts_with_directory(file_name, directory_name,
    // use_case_sensitive)` is ported from `internal/tspath/path.go`
    // (StartsWithDirectory).
    #[test]
    #[ignore = "starts_with_directory not yet implemented in Rust"]
    #[allow(unused)]
    fn test_starts_with_directory() {
        let tests: &[(&str, &str, &str, bool, bool)] = &[
            // (name, file_name, directory_name, use_case_sensitive, expected)
            (
                "exact match case sensitive",
                "/project/src/file.ts",
                "/project/src",
                true,
                true,
            ),
            (
                "exact match case insensitive",
                "/project/src/file.ts",
                "/PROJECT/SRC",
                false,
                true,
            ),
            (
                "case sensitive mismatch",
                "/project/src/file.ts",
                "/PROJECT/SRC",
                true,
                false,
            ),
            (
                "file not in directory",
                "/project/lib/file.ts",
                "/project/src",
                true,
                false,
            ),
            (
                "file in subdirectory",
                "/project/src/components/Button.tsx",
                "/project/src",
                true,
                true,
            ),
            (
                "file in parent directory",
                "/project/file.ts",
                "/project/src",
                true,
                false,
            ),
            (
                "windows style separators",
                "C:\\project\\src\\file.ts",
                "C:\\project\\src",
                true,
                true,
            ),
            (
                "mixed separators",
                "/project/src/file.ts",
                "\\project\\src",
                true,
                false,
            ),
            (
                "empty directory name",
                "/project/src/file.ts",
                "",
                true,
                false,
            ),
            ("empty file name", "", "/project/src", true, false),
            (
                "identical paths",
                "/project/src",
                "/project/src",
                true,
                false, // File name doesn't start with directory + separator
            ),
            (
                "directory with trailing separator",
                "/project/src/file.ts",
                "/project/src/",
                true,
                true,
            ),
            (
                "unicode characters",
                "/project/测试/file.ts",
                "/project/测试",
                true,
                true,
            ),
            (
                "unicode case insensitive",
                "/project/测试/file.ts",
                "/PROJECT/测试",
                false,
                true,
            ),
        ];

        for &(name, file_name, directory_name, use_case_sensitive, expected) in tests {
            // let result = starts_with_directory(file_name, directory_name, use_case_sensitive);
            // assert_eq!(
            //     result, expected,
            //     "StartsWithDirectory({:?}, {:?}, {}) = {}, expected {} ({})",
            //     file_name, directory_name, use_case_sensitive, result, expected, name
            // );
            let _ = (
                name,
                file_name,
                directory_name,
                use_case_sensitive,
                expected,
            );
        }
    }

    // ── StartsWithDirectory edge cases (ported 1:1 from Go TestStartsWithDirectoryEdgeCases) ──
    //
    // TODO: `starts_with_directory` is not yet implemented in Rust. See
    // `test_starts_with_directory` above.
    #[test]
    #[ignore = "starts_with_directory not yet implemented in Rust"]
    #[allow(unused)]
    fn test_starts_with_directory_edge_cases() {
        let tests: &[(&str, &str, &str, bool, bool)] = &[
            // (name, file_name, directory_name, use_case_sensitive, expected)
            (
                "file name shorter than directory",
                "/proj",
                "/project",
                true,
                false,
            ),
            (
                "file name starts with directory but no separator",
                "/projectsrc/file.ts",
                "/project",
                true,
                false,
            ),
            ("relative paths", "src/file.ts", "src", true, true),
            (
                "absolute vs relative",
                "/project/src/file.ts",
                "project/src",
                true,
                false,
            ),
        ];

        for &(name, file_name, directory_name, use_case_sensitive, expected) in tests {
            // let result = starts_with_directory(file_name, directory_name, use_case_sensitive);
            // assert_eq!(
            //     result, expected,
            //     "StartsWithDirectory({:?}, {:?}, {}) = {}, expected {} ({})",
            //     file_name, directory_name, use_case_sensitive, result, expected, name
            // );
            let _ = (
                name,
                file_name,
                directory_name,
                use_case_sensitive,
                expected,
            );
        }
    }
}
