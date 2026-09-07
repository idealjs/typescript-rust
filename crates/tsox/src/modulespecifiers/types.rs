use crate::ast::{Node, Symbol};
use crate::core::compiler_options::ResolutionMode;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePath {
    pub file_name: String,
    pub is_in_node_modules: bool,
    pub is_redirect: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchingMode {
    Exact,
    Directory,
    Pattern,
}

pub trait ModuleSpecifierGenerationHost {
    fn get_current_directory(&self) -> String;
    fn use_case_sensitive_file_names(&self) -> bool;
    fn common_source_directory(&self) -> String;
    fn file_exists(&self, path: &str) -> bool;
}

pub type ImportModuleSpecifierPreference = String;

pub const IMPORT_MODULE_SPECIFIER_PREFERENCE_SHORTEST: &str = "shortest";
pub const IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE: &str = "project-relative";
pub const IMPORT_MODULE_SPECIFIER_PREFERENCE_RELATIVE: &str = "relative";
pub const IMPORT_MODULE_SPECIFIER_PREFERENCE_NON_RELATIVE: &str = "non-relative";

pub type ImportModuleSpecifierEndingPreference = String;

pub const IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_AUTO: &str = "auto";
pub const IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_MINIMAL: &str = "minimal";
pub const IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_INDEX: &str = "index";
pub const IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_JS: &str = "js";

#[derive(Debug, Clone, Default)]
pub struct UserPreferences {
    pub import_module_specifier_preference: ImportModuleSpecifierPreference,
    pub import_module_specifier_ending: ImportModuleSpecifierEndingPreference,
    pub auto_import_specifier_exclude_regexes: Vec<String>,
}

#[allow(unused_variables)]
pub fn is_excluded_by_regex(module_specifier: &str, exclude_regexes: &[String]) -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ResultKind {
    #[default]
    None = 0,
    NodeModules = 1,
    Paths = 2,
    Redirect = 3,
    Relative = 4,
    Ambient = 5,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModuleSpecifierOptions {
    pub override_import_mode: ResolutionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum RelativePreferenceKind {
    Relative = 0,
    NonRelative = 1,
    #[default]
    Shortest = 2,
    ExternalNonRelative = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ModuleSpecifierEnding {
    #[default]
    Minimal = 0,
    Index = 1,
    JsExtension = 2,
    TsExtension = 3,
}

pub struct ModuleSpecifierPreferences {
    pub relative_preference: RelativePreferenceKind,
    pub exclude_regexes: Vec<String>,
}

pub trait SourceFileForSpecifierGeneration {
    fn path(&self) -> &str;
    fn file_name(&self) -> &str;
    fn is_js(&self) -> bool;
}

pub trait CheckerShape {
    fn get_symbol_at_location(&self, node: &Arc<Node>) -> Option<Arc<Symbol>>;
    fn get_aliased_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>>;
}

#[derive(Debug, Clone, Default)]
pub struct Info {
    pub importing_source_file_file_name: String,
    pub importing_source_file_directory: String,
    pub importing_source_file_is_in_node_modules: bool,
    pub common_source_directory: String,
    pub use_case_sensitive_file_names: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NodeModulePathParts {
    pub top_level_node_modules_index: usize,
    pub top_level_package_name_index: usize,
    pub package_root_index: usize,
    pub file_name_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegexPatternCacheKey {
    pub pattern: String,
    pub case_insensitive: bool,
}

#[derive(Debug, Clone)]
pub struct SpecPair {
    pub ending: ModuleSpecifierEnding,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct PkgJsonDirAttemptResult {
    pub pkg_json_directory: String,
    pub directory_exists: bool,
    pub package_name: String,
    pub version: String,
    pub root_dir_attempt_failed: bool,
}
