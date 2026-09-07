use super::lsp_server::LspServer;
use super::utils::*;

use serde_json::{Value, json};
use std::collections::HashMap;

impl LspServer {
    pub(super) fn handle_initialize(&mut self, params: &Value) -> Value {
        if let Some(root_uri) = params.get("rootUri").and_then(|v| v.as_str()) {
            if let Some(path) = root_uri.strip_prefix("file://") {
                self.workspace_root = Some(path.to_string());
            }
        } else if let Some(root_path) = params.get("rootPath").and_then(|v| v.as_str()) {
            self.workspace_root = Some(root_path.to_string());
        }

        json!({
            "capabilities": {
                "textDocumentSync": 1,
                "hoverProvider": true,
                "definitionProvider": true,
                "typeDefinitionProvider": true,
                "referencesProvider": true,
                "implementationProvider": true,
                "documentSymbolProvider": true,
                "workspaceSymbolProvider": true,
                "renameProvider": {
                    "prepareProvider": true
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
                "foldingRangeProvider": true,
                "selectionRangeProvider": true,
                "documentHighlightProvider": true,
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
                "linkedEditingRangeProvider": true,
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
                "diagnosticProvider": {
                    "interFileDependencies": true,
                    "workspaceDiagnostics": false
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
            },
            "serverInfo": {
                "name": "tsox",
                "version": "0.1.0"
            }
        })
    }

    pub(super) fn handle_did_open(&mut self, params: &Value) -> Vec<Value> {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            let text = td.get("text").and_then(|v| v.as_str()).unwrap_or("");
            self.documents.insert(uri.to_string(), text.to_string());
        }

        self.compute_all_diagnostics()
    }

    pub(super) fn handle_did_change(&mut self, params: &Value) -> Vec<Value> {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");

            if let Some(changes) = params.get("contentChanges").and_then(|v| v.as_array()) {
                if let Some(last) = changes.last() {
                    if let Some(text) = last.get("text").and_then(|v| v.as_str()) {
                        self.documents.insert(uri.to_string(), text.to_string());
                    }
                }
            }
        }

        self.compute_all_diagnostics()
    }

    pub(super) fn handle_did_close(&mut self, params: &Value) -> Vec<Value> {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            self.documents.remove(uri);

            return vec![json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": {
                    "uri": uri,
                    "diagnostics": []
                }
            })];
        }
        Vec::new()
    }

    pub(super) fn handle_did_change_watched_files(&mut self, params: &Value) -> Vec<Value> {
        if let Some(changes) = params.get("changes").and_then(|v| v.as_array()) {
            for change in changes {
                let uri = change.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                let typ = change.get("type").and_then(|v| v.as_u64()).unwrap_or(0);
                if uri.is_empty() {
                    continue;
                }
                match typ {
                    1 | 2 => {
                        let path = uri.strip_prefix("file://").unwrap_or(uri);
                        if let Ok(text) = std::fs::read_to_string(path) {
                            self.documents.insert(uri.to_string(), text);
                        }
                    }
                    3 => {
                        self.documents.remove(uri);
                    }
                    _ => {}
                }
            }
        }
        self.compute_all_diagnostics()
    }

    pub(super) fn compute_all_diagnostics(&self) -> Vec<Value> {
        let program = match build_program_from_documents(&self.documents) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut all_diags: Vec<crate::ast::diagnostic::Diagnostic> = Vec::new();
        for d in program.get_diagnostics_to_report() {
            all_diags.push((*d).clone());
        }
        all_diags.extend(program.get_semantic_diagnostics());

        let mut by_path: HashMap<&str, Vec<&crate::ast::diagnostic::Diagnostic>> = HashMap::new();
        let mut fileless: Vec<&crate::ast::diagnostic::Diagnostic> = Vec::new();
        for d in &all_diags {
            match &d.file {
                Some(f) => by_path.entry(f.file_name.as_str()).or_default().push(d),
                None => fileless.push(d),
            }
        }

        let single_uri = if self.documents.len() == 1 {
            self.documents.keys().next().cloned()
        } else {
            None
        };

        let mut notifications = Vec::with_capacity(self.documents.len());
        for uri in self.documents.keys() {
            let path = uri.strip_prefix("file://").unwrap_or(uri);
            let content = self.documents.get(uri).map(String::as_str).unwrap_or("");
            let mut lsp_diags: Vec<Value> = Vec::new();
            if let Some(list) = by_path.get(path) {
                for d in list {
                    lsp_diags.push(diagnostic_to_lsp(d, content));
                }
            }
            if single_uri.as_deref() == Some(uri.as_str()) {
                for d in &fileless {
                    lsp_diags.push(diagnostic_to_lsp(d, content));
                }
            }
            notifications.push(json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": {
                    "uri": uri,
                    "diagnostics": lsp_diags
                }
            }));
        }
        notifications
    }
}
