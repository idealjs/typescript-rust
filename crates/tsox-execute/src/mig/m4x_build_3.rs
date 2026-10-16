#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::io::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::tristate::Tristate;
use tsox_core::core::work_group::WorkGroup;
use tsox_core::tspath::{self, Path};
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use tsox_frontend::ast::Diagnostic;
use tsox_core::diagnostics::messages_generated as dg;
use tsox_compile::mig::m3l_cm_2::Host as _;
use super::m4x_build::{
    BuildInfoEntry, BuildTask, BuildKind, OrchestratorResult, OrchestratorResultStruct,
    TaskResult, UpstreamTask,
};
use super::m4x_build_2::Host;

struct SendPtr(*mut Orchestrator);
unsafe impl Send for SendPtr {}

struct SendWg(Arc<dyn WorkGroup>);
unsafe impl Send for SendWg {}

fn queue_create_build_task(
    orchestrator: SendPtr,
    old_tasks: Option<Arc<SharedTaskMap>>,
    config: String,
    wg: SendWg,
) { ::tsox_core::fntrace::enter("queue_create_build_task"); 
    let orchestrator = unsafe { &mut *orchestrator.0 };
    orchestrator.create_build_task(old_tasks.as_ref(), &config, &wg.0);
}

type SharedTaskMap = SyncMap<Path, Arc<Mutex<BuildTask>>>;

pub struct Orchestrator {
    pub opts: Options,
    pub host: Arc<Host>,
    pub compare_paths_options: tsox_core::tspath::ComparePathsOptions,
    pub tasks: SharedTaskMap,
    pub order: Vec<String>,
    pub errors: Vec<Arc<Diagnostic>>,
    pub content_mapper_host: Option<Arc<crate::mig::m5a::ContentMapperHost>>,
    pub wm: crate::mig::m5b_4::WatchManager,
    pub watch_status_reporter: Option<Box<dyn FnMut(&Diagnostic) + Send>>,
    pub error_summary_reporter: Option<Box<dyn FnMut(&[Arc<Diagnostic>]) + Send>>,
}

#[derive(Clone)]
pub struct Options {
    pub command: tsox_tsoptions::tsoptions::ParsedBuildCommandLine,
    pub sys: Arc<dyn crate::execute::System>,
    pub testing: Option<Arc<dyn crate::mig::m5a::CommandLineTesting>>,
}

impl Orchestrator {
    fn placeholder() -> Self { ::tsox_core::fntrace::enter("placeholder"); 
        unreachable!("placeholder instance replaced by new_orchestrator")
    }

    pub fn relative_file_name(&self, file_name: &str) -> String { ::tsox_core::fntrace::enter("relative_file_name"); 
        let current_directory = self.compare_paths_options.current_directory.clone();
        tsox_core::tspath::mig::m3i::get_relative_path_from_directory(
            &current_directory,
            file_name,
            &self.compare_paths_options,
        )
    }

    pub fn to_path(&self, file_name: &str) -> Path { ::tsox_core::fntrace::enter("to_path"); 
        tsox_core::tspath::to_path(
            file_name,
            &self.compare_paths_options.current_directory,
            self.compare_paths_options.use_case_sensitive_file_names,
        )
    }

    pub fn order_list(&self) -> Vec<String> { ::tsox_core::fntrace::enter("order_list"); 
        self.order.clone()
    }

    pub fn upstream(&self, config_name: &str) -> Vec<String> { ::tsox_core::fntrace::enter("upstream"); 
        let path = self.to_path(config_name);
        let task = self.get_task(&path);
        let task = task.lock().unwrap();
        task.up_stream
            .iter()
            .map(|t| t.task.lock().unwrap().config.clone())
            .collect()
    }

    pub fn downstream(&self, config_name: &str) -> Vec<String> { ::tsox_core::fntrace::enter("downstream"); 
        let path = self.to_path(config_name);
        let task = self.get_task(&path);
        let task = task.lock().unwrap();
        task.down_stream
            .iter()
            .map(|t| t.lock().unwrap().config.clone())
            .collect()
    }

    pub fn get_task(&self, path: &Path) -> Arc<Mutex<BuildTask>> { ::tsox_core::fntrace::enter("get_task"); 
        match self.tasks.load(path) {
            Some(task) => task,
            None => panic!("No build task found for {}", path.0),
        }
    }

    pub fn generate_graph_reusing_old_tasks(&mut self) { ::tsox_core::fntrace::enter("generate_graph_reusing_old_tasks"); 
        let tasks = std::mem::replace(&mut self.tasks, SyncMap::new());
        self.order.clear();
        self.errors.clear();
        self.generate_graph(Some(tasks));
    }

    pub fn generate_graph(&mut self, old_tasks: Option<SyncMap<Path, Arc<Mutex<BuildTask>>>>) { ::tsox_core::fntrace::enter("generate_graph"); 
        let projects = self.opts.command.resolved_project_paths();
        let wg: Arc<dyn WorkGroup> = Arc::from(tsox_core::core::work_group::new_work_group(
            self.opts.command.compiler_options.single_threaded == Tristate::True,
        ));
        let old_tasks = old_tasks.map(Arc::new);
        self.create_build_tasks(old_tasks.as_ref(), &projects, &wg);
        wg.run_and_wait();

        let mut completed: Set<Path> = Set::new();
        let mut analyzing: Set<Path> = Set::new();
        let mut circularity_stack: Vec<String> = Vec::new();
        for project in &projects {
            self.setup_build_task(
                project,
                None,
                false,
                &mut completed,
                &mut analyzing,
                &mut circularity_stack,
            );
        }
        if let Some(old_tasks) = old_tasks {
            for (path, old_task) in old_tasks.to_hash_map() {
                if let Some(task) = self.tasks.load(&path) {
                    if Arc::ptr_eq(&task, &old_task) {
                        continue;
                    }
                }
                let old_task = old_task.lock().unwrap();
                if let Some(project) = old_task.content_mapper_project.as_ref() {
                    let _ = project.close();
                }
            }
        }
    }

    pub fn create_build_tasks(
        &mut self,
        old_tasks: Option<&Arc<SharedTaskMap>>,
        configs: &[String],
        wg: &Arc<dyn WorkGroup>,
    ) { ::tsox_core::fntrace::enter("create_build_tasks"); 
        for config in configs {
            let orchestrator = SendPtr(self as *mut Orchestrator);
            let old_tasks = old_tasks.cloned();
            let config = config.clone();
            let wg = SendWg(Arc::clone(wg));
            let wg_inner = SendWg(Arc::clone(&wg.0));
            wg.0.queue(Box::new(move || {
                queue_create_build_task(orchestrator, old_tasks, config, wg_inner);
            }));
        }
    }

    fn create_build_task(
        &mut self,
        old_tasks: Option<&Arc<SharedTaskMap>>,
        config: &str,
        wg: &Arc<dyn WorkGroup>,
    ) { ::tsox_core::fntrace::enter("create_build_task"); 
        let path = self.to_path(config);
        let mut reused: Option<Arc<Mutex<BuildTask>>> = None;
        let mut build_info: Option<BuildInfoEntry> = None;
        if let Some(old_tasks) = old_tasks {
            if let Some(existing) = old_tasks.load(&path) {
                let existing_dirty = existing.lock().unwrap().dirty;
                if !existing_dirty {
                    reused = Some(existing);
                } else {
                    let mut existing = existing.lock().unwrap();
                    if let Some(project) = existing.content_mapper_project.as_ref() {
                        let _ = project.close();
                    }
                    build_info = existing.build_info_entry.take();
                }
            }
        }
        let task = match reused {
            Some(task) => task,
            None => Arc::new(Mutex::new(BuildTask {
                config: config.to_string(),
                resolved: None,
                up_stream: Vec::new(),
                down_stream: Vec::new(),
                status: None,
                done: std::sync::mpsc::channel().1,
                result: None,
                prev_reporter: None,
                report_done: std::sync::mpsc::channel().1,
                build_info_entry: build_info,
                package_jsons: Vec::new(),
                errors: Vec::new(),
                pending: std::sync::atomic::AtomicBool::new(true),
                is_initial_cycle: old_tasks.is_none(),
                dirty: false,
                content_mapper_project_once: std::sync::Once::new(),
                content_mapper_project: None,
                content_mapper_project_err: None,
            })),
        };
        let (_, loaded) = self.tasks.load_or_store(path.clone(), task.clone());
        if loaded {
            return;
        }
        let resolved = self.host.get_resolved_project_reference(config, path.clone());
        {
            let mut task = task.lock().unwrap();
            task.resolved = resolved.clone();
            task.up_stream = Vec::new();
        }
        if let Some(resolved) = resolved {
            let sub_references = unsafe {
                &mut *(Arc::as_ptr(&resolved) as *mut ParsedCommandLine)
            }
            .resolved_project_reference_paths();
            self.create_build_tasks(old_tasks, &sub_references, wg);
        }
    }

    pub fn setup_build_task(
        &mut self,
        config_name: &str,
        down_stream: Option<Arc<Mutex<BuildTask>>>,
        in_circular_context: bool,
        completed: &mut Set<Path>,
        analyzing: &mut Set<Path>,
        circularity_stack: &mut Vec<String>,
    ) -> Option<Arc<Mutex<BuildTask>>> { ::tsox_core::fntrace::enter("setup_build_task"); 
        let path = self.to_path(config_name);
        let task = self.get_task(&path);
        if !completed.has(&path) {
            if analyzing.has(&path) {
                if !in_circular_context {
                    self.errors
                        .push(Arc::new(tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
                            dg::PROJECT_REFERENCES_MAY_NOT_FORM_A_CIRCULAR_GRAPH_CYCLE_DETECTED_COLON_0,
                            vec![circularity_stack.join("\n")],
                        )));
                }
                return None;
            }
            analyzing.add(path.clone());
            circularity_stack.push(config_name.to_string());
            let resolved = task.lock().unwrap().resolved.clone();
            if let Some(resolved) = resolved {
                let sub_references = unsafe {
                    &mut *(Arc::as_ptr(&resolved) as *mut ParsedCommandLine)
                }
                .resolved_project_reference_paths();
                let project_references = resolved.project_references().to_vec();
                for (index, sub_reference) in sub_references.into_iter().enumerate() {
                    let circular = project_references
                        .get(index)
                        .map(|reference| reference.circular)
                        .unwrap_or(false);
                    let upstream = self.setup_build_task(
                        &sub_reference,
                        Some(Arc::clone(&task)),
                        in_circular_context || circular,
                        completed,
                        analyzing,
                        circularity_stack,
                    );
                    if let Some(upstream) = upstream {
                        task.lock()
                            .unwrap()
                            .up_stream
                            .push(UpstreamTask {
                                task: upstream,
                                ref_index: index,
                            });
                    }
                }
            }
            circularity_stack.pop();
            completed.add(path.clone());
            task.lock().unwrap().report_done = std::sync::mpsc::channel().1;
            if let Some(prev) = self.order.last().cloned() {
                let prev_path = self.to_path(&prev);
                let prev_task = self.get_task(&prev_path);
                task.lock().unwrap().prev_reporter = Some(prev_task);
            }
            task.lock().unwrap().done = std::sync::mpsc::channel().1;
            self.order.push(config_name.to_string());
        }
        if self.opts.command.compiler_options.watch == Tristate::True {
            if let Some(down_stream) = down_stream {
                task.lock().unwrap().down_stream.push(down_stream);
            }
        }
        Some(task)
    }

    pub fn start(
        &mut self,
        ctx: crate::mig::m5b_4::Context,
    ) -> crate::execute::CommandLineResult { ::tsox_core::fntrace::enter("start"); 
        self.content_mapper_host = crate::mig::m5a::new_content_mapper_host_concrete(
            self.opts.sys.as_ref(),
            &self.opts.command.compiler_options,
        );
        if self.content_mapper_host.is_some()
            && (self.opts.command.compiler_options.watch != Tristate::True
                || self.opts.testing.is_none())
        {
            let _ = self.content_mapper_host.as_ref().unwrap().close();
        }
        if self.opts.command.compiler_options.watch == Tristate::True {
            if let Some(watch_status_reporter) = self.watch_status_reporter.as_mut() {
                watch_status_reporter(&tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
                    dg::STARTING_COMPILATION_IN_WATCH_MODE,
                    vec![],
                ));
            }
        }
        self.generate_graph(None);
        let mut result = self.build_or_clean();
        if self.opts.command.compiler_options.watch == Tristate::True {
            self.watch(ctx);
        }
        result
    }

    pub fn watch(&mut self, ctx: crate::mig::m5b_4::Context) { ::tsox_core::fntrace::enter("watch"); 
        self.wm.lock();

        if self.opts.testing.is_none() {
            if let Some(value) = self.opts.sys.environment_variable("TS_WATCH_DEBUG") {
                if !value.is_empty() {
                    self.wm.debug_log = Some(Arc::new(Mutex::new(self.opts.sys.writer())));
                }
            }
            self.wm.ensure_default_backend();
        }

        self.update_watch();
        let desired_dirs = self.compute_desired_watches();
        if let Err(err) = self.wm.reconcile_watches(&desired_dirs) {
            writeln!(self.opts.sys.writer(), "{}", err).ok();
            self.wm.force_overflow();
        }
        self.reset_caches();

        self.wm.unlock();

        if self.opts.testing.is_none() {
            let this = self as *mut Orchestrator;
            self.wm.run_loop(&ctx, &mut || unsafe { (*this).do_cycle() });
        }
    }

    pub fn update_watch(&mut self) { ::tsox_core::fntrace::enter("update_watch"); 
        let old_cache = self.host.m_times.clone();
        self.host.m_times.clear();
        let paths = self.tasks.keys();
        for path in paths {
            if let Some(task) = self.tasks.load(&path) {
                task.lock().unwrap().update_watch(self, &old_cache);
            }
        }
    }

    pub fn do_cycle(&mut self) { ::tsox_core::fntrace::enter("do_cycle"); 
        self.wm.lock();

        let (changed_paths, overflow) = self.wm.drain_events();
        let has_events = !changed_paths.is_empty() || overflow;

        if !has_events {
            if let Some(mut debug_log) = self.wm.debug_log.as_ref().map(|log| log.lock().unwrap()) {
                writeln!(debug_log, "[watch] DoCycle: no events, skipping").ok();
            }
            self.wm.unlock();
            return;
        }

        let mut needs_config_update = std::sync::atomic::AtomicBool::new(false);
        let mut needs_update = std::sync::atomic::AtomicBool::new(false);

        if overflow {
            let paths = self.tasks.keys();
            for path in paths {
                if let Some(task) = self.tasks.load(&path) {
                    let mut task = task.lock().unwrap();
                    task.reset_config(self, path);
                    task.report_done = std::sync::mpsc::channel().1;
                    task.done = std::sync::mpsc::channel().1;
                }
            }
            needs_config_update.store(true, std::sync::atomic::Ordering::SeqCst);
            needs_update.store(true, std::sync::atomic::Ordering::SeqCst);
        } else {
            self.check_tasks_for_event_changes(
                &changed_paths,
                &needs_config_update,
                &needs_update,
            );
        }

        if !needs_update.load(std::sync::atomic::Ordering::SeqCst) {
            self.reset_caches();
            self.wm.unlock();
            return;
        }

        if let Some(watch_status_reporter) = self.watch_status_reporter.as_mut() {
            watch_status_reporter(&tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
                dg::FILE_CHANGE_DETECTED_STARTING_INCREMENTAL_COMPILATION,
                vec![],
            ));
        }
        if needs_config_update.load(std::sync::atomic::Ordering::SeqCst) {
            self.generate_graph_reusing_old_tasks();
        }

        self.build_or_clean();
        self.update_watch();
        let desired_dirs = self.compute_desired_watches();
        if let Err(err) = self.wm.reconcile_watches(&desired_dirs) {
            writeln!(self.opts.sys.writer(), "{}", err).ok();
            self.wm.force_overflow();
        }
        self.reset_caches();
        self.wm.unlock();
    }

    pub fn build_or_clean(&mut self) -> crate::execute::CommandLineResult { ::tsox_core::fntrace::enter("build_or_clean"); 
        if self.opts.command.build_options.clean != Tristate::True
            && self.opts.command.build_options.verbose == Tristate::True
        {
            let projects = self
                .order_list()
                .iter()
                .map(|p| format!("\r\n    * {}", self.relative_file_name(p)))
                .collect::<Vec<_>>()
                .join("");
            if let Some(watch_status_reporter) = self.watch_status_reporter.as_mut() {
                watch_status_reporter(&tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
                    dg::PROJECTS_IN_THIS_BUILD_COLON_0,
                    vec![projects],
                ));
            }
        }
        let mut build_result = OrchestratorResultStruct {
            result: Default::default(),
            errors: Vec::new(),
            statistics: Default::default(),
            files_to_delete: Vec::new(),
        };
        if self.errors.is_empty() {
            build_result.statistics.projects = self.order_list().len();
            let paths = self.tasks.keys();
            for path in paths {
                if let Some(task) = self.tasks.load(&path) {
                    self.build_or_clean_project(task, &path, &mut build_result);
                }
            }
        } else {
            build_result.result.status =
                crate::execute::ExitStatus::ProjectReferenceCycle_OutputsSkipped;
            for err in &self.errors {
                if let Some(error_summary_reporter) = self.error_summary_reporter.as_mut() {
                    error_summary_reporter(std::slice::from_ref(err));
                }
            }
            build_result.errors = self.errors.clone();
        }
        build_result.report(self);
        build_result.result
    }

    pub fn reset_caches(&mut self) { ::tsox_core::fntrace::enter("reset_caches"); 
        self.host.source_files.clear();
        self.host.config_times.clear();
    }

    pub fn resolve_build_info_file_name(&self, file_name: &str, build_info_dir: &str) -> String { ::tsox_core::fntrace::enter("resolve_build_info_file_name"); 
        if crate::mig::m4y_2::is_build_info_file_name_default_library(file_name) {
            return tspath::combine_paths(&self.host.default_library_path(), &[file_name]);
        }
        tspath::get_normalized_absolute_path(file_name, build_info_dir)
    }

    pub fn check_tasks_for_event_changes(
        &mut self,
        changed_paths: &HashMap<String, tsox_core::fswatch::mig::m5f_2::EventKind>,
        needs_config_update: &AtomicBool,
        needs_update: &AtomicBool,
    ) { ::tsox_core::fntrace::enter("check_tasks_for_event_changes"); 
        let mut normalized_paths: HashMap<Path, tsox_core::fswatch::mig::m5f_2::EventKind> =
            HashMap::with_capacity(changed_paths.len());
        for (event_path, kind) in changed_paths {
            normalized_paths.insert(self.to_path(event_path), *kind);
        }

        for index in 0..self.order.len() {
            let config = self.order[index].clone();
            let path = self.to_path(&config);
            let task = self.get_task(&path);

            let task_config = task.lock().unwrap().config.clone();
            let config_path = self.to_path(&task_config);
            if normalized_paths.contains_key(&config_path) {
                task.lock().unwrap().reset_config(self, path.clone());
                needs_config_update.store(true, Ordering::SeqCst);
                needs_update.store(true, Ordering::SeqCst);
                continue;
            }

            let resolved = task.lock().unwrap().resolved.clone();
            let Some(resolved) = resolved else {
                continue;
            };

            let mut config_changed = false;
            for file in resolved.extended_source_files() {
                let fp = self.to_path(file);
                if normalized_paths.contains_key(&fp) {
                    task.lock().unwrap().reset_config(self, path.clone());
                    needs_config_update.store(true, Ordering::SeqCst);
                    needs_update.store(true, Ordering::SeqCst);
                    config_changed = true;
                    break;
                }
            }
            if config_changed {
                continue;
            }
            for mapper in resolved.content_mappers() {
                if mapper.package_directory.is_empty() || !mapper.contribution_id.is_empty() {
                    continue;
                }
                let manifest_path = self.to_path(&tspath::combine_paths(
                    &mapper.package_directory,
                    &["package.json"],
                ));
                if normalized_paths.contains_key(&manifest_path) {
                    task.lock().unwrap().reset_config(self, path.clone());
                    needs_config_update.store(true, Ordering::SeqCst);
                    needs_update.store(true, Ordering::SeqCst);
                    config_changed = true;
                    break;
                }
            }
            if config_changed {
                continue;
            }

            let mut root_changed = false;
            let content_mapper_project = task.lock().unwrap().content_mapper_project.clone();
            if let Some(project) = content_mapper_project.as_ref() {
                let watched_files = match project.watched_files() {
                    Ok(files) => files,
                    Err(err) => {
                        let mut task = task.lock().unwrap();
                        task.content_mapper_project_err =
                            Some(tsox_compile::mig::m4v_3::ContentMapperError(err.to_string()));
                        task.reset_status();
                        needs_update.store(true, Ordering::SeqCst);
                        root_changed = true;
                        Vec::new()
                    }
                };
                for file_name in &watched_files {
                    if normalized_paths.contains_key(&self.to_path(file_name)) {
                        task.lock()
                            .unwrap()
                            .refresh_content_mapper_project(self);
                        task.lock().unwrap().reset_status();
                        needs_update.store(true, Ordering::SeqCst);
                        root_changed = true;
                        break;
                    }
                }
            }
            let file_names = resolved.file_names().to_vec();
            let mut roots: Set<Path> = Set::new();
            for file in &file_names {
                let fp = self.to_path(file);
                roots.add(fp);
                if !root_changed && normalized_paths.contains_key(&self.to_path(file)) {
                    task.lock().unwrap().reset_status();
                    needs_update.store(true, Ordering::SeqCst);
                    root_changed = true;
                }
            }

            if !root_changed {
                let build_info_entry = task.lock().unwrap().build_info_entry.clone();
                if let Some(entry) = build_info_entry {
                    if let Some(build_info) = entry.build_info.as_ref() {
                        let build_info_dir = tspath::get_directory_path(&entry.path.0);
                        for file_name in &build_info.file_names {
                            let fp = self.to_path(&self.resolve_build_info_file_name(
                                file_name,
                                &build_info_dir,
                            ));
                            if roots.has(&fp) {
                                continue;
                            }
                            if normalized_paths.contains_key(&fp) {
                                task.lock().unwrap().reset_status();
                                needs_update.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                        for package_json in build_info.get_package_jsons(&build_info_dir) {
                            if self.package_json_lookup_changed(&package_json, &normalized_paths) {
                                task.lock().unwrap().reset_status();
                                needs_update.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                        for package_json in build_info.get_missing_package_jsons(&build_info_dir) {
                            if self.package_json_lookup_changed(&package_json, &normalized_paths) {
                                task.lock().unwrap().reset_status();
                                needs_update.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                }
                let package_jsons = task.lock().unwrap().package_jsons.clone();
                for package_json in package_jsons {
                    if self.package_json_lookup_changed(&package_json, &normalized_paths) {
                        task.lock().unwrap().reset_status();
                        needs_update.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }

            {
                let mut task = task.lock().unwrap();
                task.report_done = std::sync::mpsc::channel().1;
                task.done = std::sync::mpsc::channel().1;
            }

            let new_config =
                resolved.reload_file_names_of_parsed_command_line(self.host.fs().as_ref());
            if resolved.file_names() != new_config.file_names() {
                self.host
                    .resolved_references
                    .store(path.clone(), Arc::new(new_config.clone()));
                {
                    let mut task = task.lock().unwrap();
                    task.resolved = Some(Arc::new(new_config));
                    task.reset_status();
                }
                needs_update.store(true, Ordering::SeqCst);
            }
        }

        if !needs_update.load(Ordering::SeqCst) {
            let opts = self.compare_paths_options.clone();
            for event_path in changed_paths.keys() {
                if self.host.fs().directory_exists(event_path)
                    && self.wm.is_path_under_watch(event_path, &opts)
                {
                    for (_, task) in self.tasks.to_hash_map() {
                        let mut task = task.lock().unwrap();
                        task.reset_status();
                        task.report_done = std::sync::mpsc::channel().1;
                        task.done = std::sync::mpsc::channel().1;
                    }
                    needs_update.store(true, Ordering::SeqCst);
                    break;
                }
            }
        }
    }

    pub fn package_json_lookup_changed(
        &self,
        package_json: &str,
        changed_paths: &HashMap<Path, tsox_core::fswatch::mig::m5f_2::EventKind>,
    ) -> bool { ::tsox_core::fntrace::enter("package_json_lookup_changed"); 
        let package_json_path = self.to_path(package_json);
        if changed_paths.contains_key(&package_json_path) {
            return true;
        }
        for (changed_path, kind) in changed_paths {
            if *kind == tsox_core::fswatch::mig::m5f_2::EventKind::Delete
                && changed_path.contains_path(&package_json_path)
            {
                return true;
            }
        }
        false
    }

    pub fn compute_desired_watches(&mut self) -> HashMap<String, bool> { ::tsox_core::fntrace::enter("compute_desired_watches"); 
        let mut desired_dirs = crate::mig::m5b_4::DirWatchSet::new(self.compare_paths_options.clone());

        for index in 0..self.order.len() {
            let config = self.order[index].clone();
            let path = self.to_path(&config);
            let task = self.get_task(&path);

            let task_config = task.lock().unwrap().config.clone();
            let config_dir = tspath::get_directory_path(&task_config);
            let real_config_dir = self.host.fs().realpath(&config_dir);
            desired_dirs.set(&real_config_dir, false);

            let resolved = task.lock().unwrap().resolved.clone();
            let Some(resolved) = resolved else {
                continue;
            };

            for cfg_path in resolved.extended_source_files() {
                let real_path = self.host.fs().realpath(cfg_path);
                let dir = tspath::get_directory_path(&real_path);
                desired_dirs.set(&dir, false);
            }

            for (dir, recursive) in resolved.wildcard_directories() {
                let real_dir = self.host.fs().realpath(&dir);
                desired_dirs.set(&real_dir, recursive);
            }

            for file_name in resolved.file_names() {
                let abs_path = tspath::get_normalized_absolute_path(
                    file_name,
                    &self.opts.sys.current_directory(),
                );
                let dir = tspath::get_directory_path(&abs_path);
                if !desired_dirs.covered(&dir) && crate::mig::m5b_4::can_watch_directory(&dir) {
                    desired_dirs.set(&dir, false);
                }
                for mapper in resolved.content_mappers() {
                    if mapper.package_directory.is_empty() || !mapper.contribution_id.is_empty() {
                        continue;
                    }
                    let manifest_path =
                        tspath::combine_paths(&mapper.package_directory, &["package.json"]);
                    let dir = tspath::get_directory_path(&manifest_path);
                    if !desired_dirs.covered(&dir) && crate::mig::m5b_4::can_watch_directory(&dir) {
                        desired_dirs.set(&dir, false);
                    }
                }
            }

            let content_mapper_project = task.lock().unwrap().content_mapper_project.clone();
            if let Some(project) = content_mapper_project.as_ref() {
                let watched_files = match project.watched_files() {
                    Ok(files) => files,
                    Err(err) => {
                        task.lock().unwrap().content_mapper_project_err =
                            Some(tsox_compile::mig::m4v_3::ContentMapperError(err.to_string()));
                        Vec::new()
                    }
                };
                for file_name in &watched_files {
                    let abs_path = self.host.fs().realpath(file_name);
                    let dir = tspath::get_directory_path(&abs_path);
                    if !desired_dirs.covered(&dir) && crate::mig::m5b_4::can_watch_directory(&dir) {
                        desired_dirs.set(&dir, false);
                    }
                }
            }

            let build_info_entry = task.lock().unwrap().build_info_entry.clone();
            if let Some(entry) = build_info_entry {
                if let Some(build_info) = entry.build_info.as_ref() {
                    let build_info_dir = tspath::get_directory_path(&entry.path.0);
                    let mut roots: Set<Path> = Set::new();
                    for file_name in resolved.file_names() {
                        roots.add(self.to_path(file_name));
                    }
                    for file_name in &build_info.file_names {
                        let abs_path = self.host.fs().realpath(&self.resolve_build_info_file_name(
                            file_name,
                            &build_info_dir,
                        ));
                        let fp = self.to_path(&abs_path);
                        if roots.has(&fp) {
                            continue;
                        }
                        let dir = tspath::get_directory_path(&abs_path);
                        if !desired_dirs.covered(&dir)
                            && crate::mig::m5b_4::can_watch_directory(&dir)
                        {
                            desired_dirs.set(&dir, false);
                        }
                    }
                    for package_json in build_info.get_package_jsons(&build_info_dir) {
                        self.add_package_json_watch_dirs(&mut desired_dirs, &package_json);
                    }
                    for package_json in build_info.get_missing_package_jsons(&build_info_dir) {
                        self.add_package_json_watch_dirs(&mut desired_dirs, &package_json);
                    }
                }
            }
            let package_jsons = task.lock().unwrap().package_jsons.clone();
            for package_json in package_jsons {
                self.add_package_json_watch_dirs(&mut desired_dirs, &package_json);
            }
        }

        self.wm.resolve_desired_dirs(&desired_dirs.dirs())
    }

    pub fn build_or_clean_project(
        &mut self,
        task: Arc<Mutex<BuildTask>>,
        path: &Path,
        build_result: &mut OrchestratorResultStruct,
    ) { ::tsox_core::fntrace::enter("build_or_clean_project"); 
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let status_reporter = self.create_builder_status_reporter(buffer.clone());
        let diagnostic_reporter = self.create_diagnostic_reporter(buffer.clone());
        {
            let mut task = task.lock().unwrap();
            task.result = Some(TaskResult::default());
            let result = task.result.as_mut().unwrap();
            result.report_status = Some(Box::new(move |diag: &Arc<Diagnostic>| {
                status_reporter(diag)
            }));
            result.diagnostic_reporter = Some(Box::new(move |diag: &Arc<Diagnostic>| {
                diagnostic_reporter(diag)
            }));
        }
        if self.opts.command.build_options.clean != Tristate::True {
            task.lock().unwrap().build_project(self, path.clone());
        } else {
            task.lock().unwrap().clean_project(self, path.clone());
        }
        {
            let mut task = task.lock().unwrap();
            if let Some(result) = task.result.as_mut() {
                result.builder =
                    String::from_utf8_lossy(&buffer.lock().unwrap()).into_owned();
            }
        }
        task.lock().unwrap().report(self, path.clone(), build_result);
    }

    fn create_builder_status_reporter(
        &self,
        buffer: Arc<Mutex<Vec<u8>>>,
    ) -> crate::mig::m5a_4::DiagnosticReporter { ::tsox_core::fntrace::enter("create_builder_status_reporter"); 
        crate::mig::m5a_4::create_builder_status_reporter(
            self.opts.sys.as_ref(),
            Box::new(SharedBufferWriter { buffer }),
            &self.opts.command.locale(),
            &self.opts.command.compiler_options,
            self.opts.testing.clone(),
        )
    }

    fn create_diagnostic_reporter(
        &self,
        buffer: Arc<Mutex<Vec<u8>>>,
    ) -> crate::mig::m5a_4::DiagnosticReporter { ::tsox_core::fntrace::enter("create_diagnostic_reporter"); 
        crate::mig::m5a_4::create_diagnostic_reporter(
            self.opts.sys.as_ref(),
            Box::new(SharedBufferWriter { buffer }),
            &self.opts.command.locale(),
            &self.opts.command.compiler_options,
        )
    }

    pub fn add_package_json_watch_dirs(
        &mut self,
        desired_dirs: &mut crate::mig::m5b_4::DirWatchSet,
        package_json: &str,
    ) { ::tsox_core::fntrace::enter("add_package_json_watch_dirs"); 
        let dir = tsox_core::tspath::get_directory_path(package_json);
        let mut dirs = vec![dir.clone()];
        let mut found_node_modules = false;
        let mut current = dir.clone();
        loop {
            let parent = tsox_core::tspath::get_directory_path(&current);
            if parent.is_empty() || parent == current {
                break;
            }
            dirs.push(parent.clone());
            if tsox_core::tspath::get_base_file_name(&parent) == "node_modules" {
                found_node_modules = true;
                let grandparent = tsox_core::tspath::get_directory_path(&parent);
                if !grandparent.is_empty() && grandparent != parent {
                    dirs.push(grandparent);
                }
                break;
            }
            current = parent;
        }

        if !found_node_modules {
            self.add_watch_dir(desired_dirs, &dir);
            return;
        }
        for dir in dirs {
            self.add_watch_dir(desired_dirs, &dir);
        }
    }

    fn add_watch_dir(&self, desired_dirs: &mut crate::mig::m5b_4::DirWatchSet, dir: &str) { ::tsox_core::fntrace::enter("add_watch_dir"); 
        if !desired_dirs.covered(dir) && crate::mig::m5b_4::can_watch_directory(dir) {
            desired_dirs.set(dir, false);
        }
    }
}

pub struct SharedBufferWriter {
    pub buffer: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for SharedBufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("write"); 
        self.buffer.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("flush"); 
        Ok(())
    }
}

pub struct SysWriterHandle(std::sync::Mutex<Box<dyn std::io::Write + Send>>);

impl std::io::Write for SysWriterHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("write"); 
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("flush"); 
        self.0.lock().unwrap().flush()
    }
}

impl OrchestratorResultStruct {
    pub fn report(&mut self, o: &mut Orchestrator) { ::tsox_core::fntrace::enter("report"); 
        if o.opts.command.compiler_options.watch == Tristate::True {
            let message = if self.errors.len() == 1 {
                dg::FOUND_1_ERROR_WATCHING_FOR_FILE_CHANGES
            } else {
                dg::FOUND_0_ERRORS_WATCHING_FOR_FILE_CHANGES
            };
            if let Some(watch_status_reporter) = o.watch_status_reporter.as_mut() {
                watch_status_reporter(&tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
                    message,
                    vec![self.errors.len().to_string()],
                ));
            }
        } else if let Some(error_summary_reporter) = o.error_summary_reporter.as_mut() {
            error_summary_reporter(&self.errors);
        }
        if !self.files_to_delete.is_empty() {
            let reporter = crate::mig::m5a_4::create_builder_status_reporter(
                o.opts.sys.as_ref(),
                Box::new(SysWriterHandle(std::sync::Mutex::new(o.opts.sys.writer()))),
                &o.opts.command.locale(),
                &o.opts.command.compiler_options,
                o.opts.testing.clone(),
            );
            reporter(&tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
                dg::A_NON_DRY_BUILD_WOULD_DELETE_THE_FOLLOWING_FILES_COLON_0,
                vec![self
                    .files_to_delete
                    .iter()
                    .map(|f| format!("\r\n * {}", f))
                    .collect::<Vec<_>>()
                    .join("")],
            ));
        }
        if o.opts.command.compiler_options.diagnostics != Tristate::True
            && o.opts.command.compiler_options.extended_diagnostics != Tristate::True
        {
            return;
        }
        let mut writer = o.opts.sys.writer();
        self.statistics
            .report(writer.as_mut(), o.opts.testing.as_deref());
    }
}

pub fn new_orchestrator(opts: Options) -> Orchestrator { ::tsox_core::fntrace::enter("new_orchestrator"); 
    let wm = crate::mig::m5b_4::WatchManager::new(
        opts.sys.writer(),
        Arc::new({
            let sys = opts.sys.clone();
            move |dir| sys.fs().directory_exists(dir)
        }),
    );
    let compare_paths_options = tsox_core::tspath::ComparePathsOptions {
        current_directory: opts.sys.current_directory().to_string(),
        use_case_sensitive_file_names: opts.sys.fs().use_case_sensitive_file_names(),
    };
    let watch = opts.command.compiler_options.watch == Tristate::True;
    let host_impl = tsox_compile::mig::m4v_3::new_cached_fs_compiler_host(
        opts.sys.current_directory().to_string(),
        opts.sys.fs(),
        opts.sys.default_library_path().to_string(),
        tsox_tsoptions::tsoptions::ExtendedConfigCache::new(),
        None,
        None,
    );
    let mut orchestrator = Orchestrator {
        opts,
        host: Arc::new(Host {
            orchestrator: Arc::new(Mutex::new(Orchestrator::placeholder())),
            host: Arc::new(host_impl),
            extended_config_cache: Default::default(),
            source_files: SyncMap::new(),
            config_times: SyncMap::new(),
            resolved_references: SyncMap::new(),
            m_times: SyncMap::new(),
        }),
        compare_paths_options,
        tasks: SyncMap::new(),
        order: Vec::new(),
        errors: Vec::new(),
        content_mapper_host: None,
        wm,
        watch_status_reporter: None,
        error_summary_reporter: None,
    };
    {
        let mut inner = orchestrator.host.orchestrator.lock().unwrap();
        inner.opts = orchestrator.opts.clone();
        inner.compare_paths_options = orchestrator.compare_paths_options.clone();
        inner.host = orchestrator.host.clone();
    }
    if watch {
        let reporter = crate::mig::m5a_4::create_watch_status_reporter(
            orchestrator.opts.sys.as_ref(),
            &orchestrator.opts.command.locale(),
            &orchestrator.opts.command.compiler_options,
            orchestrator.opts.testing.clone(),
        );
        orchestrator.watch_status_reporter =
            Some(Box::new(move |diagnostic: &Diagnostic| reporter(diagnostic)));
    } else {
        orchestrator.error_summary_reporter = Some(Box::new(
            crate::mig::m5a_4::create_report_error_summary(
                orchestrator.opts.sys.as_ref(),
                &orchestrator.opts.command.locale(),
                &orchestrator.opts.command.compiler_options,
            ),
        ));
    }
    orchestrator
}
