#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::diagnostics::Message;
use tsox_core::json::Value as JsonValue;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::mig::m3k::TypeAcquisition;
use tsox_core::tspath;
use tsox_frontend::ast::diagnostic::{self, Diagnostic};
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node::SourceFile;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use super::m5j::*;
use crate::tsoptions::build_options::ParsedCommandLine;
use crate::tsoptions::option_kind::OptionDecl;

#[derive(Clone)]
pub struct ParsedTsconfig {
    pub raw: Option<JsonValue>,
    pub options: Option<CompilerOptions>,
    pub type_acquisition: Option<TypeAcquisition>,
    pub extended_config_path: Option<ExtendedConfigPath>,
}

#[derive(Clone)]
pub enum ExtendedConfigPath {
    Single(String),
    Multiple(Vec<String>),
}

pub struct ParseConfigHost {
    pub fs: Arc<dyn FS>,
    pub current_directory: String,
}

use crate::vfs::fs::FS;

pub fn parse_own_config_of_json_source_file(
    source_file: &SourceFile,
    host: &ParseConfigHost,
    base_path: &str,
    config_file_name: &str,
) -> (ParsedTsconfig, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("parse_own_config_of_json_source_file"); 
    let mut compiler_options = get_default_compiler_options(config_file_name);
    let type_acquisition = get_default_type_acquisition(config_file_name);
    let mut extended_config_path: Option<ExtendedConfigPath> = None;
    let mut errors: Vec<Diagnostic> = Vec::new();

    let (json, conversion_errors) = convert_config_file_to_object(source_file, None);
    errors.extend(conversion_errors);

    let mut compiler_options = compiler_options.take().unwrap_or_default();
    let object = match &json {
        Some(JsonValue::Object(_)) => 1,
        _ => 0,
    };
    let _ = object;

    if let Some(JsonValue::Object(entries)) = &json {
        for (key_text, value) in entries {
            if key_text == "extends" {
                if let Some(extends) = value.as_str() {
                    let (config_path, err) = get_extends_config_path_or_array(
                        &JsonValue::String(extends.to_string()),
                        host,
                        base_path,
                        config_file_name,
                    );
                    extended_config_path = Some(config_path);
                    errors.extend(err);
                }
            } else if key_text == "excludes" {
                errors.push(new_compiler_diagnostic(
                    tsox_core::diagnostics::UNKNOWN_OPTION_EXCLUDES_DID_YOU_MEAN_EXCLUDE,
                    vec![],
                ));
            }
            let parse_diagnostics = parse_compiler_options_from_value(key_text, value, base_path, &mut compiler_options);
            errors.extend(parse_diagnostics);
        }
    }

    let _ = &type_acquisition;
    (
        ParsedTsconfig {
            raw: json,
            options: Some(compiler_options),
            type_acquisition,
            extended_config_path,
        },
        errors,
    )
}

fn parse_compiler_options_from_value(
    key: &str,
    value: &JsonValue,
    base_path: &str,
    options: &mut CompilerOptions,
) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("parse_compiler_options_from_value"); 
    let _ = (key, value, base_path, options);
    Vec::new()
}

pub fn read_json_config_file(
    file_name: &str,
    path: &str,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> (TsConfigSourceFile, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("read_json_config_file"); 
    let (text, diagnostic_list) = try_read_file(file_name, read_file);
    if !text.is_empty() {
        let source_file = parse_source_file_for_config(file_name, path, &text);
        (
            TsConfigSourceFile {
                extended_source_files: Vec::new(),
                config_file_specs: None,
                source_file: Arc::new(source_file),
            },
            diagnostic_list,
        )
    } else {
        let source_file = parse_source_file_for_config(file_name, path, "");
        (
            TsConfigSourceFile {
                extended_source_files: Vec::new(),
                config_file_specs: None,
                source_file: Arc::new(source_file),
            },
            diagnostic_list,
        )
    }
}

fn parse_source_file_for_config(file_name: &str, _path: &str, text: &str) -> SourceFile { ::tsox_core::fntrace::enter("parse_source_file_for_config"); 
    tsox_frontend::parser::Parser::parse_source_file_text(file_name, text.to_string())
}

pub fn parse_own_config_of_json(
    json: &mut JsonObject,
    host: &ParseConfigHost,
    base_path: &str,
    config_file_name: &str,
) -> (ParsedTsconfig, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("parse_own_config_of_json"); 
    let mut errors: Vec<Diagnostic> = Vec::new();
    if json_object_has(json, "excludes") {
        errors.push(new_compiler_diagnostic(
            tsox_core::diagnostics::UNKNOWN_OPTION_EXCLUDES_DID_YOU_MEAN_EXCLUDE,
            vec![],
        ));
    }
    let (mut options, err) = convert_compiler_options_from_json_worker(json_object_get(json, "compilerOptions"), base_path, config_file_name);
    errors.extend(err);
    let (type_acquisition, err2) =
        convert_type_acquisition_from_json_worker(json_object_get(json, "typeAcquisition"), base_path, config_file_name);
    errors.extend(err2);
    if let Some(compile_on_save) = json_object_get(json, "compileOnSave").cloned() {
        let (converted, compile_on_save_errors) = convert_json_option(
            &COMPILE_ON_SAVE_COMMAND_LINE_OPTION,
            &compile_on_save,
            base_path,
            None,
            None,
            None,
        );
        errors.extend(compile_on_save_errors);
        if let Some(converted) = converted {
            json_object_set(json, "compileOnSave", converted);
        }
    }
    let mut extended_config_path: Option<ExtendedConfigPath> = None;
    if let Some(extends) = json_object_get(json, "extends") {
        let not_empty = match extends {
            JsonValue::String(s) => !s.is_empty(),
            JsonValue::Null => false,
            _ => true,
        };
        if not_empty {
            let (config_path, err) = get_extends_config_path_or_array(extends, host, base_path, config_file_name);
            extended_config_path = Some(config_path);
            errors.extend(err);
        }
    }
    (
        ParsedTsconfig {
            raw: Some(object_to_json_value(json)),
            options,
            type_acquisition,
            extended_config_path,
        },
        errors,
    )
}

pub fn object_to_json_value(obj: &JsonObject) -> JsonValue { ::tsox_core::fntrace::enter("object_to_json_value"); 
    let map = obj.iter().cloned().collect::<serde_json::Map<String, JsonValue>>();
    JsonValue::Object(map)
}

pub fn convert_compiler_options_from_json_worker(
    json_options: Option<&JsonValue>,
    base_path: &str,
    config_file_name: &str,
) -> (Option<CompilerOptions>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_compiler_options_from_json_worker"); 
    let (mut options, errors) = convert_options_from_json_worker(json_options, base_path, config_file_name);
    if !config_file_name.is_empty() {
        if let Some(options) = options.as_mut() {
            options.config_file_path = tspath::normalize_slashes(config_file_name);
        }
    }
    (options, errors)
}

fn convert_options_from_json_worker(
    json_options: Option<&JsonValue>,
    base_path: &str,
    config_file_name: &str,
) -> (Option<CompilerOptions>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_options_from_json_worker"); 
    let mut options = get_default_compiler_options(config_file_name);
    let mut errors: Vec<Diagnostic> = Vec::new();
    if let Some(JsonValue::Object(entries)) = json_options {
        for (key, value) in entries {
            let (converted, errs) =
                convert_json_option(&COMPILER_OPTIONS_DECLARATION, value, base_path, None, None, None);
            errors.extend(errs);
            if let Some(converted) = converted {
                if let Some(options) = options.as_mut() {
                    set_option_value(options, key, &converted);
                }
            }
        }
    }
    (options, errors)
}

fn set_option_value(_options: &mut CompilerOptions, _key: &str, _value: &JsonValue) { ::tsox_core::fntrace::enter("set_option_value"); }

pub fn convert_type_acquisition_from_json_worker(
    json_options: Option<&JsonValue>,
    base_path: &str,
    config_file_name: &str,
) -> (Option<TypeAcquisition>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_type_acquisition_from_json_worker"); 
    let options = get_default_type_acquisition(config_file_name);
    let mut errors: Vec<Diagnostic> = Vec::new();
    let _ = (json_options, base_path);
    (options, errors)
}

pub fn get_default_compiler_options(config_file_name: &str) -> Option<CompilerOptions> { ::tsox_core::fntrace::enter("get_default_compiler_options"); 
    Some(CompilerOptions::default())
}

pub fn get_default_type_acquisition(config_file_name: &str) -> Option<TypeAcquisition> { ::tsox_core::fntrace::enter("get_default_type_acquisition"); 
    Some(TypeAcquisition::default())
}

pub fn convert_json_option(
    opt: &OptionDecl,
    value: &JsonValue,
    base_path: &str,
    property_assignment: Option<&Node>,
    value_expression: Option<&Node>,
    source_file: Option<&SourceFile>,
) -> (Option<JsonValue>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_json_option"); 
    let _ = (property_assignment, source_file);
    let normalized = normalize_non_list_option_value(opt, base_path, value.clone());
    (Some(normalized), Vec::new())
}

pub static COMPILER_OPTIONS_DECLARATION: OptionDecl = OptionDecl {
    name: "compilerOptions",
    short_name: None,
    kind: crate::tsoptions::option_kind::OptionKind::Enum,
    is_file_path: false,
    is_tsconfig_only: false,
    is_command_line_only: false,
    extra_validation: crate::tsoptions::option_kind::ExtraValidation::None,
    min_value: None,
    enum_values: None,
    description: "",
    show_in_simplified_help: false,
};

pub static COMPILE_ON_SAVE_COMMAND_LINE_OPTION: OptionDecl = OptionDecl {
    name: "compileOnSave",
    short_name: None,
    kind: crate::tsoptions::option_kind::OptionKind::Boolean,
    is_file_path: false,
    is_tsconfig_only: false,
    is_command_line_only: false,
    extra_validation: crate::tsoptions::option_kind::ExtraValidation::None,
    min_value: None,
    enum_values: None,
    description: "",
    show_in_simplified_help: false,
};

pub fn get_extends_config_path_or_array(
    value: &JsonValue,
    host: &ParseConfigHost,
    base_path: &str,
    config_file_name: &str,
) -> (ExtendedConfigPath, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("get_extends_config_path_or_array"); 
    let _ = (value, host, base_path, config_file_name);
    (ExtendedConfigPath::Single(String::new()), Vec::new())
}

fn entries_to_object(entries: Vec<(String, JsonValue)>) -> JsonValue { ::tsox_core::fntrace::enter("entries_to_object"); 
    let map = entries.into_iter().collect::<serde_json::Map<String, JsonValue>>();
    JsonValue::Object(map)
}

fn ts_config_source_file_statement_expression<'a>(_source_file: &'a SourceFile) -> Option<&'a Node> { ::tsox_core::fntrace::enter("ts_config_source_file_statement_expression"); 
    None
}

fn ast_is_object_literal_expression(_node: &Node) -> bool { ::tsox_core::fntrace::enter("ast_is_object_literal_expression"); 
    false
}

pub fn convert_config_file_to_object(
    source_file: &SourceFile,
    json_conversion_notifier: Option<&JsonConversionNotifier>,
) -> (Option<JsonValue>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_config_file_to_object"); 
    let _ = json_conversion_notifier;
    let root_expression = ts_config_source_file_statement_expression(source_file);
    match root_expression {
        Some(expr) if ast_is_object_literal_expression(expr) => convert_to_json(source_file, Some(expr), true, None),
        _ => (Some(entries_to_object(Vec::new())), Vec::new()),
    }
}

pub struct JsonConversionNotifier {
    pub root_options: Option<&'static OptionDecl>,
}

impl JsonConversionNotifier {
    pub fn on_property_set(
        &self,
        _key_text: &str,
        _value: Option<JsonValue>,
        _element: &Node,
        _object_option: Option<&OptionDecl>,
        _option: Option<&OptionDecl>,
    ) -> (Option<JsonValue>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("on_property_set"); 
        (None, Vec::new())
    }
}

pub fn convert_to_json(
    source_file: &SourceFile,
    root_expression: Option<&Node>,
    return_value: bool,
    json_conversion_notifier: Option<&JsonConversionNotifier>,
) -> (Option<JsonValue>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_to_json"); 
    let _ = (source_file, return_value, json_conversion_notifier);
    if root_expression.is_none() {
        return (Some(JsonValue::Object(serde_json::Map::new())), Vec::new());
    }
    (Some(entries_to_object(Vec::new())), Vec::new())
}

pub fn parse_config(
    json: Option<&mut JsonObject>,
    source_file: Option<&TsConfigSourceFile>,
    host: &ParseConfigHost,
    base_path: &str,
    config_file_name: &str,
    resolution_stack: &[String],
    extended_config_cache: Option<&ExtendedConfigCacheValue>,
) -> (ParsedTsconfig, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("parse_config"); 
    let base_path = tspath::normalize_slashes(base_path);
    let resolved_path = tspath::to_path(
        config_file_name,
        &base_path,
        host.fs.use_case_sensitive_file_names(),
    );
    let mut errors: Vec<Diagnostic> = Vec::new();
    if resolution_stack.iter().any(|p| p.as_str() == resolved_path.as_str()) {
        let raw = json.map(|json| object_to_json_value(json));
        return (
            ParsedTsconfig {
                raw,
                options: None,
                type_acquisition: None,
                extended_config_path: None,
            },
            vec![new_compiler_diagnostic(
                tsox_core::diagnostics::CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0,
                vec![resolved_path.to_string()],
            )],
        );
    }
    let (mut own_config, err) = match json {
        Some(json) => parse_own_config_of_json(json, host, &base_path, config_file_name),
        None => parse_own_config_of_json_source_file_by_tsconfig(source_file, host, &base_path, config_file_name),
    };
    errors.extend(err);
    if let Some(options) = own_config.options.as_mut() {
        if options.paths.is_some() {
            options.paths_base_path = base_path.clone();
        }
    }
    let _ = extended_config_cache;
    (own_config, errors)
}

fn parse_own_config_of_json_source_file_by_tsconfig(
    source_file: Option<&TsConfigSourceFile>,
    host: &ParseConfigHost,
    base_path: &str,
    config_file_name: &str,
) -> (ParsedTsconfig, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("parse_own_config_of_json_source_file_by_tsconfig"); 
    match source_file {
        Some(sf) => parse_own_config_of_json_source_file(&sf.source_file, host, base_path, config_file_name),
        None => (
            ParsedTsconfig {
                raw: None,
                options: get_default_compiler_options(config_file_name),
                type_acquisition: get_default_type_acquisition(config_file_name),
                extended_config_path: None,
            },
            Vec::new(),
        ),
    }
}

pub struct ExtendedConfigCacheValue;

pub fn parse_json_config_file_content_worker(
    mut json: Option<&mut JsonObject>,
    mut source_file: Option<&mut TsConfigSourceFile>,
    host: &ParseConfigHost,
    base_path: &str,
    existing_options: Option<&CompilerOptions>,
    existing_options_raw: Option<&JsonObject>,
    config_file_name: &str,
    resolution_stack: &[String],
    extended_config_cache: Option<&ExtendedConfigCacheValue>,
) -> ParsedCommandLine { ::tsox_core::fntrace::enter("parse_json_config_file_content_worker"); 
    let _ = (existing_options, existing_options_raw, extended_config_cache);
    let base_path_for_file_names = if !config_file_name.is_empty() {
        tspath::normalize_path(&directory_of_combined_path(config_file_name, base_path))
    } else {
        tspath::normalize_path(base_path)
    };
    let (mut parsed_config, mut errors) = {
        let (parsed, errs) = parse_config(json.as_deref_mut(), source_file.as_deref(), host, base_path, config_file_name, resolution_stack, None);
        (parsed, errs)
    };
    if !config_file_name.is_empty() {
        if let Some(options) = parsed_config.options.as_mut() {
            options.config_file_path = tspath::normalize_slashes(config_file_name);
        }
    }
    let raw_config = parse_json_to_string_key(parsed_config.raw.as_ref());
    handle_option_config_dir_template_substitution(parsed_config.options.as_mut().unwrap_or(&mut CompilerOptions::default()), &base_path_for_file_names);

    let mut file_spec_refs = get_prop_from_raw(&raw_config, "files");
    let include_specs = get_prop_from_raw(&raw_config, "include");
    let mut exclude_specs = get_prop_from_raw(&raw_config, "exclude");

    let mut is_default_include_spec = false;
    if exclude_specs.is_none() {
        if let Some(options) = parsed_config.options.as_ref() {
            let mut values: Vec<String> = Vec::new();
            if !options.out_dir.is_empty() {
                values.push(options.out_dir.clone());
            }
            if !options.declaration_dir.is_empty() {
                values.push(options.declaration_dir.clone());
            }
            if !values.is_empty() {
                exclude_specs = Some(values);
            }
        }
    }
    if file_spec_refs.is_none() && include_specs.is_none() {
        file_spec_refs = None;
        is_default_include_spec = true;
        let _ = DEFAULT_INCLUDE_SPEC;
    }

    let mut validated_include_specs: Vec<String> = Vec::new();
    let mut validated_exclude_specs: Vec<String> = Vec::new();
    let mut validated_files_spec: Vec<String> = Vec::new();
    let mut validated_files_spec_before_substitution: Vec<String> = Vec::new();
    let mut validated_include_specs_before_substitution: Vec<String> = Vec::new();

    if let Some(include_list) = include_specs.as_ref() {
        let (specs, err) = validate_specs(
            &string_slice_to_json(include_list),
            true,
            source_file.as_deref().map(|s| s.source_file.as_ref()),
            "include",
        );
        errors.extend(err);
        validated_include_specs_before_substitution = specs;
        validated_include_specs = get_substituted_string_array_with_config_dir_template(
            &validated_include_specs_before_substitution,
            &base_path_for_file_names,
        )
        .unwrap_or_else(|| validated_include_specs_before_substitution.clone());
    }
    if let Some(exclude_list) = exclude_specs.as_ref() {
        let (specs, err) = validate_specs(
            &string_slice_to_json(exclude_list),
            false,
            source_file.as_deref().map(|s| s.source_file.as_ref()),
            "exclude",
        );
        errors.extend(err);
        validated_exclude_specs = specs;
        if let Some(substituted) =
            get_substituted_string_array_with_config_dir_template(&validated_exclude_specs, &base_path_for_file_names)
        {
            validated_exclude_specs = substituted;
        }
    }
    if let Some(files_list) = file_spec_refs.as_ref() {
        validated_files_spec_before_substitution = files_list.clone();
        validated_files_spec = get_substituted_string_array_with_config_dir_template(
            &validated_files_spec_before_substitution,
            &base_path_for_file_names,
        )
        .unwrap_or_else(|| validated_files_spec_before_substitution.clone());
    }

    let config_file_specs = ConfigFileSpecs {
        files_specs: None,
        include_specs: None,
        exclude_specs: None,
        validated_files_spec,
        validated_include_specs,
        validated_exclude_specs,
        validated_files_spec_before_substitution,
        validated_include_specs_before_substitution,
        is_default_include_spec,
    };
    if let Some(source_file) = source_file.as_deref_mut() {
        source_file.config_file_specs = Some(config_file_specs.clone());
    }

    let file_names: Vec<String> = config_file_specs.validated_files_spec.clone();
    let literal_file_names_len = file_names.len();
    if should_report_no_input_files(
        &file_names,
        can_json_report_no_input_files(&raw_config),
        resolution_stack,
    ) {
        errors.push(new_compiler_diagnostic(
            tsox_core::diagnostics::NO_INPUTS_WERE_FOUND_IN_CONFIG_FILE_0_SPECIFIED_INCLUDE_PATHS_WERE_1_AND_EXCLUDE_PATHS_WERE_2,
            vec![
                config_file_name.to_string(),
                string_slice_to_json(&config_file_specs.validated_include_specs).to_string(),
                string_slice_to_json(&config_file_specs.validated_exclude_specs).to_string(),
            ],
        ));
    }

    let mut result = ParsedCommandLine::default();
    result.file_names = file_names;
    result.errors = errors;
    result.config_file_name = config_file_name.to_string();
    result.compiler_options = parsed_config.options.unwrap_or_default();
    result.include = config_file_specs.validated_include_specs.clone();
    result.exclude = config_file_specs.validated_exclude_specs.clone();
    result.files_spec = config_file_specs.validated_files_spec.clone();
    result.has_include_spec = config_file_specs.validated_include_specs.is_empty();
    result.has_exclude_spec = config_file_specs.validated_exclude_specs.is_empty();
    result.has_files_spec = config_file_specs.validated_files_spec.is_empty();
    result
}

fn string_slice_to_json(list: &[String]) -> JsonValue { ::tsox_core::fntrace::enter("string_slice_to_json"); 
    JsonValue::Array(list.iter().map(|s| JsonValue::String(s.clone())).collect())
}

fn get_prop_from_raw(raw_config: &JsonObject, prop: &str) -> Option<Vec<String>> { ::tsox_core::fntrace::enter("get_prop_from_raw"); 
    match json_object_get(raw_config, prop) {
        Some(JsonValue::Array(items)) => Some(
            items
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        ),
        _ => None,
    }
}

pub fn can_json_report_no_input_files(raw_config: &JsonObject) -> bool { ::tsox_core::fntrace::enter("can_json_report_no_input_files"); 
    let files_exists = json_object_has(raw_config, "files");
    let references_exists = json_object_has(raw_config, "references");
    !files_exists && !references_exists
}

pub fn parse_json_to_string_key(raw: Option<&JsonValue>) -> JsonObject { ::tsox_core::fntrace::enter("parse_json_to_string_key"); 
    match raw {
        Some(JsonValue::Object(map)) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        _ => Vec::new(),
    }
}

pub fn directory_of_combined_path(config_file_name: &str, base_path: &str) -> String { ::tsox_core::fntrace::enter("directory_of_combined_path"); 
    let combined = tspath::combine_paths(base_path, &[config_file_name]);
    tspath::get_directory_path(&combined)
}

pub fn try_read_file(file_name: &str, read_file: &dyn Fn(&str) -> Option<String>) -> (String, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("try_read_file"); 
    match read_file(file_name) {
        Some(text) => (text, Vec::new()),
        None => (
            String::new(),
            vec![new_compiler_diagnostic(
                tsox_core::diagnostics::CANNOT_READ_FILE_0,
                vec![file_name.to_string()],
            )],
        ),
    }
}
