use std::sync::Arc;

use tsox_core::core::text::TextPos;
use tsox_frontend::ast::{ModifierFlags, Node, SourceFile};
use tsox_frontend::ast::node_source_file::FileReference;

use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options_kinds::{ModuleKind, ResolutionMode};
use tsox_core::tspath::{self, ComparePathsOptions};

use crate::compiler::Program;
use tsox_tsoptions::module::ResolvedModule;
use super::m4v::SourceFileMayBeEmittedHost;

pub struct SendSyncEmitResolver(Arc<tsox_checker::checker::mig::m2d::EmitResolver>);
unsafe impl Send for SendSyncEmitResolver {}
unsafe impl Sync for SendSyncEmitResolver {}

impl std::ops::Deref for SendSyncEmitResolver {
    type Target = tsox_checker::checker::mig::m2d::EmitResolver;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SendSyncEmitResolver {
    pub fn new(resolver: Arc<tsox_checker::checker::mig::m2d::EmitResolver>) -> Self {
        Self(resolver)
    }

    pub fn get(&self) -> Arc<tsox_checker::checker::mig::m2d::EmitResolver> {
        Arc::clone(&self.0)
    }
}

pub trait EmitHost: Send + Sync {
    fn options(&self) -> &CompilerOptions;
    fn source_files(&self) -> &[Arc<SourceFile>];
    fn use_case_sensitive_file_names(&self) -> bool;
    fn get_current_directory(&self) -> String;
    fn common_source_directory(&self) -> String;
    fn is_emit_blocked(&self, file: &str) -> bool;
}

pub struct EmitHostImpl {
    pub program: Arc<Program>,
    pub emit_resolver: SendSyncEmitResolver,
}

unsafe impl Send for EmitHostImpl {}
unsafe impl Sync for EmitHostImpl {}

pub fn new_emit_host(
    program: Arc<Program>,
    file: &Arc<SourceFile>,
) -> (EmitHostImpl, impl FnOnce()) {
    let emit_resolver = SendSyncEmitResolver::new({
        let mut checker = program.get_type_checker_for_file(file);
        checker.get_emit_resolver()
    });
    (
        EmitHostImpl {
            program,
            emit_resolver,
        },
        || {},
    )
}

impl EmitHostImpl {
    pub fn get_mode_for_usage_location(
        &self,
        file: &Arc<SourceFile>,
        module_specifier: &Arc<Node>,
    ) -> ResolutionMode {
        self.program.get_mode_for_usage_location(file, module_specifier)
    }

    pub fn get_resolved_module_from_module_specifier(
        &self,
        file: &Arc<SourceFile>,
        module_specifier: &Arc<Node>,
    ) -> Option<ResolvedModule> {
        self.program
            .get_resolved_module_from_module_specifier(file, module_specifier)
    }

    pub fn get_default_resolution_mode_for_file(
        &self,
        file: &Arc<SourceFile>,
    ) -> ResolutionMode {
        self.program.get_default_resolution_mode_for_file(file)
    }

    pub fn get_emit_module_format_of_file(&self, file: &Arc<SourceFile>) -> ModuleKind {
        <Program as tsox_checker::checker::Program>::get_emit_module_format_of_file(
            &self.program,
            &file.file_name,
        )
    }

    pub fn file_exists(&self, path: &str) -> bool {
        self.program.file_exists(path)
    }

    pub fn get_global_typings_cache_location(&self) -> String {
        self.program.get_global_typings_cache_location().to_string()
    }

    pub fn get_nearest_ancestor_directory_with_package_json(&self, dirname: &str) -> String {
        self.program
            .get_nearest_ancestor_directory_with_package_json(dirname)
    }

    pub fn get_package_json_info(
        &self,
        pkg_json_path: &str,
    ) -> Option<tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry> {
        self.program.get_package_json_info(pkg_json_path)
    }

    pub fn get_source_of_project_reference_if_output_included(
        &self,
        file: &Arc<SourceFile>,
    ) -> String {
        self.program
            .get_source_of_project_reference_if_output_included(file)
    }

    pub fn get_project_reference_from_source(
        &self,
        path: &str,
    ) -> Option<tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference> {
        self.program.get_project_reference_from_source(path)
    }

    pub fn get_redirect_targets(&self, path: &str) -> Vec<String> {
        self.program.get_redirect_targets(path)
    }

    pub fn get_effective_declaration_flags(
        &self,
        node: &Arc<Node>,
        flags: ModifierFlags,
    ) -> ModifierFlags {
        self.emit_resolver.get_effective_declaration_flags(node, flags)
    }

    pub fn get_output_paths_for(
        &self,
        file: &Arc<SourceFile>,
        force_dts_paths: bool,
    ) -> tsox_emit::mig::m3n_5::r33k8_defs::OutputPathsValue {
        outputpaths::get_output_paths_for(
            file,
            self.options(),
            self,
            outputpaths::ForceEmitPaths {
                dts: force_dts_paths,
                js: false,
                declaration_map: false,
            },
        )
    }

    pub fn source_file_may_be_emitted(
        &self,
        file: &Arc<SourceFile>,
        force_dts_emit: bool,
    ) -> bool {
        super::m4v::source_file_may_be_emitted(file, self, force_dts_emit, false)
    }

    pub fn get_source_file_from_reference(
        &self,
        origin: &Arc<SourceFile>,
        reference: &FileReference,
    ) -> Option<Arc<SourceFile>> {
        self.program.get_source_file_from_reference(origin, reference)
    }

    pub fn options(&self) -> &CompilerOptions {
        &self.program.options
    }

    pub fn source_files(&self) -> &[Arc<SourceFile>] {
        &self.program.source_files
    }

    pub fn get_current_directory(&self) -> String {
        self.program.get_current_directory().to_string()
    }

    pub fn common_source_directory(&self) -> String {
        <Program as tsox_checker::checker::Program>::common_source_directory(&self.program)
    }

    pub fn content_mapper_extensions(&self) -> Vec<String> {
        self.program.content_mapper_extensions()
    }

    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.program.use_case_sensitive_file_names()
    }

    pub fn is_emit_blocked(&self, file: &str) -> bool {
        self.program.is_emit_blocked(file)
    }

    pub fn write_file(&self, file_name: &str, text: &str) -> std::io::Result<()> {
        self.program.host.fs().write_file(file_name, text)
    }

    pub fn get_emit_resolver(&self) -> Arc<tsox_checker::checker::mig::m2d::EmitResolver> {
        self.emit_resolver.get()
    }

    pub fn is_source_file_from_external_library(&self, file: &Arc<SourceFile>) -> bool {
        self.program.is_source_file_from_external_library(file)
    }

    pub fn get_symlink_cache(&self) -> &tsox_core::symlinks::KnownSymlinks {
        self.program.get_symlink_cache()
    }

    pub fn resolve_module_name(
        &self,
        module_name: &str,
        containing_file: &str,
        resolution_mode: ResolutionMode,
    ) -> Option<ResolvedModule> {
        self.program
            .resolve_module_name(module_name, containing_file, resolution_mode)
    }
}

impl EmitHost for EmitHostImpl {
    fn options(&self) -> &CompilerOptions {
        EmitHostImpl::options(self)
    }
    fn source_files(&self) -> &[Arc<SourceFile>] {
        EmitHostImpl::source_files(self)
    }
    fn use_case_sensitive_file_names(&self) -> bool {
        EmitHostImpl::use_case_sensitive_file_names(self)
    }
    fn get_current_directory(&self) -> String {
        EmitHostImpl::get_current_directory(self)
    }
    fn common_source_directory(&self) -> String {
        EmitHostImpl::common_source_directory(self)
    }
    fn is_emit_blocked(&self, file: &str) -> bool {
        EmitHostImpl::is_emit_blocked(self, file)
    }
}

impl SourceFileMayBeEmittedHost for EmitHostImpl {
    fn options(&self) -> &CompilerOptions {
        EmitHostImpl::options(self)
    }

    fn get_project_reference_from_source(
        &self,
        path: &str,
    ) -> Option<tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference> {
        EmitHostImpl::get_project_reference_from_source(self, path)
    }
    fn is_source_file_from_external_library(&self, file: &Arc<SourceFile>) -> bool {
        self.program.is_source_file_from_external_library(file)
    }
    fn is_source_file_from_project_reference(&self, file: &Arc<SourceFile>) -> bool {
        self.program.is_source_from_project_reference(&file.file_name)
    }
    fn get_current_directory(&self) -> String {
        EmitHostImpl::get_current_directory(self)
    }
    fn use_case_sensitive_file_names(&self) -> bool {
        EmitHostImpl::use_case_sensitive_file_names(self)
    }
    fn source_files(&self) -> Vec<Arc<SourceFile>> {
        EmitHostImpl::source_files(self).to_vec()
    }
}

mod outputpaths;
