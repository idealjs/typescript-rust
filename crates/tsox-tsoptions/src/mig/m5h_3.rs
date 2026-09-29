#![allow(dead_code, unused_imports, unused_variables)]

use super::*;

use std::sync::Arc;

use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::core::{filter, flat_map};
use tsox_core::core::mig::m3j_3::resolve_project_reference_path;
use tsox_core::core::project_reference::ProjectReference;
use tsox_core::diagnostics;
use tsox_core::locale;
use tsox_core::tspath;
use tsox_core::tspath::ComparePathsOptions;
use tsox_core::tspath::mig::m3i::{compare_paths, get_longest_extension_from_path};
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;

use crate::tsoptions::build_options::ParsedCommandLine;
use crate::tsoptions::impl_chunk::ParsedBuildCommandLine;

pub const FILE_GLOB_PATTERN: &str = "*.{js,jsx,mjs,cjs,ts,tsx,mts,cts,json}";
pub const RECURSIVE_FILE_GLOB_PATTERN: &str = "**/*.{js,jsx,mjs,cjs,ts,tsx,mts,cts,json}";

#[derive(Debug, Clone, Default)]
pub struct ContentMapperDefinition {
    pub package: String,
    pub extensions: Vec<String>,
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct ContentMapper {
    pub definition: ContentMapperDefinition,
    pub manifest: crate::mig::m5h_2::ContentMapperManifest,
    pub package_directory: String,
    pub contribution_id: String,
}

impl ContentMapper {
    pub fn diagnostic_name(&self) -> String {
        if !self.manifest.name.is_empty() {
            return self.manifest.name.clone();
        }
        if !self.definition.package.is_empty() {
            return self.definition.package.clone();
        }
        self.contribution_id.clone()
    }

    pub fn identity(&self) -> String {
        if !self.contribution_id.is_empty() {
            return format!("{} ({})", self.contribution_id, self.manifest_identity());
        }
        self.manifest_identity()
    }

    pub fn manifest_identity(&self) -> String {
        if self.manifest.name.is_empty() {
            return String::new();
        }
        if self.manifest.version.is_empty() {
            return self.manifest.name.clone();
        }
        format!("{}@{}", self.manifest.name, self.manifest.version)
    }
}

fn contains_path(parent: &str, target: &str, options: &ComparePathsOptions) -> bool {
    let parent = tspath::combine_paths(&options.current_directory, &[parent]);
    let target = tspath::combine_paths(&options.current_directory, &[target]);
    if parent.is_empty() || target.is_empty() {
        return false;
    }
    if parent == target {
        return true;
    }
    let parent_components =
        tspath::reduce_path_components(&tspath::get_path_components(&parent, ""));
    let target_components =
        tspath::reduce_path_components(&tspath::get_path_components(&target, ""));
    if target_components.len() < parent_components.len() {
        return false;
    }
    for (i, parent_component) in parent_components.iter().enumerate() {
        let child_component = &target_components[i];
        let equal = if i == 0 || !options.use_case_sensitive_file_names {
            parent_component.eq_ignore_ascii_case(child_component)
        } else {
            parent_component == child_component
        };
        if !equal {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone)]
pub struct SourceOutputAndProjectReference {
    pub source: String,
    pub output_dts: String,
    pub resolved: Option<Arc<ParsedCommandLine>>,
}

pub fn new_parsed_command_line(
    compiler_options: CompilerOptions,
    root_file_names: Vec<String>,
    project_references: Vec<ProjectReference>,
    compare_paths_options: ComparePathsOptions,
) -> ParsedCommandLine {
    ParsedCommandLine {
        compiler_options,
        file_names: root_file_names,
        references: project_references,
        compare_paths_options,
        ..Default::default()
    }
}

impl ParsedCommandLine {
    pub fn config_name(&self) -> String {
        if self.config_file.is_none() {
            return String::new();
        }
        self.config_file
            .as_ref()
            .unwrap()
            .source_file
            .file_name
            .clone()
    }

    pub fn source_to_project_reference(&self) -> &OrderedMap<String, SourceOutputAndProjectReference> {
        &self.source_to_project_reference
    }

    pub fn output_dts_to_project_reference(
        &self,
    ) -> &OrderedMap<String, SourceOutputAndProjectReference> {
        &self.output_dts_to_project_reference
    }

    pub fn parse_input_output_names(&mut self) {
        if self.source_and_output_maps_once.get().is_some() {
            return;
        }
        let mut source_to_output = OrderedMap::new();
        let mut output_dts_to_source = OrderedMap::new();
        let current_directory = self.get_current_directory();
        let use_case_sensitive_file_names = self.use_case_sensitive_file_names();
        for (output_dts, source) in self.get_output_declaration_and_source_file_names() {
            let path =
                tsox_core::tspath::to_path(&source, &current_directory, use_case_sensitive_file_names);
            let project_reference = SourceOutputAndProjectReference {
                source: source.clone(),
                output_dts: output_dts.clone(),
                resolved: None,
            };
            if !output_dts.is_empty() {
                output_dts_to_source.set(
                    tsox_core::tspath::to_path(
                        &output_dts,
                        &current_directory,
                        use_case_sensitive_file_names,
                    )
                    .0,
                    project_reference.clone(),
                );
            }
            source_to_output.set(path.0, project_reference);
        }
        self.output_dts_to_project_reference = output_dts_to_source;
        self.source_to_project_reference = source_to_output;
        self.source_and_output_maps_once.set(());
    }

    pub fn common_source_directory(&mut self) -> String {
        if self.common_source_directory_once.get().is_none() {
            let files = || {
                filter(&self.file_names, |file| {
                    !(self.compiler_options.no_emit_for_js_files.is_true()
                        && tsox_core::tspath::has_js_file_extension(file))
                        && !tsox_core::tspath::is_declaration_file_name(file)
                })
            };
            let pending_errors = std::cell::RefCell::new(Vec::<Diagnostic>::new());
            let check = |source_files: &[String], root_directory: &str| {
                let mut all_files_belong_to_path = true;
                for file in source_files {
                    let absolute_source_file_path = tsox_core::tspath::get_canonical_file_name(
                        &tsox_core::tspath::get_normalized_absolute_path(
                            file,
                            &self.get_current_directory(),
                        ),
                        self.use_case_sensitive_file_names(),
                    );
                    if !contains_path(root_directory, file, &self.compare_paths_options) {
                        pending_errors.borrow_mut().push(new_compiler_diagnostic(
                            tsox_core::diagnostics::FILE_0_IS_NOT_UNDER_ROOTDIR_1_ROOTDIR_IS_EXPECTED_TO_CONTAIN_ALL_SOURCE_FILES,
                            vec![absolute_source_file_path, root_directory.to_string()],
                        ));
                        all_files_belong_to_path = false;
                    }
                }
                all_files_belong_to_path
            };
            let directory = outputpaths::get_common_source_directory(
                &self.compiler_options,
                files,
                &self.get_current_directory(),
                self.use_case_sensitive_file_names(),
                check,
            );
            self.errors.extend(pending_errors.into_inner());
            self.common_source_directory = directory;
            self.common_source_directory_once.set(());
        }
        self.common_source_directory.clone()
    }

    pub fn get_current_directory(&self) -> String {
        self.compare_paths_options.current_directory.clone()
    }

    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.compare_paths_options.use_case_sensitive_file_names
    }

    pub fn get_output_declaration_and_source_file_names(&self) -> Vec<(String, String)> {
        self.file_names
            .iter()
            .map(|file_name| {
                let output_dts = if !tsox_core::tspath::is_declaration_file_name(file_name)
                    && !tsox_core::tspath::file_extension_is(file_name, tsox_core::tspath::EXTENSION_JSON)
                {
                    outputpaths::get_output_declaration_file_name_worker(
                        file_name,
                        &self.compiler_options,
                        self,
                    )
                } else {
                    String::new()
                };
                (output_dts, file_name.clone())
            })
            .collect()
    }

    pub fn get_output_file_names(&self) -> Vec<String> {
        let mut result = Vec::new();
        for file_name in &self.file_names {
            if tsox_core::tspath::is_declaration_file_name(file_name) {
                continue;
            }
            let js_file_name = outputpaths::get_output_js_file_name(
                file_name,
                &self.compiler_options,
                self,
            );
            let is_json = tsox_core::tspath::file_extension_is(file_name, tsox_core::tspath::EXTENSION_JSON);
            if !js_file_name.is_empty() {
                result.push(js_file_name.clone());
                if !is_json {
                    let source_map = outputpaths::get_source_map_file_path(&js_file_name, &self.compiler_options);
                    if !source_map.is_empty() {
                        result.push(source_map);
                    }
                }
            }
            if is_json {
                continue;
            }
            if self.compiler_options.get_emit_declarations() {
                let dts_file_name = outputpaths::get_output_declaration_file_name_worker(
                    file_name,
                    &self.compiler_options,
                    self,
                );
                if !dts_file_name.is_empty() {
                    result.push(dts_file_name.clone());
                    if self.get_content_mapper_for_file_name(file_name).is_none()
                        && self.compiler_options.get_are_declaration_maps_enabled()
                    {
                        result.push(format!("{}.map", dts_file_name));
                    }
                }
            }
        }
        result
    }

    pub fn get_build_info_file_name(&self) -> String {
        outputpaths::get_build_info_file_name(&self.compiler_options, &self.compare_paths_options)
    }

    pub fn literal_file_names(&self) -> &[String] {
        if self.config_file.is_some() {
            &self.file_names[..self.literal_file_names_len]
        } else {
            &[]
        }
    }

    pub fn compiler_options(&self) -> &CompilerOptions {
        &self.compiler_options
    }

    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }

    pub fn file_names_by_path(&mut self) -> &OrderedMap<String, String> {
        if self.file_names_by_path_once.get().is_none() {
            let mut file_names_by_path = OrderedMap::with_capacity(self.file_names.len());
            let current_directory = self.get_current_directory();
            let use_case_sensitive_file_names = self.use_case_sensitive_file_names();
            for file_name in &self.file_names {
                let path = tsox_core::tspath::to_path(
                    file_name,
                    &current_directory,
                    use_case_sensitive_file_names,
                );
                file_names_by_path.set(path.0, file_name.clone());
            }
            self.file_names_by_path = file_names_by_path;
            self.file_names_by_path_once.set(());
        }
        &self.file_names_by_path
    }

    pub fn content_mappers(&self) -> &[ContentMapper] {
        &self.content_mappers
    }

    pub fn content_mapper_extensions(&self) -> Vec<String> {
        flat_map(self.content_mappers(), |m| m.definition.extensions.as_slice())
    }

    pub fn get_content_mapper_for_file_name(&self, file_name: &str) -> Option<&ContentMapper> {
        let ignore_case = !self.use_case_sensitive_file_names();
        let extensions = self.content_mapper_extensions();
        let extension_refs: Vec<&str> = extensions.iter().map(String::as_str).collect();
        let extension = get_longest_extension_from_path(file_name, &extension_refs, ignore_case);
        self.content_mappers().iter().find(|mapper| {
            mapper.definition.extensions.iter().any(|mapper_extension| {
                extension == *mapper_extension
                    || (ignore_case && extension.eq_ignore_ascii_case(mapper_extension))
            })
        })
    }

    pub fn extended_source_files(&self) -> &[String] {
        match &self.config_file {
            Some(config_file) => &config_file.extended_source_files,
            None => &[],
        }
    }

    pub fn get_config_file_parsing_diagnostics(&self) -> Vec<Diagnostic> {
        if let Some(config_file) = &self.config_file {
            let mut result =
                tsox_frontend::ast::mig::m3b_2::diagnostics(config_file.source_file.as_ref())
                    .to_vec();
            result.extend(self.errors.iter().cloned());
            result
        } else {
            self.errors.clone()
        }
    }

    pub fn get_matched_file_spec(&self, file_name: &str) -> Option<String> {
        Some(
            self.config_file
                .as_ref()?
                .config_file_specs
                .as_ref()?
                .get_matched_file_spec(file_name, &self.compare_paths_options),
        )
    }

    pub fn get_matched_include_spec(&self, file_name: &str) -> Option<(String, bool)> {
        let config_file = self.config_file.as_ref()?;
        let config_file_specs = config_file.config_file_specs.as_ref()?;
        if config_file_specs.validated_include_specs.is_empty() {
            return None;
        }
        if config_file_specs.is_default_include_spec {
            return Some((
                config_file_specs.validated_include_specs[0].clone(),
                true,
            ));
        }
        Some((
            config_file_specs.get_matched_include_spec(file_name, &self.compare_paths_options),
            false,
        ))
    }

    pub fn locale(&mut self) -> locale::Locale {
        self.locale_once.get_or_init(|| {
            if let Some(parsed) = locale::Locale::parse(&self.compiler_options.locale) {
                self.locale = parsed;
            }
        });
        self.locale.clone()
    }

    pub fn resolved_project_reference_paths(&mut self) -> Vec<String> {
        self.resolved_project_reference_paths_once.get_or_init(|| {
            self.resolved_project_reference_paths = self
                .references
                .iter()
                .map(resolve_project_reference_path)
                .collect();
        });
        self.resolved_project_reference_paths.clone()
    }
}

impl ParsedBuildCommandLine {
    pub fn locale(&self) -> locale::Locale {
        let mut locale_result = locale::Locale::default();
        self.locale_once.get_or_init(|| {
            if let Some(parsed) = locale::Locale::parse(&self.compiler_options.locale) {
                locale_result = parsed;
            }
        });
        locale_result
    }
}

pub trait OutputPathsHost {
    fn common_source_directory(&self) -> String;
    fn content_mapper_extensions(&self) -> Vec<String>;
    fn get_current_directory(&self) -> String;
    fn use_case_sensitive_file_names(&self) -> bool;
}

impl OutputPathsHost for ParsedCommandLine {
    fn common_source_directory(&self) -> String {
        self.common_source_directory.clone()
    }

    fn content_mapper_extensions(&self) -> Vec<String> {
        self.content_mapper_extensions()
    }

    fn get_current_directory(&self) -> String {
        self.get_current_directory()
    }

    fn use_case_sensitive_file_names(&self) -> bool {
        self.use_case_sensitive_file_names()
    }
}

mod outputpaths {
    use super::*;

    use tsox_core::core::compiler_options::JsxEmit;
    use tsox_core::tspath::mig::m3i::{
        compare_paths, get_declaration_emit_extension_for_path, get_longest_extension_from_path,
        get_normalized_path_components, get_relative_path_from_directory,
    };

    pub fn get_common_source_directory(
        options: &CompilerOptions,
        files: impl Fn() -> Vec<String>,
        current_directory: &str,
        use_case_sensitive_file_names: bool,
        check_source_files_belong_to_path: impl Fn(&[String], &str) -> bool,
    ) -> String {
        let mut common_source_directory;
        if !options.root_dir.is_empty() {
            common_source_directory = options.root_dir.clone();
            check_source_files_belong_to_path(&files(), &options.root_dir);
        } else if !options.config_file_path.is_empty() {
            common_source_directory = tspath::get_directory_path(&options.config_file_path);
            check_source_files_belong_to_path(&files(), &common_source_directory);
        } else {
            common_source_directory = compute_common_source_directory_of_filenames(
                &files(),
                current_directory,
                use_case_sensitive_file_names,
            );
        }
        if !common_source_directory.is_empty() {
            common_source_directory =
                tspath::ensure_trailing_directory_separator(&common_source_directory);
        }
        common_source_directory
    }

    fn compute_common_source_directory_of_filenames(
        file_names: &[String],
        current_directory: &str,
        use_case_sensitive_file_names: bool,
    ) -> String {
        let mut common_path_components: Vec<String> = Vec::new();
        let mut have_common = false;
        for source_file in file_names {
            let mut source_path_components =
                get_normalized_path_components(source_file, current_directory);
            source_path_components.pop();
            if !have_common {
                common_path_components = source_path_components;
                have_common = true;
                continue;
            }
            let n = common_path_components.len().min(source_path_components.len());
            let mut mismatch_at = None;
            for i in 0..n {
                if tspath::get_canonical_file_name(
                    &common_path_components[i],
                    use_case_sensitive_file_names,
                ) != tspath::get_canonical_file_name(
                    &source_path_components[i],
                    use_case_sensitive_file_names,
                ) {
                    mismatch_at = Some(i);
                    break;
                }
            }
            if let Some(i) = mismatch_at {
                if i == 0 {
                    return String::new();
                }
                common_path_components.truncate(i);
            }
            if source_path_components.len() < common_path_components.len() {
                common_path_components.truncate(source_path_components.len());
            }
        }
        if common_path_components.is_empty() {
            return current_directory.to_string();
        }
        tspath::get_path_from_path_components(&common_path_components)
    }

    pub fn get_output_declaration_file_name_worker(
        input_file_name: &str,
        options: &CompilerOptions,
        host: &ParsedCommandLine,
    ) -> String {
        let mut dir = options.declaration_dir.clone();
        if dir.is_empty() {
            dir = options.out_dir.clone();
        }
        change_to_declaration_extension(
            &get_output_path_without_changing_extension(input_file_name, &dir, host),
            host,
        )
    }

    pub fn get_output_js_file_name(
        input_file_name: &str,
        options: &CompilerOptions,
        host: &ParsedCommandLine,
    ) -> String {
        if options.emit_declaration_only.is_true() || is_content_mapped_file_name(input_file_name, host)
        {
            return String::new();
        }
        let output_file_name = get_output_js_file_name_worker(input_file_name, options, host);
        if !tspath::file_extension_is(&output_file_name, tspath::EXTENSION_JSON)
            || compare_paths(
                input_file_name,
                &output_file_name,
                &ComparePathsOptions {
                    use_case_sensitive_file_names: host.use_case_sensitive_file_names(),
                    current_directory: host.get_current_directory(),
                },
            ) != 0
        {
            return output_file_name;
        }
        String::new()
    }

    fn is_content_mapped_file_name(file_name: &str, host: &ParsedCommandLine) -> bool {
        let extensions = host.content_mapper_extensions();
        let extension_refs: Vec<&str> = extensions.iter().map(String::as_str).collect();
        !get_longest_extension_from_path(
            file_name,
            &extension_refs,
            !host.use_case_sensitive_file_names(),
        )
        .is_empty()
    }

    fn get_output_js_file_name_worker(
        input_file_name: &str,
        options: &CompilerOptions,
        host: &ParsedCommandLine,
    ) -> String {
        tspath::change_extension(
            &get_output_path_without_changing_extension(input_file_name, &options.out_dir, host),
            &get_output_extension(input_file_name, options.jsx),
        )
    }

    pub fn get_output_extension(file_name: &str, jsx: JsxEmit) -> String {
        if tspath::file_extension_is(file_name, tspath::EXTENSION_JSON) {
            return tspath::EXTENSION_JSON.to_string();
        }
        if jsx == JsxEmit::Preserve
            && tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_JSX, tspath::EXTENSION_TSX])
        {
            return tspath::EXTENSION_JSX.to_string();
        }
        if tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_MTS, tspath::EXTENSION_MJS]) {
            return tspath::EXTENSION_MJS.to_string();
        }
        if tspath::file_extension_is_one_of(file_name, &[tspath::EXTENSION_CTS, tspath::EXTENSION_CJS]) {
            return tspath::EXTENSION_CJS.to_string();
        }
        tspath::EXTENSION_JS.to_string()
    }

    fn change_to_declaration_extension(path: &str, host: &ParsedCommandLine) -> String {
        let extensions = host.content_mapper_extensions();
        let extension_refs: Vec<&str> = extensions.iter().map(String::as_str).collect();
        let extension = get_longest_extension_from_path(path, &extension_refs, false);
        if !extension.is_empty() {
            return format!("{}.d{}.ts", tspath::remove_extension(path, &extension), extension);
        }
        let mut path_without_extension = tspath::remove_file_extension(path);
        if path_without_extension == path {
            let extension = tspath::get_any_extension_from_path(path, &[], false);
            if !extension.is_empty() {
                path_without_extension = tspath::remove_extension(path, &extension);
            }
        }
        format!(
            "{}{}",
            path_without_extension,
            get_declaration_emit_extension_for_path(path)
        )
    }

    fn get_output_path_without_changing_extension(
        input_file_name: &str,
        output_directory: &str,
        host: &ParsedCommandLine,
    ) -> String {
        if !output_directory.is_empty() {
            let relative = get_relative_path_from_directory(
                &<ParsedCommandLine as OutputPathsHost>::common_source_directory(host),
                input_file_name,
                &ComparePathsOptions {
                    use_case_sensitive_file_names: host.use_case_sensitive_file_names(),
                    current_directory: host.get_current_directory(),
                },
            );
            return tspath::resolve_path(output_directory, &[&relative]);
        }
        input_file_name.to_string()
    }

    pub fn get_source_map_file_path(js_file_path: &str, options: &CompilerOptions) -> String {
        if options.source_map.is_true() && !options.inline_source_map.is_true() {
            return format!("{}.map", js_file_path);
        }
        String::new()
    }

    pub fn get_build_info_file_name(
        options: &CompilerOptions,
        opts: &ComparePathsOptions,
    ) -> String {
        if !options.is_incremental() && !options.build.is_true() {
            return String::new();
        }
        if !options.ts_build_info_file.is_empty() {
            return options.ts_build_info_file.clone();
        }
        if options.config_file_path.is_empty() {
            return String::new();
        }
        let config_file_extension_less = tspath::remove_file_extension(&options.config_file_path);
        let build_info_extension_less = if !options.out_dir.is_empty() {
            if !options.root_dir.is_empty() {
                tspath::resolve_path(
                    &options.out_dir,
                    &[&get_relative_path_from_directory(
                        &options.root_dir,
                        &config_file_extension_less,
                        opts,
                    )],
                )
            } else {
                tspath::combine_paths(
                    &options.out_dir,
                    &[&tspath::get_base_file_name(&config_file_extension_less)],
                )
            }
        } else {
            config_file_extension_less
        };
        format!("{}{}", build_info_extension_less, tspath::EXTENSION_TS_BUILD_INFO)
    }
}
