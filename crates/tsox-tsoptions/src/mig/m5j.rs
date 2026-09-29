#![allow(unused_imports, dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::diagnostics::Message;
use tsox_core::json::Value as JsonValue;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_core::tspath;
use tsox_frontend::ast::diagnostic::{self, Diagnostic};
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::tsoptions::option_kind::{ExtraValidation, OptionDecl, OptionKind};
use crate::tsoptions::build_options::ParsedCommandLine;
use crate::vfs::fs::FS;

pub const CONFIG_DIR_TEMPLATE: &str = "${configDir}";
pub const DEFAULT_INCLUDE_SPEC: &str = "**/*";

pub const NO_STRUCTURED_DATA: u32 = 0xFFFFFFFF;

pub type JsonObject = Vec<(String, JsonValue)>;

pub fn json_object_get<'a>(obj: &'a JsonObject, key: &str) -> Option<&'a JsonValue> {
    obj.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

pub fn json_object_has(obj: &JsonObject, key: &str) -> bool {
    json_object_get(obj, key).is_some()
}

pub fn json_object_set(obj: &mut JsonObject, key: &str, value: JsonValue) {
    if let Some(slot) = obj.iter_mut().find(|(k, _)| k == key) {
        slot.1 = value;
    } else {
        obj.push((key.to_string(), value));
    }
}

pub fn tsconfig_to_source_file(tsconfig_source_file: Option<&TsConfigSourceFile>) -> Option<&SourceFile> {
    tsconfig_source_file.map(|t| t.source_file.as_ref())
}

#[derive(Debug)]
pub struct TsConfigSourceFile {
    pub extended_source_files: Vec<String>,
    pub config_file_specs: Option<ConfigFileSpecs>,
    pub source_file: Arc<SourceFile>,
}

#[derive(Default, Clone, Debug)]
pub struct ConfigFileSpecs {
    pub files_specs: Option<JsonValue>,
    pub include_specs: Option<JsonValue>,
    pub exclude_specs: Option<JsonValue>,
    pub validated_files_spec: Vec<String>,
    pub validated_include_specs: Vec<String>,
    pub validated_exclude_specs: Vec<String>,
    pub validated_files_spec_before_substitution: Vec<String>,
    pub validated_include_specs_before_substitution: Vec<String>,
    pub is_default_include_spec: bool,
}

impl ConfigFileSpecs {
    pub fn matches_exclude(&self, file_name: &str, compare_paths_options: &tsox_core::tspath::ComparePathsOptions) -> bool {
        if self.validated_exclude_specs.is_empty() {
            return false;
        }
        let exclude_matcher = vfsmatch_new_spec_matcher(
            &self.validated_exclude_specs,
            &compare_paths_options.current_directory,
            vfsmatch::Usage::Exclude,
            compare_paths_options.use_case_sensitive_file_names,
        );
        let Some(exclude_matcher) = exclude_matcher else {
            return false;
        };
        if exclude_matcher.match_string(file_name) {
            return true;
        }
        if !tspath::has_extension(file_name)
            && exclude_matcher.match_string(&tspath::ensure_trailing_directory_separator(file_name))
        {
            return true;
        }
        false
    }

    pub fn get_matched_include_spec(&self, file_name: &str, compare_paths_options: &tsox_core::tspath::ComparePathsOptions) -> String {
        if self.validated_include_specs.is_empty() {
            return String::new();
        }
        for (index, spec) in self.validated_include_specs.iter().enumerate() {
            let include_matcher = vfsmatch_new_spec_matcher(
                std::slice::from_ref(spec),
                &compare_paths_options.current_directory,
                vfsmatch::Usage::Files,
                compare_paths_options.use_case_sensitive_file_names,
            );
            if let Some(include_matcher) = include_matcher {
                if include_matcher.match_string(file_name) {
                    return self.validated_include_specs_before_substitution[index].clone();
                }
            }
        }
        String::new()
    }

    pub fn get_matched_file_spec(&self, file_name: &str, compare_paths_options: &tsox_core::tspath::ComparePathsOptions) -> String {
        if self.validated_files_spec.is_empty() {
            return String::new();
        }
        let file_path = tspath::to_path(
            file_name,
            &compare_paths_options.current_directory,
            compare_paths_options.use_case_sensitive_file_names,
        );
        for (index, spec) in self.validated_files_spec.iter().enumerate() {
            if tspath::to_path(
                spec,
                &compare_paths_options.current_directory,
                compare_paths_options.use_case_sensitive_file_names,
            ) == file_path
            {
                return self.validated_files_spec_before_substitution[index].clone();
            }
        }
        String::new()
    }
}

pub mod vfsmatch {
    pub enum Usage {
        Files,
        Exclude,
    }
}

pub fn vfsmatch_new_spec_matcher(
    specs: &[String],
    current_directory: &str,
    usage: vfsmatch::Usage,
    use_case_sensitive_file_names: bool,
) -> Option<SpecMatcher> {
    let _ = (specs, current_directory, usage, use_case_sensitive_file_names);
    None
}

pub struct SpecMatcher;

impl SpecMatcher {
    pub fn match_string(&self, _path: &str) -> bool {
        false
    }
}

pub fn is_compiler_options_value(option: Option<&OptionDecl>, value: &JsonValue) -> bool {
    let Some(option) = option else {
        return false;
    };
    if value.is_null() {
        return !option_disallow_null_or_undefined(option);
    }
    match option.kind {
        OptionKind::List => value.is_array(),
        OptionKind::ListOrElement => {
            value.is_array() || is_compiler_options_value(option.elements(), value)
        }
        OptionKind::String => value.is_string(),
        OptionKind::Boolean => value.is_boolean(),
        OptionKind::Number => value.is_number(),
        OptionKind::Enum => value.is_string(),
    }
}

fn option_disallow_null_or_undefined(_option: &OptionDecl) -> bool {
    false
}

pub fn validate_json_option_value(
    opt: &OptionDecl,
    val: &JsonValue,
    value_expression: Option<&Node>,
    source_file: Option<&SourceFile>,
) -> (Option<JsonValue>, Vec<Diagnostic>) {
    if val.is_null() {
        return (None, Vec::new());
    }
    let mut errors: Vec<Diagnostic> = Vec::new();
    match opt.extra_validation {
        ExtraValidation::Locale => {
            if tsox_core::locale::Locale::parse(val.as_str().unwrap_or("")).is_none() {
                errors.push(create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                    source_file,
                    value_expression,
                    tsox_core::diagnostics::LOCALE_MUST_BE_AN_IETF_BCP_47_LANGUAGE_TAG_EXAMPLES_COLON_0_1,
                    vec!["en".to_string(), "ja-jp".to_string()],
                ));
            }
        }
        _ => {}
    }
    if !errors.is_empty() {
        return (None, errors);
    }
    (Some(val.clone()), errors)
}

pub fn starts_with_config_dir_template(value: &JsonValue) -> bool {
    let Some(str) = value.as_str() else {
        return false;
    };
    str.to_lowercase().starts_with(&CONFIG_DIR_TEMPLATE.to_lowercase())
}

pub fn normalize_non_list_option_value(option: &OptionDecl, base_path: &str, value: JsonValue) -> JsonValue {
    if option.is_file_path {
        let mut value_str = value.as_str().unwrap_or("").to_string();
        value_str = tspath::normalize_slashes(&value_str);
        if !starts_with_config_dir_template(&JsonValue::String(value_str.clone())) {
            value_str = tspath::get_normalized_absolute_path(&value_str, base_path);
        }
        if value_str.is_empty() {
            return JsonValue::String(".".to_string());
        }
        return JsonValue::String(value_str);
    }
    value
}

pub fn is_double_quoted_string(node: &Node) -> bool {
    ast_is_string_literal(node)
}

fn ast_is_string_literal(_node: &Node) -> bool {
    false
}

pub fn is_string_value(value: &JsonValue) -> bool {
    value.is_string()
}

pub fn normalize_json_value(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            let mut entries: Vec<(String, JsonValue)> = map
                .iter()
                .map(|(k, v)| (k.clone(), normalize_json_value(v)))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries_to_object(entries)
        }
        JsonValue::Array(items) => JsonValue::Array(items.iter().map(normalize_json_value).collect()),
        _ => value.clone(),
    }
}

fn entries_to_object(entries: Vec<(String, JsonValue)>) -> JsonValue {
    let map = entries.into_iter().collect::<serde_json::Map<String, JsonValue>>();
    JsonValue::Object(map)
}

pub fn should_report_no_input_files(
    file_names: &[String],
    can_json_report_no_input: bool,
    resolution_stack: &[String],
) -> bool {
    file_names.is_empty() && can_json_report_no_input && resolution_stack.is_empty()
}

pub fn validate_specs(
    specs: &JsonValue,
    disallow_trailing_recursion: bool,
    json_source_file: Option<&SourceFile>,
    spec_key: &str,
) -> (Vec<String>, Vec<Diagnostic>) {
    let mut errors: Vec<Diagnostic> = Vec::new();
    let mut final_specs: Vec<String> = Vec::new();
    let Some(items) = specs.as_array() else {
        return (final_specs, errors);
    };
    for value in items {
        let Some(spec) = value.as_str() else {
            continue;
        };
        if let Some(diag_message) = spec_to_diagnostic(spec, disallow_trailing_recursion) {
            let element = get_ts_config_prop_array_element_value(json_source_file, spec_key, spec);
            errors.push(create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                json_source_file,
                element,
                diag_message,
                vec![spec.to_string()],
            ));
        } else {
            final_specs.push(spec.to_string());
        }
    }
    (final_specs, errors)
}

pub fn spec_to_diagnostic(spec: &str, disallow_trailing_recursion: bool) -> Option<Message> {
    if disallow_trailing_recursion && invalid_trailing_recursion(spec) {
        return Some(tsox_core::diagnostics::FILE_SPECIFICATION_CANNOT_END_IN_A_RECURSIVE_DIRECTORY_WILDCARD_ASTERISK_ASTERISK_COLON_0);
    }
    if invalid_dot_dot_after_recursive_wildcard(spec) {
        return Some(tsox_core::diagnostics::FILE_SPECIFICATION_CANNOT_CONTAIN_A_PARENT_DIRECTORY_THAT_APPEARS_AFTER_A_RECURSIVE_DIRECTORY_WILDCARD_ASTERISK_ASTERISK_COLON_0);
    }
    None
}

pub fn invalid_trailing_recursion(spec: &str) -> bool {
    let s = spec.strip_suffix('/').unwrap_or(spec);
    s == "**" || s.ends_with("/**")
}

pub fn invalid_dot_dot_after_recursive_wildcard(s: &str) -> bool {
    let wildcard_index = if let Some(rest) = s.strip_prefix("**/") {
        let _ = rest;
        Some(0usize)
    } else {
        s.find("/**/").map(|i| i + 1)
    };
    let Some(wildcard_index) = wildcard_index else {
        return false;
    };
    let last_dot_index = if s.ends_with("/..") {
        Some(s.len())
    } else {
        s.rfind("/../").map(|i| i + 1)
    };
    match last_dot_index {
        Some(last_dot_index) => last_dot_index > wildcard_index,
        None => false,
    }
}

pub fn get_ts_config_prop_array_element_value<'a>(
    ts_config_source_file: Option<&'a SourceFile>,
    prop_key: &str,
    element_value: &str,
) -> Option<&'a Node> {
    let _ = (ts_config_source_file, prop_key, element_value);
    None
}

pub fn create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
    source_file: Option<&SourceFile>,
    node: Option<&Node>,
    message: Message,
    args: Vec<String>,
) -> Diagnostic {
    let _ = (source_file, node);
    new_compiler_diagnostic(message, args)
}

pub fn set_content_mapper_diagnostic_location(
    mut diagnostic: Diagnostic,
    source_file: Option<&Arc<SourceFile>>,
    node: Option<&Node>,
) -> Diagnostic {
    if let (Some(source_file), Some(node)) = (source_file, node) {
        diagnostic.file = Some(source_file.clone());
        diagnostic.loc = TextRange::new(
            scanner_skip_trivia(&source_text(source_file), node.pos()),
            node.end(),
        );
    }
    diagnostic
}

fn source_text(_source_file: &SourceFile) -> String {
    String::new()
}

fn scanner_skip_trivia(_text: &str, pos: usize) -> usize {
    pos
}

pub fn get_ts_config_object_literal_expression(ts_config_source_file: Option<&SourceFile>) -> Option<&Node> {
    let ts_config_source_file = ts_config_source_file?;
    let first = ts_config_source_file_statement_expression(ts_config_source_file)?;
    if ast_is_object_literal_expression(first) {
        Some(first)
    } else {
        None
    }
}

fn ts_config_source_file_statement_expression<'a>(_source_file: &'a SourceFile) -> Option<&'a Node> {
    None
}

fn ast_is_object_literal_expression(_node: &Node) -> bool {
    false
}

pub fn get_substituted_path_with_config_dir_template(value: &str, base_path: &str) -> String {
    let substituted = value.replacen(CONFIG_DIR_TEMPLATE, "./", 1);
    tspath::get_normalized_absolute_path(&substituted, base_path)
}

pub fn get_substituted_string_array_with_config_dir_template(list: &[String], base_path: &str) -> Option<Vec<String>> {
    let mut result: Option<Vec<String>> = None;
    for (i, element) in list.iter().enumerate() {
        if starts_with_config_dir_template(&JsonValue::String(element.clone())) {
            if result.is_none() {
                result = Some(list.to_vec());
            }
            if let Some(result) = result.as_mut() {
                result[i] = get_substituted_path_with_config_dir_template(element, base_path);
            }
        }
    }
    result
}

pub fn handle_option_config_dir_template_substitution(compiler_options: &mut CompilerOptions, base_path: &str) {
    if let Some(paths) = compiler_options.paths.as_mut() {
        for (_, v) in paths.iter_mut() {
            if let Some(substitution) = get_substituted_string_array_with_config_dir_template(v, base_path) {
                *v = substitution;
            }
        }
    }
    let root_dirs = compiler_options.root_dirs.clone();
    if let Some(substituted) = get_substituted_string_array_with_config_dir_template(&root_dirs, base_path) {
        compiler_options.root_dirs = substituted;
    }
    let type_roots = compiler_options.type_roots.clone();
    if let Some(substituted) = get_substituted_string_array_with_config_dir_template(&type_roots, base_path) {
        compiler_options.type_roots = substituted;
    }
    if starts_with_config_dir_template(&JsonValue::String(compiler_options.generate_cpu_profile.clone())) {
        compiler_options.generate_cpu_profile =
            get_substituted_path_with_config_dir_template(&compiler_options.generate_cpu_profile, base_path);
    }
    if starts_with_config_dir_template(&JsonValue::String(compiler_options.generate_trace.clone())) {
        compiler_options.generate_trace =
            get_substituted_path_with_config_dir_template(&compiler_options.generate_trace, base_path);
    }
    if starts_with_config_dir_template(&JsonValue::String(compiler_options.out_file.clone())) {
        compiler_options.out_file =
            get_substituted_path_with_config_dir_template(&compiler_options.out_file, base_path);
    }
    if starts_with_config_dir_template(&JsonValue::String(compiler_options.out_dir.clone())) {
        compiler_options.out_dir =
            get_substituted_path_with_config_dir_template(&compiler_options.out_dir, base_path);
    }
    if starts_with_config_dir_template(&JsonValue::String(compiler_options.root_dir.clone())) {
        compiler_options.root_dir =
            get_substituted_path_with_config_dir_template(&compiler_options.root_dir, base_path);
    }
    if starts_with_config_dir_template(&JsonValue::String(compiler_options.ts_build_info_file.clone())) {
        compiler_options.ts_build_info_file =
            get_substituted_path_with_config_dir_template(&compiler_options.ts_build_info_file, base_path);
    }
    if starts_with_config_dir_template(&JsonValue::String(compiler_options.base_url.clone())) {
        compiler_options.base_url =
            get_substituted_path_with_config_dir_template(&compiler_options.base_url, base_path);
    }
    if starts_with_config_dir_template(&JsonValue::String(compiler_options.declaration_dir.clone())) {
        compiler_options.declaration_dir =
            get_substituted_path_with_config_dir_template(&compiler_options.declaration_dir, base_path);
    }
}

pub fn has_file_with_higher_priority_extension(
    file: &str,
    extensions: &[&[&str]],
    has_file: &dyn Fn(&str) -> bool,
) -> bool {
    let mut extension_group: Vec<&str> = Vec::new();
    for group in extensions {
        if tspath::file_extension_is_one_of(file, group) {
            extension_group.extend_from_slice(group);
        }
    }
    if extension_group.is_empty() {
        return false;
    }
    for ext in extension_group {
        if tspath::file_extension_is(file, ext)
            && !(ext == tspath::EXTENSION_TS && tspath::file_extension_is(file, tspath::EXTENSION_DTS))
        {
            return false;
        }
        if has_file(&tspath::change_extension(file, ext)) {
            if ext == tspath::EXTENSION_DTS
                && (tspath::file_extension_is(file, tspath::EXTENSION_JS)
                    || tspath::file_extension_is(file, tspath::EXTENSION_JSX))
            {
                continue;
            }
            return true;
        }
    }
    false
}

pub fn remove_wildcard_files_with_lower_priority_extension(
    file: &str,
    wildcard_files: &mut Vec<(String, String)>,
    extensions: &[&[&str]],
    key_mapper: &dyn Fn(&str) -> String,
) {
    let mut extension_group: Vec<&str> = Vec::new();
    for group in extensions {
        if tspath::file_extension_is_one_of(file, group) {
            extension_group.extend_from_slice(group);
        }
    }
    if extension_group.is_empty() {
        return;
    }
    for ext in extension_group.iter().rev() {
        if tspath::file_extension_is(file, ext) {
            return;
        }
        let lower_priority_path = key_mapper(&tspath::change_extension(file, ext));
        wildcard_files.retain(|(k, _)| *k != lower_priority_path);
    }
}
