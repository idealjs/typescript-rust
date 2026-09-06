use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bitflags::bitflags;

use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ResolutionMode,
};
use crate::packagejson;
use crate::tspath;
use crate::vfs::FS;

use super::{
    NodeResolutionFeatures, PackageId, ResolvedModule, ResolvedTypeReferenceDirective,
    mangle_scoped_package_name, parse_package_name,
};

bitflags! {

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

pub trait ResolutionHost {
    fn fs(&self) -> &dyn FS;
    fn get_current_directory(&self) -> &str;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModuleResolutionCacheKey {
    containing_directory: String,
    module_name: String,
    resolution_mode: ResolutionMode,
    redirect_config_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeRefDirectiveCacheKey {
    containing_directory: String,
    type_reference_name: String,
    resolution_mode: ResolutionMode,
    redirect_config_name: String,
    from_inferred_types_containing_file: bool,
}

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

        cache.entry(key).or_insert(value);
    }
}

impl Default for ModuleResolutionCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TypeRefDirectiveResolutionCache {
    cache: Mutex<HashMap<TypeRefDirectiveCacheKey, Arc<ResolvedTypeReferenceDirective>>>,
}

impl TypeRefDirectiveResolutionCache {
    pub fn new() -> Self {
        TypeRefDirectiveResolutionCache {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(
        &self,
        key: &TypeRefDirectiveCacheKey,
    ) -> Option<Arc<ResolvedTypeReferenceDirective>> {
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

#[derive(Clone, Debug, Default)]
pub(crate) struct Resolved {
    pub path: String,
    pub extension: String,
    pub package_id: Option<PackageId>,
    pub original_path: String,
    pub resolved_using_ts_extension: bool,
}

impl Resolved {
    #[allow(dead_code)]
    pub fn is_resolved(&self) -> bool {
        !self.path.is_empty()
    }
}

pub(crate) const CONTINUE_SEARCHING: Option<Resolved> = None;

#[derive(Clone, Debug, Default)]
struct Pattern {
    text: String,
    star_index: i32,
}

impl Pattern {
    fn try_parse(pattern: &str) -> Pattern {
        match pattern.find('*') {
            None => Pattern {
                text: pattern.to_string(),
                star_index: -1,
            },
            Some(idx) => {
                if pattern[idx + 1..].contains('*') {
                    Pattern::default()
                } else {
                    Pattern {
                        text: pattern.to_string(),
                        star_index: idx as i32,
                    }
                }
            }
        }
    }

    fn is_valid(&self) -> bool {
        self.star_index == -1 || (self.star_index as usize) < self.text.len()
    }

    fn matches(&self, candidate: &str) -> bool {
        if self.star_index == -1 {
            return self.text == candidate;
        }
        let idx = self.star_index as usize;
        let prefix = &self.text[..idx];
        let suffix = &self.text[idx + 1..];
        candidate.len() >= self.text.len() - 1
            && candidate.starts_with(prefix)
            && candidate.ends_with(suffix)
    }

    fn matched_text(&self, candidate: &str) -> String {
        if self.star_index == -1 {
            return String::new();
        }
        let idx = self.star_index as usize;
        let suffix_len = self.text.len() - idx - 1;
        candidate[idx..candidate.len() - suffix_len].to_string()
    }
}

struct ParsedPatterns {
    matchable_string_set: std::collections::HashSet<String>,
    patterns: Vec<Pattern>,
}

fn try_parse_patterns(
    path_mappings: &std::collections::HashMap<String, Vec<String>>,
) -> ParsedPatterns {
    let mut matchable_string_set = std::collections::HashSet::new();
    let mut patterns = Vec::new();
    for path in path_mappings.keys() {
        let pattern = Pattern::try_parse(path);
        if pattern.is_valid() {
            if pattern.star_index == -1 {
                matchable_string_set.insert(path.clone());
            } else {
                patterns.push(pattern);
            }
        }
    }
    ParsedPatterns {
        matchable_string_set,
        patterns,
    }
}

fn match_pattern_or_exact(parsed: &ParsedPatterns, candidate: &str) -> Option<Pattern> {
    if parsed.matchable_string_set.contains(candidate) {
        return Some(Pattern {
            text: candidate.to_string(),
            star_index: -1,
        });
    }
    if parsed.patterns.is_empty() {
        return None;
    }

    let mut best: Option<Pattern> = None;
    let mut longest = -1i32;
    for pattern in &parsed.patterns {
        if (pattern.star_index == -1 || pattern.star_index > longest) && pattern.matches(candidate)
        {
            best = Some(pattern.clone());
            longest = pattern.star_index;
        }
    }
    best
}

#[derive(Clone, Debug)]
pub struct DiagAndArgs {
    pub message: String,
    pub args: Vec<String>,
}

pub struct Resolver {
    module_cache: ModuleResolutionCache,
    type_ref_cache: TypeRefDirectiveResolutionCache,
    host: Arc<dyn ResolutionHost + Send + Sync>,
    compiler_options: Arc<CompilerOptions>,

    #[allow(dead_code)]
    typings_location: String,
    #[allow(dead_code)]
    project_name: String,
}

impl Resolver {

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

        let trace = self.compiler_options.trace_resolution.is_true();
        if !trace {
            if let Some(cached) = self.module_cache.get(&cache_key) {
                return (Some((*cached).clone()), Vec::new());
            }
        }

        let effective_mode = default_resolution_mode(
            resolution_mode,
            &self.compiler_options,
            containing_file,
            self.host.fs(),
        );
        let state = ResolutionState::new(
            module_name,
            &containing_directory,
            false,
            effective_mode,
            &self.compiler_options,
            self.host.fs(),
            self.host.get_current_directory(),
        );
        let result = state.resolve_node_like();
        let result_arc = Arc::new(result.clone());
        self.module_cache.set(cache_key, result_arc);
        (Some(result), Vec::new())
    }

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

        let fs = self.host.fs();
        let current_dir = self.host.get_current_directory();
        let (type_roots, from_config) =
            get_effective_type_roots(&self.compiler_options, current_dir);

        let mut state = ResolutionState::new(
            type_reference_directive_name,
            &containing_directory,
            true,
            default_resolution_mode(
                resolution_mode,
                &self.compiler_options,
                containing_file,
                fs,
            ),
            &self.compiler_options,
            fs,
            current_dir,
        );
        let result = state.resolve_type_reference_directive(
            &type_roots,
            from_config,
            from_inferred_types_containing_file,
        );

        let result_arc = Arc::new(result.clone());
        self.type_ref_cache.set(cache_key, result_arc);
        (Some(result), Vec::new())
    }
}

fn default_resolution_mode(
    resolution_mode: ResolutionMode,
    options: &CompilerOptions,
    containing_file: &str,
    fs: &dyn FS,
) -> ResolutionMode {
    if resolution_mode != ResolutionMode::None {
        return resolution_mode;
    }
    match options.get_module_resolution_kind() {
        ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext => {
            crate::compiler::implied_node_format_of_file(containing_file, &|p| fs.read_file(p))
        }
        _ => ResolutionMode::None,
    }
}

pub fn get_effective_type_roots(
    options: &CompilerOptions,
    current_directory: &str,
) -> (Vec<String>, bool) {
    if !options.type_roots.is_empty() {
        return (options.type_roots.clone(), true);
    }
    let base_dir = if !options.config_file_path.is_empty() {
        tspath::get_directory_path(&options.config_file_path)
    } else {
        current_directory.to_string()
    };
    let mut type_roots = Vec::new();
    let mut dir = base_dir;
    loop {
        type_roots.push(tspath::combine_paths(&dir, &["node_modules", "@types"]));
        let parent = tspath::get_directory_path(&dir);
        if parent == dir {
            break;
        }
        dir = parent;
    }
    (type_roots, false)
}

pub(crate) struct ResolutionState<'a> {
    name: String,
    containing_directory: String,
    is_config_lookup: bool,
    features: NodeResolutionFeatures,
    esm_mode: bool,
    conditions: Vec<String>,
    extensions: Extensions,
    compiler_options: &'a CompilerOptions,
    #[allow(dead_code)]
    resolve_package_directory_only: bool,
    fs: &'a dyn FS,
    current_directory: &'a str,
    resolved_package_directory: bool,
    candidate_ending_is_from_config: bool,

    export_target_depth: u32,
}

impl<'a> ResolutionState<'a> {

    pub(crate) fn new(
        name: &str,
        containing_directory: &str,
        is_type_reference_directive: bool,
        resolution_mode: ResolutionMode,
        compiler_options: &'a CompilerOptions,
        fs: &'a dyn FS,
        current_directory: &'a str,
    ) -> Self {

        let extensions = if is_type_reference_directive {
            Extensions::DECLARATION
        } else if compiler_options.no_dts_resolution.is_true() {
            Extensions::IMPLEMENTATION_FILES
        } else {
            Extensions::TYPESCRIPT
                .union(Extensions::JAVASCRIPT)
                .union(Extensions::DECLARATION)
        };
        let extensions =
            if !is_type_reference_directive && compiler_options.get_resolve_json_module() {
                extensions.union(Extensions::JSON)
            } else {
                extensions
            };

        let (features, esm_mode, conditions) = match compiler_options.get_module_resolution_kind() {

            ModuleResolutionKind::Node16 => (
                NodeResolutionFeatures::NODE16_DEFAULT,
                resolution_mode == ModuleKind::ESNext,
                get_conditions(compiler_options, resolution_mode),
            ),
            ModuleResolutionKind::NodeNext => (
                NodeResolutionFeatures::NODE_NEXT_DEFAULT,
                resolution_mode == ModuleKind::ESNext,
                get_conditions(compiler_options, resolution_mode),
            ),
            ModuleResolutionKind::Bundler => (
                NodeResolutionFeatures::BUNDLER_DEFAULT,
                false,
                get_conditions(
                    compiler_options,
                    if resolution_mode == ResolutionMode::None {
                        ModuleKind::ESNext
                    } else {
                        resolution_mode
                    },
                ),
            ),
            _ => (NodeResolutionFeatures::NONE, false, Vec::new()),
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
            fs,
            current_directory,
            resolved_package_directory: false,
            candidate_ending_is_from_config: false,
            export_target_depth: 0,
        }
    }

    fn normalize_path_for_cjs_resolution(directory: &str, name: &str) -> String {
        let combined = tspath::combine_paths(directory, &[name]);

        let last_component = tspath::get_base_file_name(&combined);
        let combined = tspath::normalize_path(&combined);
        if last_component == "." || last_component == ".." {
            tspath::ensure_trailing_directory_separator(&combined)
        } else {
            combined
        }
    }

    fn node_load_module_by_relative_name(
        &mut self,
        extensions: Extensions,
        candidate: &str,
        _consider_package_json: bool,
    ) -> Option<Resolved> {
        if !tspath::has_trailing_directory_separator(candidate) {
            let parent_of_candidate = tspath::get_directory_path(candidate);
            if !self.fs.directory_exists(&parent_of_candidate) {
                return CONTINUE_SEARCHING;
            }
            if let Some(resolved) = self.load_module_from_file(extensions, candidate) {
                return Some(resolved);
            }
        }
        if !self.fs.directory_exists(candidate) {
            return CONTINUE_SEARCHING;
        }

        if self.esm_mode {
            return CONTINUE_SEARCHING;
        }
        self.load_node_module_from_directory(extensions, candidate, true)
    }

    fn load_module_from_file(&self, extensions: Extensions, candidate: &str) -> Option<Resolved> {

        if let Some(resolved) =
            self.load_module_from_file_no_implicit_extensions(extensions, candidate)
        {
            return Some(resolved);
        }

        if !self.esm_mode {
            return self.try_adding_extensions(candidate, extensions, "");
        }
        CONTINUE_SEARCHING
    }

    fn load_module_from_file_no_implicit_extensions(
        &self,
        extensions: Extensions,
        candidate: &str,
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
        self.try_adding_extensions(&extensionless, extensions, extension)
    }

    fn try_adding_extensions(
        &self,
        extensionless: &str,
        extensions: Extensions,
        original_extension: &str,
    ) -> Option<Resolved> {
        let directory = tspath::get_directory_path(extensionless);
        if !directory.is_empty() && !self.fs.directory_exists(&directory) {
            return CONTINUE_SEARCHING;
        }

        match original_extension {
            ".mjs" | ".mts" | ".d.mts" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".mts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.mts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".mjs", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".cjs" | ".cts" | ".d.cts" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".cts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.cts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".cjs", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".json" => {
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.json.ts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JSON) {
                    if let Some(r) = self.try_extension(".json", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".tsx" | ".jsx" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".tsx", extensionless) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".ts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.ts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".jsx", extensionless) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".js", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }

            ".ts" | ".d.ts" | ".js" | "" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".ts", extensionless) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".tsx", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.ts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".js", extensionless) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".jsx", extensionless) {
                        return Some(r);
                    }
                }
                if self.is_config_lookup {
                    if let Some(r) = self.try_extension(".json", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            _ => {

                if extensions.contains(Extensions::DECLARATION)
                    && !tspath::is_declaration_file_name(&format!(
                        "{extensionless}{original_extension}"
                    ))
                {
                    let ext = format!(".d{original_extension}.ts");
                    if let Some(r) = self.try_extension(&ext, extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
        }
    }

    fn try_extension(&self, extension: &str, extensionless: &str) -> Option<Resolved> {
        let file_name = format!("{extensionless}{extension}");
        if let Some(path) = self.try_file(&file_name) {
            return Some(Resolved {
                path,
                extension: extension.to_string(),
                resolved_using_ts_extension: true,
                ..Default::default()
            });
        }
        CONTINUE_SEARCHING
    }

    fn try_file(&self, file_name: &str) -> Option<String> {
        if self.compiler_options.module_suffixes.is_empty() {
            if self.fs.file_exists(file_name) {
                return Some(file_name.to_string());
            }
            return None;
        }
        let ext = tspath::try_get_extension_from_path(file_name);
        let file_name_no_ext = tspath::remove_extension(file_name, ext);
        for suffix in &self.compiler_options.module_suffixes {
            let path = format!("{file_name_no_ext}{suffix}{ext}");
            if self.fs.file_exists(&path) {
                return Some(path);
            }
        }
        None
    }

    fn get_paths_base_path(&self) -> String {
        if !self.compiler_options.paths_base_path.is_empty() {
            return self.compiler_options.paths_base_path.clone();
        }
        if !self.compiler_options.base_url.is_empty() {
            return self.compiler_options.base_url.clone();
        }
        if !self.compiler_options.config_file_path.is_empty() {
            return tspath::get_directory_path(&self.compiler_options.config_file_path);
        }
        self.current_directory.to_string()
    }

    fn try_load_module_using_optional_resolution_settings(&mut self) -> Option<Resolved> {
        if let Some(r) = self.try_load_module_using_paths_if_eligible() {
            return Some(r);
        }
        if !tspath::is_external_module_name_relative(&self.name) {

            if !self.compiler_options.base_url.is_empty() {
                let candidate = tspath::normalize_path(&tspath::combine_paths(
                    &self.compiler_options.base_url,
                    &[&self.name],
                ));
                if let Some(r) =
                    self.node_load_module_by_relative_name(self.extensions, &candidate, true)
                {
                    return Some(r);
                }
            }

            return CONTINUE_SEARCHING;
        }

        self.try_load_module_using_root_dirs()
    }

    fn try_load_module_using_paths_if_eligible(&mut self) -> Option<Resolved> {
        let paths = match &self.compiler_options.paths {
            Some(p) if !p.is_empty() && !tspath::path_is_relative(&self.name) => p,
            _ => return CONTINUE_SEARCHING,
        };
        let base_directory = self.get_paths_base_path();
        let parsed_patterns = try_parse_patterns(paths);
        let name = self.name.clone();
        self.try_load_module_using_paths(
            self.extensions,
            &name,
            &base_directory,
            paths,
            &parsed_patterns,
        )
    }

    fn try_load_module_using_paths(
        &mut self,
        extensions: Extensions,
        module_name: &str,
        containing_directory: &str,
        paths: &std::collections::HashMap<String, Vec<String>>,
        parsed_patterns: &ParsedPatterns,
    ) -> Option<Resolved> {
        if let Some(matched_pattern) = match_pattern_or_exact(parsed_patterns, module_name) {
            let matched_star = matched_pattern.matched_text(module_name);
            if let Some(substitutions) = paths.get(&matched_pattern.text) {
                for subst in substitutions {
                    let path = subst.replace('*', &matched_star);
                    let candidate = tspath::normalize_path(&tspath::combine_paths(
                        containing_directory,
                        &[&path],
                    ));

                    let extension_from_subst = tspath::try_get_extension_from_path(subst);
                    if !extension_from_subst.is_empty() {
                        if let Some(p) = self.try_file(&candidate) {
                            return Some(Resolved {
                                path: p,
                                extension: extension_from_subst.to_string(),
                                ..Default::default()
                            });
                        }
                    }

                    let saved = self.candidate_ending_is_from_config;
                    if !extension_from_subst.is_empty() {
                        self.candidate_ending_is_from_config = true;
                    }
                    let result =
                        self.node_load_module_by_relative_name(extensions, &candidate, true);
                    self.candidate_ending_is_from_config = saved;
                    if result.is_some() {
                        return result;
                    }
                }
            }
        }
        CONTINUE_SEARCHING
    }

    fn try_load_module_using_root_dirs(&mut self) -> Option<Resolved> {
        if self.compiler_options.root_dirs.is_empty() {
            return CONTINUE_SEARCHING;
        }
        let candidate = tspath::normalize_path(&tspath::combine_paths(
            &self.containing_directory,
            &[&self.name],
        ));

        let mut matched_normalized_prefix = String::new();
        for root_dir in &self.compiler_options.root_dirs {
            let mut normalized_root = tspath::normalize_path(root_dir);
            if !normalized_root.ends_with('/') {
                normalized_root.push('/');
            }
            if candidate.starts_with(&normalized_root)
                && matched_normalized_prefix.len() < normalized_root.len()
            {
                matched_normalized_prefix = normalized_root;
            }
        }
        if matched_normalized_prefix.is_empty() {
            return CONTINUE_SEARCHING;
        }
        let suffix = &candidate[matched_normalized_prefix.len()..];

        if let Some(r) = self.node_load_module_by_relative_name(self.extensions, &candidate, true) {
            return Some(r);
        }

        let matched_root_normalized = tspath::normalize_path(
            &matched_normalized_prefix[..matched_normalized_prefix.len().saturating_sub(1)],
        );
        for root_dir in &self.compiler_options.root_dirs {
            let normalized = tspath::normalize_path(root_dir);
            if normalized == matched_root_normalized {
                continue;
            }
            let alternate = tspath::normalize_path(&tspath::combine_paths(&normalized, &[suffix]));
            if let Some(r) =
                self.node_load_module_by_relative_name(self.extensions, &alternate, true)
            {
                return Some(r);
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn resolve_node_like(mut self) -> ResolvedModule {
        let result = self.resolve_node_like_worker();
        if result.is_none() {

            if !tspath::is_external_module_name_relative(&self.name)
                && !self.features.contains(NodeResolutionFeatures::Exports)
                && self
                    .extensions
                    .intersects(Extensions::TYPESCRIPT | Extensions::DECLARATION)
            {
                self.features |= NodeResolutionFeatures::ALL;
                if let Some(alt) = self.resolve_node_like_worker() {
                    return ResolvedModule {
                        alternate_result: Some(alt.path),
                        ..Default::default()
                    };
                }
            }
            return self.create_resolved_module(None);
        }
        self.create_resolved_module(result)
    }

    fn resolve_node_like_worker(&mut self) -> Option<Resolved> {

        if let Some(resolved) = self.try_load_module_using_optional_resolution_settings() {
            return Some(resolved);
        }

        if !tspath::is_external_module_name_relative(&self.name)
            && self.name.starts_with('#')
            && self.features.contains(NodeResolutionFeatures::Imports)
        {
            if let Some(resolved) = self.load_module_from_imports() {
                return Some(resolved);
            }
        }

        if !tspath::is_external_module_name_relative(&self.name)
            && self.features.contains(NodeResolutionFeatures::SelfName)
        {
            if let Some(resolved) = self.load_module_from_self_name_reference() {
                return Some(resolved);
            }
        }
        if tspath::is_external_module_name_relative(&self.name) {

            let candidate =
                Self::normalize_path_for_cjs_resolution(&self.containing_directory, &self.name);
            return self.node_load_module_by_relative_name(self.extensions, &candidate, true);
        }

        if let Some(resolved) = self.load_module_from_nearest_node_modules_directory(false) {
            return Some(resolved);
        }

        if self.extensions.contains(Extensions::DECLARATION) {
            if let Some(resolved) = self.resolve_from_type_root() {
                return Some(resolved);
            }
        }
        CONTINUE_SEARCHING
    }

    fn resolve_from_type_root(&mut self) -> Option<Resolved> {
        let (type_roots, _) =
            get_effective_type_roots(self.compiler_options, self.current_directory);
        for type_root in &type_roots {
            if !self.fs.directory_exists(type_root) {
                continue;
            }
            let package_directory = tspath::combine_paths(type_root, &[&self.name]);
            if self.fs.directory_exists(&package_directory) {
                let result = self.load_node_module_from_directory_worker(
                    Extensions::DECLARATION,
                    &package_directory,
                    false,
                );
                if result.is_some() {
                    return result;
                }
            }
        }
        CONTINUE_SEARCHING
    }

    fn resolve_type_reference_directive(
        &mut self,
        type_roots: &[String],
        from_config: bool,
        from_inferred_types_containing_file: bool,
    ) -> ResolvedTypeReferenceDirective {

        if !type_roots.is_empty() {
            for type_root in type_roots {
                if !self.fs.directory_exists(type_root) {
                    continue;
                }
                let candidate = self.get_candidate_from_type_root(type_root);
                if from_config {

                    if let Some(resolved) =
                        self.load_module_from_file(Extensions::DECLARATION, &candidate)
                    {
                        return self.create_resolved_type_ref(Some(resolved), true);
                    }
                }
                if let Some(resolved) =
                    self.load_node_module_from_directory(Extensions::DECLARATION, &candidate, true)
                {
                    return self.create_resolved_type_ref(Some(resolved), true);
                }
            }
        }

        if !from_config || !from_inferred_types_containing_file {
            let resolved = if tspath::is_external_module_name_relative(&self.name) {
                let candidate =
                    Self::normalize_path_for_cjs_resolution(&self.containing_directory, &self.name);
                self.node_load_module_by_relative_name(Extensions::DECLARATION, &candidate, true)
            } else {
                self.load_module_from_nearest_node_modules_directory(false)
            };
            return self.create_resolved_type_ref(resolved, false);
        }

        ResolvedTypeReferenceDirective::default()
    }

    fn get_candidate_from_type_root(&self, type_root: &str) -> String {
        let name_for_lookup = if type_root.ends_with("/node_modules/@types")
            || type_root.ends_with("/node_modules/@types/")
        {
            super::mangle_scoped_package_name(&self.name)
        } else {
            self.name.clone()
        };
        tspath::combine_paths(type_root, &[&name_for_lookup])
    }

    fn create_resolved_type_ref(
        &self,
        resolved: Option<Resolved>,
        primary: bool,
    ) -> ResolvedTypeReferenceDirective {
        match resolved {
            Some(r) if !r.path.is_empty() => {
                let is_external = r.path.contains("/node_modules/");
                ResolvedTypeReferenceDirective {
                    resolved_file_name: r.path,
                    primary,
                    package_id: r.package_id,
                    is_external_library_import: is_external,
                    ..Default::default()
                }
            }
            _ => ResolvedTypeReferenceDirective::default(),
        }
    }

    fn load_module_from_nearest_node_modules_directory(
        &mut self,
        types_scope_only: bool,
    ) -> Option<Resolved> {

        let ts_ext = self
            .extensions
            .intersection(Extensions::TYPESCRIPT | Extensions::DECLARATION);
        if !ts_ext.is_empty() {
            if let Some(resolved) = self
                .load_module_from_nearest_node_modules_directory_worker(ts_ext, types_scope_only)
            {
                return Some(resolved);
            }
        }

        let js_ext = self
            .extensions
            .difference(Extensions::TYPESCRIPT | Extensions::DECLARATION);
        if !js_ext.is_empty() {
            if let Some(resolved) = self
                .load_module_from_nearest_node_modules_directory_worker(js_ext, types_scope_only)
            {
                return Some(resolved);
            }
        }
        CONTINUE_SEARCHING
    }

    fn load_module_from_nearest_node_modules_directory_worker(
        &mut self,
        ext: Extensions,
        types_scope_only: bool,
    ) -> Option<Resolved> {
        let mut directory = self.containing_directory.clone();
        loop {
            if tspath::get_base_file_name(&directory) != "node_modules" {
                if let Some(resolved) = self.load_module_from_immediate_node_modules_directory(
                    ext,
                    &directory,
                    types_scope_only,
                ) {
                    return Some(resolved);
                }
            }
            let parent = tspath::get_directory_path(&directory);
            if parent == directory {
                break;
            }
            directory = parent;
        }
        CONTINUE_SEARCHING
    }

    fn load_module_from_immediate_node_modules_directory(
        &mut self,
        ext: Extensions,
        directory: &str,
        types_scope_only: bool,
    ) -> Option<Resolved> {
        let node_modules_folder = tspath::combine_paths(directory, &["node_modules"]);
        if !self.fs.directory_exists(&node_modules_folder) {
            return CONTINUE_SEARCHING;
        }
        if !types_scope_only {
            let name = self.name.clone();
            if let Some(resolved) = self.load_module_from_specific_node_modules_directory(
                ext,
                &name,
                &node_modules_folder,
            ) {
                return Some(resolved);
            }
        }

        if ext.contains(Extensions::DECLARATION) {
            let node_modules_at_types = tspath::combine_paths(&node_modules_folder, &["@types"]);
            if self.fs.directory_exists(&node_modules_at_types) {
                let mangled = mangle_scoped_package_name(&self.name);
                if let Some(resolved) = self.load_module_from_specific_node_modules_directory(
                    Extensions::DECLARATION,
                    &mangled,
                    &node_modules_at_types,
                ) {
                    return Some(resolved);
                }
            }
        }
        CONTINUE_SEARCHING
    }

    fn load_module_from_specific_node_modules_directory(
        &mut self,
        ext: Extensions,
        module_name: &str,
        node_modules_directory: &str,
    ) -> Option<Resolved> {
        let candidate = tspath::normalize_path(&tspath::combine_paths(
            node_modules_directory,
            &[module_name],
        ));
        let (package_name, rest) = parse_package_name(module_name);
        let package_directory = tspath::combine_paths(node_modules_directory, &[&package_name]);

        let pkg_json_path = tspath::combine_paths(&package_directory, &["package.json"]);
        let package_info_exists = self.fs.file_exists(&pkg_json_path);
        if package_info_exists {
            self.resolved_package_directory = true;
        }

        if self.features.contains(NodeResolutionFeatures::Exports) && package_info_exists {
            if let Some(content) = self.fs.read_file(&pkg_json_path) {
                if let Ok(fields) = packagejson::parse(&content) {
                    let exports = &fields.path_fields.exports;
                    if exports.json_value.is_present() && !exports.json_value.is_falsy() {
                        let subpath = if rest.is_empty() {
                            ".".to_string()
                        } else {
                            format!("./{}", rest)
                        };
                        if let Some(resolved) = self.load_module_from_exports(
                            ext,
                            &subpath,
                            &package_directory,
                            exports,
                        ) {
                            return Some(resolved);
                        }
                    }
                }
            }
        }

        if !rest.is_empty() {
            if let Some(resolved) = self.load_module_from_file(ext, &candidate) {
                return Some(resolved);
            }
            return self.load_node_module_from_directory(ext, &candidate, true);
        }

        let has_exports = self
            .get_package_file(ext, &candidate)
            .map(|(_, f)| f.path_fields.exports.json_value.is_present())
            .unwrap_or(false);

        if !self.esm_mode {
            if let Some(resolved) = self.load_module_from_file(ext, &candidate) {
                return Some(resolved);
            }
        }

        if let Some(resolved) = self.load_node_module_from_directory(ext, &candidate, true) {
            return Some(resolved);
        }

        if package_info_exists && !has_exports && self.esm_mode {
            let index = tspath::combine_paths(&candidate, &["index.js"]);
            if let Some(resolved) = self.load_module_from_file(ext, &index) {
                return Some(resolved);
            }
        }
        CONTINUE_SEARCHING
    }

    fn load_node_module_from_directory(
        &mut self,
        ext: Extensions,
        candidate: &str,
        consider_package_dir: bool,
    ) -> Option<Resolved> {
        self.load_node_module_from_directory_worker(ext, candidate, consider_package_dir)
    }

    fn load_node_module_from_directory_worker(
        &mut self,
        ext: Extensions,
        candidate: &str,
        _consider_package_dir: bool,
    ) -> Option<Resolved> {
        let pkg_json_path = tspath::combine_paths(candidate, &["package.json"]);
        let package_info_exists = self.fs.file_exists(&pkg_json_path);

        if package_info_exists {

            if let Some(resolved) =
                self.try_load_module_using_package_json_type_versions(ext, candidate)
            {
                return Some(resolved);
            }
            if let Some((package_file, _)) = self.get_package_file(ext, candidate) {
                if let Some(resolved) =
                    self.load_file_name_from_package_json_field(ext, &package_file)
                {
                    return Some(resolved);
                }
            }
        }

        if !self.esm_mode {
            let index = tspath::combine_paths(candidate, &["index"]);
            if let Some(resolved) = self.load_module_from_file(ext, &index) {
                return Some(resolved);
            }
        }
        CONTINUE_SEARCHING
    }

    fn try_load_module_using_package_json_type_versions(
        &mut self,
        ext: Extensions,
        candidate: &str,
    ) -> Option<Resolved> {
        let pkg_json_path = tspath::combine_paths(candidate, &["package.json"]);
        let content = self.fs.read_file(&pkg_json_path)?;
        let fields = packagejson::parse(&content).ok()?;
        let tv = &fields.path_fields.types_versions;
        if !tv.is_present() || tv.value_type != packagejson::JsonValueType::Object {
            return CONTINUE_SEARCHING;
        }

        let (_, version_mapping) = tv.as_object().first()?;
        if version_mapping.value_type != packagejson::JsonValueType::Object {
            return CONTINUE_SEARCHING;
        }

        let mut paths_map: HashMap<String, Vec<String>> = HashMap::new();
        for (pattern, targets) in version_mapping.as_object() {
            if targets.value_type == packagejson::JsonValueType::Array {
                let target_strings: Vec<String> = targets
                    .as_array()
                    .iter()
                    .filter(|t| t.value_type == packagejson::JsonValueType::String)
                    .map(|t| t.as_string().to_string())
                    .collect();
                if !target_strings.is_empty() {
                    paths_map.insert(pattern.clone(), target_strings);
                }
            }
        }
        if paths_map.is_empty() {
            return CONTINUE_SEARCHING;
        }
        let parsed = try_parse_patterns(&paths_map);

        let (_, rest) = parse_package_name(&self.name);
        self.try_load_module_using_paths(ext, &rest, candidate, &paths_map, &parsed)
    }

    fn get_package_file(
        &self,
        ext: Extensions,
        candidate: &str,
    ) -> Option<(String, packagejson::Fields)> {
        let pkg_json_path = tspath::combine_paths(candidate, &["package.json"]);
        if !self.fs.file_exists(&pkg_json_path) {
            return None;
        }
        let content = self.fs.read_file(&pkg_json_path)?;
        let fields = packagejson::parse(&content).ok()?;

        if ext.contains(Extensions::DECLARATION) {
            if let Some(typings) = fields.path_fields.typings.get_value() {
                let path =
                    tspath::normalize_path(&tspath::combine_paths(candidate, &[typings.as_str()]));
                return Some((path, fields));
            }
            if let Some(types) = fields.path_fields.types.get_value() {
                let path =
                    tspath::normalize_path(&tspath::combine_paths(candidate, &[types.as_str()]));
                return Some((path, fields));
            }
        }
        if ext.intersects(Extensions::IMPLEMENTATION_FILES | Extensions::DECLARATION) {
            if let Some(main) = fields.path_fields.main.get_value() {
                let path =
                    tspath::normalize_path(&tspath::combine_paths(candidate, &[main.as_str()]));
                return Some((path, fields));
            }
        }
        None
    }

    fn load_file_name_from_package_json_field(
        &self,
        ext: Extensions,
        package_file: &str,
    ) -> Option<Resolved> {
        let extension = tspath::try_get_extension_from_path(package_file);
        if tspath::extension_is_ts(extension)
            && ext.intersects(Extensions::TYPESCRIPT | Extensions::DECLARATION)
        {
            if let Some(path) = self.try_file(package_file) {
                return Some(Resolved {
                    path,
                    extension: extension.to_string(),
                    resolved_using_ts_extension: true,
                    ..Default::default()
                });
            }
            return CONTINUE_SEARCHING;
        }
        self.load_module_from_file_no_implicit_extensions(ext, package_file)
    }

    fn condition_matches(&self, condition: &str) -> bool {
        if condition == "default" || self.conditions.iter().any(|c| c == condition) {
            return true;
        }

        if !self.conditions.iter().any(|c| c == "types") {
            return false;
        }
        false
    }

    fn load_module_from_exports(
        &mut self,
        ext: Extensions,
        subpath: &str,
        package_directory: &str,
        exports: &packagejson::ExportsOrImports,
    ) -> Option<Resolved> {
        if !exports.json_value.is_present() || exports.json_value.is_falsy() {
            return CONTINUE_SEARCHING;
        }

        if subpath == "." {

            match exports.json_value.value_type {
                packagejson::JsonValueType::String | packagejson::JsonValueType::Array => {
                    return self.load_module_from_target_export_or_import(
                        ext,
                        subpath,
                        package_directory,
                        false,
                        &exports.json_value,
                        "",
                        false,
                    );
                }
                packagejson::JsonValueType::Object => {
                    if exports.is_conditions() {
                        return self.load_module_from_target_export_or_import(
                            ext,
                            subpath,
                            package_directory,
                            false,
                            &exports.json_value,
                            "",
                            false,
                        );
                    }
                    if let Some(dot) = exports.json_value.get(".") {
                        return self.load_module_from_target_export_or_import(
                            ext,
                            subpath,
                            package_directory,
                            false,
                            dot,
                            "",
                            false,
                        );
                    }
                }
                _ => {}
            }
        } else if exports.json_value.value_type == packagejson::JsonValueType::Object
            && exports.is_subpaths()
        {

            return self.load_module_from_exports_or_imports(
                ext,
                subpath,
                &exports.json_value,
                package_directory,
                false,
            );
        }
        CONTINUE_SEARCHING
    }

    fn load_module_from_imports(&mut self) -> Option<Resolved> {

        if self.name == "#" {
            return CONTINUE_SEARCHING;
        }

        if self.name.starts_with("#/")
            && !self
                .features
                .contains(NodeResolutionFeatures::ImportsPatternRoot)
        {
            return CONTINUE_SEARCHING;
        }

        let directory_path = tspath::get_normalized_absolute_path(
            &self.containing_directory,
            self.current_directory,
        );
        let (package_directory, fields) = match self.get_package_scope_for_path(&directory_path) {
            Some(s) => s,
            None => return CONTINUE_SEARCHING,
        };

        let imports = &fields.path_fields.imports;
        if !imports.json_value.is_present()
            || imports.json_value.value_type != packagejson::JsonValueType::Object
        {
            return CONTINUE_SEARCHING;
        }

        let name = self.name.clone();
        self.load_module_from_exports_or_imports(
            self.extensions,
            &name,
            &imports.json_value,
            &package_directory,
            true,
        )
    }

    fn load_module_from_self_name_reference(&mut self) -> Option<Resolved> {
        let directory_path = tspath::get_normalized_absolute_path(
            &self.containing_directory,
            self.current_directory,
        );
        let (package_directory, fields) = self.get_package_scope_for_path(&directory_path)?;
        let exports = &fields.path_fields.exports;
        if !exports.json_value.is_present() || exports.json_value.is_falsy() {
            return CONTINUE_SEARCHING;
        }
        let Some(package_name) = fields.header_fields.name.get_value() else {
            return CONTINUE_SEARCHING;
        };

        let parts: Vec<&str> = self.name.split('/').filter(|p| !p.is_empty()).collect();
        let name_parts: Vec<&str> = package_name
            .split('/')
            .filter(|p| !p.is_empty())
            .collect();
        if parts.len() < name_parts.len() || parts[..name_parts.len()] != name_parts[..] {
            return CONTINUE_SEARCHING;
        }
        let trailing = &parts[name_parts.len()..];
        let subpath = if trailing.is_empty() {
            ".".to_string()
        } else {
            format!("./{}", trailing.join("/"))
        };
        self.load_module_from_exports(self.extensions, &subpath, &package_directory, exports)
    }

    fn get_package_scope_for_path(&self, directory: &str) -> Option<(String, packagejson::Fields)> {
        let mut dir = directory.to_string();
        loop {
            let pkg_json_path = tspath::combine_paths(&dir, &["package.json"]);
            if self.fs.file_exists(&pkg_json_path) {
                if let Some(content) = self.fs.read_file(&pkg_json_path) {
                    if let Ok(fields) = packagejson::parse(&content) {
                        return Some((dir, fields));
                    }
                }
            }
            let parent = tspath::get_directory_path(&dir);
            if parent == dir {
                break;
            }
            dir = parent;
        }
        None
    }

    fn load_module_from_exports_or_imports(
        &mut self,
        ext: Extensions,
        module_name: &str,
        lookup_table: &packagejson::JsonValue,
        package_directory: &str,
        is_imports: bool,
    ) -> Option<Resolved> {
        let entries = lookup_table.as_object();

        if !module_name.ends_with('/') && !module_name.contains('*') {
            for (key, value) in entries {
                if key == module_name {
                    return self.load_module_from_target_export_or_import(
                        ext,
                        module_name,
                        package_directory,
                        is_imports,
                        value,
                        "",
                        false,
                    );
                }
            }
        }

        let mut expanding_keys: Vec<(&String, &packagejson::JsonValue)> = entries
            .iter()
            .filter(|(k, _)| k.matches('*').count() == 1 || k.ends_with('/'))
            .map(|(k, v)| (k, v))
            .collect();
        expanding_keys.sort_by(|(a, _), (b, _)| super::compare_pattern_keys(a, b));

        for (potential_target, target) in expanding_keys {
            if potential_target.contains('*') {
                let star_pos = potential_target.find('*').unwrap();
                let prefix = &potential_target[..star_pos];
                let suffix = &potential_target[star_pos + 1..];
                if !suffix.is_empty() {

                    if module_name.starts_with(prefix)
                        && module_name.ends_with(suffix)
                        && module_name.len() >= prefix.len() + suffix.len()
                    {
                        let subpath = &module_name[prefix.len()..module_name.len() - suffix.len()];
                        return self.load_module_from_target_export_or_import(
                            ext,
                            module_name,
                            package_directory,
                            is_imports,
                            target,
                            subpath,
                            true,
                        );
                    }
                } else if module_name.starts_with(prefix) {

                    let subpath = &module_name[prefix.len()..];
                    return self.load_module_from_target_export_or_import(
                        ext,
                        module_name,
                        package_directory,
                        is_imports,
                        target,
                        subpath,
                        true,
                    );
                }
            } else if potential_target.ends_with('/')
                && module_name.starts_with(potential_target.as_str())
            {

                let subpath = &module_name[potential_target.len()..];
                return self.load_module_from_target_export_or_import(
                    ext,
                    module_name,
                    package_directory,
                    is_imports,
                    target,
                    subpath,
                    false,
                );
            }
        }
        CONTINUE_SEARCHING
    }

    fn load_module_from_target_export_or_import(
        &mut self,
        ext: Extensions,
        module_name: &str,
        package_directory: &str,
        is_imports: bool,
        target: &packagejson::JsonValue,
        subpath: &str,
        is_pattern: bool,
    ) -> Option<Resolved> {

        if self.export_target_depth >= 16 {
            return CONTINUE_SEARCHING;
        }
        match target.value_type {
            packagejson::JsonValueType::String => {
                let target_string = target.as_string();

                if !is_pattern && !subpath.is_empty() && !target_string.ends_with('/') {
                    return CONTINUE_SEARCHING;
                }

                if !is_imports && !target_string.starts_with("./") {
                    return CONTINUE_SEARCHING;
                }

                let parts: Vec<&str> = target_string.split('/').collect();
                if parts
                    .iter()
                    .skip(1)
                    .any(|p| *p == ".." || *p == "node_modules")
                {
                    return CONTINUE_SEARCHING;
                }

                let final_path = if is_pattern {
                    let resolved_target = target_string.replacen('*', subpath, 1);
                    let combined = tspath::combine_paths(package_directory, &[&resolved_target]);
                    tspath::normalize_path(&combined)
                } else if subpath.is_empty() {
                    let combined = tspath::combine_paths(package_directory, &[target_string]);
                    tspath::normalize_path(&combined)
                } else {
                    let combined = tspath::combine_paths(package_directory, &[target_string]);
                    let combined = tspath::combine_paths(&combined, &[subpath]);
                    tspath::normalize_path(&combined)
                };

                self.load_file_name_from_package_json_field(ext, &final_path)
            }

            packagejson::JsonValueType::Object => {

                for (condition, sub_target) in target.as_object() {
                    if self.condition_matches(condition) {
                        self.export_target_depth += 1;
                        let result = self.load_module_from_target_export_or_import(
                            ext,
                            module_name,
                            package_directory,
                            is_imports,
                            sub_target,
                            subpath,
                            is_pattern,
                        );
                        self.export_target_depth -= 1;
                        if let Some(result) = result {
                            return Some(result);
                        }
                    }
                }
                CONTINUE_SEARCHING
            }

            packagejson::JsonValueType::Array => {

                for elem in target.as_array() {
                    self.export_target_depth += 1;
                    let result = self.load_module_from_target_export_or_import(
                        ext,
                        module_name,
                        package_directory,
                        is_imports,
                        elem,
                        subpath,
                        is_pattern,
                    );
                    self.export_target_depth -= 1;
                    if let Some(result) = result {
                        return Some(result);
                    }
                }
                CONTINUE_SEARCHING
            }

            _ => CONTINUE_SEARCHING,
        }
    }

    fn create_resolved_module(&self, resolved: Option<Resolved>) -> ResolvedModule {
        match resolved {
            Some(r) => {
                let is_external = r.path.contains("/node_modules/");
                ResolvedModule {
                    resolved_file_name: r.path,
                    original_path: r.original_path,
                    extension: r.extension,
                    resolved_using_ts_extension: r.resolved_using_ts_extension,
                    is_external_library_import: is_external,
                    package_id: r.package_id,
                    ..Default::default()
                }
            }
            None => ResolvedModule::default(),
        }
    }
}

fn get_conditions(options: &CompilerOptions, resolution_mode: ModuleKind) -> Vec<String> {

    let mut conditions = Vec::new();
    if resolution_mode == ModuleKind::ESNext {
        conditions.push("import".to_string());
    } else {
        conditions.push("require".to_string());
    }
    if !options.no_dts_resolution.is_true() {
        conditions.push("types".to_string());
    }
    if options.get_module_resolution_kind() != ModuleResolutionKind::Bundler {
        conditions.push("node".to_string());
    }

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
        cache.set(key.clone(), mod2);
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
        cache.set(key.clone(), dir2);
        let result = cache.get(&key).unwrap();
        assert_eq!(result.resolved_file_name, "/foo/node2.d.ts");
    }

    #[test]
    fn effective_type_roots_default() {
        let opts = CompilerOptions::default();
        let (roots, from_config) = get_effective_type_roots(&opts, "/project/sub");
        assert!(!from_config);

        assert_eq!(roots.len(), 3);
        assert!(roots[0].contains("sub/node_modules/@types"));
        assert!(roots[1].contains("project/node_modules/@types"));

        assert_eq!(roots[2], "/node_modules/@types");
    }

    #[test]
    fn effective_type_roots_explicit() {
        let mut opts = CompilerOptions::default();
        opts.type_roots = vec!["./custom-types".to_string()];
        let (roots, from_config) = get_effective_type_roots(&opts, "/project");
        assert!(from_config);
        assert_eq!(roots, vec!["./custom-types".to_string()]);
    }

    #[test]
    fn effective_type_roots_base_on_config_file() {

        let mut opts = CompilerOptions::default();
        opts.config_file_path = "/foo/bar/tsconfig.json".to_string();
        let (roots, from_config) = get_effective_type_roots(&opts, "/src");
        assert!(!from_config);
        assert_eq!(roots.len(), 3);
        assert_eq!(roots[0], "/foo/bar/node_modules/@types");
        assert_eq!(roots[1], "/foo/node_modules/@types");
        assert_eq!(roots[2], "/node_modules/@types");
    }

    fn make_state<'a>(
        name: &str,
        containing_dir: &str,
        opts: &'a CompilerOptions,
        fs: &'a dyn FS,
    ) -> ResolutionState<'a> {
        ResolutionState::new(
            name,
            containing_dir,
            false,
            ModuleKind::None,
            opts,
            fs,
            "/",
        )
    }

    const REL_EXTS: Extensions = Extensions::TYPESCRIPT
        .union(Extensions::JAVASCRIPT)
        .union(Extensions::DECLARATION);

    #[test]
    fn resolve_relative_ts_file() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.write_file("/src/foo.ts", "export const x = 1;").unwrap();

        let opts = CompilerOptions::default();
        let mut state = make_state("./foo", "/src", &opts, &fs);
        let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo");
        let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
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
        fs.write_file("/src/component.tsx", "export const C = 1;")
            .unwrap();

        let opts = CompilerOptions::default();
        let mut state = make_state("./component", "/src", &opts, &fs);
        let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./component");
        let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().extension, ".tsx");
    }

    #[test]
    fn resolve_relative_js_specifier_swaps_to_ts() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");

        fs.write_file("/src/foo.ts", "export const x = 1;").unwrap();

        let opts = CompilerOptions::default();
        let mut state = make_state("./foo.js", "/src", &opts, &fs);
        let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo.js");
        let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/src/foo.ts");
    }

    #[test]
    fn resolve_relative_mjs_specifier_swaps_to_mts() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.write_file("/src/foo.mts", "export const x = 1;")
            .unwrap();

        let opts = CompilerOptions::default();
        let mut state = make_state("./foo.mjs", "/src", &opts, &fs);
        let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./foo.mjs");
        let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/src/foo.mts");
    }

    #[test]
    fn resolve_relative_nonexistent_returns_none() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");

        let opts = CompilerOptions::default();
        let mut state = make_state("./missing", "/src", &opts, &fs);
        let candidate = ResolutionState::normalize_path_for_cjs_resolution("/src", "./missing");
        let result = state.node_load_module_by_relative_name(REL_EXTS, &candidate, true);
        assert!(result.is_none());
    }

    #[test]
    fn exports_target_nesting_bounded() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/node_modules/pkg");
        fs.write_file("/node_modules/pkg/index.ts", "export const x = 1;")
            .unwrap();

        let opts = CompilerOptions::default();

        let shallow = r#"{"name": "pkg", "exports": {"default": {"default": "./index.ts"}}}"#;
        let fields = packagejson::parse(shallow).unwrap();
        let mut state = make_state("pkg", "/src", &opts, &fs);
        let resolved = state.load_module_from_exports(
            REL_EXTS,
            ".",
            "/node_modules/pkg",
            &fields.path_fields.exports,
        );
        assert_eq!(resolved.unwrap().path, "/node_modules/pkg/index.ts");

        let mut target = r#""./index.ts""#.to_string();
        for _ in 0..30 {
            target = format!(r#"{{"default": {target}}}"#);
        }
        let deep = format!(r#"{{"name": "pkg", "exports": {target}}}"#);
        let fields = packagejson::parse(&deep).unwrap();
        let mut state = make_state("pkg", "/src", &opts, &fs);
        let result = state.load_module_from_exports(
            REL_EXTS,
            ".",
            "/node_modules/pkg",
            &fields.path_fields.exports,
        );
        assert!(result.is_none(), "deeply nested exports must stop at the cap");
    }

    #[test]
    fn resolve_relative_parent_dir_not_exists() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();

        let opts = CompilerOptions::default();
        let mut state = make_state("./foo", "/nonexistent", &opts, &fs);
        let candidate = ResolutionState::normalize_path_for_cjs_resolution("/nonexistent", "./foo");
        let result =
            state.node_load_module_by_relative_name(Extensions::TYPESCRIPT, &candidate, true);
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

    #[test]
    fn resolve_bare_specifier_node_modules() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/foo");
        fs.write_file("/src/node_modules/foo/index.ts", "export const x = 1;")
            .unwrap();

        let opts = CompilerOptions::default();
        let state = make_state("foo", "/src", &opts, &fs);
        let result = state.resolve_node_like();
        assert!(result.is_resolved());
        assert_eq!(result.resolved_file_name, "/src/node_modules/foo/index.ts");
        assert!(result.is_external_library_import);
    }

    #[test]
    fn resolve_bare_specifier_with_types() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/foo");
        fs.insert_dir("/src/node_modules/foo/dist");
        fs.write_file(
            "/src/node_modules/foo/package.json",
            r#"{"name": "foo", "types": "./dist/index.d.ts"}"#,
        )
        .unwrap();
        fs.write_file(
            "/src/node_modules/foo/dist/index.d.ts",
            "export const x = 1;",
        )
        .unwrap();

        let opts = CompilerOptions::default();
        let state = make_state("foo", "/src", &opts, &fs);
        let result = state.resolve_node_like();
        assert!(result.is_resolved());
        assert_eq!(
            result.resolved_file_name,
            "/src/node_modules/foo/dist/index.d.ts"
        );
    }

    #[test]
    fn resolve_bare_specifier_with_main() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/foo");
        fs.insert_dir("/src/node_modules/foo/lib");
        fs.write_file(
            "/src/node_modules/foo/package.json",
            r#"{"name": "foo", "main": "./lib/index.js"}"#,
        )
        .unwrap();
        fs.write_file("/src/node_modules/foo/lib/index.js", "exports.x = 1;")
            .unwrap();

        let opts = CompilerOptions::default();
        let state = make_state("foo", "/src", &opts, &fs);
        let result = state.resolve_node_like();
        assert!(result.is_resolved());
        assert_eq!(
            result.resolved_file_name,
            "/src/node_modules/foo/lib/index.js"
        );
    }

    #[test]
    fn resolve_bare_specifier_ancestor_node_modules() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/sub");
        fs.insert_dir("/node_modules");
        fs.insert_dir("/node_modules/foo");
        fs.write_file("/node_modules/foo/index.ts", "export const x = 1;")
            .unwrap();

        let opts = CompilerOptions::default();
        let state = make_state("foo", "/src/sub", &opts, &fs);
        let result = state.resolve_node_like();
        assert!(result.is_resolved());
        assert_eq!(result.resolved_file_name, "/node_modules/foo/index.ts");
    }

    #[test]
    fn resolve_bare_specifier_not_found() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");

        let opts = CompilerOptions::default();
        let state = make_state("nonexistent", "/src", &opts, &fs);
        let result = state.resolve_node_like();
        assert!(!result.is_resolved());
        assert!(result.resolved_file_name.is_empty());
    }

    #[test]
    fn node16_conditions_follow_resolution_mode() {

        let mut opts = CompilerOptions::default();
        opts.module_resolution = ModuleResolutionKind::Node16;
        let require = get_conditions(&opts, ModuleKind::CommonJS);
        assert!(require.contains(&"require".to_string()));
        assert!(!require.contains(&"import".to_string()));
        let import = get_conditions(&opts, ModuleKind::ESNext);
        assert!(import.contains(&"import".to_string()));
        assert!(!import.contains(&"require".to_string()));

        for c in [&require, &import] {
            assert!(c.contains(&"node".to_string()));
            assert!(c.contains(&"types".to_string()));
        }
    }

    #[test]
    fn node16_exports_condition_by_file_format() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        for d in ["/proj", "/proj/sub", "/proj/node_modules/pkg"] {
            fs.insert_dir(d);
        }
        fs.insert_file(
            "/proj/package.json",
            r#"{"name": "root", "type": "module"}"#,
        );
        fs.insert_file("/proj/sub/package.json", r#"{"type": "commonjs"}"#);
        fs.insert_file(
            "/proj/node_modules/pkg/package.json",
            r#"{"name": "pkg", "exports": {"import": "./import.js", "require": "./require.js"}}"#,
        );
        fs.insert_file("/proj/node_modules/pkg/import.d.ts", "export {};\n");
        fs.insert_file("/proj/node_modules/pkg/require.d.ts", "export {};\n");
        fs.insert_file("/proj/index.ts", "import \"pkg\";\n");
        fs.insert_file("/proj/sub/index.ts", "import \"pkg\";\n");

        let mut opts = CompilerOptions::default();
        opts.module_resolution = ModuleResolutionKind::Node16;

        assert_eq!(
            default_resolution_mode(ModuleKind::None, &opts, "/proj/index.ts", &fs),
            ModuleKind::ESNext
        );
        assert_eq!(
            default_resolution_mode(ModuleKind::None, &opts, "/proj/sub/index.ts", &fs),
            ModuleKind::CommonJS
        );

        let esm = ResolutionState::new(
            "pkg",
            "/proj",
            false,
            ModuleKind::ESNext,
            &opts,
            &fs,
            "/proj",
        );
        let r = esm.resolve_node_like();
        assert!(r.is_resolved(), "esm resolve");
        assert!(r.resolved_file_name.ends_with("import.d.ts"), "{}", r.resolved_file_name);

        let cjs = ResolutionState::new(
            "pkg",
            "/proj/sub",
            false,
            ModuleKind::CommonJS,
            &opts,
            &fs,
            "/proj",
        );
        let r = cjs.resolve_node_like();
        assert!(r.is_resolved(), "cjs resolve");
        assert!(r.resolved_file_name.ends_with("require.d.ts"), "{}", r.resolved_file_name);
    }

    #[test]
    fn implied_format_from_package_json_chain() {
        use crate::core::compiler_options::ModuleKind;
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        for d in ["/a/b", "/a/node_modules"] {
            fs.insert_dir(d);
        }
        fs.insert_file("/a/package.json", r#"{"type": "module"}"#);
        let read = |p: &str| fs.read_file(p);
        assert_eq!(
            crate::compiler::implied_node_format_of_file("/a/b/x.ts", &read),
            ModuleKind::ESNext
        );
        assert_eq!(
            crate::compiler::implied_node_format_of_file("/a/b/x.mts", &read),
            ModuleKind::ESNext
        );
        assert_eq!(
            crate::compiler::implied_node_format_of_file("/a/b/x.cts", &read),
            ModuleKind::CommonJS
        );

        assert_eq!(
            crate::compiler::implied_node_format_of_file("/a/node_modules/x.ts", &read),
            ModuleKind::ESNext
        );
    }

    #[test]
    fn resolve_types_fallback() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/@types");
        fs.insert_dir("/src/node_modules/@types/foo");
        fs.write_file(
            "/src/node_modules/@types/foo/index.d.ts",
            "declare const x: number;",
        )
        .unwrap();

        let opts = CompilerOptions::default();
        let state = make_state("foo", "/src", &opts, &fs);
        let result = state.resolve_node_like();
        assert!(result.is_resolved());
        assert_eq!(
            result.resolved_file_name,
            "/src/node_modules/@types/foo/index.d.ts"
        );
    }

    #[test]
    fn resolve_paths_exact_match() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/mapped");
        fs.write_file("/src/mapped/foo.ts", "export const x = 1;")
            .unwrap();

        let mut opts = CompilerOptions::default();
        opts.paths = Some({
            let mut m = std::collections::HashMap::new();
            m.insert("foo".to_string(), vec!["./mapped/foo".to_string()]);
            m
        });
        opts.paths_base_path = "/src".to_string();

        let state =
            ResolutionState::new("foo", "/src", false, ModuleKind::None, &opts, &fs, "/src");
        let result = state.resolve_node_like();
        assert!(result.is_resolved());
        assert_eq!(result.resolved_file_name, "/src/mapped/foo.ts");
    }

    #[test]
    fn resolve_paths_wildcard() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/types");
        fs.write_file("/src/types/bar.ts", "export const x = 1;")
            .unwrap();

        let mut opts = CompilerOptions::default();
        opts.paths = Some({
            let mut m = std::collections::HashMap::new();
            m.insert("@mytypes/*".to_string(), vec!["./types/*".to_string()]);
            m
        });
        opts.paths_base_path = "/src".to_string();

        let state = ResolutionState::new(
            "@mytypes/bar",
            "/src",
            false,
            ModuleKind::None,
            &opts,
            &fs,
            "/src",
        );
        let result = state.resolve_node_like();
        assert!(result.is_resolved());
        assert_eq!(result.resolved_file_name, "/src/types/bar.ts");
    }

    #[test]
    fn resolve_paths_no_match_falls_through() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");

        let mut opts = CompilerOptions::default();
        opts.paths = Some({
            let mut m = std::collections::HashMap::new();
            m.insert("foo".to_string(), vec!["./mapped/foo".to_string()]);
            m
        });
        opts.paths_base_path = "/src".to_string();

        let state =
            ResolutionState::new("bar", "/src", false, ModuleKind::None, &opts, &fs, "/src");
        let result = state.resolve_node_like();
        assert!(!result.is_resolved());
    }

    #[test]
    fn resolve_root_dirs() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src/generated");
        fs.insert_dir("/src/manual");
        fs.write_file("/src/manual/shared.ts", "export const x = 1;")
            .unwrap();

        let mut opts = CompilerOptions::default();
        opts.root_dirs = vec!["/src/generated".to_string(), "/src/manual".to_string()];

        let state = ResolutionState::new(
            "./shared",
            "/src/generated",
            false,
            ModuleKind::None,
            &opts,
            &fs,
            "/src",
        );
        let result = state.resolve_node_like();
        assert!(result.is_resolved());
        assert_eq!(result.resolved_file_name, "/src/manual/shared.ts");
    }

    #[test]
    fn pattern_parsing() {
        let p = Pattern::try_parse("foo");
        assert_eq!(p.star_index, -1);
        assert!(p.is_valid());

        let p = Pattern::try_parse("foo/*");
        assert_eq!(p.star_index, 4);
        assert!(p.is_valid());
        assert!(p.matches("foo/bar"));
        assert!(!p.matches("baz/bar"));
        assert_eq!(p.matched_text("foo/bar"), "bar");

        let p = Pattern::try_parse("*");
        assert!(p.is_valid());

        let p = Pattern::try_parse("foo*bar*baz");
        assert!(!p.is_valid());
    }

    #[test]
    fn resolve_exports_string_main() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/mypkg");
        fs.insert_dir("/src/node_modules/mypkg/dist");
        fs.write_file(
            "/src/node_modules/mypkg/package.json",
            r#"{"name":"mypkg","exports":"./dist/index.js"}"#,
        )
        .unwrap();
        fs.write_file(
            "/src/node_modules/mypkg/dist/index.js",
            "export const x = 1;",
        )
        .unwrap();

        let opts = CompilerOptions::default();
        let state =
            ResolutionState::new("mypkg", "/src", false, ModuleKind::None, &opts, &fs, "/src");
        let result = state.resolve_node_like();
        assert!(
            result.is_resolved(),
            "expected resolved, got {:?}",
            result.resolved_file_name
        );
        assert_eq!(
            result.resolved_file_name,
            "/src/node_modules/mypkg/dist/index.js"
        );
    }

    #[test]
    fn resolve_exports_conditional_types() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/mypkg");
        fs.insert_dir("/src/node_modules/mypkg/dist");
        fs.write_file(
            "/src/node_modules/mypkg/package.json",
            r#"{"name":"mypkg","exports":{".":{"types":"./dist/index.d.ts","default":"./dist/index.js"}}}"#,
        )
        .unwrap();
        fs.write_file(
            "/src/node_modules/mypkg/dist/index.d.ts",
            "export declare const x: number;",
        )
        .unwrap();
        fs.write_file(
            "/src/node_modules/mypkg/dist/index.js",
            "export const x = 1;",
        )
        .unwrap();

        let opts = CompilerOptions::default();
        let state =
            ResolutionState::new("mypkg", "/src", false, ModuleKind::None, &opts, &fs, "/src");
        let result = state.resolve_node_like();

        assert!(
            result.is_resolved(),
            "expected resolved, got {:?}",
            result.resolved_file_name
        );
        assert_eq!(
            result.resolved_file_name,
            "/src/node_modules/mypkg/dist/index.d.ts"
        );
    }

    #[test]
    fn resolve_exports_subpath() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/mypkg");
        fs.insert_dir("/src/node_modules/mypkg/dist");
        fs.write_file(
            "/src/node_modules/mypkg/package.json",
            r#"{"name":"mypkg","exports":{"./feature":"./dist/feature.js"}}"#,
        )
        .unwrap();
        fs.write_file(
            "/src/node_modules/mypkg/dist/feature.js",
            "export const x = 1;",
        )
        .unwrap();

        let opts = CompilerOptions::default();
        let state = ResolutionState::new(
            "mypkg/feature",
            "/src",
            false,
            ModuleKind::None,
            &opts,
            &fs,
            "/src",
        );
        let result = state.resolve_node_like();
        assert!(
            result.is_resolved(),
            "expected resolved, got {:?}",
            result.resolved_file_name
        );
        assert_eq!(
            result.resolved_file_name,
            "/src/node_modules/mypkg/dist/feature.js"
        );
    }

    #[test]
    fn resolve_package_imports_exact() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/lib");
        fs.write_file(
            "/src/package.json",
            r##"{"name":"myapp","imports":{"#utils":"./lib/utils.js"}}"##,
        )
        .unwrap();
        fs.write_file("/src/lib/utils.js", "export const x = 1;")
            .unwrap();

        let opts = CompilerOptions::default();
        let state = ResolutionState::new(
            "#utils",
            "/src",
            false,
            ModuleKind::None,
            &opts,
            &fs,
            "/src",
        );
        let result = state.resolve_node_like();
        assert!(
            result.is_resolved(),
            "expected resolved, got {:?}",
            result.resolved_file_name
        );
        assert_eq!(result.resolved_file_name, "/src/lib/utils.js");
    }

    #[test]
    fn resolve_package_imports_pattern() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/components");
        fs.write_file(
            "/src/package.json",
            r##"{"name":"myapp","imports":{"#components/*":"./components/*.js"}}"##,
        )
        .unwrap();
        fs.write_file("/src/components/Button.js", "export const Button = 1;")
            .unwrap();

        let opts = CompilerOptions::default();
        let state = ResolutionState::new(
            "#components/Button",
            "/src",
            false,
            ModuleKind::None,
            &opts,
            &fs,
            "/src",
        );
        let result = state.resolve_node_like();
        assert!(
            result.is_resolved(),
            "expected resolved, got {:?}",
            result.resolved_file_name
        );
        assert_eq!(result.resolved_file_name, "/src/components/Button.js");
    }

    #[test]
    fn resolve_package_imports_lone_hash_unresolved() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.write_file(
            "/src/package.json",
            r##"{"name":"myapp","imports":{"#a":"./a.js"}}"##,
        )
        .unwrap();

        let opts = CompilerOptions::default();
        let state = ResolutionState::new("#", "/src", false, ModuleKind::None, &opts, &fs, "/src");
        let result = state.resolve_node_like();
        assert!(
            !result.is_resolved(),
            "expected unresolved for lone '#', got {:?}",
            result.resolved_file_name
        );
    }

    #[test]
    fn resolve_package_imports_walks_to_parent_scope() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/sub");
        fs.insert_dir("/src/lib");
        fs.write_file(
            "/src/package.json",
            r##"{"name":"myapp","imports":{"#utils":"./lib/utils.js"}}"##,
        )
        .unwrap();
        fs.write_file("/src/lib/utils.js", "export const x = 1;")
            .unwrap();

        let opts = CompilerOptions::default();

        let state = ResolutionState::new(
            "#utils",
            "/src/sub",
            false,
            ModuleKind::None,
            &opts,
            &fs,
            "/src",
        );
        let result = state.resolve_node_like();
        assert!(
            result.is_resolved(),
            "expected resolved, got {:?}",
            result.resolved_file_name
        );
        assert_eq!(result.resolved_file_name, "/src/lib/utils.js");
    }

    #[test]
    fn resolve_types_versions_redirect() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/foo");
        fs.insert_dir("/src/node_modules/foo/old");
        fs.insert_dir("/src/node_modules/foo/new");
        fs.write_file(
            "/src/node_modules/foo/package.json",
            r#"{"name":"foo","types":"./old/index.d.ts","typesVersions":{"*":{"*":["./new/index.d.ts"]}}}"#,
        )
        .unwrap();
        fs.write_file("/src/node_modules/foo/old/index.d.ts", "export {}")
            .unwrap();
        fs.write_file("/src/node_modules/foo/new/index.d.ts", "export {}")
            .unwrap();

        let opts = CompilerOptions::default();
        let state =
            ResolutionState::new("foo", "/src", false, ModuleKind::None, &opts, &fs, "/src");
        let result = state.resolve_node_like();
        assert!(
            result.is_resolved(),
            "expected resolved, got {:?}",
            result.resolved_file_name
        );

        assert_eq!(
            result.resolved_file_name,
            "/src/node_modules/foo/new/index.d.ts"
        );
    }

    #[test]
    fn resolve_types_versions_falls_back_when_no_match() {
        use crate::vfs::InMemoryFS;
        let fs = InMemoryFS::new();
        fs.insert_dir("/src");
        fs.insert_dir("/src/node_modules");
        fs.insert_dir("/src/node_modules/foo");
        fs.write_file(
            "/src/node_modules/foo/package.json",

            r#"{"name":"foo","types":"./index.d.ts","typesVersions":{"*":{"bar":["./other/bar.d.ts"]}}}"#,
        )
        .unwrap();
        fs.write_file("/src/node_modules/foo/index.d.ts", "export {}")
            .unwrap();

        let opts = CompilerOptions::default();
        let state =
            ResolutionState::new("foo", "/src", false, ModuleKind::None, &opts, &fs, "/src");
        let result = state.resolve_node_like();
        assert!(
            result.is_resolved(),
            "expected resolved, got {:?}",
            result.resolved_file_name
        );
        assert_eq!(
            result.resolved_file_name,
            "/src/node_modules/foo/index.d.ts"
        );
    }
}
