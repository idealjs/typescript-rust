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
    pub fn new(code: i32, message: String) -> Self { ::tsox_core::fntrace::enter("new"); 
        RequestError { code, message }
    }

    pub fn method_not_found(method: &str) -> Self { ::tsox_core::fntrace::enter("method_not_found"); 
        RequestError::new(-32601, format!("Method not found: {}", method))
    }

    pub fn invalid_params(message: &str) -> Self { ::tsox_core::fntrace::enter("invalid_params"); 
        RequestError::new(-32602, message.to_string())
    }

    pub fn internal_error(message: &str) -> Self { ::tsox_core::fntrace::enter("internal_error"); 
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
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
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

    pub fn mark_init_started(&self) { ::tsox_core::fntrace::enter("mark_init_started"); 
        self.init_started.store(true, Ordering::SeqCst);
        self.logger.mark_init_started();
    }

    pub fn is_init_started(&self) -> bool { ::tsox_core::fntrace::enter("is_init_started"); 
        self.init_started.load(Ordering::SeqCst)
    }

    pub fn run_outgoing_loop(&self, writer: &mut dyn Write) -> io::Result<()> { ::tsox_core::fntrace::enter("run_outgoing_loop"); 
        while let Some(msg) = self.outgoing_queue.get() {
            let body = serde_json::to_string(&msg)?;
            write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
            writer.flush()?;
        }
        Ok(())
    }

    pub fn handle_initialize(&self, _params: &Value) -> Value { ::tsox_core::fntrace::enter("handle_initialize"); 
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

    pub fn handle_shutdown(&self) -> Value { ::tsox_core::fntrace::enter("handle_shutdown"); 
        self.shutdown_requested.store(true, Ordering::SeqCst);
        Value::Null
    }

    pub fn handle_initialized(&self) { ::tsox_core::fntrace::enter("handle_initialized"); }

    pub fn handle_exit(&self) -> i32 { ::tsox_core::fntrace::enter("handle_exit"); 
        if self.shutdown_requested.load(Ordering::SeqCst) {
            0
        } else {
            1
        }
    }

    pub fn send_notification(&self, method: &str, params: &Value) { ::tsox_core::fntrace::enter("send_notification"); 
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        let _ = self.outgoing_queue.put(msg);
    }

    pub fn send_client_request(&self, method: &str, params: &Value) { ::tsox_core::fntrace::enter("send_client_request"); 
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
    ) -> Option<LanguageService> { ::tsox_core::fntrace::enter("language_service_for_documents"); 
        if documents.is_empty() {
            return None;
        }

        let fs = Arc::new(tsox_tsoptions::vfs::InMemoryFS::new());
        fs.insert_dir("/");
        let mut file_names = Vec::with_capacity(documents.len());
        for (uri, content) in documents {
            let path = uri.strip_prefix("file://").unwrap_or(uri);
            fs.insert_file(path, content);
            file_names.push(path.to_string());
        }

        let host = tsox_compile::compiler::CompilerHostImpl::new(
            fs,
            "/".to_string(),
            tsox_checker::bundled::lib_path(),
        );
        let host: Arc<dyn tsox_compile::compiler::CompilerHost> = Arc::new(host);

        let mut config = tsox_tsoptions::tsoptions::ParsedCommandLine::default();
        config.file_names = file_names;
        config.compiler_options.no_lib = tsox_core::core::tristate::Tristate::True;

        let program = tsox_compile::compiler::Program::new(
            tsox_compile::compiler::ProgramOptions {
                config,
                host,
                use_source_of_project_reference: false,
                single_threaded: tsox_core::core::tristate::Tristate::Unknown,
                create_checker_pool: None,
                typings_location: String::new(),
                project_name: String::new(),
                tracing: None,
                skip_module_resolution: false,
            },
        );

        let ls_host = Box::new(InMemoryLsHost::default());
        Some(LanguageService::new(
            tsox_core::tspath::Path("/".to_string()),
            program,
            ls_host,
            "",
        ))
    }
}

impl Default for Server {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}

#[derive(Default)]
pub(crate) struct InMemoryLsHost {
    pub(crate) case_sensitive: bool,
}
