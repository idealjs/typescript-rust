use std::collections::HashMap;
use std::sync::Arc;

use tsox_checker::binder::bind_source_file;
use tsox_core::diagnostics::Message;
use tsox_core::locale::Locale;
use tsox_core::tspath::Path;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use tsox_compile::mig::m3l_cm::{
    new_transform_error, Mapper, TransformError, TransformErrorKind,
};
use tsox_compile::mig::m3l_cm_2::{
    check_supplemental_file_name_collisions, transform_and_parse, Project as MapperProject,
    SourceFiles,
};

use super::m5d_2::SourceFileParseOptions;
use super::super::config_file_registry::{ConfigFileEntry, ConfigFileNames, ConfigFileRegistry, PendingReload};
use super::super::logging_log_tree::LogTree;
use super::super::project::Project;
use super::super::project_collection_builder::ProjectCollectionBuilder;
use super::super::snapshot_fs::FileSource;

pub struct TestConfigEntry {
    pub file_name: String,
    pub retaining_projects: Vec<Path>,
    pub retaining_open_files: Vec<Path>,
    pub retaining_configs: Vec<Path>,
}

pub struct TestConfigFileNamesEntry {
    pub nearest_config_file_name: String,
    pub ancestors: HashMap<String, String>,
}

#[derive(Clone, Default)]
pub struct ConfiguredContentMappers {
    pub extensions: Vec<String>,
}

pub fn collect_configured_content_mappers(
    command_lines: &[&ParsedCommandLine],
) -> ConfiguredContentMappers { ::tsox_core::fntrace::enter("collect_configured_content_mappers"); 
    let mut seen_extensions = std::collections::HashSet::new();
    let mut extensions: Vec<String> = Vec::new();
    for command_line in command_lines {
        for mapper in command_line.content_mappers() {
            for extension in &mapper.definition.extensions {
                if seen_extensions.insert(extension.clone()) {
                    extensions.push(extension.clone());
                }
            }
        }
    }
    extensions.sort();
    ConfiguredContentMappers { extensions }
}

impl ConfigFileRegistry {
    pub fn content_mappers(&self) -> ConfiguredContentMappers { ::tsox_core::fntrace::enter("content_mappers"); 
        let command_lines: Vec<&ParsedCommandLine> = self
            .configs
            .values()
            .filter_map(|entry| entry.command_line.as_ref())
            .collect();
        collect_configured_content_mappers(&command_lines)
    }

    pub fn is_tracked(&self, path: &Path) -> bool { ::tsox_core::fntrace::enter("is_tracked"); 
        self.configs.contains_key(path)
    }

    pub fn clone_registry(&self) -> ConfigFileRegistry { ::tsox_core::fntrace::enter("clone_registry"); 
        ConfigFileRegistry {
            configs: self.configs.clone(),
            config_file_names: self.config_file_names.clone(),
            custom_config_file_name: self.custom_config_file_name.clone(),
        }
    }

    pub fn for_each_test_config_entry(
        &self,
        mut cb: impl FnMut(&Path, &TestConfigEntry),
    ) { ::tsox_core::fntrace::enter("for_each_test_config_entry"); 
        for (path, entry) in &self.configs {
            cb(path, &TestConfigEntry {
                file_name: entry.file_name.clone(),
                retaining_projects: entry.retaining_projects.keys().cloned().collect(),
                retaining_open_files: entry.retaining_open_files.keys().cloned().collect(),
                retaining_configs: entry.retaining_configs.keys().cloned().collect(),
            });
        }
    }

    pub fn get_test_config_entry(&self, path: &Path) -> Option<TestConfigEntry> { ::tsox_core::fntrace::enter("get_test_config_entry"); 
        self.configs.get(path).map(|entry| TestConfigEntry {
            file_name: entry.file_name.clone(),
            retaining_projects: entry.retaining_projects.keys().cloned().collect(),
            retaining_open_files: entry.retaining_open_files.keys().cloned().collect(),
            retaining_configs: entry.retaining_configs.keys().cloned().collect(),
        })
    }

    pub fn for_each_test_config_file_names_entry(
        &self,
        mut cb: impl FnMut(&Path, &TestConfigFileNamesEntry),
    ) { ::tsox_core::fntrace::enter("for_each_test_config_file_names_entry"); 
        for (path, entry) in &self.config_file_names {
            cb(path, &TestConfigFileNamesEntry {
                nearest_config_file_name: entry.nearest_config_file_name.clone(),
                ancestors: entry.ancestors.clone(),
            });
        }
    }

    pub fn get_test_config_file_names_entry(&self, path: &Path) -> Option<TestConfigFileNamesEntry> { ::tsox_core::fntrace::enter("get_test_config_file_names_entry"); 
        self.config_file_names.get(path).map(|entry| TestConfigFileNamesEntry {
            nearest_config_file_name: entry.nearest_config_file_name.clone(),
            ancestors: entry.ancestors.clone(),
        })
    }
}

impl ConfigFileEntry {
    pub fn clone_entry(&self) -> ConfigFileEntry { ::tsox_core::fntrace::enter("clone_entry"); 
        ConfigFileEntry {
            file_name: self.file_name.clone(),
            pending_reload: self.pending_reload,
            command_line: self.command_line.clone(),
            retaining_projects: self.retaining_projects.clone(),
            retaining_open_files: self.retaining_open_files.clone(),
            retaining_configs: self.retaining_configs.clone(),
            root_files_watch: self.root_files_watch.clone(),
        }
    }
}

impl ConfigFileNames {
    pub fn clone_names(&self) -> ConfigFileNames { ::tsox_core::fntrace::enter("clone_names"); 
        ConfigFileNames {
            nearest_config_file_name: self.nearest_config_file_name.clone(),
            ancestors: self.ancestors.clone(),
        }
    }
}

pub fn new_config_file_entry(
    has_relative_pattern_capability: bool,
    file_name: &str,
) -> ConfigFileEntry { ::tsox_core::fntrace::enter("new_config_file_entry"); 
    ConfigFileEntry::new(has_relative_pattern_capability, file_name.to_string())
}

pub fn new_extended_config_file_entry(
    file_name: &str,
    extending_config_path: Path,
) -> ConfigFileEntry { ::tsox_core::fntrace::enter("new_extended_config_file_entry"); 
    let mut entry = ConfigFileEntry::new(true, file_name.to_string());
    entry.retaining_configs.insert(extending_config_path, ());
    entry
}

use tsox_compile::compiler::CompilerHost;
use tsox_frontend::ast::SourceFile;
use tsox_tsoptions::vfs::FS;

use super::super::compiler_host::CompilerHostImpl;
use super::m5e_7::new_source_fs;

pub fn new_compiler_host(
    current_directory: String,
    project: Arc<Project>,
    builder: Arc<ProjectCollectionBuilder>,
    fs: Arc<dyn FS>,
    source: Arc<dyn FileSource>,
    to_path: Arc<dyn Fn(&str) -> Path + Send + Sync>,
    logger: Option<Arc<LogTree>>,
) -> CompilerHostImpl { ::tsox_core::fntrace::enter("new_compiler_host"); 
    let mut host = CompilerHostImpl::new(
        current_directory,
        project.config_file_path.clone(),
        builder.session_options.clone(),
        fs,
    );
    host.set_source_fs(new_source_fs(true, source, to_path));
    host.set_builder(Arc::clone(&builder));
    host.set_project(project);
    host.set_logger(logger);
    host
}

impl CompilerHostImpl {
    pub fn get_resolved_project_reference(
        &self,
        file_name: &str,
        path: &Path,
    ) -> Option<ParsedCommandLine> { ::tsox_core::fntrace::enter("get_resolved_project_reference"); 
        if !self.has_builder() {
            return self
                .config_file_registry()
                .and_then(|registry| registry.get_config(path).cloned());
        }
        self.source_fs().track(file_name);
        self.config_file_registry()
            .and_then(|registry| registry.get_config(path).cloned())
    }

    pub fn get_source_file(
        &self,
        opts: &SourceFileParseOptions,
    ) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        self.ensure_alive();
        let fh = self
            .source_fs()
            .get_file_by_path(&opts.file_name, &opts.path)?;
        let content = fh.content().to_string();
        Some(Arc::new(tsox_frontend::parser::Parser::parse_source_file_text(
            &opts.file_name,
            content,
        )))
    }

    pub fn get_content_mapped_source_files(
        &self,
        parse_options: &SourceFileParseOptions,
        mapper: &Mapper,
    ) -> Result<SourceFiles, TransformError> { ::tsox_core::fntrace::enter("get_content_mapped_source_files"); 
        self.ensure_alive();
        let Some(fh) = self
            .source_fs()
            .get_file_by_path(&parse_options.file_name, &parse_options.path)
        else {
            return Ok(SourceFiles::default());
        };
        self.ensure_content_mapper_project();
        let Some(project) = self.content_mapper_project() else {
            return Err(new_transform_error(
                TransformErrorKind::Project,
                "content mapper project is unavailable".into(),
            ));
        };
        let _identity = project
            .identity(mapper)
            .map_err(|err| new_transform_error(TransformErrorKind::Project, err))?;
        let files = transform_and_parse(
            tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions {
                file_name: parse_options.file_name.clone(),
                path: parse_options.path.0.clone(),
                external_module_indicator_options: Default::default(),
            },
            fh.content(),
            mapper,
            project.as_ref(),
        )?;
        if let Some(canonical) = files.canonical.as_ref() {
            bind_source_file(canonical);
        }
        for supplemental in &files.supplemental {
            bind_source_file(supplemental);
        }
        let fs = self.fs_arc();
        match check_supplemental_file_name_collisions(&files, &move |p: &str| fs.file_exists(p)) {
            Ok(()) => Ok(files),
            Err(err) => Err(new_transform_error(
                TransformErrorKind::Unknown,
                Box::new(err),
            )),
        }
    }

    pub fn ensure_content_mapper_project(&self) { ::tsox_core::fntrace::enter("ensure_content_mapper_project"); 
        self.init_content_mapper_project(|| None);
    }

    pub fn content_mapper_project_handle(&self) -> Option<Arc<dyn MapperProject>> { ::tsox_core::fntrace::enter("content_mapper_project_handle"); 
        self.content_mapper_project()
    }

    pub fn trace(&self, msg: &Message, args: &[String]) { ::tsox_core::fntrace::enter("trace"); 
        if let Some(logger) = self.logger() {
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            logger.log(&msg.localize(&Locale::default_locale(), &args));
        }
    }
}
