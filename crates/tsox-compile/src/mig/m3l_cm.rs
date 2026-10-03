use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options_kinds::{
    JsxEmit, ModuleDetectionKind, ModuleKind, ModuleResolutionKind, NewLineKind, ScriptTarget,
};
use tsox_core::core::tristate::Tristate;
use tsox_core::json::marshal;

pub const ERR_PROJECT_UNAVAILABLE: &str = "content mapper project is unavailable";

pub type JsonValue = serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct Definition {
    pub package: String,
    pub extensions: Vec<String>,
    pub options: JsonValue,
}

#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub exec: Vec<String>,
    pub compiler_options: Vec<String>,
    pub dynamic_config: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Mapper {
    pub definition: Definition,
    pub manifest: Manifest,
    pub package_directory: String,
    pub contribution_id: String,
}

const SUPPORTED_VIRTUAL_EXTENSIONS: [&str; 9] = [
    ".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx", ".mts", ".cts", ".json",
];

pub fn is_supported_virtual_extension(extension: &str) -> bool { ::tsox_core::fntrace::enter("is_supported_virtual_extension"); 
    SUPPORTED_VIRTUAL_EXTENSIONS.contains(&extension)
}

impl Mapper {
    pub fn diagnostic_name(&self) -> String { ::tsox_core::fntrace::enter("diagnostic_name"); 
        if !self.manifest.name.is_empty() {
            return self.manifest.name.clone();
        }
        if !self.definition.package.is_empty() {
            return self.definition.package.clone();
        }
        self.contribution_id.clone()
    }

    pub fn identity(&self) -> String { ::tsox_core::fntrace::enter("identity"); 
        if !self.contribution_id.is_empty() {
            return format!("{} ({})", self.contribution_id, self.manifest_identity());
        }
        self.manifest_identity()
    }

    pub fn manifest_identity(&self) -> String { ::tsox_core::fntrace::enter("manifest_identity"); 
        if self.manifest.name.is_empty() {
            return String::new();
        }
        if self.manifest.version.is_empty() {
            return self.manifest.name.clone();
        }
        format!("{}@{}", self.manifest.name, self.manifest.version)
    }

    pub fn transform_identity(&self, options: &CompilerOptions) -> u128 { ::tsox_core::fntrace::enter("transform_identity"); 
        let declared = self.marshal_declared_options(Some(options));
        let options_json = match declared {
            Ok(declared) => match marshal(&declared) {
                Ok(json) => json,
                Err(_) => String::new(),
            },
            Err(_) => String::new(),
        };
        let identity = self.identity();
        let options_raw = self.definition.options.to_string();
        let mut buf =
            Vec::with_capacity(identity.len() + 2 + options_raw.len() + options_json.len());
        buf.extend_from_slice(identity.as_bytes());
        buf.push(0);
        buf.extend_from_slice(options_raw.as_bytes());
        buf.push(0);
        buf.extend_from_slice(options_json.as_bytes());
        xxhash_rust::xxh3::xxh3_128(&buf)
    }

    pub fn marshal_declared_options(
        &self,
        options: Option<&CompilerOptions>,
    ) -> Result<OrderedMap<String, JsonValue>, serde_json::Error> { ::tsox_core::fntrace::enter("marshal_declared_options"); 
        let mut out = OrderedMap::with_capacity(self.manifest.compiler_options.len());
        let options = match options {
            None => return Ok(out),
            Some(options) => options,
        };
        if self.manifest.compiler_options.is_empty() {
            return Ok(out);
        }
        let entries = compiler_options_json_entries(options);
        let by_name: std::collections::HashMap<&str, &JsonValue> =
            entries.iter().map(|(name, value)| (*name, value)).collect();
        for name in &self.manifest.compiler_options {
            if let Some(value) = by_name.get(name.as_str()) {
                out.set(name.clone(), (*value).clone());
            }
        }
        Ok(out)
    }
}

pub fn compiler_options_json_entries(options: &CompilerOptions) -> Vec<(&'static str, JsonValue)> { ::tsox_core::fntrace::enter("compiler_options_json_entries"); 
    let mut entries = Vec::with_capacity(64);
    if !(matches!(options.allow_js, Tristate::Unknown)) {
        entries.push(("allowJs", serde_json::to_value(&options.allow_js).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.allow_arbitrary_extensions, Tristate::Unknown)) {
        entries.push(("allowArbitraryExtensions", serde_json::to_value(&options.allow_arbitrary_extensions).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.allow_importing_ts_extensions, Tristate::Unknown)) {
        entries.push(("allowImportingTsExtensions", serde_json::to_value(&options.allow_importing_ts_extensions).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.allow_non_ts_extensions, Tristate::Unknown)) {
        entries.push(("allowNonTsExtensions", serde_json::to_value(&options.allow_non_ts_extensions).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.allow_umd_global_access, Tristate::Unknown)) {
        entries.push(("allowUmdGlobalAccess", serde_json::to_value(&options.allow_umd_global_access).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.allow_unreachable_code, Tristate::Unknown)) {
        entries.push(("allowUnreachableCode", serde_json::to_value(&options.allow_unreachable_code).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.allow_unused_labels, Tristate::Unknown)) {
        entries.push(("allowUnusedLabels", serde_json::to_value(&options.allow_unused_labels).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.assume_changes_only_affect_direct_dependencies, Tristate::Unknown)) {
        entries.push(("assumeChangesOnlyAffectDirectDependencies", serde_json::to_value(&options.assume_changes_only_affect_direct_dependencies).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.check_js, Tristate::Unknown)) {
        entries.push(("checkJs", serde_json::to_value(&options.check_js).unwrap_or(JsonValue::Null)));
    }
    if !(options.custom_conditions.is_empty()) {
        entries.push(("customConditions", serde_json::to_value(&options.custom_conditions).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.composite, Tristate::Unknown)) {
        entries.push(("composite", serde_json::to_value(&options.composite).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.emit_declaration_only, Tristate::Unknown)) {
        entries.push(("emitDeclarationOnly", serde_json::to_value(&options.emit_declaration_only).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.emit_bom, Tristate::Unknown)) {
        entries.push(("emitBOM", serde_json::to_value(&options.emit_bom).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.emit_decorator_metadata, Tristate::Unknown)) {
        entries.push(("emitDecoratorMetadata", serde_json::to_value(&options.emit_decorator_metadata).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.declaration, Tristate::Unknown)) {
        entries.push(("declaration", serde_json::to_value(&options.declaration).unwrap_or(JsonValue::Null)));
    }
    if !(options.declaration_dir.is_empty()) {
        entries.push(("declarationDir", JsonValue::from(options.declaration_dir.clone())));
    }
    if !(matches!(options.declaration_map, Tristate::Unknown)) {
        entries.push(("declarationMap", serde_json::to_value(&options.declaration_map).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.deduplicate_packages, Tristate::Unknown)) {
        entries.push(("deduplicatePackages", serde_json::to_value(&options.deduplicate_packages).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.disable_size_limit, Tristate::Unknown)) {
        entries.push(("disableSizeLimit", serde_json::to_value(&options.disable_size_limit).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.disable_source_of_project_reference_redirect, Tristate::Unknown)) {
        entries.push(("disableSourceOfProjectReferenceRedirect", serde_json::to_value(&options.disable_source_of_project_reference_redirect).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.disable_solution_searching, Tristate::Unknown)) {
        entries.push(("disableSolutionSearching", serde_json::to_value(&options.disable_solution_searching).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.disable_referenced_project_load, Tristate::Unknown)) {
        entries.push(("disableReferencedProjectLoad", serde_json::to_value(&options.disable_referenced_project_load).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.erasable_syntax_only, Tristate::Unknown)) {
        entries.push(("erasableSyntaxOnly", serde_json::to_value(&options.erasable_syntax_only).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.exact_optional_property_types, Tristate::Unknown)) {
        entries.push(("exactOptionalPropertyTypes", serde_json::to_value(&options.exact_optional_property_types).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.experimental_decorators, Tristate::Unknown)) {
        entries.push(("experimentalDecorators", serde_json::to_value(&options.experimental_decorators).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.force_consistent_casing_in_file_names, Tristate::Unknown)) {
        entries.push(("forceConsistentCasingInFileNames", serde_json::to_value(&options.force_consistent_casing_in_file_names).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.isolated_modules, Tristate::Unknown)) {
        entries.push(("isolatedModules", serde_json::to_value(&options.isolated_modules).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.isolated_declarations, Tristate::Unknown)) {
        entries.push(("isolatedDeclarations", serde_json::to_value(&options.isolated_declarations).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.ignore_config, Tristate::Unknown)) {
        entries.push(("ignoreConfig", serde_json::to_value(&options.ignore_config).unwrap_or(JsonValue::Null)));
    }
    if !(options.ignore_deprecations.is_empty()) {
        entries.push(("ignoreDeprecations", JsonValue::from(options.ignore_deprecations.clone())));
    }
    if !(matches!(options.import_helpers, Tristate::Unknown)) {
        entries.push(("importHelpers", serde_json::to_value(&options.import_helpers).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.inline_source_map, Tristate::Unknown)) {
        entries.push(("inlineSourceMap", serde_json::to_value(&options.inline_source_map).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.inline_sources, Tristate::Unknown)) {
        entries.push(("inlineSources", serde_json::to_value(&options.inline_sources).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.init, Tristate::Unknown)) {
        entries.push(("init", serde_json::to_value(&options.init).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.incremental, Tristate::Unknown)) {
        entries.push(("incremental", serde_json::to_value(&options.incremental).unwrap_or(JsonValue::Null)));
    }
    if !(options.jsx == JsxEmit::default()) {
        entries.push(("jsx", JsonValue::from(options.jsx as i32)));
    }
    if !(options.jsx_factory.is_empty()) {
        entries.push(("jsxFactory", JsonValue::from(options.jsx_factory.clone())));
    }
    if !(options.jsx_fragment_factory.is_empty()) {
        entries.push(("jsxFragmentFactory", JsonValue::from(options.jsx_fragment_factory.clone())));
    }
    if !(options.jsx_import_source.is_empty()) {
        entries.push(("jsxImportSource", JsonValue::from(options.jsx_import_source.clone())));
    }
    if !(options.lib.is_empty()) {
        entries.push(("lib", serde_json::to_value(&options.lib).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.lib_replacement, Tristate::Unknown)) {
        entries.push(("libReplacement", serde_json::to_value(&options.lib_replacement).unwrap_or(JsonValue::Null)));
    }
    if !(options.locale.is_empty()) {
        entries.push(("locale", JsonValue::from(options.locale.clone())));
    }
    if !(options.map_root.is_empty()) {
        entries.push(("mapRoot", JsonValue::from(options.map_root.clone())));
    }
    if !(options.module == ModuleKind::default()) {
        entries.push(("module", JsonValue::from(options.module as i32)));
    }
    if !(options.module_resolution == ModuleResolutionKind::default()) {
        entries.push(("moduleResolution", JsonValue::from(options.module_resolution as i32)));
    }
    if !(options.module_suffixes.is_empty()) {
        entries.push(("moduleSuffixes", serde_json::to_value(&options.module_suffixes).unwrap_or(JsonValue::Null)));
    }
    if !(options.module_detection == ModuleDetectionKind::default()) {
        entries.push(("moduleDetection", JsonValue::from(options.module_detection as i32)));
    }
    if !(options.new_line == NewLineKind::default()) {
        entries.push(("newLine", JsonValue::from(options.new_line as i32)));
    }
    if !(matches!(options.no_emit, Tristate::Unknown)) {
        entries.push(("noEmit", serde_json::to_value(&options.no_emit).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_check, Tristate::Unknown)) {
        entries.push(("noCheck", serde_json::to_value(&options.no_check).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_error_truncation, Tristate::Unknown)) {
        entries.push(("noErrorTruncation", serde_json::to_value(&options.no_error_truncation).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_fallthrough_cases_in_switch, Tristate::Unknown)) {
        entries.push(("noFallthroughCasesInSwitch", serde_json::to_value(&options.no_fallthrough_cases_in_switch).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_implicit_any, Tristate::Unknown)) {
        entries.push(("noImplicitAny", serde_json::to_value(&options.no_implicit_any).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_implicit_this, Tristate::Unknown)) {
        entries.push(("noImplicitThis", serde_json::to_value(&options.no_implicit_this).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_implicit_returns, Tristate::Unknown)) {
        entries.push(("noImplicitReturns", serde_json::to_value(&options.no_implicit_returns).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_emit_helpers, Tristate::Unknown)) {
        entries.push(("noEmitHelpers", serde_json::to_value(&options.no_emit_helpers).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_lib, Tristate::Unknown)) {
        entries.push(("noLib", serde_json::to_value(&options.no_lib).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_property_access_from_index_signature, Tristate::Unknown)) {
        entries.push(("noPropertyAccessFromIndexSignature", serde_json::to_value(&options.no_property_access_from_index_signature).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_unchecked_indexed_access, Tristate::Unknown)) {
        entries.push(("noUncheckedIndexedAccess", serde_json::to_value(&options.no_unchecked_indexed_access).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_emit_on_error, Tristate::Unknown)) {
        entries.push(("noEmitOnError", serde_json::to_value(&options.no_emit_on_error).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_unused_locals, Tristate::Unknown)) {
        entries.push(("noUnusedLocals", serde_json::to_value(&options.no_unused_locals).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_unused_parameters, Tristate::Unknown)) {
        entries.push(("noUnusedParameters", serde_json::to_value(&options.no_unused_parameters).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_resolve, Tristate::Unknown)) {
        entries.push(("noResolve", serde_json::to_value(&options.no_resolve).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_implicit_override, Tristate::Unknown)) {
        entries.push(("noImplicitOverride", serde_json::to_value(&options.no_implicit_override).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_unchecked_side_effect_imports, Tristate::Unknown)) {
        entries.push(("noUncheckedSideEffectImports", serde_json::to_value(&options.no_unchecked_side_effect_imports).unwrap_or(JsonValue::Null)));
    }
    if !(options.out_dir.is_empty()) {
        entries.push(("outDir", JsonValue::from(options.out_dir.clone())));
    }
    if !(matches!(options.preserve_const_enums, Tristate::Unknown)) {
        entries.push(("preserveConstEnums", serde_json::to_value(&options.preserve_const_enums).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.preserve_symlinks, Tristate::Unknown)) {
        entries.push(("preserveSymlinks", serde_json::to_value(&options.preserve_symlinks).unwrap_or(JsonValue::Null)));
    }
    if !(options.project.is_empty()) {
        entries.push(("project", JsonValue::from(options.project.clone())));
    }
    if !(matches!(options.resolve_json_module, Tristate::Unknown)) {
        entries.push(("resolveJsonModule", serde_json::to_value(&options.resolve_json_module).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.resolve_package_json_exports, Tristate::Unknown)) {
        entries.push(("resolvePackageJsonExports", serde_json::to_value(&options.resolve_package_json_exports).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.resolve_package_json_imports, Tristate::Unknown)) {
        entries.push(("resolvePackageJsonImports", serde_json::to_value(&options.resolve_package_json_imports).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.remove_comments, Tristate::Unknown)) {
        entries.push(("removeComments", serde_json::to_value(&options.remove_comments).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.rewrite_relative_import_extensions, Tristate::Unknown)) {
        entries.push(("rewriteRelativeImportExtensions", serde_json::to_value(&options.rewrite_relative_import_extensions).unwrap_or(JsonValue::Null)));
    }
    if !(options.react_namespace.is_empty()) {
        entries.push(("reactNamespace", JsonValue::from(options.react_namespace.clone())));
    }
    if !(options.root_dir.is_empty()) {
        entries.push(("rootDir", JsonValue::from(options.root_dir.clone())));
    }
    if !(options.root_dirs.is_empty()) {
        entries.push(("rootDirs", serde_json::to_value(&options.root_dirs).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.skip_lib_check, Tristate::Unknown)) {
        entries.push(("skipLibCheck", serde_json::to_value(&options.skip_lib_check).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.stable_type_ordering, Tristate::Unknown)) {
        entries.push(("stableTypeOrdering", serde_json::to_value(&options.stable_type_ordering).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.strict, Tristate::Unknown)) {
        entries.push(("strict", serde_json::to_value(&options.strict).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.strict_bind_call_apply, Tristate::Unknown)) {
        entries.push(("strictBindCallApply", serde_json::to_value(&options.strict_bind_call_apply).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.strict_builtin_iterator_return, Tristate::Unknown)) {
        entries.push(("strictBuiltinIteratorReturn", serde_json::to_value(&options.strict_builtin_iterator_return).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.strict_function_types, Tristate::Unknown)) {
        entries.push(("strictFunctionTypes", serde_json::to_value(&options.strict_function_types).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.strict_null_checks, Tristate::Unknown)) {
        entries.push(("strictNullChecks", serde_json::to_value(&options.strict_null_checks).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.strict_property_initialization, Tristate::Unknown)) {
        entries.push(("strictPropertyInitialization", serde_json::to_value(&options.strict_property_initialization).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.strip_internal, Tristate::Unknown)) {
        entries.push(("stripInternal", serde_json::to_value(&options.strip_internal).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.skip_default_lib_check, Tristate::Unknown)) {
        entries.push(("skipDefaultLibCheck", serde_json::to_value(&options.skip_default_lib_check).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.source_map, Tristate::Unknown)) {
        entries.push(("sourceMap", serde_json::to_value(&options.source_map).unwrap_or(JsonValue::Null)));
    }
    if !(options.source_root.is_empty()) {
        entries.push(("sourceRoot", JsonValue::from(options.source_root.clone())));
    }
    if !(matches!(options.suppress_output_path_check, Tristate::Unknown)) {
        entries.push(("suppressOutputPathCheck", serde_json::to_value(&options.suppress_output_path_check).unwrap_or(JsonValue::Null)));
    }
    if !(options.target == ScriptTarget::default()) {
        entries.push(("target", JsonValue::from(options.target as i32)));
    }
    if !(matches!(options.trace_resolution, Tristate::Unknown)) {
        entries.push(("traceResolution", serde_json::to_value(&options.trace_resolution).unwrap_or(JsonValue::Null)));
    }
    if !(options.ts_build_info_file.is_empty()) {
        entries.push(("tsBuildInfoFile", JsonValue::from(options.ts_build_info_file.clone())));
    }
    if !(options.type_roots.is_empty()) {
        entries.push(("typeRoots", serde_json::to_value(&options.type_roots).unwrap_or(JsonValue::Null)));
    }
    if !(options.types.is_empty()) {
        entries.push(("types", serde_json::to_value(&options.types).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.use_define_for_class_fields, Tristate::Unknown)) {
        entries.push(("useDefineForClassFields", serde_json::to_value(&options.use_define_for_class_fields).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.use_unknown_in_catch_variables, Tristate::Unknown)) {
        entries.push(("useUnknownInCatchVariables", serde_json::to_value(&options.use_unknown_in_catch_variables).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.verbatim_module_syntax, Tristate::Unknown)) {
        entries.push(("verbatimModuleSyntax", serde_json::to_value(&options.verbatim_module_syntax).unwrap_or(JsonValue::Null)));
    }
    if !(options.max_node_module_js_depth.is_none()) {
        entries.push(("maxNodeModuleJsDepth", serde_json::to_value(&options.max_node_module_js_depth).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.allow_synthetic_default_imports, Tristate::Unknown)) {
        entries.push(("allowSyntheticDefaultImports", serde_json::to_value(&options.allow_synthetic_default_imports).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.always_strict, Tristate::Unknown)) {
        entries.push(("alwaysStrict", serde_json::to_value(&options.always_strict).unwrap_or(JsonValue::Null)));
    }
    if !(options.base_url.is_empty()) {
        entries.push(("baseUrl", JsonValue::from(options.base_url.clone())));
    }
    if !(matches!(options.downlevel_iteration, Tristate::Unknown)) {
        entries.push(("downlevelIteration", serde_json::to_value(&options.downlevel_iteration).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.es_module_interop, Tristate::Unknown)) {
        entries.push(("esModuleInterop", serde_json::to_value(&options.es_module_interop).unwrap_or(JsonValue::Null)));
    }
    if !(options.out_file.is_empty()) {
        entries.push(("outFile", JsonValue::from(options.out_file.clone())));
    }
    if !(options.config_file_path.is_empty()) {
        entries.push(("configFilePath", JsonValue::from(options.config_file_path.clone())));
    }
    if !(matches!(options.no_dts_resolution, Tristate::Unknown)) {
        entries.push(("noDtsResolution", serde_json::to_value(&options.no_dts_resolution).unwrap_or(JsonValue::Null)));
    }
    if !(options.paths_base_path.is_empty()) {
        entries.push(("pathsBasePath", JsonValue::from(options.paths_base_path.clone())));
    }
    if !(matches!(options.diagnostics, Tristate::Unknown)) {
        entries.push(("diagnostics", serde_json::to_value(&options.diagnostics).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.extended_diagnostics, Tristate::Unknown)) {
        entries.push(("extendedDiagnostics", serde_json::to_value(&options.extended_diagnostics).unwrap_or(JsonValue::Null)));
    }
    if !(options.generate_cpu_profile.is_empty()) {
        entries.push(("generateCpuProfile", JsonValue::from(options.generate_cpu_profile.clone())));
    }
    if !(options.generate_trace.is_empty()) {
        entries.push(("generateTrace", JsonValue::from(options.generate_trace.clone())));
    }
    if !(matches!(options.list_emitted_files, Tristate::Unknown)) {
        entries.push(("listEmittedFiles", serde_json::to_value(&options.list_emitted_files).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.list_files, Tristate::Unknown)) {
        entries.push(("listFiles", serde_json::to_value(&options.list_files).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.explain_files, Tristate::Unknown)) {
        entries.push(("explainFiles", serde_json::to_value(&options.explain_files).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.list_files_only, Tristate::Unknown)) {
        entries.push(("listFilesOnly", serde_json::to_value(&options.list_files_only).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.no_emit_for_js_files, Tristate::Unknown)) {
        entries.push(("noEmitForJsFiles", serde_json::to_value(&options.no_emit_for_js_files).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.preserve_watch_output, Tristate::Unknown)) {
        entries.push(("preserveWatchOutput", serde_json::to_value(&options.preserve_watch_output).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.pretty, Tristate::Unknown)) {
        entries.push(("pretty", serde_json::to_value(&options.pretty).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.version, Tristate::Unknown)) {
        entries.push(("version", serde_json::to_value(&options.version).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.watch, Tristate::Unknown)) {
        entries.push(("watch", serde_json::to_value(&options.watch).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.show_config, Tristate::Unknown)) {
        entries.push(("showConfig", serde_json::to_value(&options.show_config).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.build, Tristate::Unknown)) {
        entries.push(("build", serde_json::to_value(&options.build).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.help, Tristate::Unknown)) {
        entries.push(("help", serde_json::to_value(&options.help).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.all, Tristate::Unknown)) {
        entries.push(("all", serde_json::to_value(&options.all).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.run_external_code, Tristate::Unknown)) {
        entries.push(("runExternalCode", serde_json::to_value(&options.run_external_code).unwrap_or(JsonValue::Null)));
    }
    if !(options.pprof_dir.is_empty()) {
        entries.push(("pprofDir", JsonValue::from(options.pprof_dir.clone())));
    }
    if !(matches!(options.single_threaded, Tristate::Unknown)) {
        entries.push(("singleThreaded", serde_json::to_value(&options.single_threaded).unwrap_or(JsonValue::Null)));
    }
    if !(matches!(options.quiet, Tristate::Unknown)) {
        entries.push(("quiet", serde_json::to_value(&options.quiet).unwrap_or(JsonValue::Null)));
    }
    if !(options.checkers.is_none()) {
        entries.push(("checkers", serde_json::to_value(&options.checkers).unwrap_or(JsonValue::Null)));
    }
    entries
}

pub fn compiler_options_to_json(options: &CompilerOptions) -> JsonValue { ::tsox_core::fntrace::enter("compiler_options_to_json"); 
    let mut map = serde_json::Map::new();
    for (name, value) in compiler_options_json_entries(options) {
        map.insert(name.to_string(), value);
    }
    JsonValue::Object(map)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TransformErrorKind {
    #[default]
    Unknown,
    Initialize,
    Project,
    Request,
    Response,
    Mappings,
}

#[derive(Debug, Default)]
pub struct TransformError {
    pub kind: TransformErrorKind,
    err: Option<Box<dyn std::error::Error + Send + Sync>>,
}

pub fn new_transform_error(
    kind: TransformErrorKind,
    err: Box<dyn std::error::Error + Send + Sync>,
) -> TransformError { ::tsox_core::fntrace::enter("new_transform_error"); 
    TransformError {
        kind,
        err: Some(err),
    }
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        match &self.err {
            Some(err) => write!(f, "content mapper transform failed: {}", err),
            None => write!(f, "content mapper transform failed"),
        }
    }
}

impl std::error::Error for TransformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { ::tsox_core::fntrace::enter("source"); 
        self.err.as_ref().map(|err| err.as_ref() as _)
    }
}

impl TransformError {
    pub fn unwrap_err(&self) -> Option<&(dyn std::error::Error + 'static)> { ::tsox_core::fntrace::enter("unwrap_err"); 
        self.err.as_ref().map(|err| err.as_ref() as _)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiagnosticDirectiveErrorKind {
    #[default]
    InvalidRange,
    InvalidPolicy,
    ExpectMissingUnusedDiagnostic,
    InvalidUnusedDiagnosticIndex,
    Overlap,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiagnosticDirectivePolicy {
    #[default]
    Ignore,
    Expect,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DiagnosticDirectiveError {
    pub kind: DiagnosticDirectiveErrorKind,
    pub index: usize,
    pub supplemental_index: isize,
    pub policy: DiagnosticDirectivePolicy,
}

impl fmt::Display for DiagnosticDirectiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        write!(f, "invalid content mapper diagnostic directive {}", self.index)
    }
}

impl std::error::Error for DiagnosticDirectiveError {}

#[derive(Debug, Clone, Default)]
pub struct InvalidVirtualExtensionError {
    pub extension: String,
}

impl fmt::Display for InvalidVirtualExtensionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        write!(f, "invalid virtual extension {:?}", self.extension)
    }
}

impl std::error::Error for InvalidVirtualExtensionError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProjectErrorKind {
    #[default]
    MalformedResponse,
    MissingConfigIdentity,
    NonAbsoluteWatchedFile,
    UnexpectedConfigIdentity,
    UnexpectedWatchedFiles,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProjectError {
    pub kind: ProjectErrorKind,
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        match self.kind {
            ProjectErrorKind::MalformedResponse => {
                write!(f, "content mapper returned a malformed project response")
            }
            ProjectErrorKind::MissingConfigIdentity => write!(
                f,
                "content mapper did not return configIdentity for dynamic configuration"
            ),
            ProjectErrorKind::NonAbsoluteWatchedFile => write!(
                f,
                "content mapper returned a non-absolute path in watchedFiles"
            ),
            ProjectErrorKind::UnexpectedConfigIdentity => write!(
                f,
                "content mapper returned configIdentity without declaring dynamicConfig"
            ),
            ProjectErrorKind::UnexpectedWatchedFiles => write!(
                f,
                "content mapper returned watchedFiles without declaring dynamicConfig"
            ),
        }
    }
}

impl std::error::Error for ProjectError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InitializeErrorKind {
    #[default]
    ProcessStart,
    ProcessExit,
    NoResponse,
    InvalidResponse,
    Request,
    PositionEncoding,
    EmptyDiagnosticSource,
    ReservedDiagnosticSource,
}

#[derive(Debug, Clone, Default)]
pub struct InitializeError {
    pub kind: InitializeErrorKind,
    pub mapper_name: String,
    pub command: String,
    pub detail: String,
    pub exit_code: i32,
    pub timeout_seconds: i64,
    pub position_encoding: String,
    pub diagnostic_source: String,
}

impl fmt::Display for InitializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        match self.kind {
            InitializeErrorKind::ProcessStart => write!(
                f,
                "could not start content mapper command {:?}: {}",
                self.command, self.detail
            ),
            InitializeErrorKind::ProcessExit => write!(
                f,
                "content mapper process exited before initialization with code {}",
                self.exit_code
            ),
            InitializeErrorKind::NoResponse => write!(
                f,
                "content mapper did not respond to the initialize request"
            ),
            InitializeErrorKind::InvalidResponse => write!(
                f,
                "content mapper returned an invalid initialize response: {}",
                self.detail
            ),
            InitializeErrorKind::Request => write!(
                f,
                "content mapper initialize request failed: {}",
                self.detail
            ),
            InitializeErrorKind::PositionEncoding => {
                write!(f, "unsupported position encoding {:?}", self.position_encoding)
            }
            InitializeErrorKind::EmptyDiagnosticSource => {
                write!(f, "diagnostic source must not be empty")
            }
            InitializeErrorKind::ReservedDiagnosticSource => write!(
                f,
                "diagnostic source {:?} is reserved by TypeScript",
                self.diagnostic_source
            ),
        }
    }
}

impl std::error::Error for InitializeError {}

#[derive(Debug, Clone, Default)]
pub struct SupplementalFileCollisionError {
    pub file_name: String,
}

impl fmt::Display for SupplementalFileCollisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        write!(
            f,
            "content mapper supplemental output file {:?} already exists",
            self.file_name
        )
    }
}

impl std::error::Error for SupplementalFileCollisionError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperationTiming {
    pub count: u64,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MapperTimings {
    pub spawn: OperationTiming,
    pub initialize: OperationTiming,
    pub open_project: OperationTiming,
    pub close_project: OperationTiming,
    pub transform: OperationTiming,
}

#[derive(Debug, Clone, Default)]
pub struct Timings {
    pub mappers: HashMap<String, MapperTimings>,
    pub request_wait: Duration,
}

impl Timings {
    pub fn since(&self, previous: &Timings) -> Timings { ::tsox_core::fntrace::enter("since"); 
        let mut result = Timings {
            mappers: HashMap::with_capacity(self.mappers.len()),
            request_wait: self.request_wait.saturating_sub(previous.request_wait),
        };
        for (identity, current) in &self.mappers {
            let before = previous.mappers.get(identity).copied().unwrap_or_default();
            result.mappers.insert(
                identity.clone(),
                MapperTimings {
                    spawn: operation_timing_since(current.spawn, before.spawn),
                    initialize: operation_timing_since(current.initialize, before.initialize),
                    open_project: operation_timing_since(current.open_project, before.open_project),
                    close_project: operation_timing_since(
                        current.close_project,
                        before.close_project,
                    ),
                    transform: operation_timing_since(current.transform, before.transform),
                },
            );
        }
        result
    }
}

pub fn operation_timing_since(
    current: OperationTiming,
    previous: OperationTiming,
) -> OperationTiming { ::tsox_core::fntrace::enter("operation_timing_since"); 
    OperationTiming {
        count: current.count.saturating_sub(previous.count),
        duration: current.duration.saturating_sub(previous.duration),
    }
}
