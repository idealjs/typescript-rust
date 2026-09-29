#![allow(dead_code, unused_imports, unused_variables)]

use serde_json::{Map, Value};

use crate::ls::lsutil::new_default_user_preferences;
use crate::ls::lsutil::UserPreferences;

pub struct ConfigPathInfo {
    pub path: String,
    pub invert: bool,
}

pub struct FieldInfo {
    pub raw_name: &'static str,
    pub config_path: &'static str,
    pub fallback_config_paths: &'static [(bool, &'static str)],
    pub field_path: &'static str,
    pub raw_invert: bool,
    pub config_invert: bool,
}

static FIELD_INFOS: &[FieldInfo] = &[
    FieldInfo { raw_name: "quotePreference", config_path: "preferences.quoteStyle", fallback_config_paths: &[], field_path: "quote_preference", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "includeCompletionsForModuleExports", config_path: "suggest.autoImports", fallback_config_paths: &[], field_path: "include_completions_for_module_exports", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "includeCompletionsForImportStatements", config_path: "suggest.includeCompletionsForImportStatements", fallback_config_paths: &[], field_path: "include_completions_for_import_statements", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "includeAutomaticOptionalChainCompletions", config_path: "suggest.includeAutomaticOptionalChainCompletions", fallback_config_paths: &[], field_path: "include_automatic_optional_chain_completions", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "includeCompletionsWithClassMemberSnippets", config_path: "suggest.classMemberSnippets.enabled", fallback_config_paths: &[], field_path: "include_completions_with_class_member_snippets", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "includeCompletionsWithObjectLiteralMethodSnippets", config_path: "suggest.objectLiteralMethodSnippets.enabled", fallback_config_paths: &[], field_path: "include_completions_with_object_literal_method_snippets", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "jsxAttributeCompletionStyle", config_path: "preferences.jsxAttributeCompletionStyle", fallback_config_paths: &[], field_path: "jsx_attribute_completion_style", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "importModuleSpecifierPreference", config_path: "preferences.importModuleSpecifier", fallback_config_paths: &[], field_path: "import_module_specifier_preference", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "importModuleSpecifierEnding", config_path: "preferences.importModuleSpecifierEnding", fallback_config_paths: &[], field_path: "import_module_specifier_ending", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "autoImportSpecifierExcludeRegexes", config_path: "preferences.autoImportSpecifierExcludeRegexes", fallback_config_paths: &[], field_path: "auto_import_specifier_exclude_regexes", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "autoImportFileExcludePatterns", config_path: "preferences.autoImportFileExcludePatterns", fallback_config_paths: &[], field_path: "auto_import_file_exclude_patterns", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "autoImportEntrypointDirectorySearch", config_path: "preferences.autoImportEntrypointDirectorySearch", fallback_config_paths: &[], field_path: "auto_import_entrypoint_directory_search", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "preferTypeOnlyAutoImports", config_path: "preferences.preferTypeOnlyAutoImports", fallback_config_paths: &[], field_path: "prefer_type_only_auto_imports", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "organizeImportsSort", config_path: "preferences.organizeImports.sort", fallback_config_paths: &[], field_path: "organize_imports_sort", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "organizeImportsCollation", config_path: "preferences.organizeImports.unicodeCollation", fallback_config_paths: &[], field_path: "organize_imports_collation", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "organizeImportsLocale", config_path: "preferences.organizeImports.locale", fallback_config_paths: &[], field_path: "organize_imports_locale", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "organizeImportsNumericCollation", config_path: "preferences.organizeImports.numericCollation", fallback_config_paths: &[], field_path: "organize_imports_numeric_collation", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "organizeImportsAccentCollation", config_path: "preferences.organizeImports.accentCollation", fallback_config_paths: &[], field_path: "organize_imports_accent_collation", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "organizeImportsCaseFirst", config_path: "preferences.organizeImports.caseFirst", fallback_config_paths: &[], field_path: "organize_imports_case_first", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "organizeImportsTypeOrder", config_path: "preferences.organizeImports.typeOrder", fallback_config_paths: &[], field_path: "organize_imports_type_order", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "providePrefixAndSuffixTextForRename", config_path: "preferences.useAliasesForRenames", fallback_config_paths: &[], field_path: "provide_prefix_and_suffix_text_for_rename", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "reportStyleChecksAsWarnings", config_path: "reportStyleChecksAsWarnings", fallback_config_paths: &[], field_path: "report_style_checks_as_warnings", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "excludeLibrarySymbolsInNavTo", config_path: "workspaceSymbols.excludeLibrarySymbols", fallback_config_paths: &[], field_path: "exclude_library_symbols_in_nav_to", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "formatEnabled", config_path: "format.enabled", fallback_config_paths: &[], field_path: "format_code_settings.format_enabled", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "validateEnabled", config_path: "validate.enabled", fallback_config_paths: &[], field_path: "validate_enabled", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "disableAutomaticTypeAcquisition", config_path: "disableAutomaticTypeAcquisition", fallback_config_paths: &[], field_path: "disable_automatic_type_acquisition", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "automaticTypeAcquisitionEnabled", config_path: "tsserver.automaticTypeAcquisition.enabled", fallback_config_paths: &[], field_path: "automatic_type_acquisition_enabled", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "customConfigFileName", config_path: "customConfigFileName", fallback_config_paths: &[], field_path: "custom_config_file_name", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "baseIndentSize", config_path: "format.baseIndentSize", fallback_config_paths: &[], field_path: "format_code_settings.base_indent_size", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "indentSize", config_path: "format.indentSize", fallback_config_paths: &[], field_path: "format_code_settings.indent_size", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "tabSize", config_path: "format.tabSize", fallback_config_paths: &[], field_path: "format_code_settings.tab_size", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "newLineCharacter", config_path: "format.newLineCharacter", fallback_config_paths: &[], field_path: "format_code_settings.new_line_character", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "convertTabsToSpaces", config_path: "format.convertTabsToSpaces", fallback_config_paths: &[], field_path: "format_code_settings.convert_tabs_to_spaces", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "indentStyle", config_path: "format.indentStyle", fallback_config_paths: &[], field_path: "format_code_settings.indent_style", raw_invert: false, config_invert: false },
    FieldInfo { raw_name: "trimTrailingWhitespace", config_path: "format.trimTrailingWhitespace", fallback_config_paths: &[], field_path: "format_code_settings.trim_trailing_whitespace", raw_invert: false, config_invert: false },
];

pub fn collect_field_infos() -> &'static [FieldInfo] {
    FIELD_INFOS
}

pub fn parse_config_path_tag(tag: &str) -> ConfigPathInfo {
    let mut parts = tag.split(',');
    let path = parts.next().unwrap_or("").to_string();
    let mut invert = false;
    for part in parts {
        if part == "invert" {
            invert = true;
        }
    }
    ConfigPathInfo { path, invert }
}

pub fn get_field_by_path<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = v;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn get_field_by_path_mut<'a>(v: &'a mut Value, path: &str) -> Option<&'a mut Value> {
    let mut current = v;
    for part in path.split('.') {
        current = current.get_mut(part)?;
    }
    Some(current)
}

pub fn set_nested_value(config: &mut Map<String, Value>, path: &str, value: Value) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = config;
    for part in &parts[..parts.len() - 1] {
        let next = current
            .entry((*part).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !next.is_object() {
            *next = Value::Object(Map::new());
        }
        current = next.as_object_mut().unwrap();
    }
    current.insert(parts[parts.len() - 1].to_string(), value);
}

pub fn set_raw_fields_from_config(
    v: &mut Value,
    infos: &[FieldInfo],
    settings: &Map<String, Value>,
) {
    for (name, value) in settings {
        let Some(info) = infos.iter().find(|info| info.raw_name == *name) else {
            continue;
        };
        let mut value = value.clone();
        if info.raw_invert {
            if let Value::Bool(b) = value {
                value = Value::Bool(!b);
            }
        }
        if let Some(field) = get_field_by_path_mut(v, info.field_path) {
            set_field_from_value(field, &value);
        }
    }
}

pub fn set_field_from_value(field: &mut Value, val: &Value) {
    if val.is_null() {
        return;
    }
    *field = val.clone();
}

pub fn serialize_field(field: &Value) -> Option<Value> {
    match field {
        Value::Null => None,
        Value::Bool(_) => Some(field.clone()),
        Value::Number(n) => {
            if n.as_i64() == Some(0) {
                None
            } else {
                Some(field.clone())
            }
        }
        Value::String(s) => {
            if s.is_empty() {
                None
            } else {
                Some(field.clone())
            }
        }
        Value::Array(items) => {
            let mut result = Vec::with_capacity(items.len());
            for item in items {
                if let Value::String(s) = item {
                    result.push(Value::String(s.clone()));
                }
            }
            Some(Value::Array(result))
        }
        Value::Object(_) => Some(field.clone()),
    }
}

impl UserPreferences {
    pub fn marshal_json_to(&self, out: &mut String) -> Result<(), serde_json::Error> {
        let mut config = Map::new();
        let v = serde_json::to_value(self)?;

        for info in collect_field_infos() {
            let Some(field) = get_field_by_path(&v, info.field_path) else {
                continue;
            };
            let Some(mut val) = serialize_field(field) else {
                continue;
            };
            if !info.config_path.is_empty() {
                if info.config_invert {
                    if let Value::Bool(b) = val {
                        val = Value::Bool(!b);
                    }
                }
                set_nested_value(&mut config, info.config_path, val);
            } else if !info.raw_name.is_empty() {
                if info.raw_invert {
                    if let Value::Bool(b) = val {
                        val = Value::Bool(!b);
                    }
                }
                set_nested_value(&mut config, &format!("unstable.{}", info.raw_name), val);
            }
        }

        out.push_str(&serde_json::to_string(&config)?);
        Ok(())
    }

    pub fn unmarshal_json_from(&mut self, dec: &Value) -> Result<(), serde_json::Error> {
        let Some(config) = dec.as_object() else {
            *self = new_default_user_preferences();
            return Ok(());
        };

        *self = new_default_user_preferences();
        let mut v = serde_json::to_value(&*self)?;
        let infos = collect_field_infos();

        set_raw_fields_from_config(&mut v, infos, config);

        if let Some(Value::Object(unstable)) = config.get("unstable") {
            set_raw_fields_from_config(&mut v, infos, unstable);
        }

        for info in infos {
            if info.config_path.is_empty() {
                continue;
            }
            let mut invert = info.config_invert;
            let mut path = info.config_path;
            let mut val = get_field_by_path(dec, path).cloned();
            if val.is_none() && !info.fallback_config_paths.is_empty() {
                for (fallback_invert, fallback_path) in info.fallback_config_paths {
                    if let Some(found) = get_field_by_path(dec, fallback_path) {
                        val = Some(found.clone());
                        path = fallback_path;
                        invert = *fallback_invert;
                        break;
                    }
                }
            }
            let Some(mut val) = val else {
                continue;
            };
            if invert {
                if let Value::Bool(b) = val {
                    val = Value::Bool(!b);
                }
            }
            if path == "preferences.organizeImports.caseSensitivity" {
                val = Value::Bool(
                    crate::ls::lsutil_user_preferences_parse_value::parse_case_sensitivity(&val)
                        .is_true(),
                );
                if let Some(field) = get_field_by_path_mut(
                    &mut v,
                    "organize_imports_ignore_case",
                ) {
                    set_field_from_value(field, &val);
                }
                continue;
            }
            if let Some(field) = get_field_by_path_mut(&mut v, info.field_path) {
                set_field_from_value(field, &val);
            }
        }

        *self = serde_json::from_value(v)?;

        if !self.custom_config_file_name.is_empty() {
            let name = self.custom_config_file_name.trim().to_string();
            if name.contains('/') || name.contains('\\') || name == ".." || name == "." {
                self.custom_config_file_name = String::new();
            } else {
                self.custom_config_file_name = name;
            }
        }

        Ok(())
    }

    pub fn parsed_auto_import_file_exclude_patterns(
        &self,
        use_case_sensitive_file_names: bool,
    ) -> Option<tsox_tsoptions::vfs::vfsmatch::SpecMatcher> {
        let specs: Vec<&str> = self
            .auto_import_file_exclude_patterns
            .iter()
            .map(|s| s.as_str())
            .collect();
        tsox_tsoptions::vfs::vfsmatch::SpecMatcher::new(
            &specs,
            "",
            tsox_tsoptions::vfs::vfsmatch::Usage::Exclude,
            use_case_sensitive_file_names,
        )
    }
}
