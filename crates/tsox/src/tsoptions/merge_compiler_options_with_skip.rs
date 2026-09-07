#![allow(unused_imports)]

use super::*;

pub(crate) fn merge_compiler_options_with_skip(
    dst: &mut CompilerOptions,
    src: &CompilerOptions,
    skip_fields: &HashSet<String>,
) {
    macro_rules! merge_tri {
        ($field:ident, $json_name:literal) => {
            if dst.$field.is_unknown() && !skip_fields.contains($json_name) {
                dst.$field = src.$field;
            }
        };
    }
    merge_tri!(no_emit, "noEmit");
    merge_tri!(no_check, "noCheck");
    merge_tri!(no_lib, "noLib");
    merge_tri!(skip_lib_check, "skipLibCheck");
    merge_tri!(skip_default_lib_check, "skipDefaultLibCheck");
    merge_tri!(strict, "strict");
    merge_tri!(strict_null_checks, "strictNullChecks");
    merge_tri!(strict_function_types, "strictFunctionTypes");
    merge_tri!(strict_bind_call_apply, "strictBindCallApply");
    merge_tri!(
        strict_property_initialization,
        "strictPropertyInitialization"
    );
    merge_tri!(
        strict_builtin_iterator_return,
        "strictBuiltinIteratorReturn"
    );
    merge_tri!(no_implicit_any, "noImplicitAny");
    merge_tri!(no_implicit_this, "noImplicitThis");
    merge_tri!(no_implicit_override, "noImplicitOverride");
    merge_tri!(no_unused_locals, "noUnusedLocals");
    merge_tri!(no_unused_parameters, "noUnusedParameters");
    merge_tri!(no_fallthrough_cases_in_switch, "noFallthroughCasesInSwitch");
    merge_tri!(no_unchecked_indexed_access, "noUncheckedIndexedAccess");
    merge_tri!(exact_optional_property_types, "exactOptionalPropertyTypes");
    merge_tri!(es_module_interop, "esModuleInterop");
    merge_tri!(allow_js, "allowJs");
    merge_tri!(check_js, "checkJs");
    merge_tri!(composite, "composite");
    merge_tri!(declaration, "declaration");
    merge_tri!(source_map, "sourceMap");
    merge_tri!(remove_comments, "removeComments");
    merge_tri!(isolated_modules, "isolatedModules");
    merge_tri!(verbatim_module_syntax, "verbatimModuleSyntax");
    merge_tri!(experimental_decorators, "experimentalDecorators");
    merge_tri!(
        force_consistent_casing_in_file_names,
        "forceConsistentCasingInFileNames"
    );
    merge_tri!(use_unknown_in_catch_variables, "useUnknownInCatchVariables");
    merge_tri!(pretty, "pretty");
    merge_tri!(incremental, "incremental");
    merge_tri!(watch, "watch");
    if dst.target == ScriptTarget::None && !skip_fields.contains("target") {
        dst.target = src.target;
    }
    if dst.module == ModuleKind::None && !skip_fields.contains("module") {
        dst.module = src.module;
    }
    if dst.module_resolution == ModuleResolutionKind::Unknown
        && !skip_fields.contains("moduleResolution")
    {
        dst.module_resolution = src.module_resolution;
    }
    if dst.jsx == JsxEmit::None && !skip_fields.contains("jsx") {
        dst.jsx = src.jsx;
    }
    if dst.out_dir.is_empty() && !skip_fields.contains("outDir") {
        dst.out_dir = src.out_dir.clone();
    }
    if dst.root_dir.is_empty() && !skip_fields.contains("rootDir") {
        dst.root_dir = src.root_dir.clone();
    }
    if dst.base_url.is_empty() && !skip_fields.contains("baseUrl") {
        dst.base_url = src.base_url.clone();
    }
    if dst.lib.is_empty() && !skip_fields.contains("lib") {
        dst.lib = src.lib.clone();
    }
    if dst.types.is_empty() && !skip_fields.contains("types") {
        dst.types = src.types.clone();
    }
    if dst.type_roots.is_empty() && !skip_fields.contains("typeRoots") {
        dst.type_roots = src.type_roots.clone();
    }
    if dst.paths.is_none() && !skip_fields.contains("paths") {
        dst.paths = src.paths.clone();
    }
    if dst.declaration_dir.is_empty() && !skip_fields.contains("declarationDir") {
        dst.declaration_dir = src.declaration_dir.clone();
    }
    if dst.source_root.is_empty() && !skip_fields.contains("sourceRoot") {
        dst.source_root = src.source_root.clone();
    }
    if dst.map_root.is_empty() && !skip_fields.contains("mapRoot") {
        dst.map_root = src.map_root.clone();
    }
    if dst.ts_build_info_file.is_empty() && !skip_fields.contains("tsBuildInfoFile") {
        dst.ts_build_info_file = src.ts_build_info_file.clone();
    }
    if dst.root_dirs.is_empty() && !skip_fields.contains("rootDirs") {
        dst.root_dirs = src.root_dirs.clone();
    }
    if dst.module_suffixes.is_empty() && !skip_fields.contains("moduleSuffixes") {
        dst.module_suffixes = src.module_suffixes.clone();
    }
    if dst.custom_conditions.is_empty() && !skip_fields.contains("customConditions") {
        dst.custom_conditions = src.custom_conditions.clone();
    }
    if dst.out_file.is_empty() && !skip_fields.contains("outFile") {
        dst.out_file = src.out_file.clone();
    }
    if dst.module_detection == ModuleDetectionKind::None && !skip_fields.contains("moduleDetection")
    {
        dst.module_detection = src.module_detection;
    }
    if dst.new_line == NewLineKind::None && !skip_fields.contains("newLine") {
        dst.new_line = src.new_line;
    }
}

pub(crate) fn expand_file_names(
    files: &[String],
    has_files_spec: bool,
    include: &[String],
    has_include_spec: bool,
    exclude: &[String],
    has_exclude_spec: bool,
    options: &CompilerOptions,
    base_dir: &str,
    fs: &dyn FS,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut effective_exclude = exclude.to_vec();
    if !has_exclude_spec {
        if !options.out_dir.is_empty() {
            effective_exclude.push(options.out_dir.clone());
        }
        if !options.declaration_dir.is_empty() {
            effective_exclude.push(options.declaration_dir.clone());
        }
    }
    let exclude_dirs: Vec<String> = effective_exclude
        .iter()
        .filter(|p| !p.contains('*') && !p.contains('?') && !p.contains('[') && !p.contains('{'))
        .map(|p| tspath::get_normalized_absolute_path(p, base_dir))
        .collect();
    let exclude_globs: Vec<Glob> = effective_exclude
        .iter()
        .filter_map(|p| {
            let spec = if tspath::path_is_absolute(p) {
                p.clone()
            } else {
                tspath::combine_paths(base_dir, &[p])
            };
            Glob::parse(&spec).ok()
        })
        .collect();

    let add = |path: &str, out: &mut Vec<String>, seen: &mut std::collections::HashSet<String>| {
        let abs = tspath::get_normalized_absolute_path(path, base_dir);
        if seen.insert(abs.clone()) {
            out.push(abs);
        }
    };

    for f in files {
        add(f, &mut result, &mut seen);
    }

    let include_specs: Vec<String> = if !has_include_spec && !has_files_spec {
        vec!["**/*".to_string()]
    } else {
        include.to_vec()
    };
    for spec in &include_specs {
        let matched = match_glob_spec(spec, base_dir, fs);
        for path in matched {
            if is_excluded(&path, &exclude_globs, &exclude_dirs) {
                continue;
            }
            if !is_supported_source_file_ex(&path, options.allow_js.is_true()) {
                continue;
            }
            add(&path, &mut result, &mut seen);
        }
    }

    result.sort();
    result
}

pub(crate) fn is_excluded(path: &str, exclude_globs: &[Glob], exclude_dirs: &[String]) -> bool {
    exclude_globs.iter().any(|g| g.is_match(path))
        || exclude_dirs.iter().any(|dir| path_is_under_dir(path, dir))
}

pub(crate) fn path_is_under_dir(path: &str, dir: &str) -> bool {
    path == dir
        || path
            .strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/'))
}

#[allow(dead_code)]
pub(crate) fn is_supported_source_file(path: &str) -> bool {
    is_supported_source_file_ex(path, false)
}

pub(crate) fn is_supported_source_file_ex(path: &str, allow_js: bool) -> bool {
    let ext = path.rfind('.').map(|i| &path[i..]).unwrap_or("");
    if matches!(
        ext,
        ".ts" | ".tsx" | ".d.ts" | ".mts" | ".cts" | ".d.mts" | ".d.cts"
    ) {
        return true;
    }
    if allow_js && matches!(ext, ".js" | ".jsx" | ".mjs" | ".cjs") {
        return true;
    }
    false
}

pub(crate) fn match_glob_spec(spec: &str, base_dir: &str, fs: &dyn FS) -> Vec<String> {
    let mut results = Vec::new();

    let abs_spec = if tspath::path_is_absolute(spec) {
        spec.to_string()
    } else {
        tspath::combine_paths(base_dir, &[spec])
    };
    if !contains_glob_char(&abs_spec) {
        if fs.file_exists(&abs_spec) {
            results.push(abs_spec);
            return results;
        }
        if fs.directory_exists(&abs_spec) {
            walk_and_collect_files(&abs_spec, fs, &mut results);
            return results;
        }
    }

    let walk_root = glob_base_dir(&abs_spec);
    walk_and_match(&abs_spec, &walk_root, fs, &mut results);
    results
}

pub(crate) fn contains_glob_char(spec: &str) -> bool {
    spec.chars()
        .any(|c| c == '*' || c == '?' || c == '{' || c == '[')
}

pub(crate) fn glob_base_dir(spec: &str) -> String {
    let first_meta = spec
        .chars()
        .position(|c| c == '*' || c == '?' || c == '{' || c == '[');
    let prefix = match first_meta {
        Some(idx) => &spec[..idx],
        None => spec,
    };

    match prefix.rfind('/') {
        Some(0) => "/".to_string(),
        Some(idx) => prefix[..idx].to_string(),
        None => ".".to_string(),
    }
}

pub(crate) fn walk_and_collect_files(dir: &str, fs: &dyn FS, results: &mut Vec<String>) {
    let entries = fs.get_accessible_entries(dir);
    for file in &entries.files {
        results.push(tspath::combine_paths(dir, &[file]));
    }
    for d in &entries.directories {
        let full = tspath::combine_paths(dir, &[d]);
        walk_and_collect_files(&full, fs, results);
    }
}
