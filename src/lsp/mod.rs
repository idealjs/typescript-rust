//! Minimal LSP server over stdio JSON-RPC 2.0.
//!
//! Implements initialize/shutdown lifecycle, text document synchronization,
//! and basic diagnostics. Hover uses the checker's nodebuilder for type info.

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;

use serde_json::{Value, json};

/// LSP server state.
pub struct LspServer {
    documents: HashMap<String, String>, // uri -> content
    workspace_root: Option<String>,
    shutdown_requested: bool,
}

impl LspServer {
    pub fn new() -> Self {
        LspServer {
            documents: HashMap::new(),
            workspace_root: None,
            shutdown_requested: false,
        }
    }

    /// Run the LSP server loop over stdio.
    pub fn run(&mut self) -> i32 {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = stdout.lock();

        loop {
            match self.read_message(&mut reader) {
                Ok(Some(msg)) => {
                    let (response, notifications) = self.handle_message(&msg);
                    if let Some(resp) = response {
                        let _ = self.write_message(&mut writer, &resp);
                    }
                    for notif in &notifications {
                        let _ = self.write_message(&mut writer, notif);
                    }
                    if self.shutdown_requested
                        && msg.get("method").and_then(|m| m.as_str()) == Some("exit")
                    {
                        return 0;
                    }
                }
                Ok(None) => {
                    // EOF
                    return if self.shutdown_requested { 0 } else { 1 };
                }
                Err(e) => {
                    eprintln!("LSP read error: {e}");
                    return 1;
                }
            }
        }
    }

    /// Read a single JSON-RPC message with Content-Length framing.
    fn read_message<R: BufRead>(&self, reader: &mut R) -> io::Result<Option<Value>> {
        // Read headers
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                return Ok(None); // EOF
            }
            let trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
            if trimmed.is_empty() {
                break; // End of headers
            }
            if let Some(rest) = trimmed.strip_prefix("Content-Length: ") {
                content_length = rest.parse::<usize>().ok();
            }
        }

        let length = match content_length {
            Some(l) => l,
            None => return Ok(None),
        };

        // Read body
        let mut body = vec![0u8; length];
        reader.read_exact(&mut body)?;
        let msg: Value = serde_json::from_slice(&body)?;
        Ok(Some(msg))
    }

    /// Write a JSON-RPC message with Content-Length framing.
    fn write_message<W: Write>(&self, writer: &mut W, msg: &Value) -> io::Result<()> {
        let body = serde_json::to_string(msg)?;
        write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        writer.flush()
    }

    /// Handle a single message, returning an optional response and a list of
    /// notifications to send back to the client (e.g. `publishDiagnostics`).
    fn handle_message(&mut self, msg: &Value) -> (Option<Value>, Vec<Value>) {
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(Value::Null);

        match method {
            "initialize" => {
                let result = self.handle_initialize(&params);
                (Some(make_response(id, result)), Vec::new())
            }
            "initialized" => (None, Vec::new()),
            "shutdown" => {
                self.shutdown_requested = true;
                (Some(make_response(id, Value::Null)), Vec::new())
            }
            "exit" => (None, Vec::new()),
            "textDocument/didOpen" => {
                let notifications = self.handle_did_open(&params);
                (None, notifications)
            }
            "textDocument/didChange" => {
                let notifications = self.handle_did_change(&params);
                (None, notifications)
            }
            "textDocument/didClose" => {
                let notifications = self.handle_did_close(&params);
                (None, notifications)
            }
            "workspace/didChangeWatchedFiles" => {
                let notifications = self.handle_did_change_watched_files(&params);
                (None, notifications)
            }
            "textDocument/hover" => {
                let result = self.handle_hover(&params);
                (Some(make_response(id, result)), Vec::new())
            }
            "textDocument/definition" => {
                let result = self.handle_definition(&params);
                (Some(make_response(id, result)), Vec::new())
            }
            "textDocument/completion" => {
                let result = self.handle_completion(&params);
                (Some(make_response(id, result)), Vec::new())
            }
            "textDocument/references" => {
                let result = self.handle_references(&params);
                (Some(make_response(id, result)), Vec::new())
            }
            "textDocument/documentSymbol" => {
                let result = self.handle_document_symbol(&params);
                (Some(make_response(id, result)), Vec::new())
            }
            "textDocument/rename" => {
                let result = self.handle_rename(&params);
                (Some(make_response(id, result)), Vec::new())
            }
            "textDocument/formatting" => {
                // Register the capability so the client offers "Format
                // Document"; no edits are produced.
                (Some(make_response(id, json!([]))), Vec::new())
            }
            _ => {
                // Unknown method — return method not found error for requests
                if id.is_some() {
                    (
                        Some(make_error_response(
                            id,
                            -32601,
                            &format!("Method not found: {method}"),
                        )),
                        Vec::new(),
                    )
                } else {
                    (None, Vec::new())
                }
            }
        }
    }

    fn handle_initialize(&mut self, params: &Value) -> Value {
        // Extract workspace root
        if let Some(root_uri) = params.get("rootUri").and_then(|v| v.as_str()) {
            if let Some(path) = root_uri.strip_prefix("file://") {
                self.workspace_root = Some(path.to_string());
            }
        } else if let Some(root_path) = params.get("rootPath").and_then(|v| v.as_str()) {
            self.workspace_root = Some(root_path.to_string());
        }

        json!({
            "capabilities": {
                "textDocumentSync": 1, // Full sync
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

    fn handle_did_open(&mut self, params: &Value) -> Vec<Value> {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            let text = td.get("text").and_then(|v| v.as_str()).unwrap_or("");
            self.documents.insert(uri.to_string(), text.to_string());
        }
        // Recompute diagnostics across ALL open documents so that opening one
        // file can surface cross-file type errors in another.
        self.compute_all_diagnostics()
    }

    fn handle_did_change(&mut self, params: &Value) -> Vec<Value> {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            // Full sync: take the last change.
            if let Some(changes) = params.get("contentChanges").and_then(|v| v.as_array()) {
                if let Some(last) = changes.last() {
                    if let Some(text) = last.get("text").and_then(|v| v.as_str()) {
                        self.documents.insert(uri.to_string(), text.to_string());
                    }
                }
            }
        }
        // Recompute diagnostics across ALL open documents so that editing one
        // file updates cross-file type errors in every other open file.
        self.compute_all_diagnostics()
    }

    fn handle_did_close(&mut self, params: &Value) -> Vec<Value> {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            self.documents.remove(uri);
            // Publish empty diagnostics to clear any previously reported
            // errors for the now-closed document.
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

    fn handle_hover(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return Value::Null,
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = build_program(path, content);
        let Some(source_file) = program.get_source_file(path) else {
            return Value::Null;
        };

        // Convert LSP 0-based (line, character) to a byte offset.
        let line = params
            .get("position")
            .and_then(|p| p.get("line"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let character = params
            .get("position")
            .and_then(|p| p.get("character"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let offset = source_file
            .line_map
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(0) as usize
            + character;

        // Find the deepest AST node covering this offset and ask the checker
        // for its quick-info (hover) text.
        let node = find_deepest_node(&source_file.node, offset);
        let mut checker = program.build_checker();
        // Prefer structured `SymbolDisplayPart[]` (colorized hover) when a
        // symbol is available; fall back to the plain-text quick-info path
        // for nodes without a symbol (e.g. `this`, literals).
        let parts = checker.get_quick_info_display_parts(&node);
        let type_str = if parts.is_empty() {
            checker.get_quick_info_text(&node)
        } else {
            display_parts_to_string(&parts)
        };
        if type_str.is_empty() {
            return Value::Null;
        }

        json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```typescript\n{}\n```", type_str)
            }
        })
    }

    /// Run the compiler pipeline (parse + bind + check) over ALL open
    /// documents and return one `textDocument/publishDiagnostics` notification
    /// per open document. Building a single program from every open file
    /// enables cross-file type checking: changing file A may surface or clear
    /// diagnostics in file B. An empty diagnostic list is published for files
    /// with no errors, which also clears previously reported errors.
    fn compute_all_diagnostics(&self) -> Vec<Value> {
        let program = match build_program_from_documents(&self.documents) {
            Some(p) => p,
            None => return Vec::new(),
        };

        // Collect parse/bind diagnostics (from the program) plus semantic
        // diagnostics (from the checker).
        let mut all_diags: Vec<crate::ast::diagnostic::Diagnostic> = Vec::new();
        for d in program.get_diagnostics_to_report() {
            all_diags.push((*d).clone());
        }
        all_diags.extend(program.get_semantic_diagnostics());

        // Group diagnostics by the path of their attached source file.
        // Diagnostics without an attached file (rare, program-level) are
        // attributed to the sole open document when only one is open, which
        // preserves the prior single-file reporting behavior; otherwise they
        // are dropped to avoid duplication across files.
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

    /// Handle `workspace/didChangeWatchedFiles` — synchronize on-disk file
    /// changes into the open-document set and recompute diagnostics across all
    /// files. Change types follow the LSP `FileChangeType` enum: 1 = Created,
    /// 2 = Changed, 3 = Deleted.
    fn handle_did_change_watched_files(&mut self, params: &Value) -> Vec<Value> {
        if let Some(changes) = params.get("changes").and_then(|v| v.as_array()) {
            for change in changes {
                let uri = change.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                let typ = change.get("type").and_then(|v| v.as_u64()).unwrap_or(0);
                if uri.is_empty() {
                    continue;
                }
                match typ {
                    1 | 2 => {
                        // Created or Changed: read the file from disk.
                        let path = uri.strip_prefix("file://").unwrap_or(uri);
                        if let Ok(text) = std::fs::read_to_string(path) {
                            self.documents.insert(uri.to_string(), text);
                        }
                    }
                    3 => {
                        // Deleted: drop from the open-document set.
                        self.documents.remove(uri);
                    }
                    _ => {}
                }
            }
        }
        self.compute_all_diagnostics()
    }

    /// Handle `textDocument/definition` — find the declaration of the symbol
    /// under the cursor.
    fn handle_definition(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return json!([]),
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = build_program(path, content);
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return json!([]),
        };

        let line = params
            .get("position")
            .and_then(|p| p.get("line"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let character = params
            .get("position")
            .and_then(|p| p.get("character"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let line_map = crate::ast::LineMap::from_text(content);
        let offset = line_map.line_starts.get(line).copied().unwrap_or(0) as usize + character;

        let node = find_deepest_node(&source_file.node, offset);
        let mut checker = program.build_checker();

        // Resolve the symbol, then find its value declaration.
        let symbol = checker.resolve_identifier(&node).or_else(|| {
            let symbol_map = checker.program.symbol_map();
            let mut current: Option<&Arc<crate::ast::Node>> = Some(&node);
            while let Some(n) = current {
                if let Some(sym) = symbol_map.symbol_of(n) {
                    return Some(Arc::clone(sym));
                }
                current = n.parent.as_ref();
            }
            None
        });

        if let Some(symbol) = symbol {
            if let Some(decl) = symbol.value_declaration.as_ref() {
                let decl_file = checker
                    .program
                    .source_files()
                    .iter()
                    .find(|sf| decl.loc.pos() >= 0 && decl.loc.pos() < sf.text.len())
                    .cloned();

                if let Some(sf) = decl_file {
                    let (dl, dc) = crate::diagnosticwriter::line_and_character(
                        &sf.line_map,
                        &sf.text,
                        decl.loc.pos(),
                    );
                    let decl_uri = if sf.file_name.starts_with('/') {
                        format!("file://{}", sf.file_name)
                    } else {
                        format!("file:///{}", sf.file_name)
                    };
                    return json!([{
                        "uri": decl_uri,
                        "range": {
                            "start": {"line": dl, "character": dc},
                            "end": {"line": dl, "character": dc + decl.loc.len().max(1)}
                        }
                    }]);
                }
            }
        }

        json!([])
    }

    /// Handle `textDocument/completion` — return completion items for the
    /// identifier or member access under the cursor.
    fn handle_completion(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return Value::Null,
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = build_program(path, content);
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return Value::Null,
        };

        let line = params
            .get("position")
            .and_then(|p| p.get("line"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let character = params
            .get("position")
            .and_then(|p| p.get("character"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let line_map = crate::ast::LineMap::from_text(content);
        let offset = line_map.line_starts.get(line).copied().unwrap_or(0) as usize + character;

        let node = find_deepest_node(&source_file.node, offset);
        let mut checker = program.build_checker();

        // Collect symbols in scope at the cursor position.
        let mut items: Vec<Value> = Vec::new();

        // Add global symbols from the checker's globals table.
        for (name, sym) in checker.globals.iter() {
            if name.starts_with("__") {
                continue;
            }
            let kind = completion_item_kind(&sym.flags);
            items.push(json!({
                "label": name,
                "kind": kind
            }));
        }

        // Add keywords as completion items for empty/identifier contexts.
        if items.len() < 50 {
            for kw in &[
                "const",
                "let",
                "var",
                "function",
                "class",
                "interface",
                "type",
                "enum",
                "if",
                "else",
                "for",
                "while",
                "do",
                "switch",
                "case",
                "break",
                "continue",
                "return",
                "try",
                "catch",
                "finally",
                "throw",
                "new",
                "delete",
                "typeof",
                "instanceof",
                "in",
                "of",
                "import",
                "export",
                "from",
                "default",
                "async",
                "await",
                "public",
                "private",
                "protected",
                "readonly",
                "static",
                "abstract",
                "declare",
                "namespace",
                "module",
                "as",
                "is",
                "satisfies",
                "true",
                "false",
                "null",
                "undefined",
                "void",
                "this",
                "super",
                "extends",
                "implements",
                "get",
                "set",
            ] {
                items.push(json!({"label": kw, "kind": 14})); // 14 = Keyword
            }
        }

        if items.is_empty() {
            Value::Null
        } else {
            json!({
                "isIncomplete": true,
                "items": items
            })
        }
    }

    /// Handle `textDocument/references` — find all references to the symbol
    /// under the cursor, across all open documents.
    fn handle_references(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return json!([]),
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = match build_program_from_documents(&self.documents) {
            Some(p) => p,
            None => return json!([]),
        };
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return json!([]),
        };

        let offset = position_to_offset(params, content);
        let symbol_map = program.symbol_map();
        let target_symbol = match crate::astnav::get_token_at_position(&source_file.node, offset) {
            Some(node) => match resolve_symbol_for_node(symbol_map, &node) {
                Some(s) => s,
                None => return json!([]),
            },
            None => return json!([]),
        };

        let include_declaration = params
            .get("context")
            .and_then(|c| c.get("includeDeclaration"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let refs = find_all_references(&program, &target_symbol);
        let mut locations = Vec::with_capacity(refs.len());
        for (sf, node) in &refs {
            if !include_declaration && is_declaration_name(symbol_map, node) {
                continue;
            }
            let (sl, sc) =
                crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.pos());
            let (el, ec) =
                crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.end());
            locations.push(json!({
                "uri": path_to_uri(&sf.file_name),
                "range": {
                    "start": {"line": sl, "character": sc},
                    "end": {"line": el, "character": ec}
                }
            }));
        }
        json!(locations)
    }

    /// Handle `textDocument/documentSymbol` — return all top-level
    /// declarations in the document (functions, classes, interfaces,
    /// variables, etc.) as a hierarchical symbol tree.
    fn handle_document_symbol(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return json!([]),
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = build_program(path, content);
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return json!([]),
        };

        let statements = match &source_file.node.data {
            crate::ast::NodeData::SourceFile(data) => data.statements.nodes.as_slice(),
            _ => return json!([]),
        };
        json!(symbols_for_statements(statements, &source_file))
    }

    /// Handle `textDocument/rename` — rename the symbol under the cursor and
    /// all of its references, returning a `WorkspaceEdit` of text edits.
    fn handle_rename(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return json!({}),
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);
        let new_name = params
            .get("newName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if new_name.is_empty() {
            return json!({});
        }

        let program = match build_program_from_documents(&self.documents) {
            Some(p) => p,
            None => return json!({}),
        };
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return json!({}),
        };

        let offset = position_to_offset(params, content);
        let symbol_map = program.symbol_map();
        let target_symbol = match crate::astnav::get_token_at_position(&source_file.node, offset) {
            Some(node) => match resolve_symbol_for_node(symbol_map, &node) {
                Some(s) => s,
                None => return json!({}),
            },
            None => return json!({}),
        };

        let refs = find_all_references(&program, &target_symbol);
        let mut changes: HashMap<String, Vec<Value>> = HashMap::new();
        for (sf, node) in &refs {
            let (sl, sc) =
                crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.pos());
            let (el, ec) =
                crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.end());
            changes
                .entry(path_to_uri(&sf.file_name))
                .or_default()
                .push(json!({
                    "range": {
                        "start": {"line": sl, "character": sc},
                        "end": {"line": el, "character": ec}
                    },
                    "newText": new_name
                }));
        }
        json!({ "changes": changes })
    }
}

/// Stage `path`/`content` in an in-memory file system and build a program for
/// it, with default lib loading disabled for speed. The returned program is
/// ready for diagnostic and hover queries.
fn build_program(path: &str, content: &str) -> Arc<crate::compiler::Program> {
    let fs = Arc::new(crate::vfs::InMemoryFS::new());
    let parent = std::path::Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());
    fs.insert_dir(&parent);
    fs.insert_file(path, content);

    let host = crate::compiler::CompilerHostImpl::new(fs, parent, crate::bundled::lib_path());
    let host: Arc<dyn crate::compiler::CompilerHost> = Arc::new(host);

    let mut config = crate::tsoptions::ParsedCommandLine::default();
    config.file_names = vec![path.to_string()];
    config.compiler_options.no_lib = crate::core::tristate::Tristate::True;

    Arc::new(crate::compiler::Program::new(
        crate::compiler::ProgramOptions { config, host },
    ))
}

/// Convert structured `SymbolDisplayPart[]` into a plain string.
fn display_parts_to_string(parts: &[crate::checker::nodebuilder::SymbolDisplayPart]) -> String {
    parts.iter().map(|p| p.text.as_str()).collect()
}

/// Recursively descend into the deepest AST node whose source range covers
/// `offset`, starting from `node`. Used to locate the token/identifier under
/// the cursor for hover information.
fn find_deepest_node(node: &Arc<crate::ast::Node>, offset: usize) -> Arc<crate::ast::Node> {
    let mut deepest = Arc::clone(node);
    loop {
        let current = Arc::clone(&deepest);
        let mut next: Option<Arc<crate::ast::Node>> = None;
        crate::ast::for_each_child(&current, |child| {
            if child.loc.pos() <= offset && offset < child.loc.end() {
                next = Some(Arc::clone(child));
                true // stop at the first containing child (siblings don't overlap)
            } else {
                false
            }
        });
        match next {
            Some(child) => deepest = child,
            None => break,
        }
    }
    deepest
}

// ─────────────────────────────────────────────────────────────────────────
// Helpers for references / documentSymbol / rename
// ─────────────────────────────────────────────────────────────────────────

/// Extract the LSP 0-based (line, character) from request `params` and
/// convert it to a byte offset within `content`.
fn position_to_offset(params: &Value, content: &str) -> usize {
    let line = params
        .get("position")
        .and_then(|p| p.get("line"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let character = params
        .get("position")
        .and_then(|p| p.get("character"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let line_map = crate::ast::LineMap::from_text(content);
    line_map.line_starts.get(line).copied().unwrap_or(0) as usize + character
}

/// Build a program from all open documents so that cross-file reference
/// resolution works. Returns `None` when no documents are open.
fn build_program_from_documents(
    documents: &HashMap<String, String>,
) -> Option<Arc<crate::compiler::Program>> {
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

    Some(Arc::new(crate::compiler::Program::new(
        crate::compiler::ProgramOptions { config, host },
    )))
}

/// Resolve a reference identifier to its declared symbol by walking up the
/// AST parent chain and consulting the binder's locals tables and container
/// symbol member/export tables. Mirrors the scope-walk in the checker's
/// `resolve_identifier_with_meaning`, but operates on the persistent AST
/// structure (parent pointers are set by the binder) rather than the
/// checker's transient scope stack. Only identifiers are resolved.
fn resolve_identifier_symbol(
    symbol_map: &crate::ast::NodeSymbolMap,
    node: &Arc<crate::ast::Node>,
) -> Option<Arc<crate::ast::Symbol>> {
    use crate::ast::{NodeData, SymbolFlags};
    let name = match &node.data {
        NodeData::Identifier(data) => data.text.as_str(),
        _ => return None,
    };
    let mut current: Option<&Arc<crate::ast::Node>> = Some(node);
    while let Some(n) = current {
        if let Some(locals) = symbol_map.locals.get(&n.id()) {
            if let Some(sym) = locals.get(name) {
                return Some(Arc::clone(sym));
            }
        }
        if let Some(container_sym) = symbol_map.symbols.get(&n.id()) {
            if let Some(sym) = container_sym.members.get(name) {
                return Some(Arc::clone(sym));
            }
            if container_sym.flags.intersects(SymbolFlags::MODULE) {
                if let Some(sym) = container_sym.exports.get(name) {
                    return Some(Arc::clone(sym));
                }
            }
        }
        current = n.parent.as_ref();
    }
    None
}

/// Resolve the symbol for an identifier node at the cursor, handling both
/// declaration names (the identifier is the `.name` of its parent declaration)
/// and references (resolved via scope walk).
fn resolve_symbol_for_node(
    symbol_map: &crate::ast::NodeSymbolMap,
    node: &Arc<crate::ast::Node>,
) -> Option<Arc<crate::ast::Symbol>> {
    // Declaration-name identifier: the node is the name of its parent and
    // the parent has an associated symbol.
    if node.kind == crate::ast::SyntaxKind::Identifier {
        if let Some(parent) = node.parent.as_ref() {
            if let Some(name) = parent.name() {
                if Arc::ptr_eq(name, node) {
                    if let Some(sym) = symbol_map.symbol_of(parent) {
                        return Some(Arc::clone(sym));
                    }
                }
            }
        }
    }
    // Reference identifier: resolve via scope walk.
    resolve_identifier_symbol(symbol_map, node)
}

/// Whether `node` is the name identifier of a declaration whose parent has
/// an associated symbol (i.e. the declaration site itself).
fn is_declaration_name(
    symbol_map: &crate::ast::NodeSymbolMap,
    node: &Arc<crate::ast::Node>,
) -> bool {
    if node.kind != crate::ast::SyntaxKind::Identifier {
        return false;
    }
    if let Some(parent) = node.parent.as_ref() {
        if let Some(name) = parent.name() {
            if Arc::ptr_eq(name, node) {
                return symbol_map.symbol_of(parent).is_some();
            }
        }
    }
    false
}

/// Whether `node` is the property-name of a property-access expression
/// (`a.b` — the `b`). Such identifiers are resolved via the left-hand type
/// rather than by scope, so they are skipped during reference collection to
/// avoid false matches against scope-resolved names.
fn is_property_access_name(node: &Arc<crate::ast::Node>) -> bool {
    use crate::ast::NodeData;
    if node.kind != crate::ast::SyntaxKind::Identifier {
        return false;
    }
    if let Some(parent) = node.parent.as_ref() {
        if let NodeData::PropertyAccessExpression(data) = &parent.data {
            return Arc::ptr_eq(&data.name, node);
        }
    }
    false
}

/// Recursively visit every node in the subtree rooted at `node`.
fn walk_all_nodes(node: &Arc<crate::ast::Node>, visitor: &mut impl FnMut(&Arc<crate::ast::Node>)) {
    visitor(node);
    let mut children = Vec::new();
    crate::ast::for_each_child(node, |child| {
        children.push(Arc::clone(child));
        false
    });
    for child in children {
        walk_all_nodes(&child, visitor);
    }
}

/// Collect all identifier nodes that resolve to `target_symbol` across every
/// source file in the program, returning each with its owning source file.
fn find_all_references(
    program: &crate::compiler::Program,
    target_symbol: &Arc<crate::ast::Symbol>,
) -> Vec<(Arc<crate::ast::SourceFile>, Arc<crate::ast::Node>)> {
    use crate::ast::SyntaxKind;
    let symbol_map = program.symbol_map();
    let mut refs = Vec::new();
    for sf in program.source_files() {
        let sf = Arc::clone(sf);
        walk_all_nodes(&sf.node, &mut |node: &Arc<crate::ast::Node>| {
            if node.kind != SyntaxKind::Identifier || is_property_access_name(node) {
                return;
            }
            if let Some(sym) = resolve_symbol_for_node(symbol_map, node) {
                if Arc::ptr_eq(&sym, target_symbol) {
                    refs.push((Arc::clone(&sf), Arc::clone(node)));
                }
            }
        });
    }
    refs
}

/// Convert a filesystem path back into a `file://` URI.
fn path_to_uri(path: &str) -> String {
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

// ─────────────────────────────────────────────────────────────────────────
// documentSymbol helpers
// ─────────────────────────────────────────────────────────────────────────

/// Build `DocumentSymbol` JSON objects for a list of statements.
/// `VariableStatement` is expanded into one symbol per declaration.
fn symbols_for_statements(
    statements: &[Arc<crate::ast::Node>],
    sf: &Arc<crate::ast::SourceFile>,
) -> Vec<Value> {
    use crate::ast::{NodeData, SyntaxKind};
    let mut result = Vec::new();
    for stmt in statements {
        match stmt.kind {
            SyntaxKind::VariableStatement => {
                if let NodeData::VariableStatement(vs) = &stmt.data {
                    if let NodeData::VariableDeclarationList(list) = &vs.declaration_list.data {
                        for decl in &list.declarations.nodes {
                            if let Some(sym) = document_symbol_for_node(decl, sf) {
                                result.push(sym);
                            }
                        }
                    }
                }
            }
            _ => {
                if let Some(sym) = document_symbol_for_node(stmt, sf) {
                    result.push(sym);
                }
            }
        }
    }
    result
}

/// Build a single `DocumentSymbol` JSON object for a declaration node, or
/// `None` if the node has no name.
fn document_symbol_for_node(
    node: &Arc<crate::ast::Node>,
    sf: &Arc<crate::ast::SourceFile>,
) -> Option<Value> {
    let name_node = node.name()?;
    let name = identifier_text(name_node)?;
    let kind = symbol_kind_for(node);

    let (rl, rc) = crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.pos());
    let (rel, rec) =
        crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.end());
    let (sl, sc) =
        crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, name_node.pos());
    let (sel, sec) =
        crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, name_node.end());

    let mut sym = json!({
        "name": name,
        "kind": kind,
        "range": {
            "start": {"line": rl, "character": rc},
            "end": {"line": rel, "character": rec}
        },
        "selectionRange": {
            "start": {"line": sl, "character": sc},
            "end": {"line": sel, "character": sec}
        }
    });
    let children = child_symbols(node, sf);
    if !children.is_empty() {
        sym["children"] = json!(children);
    }
    Some(sym)
}

/// Get the text of a name node (identifier or literal).
fn identifier_text(node: &Arc<crate::ast::Node>) -> Option<String> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::Identifier(data) => Some(data.text.clone()),
        NodeData::StringLiteral(data) => Some(data.text.clone()),
        NodeData::NumericLiteral(data) => Some(data.text.clone()),
        _ => Some(node.text().to_string()),
    }
}

/// Map a declaration node kind to an LSP `SymbolKind`.
fn symbol_kind_for(node: &Arc<crate::ast::Node>) -> i32 {
    use crate::ast::{NodeFlags, SyntaxKind as K};
    match node.kind {
        K::FunctionDeclaration => 12,  // Function
        K::ClassDeclaration => 5,      // Class
        K::InterfaceDeclaration => 11, // Interface
        K::TypeAliasDeclaration => 23, // Struct (closest to "Type")
        K::EnumDeclaration => 10,      // Enum
        K::ModuleDeclaration => 3,     // Namespace
        K::VariableDeclaration => {
            // `const` declarations map to Constant; others to Variable.
            let is_const = node
                .parent
                .as_ref()
                .map_or(false, |p| p.flags.contains(NodeFlags::Const));
            if is_const {
                14 // Constant
            } else {
                13 // Variable
            }
        }
        K::MethodDeclaration | K::MethodSignature => 6, // Method
        K::GetAccessor | K::SetAccessor => 6,           // Method
        K::Constructor => 9,                            // Constructor
        K::PropertyDeclaration | K::PropertySignature => 7, // Property
        K::EnumMember => 22,                            // EnumMember
        _ => 13,                                        // Variable
    }
}

/// Collect child `DocumentSymbol` entries for container declarations
/// (classes, interfaces, enums, namespaces).
fn child_symbols(node: &Arc<crate::ast::Node>, sf: &Arc<crate::ast::SourceFile>) -> Vec<Value> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::ClassDeclaration(d) => d
            .members
            .nodes
            .iter()
            .filter_map(|m| document_symbol_for_node(m, sf))
            .collect(),
        NodeData::ClassExpression(d) => d
            .members
            .nodes
            .iter()
            .filter_map(|m| document_symbol_for_node(m, sf))
            .collect(),
        NodeData::InterfaceDeclaration(d) => d
            .members
            .nodes
            .iter()
            .filter_map(|m| document_symbol_for_node(m, sf))
            .collect(),
        NodeData::EnumDeclaration(d) => d
            .members
            .nodes
            .iter()
            .filter_map(|m| document_symbol_for_node(m, sf))
            .collect(),
        NodeData::ModuleDeclaration(d) => {
            if let Some(body) = &d.body {
                if let NodeData::ModuleBlock(mb) = &body.data {
                    return symbols_for_statements(&mb.statements.nodes, sf);
                }
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

/// Convert a compiler diagnostic into the LSP `Diagnostic` JSON object.
/// `content` is the current document text, used to compute line/column for
/// diagnostics that are not attached to a source file.
fn diagnostic_to_lsp(diag: &crate::ast::diagnostic::Diagnostic, content: &str) -> Value {
    let (line, col) = if let Some(file) = &diag.file {
        crate::diagnosticwriter::line_and_character(&file.line_map, &file.text, diag.loc.pos())
    } else {
        // Diagnostics without an attached file (e.g. file-not-found) fall back
        // to the current document content for line/column computation.
        let line_map = crate::ast::LineMap::from_text(content);
        let pos = diag.loc.pos().min(content.len());
        crate::diagnosticwriter::line_and_character(&line_map, content, pos)
    };

    let severity = match diag.category {
        crate::diagnostics::Category::Error => 1,
        crate::diagnostics::Category::Warning => 2,
        crate::diagnostics::Category::Suggestion => 3,
        crate::diagnostics::Category::Message => 3,
    };

    let message = crate::diagnosticwriter::message_text(diag);

    json!({
        "range": {
            "start": {"line": line, "character": col},
            "end": {"line": line, "character": col + diag.loc.len().max(1)}
        },
        "severity": severity,
        "code": diag.code,
        "source": "tsox",
        "message": message
    })
}

fn make_response(id: Option<Value>, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn make_error_response(id: Option<Value>, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

/// Entry point for `--lsp` mode.
pub fn run_lsp() -> i32 {
    let mut server = LspServer::new();
    server.run()
}

/// Map TypeScript SymbolFlags to LSP CompletionItemKind.
fn completion_item_kind(flags: &crate::ast::SymbolFlags) -> i32 {
    use crate::ast::SymbolFlags as F;
    if flags.contains(F::FunctionScopedVariable | F::BlockScopedVariable)
        || flags.contains(F::Function)
    {
        3 // Function
    } else if flags.contains(F::Class) {
        7 // Class
    } else if flags.contains(F::Interface) {
        8 // Interface
    } else if flags.contains(F::RegularEnum | F::ConstEnum) {
        13 // Enum
    } else if flags.contains(F::TypeAlias | F::TypeParameter) {
        25 // TypeParameter
    } else if flags.contains(F::ValueModule | F::NamespaceModule) {
        9 // Module
    } else if flags.contains(F::Property) {
        5 // Field
    } else if flags.contains(F::Method) {
        2 // Method
    } else if flags.contains(F::ConstEnum | F::RegularEnum | F::EnumMember) {
        21 // EnumMember
    } else {
        6 // Variable
    }
}
