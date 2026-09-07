use super::events::{TraceArg, TraceEvent};
use std::collections::HashMap;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum ThreadKey {
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

pub(super) fn resolve_thread_id(
    events: &mut Vec<TraceEvent>,
    thread_ids: &mut HashMap<ThreadKey, usize>,
    args: &[(String, TraceArg)],
) -> usize {
    let key = match thread_key_from_args(args) {
        Some(k) => k,
        None => return MAIN_THREAD_ID,
    };
    if let Some(&tid) = thread_ids.get(&key) {
        return tid;
    }

    let mut tid = key.default_thread_id();
    while thread_ids.values().any(|&existing| existing == tid) {
        tid += 1;
    }
    thread_ids.insert(key.clone(), tid);
    events.push(TraceEvent {
        tid,
        ph: "M",
        cat: "__metadata",
        name: "thread_name".to_string(),
        args: vec![("name".to_string(), TraceArg::Str(key.display_name()))],
    });
    tid
}
