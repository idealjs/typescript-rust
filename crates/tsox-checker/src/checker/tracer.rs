pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::Mutex;
pub(crate) use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub name: String,
    pub timestamp: u64,
    pub duration: u64,
    pub args: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TypeRecordingEntry {
    pub type_id: u32,
    pub type_flag_names: Vec<String>,
    pub constructor_name: String,
}

pub struct Tracer {
    start: Instant,
    events: Mutex<Vec<TraceEvent>>,
    type_recordings: Mutex<Vec<TypeRecordingEntry>>,
    enabled: bool,
}

impl Tracer {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            start: Instant::now(),
            events: Mutex::new(Vec::new()),
            type_recordings: Mutex::new(Vec::new()),
            enabled: false,
        }
    }

    pub fn enabled() -> Self { ::tsox_core::fntrace::enter("enabled"); 
        Self {
            start: Instant::now(),
            events: Mutex::new(Vec::new()),
            type_recordings: Mutex::new(Vec::new()),
            enabled: true,
        }
    }

    pub fn is_enabled(&self) -> bool { ::tsox_core::fntrace::enter("is_enabled"); 
        self.enabled
    }

    pub fn start(&self, name: &str) -> TraceSpan<'_> { ::tsox_core::fntrace::enter("start"); 
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

    pub fn record_type(&self, type_id: u32, flag_names: Vec<String>, constructor_name: &str) { ::tsox_core::fntrace::enter("record_type"); 
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

    fn elapsed_us(&self) -> u64 { ::tsox_core::fntrace::enter("elapsed_us"); 
        self.start.elapsed().as_micros() as u64
    }

    pub fn events(&self) -> Vec<TraceEvent> { ::tsox_core::fntrace::enter("events"); 
        self.events.lock().unwrap().clone()
    }

    pub fn type_recordings(&self) -> Vec<TypeRecordingEntry> { ::tsox_core::fntrace::enter("type_recordings"); 
        self.type_recordings.lock().unwrap().clone()
    }
}

impl Default for Tracer {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}

pub struct TraceSpan<'a> {
    tracer: Option<&'a Tracer>,
    name: String,
    start: Instant,
}

impl<'a> TraceSpan<'a> {
    fn disabled() -> Self { ::tsox_core::fntrace::enter("disabled"); 
        Self {
            tracer: None,
            name: String::new(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for TraceSpan<'a> {
    fn drop(&mut self) { ::tsox_core::fntrace::enter("drop"); 
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
