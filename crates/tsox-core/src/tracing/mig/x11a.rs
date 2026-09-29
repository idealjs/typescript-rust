#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::tracing::Tracer;
use crate::tspath::combine_paths;

pub const TRACE_FILE_NAME: &str = "trace.json";
pub const MAIN_THREAD_ID: i32 = 0;

pub trait FS: Send + Sync {
    fn write_file(&self, path: &str, contents: &str) -> Result<(), String>;
    fn append_file(&self, path: &str, contents: &str) -> Result<(), String>;
}

#[derive(Clone, Default)]
pub struct TraceRecord {
    pub config_file_path: String,
    pub trace_path: String,
    pub types_path: String,
    pub checker_id: i32,
}

#[derive(Default)]
pub struct TraceEvent {
    pub pid: i32,
    pub tid: i32,
    pub ph: &'static str,
    pub cat: &'static str,
    pub ts: f64,
    pub name: &'static str,
    pub s: &'static str,
    pub dur: Option<f64>,
    pub args: HashMap<String, Box<dyn std::any::Any>>,
}

pub struct Tracing<'a> {
    fs: &'a dyn FS,
    trace_dir: String,
    trace_path: String,
    config_file_path: String,
    legend: Vec<TraceRecord>,
    tracers: Vec<Tracer>,
    trace_content: String,
    trace_started: AtomicBool,
    thread_ids: HashMap<TraceThreadKey, i32>,
    thread_keys: HashMap<i32, TraceThreadKey>,
    metadata_ts: f64,
    deterministic: bool,
    timestamp_counter: u64,
    start_time: Instant,
    next_thread_id: i32,
    flush_err: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct TraceThreadKey {
    pub kind: TraceThreadKind,
    pub index: i32,
    pub text: String,
    pub has_index: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TraceThreadKind {
    #[default]
    Unknown,
    File,
    Checker,
}

pub fn start_tracing<'a>(
    fs: &'a dyn FS,
    trace_dir: &str,
    config_file_path: &str,
    deterministic: bool,
) -> Result<Tracing<'a>, String> {
    let mut tr = Tracing {
        fs,
        trace_dir: trace_dir.to_string(),
        trace_path: combine_paths(trace_dir, &[TRACE_FILE_NAME]),
        config_file_path: config_file_path.to_string(),
        legend: Vec::new(),
        tracers: Vec::new(),
        trace_content: String::new(),
        trace_started: AtomicBool::new(false),
        thread_ids: HashMap::new(),
        thread_keys: HashMap::new(),
        metadata_ts: 0.0,
        deterministic,
        timestamp_counter: 0,
        start_time: std::time::Instant::now(),
        next_thread_id: 0,
        flush_err: None,
    };
    tr.trace_started.store(true, std::sync::atomic::Ordering::SeqCst);
    tr.trace_content.write_str("[\n").ok();
    let meta_ts = tr.timestamp();
    tr.metadata_ts = meta_ts;
    tr.write_event(TraceEvent {
        pid: 1,
        tid: MAIN_THREAD_ID,
        ph: "M",
        cat: "__metadata",
        ts: meta_ts,
        name: "process_name",
        args: [("name".to_string(), Box::new("tsgo") as Box<dyn std::any::Any>)].into_iter().collect(),
        ..Default::default()
    });
    tr.trace_content.write_str(",\n").ok();
    tr.write_event(TraceEvent {
        pid: 1,
        tid: MAIN_THREAD_ID,
        ph: "M",
        cat: "__metadata",
        ts: meta_ts,
        name: "thread_name",
        args: [("name".to_string(), Box::new("Main") as Box<dyn std::any::Any>)].into_iter().collect(),
        ..Default::default()
    });
    tr.trace_content.write_str(",\n").ok();
    tr.write_event(TraceEvent {
        pid: 1,
        tid: MAIN_THREAD_ID,
        ph: "M",
        cat: "disabled-by-default-devtools.timeline",
        ts: meta_ts,
        name: "TracingStartedInBrowser",
        args: HashMap::new(),
        ..Default::default()
    });
    if let Err(err) = tr.fs.write_file(&tr.trace_path, &tr.trace_content) {
        return Err(format!("failed to write trace file header: {err}"));
    }
    tr.trace_content.clear();
    Ok(tr)
}

pub fn trace_thread_key_from_args(args: &HashMap<String, Box<dyn std::any::Any>>) -> Option<TraceThreadKey> {
    if args.is_empty() {
        return None;
    }
    if let Some(checker_id) = args.get("checkerId").and_then(|v| v.downcast_ref::<i32>()) {
        return Some(TraceThreadKey {
            kind: TraceThreadKind::Checker,
            index: *checker_id,
            has_index: true,
            ..Default::default()
        });
    }
    for key in TRACE_THREAD_ARG_KEYS {
        if let Some(value) = args.get(*key) {
            if let Some(path) = value.downcast_ref::<String>() {
                if !path.is_empty() {
                    return Some(TraceThreadKey {
                        kind: TraceThreadKind::File,
                        text: path.clone(),
                        ..Default::default()
                    });
                }
            }
        }
    }
    None
}

pub const TRACE_THREAD_ARG_KEYS: &[&str] = &[
    "path",
    "fileName",
    "containingFileName",
    "jsFilePath",
    "declarationFilePath",
];

impl Tracing<'_> {
    pub fn timestamp(&mut self) -> f64 {
        if self.deterministic {
            self.timestamp_counter += 1;
            return self.timestamp_counter as f64;
        }
        self.start_time.elapsed().as_nanos() as f64 / 1000.0
    }

    pub fn write_event(&mut self, event: TraceEvent) {
        let mut obj = serde_json::Map::new();
        obj.insert("pid".to_string(), event.pid.into());
        obj.insert("tid".to_string(), event.tid.into());
        obj.insert("ph".to_string(), event.ph.into());
        obj.insert("cat".to_string(), event.cat.into());
        obj.insert("ts".to_string(), event.ts.into());
        if !event.name.is_empty() {
            obj.insert("name".to_string(), event.name.into());
        }
        if !event.s.is_empty() {
            obj.insert("s".to_string(), event.s.into());
        }
        if let Some(dur) = event.dur {
            obj.insert("dur".to_string(), dur.into());
        }
        if !event.args.is_empty() {
            let mut args = serde_json::Map::new();
            for (key, value) in &event.args {
                args.insert(key.clone(), json_arg(value));
            }
            obj.insert("args".to_string(), serde_json::Value::Object(args));
        }
        let _ = write!(self.trace_content, "{}", serde_json::Value::Object(obj));
    }

    pub fn thread_id_locked(&mut self, args: &HashMap<String, Box<dyn std::any::Any>>) -> i32 {
        let Some(key) = trace_thread_key_from_args(args) else {
            return MAIN_THREAD_ID;
        };
        if let Some(tid) = self.thread_ids.get(&key) {
            return *tid;
        }
        let tid = self.next_thread_id;
        self.next_thread_id += 1;
        self.thread_ids.insert(key.clone(), tid);
        self.thread_keys.insert(tid, key);
        tid
    }
}

fn json_arg(value: &Box<dyn std::any::Any>) -> serde_json::Value {
    if let Some(v) = value.downcast_ref::<bool>() {
        (*v).into()
    } else if let Some(v) = value.downcast_ref::<i32>() {
        (*v).into()
    } else if let Some(v) = value.downcast_ref::<i64>() {
        (*v).into()
    } else if let Some(v) = value.downcast_ref::<f64>() {
        (*v).into()
    } else if let Some(v) = value.downcast_ref::<String>() {
        serde_json::Value::String(v.clone())
    } else if let Some(v) = value.downcast_ref::<&str>() {
        serde_json::Value::String((*v).to_string())
    } else {
        serde_json::Value::Null
    }
}
