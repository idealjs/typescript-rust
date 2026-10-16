#![allow(dead_code)]

use std::sync::{Arc, OnceLock};

pub use tsox_compile::compiler::CompilerHost;
use tsox_compile::mig::m3l_cm_2::Project as MapperProject;
use tsox_core::tspath::Path;
use tsox_tsoptions::vfs::FS;

use super::config_file_registry::ConfigFileRegistry;
use super::logging_log_tree::LogTree;
use super::mig::m5e_7::SourceFS;
use super::project::Project;
use super::project_collection_builder::ProjectCollectionBuilder;

#[derive(Clone)]
pub struct SessionOptions {
    pub current_directory: String,
    pub default_library_path: String,
    pub typings_location: String,
    pub position_encoding: crate::lsp::lsproto::PositionEncodingKind,
    pub watch_enabled: bool,
    pub logging_enabled: bool,
    pub telemetry_enabled: bool,
    pub push_diagnostics_enabled: bool,
    pub debounce_delay: std::time::Duration,
    pub locale: String,
    pub run_external_code: bool,
}

impl Default for SessionOptions {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        SessionOptions {
            current_directory: String::new(),
            default_library_path: String::new(),
            typings_location: String::new(),
            position_encoding: crate::lsp::lsproto::POSITION_ENCODING_UTF16.to_string(),
            watch_enabled: false,
            logging_enabled: false,
            telemetry_enabled: false,
            push_diagnostics_enabled: false,
            debounce_delay: std::time::Duration::from_millis(250),
            locale: "en".to_string(),
            run_external_code: false,
        }
    }
}

pub struct SessionInit {
    pub options: SessionOptions,
    pub fs: Arc<dyn FS>,
}

pub struct CompilerHostImpl {
    config_file_path: Path,
    current_directory: String,
    session_options: SessionOptions,
    fs: Arc<dyn FS>,
    frozen: bool,

    source_fs: Option<SourceFS>,
    config_file_registry: Option<Arc<ConfigFileRegistry>>,
    project: Option<Arc<Project>>,
    builder: Option<Arc<ProjectCollectionBuilder>>,
    logger: Option<Arc<LogTree>>,
    content_mapper_project: OnceLock<Option<Arc<dyn MapperProject>>>,
}

impl CompilerHostImpl {
    pub fn new(
        current_directory: String,
        project_path: Path,
        session_options: SessionOptions,
        fs: Arc<dyn FS>,
    ) -> Self { ::tsox_core::fntrace::enter("new"); 
        CompilerHostImpl {
            config_file_path: project_path,
            current_directory,
            session_options,
            fs,
            frozen: false,

            source_fs: None,
            config_file_registry: None,
            project: None,
            builder: None,
            logger: None,
            content_mapper_project: OnceLock::new(),
        }
    }

    pub fn freeze(&mut self) { ::tsox_core::fntrace::enter("freeze"); 
        self.frozen = true;
    }

    pub fn ensure_alive(&self) { ::tsox_core::fntrace::enter("ensure_alive"); 
        if self.frozen {
            panic!("method must not be called after snapshot initialization");
        }
    }

    pub fn set_source_fs(&mut self, source_fs: SourceFS) { ::tsox_core::fntrace::enter("set_source_fs"); 
        self.source_fs = Some(source_fs);
    }

    pub fn source_fs(&self) -> &SourceFS { ::tsox_core::fntrace::enter("source_fs"); 
        self.source_fs
            .as_ref()
            .expect("source fs must be initialized")
    }

    pub fn set_config_file_registry(&mut self, registry: ConfigFileRegistry) { ::tsox_core::fntrace::enter("set_config_file_registry"); 
        self.config_file_registry = Some(Arc::new(registry));
    }

    pub fn config_file_registry(&self) -> Option<&ConfigFileRegistry> { ::tsox_core::fntrace::enter("config_file_registry"); 
        self.config_file_registry.as_deref()
    }

    pub fn has_builder(&self) -> bool { ::tsox_core::fntrace::enter("has_builder"); 
        self.builder.is_some()
    }

    pub fn set_builder(&mut self, builder: Arc<ProjectCollectionBuilder>) { ::tsox_core::fntrace::enter("set_builder"); 
        self.builder = Some(builder);
    }

    pub fn builder(&self) -> &ProjectCollectionBuilder { ::tsox_core::fntrace::enter("builder"); 
        self.builder
            .as_ref()
            .expect("builder must be initialized while host is alive")
    }

    pub fn set_project(&mut self, project: Arc<Project>) { ::tsox_core::fntrace::enter("set_project"); 
        self.project = Some(project);
    }

    pub fn project(&self) -> &Project { ::tsox_core::fntrace::enter("project"); 
        self.project
            .as_ref()
            .expect("project must be initialized while host is alive")
    }

    pub fn set_logger(&mut self, logger: Option<Arc<LogTree>>) { ::tsox_core::fntrace::enter("set_logger"); 
        self.logger = logger;
    }

    pub fn logger(&self) -> Option<Arc<LogTree>> { ::tsox_core::fntrace::enter("logger"); 
        self.logger.clone()
    }

    pub fn content_mapper_project(&self) -> Option<Arc<dyn MapperProject>> { ::tsox_core::fntrace::enter("content_mapper_project"); 
        let project = self.content_mapper_project.get()?;
        project.clone()
    }

    pub fn init_content_mapper_project(
        &self,
        init: impl FnOnce() -> Option<Arc<dyn MapperProject>>,
    ) { ::tsox_core::fntrace::enter("init_content_mapper_project"); 
        self.content_mapper_project.get_or_init(init);
    }
}

impl CompilerHost for CompilerHostImpl {
    fn fs(&self) -> &dyn FS { ::tsox_core::fntrace::enter("fs"); 
        self.fs.as_ref()
    }
    fn fs_arc(&self) -> Arc<dyn FS> { ::tsox_core::fntrace::enter("fs_arc"); 
        Arc::clone(&self.fs)
    }
    fn current_directory(&self) -> &str { ::tsox_core::fntrace::enter("current_directory"); 
        &self.current_directory
    }
    fn default_library_path(&self) -> &str { ::tsox_core::fntrace::enter("default_library_path"); 
        &self.session_options.default_library_path
    }
    fn get_source_file(
        &self,
        opts: &tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions,
    ) -> Option<Arc<tsox_frontend::ast::SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        let lsp_opts = crate::project::mig::m5d_2::SourceFileParseOptions {
            file_name: opts.file_name.clone(),
            path: Path(opts.path.clone()),
        };
        CompilerHostImpl::get_source_file(self, &lsp_opts)
    }
    fn get_content_mapped_source_files(
        &self,
        parse_options: &tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions,
        mapper: &tsox_compile::mig::m3l_cm::Mapper,
    ) -> Result<
        tsox_compile::mig::m3l_cm_2::SourceFiles,
        tsox_compile::mig::m4v_3::ContentMapperError,
    > { ::tsox_core::fntrace::enter("get_content_mapped_source_files"); 
        let lsp_opts = crate::project::mig::m5d_2::SourceFileParseOptions {
            file_name: parse_options.file_name.clone(),
            path: Path(parse_options.path.clone()),
        };
        CompilerHostImpl::get_content_mapped_source_files(self, &lsp_opts, mapper)
            .map_err(Into::into)
    }
    fn get_resolved_project_reference(
        &self,
        file_name: &str,
        path: &Path,
    ) -> Option<tsox_tsoptions::tsoptions::ParsedCommandLine> { ::tsox_core::fntrace::enter("get_resolved_project_reference"); 
        CompilerHostImpl::get_resolved_project_reference(self, file_name, path)
    }
}
