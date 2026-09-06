use serde_json::{Map, Value};

use crate::core::tristate::Tristate;
use crate::modulespecifiers;

use super::format_code_options::{
    FormatCodeSettings, IndentStyle, SemicolonPreference, get_default_format_code_settings,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuotePreference {
    #[default]
    Unknown,
    Auto,
    Double,
    Single,
}

impl QuotePreference {
    pub fn as_str(self) -> &'static str {
        match self {
            QuotePreference::Unknown => "",
            QuotePreference::Auto => "auto",
            QuotePreference::Double => "double",
            QuotePreference::Single => "single",
        }
    }

    pub fn parse(value: &Value) -> QuotePreference {
        if let Value::String(s) = value {
            return match s.to_ascii_lowercase().as_str() {
                "auto" => QuotePreference::Auto,
                "double" => QuotePreference::Double,
                "single" => QuotePreference::Single,
                _ => QuotePreference::Unknown,
            };
        }
        QuotePreference::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JsxAttributeCompletionStyle {
    #[default]
    Unknown,
    Auto,
    Braces,
    None,
}

impl JsxAttributeCompletionStyle {
    pub fn parse(value: &Value) -> JsxAttributeCompletionStyle {
        if let Value::String(s) = value {
            return match s.to_ascii_lowercase().as_str() {
                "braces" => JsxAttributeCompletionStyle::Braces,
                "none" => JsxAttributeCompletionStyle::None,
                _ => JsxAttributeCompletionStyle::Auto,
            };
        }
        JsxAttributeCompletionStyle::Auto
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IncludeInlayParameterNameHints {
    #[default]
    None,
    All,
    Literals,
}

impl IncludeInlayParameterNameHints {
    pub fn parse(value: &Value) -> IncludeInlayParameterNameHints {
        if let Value::String(s) = value {
            return match s.as_str() {
                "all" => IncludeInlayParameterNameHints::All,
                "literals" => IncludeInlayParameterNameHints::Literals,
                _ => IncludeInlayParameterNameHints::None,
            };
        }
        IncludeInlayParameterNameHints::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum OrganizeImportsSort {
    #[default]
    Auto = 0,
    Ordinal = 1,
    OrdinalIgnoreCase = 2,
    Natural = 3,
    NaturalIgnoreCase = 4,
}

impl OrganizeImportsSort {
    pub fn parse(value: &Value) -> OrganizeImportsSort {
        if let Value::String(s) = value {
            return match s.to_ascii_lowercase().as_str() {
                "ordinal" => OrganizeImportsSort::Ordinal,
                "ordinalignorecase" => OrganizeImportsSort::OrdinalIgnoreCase,
                "natural" => OrganizeImportsSort::Natural,
                "naturalignorecase" => OrganizeImportsSort::NaturalIgnoreCase,
                _ => OrganizeImportsSort::Auto,
            };
        }
        OrganizeImportsSort::Auto
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrganizeImportsCollation {
    #[default]
    Ordinal,
    Unicode,
}

impl OrganizeImportsCollation {
    pub fn parse(value: &Value) -> OrganizeImportsCollation {
        if let Value::String(s) = value {
            if s.to_ascii_lowercase() == "unicode" {
                return OrganizeImportsCollation::Unicode;
            }
        }
        OrganizeImportsCollation::Ordinal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum OrganizeImportsCaseFirst {
    #[default]
    False = 0,
    Lower = 1,
    Upper = 2,
}

impl OrganizeImportsCaseFirst {
    pub fn parse(value: &Value) -> OrganizeImportsCaseFirst {
        if let Value::String(s) = value {
            return match s.as_str() {
                "lower" => OrganizeImportsCaseFirst::Lower,
                "upper" => OrganizeImportsCaseFirst::Upper,
                _ => OrganizeImportsCaseFirst::False,
            };
        }
        OrganizeImportsCaseFirst::False
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum OrganizeImportsTypeOrder {
    #[default]
    Auto = 0,
    Last = 1,
    Inline = 2,
    First = 3,
}

impl OrganizeImportsTypeOrder {
    pub fn parse(value: &Value) -> OrganizeImportsTypeOrder {
        if let Value::String(s) = value {
            return match s.as_str() {
                "last" => OrganizeImportsTypeOrder::Last,
                "inline" => OrganizeImportsTypeOrder::Inline,
                "first" => OrganizeImportsTypeOrder::First,
                _ => OrganizeImportsTypeOrder::Auto,
            };
        }
        OrganizeImportsTypeOrder::Auto
    }
}

fn get_nested_value<'a>(config: &'a Map<String, Value>, path: &str) -> Option<&'a Value> {
    let mut parts = path.split('.');
    let first = parts.next()?;
    let mut current: &Value = config.get(first)?;
    for part in parts {
        current = match current {
            Value::Object(m) => m.get(part)?,
            _ => return None,
        };
    }
    Some(current)
}

#[allow(clippy::too_many_lines)]
fn apply_raw_field(prefs: &mut UserPreferences, raw_name: &str, value: &Value) {

    let invert_bool = |v: &Value| -> Value {
        if let Value::Bool(b) = v {
            Value::Bool(!b)
        } else {
            v.clone()
        }
    };
    match raw_name {
        "quotePreference" => prefs.quote_preference = QuotePreference::parse(value),
        "lazyConfiguredProjectsFromExternalProject" => {
            prefs.lazy_configured_projects_from_external_project = parse_tristate(value)
        }
        "maximumHoverLength" => prefs.maximum_hover_length = parse_i32(value),
        "includeCompletionsForModuleExports" => {
            prefs.include_completions_for_module_exports = parse_tristate(value)
        }
        "includeCompletionsForImportStatements" => {
            prefs.include_completions_for_import_statements = parse_tristate(value)
        }
        "includeAutomaticOptionalChainCompletions" => {
            prefs.include_automatic_optional_chain_completions = parse_tristate(value)
        }
        "includeCompletionsWithClassMemberSnippets" => {
            prefs.include_completions_with_class_member_snippets = parse_tristate(value)
        }
        "includeCompletionsWithObjectLiteralMethodSnippets" => {
            prefs.include_completions_with_object_literal_method_snippets = parse_tristate(value)
        }
        "jsxAttributeCompletionStyle" => {
            prefs.jsx_attribute_completion_style = JsxAttributeCompletionStyle::parse(value)
        }
        "autoClosingTags" => prefs.enable_auto_closing_tags = parse_tristate(value),
        "completeJSDocs" => prefs.enable_jsdoc_completions = parse_tristate(value),
        "generateReturnInDocTemplate" => {
            prefs.generate_return_in_doc_template = parse_tristate(value)
        }
        "importModuleSpecifierPreference" => {
            prefs.import_module_specifier_preference = parse_module_specifier_preference(value)
        }
        "importModuleSpecifierEnding" => {
            prefs.import_module_specifier_ending = parse_module_specifier_ending(value)
        }
        "autoImportSpecifierExcludeRegexes" => {
            prefs.auto_import_specifier_exclude_regexes = parse_string_array(value)
        }
        "autoImportFileExcludePatterns" => {
            prefs.auto_import_file_exclude_patterns = parse_string_array(value)
        }
        "autoImportEntrypointDirectorySearch" => {
            prefs.auto_import_entrypoint_directory_search = parse_tristate(value)
        }
        "preferTypeOnlyAutoImports" => prefs.prefer_type_only_auto_imports = parse_tristate(value),
        "organizeImportsSort" => prefs.organize_imports_sort = OrganizeImportsSort::parse(value),
        "organizeImportsIgnoreCase" => prefs.organize_imports_ignore_case = parse_tristate(value),
        "organizeImportsCollation" => {
            prefs.organize_imports_collation = OrganizeImportsCollation::parse(value)
        }
        "organizeImportsLocale" => prefs.organize_imports_locale = parse_string(value),
        "organizeImportsNumericCollation" => {
            prefs.organize_imports_numeric_collation = parse_tristate(value)
        }
        "organizeImportsAccentCollation" => {
            prefs.organize_imports_accent_collation = parse_tristate(value)
        }
        "organizeImportsCaseFirst" => {
            prefs.organize_imports_case_first = OrganizeImportsCaseFirst::parse(value)
        }
        "organizeImportsTypeOrder" => {
            prefs.organize_imports_type_order = OrganizeImportsTypeOrder::parse(value)
        }
        "allowTextChangesInNewFiles" => {
            prefs.allow_text_changes_in_new_files = parse_tristate(value)
        }
        "providePrefixAndSuffixTextForRename" => {
            prefs.use_aliases_for_rename = parse_tristate(value)
        }
        "allowRenameOfImportPath" => prefs.allow_rename_of_import_path = parse_tristate(value),
        "provideRefactorNotApplicableReason" => {
            prefs.provide_refactor_not_applicable_reason = parse_tristate(value)
        }
        "includeInlayParameterNameHints" => {
            prefs.inlay_hints.include_inlay_parameter_name_hints =
                IncludeInlayParameterNameHints::parse(value)
        }
        "includeInlayParameterNameHintsWhenArgumentMatchesName" => {
            prefs
                .inlay_hints
                .include_inlay_parameter_name_hints_when_argument_matches_name =
                parse_tristate(&invert_bool(value))
        }
        "includeInlayFunctionParameterTypeHints" => {
            prefs
                .inlay_hints
                .include_inlay_function_parameter_type_hints = parse_tristate(value)
        }
        "includeInlayVariableTypeHints" => {
            prefs.inlay_hints.include_inlay_variable_type_hints = parse_tristate(value)
        }
        "includeInlayVariableTypeHintsWhenTypeMatchesName" => {
            prefs
                .inlay_hints
                .include_inlay_variable_type_hints_when_type_matches_name =
                parse_tristate(&invert_bool(value))
        }
        "includeInlayPropertyDeclarationTypeHints" => {
            prefs
                .inlay_hints
                .include_inlay_property_declaration_type_hints = parse_tristate(value)
        }
        "includeInlayFunctionLikeReturnTypeHints" => {
            prefs
                .inlay_hints
                .include_inlay_function_like_return_type_hints = parse_tristate(value)
        }
        "includeInlayEnumMemberValueHints" => {
            prefs.inlay_hints.include_inlay_enum_member_value_hints = parse_tristate(value)
        }
        "referencesCodeLensEnabled" => {
            prefs.code_lens.references_code_lens_enabled = parse_tristate(value)
        }
        "implementationsCodeLensEnabled" => {
            prefs.code_lens.implementations_code_lens_enabled = parse_tristate(value)
        }
        "referencesCodeLensShowOnAllFunctions" => {
            prefs.code_lens.references_code_lens_show_on_all_functions = parse_tristate(value)
        }
        "implementationsCodeLensShowOnInterfaceMethods" => {
            prefs
                .code_lens
                .implementations_code_lens_show_on_interface_methods = parse_tristate(value)
        }
        "implementationsCodeLensShowOnAllClassMethods" => {
            prefs
                .code_lens
                .implementations_code_lens_show_on_all_class_methods = parse_tristate(value)
        }
        "preferGoToSourceDefinition" => prefs.prefer_go_to_source_definition = parse_bool(value),
        "excludeLibrarySymbolsInNavTo" => {
            prefs.exclude_library_symbols_in_nav_to = parse_tristate(value)
        }
        "formatEnabled" => prefs.enable_formatting = parse_tristate(value),
        "validateEnabled" => prefs.enable_validation = parse_tristate(value),
        "disableSuggestions" => prefs.disable_suggestions = parse_tristate(value),
        "disableLineTextInReferences" => {
            prefs.disable_line_text_in_references = parse_tristate(value)
        }
        "displayPartsForJSDoc" => prefs.display_parts_for_jsdoc = parse_tristate(value),
        "reportStyleChecksAsWarnings" => {
            prefs.report_style_checks_as_warnings = parse_tristate(value)
        }
        "disableAutomaticTypeAcquisition" => {
            prefs.disable_automatic_type_acquisition = parse_tristate(value)
        }
        "automaticTypeAcquisitionEnabled" => {
            prefs.automatic_type_acquisition_enabled = parse_tristate(value)
        }
        "customConfigFileName" => prefs.custom_config_file_name = parse_string(value),

        "baseIndentSize" => prefs.format_code_settings.base_indent_size = parse_i32(value),
        "indentSize" => prefs.format_code_settings.indent_size = parse_i32(value),
        "tabSize" => prefs.format_code_settings.tab_size = parse_i32(value),
        "newLineCharacter" => prefs.format_code_settings.new_line_character = parse_string(value),
        "convertTabsToSpaces" => {
            prefs.format_code_settings.convert_tabs_to_spaces = parse_tristate(value)
        }
        "indentStyle" => prefs.format_code_settings.indent_style = IndentStyle::parse(value),
        "trimTrailingWhitespace" => {
            prefs.format_code_settings.trim_trailing_whitespace = parse_tristate(value)
        }

        "insertSpaceAfterCommaDelimiter" => {
            prefs
                .format_code_settings
                .insert_space_after_comma_delimiter = parse_tristate(value)
        }
        "insertSpaceAfterSemicolonInForStatements" => {
            prefs
                .format_code_settings
                .insert_space_after_semicolon_in_for_statements = parse_tristate(value)
        }
        "insertSpaceBeforeAndAfterBinaryOperators" => {
            prefs
                .format_code_settings
                .insert_space_before_and_after_binary_operators = parse_tristate(value)
        }
        "insertSpaceAfterConstructor" => {
            prefs.format_code_settings.insert_space_after_constructor = parse_tristate(value)
        }
        "insertSpaceAfterKeywordsInControlFlowStatements" => {
            prefs
                .format_code_settings
                .insert_space_after_keywords_in_control_flow_statements = parse_tristate(value)
        }
        "insertSpaceAfterFunctionKeywordForAnonymousFunctions" => {
            prefs
                .format_code_settings
                .insert_space_after_function_keyword_for_anonymous_functions = parse_tristate(value)
        }
        "insertSpaceAfterOpeningAndBeforeClosingNonemptyParenthesis" => {
            prefs
                .format_code_settings
                .insert_space_after_opening_and_before_closing_nonempty_parenthesis =
                parse_tristate(value)
        }
        "insertSpaceAfterOpeningAndBeforeClosingNonemptyBrackets" => {
            prefs
                .format_code_settings
                .insert_space_after_opening_and_before_closing_nonempty_brackets =
                parse_tristate(value)
        }
        "insertSpaceAfterOpeningAndBeforeClosingNonemptyBraces" => {
            prefs
                .format_code_settings
                .insert_space_after_opening_and_before_closing_nonempty_braces =
                parse_tristate(value)
        }
        "insertSpaceAfterOpeningAndBeforeClosingEmptyBraces" => {
            prefs
                .format_code_settings
                .insert_space_after_opening_and_before_closing_empty_braces = parse_tristate(value)
        }
        "insertSpaceAfterOpeningAndBeforeClosingTemplateStringBraces" => {
            prefs
                .format_code_settings
                .insert_space_after_opening_and_before_closing_template_string_braces =
                parse_tristate(value)
        }
        "insertSpaceAfterOpeningAndBeforeClosingJsxExpressionBraces" => {
            prefs
                .format_code_settings
                .insert_space_after_opening_and_before_closing_jsx_expression_braces =
                parse_tristate(value)
        }
        "insertSpaceAfterTypeAssertion" => {
            prefs.format_code_settings.insert_space_after_type_assertion = parse_tristate(value)
        }
        "insertSpaceBeforeFunctionParenthesis" => {
            prefs
                .format_code_settings
                .insert_space_before_function_parenthesis = parse_tristate(value)
        }
        "placeOpenBraceOnNewLineForFunctions" => {
            prefs
                .format_code_settings
                .place_open_brace_on_new_line_for_functions = parse_tristate(value)
        }
        "placeOpenBraceOnNewLineForControlBlocks" => {
            prefs
                .format_code_settings
                .place_open_brace_on_new_line_for_control_blocks = parse_tristate(value)
        }
        "insertSpaceBeforeTypeAnnotation" => {
            prefs
                .format_code_settings
                .insert_space_before_type_annotation = parse_tristate(value)
        }
        "indentMultiLineObjectLiteralBeginningOnBlankLine" => {
            prefs
                .format_code_settings
                .indent_multi_line_object_literal_beginning_on_blank_line = parse_tristate(value)
        }
        "semicolons" => prefs.format_code_settings.semicolons = SemicolonPreference::parse(value),
        "indentSwitchCase" => prefs.format_code_settings.indent_switch_case = parse_tristate(value),
        _ => {}
    }
}

fn apply_raw_fields(prefs: &mut UserPreferences, config: &Map<String, Value>) {
    for (name, value) in config {
        apply_raw_field(prefs, name, value);
    }
}

fn apply_config_fields(prefs: &mut UserPreferences, config: &Map<String, Value>) {

    let mappings: &[(&str, &str)] = &[
        ("preferences.quoteStyle", "quotePreference"),
        ("suggest.autoImports", "includeCompletionsForModuleExports"),
        (
            "suggest.includeCompletionsForImportStatements",
            "includeCompletionsForImportStatements",
        ),
        (
            "suggest.includeAutomaticOptionalChainCompletions",
            "includeAutomaticOptionalChainCompletions",
        ),
        (
            "suggest.classMemberSnippets.enabled",
            "includeCompletionsWithClassMemberSnippets",
        ),
        (
            "suggest.objectLiteralMethodSnippets.enabled",
            "includeCompletionsWithObjectLiteralMethodSnippets",
        ),
        (
            "preferences.jsxAttributeCompletionStyle",
            "jsxAttributeCompletionStyle",
        ),
        (
            "preferences.importModuleSpecifier",
            "importModuleSpecifierPreference",
        ),
        (
            "preferences.importModuleSpecifierEnding",
            "importModuleSpecifierEnding",
        ),
        (
            "preferences.autoImportSpecifierExcludeRegexes",
            "autoImportSpecifierExcludeRegexes",
        ),
        (
            "preferences.autoImportFileExcludePatterns",
            "autoImportFileExcludePatterns",
        ),
        (
            "preferences.autoImportEntrypointDirectorySearch",
            "autoImportEntrypointDirectorySearch",
        ),
        (
            "preferences.preferTypeOnlyAutoImports",
            "preferTypeOnlyAutoImports",
        ),
        ("preferences.organizeImports.sort", "organizeImportsSort"),
        (
            "preferences.organizeImports.unicodeCollation",
            "organizeImportsCollation",
        ),
        (
            "preferences.organizeImports.locale",
            "organizeImportsLocale",
        ),
        (
            "preferences.organizeImports.numericCollation",
            "organizeImportsNumericCollation",
        ),
        (
            "preferences.organizeImports.accentCollation",
            "organizeImportsAccentCollation",
        ),
        (
            "preferences.organizeImports.caseFirst",
            "organizeImportsCaseFirst",
        ),
        (
            "preferences.organizeImports.typeOrder",
            "organizeImportsTypeOrder",
        ),
        (
            "preferences.useAliasesForRenames",
            "providePrefixAndSuffixTextForRename",
        ),
        ("reportStyleChecksAsWarnings", "reportStyleChecksAsWarnings"),
        (
            "workspaceSymbols.excludeLibrarySymbols",
            "excludeLibrarySymbolsInNavTo",
        ),
        ("format.enabled", "formatEnabled"),
        ("validate.enabled", "validateEnabled"),
        (
            "disableAutomaticTypeAcquisition",
            "disableAutomaticTypeAcquisition",
        ),
        (
            "tsserver.automaticTypeAcquisition.enabled",
            "automaticTypeAcquisitionEnabled",
        ),
        ("customConfigFileName", "customConfigFileName"),

        ("format.baseIndentSize", "baseIndentSize"),
        ("format.indentSize", "indentSize"),
        ("format.tabSize", "tabSize"),
        ("format.newLineCharacter", "newLineCharacter"),
        ("format.convertTabsToSpaces", "convertTabsToSpaces"),
        ("format.indentStyle", "indentStyle"),
        ("format.trimTrailingWhitespace", "trimTrailingWhitespace"),
        (
            "format.insertSpaceAfterCommaDelimiter",
            "insertSpaceAfterCommaDelimiter",
        ),
        (
            "format.insertSpaceAfterSemicolonInForStatements",
            "insertSpaceAfterSemicolonInForStatements",
        ),
        (
            "format.insertSpaceBeforeAndAfterBinaryOperators",
            "insertSpaceBeforeAndAfterBinaryOperators",
        ),
        (
            "format.insertSpaceAfterConstructor",
            "insertSpaceAfterConstructor",
        ),
        (
            "format.insertSpaceAfterKeywordsInControlFlowStatements",
            "insertSpaceAfterKeywordsInControlFlowStatements",
        ),
        (
            "format.insertSpaceAfterFunctionKeywordForAnonymousFunctions",
            "insertSpaceAfterFunctionKeywordForAnonymousFunctions",
        ),
        (
            "format.insertSpaceAfterOpeningAndBeforeClosingNonemptyParenthesis",
            "insertSpaceAfterOpeningAndBeforeClosingNonemptyParenthesis",
        ),
        (
            "format.insertSpaceAfterOpeningAndBeforeClosingNonemptyBrackets",
            "insertSpaceAfterOpeningAndBeforeClosingNonemptyBrackets",
        ),
        (
            "format.insertSpaceAfterOpeningAndBeforeClosingNonemptyBraces",
            "insertSpaceAfterOpeningAndBeforeClosingNonemptyBraces",
        ),
        (
            "format.insertSpaceAfterOpeningAndBeforeClosingEmptyBraces",
            "insertSpaceAfterOpeningAndBeforeClosingEmptyBraces",
        ),
        (
            "format.insertSpaceAfterOpeningAndBeforeClosingTemplateStringBraces",
            "insertSpaceAfterOpeningAndBeforeClosingTemplateStringBraces",
        ),
        (
            "format.insertSpaceAfterOpeningAndBeforeClosingJsxExpressionBraces",
            "insertSpaceAfterOpeningAndBeforeClosingJsxExpressionBraces",
        ),
        (
            "format.insertSpaceAfterTypeAssertion",
            "insertSpaceAfterTypeAssertion",
        ),
        (
            "format.insertSpaceBeforeFunctionParenthesis",
            "insertSpaceBeforeFunctionParenthesis",
        ),
        (
            "format.placeOpenBraceOnNewLineForFunctions",
            "placeOpenBraceOnNewLineForFunctions",
        ),
        (
            "format.placeOpenBraceOnNewLineForControlBlocks",
            "placeOpenBraceOnNewLineForControlBlocks",
        ),
        (
            "format.insertSpaceBeforeTypeAnnotation",
            "insertSpaceBeforeTypeAnnotation",
        ),
        (
            "format.indentMultiLineObjectLiteralBeginningOnBlankLine",
            "indentMultiLineObjectLiteralBeginningOnBlankLine",
        ),
        ("format.semicolons", "semicolons"),
        ("format.indentSwitchCase", "indentSwitchCase"),

        ("referencesCodeLens.enabled", "referencesCodeLensEnabled"),
        (
            "implementationsCodeLens.enabled",
            "implementationsCodeLensEnabled",
        ),
        (
            "referencesCodeLens.showOnAllFunctions",
            "referencesCodeLensShowOnAllFunctions",
        ),
        (
            "implementationsCodeLens.showOnInterfaceMethods",
            "implementationsCodeLensShowOnInterfaceMethods",
        ),
        (
            "implementationsCodeLens.showOnAllClassMethods",
            "implementationsCodeLensShowOnAllClassMethods",
        ),

        (
            "inlayHints.parameterNames.enabled",
            "includeInlayParameterNameHints",
        ),
        (
            "inlayHints.parameterNames.suppressWhenArgumentMatchesName",
            "includeInlayParameterNameHintsWhenArgumentMatchesName",
        ),
        (
            "inlayHints.parameterTypes.enabled",
            "includeInlayFunctionParameterTypeHints",
        ),
        (
            "inlayHints.variableTypes.enabled",
            "includeInlayVariableTypeHints",
        ),
        (
            "inlayHints.variableTypes.suppressWhenTypeMatchesName",
            "includeInlayVariableTypeHintsWhenTypeMatchesName",
        ),
        (
            "inlayHints.propertyDeclarationTypes.enabled",
            "includeInlayPropertyDeclarationTypeHints",
        ),
        (
            "inlayHints.functionLikeReturnTypes.enabled",
            "includeInlayFunctionLikeReturnTypeHints",
        ),
        (
            "inlayHints.enumMemberValues.enabled",
            "includeInlayEnumMemberValueHints",
        ),

        ("autoClosingTags.enabled", "autoClosingTags"),

        ("suggest.jsdoc.enabled", "completeJSDocs"),
        (
            "suggest.jsdoc.generateReturns",
            "generateReturnInDocTemplate",
        ),
    ];

    for (path, raw_name) in mappings {
        if let Some(value) = get_nested_value(config, path) {

            if *path == "preferences.organizeImports.caseSensitivity" {
                prefs.organize_imports_ignore_case = parse_case_sensitivity(value);
                continue;
            }
            apply_raw_field(prefs, raw_name, value);
        }
    }
}

fn parse_tristate(value: &Value) -> Tristate {
    match value {
        Value::Bool(b) => Tristate::from(*b),
        _ => Tristate::Unknown,
    }
}

fn parse_bool(value: &Value) -> bool {
    matches!(value, Value::Bool(true))
}

fn parse_i32(value: &Value) -> i32 {
    match value {
        Value::Number(n) => n.as_i64().map(|i| i as i32).unwrap_or(0),
        _ => 0,
    }
}

fn parse_string(value: &Value) -> String {
    if let Value::String(s) = value {
        s.clone()
    } else {
        String::new()
    }
}

fn parse_string_array(value: &Value) -> Vec<String> {
    if let Value::Array(arr) = value {
        arr.iter()
            .filter_map(|item| {
                if let Value::String(s) = item {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn parse_module_specifier_preference(
    value: &Value,
) -> modulespecifiers::ImportModuleSpecifierPreference {
    if let Value::String(s) = value {
        match s.to_ascii_lowercase().as_str() {
            "project-relative" => {
                modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE.to_string()
            }
            "relative" => modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_RELATIVE.to_string(),
            "non-relative" => {
                modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_NON_RELATIVE.to_string()
            }
            _ => modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_SHORTEST.to_string(),
        }
    } else {
        modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_SHORTEST.to_string()
    }
}

fn parse_module_specifier_ending(
    value: &Value,
) -> modulespecifiers::ImportModuleSpecifierEndingPreference {
    if let Value::String(s) = value {
        match s.to_ascii_lowercase().as_str() {
            "minimal" => {
                modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_MINIMAL.to_string()
            }
            "index" => {
                modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_INDEX.to_string()
            }
            "js" => modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_JS.to_string(),
            _ => modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_AUTO.to_string(),
        }
    } else {
        modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_AUTO.to_string()
    }
}

fn parse_case_sensitivity(value: &Value) -> Tristate {
    if let Value::String(s) = value {
        return match s.to_ascii_lowercase().as_str() {
            "caseinsensitive" => Tristate::True,
            "casesensitive" => Tristate::False,
            _ => Tristate::Unknown,
        };
    }
    parse_tristate(value)
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
