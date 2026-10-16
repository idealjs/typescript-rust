#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::execute::{System, ExitStatus};
use super::m5a2::MockWatchBackend;
use super::m5a2_2::TestFs;
use super::m5b_2::OutputSanitizer;
use super::m5b_4::WatchBackend;
use super::m5b_4::{WatchCloser, WatchDirectoryRequest};
use super::tsctests::TscInput;
use tsox_compile::mig::m4x_2::HasFileName;
use tsox_compile::compiler;
use crate::execute::incremental;
use crate::mig::m4y_3::SignatureUpdateKind;
use tsox_compile::mig::m3l_cm_2::Spawner as _;
use super::m5a2_2::contentmapper_test;
use super::m5a2_2::contentmapper_test::ReadWriteCloser;
use super::m5a2_2::fs_baseline_util;
use super::m5a2_2::harness_util;
use super::m5a2_2::vfstest;
use super::m5a2_2::vfstest::MapFS;
use tsox_core::fswatch::mig::m5g_2::{IgnoreFn, WatchCallback};
use tsox_core::tspath::Path;
use tsox_tsoptions::vfs::FS;

pub const TSC_LIB_PATH: &str = "/home/src/tslibs/TS/Lib";

pub mod testing {
    pub use crate::mig::m5a::CommandLineTesting;
}

pub const TSC_DEFAULT_LIB_CONTENT: &str = r#"/// <reference no-default-lib="true"/>
interface Boolean {}
interface Function {}
interface CallableFunction {}
interface NewableFunction {}
interface IArguments {}
interface Number { toExponential: any; }
interface Object {}
interface RegExp {}
interface String { charAt: any; }
interface Array<T> { length: number; [n: number]: T; }
interface ReadonlyArray<T> {}
interface SymbolConstructor {
    (desc?: string | number): symbol;
    for(name: string): symbol;
    readonly toStringTag: symbol;
}
declare var Symbol: SymbolConstructor;
interface Symbol {
    readonly [Symbol.toStringTag]: string;
}
declare const console: { log(msg: any): void; };
"#;

pub const FAKE_TIME_STAMP: &str = "HH:MM:SS AM";
pub const FAKE_DURATION: &str = "d.ddds";
pub const BUILD_STARTING_AT: &str = "build starting at ";
pub const BUILD_FINISHED_IN: &str = "build finished in ";
pub const LIST_FILE_START: &str = "!!! List files start";
pub const LIST_FILE_END: &str = "!!! List files end";
pub const STATISTICS_START: &str = "!!! Statistics start";
pub const STATISTICS_END: &str = "!!! Statistics end";
pub const BUILD_STATUS_REPORT_START: &str = "!!! Build Status Report Start";
pub const BUILD_STATUS_REPORT_END: &str = "!!! Build Status Report End";
pub const WATCH_STATUS_REPORT_START: &str = "!!! Watch Status Report Start";
pub const WATCH_STATUS_REPORT_END: &str = "!!! Watch Status Report End";
pub const TRACE_START: &str = "!!! Trace start";
pub const TRACE_END: &str = "!!! Trace end";

pub type FileMap = HashMap<String, String>;

pub fn get_test_lib_path_for(lib_name: &str) -> String { ::tsox_core::fntrace::enter("get_test_lib_path_for"); 
    let lib_file = match tsox_tsoptions::mig::m5h_5::LIB_MAP
        .iter()
        .find(|(name, _)| *name == lib_name)
    {
        Some((_, file)) => file.to_string(),
        None => format!("lib.{}.d.ts", lib_name),
    };
    format!("{}/{}", TSC_LIB_PATH, lib_file)
}

pub struct TestClock {
    start: std::time::SystemTime,
    now: Mutex<Option<std::time::SystemTime>>,
}

impl TestClock {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            start: std::time::SystemTime::now(),
            now: Mutex::new(None),
        }
    }

    pub fn now(&self) -> std::time::SystemTime { ::tsox_core::fntrace::enter("now"); 
        let mut now = self.now.lock().unwrap();
        let current = match *now {
            Some(t) => t,
            None => self.start,
        };
        let advanced = current + std::time::Duration::from_secs(1);
        *now = Some(advanced);
        advanced
    }

    pub fn since_start(&self) -> std::time::Duration { ::tsox_core::fntrace::enter("since_start"); 
        self.now()
            .duration_since(self.start)
            .unwrap_or_default()
    }
}

impl Default for TestClock {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}

impl vfstest::Clock for TestClock {
    fn now(&self) -> std::time::SystemTime { ::tsox_core::fntrace::enter("now"); 
        TestClock::now(self)
    }
}

pub fn new_tsc_system(
    files: FileMap,
    use_case_sensitive_file_names: bool,
    cwd: &str,
) -> TestSys { ::tsox_core::fntrace::enter("new_tsc_system"); 
    let clock = Arc::new(TestClock::new());
    let clock_dyn: Arc<dyn vfstest::Clock> = clock.clone();
    TestSys {
        fs: vfstest::from_map_with_clock(files, use_case_sensitive_file_names, clock_dyn),
        cwd: cwd.to_string(),
        output_is_tty: true,
        clock,
        current_write: Arc::new(Mutex::new(Vec::new())),
        program_baselines: String::new(),
        program_include_baselines: String::new(),
        tracer: None,
        fs_differ: None,
        for_incremental_correctness: false,
        mock_watch_backend: Arc::new(MockWatchBackend::new()),
        default_library_path: String::new(),
        env: HashMap::new(),
    }
}

pub fn get_file_map_with_build(mut files: FileMap, command_line_args: &[String]) -> FileMap { ::tsox_core::fntrace::enter("get_file_map_with_build"); 
    let mut sys = new_test_sys(&TscInput { files: files.clone(), ..Default::default() }, false);
    crate::execute::command_line(&sys, command_line_args);
    for key in sys.fs.written_files_keys() {
        if let Some(text) = sys.fs_from_file_map().read_file(&key) {
            files.insert(key, text);
        }
    }
    files
}

pub fn new_test_sys(tsc_input: &TscInput, for_incremental_correctness: bool) -> TestSys { ::tsox_core::fntrace::enter("new_test_sys"); 
    let cwd = if tsc_input.cwd.is_empty() {
        "/home/src/workspaces/project"
    } else {
        &tsc_input.cwd
    };
    let mut lib_path = TSC_LIB_PATH.to_string();
    if !tsc_input.windows_style_root.is_empty() {
        lib_path = format!("{}{}", tsc_input.windows_style_root, &lib_path[1..]);
    }
    let mut sys = new_tsc_system(tsc_input.files.clone(), !tsc_input.ignore_case, cwd);
    sys.default_library_path = lib_path;
    if let Some(output_is_tty) = tsc_input.output_is_tty {
        sys.output_is_tty = output_is_tty;
    }
    sys.tracer = Some(harness_util::new_tracer_for_baselining(
        tsox_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names: !tsc_input.ignore_case,
            current_directory: cwd.to_string(),
        },
    ));
    sys.env = tsc_input.env.clone();
    sys.for_incremental_correctness = for_incremental_correctness;
    let watch_backend = Arc::get_mut(&mut sys.mock_watch_backend).unwrap();
    watch_backend.directory_exists = true;
    watch_backend.use_case_sensitive_file_names = !tsc_input.ignore_case;
    sys.fs_differ = Some(fs_baseline_util::FsDiffer::new(&sys.fs));

    sys.ensure_lib_path_exists("lib.d.ts");
    for lib_file in tsox_tsoptions::mig::m5h_2::target_to_lib_map().values() {
        sys.ensure_lib_path_exists(lib_file);
    }
    for (_, lib_file) in tsox_tsoptions::mig::m5h_5::LIB_MAP.iter() {
        sys.ensure_lib_path_exists(lib_file);
    }
    sys
}

pub struct TestSys {
    pub current_write: Arc<Mutex<Vec<u8>>>,
    pub program_baselines: String,
    pub program_include_baselines: String,
    pub tracer: Option<harness_util::TracerForBaselining>,
    pub fs_differ: Option<fs_baseline_util::FsDiffer>,
    pub for_incremental_correctness: bool,
    pub mock_watch_backend: Arc<MockWatchBackend>,

    pub fs: TestFs,
    pub default_library_path: String,
    pub cwd: String,
    pub env: HashMap<String, String>,
    pub output_is_tty: bool,
    pub clock: Arc<TestClock>,
}

impl WatchBackend for MockWatchBackend {
    fn watch_directory(
        &self,
        dir: &str,
        callback: WatchCallback,
        recursive: bool,
        ignore: Option<IgnoreFn>,
    ) -> Result<Box<dyn WatchCloser>, String> { ::tsox_core::fntrace::enter("watch_directory"); 
        MockWatchBackend::watch_directory(self, dir, callback, recursive, ignore)
    }

    fn watch_directories(
        &self,
        requests: &[WatchDirectoryRequest],
    ) -> Result<Vec<Box<dyn WatchCloser>>, String> { ::tsox_core::fntrace::enter("watch_directories"); 
        MockWatchBackend::watch_directories(self, requests)
    }
}

impl TestSys {
    pub fn now(&self) -> std::time::SystemTime { ::tsox_core::fntrace::enter("now"); 
        self.clock.now()
    }

    pub fn fs(&self) -> Arc<dyn FS> { ::tsox_core::fntrace::enter("fs"); 
        Arc::clone(&self.fs.fs)
    }

    pub fn default_library_path(&self) -> &str { ::tsox_core::fntrace::enter("default_library_path"); 
        &self.default_library_path
    }

    pub fn current_directory(&self) -> &str { ::tsox_core::fntrace::enter("current_directory"); 
        &self.cwd
    }

    pub fn since_start(&self) -> std::time::Duration { ::tsox_core::fntrace::enter("since_start"); 
        self.clock.since_start()
    }

    pub fn fs_from_file_map(&self) -> Arc<dyn FS> { ::tsox_core::fntrace::enter("fs_from_file_map"); 
        self.fs.fs.clone()
    }

    pub fn map_fs(&self) -> Arc<MapFS> { ::tsox_core::fntrace::enter("map_fs"); 
        self.fs_differ.as_ref().unwrap().map_fs()
    }

    pub fn ensure_lib_path_exists(&self, path: &str) { ::tsox_core::fntrace::enter("ensure_lib_path_exists"); 
        let path = format!("{}/{}", self.default_library_path, path);
        if self.fs_from_file_map().read_file(&path).is_none() {
            self.fs.default_libs_add(&path);
            self.fs_from_file_map()
                .write_file(&path, TSC_DEFAULT_LIB_CONTENT)
                .expect("Failed to write default library file");
        }
    }

    pub fn spawn(
        &self,
        command: &[String],
        dir: &str,
        mut stderr: Box<dyn Write + Send>,
    ) -> Result<Box<dyn ReadWriteCloser>, String> { ::tsox_core::fntrace::enter("spawn"); 
        contentmapper_test::new_spawner()
            .spawn(command, dir, stderr.as_mut())
            .map_err(|e| e.to_string())
    }

    pub fn on_emitted_files(
        &self,
        result: Option<&super::m5a::EmitResult>,
        m_times_cache: Option<&mut tsox_core::collections::syncmap::SyncMap<tsox_core::tspath::Path, std::time::SystemTime>>,
    ) { ::tsox_core::fntrace::enter("on_emitted_files"); 
        let result = match result {
            Some(r) => r,
            None => return,
        };
        let mut m_times_cache = m_times_cache;
        for file in &result.emitted_files {
            let mod_time = self.map_fs().get_mod_time(file);
            let serialized_diff = self.fs_differ.as_ref().unwrap().serialized_diff();
            if let Some(diff) = serialized_diff {
                if let Some(d) = diff.snap.get(file) {
                    if d.m_time == mod_time {
                        continue;
                    }
                }
            }

            let now = self.now();
            self.fs_from_file_map()
                .chtimes(file, std::time::SystemTime::UNIX_EPOCH, now)
                .expect("Failed to change time for emitted file");

            if let Some(cache) = m_times_cache.as_deref_mut() {
                let path = tsox_core::tspath::to_path(
                    file,
                    &self.cwd,
                    self.fs().use_case_sensitive_file_names(),
                );
                if cache.load(&path).is_some() {
                    cache.store(path, now);
                }
            }
        }
    }

    pub fn on_list_files_start(&self, w: &mut dyn Write) { ::tsox_core::fntrace::enter("on_list_files_start"); 
        let _ = writeln!(w, "{}", LIST_FILE_START);
    }

    pub fn on_list_files_end(&self, w: &mut dyn Write) { ::tsox_core::fntrace::enter("on_list_files_end"); 
        let _ = writeln!(w, "{}", LIST_FILE_END);
    }

    pub fn on_statistics_start(&self, w: &mut dyn Write) { ::tsox_core::fntrace::enter("on_statistics_start"); 
        let _ = writeln!(w, "{}", STATISTICS_START);
    }

    pub fn on_statistics_end(&self, w: &mut dyn Write) { ::tsox_core::fntrace::enter("on_statistics_end"); 
        let _ = writeln!(w, "{}", STATISTICS_END);
    }

    pub fn on_build_status_report_start(&self, w: &mut dyn Write) { ::tsox_core::fntrace::enter("on_build_status_report_start"); 
        let _ = writeln!(w, "{}", BUILD_STATUS_REPORT_START);
    }

    pub fn on_build_status_report_end(&self, w: &mut dyn Write) { ::tsox_core::fntrace::enter("on_build_status_report_end"); 
        let _ = writeln!(w, "{}", BUILD_STATUS_REPORT_END);
    }

    pub fn on_watch_status_report_start(&self) { ::tsox_core::fntrace::enter("on_watch_status_report_start"); 
        let mut w = self.writer();
        let _ = writeln!(w, "{}", WATCH_STATUS_REPORT_START);
    }

    pub fn on_watch_status_report_end(&self) { ::tsox_core::fntrace::enter("on_watch_status_report_end"); 
        let mut w = self.writer();
        let _ = writeln!(w, "{}", WATCH_STATUS_REPORT_END);
    }

    pub fn get_trace<'a>(
        &'a self,
        w: &'a mut dyn Write,
        locale: tsox_core::locale::Locale,
    ) -> impl FnMut(&tsox_core::diagnostics::Message, &[String]) + 'a { ::tsox_core::fntrace::enter("get_trace"); 
        move |msg, args| {
            let _ = writeln!(w, "{}", TRACE_START);
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let str = msg.localize(&locale, &arg_refs);
            let same_as_writer = true;
            if let Some(tracer) = &self.tracer {
                tracer.trace_with_writer(w, &str, same_as_writer);
            }
            let _ = writeln!(w, "{}", TRACE_END);
        }
    }

    pub fn write_header_to_baseline(
        &self,
        builder: &mut String,
        program: &incremental::Program,
    ) { ::tsox_core::fntrace::enter("write_header_to_baseline"); 
        if !builder.is_empty() {
            builder.push('\n');
        }

        let config_file_path = program
            .options()
            .map(|options| options.config_file_path.clone())
            .unwrap_or_default();
        if !config_file_path.is_empty() {
            builder.push_str(&tsox_core::tspath::mig::m3i::get_relative_path_from_directory(
                &self.cwd,
                &config_file_path,
                &tsox_core::tspath::ComparePathsOptions {
                    use_case_sensitive_file_names: self.fs().use_case_sensitive_file_names(),
                    current_directory: self.cwd.clone(),
                },
            ));
            builder.push_str("::\n");
        }
    }

    pub fn watch_backend(&self) -> Arc<dyn WatchBackend> { ::tsox_core::fntrace::enter("watch_backend"); 
        let backend: Arc<dyn WatchBackend> = self.mock_watch_backend.clone();
        backend
    }

    pub fn on_program(&mut self, program: &incremental::Program) { ::tsox_core::fntrace::enter("on_program"); 
        let mut program_baselines = std::mem::take(&mut self.program_baselines);
        self.write_header_to_baseline(&mut program_baselines, program);
        self.program_baselines = program_baselines;

        let testing_data = program
            .get_testing_data()
            .expect("program testing data required for baselines");
        self.program_baselines.push_str("SemanticDiagnostics::\n");
        for file in program.get_program().get_source_files() {
            let file_path = Path::from(file.path());
            match testing_data.semantic_diagnostics_per_file.load(&file_path) {
                Some(diagnostics) => {
                    let changed = match testing_data
                        .old_program_semantic_diagnostics_per_file
                        .load(&file_path)
                    {
                        Some(old) => old != diagnostics,
                        None => true,
                    };
                    if changed {
                        self.program_baselines.push_str("*refresh*    ");
                        self.program_baselines.push_str(&file.file_name());
                        self.program_baselines.push('\n');
                    }
                }
                None => {
                    self.program_baselines.push_str("*not cached* ");
                    self.program_baselines.push_str(&file.file_name());
                    self.program_baselines.push('\n');
                }
            }
        }

        self.program_baselines.push_str("Signatures::\n");
        for file in program.get_program().get_source_files() {
            let file_path = Path::from(file.path());
            if let Some(kind) = testing_data.updated_signature_kinds.load(&file_path) {
                match kind {
                    SignatureUpdateKind::ComputedDts => {
                        self.program_baselines.push_str("(computed .d.ts) ");
                    }
                    SignatureUpdateKind::StoredAtEmit => {
                        self.program_baselines.push_str("(stored at emit) ");
                    }
                    SignatureUpdateKind::UsedVersion => {
                        self.program_baselines.push_str("(used version)   ");
                    }
                }
                self.program_baselines.push_str(&file.file_name());
                self.program_baselines.push('\n');
            }
        }

        let mut files_without_include_reason: Vec<String> = Vec::new();
        let mut file_not_in_program_with_include_reason: Vec<String> = Vec::new();
        let include_reasons = program.get_program().get_file_include_reasons();
        for file in program.get_program().get_source_files() {
            if !include_reasons.contains_key(&file.path()) {
                files_without_include_reason.push(file.path().as_str().to_string());
            }
        }
        for path in include_reasons.keys() {
            if program.get_program().get_source_file_by_path(path).is_none()
                && !program.get_program().is_missing_path(path)
            {
                file_not_in_program_with_include_reason.push(path.as_str().to_string());
            }
        }
        if !files_without_include_reason.is_empty() || !file_not_in_program_with_include_reason.is_empty() {
            let mut program_include_baselines = std::mem::take(&mut self.program_include_baselines);
            self.write_header_to_baseline(&mut program_include_baselines, program);
            self.program_include_baselines = program_include_baselines;
            self.program_include_baselines
                .push_str("!!! Expected all files to have include reasons\nfilesWithoutIncludeReason::\n");
            for file in &files_without_include_reason {
                self.program_include_baselines.push_str("  ");
                self.program_include_baselines.push_str(file);
                self.program_include_baselines.push('\n');
            }
            self.program_include_baselines
                .push_str("filesNotInProgramWithIncludeReason::\n");
            for file in &file_not_in_program_with_include_reason {
                self.program_include_baselines.push_str("  ");
                self.program_include_baselines.push_str(file);
                self.program_include_baselines.push('\n');
            }
        }
    }

    pub fn baseline_programs(&mut self, baseline: &mut String, header: &str) -> String { ::tsox_core::fntrace::enter("baseline_programs"); 
        baseline.push_str(&self.program_baselines.clone());
        self.program_baselines.clear();
        let mut result = String::new();
        if !self.program_include_baselines.is_empty() {
            result.push_str(&format!(
                "\n\n{}\n!!! Include reasons expectations don't match pls review!!!\n",
                header
            ));
            result.push_str(&self.program_include_baselines.clone());
            self.program_include_baselines.clear();
            baseline.push_str(&result);
        }
        result
    }

    pub fn serialize_state(&mut self, baseline: &mut String) { ::tsox_core::fntrace::enter("serialize_state"); 
        self.baseline_output(baseline);
        let mut fs_baseline = Vec::new();
        self.baseline_fs_with_diff(&mut fs_baseline);
        baseline.push_str(&String::from_utf8_lossy(&fs_baseline));
    }

    pub fn baseline_output(&self, baseline: &mut String) { ::tsox_core::fntrace::enter("baseline_output"); 
        baseline.push_str("\nOutput::\n");
        let output = self.get_output(false);
        baseline.push_str(&output);
    }

    pub fn get_output(&self, for_comparing: bool) -> String { ::tsox_core::fntrace::enter("get_output"); 
        let current = String::from_utf8_lossy(&self.current_write.lock().unwrap()).to_string();
        let lines: Vec<&str> = current.split('\n').collect();
        let mut transformer = OutputSanitizer {
            for_comparing,
            lines,
            index: 0,
            output_lines: Vec::new(),
        };
        transformer.transform_lines()
    }

    pub fn clear_output(&mut self) { ::tsox_core::fntrace::enter("clear_output"); 
        self.current_write.lock().unwrap().clear();
        if let Some(tracer) = &mut self.tracer {
            tracer.reset();
        }
    }

    pub fn baseline_fs_with_diff(&self, baseline: &mut dyn Write) { ::tsox_core::fntrace::enter("baseline_fs_with_diff"); 
        self.fs_differ.as_ref().unwrap().baseline_fs_with_diff(baseline);
    }
}
