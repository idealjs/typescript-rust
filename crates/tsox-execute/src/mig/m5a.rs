#![allow(unused_imports, dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_tsoptions::vfs::FS;

use crate::execute::{CommandLineResult, ExitStatus, System};

use tsox_compile::compiler;

mod incremental {
    pub use crate::mig::m4z::{create_host, new_build_info_reader, read_build_info_program};
    pub use crate::mig::m4z2::new_program;
}

use super::m5a_2::{CompileTimes, EmitInput, ExtendedConfigCache, emit_and_report_statistics};

pub type TraceFn = Box<dyn Fn(&Message, &[String]) + Send + Sync>;

use tsox_compile::mig::m3l_cm::{Mapper, Timings};
use tsox_compile::mig::m3l_cm_2::{Host as MapperHost, ProjectSpec, TransformOutcome};

pub struct ContentMapperHost {
    timings: ContentMapperTimings,
    locale: Mutex<Option<tsox_core::locale::Locale>>,
}

impl MapperHost for ContentMapperHost {
    fn timings(&self) -> Timings {
        Timings {
            mappers: Default::default(),
            request_wait: std::time::Duration::from_secs_f64(self.timings.request_wait.max(0.0)),
        }
    }

    fn project(&self, _spec: &ProjectSpec) -> Option<Arc<dyn tsox_compile::mig::m3l_cm_2::Project>> {
        None
    }

    fn acquire(&self, _mappers: &[Arc<Mapper>]) -> Box<dyn FnOnce()> {
        Box::new(|| {})
    }

    fn set_locale(&self, locale: tsox_core::locale::Locale) {
        *self.locale.lock().unwrap() = Some(locale);
    }

    fn transform(
        &self,
        _mapper: &Mapper,
        _request: &tsox_compile::mig::m3l_cm_2::Request,
    ) -> Result<TransformOutcome, tsox_compile::mig::m3l_cm::TransformError> {
        Err(tsox_compile::mig::m3l_cm::new_transform_error(
            tsox_compile::mig::m3l_cm::TransformErrorKind::Project,
            Box::new(tsox_compile::mig::m3l_cm::ProjectError::default()),
        ))
    }

    fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

impl ContentMapperHost {
    pub fn timings(&self) -> &ContentMapperTimings {
        &self.timings
    }
}

#[derive(Default, Clone)]
pub struct ContentMapperTimings {
    pub request_wait: f64,
    pub mappers: Vec<(String, ContentMapperTiming)>,
}

#[derive(Default, Clone)]
pub struct ContentMapperTiming {
    pub spawn: (u64, f64),
    pub initialize: (u64, f64),
    pub transform: (u64, f64),
    pub open_project: (u64, f64),
    pub close_project: (u64, f64),
}

pub type ContentMapperLogger = Arc<dyn Fn(&str) + Send + Sync>;

pub fn new_content_mapper_logger(sys: &dyn System) -> Option<ContentMapperLogger> {
    if sys.environment_variable("TS_CONTENT_MAPPER_DEBUG").unwrap_or_default().is_empty() {
        return None;
    }
    let writer = Mutex::new(Box::new(std::io::stderr()) as Box<dyn std::io::Write + Send>);
    Some(Arc::new(move |message: &str| {
        let mut writer = writer.lock().unwrap();
        let _ = writeln!(writer, "{message}");
    }))
}

pub fn new_content_mapper_host(
    sys: &dyn System,
    options: &CompilerOptions,
) -> Option<Arc<dyn MapperHost>> {
    new_content_mapper_host_concrete(sys, options).map(|h| h as Arc<dyn MapperHost>)
}

pub fn new_content_mapper_host_concrete(
    sys: &dyn System,
    options: &CompilerOptions,
) -> Option<Arc<ContentMapperHost>> {
    if !options.run_external_code.is_true() {
        return None;
    }
    let _logger = new_content_mapper_logger(sys);
    Some(Arc::new(ContentMapperHost {
        timings: ContentMapperTimings::default(),
        locale: Mutex::new(None),
    }))
}

pub fn get_trace_from_sys(
    sys: &dyn System,
    locale: Option<tsox_core::locale::Locale>,
    testing: Option<&dyn CommandLineTesting>,
) -> TraceFn {
    match testing {
        None => {
            let writer = Mutex::new(sys.writer());
            Box::new(move |msg: &Message, args: &[String]| {
                let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let localized = msg.localize(&locale.clone().unwrap_or_default(), &str_args);
                let _ = writeln!(writer.lock().unwrap(), "{localized}");
            })
        }
        Some(testing) => testing.get_trace(sys),
    }
}

pub trait CommandLineTesting: Send + Sync {
    fn on_emitted_files(&self, result: &EmitResult, m_times_cache: &Mutex<HashMap<String, f64>>);
    fn on_list_files_start(&self, w: &mut dyn std::io::Write);
    fn on_list_files_end(&self, w: &mut dyn std::io::Write);
    fn on_statistics_start(&self, w: &mut dyn std::io::Write);
    fn on_statistics_end(&self, w: &mut dyn std::io::Write);
    fn on_build_status_report_start(&self, w: &mut dyn std::io::Write);
    fn on_build_status_report_end(&self, w: &mut dyn std::io::Write);
    fn on_watch_status_report_start(&self);
    fn on_watch_status_report_end(&self);
    fn get_trace(&self, sys: &dyn System) -> TraceFn;
    fn on_program(&self, program: &IncrementalProgram);
    fn watch_backend(&self) -> Option<Arc<dyn super::m5b_4::WatchBackend>> {
        None
    }
}

pub use crate::mig::m4z2::Program as IncrementalProgram;

pub struct EmitResult {
    pub emit_skipped: bool,
    pub diagnostics: Vec<Arc<Diagnostic>>,
    pub emitted_files: Vec<String>,
}

pub struct Tracing {
    _fs: Arc<TracingFs>,
}

struct TracingFs(Arc<dyn FS>);

impl Tracing {
    pub fn stop_tracing(&self) -> Result<(), String> {
        Ok(())
    }
}

impl tsox_core::tracing::mig::x11a::FS for TracingFs {
    fn write_file(&self, path: &str, contents: &str) -> Result<(), String> {
        self.0.write_file(path, contents).map_err(|e| e.to_string())
    }
    fn append_file(&self, path: &str, contents: &str) -> Result<(), String> {
        self.0.append_file(path, contents).map_err(|e| e.to_string())
    }
}

pub fn start_tracing_if_needed(
    sys: &dyn System,
    config: &ParsedCommandLine,
    testing: Option<&dyn CommandLineTesting>,
) -> Option<Tracing> {
    let trace_dir = config.compiler_options.generate_trace.clone();
    if trace_dir.is_empty() {
        return None;
    }
    let config_file_path = String::new();
    let fs = sys.fs();
    let tracing_fs = Arc::new(TracingFs(fs));
    match tsox_core::tracing::mig::x11a::start_tracing(
        tracing_fs.as_ref(),
        &trace_dir,
        &config_file_path,
        testing.is_some(),
    ) {
        Ok(_tr) => Some(Tracing { _fs: tracing_fs }),
        Err(err) => {
            let mut writer = sys.writer();
            let _ = writeln!(writer, "Warning: Failed to start tracing: {err}");
            None
        }
    }
}

pub fn stop_tracing(sys: &dyn System, tr: Option<&Tracing>) {
    let Some(tr) = tr else { return };
    if let Err(err) = tr.stop_tracing() {
        let mut writer = sys.writer();
        let _ = writeln!(writer, "Warning: Failed to stop tracing: {err}");
    }
}

pub fn perform_incremental_compilation(
    sys: &dyn System,
    config: &ParsedCommandLine,
    report_diagnostic: &dyn Fn(&Diagnostic),
    report_error_summary: &dyn Fn(&[Arc<Diagnostic>]),
    extended_config_cache: &ExtendedConfigCache,
    compile_times: &mut CompileTimes,
    testing: Option<&dyn CommandLineTesting>,
) -> CommandLineResult {
    let content_mapper_host = new_content_mapper_host_concrete(sys, &config.compiler_options);
    let _content_mapper_project = get_content_mapper_project(
        content_mapper_host.as_ref().map(Arc::as_ref),
        config,
    );

    let host: Arc<dyn compiler::CompilerHost> = Arc::new(
        compiler::CompilerHostImpl::new(
            sys.fs(),
            sys.current_directory().to_string(),
            sys.default_library_path().to_string(),
        ),
    );

    let build_info_read_start = std::time::Instant::now();
    let old_program = incremental::read_build_info_program(
        config,
        &incremental::new_build_info_reader(Arc::clone(&host)),
        host.as_ref(),
    );
    compile_times.build_info_read_time =
        build_info_read_start.elapsed().as_secs_f64();

    let tr = start_tracing_if_needed(sys, config, testing);

    let parse_start = std::time::Instant::now();
    let program = compiler::new_program(compiler::ProgramOptions {
        config: config.clone(),
        host: Arc::clone(&host),
        use_source_of_project_reference: false,
        single_threaded: tsox_core::core::tristate::Tristate::Unknown,
        create_checker_pool: None,
        typings_location: String::new(),
        project_name: String::new(),
        tracing: None,
        skip_module_resolution: false,
    });
    compile_times.parse_time = parse_start.elapsed().as_secs_f64();

    let changes_compute_start = std::time::Instant::now();
    let incremental_program = incremental::new_program(
        Arc::clone(&program),
        old_program.as_ref(),
        Some(Arc::new(incremental::create_host(Arc::clone(&host)))),
        Some(Arc::new(|| std::time::SystemTime::now())),
        testing.is_some(),
    );
    compile_times.changes_compute_time =
        changes_compute_start.elapsed().as_secs_f64();
    if let Some(content_mapper_host) = &content_mapper_host {
        compile_times.content_mapper_times = content_mapper_host.timings().clone();
    }

    let emit_program = super::m5a_3::EmitProgram;
    let program_like = super::m5a_2::EmitProgramLike { program: &emit_program };
    let (result, _) = emit_and_report_statistics(EmitInput {
        sys,
        program_like: &program_like,
        program: &emit_program,
        config,
        report_diagnostic,
        report_error_summary,
        compile_times,
        testing,
        tracing: tr.as_ref(),
    });

    stop_tracing(sys, tr.as_ref());

    if let Some(testing) = testing {
        testing.on_program(&incremental_program);
    }

    CommandLineResult {
        status: result.status,
        watcher: None,
    }
}

pub fn get_content_mapper_project(
    host: Option<&ContentMapperHost>,
    config: &ParsedCommandLine,
) -> Option<ContentMapperProject> {
    let _ = (host, config);
    None
}

pub struct ContentMapperProject;

impl ContentMapperProject {
    pub fn close(&mut self) {}
}

use std::io::Write as _;
