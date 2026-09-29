#![allow(unused_imports)]

use crate::tsoptions::*;
use tsox_core::json::Value;
use tsox_core::core::tristate::Tristate;
use tsox_frontend::ast::diagnostic::Diagnostic;

pub struct TSConfig {
    pub compiler_options: CompilerOptions,
    pub files: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub references: Vec<tsox_core::core::project_reference::ProjectReference>,
    pub type_acquisition: Option<tsox_core::core::mig::m3k::TypeAcquisition>,
}

pub fn compute_fn<T, F>(f: F) -> impl Fn(&CompilerOptions) -> Value
where
    F: Fn(&CompilerOptions) -> T,
    T: serde::Serialize,
{
    move |options| serde_json::to_value(f(options)).unwrap_or(Value::Null)
}

pub fn filter_same_as_default_include(specs: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for spec in specs {
        let without_prefix = spec
            .strip_prefix("./")
            .unwrap_or(spec)
            .trim_end_matches('/')
            .to_string();
        if without_prefix != "**/*" {
            result.push(spec.clone());
        }
    }
    result
}

pub fn get_name_of_compiler_option_value(value: &Value, enum_map: &[&str]) -> String {
    let _ = enum_map;
    value.as_str().unwrap_or("").to_string()
}

pub fn serialize_enum_value(value: &Value, enum_map: &[&str]) -> String {
    if let Some(name) = value.as_str() {
        return name.to_string();
    }
    if let Some(num) = value.as_i64() {
        if let Some(name) = enum_map.get(num as usize) {
            return name.to_string();
        }
    }
    String::new()
}

pub fn any_dependency_provided(dependencies: &[&str], provided: &HashMap<String, bool>) -> bool {
    dependencies.iter().any(|d| provided.get(*d).copied().unwrap_or(false))
}

pub fn serialize_implied_option_value(
    option_decl: &OptionDecl,
    value: Value,
) -> Value {
    match option_decl.kind {
        OptionKind::Enum => match value.as_str() {
            Some(name) => Value::String(name.to_string()),
            None => value,
        },
        _ => value,
    }
}

pub fn add_implied_options(
    options: &CompilerOptions,
    option_decls_by_json_name: &HashMap<String, OptionDecl>,
    value_table: &HashMap<String, Value>,
    serialized_options: &mut serde_json::Map<String, Value>,
    config_file_path: &str,
    compare_paths_options: (&str, bool),
    preserve_default_value_map: &HashMap<String, bool>,
) {
    let _ = compare_paths_options;
    for (name, value) in value_table {
        let decl = match option_decls_by_json_name.get(name) {
            Some(decl) => decl,
            None => continue,
        };
        let already_present = serialized_options.contains_key(name);
        let is_default = preserve_default_value_map
            .get(name)
            .copied()
            .unwrap_or(false);
        if already_present && !is_default {
            continue;
        }
        if matches!(decl.kind, OptionKind::List | OptionKind::Boolean) && !already_present {
            continue;
        }
        let serialized = serialize_implied_option_value(decl, value.clone());
        serialized_options.insert(name.clone(), serialized);
    }
    let _ = config_file_path;
}

pub fn serialize_compiler_options(
    options: &CompilerOptions,
    option_decls_by_json_name: &HashMap<String, OptionDecl>,
    value_table: &HashMap<String, Value>,
    config_file_path: &str,
    compare_paths_options: (&str, bool),
) -> serde_json::Map<String, Value> {
    let _ = compare_paths_options;
    let mut result = serde_json::Map::new();
    for (name, value) in value_table {
        let decl = option_decls_by_json_name.get(name);
        let serialized = match decl {
            Some(decl) => serialize_implied_option_value(decl, value.clone()),
            None => value.clone(),
        };
        result.insert(name.clone(), serialized);
    }
    let _ = options;
    let _ = config_file_path;
    result
}

pub struct CommandLineOptionNameMap {
    map: HashMap<String, &'static OptionDecl>,
}

impl CommandLineOptionNameMap {
    pub fn from_decls(decls: &'static [OptionDecl]) -> Self {
        let mut map: HashMap<String, &'static OptionDecl> =
            HashMap::with_capacity(decls.len() * 2);
        for decl in decls {
            map.insert(decl.name.to_string(), decl);
            map.insert(decl.name.to_lowercase(), decl);
        }
        CommandLineOptionNameMap { map }
    }

    pub fn get(&self, name: &str) -> Option<&'static OptionDecl> {
        self.map.get(name).copied()
    }

    pub fn values(&self) -> impl Iterator<Item = &'static OptionDecl> + '_ {
        self.map.values().copied()
    }
}

pub fn command_line_options_to_map(options: &'static [OptionDecl]) -> CommandLineOptionNameMap {
    CommandLineOptionNameMap::from_decls(options)
}

pub fn can_json_report_no_input_files(raw_config: &serde_json::Map<String, Value>) -> bool {
    ["files", "include", "exclude"]
        .iter()
        .any(|key| raw_config.contains_key(*key))
}

pub fn directory_of_combined_path(file_name: &str, base_path: &str) -> String {
    let combined = tsox_core::tspath::combine_paths(file_name, &[base_path]);
    tsox_core::tspath::get_directory_path(&combined)
}

pub fn get_default_compiler_options(config_file_name: &str) -> Option<CompilerOptions> {
    if tsox_core::tspath::file_extension_is(config_file_name, tsox_core::tspath::EXTENSION_JSON) {
        let mut options = CompilerOptions::default();
        options.config_file_path = config_file_name.to_string();
        return Some(options);
    }
    None
}

pub fn get_default_type_acquisition(config_file_name: &str) -> Option<tsox_core::core::mig::m3k::TypeAcquisition> {
    if tsox_core::tspath::file_extension_is(config_file_name, tsox_core::tspath::EXTENSION_JSON) {
        let mut type_acquisition = tsox_core::core::mig::m3k::TypeAcquisition::default();
        type_acquisition.enable = Tristate::False;
        return Some(type_acquisition);
    }
    None
}

pub fn get_supported_extensions(
    compiler_options: &CompilerOptions,
    extra_extensions: &[&str],
) -> Vec<Vec<String>> {
    let mut extensions: Vec<Vec<String>> = vec![
        [".ts", ".tsx", ".d.ts"].iter().map(|s| s.to_string()).collect(),
        [".js", ".jsx"].iter().map(|s| s.to_string()).collect(),
    ];
    if !compiler_options.allow_non_ts_extensions.is_true() {
        extensions.push(vec![".cts".to_string(), ".mts".to_string()]);
    }
    extensions.insert(
        0,
        extra_extensions.iter().map(|s| s.to_string()).collect(),
    );
    extensions
}

pub fn get_supported_extensions_with_json_if_resolve_json_module(
    compiler_options: &CompilerOptions,
    supported_extensions: &[Vec<String>],
) -> Vec<Vec<String>> {
    let json_extension = vec![".json".to_string()];
    if compiler_options.resolve_json_module.is_true()
        && !supported_extensions.iter().any(|exts| exts == &json_extension)
    {
        let mut result = supported_extensions.to_vec();
        result.push(json_extension);
        return result;
    }
    supported_extensions.to_vec()
}
