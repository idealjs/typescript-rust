//! Checker pool (1:1 port of Go's `internal/project/checkerpool.go`).

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::ast::SourceFile;
use crate::compiler::Program;
use std::sync::Arc;

const CHECKER_HELD_ANONYMOUS: &str = "<anonymous>";

/// Options for the checker pool.
///
/// Go: `type CheckerPoolOptions struct { ... }`.
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

/// Manages type checkers for a project: diagnostics, temporary query, and API.
///
/// Go: `type checkerPool struct { ... }`.
pub struct CheckerPool {
    opts: CheckerPoolOptions,
    program: Option<Arc<Program>>,
    mu: Mutex<CheckerPoolInner>,
}

struct CheckerPoolInner {
    discarded: bool,
    /// `held_by[i]` is the requestID holding checker `i`, or empty.
    held_by: Vec<String>,
    last_released: Vec<Option<Instant>>,
    global_diag_accumulated: Vec<usize>, // simplified: counts
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

    /// Signals that the pool's program has been replaced.
    pub fn discard(&self) {
        let mut inner = self.mu.lock().unwrap();
        if inner.discarded {
            return;
        }
        inner.discarded = true;
    }

    /// Returns accumulated global diagnostics count (simplified).
    pub fn get_global_diagnostics_count(&self) -> usize {
        self.mu.lock().unwrap().global_diag_accumulated.len()
    }

    /// Reports whether new global diagnostics have been accumulated.
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
