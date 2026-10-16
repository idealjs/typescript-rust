use crate::modulespecifiers::types::{
    ImportModuleSpecifierEndingPreference, ModuleSpecifierEnding, ModuleSpecifierGenerationHost,
    RelativePreferenceKind, SourceFileForSpecifierGeneration, UserPreferences,
    IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_JS, IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_MINIMAL,
    IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_INDEX, IMPORT_MODULE_SPECIFIER_PREFERENCE_NON_RELATIVE,
    IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE, IMPORT_MODULE_SPECIFIER_PREFERENCE_RELATIVE,
    ModulePath,
};
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options_kinds::{ModuleKind, ModuleResolutionKind};
use tsox_core::debug::assert_never;
use tsox_core::tspath::{
    ensure_trailing_directory_separator, file_extension_is_one_of, get_directory_path,
    get_normalized_absolute_path, has_js_file_extension, has_ts_file_extension,
    is_declaration_file_name, path_is_relative, to_path, ComparePathsOptions,
};
use tsox_core::tspath::is_external_module_name_relative;

type ResolutionMode = ModuleKind;

const EXTENSIONS_NOT_SUPPORTING_EXTENSIONLESS_RESOLUTION: &[&str] = &[
    tsox_core::tspath::EXTENSION_MTS,
    tsox_core::tspath::EXTENSION_DMTS,
    tsox_core::tspath::EXTENSION_MJS,
    tsox_core::tspath::EXTENSION_CTS,
    tsox_core::tspath::EXTENSION_DCTS,
    tsox_core::tspath::EXTENSION_CJS,
];

pub fn count_path_components(path: &str) -> usize { ::tsox_core::fntrace::enter("count_path_components"); 
    let initial = if path.starts_with("./") { 2 } else { 0 };
    path[initial..].matches('/').count()
}

pub fn contains_node_modules(s: &str) -> bool { ::tsox_core::fntrace::enter("contains_node_modules"); 
    s.contains("/node_modules/")
}

fn contains_ignored_path(s: &str) -> bool { ::tsox_core::fntrace::enter("contains_ignored_path"); 
    s.contains("/node_modules/.") || s.contains("/.git") || s.contains(".#")
}

pub fn should_allow_importing_ts_extension(
    compiler_options: &CompilerOptions,
    from_file_name: &str,
) -> bool { ::tsox_core::fntrace::enter("should_allow_importing_ts_extension"); 
    compiler_options.get_allow_importing_ts_extensions()
        || (!from_file_name.is_empty() && is_declaration_file_name(from_file_name))
}

pub fn uses_extensions_on_imports(file: &dyn SourceFileForSpecifierGeneration) -> bool { ::tsox_core::fntrace::enter("uses_extensions_on_imports"); 
    for import_ref in file.imports() {
        let text = import_ref.text();
        if path_is_relative(text)
            && !file_extension_is_one_of(text, EXTENSIONS_NOT_SUPPORTING_EXTENSIONLESS_RESOLUTION)
        {
            return has_ts_file_extension(text) || has_js_file_extension(text);
        }
    }
    false
}

pub fn infer_preference(
    resolution_mode: ResolutionMode,
    source_file: Option<&dyn SourceFileForSpecifierGeneration>,
    module_resolution_is_node_next: bool,
) -> ModuleSpecifierEnding { ::tsox_core::fntrace::enter("infer_preference"); 
    let mut uses_js_extensions = false;
    let mut specifiers: Vec<std::sync::Arc<tsox_frontend::ast::Node>> = Vec::new();
    if let Some(source_file) = source_file {
        if !source_file.imports().is_empty() {
            specifiers = source_file.imports().to_vec();
        }
    }

    for specifier in &specifiers {
        let path = specifier.text();
        if path_is_relative(path) {
            if module_resolution_is_node_next && resolution_mode == ResolutionMode::CommonJS {
                continue;
            }
            if file_extension_is_one_of(path, EXTENSIONS_NOT_SUPPORTING_EXTENSIONLESS_RESOLUTION)
            {
                continue;
            }
            if has_ts_file_extension(path) {
                return ModuleSpecifierEnding::TsExtension;
            }
            if has_js_file_extension(path) {
                uses_js_extensions = true;
            }
        }
    }

    if uses_js_extensions {
        return ModuleSpecifierEnding::JsExtension;
    }
    ModuleSpecifierEnding::Minimal
}

fn module_resolution_is_node_next(module_resolution: ModuleResolutionKind) -> bool { ::tsox_core::fntrace::enter("module_resolution_is_node_next"); 
    let value = module_resolution as u8;
    let node16 = ModuleResolutionKind::Node16 as u8;
    let node_next = ModuleResolutionKind::NodeNext as u8;
    node16 <= value && value <= node_next
}

pub fn get_module_specifier_ending_preference(
    pref: &ImportModuleSpecifierEndingPreference,
    resolution_mode: ResolutionMode,
    compiler_options: &CompilerOptions,
    source_file: Option<&dyn SourceFileForSpecifierGeneration>,
) -> ModuleSpecifierEnding { ::tsox_core::fntrace::enter("get_module_specifier_ending_preference"); 
    let module_resolution = compiler_options.get_module_resolution_kind();
    let module_resolution_is_node_next = module_resolution_is_node_next(module_resolution);

    if *pref == IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_JS
        || (resolution_mode == ResolutionMode::ESNext && module_resolution_is_node_next)
    {
        if !should_allow_importing_ts_extension(compiler_options, "") {
            return ModuleSpecifierEnding::JsExtension;
        }
        if infer_preference(resolution_mode, source_file, module_resolution_is_node_next)
            != ModuleSpecifierEnding::JsExtension
        {
            return ModuleSpecifierEnding::TsExtension;
        }
        return ModuleSpecifierEnding::JsExtension;
    }

    if *pref == IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_MINIMAL {
        return ModuleSpecifierEnding::Minimal;
    }

    if *pref == IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_INDEX {
        return ModuleSpecifierEnding::Index;
    }

    if !should_allow_importing_ts_extension(compiler_options, "") {
        if let Some(source_file) = source_file {
            if uses_extensions_on_imports(source_file) {
                return ModuleSpecifierEnding::JsExtension;
            }
        }
        return ModuleSpecifierEnding::Minimal;
    }

    infer_preference(resolution_mode, source_file, module_resolution_is_node_next)
}

pub fn get_preferred_ending(
    prefs: &UserPreferences,
    host: &dyn ModuleSpecifierGenerationHost,
    compiler_options: &CompilerOptions,
    importing_source_file: &dyn SourceFileForSpecifierGeneration,
    old_import_specifier: &str,
    mut resolution_mode: ResolutionMode,
) -> ModuleSpecifierEnding { ::tsox_core::fntrace::enter("get_preferred_ending"); 
    if !old_import_specifier.is_empty() {
        if has_js_file_extension(old_import_specifier) {
            return ModuleSpecifierEnding::JsExtension;
        }
        if old_import_specifier.ends_with("/index") {
            return ModuleSpecifierEnding::Index;
        }
    }
    if resolution_mode == ResolutionMode::None {
        resolution_mode = host.get_default_resolution_mode_for_file(importing_source_file);
    }
    get_module_specifier_ending_preference(
        &prefs.import_module_specifier_ending,
        resolution_mode,
        compiler_options,
        Some(importing_source_file),
    )
}

pub struct ModuleSpecifierPreferences<'a> {
    pub relative_preference: RelativePreferenceKind,
    pub get_allowed_endings_in_preferred_order:
        Box<dyn Fn(ResolutionMode) -> Vec<ModuleSpecifierEnding> + 'a>,
    pub exclude_regexes: Vec<String>,
}

pub fn get_allowed_endings_in_preferred_order(
    prefs: &UserPreferences,
    host: &dyn ModuleSpecifierGenerationHost,
    compiler_options: &CompilerOptions,
    importing_source_file: &dyn SourceFileForSpecifierGeneration,
    old_import_specifier: &str,
    syntax_implied_node_format: ResolutionMode,
) -> Vec<ModuleSpecifierEnding> { ::tsox_core::fntrace::enter("get_allowed_endings_in_preferred_order"); 
    let mut preferred_ending = get_preferred_ending(
        prefs,
        host,
        compiler_options,
        importing_source_file,
        old_import_specifier,
        ResolutionMode::None,
    );
    let resolution_mode = host.get_default_resolution_mode_for_file(importing_source_file);
    if resolution_mode != syntax_implied_node_format {
        preferred_ending = get_preferred_ending(
            prefs,
            host,
            compiler_options,
            importing_source_file,
            old_import_specifier,
            syntax_implied_node_format,
        );
    }
    let module_resolution = compiler_options.get_module_resolution_kind();
    let module_resolution_is_node_next = module_resolution_is_node_next(module_resolution);
    let allow_importing_ts_extension =
        should_allow_importing_ts_extension(compiler_options, importing_source_file.file_name());
    let mut effective_syntax_mode = syntax_implied_node_format;
    if effective_syntax_mode == ResolutionMode::None {
        effective_syntax_mode = resolution_mode;
    }
    if effective_syntax_mode == ResolutionMode::ESNext && module_resolution_is_node_next {
        if allow_importing_ts_extension {
            return vec![
                ModuleSpecifierEnding::TsExtension,
                ModuleSpecifierEnding::JsExtension,
            ];
        }
        return vec![ModuleSpecifierEnding::JsExtension];
    }
    match preferred_ending {
        ModuleSpecifierEnding::JsExtension => {
            if allow_importing_ts_extension {
                return vec![
                    ModuleSpecifierEnding::JsExtension,
                    ModuleSpecifierEnding::TsExtension,
                    ModuleSpecifierEnding::Minimal,
                    ModuleSpecifierEnding::Index,
                ];
            }
            return vec![
                ModuleSpecifierEnding::JsExtension,
                ModuleSpecifierEnding::Minimal,
                ModuleSpecifierEnding::Index,
            ];
        }
        ModuleSpecifierEnding::TsExtension => {
            return vec![
                ModuleSpecifierEnding::TsExtension,
                ModuleSpecifierEnding::Minimal,
                ModuleSpecifierEnding::JsExtension,
                ModuleSpecifierEnding::Index,
            ]
        }
        ModuleSpecifierEnding::Index => {
            if allow_importing_ts_extension {
                return vec![
                    ModuleSpecifierEnding::Index,
                    ModuleSpecifierEnding::Minimal,
                    ModuleSpecifierEnding::TsExtension,
                    ModuleSpecifierEnding::JsExtension,
                ];
            }
            return vec![
                ModuleSpecifierEnding::Index,
                ModuleSpecifierEnding::Minimal,
                ModuleSpecifierEnding::JsExtension,
            ];
        }
        ModuleSpecifierEnding::Minimal => {
            if allow_importing_ts_extension {
                return vec![
                    ModuleSpecifierEnding::Minimal,
                    ModuleSpecifierEnding::Index,
                    ModuleSpecifierEnding::TsExtension,
                    ModuleSpecifierEnding::JsExtension,
                ];
            }
            return vec![
                ModuleSpecifierEnding::Minimal,
                ModuleSpecifierEnding::Index,
                ModuleSpecifierEnding::JsExtension,
            ];
        }
        _ => assert_never(&preferred_ending, None),
    }
}

pub fn get_module_specifier_preferences<'a>(
    prefs: &UserPreferences,
    host: &'a dyn ModuleSpecifierGenerationHost,
    compiler_options: &'a CompilerOptions,
    importing_source_file: &'a dyn SourceFileForSpecifierGeneration,
    old_import_specifier: &'a str,
) -> ModuleSpecifierPreferences<'a> { ::tsox_core::fntrace::enter("get_module_specifier_preferences"); 
    let excludes = prefs.auto_import_specifier_exclude_regexes.clone();
    let mut relative_preference = RelativePreferenceKind::Shortest;
    if !old_import_specifier.is_empty() {
        if is_external_module_name_relative(old_import_specifier) {
            relative_preference = RelativePreferenceKind::Relative;
        } else {
            relative_preference = RelativePreferenceKind::NonRelative;
        }
    } else {
        match prefs.import_module_specifier_preference.as_str() {
            IMPORT_MODULE_SPECIFIER_PREFERENCE_RELATIVE => {
                relative_preference = RelativePreferenceKind::Relative
            }
            IMPORT_MODULE_SPECIFIER_PREFERENCE_NON_RELATIVE => {
                relative_preference = RelativePreferenceKind::NonRelative
            }
            IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE => {
                relative_preference = RelativePreferenceKind::ExternalNonRelative
            }
            _ => {}
        }
    }

    let prefs = prefs.clone();
    let host_get_allowed_endings =
        move |syntax_implied_node_format: ResolutionMode| -> Vec<ModuleSpecifierEnding> {
            get_allowed_endings_in_preferred_order(
                &prefs,
                host,
                compiler_options,
                importing_source_file,
                old_import_specifier,
                syntax_implied_node_format,
            )
        };

    ModuleSpecifierPreferences {
        exclude_regexes: excludes,
        relative_preference,
        get_allowed_endings_in_preferred_order: Box::new(host_get_allowed_endings),
    }
}

pub fn get_each_file_name_of_module(
    importing_file_name: &str,
    imported_file_name: &str,
    host: &dyn ModuleSpecifierGenerationHost,
    prefer_symlinks: bool,
) -> Vec<ModulePath> { ::tsox_core::fntrace::enter("get_each_file_name_of_module"); 
    let cwd = host.get_current_directory();
    let imported_path = to_path(imported_file_name, &cwd, host.use_case_sensitive_file_names());
    let mut reference_redirect = String::new();
    let output_and_reference = host.get_project_reference_from_source(&imported_path);
    if let Some(output_and_reference) = output_and_reference {
        if !output_and_reference.output_dts.is_empty() {
            reference_redirect = output_and_reference.output_dts.clone();
        }
    }

    let redirects = host.get_redirect_targets(&imported_path);
    let mut imported_file_names: Vec<String> = Vec::with_capacity(2 + redirects.len());
    if !reference_redirect.is_empty() {
        imported_file_names.push(reference_redirect.clone());
    }
    imported_file_names.push(imported_file_name.to_string());
    imported_file_names.extend(redirects.iter().cloned());
    let targets: Vec<String> = imported_file_names
        .iter()
        .map(|f| get_normalized_absolute_path(f, &cwd))
        .collect();
    let mut should_filter_ignored_paths = !targets.iter().all(|p| contains_ignored_path(p));

    let mut results: Vec<ModulePath> = Vec::with_capacity(2);
    if !prefer_symlinks {
        for p in &targets {
            if !(should_filter_ignored_paths && contains_ignored_path(p)) {
                results.push(ModulePath {
                    file_name: p.clone(),
                    is_in_node_modules: contains_node_modules(p),
                    is_redirect: reference_redirect == *p,
                });
            }
        }
    }

    let symlink_cache = host.get_symlink_cache();
    let full_imported_file_name = get_normalized_absolute_path(imported_file_name, &cwd);
    if let Some(symlink_cache) = symlink_cache {
        tsox_core::tspath::mig::m3i::for_each_ancestor_directory_stopping_at_global_cache(
            &host.get_global_typings_cache_location(),
            &get_directory_path(&full_imported_file_name),
            &mut |real_path_directory: &str| -> Option<()> {
                let symlink_set = symlink_cache
                    .directories_by_realpath()
                    .load(
                        &to_path(
                            real_path_directory,
                            &cwd,
                            host.use_case_sensitive_file_names(),
                        )
                        .ensure_trailing_directory_separator(),
                    );
                let symlink_set = match symlink_set {
                    Some(set) => set,
                    None => return None,
                };

                if tsox_core::tspath::starts_with_directory(
                    importing_file_name,
                    real_path_directory,
                    host.use_case_sensitive_file_names(),
                ) {
                    return Some(());
                }

                for target in &targets {
                    if !tsox_core::tspath::starts_with_directory(
                        target,
                        real_path_directory,
                        host.use_case_sensitive_file_names(),
                    ) {
                        continue;
                    }

                    let relative = tsox_core::tspath::mig::m3i::get_relative_path_from_directory(
                        real_path_directory,
                        target,
                        &ComparePathsOptions {
                            use_case_sensitive_file_names: host.use_case_sensitive_file_names(),
                            current_directory: cwd.clone(),
                        },
                    );
                    for symlink_directory in symlink_set.lock().unwrap().iter() {
                        let option = tsox_core::tspath::resolve_path(symlink_directory, &[relative.as_str()]);
                        results.push(ModulePath {
                            file_name: option.clone(),
                            is_in_node_modules: contains_node_modules(&option),
                            is_redirect: *target == reference_redirect,
                        });
                        should_filter_ignored_paths = true;
                    }
                }

                None
            },
        );
    }

    if prefer_symlinks {
        for p in &targets {
            if !(should_filter_ignored_paths && contains_ignored_path(p)) {
                results.push(ModulePath {
                    file_name: p.clone(),
                    is_in_node_modules: contains_node_modules(p),
                    is_redirect: reference_redirect == *p,
                });
            }
        }
    }

    results
}
