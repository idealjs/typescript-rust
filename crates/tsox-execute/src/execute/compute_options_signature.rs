#![allow(unused_imports)]

use super::*;

pub(crate) fn compute_options_signature(options: &CompilerOptions) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("target={:?}", options.target));
    parts.push(format!("module={:?}", options.module));
    parts.push(format!("moduleResolution={:?}", options.module_resolution));
    parts.push(format!("jsx={:?}", options.jsx));
    parts.push(format!("moduleDetection={:?}", options.module_detection));
    parts.push(format!("newLine={:?}", options.new_line));
    parts.push(format!("outDir={}", options.out_dir));
    parts.push(format!("rootDir={}", options.root_dir));
    parts.push(format!("outFile={}", options.out_file));
    parts.push(format!("declarationDir={}", options.declaration_dir));
    parts.push(format!("sourceRoot={}", options.source_root));
    parts.push(format!("mapRoot={}", options.map_root));
    parts.push(format!("tsBuildInfoFile={}", options.ts_build_info_file));
    parts.push(format!("lib={:?}", options.lib));
    parts.push(format!("types={:?}", options.types));
    parts.push(format!("noEmit={:?}", options.no_emit));
    parts.push(format!("noEmitOnError={:?}", options.no_emit_on_error));
    parts.push(format!("declaration={:?}", options.declaration));
    parts.push(format!("declarationMap={:?}", options.declaration_map));
    parts.push(format!(
        "emitDeclarationOnly={:?}",
        options.emit_declaration_only
    ));
    parts.push(format!("sourceMap={:?}", options.source_map));
    parts.push(format!("inlineSourceMap={:?}", options.inline_source_map));
    parts.push(format!("removeComments={:?}", options.remove_comments));
    parts.push(format!("importHelpers={:?}", options.import_helpers));
    parts.push(format!("noResolve={:?}", options.no_resolve));
    parts.push(format!("composite={:?}", options.composite));
    parts.push(format!("incremental={:?}", options.incremental));
    parts.push(format!("isolatedModules={:?}", options.isolated_modules));
    parts.push(format!(
        "isolatedDeclarations={:?}",
        options.isolated_declarations
    ));
    parts.push(format!(
        "verbatimModuleSyntax={:?}",
        options.verbatim_module_syntax
    ));
    parts.push(format!("esModuleInterop={:?}", options.es_module_interop));
    parts.push(format!("allowJs={:?}", options.allow_js));
    parts.push(format!("checkJs={:?}", options.check_js));
    parts.push(format!("skipLibCheck={:?}", options.skip_lib_check));
    parts.push(format!("strict={:?}", options.strict));
    parts.push(format!("noImplicitAny={:?}", options.no_implicit_any));
    parts.push(format!("strictNullChecks={:?}", options.strict_null_checks));
    parts.push(format!(
        "strictFunctionTypes={:?}",
        options.strict_function_types
    ));
    parts.push(format!(
        "strictBindCallApply={:?}",
        options.strict_bind_call_apply
    ));
    parts.push(format!(
        "strictPropertyInitialization={:?}",
        options.strict_property_initialization
    ));
    parts.push(format!("noImplicitThis={:?}", options.no_implicit_this));
    parts.push(format!("alwaysStrict={:?}", options.always_strict));
    parts.push(format!(
        "exactOptionalPropertyTypes={:?}",
        options.exact_optional_property_types
    ));
    parts.push(format!(
        "noUncheckedIndexedAccess={:?}",
        options.no_unchecked_indexed_access
    ));
    parts.push(format!(
        "noFallthroughCasesInSwitch={:?}",
        options.no_fallthrough_cases_in_switch
    ));
    parts.push(format!(
        "noImplicitReturns={:?}",
        options.no_implicit_returns
    ));
    parts.push(format!(
        "noImplicitOverride={:?}",
        options.no_implicit_override
    ));
    parts.push(format!("noUnusedLocals={:?}", options.no_unused_locals));
    parts.push(format!(
        "noUnusedParameters={:?}",
        options.no_unused_parameters
    ));
    parts.push(format!(
        "forceConsistentCasingInFileNames={:?}",
        options.force_consistent_casing_in_file_names
    ));
    parts.push(format!(
        "useDefineForClassFields={:?}",
        options.use_define_for_class_fields
    ));
    parts.push(format!("jsxFactory={}", options.jsx_factory));
    parts.push(format!(
        "jsxFragmentFactory={}",
        options.jsx_fragment_factory
    ));
    parts.push(format!("jsxImportSource={}", options.jsx_import_source));
    parts.push(format!("moduleSuffixes={:?}", options.module_suffixes));
    parts.push(format!("customConditions={:?}", options.custom_conditions));
    parts.push(format!(
        "resolveJsonModule={:?}",
        options.resolve_json_module
    ));
    parts.push(format!(
        "allowSyntheticDefaultImports={:?}",
        options.allow_synthetic_default_imports
    ));
    parts.push(format!(
        "downlevelIteration={:?}",
        options.downlevel_iteration
    ));
    parts.push(format!("emitBOM={:?}", options.emit_bom));
    parts.push(format!(
        "emitDecoratorMetadata={:?}",
        options.emit_decorator_metadata
    ));
    parts.push(format!(
        "experimentalDecorators={:?}",
        options.experimental_decorators
    ));
    parts.push(format!(
        "preserveConstEnums={:?}",
        options.preserve_const_enums
    ));
    parts.push(format!("stripInternal={:?}", options.strip_internal));
    parts.push(format!(
        "erasableSyntaxOnly={:?}",
        options.erasable_syntax_only
    ));
    compute_options_hash(&parts.join("\n"))
}
