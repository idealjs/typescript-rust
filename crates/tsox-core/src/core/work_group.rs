pub(crate) use std::sync::atomic::{AtomicBool, Ordering};
pub(crate) use std::sync::{Arc, Mutex};
pub(crate) use std::thread;

pub trait WorkGroup: Send {
    fn queue(&self, f: Box<dyn FnOnce() + Send>);

    fn run_and_wait(&self);
}

pub fn new_work_group(single_threaded: bool) -> Box<dyn WorkGroup> { crate::fntrace::enter("new_work_group"); 
    if single_threaded {
        Box::new(SingleThreadedWorkGroup::new())
    } else {
        Box::new(ParallelWorkGroup::new())
    }
}

pub(crate) struct ParallelWorkGroup {
    done: Arc<AtomicBool>,
    threads: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl ParallelWorkGroup {
    fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            done: Arc::new(AtomicBool::new(false)),
            threads: Mutex::new(Vec::new()),
        }
    }
}

impl WorkGroup for ParallelWorkGroup {
    fn queue(&self, f: Box<dyn FnOnce() + Send>) { crate::fntrace::enter("queue"); 
        if self.done.load(Ordering::SeqCst) {
            panic!("Queue called after RunAndWait returned");
        }
        let handle = thread::spawn(f);
        self.threads.lock().unwrap().push(handle);
    }

    fn run_and_wait(&self) { crate::fntrace::enter("run_and_wait"); 
        let threads = std::mem::take(&mut *self.threads.lock().unwrap());
        for handle in threads {
            handle.join().expect("worker thread panicked");
        }
        self.done.store(true, Ordering::SeqCst);
    }
}

pub(crate) struct SingleThreadedWorkGroup {
    done: AtomicBool,
    fns: Mutex<Vec<Box<dyn FnOnce() + Send>>>,
}

impl SingleThreadedWorkGroup {
    fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            done: AtomicBool::new(false),
            fns: Mutex::new(Vec::new()),
        }
    }
}

impl WorkGroup for SingleThreadedWorkGroup {
    fn queue(&self, f: Box<dyn FnOnce() + Send>) { crate::fntrace::enter("queue"); 
        if self.done.load(Ordering::SeqCst) {
            panic!("Queue called after RunAndWait returned");
        }
        self.fns.lock().unwrap().push(f);
    }

    fn run_and_wait(&self) { crate::fntrace::enter("run_and_wait"); 
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
