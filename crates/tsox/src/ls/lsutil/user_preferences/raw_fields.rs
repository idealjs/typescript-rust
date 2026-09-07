use super::enums::*;
use super::parse_value::*;
use super::preferences::UserPreferences;

use super::super::format_code_options::{IndentStyle, SemicolonPreference};
use serde_json::{Map, Value};

pub(super) fn apply_raw_fields(prefs: &mut UserPreferences, config: &Map<String, Value>) {
    for (name, value) in config {
        apply_raw_field(prefs, name, value);
    }
}

#[allow(clippy::too_many_lines)]
pub(super) fn apply_raw_field(prefs: &mut UserPreferences, raw_name: &str, value: &Value) {
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
