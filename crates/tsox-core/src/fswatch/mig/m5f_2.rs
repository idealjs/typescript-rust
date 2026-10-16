#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

pub type FswatchError = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Update,
    Delete,
}

impl EventKind {
    pub fn as_str(&self) -> &'static str { crate::fntrace::enter("as_str"); 
        match self {
            EventKind::Update => "update",
            EventKind::Delete => "delete",
        }
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { crate::fntrace::enter("fmt"); 
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: EventKind,
    pub path: String,
    pub included_watch_root: bool,
}

#[derive(Debug, Default, Clone)]
struct EventEntry {
    created_seq: u64,
    updated_seq: u64,
    deleted_seq: u64,
    included_watch_root: bool,
}

impl EventEntry {
    fn is_deleted(&self) -> bool { crate::fntrace::enter("is_deleted"); 
        self.deleted_seq > self.created_seq && self.deleted_seq > self.updated_seq
    }

    fn kind_since(&self, start_seq: u64) -> Option<EventKind> { crate::fntrace::enter("kind_since"); 
        if self.deleted_seq > start_seq {
            if self.created_seq > start_seq
                && self.created_seq < self.deleted_seq
                && self.updated_seq < self.deleted_seq
            {
                return None;
            }
            return Some(EventKind::Delete);
        }
        let seq = self.created_seq.max(self.updated_seq);
        if seq > start_seq {
            Some(EventKind::Update)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
struct EventListInner {
    entries: HashMap<String, EventEntry>,
    err: Option<FswatchError>,
    seq: u64,
}

impl EventListInner {
    fn get_or_create(&mut self, path: &str) -> &mut EventEntry { crate::fntrace::enter("get_or_create"); 
        self.entries.entry(path.to_string()).or_default()
    }

    fn next_seq_locked(&mut self) -> u64 { crate::fntrace::enter("next_seq_locked"); 
        self.seq += 1;
        self.seq
    }

    fn advance_seq_locked(&mut self, seq: u64) { crate::fntrace::enter("advance_seq_locked"); 
        if seq > self.seq {
            self.seq = seq;
        }
    }

    fn create_locked(&mut self, path: &str, seq: u64) { crate::fntrace::enter("create_locked"); 
        let entry = self.get_or_create(path);
        if entry.is_deleted() {
            entry.deleted_seq = 0;
            entry.created_seq = 0;
            entry.updated_seq = seq;
        } else {
            entry.created_seq = seq;
        }
    }

    fn update_locked(&mut self, path: &str, seq: u64) { crate::fntrace::enter("update_locked"); 
        self.get_or_create(path).updated_seq = seq;
    }

    fn remove_locked(&mut self, path: &str, seq: u64) { crate::fntrace::enter("remove_locked"); 
        let entry = self.get_or_create(path);
        entry.deleted_seq = seq;
    }

    fn snapshot_locked(&self) -> Vec<Event> { crate::fntrace::enter("snapshot_locked"); 
        self.snapshot_since_locked(0)
    }

    fn snapshot_since_locked(&self, start_seq: u64) -> Vec<Event> { crate::fntrace::enter("snapshot_since_locked"); 
        let mut out = Vec::with_capacity(self.entries.len());
        for (path, e) in &self.entries {
            if let Some(kind) = e.kind_since(start_seq) {
                out.push(Event {
                    kind,
                    path: path.clone(),
                    included_watch_root: e.included_watch_root,
                });
            }
        }
        out
    }
}

pub struct EventList {
    inner: Mutex<EventListInner>,
}

impl Default for EventList {
    fn default() -> Self { crate::fntrace::enter("default"); 
        Self::new()
    }
}

impl EventList {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        EventList {
            inner: Mutex::new(EventListInner::default()),
        }
    }

    pub fn create(&self, path: &str) { crate::fntrace::enter("create"); 
        let mut inner = self.inner.lock().unwrap();
        let seq = inner.next_seq_locked();
        inner.create_locked(path, seq);
    }

    pub fn create_at(&self, path: &str, seq: u64) { crate::fntrace::enter("create_at"); 
        let mut inner = self.inner.lock().unwrap();
        inner.advance_seq_locked(seq);
        inner.create_locked(path, seq);
    }

    pub fn update(&self, path: &str) { crate::fntrace::enter("update"); 
        let mut inner = self.inner.lock().unwrap();
        let seq = inner.next_seq_locked();
        inner.update_locked(path, seq);
    }

    pub fn update_at(&self, path: &str, seq: u64) { crate::fntrace::enter("update_at"); 
        let mut inner = self.inner.lock().unwrap();
        inner.advance_seq_locked(seq);
        inner.update_locked(path, seq);
    }

    pub fn update_watch_root_at(&self, path: &str, seq: u64) { crate::fntrace::enter("update_watch_root_at"); 
        let mut inner = self.inner.lock().unwrap();
        inner.advance_seq_locked(seq);
        inner.update_locked(path, seq);
        inner.get_or_create(path).included_watch_root = true;
    }

    pub fn remove(&self, path: &str) { crate::fntrace::enter("remove"); 
        let mut inner = self.inner.lock().unwrap();
        let seq = inner.next_seq_locked();
        inner.remove_locked(path, seq);
    }

    pub fn remove_and_get_sequence(&self, path: &str) -> u64 { crate::fntrace::enter("remove_and_get_sequence"); 
        let mut inner = self.inner.lock().unwrap();
        let seq = inner.next_seq_locked();
        inner.remove_locked(path, seq);
        seq
    }

    pub fn remove_at(&self, path: &str, seq: u64) { crate::fntrace::enter("remove_at"); 
        let mut inner = self.inner.lock().unwrap();
        inner.advance_seq_locked(seq);
        inner.remove_locked(path, seq);
    }

    pub fn remove_watch_root_at(&self, path: &str, seq: u64) { crate::fntrace::enter("remove_watch_root_at"); 
        let mut inner = self.inner.lock().unwrap();
        inner.advance_seq_locked(seq);
        inner.remove_locked(path, seq);
        inner.get_or_create(path).included_watch_root = true;
    }

    pub fn size(&self) -> usize { crate::fntrace::enter("size"); 
        self.inner.lock().unwrap().entries.len()
    }

    pub fn get_events(&self) -> Vec<Event> { crate::fntrace::enter("get_events"); 
        self.inner.lock().unwrap().snapshot_locked()
    }

    pub fn drain(&self) -> (Vec<Event>, Option<FswatchError>) { crate::fntrace::enter("drain"); 
        let mut inner = self.inner.lock().unwrap();
        let out = inner.snapshot_locked();
        let err = inner.err.take();
        inner.entries.clear();
        (out, err)
    }

    pub fn drain_for_sequences(&self, start_seqs: &[u64]) -> (Vec<Vec<Event>>, Option<FswatchError>) { crate::fntrace::enter("drain_for_sequences"); 
        let mut inner = self.inner.lock().unwrap();
        let out = start_seqs
            .iter()
            .map(|start_seq| inner.snapshot_since_locked(*start_seq))
            .collect();
        let err = inner.err.take();
        inner.entries.clear();
        (out, err)
    }

    pub fn set_error(&self, err: FswatchError) { crate::fntrace::enter("set_error"); 
        let mut inner = self.inner.lock().unwrap();
        if inner.err.is_none() {
            inner.err = Some(err);
        }
    }

    pub fn has_error(&self) -> bool { crate::fntrace::enter("has_error"); 
        self.inner.lock().unwrap().err.is_some()
    }

    pub fn get_error(&self) -> Option<FswatchError> { crate::fntrace::enter("get_error"); 
        self.inner.lock().unwrap().err.clone()
    }

    pub fn sequence(&self) -> u64 { crate::fntrace::enter("sequence"); 
        self.inner.lock().unwrap().seq
    }
}
