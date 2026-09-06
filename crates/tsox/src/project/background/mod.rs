use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub struct Queue {
    closed: AtomicBool,
    threads: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl Queue {

    pub fn new() -> Self {
        Queue {
            closed: AtomicBool::new(false),
            threads: Mutex::new(Vec::new()),
        }
    }

    pub fn enqueue<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }

        let handle = thread::spawn(f);
        self.threads.lock().unwrap().push(handle);
    }

    pub fn wait(&self) {
        let mut threads = self.threads.lock().unwrap();
        for handle in threads.drain(..) {
            let _ = handle.join();
        }
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}
