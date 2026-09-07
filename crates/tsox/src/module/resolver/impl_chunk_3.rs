#![allow(unused_imports)]

use super::*;

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
    pub(crate) containing_directory: String,
    pub(crate) module_name: String,
    pub(crate) resolution_mode: ResolutionMode,
    pub(crate) redirect_config_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeRefDirectiveCacheKey {
    pub(crate) containing_directory: String,
    pub(crate) type_reference_name: String,
    pub(crate) resolution_mode: ResolutionMode,
    pub(crate) redirect_config_name: String,
    pub(crate) from_inferred_types_containing_file: bool,
}

pub struct ModuleResolutionCache {
    pub(crate) cache: Mutex<HashMap<ModuleResolutionCacheKey, Arc<ResolvedModule>>>,
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
    pub(crate) cache: Mutex<HashMap<TypeRefDirectiveCacheKey, Arc<ResolvedTypeReferenceDirective>>>,
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
pub(crate) struct Pattern {
    pub(crate) text: String,
    pub(crate) star_index: i32,
}

impl Pattern {
    pub(crate) fn try_parse(pattern: &str) -> Pattern {
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

    pub(crate) fn is_valid(&self) -> bool {
        self.star_index == -1 || (self.star_index as usize) < self.text.len()
    }

    pub(crate) fn matches(&self, candidate: &str) -> bool {
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

    pub(crate) fn matched_text(&self, candidate: &str) -> String {
        if self.star_index == -1 {
            return String::new();
        }
        let idx = self.star_index as usize;
        let suffix_len = self.text.len() - idx - 1;
        candidate[idx..candidate.len() - suffix_len].to_string()
    }
}

pub(crate) struct ParsedPatterns {
    pub(crate) matchable_string_set: std::collections::HashSet<String>,
    pub(crate) patterns: Vec<Pattern>,
}

pub(crate) fn try_parse_patterns(
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

pub(crate) fn match_pattern_or_exact(parsed: &ParsedPatterns, candidate: &str) -> Option<Pattern> {
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
    pub(crate) module_cache: ModuleResolutionCache,
    pub(crate) type_ref_cache: TypeRefDirectiveResolutionCache,
    pub(crate) host: Arc<dyn ResolutionHost + Send + Sync>,
    pub(crate) compiler_options: Arc<CompilerOptions>,

    #[allow(dead_code)]
    pub(crate) typings_location: String,
    #[allow(dead_code)]
    pub(crate) project_name: String,
}
