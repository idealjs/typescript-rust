#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::{Arc, LazyLock};

use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::collections::set::Set;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options_kinds::{ModuleResolutionKind, ResolutionMode, ScriptTarget};
use tsox_core::core::text::TextRange;
use tsox_core::core::tristate::Tristate;
use tsox_core::diagnostics::{Category, Message};
use tsox_frontend::ast::diagnostic::Diagnostic;

use super::m5h_5::LIB_MAP;
use super::m5j_2::ParseConfigHost;
use crate::module::{ResolutionHost, Resolver};
use crate::packagejson;
use crate::tsoptions::build_options::OptValue;
use crate::tsoptions::option_kind::{ExtraValidation, OptionDecl, OptionKind};
use crate::tsoptions::options_options::OPTIONS;
use crate::vfs::fs::FS;

impl OptionDecl {
    pub fn deprecated_keys(&self) -> Option<&'static Set<String>> {
        if self.kind != OptionKind::Enum {
            return None;
        }
        command_line_option_deprecated().get(&self.name)
    }

    pub fn enum_map(&self) -> Option<&'static OrderedMap<String, OptValue>> {
        if self.kind != OptionKind::Enum {
            return None;
        }
        command_line_option_enum_map().get(&self.name)
    }

    pub fn elements(&self) -> Option<&'static OptionDecl> {
        if !matches!(self.kind, OptionKind::List | OptionKind::ListOrElement) {
            return None;
        }
        command_line_option_elements().get(&self.name)
    }
}

pub struct NameMap {
    pub options_names: OrderedMap<String, OptionDecl>,
    pub short_option_names: std::collections::HashMap<String, String>,
}

impl NameMap {
    pub fn get(&self, name: &str) -> Option<&OptionDecl> {
        self.options_names.get(&name.to_lowercase())
    }

    pub fn get_from_short(&self, short_name: &str) -> Option<&OptionDecl> {
        let name = self.short_option_names.get(short_name)?;
        self.get(name)
    }

    pub fn get_option_declaration_from_name(
        &self,
        option_name: &str,
        allow_short: bool,
    ) -> Option<&OptionDecl> {
        let mut option_name = option_name.to_lowercase();
        if allow_short {
            if let Some(short) = self.short_option_names.get(&option_name) {
                option_name = short.clone();
            }
        }
        self.get(&option_name)
    }

    pub fn get_spelling_suggestion(&self, name: &str) -> Option<&OptionDecl> {
        tsox_core::core::mig::m3j_2::get_spelling_suggestion(
            name,
            self.options_names.values(),
            |option| option.name.to_string(),
            |a, b| a.name.cmp(b.name),
        )
    }
}

pub fn get_name_map_from_list(opt_decls: &[OptionDecl]) -> NameMap {
    let mut options_names = OrderedMap::with_capacity(opt_decls.len());
    let mut short_option_names = std::collections::HashMap::new();
    for option in opt_decls {
        options_names.set(option.name.to_lowercase(), *option);
        if let Some(short) = option.short_name {
            short_option_names.insert(short.to_string(), option.name.to_string());
        }
    }
    NameMap {
        options_names,
        short_option_names,
    }
}

pub fn target_to_lib_map() -> OrderedMap<ScriptTarget, String> {
    let mut map = OrderedMap::new();
    map.set(ScriptTarget::ESNext, "lib.esnext.full.d.ts".to_string());
    map.set(ScriptTarget::ES2025, "lib.es2025.full.d.ts".to_string());
    map.set(ScriptTarget::ES2024, "lib.es2024.full.d.ts".to_string());
    map.set(ScriptTarget::ES2023, "lib.es2023.full.d.ts".to_string());
    map.set(ScriptTarget::ES2022, "lib.es2022.full.d.ts".to_string());
    map.set(ScriptTarget::ES2021, "lib.es2021.full.d.ts".to_string());
    map.set(ScriptTarget::ES2020, "lib.es2020.full.d.ts".to_string());
    map.set(ScriptTarget::ES2019, "lib.es2019.full.d.ts".to_string());
    map.set(ScriptTarget::ES2018, "lib.es2018.full.d.ts".to_string());
    map.set(ScriptTarget::ES2017, "lib.es2017.full.d.ts".to_string());
    map.set(ScriptTarget::ES2016, "lib.es2016.full.d.ts".to_string());
    map.set(ScriptTarget::ES2015, "lib.es6.d.ts".to_string());
    map
}

pub fn get_default_lib_file_name(options: &CompilerOptions) -> String {
    let map = target_to_lib_map();
    match map.get(&options.get_emit_script_target()) {
        Some(name) => name.clone(),
        None => "lib.d.ts".to_string(),
    }
}

pub fn get_lib_file_name(lib_name: &str) -> Option<String> {
    let lib_name = tsox_core::tspath::to_file_name_lower_case(lib_name);
    let mut lib_files_set = Set::new();
    for (_, file) in LIB_MAP.iter() {
        lib_files_set.insert(file.to_string());
    }
    if lib_files_set.has(&lib_name) {
        return Some(lib_name);
    }
    LIB_MAP
        .iter()
        .find(|(key, _)| *key == lib_name)
        .map(|(_, file)| file.to_string())
}

fn option_strict_flag(name: &str) -> bool {
    matches!(
        name,
        "strict"
            | "strictNullChecks"
            | "strictFunctionTypes"
            | "strictBindCallApply"
            | "strictPropertyInitialization"
            | "strictBuiltinIteratorReturn"
            | "noImplicitAny"
            | "noImplicitThis"
            | "useUnknownInCatchVariables"
            | "alwaysStrict"
    )
}

fn value_as_tristate(value: &OptValue) -> Tristate {
    match value {
        OptValue::Bool(true) => Tristate::True,
        OptValue::Bool(false) => Tristate::False,
        _ => Tristate::Unknown,
    }
}

fn opt_value_eq(a: &OptValue, b: &OptValue) -> bool {
    match (a, b) {
        (OptValue::Bool(x), OptValue::Bool(y)) => x == y,
        (OptValue::Num(x), OptValue::Num(y)) => x == y,
        (OptValue::Str(x), OptValue::Str(y)) => x == y,
        (OptValue::List(x), OptValue::List(y)) => x == y,
        (OptValue::Null, OptValue::Null) => true,
        _ => false,
    }
}

pub fn options_have_changes(
    old_options: Option<&CompilerOptions>,
    new_options: Option<&CompilerOptions>,
    decl_filter: &dyn Fn(&OptionDecl) -> bool,
) -> bool {
    if old_options.is_some() && std::ptr::eq(old_options.unwrap(), new_options.unwrap()) {
        return false;
    }
    let (old_options, new_options) = match (old_options, new_options) {
        (Some(o), Some(n)) => (o, n),
        _ => return true,
    };
    for_each_compiler_option_value(new_options, decl_filter, &mut |option, value, _i| {
        let old_value = get_compiler_options_value(old_options, option.name);
        if option_strict_flag(option.name) {
            return old_options.get_strict_option_value(value_as_tristate(&old_value))
                != new_options.get_strict_option_value(value_as_tristate(value));
        }
        if option.name == "allowJs" {
            return old_options.get_allow_js() != new_options.get_allow_js();
        }
        !opt_value_eq(&old_value, value)
    })
}

pub fn compiler_options_field_names() -> Vec<&'static str> {
    OPTIONS.iter().map(|o| o.name).collect()
}

pub fn command_line_compiler_options_map() -> &'static OrderedMap<&'static str, &'static OptionDecl>
{
    static MAP: LazyLock<OrderedMap<&'static str, &'static OptionDecl>> = LazyLock::new(|| {
        let mut map = OrderedMap::with_capacity(OPTIONS.len() * 2);
        for option in OPTIONS.iter() {
            map.set(option.name, option);
            map.set(option.name.to_lowercase().leak() as &'static str, option);
        }
        map
    });
    &MAP
}

pub fn for_each_compiler_option_value(
    options: &CompilerOptions,
    decl_filter: &dyn Fn(&OptionDecl) -> bool,
    fn_: &mut dyn FnMut(&OptionDecl, &OptValue, usize) -> bool,
) -> bool {
    for (i, name) in compiler_options_field_names().iter().enumerate() {
        if let Some(option_declaration) = command_line_compiler_options_map().get(name) {
            if decl_filter(option_declaration) {
                let value = get_compiler_options_value(options, option_declaration.name);
                if fn_(option_declaration, &value, i) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn get_compiler_options_value(options: &CompilerOptions, name: &str) -> OptValue {
    macro_rules! t {
        ($f:ident) => {
            match options.$f {
                Tristate::True => OptValue::Bool(true),
                Tristate::False => OptValue::Bool(false),
                Tristate::Unknown => OptValue::Null,
            }
        };
    }
    macro_rules! s {
        ($f:ident) => {
            OptValue::Str(options.$f.clone())
        };
    }
    macro_rules! e {
        ($f:ident) => {
            OptValue::Num(options.$f as i64)
        };
    }
    macro_rules! a {
        ($f:ident) => {
            OptValue::List(options.$f.clone())
        };
    }
    match name {
        "assumeChangesOnlyAffectDirectDependencies" => {
            t!(assume_changes_only_affect_direct_dependencies)
        }
        "checkJs" => t!(check_js),
        "noImplicitAny" => t!(no_implicit_any),
        "noImplicitThis" => t!(no_implicit_this),
        "noImplicitReturns" => t!(no_implicit_returns),
        "noImplicitOverride" => t!(no_implicit_override),
        "noUnusedLocals" => t!(no_unused_locals),
        "noUnusedParameters" => t!(no_unused_parameters),
        "noFallthroughCasesInSwitch" => t!(no_fallthrough_cases_in_switch),
        "noUncheckedIndexedAccess" => t!(no_unchecked_indexed_access),
        "noPropertyAccessFromIndexSignature" => t!(no_property_access_from_index_signature),
        "noUncheckedSideEffectImports" => t!(no_unchecked_side_effect_imports),
        "strict" => t!(strict),
        "strictNullChecks" => t!(strict_null_checks),
        "strictFunctionTypes" => t!(strict_function_types),
        "strictBindCallApply" => t!(strict_bind_call_apply),
        "strictPropertyInitialization" => t!(strict_property_initialization),
        "strictBuiltinIteratorReturn" => t!(strict_builtin_iterator_return),
        "alwaysStrict" => t!(always_strict),
        "useUnknownInCatchVariables" => t!(use_unknown_in_catch_variables),
        "useDefineForClassFields" => t!(use_define_for_class_fields),
        "exactOptionalPropertyTypes" => t!(exact_optional_property_types),
        "noEmit" => t!(no_emit),
        "noEmitHelpers" => t!(no_emit_helpers),
        "noEmitOnError" => t!(no_emit_on_error),
        "noCheck" => t!(no_check),
        "noErrorTruncation" => t!(no_error_truncation),
        "noResolve" => t!(no_resolve),
        "noLib" => t!(no_lib),
        "skipLibCheck" => t!(skip_lib_check),
        "skipDefaultLibCheck" => t!(skip_default_lib_check),
        "incremental" => t!(incremental),
        "composite" => t!(composite),
        "declaration" => t!(declaration),
        "declarationMap" => t!(declaration_map),
        "emitDeclarationOnly" => t!(emit_declaration_only),
        "emitBOM" => t!(emit_bom),
        "emitDecoratorMetadata" => t!(emit_decorator_metadata),
        "experimentalDecorators" => t!(experimental_decorators),
        "esModuleInterop" => t!(es_module_interop),
        "allowSyntheticDefaultImports" => t!(allow_synthetic_default_imports),
        "allowJs" => t!(allow_js),
        "allowImportingTsExtensions" => t!(allow_importing_ts_extensions),
        "allowUnreachableCode" => t!(allow_unreachable_code),
        "allowUnusedLabels" => t!(allow_unused_labels),
        "allowUmdGlobalAccess" => t!(allow_umd_global_access),
        "allowArbitraryExtensions" => t!(allow_arbitrary_extensions),
        "allowNonTsExtensions" => t!(allow_non_ts_extensions),
        "sourceMap" => t!(source_map),
        "inlineSourceMap" => t!(inline_source_map),
        "inlineSources" => t!(inline_sources),
        "removeComments" => t!(remove_comments),
        "isolatedModules" => t!(isolated_modules),
        "isolatedDeclarations" => t!(isolated_declarations),
        "verbatimModuleSyntax" => t!(verbatim_module_syntax),
        "preserveConstEnums" => t!(preserve_const_enums),
        "preserveSymlinks" => t!(preserve_symlinks),
        "importHelpers" => t!(import_helpers),
        "forceConsistentCasingInFileNames" => t!(force_consistent_casing_in_file_names),
        "resolveJsonModule" => t!(resolve_json_module),
        "resolvePackageJsonExports" => t!(resolve_package_json_exports),
        "resolvePackageJsonImports" => t!(resolve_package_json_imports),
        "erasableSyntaxOnly" => t!(erasable_syntax_only),
        "stableTypeOrdering" => t!(stable_type_ordering),
        "rewriteRelativeImportExtensions" => t!(rewrite_relative_import_extensions),
        "stripInternal" => t!(strip_internal),
        "runExternalCode" => t!(run_external_code),
        "ignoreConfig" => t!(ignore_config),
        "noDtsResolution" => t!(no_dts_resolution),
        "deduplicatePackages" => t!(deduplicate_packages),
        "diagnostics" => t!(diagnostics),
        "extendedDiagnostics" => t!(extended_diagnostics),
        "declarationDir" => s!(declaration_dir),
        "outDir" => s!(out_dir),
        "outFile" => s!(out_file),
        "rootDir" => s!(root_dir),
        "baseUrl" => s!(base_url),
        "tsBuildInfoFile" => s!(ts_build_info_file),
        "sourceRoot" => s!(source_root),
        "mapRoot" => s!(map_root),
        "jsxFactory" => s!(jsx_factory),
        "jsxFragmentFactory" => s!(jsx_fragment_factory),
        "jsxImportSource" => s!(jsx_import_source),
        "reactNamespace" => s!(react_namespace),
        "locale" => s!(locale),
        "project" => s!(project),
        "configFilePath" => s!(config_file_path),
        "pathsBasePath" => s!(paths_base_path),
        "target" => e!(target),
        "module" => e!(module),
        "moduleResolution" => e!(module_resolution),
        "jsx" => e!(jsx),
        "newLine" => e!(new_line),
        "moduleDetection" => e!(module_detection),
        "lib" => a!(lib),
        "types" => a!(types),
        "typeRoots" => a!(type_roots),
        "rootDirs" => a!(root_dirs),
        "moduleSuffixes" => a!(module_suffixes),
        "customConditions" => a!(custom_conditions),
        "maxNodeModuleJsDepth" => match options.max_node_module_js_depth {
            Some(n) => OptValue::Num(n as i64),
            None => OptValue::Null,
        },
        _ => OptValue::Null,
    }
}

fn option_affects_semantic_diagnostics(name: &str) -> bool {
    matches!(
        name,
        "allowImportingTsExtensions"
            | "allowSyntheticDefaultImports"
            | "allowUmdGlobalAccess"
            | "allowUnreachableCode"
            | "allowUnusedLabels"
            | "assumeChangesOnlyAffectDirectDependencies"
            | "checkJs"
            | "emitDecoratorMetadata"
            | "erasableSyntaxOnly"
            | "esModuleInterop"
            | "exactOptionalPropertyTypes"
            | "experimentalDecorators"
            | "isolatedDeclarations"
            | "jsxImportSource"
            | "noErrorTruncation"
            | "noFallthroughCasesInSwitch"
            | "noImplicitAny"
            | "noImplicitOverride"
            | "noImplicitReturns"
            | "noImplicitThis"
            | "noPropertyAccessFromIndexSignature"
            | "noUncheckedIndexedAccess"
            | "noUncheckedSideEffectImports"
            | "noUnusedLocals"
            | "noUnusedParameters"
            | "rewriteRelativeImportExtensions"
            | "stableTypeOrdering"
            | "strictBindCallApply"
            | "strictBuiltinIteratorReturn"
            | "strictFunctionTypes"
            | "strictNullChecks"
            | "strictPropertyInitialization"
            | "useDefineForClassFields"
            | "useUnknownInCatchVariables"
            | "verbatimModuleSyntax"
    )
}

fn option_affects_declaration_path(name: &str) -> bool {
    matches!(name, "declarationDir" | "outDir" | "outFile" | "rootDir")
}

fn option_affects_emit(name: &str) -> bool {
    matches!(
        name,
        "alwaysStrict"
            | "assumeChangesOnlyAffectDirectDependencies"
            | "declarationDir"
            | "downlevelIteration"
            | "emitBOM"
            | "emitDecoratorMetadata"
            | "esModuleInterop"
            | "experimentalDecorators"
            | "importHelpers"
            | "inlineSources"
            | "jsx"
            | "jsxImportSource"
            | "mapRoot"
            | "module"
            | "newLine"
            | "noEmitHelpers"
            | "noEmitOnError"
            | "outDir"
            | "outFile"
            | "preserveConstEnums"
            | "reactNamespace"
            | "removeComments"
            | "rootDir"
            | "sourceRoot"
            | "stripInternal"
            | "target"
            | "tsBuildInfoFile"
            | "useDefineForClassFields"
            | "verbatimModuleSyntax"
    )
}

pub fn compiler_options_affect_semantic_diagnostics(
    old_options: Option<&CompilerOptions>,
    new_options: Option<&CompilerOptions>,
) -> bool {
    options_have_changes(old_options, new_options, &|option| {
        option_affects_semantic_diagnostics(option.name)
    })
}

pub fn compiler_options_affect_declaration_path(
    old_options: Option<&CompilerOptions>,
    new_options: Option<&CompilerOptions>,
) -> bool {
    options_have_changes(old_options, new_options, &|option| {
        option_affects_declaration_path(option.name)
    })
}

pub fn compiler_options_affect_emit(
    old_options: Option<&CompilerOptions>,
    new_options: Option<&CompilerOptions>,
) -> bool {
    options_have_changes(old_options, new_options, &|option| {
        option_affects_emit(option.name)
    })
}

fn command_line_option_deprecated() -> &'static OrderedMap<&'static str, Set<String>> {
    static MAP: LazyLock<OrderedMap<&'static str, Set<String>>> = LazyLock::new(|| {
        let mut map = OrderedMap::new();
        for (name, items) in [
            ("module", &["none", "amd", "system", "umd"][..]),
            ("moduleResolution", &["node", "classic", "node10"][..]),
            ("target", &["es5"][..]),
        ] {
            let mut set = Set::new();
            for item in items {
                set.insert(item.to_string());
            }
            map.set(name, set);
        }
        map
    });
    &MAP
}

fn command_line_option_enum_map()
-> &'static OrderedMap<&'static str, OrderedMap<String, OptValue>> {
    static MAP: LazyLock<OrderedMap<&'static str, OrderedMap<String, OptValue>>> =
        LazyLock::new(|| {
            let mut map = OrderedMap::new();
            for name in [
                "lib",
                "moduleResolution",
                "module",
                "target",
                "moduleDetection",
                "jsx",
                "newLine",
                "watchFile",
                "watchDirectory",
                "fallbackPolling",
            ] {
                let Some(decl) = OPTIONS.iter().find(|o| o.name == name) else {
                    continue;
                };
                let Some(enum_values) = decl.enum_values else {
                    continue;
                };
                let mut values = OrderedMap::new();
                for (i, key) in enum_values.iter().enumerate() {
                    values.set(key.to_string(), OptValue::Num(i as i64));
                }
                map.set(decl.name, values);
            }
            map
        });
    &MAP
}

fn command_line_option_elements() -> &'static OrderedMap<&'static str, OptionDecl> {
    static MAP: LazyLock<OrderedMap<&'static str, OptionDecl>> = LazyLock::new(|| {
        let entries: &[(&str, &str, OptionKind, bool)] = &[
            ("lib", "lib", OptionKind::Enum, false),
            ("rootDirs", "rootDirs", OptionKind::String, true),
            ("typeRoots", "typeRoots", OptionKind::String, true),
            ("types", "types", OptionKind::String, false),
            ("moduleSuffixes", "moduleSuffixes", OptionKind::String, false),
            ("customConditions", "condition", OptionKind::String, false),
            ("plugins", "plugin", OptionKind::List, false),
            ("references", "references", OptionKind::List, false),
            ("contentMappers", "contentMappers", OptionKind::List, false),
            ("files", "files", OptionKind::String, false),
            ("include", "include", OptionKind::String, false),
            ("exclude", "exclude", OptionKind::String, false),
            ("extends", "extends", OptionKind::String, false),
            (
                "excludeDirectories",
                "excludeDirectory",
                OptionKind::String,
                true,
            ),
            ("excludeFiles", "excludeFile", OptionKind::String, true),
            ("libFiles", "libFiles", OptionKind::String, false),
        ];
        let mut map = OrderedMap::new();
        for (key, name, kind, is_file_path) in entries {
            map.set(
                *key,
                OptionDecl {
                    name,
                    short_name: None,
                    kind: *kind,
                    is_file_path: *is_file_path,
                    is_tsconfig_only: false,
                    is_command_line_only: false,
                    extra_validation: ExtraValidation::None,
                    min_value: None,
                    enum_values: None,
                    description: "",
                    show_in_simplified_help: false,
                },
            );
        }
        map
    });
    &MAP
}

const THE_CONTENT_MAPPER_PACKAGE_0_COULD_NOT_BE_RESOLVED: Message = Message {
    code: 100031,
    category: Category::Error,
    key: "The_content_mapper_package_0_could_not_be_resolved_100031",
    text: "The content mapper package '{0}' could not be resolved.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

const THE_PACKAGE_JSON_OF_THE_CONTENT_MAPPER_PACKAGE_0_COULD_NOT_BE_PARSED: Message = Message {
    code: 100032,
    category: Category::Error,
    key: "The_package_json_of_the_content_mapper_package_0_could_not_be_parsed_100032",
    text: "The 'package.json' of the content mapper package '{0}' could not be parsed.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

const THE_PACKAGE_JSON_OF_THE_CONTENT_MAPPER_PACKAGE_0_DOES_NOT_SPECIFY_A_NAME: Message = Message {
    code: 100033,
    category: Category::Error,
    key: "The_package_json_of_the_content_mapper_package_0_does_not_specify_a_name_100033",
    text: "The 'package.json' of the content mapper package '{0}' does not specify a 'name'.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

const THE_PACKAGE_JSON_OF_THE_CONTENT_MAPPER_PACKAGE_0_DOES_NOT_DECLARE_A_TYPESCRIPT_CONTENT_MAPPER_OBJECT: Message = Message {
    code: 100034,
    category: Category::Error,
    key: "The_package_json_of_the_content_mapper_package_0_does_not_declare_a_typescript_contentMapper_object_100034",
    text: "The 'package.json' of the content mapper package '{0}' does not declare a 'typescript.contentMapper' object.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

const THE_TYPESCRIPT_CONTENT_MAPPER_EXEC_OF_THE_CONTENT_MAPPER_PACKAGE_0_MUST_BE_A_NON_EMPTY_ARRAY_OF_STRINGS: Message = Message {
    code: 100035,
    category: Category::Error,
    key: "The_typescript_contentMapper_exec_of_the_content_mapper_package_0_must_be_a_non_empty_array_of_strin_100035",
    text: "The 'typescript.contentMapper.exec' of the content mapper package '{0}' must be a non-empty array of strings.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

fn new_compiler_diagnostic(message: Message, args: Vec<String>) -> Diagnostic {
    Diagnostic::new(None, TextRange::undefined(), message, args)
}

#[derive(Debug, Clone, Default)]
pub struct ContentMapperManifest {
    pub name: String,
    pub version: String,
    pub exec: Vec<String>,
    pub compiler_options: Vec<String>,
    pub dynamic_config: bool,
}

struct OwnedResolutionHost {
    fs: Arc<dyn FS>,
    current_directory: String,
}

impl ResolutionHost for OwnedResolutionHost {
    fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }
    fn get_current_directory(&self) -> &str {
        &self.current_directory
    }
}

pub fn resolve_content_mapper_manifest(
    host: &ParseConfigHost,
    containing_file: &str,
    package_name: &str,
) -> (ContentMapperManifest, String, Option<Diagnostic>) {
    let mut compiler_options = CompilerOptions::default();
    compiler_options.module_resolution = ModuleResolutionKind::Bundler;
    let resolver = Resolver::new(
        Arc::new(OwnedResolutionHost {
            fs: host.fs.clone(),
            current_directory: host.current_directory.clone(),
        }),
        Arc::new(compiler_options),
        String::new(),
        String::new(),
    );
    let (resolved, _) = resolver.resolve_module_name(
        package_name,
        containing_file,
        ResolutionMode::None,
        None,
    );
    let package_directory = match resolved.map(|r| r.resolved_file_name) {
        Some(name) if !name.is_empty() => tsox_core::tspath::get_directory_path(&name),
        _ => {
            return (
                ContentMapperManifest::default(),
                String::new(),
                Some(new_compiler_diagnostic(
                    THE_CONTENT_MAPPER_PACKAGE_0_COULD_NOT_BE_RESOLVED,
                    vec![package_name.to_string()],
                )),
            )
        }
    };
    let package_json_path =
        tsox_core::tspath::combine_paths(&package_directory, &["package.json"]);
    let contents = match host.fs.read_file(&package_json_path) {
        Some(c) => c,
        None => {
            return (
                ContentMapperManifest::default(),
                package_directory,
                Some(new_compiler_diagnostic(
                    THE_CONTENT_MAPPER_PACKAGE_0_COULD_NOT_BE_RESOLVED,
                    vec![package_name.to_string()],
                )),
            )
        }
    };
    let fields = match packagejson::parse(&contents) {
        Ok(fields) => fields,
        Err(_) => {
            return (
                ContentMapperManifest::default(),
                package_directory,
                Some(new_compiler_diagnostic(
                    THE_PACKAGE_JSON_OF_THE_CONTENT_MAPPER_PACKAGE_0_COULD_NOT_BE_PARSED,
                    vec![package_name.to_string()],
                )),
            )
        }
    };
    let name = fields
        .header_fields
        .name
        .get_value()
        .cloned()
        .unwrap_or_default();
    if name.is_empty() {
        return (
            ContentMapperManifest::default(),
            package_directory,
            Some(new_compiler_diagnostic(
                THE_PACKAGE_JSON_OF_THE_CONTENT_MAPPER_PACKAGE_0_DOES_NOT_SPECIFY_A_NAME,
                vec![package_name.to_string()],
            )),
        );
    }
    let version = fields
        .header_fields
        .version
        .get_value()
        .cloned()
        .unwrap_or_default();
    (
        ContentMapperManifest {
            name,
            version,
            exec: Vec::new(),
            compiler_options: Vec::new(),
            dynamic_config: false,
        },
        package_directory,
        None,
    )
}
