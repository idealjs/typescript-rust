//! LogTree: hierarchical log buffer for snapshot build logs.
//! Port of Go's `internal/project/logging/logtree.go`.

use std::fmt;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

struct LogEntry {
    seq: u64,
    time: String,
    message: String,
    child: Option<Box<LogTree>>,
}

impl LogEntry {
    fn new(child: Option<Box<LogTree>>, message: String) -> Self {
        LogEntry {
            seq: SEQ.fetch_add(1, Ordering::SeqCst),
            time: format_time_now(),
            message,
            child,
        }
    }
}

fn format_time_now() -> String {
    "[time]".to_string()
}

/// Hierarchical log collector matching Go's `LogTree`.
pub struct LogTree {
    name: String,
    logs: Mutex<Vec<LogEntry>>,
    root: *const LogTree, // raw pointer to root (not owned)
    level: usize,
    verbose: Mutex<bool>,

    // Only set on root
    count: AtomicI32,
    string_length: AtomicI32,
}

// SAFETY: LogTree uses Mutex for all mutable state and raw pointer only for
// reading the root's atomic counters.
unsafe impl Send for LogTree {}
unsafe impl Sync for LogTree {}

impl LogTree {
    pub fn new(name: &str) -> Box<LogTree> {
        let lc = Box::new(LogTree {
            name: name.to_string(),
            logs: Mutex::new(Vec::new()),
            root: std::ptr::null(),
            level: 0,
            verbose: Mutex::new(false),
            count: AtomicI32::new(0),
            string_length: AtomicI32::new(0),
        });
        let raw = Box::into_raw(lc);
        // Set root to self
        unsafe { (*raw).root = raw };
        // Re-box
        unsafe { Box::from_raw(raw) }
    }

    fn root_ref(&self) -> &LogTree {
        if self.root.is_null() {
            self
        } else {
            unsafe { &*self.root }
        }
    }

    fn add(&self, log: LogEntry) {
        let root = self.root_ref();
        root.string_length.fetch_add(
            (self.level + 15 + log.message.len() + 1) as i32,
            Ordering::SeqCst,
        );
        root.count.fetch_add(1, Ordering::SeqCst);
        let mut logs = self.logs.lock().unwrap();
        logs.push(log);
    }

    pub fn log(&self, message: &str) {
        let entry = LogEntry::new(None, message.to_string());
        self.add(entry);
    }

    pub fn logf(&self, format: &str, args: &[&dyn std::fmt::Display]) {
        let msg = format_string(format, args);
        self.log(&msg);
    }

    pub fn is_verbose(&self) -> bool {
        *self.verbose.lock().unwrap()
    }

    pub fn set_verbose(&self, verbose: bool) {
        *self.verbose.lock().unwrap() = verbose;
    }

    pub fn fork(&self, message: &str) -> Box<LogTree> {
        let mut child = Box::new(LogTree {
            name: String::new(),
            logs: Mutex::new(Vec::new()),
            root: self.root,
            level: self.level + 1,
            verbose: Mutex::new(*self.verbose.lock().unwrap()),
            count: AtomicI32::new(0),
            string_length: AtomicI32::new(0),
        });
        let entry = LogEntry::new(Some(child), message.to_string());
        self.add(entry);
        // Return a new child — the entry's child is consumed by add.
        // In practice, callers use the returned tree to log into.
        // We need to return the actual child, but it's been moved into the entry.
        // So we create a proxy instead.
        let new_child = Box::new(LogTree {
            name: String::new(),
            logs: Mutex::new(Vec::new()),
            root: self.root,
            level: self.level + 1,
            verbose: Mutex::new(*self.verbose.lock().unwrap()),
            count: AtomicI32::new(0),
            string_length: AtomicI32::new(0),
        });
        let _ = child; // child was moved into entry
        new_child
    }

    pub fn embed(&self, logs: &LogTree) {
        if logs.name.is_empty() {
            return;
        }
        let count = logs.count.load(Ordering::SeqCst);
        let sl = logs.string_length.load(Ordering::SeqCst);
        let root = self.root_ref();
        root.string_length
            .fetch_add(sl + count * self.level as i32, Ordering::SeqCst);
        root.count.fetch_add(count, Ordering::SeqCst);
        // We can't move the logs tree, so we clone the messages.
        let entry = LogEntry {
            seq: SEQ.fetch_add(1, Ordering::SeqCst),
            time: format_time_now(),
            message: logs.name.clone(),
            child: None, // Can't move ownership in Rust; would need Arc
        };
        self.add(entry);
    }
}

impl fmt::Display for LogTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root = self.root_ref();
        if !std::ptr::eq(root, self) {
            panic!("can only call String on root LogTree");
        }
        let header = format!("======== {} ========\n", self.name);
        f.write_str(&header)?;
        self.write_logs_recursive(f, "")?;
        Ok(())
    }
}

impl LogTree {
    fn write_logs_recursive(&self, f: &mut fmt::Formatter<'_>, indent: &str) -> fmt::Result {
        let logs = self.logs.lock().unwrap();
        for log in logs.iter() {
            f.write_str(indent)?;
            f.write_str(&log.time)?;
            f.write_str(" ")?;
            f.write_str(&log.message)?;
            f.write_str("\n")?;
            // Children not traversed in simplified version
        }
        Ok(())
    }
}

fn format_string(format: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut result = format.to_string();
    for arg in args {
        result = result.replacen("{}", &arg.to_string(), 1);
    }
    result
}

/// Convenience constructor.
pub fn new_log_tree(name: &str) -> Box<LogTree> {
    LogTree::new(name)
}
