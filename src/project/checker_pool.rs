#![allow(dead_code)]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::compiler::Program;
use std::sync::Arc;

const CHECKER_HELD_ANONYMOUS: &str = "<anonymous>";

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

pub struct CheckerPool {
    opts: CheckerPoolOptions,
    program: Option<Arc<Program>>,
    mu: Mutex<CheckerPoolInner>,
}

struct CheckerPoolInner {
    discarded: bool,

    held_by: Vec<String>,
    last_released: Vec<Option<Instant>>,
    global_diag_accumulated: Vec<usize>,
    global_diag_changed: bool,
}

impl CheckerPool {
    pub fn new(opts: CheckerPoolOptions, program: Option<Arc<Program>>) -> Self {
        let max = if opts.max_checkers <= 0 {
            4
        } else if opts.max_checkers < 2 {
            2
        } else {
            opts.max_checkers
        };
        CheckerPool {
            opts: CheckerPoolOptions {
                max_checkers: max,
                ..opts
            },
            program,
            mu: Mutex::new(CheckerPoolInner {
                discarded: false,
                held_by: vec![String::new(); max],
                last_released: vec![None; max],
                global_diag_accumulated: Vec::new(),
                global_diag_changed: false,
            }),
        }
    }

    pub fn discard(&self) {
        let mut inner = self.mu.lock().unwrap();
        if inner.discarded {
            return;
        }
        inner.discarded = true;
    }

    pub fn get_global_diagnostics_count(&self) -> usize {
        self.mu.lock().unwrap().global_diag_accumulated.len()
    }

    pub fn take_new_global_diagnostics(&self) -> bool {
        let mut inner = self.mu.lock().unwrap();
        let changed = inner.global_diag_changed;
        inner.global_diag_changed = false;
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
