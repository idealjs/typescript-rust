//! Module resolution infrastructure, ported from `internal/module/resolver.go`,
//! `internal/module/types.go`, and `internal/module/cache.go`.
//!
//! This module provides the `Resolver` struct, `ResolutionHost` trait,
//! cache types, and the `resolutionState` state machine for resolving
//! module specifiers and type reference directives.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bitflags::bitflags;

use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ResolutionMode,
};
use crate::tspath;
use crate::vfs::FS;

use super::{NodeResolutionFeatures, PackageId, ResolvedModule, ResolvedTypeReferenceDirective};

// ── Extensions bitfield ─────────────────────────────────────────────

bitflags! {
    /// Bitfield for extension sets used during module resolution.
    /// Mirrors Go's `extensions` int32 type.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Extensions: i32 {
        const TYPESCRIPT     = 1;
        const JAVASCRIPT     = 1 << 1;
        const DECLARATION    = 1 << 2;
        const JSON           = 1 << 3;
    }
}

impl Extensions {
    pub const IMPLEMENTATION_FILES: Extensions =
        Extensions::TYPESCRIPT.union(Extensions::JAVASCRIPT);

    /// Expand the bitfield into an ordered list of file extensions.
    pub fn array(&self) -> Vec<&'static str> {
        let mut result = Vec::new();
        if self.contains(Extensions::TYPESCRIPT) {
            result.extend_from_slice(&tspath::SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS);
        }
        if self.contains(Extensions::JAVASCRIPT) {
            result.extend_from_slice(&tspath::SUPPORTED_JS_EXTENSIONS_FLAT);
        }
        if self.contains(Extensions::DECLARATION) {
            result.extend_from_slice(&tspath::SUPPORTED_DECLARATION_EXTENSIONS);
        }
        if self.contains(Extensions::JSON) {
            result.push(tspath::EXTENSION_JSON);
        }
        result
    }

    pub fn extensions_string(&self) -> String {
        let mut parts = Vec::new();
        if self.contains(Extensions::TYPESCRIPT) {
            parts.push("TypeScript");
        }
        if self.contains(Extensions::JAVASCRIPT) {
            parts.push("JavaScript");
        }
        if self.contains(Extensions::DECLARATION) {
            parts.push("Declaration");
        }
        if self.contains(Extensions::JSON) {
            parts.push("JSON");
        }
        parts.join(", ")
    }
}

// ── ResolutionHost trait ────────────────────────────────────────────

/// Abstraction over the file system and environment needed for module
/// resolution. Mirrors Go's `ResolutionHost` interface.
pub trait ResolutionHost {
    fn fs(&self) -> &dyn FS;
    fn get_current_directory(&self) -> &str;
}

// ── Cache types ─────────────────────────────────────────────────────

/// Key for module resolution cache entries.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ModuleResolutionCacheKey {
    containing_directory: String,
    module_name: String,
    resolution_mode: ResolutionMode,
    redirect_config_name: String,
}

/// Key for type reference directive cache entries.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct TypeRefDirectiveCacheKey {
    containing_directory: String,
    type_reference_name: String,
    resolution_mode: ResolutionMode,
    redirect_config_name: String,
    from_inferred_types_containing_file: bool,
}

/// Cache for module resolution results. Uses first-writer-wins semantics
/// (mirrors Go's `moduleResolutionCache.Set` → `LoadOrStore`).
pub struct ModuleResolutionCache {
    cache: Mutex<HashMap<ModuleResolutionCacheKey, Arc<ResolvedModule>>>,
}

impl ModuleResolutionCache {
    pub fn new() -> Self {
        ModuleResolutionCache {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &ModuleResolutionCacheKey) -> Option<Arc<ResolvedModule>> {
        self.cache.lock().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: ModuleResolutionCacheKey, value: Arc<ResolvedModule>) {
        let mut cache = self.cache.lock().unwrap();
        // First-writer-wins: don't overwrite existing entries.
        cache.entry(key).or_insert(value);
    }
}

impl Default for ModuleResolutionCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache for type reference directive results. Uses last-writer-wins
/// semantics (mirrors Go's `typeRefDirectiveResolutionCache.Set` → `Store`).
pub struct TypeRefDirectiveResolutionCache {
    cache: Mutex<HashMap<TypeRefDirectiveCacheKey, Arc<ResolvedTypeReferenceDirective>>>,
}

impl TypeRefDirectiveResolutionCache {
    pub fn new() -> Self {
        TypeRefDirectiveResolutionCache {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &TypeRefDirectiveCacheKey) -> Option<Arc<ResolvedTypeReferenceDirective>> {
        self.cache.lock().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: TypeRefDirectiveCacheKey, value: Arc<ResolvedTypeReferenceDirective>) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, value);
    }
}

impl Default for TypeRefDirectiveResolutionCache {
    fn default() -> Self {
        Self::new()
    }
}

// ── Resolved (internal) ─────────────────────────────────────────────

/// Internal resolution result, lower-level than `ResolvedModule`.
#[derive(Clone, Debug, Default)]
pub(crate) struct Resolved {
    pub path: String,
    pub extension: String,
    pub package_id: Option<PackageId>,
    pub original_path: String,
    pub resolved_using_ts_extension: bool,
}

impl Resolved {
    pub fn is_resolved(&self) -> bool {
        !self.path.is_empty()
    }
}

/// Sentinel for "continue searching" (Go returns nil pointer).
pub(crate) const CONTINUE_SEARCHING: Option<Resolved> = None;

// ── DiagAndArgs (for trace resolution) ──────────────────────────────

/// A diagnostic message with arguments, used for `--traceResolution` output.
#[derive(Clone, Debug)]
pub struct DiagAndArgs {
    pub message: String,
    pub args: Vec<String>,
}

// ── Resolver ────────────────────────────────────────────────────────

/// The module resolver. Mirrors Go's `Resolver` struct.
///
/// Holds caches, the resolution host, and compiler options. Entry points
/// are `resolve_module_name` and `resolve_type_reference_directive`.
pub struct Resolver {
    module_cache: ModuleResolutionCache,
    type_ref_cache: TypeRefDirectiveResolutionCache,
    host: Arc<dyn ResolutionHost + Send + Sync>,
    compiler_options: Arc<CompilerOptions>,
    typings_location: String,
    project_name: String,
}

impl Resolver {
    /// Create a new resolver.
    pub fn new(
        host: Arc<dyn ResolutionHost + Send + Sync>,
        compiler_options: Arc<CompilerOptions>,
        typings_location: String,
        project_name: String,
    ) -> Self {
        Resolver {
            module_cache: ModuleResolutionCache::new(),
            type_ref_cache: TypeRefDirectiveResolutionCache::new(),
            host,
            compiler_options,
            typings_location,
            project_name,
        }
    }

    pub fn host(&self) -> &dyn ResolutionHost {
        self.host.as_ref()
    }

    pub fn compiler_options(&self) -> &CompilerOptions {
        &self.compiler_options
    }

    /// Resolve a module specifier (e.g., `"./foo"` or `"react"`).
    ///
    /// Returns `(resolved_module, traces)`. When `--traceResolution` is off,
    /// `traces` is empty.
    pub fn resolve_module_name(
        &self,
        module_name: &str,
        containing_file: &str,
        resolution_mode: ResolutionMode,
        _redirected_reference: Option<&str>,
    ) -> (Option<ResolvedModule>, Vec<DiagAndArgs>) {
        let containing_directory = tspath::get_directory_path(containing_file);
        let cache_key = ModuleResolutionCacheKey {
            containing_directory: containing_directory.to_string(),
            module_name: module_name.to_string(),
            resolution_mode,
            redirect_config_name: String::new(),
        };

        // Check cache (skip when tracing).
        let trace = self.compiler_options.trace_resolution.is_true();
        if !trace {
            if let Some(cached) = self.module_cache.get(&cache_key) {
                return (Some((*cached).clone()), Vec::new());
            }
        }

        // TODO: Port the full resolution pipeline (resolveNodeLike).
        // For now, return unresolved.
        let result = ResolvedModule::default();
        let result_arc = Arc::new(result.clone());
        self.module_cache.set(cache_key, result_arc);
        (Some(result), Vec::new())
    }

    /// Resolve a type reference directive (e.g., `"node"` or `"jest"`).
    pub fn resolve_type_reference_directive(
        &self,
        type_reference_directive_name: &str,
        containing_file: &str,
        resolution_mode: ResolutionMode,
        _redirected_reference: Option<&str>,
    ) -> (Option<ResolvedTypeReferenceDirective>, Vec<DiagAndArgs>) {
        let containing_directory = tspath::get_directory_path(containing_file);
        let from_inferred_types_containing_file =
            containing_file.ends_with(super::INFERRED_TYPES_CONTAINING_FILE);
        let cache_key = TypeRefDirectiveCacheKey {
            containing_directory: containing_directory.to_string(),
            type_reference_name: type_reference_directive_name.to_string(),
            resolution_mode,
            redirect_config_name: String::new(),
            from_inferred_types_containing_file,
        };

        let trace = self.compiler_options.trace_resolution.is_true();
        if !trace {
            if let Some(cached) = self.type_ref_cache.get(&cache_key) {
                return (Some((*cached).clone()), Vec::new());
            }
        }

        // TODO: Port the full type reference directive resolution.
        let result = ResolvedTypeReferenceDirective::default();
        let result_arc = Arc::new(result.clone());
        self.type_ref_cache.set(cache_key, result_arc);
        (Some(result), Vec::new())
    }
}

// ── Effective type roots ────────────────────────────────────────────

/// Compute effective type roots for type reference directive resolution.
/// Mirrors Go's `CompilerOptions.GetEffectiveTypeRoots`.
///
/// When `typeRoots` is explicitly set in compiler options, returns it directly.
/// Otherwise, computes `[cwd]/node_modules/@types` as the default.
pub fn get_effective_type_roots(
    options: &CompilerOptions,
    current_directory: &str,
) -> (Vec<String>, bool) {
    if !options.type_roots.is_empty() {
        return (options.type_roots.clone(), true);
    }
    // Default: <cwd>/node_modules/@types
    let default_type_root = tspath::combine_paths(current_directory, &["node_modules", "@types"]);
    (vec![default_type_root], false)
}

// ── ResolutionState ─────────────────────────────────────────────────

/// Per-resolution mutable state. Mirrors Go's `resolutionState`.
///
/// This is the workhorse of the resolver — it carries the request parameters
/// (module name, containing directory, features, extensions, conditions)
/// and accumulates diagnostics as resolution proceeds.
pub(crate) struct ResolutionState<'a> {
    name: String,
    containing_directory: String,
    is_config_lookup: bool,
    features: NodeResolutionFeatures,
    esm_mode: bool,
    conditions: Vec<String>,
    extensions: Extensions,
    compiler_options: &'a CompilerOptions,
    resolve_package_directory_only: bool,
}

impl<'a> ResolutionState<'a> {
    /// Create a new resolution state, deriving features/esmMode/conditions
    /// from the module resolution kind.
    pub(crate) fn new(
        name: &str,
        containing_directory: &str,
        is_type_reference_directive: bool,
        _resolution_mode: ResolutionMode,
        compiler_options: &'a CompilerOptions,
    ) -> Self {
        // Compute extensions.
        let extensions = if is_type_reference_directive {
            Extensions::DECLARATION
        } else if compiler_options.no_dts_resolution.is_true() {
            Extensions::IMPLEMENTATION_FILES
        } else {
            Extensions::TYPESCRIPT
                .union(Extensions::JAVASCRIPT)
                .union(Extensions::DECLARATION)
        };
        let extensions = if !is_type_reference_directive
            && compiler_options.get_resolve_json_module()
        {
            extensions.union(Extensions::JSON)
        } else {
            extensions
        };

        // Compute features, esmMode, conditions from module resolution kind.
        let (features, esm_mode, conditions) = match compiler_options.get_module_resolution_kind() {
            ModuleResolutionKind::Node16 => (
                NodeResolutionFeatures::NODE16_DEFAULT,
                true,
                get_conditions(compiler_options, ModuleKind::Node16),
            ),
            ModuleResolutionKind::NodeNext => (
                NodeResolutionFeatures::NODE_NEXT_DEFAULT,
                true,
                get_conditions(compiler_options, ModuleKind::NodeNext),
            ),
            ModuleResolutionKind::Bundler => (
                NodeResolutionFeatures::BUNDLER_DEFAULT,
                false,
                get_conditions(compiler_options, ModuleKind::ESNext),
            ),
            _ => (
                NodeResolutionFeatures::NONE,
                false,
                Vec::new(),
            ),
        };

        ResolutionState {
            name: name.to_string(),
            containing_directory: containing_directory.to_string(),
            is_config_lookup: false,
            features,
            esm_mode,
            conditions,
            extensions,
            compiler_options,
            resolve_package_directory_only: false,
        }
    }

    // ── Relative path resolution ────────────────────────────────────

    /// Normalize a path for CJS resolution. If the last path component is
    /// `.` or `..`, append a trailing directory separator.
    fn normalize_path_for_cjs_resolution(directory: &str, name: &str) -> String {
        let combined = tspath::combine_paths(directory, &[name]);
        // Check the last component BEFORE normalization, since normalize
        // would strip `.` and `..` components.
        let last_component = tspath::get_base_file_name(&combined);
        let combined = tspath::normalize_path(&combined);
        if last_component == "." || last_component == ".." {
            tspath::ensure_trailing_directory_separator(&combined)
        } else {
            combined
        }
    }

    /// Try to load a module by relative name. First tries file resolution
    /// (with extension swapping), then directory resolution (package.json
    /// + index.js). Mirrors Go's `nodeLoadModuleByRelativeName`.
    fn node_load_module_by_relative_name(
        &mut self,
        extensions: Extensions,
        candidate: &str,
        _consider_package_json: bool,
        fs: &dyn FS,
    ) -> Option<Resolved> {
        if !tspath::has_trailing_directory_separator(candidate) {
            let parent_of_candidate = tspath::get_directory_path(candidate);
            if !fs.directory_exists(&parent_of_candidate) {
                return CONTINUE_SEARCHING;
            }
            if let Some(resolved) = self.load_module_from_file(extensions, candidate, fs) {
                return Some(resolved);
            }
        }
        if !fs.directory_exists(candidate) {
            return CONTINUE_SEARCHING;
        }
        // In ESM mode, directory lookups (package.json redirection and
        // implicit index.js) are skipped — only file resolution applies.
        if self.esm_mode {
            return CONTINUE_SEARCHING;
        }
        // TODO: load_node_module_from_directory (package.json types/main + index lookup)
        CONTINUE_SEARCHING
    }

    /// Try to load a module from a file, first by replacing the extension,
    /// then by appending extensions. Mirrors Go's `loadModuleFromFile`.
    fn load_module_from_file(
        &self,
        extensions: Extensions,
        candidate: &str,
        fs: &dyn FS,
    ) -> Option<Resolved> {
        // ./foo.js → ./foo.ts (extension replacement)
        if let Some(resolved) =
            self.load_module_from_file_no_implicit_extensions(extensions, candidate, fs)
        {
            return Some(resolved);
        }
        // ./foo → ./foo.ts (extension appending, CJS only)
        if !self.esm_mode {
            return self.try_adding_extensions(candidate, extensions, "", fs);
        }
        CONTINUE_SEARCHING
    }

    /// Strip the candidate's extension and try replacing it with TS/DTS/JS
    /// extensions. Mirrors Go's `loadModuleFromFileNoImplicitExtensions`.
    fn load_module_from_file_no_implicit_extensions(
        &self,
        extensions: Extensions,
        candidate: &str,
        fs: &dyn FS,
    ) -> Option<Resolved> {
        let base = tspath::get_base_file_name(candidate);
        if !base.contains('.') {
            return CONTINUE_SEARCHING;
        }
        let extensionless = tspath::remove_file_extension(candidate);
        if extensionless == candidate {
            return CONTINUE_SEARCHING;
        }
        let extension = &candidate[extensionless.len()..];
        self.try_adding_extensions(&extensionless, extensions, extension, fs)
    }

    /// The core extension-priority table. Given an extensionless path and
    /// the original extension, tries TS/DTS/JS extensions in priority order.
    /// Mirrors Go's `tryAddingExtensions`.
    fn try_adding_extensions(
        &self,
        extensionless: &str,
        extensions: Extensions,
        original_extension: &str,
        fs: &dyn FS,
    ) -> Option<Resolved> {
        let directory = tspath::get_directory_path(extensionless);
        if !directory.is_empty() && !fs.directory_exists(&directory) {
            return CONTINUE_SEARCHING;
        }

        match original_extension {
            ".mjs" | ".mts" | ".d.mts" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".mts", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.mts", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".mjs", extensionless, fs) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".cjs" | ".cts" | ".d.cts" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".cts", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.cts", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".cjs", extensionless, fs) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".json" => {
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.json.ts", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JSON) {
                    if let Some(r) = self.try_extension(".json", extensionless, fs) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".tsx" | ".jsx" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".tsx", extensionless, fs) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".ts", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.ts", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".jsx", extensionless, fs) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".js", extensionless, fs) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            // .ts, .d.ts, .js, or "" (extensionless)
            ".ts" | ".d.ts" | ".js" | "" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".ts", extensionless, fs) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".tsx", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.ts", extensionless, fs) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".js", extensionless, fs) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".jsx", extensionless, fs) {
                        return Some(r);
                    }
                }
                if self.is_config_lookup {
                    if let Some(r) = self.try_extension(".json", extensionless, fs) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            _ => {
                // Arbitrary extensions: try .d.<ext>.ts for declaration mapping
                if extensions.contains(Extensions::DECLARATION)
                    && !tspath::is_declaration_file_name(&format!("{extensionless}{original_extension}"))
                {
                    let ext = format!(".d{original_extension}.ts");
                    if let Some(r) = self.try_extension(&ext, extensionless, fs) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
        }
    }

    /// Try a single extension on an extensionless path. Returns the resolved
    /// path if the file exists. Mirrors Go's `tryExtension`.
    fn try_extension(
        &self,
        extension: &str,
        extensionless: &str,
        fs: &dyn FS,
    ) -> Option<Resolved> {
        let file_name = format!("{extensionless}{extension}");
        if self.try_file(&file_name, fs) {
            return Some(Resolved {
                path: file_name,
                extension: extension.to_string(),
                ..Default::default()
            });
        }
        CONTINUE_SEARCHING
    }

    /// Check if a file exists, applying moduleSuffixes if configured.
    /// Mirrors Go's `tryFile`.
    fn try_file(&self, file_name: &str, fs: &dyn FS) -> bool {
        if self.compiler_options.module_suffixes.is_empty() {
            return fs.file_exists(file_name);
        }
        let ext = tspath::try_get_extension_from_path(file_name);
        let file_name_no_ext = tspath::remove_extension(file_name, ext);
        for suffix in &self.compiler_options.module_suffixes {
            let path = format!("{file_name_no_ext}{suffix}{ext}");
            if fs.file_exists(&path) {
                return true;
            }
        }
        false
    }
}

/// Derive the conditions array for conditional exports/imports resolution.
/// Mirrors Go's `GetConditions`.
fn get_conditions(options: &CompilerOptions, module_kind: ModuleKind) -> Vec<String> {
    let mut conditions = Vec::new();
    match module_kind {
        ModuleKind::Node16 | ModuleKind::NodeNext => {
            conditions.push("node".to_string());
            // Resolution mode determines import vs require.
            // For now, add both.
            conditions.push("import".to_string());
            conditions.push("require".to_string());
            conditions.push("types".to_string());
        }
        _ => {
            conditions.push("import".to_string());
            conditions.push("types".to_string());
        }
    }
    // Add custom conditions.
    for custom in &options.custom_conditions {
        conditions.push(custom.clone());
    }
    conditions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_bitfield() {
        let ts = Extensions::TYPESCRIPT;
        assert!(ts.contains(Extensions::TYPESCRIPT));
        assert!(!ts.contains(Extensions::JAVASCRIPT));

        let both = Extensions::TYPESCRIPT.union(Extensions::JAVASCRIPT);
        assert_eq!(both, Extensions::IMPLEMENTATION_FILES);

        let all = Extensions::TYPESCRIPT
            .union(Extensions::JAVASCRIPT)
            .union(Extensions::DECLARATION)
            .union(Extensions::JSON);
        assert_eq!(all.bits(), 0b1111);
    }

    #[test]
    fn extensions_array() {
        let ts = Extensions::TYPESCRIPT;
        let arr = ts.array();
        assert!(arr.contains(&".ts"));
        assert!(arr.contains(&".tsx"));

        let decl = Extensions::DECLARATION;
        let arr = decl.array();
        assert!(arr.contains(&".d.ts"));
    }

    #[test]
    fn extensions_string() {
        let both = Extensions::TYPESCRIPT.union(Extensions::JAVASCRIPT);
        assert_eq!(both.extensions_string(), "TypeScript, JavaScript");
    }

    #[test]
    fn module_cache_first_writer_wins() {
        let cache = ModuleResolutionCache::new();
        let key = ModuleResolutionCacheKey {
            containing_directory: "/foo".to_string(),
            module_name: "bar".to_string(),
            resolution_mode: ModuleKind::None,
            redirect_config_name: String::new(),
        };
        let mod1 = Arc::new(ResolvedModule {
            resolved_file_name: "/foo/bar1.ts".to_string(),
            ..Default::default()
        });
        let mod2 = Arc::new(ResolvedModule {
            resolved_file_name: "/foo/bar2.ts".to_string(),
            ..Default::default()
        });
        cache.set(key.clone(), mod1);
        cache.set(key.clone(), mod2); // should NOT overwrite
        let result = cache.get(&key).unwrap();
        assert_eq!(result.resolved_file_name, "/foo/bar1.ts");
    }

    #[test]
    fn type_ref_cache_last_writer_wins() {
        let cache = TypeRefDirectiveResolutionCache::new();
        let key = TypeRefDirectiveCacheKey {
            containing_directory: "/foo".to_string(),
            type_reference_name: "node".to_string(),
            resolution_mode: ModuleKind::None,
            redirect_config_name: String::new(),
            from_inferred_types_containing_file: false,
        };
        let dir1 = Arc::new(ResolvedTypeReferenceDirective {
            resolved_file_name: "/foo/node1.d.ts".to_string(),
            ..Default::default()
        });
        let dir2 = Arc::new(ResolvedTypeReferenceDirective {
            resolved_file_name: "/foo/node2.d.ts".to_string(),
            ..Default::default()
        });
        cache.set(key.clone(), dir1);
        cache.set(key.clone(), dir2); // SHOULD overwrite
        let result = cache.get(&key).unwrap();
        assert_eq!(result.resolved_file_name, "/foo/node2.d.ts");
    }

    #[test]
    fn effective_type_roots_default() {
        let opts = CompilerOptions::default();
        let (roots, from_config) = get_effective_type_roots(&opts, "/project");
        assert!(!from_config);
        assert_eq!(roots.len(), 1);
        assert!(roots[0].contains("node_modules/@types"));
    }

    #[test]
    fn effective_type_roots_explicit() {
        let mut opts = CompilerOptions::default();
        opts.type_roots = vec!["./custom-types".to_string()];
        let (roots, from_config) = get_effective_type_roots(&opts, "/project");
        assert!(from_config);
        assert_eq!(roots, vec!["./custom-types".to_string()]);
    }

    // ── Relative path resolution tests ─────────────────────────────

    fn make_state<'a>(name: &str, containing_dir: &str, opts: &'a CompilerOptions) -> ResolutionState<'a> {
        ResolutionState::new(
            name,
            containing_dir,
            false, // is_type_reference_directive
            ModuleKind::None,
            opts,
        )
    }

    #[test]
    fn resolve_relative_ts_file() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.write_file("/src/foo.ts", "export const x = 1;").unwrap();

        let opts = CompilerOptions::default();
        let mut state = make_state("./foo", "/src", &opts);
        let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo");
        let result = state.node_load_module_by_relative_name(
            Extensions::TYPESCRIPT
                .union(Extensions::JAVASCRIPT)
                .union(Extensions::DECLARATION),
            &candidate,
            true,
            &fs,
        );
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.path, "/src/foo.ts");
        assert_eq!(resolved.extension, ".ts");
    }

    #[test]
    fn resolve_relative_tsx_file() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.write_file("/src/component.tsx", "export const C = 1;").unwrap();

        let opts = CompilerOptions::default();
        let mut state = make_state("./component", "/src", &opts);
        let candidate =
            ResolutionState::normalize_path_for_cjs_resolution("/src", "./component");
        let result = state.node_load_module_by_relative_name(
            Extensions::TYPESCRIPT
                .union(Extensions::JAVASCRIPT)
                .union(Extensions::DECLARATION),
            &candidate,
            true,
            &fs,
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().extension, ".tsx");
    }

    #[test]
    fn resolve_relative_js_specifier_swaps_to_ts() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        // Import "./foo.js" but only foo.ts exists → should resolve to foo.ts
        fs.write_file("/src/foo.ts", "export const x = 1;").unwrap();

        let opts = CompilerOptions::default();
        let mut state = make_state("./foo.js", "/src", &opts);
        let candidate =
            ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo.js");
        let result = state.node_load_module_by_relative_name(
            Extensions::TYPESCRIPT
                .union(Extensions::JAVASCRIPT)
                .union(Extensions::DECLARATION),
            &candidate,
            true,
            &fs,
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/src/foo.ts");
    }

    #[test]
    fn resolve_relative_mjs_specifier_swaps_to_mts() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.write_file("/src/foo.mts", "export const x = 1;").unwrap();

        let opts = CompilerOptions::default();
        let mut state = make_state("./foo.mjs", "/src", &opts);
        let candidate =
            ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo.mjs");
        let result = state.node_load_module_by_relative_name(
            Extensions::TYPESCRIPT
                .union(Extensions::JAVASCRIPT)
                .union(Extensions::DECLARATION),
            &candidate,
            true,
            &fs,
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/src/foo.mts");
    }

    #[test]
    fn resolve_relative_nonexistent_returns_none() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");

        let opts = CompilerOptions::default();
        let mut state = make_state("./missing", "/src", &opts);
        let candidate =
            ResolutionState::normalize_path_for_cjs_resolution("/src", "./missing");
        let result = state.node_load_module_by_relative_name(
            Extensions::TYPESCRIPT
                .union(Extensions::JAVASCRIPT)
                .union(Extensions::DECLARATION),
            &candidate,
            true,
            &fs,
        );
        assert!(result.is_none());
    }

    #[test]
    fn resolve_relative_parent_dir_not_exists() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        // No directory created at all.

        let opts = CompilerOptions::default();
        let mut state = make_state("./foo", "/nonexistent", &opts);
        let candidate =
            ResolutionState::normalize_path_for_cjs_resolution("/nonexistent", "./foo");
        let result = state.node_load_module_by_relative_name(
            Extensions::TYPESCRIPT,
            &candidate,
            true,
            &fs,
        );
        assert!(result.is_none());
    }

    #[test]
    fn normalize_path_for_dot() {
        let result = ResolutionState::normalize_path_for_cjs_resolution("/src", ".");
        assert!(result.ends_with('/'));
        assert!(tspath::has_trailing_directory_separator(&result));
    }

    #[test]
    fn normalize_path_for_dot_dot() {
        let result = ResolutionState::normalize_path_for_cjs_resolution("/src", "..");
        assert!(tspath::has_trailing_directory_separator(&result));
    }
}
