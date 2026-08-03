//! Background task queue (1:1 port of Go's `internal/project/background/queue.go`).

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

/// Queue manages background task execution.
pub struct Queue {
    closed: AtomicBool,
    threads: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl Queue {
    /// Creates a new background queue.
    pub fn new() -> Self {
        Queue {
            closed: AtomicBool::new(false),
            threads: Mutex::new(Vec::new()),
        }
    }

    /// Enqueue a background task. Does nothing if queue is closed.
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

    /// Wait for all active tasks to complete.
    pub fn wait(&self) {
        let mut threads = self.threads.lock().unwrap();
        for handle in threads.drain(..) {
            let _ = handle.join();
        }
    }

    /// Close the queue, preventing new tasks from being enqueued.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}
