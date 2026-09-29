#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use tsox_checker::checker::Checker;
use tsox_compile::compiler::Program;
use tsox_frontend::ast::Diagnostic;

pub const CHECKER_HELD_ANONYMOUS: &str = "<anonymous>";

#[derive(Clone, Debug)]
pub struct CheckerPoolOptions {
    pub max_checkers: usize,
    pub idle_timeout: Duration,
}

impl Default for CheckerPoolOptions {
    fn default() -> Self {
        CheckerPoolOptions {
            max_checkers: 4,
            idle_timeout: Duration::from_secs(30),
        }
    }
}

pub struct Semaphore {
    permits: Mutex<usize>,
    cond: Condvar,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Semaphore {
            permits: Mutex::new(permits),
            cond: Condvar::new(),
        }
    }

    pub fn acquire(&self) {
        let mut n = self.permits.lock().unwrap();
        while *n == 0 {
            n = self.cond.wait(n).unwrap();
        }
        *n -= 1;
    }

    pub fn release(&self) {
        let mut n = self.permits.lock().unwrap();
        *n += 1;
        self.cond.notify_one();
    }
}

pub struct SendChecker(pub Arc<Checker>);
unsafe impl Send for SendChecker {}
unsafe impl Sync for SendChecker {}

pub struct CheckerPoolState {
    pub discarded: bool,
    pub checkers: Vec<Option<Arc<Checker>>>,
    pub held_by: Vec<String>,
    pub file_associations: HashMap<usize, usize>,
    pub request_associations: HashMap<String, usize>,
    pub last_released: Vec<Option<Instant>>,
    pub cleanup_timer: Option<CleanupTimer>,
    pub persistent_checker: Option<Arc<Checker>>,
    pub persistent_held: bool,
    pub log: Option<Box<dyn Fn(&str) + Send + Sync>>,
    pub global_diag_accumulated: Vec<Arc<Diagnostic>>,
    pub global_diag_changed: bool,
    pub global_diag_checker_count: Vec<usize>,
}

pub struct CleanupTimer {
    stop_flag: Arc<Mutex<bool>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl CleanupTimer {
    pub fn starting(pool: Arc<CheckerPool>, delay: Duration) -> Self {
        let mut timer = CleanupTimer {
            stop_flag: Arc::new(Mutex::new(false)),
            handle: None,
        };
        timer.reset(pool, delay);
        timer
    }

    pub fn stop(&mut self) {
        *self.stop_flag.lock().unwrap() = true;
        self.handle = None;
    }

    pub fn reset(&mut self, pool: Arc<CheckerPool>, delay: Duration) {
        self.stop();
        let stop_flag = Arc::new(Mutex::new(false));
        self.stop_flag = stop_flag.clone();
        self.handle = Some(std::thread::spawn(move || {
            let deadline = Instant::now() + delay;
            loop {
                if *stop_flag.lock().unwrap() {
                    return;
                }
                if Instant::now() >= deadline {
                    pool.cleanup_idle_checkers();
                    return;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }));
    }
}

pub struct CheckerPool {
    opts: CheckerPoolOptions,
    program: Option<Arc<Program>>,
    state: Mutex<CheckerPoolState>,
    diag_sem: Semaphore,
    query_sem: Semaphore,
    persistent_sem: Semaphore,
    log: Mutex<Option<Box<dyn Fn(&str) + Send + Sync>>>,
}

unsafe impl Send for CheckerPool {}
unsafe impl Sync for CheckerPool {}

impl CheckerPool {
    pub fn new(opts: CheckerPoolOptions, program: Option<Arc<Program>>) -> Self {
        let max = if opts.max_checkers <= 0 {
            4
        } else if opts.max_checkers < 2 {
            2
        } else {
            opts.max_checkers
        };
        let query_slots = max.saturating_sub(1);
        CheckerPool {
            opts: CheckerPoolOptions {
                max_checkers: max,
                ..opts
            },
            program,
            state: Mutex::new(CheckerPoolState {
                discarded: false,
                checkers: vec![None; max],
                held_by: vec![String::new(); max],
                file_associations: HashMap::new(),
                request_associations: HashMap::new(),
                last_released: vec![None; max],
                cleanup_timer: None,
                persistent_checker: None,
                persistent_held: false,
                log: None,
                global_diag_accumulated: Vec::new(),
                global_diag_changed: false,
                global_diag_checker_count: vec![0; max],
            }),
            diag_sem: Semaphore::new(1),
            query_sem: Semaphore::new(query_slots),
            persistent_sem: Semaphore::new(1),
            log: Mutex::new(None),
        }
    }

    pub fn init_state(&self, mut state: CheckerPoolState) {
        *self.log.lock().unwrap() = state.log.take();
        *self.state.lock().unwrap() = state;
    }

    pub fn lock_state(&self) -> MutexGuard<'_, CheckerPoolState> {
        self.state.lock().unwrap()
    }

    pub fn diag_sem(&self) -> &Semaphore {
        &self.diag_sem
    }

    pub fn query_sem(&self) -> &Semaphore {
        &self.query_sem
    }

    pub fn persistent_sem(&self) -> &Semaphore {
        &self.persistent_sem
    }

    pub fn log_msg(&self, msg: &str) {
        if let Some(log) = self.log.lock().unwrap().as_ref() {
            log(msg);
        }
    }

    pub fn program(&self) -> Option<Arc<Program>> {
        self.program.clone()
    }

    pub fn max_checkers(&self) -> usize {
        self.opts.max_checkers
    }

    pub fn idle_timeout(&self) -> Duration {
        self.opts.idle_timeout
    }

    pub fn discard(&self) {
        let mut state = self.lock_state();
        if state.discarded {
            return;
        }
        if let Some(timer) = state.cleanup_timer.as_mut() {
            timer.stop();
        }
        state.cleanup_timer = None;
        state.discarded = true;
    }

    pub fn get_global_diagnostics_count(&self) -> usize {
        self.lock_state().global_diag_accumulated.len()
    }

    pub fn take_new_global_diagnostics(&self) -> bool {
        let mut state = self.lock_state();
        let changed = state.global_diag_changed;
        state.global_diag_changed = false;
        changed
    }
}

fn hold_tag(request_id: &str) -> String {
    if request_id.is_empty() {
        CHECKER_HELD_ANONYMOUS.to_string()
    } else {
        request_id.to_string()
    }
}
