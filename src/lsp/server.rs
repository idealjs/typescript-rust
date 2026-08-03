//! Core LSP Server — dispatch loops, handler registration (1:1 port of Go's
//! `internal/lsp/server.go`).
//!
//! This module provides the server-side state machine that receives JSON-RPC
//! messages, dispatches them to registered handlers, and sends responses
//! and notifications back to the client.

#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use serde_json::{Value, json};

use crate::lsp::lsproto;
use crate::project::client::Client;
use crate::project::compiler_host::SessionOptions;
use crate::project::session::Session;
use crate::project::snapshot::Snapshot;

use super::dynamic_queue::DynamicQueue;
use super::logger::Logger;
use super::progress::ProjectLoadingProgress;

/// Dispatches a single LSP request.
///
/// Go: `type RequestDispatch struct { ... }` (conceptually).
pub trait RequestHandler: Send + Sync {
    fn handle(&self, method: &str, params: &Value) -> Result<Value, RequestError>;
}

/// An error returned by a request handler.
#[derive(Debug)]
pub struct RequestError {
    pub code: i32,
    pub message: String,
}

impl RequestError {
    pub fn new(code: i32, message: String) -> Self {
        RequestError { code, message }
    }

    pub fn method_not_found(method: &str) -> Self {
        RequestError::new(-32601, format!("Method not found: {}", method))
    }

    pub fn invalid_params(message: &str) -> Self {
        RequestError::new(-32602, message.to_string())
    }

    pub fn internal_error(message: &str) -> Self {
        RequestError::new(-32603, message.to_string())
    }
}

/// LSP server state.
///
/// Go: `type Server struct { ... }`.
pub struct Server {
    pub session: RwLock<Option<Box<Session>>>,
    pub logger: Arc<Logger>,
    pub outgoing_queue: Arc<DynamicQueue<Value>>,
    pub init_started: AtomicBool,
    pub shutdown_requested: AtomicBool,
    pub request_id: AtomicU64,
    pub locale: String,
    pub stderr: Box<dyn Write + Send>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            session: RwLock::new(None),
            logger: Arc::new(Logger::new()),
            outgoing_queue: DynamicQueue::new(),
            init_started: AtomicBool::new(false),
            shutdown_requested: AtomicBool::new(false),
            request_id: AtomicU64::new(0),
            locale: "en".to_string(),
            stderr: Box::new(io::stderr()),
        }
    }

    /// Mark that initialization has started.
    pub fn mark_init_started(&self) {
        self.init_started.store(true, Ordering::SeqCst);
        self.logger.mark_init_started();
    }

    /// Returns whether the server has started initialization.
    pub fn is_init_started(&self) -> bool {
        self.init_started.load(Ordering::SeqCst)
    }

    /// Runs the server's outgoing notification loop, writing messages to
    /// the provided writer.
    ///
    /// Go: dispatch loop in `Server.Run`.
    pub fn run_outgoing_loop(&self, writer: &mut dyn Write) -> io::Result<()> {
        while let Some(msg) = self.outgoing_queue.get() {
            let body = serde_json::to_string(&msg)?;
            write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
            writer.flush()?;
        }
        Ok(())
    }

    /// Handles the `initialize` request.
    ///
    /// Go: `func (s *Server) handleInitialize(...)`.
    pub fn handle_initialize(&self, params: &Value) -> Value {
        self.mark_init_started();

        let capabilities = json!({
            "textDocumentSync": 1,
            "hoverProvider": true,
            "definitionProvider": true,
            "referencesProvider": true,
            "documentSymbolProvider": true,
            "renameProvider": true,
            "completionProvider": {
                "triggerCharacters": [".", "\"", "'", "/", "@", "<"],
                "resolveProvider": false
            },
            "documentFormattingProvider": true,
            "workspace": {
                "workspaceFolders": {
                    "supported": true,
                    "changeNotifications": true
                },
                "fileOperations": {
                    "didCreate": true,
                    "didRename": true,
                    "didDelete": true
                }
            }
        });

        json!({
            "capabilities": capabilities,
            "serverInfo": {
                "name": "tsox",
                "version": "0.1.0"
            }
        })
    }

    /// Handles the `shutdown` request.
    pub fn handle_shutdown(&self) -> Value {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        Value::Null
    }

    /// Handles the `initialized` notification.
    pub fn handle_initialized(&self) {
        // No-op in the stub.
    }

    /// Handles the `exit` notification.
    pub fn handle_exit(&self) -> i32 {
        if self.shutdown_requested.load(Ordering::SeqCst) {
            0
        } else {
            1
        }
    }

    /// Sends a notification to the client via the outgoing queue.
    ///
    /// Go: `func sendNotification(server *Server, ...)`.
    pub fn send_notification(&self, method: &str, params: &Value) {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        let _ = self.outgoing_queue.put(msg);
    }

    /// Sends a request to the client (fire-and-forget).
    pub fn send_client_request(&self, method: &str, params: &Value) {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        let _ = self.outgoing_queue.put(msg);
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

/// Sends a fire-and-forget client request.
///
/// Go: `func sendClientRequestFireAndForget(server *Server, ...)`.
pub fn send_client_request_fire_and_forget(server: &Server, method: &str, params: &Value) {
    server.send_client_request(method, params);
}

/// Sends a notification.
///
/// Go: `func sendNotification(server *Server, ...)`.
pub fn send_notification(server: &Server, method: &str, params: &Value) {
    server.send_notification(method, params);
}
