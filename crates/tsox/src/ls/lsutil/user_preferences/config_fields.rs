use super::parse_value::parse_case_sensitivity;
use super::preferences::UserPreferences;
use super::raw_fields::apply_raw_field;

use serde_json::{Map, Value};

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

pub(super) fn apply_config_fields(prefs: &mut UserPreferences, config: &Map<String, Value>) {
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
