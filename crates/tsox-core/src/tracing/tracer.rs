use super::events::{Phase, TraceArg, TraceEvent};
use super::thread_id::{ThreadKey, resolve_thread_id};
use std::collections::HashMap;
use std::sync::Mutex;

struct Inner {
    events: Vec<TraceEvent>,
    thread_ids: HashMap<ThreadKey, usize>,
}

pub struct Tracer {
    inner: Mutex<Inner>,
}

impl Tracer {
    pub fn new() -> Self {
        Tracer {
            inner: Mutex::new(Inner {
                events: Vec::new(),
                thread_ids: HashMap::new(),
            }),
        }
    }

    pub fn push(&self, phase: Phase, name: &str, args: Vec<(String, TraceArg)>) -> EventGuard<'_> {
        let mut inner = self.inner.lock().unwrap();
        let Inner { events, thread_ids } = &mut *inner;
        let tid = resolve_thread_id(events, thread_ids, &args);
        events.push(TraceEvent {
            tid,
            ph: "B",
            cat: phase.as_str(),
            name: name.to_string(),
            args: args.clone(),
        });
        EventGuard {
            tracer: self,
            tid,
            cat: phase.as_str(),
            name: name.to_string(),
            args,
        }
    }

    pub fn take_events(&self) -> Vec<TraceEvent> {
        std::mem::take(&mut self.inner.lock().unwrap().events)
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventGuard<'a> {
    tracer: &'a Tracer,
    tid: usize,
    cat: &'static str,
    name: String,
    args: Vec<(String, TraceArg)>,
}

impl Drop for EventGuard<'_> {
    fn drop(&mut self) {
        let mut inner = self.tracer.inner.lock().unwrap();
        inner.events.push(TraceEvent {
            tid: self.tid,
            ph: "E",
            cat: self.cat,
            name: self.name.clone(),
            args: self.args.clone(),
        });
    }
}
