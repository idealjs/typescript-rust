#![allow(unused_imports)]

use super::*;
use crate::json::Value;

pub(crate) fn insert_bool_options(
    map: &mut crate::json::Map<String, crate::json::Value>,
    options: &CompilerOptions,
) {
    let bool_opts: &[(&str, Tristate)] = &[
        ("allowJs", options.allow_js),
        (
            "allowImportingTsExtensions",
            options.allow_importing_ts_extensions,
        ),
        ("allowUmdGlobalAccess", options.allow_umd_global_access),
        ("allowUnreachableCode", options.allow_unreachable_code),
        ("allowUnusedLabels", options.allow_unused_labels),
        ("alwaysStrict", options.always_strict),
        ("checkJs", options.check_js),
        ("composite", options.composite),
        ("declaration", options.declaration),
        ("declarationMap", options.declaration_map),
        ("downlevelIteration", options.downlevel_iteration),
        ("emitBOM", options.emit_bom),
        ("emitDeclarationOnly", options.emit_declaration_only),
        ("emitDecoratorMetadata", options.emit_decorator_metadata),
        ("esModuleInterop", options.es_module_interop),
        (
            "exactOptionalPropertyTypes",
            options.exact_optional_property_types,
        ),
        ("experimentalDecorators", options.experimental_decorators),
        (
            "forceConsistentCasingInFileNames",
            options.force_consistent_casing_in_file_names,
        ),
        ("importHelpers", options.import_helpers),
        ("incremental", options.incremental),
        ("inlineSourceMap", options.inline_source_map),
        ("inlineSources", options.inline_sources),
        ("isolatedModules", options.isolated_modules),
        ("isolatedDeclarations", options.isolated_declarations),
        ("noCheck", options.no_check),
        ("noEmit", options.no_emit),
        ("noEmitOnError", options.no_emit_on_error),
        ("noErrorTruncation", options.no_error_truncation),
        (
            "noFallthroughCasesInSwitch",
            options.no_fallthrough_cases_in_switch,
        ),
        ("noImplicitAny", options.no_implicit_any),
        ("noImplicitOverride", options.no_implicit_override),
        ("noImplicitReturns", options.no_implicit_returns),
        ("noImplicitThis", options.no_implicit_this),
        ("noLib", options.no_lib),
        (
            "noPropertyAccessFromIndexSignature",
            options.no_property_access_from_index_signature,
        ),
        ("noResolve", options.no_resolve),
        (
            "noUncheckedIndexedAccess",
            options.no_unchecked_indexed_access,
        ),
        (
            "noUncheckedSideEffectImports",
            options.no_unchecked_side_effect_imports,
        ),
        ("noUnusedLocals", options.no_unused_locals),
        ("noUnusedParameters", options.no_unused_parameters),
        ("preserveConstEnums", options.preserve_const_enums),
        ("removeComments", options.remove_comments),
        ("resolveJsonModule", options.resolve_json_module),
        (
            "resolvePackageJsonExports",
            options.resolve_package_json_exports,
        ),
        (
            "resolvePackageJsonImports",
            options.resolve_package_json_imports,
        ),
        (
            "rewriteRelativeImportExtensions",
            options.rewrite_relative_import_extensions,
        ),
        ("skipLibCheck", options.skip_lib_check),
        ("strict", options.strict),
        ("strictBindCallApply", options.strict_bind_call_apply),
        (
            "strictBuiltinIteratorReturn",
            options.strict_builtin_iterator_return,
        ),
        ("strictFunctionTypes", options.strict_function_types),
        ("strictNullChecks", options.strict_null_checks),
        (
            "strictPropertyInitialization",
            options.strict_property_initialization,
        ),
        ("stripInternal", options.strip_internal),
        (
            "useDefineForClassFields",
            options.use_define_for_class_fields,
        ),
        (
            "useUnknownInCatchVariables",
            options.use_unknown_in_catch_variables,
        ),
        ("verbatimModuleSyntax", options.verbatim_module_syntax),
    ];
    for (name, t) in bool_opts {
        match *t {
            Tristate::True => {
                map.insert(name.to_string(), Value::Bool(true));
            }
            Tristate::False => {
                map.insert(name.to_string(), Value::Bool(false));
            }
            Tristate::Unknown => {}
        }
    }
}
