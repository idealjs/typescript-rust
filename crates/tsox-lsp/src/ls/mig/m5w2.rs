#![allow(dead_code, unused_imports, unused_variables)]

//! string_completions.go 移植(模块名/路径补全族),缺失依赖按命名约定调用待接线。

use std::collections::HashMap;
use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_compile::compiler::Program;
use tsox_core::core::text::TextRange;
use tsox_core::core::compiler_options::ModuleResolutionKind;
use tsox_core::core::compiler_options_kinds::ResolutionMode;
use tsox_core::tspath as tsp;
use tsox_frontend::ast::{Node, SourceFile, Symbol};
use tsox_tsoptions::packagejson::{JsonValue, JsonValueType, ExportsOrImports};

use crate::ls::language_service::LanguageService;
use crate::ls::lsutil_symbol_display::{ScriptElementKind, ScriptElementKindModifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleCompletionKind {
    Directory,
    File,
    ExternalModuleName,
}

#[derive(Debug, Clone)]
pub struct ModuleCompletionNameAndKind {
    pub name: String,
    pub kind: ModuleCompletionKind,
    pub extension: String,
}

#[derive(Default)]
pub struct ModuleCompletionNameAndKindSet {
    pub names: HashMap<String, ModuleCompletionNameAndKind>,
}

impl ModuleCompletionNameAndKindSet {
    pub fn add(&mut self, entry: ModuleCompletionNameAndKind) {
        match self.names.get(&entry.name) {
            Some(existing) if existing.kind >= entry.kind => {}
            _ => {
                self.names.insert(entry.name.clone(), entry);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    FileName,
    ModuleSpecifier,
}

pub struct ExtensionOptions {
    pub extensions_to_search: Vec<String>,
    pub reference_kind: ReferenceKind,
    pub importing_source_file: Option<Arc<SourceFile>>,
    pub ending_preference: String,
    pub resolution_mode: ResolutionMode,
}

pub struct PathCompletion {
    pub name: String,
    pub kind: ScriptElementKind,
    pub extension: String,
}

pub struct PathCompletions {
    pub entries: Vec<PathCompletion>,
    pub replacement_span: Option<crate::lsp::lsproto::Range>,
}

impl LanguageService {
    pub fn m5w2_get_extension_options(
        &self,
        options: &tsox_core::core::compiler_options::CompilerOptions,
        reference_kind: ReferenceKind,
        file: Option<&Arc<SourceFile>>,
        mode: ResolutionMode,
        checker: Option<&mut Checker>,
    ) -> ExtensionOptions {
        let content_mapper_extensions = self
            .get_program()
            .command_line()
            .content_mapper_extensions();
        let extensions_to_search = get_supported_extensions_for_module_resolution(
            options,
            &content_mapper_extensions,
            checker,
        );
        ExtensionOptions {
            extensions_to_search,
            reference_kind,
            importing_source_file: file.cloned(),
            ending_preference: self.user_preferences().import_module_specifier_ending.clone(),
            resolution_mode: mode,
        }
    }

    pub fn m5w2_get_completion_entries_from_typings(
        &self,
        program: &Arc<Program>,
        script_path: &str,
        fragment_directory: &str,
        extension_options: &ExtensionOptions,
        result: &mut ModuleCompletionNameAndKindSet,
    ) {
        let options = program.options();
        let mut seen: HashMap<String, bool> = HashMap::new();

        let (type_roots, _) = options.get_effective_type_roots(&program.get_current_directory());

        for root in &type_roots {
            self.m5w2_get_completion_entries_from_typings_directories(
                root,
                options,
                fragment_directory,
                extension_options,
                program,
                &mut seen,
                result,
            );
        }

        let global_cache_location = program.get_global_typings_cache_location();
        tsp::mig::m3i::for_each_ancestor_directory_stopping_at_global_cache(
            global_cache_location,
            script_path,
            |directory| {
                let types_dir = tsp::combine_paths(directory, &["node_modules/@types"]);
                self.m5w2_get_completion_entries_from_typings_directories(
                    &types_dir,
                    options,
                    fragment_directory,
                    extension_options,
                    program,
                    &mut seen,
                    result,
                );
                None::<()>
            },
        );
    }

    fn m5w2_get_completion_entries_from_typings_directories(
        &self,
        directory: &str,
        options: &tsox_core::core::compiler_options::CompilerOptions,
        fragment_directory: &str,
        extension_options: &ExtensionOptions,
        program: &Arc<Program>,
        seen: &mut HashMap<String, bool>,
        result: &mut ModuleCompletionNameAndKindSet,
    ) {
        if !self.directory_exists(directory) {
            return;
        }

        for type_directory_name in self.get_directories(directory) {
            let package_name = tsox_tsoptions::module::unmangle_scoped_package_name(&type_directory_name);
            if !options.types.is_empty() && !options.types.contains(&package_name) {
                continue;
            }

            if fragment_directory.is_empty() {
                if !seen.get(&package_name).copied().unwrap_or(false) {
                    result.add(ModuleCompletionNameAndKind {
                        name: package_name.clone(),
                        kind: ModuleCompletionKind::ExternalModuleName,
                        extension: String::new(),
                    });
                    seen.insert(package_name, true);
                }
            } else {
                let base_directory = tsp::combine_paths(directory, &[&type_directory_name]);
                if let Some(remaining_fragment) = try_remove_directory_prefix(
                    fragment_directory,
                    &package_name,
                    program.use_case_sensitive_file_names(),
                ) {
                    self.m5w2_get_completion_entries_for_directory_fragment(
                        &remaining_fragment,
                        &base_directory,
                        extension_options,
                        program,
                        false,
                        "",
                        result,
                    );
                }
            }
        }
    }

    pub fn m5w2_enumerate_node_modules_visible_to_script(&self, script_path: &str) -> Vec<String> {
        let mut result = Vec::new();
        let program = self.get_program();
        let global_cache_location = program.get_global_typings_cache_location();

        tsp::mig::m3i::for_each_ancestor_directory_stopping_at_global_cache(
            global_cache_location,
            script_path,
            |directory| {
                let package_json_path = tsp::combine_paths(directory, &["package.json"]);
                if let Some(package_json_info) = program.get_package_json_info(&package_json_path) {
                    if package_json_info.exists() {
                        if let Some(contents) = &package_json_info.contents {
                            contents
                                .dependency_fields
                                .for_each_dependency(|name, _version, _field| {
                                    if !name.starts_with("@types/") {
                                        result.push(name.to_string());
                                    }
                                    true
                                });
                        }
                    }
                }
                None::<()>
            },
        );

        result
    }

    pub fn m5w2_get_completion_entries_for_relative_modules(
        &self,
        literal_value: &str,
        script_directory: &str,
        program: &Arc<Program>,
        script_path: &str,
        extension_options: &ExtensionOptions,
    ) -> Vec<ModuleCompletionNameAndKind> {
        let options = program.options();
        if !options.root_dirs.is_empty() {
            self.m5w2_get_completion_entries_for_directory_fragment_with_root_dirs(
                &options.root_dirs,
                literal_value,
                script_directory,
                program,
                script_path,
                extension_options,
            )
        } else {
            let mut result = ModuleCompletionNameAndKindSet::default();
            self.m5w2_get_completion_entries_for_directory_fragment(
                literal_value,
                script_directory,
                extension_options,
                program,
                true,
                script_path,
                &mut result,
            );
            result.names.into_values().collect()
        }
    }

    fn m5w2_get_completion_entries_for_directory_fragment_with_root_dirs(
        &self,
        root_dirs: &[String],
        fragment: &str,
        script_directory: &str,
        program: &Arc<Program>,
        exclude: &str,
        extension_options: &ExtensionOptions,
    ) -> Vec<ModuleCompletionNameAndKind> {
        let options = program.options();
        let base_path = if !options.project.is_empty() {
            options.project.clone()
        } else {
            program.get_current_directory().to_string()
        };
        let ignore_case = !program.use_case_sensitive_file_names();
        let base_directories =
            get_base_directories_from_root_dirs(root_dirs, &base_path, script_directory, ignore_case);

        let mut all_completions = Vec::new();
        for base_directory in &base_directories {
            let mut result = ModuleCompletionNameAndKindSet::default();
            self.m5w2_get_completion_entries_for_directory_fragment(
                fragment,
                base_directory,
                extension_options,
                program,
                true,
                exclude,
                &mut result,
            );
            all_completions.extend(result.names.into_values());
        }

        deduplicate_module_completions(all_completions)
    }

    pub fn m5w2_get_completion_entries_for_directory_fragment<'a>(
        &self,
        fragment: &str,
        script_directory: &str,
        extension_options: &ExtensionOptions,
        program: &Arc<Program>,
        module_specifier_is_relative: bool,
        exclude: &str,
        result: &'a mut ModuleCompletionNameAndKindSet,
    ) -> &'a mut ModuleCompletionNameAndKindSet {
        let mut fragment = tsp::normalize_slashes(fragment);

        if !tsp::has_trailing_directory_separator(&fragment) {
            fragment = tsp::get_directory_path(&fragment);
        }

        if fragment.is_empty() {
            fragment = ".".to_string();
        }

        let fragment = tsp::ensure_trailing_directory_separator(&fragment);

        let base_directory = tsp::resolve_path(script_directory, &[&fragment]);
        if !module_specifier_is_relative {
            let package_json_directory =
                program.get_nearest_ancestor_directory_with_package_json(&base_directory);
            if !package_json_directory.is_empty() {
                let package_json_path =
                    tsp::combine_paths(&package_json_directory, &["package.json"]);
                if let Some(package_json_info) = program.get_package_json_info(&package_json_path) {
                    if let Some(fields) = package_json_info.get_contents() {
                    if let Some(version_paths) =
                        get_types_versions_paths(&fields.path_fields.types_versions)
                    {
                        let paths = &version_paths;
                        if !paths.is_empty() {
                            let package_dir_prefix =
                                tsp::ensure_trailing_directory_separator(&package_json_directory);
                            let path_in_package = base_directory[package_dir_prefix.len()..].to_string();
                            if self.m5w2_add_completion_entries_from_paths(
                                result,
                                program,
                                &path_in_package,
                                &package_json_directory,
                                extension_options,
                                paths,
                            ) {
                                return result;
                            }
                        }
                    }
                    }
                }
            }
        }

        if !self.directory_exists(&base_directory) {
            return result;
        }

        let files = self.read_directory(
            &base_directory,
            &extension_options.extensions_to_search,
            &vec!["./*".to_string()],
        );

        for file_path in files {
            let exclude_matches = tsp::mig::m3i::compare_paths(
                exclude,
                &file_path,
                &tsp::ComparePathsOptions {
                    use_case_sensitive_file_names: program.use_case_sensitive_file_names(),
                    current_directory: program.get_current_directory().to_string(),
                },
            ) == 0;
            if exclude_matches {
                continue;
            }

            let (name, extension) = get_filename_with_extension_option(
                &tsp::get_base_file_name(&file_path),
                program,
                extension_options,
                false,
            );
            result.add(ModuleCompletionNameAndKind {
                name,
                kind: ModuleCompletionKind::File,
                extension,
            });
        }

        let directories = self.get_directories(&base_directory);

        for directory in directories {
            let directory_name = tsp::get_base_file_name(&directory);
            if directory_name != "@types" {
                result.add(ModuleCompletionNameAndKind {
                    name: directory_name,
                    kind: ModuleCompletionKind::Directory,
                    extension: String::new(),
                });
            }
        }

        result
    }

    pub fn m5w2_add_completion_entries_from_paths(
        &self,
        result: &mut ModuleCompletionNameAndKindSet,
        program: &Arc<Program>,
        fragment: &str,
        base_directory: &str,
        extension_options: &ExtensionOptions,
        paths: &tsox_core::collections::ordered_map::OrderedMap<String, Vec<String>>,
    ) -> bool {
        let get_patterns_for_keys = |key: &str| paths.get(&key.to_string()).cloned();
        let compare_paths = |a: &str, b: &str| -> std::cmp::Ordering {
            let pattern_a = tsox_core::core::core::try_parse_pattern(a);
            let pattern_b = tsox_core::core::core::try_parse_pattern(b);
            let length_a = if pattern_a.star_index != -1 { pattern_a.star_index as usize } else { a.len() };
            let length_b = if pattern_b.star_index != -1 { pattern_b.star_index as usize } else { b.len() };
            length_b.cmp(&length_a)
        };
        self.m5w2_add_completion_entries_from_paths_or_exports_or_imports(
            result,
            program,
            false,
            false,
            fragment,
            base_directory,
            extension_options,
            paths.keys().cloned().collect(),
            get_patterns_for_keys,
            compare_paths,
        )
    }

    pub fn m5w2_add_completion_entries_from_paths_or_exports_or_imports(
        &self,
        result: &mut ModuleCompletionNameAndKindSet,
        program: &Arc<Program>,
        is_exports: bool,
        is_imports: bool,
        fragment: &str,
        base_directory: &str,
        extension_options: &ExtensionOptions,
        keys: Vec<String>,
        get_patterns_for_key: impl Fn(&str) -> Option<Vec<String>>,
        compare_paths: impl Fn(&str, &str) -> std::cmp::Ordering,
    ) -> bool {
        struct PathResult {
            results: Vec<ModuleCompletionNameAndKind>,
            matched: bool,
        }
        let mut path_results: Vec<PathResult> = Vec::new();
        let mut matched_path: Option<String> = None;
        for key in &keys {
            if key == "." {
                continue;
            }
            let mut normalized_key = key.strip_prefix("./").unwrap_or(key).to_string();
            if (is_exports || is_imports) && key.ends_with('/') {
                normalized_key.push('*');
            }
            if let Some(patterns) = get_patterns_for_key(key) {
                if patterns.is_empty() {
                    continue;
                }
                let path_pattern = tsox_core::core::core::try_parse_pattern(&normalized_key);
                if !path_pattern.is_valid() {
                    continue;
                }
                let is_match = path_pattern.matches(fragment);
                let is_longest_match = is_match
                    && match &matched_path {
                        None => true,
                        Some(matched) => {
                            compare_paths(&normalized_key, matched) == std::cmp::Ordering::Less
                        }
                    };
                if is_longest_match {
                    matched_path = Some(normalized_key.clone());
                    path_results.retain(|pr| !pr.matched);
                }
                if path_pattern.star_index == -1
                    || matched_path.is_none()
                    || match &matched_path {
                        None => true,
                        Some(matched) => {
                            compare_paths(&normalized_key, matched) != std::cmp::Ordering::Greater
                        }
                    }
                {
                    let results = self.m5w2_get_completions_for_path_mapping(
                        &normalized_key,
                        &patterns,
                        fragment,
                        base_directory,
                        is_exports,
                        is_imports,
                        extension_options,
                        program,
                    );
                    path_results.push(PathResult {
                        matched: is_match,
                        results,
                    });
                }
            }
        }

        for pr in path_results {
            for res in pr.results {
                result.add(res);
            }
        }

        matched_path.is_some()
    }

    pub fn m5w2_get_completions_for_path_mapping(
        &self,
        path: &str,
        patterns: &[String],
        fragment: &str,
        package_directory: &str,
        is_exports: bool,
        is_imports: bool,
        extension_options: &ExtensionOptions,
        program: &Arc<Program>,
    ) -> Vec<ModuleCompletionNameAndKind> {
        let mut fragment_directory = get_fragment_directory(fragment);
        if !fragment_directory.is_empty() {
            fragment_directory = tsp::ensure_trailing_directory_separator(&fragment_directory);
        }
        let just_path_mapping_name = |name: &str,
                                      kind: ModuleCompletionKind,
                                      extension: String|
         -> Vec<ModuleCompletionNameAndKind> {
            if name.starts_with(fragment) {
                let mut name = tsp::remove_trailing_directory_separator(name);
                if !fragment_directory.is_empty() {
                    name = name
                        .strip_prefix(fragment_directory.as_str())
                        .unwrap_or(&name)
                        .to_string();
                }
                return vec![ModuleCompletionNameAndKind {
                    name,
                    kind,
                    extension,
                }];
            }
            Vec::new()
        };

        let parsed_path = tsox_core::core::core::try_parse_pattern(path);
        if !parsed_path.is_valid() {
            return Vec::new();
        }
        if parsed_path.star_index == -1 {
            let pattern = patterns.first().cloned().unwrap_or_default();
            let extension = get_file_extension(&pattern);
            return just_path_mapping_name(path, ModuleCompletionKind::File, extension);
        }

        let path_prefix = &parsed_path.text[..parsed_path.star_index as usize];
        let path_suffix = &parsed_path.text[parsed_path.star_index as usize + 1..];
        if !fragment.starts_with(path_prefix) {
            if !path_prefix.starts_with(fragment) {
                return Vec::new();
            }
            let star_is_full_path_component = path.ends_with("/*");
            if star_is_full_path_component {
                return just_path_mapping_name(path_prefix, ModuleCompletionKind::Directory, String::new());
            }
            let remaining_directory_prefix = &path_prefix[fragment_directory.len()..];
            let mut completions = Vec::new();
            for pattern in patterns {
                let mut modules = self.m5w2_get_modules_for_paths_pattern(
                    "",
                    package_directory,
                    pattern,
                    is_exports,
                    is_imports,
                    extension_options,
                    program,
                );
                for module in &mut modules {
                    module.name = format!(
                        "{}{}{}",
                        remaining_directory_prefix,
                        module.name,
                        if module.kind == ModuleCompletionKind::File { path_suffix } else { "" }
                    );
                }
                completions.extend(modules);
            }
            return completions;
        }
        let remaining_fragment = &fragment[path_prefix.len()..];
        let remaining_directory_fragment = if !fragment_directory.starts_with(path_prefix) {
            path_prefix[fragment_directory.len()..].to_string()
        } else {
            String::new()
        };
        let mut all = Vec::new();
        for pattern in patterns {
            let mut modules = self.m5w2_get_modules_for_paths_pattern(
                remaining_fragment,
                package_directory,
                pattern,
                is_exports,
                is_imports,
                extension_options,
                program,
            );
            for module in &mut modules {
                module.name = format!(
                    "{}{}{}",
                    remaining_directory_fragment,
                    module.name,
                    if module.kind == ModuleCompletionKind::File { path_suffix } else { "" }
                );
            }
            all.extend(modules);
        }
        all
    }

    pub fn m5w2_get_modules_for_paths_pattern(
        &self,
        fragment: &str,
        package_directory: &str,
        pattern: &str,
        is_exports: bool,
        is_imports: bool,
        extension_options: &ExtensionOptions,
        program: &Arc<Program>,
    ) -> Vec<ModuleCompletionNameAndKind> {
        let parsed = tsox_core::core::core::try_parse_pattern(pattern);
        if !parsed.is_valid() || parsed.star_index == -1 {
            return Vec::new();
        }

        let prefix = &parsed.text[..parsed.star_index as usize];
        let suffix = &parsed.text[parsed.star_index as usize + 1..];

        let normalized_prefix = tsp::resolve_path(prefix, &[""]);
        let (normalized_prefix_directory, normalized_prefix_base) =
            if tsp::has_trailing_directory_separator(prefix) {
                (normalized_prefix.clone(), String::new())
            } else {
                (
                    tsp::get_directory_path(&normalized_prefix),
                    tsp::get_base_file_name(&normalized_prefix),
                )
            };

        let fragment_has_path = contains_slash(fragment);
        let fragment_directory = if fragment_has_path {
            if tsp::has_trailing_directory_separator(fragment) {
                fragment.to_string()
            } else {
                tsp::get_directory_path(fragment)
            }
        } else {
            String::new()
        };

        let options = program.options();
        let ignore_case = !program.use_case_sensitive_file_names();
        let out_dir = options.out_dir.clone();
        let declaration_dir = options.declaration_dir.clone();

        let expanded_prefix_directory = if fragment_has_path {
            tsp::combine_paths(
                &normalized_prefix_directory,
                &[&format!("{}{}", normalized_prefix_base, fragment_directory)],
            )
        } else {
            normalized_prefix_directory.clone()
        };
        let base_directory = tsp::normalize_path(&tsp::combine_paths(
            package_directory,
            &[expanded_prefix_directory.as_str()],
        ));

        let mut possible_input_base_directory_for_out_dir = String::new();
        let mut possible_input_base_directory_for_declaration_dir = String::new();
        if is_imports {
            if !out_dir.is_empty() {
                possible_input_base_directory_for_out_dir =
                    get_possible_original_input_path_without_changing_ext(
                        &base_directory,
                        ignore_case,
                        &out_dir,
                        || {
                            tsox_compile::mig::m4v::get_common_source_directory(
                                program.options(),
                                || {
                                    program
                                        .source_files()
                                        .iter()
                                        .map(|f| f.file_name.clone())
                                        .collect()
                                },
                                &program.get_current_directory(),
                                program.use_case_sensitive_file_names(),
                            )
                        },
                    );
            }
            if !declaration_dir.is_empty() {
                possible_input_base_directory_for_declaration_dir =
                    get_possible_original_input_path_without_changing_ext(
                        &base_directory,
                        ignore_case,
                        &declaration_dir,
                        || {
                            tsox_compile::mig::m4v::get_common_source_directory(
                                program.options(),
                                || {
                                    program
                                        .source_files()
                                        .iter()
                                        .map(|f| f.file_name.clone())
                                        .collect()
                                },
                                &program.get_current_directory(),
                                program.use_case_sensitive_file_names(),
                            )
                        },
                    );
            }
        }

        let normalized_suffix = tsp::normalize_path(suffix);

        let declaration_extension = if !normalized_suffix.is_empty() {
            tsp::mig::m3i::get_declaration_emit_extension_for_path(&format!("_{}", normalized_suffix))
        } else {
            String::new()
        };
        let input_extensions: Vec<String> = if !normalized_suffix.is_empty() {
            tsp::mig::m3i::get_possible_original_input_extension_for_extension(&format!(
                "_{}",
                normalized_suffix
            ))
        } else {
            Vec::new()
        };

        let mut matching_suffixes: Vec<String> = Vec::new();
        if !declaration_extension.is_empty() {
            matching_suffixes.push(tsp::change_extension(&normalized_suffix, &declaration_extension));
        }
        for ext in &input_extensions {
            matching_suffixes.push(tsp::change_extension(&normalized_suffix, ext));
        }
        matching_suffixes.push(normalized_suffix.clone());

        let include_globs: Vec<String> = if !normalized_suffix.is_empty() {
            matching_suffixes
                .iter()
                .map(|suffix| format!("**/*{}", suffix))
                .collect()
        } else {
            vec!["./*".to_string()]
        };

        let is_exports_or_imports_wildcard =
            (is_exports || is_imports) && pattern.ends_with("/*");

        let trim_prefix_and_suffix = |path: &str, prefix_str: &str| -> String {
            for suffix in &matching_suffixes {
                if let Some(inner) = without_start_and_end(
                    &tsp::normalize_path(path),
                    prefix_str,
                    suffix,
                ) {
                    return remove_leading_directory_separator(&inner);
                }
            }
            String::new()
        };

        let get_matches_with_prefix = |directory: &str| -> Vec<ModuleCompletionNameAndKind> {
            let complete_prefix = if fragment_has_path {
                directory.to_string()
            } else {
                format!(
                    "{}{}",
                    tsp::ensure_trailing_directory_separator(directory),
                    normalized_prefix_base
                )
            };

            let matches = self.read_directory(
                directory,
                &extension_options.extensions_to_search,
                &include_globs,
            );

            let mut result = Vec::new();
            for matched in matches {
                let trimmed_with_pattern = trim_prefix_and_suffix(&matched, &complete_prefix);
                if !trimmed_with_pattern.is_empty() {
                    if contains_slash(&trimmed_with_pattern) {
                        let path_components = tsp::get_path_components(
                            &remove_leading_directory_separator(&trimmed_with_pattern),
                            "",
                        );
                        if path_components.len() > 1 {
                            result.push(ModuleCompletionNameAndKind {
                                name: path_components[1].clone(),
                                kind: ModuleCompletionKind::Directory,
                                extension: String::new(),
                            });
                        }
                    } else {
                        let (mut name, mut extension) = get_filename_with_extension_option(
                            &trimmed_with_pattern,
                            program,
                            extension_options,
                            is_exports_or_imports_wildcard,
                        );
                        if extension.is_empty() {
                            extension = get_file_extension(&matched);
                        }
                        result.push(ModuleCompletionNameAndKind {
                            name,
                            kind: ModuleCompletionKind::File,
                            extension,
                        });
                    }
                }
            }
            result
        };

        let get_directory_matches = |directory_name: &str| -> Vec<ModuleCompletionNameAndKind> {
            let directories = self.get_directories(directory_name);
            let mut result = Vec::new();
            for dir in directories {
                if dir != "node_modules" {
                    result.push(ModuleCompletionNameAndKind {
                        name: dir,
                        kind: ModuleCompletionKind::Directory,
                        extension: String::new(),
                    });
                }
            }
            result
        };

        let mut matches = Vec::new();
        matches.extend(get_matches_with_prefix(&base_directory));

        if !possible_input_base_directory_for_out_dir.is_empty() {
            matches.extend(get_matches_with_prefix(&possible_input_base_directory_for_out_dir));
        }
        if !possible_input_base_directory_for_declaration_dir.is_empty() {
            matches.extend(get_matches_with_prefix(
                &possible_input_base_directory_for_declaration_dir,
            ));
        }

        if normalized_suffix.is_empty() {
            matches.extend(get_directory_matches(&base_directory));
            if !possible_input_base_directory_for_out_dir.is_empty() {
                matches.extend(get_directory_matches(&possible_input_base_directory_for_out_dir));
            }
            if !possible_input_base_directory_for_declaration_dir.is_empty() {
                matches.extend(get_directory_matches(
                    &possible_input_base_directory_for_declaration_dir,
                ));
            }
        }

        matches
    }
}

impl LanguageService {
    pub fn m5w2_get_string_literal_completions_from_module_names(
        &self,
        file: &Arc<SourceFile>,
        node: &Arc<Node>,
        program: &Arc<Program>,
        checker: &mut Checker,
    ) -> Option<crate::ls::mig::m5w2_2::StringLiteralCompletions> {
        let text_start = tsox_frontend::astnav::get_start_of_node(node, file, false) + 1;
        let replacement_span = self.path_completion_replacement_span(
            file,
            get_directory_fragment_range(&node.text(), text_start),
        )?;
        let name_and_kinds = self.m5w2_get_string_literal_completions_from_module_names_worker(
            file,
            node,
            program,
            checker,
        );
        Some(crate::ls::mig::m5w2_2::StringLiteralCompletions {
            from_paths: Some(PathCompletions {
                entries: m5w2_to_path_completions(name_and_kinds),
                replacement_span: Some(replacement_span),
            }),
            from_types: None,
            from_properties: None,
        })
    }

    fn m5w2_get_string_literal_completions_from_module_names_worker(
        &self,
        file: &Arc<SourceFile>,
        node: &Arc<Node>,
        program: &Arc<Program>,
        checker: &mut Checker,
    ) -> Vec<ModuleCompletionNameAndKind> {
        let literal_value = tsp::normalize_slashes(&node.text());
        let mode = if tsox_frontend::ast::is_string_literal_like(node) {
            program.get_mode_for_usage_location(file, node)
        } else {
            ResolutionMode::None
        };

        let script_path = file.file_name.clone();
        let script_directory = tsp::get_directory_path(&script_path);
        let options = program.options();
        let extension_options = self.m5w2_get_extension_options(
            options,
            ReferenceKind::ModuleSpecifier,
            Some(file),
            mode,
            Some(&mut *checker),
        );

        if is_path_relative_to_script(&literal_value)
            || (options
                .paths
                .as_ref()
                .map_or(true, |p| p.is_empty())
                && (tsp::is_rooted_disk_path(&literal_value) || tsp::is_url(&literal_value)))
        {
            self.m5w2_get_completion_entries_for_relative_modules(
                &literal_value,
                &script_directory,
                program,
                &script_path,
                &extension_options,
            )
        } else {
            self.m5w2_get_completion_entries_for_non_relative_modules(
                &literal_value,
                &script_directory,
                mode,
                program,
                checker,
                &extension_options,
            )
        }
    }

    pub fn m5w2_get_completion_entries_for_non_relative_modules(
        &self,
        fragment: &str,
        script_path: &str,
        mode: ResolutionMode,
        program: &Arc<Program>,
        type_checker: &mut Checker,
        extension_options: &ExtensionOptions,
    ) -> Vec<ModuleCompletionNameAndKind> {
        let compiler_options = program.options();
        let paths = compiler_options.paths.as_ref();

        let mut result = ModuleCompletionNameAndKindSet::default();
        let module_resolution = compiler_options.get_module_resolution_kind();

        if paths.is_some_and(|p| !p.is_empty()) {
            let mut paths_ordered = tsox_core::collections::ordered_map::OrderedMap::new();
            for (key, value) in paths.unwrap() {
                paths_ordered.insert(key.clone(), value.clone());
            }
            let absolute = compiler_options.get_paths_base_path(&program.get_current_directory());
            self.m5w2_add_completion_entries_from_paths(
                &mut result,
                program,
                fragment,
                &absolute,
                extension_options,
                &paths_ordered,
            );
        }

        let fragment_directory = get_fragment_directory(fragment);
        for ambient_name in get_ambient_module_completions(fragment, &fragment_directory, type_checker)
        {
            result.add(ModuleCompletionNameAndKind {
                name: ambient_name,
                kind: ModuleCompletionKind::ExternalModuleName,
                extension: String::new(),
            });
        }

        self.m5w2_get_completion_entries_from_typings(
            program,
            script_path,
            &fragment_directory,
            extension_options,
            &mut result,
        );

        if module_resolution_uses_node_modules(module_resolution) {
            let mut found_global = false;
            if fragment_directory.is_empty() {
                for module_name in self.m5w2_enumerate_node_modules_visible_to_script(script_path) {
                    if !result.names.contains_key(&module_name) {
                        found_global = true;
                        result.add(ModuleCompletionNameAndKind {
                            name: module_name,
                            kind: ModuleCompletionKind::ExternalModuleName,
                            extension: String::new(),
                        });
                    }
                }
            }
            if !found_global {
                let resolve_package_json_exports = compiler_options.get_resolve_package_json_exports();
                let resolve_package_json_imports = compiler_options.get_resolve_package_json_imports();
                let mut seen_package_scope = false;
                let conditions = get_conditions(compiler_options, mode);

                let exports_or_imports_lookup =
                    |lookup_table: Option<&ExportsOrImports>,
                     fragment: &str,
                     base_directory: &str,
                     is_exports: bool,
                     is_imports: bool,
                     result: &mut ModuleCompletionNameAndKindSet|
                     -> bool {
                        let Some(lookup_table) = lookup_table else {
                            return false;
                        };
                        if lookup_table.json_value.value_type != JsonValueType::Object {
                            return lookup_table.json_value.value_type != JsonValueType::NotPresent;
                        }
                        let keys: Vec<String> = lookup_table
                            .json_value
                            .as_object()
                            .iter()
                            .map(|(k, _)| k.clone())
                            .collect();
                        self.m5w2_add_completion_entries_from_paths_or_exports_or_imports(
                            result,
                            program,
                            is_exports,
                            is_imports,
                            fragment,
                            base_directory,
                            extension_options,
                            keys,
                            |key| {
                                let key_value = lookup_table
                                    .json_value
                                    .as_object()
                                    .iter()
                                    .find(|(k, _)| k == key)
                                    .map(|(_, v)| v)?;
                                let pattern =
                                    get_pattern_from_first_matching_condition(key_value, &conditions);
                                if pattern.is_empty() {
                                    return None;
                                }
                                if key.ends_with('/') && pattern.ends_with('/') {
                                    return Some(vec![format!("{}*", pattern)]);
                                }
                                Some(vec![pattern])
                            },
                            tsox_tsoptions::module::compare_pattern_keys,
                        );
                        true
                    };

                let imports_lookup = |directory: &str,
                                      seen_package_scope: &mut bool,
                                      result: &mut ModuleCompletionNameAndKindSet| {
                    if resolve_package_json_imports && !*seen_package_scope {
                        let package_file = tsp::combine_paths(directory, &["package.json"]);
                        if let Some(package_json_info) = program.get_package_json_info(&package_file) {
                            if package_json_info.exists() {
                                if let Some(contents) = &package_json_info.contents {
                                    *seen_package_scope = true;
                                    exports_or_imports_lookup(
                                        Some(&contents.path_fields.imports),
                                        fragment,
                                        directory,
                                        false,
                                        true,
                                        result,
                                    );
                                }
                            }
                        }
                    }
                };

                let node_modules_lookup = |ancestor: &str,
                                          seen_package_scope: &mut bool,
                                          result: &mut ModuleCompletionNameAndKindSet| {
                    let node_modules = tsp::combine_paths(ancestor, &["node_modules"]);
                    if self.directory_exists(&node_modules) {
                        self.m5w2_get_completion_entries_for_directory_fragment(
                            fragment,
                            &node_modules,
                            extension_options,
                            program,
                            false,
                            "",
                            result,
                        );
                    }
                    imports_lookup(ancestor, seen_package_scope, result);
                };

                let use_package_lookup =
                    !fragment_directory.is_empty() && resolve_package_json_exports;
                let mut ancestor_lookup = |ancestor: &str,
                                          seen_package_scope: &mut bool,
                                          result: &mut ModuleCompletionNameAndKindSet| {
                    if !use_package_lookup {
                        node_modules_lookup(ancestor, seen_package_scope, result);
                        return;
                    }
                    let mut components: Vec<String> =
                            tsp::get_path_components(fragment, "").into_iter().skip(1).collect();
                    if components.is_empty() {
                        node_modules_lookup(ancestor, seen_package_scope, result);
                        return;
                    }
                    let mut package_path = components.remove(0);
                    if package_path.starts_with('@') {
                        if components.is_empty() {
                            node_modules_lookup(ancestor, seen_package_scope, result);
                            return;
                        }
                            let sub_name = components.remove(0);
                            package_path = tsp::combine_paths(&package_path, &[&sub_name]);
                        }
                        if resolve_package_json_imports && package_path.starts_with('#') {
                            imports_lookup(ancestor, seen_package_scope, result);
                            return;
                        }
                        let package_directory =
                            tsp::combine_paths(ancestor, &["node_modules", &package_path]);
                        let package_file = tsp::combine_paths(&package_directory, &["package.json"]);
                        if let Some(package_json_info) = program.get_package_json_info(&package_file) {
                            if package_json_info.exists() {
                                let mut fragment_subpath = components.join("/");
                                if !components.is_empty()
                                    && tsp::has_trailing_directory_separator(fragment)
                                {
                                    fragment_subpath.push('/');
                                }
                                if let Some(contents) = &package_json_info.contents {
                                    if exports_or_imports_lookup(
                                        Some(&contents.path_fields.exports),
                                        &fragment_subpath,
                                        &package_directory,
                                        true,
                                        false,
                                        result,
                                    ) {
                                        return;
                                    }
                                }
                            }
                        }
                    node_modules_lookup(ancestor, seen_package_scope, result);
                };

                let global_cache_location = program.get_global_typings_cache_location();
                let mut seen = seen_package_scope;
                tsp::mig::m3i::for_each_ancestor_directory_stopping_at_global_cache(
                    global_cache_location,
                    script_path,
                    |ancestor| {
                        ancestor_lookup(ancestor, &mut seen, &mut result);
                        None::<()>
                    },
                );
            }
        }

        result.names.into_values().collect()
    }
}

pub fn get_pattern_from_first_matching_condition(
    target: &JsonValue,
    conditions: &[String],
) -> String {
    if target.value_type == JsonValueType::String {
        return target.as_string().to_string();
    }
    if target.value_type == JsonValueType::Object {
        let obj = target.as_object();
        for (condition, value) in obj {
            if condition == "default"
                || conditions.iter().any(|c| c == condition)
                || (conditions.iter().any(|c| c == "types")
                    && tsox_tsoptions::module::mig::m3i::is_applicable_versioned_types_key(condition))
            {
                return get_pattern_from_first_matching_condition(value, conditions);
            }
        }
    }
    String::new()
}

pub fn is_in_reference_comment(file: &Arc<SourceFile>, position: usize) -> bool {
    let token = tsox_frontend::astnav::get_token_at_position(&file.node, position);
    let Some(comment_range) = crate::ls::utilities::is_in_comment(file, position, token.as_ref())
    else {
        return false;
    };
    let comment_text = &file.text[comment_range.pos..comment_range.end];
    has_triple_slash_prefix(comment_text)
}

pub fn has_triple_slash_prefix(comment_text: &str) -> bool {
    comment_text.starts_with("///") && comment_text[3..].trim_start().starts_with('<')
}

pub fn get_file_extension(file_name: &str) -> String {
    let extension = tsp::try_get_extension_from_path(file_name).to_string();
    if extension.is_empty() {
        return tsp::get_any_extension_from_path(file_name, &[], false);
    }
    extension
}

pub fn contains_slash(fragment: &str) -> bool {
    fragment.contains(tsp::DIRECTORY_SEPARATOR)
}

pub fn without_start_and_end(s: &str, start: &str, end: &str) -> Option<String> {
    if s.starts_with(start) && s.ends_with(end) && s.len() >= start.len() + end.len() {
        return Some(s[start.len()..s.len() - end.len()].to_string());
    }
    None
}

pub fn remove_leading_directory_separator(path: &str) -> String {
    path.strip_prefix(tsp::DIRECTORY_SEPARATOR)
        .unwrap_or(path)
        .to_string()
}

pub fn get_possible_original_input_path_without_changing_ext(
    file_path: &str,
    ignore_case: bool,
    output_dir: &str,
    get_common_source_directory: impl Fn() -> String,
) -> String {
    if !output_dir.is_empty() {
        return tsp::resolve_path(
            &get_common_source_directory(),
            &[&tsp::mig::m3i::get_relative_path_from_directory(
                output_dir,
                file_path,
                &tsp::ComparePathsOptions {
                    use_case_sensitive_file_names: !ignore_case,
                    current_directory: String::new(),
                },
            )],
        );
    }
    file_path.to_string()
}

pub fn get_filename_with_extension_option(
    name: &str,
    program: &Arc<Program>,
    extension_options: &ExtensionOptions,
    is_exports_or_imports_wildcard: bool,
) -> (String, String) {
    use tsox_tsoptions::modulespecifiers::ModuleSpecifierEnding;

    let non_js =
        tsox_tsoptions::modulespecifiers::try_get_real_file_name_for_non_js_declaration_file_name(name);
    if !non_js.is_empty() {
        return (non_js.clone(), tsp::try_get_extension_from_path(&non_js).to_string());
    }
    if extension_options.reference_kind == ReferenceKind::FileName {
        return (name.to_string(), tsp::try_get_extension_from_path(name).to_string());
    }

    let host = ProgramModuleSpecifierHost(program);
    let importing_source_file: &dyn tsox_tsoptions::modulespecifiers::SourceFileForSpecifierGeneration =
        extension_options
            .importing_source_file
            .as_deref()
            .expect("importing source file must be set for ending preference");
    let mut allowed_endings: Vec<ModuleSpecifierEnding> =
        tsox_tsoptions::modulespecifiers::get_allowed_endings_in_preferred_order(
            &tsox_tsoptions::modulespecifiers::UserPreferences {
                import_module_specifier_preference: String::new(),
                import_module_specifier_ending: extension_options.ending_preference.clone(),
                auto_import_specifier_exclude_regexes: Vec::new(),
            },
            &host,
            program.options(),
            importing_source_file,
            "",
            extension_options.resolution_mode,
        );

    if is_exports_or_imports_wildcard {
        allowed_endings
            .retain(|e| !matches!(e, ModuleSpecifierEnding::Minimal | ModuleSpecifierEnding::Index));
    }

    if !allowed_endings.is_empty() && allowed_endings[0] == ModuleSpecifierEnding::TsExtension {
        if tsp::file_extension_is_one_of(name, tsp::SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS) {
            return (name.to_string(), tsp::try_get_extension_from_path(name).to_string());
        }
        let output_extension =
            tsox_tsoptions::module::mig::m3i::try_get_js_extension_for_file(name, program.options());
        if !output_extension.is_empty() {
            return (
                tsp::change_extension(name, output_extension),
                output_extension.to_string(),
            );
        }
        return (name.to_string(), tsp::try_get_extension_from_path(name).to_string());
    }

    if !is_exports_or_imports_wildcard
        && !allowed_endings.is_empty()
        && matches!(
            allowed_endings[0],
            ModuleSpecifierEnding::Minimal | ModuleSpecifierEnding::Index
        )
        && tsp::file_extension_is_one_of(
            name,
            &[
                tsp::EXTENSION_JS,
                tsp::EXTENSION_JSX,
                tsp::EXTENSION_TS,
                tsp::EXTENSION_TSX,
                tsp::EXTENSION_DTS,
            ],
        )
    {
        return (
            tsp::remove_file_extension(name),
            tsp::try_get_extension_from_path(name).to_string(),
        );
    }

    let output_extension =
        tsox_tsoptions::module::mig::m3i::try_get_js_extension_for_file(name, program.options());
    if !output_extension.is_empty() {
        return (
            tsp::change_extension(name, output_extension),
            output_extension.to_string(),
        );
    }
    (name.to_string(), tsp::try_get_extension_from_path(name).to_string())
}

struct ProgramModuleSpecifierHost<'a>(&'a Program);

impl tsox_tsoptions::modulespecifiers::ModuleSpecifierGenerationHost
    for ProgramModuleSpecifierHost<'_>
{
    fn get_current_directory(&self) -> String {
        self.0.get_current_directory().to_string()
    }

    fn use_case_sensitive_file_names(&self) -> bool {
        self.0.use_case_sensitive_file_names()
    }

    fn common_source_directory(&self) -> String {
        tsox_compile::mig::m4v::get_common_source_directory(
            self.0.options(),
            || {
                self.0
                    .source_files()
                    .iter()
                    .map(|f| f.file_name.clone())
                    .collect()
            },
            &self.0.get_current_directory(),
            self.0.use_case_sensitive_file_names(),
        )
    }

    fn file_exists(&self, path: &str) -> bool {
        self.0.file_exists(path)
    }
}

pub fn get_supported_extensions_for_module_resolution(
    options: &tsox_core::core::compiler_options::CompilerOptions,
    extra_extensions: &[String],
    checker: Option<&mut Checker>,
) -> Vec<String> {
    let mut extensions: Vec<String> = Vec::new();
    if let Some(checker) = checker {
        for module in checker.get_ambient_modules() {
            let name = get_ambient_module_name(&module);
            if !name.starts_with("*.") || name.contains('/') {
                continue;
            }
            extensions.push(name[1..].to_string());
        }
    }
    let extra_extensions: Vec<&str> = extra_extensions.iter().map(|s| s.as_str()).collect();
    let supported_extensions =
        tsox_tsoptions::mig::m5i_3::get_supported_extensions(options, &extra_extensions);
    for ext in &supported_extensions {
        extensions.extend(ext.iter().cloned());
    }
    let module_resolution = options.get_module_resolution_kind();
    if module_resolution_uses_node_modules(module_resolution) {
        let wrapped =
            tsox_tsoptions::mig::m5i_3::get_supported_extensions_with_json_if_resolve_json_module(
                options,
                &[extensions],
            );
        return wrapped.into_iter().flatten().collect();
    }
    extensions
}

pub fn get_conditions(
    options: &tsox_core::core::compiler_options::CompilerOptions,
    mut resolution_mode: ResolutionMode,
) -> Vec<String> {
    let module_resolution = options.get_module_resolution_kind();
    if resolution_mode == ResolutionMode::None
        && module_resolution == ModuleResolutionKind::Bundler
    {
        resolution_mode = ResolutionMode::ESNext;
    }
    let mut conditions = Vec::with_capacity(3 + options.custom_conditions.len());
    if resolution_mode == ResolutionMode::ESNext {
        conditions.push("import".to_string());
    } else {
        conditions.push("require".to_string());
    }
    if !options.no_dts_resolution.is_true() {
        conditions.push("types".to_string());
    }
    if module_resolution != ModuleResolutionKind::Bundler {
        conditions.push("node".to_string());
    }
    conditions.extend(options.custom_conditions.iter().cloned());
    conditions
}

pub fn module_resolution_uses_node_modules(module_resolution: ModuleResolutionKind) -> bool {
    matches!(
        module_resolution,
        ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext
    ) || module_resolution == ModuleResolutionKind::Bundler
}

pub fn is_path_relative_to_script(path: &str) -> bool {
    path.starts_with("./") || path.starts_with("../")
}

pub fn get_fragment_directory(fragment: &str) -> String {
    if !contains_slash(fragment) {
        return String::new();
    }
    if tsp::has_trailing_directory_separator(fragment) {
        return fragment.to_string();
    }
    tsp::get_directory_path(fragment)
}

pub fn get_ambient_module_completions(
    fragment: &str,
    fragment_directory: &str,
    type_checker: &mut Checker,
) -> Vec<String> {
    let ambient_modules = type_checker.get_ambient_modules();
    let mut non_relative_module_names: Vec<String> = Vec::new();
    for sym in &ambient_modules {
        let module_name = get_ambient_module_name(sym);
        if module_name.starts_with(fragment) && !module_name.contains('*') {
            non_relative_module_names.push(module_name);
        }
    }

    if !fragment_directory.is_empty() {
        let module_name_with_separator =
            tsp::ensure_trailing_directory_separator(fragment_directory);
        for module_name in &mut non_relative_module_names {
            *module_name = module_name
                .strip_prefix(module_name_with_separator.as_str())
                .unwrap_or(module_name)
                .to_string();
        }
    }
    non_relative_module_names
}

pub fn get_ambient_module_name(symbol: &Arc<Symbol>) -> String {
    let declaration = tsox_frontend::ast::mig::m3f::get_non_augmentation_declaration(symbol);
    if let Some(declaration) = declaration {
        if tsox_frontend::ast::is_module_with_string_literal_name(&declaration) {
            return declaration
                .name()
                .map(|n| n.text().to_string())
                .unwrap_or_default();
        }
    }
    strip_quotes(&symbol.name).to_string()
}

fn strip_quotes(name: &str) -> &str {
    let bytes = name.as_bytes();
    if bytes.len() >= 2
        && bytes[0] == bytes[bytes.len() - 1]
        && (bytes[0] == b'"' || bytes[0] == b'\'')
    {
        return &name[1..bytes.len() - 1];
    }
    name
}

pub fn try_remove_directory_prefix(
    path: &str,
    prefix: &str,
    use_case_sensitive_file_names: bool,
) -> Option<String> {
    let (without_prefix, ok) =
        tsp::mig::m3j::trim_file_path_prefix(path, prefix, use_case_sensitive_file_names);
    if !ok {
        return None;
    }
    let without_prefix = if without_prefix.starts_with('/') || without_prefix.starts_with('\\') {
        without_prefix[1..].to_string()
    } else {
        without_prefix
    };
    Some(without_prefix)
}

pub fn contains_path(parent: &str, child: &str, options: &tsp::ComparePathsOptions) -> bool {
    let parent = tsp::combine_paths(&options.current_directory, &[parent]);
    let child = tsp::combine_paths(&options.current_directory, &[child]);
    if parent.is_empty() || child.is_empty() {
        return false;
    }
    if parent == child {
        return true;
    }
    let parent_components = tsp::reduce_path_components(&tsp::get_path_components(&parent, ""));
    let child_components = tsp::reduce_path_components(&tsp::get_path_components(&child, ""));
    if child_components.len() < parent_components.len() {
        return false;
    }
    let equal = options.get_comparer();
    for i in 1..parent_components.len() {
        if equal(&parent_components[i], &child_components[i]) != 0 {
            return false;
        }
    }
    true
}

pub fn get_base_directories_from_root_dirs(
    root_dirs: &[String],
    base_path: &str,
    script_directory: &str,
    ignore_case: bool,
) -> Vec<String> {
    let mut normalized_root_dirs: Vec<String> = Vec::with_capacity(root_dirs.len());
    for root_directory in root_dirs {
        let normalized_path = if tsp::is_rooted_disk_path(root_directory) {
            root_directory.clone()
        } else {
            tsp::combine_paths(base_path, &[root_directory])
        };
        normalized_root_dirs.push(tsp::ensure_trailing_directory_separator(
            &tsp::normalize_path(&normalized_path),
        ));
    }

    let mut relative_directory = String::new();
    let compare_paths_options = tsp::ComparePathsOptions {
        use_case_sensitive_file_names: !ignore_case,
        current_directory: base_path.to_string(),
    };
    for root_directory in &normalized_root_dirs {
        if contains_path(root_directory, script_directory, &compare_paths_options) {
            if root_directory.len() > script_directory.len() {
                relative_directory = String::new();
            } else {
                relative_directory = script_directory[root_directory.len()..].to_string();
            }
            break;
        }
    }

    let mut directories: Vec<String> = Vec::new();
    for root_directory in &normalized_root_dirs {
        directories.push(tsp::remove_trailing_directory_separator(
            &tsp::combine_paths(root_directory, &[&relative_directory]),
        ));
    }
    directories.push(tsp::remove_trailing_directory_separator(script_directory));

    deduplicate_strings(directories)
}

pub fn deduplicate_strings(slice: Vec<String>) -> Vec<String> {
    if slice.len() <= 1 {
        return slice;
    }
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for s in slice {
        if seen.insert(s.clone()) {
            result.push(s);
        }
    }
    result
}

pub fn deduplicate_module_completions(
    completions: Vec<ModuleCompletionNameAndKind>,
) -> Vec<ModuleCompletionNameAndKind> {
    if completions.len() <= 1 {
        return completions;
    }
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for c in completions {
        let key = (c.name.clone(), c.kind, c.extension.clone());
        if seen.insert(key) {
            result.push(c);
        }
    }
    result
}

pub fn modulet_to_script_element_kind(kind: ModuleCompletionKind) -> ScriptElementKind {
    match kind {
        ModuleCompletionKind::Directory => ScriptElementKind::Directory,
        ModuleCompletionKind::File => ScriptElementKind::ScriptElement,
        ModuleCompletionKind::ExternalModuleName => ScriptElementKind::ExternalModuleName,
    }
}

pub fn is_any_directory_separator(r: char) -> bool {
    r == '/' || r == '\\'
}

pub fn get_directory_fragment_range(text: &str, text_start: usize) -> Option<TextRange> {
    let index = text.rfind(is_any_directory_separator);
    let offset = match index {
        Some(i) => i + 1,
        None => 0,
    };
    let length = text.len() - offset;
    if length == 0 {
        return None;
    }
    Some(TextRange::new(
        text_start + offset,
        text_start + offset + length,
    ))
}

pub fn kind_modifiers_from_extension(extension: &str) -> ScriptElementKindModifier {
    match extension {
        tsp::EXTENSION_DTS => ScriptElementKindModifier::DTS,
        tsp::EXTENSION_JS => ScriptElementKindModifier::JS,
        tsp::EXTENSION_JSON => ScriptElementKindModifier::JSON,
        tsp::EXTENSION_JSX => ScriptElementKindModifier::JSX,
        tsp::EXTENSION_TS => ScriptElementKindModifier::TS,
        tsp::EXTENSION_TSX => ScriptElementKindModifier::TSX,
        tsp::EXTENSION_DMTS => ScriptElementKindModifier::DMTS,
        tsp::EXTENSION_MJS => ScriptElementKindModifier::MJS,
        tsp::EXTENSION_MTS => ScriptElementKindModifier::MTS,
        tsp::EXTENSION_DCTS => ScriptElementKindModifier::DCTS,
        tsp::EXTENSION_CJS => ScriptElementKindModifier::CJS,
        tsp::EXTENSION_CTS => ScriptElementKindModifier::CTS,
        tsp::EXTENSION_TS_BUILD_INFO => panic!("Extension {:?} is unsupported.", tsp::EXTENSION_TS_BUILD_INFO),
        _ => ScriptElementKindModifier::NONE,
    }
}

pub fn m5w2_to_path_completions(
    names: Vec<ModuleCompletionNameAndKind>,
) -> Vec<PathCompletion> {
    names
        .into_iter()
        .map(|name_and_kind| PathCompletion {
            name: name_and_kind.name,
            kind: modulet_to_script_element_kind(name_and_kind.kind),
            extension: name_and_kind.extension,
        })
        .collect()
}


fn get_types_versions_paths(
    types_versions: &tsox_tsoptions::packagejson::JsonValue,
) -> Option<tsox_core::collections::ordered_map::OrderedMap<String, Vec<String>>> {
    use tsox_tsoptions::packagejson::JsonValueType;

    if types_versions.value_type != JsonValueType::Object {
        return None;
    }
    let version = tsox_core::semver::try_parse_version(
        tsox_core::core::mig::m3k::version_major_minor(),
    )
    .ok()?;
    for (key, value) in types_versions.as_object() {
        let Some(range) = tsox_core::semver::try_parse_version_range(key) else {
            continue;
        };
        if range.test(&version) {
            if value.value_type != JsonValueType::Object {
                return None;
            }
            let mut paths = tsox_core::collections::ordered_map::OrderedMap::new();
            for (path_key, path_value) in value.as_object() {
                let entries: Vec<String> = path_value
                    .as_array()
                    .iter()
                    .map(|v| v.as_string().to_string())
                    .collect();
                paths.insert(path_key.clone(), entries);
            }
            return Some(paths);
        }
    }
    None
}
