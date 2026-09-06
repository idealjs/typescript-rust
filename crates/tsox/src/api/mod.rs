use std::io::{self, BufRead, BufReader, Write};

use serde_json::{Value, json};

pub struct ApiServer {
    shutdown_requested: bool,
}

impl ApiServer {
    pub fn new() -> Self {
        ApiServer {
            shutdown_requested: false,
        }
    }

    pub fn run(&mut self) -> i32 {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = stdout.lock();

        loop {
            match Self::read_message(&mut reader) {
                Ok(Some(msg)) => {
                    if let Some(response) = self.handle_message(&msg) {
                        let _ = Self::write_message(&mut writer, &response);
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
                    eprintln!("API read error: {e}");
                    return 1;
                }
            }
        }
    }

    fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
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

    fn write_message<W: Write>(writer: &mut W, msg: &Value) -> io::Result<()> {
        let body = serde_json::to_string(msg)?;
        write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        writer.flush()
    }

    fn handle_message(&mut self, msg: &Value) -> Option<Value> {
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let _params = msg.get("params").cloned().unwrap_or(Value::Null);

        match method {
            "configure" => {

                Some(make_response(id, json!({})))
            }
            "createProject" => {

                Some(make_response(id, json!({ "projectId": "default" })))
            }
            "updateProject" => {

                Some(make_response(id, json!({})))
            }
            "getDiagnostics" => {

                Some(make_response(id, json!({ "diagnostics": [] })))
            }
            "closeProject" => Some(make_response(id, json!({}))),
            "getQuickInfo" => {

                Some(make_response(id, Value::Null))
            }
            "shutdown" => {
                self.shutdown_requested = true;
                Some(make_response(id, Value::Null))
            }
            "exit" => None,
            _ => {
                if id.is_some() {
                    Some(make_error_response(
                        id,
                        -32601,
                        &format!("Method not found: {method}"),
                    ))
                } else {
                    None
                }
            }
        }
    }
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

pub fn run_api() -> i32 {
    let mut server = ApiServer::new();
    server.run()
}
