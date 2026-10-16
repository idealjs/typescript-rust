#![allow(dead_code)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};

use tsox_compile::compiler::Program;
use tsox_core::collections::set::Set;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options_kinds::{
    JsxEmit, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use tsox_core::core::mig::m3k;
use tsox_core::core::project_reference::ProjectReference;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath::{self, ComparePathsOptions, Path};
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use crate::lsp::lsproto;
use crate::project::ata_discover_typings as ata;
use tsox_frontend::ast;
use crate::project::checker_pool::CheckerPool;
use crate::project::compiler_host::{CompilerHost, CompilerHostImpl};
use crate::project::logging_log_tree::LogTree;
use crate::project::project::{Kind, ProgramUpdateKind, Project, INFERRED_PROJECT_NAME, HR};
use crate::project::project_collection_builder::ProjectCollectionBuilder;
use crate::project::watch::{PatternsAndIgnored, WatchedFiles};

pub type SyncSetOfPaths = Set<Path>;

#[derive(Clone, Default)]
pub struct ProjectExt {
    pub host: Option<Arc<CompilerHostImpl>>,
    pub program_files_watch: Option<Arc<WatchedFiles<SyncSetOfPaths>>>,
    pub typings_watch: Option<Arc<WatchedFiles<PatternsAndIgnored>>>,
    pub content_mapper_watch: Option<Arc<WatchedFiles<Vec<String>>>>,
    pub content_mapper_watched_files: Option<Set<Path>>,
    pub checker_pool: Option<Arc<CheckerPool>>,
    pub installed_typings_info: Option<ata::TypingsInfo>,
}

fn project_ext_map() -> &'static Mutex<HashMap<Path, ProjectExt>> { ::tsox_core::fntrace::enter("project_ext_map"); 
    static MAP: OnceLock<Mutex<HashMap<Path, ProjectExt>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn new_configured_project(
    config_file_name: String,
    config_file_path: Path,
    builder: &ProjectCollectionBuilder,
    logger: Option<&LogTree>,
) -> Project { ::tsox_core::fntrace::enter("new_configured_project"); 
    let _ = config_file_path;
    let directory = tsox_core::tspath::get_directory_path(&config_file_name);
    new_project(
        config_file_name,
        Kind::Configured,
        directory,
        builder,
        logger,
    )
}

pub fn new_inferred_project(
    current_directory: String,
    compiler_options: Option<CompilerOptions>,
    root_file_names: Vec<String>,
    project_references: Vec<ProjectReference>,
    content_mappers: Vec<tsox_compile::mig::m3l_cm::Mapper>,
    builder: &ProjectCollectionBuilder,
    logger: Option<&LogTree>,
) -> Project { ::tsox_core::fntrace::enter("new_inferred_project"); 
    let mut p = new_project(
        INFERRED_PROJECT_NAME.to_string(),
        Kind::Inferred,
        current_directory.clone(),
        builder,
        logger,
    );
    let compiler_options = compiler_options.unwrap_or_else(|| CompilerOptions {
        allow_js: Tristate::True,
        module: ModuleKind::ESNext,
        module_resolution: ModuleResolutionKind::Bundler,
        target: ScriptTarget::ESNext,
        jsx: JsxEmit::ReactJSX,
        allow_importing_ts_extensions: Tristate::True,
        strict_null_checks: Tristate::True,
        strict_function_types: Tristate::True,
        source_map: Tristate::True,
        allow_non_ts_extensions: Tristate::True,
        resolve_json_module: Tristate::True,
        ..Default::default()
    });
    p.command_line = Some(new_inferred_project_command_line(
        compiler_options,
        &root_file_names,
        &project_references,
        &content_mappers,
        ComparePathsOptions {
            current_directory: current_directory.clone(),
            ..Default::default()
        },
    ));
    p
}

pub fn new_inferred_project_command_line(
    compiler_options: CompilerOptions,
    root_file_names: &[String],
    project_references: &[ProjectReference],
    content_mappers: &[tsox_compile::mig::m3l_cm::Mapper],
    compare_paths_options: tsox_core::tspath::ComparePathsOptions,
) -> ParsedCommandLine { ::tsox_core::fntrace::enter("new_inferred_project_command_line"); 
    let mut command_line = tsox_tsoptions::mig::m5h_3::new_parsed_command_line(
        compiler_options,
        root_file_names.to_vec(),
        project_references.to_vec(),
        compare_paths_options,
    );
    command_line.content_mappers = to_tsoptions_content_mappers(content_mappers);
    command_line
}

fn to_tsoptions_content_mappers(
    mappers: &[tsox_compile::mig::m3l_cm::Mapper],
) -> Vec<tsox_tsoptions::mig::m5h_3::ContentMapper> { ::tsox_core::fntrace::enter("to_tsoptions_content_mappers"); 
    mappers
        .iter()
        .map(|m| tsox_tsoptions::mig::m5h_3::ContentMapper {
            definition: tsox_tsoptions::mig::m5h_3::ContentMapperDefinition {
                package: m.definition.package.clone(),
                extensions: m.definition.extensions.clone(),
                options: m.definition.options.clone(),
            },
            manifest: tsox_tsoptions::mig::m5h_2::ContentMapperManifest {
                name: m.manifest.name.clone(),
                version: m.manifest.version.clone(),
                exec: m.manifest.exec.clone(),
                compiler_options: m.manifest.compiler_options.clone(),
                dynamic_config: m.manifest.dynamic_config,
            },
            package_directory: m.package_directory.clone(),
            contribution_id: m.contribution_id.clone(),
        })
        .collect()
}

pub fn new_inferred_project_from_project(
    project: &Project,
    builder: &ProjectCollectionBuilder,
    logger: Option<&LogTree>,
) -> Project { ::tsox_core::fntrace::enter("new_inferred_project_from_project"); 
    let mut inferred = new_project(
        INFERRED_PROJECT_NAME.to_string(),
        Kind::Inferred,
        project.current_directory.clone(),
        builder,
        logger,
    );
    inferred.command_line = project
        .program
        .as_ref()
        .map(|p| p.command_line().clone());
    inferred.program = project.program.clone();
    inferred.program_last_update = project.program_last_update;
    let src = project.ext();
    let mut ext = inferred.ext();
    ext.host = src.host.clone();
    ext.checker_pool = src.checker_pool.clone();
    ext.content_mapper_watched_files = src.content_mapper_watched_files.clone();
    inferred.set_ext(ext);
    inferred.dirty = false;
    inferred
}

fn default_use_case_sensitive_file_names() -> bool { ::tsox_core::fntrace::enter("default_use_case_sensitive_file_names"); 
    ComparePathsOptions::default().use_case_sensitive_file_names
}

fn resolved_relative_pattern_support() -> bool { ::tsox_core::fntrace::enter("resolved_relative_pattern_support"); 
    let ctx = crate::mig::m5m::ResolvedClientCapabilitiesContext {
        capabilities: None,
    };
    let caps = crate::mig::m5m::get_client_capabilities(&ctx);
    caps.raw
        .get("workspace")
        .and_then(|w| w.get("didChangeWatchedFiles"))
        .and_then(|d| d.get("relativePatternSupport"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub fn new_project(
    config_file_name: String,
    kind: Kind,
    current_directory: String,
    builder: &ProjectCollectionBuilder,
    logger: Option<&LogTree>,
) -> Project { ::tsox_core::fntrace::enter("new_project"); 
    if let Some(logger) = logger {
        logger.log(&format!(
            "Creating {}Project: {}, currentDirectory: {}",
            kind, config_file_name, current_directory
        ));
    }
    let use_case_sensitive = default_use_case_sensitive_file_names();
    let mut project = Project::new(config_file_name.clone(), kind, current_directory.clone());
    project.config_file_path = tsox_core::tspath::to_path(
        &config_file_name,
        &current_directory,
        use_case_sensitive,
    );
    let relative_pattern_support = resolved_relative_pattern_support();
    let mut ext = ProjectExt {
        host: None,
        program_files_watch: Some(Arc::new(WatchedFiles::new(
            &format!("program files for {}", config_file_name),
            lsproto::WATCH_KIND_CREATE
                | lsproto::WATCH_KIND_CHANGE
                | lsproto::WATCH_KIND_DELETE,
            relative_pattern_support,
            create_resolution_lookup_glob_mapper(
                &builder.session_options.current_directory,
                &builder.session_options.default_library_path,
                &project.current_directory,
                use_case_sensitive,
            ),
        ))),
        typings_watch: None,
        content_mapper_watch: None,
        content_mapper_watched_files: None,
        checker_pool: None,
        installed_typings_info: None,
    };
    if !builder.session_options.typings_location.is_empty() {
        ext.typings_watch = Some(Arc::new(WatchedFiles::new(
            "typings installer files",
            lsproto::WATCH_KIND_CREATE | lsproto::WATCH_KIND_CHANGE | lsproto::WATCH_KIND_DELETE,
            relative_pattern_support,
            core_identity,
        )));
    }
    ext.content_mapper_watch = Some(Arc::new(new_watched_files_for_paths(
        format!(
            "content mapper configuration files for {}",
            config_file_name
        ),
        lsproto::WATCH_KIND_CREATE | lsproto::WATCH_KIND_CHANGE | lsproto::WATCH_KIND_DELETE,
        relative_pattern_support,
        builder.session_options.current_directory.clone(),
        builder.session_options.current_directory.clone(),
        use_case_sensitive,
    )));
    project.set_ext(ext);
    project
}

fn new_watched_files_for_paths(
    name: String,
    watch_kind: lsproto::WatchKind,
    has_relative_pattern_capability: bool,
    workspace_directory: String,
    _current_directory: String,
    _use_case_sensitive_file_names: bool,
) -> WatchedFiles<Vec<String>> { ::tsox_core::fntrace::enter("new_watched_files_for_paths"); 
    let workspace = Path(workspace_directory);
    WatchedFiles::new(
        &name,
        watch_kind,
        has_relative_pattern_capability,
        move |files: &Vec<String>| {
            let mut result = PatternsAndIgnored::default();
            for file in files {
                if workspace.contains_path(&Path(file.clone())) {
                    result.patterns_inside_workspace.push(file.clone());
                } else {
                    result.directories_outside_workspace
                        .push(tsox_core::tspath::get_directory_path(file));
                }
            }
            result
        },
    )
}

impl Project {
    pub fn current_directory(&self) -> &str { ::tsox_core::fntrace::enter("current_directory"); 
        &self.current_directory
    }

    pub fn config_file_name(&self) -> &str { ::tsox_core::fntrace::enter("config_file_name"); 
        if self.kind != Kind::Configured {
            panic!("ConfigFileName called on non-configured project");
        }
        &self.config_file_name
    }

    pub fn ext(&self) -> ProjectExt { ::tsox_core::fntrace::enter("ext"); 
        project_ext_map()
            .lock()
            .unwrap()
            .get(&self.config_file_path)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_ext(&self, ext: ProjectExt) { ::tsox_core::fntrace::enter("set_ext"); 
        project_ext_map()
            .lock()
            .unwrap()
            .insert(self.config_file_path.clone(), ext);
    }

    pub fn host(&self) -> Option<Arc<CompilerHostImpl>> { ::tsox_core::fntrace::enter("host"); 
        self.ext().host
    }

    pub fn checker_pool(&self) -> Option<Arc<CheckerPool>> { ::tsox_core::fntrace::enter("checker_pool"); 
        self.ext().checker_pool
    }

    pub fn set_checker_pool(&self, pool: Option<Arc<CheckerPool>>) { ::tsox_core::fntrace::enter("set_checker_pool"); 
        let mut ext = self.ext();
        ext.checker_pool = pool;
        self.set_ext(ext);
    }

    pub fn program_files_watch(&self) -> Option<Arc<WatchedFiles<SyncSetOfPaths>>> { ::tsox_core::fntrace::enter("program_files_watch"); 
        self.ext().program_files_watch
    }

    pub fn get_project_diagnostics(&self) -> Vec<Arc<ast::Diagnostic>> { ::tsox_core::fntrace::enter("get_project_diagnostics"); 
        let global_diags = match &self.checker_pool() {
            Some(_pool) => {
                todo!("CheckerPool::get_global_diagnostics 未移植：Rust CheckerPool 无该公开方法")
            }
            None => Vec::new(),
        };
        let program = self
            .program
            .as_ref()
            .expect("GetProjectDiagnostics requires program");
        tsox_compile::mig::m4w_4::sort_and_deduplicate_diagnostics(
            program
                .get_config_file_parsing_diagnostics()
                .into_iter()
                .chain(program.get_program_diagnostics())
                .chain(global_diags)
                .collect(),
        )
    }

    pub fn has_file_in_program(&self, file_name: &str) -> bool { ::tsox_core::fntrace::enter("has_file_in_program"); 
        self.contains_file_in_program(&self.to_path(file_name))
    }

    pub fn contains_file_in_program(&self, path: &Path) -> bool { ::tsox_core::fntrace::enter("contains_file_in_program"); 
        match &self.program {
            Some(program) => program.get_source_file_by_path(&path.0).is_some(),
            None => false,
        }
    }

    pub fn is_source_from_project_reference_in_program(&self, path: &Path) -> bool { ::tsox_core::fntrace::enter("is_source_from_project_reference_in_program"); 
        match &self.program {
            Some(program) => program.is_source_from_project_reference(&path.0),
            None => false,
        }
    }

    pub fn clone_project(&self) -> Project { ::tsox_core::fntrace::enter("clone_project"); 
        self.clone_shallow()
    }

    pub fn get_command_line_with_typings_files(&self) -> Option<ParsedCommandLine> { ::tsox_core::fntrace::enter("get_command_line_with_typings_files"); 
        if self.typings_files.is_empty() {
            return self.command_line.clone();
        }
        match self.get_type_acquisition() {
            Some(ta) if ta.enable == Tristate::True => {}
            _ => return self.command_line.clone(),
        }
        let command_line = self.command_line.as_ref().expect("command line required");
        let original_root_names = command_line.file_names();
        let mut new_root_names =
            Vec::with_capacity(original_root_names.len() + self.typings_files.len());
        new_root_names.extend(original_root_names.iter().cloned());
        new_root_names.extend(self.typings_files.iter().cloned());
        Some(command_line.with_file_names(new_root_names))
    }

    pub fn set_potential_project_reference(&mut self, config_file_path: Path) { ::tsox_core::fntrace::enter("set_potential_project_reference"); 
        match &mut self.potential_project_references {
            None => {
                let mut set = HashSet::new();
                set.insert(config_file_path);
                self.potential_project_references = Some(set);
            }
            Some(existing) => {
                existing.insert(config_file_path);
            }
        }
    }

    pub fn has_potential_project_reference(
        &self,
        project_tree_request: &ProjectTreeRequest,
    ) -> bool { ::tsox_core::fntrace::enter("has_potential_project_reference"); 
        if let Some(command_line) = &self.command_line {
            let mut command_line = command_line.clone();
            for path in command_line.resolved_project_reference_paths() {
                if project_tree_request.is_project_referenced(&self.to_path(&path)) {
                    return true;
                }
            }
        } else if let Some(potential) = &self.potential_project_references {
            for path in potential {
                if project_tree_request.is_project_referenced(path) {
                    return true;
                }
            }
        }
        false
    }

    pub fn clone_watchers(&self) -> WatchedFiles<SyncSetOfPaths> { ::tsox_core::fntrace::enter("clone_watchers"); 
        todo!("host.source_fs.seen_files 未接线：snapshotFS 尚未并入 Project host")
    }

    pub fn log(&self, _msg: &str) { ::tsox_core::fntrace::enter("log"); }

    pub fn to_path(&self, file_name: &str) -> Path { ::tsox_core::fntrace::enter("to_path"); 
        let use_case_sensitive = self
            .host()
            .map(|h| h.fs().use_case_sensitive_file_names())
            .unwrap_or_else(default_use_case_sensitive_file_names);
        tsox_core::tspath::to_path(
            file_name,
            &self.current_directory,
            use_case_sensitive,
        )
    }

    pub fn print(
        &self,
        write_file_names: bool,
        write_file_explanation: bool,
        builder: &mut String,
    ) -> String { ::tsox_core::fntrace::enter("print"); 
        let _ = write_file_explanation;
        builder.push_str(&format!("\nProject '{}'\n", self.name()));
        match &self.program {
            None => builder.push_str("\tFiles (0) NoProgram\n"),
            Some(program) => {
                let source_files = program.get_source_files();
                builder.push_str(&format!("\tFiles ({})\n", source_files.len()));
                if write_file_names {
                    for source_file in source_files {
                        builder.push_str("\t\t");
                        builder.push_str(&source_file.file_name);
                        builder.push('\n');
                    }
                }
            }
        }
        builder.push_str(HR);
        builder.clone()
    }

    pub fn get_type_acquisition(&self) -> Option<m3k::TypeAcquisition> { ::tsox_core::fntrace::enter("get_type_acquisition"); 
        if self.kind == Kind::Inferred {
            return Some(m3k::TypeAcquisition {
                enable: Tristate::True,
                include: Vec::new(),
                exclude: Vec::new(),
                disable_filename_based_type_acquisition: Tristate::False,
            });
        }
        self.command_line
            .as_ref()
            .and_then(|cl| cl.type_acquisition())
            .cloned()
    }

    pub fn get_unresolved_imports(&self) -> Option<Set<String>> { ::tsox_core::fntrace::enter("get_unresolved_imports"); 
        self.program.as_ref().map(|p| {
            let mut set = Set::new();
            for item in p.get_unresolved_imports() {
                set.insert(item);
            }
            set
        })
    }

    pub fn should_trigger_ata(&self, snapshot_id: u64) -> bool { ::tsox_core::fntrace::enter("should_trigger_ata"); 
        if self.program.is_none() || self.command_line.is_none() {
            return false;
        }
        match self.get_type_acquisition() {
            Some(ta) if ta.enable == Tristate::True => {}
            _ => return false,
        }
        match &self.ext().installed_typings_info {
            None => true,
            Some(info) => {
                if self.program_last_update == snapshot_id
                    && self.program_update_kind == ProgramUpdateKind::NewFiles
                {
                    return true;
                }
                !typings_info_equal(info, &self.compute_typings_info())
            }
        }
    }

    pub fn compute_typings_info(&self) -> ata::TypingsInfo { ::tsox_core::fntrace::enter("compute_typings_info"); 
        ata::TypingsInfo {
            compiler_options: self
                .command_line
                .as_ref()
                .map(|cl| cl.compiler_options().clone())
                .unwrap_or_default(),
            type_acquisition: self.get_type_acquisition().as_ref().map(to_ata_type_acquisition),
            unresolved_imports: self.get_unresolved_imports(),
        }
    }
}

fn to_ata_type_acquisition(ta: &m3k::TypeAcquisition) -> ata::TypeAcquisition { ::tsox_core::fntrace::enter("to_ata_type_acquisition"); 
    ata::TypeAcquisition {
        enable: ta.enable == Tristate::True,
        include: if ta.include.is_empty() {
            None
        } else {
            Some(ta.include.clone())
        },
        exclude: ta.exclude.clone(),
        disable_filename_based_type_acquisition: ta.disable_filename_based_type_acquisition,
    }
}

fn typings_info_equal(a: &ata::TypingsInfo, b: &ata::TypingsInfo) -> bool { ::tsox_core::fntrace::enter("typings_info_equal"); 
    format!("{:?}", a) == format!("{:?}", b)
}

use crate::project::snapshot::ProjectTreeRequest;

fn create_resolution_lookup_glob_mapper(
    _session_current_directory: &str,
    _default_library_path: &str,
    _project_current_directory: &str,
    _use_case_sensitive_file_names: bool,
) -> Box<dyn Fn(&SyncSetOfPaths) -> PatternsAndIgnored + Send + Sync> { ::tsox_core::fntrace::enter("create_resolution_lookup_glob_mapper"); 
    Box::new(|_seen_files: &SyncSetOfPaths| PatternsAndIgnored::default())
}

fn core_identity(patterns: &PatternsAndIgnored) -> PatternsAndIgnored { ::tsox_core::fntrace::enter("core_identity"); 
    patterns.clone()
}
