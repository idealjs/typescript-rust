#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Once};
use std::time::SystemTime;

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath::{self, Path};
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use tsox_frontend::ast::Diagnostic;
use tsox_core::diagnostics::messages_generated as dg;
use tsox_core::diagnostics::Message;

use super::m4x_build_2::CompilerHost;
use super::m4x_build_3::Orchestrator;
use tsox_compile::mig::m3l_cm_2::Host as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildKind {
    #[default]
    None,
    Pseudo,
    Program,
}

#[derive(Clone)]
pub struct UpstreamTask {
    pub task: Arc<Mutex<BuildTask>>,
    pub ref_index: usize,
}

#[derive(Clone)]
pub struct BuildInfoEntry {
    pub build_info: Option<crate::mig::m4y_2::BuildInfo>,
    pub path: Path,
    pub m_time: SystemTime,
    pub dts_time: Option<SystemTime>,
}

pub struct TaskResult {
    pub builder: String,
    pub report_status: Option<Box<dyn FnMut(&Arc<Diagnostic>) + Send>>,
    pub diagnostic_reporter: Option<Box<dyn FnMut(&Arc<Diagnostic>) + Send>>,
    pub exit_status: crate::execute::ExitStatus,
    pub statistics: Option<crate::mig::m5a_3::Statistics>,
    pub program: Option<crate::mig::m4z2::Program>,
    pub build_kind: BuildKind,
    pub files_to_delete: Vec<String>,
}

impl Default for TaskResult {
    fn default() -> Self {
        Self {
            builder: String::new(),
            report_status: None,
            diagnostic_reporter: None,
            exit_status: crate::execute::ExitStatus::Success,
            statistics: None,
            program: None,
            build_kind: BuildKind::None,
            files_to_delete: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpToDateStatusType {
    UpToDate,
    UpToDateWithUpstreamTypes,
    UpToDateWithInputFileText,
    UpstreamErrors,
    Solution,
    ConfigFileNotFound,
    BuildErrors,
    InputFileMissing,
    OutputMissing,
    InputFileNewer,
    OutOfDateBuildInfoWithPendingEmit,
    OutOfDateBuildInfoWithErrors,
    OutOfDateOptions,
    OutOfDateRoots,
    TsVersionOutputOfDate,
    ForceBuild,
}

#[derive(Clone)]
pub enum UpToDateStatusData {
    None,
    Text(String),
    InputOutputName(InputOutputName),
    InputOutputFileAndTime(InputOutputFileAndTime),
    UpstreamErrors(UpstreamErrors),
}

#[derive(Clone)]
pub struct InputOutputName {
    pub input: String,
    pub output: String,
}

#[derive(Clone)]
pub struct FileAndTime {
    pub file: String,
    pub time: SystemTime,
}

#[derive(Clone)]
pub struct InputOutputFileAndTime {
    pub input: FileAndTime,
    pub output: FileAndTime,
    pub build_info_path: String,
}

#[derive(Clone)]
pub struct UpstreamErrors {
    pub r#ref: String,
    pub ref_has_upstream_errors: bool,
}

#[derive(Clone)]
pub struct UpToDateStatus {
    pub kind: UpToDateStatusType,
    pub data: UpToDateStatusData,
}

impl UpToDateStatus {
    pub fn new(kind: UpToDateStatusType) -> Self {
        Self {
            kind,
            data: UpToDateStatusData::None,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(
            self.kind,
            UpToDateStatusType::UpstreamErrors
                | UpToDateStatusType::OutOfDateBuildInfoWithErrors
                | UpToDateStatusType::OutOfDateBuildInfoWithPendingEmit
                | UpToDateStatusType::OutOfDateOptions
                | UpToDateStatusType::OutOfDateRoots
                | UpToDateStatusType::TsVersionOutputOfDate
                | UpToDateStatusType::ConfigFileNotFound
                | UpToDateStatusType::InputFileMissing
                | UpToDateStatusType::InputFileNewer
                | UpToDateStatusType::OutputMissing
        )
    }

    pub fn is_pseudo_build(&self) -> bool {
        matches!(
            self.kind,
            UpToDateStatusType::UpToDateWithInputFileText | UpToDateStatusType::OutOfDateRoots
        )
    }

    pub fn upstream_errors(&self) -> Option<&UpstreamErrors> {
        match &self.data {
            UpToDateStatusData::UpstreamErrors(errors) => Some(errors),
            _ => None,
        }
    }

    pub fn input_output_name(&self) -> Option<&InputOutputName> {
        match &self.data {
            UpToDateStatusData::InputOutputName(name) => Some(name),
            _ => None,
        }
    }

    pub fn input_output_file_and_time(&self) -> Option<&InputOutputFileAndTime> {
        match &self.data {
            UpToDateStatusData::InputOutputFileAndTime(value) => Some(value),
            _ => None,
        }
    }

    pub fn oldest_output_file_name(&self) -> String {
        match self.data.clone() {
            UpToDateStatusData::InputOutputName(name) => name.output,
            UpToDateStatusData::InputOutputFileAndTime(value) => value.output.file,
            _ => String::new(),
        }
    }
}

pub struct BuildTask {
    pub config: String,
    pub resolved: Option<Arc<ParsedCommandLine>>,
    pub up_stream: Vec<UpstreamTask>,
    pub down_stream: Vec<Arc<Mutex<BuildTask>>>,
    pub status: Option<UpToDateStatus>,
    pub done: std::sync::mpsc::Receiver<()>,

    pub result: Option<TaskResult>,
    pub prev_reporter: Option<Arc<Mutex<BuildTask>>>,
    pub report_done: std::sync::mpsc::Receiver<()>,

    pub build_info_entry: Option<BuildInfoEntry>,
    pub package_jsons: Vec<String>,

    pub errors: Vec<Arc<Diagnostic>>,
    pub pending: std::sync::atomic::AtomicBool,
    pub is_initial_cycle: bool,
    pub dirty: bool,

    pub content_mapper_project_once: Once,
    pub content_mapper_project: Option<Arc<dyn tsox_compile::mig::m3l_cm_2::Project>>,
    pub content_mapper_project_err: Option<tsox_compile::mig::m4v_3::ContentMapperError>,
}

impl BuildTask {
    pub fn get_content_mapper_project(
        &mut self,
        orchestrator: &mut Orchestrator,
    ) -> Result<Option<Arc<dyn tsox_compile::mig::m3l_cm_2::Project>>, tsox_compile::mig::m4v_3::ContentMapperError>
    {
        self.content_mapper_project_once.call_once(|| {
            if orchestrator.content_mapper_host.is_none()
                || self.resolved.is_none()
                || self.resolved.as_ref().unwrap().content_mappers().is_empty()
            {
                return;
            }
            let resolved = self.resolved.as_ref().unwrap();
            self.content_mapper_project = orchestrator
                .content_mapper_host
                .as_mut()
                .unwrap()
                .project(&tsox_compile::mig::m3l_cm_2::ProjectSpec {
                    config_file_name: resolved.config_name(),
                    mappers: resolved
                        .content_mappers()
                        .iter()
                        .map(|mapper| Arc::new(content_mapper_to_mapper(mapper)))
                        .collect(),
                    compiler_options: Some(Arc::new(resolved.compiler_options().clone())),
                });
        });
        match self.content_mapper_project_err.as_ref() {
            Some(err) => Err(tsox_compile::mig::m4v_3::ContentMapperError(err.0.clone())),
            None => Ok(self.content_mapper_project.clone()),
        }
    }

    pub fn refresh_content_mapper_project(&mut self, _orchestrator: &mut Orchestrator) {
        if let Some(project) = self.content_mapper_project.as_ref() {
            self.content_mapper_project_err = project
                .refresh()
                .err()
                .map(|e| tsox_compile::mig::m4v_3::ContentMapperError(e.to_string()));
        }
    }

    pub fn wait_on_upstream(&self) {
        for upstream in &self.up_stream {
            drop(upstream.task.lock().unwrap());
        }
    }

    pub fn unblock_downstream(&mut self) {
        self.pending
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.is_initial_cycle = false;
    }

    pub fn report_diagnostic(&mut self, err: Arc<Diagnostic>) {
        self.errors.push(err.clone());
        if let Some(reporter) = self.result.as_mut().unwrap().diagnostic_reporter.as_mut() {
            reporter(&err);
        }
    }

    pub fn report(
        &mut self,
        orchestrator: &mut Orchestrator,
        config_path: Path,
        build_result: &mut OrchestratorResult,
    ) {
        if let Some(prev_reporter) = self.prev_reporter.take() {
            drop(prev_reporter.lock().unwrap());
        }
        if !self.errors.is_empty() {
            build_result
                .errors
                .extend(self.errors.iter().cloned());
        }
        orchestrator
            .opts
            .sys
            .writer()
            .write_all(self.result.as_ref().unwrap().builder.as_bytes())
            .ok();
        if self.result.as_ref().unwrap().exit_status > build_result.result.status {
            build_result.result.status = self.result.as_ref().unwrap().exit_status;
        }
        if let Some(statistics) = self.result.as_ref().unwrap().statistics.as_ref() {
            build_result.statistics.aggregate(statistics);
        }
        match self.result.as_ref().unwrap().build_kind {
            BuildKind::Program => {
                if orchestrator.opts.testing.is_some() {
                    orchestrator
                        .opts
                        .testing
                        .as_mut()
                        .unwrap()
                        .on_program(self.result.as_ref().unwrap().program.as_ref().unwrap());
                }
                build_result.statistics.projects_built += 1;
            }
            BuildKind::Pseudo => {
                build_result.statistics.timestamp_updates += 1;
            }
            BuildKind::None => {}
        }
        build_result
            .files_to_delete
            .extend(self.result.as_ref().unwrap().files_to_delete.iter().cloned());
        self.result = None;
    }

    pub fn build_project(&mut self, orchestrator: &mut Orchestrator, path: Path) {
        self.wait_on_upstream();
        if self
            .pending
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            self.status = Some(self.get_up_to_date_status(orchestrator, path.clone()));
            self.report_up_to_date_status(orchestrator);
            if !self.handle_status_that_doesnt_require_build(orchestrator) {
                self.compile_and_emit(orchestrator, path.clone());
                self.update_downstream(orchestrator, path.clone());
            } else {
                if let Some(resolved) = self.resolved.as_ref() {
                    for diagnostic in resolved.get_config_file_parsing_diagnostics() {
                        self.report_diagnostic(Arc::new(diagnostic));
                    }
                }
                if !self.errors.is_empty() {
                    self.result.as_mut().unwrap().exit_status =
                        crate::execute::ExitStatus::DiagnosticsPresent_OutputsSkipped;
                }
            }
        } else {
            if !self.errors.is_empty() {
                self.report_up_to_date_status(orchestrator);
                for err in &self.errors {
                    if let Some(reporter) =
                        self.result.as_mut().unwrap().diagnostic_reporter.as_mut()
                    {
                        reporter(err);
                    }
                }
            }
        }
        self.unblock_downstream();
    }

    pub fn update_downstream(&mut self, orchestrator: &mut Orchestrator, path: Path) {
        if self.is_initial_cycle {
            return;
        }
        if orchestrator
            .opts
            .command
            .build_options
            .stop_build_on_errors
            == Tristate::True
            && self.status.as_ref().is_some_and(|status| status.is_error())
        {
            return;
        }
        let has_program = self
            .result
            .as_ref()
            .and_then(|result| result.program.as_ref())
            .is_some();
        if !has_program {
            for down_stream in &self.down_stream {
                let mut down_stream = down_stream.lock().unwrap();
                down_stream.reset_status();
                down_stream
                    .pending
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
            return;
        }

        let has_changed_dts_file = self
            .result
            .as_ref()
            .unwrap()
            .program
            .as_ref()
            .unwrap()
            .has_changed_dts_file();
        for down_stream in &self.down_stream {
            let mut down_stream = down_stream.lock().unwrap();
            if let Some(status) = down_stream.status.as_ref() {
                match status.kind {
                    UpToDateStatusType::UpToDate => {
                        if !has_changed_dts_file {
                            down_stream.status = Some(UpToDateStatus {
                                kind: UpToDateStatusType::UpToDateWithUpstreamTypes,
                                data: status.data.clone(),
                            });
                        } else {
                            down_stream.status = Some(UpToDateStatus {
                                kind: UpToDateStatusType::InputFileNewer,
                                data: UpToDateStatusData::InputOutputName(InputOutputName {
                                    input: self.config.clone(),
                                    output: down_stream
                                        .status
                                        .as_ref()
                                        .unwrap()
                                        .oldest_output_file_name(),
                                }),
                            });
                        }
                    }
                    UpToDateStatusType::UpToDateWithUpstreamTypes
                    | UpToDateStatusType::UpToDateWithInputFileText => {
                        if has_changed_dts_file {
                            down_stream.status = Some(UpToDateStatus {
                                kind: UpToDateStatusType::InputFileNewer,
                                data: UpToDateStatusData::InputOutputName(InputOutputName {
                                    input: self.config.clone(),
                                    output: down_stream
                                        .status
                                        .as_ref()
                                        .unwrap()
                                        .oldest_output_file_name(),
                                }),
                            });
                        }
                    }
                    UpToDateStatusType::UpstreamErrors => {
                        let upstream_errors = down_stream
                            .status
                            .as_ref()
                            .unwrap()
                            .upstream_errors()
                            .unwrap()
                            .r#ref
                            .clone();
                        let ref_config =
                            tsox_core::core::mig::m3j_3::resolve_config_file_name_of_project_reference(
                                &upstream_errors,
                            );
                        if orchestrator.to_path(&ref_config) == path {
                            down_stream.reset_status();
                        }
                    }
                    _ => {}
                }
            }
            down_stream
                .pending
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    pub fn compile_and_emit(&mut self, orchestrator: &mut Orchestrator, path: Path) {
        self.errors.clear();
        if orchestrator.opts.command.build_options.verbose == Tristate::True {
            self.report_status(
                orchestrator,
                new_compiler_diagnostic(
                    &dg::BUILDING_PROJECT_0,
                    &[orchestrator.relative_file_name(&self.config)],
                ),
            );
        }

        let mut compile_times = crate::mig::m5a_2::CompileTimes::default();
        let config_time = orchestrator
            .host
            .config_times
            .load(&path)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        compile_times.config_time = config_time;
        let build_info_read_start = SystemTime::now();
        let content_mapper_project = match self.get_content_mapper_project(orchestrator) {
            Ok(content_mapper_project) => content_mapper_project,
            Err(err) => {
                self.report_diagnostic(Arc::new(
                    tsox_compile::mig::m4v_2::content_mapper_project_diagnostic(
                        &tsox_compile::mig::m4v_2::ContentMapperError::from(err),
                    ),
                ));
                self.status = Some(UpToDateStatus::new(UpToDateStatusType::BuildErrors));
                self.result.as_mut().unwrap().exit_status =
                    crate::execute::ExitStatus::DiagnosticsPresent_OutputsSkipped;
                return;
            }
        };
        let compiler_host = Arc::new(CompilerHost::new(
            orchestrator.host.clone(),
            crate::mig::m5a::get_trace_from_sys(
                orchestrator.opts.sys.as_ref(),
                Some(orchestrator.opts.command.locale()),
                orchestrator.opts.testing.as_deref(),
            ),
            content_mapper_project.clone(),
        ));
        let old_program: Option<crate::mig::m4z2::Program> =
            if orchestrator.opts.command.build_options.force != Tristate::True {
                self.result
                    .as_mut()
                    .and_then(|result| result.program.take())
            } else {
                None
            };
        compile_times.build_info_read_time = SystemTime::now()
            .duration_since(build_info_read_start)
            .unwrap_or_default()
            .as_secs_f64();
        let parse_start = SystemTime::now();
        let program = tsox_compile::compiler::new_program(tsox_compile::compiler::ProgramOptions {
            config: self.resolved.as_ref().unwrap().as_ref().clone(),
            host: compiler_host,
            use_source_of_project_reference: false,
            single_threaded: Tristate::Unknown,
            create_checker_pool: None,
            typings_location: String::new(),
            project_name: String::new(),
            tracing: None,
            skip_module_resolution: false,
        });
        compile_times.parse_time = SystemTime::now()
            .duration_since(parse_start)
            .unwrap_or_default()
            .as_secs_f64();
        let changes_compute_start = SystemTime::now();
        self.result.as_mut().unwrap().program =
            Some(crate::mig::m4z2::new_program(
                program,
                old_program.as_ref(),
                None,
                None,
                orchestrator.opts.testing.is_some(),
            ));
        compile_times.changes_compute_time = SystemTime::now()
            .duration_since(changes_compute_start)
            .unwrap_or_default()
            .as_secs_f64();
        self.package_jsons = self
            .result
            .as_ref()
            .unwrap()
            .program
            .as_ref()
            .unwrap()
            .package_json_lookup_paths();
        self.result.as_mut().unwrap().build_kind = BuildKind::Program;
    }

    pub fn report_status(&mut self, _orchestrator: &mut Orchestrator, diag: Arc<Diagnostic>) {
        if let Some(report_status) = self.result.as_mut().unwrap().report_status.as_mut() {
            report_status(&diag);
        }
    }

    pub fn handle_status_that_doesnt_require_build(
        &mut self,
        orchestrator: &mut Orchestrator,
    ) -> bool {
        let status = self.status.clone().unwrap();
        match status.kind {
            UpToDateStatusType::UpToDate => {
                if orchestrator.opts.command.build_options.dry == Tristate::True {
                    self.report_status(
                        orchestrator,
                        new_compiler_diagnostic(
                            &dg::PROJECT_0_IS_UP_TO_DATE,
                            &[self.config.clone()],
                        ),
                    );
                }
                true
            }
            UpToDateStatusType::UpstreamErrors => {
                let upstream_status = status.upstream_errors().unwrap().clone();
                if orchestrator.opts.command.build_options.verbose == Tristate::True {
                    self.report_status(
                        orchestrator,
                        new_compiler_diagnostic(
                            if upstream_status.ref_has_upstream_errors {
                                &dg::SKIPPING_BUILD_OF_PROJECT_0_BECAUSE_ITS_DEPENDENCY_1_WAS_NOT_BUILT
                            } else {
                                &dg::SKIPPING_BUILD_OF_PROJECT_0_BECAUSE_ITS_DEPENDENCY_1_HAS_ERRORS
                            },
                            &[
                                orchestrator.relative_file_name(&self.config),
                                orchestrator.relative_file_name(&upstream_status.r#ref),
                            ],
                        ),
                    );
                }
                true
            }
            UpToDateStatusType::Solution => true,
            UpToDateStatusType::ConfigFileNotFound => {
                self.report_diagnostic(new_compiler_diagnostic(
                    &dg::FILE_0_NOT_FOUND,
                    &[self.config.clone()],
                ));
                true
            }
            _ => {
                if status.is_pseudo_build() {
                    if orchestrator.opts.command.build_options.dry == Tristate::True {
                        self.report_status(
                            orchestrator,
                            new_compiler_diagnostic(
                                &dg::A_NON_DRY_BUILD_WOULD_UPDATE_TIMESTAMPS_FOR_OUTPUT_OF_PROJECT_0,
                                &[self.config.clone()],
                            ),
                        );
                        self.status = Some(UpToDateStatus::new(UpToDateStatusType::UpToDate));
                        return true;
                    }
                    self.update_time_stamps(
                        orchestrator,
                        &[],
                        &dg::UPDATING_OUTPUT_TIMESTAMPS_OF_PROJECT_0,
                    );
                    self.status = Some(UpToDateStatus {
                        kind: UpToDateStatusType::UpToDate,
                        data: status.data.clone(),
                    });
                    self.result.as_mut().unwrap().build_kind = BuildKind::Pseudo;
                    return true;
                }

                if orchestrator.opts.command.build_options.dry == Tristate::True {
                    self.report_status(
                        orchestrator,
                        new_compiler_diagnostic(
                            &dg::A_NON_DRY_BUILD_WOULD_BUILD_PROJECT_0,
                            &[self.config.clone()],
                        ),
                    );
                    self.status = Some(UpToDateStatus::new(UpToDateStatusType::UpToDate));
                    return true;
                }
                false
            }
        }
    }

    pub fn get_up_to_date_status(
        &mut self,
        orchestrator: &mut Orchestrator,
        config_path: Path,
    ) -> UpToDateStatus {
        if let Some(status) = self.status.as_ref() {
            return status.clone();
        }
        if self.resolved.is_none() {
            return UpToDateStatus::new(UpToDateStatusType::ConfigFileNotFound);
        }
        let resolved = self.resolved.as_ref().unwrap().clone();

        if resolved.file_names().is_empty() && !resolved.project_references().is_empty() {
            return UpToDateStatus::new(UpToDateStatusType::Solution);
        }

        for upstream in &self.up_stream {
            let upstream_task = upstream.task.lock().unwrap();
            let stop_build_on_errors =
                orchestrator.opts.command.build_options.stop_build_on_errors == Tristate::True;
            if stop_build_on_errors
                && upstream_task
                    .status
                    .as_ref()
                    .is_some_and(|status| status.is_error())
            {
                let ref_path = resolved
                    .project_references()
                    .get(upstream.ref_index)
                    .map(|r| r.path.clone())
                    .unwrap_or_default();
                return UpToDateStatus {
                    kind: UpToDateStatusType::UpstreamErrors,
                    data: UpToDateStatusData::UpstreamErrors(UpstreamErrors {
                        r#ref: ref_path,
                        ref_has_upstream_errors: upstream_task
                            .status
                            .as_ref()
                            .unwrap()
                            .kind
                            == UpToDateStatusType::UpstreamErrors,
                    }),
                };
            }
        }

        if orchestrator.opts.command.build_options.force == Tristate::True {
            return UpToDateStatus::new(UpToDateStatusType::ForceBuild);
        }

        let build_info_path = resolved.get_build_info_file_name();
        let (build_info, build_info_time) =
            self.load_or_store_build_info(orchestrator, config_path, &build_info_path);
        let build_info = match build_info {
            None => {
                return UpToDateStatus {
                    kind: UpToDateStatusType::OutputMissing,
                    data: UpToDateStatusData::Text(build_info_path.clone()),
                };
            }
            Some(build_info) => build_info,
        };

        if !build_info.is_valid_version() {
            return UpToDateStatus {
                kind: UpToDateStatusType::TsVersionOutputOfDate,
                data: UpToDateStatusData::Text(build_info.version.clone()),
            };
        }

        let content_mapper_project = self.get_content_mapper_project(orchestrator);
        let content_mapper_identities = match content_mapper_project.ok().flatten().as_deref() {
            None => Ok(Vec::new()),
            Some(project) => project.identities(),
        };
        if content_mapper_identities.is_err()
            || !build_info.content_mapper_identities_match(&content_mapper_identities.unwrap_or_default())
        {
            return UpToDateStatus {
                kind: UpToDateStatusType::OutOfDateOptions,
                data: UpToDateStatusData::Text(build_info_path.clone()),
            };
        }

        if build_info.errors
            || (resolved.compiler_options().no_check != Tristate::True
                && (build_info.semantic_errors || build_info.check_pending))
        {
            return UpToDateStatus {
                kind: UpToDateStatusType::OutOfDateBuildInfoWithErrors,
                data: UpToDateStatusData::Text(build_info_path.clone()),
            };
        }

        UpToDateStatus::new(UpToDateStatusType::UpToDate)
    }

    pub fn report_up_to_date_status(&mut self, orchestrator: &mut Orchestrator) {
        if orchestrator.opts.command.build_options.verbose != Tristate::True {
            return;
        }
        let status = self.status.clone().unwrap();
        let config = orchestrator.relative_file_name(&self.config);
        let report = |task: &mut BuildTask,
                      orchestrator: &mut Orchestrator,
                      message: &'static Message,
                      args: Vec<String>| {
            task.report_status(orchestrator, new_compiler_diagnostic(message, &args));
        };
        match status.kind {
            UpToDateStatusType::ConfigFileNotFound => report(
                self,
                orchestrator,
                &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_CONFIG_FILE_DOES_NOT_EXIST,
                vec![config],
            ),
            UpToDateStatusType::UpstreamErrors => {
                let upstream_status = status.upstream_errors().unwrap().clone();
                let dependency = orchestrator.relative_file_name(&upstream_status.r#ref);
                report(
                    self,
                    orchestrator,
                    if upstream_status.ref_has_upstream_errors {
                        &dg::PROJECT_0_CAN_T_BE_BUILT_BECAUSE_ITS_DEPENDENCY_1_WAS_NOT_BUILT
                    } else {
                        &dg::PROJECT_0_CAN_T_BE_BUILT_BECAUSE_ITS_DEPENDENCY_1_HAS_ERRORS
                    },
                    vec![config, dependency],
                );
            }
            UpToDateStatusType::BuildErrors => report(
                self,
                orchestrator,
                &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_IT_HAS_ERRORS,
                vec![config],
            ),
            UpToDateStatusType::UpToDate => {
                if let Some(input_output_file_and_time) = status.input_output_file_and_time() {
                    let input = orchestrator
                        .relative_file_name(&input_output_file_and_time.input.file);
                    let output = orchestrator
                        .relative_file_name(&input_output_file_and_time.output.file);
                    report(
                        self,
                        orchestrator,
                        &dg::PROJECT_0_IS_UP_TO_DATE_BECAUSE_NEWEST_INPUT_1_IS_OLDER_THAN_OUTPUT_2,
                        vec![config, input, output],
                    );
                }
            }
            UpToDateStatusType::UpToDateWithUpstreamTypes => report(
                self,
                orchestrator,
                &dg::PROJECT_0_IS_UP_TO_DATE_WITH_D_TS_FILES_FROM_ITS_DEPENDENCIES,
                vec![config],
            ),
            UpToDateStatusType::UpToDateWithInputFileText => report(
                self,
                orchestrator,
                &dg::PROJECT_0_IS_UP_TO_DATE_BUT_NEEDS_TO_UPDATE_TIMESTAMPS_OF_OUTPUT_FILES_THAT_ARE_OLDER_THAN_INPUT_FILES,
                vec![config],
            ),
            UpToDateStatusType::InputFileMissing => {
                let data = status_text(&status);
                let data = orchestrator.relative_file_name(&data);
                report(
                    self,
                    orchestrator,
                    &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_INPUT_1_DOES_NOT_EXIST,
                    vec![config, data],
                );
            }
            UpToDateStatusType::OutputMissing => {
                let data = status_text(&status);
                let data = orchestrator.relative_file_name(&data);
                report(
                    self,
                    orchestrator,
                    &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_OUTPUT_FILE_1_DOES_NOT_EXIST,
                    vec![config, data],
                );
            }
            UpToDateStatusType::InputFileNewer => {
                let input_output = status.input_output_name().unwrap().clone();
                let output = orchestrator.relative_file_name(&input_output.output);
                let input = orchestrator.relative_file_name(&input_output.input);
                report(
                    self,
                    orchestrator,
                    &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_OUTPUT_1_IS_OLDER_THAN_INPUT_2,
                    vec![config, output, input],
                );
            }
            UpToDateStatusType::OutOfDateBuildInfoWithPendingEmit => {
                let data = status_text(&status);
                let data = orchestrator.relative_file_name(&data);
                report(
                    self,
                    orchestrator,
                    &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_BUILDINFO_FILE_1_INDICATES_THAT_SOME_OF_THE_CHANGES_WERE_NOT_EMITTED,
                    vec![config, data],
                );
            }
            UpToDateStatusType::OutOfDateBuildInfoWithErrors => {
                let data = status_text(&status);
                let data = orchestrator.relative_file_name(&data);
                report(
                    self,
                    orchestrator,
                    &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_BUILDINFO_FILE_1_INDICATES_THAT_PROGRAM_NEEDS_TO_REPORT_ERRORS,
                    vec![config, data],
                );
            }
            UpToDateStatusType::OutOfDateOptions => {
                let data = status_text(&status);
                let data = orchestrator.relative_file_name(&data);
                report(
                    self,
                    orchestrator,
                    &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_BUILDINFO_FILE_1_INDICATES_THERE_IS_CHANGE_IN_COMPILEROPTIONS,
                    vec![config, data],
                );
            }
            UpToDateStatusType::OutOfDateRoots => {
                let input_output = status.input_output_name().unwrap().clone();
                let output = orchestrator.relative_file_name(&input_output.output);
                let input = orchestrator.relative_file_name(&input_output.input);
                report(
                    self,
                    orchestrator,
                    &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_BUILDINFO_FILE_1_INDICATES_THAT_FILE_2_WAS_ROOT_FILE_OF_COMPILATION_BUT_NOT_ANY_MORE,
                    vec![config, output, input],
                );
            }
            UpToDateStatusType::TsVersionOutputOfDate => {
                let data = status_text(&status);
                let data = orchestrator.relative_file_name(&data);
                report(
                    self,
                    orchestrator,
                    &dg::PROJECT_0_IS_OUT_OF_DATE_BECAUSE_OUTPUT_FOR_IT_WAS_GENERATED_WITH_VERSION_1_THAT_DIFFERS_WITH_CURRENT_VERSION_2,
                    vec![config, data, tsox_core::core::mig::m3k::version().to_string()],
                );
            }
            UpToDateStatusType::ForceBuild => report(
                self,
                orchestrator,
                &dg::PROJECT_0_IS_BEING_FORCIBLY_REBUILT,
                vec![config],
            ),
            UpToDateStatusType::Solution => {}
        }
    }

    pub fn can_update_js_dts_output_timestamps(&self) -> bool {
        let resolved = self.resolved.as_ref().unwrap();
        resolved.compiler_options().no_emit != Tristate::True
            && !resolved.compiler_options().is_incremental()
    }

    pub fn update_time_stamps(
        &mut self,
        orchestrator: &mut Orchestrator,
        emitted_files: &[String],
        verbose_message: &'static Message,
    ) {
        let mut emitted: Set<String> = Set::new();
        for file in emitted_files {
            emitted.add(file.clone());
        }
        let mut verbose_message_reported = false;
        let build_info_name = self
            .resolved
            .as_ref()
            .unwrap()
            .get_build_info_file_name();
        let now = SystemTime::now();
        let mut update_time_stamp = |task: &mut BuildTask, file: &str| {
            if emitted.has(&file.to_string()) {
                return;
            }
            if !verbose_message_reported
                && orchestrator.opts.command.build_options.verbose == Tristate::True
            {
                task.report_status(
                    orchestrator,
                    new_compiler_diagnostic(
                        verbose_message,
                        &[orchestrator.relative_file_name(&task.config)],
                    ),
                );
                verbose_message_reported = true;
            }
            if orchestrator.host.set_m_time(file, now).is_ok() {
                if file == build_info_name {
                    if let Some(entry) = task.build_info_entry.as_mut() {
                        entry.m_time = now;
                    }
                } else if task.store_output_time_stamp(orchestrator) {
                    orchestrator.host.store_m_time(file, now);
                }
            }
        };

        if self.can_update_js_dts_output_timestamps() {
            for output_file in self
                .resolved
                .as_ref()
                .unwrap()
                .get_output_file_names()
            {
                update_time_stamp(self, &output_file);
            }
        }
        update_time_stamp(self, &build_info_name);
    }

    pub fn clean_project(&mut self, orchestrator: &mut Orchestrator, path: Path) {
        if self.resolved.is_none() {
            self.report_diagnostic(new_compiler_diagnostic(
                &dg::FILE_0_NOT_FOUND,
                &[self.config.clone()],
            ));
            self.result.as_mut().unwrap().exit_status =
                crate::execute::ExitStatus::DiagnosticsPresent_OutputsSkipped;
            return;
        }
        let resolved = self.resolved.as_ref().unwrap().clone();
        let mut inputs: Set<Path> = Set::new();
        for file in resolved.file_names() {
            inputs.add(orchestrator.to_path(file));
        }
        for output_file in resolved.get_output_file_names() {
            self.clean_project_output(orchestrator, &output_file, &inputs);
        }
        let build_info_file_name = resolved.get_build_info_file_name();
        self.clean_project_output(orchestrator, &build_info_file_name, &inputs);
    }

    pub fn clean_project_output(
        &mut self,
        orchestrator: &mut Orchestrator,
        output_file: &str,
        inputs: &Set<Path>,
    ) {
        let output_path = orchestrator.to_path(output_file);
        if inputs.has(&output_path) {
            return;
        }
        if orchestrator.host.fs().file_exists(output_file) {
            if orchestrator.opts.command.build_options.dry != Tristate::True {
                if orchestrator.host.fs().remove(output_file).is_err() {
                    self.report_diagnostic(new_compiler_diagnostic(
                        &dg::FAILED_TO_DELETE_FILE_0,
                        &[output_file.to_string()],
                    ));
                }
            } else {
                self.result
                    .as_mut()
                    .unwrap()
                    .files_to_delete
                    .push(output_file.to_string());
            }
        }
    }

    pub fn update_watch(
        &mut self,
        orchestrator: &mut Orchestrator,
        old_cache: &SyncMap<Path, SystemTime>,
    ) {
        if self.resolved.is_some() && self.can_update_js_dts_output_timestamps() {
            for output_file in self
                .resolved
                .as_ref()
                .unwrap()
                .get_output_file_names()
            {
                orchestrator
                    .host
                    .store_m_time_from_old_cache(&output_file, old_cache);
            }
        }
    }

    pub fn reset_status(&mut self) {
        self.status = None;
        self.pending
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.errors.clear();
    }

    pub fn reset_config(&mut self, orchestrator: &mut Orchestrator, path: Path) {
        self.dirty = true;
        orchestrator.host.resolved_references.delete(&path);
    }

    pub fn load_or_store_build_info(
        &mut self,
        orchestrator: &mut Orchestrator,
        config_path: Path,
        build_info_file_name: &str,
    ) -> (Option<crate::mig::m4y_2::BuildInfo>, SystemTime) {
        let path = orchestrator.to_path(build_info_file_name);
        if let Some(entry) = self.build_info_entry.as_ref() {
            if entry.path == path {
                return (entry.build_info.clone(), entry.m_time);
            }
        }
        let build_info = orchestrator
            .host
            .fs()
            .read_file(build_info_file_name)
            .and_then(|data| {
                serde_json::from_str::<crate::mig::m4y_2::BuildInfo>(&data).ok()
            });
        let mut m_time = SystemTime::UNIX_EPOCH;
        if build_info.is_some() {
            m_time = orchestrator.host.get_m_time(build_info_file_name);
        }
        self.build_info_entry = Some(BuildInfoEntry {
            build_info: build_info.clone(),
            path,
            m_time,
            dts_time: None,
        });
        (build_info, m_time)
    }

    pub fn on_build_info_emit(
        &mut self,
        orchestrator: &mut Orchestrator,
        build_info_file_name: &str,
        build_info: crate::mig::m4y_2::BuildInfo,
        has_changed_dts_file: bool,
    ) {
        let m_time = SystemTime::now();
        let dts_time = if has_changed_dts_file {
            Some(m_time)
        } else {
            self.build_info_entry
                .as_ref()
                .and_then(|entry| entry.dts_time)
        };
        self.build_info_entry = Some(BuildInfoEntry {
            build_info: Some(build_info),
            path: orchestrator.to_path(build_info_file_name),
            m_time,
            dts_time,
        });
    }

    pub fn has_conflicting_build_info(
        &self,
        _orchestrator: &Orchestrator,
        upstream: &BuildTask,
    ) -> bool {
        match (self.build_info_entry.as_ref(), upstream.build_info_entry.as_ref()) {
            (Some(mine), Some(theirs)) => mine.path == theirs.path,
            _ => false,
        }
    }

    pub fn get_latest_changed_dts_m_time(&mut self, orchestrator: &Orchestrator) -> SystemTime {
        if let Some(dts_time) = self.build_info_entry.as_ref().and_then(|entry| entry.dts_time) {
            return dts_time;
        }
        let latest_changed_dts_file = self
            .build_info_entry
            .as_ref()
            .and_then(|entry| entry.build_info.as_ref())
            .map(|build_info| build_info.latest_changed_dts_file.clone())
            .unwrap_or_default();
        let directory = self
            .build_info_entry
            .as_ref()
            .map(|entry| tspath::get_directory_path(&entry.path.0))
            .unwrap_or_default();
        let dts_time = orchestrator.host.get_m_time(&tspath::get_normalized_absolute_path(
            &latest_changed_dts_file,
            &directory,
        ));
        if let Some(entry) = self.build_info_entry.as_mut() {
            entry.dts_time = Some(dts_time);
        }
        dts_time
    }

    pub fn store_output_time_stamp(&self, orchestrator: &Orchestrator) -> bool {
        orchestrator.opts.command.compiler_options.watch == Tristate::True
            && !self
                .resolved
                .as_ref()
                .unwrap()
                .compiler_options()
                .is_incremental()
    }

    pub fn write_file(
        &mut self,
        orchestrator: &mut Orchestrator,
        file_name: &str,
        text: &str,
        data: Option<&crate::mig::m4y_3::WriteFileData>,
    ) -> Result<(), std::io::Error> {
        let err = orchestrator.host.fs().write_file(file_name, text);
        if err.is_ok() {
            if let Some(data) = data {
                if let Some(build_info) = data.build_info.as_ref() {
                    let has_changed_dts_file = self
                        .result
                        .as_ref()
                        .unwrap()
                        .program
                        .as_ref()
                        .is_some_and(|program| program.has_changed_dts_file());
                    self.on_build_info_emit(
                        orchestrator,
                        file_name,
                        build_info.clone(),
                        has_changed_dts_file,
                    );
                    return Ok(());
                }
            }
            if self.store_output_time_stamp(orchestrator) {
                orchestrator
                    .host
                    .store_m_time(file_name, SystemTime::now());
            }
        }
        err
    }
}

pub fn is_content_mapper_supplemental_build_info_path(
    input_path: &Path,
    roots: impl Iterator<Item = Path>,
) -> bool {
    for root in roots {
        let suffix = match input_path.0.strip_prefix(&format!("{}.", root.0)) {
            Some(suffix) => suffix.to_string(),
            None => continue,
        };
        let (index, extension) = match suffix.split_once('.') {
            Some(parts) => parts,
            None => continue,
        };
        if extension.is_empty() {
            continue;
        }
        if index.parse::<i64>().is_ok()
            && tsox_compile::mig::m3l_cm::is_supported_virtual_extension(&format!(".{}", extension))
        {
            return true;
        }
    }
    false
}

fn content_mapper_to_mapper(
    mapper: &tsox_tsoptions::mig::m5h_3::ContentMapper,
) -> tsox_compile::mig::m3l_cm::Mapper {
    tsox_compile::mig::m3l_cm::Mapper {
        definition: tsox_compile::mig::m3l_cm::Definition {
            package: mapper.definition.package.clone(),
            extensions: mapper.definition.extensions.clone(),
            options: mapper.definition.options.clone(),
        },
        manifest: tsox_compile::mig::m3l_cm::Manifest {
            name: mapper.manifest.name.clone(),
            version: mapper.manifest.version.clone(),
            exec: mapper.manifest.exec.clone(),
            compiler_options: mapper.manifest.compiler_options.clone(),
            dynamic_config: mapper.manifest.dynamic_config,
        },
        package_directory: mapper.package_directory.clone(),
        contribution_id: mapper.contribution_id.clone(),
    }
}

fn status_text(status: &UpToDateStatus) -> String {
    match &status.data {
        UpToDateStatusData::Text(text) => text.clone(),
        _ => String::new(),
    }
}

pub fn new_compiler_diagnostic(message: &'static Message, args: &[String]) -> Arc<Diagnostic> {
    Arc::new(tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
        *message,
        args.to_vec(),
    ))
}

pub type OrchestratorResult = OrchestratorResultStruct;

pub struct OrchestratorResultStruct {
    pub result: crate::execute::CommandLineResult,
    pub errors: Vec<Arc<Diagnostic>>,
    pub statistics: crate::mig::m5a_3::Statistics,
    pub files_to_delete: Vec<String>,
}
