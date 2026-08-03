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

// ────────────────────────────────────────────────────────────────────────────
// Types needed by the project package (stubs matching Go lsproto shapes).
// ────────────────────────────────────────────────────────────────────────────

/// A bit-flag for the kind of changes a file watcher monitors.
/// Go: `type WatchKind = uint32`
pub type WatchKind = u32;
pub const WATCH_KIND_CREATE: WatchKind = 1;
pub const WATCH_KIND_CHANGE: WatchKind = 2;
pub const WATCH_KIND_DELETE: WatchKind = 4;

/// A language identifier string (e.g. "typescript").
/// Go: `type LanguageKind = string`
pub type LanguageKind = String;

/// File change type for workspace/didChangeWatchedFiles.
/// Go: `type FileChangeType = uint32`
pub type FileChangeType = u32;
pub const FILE_CHANGE_TYPE_CREATED: FileChangeType = 1;
pub const FILE_CHANGE_TYPE_CHANGED: FileChangeType = 2;
pub const FILE_CHANGE_TYPE_DELETED: FileChangeType = 3;

/// Position encoding kind.
/// Go: `type PositionEncodingKind = string`
pub type PositionEncodingKind = String;
pub const POSITION_ENCODING_UTF16: &str = "utf-16";
pub const POSITION_ENCODING_UTF8: &str = "utf-8";
pub const POSITION_ENCODING_UTF32: &str = "utf-32";

/// Log verbosity level.
pub type LogVerbosity = i32;
pub const LOG_VERBOSITY_OFF: LogVerbosity = 0;
pub const LOG_VERBOSITY_ERROR: LogVerbosity = 1;
pub const LOG_VERBOSITY_WARNING: LogVerbosity = 2;
pub const LOG_VERBOSITY_INFO: LogVerbosity = 3;
pub const LOG_VERBOSITY_DEBUG: LogVerbosity = 4;
pub const LOG_VERBOSITY_TRACE: LogVerbosity = 5;

/// Message type for window/logMessage.
pub type MessageType = i32;
pub const MESSAGE_TYPE_ERROR: MessageType = 1;
pub const MESSAGE_TYPE_WARNING: MessageType = 2;
pub const MESSAGE_TYPE_INFO: MessageType = 3;
pub const MESSAGE_TYPE_DEBUG: MessageType = 4;

/// A glob pattern: either a plain string or a relative pattern.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternOrRelativePattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_pattern: Option<RelativePattern>,
}

/// A relative pattern with a base URI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelativePattern {
    pub base_uri: WorkspaceFolderOrURI,
    pub pattern: String,
}

/// A workspace folder or a plain URI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceFolderOrURI {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folder: Option<WorkspaceFolder>,
}

/// A workspace folder.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: DocumentUri,
    pub name: String,
}

/// A file system watcher registration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileSystemWatcher {
    pub glob_pattern: PatternOrRelativePattern,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<WatchKind>,
}

/// A file event in a `workspace/didChangeWatchedFiles` notification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileEvent {
    pub uri: DocumentUri,
    #[serde(rename = "type")]
    pub change_type: FileChangeType,
}

/// Parameters for `textDocument/publishDiagnostics`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishDiagnosticsParams {
    pub uri: DocumentUri,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    pub diagnostics: Vec<Diagnostic>,
}

/// An LSP Diagnostic.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub message: String,
}

/// A text document content change — either partial (range-based) or whole.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentContentChangePartialOrWholeDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial: Option<TextDocumentContentChangePartial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whole_document: Option<TextDocumentContentChangeWholeDocument>,
}

/// A partial (range-based) text change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentContentChangePartial {
    pub range: Range,
    pub text: String,
}

/// A whole-document text change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentContentChangeWholeDocument {
    pub text: String,
}

/// Work done progress begin/report/end union.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressBeginOrReportOrEnd {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin: Option<WorkDoneProgressBegin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<WorkDoneProgressReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<WorkDoneProgressEnd>,
}

/// WorkDoneProgressBegin.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressBegin {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// WorkDoneProgressReport.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// WorkDoneProgressEnd.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressEnd {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// WorkDoneProgressCreateParams.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressCreateParams {
    pub token: IntegerOrString,
}

/// An integer-or-string union for progress tokens.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntegerOrString {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integer: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,
}

/// ProgressParams for $/progress notification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressParams {
    pub token: IntegerOrString,
    pub value: WorkDoneProgressBeginOrReportOrEnd,
}

/// TelemetryEvent (stub — full shape in Go has many variants).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_stats_telemetry_event: Option<PerformanceStatsTelemetryEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_info_telemetry_event: Option<ProjectInfoTelemetryEvent>,
}

/// Performance stats telemetry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceStatsTelemetryEvent {
    pub measurements: PerformanceStatsTelemetryMeasurements,
}

/// Performance stats measurements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceStatsTelemetryMeasurements {
    pub open_file_count: f64,
    pub uptime_seconds: f64,
    pub project_count: f64,
    pub config_count: f64,
    pub cached_disk_file_count: f64,
    pub memory_used_bytes: f64,
    pub go_mem_limit: f64,
    pub go_gc_percent: f64,
    pub heap_goal_bytes: f64,
    pub heap_live_bytes: f64,
    pub heap_object_count: f64,
    pub heap_stack_bytes: f64,
    pub heap_released_bytes: f64,
    pub heap_free_bytes: f64,
    pub gc_scan_heap_bytes: f64,
    pub go_max_procs: f64,
    pub goroutine_count: f64,
    pub gc_cycles_total: f64,
    pub gc_cpu_seconds: f64,
    pub user_cpu_seconds: f64,
    pub system_mem_total: f64,
    pub system_mem_used: f64,
    pub auto_import_project_bucket_count: f64,
    pub auto_import_node_modules_bucket_count: f64,
    pub auto_import_unique_package_count: f64,
    pub auto_import_project_export_count: f64,
    pub auto_import_project_file_count: f64,
    pub auto_import_node_modules_export_count: f64,
    pub auto_import_node_modules_file_count: f64,
    pub auto_import_node_modules_unfiltered_bucket_count: f64,
}

/// Project info telemetry event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectInfoTelemetryEvent {
    pub properties: std::collections::HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurements: Option<ProjectInfoTelemetryMeasurements>,
}

/// Project info telemetry measurements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectInfoTelemetryMeasurements {
    pub ts_file_count: f64,
    pub ts_file_size: f64,
    pub tsx_file_count: f64,
    pub tsx_file_size: f64,
    pub js_file_count: f64,
    pub js_file_size: f64,
    pub jsx_file_count: f64,
    pub jsx_file_size: f64,
    pub dts_file_count: f64,
    pub dts_file_size: f64,
}

/// Client capabilities (minimal stub).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub workspace: WorkspaceClientCapabilities,
}

/// Workspace client capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceClientCapabilities {
    pub did_change_watched_files: DidChangeWatchedFilesClientCapabilities,
}

/// DidChangeWatchedFiles client capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DidChangeWatchedFilesClientCapabilities {
    pub relative_pattern_support: bool,
}

/// Log message params.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogMessageParams {
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub message: String,
}
