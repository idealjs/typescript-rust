//! Program reference counter (1:1 port of Go's `internal/project/programcounter.go`).

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use crate::compiler::Program;
use std::sync::Arc;

/// Tracks how many snapshots reference each program. When a program is no
/// longer referenced by any snapshot, its source files are released from the
/// parse cache.
pub struct ProgramCounter {
    refs: Mutex<HashMap<usize, i32>>,
}

impl ProgramCounter {
    pub fn new() -> Self {
        ProgramCounter {
            refs: Mutex::new(HashMap::new()),
        }
    }

    /// Increments the reference count for a program. If the program is not yet
    /// tracked, it is added with a reference count of 1.
    pub fn r#ref(&self, program: &Arc<Program>) {
        let key = Arc::as_ptr(program) as usize;
        let mut refs = self.refs.lock().unwrap();
        *refs.entry(key).or_insert(0) += 1;
    }

    /// Decrements the reference count for a program. Returns `true` when the
    /// count reaches zero (the program is no longer referenced by any snapshot).
    pub fn deref(&self, program: &Arc<Program>) -> bool {
        let key = Arc::as_ptr(program) as usize;
        let mut refs = self.refs.lock().unwrap();
        match refs.get_mut(&key) {
            None => false,
            Some(count) => {
                *count -= 1;
                if *count < 0 {
                    panic!("program reference count went below zero");
                }
                if *count == 0 {
                    refs.remove(&key);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.refs.lock().unwrap().len()
    }
}

impl Default for ProgramCounter {
    fn default() -> Self {
        Self::new()
    }
}
