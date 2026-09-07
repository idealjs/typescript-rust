#![allow(unused_imports)]

use super::*;
use crate::json::Value;
use crate::tsoptions as opts;

pub(crate) fn insert_enum_options(
    map: &mut crate::json::Map<String, crate::json::Value>,
    options: &CompilerOptions,
) {
    if let Some(s) = opts::script_target_name(options.target) {
        map.insert("target".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::module_kind_name(options.module) {
        map.insert("module".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::module_resolution_name(options.module_resolution) {
        map.insert("moduleResolution".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::jsx_emit_name(options.jsx) {
        map.insert("jsx".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::module_detection_name(options.module_detection) {
        map.insert("moduleDetection".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::new_line_name(options.new_line) {
        map.insert("newLine".to_string(), Value::String(s.to_string()));
    }
}

pub(crate) fn insert_path_and_string_options(
    map: &mut crate::json::Map<String, crate::json::Value>,
    options: &CompilerOptions,
    config_file_name: &str,
) {
    let config_dir = tspath::get_directory_path(config_file_name);
    let to_relative = |val: &str| -> String {
        if val.is_empty() {
            return val.to_string();
        }

        if !tspath::path_is_absolute(val) {
            return val.to_string();
        }

        let abs_val = tspath::get_normalized_absolute_path(val, "");
        let abs_config_dir = tspath::get_normalized_absolute_path(&config_dir, "");
        let abs_config_dir_with_sep = tspath::ensure_trailing_directory_separator(&abs_config_dir);
        if abs_val == abs_config_dir {
            return ".".to_string();
        }
        if let Some(stripped) = abs_val.strip_prefix(&abs_config_dir_with_sep) {
            return stripped.to_string();
        }
        val.to_string()
    };
    for (name, val, is_path) in [
        ("outDir", &options.out_dir, true),
        ("outFile", &options.out_file, true),
        ("rootDir", &options.root_dir, true),
        ("declarationDir", &options.declaration_dir, true),
        ("sourceRoot", &options.source_root, true),
        ("mapRoot", &options.map_root, true),
        ("tsBuildInfoFile", &options.ts_build_info_file, true),
        ("jsxFactory", &options.jsx_factory, false),
        ("jsxFragmentFactory", &options.jsx_fragment_factory, false),
        ("jsxImportSource", &options.jsx_import_source, false),
        ("baseUrl", &options.base_url, true),
        ("locale", &options.locale, false),
    ] {
        if !val.is_empty() {
            let display = if is_path {
                to_relative(val)
            } else {
                val.clone()
            };
            map.insert(name.to_string(), Value::String(display));
        }
    }
}

pub(crate) fn insert_list_options(
    map: &mut crate::json::Map<String, crate::json::Value>,
    options: &CompilerOptions,
) {
    if !options.lib.is_empty() {
        map.insert(
            "lib".to_string(),
            Value::Array(
                options
                    .lib
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.types.is_empty() {
        map.insert(
            "types".to_string(),
            Value::Array(
                options
                    .types
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.type_roots.is_empty() {
        map.insert(
            "typeRoots".to_string(),
            Value::Array(
                options
                    .type_roots
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.root_dirs.is_empty() {
        map.insert(
            "rootDirs".to_string(),
            Value::Array(
                options
                    .root_dirs
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.module_suffixes.is_empty() {
        map.insert(
            "moduleSuffixes".to_string(),
            Value::Array(
                options
                    .module_suffixes
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.custom_conditions.is_empty() {
        map.insert(
            "customConditions".to_string(),
            Value::Array(
                options
                    .custom_conditions
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }

    if let Some(paths) = &options.paths {
        let mut paths_map = crate::json::Map::new();
        for (k, v) in paths {
            paths_map.insert(
                k.clone(),
                Value::Array(v.iter().map(|s| Value::String(s.clone())).collect()),
            );
        }
        map.insert("paths".to_string(), Value::Object(paths_map));
    }
}
