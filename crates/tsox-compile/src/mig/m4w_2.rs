#![allow(unused_imports)]
use crate::compiler::{Program, ProgramOptions};
use super::m4u_2::CheckerPool;
use std::sync::Arc;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_core::tspath;
use tsox_core::tspath::mig::m3i;
use tsox_frontend::ast::mig::m3b_2;
use super::m4w_7::{
    equal_check_js_directives, equal_file_references, equal_module_augmentation_names,
    equal_module_specifiers,
};

const SUPPORTED_TS_EXTENSIONS_WITH_JSON_FLAT: &[&str] = &[
    ".ts", ".tsx", ".d.ts", ".cts", ".d.cts", ".mts", ".d.mts", ".json",
];

fn zip_all_eq<T, F: Fn(&T, &T) -> bool>(a: &[T], b: &[T], eq: F) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| eq(x, y))
}

pub fn source_file_content_mapper_identity(file: &Arc<SourceFile>) -> String {
    crate::mig::m3l_cm_2::content_mapper_source_file_info(&file.file_name)
        .map(|info| info.content_mapper)
        .unwrap_or_default()
}
impl Program {
    pub fn update_program(
        &self,
        changed_file_path: &str,
        new_host: Arc<dyn crate::compiler::CompilerHost>,
        create_checker_pool: Option<crate::compiler::CreateCheckerPoolFn>,
    ) -> (Arc<Program>, Option<Arc<SourceFile>>, bool) {
        let (result, new_file, reused) = self.reuse_program(
            changed_file_path,
            new_host.clone(),
            create_checker_pool.clone(),
        );
        if reused {
            (result.unwrap(), new_file, true)
        } else {
            let mut new_opts = self.opts.clone();
            new_opts.host = new_host;
            if let Some(create_checker_pool) = create_checker_pool {
                new_opts.create_checker_pool = Some(create_checker_pool);
            }
            (Program::new(new_opts), new_file, false)
        }
    }
    pub fn get_checker_pool(&self) -> Arc<dyn CheckerPool> {
        self.checker_pool
            .get()
            .cloned()
            .expect("checker pool is initialized after program construction")
    }
    pub fn can_replace_file_in_program(
        &self,
        file1: &Arc<SourceFile>,
        file2: &Arc<SourceFile>,
    ) -> bool {
        m3b_2::parse_options(&file2) == m3b_2::parse_options(&file1)
            && file1.script_kind == file2.script_kind
            && tsox_frontend::ast::is_external_or_common_js_module(file1)
                == tsox_frontend::ast::is_external_or_common_js_module(file2)
            && file1.uses_uri_style_node_core_modules == file2.uses_uri_style_node_core_modules
            && zip_all_eq(&file1.imports, &file2.imports, |n1, n2| {
                equal_module_specifiers(n1, n2)
                    && self.get_mode_for_usage_location(file1, n1)
                        == self.get_mode_for_usage_location(file2, n2)
            })
            && zip_all_eq(
                &file1.module_augmentations,
                &file2.module_augmentations,
                equal_module_augmentation_names,
            )
            && file1.ambient_module_names == file2.ambient_module_names
            && zip_all_eq(&file1.referenced_files, &file2.referenced_files, equal_file_references)
            && zip_all_eq(
                &file1.type_reference_directives,
                &file2.type_reference_directives,
                equal_file_references,
            )
            && zip_all_eq(
                &file1.lib_reference_directives,
                &file2.lib_reference_directives,
                equal_file_references,
            )
    }
    pub fn get_content_mapper(
        &self,
        file: &Arc<SourceFile>,
    ) -> Option<&tsox_tsoptions::mig::m5h_3::ContentMapper> {
        let identity = source_file_content_mapper_identity(file);
        if identity.is_empty() {
            return None;
        }
        let mapper = self
            .opts
            .config
            .get_content_mapper_for_file_name(&file.file_name)?;
        if mapper.identity() == identity {
            return Some(mapper);
        }
        None
    }
    pub fn content_mapper_extensions(&self) -> Vec<String> {
        self.opts.config.content_mapper_extensions()
    }
    pub fn command_line(&self) -> &ParsedCommandLine {
        &self.opts.config
    }
    pub fn tracing(&self) -> Option<&Arc<tsox_core::tracing::mig::x11a::Tracing<'static>>> {
        self.tracing.as_ref()
    }
    pub fn get_config_file_parsing_diagnostics(&self) -> Vec<Arc<Diagnostic>> {
        self.opts
            .config
            .get_config_file_parsing_diagnostics()
            .into_iter()
            .map(Arc::new)
            .collect()
    }
    pub fn get_unresolved_imports(&self) -> std::collections::HashSet<String> {
        self.unresolved_imports
            .get_value(|| self.extract_unresolved_imports())
            .clone()
    }
    pub fn extract_unresolved_imports(&self) -> std::collections::HashSet<String> {
        let mut unresolved_set = std::collections::HashSet::new();
        for source_file in &self.source_files {
            let unresolved_imports = self.extract_unresolved_imports_from_source_file(source_file);
            for imp in unresolved_imports {
                unresolved_set.insert(imp);
            }
        }
        unresolved_set
    }
    pub fn extract_unresolved_imports_from_source_file(
        &self,
        file: &Arc<SourceFile>,
    ) -> Vec<String> {
        let mut unresolved_imports = Vec::new();
        let file_path = m3b_2::path(file);
        if let Some(resolved_modules) = self.resolved_modules.get(file_path.as_str()) {
            for (cache_key, resolution) in resolved_modules {
                let resolved = resolution.is_resolved();
                if (!resolved
                    || !m3i::extension_is_one_of(
                        &resolution.extension,
                        SUPPORTED_TS_EXTENSIONS_WITH_JSON_FLAT,
                    ))
                    && !tspath::is_external_module_name_relative(&cache_key.name)
                {
                    unresolved_imports.push(cache_key.name.clone());
                }
            }
        }
        unresolved_imports
    }
    pub fn get_type_checker(
        &self,
    ) -> std::sync::MutexGuard<'_, tsox_checker::checker::Checker> {
        if let Some(compiler_checker_pool) = self.compiler_checker_pool.get() {
            return compiler_checker_pool.get_checker(None);
        }
        self.checker_pool
            .get()
            .expect("checker pool is initialized after program construction")
            .get_checker(None)
    }
    pub fn for_each_checker_parallel(
        &self,
        cb: &mut (dyn FnMut(usize, &mut tsox_checker::checker::Checker) + Send),
    ) {
        if let Some(compiler_checker_pool) = self.compiler_checker_pool.get() {
            compiler_checker_pool.for_each_checker_parallel(cb);
        }
    }
    pub fn get_type_checker_for_file(
        &self,
        file: &Arc<SourceFile>,
    ) -> std::sync::MutexGuard<'_, tsox_checker::checker::Checker> {
        if let Some(compiler_checker_pool) = self.compiler_checker_pool.get() {
            return compiler_checker_pool.get_checker(Some(file));
        }
        self.checker_pool
            .get()
            .expect("checker pool is initialized after program construction")
            .get_checker(Some(file))
    }
    pub fn get_type_checker_for_file_exclusive(
        &self,
        file: &Arc<SourceFile>,
    ) -> std::sync::MutexGuard<'_, tsox_checker::checker::Checker> {
        if let Some(compiler_checker_pool) = self.compiler_checker_pool.get() {
            return compiler_checker_pool.get_checker(Some(file));
        }
        self.checker_pool
            .get()
            .expect("checker pool is initialized after program construction")
            .get_checker(Some(file))
    }
}
