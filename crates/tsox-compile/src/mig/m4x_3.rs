#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::tspath::{self, Path};
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use super::m4x::new_project_reference_dts_faking_host;
use super::m4x_2::{FileLoader, ProjectReferenceFileMapper};
use tsox_core::core::work_group::{new_work_group, WorkGroup};

#[derive(Clone)]
pub struct ProjectReferenceParseTask {
    pub config_name: String,
    pub resolved: Option<Arc<ParsedCommandLine>>,
    pub sub_tasks: Vec<ProjectReferenceParseTask>,
}

impl ProjectReferenceParseTask {
    pub fn parse(&mut self, project_reference_parser: &ProjectReferenceParser) {
        let loader = match project_reference_parser.loader.as_ref() {
            None => return,
            Some(loader) => Arc::clone(loader),
        };
        let (host, path) = {
            let loader_guard = loader.lock().unwrap();
            (
                Arc::clone(&loader_guard.opts.host),
                loader_guard.to_path(&self.config_name),
            )
        };
        self.resolved = host
            .get_resolved_project_reference(&self.config_name, &path)
            .map(Arc::new);
        let mut resolved = match self.resolved.as_ref() {
            None => return,
            Some(resolved) => resolved.clone(),
        };
        let resolved_mut = Arc::get_mut(&mut resolved)
            .expect("task exclusively owns its freshly resolved project reference");
        resolved_mut.parse_input_output_names();
        let sub_references = resolved_mut.resolved_project_reference_paths();
        if !sub_references.is_empty() {
            self.sub_tasks = create_project_reference_parse_tasks(&sub_references);
        }
    }
}

pub fn create_project_reference_parse_tasks(
    project_references: &[String],
) -> Vec<ProjectReferenceParseTask> {
    project_references
        .iter()
        .map(|config_name| ProjectReferenceParseTask {
            config_name: config_name.clone(),
            resolved: None,
            sub_tasks: Vec::new(),
        })
        .collect()
}

pub struct ProjectReferenceParser {
    pub loader: Option<Arc<Mutex<FileLoader>>>,
    pub wg: Box<dyn WorkGroup>,
    pub tasks_by_file_name: SyncMap<Path, Arc<Mutex<ProjectReferenceParseTask>>>,
}

impl Default for ProjectReferenceParser {
    fn default() -> Self {
        Self {
            loader: None,
            wg: new_work_group(true),
            tasks_by_file_name: SyncMap::new(),
        }
    }
}

impl ProjectReferenceParser {
    pub fn parse(&mut self, tasks: Vec<ProjectReferenceParseTask>, loader: Arc<Mutex<FileLoader>>) {
        {
            let mut loader_guard = loader.lock().unwrap();
            Arc::get_mut(&mut loader_guard.project_reference_file_mapper)
                .expect("loader owns the project reference file mapper")
                .loader = Some(Arc::clone(&loader));
        }
        self.loader = Some(loader);
        let mut tasks: Vec<_> = tasks
            .into_iter()
            .map(|t| Arc::new(Mutex::new(t)))
            .collect();
        self.start(&mut tasks);
        self.loader_wait();
        self.init_mapper(&mut tasks);
    }

    fn loader_wait(&self) {
        self.wg.run_and_wait();
    }

    pub fn start(&mut self, tasks: &mut [Arc<Mutex<ProjectReferenceParseTask>>]) {
        let loader = match self.loader.as_ref() {
            None => return,
            Some(loader) => Arc::clone(loader),
        };
        let mut queued: Vec<Arc<Mutex<ProjectReferenceParseTask>>> = Vec::new();
        for i in 0..tasks.len() {
            let task = tasks[i].clone();
            let path = loader
                .lock()
                .unwrap()
                .to_path(&task.lock().unwrap().config_name);
            let loaded = self
                .tasks_by_file_name
                .load_or_store(path, task.clone());
            if loaded.1 {
                tasks[i] = loaded.0;
            } else {
                queued.push(task);
            }
        }
        for task in queued {
            let mut sub_tasks = {
                let mut task = task.lock().unwrap();
                task.parse(self);
                task.sub_tasks
                    .iter()
                    .map(|t| Arc::new(Mutex::new(t.clone())))
                    .collect::<Vec<_>>()
            };
            self.start(&mut sub_tasks);
        }
    }

    pub fn init_mapper(&mut self, tasks: &mut [Arc<Mutex<ProjectReferenceParseTask>>]) {
        let loader = match self.loader.as_ref() {
            None => return,
            Some(loader) => Arc::clone(loader),
        };
        let mut loader_guard = loader.lock().unwrap();
        let total_references = self.tasks_by_file_name_size() + 1;
        {
            let mapper = Arc::get_mut(&mut loader_guard.project_reference_file_mapper)
                .expect("loader owns the project reference file mapper");
            mapper.config_to_project_reference = HashMap::with_capacity(total_references);
            mapper.references_in_config_file = HashMap::with_capacity(total_references);
            mapper.source_to_project_reference = HashMap::new();
            mapper.output_dts_to_project_reference = HashMap::new();
        }
        let mut seen = Set::new();
        let root = loader_guard
            .project_reference_file_mapper
            .root_config_path();
        let references_in_config = {
            let loader: &mut FileLoader = &mut loader_guard;
            let FileLoader {
                project_reference_file_mapper,
                dts_directories,
                ..
            } = loader;
            let mapper = Arc::get_mut(project_reference_file_mapper)
                .expect("loader owns the project reference file mapper");
            self.init_mapper_worker(tasks, &mut seen, mapper, dts_directories)
        };
        Arc::get_mut(&mut loader_guard.project_reference_file_mapper)
            .expect("loader owns the project reference file mapper")
            .references_in_config_file
            .insert(root, references_in_config);
        if loader_guard
            .project_reference_file_mapper
            .opts
            .can_use_project_reference_source()
            && !loader_guard
                .project_reference_file_mapper
                .output_dts_to_project_reference
                .is_empty()
        {
            let host = new_project_reference_dts_faking_host(
                Arc::clone(&loader_guard.opts.host),
                Arc::downgrade(&loader_guard.project_reference_file_mapper),
                loader_guard.dts_directories.clone(),
            );
            Arc::get_mut(&mut loader_guard.project_reference_file_mapper)
                .expect("loader owns the project reference file mapper")
                .host = Some(Box::new(host));
        }
    }

    fn tasks_by_file_name_size(&self) -> usize {
        self.tasks_by_file_name.len()
    }

    pub fn init_mapper_worker(
        &self,
        tasks: &mut [Arc<Mutex<ProjectReferenceParseTask>>],
        seen: &mut Set<usize>,
        mapper: &mut ProjectReferenceFileMapper,
        dts_directories: &mut Set<Path>,
    ) -> Vec<Path> {
        if tasks.is_empty() {
            return Vec::new();
        }
        let mut results = Vec::with_capacity(tasks.len());
        for task in tasks.iter() {
            let path = tspath::to_path(
                &task.lock().unwrap().config_name,
                mapper.opts.host.current_directory(),
                mapper.opts.host.use_case_sensitive_file_names(),
            );
            results.push(path.clone());
            if !seen.add_if_absent(Arc::as_ptr(task) as usize) {
                continue;
            }
            let (resolved, sub_tasks) = {
                let task = task.lock().unwrap();
                (task.resolved.clone(), task.sub_tasks.clone())
            };
            mapper
                .config_to_project_reference
                .insert(path.clone(), resolved.clone());
            if let Some(resolved) = resolved.as_ref() {
                let same_config = match (&mapper.opts.config.config_file, &resolved.config_file) {
                    (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                    _ => false,
                };
                if !same_config {
                    for (k, v) in resolved.source_to_project_reference().iter() {
                        mapper
                            .source_to_project_reference
                            .insert(Path(k.clone()), Some(v.clone()));
                    }
                    for (k, v) in resolved.output_dts_to_project_reference().iter() {
                        mapper
                            .output_dts_to_project_reference
                            .insert(Path(k.clone()), Some(v.clone()));
                    }
                    if mapper.opts.can_use_project_reference_source() {
                        let mut decl_dir = resolved.compiler_options().declaration_dir.clone();
                        if decl_dir.is_empty() {
                            decl_dir = resolved.compiler_options().out_dir.clone();
                        }
                        if !decl_dir.is_empty() {
                            dts_directories.add(tspath::to_path(
                                &decl_dir,
                                mapper.opts.host.current_directory(),
                                mapper.opts.host.use_case_sensitive_file_names(),
                            ));
                        }
                    }
                }
            }
            let mut sub = sub_tasks
                .iter()
                .map(|t| Arc::new(Mutex::new(t.clone())))
                .collect::<Vec<_>>();
            let references_in_config =
                self.init_mapper_worker(&mut sub, seen, mapper, dts_directories);
            mapper
                .references_in_config_file
                .insert(path, references_in_config);
        }
        results
    }
}
