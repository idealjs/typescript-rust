//! LSP protocol core types: Method, DocumentUri, RequestInfo, UnmarshalParams.
//!
//! Ported from Go's `internal/lsp/lsproto/lsp.go`.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::jsonrpc::jsonrpc::Id as JsonrpcId;
use crate::tspath;

/// A document URI (typically `file://...`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentUri(pub String);

impl DocumentUri {
    /// Converts a document URI to a file name (file path).
    pub fn file_name(&self) -> String {
        let uri = &self.0;

        // Bundled files are returned as-is.
        if crate::bundled::is_bundled(uri) {
            return uri.clone();
        }

        if let Some(rest) = uri.strip_prefix("file://") {
            // Simple parsing: strip "file://" prefix and handle host.
            if let Some(stripped) = rest.strip_prefix("//") {
                // Has authority: //host/path
                if let Some(slash_idx) = stripped.find('/') {
                    let (_host, path) = stripped.split_at(slash_idx);
                    return path.to_string();
                }
                return stripped.to_string();
            }
            // Check for Windows drive letter (file:///C:/...)
            if let Some(rest2) = rest.strip_prefix('/') {
                if rest2.len() >= 2 && rest2.as_bytes()[1] == b':' {
                    return rest2.to_string();
                }
            }
            return rest.to_string();
        }

        // Leave all other URIs as-is.
        let (scheme, path, ok) = split_once(uri, ':');
        if !ok {
            panic!("invalid URI: {uri}");
        }

        let authority = "ts-nul-authority";
        let mut file_path = path;
        if let Some(rest) = path.strip_prefix("//") {
            let (_auth, rest_path, ok) = split_once(rest, '/');
            if ok {
                file_path = rest_path;
            }
        }

        format!("^/{scheme}/{authority}/{file_path}")
    }

    /// Converts a document URI to a tspath::Path.
    pub fn path(&self, use_case_sensitive_file_names: bool) -> tspath::Path {
        let file_name = self.file_name();
        tspath::to_path(&file_name, "", use_case_sensitive_file_names)
    }
}

impl fmt::Display for DocumentUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A generic URI string.
pub type Uri = String;

/// An LSP method name (e.g. "textDocument/hover").
pub type Method = String;

/// A generic URI type alias.
// !!! Go has `type URI string`; we use String directly.

/// Splits at the first occurrence of `sep`, returning (before, after, true) or
/// (original, "", false).
fn split_once(s: &str, sep: char) -> (&str, &str, bool) {
    match s.find(sep) {
        Some(idx) => (&s[..idx], &s[idx + sep.len_utf8()..], true),
        None => (s, "", false),
    }
}

// !!! Trait for types that have a text document URI.
pub trait HasTextDocumentUri {
    fn text_document_uri(&self) -> &DocumentUri;
}

pub trait HasTextDocumentPosition: HasTextDocumentUri {
    fn text_document_position(&self) -> &Position;
}

pub trait HasLocations {
    fn get_locations(&self) -> &Vec<Location>;
}

pub trait HasLocation {
    fn get_location(&self) -> &Location;
}

// !!! RequestInfo / NotificationInfo are compile-time typed in Go via generics.
// In Rust, we use them as simple structs carrying the Method string.

/// Information about an LSP request method.
#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub method: Method,
}

impl RequestInfo {
    pub fn new_request_message(&self, id: Option<JsonrpcId>, params: Value) -> RequestMessage {
        RequestMessage {
            jsonrpc: Default::default(),
            id,
            method: self.method.clone(),
            params: Some(params),
        }
    }
}

/// Information about an LSP notification method.
#[derive(Debug, Clone)]
pub struct NotificationInfo {
    pub method: Method,
}

impl NotificationInfo {
    pub fn new_notification_message(&self, params: Value) -> RequestMessage {
        RequestMessage {
            jsonrpc: Default::default(),
            id: None,
            method: self.method.clone(),
            params: Some(params),
        }
    }
}

/// Error codes for LSP (extends JSON-RPC error codes).
pub const ERR_CODE_REQUEST_CANCELLED: i32 = -32800;
pub const ERR_CODE_SERVER_CANCELLED: i32 = -32802;
pub const ERR_CODE_CONTENT_MODIFIED: i32 = -32801;
pub const ERR_CODE_REQUEST_FAILED: i32 = -32803;

// !!! Custom error types
pub const ERR_CODE_INVALID_REQUEST: i32 = crate::jsonrpc::jsonrpc::CODE_INVALID_REQUEST;
pub const ERR_CODE_INVALID_PARAMS: i32 = crate::jsonrpc::jsonrpc::CODE_INVALID_PARAMS;

/// NoParams sentinel type for methods that carry no parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoParams;

impl NoParams {
    pub fn is_zero(&self) -> bool {
        true
    }
}

/// Null sentinel type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Null;

/// Preferred markup kind helper.
pub fn preferred_markup_kind(formats: &[MarkupKind]) -> MarkupKind {
    if !formats.is_empty() {
        formats[0].clone()
    } else {
        MarkupKind::PlainText
    }
}

// !!! Code action kinds
pub const CODE_ACTION_KIND_SOURCE_REMOVE_UNUSED_IMPORTS: &str = "source.removeUnusedImports";
pub const CODE_ACTION_KIND_SOURCE_SORT_IMPORTS: &str = "source.sortImports";

// ============================================================================
// Core LSP types (from lsp_generated.go — hand-selected most important)
// ============================================================================

/// A position in a text document expressed as zero-based line and character offset.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// A range in a text document expressed as (zero-based, inclusive) start and
/// (zero-based, exclusive) end positions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Location {
    pub uri: DocumentUri,
    pub range: Range,
}

/// A textual edit applicable to a text document.
///
/// Mirrors `lsproto.TextEdit` in Go.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextEdit {
    pub new_text: String,
    pub range: Range,
}

/// Value-object describing what options formatting should use.
///
/// Mirrors `lsproto.FormattingOptions` in Go.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormattingOptions {
    pub tab_size: u32,
    pub insert_spaces: bool,
    pub trim_trailing_whitespace: Option<bool>,
}

/// Markup kind enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkupKind {
    PlainText,
    Markdown,
}

impl Default for MarkupKind {
    fn default() -> Self {
        MarkupKind::PlainText
    }
}

/// Describes the content type that a client supports in various result literals.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StringOrMarkupContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markup_content: Option<MarkupContent>,
}

impl StringOrMarkupContent {
    pub fn as_string(&self) -> String {
        if let Some(s) = &self.string {
            return s.clone();
        }
        if let Some(mc) = &self.markup_content {
            return mc.value.clone();
        }
        String::new()
    }
}

/// A `MarkupContent` literal represents a string content which is interpreted
/// based on its kind flag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

/// LSP request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    #[serde(default)]
    pub jsonrpc: crate::jsonrpc::jsonrpc::JsonrpcVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonrpcId>,
    pub method: Method,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl RequestMessage {
    pub fn message(&self) -> Message {
        let kind = if self.id.is_none() {
            crate::jsonrpc::jsonrpc::MessageKind::Notification
        } else {
            crate::jsonrpc::jsonrpc::MessageKind::Request
        };
        Message {
            kind,
            msg: MessageData::Request(self.clone()),
        }
    }
}

/// LSP response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    #[serde(default)]
    pub jsonrpc: crate::jsonrpc::jsonrpc::JsonrpcVersion,
    pub id: Option<JsonrpcId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<crate::jsonrpc::jsonrpc::ResponseError>,
}

impl ResponseMessage {
    pub fn message(&self) -> Message {
        Message {
            kind: crate::jsonrpc::jsonrpc::MessageKind::Response,
            msg: MessageData::Response(self.clone()),
        }
    }
}

/// Enum representing which type of message an [`Message`] is.
#[derive(Debug)]
pub enum MessageData {
    Request(RequestMessage),
    Response(ResponseMessage),
}

/// An LSP message (request, notification, or response).
#[derive(Debug)]
pub struct Message {
    pub kind: crate::jsonrpc::jsonrpc::MessageKind,
    pub msg: MessageData,
}

impl Message {
    pub fn as_request(&self) -> &RequestMessage {
        match &self.msg {
            MessageData::Request(r) => r,
            _ => panic!("Message is not a request"),
        }
    }

    pub fn as_response(&self) -> &ResponseMessage {
        match &self.msg {
            MessageData::Response(r) => r,
            _ => panic!("Message is not a response"),
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.msg {
            MessageData::Request(r) => r.serialize(serializer),
            MessageData::Response(r) => r.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMessage {
            #[serde(default)]
            #[allow(dead_code)]
            jsonrpc: crate::jsonrpc::jsonrpc::JsonrpcVersion,
            #[serde(skip_serializing_if = "Option::is_none")]
            id: Option<JsonrpcId>,
            #[serde(default)]
            method: Method,
            #[serde(skip_serializing_if = "Option::is_none")]
            params: Option<Value>,
            #[serde(skip_serializing_if = "Option::is_none")]
            result: Option<Value>,
            #[serde(skip_serializing_if = "Option::is_none")]
            error: Option<crate::jsonrpc::jsonrpc::ResponseError>,
        }

        let raw = RawMessage::deserialize(deserializer)?;

        if raw.id.is_some() && raw.method.is_empty() {
            // Response
            return Ok(Message {
                kind: crate::jsonrpc::jsonrpc::MessageKind::Response,
                msg: MessageData::Response(ResponseMessage {
                    jsonrpc: Default::default(),
                    id: raw.id,
                    result: raw.result,
                    error: raw.error,
                }),
            });
        }

        // Request or notification
        let kind = if raw.id.is_none() {
            crate::jsonrpc::jsonrpc::MessageKind::Notification
        } else {
            crate::jsonrpc::jsonrpc::MessageKind::Request
        };

        Ok(Message {
            kind,
            msg: MessageData::Request(RequestMessage {
                jsonrpc: Default::default(),
                id: raw.id,
                method: raw.method,
                params: raw.params,
            }),
        })
    }
}

/// Text document identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: DocumentUri,
}

/// Text document position parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentPositionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

impl HasTextDocumentUri for TextDocumentPositionParams {
    fn text_document_uri(&self) -> &DocumentUri {
        &self.text_document.uri
    }
}

impl HasTextDocumentPosition for TextDocumentPositionParams {
    fn text_document_position(&self) -> &Position {
        &self.position
    }
}
