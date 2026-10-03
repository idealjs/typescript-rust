use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options::ResolutionMode;
use tsox_core::tspath;

use crate::module::mig::m3i::try_get_js_extension_for_file;
use crate::modulespecifiers::get_allowed_endings_in_preferred_order;
use crate::modulespecifiers::types::ModuleSpecifierEnding;
use crate::modulespecifiers::types::ModuleSpecifierGenerationHost;
use crate::modulespecifiers::types::SourceFileForSpecifierGeneration;
use crate::modulespecifiers::types::UserPreferences;

use super::m3m::get_js_extension_for_declaration_file_extension;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ending {
    #[default]
    Fixed = 0,
    ExtensionChangeable = 1,
    Changeable = 2,
}

pub struct ResolvedEntrypoint {
    pub original_file_name: String,
    pub resolved_file_name: String,
    pub module_specifier: String,
    pub ending: Ending,
    pub include_conditions: Vec<String>,
    pub exclude_conditions: Vec<String>,
}

impl ResolvedEntrypoint {
    pub fn symlink_or_realpath(&self) -> &str { ::tsox_core::fntrace::enter("symlink_or_realpath"); 
        if !self.original_file_name.is_empty() {
            return &self.original_file_name;
        }
        &self.resolved_file_name
    }
}

pub fn process_entrypoint_ending(
    entrypoint: &ResolvedEntrypoint,
    prefs: &UserPreferences,
    host: &dyn ModuleSpecifierGenerationHost,
    options: &CompilerOptions,
    importing_source_file: &dyn SourceFileForSpecifierGeneration,
    allowed_endings: &[ModuleSpecifierEnding],
) -> String { ::tsox_core::fntrace::enter("process_entrypoint_ending"); 
    let mut specifier = entrypoint.module_specifier.clone();
    if entrypoint.ending == Ending::Fixed {
        return specifier;
    }

    let allowed_endings_owned;
    let allowed_endings = if allowed_endings.is_empty() {
        allowed_endings_owned = get_allowed_endings_in_preferred_order(
            prefs,
            host,
            options,
            importing_source_file,
            "",
            host.get_default_resolution_mode_for_file(importing_source_file),
        );
        &allowed_endings_owned
    } else {
        allowed_endings
    };

    let preferred_ending = allowed_endings[0];

    let dts_extension = tspath::get_declaration_file_extension(&specifier);
    if !dts_extension.is_empty() {
        match preferred_ending {
            ModuleSpecifierEnding::TsExtension | ModuleSpecifierEnding::JsExtension => {
                let js_extension =
                    get_js_extension_for_declaration_file_extension(&dts_extension);
                return tspath::mig::m3i::change_any_extension(
                    &specifier,
                    &js_extension,
                    &[dts_extension.as_str()],
                    false,
                );
            }
            ModuleSpecifierEnding::Minimal | ModuleSpecifierEnding::Index => {
                if entrypoint.ending == Ending::Changeable {
                    if dts_extension == tspath::EXTENSION_DTS {
                        specifier = tspath::remove_extension(&specifier, &dts_extension);
                        if preferred_ending == ModuleSpecifierEnding::Minimal {
                            specifier = specifier
                                .strip_suffix("/index")
                                .unwrap_or(&specifier)
                                .to_string();
                        }
                        return specifier;
                    }
                    let js_extension =
                        get_js_extension_for_declaration_file_extension(&dts_extension);
                    return tspath::mig::m3i::change_any_extension(
                        &specifier,
                        &js_extension,
                        &[dts_extension.as_str()],
                        false,
                    );
                }
                let js_extension = get_js_extension_for_declaration_file_extension(&dts_extension);
                return tspath::mig::m3i::change_any_extension(
                    &specifier,
                    &js_extension,
                    &[dts_extension.as_str()],
                    false,
                );
            }
        }
    }

    if tspath::file_extension_is_one_of(
        &specifier,
        &[
            tspath::EXTENSION_TS,
            tspath::EXTENSION_TSX,
            tspath::EXTENSION_MTS,
            tspath::EXTENSION_CTS,
        ],
    ) {
        match preferred_ending {
            ModuleSpecifierEnding::TsExtension => return specifier,
            ModuleSpecifierEnding::JsExtension => {
                let js_extension = try_get_js_extension_for_file(&specifier, options);
                if !js_extension.is_empty() {
                    return format!("{}{}", tspath::remove_file_extension(&specifier), js_extension);
                }
                return specifier;
            }
            ModuleSpecifierEnding::Minimal | ModuleSpecifierEnding::Index => {
                if entrypoint.ending == Ending::Changeable {
                    specifier = tspath::remove_file_extension(&specifier);
                    if preferred_ending == ModuleSpecifierEnding::Minimal {
                        specifier = specifier
                            .strip_suffix("/index")
                            .unwrap_or(&specifier)
                            .to_string();
                    }
                    return specifier;
                }
                let js_extension = try_get_js_extension_for_file(&specifier, options);
                if !js_extension.is_empty() {
                    return format!("{}{}", tspath::remove_file_extension(&specifier), js_extension);
                }
                return specifier;
            }
        }
    }

    if tspath::file_extension_is_one_of(
        &specifier,
        &[
            tspath::EXTENSION_JS,
            tspath::EXTENSION_JSX,
            tspath::EXTENSION_MJS,
            tspath::EXTENSION_CJS,
        ],
    ) {
        match preferred_ending {
            ModuleSpecifierEnding::TsExtension | ModuleSpecifierEnding::JsExtension => {
                return specifier
            }
            ModuleSpecifierEnding::Minimal | ModuleSpecifierEnding::Index => {
                if entrypoint.ending == Ending::Changeable {
                    specifier = tspath::remove_file_extension(&specifier);
                    if preferred_ending == ModuleSpecifierEnding::Minimal {
                        specifier = specifier
                            .strip_suffix("/index")
                            .unwrap_or(&specifier)
                            .to_string();
                    }
                    return specifier;
                }
                return specifier;
            }
        }
    }

    specifier
}
