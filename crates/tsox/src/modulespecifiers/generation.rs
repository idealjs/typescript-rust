use super::types::*;
use crate::ast::Symbol;
use crate::core::compiler_options::{CompilerOptions, ResolutionMode};
use crate::tspath;
use std::sync::Arc;

pub fn get_each_file_name_of_module(
    _importing_file_name: &str,
    imported_file_name: &str,
    host: &dyn ModuleSpecifierGenerationHost,
    _prefer_symlinks: bool,
) -> Vec<ModulePath> {
    let cwd = host.get_current_directory();
    let normalized = tspath::get_normalized_absolute_path(imported_file_name, &cwd);
    let in_nm = super::paths::contains_node_modules(&normalized);
    vec![ModulePath {
        file_name: normalized,
        is_in_node_modules: in_nm,
        is_redirect: false,
    }]
}

pub fn should_allow_importing_ts_extension(
    compiler_options: &CompilerOptions,
    from_file_name: &str,
) -> bool {
    compiler_options.get_allow_importing_ts_extensions()
        || (!from_file_name.is_empty() && tspath::is_declaration_file_name(from_file_name))
}

pub fn get_allowed_endings_in_preferred_order(
    _prefs: &UserPreferences,
    _host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _old_import_specifier: &str,
    _syntax_implied_node_format: ResolutionMode,
) -> Vec<ModuleSpecifierEnding> {
    vec![ModuleSpecifierEnding::Minimal]
}

pub fn get_module_specifier_preferences(
    prefs: &UserPreferences,
    _host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    old_import_specifier: &str,
) -> ModuleSpecifierPreferences {
    let excludes = prefs.auto_import_specifier_exclude_regexes.clone();
    let relative_preference;
    if !old_import_specifier.is_empty() {
        if tspath::is_external_module_name_relative(old_import_specifier) {
            relative_preference = RelativePreferenceKind::Relative;
        } else {
            relative_preference = RelativePreferenceKind::NonRelative;
        }
    } else {
        relative_preference = match prefs.import_module_specifier_preference.as_str() {
            IMPORT_MODULE_SPECIFIER_PREFERENCE_RELATIVE => RelativePreferenceKind::Relative,
            IMPORT_MODULE_SPECIFIER_PREFERENCE_NON_RELATIVE => RelativePreferenceKind::NonRelative,
            IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE => {
                RelativePreferenceKind::ExternalNonRelative
            }

            _ => RelativePreferenceKind::Shortest,
        };
    }
    ModuleSpecifierPreferences {
        exclude_regexes: excludes,
        relative_preference,
    }
}

pub fn get_module_specifiers(
    _module_symbol: &Arc<Symbol>,
    _checker: &dyn CheckerShape,
    _compiler_options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _host: &dyn ModuleSpecifierGenerationHost,
    _user_preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
    _for_auto_imports: bool,
) -> Vec<String> {
    Vec::new()
}

pub fn get_module_specifiers_with_info(
    _module_symbol: &Arc<Symbol>,
    _checker: &dyn CheckerShape,
    _compiler_options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _host: &dyn ModuleSpecifierGenerationHost,
    _user_preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
    _for_auto_imports: bool,
) -> (Vec<String>, ResultKind) {
    (Vec::new(), ResultKind::None)
}

pub fn get_module_specifiers_for_file_with_info(
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _module_file_name: &str,
    _compiler_options: &CompilerOptions,
    _host: &dyn ModuleSpecifierGenerationHost,
    _user_preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
    _for_auto_imports: bool,
) -> (Vec<String>, ResultKind) {
    (Vec::new(), ResultKind::None)
}

pub fn get_module_specifier(
    _from_file_name: &str,
    _to_file_name: &str,
    _host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
    _preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
) -> Option<String> {
    None
}

pub fn update_module_specifier(
    _from_file_name: &str,
    _to_file_name: &str,
    _host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
    _preferences: &UserPreferences,
    _old_import_specifier: &str,
    _options: &ModuleSpecifierOptions,
) -> Option<String> {
    None
}

pub fn get_node_modules_package_name(
    _compiler_options: &CompilerOptions,
    _importing_source_file_file_name: &str,
    _node_modules_file_name: &str,
    _host: &dyn ModuleSpecifierGenerationHost,
    _preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
) -> String {
    String::new()
}

pub fn process_entrypoint_ending(
    _entrypoint_module_specifier: &str,
    _entrypoint_is_fixed: bool,
    _prefs: &UserPreferences,
    _host: &dyn ModuleSpecifierGenerationHost,
    _options: &CompilerOptions,
    _importing_source_file: &dyn SourceFileForSpecifierGeneration,
    _allowed_endings: &[ModuleSpecifierEnding],
) -> String {
    String::new()
}

pub fn get_js_extension_for_file(file_name: &str, _options: &CompilerOptions) -> String {
    super::paths::extension_from_path(file_name)
}
