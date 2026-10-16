#![allow(dead_code, unused_imports, unused_variables)]

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, Weak};

use super::m5f_2::FswatchError;
use super::m5g_2::DirWatchError;
use super::m5g_3::DirWatch;
use super::m5g_2::WatcherBackend;
use super::m5g_2::ERR_WATCH_TERMINATED;

struct WatcherBaseMu {
    subscriptions: HashMap<usize, Arc<DirWatch>>,
    start_err: Option<FswatchError>,
}

pub struct WatcherBase {
    self_backend: Mutex<Option<Weak<dyn WatcherBackend>>>,
    mu: Mutex<WatcherBaseMu>,
    started: (Mutex<bool>, Condvar),
}

impl Default for WatcherBase {
    fn default() -> Self { crate::fntrace::enter("default"); 
        WatcherBase {
            self_backend: Mutex::new(None),
            mu: Mutex::new(WatcherBaseMu {
                subscriptions: HashMap::new(),
                start_err: None,
            }),
            started: (Mutex::new(false), Condvar::new()),
        }
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> FswatchError { crate::fntrace::enter("panic_message"); 
    if let Some(s) = payload.downcast_ref::<&str>() {
        return (*s).to_string();
    }
    if let Some(s) = payload.downcast_ref::<String>() {
        return s.clone();
    }
    "fswatch: panic in backend start".to_string()
}

impl WatcherBase {
    pub fn init(&self, self_backend: Arc<dyn WatcherBackend>) { crate::fntrace::enter("init"); 
        *self.self_backend.lock().unwrap() = Some(Arc::downgrade(&self_backend));
        let mut mu = self.mu.lock().unwrap();
        mu.subscriptions.clear();
        mu.start_err = None;
        *self.started.0.lock().unwrap() = false;
    }

    fn get_self(&self) -> Option<Arc<dyn WatcherBackend>> { crate::fntrace::enter("get_self"); 
        self.self_backend
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|w| w.upgrade())
    }

    pub fn notify_started(&self) { crate::fntrace::enter("notify_started"); 
        let (lock, cv) = &self.started;
        let mut started = lock.lock().unwrap();
        if !*started {
            *started = true;
            cv.notify_all();
        }
    }

    pub fn shutdown(&self) { crate::fntrace::enter("shutdown"); }

    pub fn run(self: &Arc<Self>) -> Result<(), FswatchError> { crate::fntrace::enter("run"); 
        let self_backend = self
            .get_self()
            .ok_or_else(|| "fswatch: watcher base not initialized".to_string())?;
        let base = Arc::clone(self);
        let _ = std::thread::Builder::new()
            .name("fswatch-backend".into())
            .spawn(move || {
                let result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self_backend.start()));
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => base.handle_start_error(&err),
                    Err(payload) => base.handle_start_error(&panic_message(payload)),
                }
            });
        let (lock, cv) = &self.started;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cv.wait(started).unwrap();
        }
        let mu = self.mu.lock().unwrap();
        match &mu.start_err {
            Some(err) => Err(err.clone()),
            None => Ok(()),
        }
    }

    pub fn handle_start_error(&self, err: &FswatchError) { crate::fntrace::enter("handle_start_error"); 
        let subs: Vec<Arc<DirWatch>> = {
            let mut mu = self.mu.lock().unwrap();
            mu.start_err = Some(err.clone());
            mu.subscriptions.values().cloned().collect()
        };
        for w in &subs {
            w.notify_error(err);
        }
        self.notify_started();
    }

    pub fn watch_add(&self, w: &Arc<DirWatch>) -> Result<(), FswatchError> { crate::fntrace::enter("watch_add"); 
        self.watch_add_many(std::slice::from_ref(w))
    }

    pub fn watch_add_many(&self, watches: &[Arc<DirWatch>]) -> Result<(), FswatchError> { crate::fntrace::enter("watch_add_many"); 
        let self_backend = self
            .get_self()
            .ok_or_else(|| "fswatch: watcher base not initialized".to_string())?;
        let to_add: Vec<Arc<DirWatch>> = {
            let mu = self.mu.lock().unwrap();
            watches
                .iter()
                .filter(|w| !mu.subscriptions.contains_key(&(Arc::as_ptr(*w) as usize)))
                .cloned()
                .collect()
        };
        if to_add.is_empty() {
            return Ok(());
        }
        let mut added: Vec<Arc<DirWatch>> = Vec::new();
        for w in &to_add {
            match self_backend.subscribe(w) {
                Ok(()) => {
                    let mut mu = self.mu.lock().unwrap();
                    mu.subscriptions
                        .insert(Arc::as_ptr(w) as usize, Arc::clone(w));
                    added.push(Arc::clone(w));
                }
                Err(err) => {
                    let mut mu = self.mu.lock().unwrap();
                    for added_watch in &added {
                        mu.subscriptions
                            .remove(&(Arc::as_ptr(added_watch) as usize));
                        let _ = self_backend.close_watch(added_watch);
                    }
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    pub fn watch_remove(&self, w: &Arc<DirWatch>) { crate::fntrace::enter("watch_remove"); 
        let mut mu = self.mu.lock().unwrap();
        if mu
            .subscriptions
            .remove(&(Arc::as_ptr(w) as usize))
            .is_some()
        {
            if let Some(self_backend) = self.get_self() {
                let _ = self_backend.close_watch(w);
            }
        }
    }

    pub fn handle_watcher_error(&self, werr: &DirWatchError) { crate::fntrace::enter("handle_watcher_error"); 
        self.watch_remove(&werr.dir_watch);
        werr
            .dir_watch
            .notify_error(&format!("{}: {}", ERR_WATCH_TERMINATED, werr));
    }
}
