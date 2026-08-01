//! Module resolution, ported from `internal/module/`.
//!
//! This module provides the core types and utility functions for module
//! resolution, plus the `Resolver` struct and resolution infrastructure.

pub mod resolver;

// Re-export key types from the resolver submodule.
pub use resolver::{
    get_effective_type_roots, DiagAndArgs, Extensions as ExtensionsBitfield, ResolutionHost,
    Resolver,
};

use crate::tspath;
use bitflags::bitflags;

/// The result of resolving a module.
#[derive(Clone, Debug, Default)]
pub struct ResolvedModule {
    pub resolved_file_name: String,
    pub original_path: String,
    pub extension: String,
    pub resolved_using_ts_extension: bool,
    pub package_id: Option<PackageId>,
    pub is_external_library_import: bool,
    pub alternate_result: Option<String>,
}

impl ResolvedModule {
    pub fn is_resolved(&self) -> bool {
        !self.resolved_file_name.is_empty()
    }
}

/// The result of resolving a type reference directive.
#[derive(Clone, Debug, Default)]
pub struct ResolvedTypeReferenceDirective {
    pub primary: bool,
    pub resolved_file_name: String,
    pub original_path: String,
    pub package_id: Option<PackageId>,
    pub is_external_library_import: bool,
}

impl ResolvedTypeReferenceDirective {
    pub fn is_resolved(&self) -> bool {
        !self.resolved_file_name.is_empty()
    }
}

/// Identifies a package within node_modules.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub name: String,
    pub sub_module_name: String,
    pub version: String,
    pub peer_dependencies: String,
}

impl PackageId {
    pub fn package_name(&self) -> String {
        if self.sub_module_name.is_empty() {
            self.name.clone()
        } else {
            format!("{}/{}", self.name, self.sub_module_name)
        }
    }
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}{}",
            self.package_name(),
            self.version,
            self.peer_dependencies
        )
    }
}

bitflags! {
    /// Node resolution features for module resolution.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct NodeResolutionFeatures: i32 {
        const Imports = 1;
        const SelfName = 1 << 1;
        const Exports = 1 << 2;
        const ExportsPatternTrailers = 1 << 3;
        const ImportsPatternRoot = 1 << 4;
    }
}

impl NodeResolutionFeatures {
    pub const NONE: NodeResolutionFeatures = NodeResolutionFeatures::empty();
    pub const ALL: NodeResolutionFeatures = NodeResolutionFeatures::Imports
        .union(Self::SelfName)
        .union(Self::Exports)
        .union(Self::ExportsPatternTrailers)
        .union(Self::ImportsPatternRoot);
    pub const NODE16_DEFAULT: NodeResolutionFeatures = NodeResolutionFeatures::Imports
        .union(Self::SelfName)
        .union(Self::Exports)
        .union(Self::ExportsPatternTrailers);
    pub const NODE_NEXT_DEFAULT: NodeResolutionFeatures = NodeResolutionFeatures::ALL;
    pub const BUNDLER_DEFAULT: NodeResolutionFeatures = NodeResolutionFeatures::Imports
        .union(Self::SelfName)
        .union(Self::Exports)
        .union(Self::ExportsPatternTrailers)
        .union(Self::ImportsPatternRoot);
}

/// Extension types for module resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Extensions {
    TypeScript,
    JavaScript,
    Declaration,
    Json,
    ImplementationFiles,
}

impl Extensions {
    /// Get the file extensions for this extension set.
    pub fn array(&self) -> Vec<&'static str> {
        match self {
            Extensions::TypeScript => tspath::SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS.to_vec(),
            Extensions::JavaScript => tspath::SUPPORTED_JS_EXTENSIONS_FLAT.to_vec(),
            Extensions::Declaration => tspath::SUPPORTED_DECLARATION_EXTENSIONS.to_vec(),
            Extensions::Json => vec![tspath::EXTENSION_JSON],
            Extensions::ImplementationFiles => {
                let mut result = tspath::SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS.to_vec();
                result.extend_from_slice(&tspath::SUPPORTED_JS_EXTENSIONS_FLAT);
                result
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Utility functions
// ─────────────────────────────────────────────────────────────────────

/// The inferred types containing file name.
pub const INFERRED_TYPES_CONTAINING_FILE: &str = "__inferred type names__.ts";

/// Parse a module name into (package_name, rest).
///
/// For scoped packages like `@scope/name/sub`, returns (`@scope/name`, `sub`).
/// For regular packages like `foo/bar`, returns (`foo`, `bar`).
pub fn parse_package_name(module_name: &str) -> (String, String) {
    let mut idx = module_name.find('/');
    if !module_name.is_empty() && module_name.starts_with('@') {
        if let Some(slash_idx) = idx {
            let offset = slash_idx + 1;
            idx = module_name[offset..].find('/').map(|i| i + offset);
        }
    }
    match idx {
        Some(i) => (
            module_name[..i].to_string(),
            module_name[i + 1..].to_string(),
        ),
        None => (module_name.to_string(), String::new()),
    }
}

/// Mangle a scoped package name: `@scope/name` → `scope__name`.
pub fn mangle_scoped_package_name(package_name: &str) -> String {
    if package_name.starts_with('@') {
        if let Some(idx) = package_name.find('/') {
            return format!("{}__{}", &package_name[1..idx], &package_name[idx + 1..]);
        }
    }
    package_name.to_string()
}

/// Unmangle a scoped package name: `scope__name` → `@scope/name`.
pub fn unmangle_scoped_package_name(package_name: &str) -> String {
    if let Some(idx) = package_name.find("__") {
        return format!("@{}/{}", &package_name[..idx], &package_name[idx + 2..]);
    }
    package_name.to_string()
}

/// Get the @types package name for a given package name.
pub fn get_types_package_name(package_name: &str) -> String {
    format!("@types/{}", mangle_scoped_package_name(package_name))
}

/// Get the original package name from a @types package name.
pub fn get_package_name_from_types_package_name(mangled_name: &str) -> String {
    if let Some(rest) = mangled_name.strip_prefix("@types/") {
        unmangle_scoped_package_name(rest)
    } else {
        mangled_name.to_string()
    }
}

/// Parse the node_modules package path from a resolved path.
///
/// Returns the directory containing the package, or empty string if not found.
pub fn parse_node_module_from_path(resolved: &str, is_folder: bool) -> String {
    let path = tspath::normalize_path(resolved);
    let idx = match path.rfind("/node_modules/") {
        Some(i) => i,
        None => return String::new(),
    };

    let index_after_node_modules = idx + "/node_modules/".len();
    let mut index_after_package_name =
        move_to_next_directory_separator_if_available(&path, index_after_node_modules, is_folder);

    if path.as_bytes().get(index_after_node_modules) == Some(&b'@') {
        index_after_package_name = move_to_next_directory_separator_if_available(
            &path,
            index_after_package_name,
            is_folder,
        );
    }

    path[..index_after_package_name].to_string()
}

fn move_to_next_directory_separator_if_available(
    path: &str,
    prev_separator_index: usize,
    is_folder: bool,
) -> usize {
    let offset = prev_separator_index + 1;
    if offset > path.len() {
        return if is_folder {
            path.len()
        } else {
            prev_separator_index
        };
    }
    match path[offset..].find('/') {
        Some(rel) => offset + rel,
        None => {
            if is_folder {
                path.len()
            } else {
                prev_separator_index
            }
        }
    }
}

/// Compare two pattern keys for sorting (longer base = higher priority).
pub fn compare_pattern_keys(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let a_pattern_index = a.find('*');
    let b_pattern_index = b.find('*');
    let base_len_a = a_pattern_index.map_or(a.len(), |i| i + 1);
    let base_len_b = b_pattern_index.map_or(b.len(), |i| i + 1);

    if base_len_a > base_len_b {
        return Ordering::Less;
    }
    if base_len_b > base_len_a {
        return Ordering::Greater;
    }
    if a_pattern_index.is_none() {
        return Ordering::Greater;
    }
    if b_pattern_index.is_none() {
        return Ordering::Less;
    }
    if a.len() > b.len() {
        return Ordering::Less;
    }
    if b.len() > a.len() {
        return Ordering::Greater;
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_package_name_simple() {
        let (pkg, rest) = parse_package_name("foo");
        assert_eq!(pkg, "foo");
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_package_name_with_subpath() {
        let (pkg, rest) = parse_package_name("foo/bar");
        assert_eq!(pkg, "foo");
        assert_eq!(rest, "bar");
    }

    #[test]
    fn parse_package_name_scoped() {
        let (pkg, rest) = parse_package_name("@scope/name");
        assert_eq!(pkg, "@scope/name");
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_package_name_scoped_with_subpath() {
        let (pkg, rest) = parse_package_name("@scope/name/sub");
        assert_eq!(pkg, "@scope/name");
        assert_eq!(rest, "sub");
    }

    #[test]
    fn mangle_scoped_package() {
        assert_eq!(mangle_scoped_package_name("@scope/name"), "scope__name");
        assert_eq!(mangle_scoped_package_name("foo"), "foo");
    }

    #[test]
    fn unmangle_scoped_package() {
        assert_eq!(unmangle_scoped_package_name("scope__name"), "@scope/name");
        assert_eq!(unmangle_scoped_package_name("foo"), "foo");
    }

    #[test]
    fn types_package_name() {
        assert_eq!(get_types_package_name("foo"), "@types/foo");
        assert_eq!(get_types_package_name("@scope/name"), "@types/scope__name");
    }

    #[test]
    fn package_name_from_types() {
        assert_eq!(
            get_package_name_from_types_package_name("@types/foo"),
            "foo"
        );
        assert_eq!(
            get_package_name_from_types_package_name("@types/scope__name"),
            "@scope/name"
        );
    }

    #[test]
    fn parse_node_module_path() {
        // From Go tests
        assert_eq!(
            parse_node_module_from_path("/a/node_modules/b/lib/index.d.ts", false),
            "/a/node_modules/b"
        );
        assert_eq!(
            parse_node_module_from_path("/a/node_modules/@scope/b/lib/index.d.ts", false),
            "/a/node_modules/@scope/b"
        );
        assert_eq!(
            parse_node_module_from_path("/a/node_modules/b/lib/File", true),
            "/a/node_modules/b"
        );
        assert_eq!(
            parse_node_module_from_path("/a/node_modules/@scope/b/lib/File", true),
            "/a/node_modules/@scope/b"
        );
        assert_eq!(
            parse_node_module_from_path("/a/node_modules/b", true),
            "/a/node_modules/b"
        );
        assert_eq!(
            parse_node_module_from_path("/a/node_modules/@scope/b", true),
            "/a/node_modules/@scope/b"
        );
        assert_eq!(
            parse_node_module_from_path("/a/node_modules/@scope", true),
            "/a/node_modules/@scope"
        );
        assert_eq!(
            parse_node_module_from_path("/a/node_modules/@types", true),
            "/a/node_modules/@types"
        );
        assert_eq!(parse_node_module_from_path("/a/src/index.ts", false), "");
    }

    #[test]
    fn parse_node_module_not_found() {
        assert_eq!(parse_node_module_from_path("/home/user/foo.js", false), "");
    }
}
