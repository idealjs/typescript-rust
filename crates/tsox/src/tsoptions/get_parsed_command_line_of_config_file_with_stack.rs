#![allow(unused_imports)]

use super::*;

pub(crate) fn get_parsed_command_line_of_config_file_with_stack(
    config_file_name: &str,
    base_options: &CompilerOptions,
    current_dir: &str,
    fs: &dyn FS,
    resolution_stack: &[String],
    cache: &mut ExtendedConfigCache,
) -> ParsedCommandLine {
    let mut result = ParsedCommandLine::default();
    result.compiler_options = base_options.clone();
    result.config_file_name = config_file_name.to_string();

    let resolved_path = tspath::get_normalized_absolute_path(config_file_name, current_dir);
    if resolution_stack.iter().any(|p| p == &resolved_path) {
        result.errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0,
            vec![resolved_path],
        ));
        return result;
    }

    let config_text = match fs.read_file(config_file_name) {
        Some(t) => t,
        None => {
            result.errors.push(err(format!(
                "Cannot find a tsconfig.json file at the specified directory: '{config_file_name}'."
            )));
            return result;
        }
    };

    let jsonc = strip_jsonc(&config_text);

    let root: crate::json::Value = if jsonc.trim().is_empty() {
        crate::json::Value::Object(crate::json::Map::new())
    } else {
        match crate::json::from_str(&jsonc) {
            Ok(v) => v,
            Err(e) => {
                result
                    .errors
                    .push(err(format!("Failed to parse tsconfig.json: {e}.")));
                return result;
            }
        }
    };

    let root_obj = match root.as_object() {
        Some(o) => o,
        None => {
            result.errors.push(err("tsconfig.json must be an object."));
            return result;
        }
    };

    let mut extended_opts = CompilerOptions::default();
    if let Some(extends) = root_obj.get("extends") {
        let extends_paths = extends_as_paths(extends, config_file_name, current_dir, fs);
        if !extends_paths.is_empty() {
            let mut new_stack: Vec<String> = resolution_stack.to_vec();
            new_stack.push(resolved_path.clone());

            let mut extended_configs: Vec<(String, ParsedCommandLine)> = Vec::new();
            for ext_path in &extends_paths {
                let ext_resolved = tspath::get_normalized_absolute_path(ext_path, current_dir);
                let parent =
                    cache.get_or_parse(&ext_resolved, ext_path, current_dir, fs, &new_stack);
                extended_configs.push((ext_path.clone(), parent));
            }

            for (_, parent) in extended_configs.iter().rev() {
                merge_compiler_options(&mut extended_opts, &parent.compiler_options);
            }

            let own_config_dir = tspath::get_directory_path(config_file_name);
            let compare_opts = tspath::ComparePathsOptions {
                use_case_sensitive_file_names: fs.use_case_sensitive_file_names(),
                current_directory: own_config_dir.clone(),
            };
            for (ext_path, parent) in &extended_configs {
                let ext_dir = tspath::get_directory_path(ext_path);
                let relative_difference = tspath::convert_to_relative_path(&ext_dir, &compare_opts);
                let rewrite = |spec: &str| -> String {
                    if starts_with_config_dir_template(spec) || tspath::is_rooted_disk_path(spec) {
                        spec.to_string()
                    } else {
                        tspath::combine_paths(&relative_difference, &[spec])
                    }
                };
                if !result.has_include_spec && parent.has_include_spec {
                    result.include = parent.include.iter().map(|s| rewrite(s)).collect();
                    result.has_include_spec = true;
                }
                if !result.has_exclude_spec && parent.has_exclude_spec {
                    result.exclude = parent.exclude.iter().map(|s| rewrite(s)).collect();
                    result.has_exclude_spec = true;
                }
                if !result.has_files_spec && parent.has_files_spec {
                    result.files_spec = parent.files_spec.iter().map(|s| rewrite(s)).collect();
                    result.has_files_spec = true;
                }
                result.errors.extend(parent.errors.clone());
            }
        }
    }

    if let Some(value) = root_obj.get("compileOnSave").and_then(|v| v.as_bool()) {
        result.compile_on_save = Some(value);
    }

    if let Some(references) = root_obj.get("references").and_then(|v| v.as_array()) {
        let config_dir_for_refs = tspath::get_directory_path(config_file_name);
        result.references = references
            .iter()
            .filter_map(|entry| {
                let raw_path = entry.as_object()?.get("path")?.as_str()?;
                Some(crate::core::project_reference::ProjectReference {
                    path: tspath::get_normalized_absolute_path(raw_path, &config_dir_for_refs),
                    original_path: raw_path.to_string(),
                    circular: false,
                })
            })
            .collect();
    }

    if let Some(files) = root_obj.get("files").and_then(|v| v.as_array()) {
        result.has_files_spec = true;
        result.files_spec.clear();
        for f in files {
            if let Some(s) = f.as_str() {
                result.files_spec.push(s.to_string());
            }
        }
    }

    if let Some(include) = root_obj.get("include").and_then(|v| v.as_array()) {
        result.has_include_spec = true;
        result.include.clear();
        for f in include {
            if let Some(s) = f.as_str() {
                result.include.push(s.to_string());
            }
        }
    }

    if let Some(exclude) = root_obj.get("exclude").and_then(|v| v.as_array()) {
        result.has_exclude_spec = true;
        result.exclude.clear();
        for f in exclude {
            if let Some(s) = f.as_str() {
                result.exclude.push(s.to_string());
            }
        }
    }

    let mut explicit_null_fields: HashSet<String> = HashSet::new();
    if let Some(co) = root_obj.get("compilerOptions").and_then(|v| v.as_object()) {
        for (key, value) in co {
            if value.is_null() {
                explicit_null_fields.insert(key.clone());
            }
        }
    }

    if let Some(co) = root_obj.get("compilerOptions").and_then(|v| v.as_object()) {
        result.raw_options = Some(crate::json::Value::Object(co.clone()));
        let (opts, opts_errors) = json_object_to_options(co);
        result.errors.extend(opts_errors);

        let mut config_opts = CompilerOptions::default();
        apply_options(&opts, &mut config_opts);

        if let Some(paths_val) = co.get("paths").and_then(|v| v.as_object()) {
            let mut paths_map = HashMap::new();
            for (key, val) in paths_val {
                if let Some(arr) = val.as_array() {
                    let targets: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    paths_map.insert(key.clone(), targets);
                }
            }
            config_opts.paths = Some(paths_map);
        }

        let config_dir_for_opts = tspath::get_directory_path(config_file_name);
        resolve_file_path_options(&mut config_opts, &config_dir_for_opts);

        merge_compiler_options(&mut result.compiler_options, &config_opts);
        merge_compiler_options_with_skip(
            &mut result.compiler_options,
            &extended_opts,
            &explicit_null_fields,
        );
    } else {
        merge_compiler_options(&mut result.compiler_options, &extended_opts);
    }

    let config_dir = tspath::get_directory_path(config_file_name);
    handle_config_dir_template_substitution(&mut result.compiler_options, &config_dir);

    if resolution_stack.is_empty() {
        if let Some(substituted) =
            get_substituted_string_array_with_config_dir_template(&result.include, &config_dir)
        {
            result.include = substituted;
        }
        if let Some(substituted) =
            get_substituted_string_array_with_config_dir_template(&result.exclude, &config_dir)
        {
            result.exclude = substituted;
        }
        if let Some(substituted) =
            get_substituted_string_array_with_config_dir_template(&result.files_spec, &config_dir)
        {
            result.files_spec = substituted;
        }
    }

    result.file_names = expand_file_names(
        &result.files_spec,
        result.has_files_spec,
        &result.include,
        result.has_include_spec,
        &result.exclude,
        result.has_exclude_spec,
        &result.compiler_options,
        &config_dir,
        fs,
    );

    if result.file_names.is_empty() && resolution_stack.is_empty() {
        let can_report = !root_obj.contains_key("files") && !root_obj.contains_key("references");
        if can_report {
            let include_json =
                serde_json::to_string(&result.include).unwrap_or_else(|_| "[]".into());
            let exclude_json =
                serde_json::to_string(&result.exclude).unwrap_or_else(|_| "[]".into());
            result.errors.push(Diagnostic::new(
                None,
                TextRange::undefined(),
                NO_INPUTS_WERE_FOUND_IN_CONFIG_FILE_0_SPECIFIED_INCLUDE_PATHS_WERE_1_AND_EXCLUDE_PATHS_WERE_2,
                vec![config_file_name.to_string(), include_json, exclude_json],
            ));
        }
    }

    result
}

pub(crate) fn extends_as_paths(
    extends: &crate::json::Value,
    config_file_name: &str,
    current_dir: &str,
    fs: &dyn FS,
) -> Vec<String> {
    let specs: Vec<String> = match extends {
        crate::json::Value::String(s) => vec![s.clone()],
        crate::json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => return Vec::new(),
    };
    specs
        .into_iter()
        .filter_map(|s| resolve_single_extends_path(&s, config_file_name, current_dir, fs))
        .collect()
}

pub(crate) fn resolve_single_extends_path(
    s: &str,
    config_file_name: &str,
    current_dir: &str,
    fs: &dyn FS,
) -> Option<String> {
    let config_dir = tspath::get_directory_path(config_file_name);

    if tspath::is_external_module_name_relative(s) {
        resolve_relative_extends_path(s, &config_dir, current_dir, fs)
    } else {
        resolve_config_via_node_modules(s, &config_dir, fs)
    }
}
