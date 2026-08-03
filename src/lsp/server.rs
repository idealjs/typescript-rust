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

use crate::ls::host::{AutoImportRegistry, EcmaLineInfo, Host};
use crate::ls::language_service::LanguageService;
use crate::ls::lsconv::converters::{Converters, PositionEncodingKind};
use crate::ls::lsutil::new_default_user_preferences;
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
    ///
    /// Advertises the full set of capabilities backed by the `ls/`
    /// `LanguageService` providers (1:1 with the Go server's
    /// `ServerCapabilities`). Each capability here corresponds to a
    /// `provide_*` method on `LanguageService`.
    pub fn handle_initialize(&self, params: &Value) -> Value {
        self.mark_init_started();

        // Position encoding: prefer UTF-8, the rest of the LS converters
        // support UTF-16/UTF-32 as well.
        let position_encoding = "utf-8";

        let capabilities = json!({
            "positionEncoding": position_encoding,
            "textDocumentSync": {
                "openClose": true,
                "change": 2, // TextDocumentSyncKind.Incremental
                "save": true
            },
            "hoverProvider": true,
            "definitionProvider": true,
            "typeDefinitionProvider": true,
            "referencesProvider": true,
            "implementationProvider": true,
            "diagnosticProvider": {
                "identifier": "typescript",
                "interFileDependencies": true,
                "workspaceDiagnostics": false
            },
            "completionProvider": {
                "triggerCharacters": [".", "\"", "'", "/", "@", "<"],
                "resolveProvider": true
            },
            "signatureHelpProvider": {
                "triggerCharacters": ["(", ",", "<"],
                "retriggerCharacters": [")"]
            },
            "documentFormattingProvider": true,
            "documentRangeFormattingProvider": true,
            "documentOnTypeFormattingProvider": {
                "firstTriggerCharacter": "{",
                "moreTriggerCharacter": ["}", ";", "\n"]
            },
            "workspaceSymbolProvider": true,
            "documentSymbolProvider": true,
            "foldingRangeProvider": true,
            "renameProvider": {
                "prepareProvider": true
            },
            "documentHighlightProvider": true,
            "selectionRangeProvider": true,
            "linkedEditingRangeProvider": true,
            "inlayHintProvider": true,
            "codeLensProvider": {
                "resolveProvider": true
            },
            "codeActionProvider": {
                "codeActionKinds": [
                    "quickfix",
                    "source.organizeImports",
                    "source.removeUnusedImports",
                    "source.sortImports",
                    "source.fixAll"
                ]
            },
            "callHierarchyProvider": true,
            "semanticTokensProvider": {
                "legend": {
                    "tokenTypes": [
                        "namespace", "class", "enum", "interface", "struct",
                        "typeParameter", "type", "parameter", "variable",
                        "property", "enumMember", "decorator", "event",
                        "function", "method", "macro", "label", "comment",
                        "string", "keyword", "number", "regexp", "operator"
                    ],
                    "tokenModifiers": [
                        "declaration", "definition", "readonly", "static",
                        "deprecated", "abstract", "async", "modification",
                        "documentation", "defaultLibrary", "local"
                    ]
                },
                "full": true,
                "range": true
            },
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

    /// Build a `LanguageService` over the given open documents so that the
    /// dispatch handlers can delegate to the `ls/` `provide_*` providers.
    ///
    /// This is the integration hook between the LSP server and the
    /// language-service layer: it stages every open document in an in-memory
    /// file system, builds a `Program` (parse + bind + check), wraps it in an
    /// `InMemoryLsHost`, and returns a `LanguageService` ready to answer
    /// hover / definition / completion / references / rename / diagnostics /
    /// semantic-tokens / signature-help / code-action / inlay-hint / …
    /// requests.
    ///
    /// Returns `None` when no documents are open.
    ///
    /// Go: conceptually `Server.session.GetService(...)`.
    pub fn language_service_for_documents(
        &self,
        documents: &HashMap<String, String>,
    ) -> Option<LanguageService> {
        if documents.is_empty() {
            return None;
        }

        let fs = Arc::new(crate::vfs::InMemoryFS::new());
        fs.insert_dir("/");
        let mut file_names = Vec::with_capacity(documents.len());
        for (uri, content) in documents {
            let path = uri.strip_prefix("file://").unwrap_or(uri);
            fs.insert_file(path, content);
            file_names.push(path.to_string());
        }

        let host =
            crate::compiler::CompilerHostImpl::new(fs, "/".to_string(), crate::bundled::lib_path());
        let host: Arc<dyn crate::compiler::CompilerHost> = Arc::new(host);

        let mut config = crate::tsoptions::ParsedCommandLine::default();
        config.file_names = file_names;
        config.compiler_options.no_lib = crate::core::tristate::Tristate::True;

        let program = Arc::new(crate::compiler::Program::new(
            crate::compiler::ProgramOptions { config, host },
        ));

        let ls_host = Box::new(InMemoryLsHost::default());
        Some(LanguageService::new(
            crate::tspath::Path("/".to_string()),
            program,
            ls_host,
            "",
        ))
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

/// An in-memory `ls::Host` backed by the open-document set.
///
/// Used by [`Server::language_service_for_documents`] to construct a
/// [`LanguageService`] without a real project session. It provides default
/// converters (UTF-16) and user preferences; file-system queries are
/// best-effort since the `LanguageService` providers primarily rely on the
/// already-built `Program`.
#[derive(Default)]
struct InMemoryLsHost {
    case_sensitive: bool,
}

impl Host for InMemoryLsHost {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.case_sensitive
    }

    fn read_file(&self, _path: &str) -> Option<String> {
        // The program already holds the open-document contents; the host
        // read_file is only needed for out-of-band lookups.
        None
    }

    fn converters(&self) -> Converters {
        Converters::new(PositionEncodingKind::Utf16)
    }

    fn get_preferences(&self, _active_file: &str) -> crate::ls::lsutil::UserPreferences {
        new_default_user_preferences()
    }

    fn get_ecma_line_info(&self, _file_name: &str) -> Option<EcmaLineInfo> {
        None
    }

    fn auto_import_registry(&self) -> AutoImportRegistry {
        AutoImportRegistry
    }

    fn read_directory(
        &self,
        _current_dir: &str,
        _path: &str,
        _extensions: &[String],
        _excludes: &[String],
        _includes: &[String],
        _depth: i32,
    ) -> Vec<String> {
        Vec::new()
    }

    fn get_directories(&self, _path: &str) -> Vec<String> {
        Vec::new()
    }

    fn directory_exists(&self, _path: &str) -> bool {
        false
    }

    fn file_exists(&self, _path: &str) -> bool {
        false
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
