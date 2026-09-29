#![allow(unused_imports)]

use crate::compiler::Program;
use super::m4v::source_file_may_be_emitted;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_frontend::ast::SourceFile;
use tsox_core::tspath;
use tsox_core::tspath::mig::m3i;
use tsox_frontend::ast::mig::m3b_2;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModeAwareCacheKey {
    pub name: String,
    pub mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
}

impl ModeAwareCacheKey {
    pub fn new(
        name: String,
        mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
    ) -> Self {
        Self { name, mode }
    }
}

pub type ModeAwareCache<T> = HashMap<ModeAwareCacheKey, T>;

#[derive(Default)]
pub struct PackageNamesInfo {
    pub resolved: HashSet<String>,
    pub unresolved: HashSet<String>,
    pub deep_import_packages: HashSet<String>,
}

pub struct JsxRuntimeImportSpecifier {
    pub module_reference: String,
    pub specifier: Option<Arc<tsox_frontend::ast::Node>>,
}

pub struct LazyValue<T> {
    value: std::sync::OnceLock<T>,
}

impl<T> Default for LazyValue<T> {
    fn default() -> Self {
        Self {
            value: std::sync::OnceLock::default(),
        }
    }
}

impl<T> LazyValue<T> {
    pub fn get_value(&self, compute: impl FnOnce() -> T) -> &T {
        self.value.get_or_init(compute)
    }
}

impl super::m4v::SourceFileMayBeEmittedHost for crate::compiler::Program {
    fn options(&self) -> &tsox_core::core::compiler_options::CompilerOptions {
        crate::compiler::Program::options(self)
    }
    fn get_project_reference_from_source(
        &self,
        path: &str,
    ) -> Option<tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference> {
        crate::compiler::Program::get_project_reference_from_source(self, path)
    }
    fn is_source_file_from_external_library(&self, file: &Arc<SourceFile>) -> bool {
        crate::compiler::Program::is_source_file_from_external_library(self, file)
    }
    fn is_source_file_from_project_reference(&self, file: &Arc<SourceFile>) -> bool {
        let path = m3b_2::path(file);
        crate::compiler::Program::is_source_from_project_reference(self, &path)
    }
    fn get_current_directory(&self) -> String {
        crate::compiler::Program::get_current_directory(self).to_string()
    }
    fn use_case_sensitive_file_names(&self) -> bool {
        crate::compiler::Program::use_case_sensitive_file_names(self)
    }
    fn source_files(&self) -> Vec<Arc<SourceFile>> {
        crate::compiler::Program::source_files(self).to_vec()
    }
}

impl Program {
    pub fn explain_files(&self, w: &mut dyn std::io::Write, locale: tsox_core::locale::Locale) {
        let to_relative_file_name = |file_name: &str| -> String {
            m3i::get_relative_path_from_directory(
                self.get_current_directory(),
                file_name,
                &self.compare_paths_options,
            )
        };
        let files_explained = 0usize;
        let explain_file = |w: &mut dyn std::io::Write, file: &dyn super::m4x_2::HasFileName| {
            let _ = writeln!(w, "{}", to_relative_file_name(&file.file_name()));
            let file_path = tspath::Path(file.path());
            let include_processor = self.include_processor.as_deref();
            if let Some(reasons) =
                include_processor.and_then(|ip| ip.file_include_reasons.get(&file_path))
            {
                for reason in reasons {
                    let _ = writeln!(
                        w,
                        "  {}",
                        reason
                            .to_diagnostic(self, true)
                            .unwrap()
                            .localize(locale.clone())
                    );
                }
            }
            if let Some(include_processor) = include_processor {
                for diag in include_processor.explain_redirect_and_implied_format(
                    self,
                    &file_path,
                    &to_relative_file_name,
                ) {
                    let _ = writeln!(w, "  {}", diag.localize(locale.clone()));
                }
            }
        };

        let mut redirect_files: Vec<&crate::mig::m4v_2::RedirectsFile> =
            self.redirect_files_by_path.values().collect();
        redirect_files.sort_by_key(|f| f.index);

        let files = self.get_source_files();
        let mut source_file_index = 0usize;
        let mut explained = files_explained;
        let mut explain_source_files = |w: &mut dyn std::io::Write, end_index: usize| {
            while explained < end_index {
                explain_file(w, &files[source_file_index]);
                source_file_index += 1;
                explained += 1;
            }
        };

        for redirect_file in &redirect_files {
            explain_source_files(w, redirect_file.index);
            explain_file(w, *redirect_file);
        }

        explain_source_files(w, files.len() + redirect_files.len());
    }

    pub fn get_lib_file_from_reference(
        &self,
        r: &tsox_frontend::ast::node_source_file::FileReference,
    ) -> Option<Arc<SourceFile>> {
        let path = tsox_tsoptions::mig::m5h_2::get_lib_file_name(&r.file_name)?;
        self.files_by_path.get(path.as_str()).cloned()
    }

    pub fn get_resolved_type_reference_directive_from_type_reference_directive(
        &self,
        type_ref: &tsox_frontend::ast::node_source_file::FileReference,
        source_file: &Arc<SourceFile>,
    ) -> Option<tsox_tsoptions::module::ResolvedTypeReferenceDirective> {
        let file_path = m3b_2::path(source_file);
        if let Some(resolutions) = self.type_resolutions_in_file.get(file_path.as_str()) {
            let key = ModeAwareCacheKey {
                name: type_ref.file_name.clone(),
                mode: self.get_mode_for_type_reference_directive_in_file(type_ref, source_file),
            };
            if let Some(resolved) = resolutions.get(&key) {
                return Some(resolved.clone());
            }
        }
        None
    }

    pub fn get_resolved_type_reference_directives(
        &self,
    ) -> &HashMap<String, ModeAwareCache<tsox_tsoptions::module::ResolvedTypeReferenceDirective>>
    {
        &self.type_resolutions_in_file
    }

    pub fn is_source_file_from_external_library(&self, file: &Arc<SourceFile>) -> bool {
        self.source_files_found_searching_node_modules
            .contains(m3b_2::path(file).as_str())
    }

    pub fn get_jsx_runtime_import_specifier(
        &self,
        path: &str,
    ) -> (String, Option<Arc<tsox_frontend::ast::Node>>) {
        if let Some(result) = self.jsx_runtime_import_specifiers.get(path) {
            return (result.module_reference.clone(), result.specifier.clone());
        }
        (String::new(), None)
    }

    pub fn get_import_helpers_import_specifier(
        &self,
        path: &str,
    ) -> Option<Arc<tsox_frontend::ast::Node>> {
        self.import_helpers_import_specifiers.get(path).cloned()
    }

    pub fn source_file_may_be_emitted(
        &self,
        source_file: &Arc<SourceFile>,
        force_dts_emit: bool,
    ) -> bool {
        source_file_may_be_emitted(source_file, self, force_dts_emit, false)
    }

    pub fn resolved_package_names(&self) -> &HashSet<String> {
        &self.collect_package_names().resolved
    }

    pub fn unresolved_package_names(&self) -> &HashSet<String> {
        &self.collect_package_names().unresolved
    }

    pub fn deep_import_package_names(&self) -> &HashSet<String> {
        &self.collect_package_names().deep_import_packages
    }

    pub fn collect_package_names(&self) -> &PackageNamesInfo {
        self.package_names.get_value(|| {
            let mut package_names = PackageNamesInfo::default();
            for file in &self.source_files {
                let file_path = m3b_2::path(file);
                if self.is_source_file_default_library(file_path.as_str())
                    || self.is_source_file_from_external_library(file)
                    || file.file_name.contains("/node_modules/")
                {
                    continue;
                }
                for imp in &file.imports {
                    if tspath::is_external_module_name_relative(imp.text()) {
                        continue;
                    }
                    let mut handled = false;
                    if let Some(resolved_modules) = self.resolved_modules.get(file_path.as_str()) {
                        let key = ModeAwareCacheKey {
                            name: imp.text().to_string(),
                            mode: self.get_mode_for_usage_location(file, imp),
                        };
                        if let Some(resolved_module) = resolved_modules.get(&key) {
                            if resolved_module.is_resolved() {
                                if !resolved_module.is_external_library_import {
                                    handled = true;
                                } else {
                                    let mut name = resolved_module
                                        .package_id
                                        .as_ref()
                                        .map(|p| p.name.clone())
                                        .unwrap_or_default();
                                    if name.is_empty() {
                                        if let Some(package_scope) =
                                            super::m4w::get_package_scope_for_path(
                                                &self.resolver,
                                                &self.typings_location,
                                                &resolved_module.resolved_file_name,
                                            )
                                            .filter(|scope| scope.exists())
                                        {
                                            if let Some(scope_name) = package_scope
                                                .get_contents()
                                                .and_then(|contents| {
                                                    contents.header_fields.name.get_value()
                                                })
                                            {
                                                name = scope_name.clone();
                                            }
                                        }
                                    }
                                    if name.is_empty() {
                                        name = tsox_tsoptions::modulespecifiers::
                                            get_package_name_from_directory(
                                                &resolved_module.resolved_file_name,
                                            );
                                    }
                                    if !name.is_empty() {
                                        package_names.resolved.insert(name.clone());
                                        let (_, rest) =
                                            tsox_tsoptions::module::parse_package_name(imp.text());
                                        if !rest.is_empty() {
                                            if let Some(package_scope) =
                                                super::m4w::get_package_scope_for_path(
                                                    &self.resolver,
                                                    &self.typings_location,
                                                    &resolved_module.resolved_file_name,
                                                )
                                            {
                                                let has_exports = package_scope
                                                    .exists()
                                                    && package_scope
                                                        .get_contents()
                                                        .map(|contents| {
                                                            contents
                                                                .path_fields
                                                                .exports
                                                                .json_value
                                                                .is_present()
                                                        })
                                                        .unwrap_or(true);
                                                if !has_exports {
                                                    package_names.deep_import_packages.insert(
                                                        tsox_tsoptions::module::
                                                            get_package_name_from_types_package_name(
                                                                &name,
                                                            ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    handled = true;
                                }
                            }
                        }
                    }
                    if !handled {
                        package_names.unresolved.insert(imp.text().to_string());
                    }
                }
            }
            package_names
        })
    }

    pub fn is_lib_file(&self, source_file: &Arc<SourceFile>) -> bool {
        self.lib_files
            .contains_key(m3b_2::path(source_file).as_str())
    }

    pub fn has_ts_file(&self) -> bool {
        *self.has_ts_file_once.get_value(|| {
            self.source_files
                .iter()
                .any(|file| m3i::has_implementation_ts_file_extension(&file.file_name))
        })
    }
}
