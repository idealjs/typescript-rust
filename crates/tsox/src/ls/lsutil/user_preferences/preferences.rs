use super::config_fields::apply_config_fields;
use super::enums::*;
use super::raw_fields::apply_raw_fields;

use crate::core::tristate::Tristate;
use crate::modulespecifiers;

use super::super::format_code_options::{FormatCodeSettings, get_default_format_code_settings};
use serde_json::{Map, Value};

pub fn new_default_user_preferences() -> UserPreferences {
    UserPreferences {
        format_code_settings: get_default_format_code_settings(),

        quote_preference: QuotePreference::Unknown,
        lazy_configured_projects_from_external_project: Tristate::Unknown,
        maximum_hover_length: 0,

        include_completions_for_module_exports: Tristate::True,
        include_completions_for_import_statements: Tristate::True,
        include_automatic_optional_chain_completions: Tristate::Unknown,
        include_completions_with_class_member_snippets: Tristate::Unknown,
        include_completions_with_object_literal_method_snippets: Tristate::Unknown,
        jsx_attribute_completion_style: JsxAttributeCompletionStyle::Unknown,
        enable_auto_closing_tags: Tristate::True,
        enable_jsdoc_completions: Tristate::True,
        generate_return_in_doc_template: Tristate::True,

        import_module_specifier_preference: String::new(),
        import_module_specifier_ending: String::new(),
        auto_import_specifier_exclude_regexes: Vec::new(),
        auto_import_file_exclude_patterns: Vec::new(),
        auto_import_entrypoint_directory_search: Tristate::Unknown,
        prefer_type_only_auto_imports: Tristate::Unknown,

        organize_imports_sort: OrganizeImportsSort::Auto,
        organize_imports_ignore_case: Tristate::Unknown,
        organize_imports_collation: OrganizeImportsCollation::Ordinal,
        organize_imports_locale: String::new(),
        organize_imports_numeric_collation: Tristate::Unknown,
        organize_imports_accent_collation: Tristate::Unknown,
        organize_imports_case_first: OrganizeImportsCaseFirst::False,
        organize_imports_type_order: OrganizeImportsTypeOrder::Auto,

        allow_text_changes_in_new_files: Tristate::Unknown,

        use_aliases_for_rename: Tristate::Unknown,
        allow_rename_of_import_path: Tristate::True,

        provide_refactor_not_applicable_reason: Tristate::True,

        inlay_hints: InlayHintsPreferences::default(),
        code_lens: CodeLensUserPreferences::default(),

        prefer_go_to_source_definition: false,

        exclude_library_symbols_in_nav_to: Tristate::True,

        enable_formatting: Tristate::True,
        enable_validation: Tristate::True,
        disable_suggestions: Tristate::Unknown,
        disable_line_text_in_references: Tristate::True,
        display_parts_for_jsdoc: Tristate::True,
        report_style_checks_as_warnings: Tristate::True,

        disable_automatic_type_acquisition: Tristate::Unknown,
        automatic_type_acquisition_enabled: Tristate::Unknown,

        custom_config_file_name: String::new(),
    }
}

#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub format_code_settings: FormatCodeSettings,

    pub quote_preference: QuotePreference,

    pub lazy_configured_projects_from_external_project: Tristate,

    pub maximum_hover_length: i32,

    pub include_completions_for_module_exports: Tristate,

    pub include_completions_for_import_statements: Tristate,

    pub include_automatic_optional_chain_completions: Tristate,

    pub include_completions_with_class_member_snippets: Tristate,

    pub include_completions_with_object_literal_method_snippets: Tristate,

    pub jsx_attribute_completion_style: JsxAttributeCompletionStyle,

    pub enable_auto_closing_tags: Tristate,

    pub enable_jsdoc_completions: Tristate,

    pub generate_return_in_doc_template: Tristate,

    pub import_module_specifier_preference: modulespecifiers::ImportModuleSpecifierPreference,

    pub import_module_specifier_ending: modulespecifiers::ImportModuleSpecifierEndingPreference,

    pub auto_import_specifier_exclude_regexes: Vec<String>,

    pub auto_import_file_exclude_patterns: Vec<String>,

    pub auto_import_entrypoint_directory_search: Tristate,

    pub prefer_type_only_auto_imports: Tristate,

    pub organize_imports_sort: OrganizeImportsSort,

    pub organize_imports_ignore_case: Tristate,

    pub organize_imports_collation: OrganizeImportsCollation,

    pub organize_imports_locale: String,

    pub organize_imports_numeric_collation: Tristate,

    pub organize_imports_accent_collation: Tristate,

    pub organize_imports_case_first: OrganizeImportsCaseFirst,

    pub organize_imports_type_order: OrganizeImportsTypeOrder,

    pub allow_text_changes_in_new_files: Tristate,

    pub use_aliases_for_rename: Tristate,

    pub allow_rename_of_import_path: Tristate,

    pub provide_refactor_not_applicable_reason: Tristate,

    pub inlay_hints: InlayHintsPreferences,

    pub code_lens: CodeLensUserPreferences,

    pub prefer_go_to_source_definition: bool,

    pub exclude_library_symbols_in_nav_to: Tristate,

    pub enable_formatting: Tristate,

    pub enable_validation: Tristate,

    pub disable_suggestions: Tristate,

    pub disable_line_text_in_references: Tristate,

    pub display_parts_for_jsdoc: Tristate,

    pub report_style_checks_as_warnings: Tristate,

    pub disable_automatic_type_acquisition: Tristate,

    pub automatic_type_acquisition_enabled: Tristate,

    pub custom_config_file_name: String,
}

impl UserPreferences {
    pub fn is_ata_disabled(&self) -> bool {
        if !self.automatic_type_acquisition_enabled.is_unknown() {
            return !self.automatic_type_acquisition_enabled.is_true();
        }
        self.disable_automatic_type_acquisition.is_true()
    }

    pub fn module_specifier_preferences(&self) -> modulespecifiers::UserPreferences {
        modulespecifiers::UserPreferences {
            import_module_specifier_preference: self.import_module_specifier_preference.clone(),
            import_module_specifier_ending: self.import_module_specifier_ending.clone(),
            auto_import_specifier_exclude_regexes: self
                .auto_import_specifier_exclude_regexes
                .clone(),
        }
    }

    pub fn is_module_specifier_excluded(&self, module_specifier: &str) -> bool {
        modulespecifiers::is_excluded_by_regex(
            module_specifier,
            &self.auto_import_specifier_exclude_regexes,
        )
    }

    pub fn with_config(&self, config: &Map<String, Value>) -> UserPreferences {
        let mut prefs = self.clone();

        apply_raw_fields(&mut prefs, config);

        if let Some(Value::Object(unstable)) = config.get("unstable") {
            apply_raw_fields(&mut prefs, unstable);
        }

        apply_config_fields(&mut prefs, config);

        if !prefs.custom_config_file_name.is_empty() {
            let name = prefs.custom_config_file_name.trim();
            if name.contains('/') || name.contains('\\') || name == ".." || name == "." {
                prefs.custom_config_file_name.clear();
            } else {
                prefs.custom_config_file_name = name.to_string();
            }
        }

        prefs
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InlayHintsPreferences {
    pub include_inlay_parameter_name_hints: IncludeInlayParameterNameHints,
    pub include_inlay_parameter_name_hints_when_argument_matches_name: Tristate,
    pub include_inlay_function_parameter_type_hints: Tristate,
    pub include_inlay_variable_type_hints: Tristate,
    pub include_inlay_variable_type_hints_when_type_matches_name: Tristate,
    pub include_inlay_property_declaration_type_hints: Tristate,
    pub include_inlay_function_like_return_type_hints: Tristate,
    pub include_inlay_enum_member_value_hints: Tristate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeLensUserPreferences {
    pub references_code_lens_enabled: Tristate,
    pub implementations_code_lens_enabled: Tristate,
    pub references_code_lens_show_on_all_functions: Tristate,
    pub implementations_code_lens_show_on_interface_methods: Tristate,
    pub implementations_code_lens_show_on_all_class_methods: Tristate,
}

pub fn parse_user_preferences(items: &Map<String, Value>) -> UserPreferences {
    let mut prefs = new_default_user_preferences();

    if let Some(Value::Object(editor_settings)) = items.get("editor") {
        let unstable = serde_json::json!({ "unstable": Value::Object(editor_settings.clone()) });
        if let Value::Object(map) = unstable {
            prefs = prefs.with_config(&map);
        }
    }

    for section in ["javascript", "typescript", "js/ts"] {
        if let Some(Value::Object(settings)) = items.get(section) {
            prefs = prefs.with_config(settings);
        }
    }

    prefs
}
