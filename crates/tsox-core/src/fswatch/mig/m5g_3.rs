#![allow(dead_code, unused_imports, unused_variables)]

use std::any::Any;
use std::sync::{Arc, Mutex};

use super::m5f::Debounce;
use super::m5f_2::{Event, EventKind, EventList, FswatchError};
use super::m5g_2::{is_direct_child, is_in_directory_or_self, rebase_path, Callback, IgnoreFn, SequenceFn, WatchCallback};

pub struct DirWatchMu {
    pub callbacks: Vec<Callback>,
    pub debounce: Option<Arc<Debounce>>,
    pub debounce_key: String,
    pub next_cb_id: u64,
}

pub struct DirWatch {
    pub dir: String,
    pub physical_dir: String,
    pub recursive: Mutex<bool>,
    pub events: EventList,
    pub state: Option<Arc<dyn Any + Send + Sync>>,
    pub sequence: Mutex<Option<SequenceFn>>,
    pub mu: Mutex<DirWatchMu>,
}

pub fn new_dir_watch(
    dir: &str,
    physical_dir: &str,
    recursive: bool,
    db: &Arc<Debounce>,
) -> Arc<DirWatch> { crate::fntrace::enter("new_dir_watch"); 
    let dw = Arc::new(DirWatch {
        dir: dir.to_string(),
        physical_dir: physical_dir.to_string(),
        recursive: Mutex::new(recursive),
        events: EventList::new(),
        state: None,
        sequence: Mutex::new(None),
        mu: Mutex::new(DirWatchMu {
            callbacks: Vec::new(),
            debounce: Some(Arc::clone(db)),
            debounce_key: String::new(),
            next_cb_id: 0,
        }),
    });
    let key = format!("{:p}", Arc::as_ptr(&dw));
    dw.mu.lock().unwrap().debounce_key = key.clone();
    let dw_for_cb = Arc::clone(&dw);
    db.add(key, Arc::new(move || dw_for_cb.trigger_callbacks()));
    dw
}

impl DirWatch {
    pub fn recursive_value(&self) -> bool { crate::fntrace::enter("recursive_value"); 
        *self.recursive.lock().unwrap()
    }

    pub fn set_sequence(&self, sequence: Option<SequenceFn>) { crate::fntrace::enter("set_sequence"); 
        *self.sequence.lock().unwrap() = sequence;
    }

    pub fn display_path(&self, watch_path: &str) -> String { crate::fntrace::enter("display_path"); 
        rebase_path(watch_path, &self.physical_dir, &self.dir)
    }

    pub fn physical_path(&self, display_path: &str) -> String { crate::fntrace::enter("physical_path"); 
        rebase_path(display_path, &self.dir, &self.physical_dir)
    }

    pub fn destroy_debounce(&self) { crate::fntrace::enter("destroy_debounce"); 
        let (db, key) = {
            let mut mu = self.mu.lock().unwrap();
            (mu.debounce.take(), mu.debounce_key.clone())
        };
        if let Some(db) = db {
            db.remove(&key);
        }
    }

    pub fn notify(&self) { crate::fntrace::enter("notify"); 
        let (has_pending, has_terminal, db) = {
            let mu = self.mu.lock().unwrap();
            (
                mu.callbacks.iter().any(|cb| !cb.delivered),
                mu.callbacks
                    .iter()
                    .any(|cb| cb.terminal.is_some() && !cb.delivered),
                mu.debounce.clone(),
            )
        };
        let has_events = self.events.size() > 0;
        let has_error = self.events.has_error();
        if has_pending && (has_events || has_error || has_terminal) {
            if let Some(db) = db {
                db.trigger();
            }
        }
    }

    pub fn notify_error(&self, err: &FswatchError) { crate::fntrace::enter("notify_error"); 
        let cbs: Vec<Callback> = {
            let mut mu = self.mu.lock().unwrap();
            std::mem::take(&mut mu.callbacks)
        };
        for cb in cbs {
            (cb.fn_)(&[], Some(err));
        }
    }

    pub fn trigger_callbacks(&self) { crate::fntrace::enter("trigger_callbacks"); 
        let (cbs, events_by_callback, drain_err) = {
            let mut mu = self.mu.lock().unwrap();
            let has_error = self.events.has_error();
            let has_events = self.events.size() > 0;
            let mut cbs: Vec<Callback> = Vec::with_capacity(mu.callbacks.len());
            for cb in &mu.callbacks {
                if cb.delivered {
                    continue;
                }
                cbs.push(cb.clone());
            }
            if cbs.is_empty() {
                if has_events || has_error {
                    let _ = self.events.drain();
                }
                return;
            }
            let has_terminal = cbs.iter().any(|cb| cb.terminal.is_some());
            if !has_events && !has_error && !has_terminal {
                return;
            }
            let start_seqs: Vec<u64> = cbs.iter().map(|cb| cb.since_seq).collect();
            let (events_by_callback, err) = self.events.drain_for_sequences(&start_seqs);
            for cb in &cbs {
                if cb.terminal.is_none() {
                    continue;
                }
                for stored in &mut mu.callbacks {
                    if stored.id == cb.id {
                        stored.delivered = true;
                        break;
                    }
                }
            }
            (cbs, events_by_callback, err)
        };

        for (i, cb) in cbs.iter().enumerate() {
            let mut cb_events = events_by_callback[i].clone();
            if cb.ignore.is_some() || !cb.recursive || cb.dir != self.dir {
                let mut filtered = Vec::with_capacity(cb_events.len());
                for e in cb_events {
                    let e = cb.map_event(e);
                    if let Some(ignore) = &cb.ignore {
                        if ignore(&e.path) {
                            continue;
                        }
                    }
                    if cb.dir != self.dir
                        && !e.included_watch_root
                        && e.path == cb.dir
                        && e.kind == EventKind::Update
                    {
                        continue;
                    }
                    if cb.recursive {
                        if cb.dir != self.dir && !is_in_directory_or_self(&cb.dir, &e.path) {
                            continue;
                        }
                    } else if !is_direct_child(&cb.dir, &e.path)
                        && !(cb.dir != self.dir && e.path == cb.dir)
                    {
                        continue;
                    }
                    filtered.push(e);
                }
                cb_events = filtered;
            }
            let cb_err: Option<FswatchError> = match &cb.terminal {
                Some(terminal) => Some(terminal.clone()),
                None => drain_err.clone(),
            };
            if !cb_events.is_empty() || cb_err.is_some() {
                (cb.fn_)(&cb_events, cb_err.as_ref());
            }
        }
    }

    pub fn terminate_callbacks_for_deleted_root(
        &self,
        path: &str,
        seq: u64,
        err: &FswatchError,
    ) -> bool { crate::fntrace::enter("terminate_callbacks_for_deleted_root"); 
        let mut mu = self.mu.lock().unwrap();
        let mut changed = false;
        for cb in &mut mu.callbacks {
            if cb.delivered || cb.terminal.is_some() || cb.since_seq >= seq {
                continue;
            }
            let physical_path = cb.event_physical_path(path);
            if is_in_directory_or_self(path, &cb.dir)
                || (cb.physical_dir != cb.dir
                    && is_in_directory_or_self(&physical_path, &cb.physical_dir))
            {
                cb.terminal = Some(err.clone());
                changed = true;
            }
        }
        changed
    }

    pub fn watch(
        &self,
        dir: &str,
        physical_dir: &str,
        recursive: bool,
        fn_: WatchCallback,
        ignore: Option<IgnoreFn>,
    ) -> u64 { crate::fntrace::enter("watch"); 
        let mut mu = self.mu.lock().unwrap();
        mu.next_cb_id += 1;
        let id = mu.next_cb_id;
        let mut since_seq = self.events.sequence();
        if let Some(sequence) = self.sequence.lock().unwrap().as_ref() {
            since_seq = sequence();
        }
        mu.callbacks.push(Callback {
            id,
            dir: dir.to_string(),
            physical_dir: physical_dir.to_string(),
            watch_dir: self.dir.clone(),
            watch_physical_dir: self.physical_dir.clone(),
            recursive,
            fn_,
            ignore,
            since_seq,
            terminal: None,
            delivered: false,
        });
        id
    }

    pub fn unwatch(&self, id: u64) -> bool { crate::fntrace::enter("unwatch"); 
        let mut mu = self.mu.lock().unwrap();
        if let Some(i) = mu.callbacks.iter().position(|cb| cb.id == id) {
            mu.callbacks.remove(i);
            return mu.callbacks.is_empty();
        }
        false
    }

    pub fn unref(self: &Arc<Self>, w: &super::m5g_5::WatcherImpl) { crate::fntrace::enter("unref"); 
        let empty = self.mu.lock().unwrap().callbacks.is_empty();
        if empty {
            w.remove_dir_watch(self);
        }
    }
}
