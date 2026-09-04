use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub trait WorkGroup: Send {

    fn queue(&self, f: Box<dyn FnOnce() + Send>);

    fn run_and_wait(&self);
}

pub fn new_work_group(single_threaded: bool) -> Box<dyn WorkGroup> {
    if single_threaded {
        Box::new(SingleThreadedWorkGroup::new())
    } else {
        Box::new(ParallelWorkGroup::new())
    }
}

struct ParallelWorkGroup {
    done: Arc<AtomicBool>,
    threads: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl ParallelWorkGroup {
    fn new() -> Self {
        Self {
            done: Arc::new(AtomicBool::new(false)),
            threads: Mutex::new(Vec::new()),
        }
    }
}

impl WorkGroup for ParallelWorkGroup {
    fn queue(&self, f: Box<dyn FnOnce() + Send>) {
        if self.done.load(Ordering::SeqCst) {
            panic!("Queue called after RunAndWait returned");
        }
        let handle = thread::spawn(f);
        self.threads.lock().unwrap().push(handle);
    }

    fn run_and_wait(&self) {
        let threads = std::mem::take(&mut *self.threads.lock().unwrap());
        for handle in threads {
            handle.join().expect("worker thread panicked");
        }
        self.done.store(true, Ordering::SeqCst);
    }
}

struct SingleThreadedWorkGroup {
    done: AtomicBool,
    fns: Mutex<Vec<Box<dyn FnOnce() + Send>>>,
}

impl SingleThreadedWorkGroup {
    fn new() -> Self {
        Self {
            done: AtomicBool::new(false),
            fns: Mutex::new(Vec::new()),
        }
    }
}

impl WorkGroup for SingleThreadedWorkGroup {
    fn queue(&self, f: Box<dyn FnOnce() + Send>) {
        if self.done.load(Ordering::SeqCst) {
            panic!("Queue called after RunAndWait returned");
        }
        self.fns.lock().unwrap().push(f);
    }

    fn run_and_wait(&self) {
        loop {
            let f = self.fns.lock().unwrap().pop();
            match f {
                Some(f) => f(),
                None => break,
            }
        }
        self.done.store(true, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn parallel() {
        let wg = new_work_group(false);
        let counter = Arc::new(AtomicUsize::new(0));
        for _ in 0..10 {
            let counter = counter.clone();
            wg.queue(Box::new(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }));
        }
        wg.run_and_wait();
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn single_threaded() {
        let wg = new_work_group(true);
        let counter = Arc::new(AtomicUsize::new(0));
        for _ in 0..10 {
            let counter = counter.clone();
            wg.queue(Box::new(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }));
        }
        wg.run_and_wait();
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }
}
