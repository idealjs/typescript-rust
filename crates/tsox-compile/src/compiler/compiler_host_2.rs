#![allow(unused_imports)]

use super::*;

pub trait CompilerHost: Send + Sync {
    fn fs(&self) -> &dyn FS;

    fn fs_arc(&self) -> Arc<dyn FS>;
    fn current_directory(&self) -> &str;
    fn default_library_path(&self) -> &str;
    fn use_case_sensitive_file_names(&self) -> bool {
        self.fs().use_case_sensitive_file_names()
    }
    fn content_mapper_project(&self) -> Option<Arc<dyn crate::mig::m3l_cm_2::Project>> {
        None
    }
    fn get_source_file(
        &self,
        opts: &tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions,
    ) -> Option<Arc<SourceFile>>;
    fn get_content_mapped_source_files(
        &self,
        parse_options: &tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions,
        mapper: &crate::mig::m3l_cm::Mapper,
    ) -> Result<crate::mig::m3l_cm_2::SourceFiles, crate::mig::m4v_3::ContentMapperError>;
    fn get_resolved_project_reference(
        &self,
        file_name: &str,
        path: &tsox_core::tspath::Path,
    ) -> Option<ParsedCommandLine> {
        let sys = tsox_tsoptions::mig::m5j_2::ParseConfigHost {
            fs: self.fs_arc(),
            current_directory: self.current_directory().to_string(),
        };
        let (command_line, _) = tsox_tsoptions::mig::m5i2_4::get_parsed_command_line_of_config_file_path(
            file_name,
            path.as_str(),
            None,
            None,
            &sys,
            None,
        );
        command_line
    }
}

impl ProgramOptions {
    pub fn can_use_project_reference_source(&self) -> bool {
        self.use_source_of_project_reference
            && !self
                .config
                .compiler_options()
                .disable_source_of_project_reference_redirect
                .is_true()
    }
}

unsafe impl Send for Program {}
unsafe impl Sync for Program {}

pub struct CompilerHostImpl {
    pub(crate) fs: Arc<dyn FS>,
    pub(crate) current_directory: String,
    pub(crate) default_library_path: String,
}

impl CompilerHostImpl {
    pub fn new(fs: Arc<dyn FS>, current_directory: String, default_library_path: String) -> Self {
        Self {
            fs,
            current_directory,
            default_library_path,
        }
    }
}

impl CompilerHost for CompilerHostImpl {
    fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }
    fn get_source_file(
        &self,
        opts: &tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions,
    ) -> Option<Arc<SourceFile>> {
        let text = self.fs.read_file(&opts.file_name)?;
        Some(Arc::new(Parser::parse_source_file_text(
            &opts.file_name,
            text.to_string(),
        )))
    }
    fn get_content_mapped_source_files(
        &self,
        parse_options: &tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions,
        mapper: &crate::mig::m3l_cm::Mapper,
    ) -> Result<crate::mig::m3l_cm_2::SourceFiles, crate::mig::m4v_3::ContentMapperError> {
        let Some(project) = self.content_mapper_project() else {
            return Err(crate::mig::m4v_3::ContentMapperError::project_unavailable());
        };
        let Some(content) = self.fs.read_file(&parse_options.file_name) else {
            return Ok(Default::default());
        };
        let files = crate::mig::m3l_cm_2::transform_and_parse(
            tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions {
                file_name: parse_options.file_name.clone(),
                path: parse_options.path.clone(),
                external_module_indicator_options: Default::default(),
            },
            &content,
            mapper,
            project.as_ref(),
        )?;
        crate::mig::m3l_cm_2::check_supplemental_file_name_collisions(
            &files,
            &|name: &str| self.fs.file_exists(name),
        )?;
        Ok(files)
    }
    fn fs_arc(&self) -> Arc<dyn FS> {
        Arc::clone(&self.fs)
    }
    fn current_directory(&self) -> &str {
        &self.current_directory
    }
    fn default_library_path(&self) -> &str {
        &self.default_library_path
    }
}

pub(crate) struct ResolutionHostAdapter {
    pub(crate) fs: Arc<dyn FS>,
    pub(crate) current_directory: String,
}

impl ResolutionHostAdapter {
    pub(crate) fn new(host: &dyn CompilerHost) -> Self {
        Self {
            fs: host.fs_arc(),
            current_directory: host.current_directory().to_string(),
        }
    }
}

impl tsox_tsoptions::module::ResolutionHost for ResolutionHostAdapter {
    fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }
    fn get_current_directory(&self) -> &str {
        &self.current_directory
    }
}

pub type CreateCheckerPoolFn = Arc<
    dyn Fn(&Program) -> Arc<dyn crate::mig::m4u_2::CheckerPool> + Send + Sync,
>;

#[derive(Clone)]
pub struct ProgramOptions {
    pub config: ParsedCommandLine,
    pub host: Arc<dyn CompilerHost>,
    pub use_source_of_project_reference: bool,
    pub single_threaded: Tristate,
    pub create_checker_pool: Option<CreateCheckerPoolFn>,
    pub typings_location: String,
    pub project_name: String,
    pub tracing: Option<Arc<tsox_core::tracing::mig::x11a::Tracing<'static>>>,
    pub skip_module_resolution: bool,
}

pub struct Program {
    pub(crate) options: CompilerOptions,
    pub(crate) source_files: Vec<Arc<SourceFile>>,
    pub(crate) source_files_by_name: HashMap<String, Arc<SourceFile>>,
    pub(crate) default_library_file_names: std::collections::HashSet<String>,
    pub(crate) diagnostics: Vec<Arc<Diagnostic>>,
    pub(crate) host: Arc<dyn CompilerHost>,
    pub(crate) config_file_name: String,

    pub(crate) symbol_map: NodeSymbolMap,

    pub(crate) opts: ProgramOptions,
    pub(crate) resolver: Arc<tsox_tsoptions::module::Resolver>,
    pub(crate) checker_pool: std::sync::OnceLock<Arc<dyn crate::mig::m4u_2::CheckerPool>>,
    pub(crate) compiler_checker_pool: std::sync::OnceLock<Arc<dyn crate::mig::m4u_2::CheckerPool>>,
    pub(crate) compare_paths_options: tsox_core::tspath::ComparePathsOptions,
    pub(crate) files_by_path: HashMap<String, Arc<SourceFile>>,
    pub(crate) project_reference_file_mapper:
        Arc<crate::mig::m4x_2::ProjectReferenceFileMapper>,
    pub(crate) missing_files: Vec<String>,
    pub(crate) resolved_modules:
        HashMap<String, crate::mig::m4w_5::ModeAwareCache<tsox_tsoptions::module::ResolvedModule>>,
    pub(crate) type_resolutions_in_file: HashMap<
        String,
        crate::mig::m4w_5::ModeAwareCache<tsox_tsoptions::module::ResolvedTypeReferenceDirective>,
    >,
    pub(crate) source_file_meta_datas:
        HashMap<String, tsox_frontend::ast::mig::x4ast::SourceFileMetaData>,
    pub(crate) jsx_runtime_import_specifiers:
        HashMap<String, crate::mig::m4w_5::JsxRuntimeImportSpecifier>,
    pub(crate) import_helpers_import_specifiers: HashMap<String, Arc<tsox_frontend::ast::Node>>,
    pub(crate) lib_files: HashMap<String, LibFile>,
    pub(crate) source_files_found_searching_node_modules: std::collections::HashSet<String>,
    pub(crate) include_processor: Option<Box<crate::mig::m4v_3::IncludeProcessor>>,
    pub(crate) output_file_to_project_reference_source: HashMap<String, String>,
    pub(crate) redirect_targets_map: HashMap<String, Vec<String>>,
    pub(crate) redirect_files_by_path: HashMap<String, crate::mig::m4v_2::RedirectsFile>,
    pub(crate) content_mapper_diagnostics: Vec<Arc<Diagnostic>>,
    pub(crate) program_diagnostics: Vec<Arc<Diagnostic>>,
    pub(crate) content_mapper_option_diagnostics: Vec<Arc<Diagnostic>>,
    pub(crate) has_emit_blocking_diagnostics: std::collections::HashSet<String>,
    pub(crate) unresolved_imports: crate::mig::m4w_5::LazyValue<std::collections::HashSet<String>>,
    pub(crate) known_symlinks: crate::mig::m4w_5::LazyValue<tsox_core::symlinks::KnownSymlinks>,
    pub(crate) package_names: crate::mig::m4w_5::LazyValue<crate::mig::m4w_5::PackageNamesInfo>,
    pub(crate) has_ts_file_once: crate::mig::m4w_5::LazyValue<bool>,
    pub(crate) uses_uri_style_node_core_modules: Tristate,
    pub(crate) typings_location: String,
    pub(crate) use_source_of_project_reference: bool,
    pub(crate) tracing: Option<Arc<tsox_core::tracing::mig::x11a::Tracing<'static>>>,
    pub(crate) files: Vec<Arc<SourceFile>>,
    pub(crate) finished_processing: bool,
}
