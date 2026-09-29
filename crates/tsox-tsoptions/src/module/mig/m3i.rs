#![allow(dead_code)]

use crate::module::resolver::{
    DiagAndArgs, Extensions, ModuleResolutionCache, ParsedPatterns, ResolutionHost, Resolved,
    TypeRefDirectiveResolutionCache, get_effective_type_roots, try_parse_patterns,
};
use crate::module::{NodeResolutionFeatures, ResolvedModule};
use crate::packagejson;
use crate::vfs::FS;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::core::compiler_options::{CompilerOptions, JsxEmit, ModuleResolutionKind};
use tsox_core::core::mig::m3j_2::deduplicate;
use tsox_core::core::tristate::Tristate;
use tsox_core::diagnostics;
use tsox_core::semver;
use tsox_core::tspath::{
    self, EXTENSION_CJS, EXTENSION_CTS, EXTENSION_DCTS, EXTENSION_DMTS, EXTENSION_DTS,
    EXTENSION_JS, EXTENSION_JSON, EXTENSION_JSX, EXTENSION_MJS, EXTENSION_MTS, EXTENSION_TS,
    EXTENSION_TSX,
};
use tsox_frontend::ast::node_source_file::SourceFile;

pub struct Tracer {
    pub traces: Vec<DiagAndArgs>,
}

impl Tracer {
    pub fn write(&mut self, message: &'static diagnostics::Message, args: Vec<String>) {
        self.traces.push(DiagAndArgs {
            message,
            args,
        });
    }
}

pub(crate) fn write_tracer(
    t: Option<&mut Tracer>,
    message: &'static diagnostics::Message,
    args: Vec<String>,
) {
    if let Some(t) = t {
        t.traces.push(DiagAndArgs { message, args });
    }
}

pub(crate) fn continue_searching() -> Option<Resolved> {
    None
}

pub(crate) fn unresolved() -> Option<Resolved> {
    Some(Resolved::default())
}

pub(crate) fn matches_pattern_with_trailer(target: &str, name: &str) -> bool {
    if target.ends_with('*') {
        return false;
    }
    let Some(star) = target.find('*') else {
        return false;
    };
    let before = &target[..star];
    let after = &target[star + 1..];
    name.starts_with(before) && name.ends_with(after)
}

pub fn get_node_resolution_features(options: &CompilerOptions) -> NodeResolutionFeatures {
    let mut features = NodeResolutionFeatures::NONE;
    match options.get_module_resolution_kind() {
        ModuleResolutionKind::Node16 => features = NodeResolutionFeatures::NODE16_DEFAULT,
        ModuleResolutionKind::NodeNext => features = NodeResolutionFeatures::NODE_NEXT_DEFAULT,
        ModuleResolutionKind::Bundler => features = NodeResolutionFeatures::BUNDLER_DEFAULT,
        _ => {}
    }
    if options.resolve_package_json_exports == Tristate::True {
        features |= NodeResolutionFeatures::Exports;
    } else if options.resolve_package_json_exports == Tristate::False {
        features -= NodeResolutionFeatures::Exports;
    }
    if options.resolve_package_json_imports == Tristate::True {
        features |= NodeResolutionFeatures::Imports;
    } else if options.resolve_package_json_imports == Tristate::False {
        features -= NodeResolutionFeatures::Imports;
    }
    features
}

pub struct ParsedPatternsCache {
    cache: Mutex<HashMap<usize, Arc<ParsedPatterns>>>,
}

impl ParsedPatternsCache {
    pub fn get(
        &self,
        path_mappings: &Arc<OrderedMap<String, Vec<String>>>,
    ) -> Arc<ParsedPatterns> {
        let key = Arc::as_ptr(path_mappings) as usize;
        if let Some(patterns) = self.cache.lock().unwrap().get(&key) {
            return patterns.clone();
        }
        let mappings: HashMap<String, Vec<String>> = path_mappings
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let patterns = Arc::new(try_parse_patterns(&mappings));
        self.cache
            .lock()
            .unwrap()
            .entry(key)
            .or_insert_with(|| patterns.clone())
            .clone()
    }
}

pub struct InfoCacheEntry {
    pub package_directory: String,
    pub directory_exists: bool,
    pub contents: Option<packagejson::Fields>,
}

pub struct InfoCache {
    pub(crate) cache: Mutex<HashMap<String, InfoCacheEntry>>,
    pub(crate) current_directory: String,
    pub(crate) use_case_sensitive_file_names: bool,
}

impl InfoCache {
    pub fn new(current_directory: &str, use_case_sensitive_file_names: bool) -> Self {
        InfoCache {
            cache: Mutex::new(HashMap::new()),
            current_directory: current_directory.to_string(),
            use_case_sensitive_file_names,
        }
    }
}

pub struct Caches {
    pub package_json_info_cache: InfoCache,
    pub module_resolution_cache: ModuleResolutionCache,
    pub type_ref_directive_resolution_cache: TypeRefDirectiveResolutionCache,
    pub parsed_patterns_for_paths: ParsedPatternsCache,
}

pub fn new_caches(
    current_directory: &str,
    use_case_sensitive_file_names: bool,
    _options: &CompilerOptions,
) -> Caches {
    Caches {
        package_json_info_cache: InfoCache::new(current_directory, use_case_sensitive_file_names),
        module_resolution_cache: ModuleResolutionCache::default(),
        type_ref_directive_resolution_cache: TypeRefDirectiveResolutionCache::default(),
        parsed_patterns_for_paths: ParsedPatternsCache {
            cache: Mutex::new(HashMap::new()),
        },
    }
}

pub trait ResolvedProjectReference {
    fn config_name(&self) -> String;
    fn compiler_options(&self) -> &CompilerOptions;
}

pub fn get_redirect_config_name(redirect: Option<&dyn ResolvedProjectReference>) -> String {
    match redirect {
        None => String::new(),
        Some(r) => r.config_name(),
    }
}

impl std::fmt::Display for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = Vec::new();
        if self.contains(Extensions::TYPESCRIPT) {
            result.push("TypeScript");
        }
        if self.contains(Extensions::JAVASCRIPT) {
            result.push("JavaScript");
        }
        if self.contains(Extensions::DECLARATION) {
            result.push("Declaration");
        }
        if self.contains(Extensions::JSON) {
            result.push("JSON");
        }
        write!(f, "{}", result.join(", "))
    }
}

fn type_script_version() -> semver::Version {
    semver::must_parse(tsox_core::core::mig::m3k::version())
}

pub fn is_applicable_versioned_types_key(key: &str) -> bool {
    let Some(range_str) = key.strip_prefix("types@") else {
        return false;
    };
    let Some(range) = semver::try_parse_version_range(range_str) else {
        return false;
    };
    range.test(&type_script_version())
}

pub fn get_resolution_diagnostic(
    options: &CompilerOptions,
    resolved_module: &ResolvedModule,
    file: &SourceFile,
) -> Option<&'static diagnostics::Message> {
    let need_jsx = || -> Option<&'static diagnostics::Message> {
        if options.jsx != JsxEmit::None {
            return None;
        }
        Some(&diagnostics::MODULE_0_WAS_RESOLVED_TO_1_BUT_JSX_IS_NOT_SET)
    };

    let need_allow_js = || -> Option<&'static diagnostics::Message> {
        if options.get_allow_js()
            || !options
                .no_implicit_any
                .default_if_unknown(options.strict)
                .is_true()
        {
            return None;
        }
        Some(&diagnostics::COULD_NOT_FIND_A_DECLARATION_FILE_FOR_MODULE_0_1_IMPLICITLY_HAS_AN_ANY_TYPE)
    };

    let need_resolve_json_module = || -> Option<&'static diagnostics::Message> {
        if options.get_resolve_json_module() {
            return None;
        }
        Some(&diagnostics::MODULE_0_WAS_RESOLVED_TO_1_BUT_RESOLVEJSONMODULE_IS_NOT_USED)
    };

    let need_allow_arbitrary_extensions = || -> Option<&'static diagnostics::Message> {
        if file.is_declaration_file || options.allow_arbitrary_extensions.is_true() {
            return None;
        }
        Some(&diagnostics::MODULE_0_WAS_RESOLVED_TO_1_BUT_ALLOWARBITRARYEXTENSIONS_IS_NOT_SET)
    };

    if resolved_module.resolved_using_extra_extensions {
        return None;
    }

    match resolved_module.extension.as_str() {
        EXTENSION_TS | EXTENSION_DTS | EXTENSION_MTS | EXTENSION_DMTS | EXTENSION_CTS
        | EXTENSION_DCTS => None,
        EXTENSION_TSX => need_jsx(),
        EXTENSION_JSX => need_jsx().or_else(need_allow_js),
        EXTENSION_JS | EXTENSION_MJS | EXTENSION_CJS => need_allow_js(),
        EXTENSION_JSON => need_resolve_json_module(),
        _ => need_allow_arbitrary_extensions(),
    }
}

pub fn try_get_js_extension_for_file<'a>(file_name: &'a str, options: &CompilerOptions) -> &'a str {
    let ext = tspath::try_get_extension_from_path(file_name);
    match ext {
        EXTENSION_TS | EXTENSION_DTS => EXTENSION_JS,
        EXTENSION_TSX => {
            if options.jsx == JsxEmit::Preserve {
                EXTENSION_JSX
            } else {
                EXTENSION_JS
            }
        }
        EXTENSION_JS | EXTENSION_JSX | EXTENSION_JSON => ext,
        EXTENSION_DMTS | EXTENSION_MTS | EXTENSION_MJS => EXTENSION_MJS,
        EXTENSION_DCTS | EXTENSION_CTS | EXTENSION_CJS => EXTENSION_CJS,
        _ => "",
    }
}

pub fn get_automatic_type_directive_names(
    options: &CompilerOptions,
    host: &dyn ResolutionHost,
) -> Vec<String> {
    if !options.uses_wildcard_types() {
        return options.types.clone();
    }

    let mut wildcard_matches = Vec::new();
    let (type_roots, _) = get_effective_type_roots(options, host.get_current_directory());
    for root in type_roots {
        if host.fs().directory_exists(&root) {
            for type_directive_path in &host.fs().get_accessible_entries(&root).directories {
                let normalized = tspath::normalize_path(type_directive_path);
                let package_json_path = tspath::combine_paths(&root, &[&normalized, "package.json"]);
                let mut is_not_needed_package = false;
                if host.fs().file_exists(&package_json_path) {
                    let contents = host.fs().read_file(&package_json_path).unwrap_or_default();
                    is_not_needed_package = packagejson::parse(&contents)
                        .map(|fields| fields.path_fields.typings.null)
                        .unwrap_or(false);
                }
                if !is_not_needed_package {
                    let base_file_name = tspath::get_base_file_name(&normalized);
                    if !base_file_name.starts_with('.') {
                        wildcard_matches.push(base_file_name);
                    }
                }
            }
        }
    }

    let mut result = Vec::new();
    for t in &options.types {
        if t == "*" {
            result.extend(wildcard_matches.iter().cloned());
        } else {
            result.push(t.clone());
        }
    }
    deduplicate(&result)
}
