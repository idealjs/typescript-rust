//! Compiler options, ported from `internal/core/compileroptions.go`.
//!
//! Defines the `CompilerOptions` struct and related enums (ModuleKind,
//! ModuleResolutionKind, JsxEmit, ScriptTarget, etc.).

use crate::core::tristate::Tristate;
use crate::tspath;
use std::collections::HashMap;

/// Script target (language version).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum ScriptTarget {
    #[default]
    None = 0,
    ES5 = 1,
    ES2015 = 2,
    ES2016 = 3,
    ES2017 = 4,
    ES2018 = 5,
    ES2019 = 6,
    ES2020 = 7,
    ES2021 = 8,
    ES2022 = 9,
    ES2023 = 10,
    ES2024 = 11,
    ES2025 = 12,
    ESNext = 99,
    JSON = 100,
}

impl ScriptTarget {
    pub const LATEST: ScriptTarget = ScriptTarget::ESNext;
    pub const LATEST_STANDARD: ScriptTarget = ScriptTarget::ES2025;
}

/// Module kind (output module format).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum ModuleKind {
    #[default]
    None = 0,
    CommonJS = 1,
    AMD = 2,
    UMD = 3,
    System = 4,
    ES2015 = 5,
    ES2020 = 6,
    ES2022 = 7,
    ESNext = 99,
    Node16 = 100,
    Node18 = 101,
    Node20 = 102,
    NodeNext = 199,
    Preserve = 200,
}

impl ModuleKind {
    pub fn is_non_node_esm(&self) -> bool {
        *self >= ModuleKind::ES2015 && *self <= ModuleKind::ESNext
    }

    pub fn supports_import_attributes(&self) -> bool {
        (*self >= ModuleKind::Node18 && *self <= ModuleKind::NodeNext)
            || *self == ModuleKind::Preserve
            || *self == ModuleKind::ESNext
    }
}

/// Module resolution kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ModuleResolutionKind {
    #[default]
    Unknown = 0,
    Classic = 1,
    Node10 = 2,
    Node16 = 3,
    NodeNext = 99,
    Bundler = 100,
}

impl std::fmt::Display for ModuleResolutionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleResolutionKind::Unknown => write!(f, "Unknown"),
            ModuleResolutionKind::Classic => write!(f, "Classic"),
            ModuleResolutionKind::Node10 => write!(f, "Node10"),
            ModuleResolutionKind::Node16 => write!(f, "Node16"),
            ModuleResolutionKind::NodeNext => write!(f, "NodeNext"),
            ModuleResolutionKind::Bundler => write!(f, "Bundler"),
        }
    }
}

/// Resolution mode (used for mode-aware module resolution).
pub type ResolutionMode = ModuleKind;

/// Module detection kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ModuleDetectionKind {
    #[default]
    None = 0,
    Auto = 1,
    Legacy = 2,
    Force = 3,
}

/// JSX emit kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum JsxEmit {
    #[default]
    None = 0,
    Preserve = 1,
    ReactNative = 2,
    React = 3,
    ReactJSX = 4,
    ReactJSXDev = 5,
}

impl std::fmt::Display for JsxEmit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsxEmit::None => write!(f, "none"),
            JsxEmit::Preserve => write!(f, "preserve"),
            JsxEmit::ReactNative => write!(f, "react-native"),
            JsxEmit::React => write!(f, "react"),
            JsxEmit::ReactJSX => write!(f, "react-jsx"),
            JsxEmit::ReactJSXDev => write!(f, "react-jsxdev"),
        }
    }
}

/// New line kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum NewLineKind {
    #[default]
    None = 0,
    CRLF = 1,
    LF = 2,
}

impl NewLineKind {
    pub fn from_str(s: &str) -> NewLineKind {
        match s {
            "\r\n" => NewLineKind::CRLF,
            "\n" => NewLineKind::LF,
            _ => NewLineKind::None,
        }
    }

    pub fn get_new_line_character(&self) -> &'static str {
        match self {
            NewLineKind::CRLF => "\r\n",
            _ => "\n",
        }
    }
}

/// Compiler options.
///
/// Mirrors `core.CompilerOptions` in Go. Fields use `Tristate` for boolean
/// options that can be true, false, or unspecified.
#[derive(Clone, Debug, Default)]
pub struct CompilerOptions {
    pub allow_js: Tristate,
    pub allow_arbitrary_extensions: Tristate,
    pub allow_importing_ts_extensions: Tristate,
    pub allow_non_ts_extensions: Tristate,
    pub allow_umd_global_access: Tristate,
    pub allow_unreachable_code: Tristate,
    pub allow_unused_labels: Tristate,
    pub assume_changes_only_affect_direct_dependencies: Tristate,
    pub check_js: Tristate,
    pub custom_conditions: Vec<String>,
    pub composite: Tristate,
    pub emit_declaration_only: Tristate,
    pub emit_bom: Tristate,
    pub emit_decorator_metadata: Tristate,
    pub declaration: Tristate,
    pub declaration_dir: String,
    pub declaration_map: Tristate,
    pub deduplicate_packages: Tristate,
    pub disable_size_limit: Tristate,
    pub disable_source_of_project_reference_redirect: Tristate,
    pub disable_solution_searching: Tristate,
    pub disable_referenced_project_load: Tristate,
    pub erasable_syntax_only: Tristate,
    pub exact_optional_property_types: Tristate,
    pub experimental_decorators: Tristate,
    pub force_consistent_casing_in_file_names: Tristate,
    pub isolated_modules: Tristate,
    pub isolated_declarations: Tristate,
    pub ignore_config: Tristate,
    pub ignore_deprecations: String,
    pub import_helpers: Tristate,
    pub inline_source_map: Tristate,
    pub inline_sources: Tristate,
    pub init: Tristate,
    pub incremental: Tristate,
    pub jsx: JsxEmit,
    pub jsx_factory: String,
    pub jsx_fragment_factory: String,
    pub jsx_import_source: String,
    pub lib: Vec<String>,
    pub lib_replacement: Tristate,
    pub locale: String,
    pub map_root: String,
    pub module: ModuleKind,
    pub module_resolution: ModuleResolutionKind,
    pub module_suffixes: Vec<String>,
    pub module_detection: ModuleDetectionKind,
    pub new_line: NewLineKind,
    pub no_emit: Tristate,
    pub no_check: Tristate,
    pub no_error_truncation: Tristate,
    pub no_fallthrough_cases_in_switch: Tristate,
    pub no_implicit_any: Tristate,
    pub no_implicit_this: Tristate,
    pub no_implicit_returns: Tristate,
    pub no_emit_helpers: Tristate,
    pub no_lib: Tristate,
    pub no_property_access_from_index_signature: Tristate,
    pub no_unchecked_indexed_access: Tristate,
    pub no_emit_on_error: Tristate,
    pub no_unused_locals: Tristate,
    pub no_unused_parameters: Tristate,
    pub no_resolve: Tristate,
    pub no_implicit_override: Tristate,
    pub no_unchecked_side_effect_imports: Tristate,
    pub out_dir: String,
    pub paths: Option<HashMap<String, Vec<String>>>,
    pub preserve_const_enums: Tristate,
    pub preserve_symlinks: Tristate,
    pub project: String,
    pub resolve_json_module: Tristate,
    pub resolve_package_json_exports: Tristate,
    pub resolve_package_json_imports: Tristate,
    pub remove_comments: Tristate,
    pub rewrite_relative_import_extensions: Tristate,
    pub react_namespace: String,
    pub root_dir: String,
    pub root_dirs: Vec<String>,
    pub skip_lib_check: Tristate,
    pub stable_type_ordering: Tristate,
    pub strict: Tristate,
    pub strict_bind_call_apply: Tristate,
    pub strict_builtin_iterator_return: Tristate,
    pub strict_function_types: Tristate,
    pub strict_null_checks: Tristate,
    pub strict_property_initialization: Tristate,
    pub strip_internal: Tristate,
    pub skip_default_lib_check: Tristate,
    pub source_map: Tristate,
    pub source_root: String,
    pub suppress_output_path_check: Tristate,
    pub target: ScriptTarget,
    pub trace_resolution: Tristate,
    pub ts_build_info_file: String,
    pub type_roots: Vec<String>,
    pub types: Vec<String>,
    pub use_define_for_class_fields: Tristate,
    pub use_unknown_in_catch_variables: Tristate,
    pub verbatim_module_syntax: Tristate,
    pub max_node_module_js_depth: Option<i32>,

    // Deprecated fields
    pub allow_synthetic_default_imports: Tristate,
    pub always_strict: Tristate,
    pub base_url: String,
    pub downlevel_iteration: Tristate,
    pub es_module_interop: Tristate,
    pub out_file: String,

    // Internal fields
    pub config_file_path: String,
    pub no_dts_resolution: Tristate,
    pub paths_base_path: String,
    pub diagnostics: Tristate,
    pub extended_diagnostics: Tristate,
    pub generate_cpu_profile: String,
    pub generate_trace: String,
    pub list_emitted_files: Tristate,
    pub list_files: Tristate,
    pub explain_files: Tristate,
    pub list_files_only: Tristate,
    pub no_emit_for_js_files: Tristate,
    pub preserve_watch_output: Tristate,
    pub pretty: Tristate,
    pub version: Tristate,
    pub watch: Tristate,
    pub show_config: Tristate,
    pub build: Tristate,
    pub help: Tristate,
    pub all: Tristate,

    pub pprof_dir: String,
    pub single_threaded: Tristate,
    pub quiet: Tristate,
    pub checkers: Option<i32>,
}

/// Empty compiler options (all defaults).
pub fn empty_compiler_options() -> CompilerOptions {
    CompilerOptions::default()
}

impl CompilerOptions {
    pub fn get_emit_script_target(&self) -> ScriptTarget {
        if self.target != ScriptTarget::None {
            self.target
        } else {
            ScriptTarget::LATEST_STANDARD
        }
    }

    pub fn get_emit_module_kind(&self) -> ModuleKind {
        if self.module != ModuleKind::None {
            return self.module;
        }
        let target = self.get_emit_script_target();
        if target == ScriptTarget::ESNext {
            ModuleKind::ESNext
        } else if target >= ScriptTarget::ES2022 {
            ModuleKind::ES2022
        } else if target >= ScriptTarget::ES2020 {
            ModuleKind::ES2020
        } else if target >= ScriptTarget::ES2015 {
            ModuleKind::ES2015
        } else {
            ModuleKind::CommonJS
        }
    }

    pub fn get_module_resolution_kind(&self) -> ModuleResolutionKind {
        match self.module_resolution {
            ModuleResolutionKind::Unknown
            | ModuleResolutionKind::Classic
            | ModuleResolutionKind::Node10 => match self.get_emit_module_kind() {
                ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 => {
                    ModuleResolutionKind::Node16
                }
                ModuleKind::NodeNext => ModuleResolutionKind::NodeNext,
                _ => ModuleResolutionKind::Bundler,
            },
            other => other,
        }
    }

    pub fn get_emit_module_detection_kind(&self) -> ModuleDetectionKind {
        if self.module_detection != ModuleDetectionKind::None {
            return self.module_detection;
        }
        let module_kind = self.get_emit_module_kind();
        if module_kind >= ModuleKind::Node16 && module_kind <= ModuleKind::NodeNext {
            ModuleDetectionKind::Force
        } else {
            ModuleDetectionKind::Auto
        }
    }

    pub fn get_resolve_package_json_exports(&self) -> bool {
        self.resolve_package_json_exports.is_true_or_unknown()
    }

    pub fn get_resolve_package_json_imports(&self) -> bool {
        self.resolve_package_json_imports.is_true_or_unknown()
    }

    pub fn get_allow_importing_ts_extensions(&self) -> bool {
        self.allow_importing_ts_extensions.is_true()
            || self.rewrite_relative_import_extensions.is_true()
    }

    pub fn allow_importing_ts_extensions_from(&self, file_name: &str) -> bool {
        self.get_allow_importing_ts_extensions() || tspath::is_declaration_file_name(file_name)
    }

    pub fn get_resolve_json_module(&self) -> bool {
        if self.resolve_json_module != Tristate::Unknown {
            return self.resolve_json_module == Tristate::True;
        }
        match self.get_emit_module_kind() {
            ModuleKind::Node20 | ModuleKind::NodeNext => true,
            _ => self.get_module_resolution_kind() == ModuleResolutionKind::Bundler,
        }
    }

    pub fn should_preserve_const_enums(&self) -> bool {
        self.preserve_const_enums == Tristate::True || self.get_isolated_modules()
    }

    pub fn get_allow_js(&self) -> bool {
        if self.allow_js != Tristate::Unknown {
            self.allow_js == Tristate::True
        } else {
            self.check_js == Tristate::True
        }
    }

    pub fn get_jsx_transform_enabled(&self) -> bool {
        matches!(
            self.jsx,
            JsxEmit::React | JsxEmit::ReactJSX | JsxEmit::ReactJSXDev
        )
    }

    pub fn get_strict_option_value(&self, value: Tristate) -> bool {
        if value != Tristate::Unknown {
            return value == Tristate::True;
        }
        self.strict != Tristate::False
    }

    pub fn get_isolated_modules(&self) -> bool {
        self.isolated_modules == Tristate::True || self.verbatim_module_syntax == Tristate::True
    }

    pub fn is_incremental(&self) -> bool {
        self.incremental.is_true() || self.composite.is_true()
    }

    pub fn get_emit_standard_class_fields(&self) -> bool {
        self.use_define_for_class_fields != Tristate::False
            && self.get_emit_script_target() >= ScriptTarget::ES2022
    }

    pub fn get_use_define_for_class_fields(&self) -> bool {
        if self.use_define_for_class_fields == Tristate::Unknown {
            self.get_emit_script_target() >= ScriptTarget::ES2022
        } else {
            self.use_define_for_class_fields == Tristate::True
        }
    }

    pub fn get_emit_declarations(&self) -> bool {
        self.declaration.is_true() || self.composite.is_true()
    }

    pub fn get_are_declaration_maps_enabled(&self) -> bool {
        self.declaration_map == Tristate::True && self.get_emit_declarations()
    }

    pub fn has_json_module_emit_enabled(&self) -> bool {
        !matches!(
            self.get_emit_module_kind(),
            ModuleKind::System | ModuleKind::UMD
        )
    }

    pub fn uses_wildcard_types(&self) -> bool {
        self.types.iter().any(|t| t == "*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        let opts = CompilerOptions::default();
        assert_eq!(opts.get_emit_script_target(), ScriptTarget::LATEST_STANDARD);
        assert_eq!(opts.get_emit_module_kind(), ModuleKind::ES2022);
        assert_eq!(
            opts.get_module_resolution_kind(),
            ModuleResolutionKind::Bundler
        );
    }

    #[test]
    fn commonjs_target() {
        let mut opts = CompilerOptions::default();
        opts.target = ScriptTarget::ES5;
        assert_eq!(opts.get_emit_module_kind(), ModuleKind::CommonJS);
    }

    #[test]
    fn node_next_resolution() {
        let mut opts = CompilerOptions::default();
        opts.module = ModuleKind::NodeNext;
        assert_eq!(
            opts.get_module_resolution_kind(),
            ModuleResolutionKind::NodeNext
        );
    }

    #[test]
    fn get_allow_js() {
        let mut opts = CompilerOptions::default();
        assert!(!opts.get_allow_js());
        opts.allow_js = Tristate::True;
        assert!(opts.get_allow_js());
        opts.allow_js = Tristate::Unknown;
        opts.check_js = Tristate::True;
        assert!(opts.get_allow_js());
    }

    #[test]
    fn strict_option_value() {
        let mut opts = CompilerOptions::default();
        opts.strict = Tristate::True;
        assert!(opts.get_strict_option_value(Tristate::Unknown));
        opts.strict = Tristate::False;
        assert!(!opts.get_strict_option_value(Tristate::Unknown));
        assert!(opts.get_strict_option_value(Tristate::True));
        assert!(!opts.get_strict_option_value(Tristate::False));
    }
}
