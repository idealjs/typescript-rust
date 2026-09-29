#![allow(unused_imports, dead_code)]

use std::io::{BufReader, BufWriter, Read, Write};

use serde_json::Value as JsonValue;

pub const MESSAGE_TYPE_UNKNOWN: u8 = 0;
pub const MESSAGE_TYPE_REQUEST: u8 = 1;
pub const MESSAGE_TYPE_CALL_RESPONSE: u8 = 2;
pub const MESSAGE_TYPE_CALL_ERROR: u8 = 3;
pub const MESSAGE_TYPE_RESPONSE: u8 = 4;
pub const MESSAGE_TYPE_ERROR: u8 = 5;
pub const MESSAGE_TYPE_CALL: u8 = 6;

pub fn message_type_is_valid(m: u8) -> bool {
    (MESSAGE_TYPE_REQUEST..=MESSAGE_TYPE_CALL).contains(&m)
}

pub const MSGPACK_FIXED_ARRAY_3: u8 = 0x93;
pub const MSGPACK_BIN_8: u8 = 0xC4;
pub const MSGPACK_BIN_16: u8 = 0xC5;
pub const MSGPACK_BIN_32: u8 = 0xC6;
pub const MSGPACK_U_8: u8 = 0xCC;

pub struct MessagePackProtocol<R: Read, W: Write> {
    r: BufReader<R>,
    w: BufWriter<W>,
}

pub fn new_message_pack_protocol<R: Read, W: Write>(rw: (R, W)) -> MessagePackProtocol<R, W> {
    MessagePackProtocol {
        r: BufReader::new(rw.0),
        w: BufWriter::new(rw.1),
    }
}

impl<R: Read, W: Write> MessagePackProtocol<R, W> {
    pub fn read_message(&mut self) -> Result<tsox_lsp::jsonrpc::jsonrpc::Message, String> {
        let (msg_type, method, payload) = self.read_tuple()?;
        let mut msg = tsox_lsp::jsonrpc::jsonrpc::Message {
            jsonrpc: Default::default(),
            id: None,
            method: String::new(),
            params: None,
            result: None,
            error: None,
        };
        match msg_type {
            MESSAGE_TYPE_REQUEST => {
                let id = new_id_string(&method);
                msg.id = Some(id);
                msg.method = method;
                msg.params = Some(serde_json::from_slice(&payload).unwrap_or(JsonValue::Null));
            }
            MESSAGE_TYPE_CALL_RESPONSE => {
                let id = new_id_string(&method);
                msg.id = Some(id);
                msg.result = Some(serde_json::from_slice(&payload).unwrap_or(JsonValue::Null));
            }
            MESSAGE_TYPE_CALL_ERROR => {
                let id = new_id_string(&method);
                msg.id = Some(id);
                msg.error = Some(tsox_lsp::jsonrpc::jsonrpc::ResponseError {
                    code: tsox_lsp::jsonrpc::jsonrpc::CODE_INTERNAL_ERROR,
                    message: String::from_utf8_lossy(&payload).into_owned(),
                    data: None,
                });
            }
            other => {
                return Err(format!("unexpected message type: {}", other));
            }
        }
        Ok(msg)
    }

    pub fn write_request(
        &mut self,
        _id: Option<&tsox_lsp::jsonrpc::jsonrpc::Id>,
        method: &str,
        params: &JsonValue,
    ) -> Result<(), String> {
        let payload = serde_json::to_vec(params).map_err(|e| e.to_string())?;
        self.write_tuple(MESSAGE_TYPE_CALL, method, &payload)
    }

    pub fn write_notification(&mut self, method: &str, params: &JsonValue) -> Result<(), String> {
        self.write_request(None, method, params)
    }

    pub fn write_response(
        &mut self,
        id: Option<&tsox_lsp::jsonrpc::jsonrpc::Id>,
        result: ProtocolResult,
    ) -> Result<(), String> {
        let method = match id {
            Some(id) => id_to_string_value(id),
            None => String::new(),
        };
        let payload = match result {
            ProtocolResult::RawBinary(raw) => raw,
            ProtocolResult::Json(value) => serde_json::to_vec(&value).map_err(|e| e.to_string())?,
        };
        self.write_tuple(MESSAGE_TYPE_RESPONSE, &method, &payload)
    }

    pub fn write_error(
        &mut self,
        id: Option<&tsox_lsp::jsonrpc::jsonrpc::Id>,
        resp_err: &tsox_lsp::jsonrpc::jsonrpc::ResponseError,
    ) -> Result<(), String> {
        let method = match id {
            Some(id) => id_to_string_value(id),
            None => String::new(),
        };
        self.write_tuple(MESSAGE_TYPE_ERROR, &method, resp_err.message.as_bytes())
    }

    fn read_tuple(&mut self) -> Result<(u8, String, Vec<u8>), String> {
        let mut marker = [0u8; 1];
        self.r
            .read_exact(&mut marker)
            .map_err(|e| e.to_string())?;
        if marker[0] != MSGPACK_FIXED_ARRAY_3 {
            return Err(format!(
                "{}: expected fixed 3-element array (0x93), received: 0x{:02x}",
                ERR_INVALID_REQUEST, marker[0]
            ));
        }
        let mut t = [0u8; 1];
        self.r.read_exact(&mut t).map_err(|e| e.to_string())?;
        let raw_type = if t[0] <= 0x7f {
            t[0]
        } else if t[0] == MSGPACK_U_8 {
            let mut raw = [0u8; 1];
            self.r.read_exact(&mut raw).map_err(|e| e.to_string())?;
            raw[0]
        } else {
            return Err(format!(
                "{}: expected positive fixint or uint8 marker, received: 0x{:02x}",
                ERR_INVALID_REQUEST, t[0]
            ));
        };
        if !message_type_is_valid(raw_type) {
            return Err(format!(
                "{}: unknown message type: {}",
                ERR_INVALID_REQUEST, raw_type
            ));
        }
        let method = String::from_utf8_lossy(&self.read_bin()?).into_owned();
        let payload = self.read_bin()?;
        Ok((raw_type, method, payload))
    }

    fn read_bin(&mut self) -> Result<Vec<u8>, String> {
        let mut t = [0u8; 1];
        self.r.read_exact(&mut t).map_err(|e| e.to_string())?;
        let size: usize = match t[0] {
            MSGPACK_BIN_8 => {
                let mut size = [0u8; 1];
                self.r.read_exact(&mut size).map_err(|e| e.to_string())?;
                size[0] as usize
            }
            MSGPACK_BIN_16 => {
                let mut size = [0u8; 2];
                self.r.read_exact(&mut size).map_err(|e| e.to_string())?;
                u16::from_be_bytes(size) as usize
            }
            MSGPACK_BIN_32 => {
                let mut size = [0u8; 4];
                self.r.read_exact(&mut size).map_err(|e| e.to_string())?;
                u32::from_be_bytes(size) as usize
            }
            other => {
                return Err(format!(
                    "{}: expected binary data (0xc4-0xc6), received: 0x{:02x}",
                    ERR_INVALID_REQUEST, other
                ));
            }
        };
        let mut payload = vec![0u8; size];
        self.r
            .read_exact(&mut payload)
            .map_err(|e| e.to_string())?;
        Ok(payload)
    }

    fn write_tuple(&mut self, msg_type: u8, method: &str, payload: &[u8]) -> Result<(), String> {
        self.w
            .write_all(&[MSGPACK_FIXED_ARRAY_3])
            .map_err(|e| e.to_string())?;
        self.w.write_all(&[msg_type]).map_err(|e| e.to_string())?;
        self.write_bin(method.as_bytes())?;
        self.write_bin(payload)?;
        self.w.flush().map_err(|e| e.to_string())
    }

    fn write_bin(&mut self, data: &[u8]) -> Result<(), String> {
        let length = data.len();
        if length < 256 {
            self.w
                .write_all(&[MSGPACK_BIN_8, length as u8])
                .map_err(|e| e.to_string())?;
        } else if length < 1 << 16 {
            self.w
                .write_all(&[MSGPACK_BIN_16])
                .map_err(|e| e.to_string())?;
            self.w
                .write_all(&(length as u16).to_be_bytes())
                .map_err(|e| e.to_string())?;
        } else {
            self.w
                .write_all(&[MSGPACK_BIN_32])
                .map_err(|e| e.to_string())?;
            self.w
                .write_all(&(length as u32).to_be_bytes())
                .map_err(|e| e.to_string())?;
        }
        self.w.write_all(data).map_err(|e| e.to_string())
    }
}

pub enum ProtocolResult {
    Json(JsonValue),
    RawBinary(Vec<u8>),
}

pub type RawBinary = Vec<u8>;

const ERR_INVALID_REQUEST: &str = "invalid request";

fn new_id_string(method: &str) -> tsox_lsp::jsonrpc::jsonrpc::Id {
    tsox_lsp::jsonrpc::jsonrpc::Id::new_string(method)
}

fn id_to_string_value(id: &tsox_lsp::jsonrpc::jsonrpc::Id) -> String {
    match id {
        tsox_lsp::jsonrpc::jsonrpc::Id::Int(v) => v.to_string(),
        tsox_lsp::jsonrpc::jsonrpc::Id::Str(s) => s.clone(),
    }
}

pub struct StdioServer {
    pub options: StdioServerOptions,
}

pub struct StdioServerOptions {
    pub cwd: String,
    pub pipe_path: String,
    pub input: std::process::Stdio,
    pub output: std::process::Stdio,
}

pub fn new_stdio_server(options: StdioServerOptions) -> StdioServer {
    if options.cwd.is_empty() {
        panic!("StdioServerOptions.Cwd is required");
    }
    StdioServer { options }
}
