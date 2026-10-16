#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath::{self, Path};
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use tsox_frontend::ast::SourceFile;
use super::m4x_build::BuildTask;
use super::m4x_build_3::Orchestrator;

pub struct Host {
    pub orchestrator: Arc<Mutex<Orchestrator>>,
    pub host: Arc<tsox_compile::mig::m4v_3::CompilerHostImpl>,

    pub extended_config_cache: crate::mig::m5a_2::ExtendedConfigCache,
    pub source_files: ParseCache<SourceFileParseOptions, Arc<SourceFile>>,
    pub config_times: SyncMap<Path, std::time::Duration>,

    pub resolved_references: ParseCache<Path, Arc<ParsedCommandLine>>,
    pub m_times: SyncMap<Path, SystemTime>,
}

pub type SourceFileParseOptions = tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions;
pub type ParseCache<K, V> = SyncMap<K, V>;

impl Host {
    pub fn fs(&self) -> Arc<dyn tsox_tsoptions::vfs::FS> { ::tsox_core::fntrace::enter("fs"); 
        self.host.fs().clone()
    }

    pub fn default_library_path(&self) -> String { ::tsox_core::fntrace::enter("default_library_path"); 
        self.host.default_library_path().to_string()
    }

    pub fn current_directory(&self) -> String { ::tsox_core::fntrace::enter("current_directory"); 
        self.host.get_current_directory().to_string()
    }

    pub fn trace(&self, msg: &'static tsox_core::diagnostics::Message, args: &[String]) { ::tsox_core::fntrace::enter("trace"); 
        panic!(
            "build.Orchestrator.host does not support tracing; use a different host for tracing"
        );
    }

    pub fn get_source_file(&self, opts: SourceFileParseOptions) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        if tsox_core::tspath::is_declaration_file_name(&opts.file_name)
            || tsox_core::tspath::file_extension_is(&opts.file_name, ".json")
        {
            if let Some(existing) = self.source_files.load(&opts) {
                return Some(existing);
            }
            let value = self.host.get_source_file(&tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions {
                file_name: opts.file_name.clone(),
                path: opts.path.clone(),
            })?;
            let (stored, _) = self.source_files.load_or_store(opts, value);
            return Some(stored);
        }
        self.host.get_source_file(&tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions {
            file_name: opts.file_name,
            path: opts.path,
        })
    }

    pub fn get_content_mapped_source_files(
        &self,
        parse_options: SourceFileParseOptions,
        mapper: &tsox_compile::mig::m3l_cm::Mapper,
    ) -> Result<tsox_compile::mig::m3l_cm_2::SourceFiles, tsox_compile::mig::m4v_3::ContentMapperError>
    { ::tsox_core::fntrace::enter("get_content_mapped_source_files"); 
        Err(tsox_compile::mig::m4v_3::ContentMapperError::project_unavailable())
    }

    pub fn content_mapper_project(&self) -> Option<Arc<dyn tsox_compile::mig::m3l_cm_2::Project>> { ::tsox_core::fntrace::enter("content_mapper_project"); 
        panic!(
            "build.Orchestrator.host does not support content mapper project; use an individual project's compiler host instead"
        );
    }

    pub fn get_resolved_project_reference(
        &self,
        file_name: &str,
        path: Path,
    ) -> Option<Arc<ParsedCommandLine>> { ::tsox_core::fntrace::enter("get_resolved_project_reference"); 
        if let Some(existing) = self.resolved_references.load(&path) {
            return Some(existing);
        }
        let orchestrator = self.orchestrator.lock().unwrap();
        let config_start = SystemTime::now();
        let sys = tsox_tsoptions::mig::m5j_2::ParseConfigHost {
            fs: self.fs(),
            current_directory: self.current_directory(),
        };
        let (command_line, _) =
            tsox_tsoptions::mig::m5i2_4::get_parsed_command_line_of_config_file_path(
                file_name,
                path.as_str(),
                Some(&orchestrator.opts.command.compiler_options),
                None,
                &sys,
                None,
            );
        let config_time = SystemTime::now()
            .duration_since(config_start)
            .unwrap_or_default();
        self.config_times.store(path.clone(), config_time);
        if let Some(command_line) = &command_line {
            self.resolved_references.store(path, Arc::new(command_line.clone()));
        }
        command_line.map(Arc::new)
    }

    pub fn read_build_info(
        &self,
        config: &ParsedCommandLine,
    ) -> Option<crate::mig::m4y_2::BuildInfo> { ::tsox_core::fntrace::enter("read_build_info"); 
        let mut orchestrator = self.orchestrator.lock().unwrap();
        let config_path = orchestrator.to_path(&config.config_name());
        let task = orchestrator.get_task(&config_path);
        let (build_info, _) = task.lock().unwrap().load_or_store_build_info(
            &mut *orchestrator,
            config_path,
            &config.get_build_info_file_name(),
        );
        build_info
    }

    pub fn get_m_time(&self, file: &str) -> SystemTime { ::tsox_core::fntrace::enter("get_m_time"); 
        self.load_or_store_m_time(file, None, true)
    }

    pub fn set_m_time(&self, file: &str, m_time: SystemTime) -> Result<(), std::io::Error> { ::tsox_core::fntrace::enter("set_m_time"); 
        self.fs()
            .chtimes(file, SystemTime::UNIX_EPOCH, m_time)
    }

    pub fn load_or_store_m_time(
        &self,
        file: &str,
        old_cache: Option<&SyncMap<Path, SystemTime>>,
        store: bool,
    ) -> SystemTime { ::tsox_core::fntrace::enter("load_or_store_m_time"); 
        let path = self.orchestrator.lock().unwrap().to_path(file);
        if let Some(existing) = self.m_times.load(&path) {
            return existing;
        }
        let mut found = false;
        let mut m_time = SystemTime::UNIX_EPOCH;
        if let Some(old_cache) = old_cache {
            if let Some(value) = old_cache.load(&path) {
                m_time = value;
                found = true;
            }
        }
        if !found {
            m_time = self
                .host
                .fs()
                .clone()
                .stat(file)
                .map(|info| info.modified)
                .unwrap_or(SystemTime::UNIX_EPOCH);
        }
        if store {
            let (stored, _) = self.m_times.load_or_store(path, m_time);
            m_time = stored;
        }
        m_time
    }

    pub fn store_m_time(&self, file: &str, m_time: SystemTime) { ::tsox_core::fntrace::enter("store_m_time"); 
        let path = self.orchestrator.lock().unwrap().to_path(file);
        self.m_times.store(path, m_time);
    }

    pub fn store_m_time_from_old_cache(
        &self,
        file: &str,
        old_cache: &SyncMap<Path, SystemTime>,
    ) { ::tsox_core::fntrace::enter("store_m_time_from_old_cache"); 
        let path = self.orchestrator.lock().unwrap().to_path(file);
        if let Some(m_time) = old_cache.load(&path) {
            self.m_times.store(path, m_time);
        }
    }
}

pub struct CompilerHost {
    pub host: Arc<Host>,
    pub trace: crate::mig::m5a::TraceFn,
    pub content_mapper_project: Option<Arc<dyn tsox_compile::mig::m3l_cm_2::Project>>,
    fs: Arc<dyn tsox_tsoptions::vfs::FS>,
    current_directory: String,
    default_library_path: String,
}

impl CompilerHost {
    pub fn new(
        host: Arc<Host>,
        trace: crate::mig::m5a::TraceFn,
        content_mapper_project: Option<Arc<dyn tsox_compile::mig::m3l_cm_2::Project>>,
    ) -> Self { ::tsox_core::fntrace::enter("new"); 
        let fs = host.fs();
        let current_directory = host.current_directory();
        let default_library_path = host.default_library_path();
        Self {
            host,
            trace,
            content_mapper_project,
            fs,
            current_directory,
            default_library_path,
        }
    }

    pub fn fs(&self) -> Arc<dyn tsox_tsoptions::vfs::FS> { ::tsox_core::fntrace::enter("fs"); 
        self.host.fs()
    }

    pub fn default_library_path(&self) -> String { ::tsox_core::fntrace::enter("default_library_path"); 
        self.host.default_library_path()
    }

    pub fn current_directory(&self) -> String { ::tsox_core::fntrace::enter("current_directory"); 
        self.host.current_directory()
    }

    pub fn trace_message(&self, msg: &'static tsox_core::diagnostics::Message, args: &[String]) { ::tsox_core::fntrace::enter("trace_message"); 
        (self.trace)(msg, args);
    }

    pub fn get_source_file(&self, opts: SourceFileParseOptions) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        self.host.get_source_file(opts)
    }

    pub fn get_content_mapped_source_files(
        &self,
        parse_options: SourceFileParseOptions,
        mapper: &tsox_compile::mig::m3l_cm::Mapper,
    ) -> Result<tsox_compile::mig::m3l_cm_2::SourceFiles, tsox_compile::mig::m4v_3::ContentMapperError>
    { ::tsox_core::fntrace::enter("get_content_mapped_source_files"); 
        let content_mapper_project = match self.content_mapper_project.as_ref() {
            None => return Err(tsox_compile::mig::m4v_3::ContentMapperError::project_unavailable()),
            Some(project) => project.clone(),
        };
        let content = match self.fs().read_file(&parse_options.file_name) {
            None => return Ok(Default::default()),
            Some(content) => content,
        };
        let files = tsox_compile::mig::m3l_cm_2::transform_and_parse(
            parse_options.clone(),
            &content,
            mapper,
            content_mapper_project.as_ref(),
        );
        match files {
            Ok(files) => {
                if let Err(err) = tsox_compile::mig::m3l_cm_2::check_supplemental_file_name_collisions(
                    &files,
                    &|file| self.fs().file_exists(file),
                ) {
                    Err(tsox_compile::mig::m4v_3::ContentMapperError::from(err))
                } else {
                    Ok(files)
                }
            }
            Err(err) => Err(tsox_compile::mig::m4v_3::ContentMapperError::from(err)),
        }
    }

    pub fn content_mapper_project_value(
        &self,
    ) -> Option<Arc<dyn tsox_compile::mig::m3l_cm_2::Project>> { ::tsox_core::fntrace::enter("content_mapper_project_value"); 
        self.content_mapper_project.clone()
    }

    pub fn get_resolved_project_reference(
        &self,
        file_name: &str,
        path: Path,
    ) -> Option<Arc<ParsedCommandLine>> { ::tsox_core::fntrace::enter("get_resolved_project_reference"); 
        self.host.get_resolved_project_reference(file_name, path)
    }
}

impl tsox_compile::compiler::CompilerHost for CompilerHost {
    fn fs(&self) -> &dyn tsox_tsoptions::vfs::FS { ::tsox_core::fntrace::enter("fs"); 
        self.fs.as_ref()
    }

    fn fs_arc(&self) -> Arc<dyn tsox_tsoptions::vfs::FS> { ::tsox_core::fntrace::enter("fs_arc"); 
        Arc::clone(&self.fs)
    }

    fn current_directory(&self) -> &str { ::tsox_core::fntrace::enter("current_directory"); 
        &self.current_directory
    }

    fn default_library_path(&self) -> &str { ::tsox_core::fntrace::enter("default_library_path"); 
        &self.default_library_path
    }

    fn get_source_file(
        &self,
        opts: &tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions,
    ) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        self.get_source_file(SourceFileParseOptions {
            file_name: opts.file_name.clone(),
            path: opts.path.clone(),
            external_module_indicator_options: Default::default(),
        })
    }

    fn get_content_mapped_source_files(
        &self,
        parse_options: &tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions,
        mapper: &tsox_compile::mig::m3l_cm::Mapper,
    ) -> Result<tsox_compile::mig::m3l_cm_2::SourceFiles, tsox_compile::mig::m4v_3::ContentMapperError>
    { ::tsox_core::fntrace::enter("get_content_mapped_source_files"); 
        self.get_content_mapped_source_files(
            SourceFileParseOptions {
                file_name: parse_options.file_name.clone(),
                path: parse_options.path.clone(),
                external_module_indicator_options: Default::default(),
            },
            mapper,
        )
    }
}
