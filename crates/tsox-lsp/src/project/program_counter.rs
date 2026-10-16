#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use std::sync::Arc;
use tsox_compile::compiler::Program;

pub struct ProgramCounter {
    refs: Mutex<HashMap<usize, i32>>,
}

impl ProgramCounter {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        ProgramCounter {
            refs: Mutex::new(HashMap::new()),
        }
    }

    pub fn r#ref(&self, program: &Arc<Program>) {
        let key = Arc::as_ptr(program) as usize;
        let mut refs = self.refs.lock().unwrap();
        *refs.entry(key).or_insert(0) += 1;
    }

    pub fn deref(&self, program: &Arc<Program>) -> bool { ::tsox_core::fntrace::enter("deref"); 
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

    pub fn len(&self) -> usize { ::tsox_core::fntrace::enter("len"); 
        self.refs.lock().unwrap().len()
    }
}

impl Default for ProgramCounter {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}
