use std::collections::HashMap;
use std::sync::{Arc, Once};
use std::time::{Duration, Instant};

use tsox_checker::checker::Checker;
use tsox_checker::checker::mig::m1a;
use tsox_checker::checker::Tracer;
use tsox_compile::compiler::Program;
use tsox_compile::mig::m4w_4::sort_and_deduplicate_diagnostics;
use tsox_core::core::mig::m3j_3::{
    get_checker_lifetime, get_request_id, CheckerLifetime, RequestContext,
};
use tsox_frontend::ast::{Diagnostic, SourceFile};

pub use super::super::checker_pool::{CheckerPoolState, CleanupTimer, Semaphore};
use super::super::checker_pool::{
    CheckerPool, CheckerPoolOptions, SendChecker, CHECKER_HELD_ANONYMOUS,
};

pub type ReleaseFn = Box<dyn FnOnce() + Send>;

pub fn new_checker_pool(
    mut opts: CheckerPoolOptions,
    program: Option<Arc<Program>>,
    log: Option<Box<dyn Fn(&str) + Send + Sync>>,
) -> CheckerPool {
    if opts.max_checkers == 0 {
        opts.max_checkers = 4;
    } else if opts.max_checkers < 2 {
        opts.max_checkers = 2;
    }
    if opts.idle_timeout == Duration::ZERO {
        opts.idle_timeout = Duration::from_secs(30);
    }
    let pool = CheckerPool::new(opts, program);
    pool.init_state(CheckerPoolState {
        discarded: false,
        checkers: vec![None; max_checkers_of(&pool)],
        held_by: vec![String::new(); max_checkers_of(&pool)],
        file_associations: HashMap::new(),
        request_associations: HashMap::new(),
        last_released: vec![None; max_checkers_of(&pool)],
        cleanup_timer: None,
        persistent_checker: None,
        persistent_held: false,
        log,
        global_diag_accumulated: Vec::new(),
        global_diag_changed: false,
        global_diag_checker_count: vec![0; max_checkers_of(&pool)],
    });
    pool
}

pub fn noop() {}

pub fn hold_tag_or_anonymous(request_id: &str) -> String {
    if request_id.is_empty() {
        CHECKER_HELD_ANONYMOUS.to_string()
    } else {
        request_id.to_string()
    }
}

fn max_checkers_of(pool: &CheckerPool) -> usize {
    pool.max_checkers()
}

fn request_context_can_be_canceled(ctx: &RequestContext) -> bool {
    !get_request_id(ctx).is_empty()
}

fn new_checker_for(program: Option<Arc<Program>>) -> Arc<Checker> {
    let program = program.expect("checker pool requires a program to create checkers");
    let program: Arc<dyn tsox_checker::checker::Program> = program;
    let (checker, _guard) = m1a::new_checker(program, Arc::new(Tracer::new()));
    Arc::from(checker)
}

impl CheckerPool {
    pub fn get_checker(
        self: &Arc<Self>,
        ctx: &RequestContext,
        file: Option<Arc<SourceFile>>,
    ) -> (Arc<Checker>, ReleaseFn) {
        let lifetime = get_checker_lifetime(ctx);
        let mut request_id = get_request_id(ctx);
        if !request_context_can_be_canceled(ctx) {
            request_id = String::new();
        }
        match lifetime {
            CheckerLifetime::Diagnostics => self.get_diagnostics_checker(ctx, &request_id),
            CheckerLifetime::Api => self.get_persistent_checker(),
            _ => self.get_query_checker(ctx, &request_id, file),
        }
    }

    fn try_reacquire_for_request(
        self: &Arc<Self>,
        request_id: &str,
        sem: &Semaphore,
        is_diag: bool,
    ) -> Option<(Arc<Checker>, ReleaseFn)> {
        if request_id.is_empty() {
            sem.acquire();
            return None;
        }
        let mut state = self.lock_state();
        let Some(&index) = state.request_associations.get(request_id) else {
            drop(state);
            sem.acquire();
            return None;
        };
        if (is_diag && index != 0) || (!is_diag && index == 0) {
            state.request_associations.remove(request_id);
            drop(state);
            sem.acquire();
            return None;
        }
        let Some(c) = state.checkers[index].clone() else {
            state.request_associations.remove(request_id);
            drop(state);
            sem.acquire();
            return None;
        };
        let held = state.held_by[index].clone();
        if held == request_id {
            drop(state);
            return Some((c, boxed_noop()));
        }
        if held.is_empty() {
            drop(state);
            sem.acquire();
            let mut state = self.lock_state();
            let same_checker = state.checkers[index]
                .as_ref()
                .map(|cc| Arc::ptr_eq(cc, &c))
                .unwrap_or(false);
            if same_checker && state.held_by[index].is_empty() {
                state.held_by[index] = request_id.to_string();
                drop(state);
                let release = self.create_release(request_id.to_string(), index, c.clone());
                return Some((c, release));
            }
            drop(state);
            return None;
        }
        drop(state);
        sem.acquire();
        None
    }

    fn get_diagnostics_checker(
        self: &Arc<Self>,
        ctx: &RequestContext,
        request_id: &str,
    ) -> (Arc<Checker>, ReleaseFn) {
        const DIAG_INDEX: usize = 0;
        if let Some(result) = self.try_reacquire_for_request(request_id, &self.diag_sem(), true) {
            return result;
        }
        let mut state = self.lock_state();
        if state.checkers[DIAG_INDEX].is_none() {
            self.log_msg("checkerpool: Creating diagnostics checker");
            let c = new_checker_for(self.program());
            state.checkers[DIAG_INDEX] = Some(c);
        }
        let c = state.checkers[DIAG_INDEX].clone().unwrap();
        state.held_by[DIAG_INDEX] = hold_tag_or_anonymous(request_id);
        self.log_msg(&format!(
            "checkerpool: Acquired diagnostics checker for request {}",
            hold_tag_or_anonymous(request_id)
        ));
        if !request_id.is_empty() && !state.request_associations.contains_key(request_id) {
            state
                .request_associations
                .insert(request_id.to_string(), DIAG_INDEX);
            self.register_request_cleanup(ctx, request_id);
        }
        let release = self.create_release(request_id.to_string(), DIAG_INDEX, c.clone());
        drop(state);
        (c, release)
    }

    fn get_query_checker(
        self: &Arc<Self>,
        ctx: &RequestContext,
        request_id: &str,
        file: Option<Arc<SourceFile>>,
    ) -> (Arc<Checker>, ReleaseFn) {
        if let Some(result) = self.try_reacquire_for_request(request_id, &self.query_sem(), false) {
            return result;
        }
        let mut state = self.lock_state();
        if let Some(file) = &file {
            let file_key = Arc::as_ptr(file) as usize;
            if let Some(&index) = state.file_associations.get(&file_key) {
                if index > 0 {
                    if let Some(c) = state.checkers[index].clone() {
                        if state.held_by[index].is_empty() {
                            state.held_by[index] = hold_tag_or_anonymous(request_id);
                            if !request_id.is_empty()
                                && !state.request_associations.contains_key(request_id)
                            {
                                state
                                    .request_associations
                                    .insert(request_id.to_string(), index);
                                self.register_request_cleanup(ctx, request_id);
                            }
                            let release =
                                self.create_release(request_id.to_string(), index, c.clone());
                            drop(state);
                            return (c, release);
                        }
                    }
                }
            }
        }
        let (c, index) =
            find_or_create_query_checker_locked(&mut state, self.program(), &|m| self.log_msg(m));
        state.held_by[index] = hold_tag_or_anonymous(request_id);
        self.log_msg(&format!(
            "checkerpool: Acquired query checker {} for request {}",
            index,
            hold_tag_or_anonymous(request_id)
        ));
        if !request_id.is_empty() && !state.request_associations.contains_key(request_id) {
            state.request_associations.insert(request_id.to_string(), index);
            self.register_request_cleanup(ctx, request_id);
        }
        if let Some(file) = &file {
            state
                .file_associations
                .insert(Arc::as_ptr(file) as usize, index);
        }
        let release = self.create_release(request_id.to_string(), index, c.clone());
        drop(state);
        (c, release)
    }

    fn get_persistent_checker(self: &Arc<Self>) -> (Arc<Checker>, ReleaseFn) {
        self.persistent_sem().acquire();
        let mut state = self.lock_state();
        if state.persistent_checker.is_none() {
            self.log_msg("checkerpool: Creating persistent checker");
            let c = new_checker_for(self.program());
            state.persistent_checker = Some(c);
        }
        let c = state.persistent_checker.clone().unwrap();
        state.persistent_held = true;
        drop(state);
        let pool = self.clone();
        let c_for_release = SendChecker(c.clone());
        let once = Arc::new(Once::new());
        let release: ReleaseFn = Box::new(move || {
            let once = once.clone();
            once.call_once(move || {
                let mut state = pool.lock_state();
                state.persistent_held = false;
                if c_for_release.0.was_canceled() {
                    pool.log_msg("checkerpool: Persistent checker was canceled, disposing");
                    let same = state
                        .persistent_checker
                        .as_ref()
                        .map(|pc| Arc::ptr_eq(pc, &c_for_release.0))
                        .unwrap_or(false);
                    if same {
                        state.persistent_checker = None;
                    }
                }
                drop(state);
                pool.persistent_sem().release();
            });
        });
        (c, release)
    }

    fn create_release(
        self: &Arc<Self>,
        request_id: String,
        index: usize,
        c: Arc<Checker>,
    ) -> ReleaseFn {
        let pool = self.clone();
        let once = Arc::new(Once::new());
        let c = SendChecker(c);
        Box::new(move || {
            let once = once.clone();
            once.call_once(move || {
                let mut state = pool.lock_state();
                if c.0.was_canceled() {
                    pool.log_msg(&format!(
                        "checkerpool: Checker {} for request {} was canceled, disposing",
                        index,
                        hold_tag_or_anonymous(&request_id)
                    ));
                    dispose_checker_locked(&mut state, index, &c.0);
                } else {
                    merge_global_diagnostics_from_checker_locked(&mut state, index, &c.0);
                    state.held_by[index] = String::new();
                    state.last_released[index] = Some(Instant::now());
                    if !state.discarded {
                        schedule_cleanup_locked(&mut state, &pool);
                    }
                }
                drop(state);
                if index == 0 {
                    pool.diag_sem().release();
                } else {
                    pool.query_sem().release();
                }
            });
        })
    }

    fn register_request_cleanup(self: &Arc<Self>, ctx: &RequestContext, request_id: &str) {
        let pool = Arc::clone(self);
        let request_id = request_id.to_string();
        ctx.after_func(Box::new(move || {
            let mut state = pool.lock_state();
            state.request_associations.remove(&request_id);
        }));
    }

    pub fn get_global_diagnostics(&self) -> Vec<Arc<Diagnostic>> {
        let state = self.lock_state();
        state.global_diag_accumulated.clone()
    }

    pub fn cleanup_idle_checkers(self: &Arc<Self>) {
        let mut state = self.lock_state();
        if state.discarded {
            return;
        }
        let now = Instant::now();
        for i in 0..state.checkers.len() {
            let Some(c) = state.checkers[i].clone() else {
                continue;
            };
            if !state.held_by[i].is_empty() {
                continue;
            }
            let Some(last_released) = state.last_released[i] else {
                continue;
            };
            let idle = now.saturating_duration_since(last_released);
            if idle >= self.idle_timeout() {
                self.log_msg(&format!(
                    "checkerpool: Disposing idle checker {} (idle {:?})",
                    i, idle
                ));
                dispose_checker_locked(&mut state, i, &c);
            }
        }
        schedule_cleanup_locked(&mut state, self);
    }
}

fn boxed_noop() -> ReleaseFn {
    Box::new(noop)
}

fn find_or_create_query_checker_locked(
    state: &mut CheckerPoolState,
    program: Option<Arc<Program>>,
    log: &dyn Fn(&str),
) -> (Arc<Checker>, usize) {
    for i in 1..state.checkers.len() {
        if let Some(c) = &state.checkers[i] {
            if state.held_by[i].is_empty() {
                return (c.clone(), i);
            }
        }
    }
    for i in 1..state.checkers.len() {
        if state.checkers[i].is_none() {
            log(&format!("checkerpool: Creating query checker {}", i));
            let c = new_checker_for(program);
            state.checkers[i] = Some(c.clone());
            return (c, i);
        }
    }
    panic!("checkerpool: no available query slot despite holding semaphore token");
}

pub fn schedule_cleanup_locked(state: &mut CheckerPoolState, pool: &Arc<CheckerPool>) {
    let mut earliest_deadline: Option<Instant> = None;
    for i in 0..state.checkers.len() {
        if state.checkers[i].is_none() || !state.held_by[i].is_empty() {
            continue;
        }
        let Some(last_released) = state.last_released[i] else {
            continue;
        };
        let deadline = last_released + pool.idle_timeout();
        match earliest_deadline {
            None => earliest_deadline = Some(deadline),
            Some(current) if deadline < current => earliest_deadline = Some(deadline),
            _ => {}
        }
    }
    let Some(earliest_deadline) = earliest_deadline else {
        if let Some(timer) = state.cleanup_timer.as_mut() {
            timer.stop();
        }
        state.cleanup_timer = None;
        return;
    };
    let now = Instant::now();
    let delay = if earliest_deadline <= now {
        Duration::from_millis(1)
    } else {
        earliest_deadline - now
    };
    if let Some(timer) = state.cleanup_timer.as_mut() {
        timer.reset(pool.clone(), delay);
    } else {
        state.cleanup_timer = Some(CleanupTimer::starting(pool.clone(), delay));
    }
}

pub fn dispose_checker_locked(state: &mut CheckerPoolState, index: usize, c: &Arc<Checker>) {
    let same = state.checkers[index]
        .as_ref()
        .map(|existing| Arc::ptr_eq(existing, c))
        .unwrap_or(false);
    debug_assert!(same);
    state.checkers[index] = None;
    state.held_by[index] = String::new();
    state.global_diag_checker_count[index] = 0;
    state.last_released[index] = None;
    state.file_associations.retain(|_, idx| *idx != index);
    state.request_associations.retain(|_, idx| *idx != index);
}

pub fn merge_global_diagnostics_from_checker_locked(
    state: &mut CheckerPoolState,
    index: usize,
    c: &Checker,
) {
    let globals = c.diagnostics.get_global_diagnostics();
    if globals.len() == state.global_diag_checker_count[index] {
        return;
    }
    state.global_diag_checker_count[index] = globals.len();
    let before = state.global_diag_accumulated.len();
    state
        .global_diag_accumulated
        .extend(globals.into_iter().map(Arc::new));
    state.global_diag_accumulated = sort_and_deduplicate_diagnostics(std::mem::take(
        &mut state.global_diag_accumulated,
    ));
    if state.global_diag_accumulated.len() != before {
        state.global_diag_changed = true;
    }
}
