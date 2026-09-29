#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::diagnostics::Message;
use tsox_core::json::Value as JsonValue;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_core::tspath::{self, ComparePathsOptions};
use tsox_core::tspath::mig::m3i as tspath_mig;
use tsox_frontend::ast::diagnostic::{self, Diagnostic};
use tsox_frontend::ast::node_data_generated::{is_array_literal_expression, is_object_literal_expression, NodeData};
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node::SourceFile;

use super::m5i2::*;
use super::m5i2_2::*;
use super::m5i2_3::*;
use super::m5j::*;
use super::m5j_2::*;
use super::m5j_3::parse_json_source_file_config_file_content;
use super::m5i_3::{add_implied_options, filter_same_as_default_include, serialize_compiler_options, TSConfig};
use crate::tsoptions::build_options::ParsedCommandLine;
use crate::tsoptions::option_kind::{ExtraValidation, OptionDecl, OptionKind};

use std::collections::HashMap;

pub static EXTENDS_OPTION_DECLARATION: OptionDecl = OptionDecl {
    name: "extends",
    short_name: None,
    kind: OptionKind::List,
    is_file_path: false,
    is_tsconfig_only: false,
    is_command_line_only: false,
    extra_validation: ExtraValidation::None,
    min_value: None,
    enum_values: None,
    description: "",
    show_in_simplified_help: false,
};

static EXTENDS_ELEMENT_OPTION_DECLARATION: OptionDecl = OptionDecl {
    name: "extends",
    short_name: None,
    kind: OptionKind::String,
    is_file_path: false,
    is_tsconfig_only: false,
    is_command_line_only: false,
    extra_validation: ExtraValidation::None,
    min_value: None,
    enum_values: None,
    description: "",
    show_in_simplified_help: false,
};

pub fn option_elements(decl: &OptionDecl) -> OptionDecl {
    if !matches!(decl.kind, OptionKind::List) {
        return *decl;
    }
    EXTENDS_ELEMENT_OPTION_DECLARATION
}

fn compiler_options_value_table(options: &CompilerOptions) -> HashMap<String, JsonValue> {
    let _ = options;
    HashMap::new()
}

fn option_decls_by_json_name() -> HashMap<String, OptionDecl> {
    HashMap::new()
}

fn default_value_map() -> HashMap<String, bool> {
    HashMap::new()
}

fn compiler_options_from_json_map(option_map: &serde_json::Map<String, JsonValue>) -> CompilerOptions {
    let _ = option_map;
    CompilerOptions::default()
}

fn skip_trivia(_text: &str, pos: usize) -> usize {
    pos
}

pub fn get_extended_config(
    source_file: Option<&TsConfigSourceFile>,
    extended_config_file_name: &str,
    host: &ParseConfigHost,
    resolution_stack: &[String],
    extended_config_cache: Option<&ExtendedConfigCacheValue>,
    extended_source_files: &mut Vec<String>,
) -> (Option<ParsedTsconfig>, Vec<Diagnostic>) {
    let _ = source_file;
    let mut errors: Vec<Diagnostic> = Vec::new();
    let extended_config_path = tspath::to_path(
        extended_config_file_name,
        &host.current_directory,
        host.fs.use_case_sensitive_file_names(),
    );
    let cache_entry = parse_extended_config(
        extended_config_file_name,
        extended_config_path.as_str(),
        resolution_stack,
        host,
        extended_config_cache,
    );
    errors.extend(cache_entry.errors.iter().cloned());
    if let Some(extended_result) = &cache_entry.extended_result {
        extended_source_files.push(extended_result.source_file.file_name.clone());
        for extended_source_file in &extended_result.extended_source_files {
            extended_source_files.push(extended_source_file.clone());
        }
    }
    (cache_entry.extended_config, errors)
}

pub fn parse_extended_config(
    file_name: &str,
    path: &str,
    resolution_stack: &[String],
    host: &ParseConfigHost,
    extended_config_cache: Option<&ExtendedConfigCacheValue>,
) -> ExtendedConfigCacheEntry {
    let read_file = |name: &str| host.fs.read_file(name);
    let (extended_result, read_errors) = read_json_config_file(file_name, path, &read_file);
    if !read_errors.is_empty() {
        return ExtendedConfigCacheEntry {
            extended_result: Some(extended_result),
            extended_config: None,
            errors: read_errors,
        };
    }
    let parse_diagnostics =
        tsox_frontend::ast::mig::m3b_2::diagnostics(extended_result.source_file.as_ref()).to_vec();
    if !parse_diagnostics.is_empty() {
        return ExtendedConfigCacheEntry {
            extended_result: Some(extended_result),
            extended_config: None,
            errors: parse_diagnostics,
        };
    }
    let (extended_config, parse_errors) = parse_config(
        None,
        Some(&extended_result),
        host,
        &tspath::get_directory_path(file_name),
        &tspath::get_base_file_name(file_name),
        resolution_stack,
        extended_config_cache,
    );
    ExtendedConfigCacheEntry {
        extended_result: Some(extended_result),
        extended_config: Some(extended_config),
        errors: parse_errors,
    }
}

pub fn get_extends_config_path(
    extended_config: &str,
    host: &ParseConfigHost,
    base_path: &str,
    value_expression: Option<&Node>,
    source_file: Option<&SourceFile>,
) -> (String, Vec<Diagnostic>) {
    let extended_config = tspath::normalize_slashes(extended_config);
    let mut errors: Vec<Diagnostic> = Vec::new();
    let error_file = source_file;
    if tspath::is_rooted_disk_path(&extended_config)
        || extended_config.starts_with("./")
        || extended_config.starts_with("../")
    {
        let mut extended_config_path = tspath::get_normalized_absolute_path(&extended_config, base_path);
        if !host.fs.file_exists(&extended_config_path) && !extended_config_path.ends_with(tspath::EXTENSION_JSON) {
            extended_config_path = format!("{}{}", extended_config_path, tspath::EXTENSION_JSON);
            if !host.fs.file_exists(&extended_config_path) {
                errors.push(create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                    error_file,
                    value_expression,
                    tsox_core::diagnostics::FILE_0_NOT_FOUND,
                    vec![extended_config.clone()],
                ));
                return (String::new(), errors);
            }
        }
        return (extended_config_path, errors);
    }
    if let Some(resolved_file_name) = crate::tsoptions::resolve_config_via_node_modules(
        &extended_config,
        &tspath::combine_paths(base_path, &["tsconfig.json"]),
        &*host.fs,
    ) {
        return (resolved_file_name, errors);
    }
    if extended_config.is_empty() {
        errors.push(create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
            error_file,
            value_expression,
            tsox_core::diagnostics::COMPILER_OPTION_0_CANNOT_BE_GIVEN_AN_EMPTY_STRING,
            vec!["extends".to_string()],
        ));
    } else {
        errors.push(create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
            error_file,
            value_expression,
            tsox_core::diagnostics::FILE_0_NOT_FOUND,
            vec![extended_config.clone()],
        ));
    }
    (String::new(), errors)
}

pub fn get_extends_config_path_or_array(
    value: &JsonValue,
    host: &ParseConfigHost,
    base_path: &str,
    config_file_name: &str,
    property_assignment: Option<&Node>,
    value_expression: Option<&Node>,
    source_file: Option<&SourceFile>,
) -> (Vec<String>, Vec<Diagnostic>) {
    let _ = property_assignment;
    let mut extended_config_path_array: Vec<String> = Vec::new();
    let mut new_base = base_path.to_string();
    if !config_file_name.is_empty() {
        new_base = directory_of_combined_path(config_file_name, base_path);
    }
    if value.is_null() {
        let (_, errors) = convert_json_option(
            &EXTENDS_OPTION_DECLARATION,
            value,
            base_path,
            property_assignment,
            value_expression,
            source_file,
        );
        return (extended_config_path_array, errors);
    }
    if let Some(val) = value.as_str() {
        let (val, err) = get_extends_config_path(val, host, &new_base, value_expression, source_file);
        if !val.is_empty() {
            extended_config_path_array.push(val);
        }
        return (extended_config_path_array, err);
    }
    let mut errors: Vec<Diagnostic> = Vec::new();
    if let Some(list) = value.as_array() {
        for (index, file_name) in list.iter().enumerate() {
            let expression: Option<&Node> = value_expression.and_then(|value_expression| match &value_expression.data {
                NodeData::ArrayLiteralExpression(data) => data.elements.nodes.get(index).map(|node| node.as_ref()),
                _ => None,
            });
            if let Some(file_name) = file_name.as_str() {
                let (val, err) = get_extends_config_path(file_name, host, &new_base, expression, source_file);
                if !val.is_empty() {
                    extended_config_path_array.push(val);
                }
                errors.extend(err);
            } else {
                let (_, err) = convert_json_option(
                    &option_elements(&EXTENDS_OPTION_DECLARATION),
                    file_name,
                    base_path,
                    property_assignment,
                    expression,
                    source_file,
                );
                errors.extend(err);
            }
        }
    } else {
        let (_, errs) = convert_json_option(
            &EXTENDS_OPTION_DECLARATION,
            value,
            base_path,
            property_assignment,
            value_expression,
            source_file,
        );
        errors = errs;
    }
    (extended_config_path_array, errors)
}

pub fn parse_json_config_file_content(
    json: &JsonValue,
    host: &ParseConfigHost,
    base_path: &str,
    existing_options: Option<&CompilerOptions>,
    config_file_name: &str,
    resolution_stack: &[String],
    extended_config_cache: Option<&ExtendedConfigCacheValue>,
) -> ParsedCommandLine {
    let normalized = normalize_json_value(json);
    let mut json_object: JsonObject = match &normalized {
        JsonValue::Object(entries) => entries
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
        _ => Vec::new(),
    };
    parse_json_config_file_content_worker(
        Some(&mut json_object),
        None,
        host,
        base_path,
        existing_options,
        None,
        config_file_name,
        resolution_stack,
        extended_config_cache,
    )
}

pub fn get_parsed_command_line_of_config_file(
    config_file_name: &str,
    options: Option<&CompilerOptions>,
    options_raw: Option<&JsonObject>,
    sys: &ParseConfigHost,
    extended_config_cache: Option<&ExtendedConfigCacheValue>,
) -> (Option<ParsedCommandLine>, Vec<Diagnostic>) {
    let config_file_name = tspath::get_normalized_absolute_path(config_file_name, &sys.current_directory);
    let path = tspath::to_path(
        &config_file_name,
        &sys.current_directory,
        sys.fs.use_case_sensitive_file_names(),
    );
    get_parsed_command_line_of_config_file_path(
        &config_file_name,
        path.as_str(),
        options,
        options_raw,
        sys,
        extended_config_cache,
    )
}

pub fn get_parsed_command_line_of_config_file_path(
    config_file_name: &str,
    path: &str,
    options: Option<&CompilerOptions>,
    options_raw: Option<&JsonObject>,
    sys: &ParseConfigHost,
    extended_config_cache: Option<&ExtendedConfigCacheValue>,
) -> (Option<ParsedCommandLine>, Vec<Diagnostic>) {
    let mut errors: Vec<Diagnostic> = Vec::new();
    let read_file = |name: &str| sys.fs.read_file(name);
    let (config_file_text, read_errors) = try_read_file(config_file_name, &read_file);
    errors.extend(read_errors);
    if !errors.is_empty() {
        return (None, errors);
    }
    let ts_config_source_file = new_tsconfig_source_file_from_file_path(config_file_name, path, &config_file_text);
    (
        Some(parse_json_source_file_config_file_content(
            &ts_config_source_file,
            sys,
            &tspath::get_directory_path(config_file_name),
            options,
            options_raw,
            config_file_name,
            &[],
            extended_config_cache,
        )),
        Vec::new(),
    )
}

pub fn convert_to_ts_config(config_parse_result: &ParsedCommandLine, config_file_name: &str) -> TSConfig {
    let config_file_name = if config_file_name.is_empty() {
        "tsconfig.json"
    } else {
        config_file_name
    };
    let current_directory = config_parse_result.get_current_directory();
    let normalized_config_path = tspath::get_normalized_absolute_path(config_file_name, &current_directory);
    let compare_paths_options = ComparePathsOptions {
        current_directory: current_directory.clone(),
        use_case_sensitive_file_names: config_parse_result.use_case_sensitive_file_names(),
    };

    let mut files: Vec<String> = Vec::new();
    for file_name in &config_parse_result.file_names {
        let normalized_file_path = tspath::get_normalized_absolute_path(file_name, &current_directory);
        files.push(tspath_mig::get_relative_path_from_file(
            &normalized_config_path,
            &normalized_file_path,
            &compare_paths_options,
        ));
    }

    let value_table = compiler_options_value_table(&config_parse_result.compiler_options);
    let mut option_map = serialize_compiler_options(
        &config_parse_result.compiler_options,
        &option_decls_by_json_name(),
        &value_table,
        &normalized_config_path,
        (&current_directory, config_parse_result.use_case_sensitive_file_names()),
    );
    for name in [
        "showConfig",
        "configFile",
        "configFilePath",
        "help",
        "init",
        "listFilesOnly",
        "listEmittedFiles",
        "project",
        "build",
        "version",
    ] {
        option_map.shift_remove(name);
    }
    add_implied_options(
        &config_parse_result.compiler_options,
        &option_decls_by_json_name(),
        &value_table,
        &mut option_map,
        &normalized_config_path,
        (&current_directory, config_parse_result.use_case_sensitive_file_names()),
        &default_value_map(),
    );

    let mut config = TSConfig {
        compiler_options: compiler_options_from_json_map(&option_map),
        files: Vec::new(),
        include: Vec::new(),
        exclude: Vec::new(),
        references: Vec::new(),
        type_acquisition: None,
    };
    if !config_parse_result.references.is_empty() {
        config.references = config_parse_result.references.clone();
    }
    if !files.is_empty() {
        config.files = files;
    }
    let include = filter_same_as_default_include(&config_parse_result.include);
    if !include.is_empty() {
        config.include = include;
    }
    config.exclude = config_parse_result.exclude.clone();
    config
}
