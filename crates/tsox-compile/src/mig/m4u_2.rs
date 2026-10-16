use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use tsox_checker::checker::Checker;
use tsox_core::core::work_group::new_work_group;
use tsox_frontend::ast::{Diagnostic, SourceFile};

use crate::compiler::Program;
use super::m4u::{
    get_checker_association_base_weight, get_checker_association_order,
    get_checker_association_policy, get_checker_association_weights,
    get_checker_associations_in_order, CheckerAssociationPolicy,
};

struct SendCheckerSlot(Arc<Mutex<Option<Box<Checker>>>>);
unsafe impl Send for SendCheckerSlot {}

impl Clone for SendCheckerSlot {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        Self(Arc::clone(&self.0))
    }
}

impl SendCheckerSlot {
    fn lock(&self) -> MutexGuard<'_, Option<Box<Checker>>> { ::tsox_core::fntrace::enter("lock"); 
        self.0.lock().unwrap()
    }

    fn take_checker(&self) -> Box<Checker> { ::tsox_core::fntrace::enter("take_checker"); 
        self.lock().take().expect("checker construction failed")
    }
}

struct SendChecker(Arc<Mutex<Checker>>);
unsafe impl Send for SendChecker {}
unsafe impl Sync for SendChecker {}

impl Clone for SendChecker {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        Self(Arc::clone(&self.0))
    }
}

impl SendChecker {
    fn new(checker: Box<Checker>) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self(Arc::new(Mutex::new(*checker)))
    }

    fn lock(&self) -> MutexGuard<'_, Checker> { ::tsox_core::fntrace::enter("lock"); 
        self.0.lock().unwrap()
    }

    fn clone_inner(&self) -> Arc<Mutex<Checker>> { ::tsox_core::fntrace::enter("clone_inner"); 
        Arc::clone(&self.0)
    }
}

pub trait CheckerPool: Send + Sync {
    fn get_checker(&self, file: Option<&Arc<SourceFile>>) -> MutexGuard<'_, Checker>;
    fn get_checker_non_exclusive(&self) -> Arc<Mutex<Checker>>;
    fn for_each_checker_parallel(&self, cb: &mut (dyn FnMut(usize, &mut Checker) + Send));
    fn for_each_checker_group_do(
        &self,
        files: &[Arc<SourceFile>],
        single_threaded: bool,
        cb: &mut (dyn FnMut(&mut Checker, usize, &Arc<SourceFile>) + Send),
    );
    fn get_global_diagnostics(&self) -> Vec<Arc<Diagnostic>>;
}

struct CheckerPoolState {
    checkers: Vec<SendChecker>,
    file_associations: HashMap<String, usize>,
}

pub struct CheckerPoolImpl {
    program: std::sync::Weak<Program>,
    tracing: Option<Arc<tsox_core::tracing::mig::x11a::Tracing<'static>>>,
    checker_count: usize,
    state: OnceLock<CheckerPoolState>,
}

unsafe impl Send for CheckerPoolImpl {}
unsafe impl Sync for CheckerPoolImpl {}

pub fn new_checker_pool(program: Arc<Program>) -> CheckerPoolImpl { ::tsox_core::fntrace::enter("new_checker_pool"); 
    new_checker_pool_with_tracing(program, None)
}

pub fn new_checker_pool_with_tracing(
    program: Arc<Program>,
    tracing: Option<Arc<tsox_core::tracing::mig::x11a::Tracing<'static>>>,
) -> CheckerPoolImpl { ::tsox_core::fntrace::enter("new_checker_pool_with_tracing"); 
    let mut checker_count = 4usize;
    if program.single_threaded() {
        checker_count = 1;
    } else if let Some(c) = program.options.checkers {
        checker_count = c.max(0) as usize;
    }

    checker_count = checker_count
        .min(program.source_files.len())
        .min(256)
        .max(1);

    CheckerPoolImpl {
        program: std::sync::Arc::downgrade(&program),
        tracing,
        checker_count,
        state: OnceLock::new(),
    }
}

fn file_node_count(file: &SourceFile) -> usize { ::tsox_core::fntrace::enter("file_node_count"); 
    use tsox_frontend::ast::node_data_generated::for_each_child;
    let mut count = 0usize;
    let mut stack = vec![Arc::clone(&file.node)];
    while let Some(node) = stack.pop() {
        count += 1;
        for_each_child(&node, |child| {
            stack.push(Arc::clone(child));
            false
        });
    }
    count
}

impl CheckerPoolImpl {
    fn association_of(&self, state: &CheckerPoolState, file: &Arc<SourceFile>) -> usize { ::tsox_core::fntrace::enter("association_of"); 
        state
            .file_associations
            .get(&file.file_name)
            .copied()
            .expect("checker association missing for file")
    }

    pub fn get_checker(&self, file: Option<&Arc<SourceFile>>) -> MutexGuard<'_, Checker> { ::tsox_core::fntrace::enter("get_checker"); 
        if let Some(file) = file {
            return self.get_checker_for_file_exclusive(file);
        }
        let state = self.state.get_or_init(|| self.build_checkers());
        state.checkers[0].lock()
    }

    pub fn get_checker_for_file_non_exclusive(
        &self,
        file: &Arc<SourceFile>,
    ) -> Arc<Mutex<Checker>> { ::tsox_core::fntrace::enter("get_checker_for_file_non_exclusive"); 
        let state = self.state.get_or_init(|| self.build_checkers());
        let idx = self.association_of(state, file);
        state.checkers[idx].clone_inner()
    }

    fn get_checker_for_file_exclusive(&self, file: &Arc<SourceFile>) -> MutexGuard<'_, Checker> { ::tsox_core::fntrace::enter("get_checker_for_file_exclusive"); 
        let state = self.state.get_or_init(|| self.build_checkers());
        let idx = self.association_of(state, file);
        state.checkers[idx].lock()
    }

    pub fn get_checker_non_exclusive(&self) -> Arc<Mutex<Checker>> { ::tsox_core::fntrace::enter("get_checker_non_exclusive"); 
        let state = self.state.get_or_init(|| self.build_checkers());
        state.checkers[0].clone_inner()
    }

    fn build_checkers(&self) -> CheckerPoolState { ::tsox_core::fntrace::enter("build_checkers"); 
        let program = self.program.upgrade().expect("checker pool outlived program");
        let single_threaded = program.single_threaded();
        let checker_count = self.checker_count;

        let mut staged: Vec<SendCheckerSlot> = Vec::with_capacity(checker_count);
        let wg = new_work_group(single_threaded);
        for _i in 0..checker_count {
            let program = Arc::clone(&program);
            let tracer = Arc::new(tsox_checker::checker::Tracer::new());
            let slot = SendCheckerSlot(Arc::new(Mutex::new(None)));
            staged.push(slot.clone());
            wg.queue(Box::new(move || {
                let program: Arc<dyn tsox_checker::checker::Program> = program as _;
                let (checker, _lock) =
                    tsox_checker::checker::mig::m1a::new_checker(program, tracer);
                *slot.lock() = Some(checker);
            }));
        }
        wg.run_and_wait();
        let checkers: Vec<SendChecker> = staged
            .into_iter()
            .map(|slot| SendChecker::new(slot.take_checker()))
            .collect();

        let mut associations = vec![0usize; program.source_files.len()];
        if checker_count > 1 {
            let files = &program.source_files;
            let mut base_weights = vec![0usize; files.len()];
            let mut import_counts = vec![0usize; files.len()];
            let mut is_declaration_file = vec![false; files.len()];
            let mut total_base_weight = 0usize;
            let mut declaration_base_weight = 0usize;
            for (i, file) in files.iter().enumerate() {
                let base_weight =
                    get_checker_association_base_weight(file_node_count(file), file.text.len());
                total_base_weight += base_weight;
                if file.is_declaration_file {
                    declaration_base_weight += base_weight;
                }
                base_weights[i] = base_weight;
                import_counts[i] = file.imports.len();
                is_declaration_file[i] = file.is_declaration_file;
            }
            let policy = get_checker_association_policy(
                total_base_weight,
                declaration_base_weight,
                checker_count,
            );
            if policy.source_file_weight_multiplier != 1 {
                for (i, declaration) in is_declaration_file.iter().enumerate() {
                    if !declaration {
                        base_weights[i] *= policy.source_file_weight_multiplier;
                    }
                }
            }
            let file_weights = get_checker_association_weights(&base_weights, &import_counts);
            let adjacent_files = self.get_import_adjacency();
            let file_order = get_checker_association_order(
                &file_weights,
                &is_declaration_file,
                policy.prioritize_source_files,
            );
            associations = get_checker_associations_in_order(
                &file_weights,
                &adjacent_files,
                file_order.as_deref(),
                checker_count,
                policy.balance_penalty_multiplier,
            );
        }
        let mut file_associations = HashMap::with_capacity(program.source_files.len());
        for (i, file) in program.source_files.iter().enumerate() {
            file_associations.insert(file.file_name.clone(), associations[i]);
        }
        CheckerPoolState {
            checkers,
            file_associations,
        }
    }

    fn get_import_adjacency(&self) -> Vec<Vec<usize>> { ::tsox_core::fntrace::enter("get_import_adjacency"); 
        let program = self
            .program
            .upgrade()
            .expect("checker pool outlived program");
        let files = &program.source_files;
        let mut file_indices: HashMap<&str, usize> = HashMap::with_capacity(files.len());
        for (i, file) in files.iter().enumerate() {
            file_indices.insert(&file.file_name, i);
        }
        let mut adjacent_files: Vec<Vec<usize>> = vec![Vec::new(); files.len()];
        let resolved_modules = program.get_resolved_modules();
        for (file_index, file) in files.iter().enumerate() {
            let Some(entries) = resolved_modules.get(&file.file_name) else {
                continue;
            };
            for (_name, resolved) in entries {
                let Some(resolved) = resolved.as_ref() else { continue };
                if !resolved.is_resolved() {
                    continue;
                }
                let Some(imported_file) =
                    program.get_source_file_by_path(&resolved.resolved_file_name)
                else {
                    continue;
                };
                let Some(&imported_index) = file_indices.get(imported_file.file_name.as_str())
                else {
                    continue;
                };
                if imported_index == file_index {
                    continue;
                }
                adjacent_files[file_index].push(imported_index);
                adjacent_files[imported_index].push(file_index);
            }
        }
        adjacent_files
    }

    pub fn for_each_checker_parallel(&self, cb: &mut (dyn FnMut(usize, &mut Checker) + Send)) { ::tsox_core::fntrace::enter("for_each_checker_parallel"); 
        let state = self.state.get_or_init(|| self.build_checkers());
        let checkers = &state.checkers;
        let program = self
            .program
            .upgrade()
            .expect("checker pool outlived program");
        if program.single_threaded() {
            for (idx, checker) in checkers.iter().enumerate() {
                let mut guard = checker.lock();
                (*cb)(idx, &mut guard);
            }
            return;
        }
        let cb = Mutex::new(&mut *cb);
        std::thread::scope(|scope| {
            for (idx, checker) in checkers.iter().enumerate() {
                scope.spawn({
                    let cb = &cb;
                    move || {
                        let mut guard = checker.lock();
                        (*cb.lock().unwrap())(idx, &mut guard);
                    }
                });
            }
        });
    }

    pub fn get_global_diagnostics(&self) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_global_diagnostics"); 
        let checker_count = self.checker_count;
        let global_diagnostics: Mutex<Vec<Vec<Arc<Diagnostic>>>> =
            Mutex::new((0..checker_count).map(|_| Vec::new()).collect());
        let mut collect = |idx: usize, checker: &mut Checker| {
            global_diagnostics.lock().unwrap()[idx] = checker
                .get_global_diagnostics()
                .into_iter()
                .map(Arc::new)
                .collect();
        };
        self.for_each_checker_parallel(&mut collect);
        let per_checker = global_diagnostics.into_inner().unwrap();
        let all: Vec<Arc<Diagnostic>> = per_checker.into_iter().flatten().collect();
        super::m4w_4::sort_and_deduplicate_diagnostics(all)
    }

    pub fn for_each_checker_group_do(
        &self,
        files: &[Arc<SourceFile>],
        single_threaded: bool,
        cb: &mut (dyn FnMut(&mut Checker, usize, &Arc<SourceFile>) + Send),
    ) { ::tsox_core::fntrace::enter("for_each_checker_group_do"); 
        let state = self.state.get_or_init(|| self.build_checkers());
        let checkers = &state.checkers;
        let associations = &state.file_associations;
        let group_task = |checker_idx: usize, cb: &mut (dyn FnMut(&mut Checker, usize, &Arc<SourceFile>) + Send)| {
            let mut guard = checkers[checker_idx].lock();
            for (i, file) in files.iter().enumerate() {
                if associations.get(&file.file_name) == Some(&checker_idx) {
                    (*cb)(&mut guard, i, file);
                }
            }
        };
        if single_threaded {
            for checker_idx in 0..checkers.len() {
                group_task(checker_idx, cb);
            }
            return;
        }
        let cb = Mutex::new(&mut *cb);
        std::thread::scope(|scope| {
            for checker_idx in 0..checkers.len() {
                scope.spawn({
                    let cb = &cb;
                    move || {
                        let mut guard = checkers[checker_idx].lock();
                        for (i, file) in files.iter().enumerate() {
                            if associations.get(&file.file_name) == Some(&checker_idx) {
                                (*cb.lock().unwrap())(&mut guard, i, file);
                            }
                        }
                    }
                });
            }
        });
    }
}

impl CheckerPool for CheckerPoolImpl {
    fn get_checker(&self, file: Option<&Arc<SourceFile>>) -> MutexGuard<'_, Checker> { ::tsox_core::fntrace::enter("get_checker"); 
        CheckerPoolImpl::get_checker(self, file)
    }

    fn get_checker_non_exclusive(&self) -> Arc<Mutex<Checker>> { ::tsox_core::fntrace::enter("get_checker_non_exclusive"); 
        CheckerPoolImpl::get_checker_non_exclusive(self)
    }

    fn for_each_checker_parallel(&self, cb: &mut (dyn FnMut(usize, &mut Checker) + Send)) { ::tsox_core::fntrace::enter("for_each_checker_parallel"); 
        CheckerPoolImpl::for_each_checker_parallel(self, cb)
    }

    fn for_each_checker_group_do(
        &self,
        files: &[Arc<SourceFile>],
        single_threaded: bool,
        cb: &mut (dyn FnMut(&mut Checker, usize, &Arc<SourceFile>) + Send),
    ) { ::tsox_core::fntrace::enter("for_each_checker_group_do"); 
        CheckerPoolImpl::for_each_checker_group_do(self, files, single_threaded, cb)
    }

    fn get_global_diagnostics(&self) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_global_diagnostics"); 
        CheckerPoolImpl::get_global_diagnostics(self)
    }
}

pub fn noop() { ::tsox_core::fntrace::enter("noop"); }
