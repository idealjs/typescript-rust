#![allow(unused_imports)]

use super::*;

pub(crate) fn set_bool(options: &mut CompilerOptions, name: &str, b: bool) {
    let t = Tristate::from(b);

    let name = name.to_ascii_lowercase();
    match name.as_str() {
        "noemit" => options.no_emit = t,
        "nocheck" => options.no_check = t,
        "nolib" => options.no_lib = t,
        "skiplibcheck" => options.skip_lib_check = t,
        "skipdefaultlibcheck" => options.skip_default_lib_check = t,
        "strictnullchecks" => options.strict_null_checks = t,
        "strictfunctiontypes" => options.strict_function_types = t,
        "strictbindcallapply" => options.strict_bind_call_apply = t,
        "strictpropertyinitialization" => options.strict_property_initialization = t,
        "strictbuiltiniteratorreturn" => options.strict_builtin_iterator_return = t,
        "noimplicitany" => options.no_implicit_any = t,
        "noimplicitthis" => options.no_implicit_this = t,
        "noimplicitoverride" => options.no_implicit_override = t,
        "nounusedlocals" => options.no_unused_locals = t,
        "nounusedparameters" => options.no_unused_parameters = t,
        "nofallthroughcasesinswitch" => options.no_fallthrough_cases_in_switch = t,
        "nouncheckedindexedaccess" => options.no_unchecked_indexed_access = t,
        "nopropertyaccessfromindexsignature" => options.no_property_access_from_index_signature = t,
        "noerrortruncation" => options.no_error_truncation = t,
        "noemitonerror" => options.no_emit_on_error = t,
        "noresolve" => options.no_resolve = t,
        "useunknownincatchvariables" => options.use_unknown_in_catch_variables = t,
        "exactoptionalpropertytypes" => options.exact_optional_property_types = t,
        "esmoduleinterop" => options.es_module_interop = t,
        "allowsyntheticdefaultimports" => options.allow_synthetic_default_imports = t,
        "allowjs" => options.allow_js = t,
        "alwaysstrict" => options.always_strict = t,
        "checkjs" => options.check_js = t,
        "composite" => options.composite = t,
        "declaration" => options.declaration = t,
        "declarationmap" => options.declaration_map = t,
        "emitdeclarationonly" => options.emit_declaration_only = t,
        "sourcemap" => options.source_map = t,
        "inlinesourcemap" => options.inline_source_map = t,
        "inlinesources" => options.inline_sources = t,
        "removecomments" => options.remove_comments = t,
        "isolatedmodules" => options.isolated_modules = t,
        "isolateddeclarations" => options.isolated_declarations = t,
        "verbatimmodulesyntax" => options.verbatim_module_syntax = t,
        "preserveconstenums" => options.preserve_const_enums = t,
        "importhelpers" => options.import_helpers = t,
        "experimentaldecorators" => options.experimental_decorators = t,
        "emitdecoratormetadata" => options.emit_decorator_metadata = t,
        "forceconsistentcasinginfilenames" => options.force_consistent_casing_in_file_names = t,
        "listfiles" => options.list_files = t,
        "listfilesonly" => options.list_files_only = t,
        "listemittedfiles" => options.list_emitted_files = t,
        "explainfiles" => options.explain_files = t,
        "extendeddiagnostics" => options.extended_diagnostics = t,
        "diagnostics" => options.diagnostics = t,
        "pretty" => options.pretty = t,
        "showconfig" => options.show_config = t,
        "ignoreconfig" => options.ignore_config = t,
        "incremental" => options.incremental = t,
        "watch" => options.watch = t,
        "version" => options.version = t,
        "help" => options.help = t,
        "all" => options.all = t,
        "init" => options.init = t,
        "build" => options.build = t,
        "singlethreaded" => options.single_threaded = t,
        "quiet" => options.quiet = t,
        "strict" => {
            options.strict = t;

            options.strict_null_checks = t;
            options.strict_function_types = t;
            options.strict_bind_call_apply = t;
            options.strict_property_initialization = t;
            options.strict_builtin_iterator_return = t;
            options.no_implicit_any = t;
            options.no_implicit_this = t;
            options.use_unknown_in_catch_variables = t;
            options.always_strict = t;
        }
        _ => {}
    }
}

pub fn apply_test_settings(settings: &HashMap<String, String>) -> (CompilerOptions, Vec<String>) {
    apply_test_settings_with_base(settings, CompilerOptions::default())
}

pub fn apply_test_settings_with_base(
    settings: &HashMap<String, String>,
    base: CompilerOptions,
) -> (CompilerOptions, Vec<String>) {
    const KNOWN_BOOL_OPTIONS: &[&str] = &[
        "noemit",
        "nocheck",
        "nolib",
        "skiplibcheck",
        "skipdefaultlibcheck",
        "strictnullchecks",
        "strictfunctiontypes",
        "strictbindcallapply",
        "strictpropertyinitialization",
        "strictbuiltiniteratorreturn",
        "noimplicitany",
        "noimplicitthis",
        "noimplicitoverride",
        "nounsusedlocals",
        "nounsusedparameters",
        "nofallthroughcasesinswitch",
        "nouncheckedindexedaccess",
        "nopropertyaccessfromindexsignature",
        "noerrortruncation",
        "noemitonerror",
        "noresolve",
        "useunknownincatchvariables",
        "exactoptionalpropertytypes",
        "esmoduleinterop",
        "allowsyntheticdefaultimports",
        "allowjs",
        "checkjs",
        "composite",
        "declaration",
        "declarationmap",
        "emitdeclarationonly",
        "sourcemap",
        "inlinesourcemap",
        "inlinesources",
        "removecomments",
        "isolatedmodules",
        "isolateddeclarations",
        "verbatimmodulesyntax",
        "preserveconstenums",
        "importhelpers",
        "experimentaldecorators",
        "emitdecoratormetadata",
        "forceconsistencingcasingfilenames",
        "listfiles",
        "listfilesonly",
        "listemittedfiles",
        "explainfiles",
        "extendeddiagnostics",
        "diagnostics",
        "pretty",
        "showconfig",
        "ignoreconfig",
        "incremental",
        "watch",
        "version",
        "help",
        "all",
        "init",
        "build",
        "singlethreaded",
        "quiet",
        "strict",
        "alwaysstrict",
    ];
    const KNOWN_STR_OPTIONS: &[&str] = &[
        "target",
        "module",
        "moduleresolution",
        "jsx",
        "newline",
        "moduledetection",
        "outdir",
        "outfile",
        "rootdir",
        "declarationdir",
        "tsbuildinfofile",
        "sourceroot",
        "maproot",
        "jsxfactory",
        "jsxfragmentfactory",
        "jsximportsource",
        "reactnamespace",
        "locale",
        "baseurl",
        "modulosuffixes",
        "customconditions",
        "jsxmode",
    ];
    const KNOWN_LIST_OPTIONS: &[&str] = &["lib", "types", "typeroots", "rootdirs"];

    let mut options = base;
    let mut unrecognized: Vec<String> = Vec::new();

    let has_strict_directive = settings.keys().any(|k| k.eq_ignore_ascii_case("strict"));
    let has_nia_directive = settings
        .keys()
        .any(|k| k.eq_ignore_ascii_case("noimplicitany"));
    if !has_strict_directive && !has_nia_directive && options.no_implicit_any.is_unknown() {
        options.no_implicit_any = crate::core::tristate::Tristate::True;
    }

    for (name, raw_value) in settings {
        let lower = name.to_lowercase();
        let trimmed = raw_value.trim().trim_end_matches(';').to_string();

        let known = KNOWN_BOOL_OPTIONS.contains(&lower.as_str())
            || KNOWN_STR_OPTIONS.contains(&lower.as_str())
            || KNOWN_LIST_OPTIONS.contains(&lower.as_str());

        if !known {
            unrecognized.push(name.clone());
            continue;
        }

        let is_bool_val = matches!(trimmed.as_str(), "true" | "false")
            && KNOWN_BOOL_OPTIONS.contains(&lower.as_str());

        let canonical = find_option(&lower)
            .map(|o| o.name.to_string())
            .unwrap_or_else(|| lower.clone());
        if is_bool_val {
            set_bool(&mut options, &lower, trimmed.eq_ignore_ascii_case("true"));
        } else if KNOWN_LIST_OPTIONS.contains(&lower.as_str()) {
            let list: Vec<String> = trimmed
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let mut map = HashMap::new();
            map.insert(canonical, OptValue::List(list));
            apply_options(&map, &mut options);
        } else {
            let mut map = HashMap::new();
            map.insert(canonical, OptValue::Str(trimmed.clone()));
            apply_options(&map, &mut options);
        }
    }

    (options, unrecognized)
}
