#![allow(unused_imports)]

use super::m4v::{
    IncludeExplainingDiagnostic, ProcessingDiagnostic, ProcessingDiagnosticData,
    ProcessingDiagnosticKind,
};
use crate::compiler::{LibFile, Program};
use super::m4u_2::CheckerPool;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_core::tspath;
use tsox_frontend::ast::{Node, SourceFile};
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3b_2;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

fn contains_path(parent: &str, target: &str, options: &tspath::ComparePathsOptions) -> bool {
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

pub trait ProgramLike {
    fn options(&self) -> &tsox_core::core::compiler_options::CompilerOptions;
    fn get_source_file_by_name(&self, path: &str) -> Option<Arc<SourceFile>>;
    fn get_source_files(&self) -> Vec<Arc<SourceFile>>;
    fn get_config_file_parsing_diagnostics(&self) -> Vec<Arc<Diagnostic>>;
    fn get_syntactic_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Arc<Diagnostic>>;
    fn get_bind_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Arc<Diagnostic>>;
    fn get_program_diagnostics(&self) -> Vec<Arc<Diagnostic>>;
    fn get_global_diagnostics(&self) -> Vec<Arc<Diagnostic>>;
    fn get_semantic_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Arc<Diagnostic>>;
    fn get_declaration_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Arc<Diagnostic>>;
    fn get_suggestion_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Arc<Diagnostic>>;
    fn emit(&self, options: tsox_emit::emitter::EmitOptions) -> tsox_emit::emitter::EmitResult;
    fn common_source_directory(&self) -> String;
    fn is_source_file_default_library(&self, path: &str) -> bool;
    fn program(&self) -> &Program;
}

impl Program {
    pub fn get_global_diagnostics(&self) -> Vec<Arc<Diagnostic>> {
        if self.source_files.is_empty() {
            return Vec::new();
        }
        if let Some(compiler_checker_pool) = self.compiler_checker_pool.get() {
            return compiler_checker_pool.get_global_diagnostics();
        }
        Vec::new()
    }

    pub fn get_declaration_diagnostics(
        self: &Arc<Self>,
        source_file: Option<&Arc<SourceFile>>,
    ) -> Vec<Arc<Diagnostic>> {
        self.collect_diagnostics(source_file, true, &mut |file| {
            self.get_declaration_diagnostics_for_file(file)
        })
    }

    pub fn program(&self) -> &Program {
        self
    }

    pub fn get_source_file_meta_data(
        &self,
        path: &str,
    ) -> Option<tsox_frontend::ast::mig::x4ast::SourceFileMetaData> {
        self.source_file_meta_datas.get(path).cloned()
    }

    pub fn get_emit_syntax_for_usage_location(
        &self,
        source_file: &Arc<SourceFile>,
        location: &Arc<Node>,
    ) -> tsox_core::core::compiler_options_kinds::ResolutionMode {
        let meta = self
            .source_file_meta_datas
            .get(m3b_2::path(source_file).as_str())
            .cloned()
            .unwrap_or_default();
        super::m4v_2::get_emit_syntax_for_usage_location_worker(
            &source_file.file_name,
            &meta,
            location,
            &self
                .project_reference_file_mapper
                .get_compiler_options_for_file(source_file),
        )
    }

    pub fn get_implied_node_format_for_emit(
        &self,
        source_file: &Arc<SourceFile>,
    ) -> tsox_core::core::compiler_options_kinds::ResolutionMode {
        tsox_frontend::ast::mig::x4ast::get_implied_node_format_for_emit_worker(
            &source_file.file_name,
            self.project_reference_file_mapper
                .get_compiler_options_for_file(source_file)
                .get_emit_module_kind(),
            self.get_source_file_meta_data(m3b_2::path(source_file).as_str())
                .unwrap_or_default(),
        )
    }

    pub fn get_mode_for_usage_location(
        &self,
        source_file: &Arc<SourceFile>,
        location: &Arc<Node>,
    ) -> tsox_core::core::compiler_options_kinds::ResolutionMode {
        let meta = self
            .source_file_meta_datas
            .get(m3b_2::path(source_file).as_str())
            .cloned()
            .unwrap_or_default();
        super::m4v_2::get_mode_for_usage_location(
            &source_file.file_name,
            &meta,
            location,
            &self
                .project_reference_file_mapper
                .get_compiler_options_for_file(source_file),
        )
    }

    pub fn get_default_resolution_mode_for_file(
        &self,
        source_file: &Arc<SourceFile>,
    ) -> tsox_core::core::compiler_options_kinds::ResolutionMode {
        let meta = self
            .source_file_meta_datas
            .get(m3b_2::path(source_file).as_str())
            .cloned()
            .unwrap_or_default();
        super::m4v_2::get_default_resolution_mode_for_file(
            &source_file.file_name,
            &meta,
            &self
                .project_reference_file_mapper
                .get_compiler_options_for_file(source_file),
        )
    }

    pub fn is_global_typings_file(&self, file_name: &str) -> bool {
        if !tspath::is_declaration_file_name(file_name) {
            return false;
        }
        contains_path(
            self.get_global_typings_cache_location(),
            file_name,
            &self.compare_paths_options,
        )
    }

    pub fn get_default_lib_file(&self, path: &str) -> Option<LibFile> {
        self.lib_files.get(path).cloned()
    }

    pub fn check_source_files_belong_to_path(
        &mut self,
        source_files: &[String],
        root_directory: &str,
    ) -> bool {
        let mut all_files_belong_to_path = true;
        for file in source_files {
            let absolute_sourceFilePath = tspath::get_canonical_file_name(
                &tspath::get_normalized_absolute_path(file, self.get_current_directory()),
                self.use_case_sensitive_file_names(),
            );
            if !contains_path(root_directory, file, &self.compare_paths_options) {
                if let Some(include_processor) = self.include_processor.as_deref_mut() {
                    include_processor.add_processing_diagnostics(vec![Arc::new(
                        ProcessingDiagnostic {
                            kind: ProcessingDiagnosticKind::ExplainingFileInclude,
                            data: ProcessingDiagnosticData::IncludeExplainingDiagnostic(
                                Box::new(IncludeExplainingDiagnostic {
                                    file: tspath::Path(absolute_sourceFilePath),
                                    diagnostic_reason: std::ptr::null(),
                                    diagnostic_reason_opt: None,
                                    message:
                                        tsox_core::diagnostics::messages_generated::FILE_0_IS_NOT_UNDER_ROOTDIR_1_ROOTDIR_IS_EXPECTED_TO_CONTAIN_ALL_SOURCE_FILES,
                                    args: vec![file.clone(), root_directory.to_string()],
                                }),
                            ),
                        },
                    )]);
                }
                all_files_belong_to_path = false;
            }
        }
        all_files_belong_to_path
    }

    pub fn get_source_file_for_resolved_module(
        &self,
        file_name: &str,
    ) -> Option<Arc<SourceFile>> {
        let file = self.get_source_file(file_name);
        if file.is_none() {
            let filename = self.get_parse_file_redirect(file_name);
            if !filename.is_empty() {
                return self.get_source_file(&filename);
            }
        }
        file
    }

    pub fn files_by_path(&self) -> &HashMap<String, Arc<SourceFile>> {
        &self.files_by_path
    }

    pub fn has_same_file_names(&self, other: &Program) -> bool {
        maps_equal_by(&self.files_by_path, &other.files_by_path, |a, b| {
            a.file_name == b.file_name
        }) && maps_equal_by(
            &self.redirect_files_by_path,
            &other.redirect_files_by_path,
            |a, b| a.file_name() == b.file_name(),
        )
    }

    pub fn is_emit_blocked(&self, emit_file_name: &str) -> bool {
        self.has_emit_blocking_diagnostics
            .contains(&self.to_path(emit_file_name).0)
    }
}

fn maps_equal_by<K: std::hash::Hash + Eq, V, F: Fn(&V, &V) -> bool>(
    a: &HashMap<K, V>,
    b: &HashMap<K, V>,
    eq: F,
) -> bool {
    a.len() == b.len()
        && a.iter().all(|(k, va)| match b.get(k) {
            Some(vb) => eq(va, vb),
            None => false,
        })
}

pub fn filter_no_emit_semantic_diagnostics(
    diagnostics: Vec<Arc<Diagnostic>>,
    options: &tsox_core::core::compiler_options::CompilerOptions,
) -> Vec<Arc<Diagnostic>> {
    if !options.no_emit.is_true() {
        return diagnostics;
    }
    diagnostics
        .into_iter()
        .filter(|d| !d.skipped_on_no_emit())
        .collect()
}

pub fn sort_and_deduplicate_diagnostics(
    diagnostics: Vec<Arc<Diagnostic>>,
) -> Vec<Arc<Diagnostic>> {
    let mut diagnostics = diagnostics;
    diagnostics.sort_by(|a, b| tsox_frontend::ast::mig::m3d_2::compare_diagnostics(a, b));
    compact_and_merge_related_infos(diagnostics)
}

pub fn compact_and_merge_related_infos(
    mut diagnostics: Vec<Arc<Diagnostic>>,
) -> Vec<Arc<Diagnostic>> {
    if diagnostics.len() < 2 {
        return diagnostics;
    }
    let mut result: Vec<Arc<Diagnostic>> = Vec::new();
    let mut i = 0;
    while i < diagnostics.len() {
        let d = diagnostics[i].clone();
        let mut n = 1;
        while i + n < diagnostics.len()
            && tsox_frontend::ast::mig::m3d_2::equal_diagnostics_no_related_info(
                &d,
                &diagnostics[i + n],
            )
        {
            n += 1;
        }
        let d = if n > 1 {
            let mut related_infos: Vec<Diagnostic> = Vec::new();
            for k in 0..n {
                related_infos.extend(diagnostics[i + k].related_information().iter().cloned());
            }
            if !related_infos.is_empty() {
                related_infos.sort_by(|a, b| {
                    tsox_frontend::ast::mig::m3d_2::compare_diagnostics(a, b)
                });
                related_infos.dedup_by(|a, b| {
                    tsox_frontend::ast::mig::m3d_2::equal_diagnostics(a, b)
                });
                let mut merged = (*d).clone();
                merged.set_related_info(related_infos);
                Arc::new(merged)
            } else {
                d
            }
        } else {
            d
        };
        result.push(d);
        i += n;
    }
    result
}

pub fn combine_emit_results(
    results: Vec<tsox_emit::emitter::EmitResult>,
) -> tsox_emit::emitter::EmitResult {
    let mut result = tsox_emit::emitter::EmitResult::default();
    for emit_result in results {
        if emit_result.emit_skipped {
            result.emit_skipped = true;
        }
        result.diagnostics.extend(emit_result.diagnostics);
        result.emitted_files.extend(emit_result.emitted_files);
    }
    result
}
