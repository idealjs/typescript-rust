#![allow(unused_imports)]

use crate::compiler::{CompilerHost, Program};
use super::m4u_2::CheckerPool;
use super::m4v_3::update_file_include_processor;
use std::sync::Arc;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::node_source_file::FileReference;
use tsox_core::tspath;

#[derive(Clone, Debug, PartialEq)]
pub struct CheckJsDirective {
    pub enabled: bool,
    pub range: tsox_core::core::text::TextRange,
}

impl Program {
    pub fn reuse_program(
        &self,
        changed_file_path: &str,
        new_host: Arc<dyn crate::compiler::CompilerHost>,
        create_checker_pool: Option<crate::compiler::CreateCheckerPoolFn>,
    ) -> (Option<Arc<Program>>, Option<Arc<SourceFile>>, bool) { ::tsox_core::fntrace::enter("reuse_program"); 
        let mut new_opts = self.opts.clone();
        new_opts.host = new_host;
        if let Some(create_checker_pool) = create_checker_pool {
            new_opts.create_checker_pool = Some(create_checker_pool);
        }

        let old_file = match self.files_by_path.get(changed_file_path) {
            Some(f) => f.clone(),
            None => return (None, None, false),
        };
        let parse_opts = tsox_frontend::ast::mig::m3b_2::parse_options(old_file.as_ref());
        let mut new_file: Option<Arc<SourceFile>> = None;
        let mut old_supplemental_files: Vec<Arc<SourceFile>> = Vec::new();
        let mut new_supplemental_files: Vec<Arc<SourceFile>> = Vec::new();
        if !super::m4w_2::source_file_content_mapper_identity(&old_file).is_empty() {
            let mapper = new_opts
                .config
                .get_content_mapper_for_file_name(&old_file.file_name)
                .expect("content mapper for content-mapped file");
            let mapper = super::m3l_cm::Mapper {
                definition: super::m3l_cm::Definition {
                    package: mapper.definition.package.clone(),
                    extensions: mapper.definition.extensions.clone(),
                    options: Default::default(),
                },
                ..Default::default()
            };
            let files = match new_opts
                .host
                .get_content_mapped_source_files(&parse_opts, &mapper)
            {
                Ok(files) => files,
                Err(_) => return (None, None, false),
            };
            new_file = files.canonical.clone();
            old_supplemental_files = old_file.supplemental_source_files();
            new_supplemental_files = files.supplemental.clone();
        } else {
            new_file = new_opts.host.get_source_file(&parse_opts);
        }

        let in_redirect_files = self.redirect_files_by_path.contains_key(changed_file_path);
        let is_redirect_target = self.redirect_targets_map.contains_key(changed_file_path);
        if in_redirect_files || is_redirect_target {
            return (None, new_file, false);
        }

        let new_file_ref = match &new_file {
            Some(f) => f,
            None => return (None, new_file, false),
        };
        if !self.can_replace_file_in_program(&old_file, new_file_ref) {
            return (None, new_file, false);
        }
        if self
            .import_helpers_import_specifiers
            .get(old_file.file_name.as_str())
            .is_some()
            || self.needs_import_helpers_import_specifier(new_file_ref)
        {
            return (None, new_file, false);
        }
        if self
            .jsx_runtime_import_specifiers
            .get(old_file.file_name.as_str())
            .is_some()
            || !self.jsx_runtime_import_specifier(new_file_ref).is_empty()
        {
            return (None, new_file, false);
        }
        if old_supplemental_files.len() != new_supplemental_files.len() {
            return (None, new_file, false);
        }
        for (i, old_supplemental) in old_supplemental_files.iter().enumerate() {
            let new_supplemental = &new_supplemental_files[i];
            if old_supplemental.file_name != new_supplemental.file_name
                || !self.can_replace_file_in_program(old_supplemental, new_supplemental)
            {
                return (None, new_file, false);
            }
            if self
                .import_helpers_import_specifiers
                .get(old_supplemental.file_name.as_str())
                .is_some()
                || self.needs_import_helpers_import_specifier(new_supplemental)
            {
                return (None, new_file, false);
            }
            if self
                .jsx_runtime_import_specifiers
                .get(old_supplemental.file_name.as_str())
                .is_some()
                || !self.jsx_runtime_import_specifier(new_supplemental).is_empty()
            {
                return (None, new_file, false);
            }
        }
        let mut result = Program {
            opts: new_opts,
            options: self.options.clone(),
            compare_paths_options: self.compare_paths_options.clone(),
            uses_uri_style_node_core_modules: self.uses_uri_style_node_core_modules,
            program_diagnostics: self.program_diagnostics.clone(),
            has_emit_blocking_diagnostics: self.has_emit_blocking_diagnostics.clone(),
            content_mapper_option_diagnostics: self.content_mapper_option_diagnostics.clone(),
            files: self.files.clone(),
            files_by_path: self.files_by_path.clone(),
            source_files: self.source_files.clone(),
            source_files_by_name: self.source_files_by_name.clone(),
            default_library_file_names: self.default_library_file_names.clone(),
            diagnostics: self.diagnostics.clone(),
            host: Arc::clone(&self.host),
            config_file_name: self.config_file_name.clone(),
            symbol_map: crate::compiler::NodeSymbolMap::default(),
            resolver: Arc::clone(&self.resolver),
            checker_pool: std::sync::OnceLock::new(),
            compiler_checker_pool: std::sync::OnceLock::new(),
            project_reference_file_mapper: Arc::clone(&self.project_reference_file_mapper),
            missing_files: self.missing_files.clone(),
            resolved_modules: self.resolved_modules.clone(),
            type_resolutions_in_file: self.type_resolutions_in_file.clone(),
            source_file_meta_datas: self.source_file_meta_datas.clone(),
            jsx_runtime_import_specifiers: Default::default(),
            import_helpers_import_specifiers: self.import_helpers_import_specifiers.clone(),
            lib_files: self.lib_files.clone(),
            source_files_found_searching_node_modules: self
                .source_files_found_searching_node_modules
                .clone(),
            include_processor: None,
            output_file_to_project_reference_source: self
                .output_file_to_project_reference_source
                .clone(),
            redirect_targets_map: self.redirect_targets_map.clone(),
            redirect_files_by_path: Default::default(),
            content_mapper_diagnostics: self.content_mapper_diagnostics.clone(),
            unresolved_imports: Default::default(),
            known_symlinks: Default::default(),
            package_names: Default::default(),
            has_ts_file_once: Default::default(),
            typings_location: self.typings_location.clone(),
            use_source_of_project_reference: self.use_source_of_project_reference,
            tracing: self.tracing.clone(),
            finished_processing: self.finished_processing,
        };
        let index = result
            .files
            .iter()
            .position(|file| file.file_name == new_file_ref.file_name)
            .unwrap();
        result.files = result.files.clone();
        result.files[index] = new_file_ref.clone();
        result.files_by_path = result.files_by_path.clone();
        result
            .files_by_path
            .insert(new_file_ref.file_name.clone(), new_file_ref.clone());
        if !old_supplemental_files.is_empty() {
            for (i, old_supplemental) in old_supplemental_files.iter().enumerate() {
                let new_supplemental = new_supplemental_files[i].clone();
                let supplemental_index = result
                    .files
                    .iter()
                    .position(|file| Arc::ptr_eq(file, old_supplemental))
                    .unwrap();
                result.files[supplemental_index] = new_supplemental.clone();
                result
                .files_by_path
                .insert(new_supplemental.file_name.clone(), new_supplemental);
            }
        }
        update_file_include_processor(&mut result);
        let result = Arc::new(result);
        result.init_checker_pool();
        (Some(result), new_file, true)
    }
}

pub fn equal_module_specifiers(
    n1: &Arc<tsox_frontend::ast::Node>,
    n2: &Arc<tsox_frontend::ast::Node>,
) -> bool { ::tsox_core::fntrace::enter("equal_module_specifiers"); 
    n1.kind == n2.kind
        && (!tsox_frontend::ast::is_string_literal(n1) || n1.text() == n2.text())
}

pub fn equal_module_augmentation_names(
    n1: &Arc<tsox_frontend::ast::Node>,
    n2: &Arc<tsox_frontend::ast::Node>,
) -> bool { ::tsox_core::fntrace::enter("equal_module_augmentation_names"); 
    n1.kind == n2.kind && n1.text() == n2.text()
}

pub fn equal_file_references(f1: &FileReference, f2: &FileReference) -> bool { ::tsox_core::fntrace::enter("equal_file_references"); 
    f1.file_name == f2.file_name
        && f1.resolution_mode == f2.resolution_mode
        && f1.preserve == f2.preserve
}

pub fn equal_check_js_directives(
    d1: &Option<CheckJsDirective>,
    d2: &Option<CheckJsDirective>,
) -> bool { ::tsox_core::fntrace::enter("equal_check_js_directives"); 
    match (d1, d2) {
        (None, None) => true,
        (Some(d1), Some(d2)) => d1.enabled == d2.enabled,
        _ => false,
    }
}

fn zip_all_eq<T, F: Fn(&T, &T) -> bool>(a: &[T], b: &[T], eq: F) -> bool { ::tsox_core::fntrace::enter("zip_all_eq"); 
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| eq(x, y))
}

use std::sync::Arc as _Arc;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_core::diagnostics::Category;
use tsox_frontend::ast::SourceFile as _SF;


