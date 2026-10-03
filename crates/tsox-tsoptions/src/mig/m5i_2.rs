#![allow(unused_imports)]

use crate::tsoptions::*;
use std::sync::LazyLock;
use super::m5h_2::{get_name_map_from_list, NameMap};
use tsox_core::json::Value;
use tsox_core::core::tristate::Tristate;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;

pub type CommandLineOptionNameMap = NameMap;

#[derive(Debug, Default)]
pub struct ContentMapperDefinition {
    pub extensions: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ContentMapper {
    pub package: String,
    pub definition: ContentMapperDefinition,
    pub options: String,
}

pub fn command_line_compiler_options_map() -> &'static NameMap { ::tsox_core::fntrace::enter("command_line_compiler_options_map"); 
    static MAP: LazyLock<NameMap> =
        LazyLock::new(|| get_name_map_from_list(&crate::tsoptions::OPTIONS));
    &MAP
}

pub fn build_name_map() -> &'static NameMap { ::tsox_core::fntrace::enter("build_name_map"); 
    static MAP: LazyLock<NameMap> =
        LazyLock::new(|| get_name_map_from_list(crate::tsoptions::BUILD_OPTIONS));
    &MAP
}

pub fn parse_tristate(value: &Value) -> Tristate { ::tsox_core::fntrace::enter("parse_tristate"); 
    match value {
        Value::Null => Tristate::Unknown,
        Value::Bool(true) => Tristate::True,
        _ => Tristate::False,
    }
}

pub fn parse_string_array(value: &Value) -> Vec<String> { ::tsox_core::fntrace::enter("parse_string_array"); 
    match value.as_array() {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        None => Vec::new(),
    }
}

pub fn parse_string_map(value: &Value) -> Vec<(String, Vec<String>)> { ::tsox_core::fntrace::enter("parse_string_map"); 
    match value.as_object() {
        Some(m) => m
            .iter()
            .map(|(k, v)| (k.clone(), parse_string_array(v)))
            .collect(),
        None => Vec::new(),
    }
}

pub fn parse_string(value: &Value) -> String { ::tsox_core::fntrace::enter("parse_string"); 
    value.as_str().unwrap_or("").to_string()
}

pub fn parse_number(value: &Value) -> Option<i64> { ::tsox_core::fntrace::enter("parse_number"); 
    if let Some(n) = value.as_i64() {
        return Some(n);
    }
    value.as_f64().map(|f| f as i64)
}

#[derive(Debug)]
pub struct ProjectReferenceParseResult {
    pub reference: tsox_core::core::project_reference::ProjectReference,
    pub has_path: bool,
    pub path_valid: bool,
    pub has_circular: bool,
    pub circular_valid: bool,
}

impl Default for ProjectReferenceParseResult {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self {
            reference: tsox_core::core::project_reference::ProjectReference {
                path: String::new(),
                original_path: String::new(),
                circular: false,
            },
            has_path: false,
            path_valid: false,
            has_circular: false,
            circular_valid: false,
        }
    }
}

pub fn parse_project_reference(json: &Value) -> Option<ProjectReferenceParseResult> { ::tsox_core::fntrace::enter("parse_project_reference"); 
    let v = json.as_object()?;
    let mut result = ProjectReferenceParseResult::default();
    if let Some(value) = v.get("path") {
        result.has_path = true;
        if let Some(path) = value.as_str() {
            result.reference.path = path.to_string();
            result.path_valid = true;
        }
    }
    if let Some(value) = v.get("circular") {
        result.has_circular = true;
        if let Some(circular) = value.as_bool() {
            result.reference.circular = circular;
            result.circular_valid = true;
        }
    }
    Some(result)
}

pub fn parse_content_mapper(value: &Value) -> (Option<ContentMapper>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("parse_content_mapper"); 
    let v = match value.as_object() {
        Some(v) => v,
        None => return (None, Vec::new()),
    };
    let mut errors = Vec::new();
    let mut mapper = ContentMapper::default();
    match v.get("package").and_then(|p| p.as_str()) {
        Some(str) if !str.is_empty() => mapper.package = str.to_string(),
        _ => errors.push(new_compiler_diagnostic(
            tsox_core::diagnostics::COMPILER_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
            vec!["contentMapper.package".to_string(), "string".to_string()],
        )),
    }
    match v.get("extensions") {
        Some(extensions) => match parse_string_array_strict(extensions) {
            Some(strs) => mapper.definition.extensions = strs,
            None => errors.push(new_compiler_diagnostic(
                tsox_core::diagnostics::COMPILER_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
                vec![
                    "contentMapper.extensions".to_string(),
                    "string[]".to_string(),
                ],
            )),
        },
        None => errors.push(new_compiler_diagnostic(
            tsox_core::diagnostics::COMPILER_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
            vec![
                "contentMapper.extensions".to_string(),
                "string[]".to_string(),
            ],
        )),
    }
    if let Some(options) = v.get("options") {
        if !options.is_object() {
            errors.push(new_compiler_diagnostic(
                tsox_core::diagnostics::COMPILER_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
                vec!["contentMapper.options".to_string(), "object".to_string()],
            ));
        } else {
            mapper.options = serde_json::to_string(options).unwrap_or_default();
        }
    }
    if !errors.is_empty() {
        return (None, errors);
    }
    (Some(mapper), errors)
}

pub fn parse_string_array_strict(value: &Value) -> Option<Vec<String>> { ::tsox_core::fntrace::enter("parse_string_array_strict"); 
    let arr = value.as_array()?;
    let mut result = Vec::with_capacity(arr.len());
    for v in arr {
        result.push(v.as_str()?.to_string());
    }
    Some(result)
}

pub fn parse_json_to_string_key(json: &Value) -> serde_json::Map<String, Value> { ::tsox_core::fntrace::enter("parse_json_to_string_key"); 
    let mut result = serde_json::Map::new();
    if let Some(m) = json.as_object() {
        for key in ["include", "exclude", "files", "references", "contentMappers", "compilerOptions", "excludes", "typeAcquisition"] {
            if let Some(v) = m.get(key) {
                result.insert(key.to_string(), v.clone());
            }
        }
        if let Some(v) = m.get("extends") {
            if let Some(str) = v.as_str() {
                result.insert("extends".to_string(), Value::Array(vec![Value::String(str.to_string())]));
            } else {
                result.insert("extends".to_string(), v.clone());
            }
        }
    }
    result
}

pub fn extra_key_diagnostics(section: &str) -> Option<Message> { ::tsox_core::fntrace::enter("extra_key_diagnostics"); 
    match section {
        "compilerOptions" => Some(tsox_core::diagnostics::UNKNOWN_COMPILER_OPTION_0),
        "watchOptions" => Some(tsox_core::diagnostics::UNKNOWN_WATCH_OPTION_0),
        "typeAcquisition" => Some(tsox_core::diagnostics::UNKNOWN_TYPE_ACQUISITION_OPTION_0),
        "buildOptions" => Some(tsox_core::diagnostics::UNKNOWN_BUILD_OPTION_0),
        _ => None,
    }
}

pub fn extra_key_did_you_mean_diagnostics(section: &str) -> Option<Message> { ::tsox_core::fntrace::enter("extra_key_did_you_mean_diagnostics"); 
    match section {
        "compilerOptions" => Some(tsox_core::diagnostics::UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1),
        "watchOptions" => Some(tsox_core::diagnostics::UNKNOWN_WATCH_OPTION_0_DID_YOU_MEAN_1),
        "typeAcquisition" => Some(
            tsox_core::diagnostics::UNKNOWN_TYPE_ACQUISITION_OPTION_0_DID_YOU_MEAN_1,
        ),
        "buildOptions" => Some(tsox_core::diagnostics::UNKNOWN_BUILD_OPTION_0_DID_YOU_MEAN_1),
        _ => None,
    }
}

pub struct CompilerOptionsParser<'a> {
    pub options: &'a mut CompilerOptions,
}

impl<'a> CompilerOptionsParser<'a> {
    pub fn parse_option(&mut self, key: &str, value: &Value) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("parse_option"); 
        parse_compiler_options_public(key, value, self.options);
        Vec::new()
    }
    pub fn unknown_option_diagnostic(&self) -> Option<Message> { ::tsox_core::fntrace::enter("unknown_option_diagnostic"); 
        extra_key_diagnostics("compilerOptions")
    }
    pub fn unknown_did_you_mean_diagnostic(&self) -> Option<Message> { ::tsox_core::fntrace::enter("unknown_did_you_mean_diagnostic"); 
        extra_key_did_you_mean_diagnostics("compilerOptions")
    }
}

pub struct WatchOptionsParser<'a> {
    pub options: &'a mut WatchOptions,
}

impl<'a> WatchOptionsParser<'a> {
    pub fn parse_option(&mut self, key: &str, value: &Value) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("parse_option"); 
        parse_watch_options(key, value, self.options);
        Vec::new()
    }
    pub fn unknown_option_diagnostic(&self) -> Option<Message> { ::tsox_core::fntrace::enter("unknown_option_diagnostic"); 
        extra_key_diagnostics("watchOptions")
    }
    pub fn unknown_did_you_mean_diagnostic(&self) -> Option<Message> { ::tsox_core::fntrace::enter("unknown_did_you_mean_diagnostic"); 
        extra_key_did_you_mean_diagnostics("watchOptions")
    }
}

pub struct TypeAcquisitionParser<'a> {
    pub options: &'a mut tsox_core::core::mig::m3k::TypeAcquisition,
}

impl<'a> TypeAcquisitionParser<'a> {
    pub fn parse_option(&mut self, key: &str, value: &Value) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("parse_option"); 
        parse_type_acquisition(key, value, self.options);
        Vec::new()
    }
    pub fn unknown_option_diagnostic(&self) -> Option<Message> { ::tsox_core::fntrace::enter("unknown_option_diagnostic"); 
        extra_key_diagnostics("typeAcquisition")
    }
    pub fn unknown_did_you_mean_diagnostic(&self) -> Option<Message> { ::tsox_core::fntrace::enter("unknown_did_you_mean_diagnostic"); 
        extra_key_did_you_mean_diagnostics("typeAcquisition")
    }
}

pub struct BuildOptionsParser<'a> {
    pub options: &'a mut BuildOptions,
}

impl<'a> BuildOptionsParser<'a> {
    pub fn parse_option(&mut self, key: &str, value: &Value) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("parse_option"); 
        parse_build_options(key, value, self.options);
        Vec::new()
    }
    pub fn unknown_option_diagnostic(&self) -> Option<Message> { ::tsox_core::fntrace::enter("unknown_option_diagnostic"); 
        extra_key_diagnostics("buildOptions")
    }
    pub fn unknown_did_you_mean_diagnostic(&self) -> Option<Message> { ::tsox_core::fntrace::enter("unknown_did_you_mean_diagnostic"); 
        extra_key_did_you_mean_diagnostics("buildOptions")
    }
}

pub fn parse_compiler_options_public(key: &str, value: &Value, all_options: &mut CompilerOptions) { ::tsox_core::fntrace::enter("parse_compiler_options_public"); 
    if value.is_null() {
        return;
    }
    parse_compiler_options_inner(key, value, all_options);
}

pub fn parse_compiler_options_inner(key: &str, value: &Value, o: &mut CompilerOptions) -> bool { ::tsox_core::fntrace::enter("parse_compiler_options_inner"); 
    macro_rules! t { ($f:ident) => {{ o.$f = parse_tristate(value); }} }
    macro_rules! s { ($f:ident) => {{ o.$f = parse_string(value); }} }
    macro_rules! a { ($f:ident) => {{ o.$f = parse_string_array(value); }} }
    macro_rules! e { ($f:ident, $k:ident) => {{ o.$f = float_or_int32_to_flag::<$k>(value); }} }
    let key = command_line_compiler_options_map()
        .get(key)
        .map(|option| option.name)
        .unwrap_or(key);
    match key {
        "allowJs" => t!(allow_js),
        "allowImportingTsExtensions" => t!(allow_importing_ts_extensions),
        "allowSyntheticDefaultImports" => t!(allow_synthetic_default_imports),
        "allowNonTsExtensions" => t!(allow_non_ts_extensions),
        "allowUmdGlobalAccess" => t!(allow_umd_global_access),
        "allowUnreachableCode" => t!(allow_unreachable_code),
        "allowUnusedLabels" => t!(allow_unused_labels),
        "allowArbitraryExtensions" => t!(allow_arbitrary_extensions),
        "alwaysStrict" => t!(always_strict),
        "assumeChangesOnlyAffectDirectDependencies" => t!(assume_changes_only_affect_direct_dependencies),
        "baseUrl" => s!(base_url),
        "build" => t!(build),
        "checkJs" => t!(check_js),
        "customConditions" => a!(custom_conditions),
        "composite" => t!(composite),
        "declarationDir" => s!(declaration_dir),
        "deduplicatePackages" => t!(deduplicate_packages),
        "diagnostics" => t!(diagnostics),
        "disableSizeLimit" => t!(disable_size_limit),
        "disableSourceOfProjectReferenceRedirect" => t!(disable_source_of_project_reference_redirect),
        "disableSolutionSearching" => t!(disable_solution_searching),
        "disableReferencedProjectLoad" => t!(disable_referenced_project_load),
        "declarationMap" => t!(declaration_map),
        "declaration" => t!(declaration),
        "downlevelIteration" => t!(downlevel_iteration),
        "erasableSyntaxOnly" => t!(erasable_syntax_only),
        "emitDeclarationOnly" => t!(emit_declaration_only),
        "extendedDiagnostics" => t!(extended_diagnostics),
        "emitDecoratorMetadata" => t!(emit_decorator_metadata),
        "emitBOM" => t!(emit_bom),
        "esModuleInterop" => t!(es_module_interop),
        "exactOptionalPropertyTypes" => t!(exact_optional_property_types),
        "explainFiles" => t!(explain_files),
        "experimentalDecorators" => t!(experimental_decorators),
        "forceConsistentCasingInFileNames" => t!(force_consistent_casing_in_file_names),
        "generateCpuProfile" => s!(generate_cpu_profile),
        "generateTrace" => s!(generate_trace),
        "isolatedModules" => t!(isolated_modules),
        "ignoreConfig" => t!(ignore_config),
        "ignoreDeprecations" => s!(ignore_deprecations),
        "importHelpers" => t!(import_helpers),
        "incremental" => t!(incremental),
        "init" => t!(init),
        "inlineSourceMap" => t!(inline_source_map),
        "inlineSources" => t!(inline_sources),
        "isolatedDeclarations" => t!(isolated_declarations),
        "jsx" => e!(jsx, JsxEmit),
        "jsxFactory" => s!(jsx_factory),
        "jsxFragmentFactory" => s!(jsx_fragment_factory),
        "jsxImportSource" => s!(jsx_import_source),
        "lib" => o.lib = parse_string_array(value),
        "libReplacement" => t!(lib_replacement),
        "listEmittedFiles" => t!(list_emitted_files),
        "listFiles" => t!(list_files),
        "listFilesOnly" => t!(list_files_only),
        "locale" => s!(locale),
        "mapRoot" => s!(map_root),
        "module" => e!(module, ModuleKind),
        "moduleDetectionKind" | "moduleDetection" => e!(module_detection, ModuleDetectionKind),
        "moduleResolution" => e!(module_resolution, ModuleResolutionKind),
        "moduleSuffixes" => a!(module_suffixes),
        "noCheck" => t!(no_check),
        "noFallthroughCasesInSwitch" => t!(no_fallthrough_cases_in_switch),
        "noEmitForJsFiles" => t!(no_emit_for_js_files),
        "noErrorTruncation" => t!(no_error_truncation),
        "noImplicitAny" => t!(no_implicit_any),
        "noImplicitThis" => t!(no_implicit_this),
        "noLib" => t!(no_lib),
        "noPropertyAccessFromIndexSignature" => t!(no_property_access_from_index_signature),
        "noUncheckedIndexedAccess" => t!(no_unchecked_indexed_access),
        "noEmitHelpers" => t!(no_emit_helpers),
        "noEmitOnError" => t!(no_emit_on_error),
        "noImplicitReturns" => t!(no_implicit_returns),
        "noUnusedLocals" => t!(no_unused_locals),
        "noUnusedParameters" => t!(no_unused_parameters),
        "noImplicitOverride" => t!(no_implicit_override),
        "noUncheckedSideEffectImports" => t!(no_unchecked_side_effect_imports),
        "outFile" => s!(out_file),
        "noResolve" => t!(no_resolve),
        "paths" => o.paths = Some(parse_string_map(value).into_iter().collect()),
        "preserveWatchOutput" => t!(preserve_watch_output),
        "preserveConstEnums" => t!(preserve_const_enums),
        "preserveSymlinks" => t!(preserve_symlinks),
        "project" => s!(project),
        "pretty" => t!(pretty),
        "resolveJsonModule" => t!(resolve_json_module),
        "resolvePackageJsonExports" => t!(resolve_package_json_exports),
        "resolvePackageJsonImports" => t!(resolve_package_json_imports),
        "reactNamespace" => s!(react_namespace),
        "rewriteRelativeImportExtensions" => t!(rewrite_relative_import_extensions),
        "rootDir" => s!(root_dir),
        "rootDirs" => a!(root_dirs),
        "removeComments" => t!(remove_comments),
        "stableTypeOrdering" => t!(stable_type_ordering),
        "strict" => t!(strict),
        "strictBindCallApply" => t!(strict_bind_call_apply),
        "strictBuiltinIteratorReturn" => t!(strict_builtin_iterator_return),
        "strictFunctionTypes" => t!(strict_function_types),
        "strictNullChecks" => t!(strict_null_checks),
        "strictPropertyInitialization" => t!(strict_property_initialization),
        "skipDefaultLibCheck" => t!(skip_default_lib_check),
        "sourceMap" => t!(source_map),
        "sourceRoot" => s!(source_root),
        "stripInternal" => t!(strip_internal),
        "suppressOutputPathCheck" => t!(suppress_output_path_check),
        "target" => e!(target, ScriptTarget),
        "traceResolution" => t!(trace_resolution),
        "tsBuildInfoFile" => s!(ts_build_info_file),
        "typeRoots" => a!(type_roots),
        "types" => a!(types),
        "useDefineForClassFields" => t!(use_define_for_class_fields),
        "useUnknownInCatchVariables" => t!(use_unknown_in_catch_variables),
        "verbatimModuleSyntax" => t!(verbatim_module_syntax),
        "version" => t!(version),
        "help" => t!(help),
        "all" => t!(all),
        "maxNodeModuleJsDepth" => o.max_node_module_js_depth = parse_number(value).map(|n| n as i32),
        "skipLibCheck" => t!(skip_lib_check),
        "noEmit" => t!(no_emit),
        "showConfig" => t!(show_config),
        "configFilePath" => s!(config_file_path),
        "noDtsResolution" => t!(no_dts_resolution),
        "pathsBasePath" => s!(paths_base_path),
        "outDir" => s!(out_dir),
        "newLine" => e!(new_line, NewLineKind),
        "watch" => t!(watch),
        "pprofDir" => s!(pprof_dir),
        "singleThreaded" => t!(single_threaded),
        "quiet" => t!(quiet),
        "checkers" => o.checkers = parse_number(value).map(|n| n as i32),
        "runExternalCode" => t!(run_external_code),
        _ => return false,
    }
    true
}

pub fn float_or_int32_to_flag<T: From<i64>>(value: &Value) -> T { ::tsox_core::fntrace::enter("float_or_int32_to_flag"); 
    if let Some(n) = value.as_i64() {
        return T::from(n);
    }
    T::from(value.as_f64().map(|f| f as i64).unwrap_or(0))
}

pub fn parse_watch_options(key: &str, value: &Value, o: &mut WatchOptions) { ::tsox_core::fntrace::enter("parse_watch_options"); 
    match key {
        "watchInterval" => o.interval = parse_number(value).map(|n| n as i32),
        "watchFile" => {
            if !value.is_null() {
                if let Some(kind) = parse_watch_file_kind(&parse_string(value)) {
                    o.file_kind = kind;
                }
            }
        }
        "watchDirectory" => {
            if !value.is_null() {
                if let Some(kind) = parse_watch_directory_kind(&parse_string(value)) {
                    o.directory_kind = kind;
                }
            }
        }
        "fallbackPolling" => {
            if !value.is_null() {
                if let Some(kind) = parse_polling_kind(&parse_string(value)) {
                    o.fallback_polling = kind;
                }
            }
        }
        "synchronousWatchDirectory" => o.sync_watch_dir = parse_tristate(value),
        "excludeDirectories" => o.exclude_dir = parse_string_array(value),
        "excludeFiles" => o.exclude_files = parse_string_array(value),
        _ => {}
    }
}

pub fn parse_type_acquisition(key: &str, value: &Value, o: &mut tsox_core::core::mig::m3k::TypeAcquisition) { ::tsox_core::fntrace::enter("parse_type_acquisition"); 
    if value.is_null() {
        return;
    }
    match key {
        "enable" => o.enable = parse_tristate(value),
        "include" => o.include = parse_string_array(value),
        "exclude" => o.exclude = parse_string_array(value),
        "disableFilenameBasedTypeAcquisition" => o.disable_filename_based_type_acquisition = parse_tristate(value),
        _ => {}
    }
}

pub fn parse_build_options(key: &str, value: &Value, o: &mut BuildOptions) { ::tsox_core::fntrace::enter("parse_build_options"); 
    if value.is_null() {
        return;
    }
    let key = build_name_map()
        .get(key)
        .map(|option| option.name)
        .unwrap_or(key);
    match key {
        "clean" => o.clean = parse_tristate(value),
        "dry" => o.dry = parse_tristate(value),
        "force" => o.force = parse_tristate(value),
        "builders" => o.builders = parse_number(value).map(|n| n as i32),
        "stopBuildOnErrors" => o.stop_build_on_errors = parse_tristate(value),
        "verbose" => o.verbose = parse_tristate(value),
        _ => {}
    }
}

pub fn merge_compiler_options_full(
    target_options: &mut CompilerOptions,
    source_options: &CompilerOptions,
    raw_source: Option<&Value>,
) { ::tsox_core::fntrace::enter("merge_compiler_options_full"); 
    let mut explicit_null_fields: HashSet<String> = HashSet::new();
    if let Some(raw_source) = raw_source {
        if let Some(compiler_options_raw) = raw_source.get("compilerOptions") {
            if let Some(compiler_options_map) = compiler_options_raw.as_object() {
                for (key, value) in compiler_options_map {
                    if value.is_null() {
                        explicit_null_fields.insert(key.clone());
                    }
                }
            }
        }
    }
    crate::tsoptions::resolve_relative_extends_path::merge_compiler_options(target_options, source_options);
    if !explicit_null_fields.is_empty() {
        zero_fields_with_json_names(target_options, &explicit_null_fields);
    }
}

pub fn zero_fields_with_json_names(o: &mut CompilerOptions, fields: &HashSet<String>) { ::tsox_core::fntrace::enter("zero_fields_with_json_names"); 
    macro_rules! z { ($f:ident, $n:expr) => { if fields.contains($n) { o.$f = Default::default(); } } }
    z!(allow_js, "allowJs"); z!(allow_importing_ts_extensions, "allowImportingTsExtensions"); z!(allow_synthetic_default_imports, "allowSyntheticDefaultImports"); z!(allow_non_ts_extensions, "allowNonTsExtensions"); z!(allow_umd_global_access, "allowUmdGlobalAccess"); z!(allow_unreachable_code, "allowUnreachableCode"); z!(allow_unused_labels, "allowUnusedLabels"); z!(allow_arbitrary_extensions, "allowArbitraryExtensions");
    z!(always_strict, "alwaysStrict"); z!(assume_changes_only_affect_direct_dependencies, "assumeChangesOnlyAffectDirectDependencies"); z!(base_url, "baseUrl"); z!(build, "build"); z!(check_js, "checkJs"); z!(custom_conditions, "customConditions"); z!(composite, "composite"); z!(declaration_dir, "declarationDir");
    z!(deduplicate_packages, "deduplicatePackages"); z!(diagnostics, "diagnostics"); z!(disable_size_limit, "disableSizeLimit"); z!(disable_source_of_project_reference_redirect, "disableSourceOfProjectReferenceRedirect"); z!(disable_solution_searching, "disableSolutionSearching"); z!(disable_referenced_project_load, "disableReferencedProjectLoad"); z!(declaration_map, "declarationMap"); z!(declaration, "declaration");
    z!(downlevel_iteration, "downlevelIteration"); z!(erasable_syntax_only, "erasableSyntaxOnly"); z!(emit_declaration_only, "emitDeclarationOnly"); z!(extended_diagnostics, "extendedDiagnostics"); z!(emit_decorator_metadata, "emitDecoratorMetadata"); z!(emit_bom, "emitBOM"); z!(es_module_interop, "esModuleInterop"); z!(exact_optional_property_types, "exactOptionalPropertyTypes");
    z!(explain_files, "explainFiles"); z!(experimental_decorators, "experimentalDecorators"); z!(force_consistent_casing_in_file_names, "forceConsistentCasingInFileNames"); z!(generate_cpu_profile, "generateCpuProfile"); z!(generate_trace, "generateTrace"); z!(isolated_modules, "isolatedModules"); z!(ignore_config, "ignoreConfig"); z!(ignore_deprecations, "ignoreDeprecations");
    z!(import_helpers, "importHelpers"); z!(incremental, "incremental"); z!(init, "init"); z!(inline_source_map, "inlineSourceMap"); z!(inline_sources, "inlineSources"); z!(isolated_declarations, "isolatedDeclarations"); z!(jsx, "jsx"); z!(jsx_factory, "jsxFactory");
    z!(jsx_fragment_factory, "jsxFragmentFactory"); z!(jsx_import_source, "jsxImportSource"); z!(lib, "lib"); z!(lib_replacement, "libReplacement"); z!(list_emitted_files, "listEmittedFiles"); z!(list_files, "listFiles"); z!(list_files_only, "listFilesOnly"); z!(locale, "locale");
    z!(map_root, "mapRoot"); z!(module, "module"); z!(module_detection, "moduleDetection"); z!(module_resolution, "moduleResolution"); z!(module_suffixes, "moduleSuffixes"); z!(no_check, "noCheck"); z!(no_fallthrough_cases_in_switch, "noFallthroughCasesInSwitch"); z!(no_emit_for_js_files, "noEmitForJsFiles");
    z!(no_error_truncation, "noErrorTruncation"); z!(no_implicit_any, "noImplicitAny"); z!(no_implicit_this, "noImplicitThis"); z!(no_lib, "noLib"); z!(no_property_access_from_index_signature, "noPropertyAccessFromIndexSignature"); z!(no_unchecked_indexed_access, "noUncheckedIndexedAccess"); z!(no_emit_helpers, "noEmitHelpers"); z!(no_emit_on_error, "noEmitOnError");
    z!(no_implicit_returns, "noImplicitReturns"); z!(no_unused_locals, "noUnusedLocals"); z!(no_unused_parameters, "noUnusedParameters"); z!(no_implicit_override, "noImplicitOverride"); z!(no_unchecked_side_effect_imports, "noUncheckedSideEffectImports"); z!(out_file, "outFile"); z!(no_resolve, "noResolve"); z!(paths, "paths");
    z!(preserve_watch_output, "preserveWatchOutput"); z!(preserve_const_enums, "preserveConstEnums"); z!(preserve_symlinks, "preserveSymlinks"); z!(project, "project"); z!(pretty, "pretty"); z!(resolve_json_module, "resolveJsonModule"); z!(resolve_package_json_exports, "resolvePackageJsonExports"); z!(resolve_package_json_imports, "resolvePackageJsonImports");
    z!(react_namespace, "reactNamespace"); z!(rewrite_relative_import_extensions, "rewriteRelativeImportExtensions"); z!(root_dir, "rootDir"); z!(root_dirs, "rootDirs"); z!(remove_comments, "removeComments"); z!(stable_type_ordering, "stableTypeOrdering"); z!(strict, "strict"); z!(strict_bind_call_apply, "strictBindCallApply");
    z!(strict_builtin_iterator_return, "strictBuiltinIteratorReturn"); z!(strict_function_types, "strictFunctionTypes"); z!(strict_null_checks, "strictNullChecks"); z!(strict_property_initialization, "strictPropertyInitialization"); z!(skip_default_lib_check, "skipDefaultLibCheck"); z!(source_map, "sourceMap"); z!(source_root, "sourceRoot"); z!(strip_internal, "stripInternal");
    z!(suppress_output_path_check, "suppressOutputPathCheck"); z!(target, "target"); z!(trace_resolution, "traceResolution"); z!(ts_build_info_file, "tsBuildInfoFile"); z!(type_roots, "typeRoots"); z!(types, "types"); z!(use_define_for_class_fields, "useDefineForClassFields"); z!(use_unknown_in_catch_variables, "useUnknownInCatchVariables");
    z!(verbatim_module_syntax, "verbatimModuleSyntax"); z!(version, "version"); z!(help, "help"); z!(all, "all"); z!(max_node_module_js_depth, "maxNodeModuleJsDepth"); z!(skip_lib_check, "skipLibCheck"); z!(no_emit, "noEmit"); z!(show_config, "showConfig");
    z!(config_file_path, "configFilePath"); z!(no_dts_resolution, "noDtsResolution"); z!(paths_base_path, "pathsBasePath"); z!(out_dir, "outDir"); z!(new_line, "newLine"); z!(watch, "watch"); z!(pprof_dir, "pprofDir"); z!(single_threaded, "singleThreaded");
    z!(quiet, "quiet"); z!(checkers, "checkers"); z!(run_external_code, "runExternalCode");
}

pub fn option_elements(option: &OptionDecl) -> Option<&'static OptionDecl> { ::tsox_core::fntrace::enter("option_elements"); 
    option.elements()
}

pub fn convert_to_options_with_absolute_paths(
    options_base: &mut serde_json::Map<String, Value>,
    option_map: &CommandLineOptionNameMap,
    cwd: &str,
) { ::tsox_core::fntrace::enter("convert_to_options_with_absolute_paths"); 
    for (o, v) in options_base.iter_mut() {
        if let Some(result) = convert_option_to_absolute_path(o, v, option_map, cwd) {
            *v = result;
        }
    }
}

pub fn convert_option_to_absolute_path(
    o: &str,
    v: &Value,
    option_map: &CommandLineOptionNameMap,
    cwd: &str,
) -> Option<Value> { ::tsox_core::fntrace::enter("convert_option_to_absolute_path"); 
    let option = option_map.get(o)?;
    if option.kind == OptionKind::List {
        if option_elements(option).is_some_and(|elements| elements.is_file_path) {
            if let Some(arr) = v.as_array() {
                return Some(Value::Array(
                    arr.iter()
                        .map(|item| match item.as_str() {
                            Some(s) => {
                                Value::String(tsox_core::tspath::get_normalized_absolute_path(s, cwd))
                            }
                            None => item.clone(),
                        })
                        .collect(),
                ));
            }
        }
    } else if option.is_file_path {
        if let Some(value) = v.as_str() {
            return Some(Value::String(tsox_core::tspath::get_normalized_absolute_path(
                value, cwd,
            )));
        }
    }
    None
}
