#![allow(dead_code, unused_imports, unused_variables)]

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::jsonrpc::baseproto::{Reader, Writer};
use crate::jsonrpc::jsonrpc::{Id, Message, ResponseError};

pub const ERR_CONN_CLOSED: &str = "ipc: connection closed";
pub const ERR_REQUEST_TIMEOUT: &str = "ipc: request timeout";

pub trait Handler {
    fn handle_request(&self, method: &str, params: &Value) -> Result<Value, ResponseError>;
    fn handle_notification(&self, method: &str, params: &Value) -> Result<(), String>;
}

pub fn unmarshal_params<T: serde::de::DeserializeOwned>(params: &Value) -> Result<Option<T>, String> { ::tsox_core::fntrace::enter("unmarshal_params"); 
    if params.is_null() {
        return Ok(None);
    }
    let v: T = serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
    Ok(Some(v))
}

pub struct JsonRpcProtocol<RW: Read + Write> {
    reader: Reader<RW>,
    writer: Writer<RW>,
}

impl<RW: Read + Write + Clone> JsonRpcProtocol<RW> {
    pub fn new(rw: RW) -> JsonRpcProtocol<RW> { ::tsox_core::fntrace::enter("new"); 
        JsonRpcProtocol {
            reader: Reader::new(rw.clone()),
            writer: Writer::new(rw),
        }
    }
}

impl<RW: Read + Write> JsonRpcProtocol<RW> {
    pub fn read_message(&mut self) -> Result<Message, String> { ::tsox_core::fntrace::enter("read_message"); 
        let data = self.reader.read().map_err(|e| e.to_string())?;
        serde_json::from_slice(&data).map_err(|e| e.to_string())
    }

    pub fn write_request(
        &mut self,
        id: &Id,
        method: &str,
        params: &Value,
    ) -> Result<(), String> { ::tsox_core::fntrace::enter("write_request"); 
        let msg = Message {
            jsonrpc: Default::default(),
            id: Some(id.clone()),
            method: method.to_string(),
            params: Some(params.clone()),
            result: None,
            error: None,
        };
        self.write_message(&msg)
    }

    pub fn write_notification(&mut self, method: &str, params: &Value) -> Result<(), String> { ::tsox_core::fntrace::enter("write_notification"); 
        let msg = Message {
            jsonrpc: Default::default(),
            id: None,
            method: method.to_string(),
            params: Some(params.clone()),
            result: None,
            error: None,
        };
        self.write_message(&msg)
    }

    pub fn write_response(&mut self, id: &Id, result: &Value) -> Result<(), String> { ::tsox_core::fntrace::enter("write_response"); 
        let result = if result.is_null() {
            Value::Null
        } else {
            result.clone()
        };
        let msg = Message {
            jsonrpc: Default::default(),
            id: Some(id.clone()),
            method: String::new(),
            params: None,
            result: Some(result),
            error: None,
        };
        self.write_message(&msg)
    }

    pub fn write_error(&mut self, id: &Id, resp_err: &ResponseError) -> Result<(), String> { ::tsox_core::fntrace::enter("write_error"); 
        let msg = Message {
            jsonrpc: Default::default(),
            id: Some(id.clone()),
            method: String::new(),
            params: None,
            result: None,
            error: Some(resp_err.clone()),
        };
        self.write_message(&msg)
    }

    fn write_message(&mut self, msg: &Message) -> Result<(), String> { ::tsox_core::fntrace::enter("write_message"); 
        let data = serde_json::to_vec(msg).map_err(|e| e.to_string())?;
        self.writer.write(&data).map_err(|e| e.to_string())
    }
}

pub const METHOD_GET_SERVER_TIMING: &str = "getServerTiming";
pub const METHOD_RESET_SERVER_TIMING: &str = "resetServerTiming";
pub const SERVER_RECENT_REQUEST_CAPACITY: usize = 5;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerRequestTiming {
    pub method: String,
    pub processing_time_ms: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ServerTimingTotals {
    pub request_count: u64,
    pub total_processing_time_ms: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerTimingInfo {
    pub enabled: bool,
    pub totals: ServerTimingTotals,
    pub recent_requests: Vec<ServerRequestTiming>,
}

#[derive(Debug, Default)]
pub struct TimingCollector {
    totals: ServerTimingTotals,
    ring: Vec<ServerRequestTiming>,
    head: usize,
}

pub fn new_timing_collector() -> TimingCollector { ::tsox_core::fntrace::enter("new_timing_collector"); 
    TimingCollector::default()
}

pub fn duration_to_millis(d: Duration) -> f64 { ::tsox_core::fntrace::enter("duration_to_millis"); 
    if d.is_zero() || d.as_nanos() == 0 {
        return 0.0;
    }
    (d.as_nanos() as f64 / 1_000_000.0).max(0.0)
}

impl TimingCollector {
    pub fn record(&mut self, method: &str, d: Duration) { ::tsox_core::fntrace::enter("record"); 
        let processing_ms = duration_to_millis(d);

        self.totals.request_count += 1;
        self.totals.total_processing_time_ms += processing_ms;

        let entry = ServerRequestTiming {
            method: method.to_string(),
            processing_time_ms: processing_ms,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|e| e.as_millis() as i64)
                .unwrap_or(0),
        };
        if self.ring.len() < SERVER_RECENT_REQUEST_CAPACITY {
            self.ring.push(entry);
        } else {
            self.ring[self.head] = entry;
            self.head = (self.head + 1) % SERVER_RECENT_REQUEST_CAPACITY;
        }
    }

    pub fn snapshot(&self) -> ServerTimingInfo { ::tsox_core::fntrace::enter("snapshot"); 
        let n = self.ring.len();
        let recent: Vec<ServerRequestTiming> = (0..n)
            .map(|i| self.ring[(self.head + i) % n.max(1)].clone())
            .collect();
        ServerTimingInfo {
            enabled: true,
            totals: self.totals.clone(),
            recent_requests: recent,
        }
    }

    pub fn reset(&mut self) { ::tsox_core::fntrace::enter("reset"); 
        self.totals = ServerTimingTotals::default();
        self.ring = Vec::new();
        self.head = 0;
    }
}

pub fn server_timing_snapshot(collector: Option<&TimingCollector>) -> ServerTimingInfo { ::tsox_core::fntrace::enter("server_timing_snapshot"); 
    match collector {
        None => disabled_server_timing_info(),
        Some(c) => c.snapshot(),
    }
}

pub fn disabled_server_timing_info() -> ServerTimingInfo { ::tsox_core::fntrace::enter("disabled_server_timing_info"); 
    ServerTimingInfo {
        enabled: false,
        totals: ServerTimingTotals::default(),
        recent_requests: Vec::new(),
    }
}
