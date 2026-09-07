#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use serde_json::{Value, json};

use crate::ls::language_service::LanguageService;
use crate::project::session::Session;

use crate::lsp::dynamic_queue::DynamicQueue;
use crate::lsp::logger::Logger;

pub trait RequestHandler: Send + Sync {
    fn handle(&self, method: &str, params: &Value) -> Result<Value, RequestError>;
}

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

    pub fn mark_init_started(&self) {
        self.init_started.store(true, Ordering::SeqCst);
        self.logger.mark_init_started();
    }

    pub fn is_init_started(&self) -> bool {
        self.init_started.load(Ordering::SeqCst)
    }

    pub fn run_outgoing_loop(&self, writer: &mut dyn Write) -> io::Result<()> {
        while let Some(msg) = self.outgoing_queue.get() {
            let body = serde_json::to_string(&msg)?;
            write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
            writer.flush()?;
        }
        Ok(())
    }

    pub fn handle_initialize(&self, _params: &Value) -> Value {
        self.mark_init_started();

        let position_encoding = "utf-8";

        let capabilities = json!({
            "positionEncoding": position_encoding,
            "textDocumentSync": {
                "openClose": true,
                "change": 2,
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

    pub fn handle_shutdown(&self) -> Value {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        Value::Null
    }

    pub fn handle_initialized(&self) {}

    pub fn handle_exit(&self) -> i32 {
        if self.shutdown_requested.load(Ordering::SeqCst) {
            0
        } else {
            1
        }
    }

    pub fn send_notification(&self, method: &str, params: &Value) {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        let _ = self.outgoing_queue.put(msg);
    }

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

#[derive(Default)]
pub(crate) struct InMemoryLsHost {
    pub(crate) case_sensitive: bool,
}
