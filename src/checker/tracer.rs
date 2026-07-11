//! Tracing infrastructure for the checker.
//!
//! Ported from `internal/checker/tracer.go`. Provides timing and type
//! recording for `--generateTrace` support. This is a minimal port that
//! provides the API surface; full tracing logic will be added later.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// A traced event with a timestamp and optional data.
#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub name: String,
    pub timestamp: u64, // microseconds since tracer start
    pub duration: u64,  // microseconds
    pub args: HashMap<String, String>,
}

/// A type recording entry for `--generateTrace`.
#[derive(Debug, Clone)]
pub struct TypeRecordingEntry {
    pub type_id: u32,
    pub type_flag_names: Vec<String>,
    pub constructor_name: String,
}

/// The tracer for checker events.
pub struct Tracer {
    start: Instant,
    events: Mutex<Vec<TraceEvent>>,
    type_recordings: Mutex<Vec<TypeRecordingEntry>>,
    enabled: bool,
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            events: Mutex::new(Vec::new()),
            type_recordings: Mutex::new(Vec::new()),
            enabled: false,
        }
    }

    pub fn enabled() -> Self {
        Self {
            start: Instant::now(),
            events: Mutex::new(Vec::new()),
            type_recordings: Mutex::new(Vec::new()),
            enabled: true,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start a named span. Returns a guard that records the event when dropped.
    pub fn start(&self, name: &str) -> TraceSpan<'_> {
        if self.enabled {
            TraceSpan {
                tracer: Some(self),
                name: name.to_string(),
                start: Instant::now(),
            }
        } else {
            TraceSpan::disabled()
        }
    }

    /// Record a type for `--generateTrace`.
    pub fn record_type(&self, type_id: u32, flag_names: Vec<String>, constructor_name: &str) {
        if !self.enabled {
            return;
        }
        self.type_recordings
            .lock()
            .unwrap()
            .push(TypeRecordingEntry {
                type_id,
                type_flag_names: flag_names,
                constructor_name: constructor_name.to_string(),
            });
    }

    /// Get elapsed time in microseconds since tracer start.
    fn elapsed_us(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    /// Get all recorded events.
    pub fn events(&self) -> Vec<TraceEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Get all type recordings.
    pub fn type_recordings(&self) -> Vec<TypeRecordingEntry> {
        self.type_recordings.lock().unwrap().clone()
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

/// A traced span. Records its duration when dropped.
pub struct TraceSpan<'a> {
    tracer: Option<&'a Tracer>,
    name: String,
    start: Instant,
}

impl<'a> TraceSpan<'a> {
    fn disabled() -> Self {
        Self {
            tracer: None,
            name: String::new(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for TraceSpan<'a> {
    fn drop(&mut self) {
        if let Some(tracer) = self.tracer {
            let duration = self.start.elapsed().as_micros() as u64;
            let timestamp = tracer.elapsed_us() - duration;
            tracer.events.lock().unwrap().push(TraceEvent {
                name: self.name.clone(),
                timestamp,
                duration,
                args: HashMap::new(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn tracer_basic() {
        let tracer = Tracer::enabled();
        assert!(tracer.is_enabled());

        {
            let _span = tracer.start("test");
            std::thread::sleep(Duration::from_micros(100));
        }

        let events = tracer.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name, "test");
        assert!(events[0].duration >= 50); // at least some time passed
    }

    #[test]
    fn tracer_disabled() {
        let tracer = Tracer::new();
        assert!(!tracer.is_enabled());

        {
            let _span = tracer.start("test");
        }

        let events = tracer.events();
        assert!(events.is_empty());
    }

    #[test]
    fn tracer_record_type() {
        let tracer = Tracer::enabled();
        tracer.record_type(1, vec!["String".to_string()], "IntrinsicType");
        tracer.record_type(2, vec!["Number".to_string()], "IntrinsicType");

        let recordings = tracer.type_recordings();
        assert_eq!(recordings.len(), 2);
        assert_eq!(recordings[0].type_id, 1);
        assert_eq!(recordings[1].type_id, 2);
    }
}
