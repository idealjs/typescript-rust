#![allow(unused_imports)]

use crate::tsoptions::*;
use std::collections::HashMap;
use tsox_core::json::Value;
use tsox_core::tspath::Path;
use tsox_frontend::ast::diagnostic::Diagnostic;

#[derive(Debug, Clone)]
pub struct SourceOutputAndProjectReference {
    pub source: String,
    pub output_dts: String,
    pub resolved: Option<u32>,
}

impl ParsedCommandLine {
    pub fn project_references(&self) -> &[tsox_core::core::project_reference::ProjectReference] {
        &self.references
    }

    pub fn set_compiler_options(&mut self, o: CompilerOptions) {
        self.compiler_options = o;
    }

    pub fn set_parsed_options(&mut self, o: ParsedCommandLine) {
        *self = o;
    }

    pub fn set_type_acquisition(&mut self, o: tsox_core::core::mig::m3k::TypeAcquisition) {
        self.type_acquisition = Some(o);
    }

    pub fn type_acquisition(&self) -> Option<&tsox_core::core::mig::m3k::TypeAcquisition> {
        self.type_acquisition.as_ref()
    }

    pub fn wildcard_directories(&self) -> HashMap<String, bool> {
        super::m5j_3::get_wildcard_directories(
            &self.include,
            &self.exclude,
            &tsox_core::tspath::ComparePathsOptions {
                use_case_sensitive_file_names: self.use_case_sensitive_file_names(),
                current_directory: tsox_core::tspath::get_directory_path(&self.config_file_name),
            },
        )
    }

    pub fn file_glob_patterns(&self) -> (String, String) {
        const FILE_GLOB: &str = "*.{js,jsx,mjs,cjs,ts,tsx,mts,cts,json}";
        const RECURSIVE_FILE_GLOB: &str = "**/*.{js,jsx,mjs,cjs,ts,tsx,mts,cts,json}";
        let mapper_extensions = self.content_mapper_extensions();
        if mapper_extensions.is_empty() {
            return (FILE_GLOB.to_string(), RECURSIVE_FILE_GLOB.to_string());
        }
        let mut extensions: Vec<String> = vec![
            "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "json",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        for extension in &mapper_extensions {
            extensions.push(extension.trim_start_matches('.').to_string());
        }
        let file_glob = format!("*.{{{}}}", extensions.join(","));
        (file_glob.clone(), format!("**/{file_glob}"))
    }

    pub fn wildcard_directory_globs(&self) -> Vec<tsox_core::glob::Glob> {
        let wildcard_directories = self.wildcard_directories();
        if wildcard_directories.is_empty() {
            return Vec::new();
        }
        let (file_glob, recursive_file_glob) = self.file_glob_patterns();
        let mut globs = Vec::with_capacity(wildcard_directories.len());
        for (dir, recursive) in &wildcard_directories {
            let pattern = if *recursive {
                format!("{}/{recursive_file_glob}", tsox_core::tspath::normalize_path(dir))
            } else {
                format!("{}/{file_glob}", tsox_core::tspath::normalize_path(dir))
            };
            if let Ok(parsed) = tsox_core::glob::Glob::parse(&pattern) {
                globs.push(parsed);
            }
        }
        globs
    }

    pub fn with_file_names(&self, file_names: Vec<String>) -> ParsedCommandLine {
        let mut result = self.clone();
        result.file_names = file_names;
        result
    }

    pub fn possibly_matches_file_name(&self, file_name: &str) -> bool {
        let path = tsox_core::tspath::to_path(file_name, "", self.use_case_sensitive_file_names());
        if self.file_names.iter().any(|f| {
            tsox_core::tspath::to_path(f, "", self.use_case_sensitive_file_names()) == path
        }) {
            return true;
        }
        for include in &self.include {
            if !include.contains('*') && !include.contains('?') && !crate::vfs::vfsmatch::is_implicit_glob(include) {
                let include_path =
                    tsox_core::tspath::to_path(include, "", self.use_case_sensitive_file_names());
                if include_path == path {
                    return true;
                }
            }
        }
        if self.get_content_mapper_for_file_name(file_name).is_some() {
            let directory_path = path.get_directory_path();
            if self.possibly_matches_directory_name(&directory_path) {
                return true;
            }
        }
        let wildcard_directory_globs = self.wildcard_directory_globs();
        for glob in &wildcard_directory_globs {
            if glob.is_match(file_name) {
                return true;
            }
        }
        false
    }

    pub fn possibly_matches_directory_name(&self, directory_path: &Path) -> bool {
        for (wildcard_dir, recursive) in &self.wildcard_directories() {
            let wildcard_dir_path =
                tsox_core::tspath::to_path(wildcard_dir, "", self.use_case_sensitive_file_names());
            if *recursive {
                if wildcard_dir_path.contains_path(directory_path) {
                    return true;
                }
            } else if wildcard_dir_path == *directory_path {
                return true;
            }
        }
        false
    }

    pub fn reload_file_names_of_parsed_command_line(&self, fs: &dyn FS) -> ParsedCommandLine {
        let mut result = self.clone();
        let (file_names, literal_file_names_len) = get_file_names_from_config_specs(
            &self.files_spec,
            &self.include,
            &self.exclude,
            &self.compiler_options,
            fs,
            &self.content_mapper_extensions(),
        );
        result.file_names = file_names;
        result.literal_file_names_len = literal_file_names_len;
        result
    }
}

fn contains_path_str(parent: &str, child: &str) -> bool {
    use tsox_core::tspath::Path;
    Path::from(parent).contains_path(&Path::from(child))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapperDefinition {
    pub extensions: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mapper {
    pub definition: MapperDefinition,
}

mod outputpaths {
    use super::*;

    use tsox_core::tspath::mig::m3i::{
        get_declaration_emit_extension_for_path, get_relative_path_from_directory,
    };

    pub fn get_output_declaration_file_name_worker(
        input_file_name: &str,
        options: &CompilerOptions,
    ) -> String {
        let dir = if options.declaration_dir.is_empty() {
            &options.out_dir
        } else {
            &options.declaration_dir
        };
        if dir.is_empty() {
            return change_to_declaration_extension(input_file_name);
        }
        let current_directory = tsox_core::tspath::get_directory_path(&options.config_file_path);
        let relative = get_relative_path_from_directory(
            &current_directory,
            input_file_name,
            &tsox_core::tspath::ComparePathsOptions {
                use_case_sensitive_file_names: true,
                current_directory: current_directory.clone(),
            },
        );
        let output = tsox_core::tspath::resolve_path(dir, &[relative.as_str()]);
        change_to_declaration_extension(&output)
    }

    fn change_to_declaration_extension(path: &str) -> String {
        let path_without_extension = tsox_core::tspath::remove_file_extension(path);
        format!(
            "{}{}",
            path_without_extension,
            get_declaration_emit_extension_for_path(path)
        )
    }
}

fn get_supported_extensions(
    compiler_options: &CompilerOptions,
    extra_extensions: &[String],
) -> Vec<Vec<String>> {
    let builtins: Vec<Vec<String>> = if compiler_options.get_allow_js() {
        vec![
            vec![
                ".ts".into(),
                ".tsx".into(),
                ".d.ts".into(),
                ".js".into(),
                ".jsx".into(),
            ],
            vec![".cts".into(), ".d.cts".into(), ".cjs".into()],
            vec![".mts".into(), ".d.mts".into(), ".mjs".into()],
        ]
    } else {
        vec![
            vec![".ts".into(), ".tsx".into(), ".d.ts".into()],
            vec![".cts".into(), ".d.cts".into()],
            vec![".mts".into(), ".d.mts".into()],
        ]
    };
    if extra_extensions.is_empty() {
        return builtins;
    }
    let flat_builtins: Vec<&str> = builtins.iter().flatten().map(String::as_str).collect();
    let mut result: Vec<Vec<String>> = Vec::new();
    for ext in extra_extensions {
        if !flat_builtins.contains(&ext.as_str()) {
            result.push(vec![ext.clone()]);
        }
    }
    if result.is_empty() {
        return builtins;
    }
    let mut all = builtins;
    all.extend(result);
    all
}

fn get_supported_extensions_with_json_if_resolve_json_module(
    compiler_options: &CompilerOptions,
    supported_extensions: &[Vec<String>],
) -> Vec<Vec<String>> {
    if !compiler_options.get_resolve_json_module() {
        return supported_extensions.to_vec();
    }
    if supported_extensions
        .iter()
        .any(|group| group.len() == 1 && group[0] == tsox_core::tspath::EXTENSION_JSON)
    {
        return supported_extensions.to_vec();
    }
    let mut result = supported_extensions.to_vec();
    result.push(vec![tsox_core::tspath::EXTENSION_JSON.to_string()]);
    result
}

fn has_file_with_higher_priority_extension(
    file: &str,
    extensions: &[Vec<String>],
    has_file: &dyn Fn(&str) -> bool,
) -> bool {
    let mut extension_group: Vec<&str> = Vec::new();
    for group in extensions {
        let group_refs: Vec<&str> = group.iter().map(String::as_str).collect();
        if tsox_core::tspath::file_extension_is_one_of(file, &group_refs) {
            extension_group.extend(group.iter().map(String::as_str));
        }
    }
    if extension_group.is_empty() {
        return false;
    }
    for ext in &extension_group {
        if tsox_core::tspath::file_extension_is(file, ext)
            && (*ext != tsox_core::tspath::EXTENSION_TS
                || !tsox_core::tspath::file_extension_is(
                    file,
                    tsox_core::tspath::EXTENSION_DTS,
                ))
        {
            return false;
        }
        if has_file(&tsox_core::tspath::change_extension(file, ext)) {
            if *ext == tsox_core::tspath::EXTENSION_DTS
                && tsox_core::tspath::file_extension_is_one_of(
                    file,
                    &[
                        tsox_core::tspath::EXTENSION_JS,
                        tsox_core::tspath::EXTENSION_JSX,
                    ],
                )
            {
                continue;
            }
            return true;
        }
    }
    false
}

fn remove_wildcard_files_with_lower_priority_extension(
    file: &str,
    wildcard_files: &mut tsox_core::collections::ordered_map::OrderedMap<String, String>,
    extensions: &[Vec<String>],
    key_mapper: &dyn Fn(&str) -> String,
) {
    let mut extension_group: Vec<&str> = Vec::new();
    for group in extensions {
        let group_refs: Vec<&str> = group.iter().map(String::as_str).collect();
        if tsox_core::tspath::file_extension_is_one_of(file, &group_refs) {
            extension_group.extend(group.iter().map(String::as_str));
        }
    }
    if extension_group.is_empty() {
        return;
    }
    for ext in extension_group.iter().rev() {
        if tsox_core::tspath::file_extension_is(file, ext) {
            return;
        }
        let lower_priority_path = key_mapper(&tsox_core::tspath::change_extension(file, ext));
        wildcard_files.delete(&lower_priority_path);
    }
}

fn get_file_names_from_config_specs(
    validated_files_spec: &[String],
    validated_include_specs: &[String],
    validated_exclude_specs: &[String],
    options: &CompilerOptions,
    host: &dyn FS,
    extra_extensions: &[String],
) -> (Vec<String>, usize) {
    let base_path = tsox_core::tspath::normalize_path(&tsox_core::tspath::get_directory_path(
        &options.config_file_path,
    ));
    let use_case_sensitive_file_names = host.use_case_sensitive_file_names();
    let key_mapper = |value: &str| tsox_core::tspath::get_canonical_file_name(value, use_case_sensitive_file_names);

    let mut literal_file_map: tsox_core::collections::ordered_map::OrderedMap<String, String> =
        Default::default();
    let mut wildcard_file_map: tsox_core::collections::ordered_map::OrderedMap<String, String> =
        Default::default();
    let mut wild_card_json_file_map: tsox_core::collections::ordered_map::OrderedMap<
        String,
        String,
    > = Default::default();

    let supported_extensions = get_supported_extensions(options, extra_extensions);
    let supported_extensions_with_json_if_resolve_json_module =
        get_supported_extensions_with_json_if_resolve_json_module(options, &supported_extensions);

    for file_name in validated_files_spec {
        let file = tsox_core::tspath::get_normalized_absolute_path(file_name, &base_path);
        literal_file_map.set(key_mapper(file_name), file);
    }

    if !validated_include_specs.is_empty() {
        let flat_extensions: Vec<&str> =
            supported_extensions_with_json_if_resolve_json_module
                .iter()
                .flatten()
                .map(String::as_str)
                .collect();
        let excludes: Vec<&str> = validated_exclude_specs.iter().map(String::as_str).collect();
        let includes: Vec<&str> = validated_include_specs.iter().map(String::as_str).collect();
        let files = crate::vfs::vfsmatch::read_directory(
            host,
            &base_path,
            &base_path,
            &flat_extensions,
            &excludes,
            &includes,
            crate::vfs::vfsmatch::UNLIMITED_DEPTH,
        );
        let mut json_only_include_matchers: Option<crate::vfs::vfsmatch::SpecMatcher> = None;
        for file in files {
            if tsox_core::tspath::file_extension_is(
                &file,
                tsox_core::tspath::EXTENSION_JSON,
            ) {
                if json_only_include_matchers.is_none() {
                    let includes: Vec<&str> = validated_include_specs
                        .iter()
                        .map(String::as_str)
                        .filter(|include| include.ends_with(tsox_core::tspath::EXTENSION_JSON))
                        .collect();
                    json_only_include_matchers = crate::vfs::vfsmatch::SpecMatcher::new(
                        &includes,
                        &base_path,
                        crate::vfs::vfsmatch::Usage::Files,
                        use_case_sensitive_file_names,
                    );
                }
                let include_index = json_only_include_matchers
                    .as_ref()
                    .map(|m| m.match_index(&file))
                    .unwrap_or(-1);
                if include_index != -1 {
                    let key = key_mapper(&file);
                    if !literal_file_map.has(&key) && !wild_card_json_file_map.has(&key) {
                        wild_card_json_file_map.set(key, file.clone());
                    }
                }
                continue;
            }
            let has_higher = has_file_with_higher_priority_extension(&file, &supported_extensions, &|file_name| {
                let canonical = key_mapper(file_name);
                literal_file_map.has(&canonical) || wildcard_file_map.has(&canonical)
            });
            if has_higher {
                continue;
            }
            remove_wildcard_files_with_lower_priority_extension(
                &file,
                &mut wildcard_file_map,
                &supported_extensions,
                &key_mapper,
            );
            let key = key_mapper(&file);
            if !literal_file_map.has(&key) && !wildcard_file_map.has(&key) {
                wildcard_file_map.set(key, file.clone());
            }
        }
    }

    let mut files: Vec<String> = Vec::with_capacity(
        literal_file_map.len() + wildcard_file_map.len() + wild_card_json_file_map.len(),
    );
    files.extend(literal_file_map.values().cloned());
    files.extend(wildcard_file_map.values().cloned());
    files.extend(wild_card_json_file_map.values().cloned());
    (files, literal_file_map.len())
}
