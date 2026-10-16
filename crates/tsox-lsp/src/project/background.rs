use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub struct Queue {
    closed: AtomicBool,
    threads: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl Queue {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Queue {
            closed: AtomicBool::new(false),
            threads: Mutex::new(Vec::new()),
        }
    }

    pub fn enqueue<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    { ::tsox_core::fntrace::enter("enqueue"); 
        if self.closed.load(Ordering::SeqCst) {
            return;
        }

        let handle = thread::spawn(f);
        self.threads.lock().unwrap().push(handle);
    }

    pub fn wait(&self) { ::tsox_core::fntrace::enter("wait"); 
        let mut threads = self.threads.lock().unwrap();
        for handle in threads.drain(..) {
            let _ = handle.join();
        }
    }

    pub fn close(&self) { ::tsox_core::fntrace::enter("close"); 
        self.closed.store(true, Ordering::SeqCst);
    }
}

impl Default for Queue {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}
