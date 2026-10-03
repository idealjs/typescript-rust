use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use serde::de::DeserializeOwned;

use crate::lsp::dynamic_queue::DynamicQueue;
use crate::lsp::lsproto_lsp_basic::MarkupKind;
use crate::lsp::lsproto_lsp_uri::DocumentUri;
use crate::lsp::lsproto_jsonrpc::MessageKind;

pub const MARKUP_KIND_PLAIN_TEXT: MarkupKind = MarkupKind::PlainText;

pub const CODE_ACTION_KIND_EMPTY: &str = "";
pub const CODE_ACTION_KIND_SOURCE: &str = "source";
pub const CODE_ACTION_KIND_SOURCE_FIX_ALL: &str = "source.fixAll";
pub const CODE_ACTION_KIND_SOURCE_ORGANIZE_IMPORTS: &str = "source.organizeImports";

const CODE_INVALID_REQUEST: i32 = -32600;
const CODE_INVALID_PARAMS: i32 = -32602;

impl<T: Send> DynamicQueue<T> {
    pub fn get_any(&self) -> Option<crate::lsp::dynamic_queue::DynamicQueueState<T>> { ::tsox_core::fntrace::enter("get_any"); 
        if let Some(state) = self
            .idle_rx
            .lock()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(0))
            .ok()
            .flatten()
        {
            return Some(state);
        }
        self.ready_rx
            .lock()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(0))
            .ok()
            .flatten()
    }

    pub fn get_ready(&self) -> Option<crate::lsp::dynamic_queue::DynamicQueueState<T>> { ::tsox_core::fntrace::enter("get_ready"); 
        self.ready_rx.lock().unwrap().recv().ok().flatten()
    }
}

pub fn err_not_object(kind: impl std::fmt::Display) -> String { ::tsox_core::fntrace::enter("err_not_object"); 
    format!("expected object start, but encountered {kind}")
}

pub fn err_null(field: &str) -> String { ::tsox_core::fntrace::enter("err_null"); 
    format!("null value is not allowed for field \"{field}\"")
}

pub fn err_missing(props: &[&str]) -> String { ::tsox_core::fntrace::enter("err_missing"); 
    format!("missing required properties: {}", props.join(", "))
}

pub fn err_invalid_kind(type_name: &str, got: impl std::fmt::Display) -> String { ::tsox_core::fntrace::enter("err_invalid_kind"); 
    format!("invalid {type_name}: got {got}")
}

pub fn err_invalid_value(type_name: &str, data: &[u8]) -> String { ::tsox_core::fntrace::enter("err_invalid_value"); 
    format!("invalid {type_name}: {}", String::from_utf8_lossy(data))
}

pub fn err_literal_mismatch(type_name: &str, expected: &str, got: &[u8]) -> String { ::tsox_core::fntrace::enter("err_literal_mismatch"); 
    format!(
        "expected {type_name} value {expected}, got {}",
        String::from_utf8_lossy(got)
    )
}

pub fn assert_only_one(message: &str, count: usize) { ::tsox_core::fntrace::enter("assert_only_one"); 
    if count != 1 {
        panic!("{message}");
    }
}

pub fn assert_at_most_one(message: &str, count: usize) { ::tsox_core::fntrace::enter("assert_at_most_one"); 
    if count > 1 {
        panic!("{message}");
    }
}

pub fn json_key_check(name: &[u8], key: &str) -> bool { ::tsox_core::fntrace::enter("json_key_check"); 
    name.len() == key.len() + 2 && name[0] == b'"' && &name[1..name.len() - 1] == key.as_bytes()
}

pub fn json_object_raw_field(data: &[u8], field: &str) -> Option<serde_json::Value> { ::tsox_core::fntrace::enter("json_object_raw_field"); 
    let value: serde_json::Value = serde_json::from_slice(data).ok()?;
    let object = value.as_object()?;
    object.get(field).cloned()
}

pub fn json_object_has_key(data: &[u8], keys: &[&str]) -> isize { ::tsox_core::fntrace::enter("json_object_has_key"); 
    let value: serde_json::Value = match serde_json::from_slice(data) {
        Ok(value) => value,
        Err(_) => return -1,
    };
    let object = match value.as_object() {
        Some(object) => object,
        None => return -1,
    };
    for (i, key) in keys.iter().enumerate() {
        if object.contains_key(*key) {
            return i as isize;
        }
    }
    -1
}

pub fn fix_windows_uri_path(path: &str) -> String { ::tsox_core::fntrace::enter("fix_windows_uri_path"); 
    if let Some(rest) = path.strip_prefix('/') {
        if let Some((volume, rest)) = split_volume_path(rest) {
            return format!("{volume}{rest}");
        }
    }
    path.to_string()
}

fn split_volume_path(path: &str) -> Option<(&str, &str)> { ::tsox_core::fntrace::enter("split_volume_path"); 
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        let volume = &path[..2];
        let (_, rest) = path.split_at(2);
        return Some((volume, rest));
    }
    None
}

pub struct RequestInfo<Params, Resp> {
    _params: std::marker::PhantomData<Params>,
    _resp: std::marker::PhantomData<Resp>,
    pub method: String,
}

impl<Params, Resp> RequestInfo<Params, Resp>
where
    Params: DeserializeOwned,
    Resp: DeserializeOwned,
{
    pub fn unmarshal_result(&self, result: &serde_json::Value) -> Result<Resp, String> { ::tsox_core::fntrace::enter("unmarshal_result"); 
        serde_json::from_value(result.clone()).map_err(|e| e.to_string())
    }
}

pub fn unmarshal_params<T>(req_params: Option<&serde_json::Value>) -> Result<T, String>
where
    T: DeserializeOwned + LspNoParams,
{ ::tsox_core::fntrace::enter("unmarshal_params"); 
    let raw = req_params.cloned().unwrap_or(serde_json::Value::Null);

    if T::DECLARES_NO_PARAMS {
        if raw.is_null() {
            return serde_json::from_value(raw).map_err(|e| e.to_string());
        }
        return Err(format!(
            "{CODE_INVALID_PARAMS}: expected no params, got {raw}"
        ));
    }

    if !raw.is_object() && !raw.is_array() {
        return Err(format!(
            "{CODE_INVALID_PARAMS}: params must be an object or array"
        ));
    }
    serde_json::from_value(raw).map_err(|e| format!("{CODE_INVALID_PARAMS}: {e}"))
}

pub trait LspNoParams {
    const DECLARES_NO_PARAMS: bool = false;
}

pub fn null_marshal_json_to(enc: &mut serde_json::Serializer<Vec<u8>>) -> Result<(), String> { ::tsox_core::fntrace::enter("null_marshal_json_to"); 
    use serde::Serialize;
    serde_json::Value::Null
        .serialize(enc)
        .map_err(|e| e.to_string())
}

pub fn null_unmarshal_json_from(data: &[u8]) -> Result<(), String> { ::tsox_core::fntrace::enter("null_unmarshal_json_from"); 
    if data != b"null" {
        return Err(format!("expected null, got {}", String::from_utf8_lossy(data)));
    }
    Ok(())
}

static CLIENT_CAPABILITIES: LazyLock<Mutex<HashMap<TypeId, Arc<ResolvedClientCapabilities>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct ResolvedClientCapabilitiesContext {
    pub capabilities: Option<Arc<ResolvedClientCapabilities>>,
}

pub struct ResolvedClientCapabilities {
    pub raw: serde_json::Value,
}

impl Default for ResolvedClientCapabilities {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        ResolvedClientCapabilities {
            raw: serde_json::Value::Null,
        }
    }
}

pub fn with_client_capabilities(
    ctx: &mut ResolvedClientCapabilitiesContext,
    caps: Arc<ResolvedClientCapabilities>,
) { ::tsox_core::fntrace::enter("with_client_capabilities"); 
    ctx.capabilities = Some(caps);
}

pub fn get_client_capabilities(
    ctx: &ResolvedClientCapabilitiesContext,
) -> Arc<ResolvedClientCapabilities> { ::tsox_core::fntrace::enter("get_client_capabilities"); 
    ctx.capabilities
        .clone()
        .unwrap_or_else(|| Arc::new(ResolvedClientCapabilities::default()))
}

pub fn preferred_markup_kind(formats: &[MarkupKind]) -> MarkupKind { ::tsox_core::fntrace::enter("preferred_markup_kind"); 
    if !formats.is_empty() {
        return formats[0].clone();
    }
    MARKUP_KIND_PLAIN_TEXT
}

pub fn code_action_kind_contains(kind: &str, other: &str) -> bool { ::tsox_core::fntrace::enter("code_action_kind_contains"); 
    kind == other || kind == CODE_ACTION_KIND_EMPTY || other.starts_with(&format!("{kind}."))
}

pub fn document_uri_file_name(uri: &DocumentUri) -> String { ::tsox_core::fntrace::enter("document_uri_file_name"); 
    let raw = uri.0.clone();
    if raw.starts_with("file://") {
        let rest = &raw["file://".len()..];
        let (host, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => ("", ""),
        };
        if !host.is_empty() {
            return format!("//{host}{path}");
        }
        return fix_windows_uri_path(path);
    }
    let (scheme, path) = match raw.split_once(':') {
        Some(parts) => parts,
        None => panic!("invalid URI: {raw}"),
    };
    let mut authority = "ts-nul-authority".to_string();
    let mut path = path.to_string();
    if path.starts_with("//") {
        let parsed = path[2..]
            .split_once('/')
            .map(|(auth, remaining)| (auth.to_string(), remaining.to_string()));
        match parsed {
            Some((auth, remaining)) => {
                authority = auth;
                path = remaining;
            }
            None => panic!("invalid URI: {raw}"),
        }
    }
    format!("^/{scheme}/{authority}/{path}")
}
