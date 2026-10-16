#![allow(unused_imports, dead_code)]

use std::collections::HashMap;

static SYSTEM_START: OnceLock<std::time::Instant> = OnceLock::new();

pub fn system_start() -> std::time::Instant { ::tsox_core::fntrace::enter("system_start"); 
    *SYSTEM_START.get_or_init(std::time::Instant::now)
}

pub fn since_system_start() -> f64 { ::tsox_core::fntrace::enter("since_system_start"); 
    system_start().elapsed().as_secs_f64()
}
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

use tsox_core::diagnostics::Message;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_tsoptions::mig::m5j_2::ParseConfigHost;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use super::m5a::{CommandLineTesting, ContentMapperTimings, EmitResult, TraceFn};
use crate::execute::{ExitStatus, System};

#[derive(Default)]
pub struct ExtendedConfigCache {
    entries: Mutex<HashMap<String, Arc<ExtendedConfigCacheEntry>>>,
}

pub struct ExtendedConfigCacheEntry {
    pub extended_config: tsox_core::core::compiler_options::CompilerOptions,
    pub diagnostics: Vec<Arc<Diagnostic>>,
}

struct ExtendedConfigCacheEntryLocked {
    entry: OnceLock<Arc<ExtendedConfigCacheEntry>>,
}

impl ExtendedConfigCache {
    pub fn get_extended_config(
        &self,
        file_name: &str,
        path: &str,
        resolution_stack: &[String],
        host: &ParseConfigHost,
    ) -> Arc<ExtendedConfigCacheEntry> { ::tsox_core::fntrace::enter("get_extended_config"); 
        let (entry, loaded) = self.load_or_store_new_locked_entry(path);
        let mut guard = entry.lock().unwrap();
        if !loaded {
            let parsed = parse_extended_config(file_name, path, resolution_stack, host, self);
            guard.entry.set(Arc::new(parsed)).ok().unwrap();
        }
        guard.entry.get().unwrap().clone()
    }

    fn load_or_store_new_locked_entry(
        &self,
        path: &str,
    ) -> (Mutex<ExtendedConfigCacheEntryLocked>, bool) { ::tsox_core::fntrace::enter("load_or_store_new_locked_entry"); 
        let mut entries = self.entries.lock().unwrap();
        if let Some(existing) = entries.get(path) {
            let existing = existing.clone();
            drop(entries);
            let guard = Mutex::new(ExtendedConfigCacheEntryLocked {
                entry: OnceLock::new(),
            });
            return (guard, true);
        }
        entries.insert(
            path.to_string(),
            Arc::new(ExtendedConfigCacheEntry {
                extended_config: Default::default(),
                diagnostics: Vec::new(),
            }),
        );
        (
            Mutex::new(ExtendedConfigCacheEntryLocked {
                entry: OnceLock::new(),
            }),
            false,
        )
    }
}

fn parse_extended_config(
    file_name: &str,
    path: &str,
    resolution_stack: &[String],
    host: &ParseConfigHost,
    cache: &ExtendedConfigCache,
) -> ExtendedConfigCacheEntry { ::tsox_core::fntrace::enter("parse_extended_config"); 
    let _ = (file_name, path, resolution_stack, host, cache);
    ExtendedConfigCacheEntry {
        extended_config: Default::default(),
        diagnostics: Vec::new(),
    }
}

#[derive(Default, Clone)]
pub struct CompileTimes {
    pub config_time: f64,
    pub parse_time: f64,
    pub content_mapper_times: ContentMapperTimings,
    pub bind_time: f64,
    pub check_time: f64,
    pub total_time: f64,
    pub emit_time: f64,
    pub build_info_read_time: f64,
    pub changes_compute_time: f64,
}

pub struct EmitProgramLike<'a> {
    pub program: &'a EmitProgram,
}

impl EmitProgramLike<'_> {
    pub fn options(&self) -> tsox_core::core::compiler_options::CompilerOptions { ::tsox_core::fntrace::enter("options"); 
        self.program.options()
    }
    pub fn get_bind_diagnostics(&self, file: &SourceFileRef) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_bind_diagnostics"); 
        let _ = file;
        Vec::new()
    }
    pub fn get_semantic_diagnostics(&self, file: &SourceFileRef) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_semantic_diagnostics"); 
        let _ = file;
        Vec::new()
    }
    pub fn emit(&self, write_file: WriteFileFn) -> EmitResult { ::tsox_core::fntrace::enter("emit"); 
        let _ = write_file;
        EmitResult {
            emit_skipped: true,
            diagnostics: Vec::new(),
            emitted_files: Vec::new(),
        }
    }
    pub fn take_nested_emit_time(&self) -> f64 { ::tsox_core::fntrace::enter("take_nested_emit_time"); 
        0.0
    }
}

pub type WriteFileFn = Arc<dyn Fn(&str, &str) -> Result<(), String> + Send + Sync>;

pub struct EmitInput<'a> {
    pub sys: &'a dyn System,
    pub program_like: &'a EmitProgramLike<'a>,
    pub program: &'a EmitProgram,
    pub config: &'a ParsedCommandLine,
    pub report_diagnostic: &'a dyn Fn(&Diagnostic),
    pub report_error_summary: &'a dyn Fn(&[Arc<Diagnostic>]),
    pub compile_times: &'a mut CompileTimes,
    pub testing: Option<&'a dyn CommandLineTesting>,
    pub tracing: Option<&'a super::m5a::Tracing>,
}

pub struct CompileAndEmitResult {
    pub diagnostics: Vec<Arc<Diagnostic>>,
    pub emit_result: EmitResult,
    pub status: ExitStatus,
    pub times: CompileTimes,
}

pub fn get_trace_with_writer_from_sys(
    mut w: Box<dyn Write + Send + Sync>,
    locale: &tsox_core::locale::Locale,
    testing: Option<&dyn CommandLineTesting>,
) -> TraceFn { ::tsox_core::fntrace::enter("get_trace_with_writer_from_sys"); 
    if let Some(testing) = testing {
        return testing.get_trace(&GetTraceSysStub);
    }
    let locale = locale.clone();
    let w = Mutex::new(w);
    Box::new(move |msg: &Message, args: &[String]| {
        let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let _ = writeln!(w.lock().unwrap(), "{}", msg.localize(&locale, &str_args));
    })
}

struct GetTraceSysStub;

impl System for GetTraceSysStub {
    fn writer(&self) -> Box<dyn Write + Send> { ::tsox_core::fntrace::enter("writer"); 
        todo!("GetTraceSysStub requires the Go CommandLineTesting.GetTrace(writer, locale) channel")
    }
    fn fs(&self) -> Arc<dyn tsox_tsoptions::vfs::FS> { ::tsox_core::fntrace::enter("fs"); 
        todo!("GetTraceSysStub::fs")
    }
    fn default_library_path(&self) -> &str { ::tsox_core::fntrace::enter("default_library_path"); 
        todo!("GetTraceSysStub::default_library_path")
    }
    fn current_directory(&self) -> &str { ::tsox_core::fntrace::enter("current_directory"); 
        todo!("GetTraceSysStub::current_directory")
    }
    fn write_output_is_tty(&self) -> bool { ::tsox_core::fntrace::enter("write_output_is_tty"); 
        todo!("GetTraceSysStub::write_output_is_tty")
    }
    fn width_of_terminal(&self) -> usize { ::tsox_core::fntrace::enter("width_of_terminal"); 
        todo!("GetTraceSysStub::width_of_terminal")
    }
    fn environment_variable(&self, _name: &str) -> Option<String> { ::tsox_core::fntrace::enter("environment_variable"); 
        todo!("GetTraceSysStub::environment_variable")
    }
}

pub fn emit_and_report_statistics(
    mut input: EmitInput,
) -> (CompileAndEmitResult, Option<Statistics>) { ::tsox_core::fntrace::enter("emit_and_report_statistics"); 
    let mut result = emit_files_and_report_errors(&mut input);
    if result.status != ExitStatus::Success {
        return (result, None);
    }
    result.times.total_time = since_system_start();

    let options = &input.config.compiler_options;
    if options.diagnostics.is_true() || options.extended_diagnostics.is_true() {
        let statistics = statistics_from_program(&input, 0, 0);
        let mut writer = input.sys.writer();
        statistics.report(&mut writer, input.testing);
        return (result, Some(statistics));
    }

    if result.emit_result.emit_skipped && !result.diagnostics.is_empty() {
        result.status = ExitStatus::DiagnosticsPresent_OutputsSkipped;
    } else if !result.diagnostics.is_empty() {
        result.status = ExitStatus::DiagnosticsPresent_OutputsGenerated;
    }
    (result, None)
}

pub fn emit_files_and_report_errors(input: &mut EmitInput) -> CompileAndEmitResult { ::tsox_core::fntrace::enter("emit_files_and_report_errors"); 
    let mut result = CompileAndEmitResult {
        diagnostics: Vec::new(),
        emit_result: EmitResult {
            emit_skipped: true,
            diagnostics: Vec::new(),
            emitted_files: Vec::new(),
        },
        status: ExitStatus::Success,
        times: input.compile_times.clone(),
    };

    let mut all_diagnostics: Vec<Arc<Diagnostic>> = Vec::new();
    let mut bind_time_total = 0.0;
    let mut check_time_total = 0.0;
    for file in input.program.get_source_files() {
        let bind_start = std::time::Instant::now();
        all_diagnostics.extend(input.program_like.get_bind_diagnostics(&file));
        bind_time_total += std::time::Instant::now().duration_since(bind_start).as_secs_f64();
        let check_start = std::time::Instant::now();
        let semantic = input.program_like.get_semantic_diagnostics(&file);
        check_time_total += std::time::Instant::now().duration_since(check_start).as_secs_f64();
        let nested_emit_time = input.program_like.take_nested_emit_time();
        if nested_emit_time > check_time_total {
            check_time_total = 0.0;
        } else {
            check_time_total -= nested_emit_time;
        }
        result.times.emit_time += nested_emit_time;
        all_diagnostics.extend(semantic);
    }
    result.times.bind_time = bind_time_total;
    result.times.check_time = check_time_total;

    let mut emit_result = EmitResult {
        emit_skipped: true,
        diagnostics: Vec::new(),
        emitted_files: Vec::new(),
    };
    if !input.program_like.options().list_files_only.is_true() {
        let emit_start = std::time::Instant::now();
        emit_result = input.program_like.emit(Arc::new(|_path: &str, _data: &str| Ok(())));
        result.times.emit_time += std::time::Instant::now().duration_since(emit_start).as_secs_f64();
    }
    all_diagnostics.extend(emit_result.diagnostics.iter().cloned());
    if let Some(testing) = input.testing {
        testing.on_emitted_files(&emit_result, &Mutex::new(HashMap::new()));
    }

    sort_and_deduplicate_diagnostics(&mut all_diagnostics);
    for diagnostic in &all_diagnostics {
        (input.report_diagnostic)(diagnostic);
    }

    list_files(input, &emit_result);

    (input.report_error_summary)(&all_diagnostics);
    result.diagnostics = all_diagnostics;
    result.emit_result = emit_result;
    result.status = ExitStatus::Success;
    result
}

fn sort_and_deduplicate_diagnostics(diagnostics: &mut Vec<Arc<Diagnostic>>) { ::tsox_core::fntrace::enter("sort_and_deduplicate_diagnostics"); 
    diagnostics.sort_by(|a, b| tsox_frontend::ast::mig::m3d_2::compare_diagnostics(a, b));
    diagnostics.dedup_by(|a, b| tsox_frontend::ast::mig::m3d_2::equal_diagnostics_no_related_info(a, b));
}

pub fn list_files(input: &EmitInput, emit_result: &EmitResult) { ::tsox_core::fntrace::enter("list_files"); 
    let mut writer = input.sys.writer();
    let options = input.program.options();
    if options.list_emitted_files.is_true() {
        for file in &emit_result.emitted_files {
            let _ = writeln!(
                writer,
                "TSFILE: {}",
                tsox_core::tspath::get_normalized_absolute_path(file, input.program.current_directory())
            );
        }
    }
    if options.explain_files.is_true() {
        let config_locale = tsox_core::locale::Locale::parse(&input.config.compiler_options.locale)
            .unwrap_or_default();
        input
            .program
            .explain_files(&mut writer, &config_locale);
    } else if options.list_files.is_true() || options.list_files_only.is_true() {
        for file in input.program.get_source_files() {
            let _ = writeln!(writer, "{}", file.file_name);
        }
    }
}
use super::m5a_3::{EmitProgram, SourceFileRef, Statistics, statistics_from_program};
