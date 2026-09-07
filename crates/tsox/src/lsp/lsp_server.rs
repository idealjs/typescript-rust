use super::utils::*;

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};

use serde_json::{Value, json};

pub struct LspServer {
    pub(super) documents: HashMap<String, String>,
    pub(super) workspace_root: Option<String>,
    pub(super) shutdown_requested: bool,
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
            "textDocument/formatting" => (Some(make_response(id, json!([]))), Vec::new()),
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
}

pub fn run_lsp() -> i32 {
    let mut server = LspServer::new();
    server.run()
}
