use super::basic::Position;
use super::basic::Range;
use super::traits::{HasTextDocumentPosition, HasTextDocumentUri};
use super::uri::{DocumentUri, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: DocumentUri,
}

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

pub type WatchKind = u32;
pub const WATCH_KIND_CREATE: WatchKind = 1;
pub const WATCH_KIND_CHANGE: WatchKind = 2;
pub const WATCH_KIND_DELETE: WatchKind = 4;

pub type LanguageKind = String;

pub type FileChangeType = u32;
pub const FILE_CHANGE_TYPE_CREATED: FileChangeType = 1;
pub const FILE_CHANGE_TYPE_CHANGED: FileChangeType = 2;
pub const FILE_CHANGE_TYPE_DELETED: FileChangeType = 3;

pub type PositionEncodingKind = String;
pub const POSITION_ENCODING_UTF16: &str = "utf-16";
pub const POSITION_ENCODING_UTF8: &str = "utf-8";
pub const POSITION_ENCODING_UTF32: &str = "utf-32";

pub type LogVerbosity = i32;
pub const LOG_VERBOSITY_OFF: LogVerbosity = 0;
pub const LOG_VERBOSITY_ERROR: LogVerbosity = 1;
pub const LOG_VERBOSITY_WARNING: LogVerbosity = 2;
pub const LOG_VERBOSITY_INFO: LogVerbosity = 3;
pub const LOG_VERBOSITY_DEBUG: LogVerbosity = 4;
pub const LOG_VERBOSITY_TRACE: LogVerbosity = 5;

pub type MessageType = i32;
pub const MESSAGE_TYPE_ERROR: MessageType = 1;
pub const MESSAGE_TYPE_WARNING: MessageType = 2;
pub const MESSAGE_TYPE_INFO: MessageType = 3;
pub const MESSAGE_TYPE_DEBUG: MessageType = 4;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternOrRelativePattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_pattern: Option<RelativePattern>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelativePattern {
    pub base_uri: WorkspaceFolderOrURI,
    pub pattern: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceFolderOrURI {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folder: Option<WorkspaceFolder>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: DocumentUri,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileSystemWatcher {
    pub glob_pattern: PatternOrRelativePattern,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<WatchKind>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileEvent {
    pub uri: DocumentUri,
    #[serde(rename = "type")]
    pub change_type: FileChangeType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublishDiagnosticsParams {
    pub uri: DocumentUri,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentContentChangePartialOrWholeDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial: Option<TextDocumentContentChangePartial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whole_document: Option<TextDocumentContentChangeWholeDocument>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentContentChangePartial {
    pub range: Range,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentContentChangeWholeDocument {
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressBeginOrReportOrEnd {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin: Option<WorkDoneProgressBegin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<WorkDoneProgressReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<WorkDoneProgressEnd>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressBegin {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressEnd {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkDoneProgressCreateParams {
    pub token: IntegerOrString,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntegerOrString {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integer: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressParams {
    pub token: IntegerOrString,
    pub value: WorkDoneProgressBeginOrReportOrEnd,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_stats_telemetry_event: Option<PerformanceStatsTelemetryEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_info_telemetry_event: Option<ProjectInfoTelemetryEvent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceStatsTelemetryEvent {
    pub measurements: PerformanceStatsTelemetryMeasurements,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectInfoTelemetryEvent {
    pub properties: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurements: Option<ProjectInfoTelemetryMeasurements>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub workspace: WorkspaceClientCapabilities,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceClientCapabilities {
    pub did_change_watched_files: DidChangeWatchedFilesClientCapabilities,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DidChangeWatchedFilesClientCapabilities {
    pub relative_pattern_support: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogMessageParams {
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub message: String,
}
