//! Module resolution, ported from `internal/module/`.
//!
//! This module provides the core types and utility functions for module
//! resolution, plus the `Resolver` struct and resolution infrastructure.

pub mod resolver;

// Re-export key types from the resolver submodule.
pub use resolver::{
    DiagAndArgs, Extensions as ExtensionsBitfield, ResolutionHost, Resolver,
    get_effective_type_roots,
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

    /// Ported from Go `TestParseNodeModuleFromPath`.
    #[test]
    fn parse_node_module_from_path() {
        let cases: &[(&str, &str, bool, &str)] = &[
            (
                "file in package",
                "/a/node_modules/b/lib/index.d.ts",
                false,
                "/a/node_modules/b",
            ),
            (
                "file in scoped package",
                "/a/node_modules/@scope/b/lib/index.d.ts",
                false,
                "/a/node_modules/@scope/b",
            ),
            (
                "folder subpath",
                "/a/node_modules/b/lib/File",
                true,
                "/a/node_modules/b",
            ),
            (
                "folder subpath scoped",
                "/a/node_modules/@scope/b/lib/File",
                true,
                "/a/node_modules/@scope/b",
            ),
            (
                "package root folder",
                "/a/node_modules/b",
                true,
                "/a/node_modules/b",
            ),
            (
                "scoped package root folder",
                "/a/node_modules/@scope/b",
                true,
                "/a/node_modules/@scope/b",
            ),
            // A bare scope directory has no package name; must not panic.
            (
                "scope-only folder",
                "/a/node_modules/@scope",
                true,
                "/a/node_modules/@scope",
            ),
            (
                "types scope-only folder",
                "/a/node_modules/@types",
                true,
                "/a/node_modules/@types",
            ),
            ("not in node_modules", "/a/src/index.ts", false, ""),
        ];
        for (name, path, is_folder, want) in cases {
            let got = super::parse_node_module_from_path(path, *is_folder);
            assert_eq!(
                got, *want,
                "parse_node_module_from_path({path:?}, {is_folder})"
            );
            let _ = name;
        }
    }

    /// Ported from Go `TestResolveModuleNameTrailingSlash`.
    ///
    /// Resolving a node_modules import with a trailing slash (e.g. `pkg/`) must
    /// produce the same result as without one.
    #[test]
    fn resolve_module_name_trailing_slash() {
        use crate::core::compiler_options::{
            CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
        };
        use crate::vfs::{FS, InMemoryFS};
        use std::sync::Arc;

        struct ResolutionHostStub {
            fs: InMemoryFS,
            cwd: String,
        }
        impl ResolutionHost for ResolutionHostStub {
            fn fs(&self) -> &dyn FS {
                &self.fs
            }
            fn get_current_directory(&self) -> &str {
                &self.cwd
            }
        }

        let fs = InMemoryFS::new();
        for dir in [
            "/repo",
            "/repo/src",
            "/repo/node_modules",
            "/repo/node_modules/pkg",
        ] {
            fs.insert_dir(dir);
        }
        fs.write_file(
            "/repo/node_modules/pkg/package.json",
            r#"{"name":"pkg","main":"main.js","types":"main.d.ts"}"#,
        )
        .unwrap();
        fs.write_file(
            "/repo/node_modules/pkg/main.d.ts",
            "export const x: number;",
        )
        .unwrap();
        fs.write_file("/repo/node_modules/pkg/main.js", "exports.x = 1;")
            .unwrap();
        fs.write_file("/repo/src/file.ts", "").unwrap();

        let host = Arc::new(ResolutionHostStub {
            fs,
            cwd: "/repo".to_string(),
        });
        let mut opts = CompilerOptions::default();
        opts.module_resolution = ModuleResolutionKind::Bundler;
        opts.module = ModuleKind::ESNext;
        opts.target = ScriptTarget::ESNext;
        let resolver = Resolver::new(host, Arc::new(opts), String::new(), String::new());

        for name in ["pkg", "pkg/"] {
            let (resolved, _) =
                resolver.resolve_module_name(name, "/repo/src/file.ts", ModuleKind::ESNext, None);
            assert!(
                resolved.as_ref().is_some_and(|r| r.is_resolved()),
                "{name:?} failed to resolve"
            );
        }
    }

    // The following tests are adapted from Go goroutine race tests. Rust
    // prevents the original data races at compile time, so instead of testing
    // for races we exercise the same module-resolution logic concurrently
    // across multiple threads and verify every thread gets the same correct
    // result.

    /// Ported from Go `TestResolveModuleNameTrailingSlashRace`.
    ///
    /// Rust prevents the original data race at compile time, so instead we
    /// exercise the same module-resolution logic concurrently: resolving
    /// `pkg` and `pkg/` (trailing slash) from N threads and asserting every
    /// thread resolves successfully and to the identical file name.
    #[test]
    fn resolve_module_name_trailing_slash_race() {
        use crate::core::compiler_options::ModuleKind;
        use std::thread;

        const N: usize = 8;
        let resolver = build_concurrent_resolver();

        let results = thread::scope(|s| {
            (0..N)
                .map(|i| {
                    let name = if i % 2 == 0 { "pkg" } else { "pkg/" };
                    let r = &resolver;
                    s.spawn(move || {
                        let (resolved, _) = r.resolve_module_name(
                            name,
                            "/repo/src/file.ts",
                            ModuleKind::ESNext,
                            None,
                        );
                        (name, resolved)
                    })
                })
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });

        // Every thread resolves successfully.
        for (name, r) in &results {
            assert!(
                r.as_ref().is_some_and(|m| m.is_resolved()),
                "{name:?} failed to resolve"
            );
        }
        // "pkg" and "pkg/" resolve to the same file (no trailing-slash race).
        let expected = results[0].1.as_ref().unwrap().resolved_file_name.clone();
        for (name, r) in &results {
            assert_eq!(
                r.as_ref().unwrap().resolved_file_name,
                expected,
                "{name:?} resolved to a different file than the first thread"
            );
        }
    }

    /// Builds a resolver backed by an in-memory FS containing a `pkg`
    /// node_modules package, mirroring the layout used by
    /// `resolve_module_name_trailing_slash`. The `Resolver` is `Send + Sync`
    /// (its caches are guarded by `Mutex`), so `&Resolver` can be shared across
    /// scoped threads.
    fn build_concurrent_resolver() -> Resolver {
        use crate::core::compiler_options::{
            CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
        };
        use crate::vfs::{FS, InMemoryFS};
        use std::sync::Arc;

        struct ResolutionHostStub {
            fs: InMemoryFS,
            cwd: String,
        }
        impl ResolutionHost for ResolutionHostStub {
            fn fs(&self) -> &dyn FS {
                &self.fs
            }
            fn get_current_directory(&self) -> &str {
                &self.cwd
            }
        }

        let fs = InMemoryFS::new();
        for dir in [
            "/repo",
            "/repo/src",
            "/repo/node_modules",
            "/repo/node_modules/pkg",
        ] {
            fs.insert_dir(dir);
        }
        fs.write_file(
            "/repo/node_modules/pkg/package.json",
            r#"{"name":"pkg","main":"main.js","types":"main.d.ts"}"#,
        )
        .unwrap();
        fs.write_file(
            "/repo/node_modules/pkg/main.d.ts",
            "export const x: number;",
        )
        .unwrap();
        fs.write_file("/repo/node_modules/pkg/main.js", "exports.x = 1;")
            .unwrap();
        fs.write_file("/repo/src/file.ts", "").unwrap();

        let host = Arc::new(ResolutionHostStub {
            fs,
            cwd: "/repo".to_string(),
        });
        let mut opts = CompilerOptions::default();
        opts.module_resolution = ModuleResolutionKind::Bundler;
        opts.module = ModuleKind::ESNext;
        opts.target = ScriptTarget::ESNext;
        Resolver::new(host, Arc::new(opts), String::new(), String::new())
    }

    /// Adapted from Go `TestResolveSubpathNilContentsRace`.
    ///
    /// Resolves the `pkg` module from N threads concurrently and asserts every
    /// thread resolves to the same file.
    #[test]
    fn resolve_subpath_nil_contents_race() {
        use crate::core::compiler_options::ModuleKind;
        use std::thread;

        const N: usize = 8;
        let resolver = build_concurrent_resolver();

        let results = thread::scope(|s| {
            (0..N)
                .map(|_| {
                    s.spawn(|| {
                        let (resolved, _) = resolver.resolve_module_name(
                            "pkg",
                            "/repo/src/file.ts",
                            ModuleKind::ESNext,
                            None,
                        );
                        resolved
                    })
                })
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });

        assert!(
            results
                .iter()
                .all(|r| r.as_ref().is_some_and(|m| m.is_resolved()))
        );
        let expected = results[0].as_ref().unwrap().resolved_file_name.clone();
        assert!(
            results
                .iter()
                .all(|r| r.as_ref().unwrap().resolved_file_name == expected),
            "threads disagreed on resolved file name: {results:?}"
        );
    }

    /// Adapted from Go `TestResolvePeerDependencyNilContentsRace`.
    ///
    /// Same as the subpath test but resolves `pkg` and `pkg/` (trailing slash)
    /// concurrently, verifying both spellings resolve identically across all
    /// threads without races.
    #[test]
    fn resolve_peer_dependency_nil_contents_race() {
        use crate::core::compiler_options::ModuleKind;
        use std::thread;

        const N: usize = 8;
        let resolver = build_concurrent_resolver();

        let results = thread::scope(|s| {
            (0..N)
                .flat_map(|i| {
                    let name = if i % 2 == 0 { "pkg" } else { "pkg/" };
                    let r = &resolver;
                    Some(s.spawn(move || {
                        let (resolved, _) = r.resolve_module_name(
                            name,
                            "/repo/src/file.ts",
                            ModuleKind::ESNext,
                            None,
                        );
                        resolved
                    }))
                })
                .map(|h| h.join().unwrap())
                .collect::<Vec<_>>()
        });

        assert!(
            results
                .iter()
                .all(|r| r.as_ref().is_some_and(|m| m.is_resolved()))
        );
        let expected = results[0].as_ref().unwrap().resolved_file_name.clone();
        assert!(
            results
                .iter()
                .all(|r| r.as_ref().unwrap().resolved_file_name == expected),
            "threads disagreed on resolved file name: {results:?}"
        );
    }
}
