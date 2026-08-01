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
                self.handle_did_close(&params);
                (None, Vec::new())
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
                "completionProvider": {
                    "triggerCharacters": [".", "\"", "'", "/", "@", "<"],
                    "resolveProvider": false
                },
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
            return self.compute_diagnostics(uri);
        }
        Vec::new()
    }

    fn handle_did_change(&mut self, params: &Value) -> Vec<Value> {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            // Full sync: take the last change.
            if let Some(changes) = params.get("contentChanges").and_then(|v| v.as_array()) {
                if let Some(last) = changes.last() {
                    if let Some(text) = last.get("text").and_then(|v| v.as_str()) {
                        self.documents.insert(uri.to_string(), text.to_string());
                        return self.compute_diagnostics(uri);
                    }
                }
            }
        }
        Vec::new()
    }

    fn handle_did_close(&mut self, params: &Value) {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            self.documents.remove(uri);
        }
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
        let type_str = checker.get_quick_info_text(&node);
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

    /// Run the compiler pipeline (parse + bind + check) for a single document
    /// and return a `textDocument/publishDiagnostics` notification with all of
    /// its diagnostics. An empty diagnostic list is published to clear any
    /// previously reported errors for the file.
    fn compute_diagnostics(&self, uri: &str) -> Vec<Value> {
        let path = uri.strip_prefix("file://").unwrap_or(uri);
        let content = match self.documents.get(uri) {
            Some(c) => c.clone(),
            None => return Vec::new(),
        };

        let program = build_program(path, &content);

        // Collect parse/bind diagnostics (from the program) plus semantic
        // diagnostics (from the checker).
        let mut all_diags: Vec<crate::ast::diagnostic::Diagnostic> = Vec::new();
        for d in program.get_diagnostics_to_report() {
            all_diags.push((*d).clone());
        }
        all_diags.extend(program.get_semantic_diagnostics());

        let lsp_diags: Vec<Value> = all_diags
            .iter()
            .map(|d| diagnostic_to_lsp(d, &content))
            .collect();

        vec![json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": uri,
                "diagnostics": lsp_diags
            }
        })]
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
