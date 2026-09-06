pub mod dynamic_queue;
pub mod logger;
pub mod lsproto;
pub mod lspwatcher;
pub mod progress;
pub mod server;
pub mod stack_sanitizer;

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;

use serde_json::{Value, json};

pub struct LspServer {
    documents: HashMap<String, String>,
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

                    return if self.shutdown_requested { 0 } else { 1 };
                }
                Err(e) => {
                    eprintln!("LSP read error: {e}");
                    return 1;
                }
            }
        }
    }

    fn read_message<R: BufRead>(&self, reader: &mut R) -> io::Result<Option<Value>> {

        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                return Ok(None);
            }
            let trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
            if trimmed.is_empty() {
                break;
            }
            if let Some(rest) = trimmed.strip_prefix("Content-Length: ") {
                content_length = rest.parse::<usize>().ok();
            }
        }

        let length = match content_length {
            Some(l) => l,
            None => return Ok(None),
        };

        let mut body = vec![0u8; length];
        reader.read_exact(&mut body)?;
        let msg: Value = serde_json::from_slice(&body)?;
        Ok(Some(msg))
    }

    fn write_message<W: Write>(&self, writer: &mut W, msg: &Value) -> io::Result<()> {
        let body = serde_json::to_string(msg)?;
        write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        writer.flush()
    }

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

                (Some(make_response(id, json!([]))), Vec::new())
            }
            _ => {

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

    fn handle_did_open(&mut self, params: &Value) -> Vec<Value> {
        if let Some(td) = params.get("textDocument") {
            let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            let text = td.get("text").and_then(|v| v.as_str()).unwrap_or("");
            self.documents.insert(uri.to_string(), text.to_string());
        }

        self.compute_all_diagnostics()
    }

    fn handle_did_change(&mut self, params: &Value) -> Vec<Value> {
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

    fn handle_did_close(&mut self, params: &Value) -> Vec<Value> {
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

        let node = find_deepest_node(&source_file.node, offset);
        let mut checker = program.build_checker();

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

    fn compute_all_diagnostics(&self) -> Vec<Value> {
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
        let checker = program.build_checker();

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
                    .find(|sf| decl.loc.pos() < sf.text.len())
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

        let _node = find_deepest_node(&source_file.node, offset);
        let checker = program.build_checker();

        let mut items: Vec<Value> = Vec::new();

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
                items.push(json!({"label": kw, "kind": 14}));
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

fn display_parts_to_string(parts: &[crate::checker::nodebuilder::SymbolDisplayPart]) -> String {
    parts.iter().map(|p| p.text.as_str()).collect()
}

fn find_deepest_node(node: &Arc<crate::ast::Node>, offset: usize) -> Arc<crate::ast::Node> {
    let mut deepest = Arc::clone(node);
    loop {
        let current = Arc::clone(&deepest);
        let mut next: Option<Arc<crate::ast::Node>> = None;
        crate::ast::for_each_child(&current, |child| {
            if child.loc.pos() <= offset && offset < child.loc.end() {
                next = Some(Arc::clone(child));
                true
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

fn resolve_symbol_for_node(
    symbol_map: &crate::ast::NodeSymbolMap,
    node: &Arc<crate::ast::Node>,
) -> Option<Arc<crate::ast::Symbol>> {

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

    resolve_identifier_symbol(symbol_map, node)
}

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

fn path_to_uri(path: &str) -> String {
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

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

fn identifier_text(node: &Arc<crate::ast::Node>) -> Option<String> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::Identifier(data) => Some(data.text.clone()),
        NodeData::StringLiteral(data) => Some(data.text.clone()),
        NodeData::NumericLiteral(data) => Some(data.text.clone()),
        _ => Some(node.text().to_string()),
    }
}

fn symbol_kind_for(node: &Arc<crate::ast::Node>) -> i32 {
    use crate::ast::{NodeFlags, SyntaxKind as K};
    match node.kind {
        K::FunctionDeclaration => 12,
        K::ClassDeclaration => 5,
        K::InterfaceDeclaration => 11,
        K::TypeAliasDeclaration => 23,
        K::EnumDeclaration => 10,
        K::ModuleDeclaration => 3,
        K::VariableDeclaration => {

            let is_const = node
                .parent
                .as_ref()
                .map_or(false, |p| p.flags.contains(NodeFlags::Const));
            if is_const {
                14
            } else {
                13
            }
        }
        K::MethodDeclaration | K::MethodSignature => 6,
        K::GetAccessor | K::SetAccessor => 6,
        K::Constructor => 9,
        K::PropertyDeclaration | K::PropertySignature => 7,
        K::EnumMember => 22,
        _ => 13,
    }
}

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

fn diagnostic_to_lsp(diag: &crate::ast::diagnostic::Diagnostic, content: &str) -> Value {
    let (line, col) = if let Some(file) = &diag.file {
        crate::diagnosticwriter::line_and_character(&file.line_map, &file.text, diag.loc.pos())
    } else {

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

    let message = crate::diagnosticwriter::message_text(diag, None);

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

pub fn run_lsp() -> i32 {
    let mut server = LspServer::new();
    server.run()
}

fn completion_item_kind(flags: &crate::ast::SymbolFlags) -> i32 {
    use crate::ast::SymbolFlags as F;
    if flags.contains(F::FunctionScopedVariable | F::BlockScopedVariable)
        || flags.contains(F::Function)
    {
        3
    } else if flags.contains(F::Class) {
        7
    } else if flags.contains(F::Interface) {
        8
    } else if flags.contains(F::RegularEnum | F::ConstEnum) {
        13
    } else if flags.contains(F::TypeAlias | F::TypeParameter) {
        25
    } else if flags.contains(F::ValueModule | F::NamespaceModule) {
        9
    } else if flags.contains(F::Property) {
        5
    } else if flags.contains(F::Method) {
        2
    } else if flags.contains(F::ConstEnum | F::RegularEnum | F::EnumMember) {
        21
    } else {
        6
    }
}
