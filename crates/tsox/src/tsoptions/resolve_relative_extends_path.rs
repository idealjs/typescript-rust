#![allow(unused_imports)]

use super::*;

pub(crate) fn resolve_relative_extends_path(
    s: &str,
    config_dir: &str,
    current_dir: &str,
    fs: &dyn FS,
) -> Option<String> {
    let base = tspath::normalize_path(&tspath::combine_paths(&config_dir, &[s]));

    if fs.file_exists(&base) {
        return Some(base);
    }

    if !base.ends_with(".json") {
        let with_json = format!("{base}.json");
        if fs.file_exists(&with_json) {
            return Some(with_json);
        }
    }

    let dir_form = tspath::combine_paths(&base, &["tsconfig.json"]);
    if fs.file_exists(&dir_form) {
        return Some(dir_form);
    }

    let abs = tspath::get_normalized_absolute_path(s, current_dir);
    if fs.file_exists(&abs) {
        Some(abs)
    } else {
        Some(tspath::combine_paths(&abs, &["tsconfig.json"]))
    }
}

pub(crate) fn resolve_config_via_node_modules(
    module_name: &str,
    containing_directory: &str,
    fs: &dyn FS,
) -> Option<String> {
    let mut result: Option<String> = None;
    tspath::for_each_ancestor_directory(containing_directory, |ancestor| {
        if tspath::get_base_file_name(ancestor) == "node_modules" {
            return false;
        }
        let node_modules = tspath::combine_paths(ancestor, &["node_modules"]);
        if !fs.directory_exists(&node_modules) {
            return false;
        }
        if let Some(resolved) = load_config_from_node_modules(module_name, &node_modules, fs) {
            result = Some(resolved);
            return true;
        }
        false
    });
    result
}

pub(crate) fn load_config_from_node_modules(
    module_name: &str,
    node_modules_dir: &str,
    fs: &dyn FS,
) -> Option<String> {
    let (package_name, _rest) = crate::module::parse_package_name(module_name);

    let candidate =
        tspath::normalize_path(&tspath::combine_paths(node_modules_dir, &[module_name]));

    if candidate.ends_with(".json") {
        if fs.file_exists(&candidate) {
            return Some(candidate);
        }
    } else {
        let with_json = format!("{candidate}.json");
        if fs.file_exists(&with_json) {
            return Some(with_json);
        }
    }

    let tsconfig_in_dir = tspath::combine_paths(&candidate, &["tsconfig.json"]);
    if fs.file_exists(&tsconfig_in_dir) {
        return Some(tsconfig_in_dir);
    }

    let package_dir = tspath::combine_paths(node_modules_dir, &[&package_name]);
    let package_json_path = tspath::combine_paths(&package_dir, &["package.json"]);
    if fs.file_exists(&package_json_path) {
        if let Some(content) = fs.read_file(&package_json_path) {
            if let Ok(fields) = crate::packagejson::parse(&content) {
                if let Some(tsconfig_field) = fields.path_fields.tsconfig.get_value() {
                    let resolved =
                        tspath::get_normalized_absolute_path(tsconfig_field, &package_dir);
                    if fs.file_exists(&resolved) {
                        return Some(resolved);
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn json_object_to_options(
    obj: &crate::json::Map<String, crate::json::Value>,
) -> (HashMap<String, OptValue>, Vec<Diagnostic>) {
    let mut out = HashMap::new();
    let mut errors = Vec::new();
    for (k, v) in obj {
        if let Some(opt) = find_option(k) {
            if opt.name != k {
                errors.push(Diagnostic::new(
                    None,
                    TextRange::undefined(),
                    UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1,
                    vec![k.clone(), opt.name.to_string()],
                ));
                continue;
            }
        }
        let val = json_to_opt_value(v);
        out.insert(k.clone(), val);
    }
    (out, errors)
}

pub(crate) fn json_to_opt_value(v: &crate::json::Value) -> OptValue {
    match v {
        crate::json::Value::Bool(b) => OptValue::Bool(*b),
        crate::json::Value::String(s) => OptValue::Str(s.clone()),
        crate::json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                OptValue::Num(i)
            } else {
                OptValue::Str(n.to_string())
            }
        }
        crate::json::Value::Array(arr) => {
            let list = arr
                .iter()
                .filter_map(|e| e.as_str().map(|s| s.to_string()))
                .collect();
            OptValue::List(list)
        }
        crate::json::Value::Null => OptValue::Null,
        crate::json::Value::Object(_) => OptValue::Null,
    }
}

pub(crate) const CONFIG_DIR_TEMPLATE: &str = "${configDir}";

pub(crate) fn starts_with_config_dir_template(value: &str) -> bool {
    value
        .to_ascii_lowercase()
        .starts_with(&CONFIG_DIR_TEMPLATE.to_ascii_lowercase())
}

pub(crate) fn get_substituted_path_with_config_dir_template(
    value: &str,
    base_path: &str,
) -> String {
    let replaced = value.replacen(CONFIG_DIR_TEMPLATE, "./", 1);
    tspath::get_normalized_absolute_path(&replaced, base_path)
}

pub(crate) fn get_substituted_string_array_with_config_dir_template(
    list: &[String],
    base_path: &str,
) -> Option<Vec<String>> {
    let mut result: Option<Vec<String>> = None;
    for (i, element) in list.iter().enumerate() {
        if starts_with_config_dir_template(element) {
            let arr = result.get_or_insert_with(|| list.to_vec());
            arr[i] = get_substituted_path_with_config_dir_template(element, base_path);
        }
    }
    result
}

pub(crate) fn handle_config_dir_template_substitution(
    options: &mut CompilerOptions,
    base_path: &str,
) {
    if let Some(paths) = options.paths.as_mut() {
        let mut changed = false;
        for (_, targets) in paths.iter_mut() {
            if let Some(substituted) =
                get_substituted_string_array_with_config_dir_template(targets, base_path)
            {
                *targets = substituted;
                changed = true;
            }
        }
        if !changed {}
    }

    if let Some(root_dirs) =
        get_substituted_string_array_with_config_dir_template(&options.root_dirs, base_path)
    {
        options.root_dirs = root_dirs;
    }

    if let Some(type_roots) =
        get_substituted_string_array_with_config_dir_template(&options.type_roots, base_path)
    {
        options.type_roots = type_roots;
    }

    if starts_with_config_dir_template(&options.generate_cpu_profile) {
        options.generate_cpu_profile =
            get_substituted_path_with_config_dir_template(&options.generate_cpu_profile, base_path);
    }
    if starts_with_config_dir_template(&options.generate_trace) {
        options.generate_trace =
            get_substituted_path_with_config_dir_template(&options.generate_trace, base_path);
    }
    if starts_with_config_dir_template(&options.out_file) {
        options.out_file =
            get_substituted_path_with_config_dir_template(&options.out_file, base_path);
    }
    if starts_with_config_dir_template(&options.out_dir) {
        options.out_dir =
            get_substituted_path_with_config_dir_template(&options.out_dir, base_path);
    }
    if starts_with_config_dir_template(&options.root_dir) {
        options.root_dir =
            get_substituted_path_with_config_dir_template(&options.root_dir, base_path);
    }
    if starts_with_config_dir_template(&options.ts_build_info_file) {
        options.ts_build_info_file =
            get_substituted_path_with_config_dir_template(&options.ts_build_info_file, base_path);
    }
    if starts_with_config_dir_template(&options.base_url) {
        options.base_url =
            get_substituted_path_with_config_dir_template(&options.base_url, base_path);
    }
    if starts_with_config_dir_template(&options.declaration_dir) {
        options.declaration_dir =
            get_substituted_path_with_config_dir_template(&options.declaration_dir, base_path);
    }
}

pub(crate) fn resolve_file_path_options(options: &mut CompilerOptions, base_path: &str) {
    let resolve = |s: &str| -> String {
        if s.is_empty() {
            return s.to_string();
        }

        if starts_with_config_dir_template(s) {
            return s.to_string();
        }
        tspath::get_normalized_absolute_path(s, base_path)
    };
    options.root_dir = resolve(&options.root_dir);
    options.out_dir = resolve(&options.out_dir);
    options.out_file = resolve(&options.out_file);
    options.declaration_dir = resolve(&options.declaration_dir);
    options.base_url = resolve(&options.base_url);
    options.ts_build_info_file = resolve(&options.ts_build_info_file);
    options.source_root = resolve(&options.source_root);
    options.map_root = resolve(&options.map_root);
    options.project = resolve(&options.project);
    options.generate_cpu_profile = resolve(&options.generate_cpu_profile);
    options.generate_trace = resolve(&options.generate_trace);
    if !options.root_dirs.is_empty() {
        options.root_dirs = options.root_dirs.iter().map(|s| resolve(s)).collect();
    }
}

pub(crate) fn merge_compiler_options(dst: &mut CompilerOptions, src: &CompilerOptions) {
    let empty = HashSet::new();
    merge_compiler_options_with_skip(dst, src, &empty);
}
