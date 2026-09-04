use std::collections::HashMap;
use std::sync::Mutex;

use xxhash_rust::xxh3;

const MAIN_THREAD_ID: usize = 1;
const FIRST_SYNTHETIC_THREAD_ID: usize = 2;
const FIRST_FILE_THREAD_ID: usize = 1_000_000;
const FILE_THREAD_ID_HASH_RANGE: usize = 1_000_000_000;

const FILE_THREAD_ARG_KEYS: &[&str] = &[
    "path",
    "fileName",
    "containingFileName",
    "jsFilePath",
    "declarationFilePath",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Parse,
    Program,
    Bind,
    Check,
    CheckTypes,
    Emit,
    Session,
}

impl Phase {
    fn as_str(self) -> &'static str {
        match self {
            Phase::Parse => "parse",
            Phase::Program => "program",
            Phase::Bind => "bind",
            Phase::Check => "check",
            Phase::CheckTypes => "checkTypes",
            Phase::Emit => "emit",
            Phase::Session => "session",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceArg {
    Int(i64),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub tid: usize,

    pub ph: &'static str,
    pub cat: &'static str,
    pub name: String,
    pub args: Vec<(String, TraceArg)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ThreadKey {
    Checker { index: usize },
    File { text: String },
}

impl ThreadKey {

    fn display_name(&self) -> String {
        match self {
            ThreadKey::Checker { index } => format!("checker:{index}"),
            ThreadKey::File { text } => format!("file:{text}"),
        }
    }

    fn default_thread_id(&self) -> usize {
        match self {

            ThreadKey::Checker { index } => FIRST_SYNTHETIC_THREAD_ID + index,
            ThreadKey::File { .. } => stable_trace_thread_id(self),
        }
    }
}

fn stable_trace_thread_id(key: &ThreadKey) -> usize {
    let input = match key {
        ThreadKey::Checker { index } => format!("checker:{index}"),
        ThreadKey::File { text } => format!("file:{text}"),
    };
    let hash = xxh3::xxh3_64(input.as_bytes());
    FIRST_FILE_THREAD_ID + (hash as usize % FILE_THREAD_ID_HASH_RANGE)
}

fn thread_key_from_args(args: &[(String, TraceArg)]) -> Option<ThreadKey> {
    if args.is_empty() {
        return None;
    }

    for (key, value) in args {
        if key == "checkerId" {
            if let TraceArg::Int(id) = value {
                return Some(ThreadKey::Checker {
                    index: *id as usize,
                });
            }
        }
    }

    for arg_key in FILE_THREAD_ARG_KEYS {
        for (key, value) in args {
            if key == *arg_key {
                if let TraceArg::Str(path) = value {
                    if !path.is_empty() {
                        return Some(ThreadKey::File { text: path.clone() });
                    }
                }
            }
        }
    }
    None
}

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

    fn resolve_thread_id(inner: &mut Inner, args: &[(String, TraceArg)]) -> usize {
        let key = match thread_key_from_args(args) {
            Some(k) => k,
            None => return MAIN_THREAD_ID,
        };
        if let Some(&tid) = inner.thread_ids.get(&key) {
            return tid;
        }

        let mut tid = key.default_thread_id();
        while inner.thread_ids.values().any(|&existing| existing == tid) {
            tid += 1;
        }
        inner.thread_ids.insert(key.clone(), tid);
        inner.events.push(TraceEvent {
            tid,
            ph: "M",
            cat: "__metadata",
            name: "thread_name".to_string(),
            args: vec![("name".to_string(), TraceArg::Str(key.display_name()))],
        });
        tid
    }

    pub fn push(&self, phase: Phase, name: &str, args: Vec<(String, TraceArg)>) -> EventGuard<'_> {
        let mut inner = self.inner.lock().unwrap();
        let tid = Self::resolve_thread_id(&mut inner, &args);
        inner.events.push(TraceEvent {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_duration_events_use_separate_thread_ids() {
        let tr = Tracer::new();

        let end_a = tr.push(
            Phase::Parse,
            "createSourceFile",
            vec![("path".into(), TraceArg::Str("/a.ts".into()))],
        );
        let end_b = tr.push(
            Phase::Parse,
            "createSourceFile",
            vec![("path".into(), TraceArg::Str("/b.ts".into()))],
        );
        drop(end_a);
        drop(end_b);

        let end_check = tr.push(
            Phase::Check,
            "checkSourceFile",
            vec![
                ("checkerId".into(), TraceArg::Int(0)),
                ("path".into(), TraceArg::Str("/a.ts".into())),
            ],
        );
        let end_variance = tr.push(
            Phase::CheckTypes,
            "getVariancesWorker",
            vec![
                ("checkerId".into(), TraceArg::Int(0)),
                ("id".into(), TraceArg::Int(1)),
            ],
        );
        drop(end_variance);
        drop(end_check);

        let events = tr.take_events();

        let a_begin = find_event(
            &events,
            "B",
            "createSourceFile",
            "path",
            &TraceArg::Str("/a.ts".into()),
        );
        let a_end = find_event(
            &events,
            "E",
            "createSourceFile",
            "path",
            &TraceArg::Str("/a.ts".into()),
        );
        let b_begin = find_event(
            &events,
            "B",
            "createSourceFile",
            "path",
            &TraceArg::Str("/b.ts".into()),
        );
        let b_end = find_event(
            &events,
            "E",
            "createSourceFile",
            "path",
            &TraceArg::Str("/b.ts".into()),
        );
        assert_eq!(a_begin.tid, a_end.tid);
        assert_eq!(b_begin.tid, b_end.tid);
        assert_ne!(a_begin.tid, b_begin.tid);
        assert_thread_name(&events, a_begin.tid, "file:/a.ts");
        assert_thread_name(&events, b_begin.tid, "file:/b.ts");

        let check_begin = find_event(
            &events,
            "B",
            "checkSourceFile",
            "path",
            &TraceArg::Str("/a.ts".into()),
        );
        let variance_begin =
            find_event(&events, "B", "getVariancesWorker", "id", &TraceArg::Int(1));
        assert_eq!(check_begin.tid, variance_begin.tid);
        assert_thread_name(&events, check_begin.tid, "checker:0");

        assert_duration_events_are_well_nested_by_thread(&events);
    }

    #[test]
    fn thread_ids_are_stable_across_first_seen_order() {
        let first = trace_thread_ids_for_paths(&["/a.ts", "/b.ts"]);
        let second = trace_thread_ids_for_paths(&["/b.ts", "/a.ts"]);
        assert_eq!(first, second);
    }

    fn trace_thread_ids_for_paths(paths: &[&str]) -> HashMap<String, usize> {
        let tr = Tracer::new();
        for path in paths {
            let end = tr.push(
                Phase::Parse,
                "createSourceFile",
                vec![("path".into(), TraceArg::Str((*path).into()))],
            );
            drop(end);
        }
        let events = tr.take_events();
        let mut ids = HashMap::new();
        for path in paths {
            let begin = find_event(
                &events,
                "B",
                "createSourceFile",
                "path",
                &TraceArg::Str((*path).into()),
            );
            ids.insert((*path).to_string(), begin.tid);
        }
        ids
    }

    fn find_event(
        events: &[TraceEvent],
        ph: &str,
        name: &str,
        arg_name: &str,
        arg_value: &TraceArg,
    ) -> TraceEvent {
        for event in events {
            if event.ph == ph && event.name == name {
                for (k, v) in &event.args {
                    if k == arg_name && v == arg_value {
                        return event.clone();
                    }
                }
            }
        }
        panic!("failed to find {ph} event {name:?} with {arg_name}={arg_value:?}");
    }

    fn assert_thread_name(events: &[TraceEvent], tid: usize, name: &str) {
        for event in events {
            if event.ph == "M" && event.name == "thread_name" && event.tid == tid {
                for (k, v) in &event.args {
                    if k == "name" && v == &TraceArg::Str(name.to_string()) {
                        return;
                    }
                }
            }
        }
        panic!("failed to find thread_name metadata for thread {tid} named {name:?}");
    }

    fn assert_duration_events_are_well_nested_by_thread(events: &[TraceEvent]) {
        let mut stacks: HashMap<usize, Vec<&TraceEvent>> = HashMap::new();
        for event in events {
            match event.ph {
                "B" => stacks.entry(event.tid).or_default().push(event),
                "E" => {
                    let stack = stacks.entry(event.tid).or_default();
                    assert!(
                        !stack.is_empty(),
                        "unmatched end event {:?} on thread {}",
                        event.name,
                        event.tid
                    );
                    let begin = stack.pop().unwrap();
                    assert_eq!(
                        begin.cat, event.cat,
                        "mismatched cat on thread {}",
                        event.tid
                    );
                    assert_eq!(
                        begin.name, event.name,
                        "mismatched name on thread {}",
                        event.tid
                    );
                }
                _ => {}
            }
        }
        for (tid, stack) in &stacks {
            assert!(
                stack.is_empty(),
                "thread {tid} has {} unterminated events",
                stack.len()
            );
        }
    }
}
