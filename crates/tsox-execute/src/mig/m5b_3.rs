#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::m4x_build::new_compiler_diagnostic;
use super::m5a::{get_trace_from_sys as get_trace_from_sys_tsc, new_content_mapper_host};
use super::m5a_2::{
    CompileAndEmitResult, CompileTimes, EmitInput, EmitProgramLike, ExtendedConfigCache,
    emit_files_and_report_errors,
};
use super::m5a_4::create_watch_status_reporter;
use super::m5b_4::{DirWatchSet, WatchManager, can_watch_directory};
use super::tsctests::TscInput;
use crate::execute::System;
mod incremental {
    pub use crate::mig::m4z::{new_build_info_reader, read_build_info_program};
    pub use crate::mig::m4z2::{new_program, Program};
}
use tsox_compile::compiler;
use tsox_compile::compiler::{CompilerHost, Program, ProgramOptions};
use tsox_compile::mig::m3l_cm_2 as contentmapper;
use tsox_compile::mig::m4v_2::content_mapper_project_diagnostic;
use tsox_compile::mig::m4v_3;
use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::fswatch::mig::m5f_2::EventKind;
use tsox_frontend::ast::{self, Diagnostic};
use tsox_frontend::ast::node_source_file::{ScriptKind, SourceFile};
use tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions;
use tsox_frontend::ast::mig::x4ast::{get_jsx_implicit_import_base, get_jsx_runtime_import};
use tsox_core::tspath::{self, ComparePathsOptions, Path};
use tsox_tsoptions::mig::m5i2_4::get_parsed_command_line_of_config_file;
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_tsoptions::vfs::FS;
use tsox_tsoptions::vfs::cachedvfs::CachedFS;
use tsox_tsoptions::vfs::mig::m3k_vfs::TrackingFS;

static THE_CONTENT_MAPPER_PROCESS_COULD_NOT_BE_STARTED_OR_INITIALIZED: tsox_core::diagnostics::Message =
    tsox_core::diagnostics::Message {
        code: 100041,
        category: tsox_core::diagnostics::Category::Message,
        key: "The_content_mapper_process_could_not_be_started_or_initialized_100041",
        text: "The content mapper process could not be started or initialized.",
        reports_unnecessary: false,
        elided_in_compatibility_pyramid: false,
        reports_deprecated: false,
    };

pub struct CachedSourceFile {
    pub file: Arc<SourceFile>,
    pub mod_time: SystemTime,
}

pub struct WatchCompilerHost {
    pub inner: compiler::CompilerHostImpl,
    pub cache: tsox_core::collections::syncmap::SyncMap<Path, Arc<CachedSourceFile>>,
}

impl WatchCompilerHost {
    pub fn get_source_file(&self, opts: &SourceFileParseOptions) -> Option<Arc<SourceFile>> {
        let info = self.inner.fs().stat(&opts.file_name);
        let path = Path(opts.path.clone());

        if let Some(cached) = self.cache.load(&path) {
            if let Some(info) = &info {
                if info.modified == cached.mod_time {
                    return Some(Arc::clone(&cached.file));
                }
            }
        }

        let file = CompilerHost::get_source_file(&self.inner, opts);
        match &file {
            Some(file) => {
                if let Some(info) = &info {
                    self.cache.store(
                        path,
                        Arc::new(CachedSourceFile {
                            file: Arc::clone(file),
                            mod_time: info.modified,
                        }),
                    );
                }
            }
            None => {
                self.cache.delete(&path);
            }
        }
        file
    }
}

impl CompilerHost for WatchCompilerHost {
    fn fs(&self) -> &dyn FS {
        CompilerHost::fs(&self.inner)
    }

    fn fs_arc(&self) -> Arc<dyn FS> {
        CompilerHost::fs_arc(&self.inner)
    }

    fn current_directory(&self) -> &str {
        CompilerHost::current_directory(&self.inner)
    }

    fn default_library_path(&self) -> &str {
        CompilerHost::default_library_path(&self.inner)
    }

    fn content_mapper_project(&self) -> Option<Arc<dyn contentmapper::Project>> {
        CompilerHost::content_mapper_project(&self.inner)
    }

    fn get_source_file(&self, opts: &SourceFileParseOptions) -> Option<Arc<SourceFile>> {
        WatchCompilerHost::get_source_file(self, opts)
    }

    fn get_content_mapped_source_files(
        &self,
        parse_options: &SourceFileParseOptions,
        mapper: &tsox_compile::mig::m3l_cm::Mapper,
    ) -> Result<tsox_compile::mig::m3l_cm_2::SourceFiles, m4v_3::ContentMapperError> {
        CompilerHost::get_content_mapped_source_files(&self.inner, parse_options, mapper)
    }
}

pub struct Watcher {
    pub sys: Arc<dyn System>,
    pub config_file_name: String,
    pub config: ParsedCommandLine,
    pub compiler_options_from_command_line: CompilerOptions,
    pub command_line_raw: Option<OrderedMap<String, serde_json::Value>>,
    pub report_diagnostic: Box<dyn Fn(&ast::Diagnostic)>,
    pub report_error_summary: Box<dyn Fn(&[Arc<ast::Diagnostic>])>,
    pub report_watch_status: Box<dyn Fn(&ast::Diagnostic)>,
    pub testing: Option<Arc<dyn crate::mig::m5b::testing::CommandLineTesting>>,

    pub content_mapper_host: Option<Arc<dyn contentmapper::Host>>,
    pub content_mapper_project: Option<Arc<dyn contentmapper::Project>>,

    pub program: Option<Arc<incremental::Program>>,
    pub extended_config_cache: Option<ExtendedConfigCache>,
    pub config_modified: bool,
    pub config_has_errors: bool,
    pub config_file_paths: Vec<String>,

    pub source_file_cache: tsox_core::collections::syncmap::SyncMap<Path, Arc<CachedSourceFile>>,

    pub wm: WatchManager,
    pub seen_files: HashSet<Path>,
    pub config_mtimes: HashMap<String, SystemTime>,
    pub watch_set_dirty: bool,
    pub force_full_rebuild: bool,
    pub program_ready: bool,

    pub fast_path_builds: usize,
    pub full_builds: usize,
}

pub fn create_watcher(
    sys: Arc<dyn System>,
    mut config_parse_result: ParsedCommandLine,
    compiler_options_from_command_line: CompilerOptions,
    command_line_raw: Option<OrderedMap<String, serde_json::Value>>,
    report_diagnostic: Box<dyn Fn(&ast::Diagnostic)>,
    report_error_summary: Box<dyn Fn(&[Arc<ast::Diagnostic>])>,
    testing: Option<Arc<dyn crate::mig::m5b::testing::CommandLineTesting>>,
) -> Watcher {
    let sys_for_wm = Arc::clone(&sys);
    let mut wm =
        WatchManager::new(sys.writer(), Arc::new(move |dir| sys_for_wm.fs().directory_exists(dir)));
    if let Some(t) = &testing {
        if let Some(backend) = t.watch_backend() {
            wm.set_backend(backend);
        }
    }
    let mut w = Watcher {
        report_watch_status: Box::new(create_watch_status_reporter(
            sys.as_ref(),
            &config_parse_result.locale(),
            &config_parse_result.compiler_options,
            testing.clone(),
        )),
        sys,
        config: config_parse_result,
        compiler_options_from_command_line,
        command_line_raw,
        report_diagnostic,
        report_error_summary,
        testing,
        content_mapper_host: None,
        content_mapper_project: None,
        program: None,
        extended_config_cache: None,
        config_modified: false,
        config_has_errors: false,
        config_file_paths: Vec::new(),
        source_file_cache: tsox_core::collections::syncmap::SyncMap::new(),
        wm,
        seen_files: HashSet::new(),
        config_mtimes: HashMap::new(),
        watch_set_dirty: false,
        force_full_rebuild: false,
        program_ready: false,
        fast_path_builds: 0,
        full_builds: 0,
        config_file_name: String::new(),
    };
    if let Some(config_file) = &w.config.config_file {
        w.config_file_name = config_file.source_file.file_name.clone();
    }
    w
}

impl Watcher {
    pub fn start(&mut self, ctx: &crate::mig::m5b_4::Context) {
        self.content_mapper_host =
            new_content_mapper_host(&*self.sys, &self.config.compiler_options);
        let owns_host = self.content_mapper_host.is_some() && self.testing.is_none();
        let config_snapshot = self.config.clone();
        self.replace_content_mapper_project(&config_snapshot);
        self.wm.lock();
        self.extended_config_cache = Some(ExtendedConfigCache::default());
        let host: Arc<dyn CompilerHost> = Arc::new(WatchCompilerHost {
            inner: compiler::CompilerHostImpl::new(
                self.sys.fs(),
                self.sys.current_directory().to_string(),
                self.sys.default_library_path().to_string(),
            ),
            cache: tsox_core::collections::syncmap::SyncMap::new(),
        });
        let reader = incremental::new_build_info_reader(Arc::clone(&host));
        self.program = incremental::read_build_info_program(
            &self.config,
            &reader,
            host.as_ref(),
        )
        .map(Arc::new);

        if !self.config_file_name.is_empty() {
            let mut paths = vec![self.config_file_name.clone()];
            paths.extend(self.config.extended_source_files().iter().cloned());
            self.config_file_paths = paths;
        }

        if self.sys.environment_variable("TS_WATCH_DEBUG").map_or(false, |v| !v.is_empty()) {
            self.wm.debug_log = Some(Arc::new(Mutex::new(self.sys.writer())));
        }

        if self.testing.is_none() {
            self.wm.ensure_default_backend();
        }

        (self.report_watch_status)(&new_compiler_diagnostic(
            &tsox_core::diagnostics::STARTING_COMPILATION_IN_WATCH_MODE,
            &[],
        ));
        self.watch_set_dirty = true;
        if self.do_build().is_err() {
            self.wm.force_overflow();
        }
        self.wm.unlock();

        if self.testing.is_none() {
            let sys_for_loop = Arc::clone(&self.sys);
            let wm = std::mem::replace(
                &mut self.wm,
                WatchManager::new(
                    self.sys.writer(),
                    Arc::new(move |dir| sys_for_loop.fs().directory_exists(dir)),
                ),
            );
            wm.run_loop(ctx, &mut || self.do_cycle());
            self.wm = wm;
        }
        if owns_host {
            if let Some(host) = &mut self.content_mapper_host {
                let _ = host.close();
            }
        }
    }

    pub fn replace_content_mapper_project(&mut self, config: &ParsedCommandLine) {
        let host = match &mut self.content_mapper_host {
            Some(h) => h,
            None => return,
        };
        let project = host.project(&contentmapper::ProjectSpec {
            config_file_name: config.config_name(),
            mappers: config
                .content_mappers()
                .iter()
                .map(|m| Arc::new(content_mapper_to_mapper(m)))
                .collect(),
            compiler_options: Some(Arc::new(config.compiler_options.clone())),
        });
        if let Some(old) = &self.content_mapper_project {
            let _ = old.close();
        }
        self.content_mapper_project = project;
    }

    pub fn content_mapper_watched_files(&self) -> Vec<String> {
        let mut files: Vec<String> = Vec::new();
        for mapper in self.config.content_mappers() {
            if !mapper.package_directory.is_empty() && mapper.contribution_id.is_empty() {
                files.push(tspath::combine_paths(&mapper.package_directory, &["package.json"]));
            }
        }
        if let Some(project) = &self.content_mapper_project {
            match project.watched_files() {
                Ok(dynamic_files) => files.extend(dynamic_files),
                Err(err) => {
                    (self.report_diagnostic)(&content_mapper_project_diagnostic(
                        &tsox_compile::mig::m4v_2::ContentMapperError::from(m4v_3::ContentMapperError(
                            err.to_string(),
                        )),
                    ));
                    return files;
                }
            }
        }
        files.sort();
        files.dedup();
        files
    }

    pub fn compute_desired_watches(&mut self, seen_file_paths: &[String]) -> HashMap<String, bool> {
        let cwd = self.sys.current_directory().to_string();

        let mut desired_dirs: HashMap<String, bool> = HashMap::new();

        if self.config.config_file.is_some() {
            for (dir, recursive) in self.config.wildcard_directories() {
                let real_dir = self.sys.fs().realpath(&dir);
                desired_dirs.insert(real_dir, recursive);
            }
        }

        if self.config.config_file.is_none() && desired_dirs.is_empty() {
            let dir = self.sys.fs().realpath(&cwd);
            desired_dirs.insert(dir, false);
        }

        for cfg_path in &self.config_file_paths {
            let real_path = self.sys.fs().realpath(cfg_path);
            let dir = tspath::get_directory_path(&real_path);
            desired_dirs.entry(dir).or_insert(false);
        }

        if self.config.config_file.is_none() {
            for file_name in self.config.file_names() {
                let abs_path = tspath::get_normalized_absolute_path(file_name, &cwd);
                let real_path = self.sys.fs().realpath(&abs_path);
                let dir = tspath::get_directory_path(&real_path);
                desired_dirs.entry(dir).or_insert(false);
            }
        }

        let resolved_dirs = self.wm.resolve_desired_dirs(&desired_dirs);

        let mut coverage = DirWatchSet::new(self.compare_paths_options());
        for (dir, recursive) in &resolved_dirs {
            coverage.set(dir, *recursive);
        }
        for file_path in seen_file_paths {
            let dir = tspath::get_directory_path(file_path);
            if !coverage.covered(&dir) && can_watch_directory(&dir) {
                coverage.set(&dir, false);
            }
        }

        self.wm.resolve_desired_dirs(&coverage.dirs())
    }

    pub fn reconcile_watches(&mut self, seen_file_paths: &[String]) -> Result<(), String> {
        let desired_dirs = self.compute_desired_watches(seen_file_paths);
        self.wm.reconcile_watches(&desired_dirs)
    }

    pub fn compare_paths_options(&self) -> ComparePathsOptions {
        ComparePathsOptions {
            use_case_sensitive_file_names: self.sys.fs().use_case_sensitive_file_names(),
            current_directory: self.sys.current_directory().to_string(),
        }
    }

    pub fn do_cycle(&mut self) {
        self.wm.lock();
        let (changed_paths, overflow) = self.wm.drain_events();
        let has_events = !changed_paths.is_empty() || overflow;

        if self.recheck_ts_config(self.content_mapper_manifest_changed(&changed_paths)) {
            self.wm.unlock();
            return;
        }

        if has_events && !overflow && !self.config_modified {
            if self.is_relevant_change(&changed_paths) {
                self.evict_changed_source_files(&changed_paths);
                let case_sensitive = self.sys.fs().use_case_sensitive_file_names();
                let cwd = self.sys.current_directory().to_string();
                let program_files = self.program.as_ref().unwrap().get_program().files_by_path();
                let content_mapper_watched_files: HashSet<Path> = self
                    .content_mapper_watched_files()
                    .into_iter()
                    .map(|file_name| tspath::to_path(&file_name, &cwd, case_sensitive))
                    .collect();
                let mut content_mapper_config_changed = false;
                for event_path in changed_paths.keys() {
                    if self.sys.fs().directory_exists(event_path) {
                        self.watch_set_dirty = true;
                        continue;
                    }
                    let p = tspath::to_path(event_path, &cwd, case_sensitive);
                    if content_mapper_watched_files.contains(&p) {
                        content_mapper_config_changed = true;
                        self.force_full_rebuild = true;
                    }
                    if self.config.config_file.is_some() && self.config.possibly_matches_file_name(event_path) {
                        if !self.seen_files.contains(&p) {
                            self.watch_set_dirty = true;
                            self.force_full_rebuild = true;
                            continue;
                        }
                    }
                    if let Some(source_file) = program_files.get(p.as_str()) {
                        if !tsox_compile::mig::m4w_2::source_file_content_mapper_identity(source_file)
                            .is_empty()
                        {
                            self.force_full_rebuild = true;
                        }
                    } else if !program_files.contains_key(p.as_str()) && self.seen_files.contains(&p) {
                        self.force_full_rebuild = true;
                    }
                }
                if content_mapper_config_changed && self.content_mapper_project.is_some() {
                    if self.content_mapper_project.as_mut().unwrap().refresh().is_err() {
                        (self.report_diagnostic)(&new_compiler_diagnostic(
                            &THE_CONTENT_MAPPER_PROCESS_COULD_NOT_BE_STARTED_OR_INITIALIZED,
                            &[],
                        ));
                        self.wm.unlock();
                        return;
                    }
                }
            } else {
                if let Some(mut debug_log) = self.wm.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                    let _ = writeln!(
                        debug_log,
                        "[watch] DoCycle: {} event(s) not relevant to compilation, skipping rebuild",
                        changed_paths.len()
                    );
                }
                if let Some(testing) = &self.testing {
                    testing.on_program(self.program.as_deref().unwrap());
                }
                self.wm.unlock();
                return;
            }
        } else if overflow {
            self.source_file_cache = tsox_core::collections::syncmap::SyncMap::new();
            self.watch_set_dirty = true;
            self.force_full_rebuild = true;
        } else if !has_events && !self.config_modified {
            if let Some(mut debug_log) = self.wm.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                let _ = writeln!(debug_log, "[watch] DoCycle: no events, skipping");
            }
            if let Some(testing) = &self.testing {
                testing.on_program(self.program.as_deref().unwrap());
            }
            self.wm.unlock();
            return;
        }

        (self.report_watch_status)(&new_compiler_diagnostic(
            &tsox_core::diagnostics::FILE_CHANGE_DETECTED_STARTING_INCREMENTAL_COMPILATION,
            &[],
        ));
        if self.do_build().is_err() {
            self.wm.force_overflow();
        }
        self.wm.unlock();
    }

    pub fn is_relevant_change(&self, changed_paths: &HashMap<String, EventKind>) -> bool {
        let case_sensitive = self.sys.fs().use_case_sensitive_file_names();
        let cwd = self.sys.current_directory().to_string();
        let opts = self.compare_paths_options();
        let content_mapper_watched_files: HashSet<Path> = self
            .content_mapper_watched_files()
            .into_iter()
            .map(|file_name| tspath::to_path(&file_name, &cwd, case_sensitive))
            .collect();
        for event_path in changed_paths.keys() {
            let p = tspath::to_path(event_path, &cwd, case_sensitive);
            if content_mapper_watched_files.contains(&p) {
                return true;
            }
            if self.seen_files.contains(&p) {
                return true;
            }
            if self.config.config_file.is_some() && self.config.possibly_matches_file_name(event_path) {
                return true;
            }
            if self.config.config_file.is_some() && self.config.possibly_matches_directory_name(&p) {
                return true;
            }
            if self.sys.fs().directory_exists(event_path) && self.wm.is_path_under_watch(event_path, &opts) {
                return true;
            }
        }
        false
    }

    pub fn do_build(&mut self) -> Result<(), String> {
        if self.config_modified {
            self.source_file_cache = tsox_core::collections::syncmap::SyncMap::new();
            self.watch_set_dirty = true;
        }

        let mut reloaded_file_names = false;
        if self.watch_set_dirty {
            if self.config.config_file.is_some() && !self.config.wildcard_directories().is_empty() {
                let new_config =
                    self.config.reload_file_names_of_parsed_command_line(self.sys.fs().as_ref());
                reloaded_file_names = true;
                if self.config.file_names() != new_config.file_names() {
                    self.config = new_config;
                } else {
                    self.watch_set_dirty = false;
                    self.config = new_config;
                }
            } else if !self.config_modified {
                self.watch_set_dirty = false;
            }
        }

        if self.program.is_some() && self.program_ready && !self.config_modified && !self.watch_set_dirty && !self.force_full_rebuild {
            let cached = Arc::new(CachedFS::new(self.sys.fs()));
            let inner_host = compiler::CompilerHostImpl::new(
                cached.clone(),
                self.sys.current_directory().to_string(),
                self.sys.default_library_path().to_string(),
            );
            let host = Arc::new(WatchCompilerHost { inner: inner_host, cache: self.source_file_cache.clone() });

            if self.try_update_program(host) {
                self.fast_path_builds += 1;
                let result = self.compile_and_emit();
                cached.disable_and_clear_cache();

                self.config_mtimes = HashMap::with_capacity(self.config_file_paths.len());
                for cfg_path in &self.config_file_paths {
                    if let Some(s) = self.sys.fs().stat(cfg_path) {
                        self.config_mtimes.insert(cfg_path.clone(), s.modified);
                    }
                }
                self.config_modified = false;

                let error_count = result.diagnostics.len();
                if error_count == 1 {
                    (self.report_watch_status)(&new_compiler_diagnostic(
                        &tsox_core::diagnostics::FOUND_1_ERROR_WATCHING_FOR_FILE_CHANGES,
                        &[],
                    ));
                } else {
                    (self.report_watch_status)(&new_compiler_diagnostic(
                        &tsox_core::diagnostics::FOUND_0_ERRORS_WATCHING_FOR_FILE_CHANGES,
                        &[error_count.to_string()],
                    ));
                }
                if let Some(testing) = &self.testing {
                    testing.on_program(self.program.as_deref().unwrap());
                }
                return Ok(());
            }
            cached.disable_and_clear_cache();
        }

        let cached = Arc::new(CachedFS::new(self.sys.fs()));
        let tfs = Arc::new(TrackingFS::new(cached.clone()));
        let inner_host = compiler::CompilerHostImpl::new(
            tfs.clone(),
            self.sys.current_directory().to_string(),
            self.sys.default_library_path().to_string(),
        );
        let host = Arc::new(WatchCompilerHost { inner: inner_host, cache: self.source_file_cache.clone() });

        if self.config.config_file.is_some() {
            for dir in self.config.wildcard_directories().keys() {
                tfs.seen_files.lock().unwrap().insert(dir.clone());
            }
            if !reloaded_file_names && !self.watch_set_dirty && !self.config.wildcard_directories().is_empty() {
                self.config =
                    self.config.reload_file_names_of_parsed_command_line(self.sys.fs().as_ref());
            }
        }
        for path in &self.config_file_paths {
            tfs.seen_files.lock().unwrap().insert(path.clone());
        }
        for path in self.content_mapper_watched_files() {
            tfs.seen_files.lock().unwrap().insert(path);
        }

        self.program = Some(Arc::new(incremental::new_program(
            compiler::new_program(ProgramOptions {
                config: self.config.clone(),
                host,
                use_source_of_project_reference: false,
                single_threaded: tsox_core::core::tristate::Tristate::Unknown,
                create_checker_pool: None,
                typings_location: String::new(),
                project_name: String::new(),
                tracing: None,
                skip_module_resolution: false,
            }),
            self.program.as_deref(),
            None,
            Some(Arc::new(|| std::time::SystemTime::now())),
            self.testing.is_some(),
        )));
        self.program_ready = true;
        self.full_builds += 1;

        let result = self.compile_and_emit();
        cached.disable_and_clear_cache();

        let case_sensitive = self.sys.fs().use_case_sensitive_file_names();
        let cwd = self.sys.current_directory().to_string();
        let seen_slice: Vec<String> = {
            let seen_files = tfs.seen_files.lock().unwrap();
            let mut seen: Vec<String> = seen_files.iter().cloned().collect();
            seen.sort();
            seen
        };
        let mut seen_files = HashSet::with_capacity(seen_slice.len());
        for p in &seen_slice {
            seen_files.insert(tspath::to_path(p, &cwd, case_sensitive));
        }
        self.seen_files = seen_files;

        self.config_mtimes = HashMap::with_capacity(self.config_file_paths.len());
        for cfg_path in &self.config_file_paths {
            if let Some(s) = self.sys.fs().stat(cfg_path) {
                self.config_mtimes.insert(cfg_path.clone(), s.modified);
            }
        }

        if let Err(err) = self.reconcile_watches(&seen_slice) {
            let mut writer = self.sys.writer();
            let _ = writeln!(writer, "{}", err);
            return Err(err);
        }
        self.watch_set_dirty = false;
        self.config_modified = false;
        self.force_full_rebuild = false;

        let program_files = self.program.as_ref().unwrap().get_program().files_by_path();
        for path in self.source_file_cache.keys() {
            if !program_files.contains_key(path.as_str()) {
                self.source_file_cache.delete(&path);
            }
        }

        let error_count = result.diagnostics.len();
        if error_count == 1 {
            (self.report_watch_status)(&new_compiler_diagnostic(
                &tsox_core::diagnostics::FOUND_1_ERROR_WATCHING_FOR_FILE_CHANGES,
                &[],
            ));
        } else {
            (self.report_watch_status)(&new_compiler_diagnostic(
                &tsox_core::diagnostics::FOUND_0_ERRORS_WATCHING_FOR_FILE_CHANGES,
                &[error_count.to_string()],
            ));
        }

        if let Some(testing) = &self.testing {
            testing.on_program(self.program.as_deref().unwrap());
        }
        Ok(())
    }

    pub fn try_update_program(&mut self, host: Arc<WatchCompilerHost>) -> bool {
        let old_program = self.program.as_ref().unwrap().get_program();

        let mut changed_path: Option<Path> = None;
        let mut changed_count = 0;
        for (path, file) in old_program.files_by_path() {
            if !tsox_compile::mig::m4w_2::source_file_content_mapper_identity(file).is_empty() {
                continue;
            }
            if self.source_file_cache.load(&Path(path.clone())).is_none() {
                changed_path = Some(Path(path.clone()));
                changed_count += 1;
                if changed_count > 1 {
                    return false;
                }
            }
        }
        let changed_path = match changed_path {
            Some(p) => p,
            None => return false,
        };

        if let Some(old_file) = old_program.files_by_path().get(changed_path.as_str()) {
            if let Some(new_file) =
                host.get_source_file(&tsox_frontend::ast::mig::m3b_2::parse_options(old_file.as_ref()))
            {
                if !equal_jsx_implicit_import(old_program.options(), old_file, &new_file) {
                    return false;
                }
            }
        }

        let (new_program, _, reused) = old_program.reuse_program(changed_path.as_str(), host, None);
        if reused {
            if let Some(new_program) = new_program {
                self.program = Some(Arc::new(incremental::new_program(
                    new_program,
                    self.program.as_deref(),
                    None,
                    Some(Arc::new(|| std::time::SystemTime::now())),
                    self.testing.is_some(),
                )));
            }
        }
        reused
    }

    pub fn fast_path_builds(&self) -> usize {
        self.fast_path_builds
    }

    pub fn full_builds(&self) -> usize {
        self.full_builds
    }

    pub fn evict_changed_source_files(&mut self, changed_paths: &HashMap<String, EventKind>) {
        let case_sensitive = self.sys.fs().use_case_sensitive_file_names();
        let cwd = self.sys.current_directory().to_string();
        for event_path in changed_paths.keys() {
            let p = tspath::to_path(event_path, &cwd, case_sensitive);
            if self.source_file_cache.load(&p).is_some() {
                if let Some(mut debug_log) = self.wm.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                    let _ = writeln!(debug_log, "[watch] evicting cached source file: {}", p);
                }
                self.source_file_cache.delete(&p);
            }
        }
    }

    pub fn compile_and_emit(&mut self) -> CompileAndEmitResult {
        let mut compile_times = CompileTimes::default();
        let emit_program = super::m5a_3::EmitProgram;
        let program_like = EmitProgramLike { program: &emit_program };
        let mut input = EmitInput {
            sys: self.sys.as_ref(),
            program_like: &program_like,
            program: &emit_program,
            config: &self.config,
            report_diagnostic: self.report_diagnostic.as_ref(),
            report_error_summary: self.report_error_summary.as_ref(),
            compile_times: &mut compile_times,
            testing: self.testing.as_deref(),
            tracing: None,
        };
        emit_files_and_report_errors(&mut input)
    }

    pub fn content_mapper_manifest_changed(&self, changed_paths: &HashMap<String, EventKind>) -> bool {
        for mapper in self.config.content_mappers() {
            if mapper.package_directory.is_empty() || !mapper.contribution_id.is_empty() {
                continue;
            }
            if changed_paths.contains_key(&tspath::combine_paths(&mapper.package_directory, &["package.json"])) {
                return true;
            }
        }
        false
    }

    pub fn recheck_ts_config(&mut self, force: bool) -> bool {
        if self.config_file_name.is_empty() {
            return false;
        }

        if !force && !self.config_has_errors && !self.config_file_paths.is_empty() {
            let mut changed = false;
            for path in &self.config_file_paths {
                let old_mtime = self.config_mtimes.get(path);
                let s = self.sys.fs().stat(path);
                match old_mtime {
                    None => {
                        if s.is_some() {
                            changed = true;
                            break;
                        }
                    }
                    Some(old_mtime) => {
                        if s.is_none() || s.unwrap().modified != *old_mtime {
                            changed = true;
                            break;
                        }
                    }
                }
            }
            if !changed {
                return false;
            }
        }

        let config_parse_result = match self.parse_config_file() {
            Some(c) => c,
            None => return true,
        };
        if self.config_has_errors {
            self.config_modified = true;
        }
        self.config_has_errors = false;
        let mut paths = vec![self.config_file_name.clone()];
        paths.extend(config_parse_result.extended_source_files().iter().cloned());
        self.config_file_paths = paths;
        if self.config.raw_options != config_parse_result.raw_options {
            self.config_modified = true;
        }
        self.replace_content_mapper_project(&config_parse_result);
        self.config = config_parse_result;
        false
    }

    pub fn parse_config_file(&mut self) -> Option<ParsedCommandLine> {
        let extended_config_cache = ExtendedConfigCache::default();
        let sys = tsox_tsoptions::mig::m5j_2::ParseConfigHost {
            fs: self.sys.fs(),
            current_directory: self.sys.current_directory().to_string(),
        };
        let command_line_raw = self
            .command_line_raw
            .as_ref()
            .map(|raw| raw.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>());
        let (config_parse_result, errors) = get_parsed_command_line_of_config_file(
            &self.config_file_name,
            Some(&self.compiler_options_from_command_line),
            command_line_raw.as_ref(),
            &sys,
            None,
        );
        if !errors.is_empty() {
            for e in &errors {
                (self.report_diagnostic)(e);
            }
            self.config_has_errors = true;
            let error_count = errors.len();
            if error_count == 1 {
                (self.report_watch_status)(&new_compiler_diagnostic(
                    &tsox_core::diagnostics::FOUND_1_ERROR_WATCHING_FOR_FILE_CHANGES,
                    &[],
                ));
            } else {
                (self.report_watch_status)(&new_compiler_diagnostic(
                    &tsox_core::diagnostics::FOUND_0_ERRORS_WATCHING_FOR_FILE_CHANGES,
                    &[error_count.to_string()],
                ));
            }
            return None;
        }
        self.extended_config_cache = Some(extended_config_cache);
        config_parse_result
    }
}

pub fn equal_jsx_implicit_import(
    options: &CompilerOptions,
    old_file: &SourceFile,
    new_file: &SourceFile,
) -> bool {
    let is_jsx = |file: &SourceFile| {
        file.script_kind == ScriptKind::Jsx
            || file.script_kind == ScriptKind::Tsx
    };
    if !is_jsx(old_file) && !is_jsx(new_file) {
        return true;
    }
    let old_import = get_jsx_runtime_import(&get_jsx_implicit_import_base(options, old_file), options);
    let new_import = get_jsx_runtime_import(&get_jsx_implicit_import_base(options, new_file), options);
    old_import == new_import
}

fn get_trace_from_sys(
    sys: &dyn System,
    locale: Option<tsox_core::locale::Locale>,
    testing: Option<&dyn crate::mig::m5b::testing::CommandLineTesting>,
) -> super::m5a::TraceFn {
    get_trace_from_sys_tsc(sys, locale, testing)
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
