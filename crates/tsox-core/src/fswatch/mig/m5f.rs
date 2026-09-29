#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub const MIN_WAIT_TIME: Duration = Duration::from_millis(50);
pub const MAX_WAIT_TIME: Duration = Duration::from_millis(500);

pub type DebounceKey = String;
pub type DebounceCallback = Arc<dyn Fn() + Send + Sync>;

pub fn canonicalize_path(p: &str) -> String {
    p.to_string()
}

#[derive(Clone)]
struct CloseChannel(Arc<(Mutex<bool>, Condvar)>);

impl CloseChannel {
    fn new() -> Self {
        CloseChannel(Arc::new((Mutex::new(false), Condvar::new())))
    }

    fn close(&self) {
        let (lock, cv) = &*self.0;
        let mut closed = lock.lock().unwrap();
        if !*closed {
            *closed = true;
            cv.notify_all();
        }
    }

    fn wait(&self) {
        let (lock, cv) = &*self.0;
        let mut closed = lock.lock().unwrap();
        while !*closed {
            closed = cv.wait(closed).unwrap();
        }
    }

    fn wait_timeout(&self, timeout: Duration) -> bool {
        let (lock, cv) = &*self.0;
        let mut closed = lock.lock().unwrap();
        let deadline = Instant::now() + timeout;
        while !*closed {
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            let (guard, res) = cv.wait_timeout(closed, deadline - now).unwrap();
            closed = guard;
            if res.timed_out() {
                return *closed;
            }
        }
        true
    }
}

struct DebounceState {
    callbacks: HashMap<DebounceKey, DebounceCallback>,
    last_time: Option<Instant>,
}

struct DebounceLatch {
    wait_ch: Option<CloseChannel>,
    trigger_ch: Option<CloseChannel>,
    notified: bool,
}

impl DebounceLatch {
    fn wait_ch_locked(&mut self) -> CloseChannel {
        self.wait_ch.get_or_insert_with(CloseChannel::new).clone()
    }

    fn trigger_ch_locked(&mut self) -> CloseChannel {
        self.trigger_ch.get_or_insert_with(CloseChannel::new).clone()
    }
}

pub struct Debounce {
    mu: Mutex<DebounceState>,
    latch_mu: Mutex<DebounceLatch>,
}

impl Debounce {
    fn new() -> Self {
        Debounce {
            mu: Mutex::new(DebounceState {
                callbacks: HashMap::new(),
                last_time: None,
            }),
            latch_mu: Mutex::new(DebounceLatch {
                wait_ch: None,
                trigger_ch: None,
                notified: false,
            }),
        }
    }

    pub fn add(&self, key: DebounceKey, cb: DebounceCallback) {
        self.mu.lock().unwrap().callbacks.insert(key, cb);
    }

    pub fn remove(&self, key: &str) {
        self.mu.lock().unwrap().callbacks.remove(key);
    }

    pub fn trigger(&self) {
        let mut latch = self.latch_mu.lock().unwrap();
        if !latch.notified {
            latch.notified = true;
            latch.wait_ch_locked().close();
        }
        latch.trigger_ch_locked().close();
        latch.trigger_ch = Some(CloseChannel::new());
    }

    fn run_loop(self: &Arc<Self>) {
        loop {
            self.latch_wait();
            self.notify_if_ready();
        }
    }

    fn notify_if_ready(&self) {
        let fire = {
            let mut state = self.mu.lock().unwrap();
            let now = Instant::now();
            let gap_exceeded = match state.last_time {
                Some(last) => now.duration_since(last) > MAX_WAIT_TIME,
                None => true,
            };
            if gap_exceeded {
                state.last_time = Some(now);
                true
            } else {
                false
            }
        };
        if fire {
            self.fire_callbacks();
        } else {
            self.coalesce_wait();
        }
    }

    fn coalesce_wait(&self) {
        let ch = {
            let mut latch = self.latch_mu.lock().unwrap();
            latch.trigger_ch_locked()
        };
        if !ch.wait_timeout(MIN_WAIT_TIME) {
            self.fire_callbacks();
        }
    }

    fn fire_callbacks(&self) {
        let cbs: Vec<DebounceCallback> = {
            let mut state = self.mu.lock().unwrap();
            state.last_time = Some(Instant::now());
            state.callbacks.values().cloned().collect()
        };
        self.latch_reset();
        for cb in cbs {
            cb();
        }
    }

    fn latch_wait(&self) {
        let ch = {
            let mut latch = self.latch_mu.lock().unwrap();
            latch.wait_ch_locked()
        };
        ch.wait();
    }

    fn latch_reset(&self) {
        let mut latch = self.latch_mu.lock().unwrap();
        if latch.notified {
            latch.notified = false;
            latch.wait_ch = Some(CloseChannel::new());
        }
    }
}

pub fn new_debounce() -> Arc<Debounce> {
    let d = Arc::new(Debounce::new());
    let worker = Arc::clone(&d);
    let _ = std::thread::Builder::new()
        .name("fswatch-debounce".into())
        .spawn(move || worker.run_loop());
    d
}
