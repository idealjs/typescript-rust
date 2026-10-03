#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ls::language_service::LanguageService;
use crate::ls::lsutil;
use crate::lsp::logger::Logger;
use crate::lsp::lsproto;
use crate::lsp::lsproto_baseproto::{BaseReader, BaseWriter};
use crate::lsp::progress::{ProjectLoadingProgress, ProgressReporter};
use crate::lsp::server_request_handler::Server as StubServer;
use crate::project::session::Session;
use tsox_core::diagnostics::Message;
use tsox_core::locale::Locale;

pub type LspResult<T> = Result<T, LspError>;

pub mod api {
    pub mod session {
        use std::sync::Arc;

        pub struct Session {
            pub lsp_session: Arc<crate::project::session::Session>,
        }

        impl Session {
            pub fn new(lsp_session: Arc<crate::project::session::Session>) -> Arc<Self> { ::tsox_core::fntrace::enter("new"); 
                Arc::new(Session { lsp_session })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerNotInitialized = -32002,
    UnknownErrorCode = -32001,
    RequestFailed = -32803,
    ServerCancelled = -32802,
    ContentModified = -32801,
    RequestCancelled = -32800,
    EOF = -32900,
    NoProjectForUnknownScriptKind = -32901,
    NeedsAutoImports = -32902,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        let name = match self {
            ErrorCode::ParseError => "ParseError",
            ErrorCode::InvalidRequest => "InvalidRequest",
            ErrorCode::MethodNotFound => "MethodNotFound",
            ErrorCode::InvalidParams => "InvalidParams",
            ErrorCode::InternalError => "InternalError",
            ErrorCode::ServerNotInitialized => "ServerNotInitialized",
            ErrorCode::UnknownErrorCode => "UnknownErrorCode",
            ErrorCode::RequestFailed => "RequestFailed",
            ErrorCode::ServerCancelled => "ServerCancelled",
            ErrorCode::ContentModified => "ContentModified",
            ErrorCode::RequestCancelled => "RequestCancelled",
            ErrorCode::EOF => "EOF",
            ErrorCode::NoProjectForUnknownScriptKind => "NoProjectForUnknownScriptKind",
            ErrorCode::NeedsAutoImports => "NeedsAutoImports",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone)]
pub struct LspError {
    pub code: ErrorCode,
    pub message: String,
    pub request: Option<lsproto::RequestMessage>,
}

impl LspError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self { ::tsox_core::fntrace::enter("new"); 
        LspError { code, message: message.into(), request: None }
    }

    pub fn is_code(&self, code: ErrorCode) -> bool { ::tsox_core::fntrace::enter("is_code"); 
        self.code == code
    }
}

pub struct MessageMarshalError {
    pub err: LspError,
}

impl MessageMarshalError {
    pub fn error(&self) -> String { ::tsox_core::fntrace::enter("error"); 
        format!("failed to marshal message: {}", self.err.message)
    }

    pub fn unwrap(&self) -> Vec<ErrorCode> { ::tsox_core::fntrace::enter("unwrap"); 
        vec![ErrorCode::InternalError]
    }
}

pub struct UserFacingRequestFailedError(pub String);

impl UserFacingRequestFailedError {
    pub fn error(&self) -> String { ::tsox_core::fntrace::enter("error"); 
        self.0.clone()
    }

    pub fn unwrap(&self) -> ErrorCode { ::tsox_core::fntrace::enter("unwrap"); 
        ErrorCode::RequestFailed
    }
}

pub trait Reader: Send + Sync {
    fn read(&self) -> LspResult<lsproto::Message>;
}

pub trait Writer: Send + Sync {
    fn write(&self, msg: &lsproto::Message) -> Result<(), LspError>;
}

pub struct LspReader {
    pub base: Mutex<BaseReader<Box<dyn std::io::Read + Send + Sync>>>,
}

impl LspReader {
    pub fn read(&self) -> LspResult<lsproto::Message> { ::tsox_core::fntrace::enter("read"); 
        let data = self.base.lock().unwrap().inner.read().map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                LspError::new(ErrorCode::EOF, e.to_string())
            } else {
                LspError::new(ErrorCode::InternalError, e.to_string())
            }
        })?;
        serde_json::from_slice::<lsproto::Message>(&data)
            .map_err(|_| LspError::new(ErrorCode::InvalidRequest, "failed to unmarshal message"))
    }
}

impl Reader for LspReader {
    fn read(&self) -> LspResult<lsproto::Message> { ::tsox_core::fntrace::enter("read"); 
        LspReader::read(self)
    }
}

pub fn to_reader(r: impl std::io::Read + Send + Sync + 'static) -> Arc<dyn Reader> { ::tsox_core::fntrace::enter("to_reader"); 
    Arc::new(LspReader { base: Mutex::new(BaseReader::new(Box::new(r))) })
}

pub struct LspWriter {
    pub base: Mutex<BaseWriter<Box<dyn std::io::Write + Send + Sync>>>,
}

impl LspWriter {
    pub fn write(&self, msg: &lsproto::Message) -> Result<(), LspError> { ::tsox_core::fntrace::enter("write"); 
        let data = serde_json::to_vec(msg).map_err(|e| {
            LspError::new(ErrorCode::InternalError, format!("failed to marshal message: {e}"))
        })?;
        self.base
            .lock()
            .unwrap()
            .inner
            .write(&data)
            .map_err(|e| LspError::new(ErrorCode::InternalError, e.to_string()))
    }
}

impl Writer for LspWriter {
    fn write(&self, msg: &lsproto::Message) -> Result<(), LspError> { ::tsox_core::fntrace::enter("write"); 
        LspWriter::write(self, msg)
    }
}

pub fn to_writer(w: impl std::io::Write + Send + Sync + 'static) -> Arc<dyn Writer> { ::tsox_core::fntrace::enter("to_writer"); 
    Arc::new(LspWriter { base: Mutex::new(BaseWriter::new(Box::new(w))) })
}

#[derive(Default)]
pub struct BackgroundCtx {
    pub cancelled: AtomicBool,
}

impl BackgroundCtx {
    pub fn is_done(&self) -> bool { ::tsox_core::fntrace::enter("is_done"); 
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[derive(Clone, Default)]
pub struct RequestContext {
    pub locale: Locale,
    pub request_id: Option<String>,
    pub cancelled: Arc<AtomicBool>,
}

pub fn with_request_id_cancellable(
    ctx: RequestContext,
    request_id: &str,
) -> (RequestContext, Arc<dyn Fn() + Send + Sync>) { ::tsox_core::fntrace::enter("with_request_id_cancellable"); 
    let cancelled = Arc::new(AtomicBool::new(false));
    let handle = cancelled.clone();
    let ctx = RequestContext {
        locale: ctx.locale,
        request_id: Some(request_id.to_string()),
        cancelled: cancelled.clone(),
    };
    (ctx, Arc::new(move || handle.store(true, Ordering::SeqCst)))
}

#[derive(Clone, Default)]
pub struct InitCompleteChannel {
    done: Arc<(Mutex<bool>, std::sync::Condvar)>,
}

impl InitCompleteChannel {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self::default()
    }

    pub fn close(&self) { ::tsox_core::fntrace::enter("close"); 
        let (lock, cvar) = &*self.done;
        let mut done = lock.lock().unwrap();
        *done = true;
        cvar.notify_all();
    }

    pub fn waiter(&self) -> InitCompleteWaiter { ::tsox_core::fntrace::enter("waiter"); 
        InitCompleteWaiter { done: self.done.clone() }
    }
}

pub struct InitCompleteWaiter {
    done: Arc<(Mutex<bool>, std::sync::Condvar)>,
}

impl InitCompleteWaiter {
    pub fn wait(&self) { ::tsox_core::fntrace::enter("wait"); 
        let (lock, cvar) = &*self.done;
        let mut done = lock.lock().unwrap();
        while !*done {
            done = cvar.wait(done).unwrap();
        }
    }
}

pub struct SyncSet<T: Eq + std::hash::Hash + Clone> {
    inner: Mutex<HashSet<T>>,
}

impl<T: Eq + std::hash::Hash + Clone> Default for SyncSet<T> {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        SyncSet { inner: Mutex::new(HashSet::new()) }
    }
}

impl<T: Eq + std::hash::Hash + Clone> SyncSet<T> {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self::default()
    }

    pub fn add(&self, value: &T) { ::tsox_core::fntrace::enter("add"); 
        self.inner.lock().unwrap().insert(value.clone());
    }

    pub fn has(&self, value: &T) -> bool { ::tsox_core::fntrace::enter("has"); 
        self.inner.lock().unwrap().contains(value)
    }

    pub fn delete(&self, value: &T) { ::tsox_core::fntrace::enter("delete"); 
        self.inner.lock().unwrap().remove(value);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum DiagnosticFlakeLogLevel {
    #[default]
    Off = 0,
    Log = 1,
    Panic = 2,
}

pub const METHOD_INITIALIZE: &str = "initialize";
pub const METHOD_INITIALIZED: &str = "initialized";
pub const METHOD_CANCEL_REQUEST: &str = "$/cancelRequest";
pub const METHOD_WORKSPACE_DID_CHANGE_WATCHED_FILES: &str = "workspace/didChangeWatchedFiles";
pub const METHOD_WORKSPACE_CONFIGURATION: &str = "workspace/configuration";
pub const METHOD_CLIENT_REGISTER_CAPABILITY: &str = "client/registerCapability";
pub const METHOD_CLIENT_UNREGISTER_CAPABILITY: &str = "client/unregisterCapability";
pub const METHOD_WORKSPACE_DIAGNOSTIC_REFRESH: &str = "workspace/diagnostic/refresh";
pub const METHOD_WORKSPACE_INLAY_HINT_REFRESH: &str = "workspace/inlayHint/refresh";
pub const METHOD_WORKSPACE_CODE_LENS_REFRESH: &str = "workspace/codeLens/refresh";
pub const METHOD_WINDOW_WORK_DONE_PROGRESS_CREATE: &str = "window/workDoneProgress/create";
pub const METHOD_TEXT_DOCUMENT_PUBLISH_DIAGNOSTICS: &str = "textDocument/publishDiagnostics";
pub const METHOD_TELEMETRY_EVENT: &str = "telemetry/event";
pub const METHOD_PROGRESS: &str = "$/progress";

pub static TEXT_DOCUMENT_PUBLISH_DIAGNOSTICS_INFO: LazyLock<lsproto::NotificationInfo> =
    LazyLock::new(|| lsproto::NotificationInfo { method: METHOD_TEXT_DOCUMENT_PUBLISH_DIAGNOSTICS.to_string() });
pub static TELEMETRY_EVENT_INFO: LazyLock<lsproto::NotificationInfo> =
    LazyLock::new(|| lsproto::NotificationInfo { method: METHOD_TELEMETRY_EVENT.to_string() });
pub static PROGRESS_INFO: LazyLock<lsproto::NotificationInfo> =
    LazyLock::new(|| lsproto::NotificationInfo { method: METHOD_PROGRESS.to_string() });
pub static WORKSPACE_DIAGNOSTIC_REFRESH_INFO: LazyLock<lsproto::RequestInfo> =
    LazyLock::new(|| lsproto::RequestInfo { method: METHOD_WORKSPACE_DIAGNOSTIC_REFRESH.to_string() });
pub static WORKSPACE_INLAY_HINT_REFRESH_INFO: LazyLock<lsproto::RequestInfo> =
    LazyLock::new(|| lsproto::RequestInfo { method: METHOD_WORKSPACE_INLAY_HINT_REFRESH.to_string() });
pub static WORKSPACE_CODE_LENS_REFRESH_INFO: LazyLock<lsproto::RequestInfo> =
    LazyLock::new(|| lsproto::RequestInfo { method: METHOD_WORKSPACE_CODE_LENS_REFRESH.to_string() });
pub static CLIENT_REGISTER_CAPABILITY_INFO: LazyLock<lsproto::RequestInfo> =
    LazyLock::new(|| lsproto::RequestInfo { method: METHOD_CLIENT_REGISTER_CAPABILITY.to_string() });
pub static CLIENT_UNREGISTER_CAPABILITY_INFO: LazyLock<lsproto::RequestInfo> =
    LazyLock::new(|| lsproto::RequestInfo { method: METHOD_CLIENT_UNREGISTER_CAPABILITY.to_string() });
pub static WORKSPACE_CONFIGURATION_INFO: LazyLock<lsproto::RequestInfo> =
    LazyLock::new(|| lsproto::RequestInfo { method: METHOD_WORKSPACE_CONFIGURATION.to_string() });
pub static WINDOW_WORK_DONE_PROGRESS_CREATE_INFO: LazyLock<lsproto::RequestInfo> =
    LazyLock::new(|| lsproto::RequestInfo { method: METHOD_WINDOW_WORK_DONE_PROGRESS_CREATE.to_string() });

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "workDoneToken", skip_serializing_if = "Option::is_none")]
    pub work_done_token: Option<lsproto::IntegerOrString>,
    #[serde(rename = "processId", skip_serializing_if = "Option::is_none")]
    pub process_id: Option<i64>,
    #[serde(rename = "clientInfo", skip_serializing_if = "Option::is_none")]
    pub client_info: Option<ClientInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(rename = "rootPath", skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,
    #[serde(rename = "rootUri", skip_serializing_if = "Option::is_none")]
    pub root_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<lsproto::ClientCapabilities>,
    #[serde(rename = "initializationOptions", skip_serializing_if = "Option::is_none")]
    pub initialization_options: Option<Value>,
    #[serde(rename = "workspaceFolders", skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<Vec<lsproto::WorkspaceFolder>>,
}

impl crate::mig::m5m::LspNoParams for InitializeParams {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitializationOptions {
    #[serde(rename = "disablePushDiagnostics", skip_serializing_if = "Option::is_none")]
    pub disable_push_diagnostics: Option<bool>,
    #[serde(rename = "codeLensShowLocationsCommandName", skip_serializing_if = "Option::is_none")]
    pub code_lens_show_locations_command_name: Option<String>,
    #[serde(rename = "userPreferences", skip_serializing_if = "Option::is_none")]
    pub user_preferences: Option<Value>,
    #[serde(rename = "enableTelemetry", skip_serializing_if = "Option::is_none")]
    pub enable_telemetry: Option<bool>,
    #[serde(rename = "logVerbosity", skip_serializing_if = "Option::is_none")]
    pub log_verbosity: Option<lsproto::LogVerbosity>,
    #[serde(rename = "runExternalCode", skip_serializing_if = "Option::is_none")]
    pub run_external_code: Option<bool>,
    #[serde(rename = "trackFlakyDiagnostics", skip_serializing_if = "Option::is_none")]
    pub track_flaky_diagnostics: Option<DiagnosticFlakeLogLevel>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancelParams {
    pub id: crate::jsonrpc::jsonrpc::Id,
}

impl Default for CancelParams {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        CancelParams { id: crate::jsonrpc::jsonrpc::Id::Int(0) }
    }
}

impl crate::mig::m5m::LspNoParams for CancelParams {}

impl crate::mig::m5m::LspNoParams for Value {}

#[derive(Debug, Clone, Default)]
pub struct RefreshSupport {
    pub refresh_support: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DynamicRegistrationSupport {
    pub dynamic_registration: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FileOperationsResolvedClientCapabilities {
    pub dynamic_registration: bool,
    pub will_rename: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TextDocumentResolvedClientCapabilities {
    pub synchronization: DynamicRegistrationSupport,
    pub completion: DynamicRegistrationSupport,
    pub hover: DynamicRegistrationSupport,
    pub signature_help: DynamicRegistrationSupport,
    pub declaration: DynamicRegistrationSupport,
    pub definition: DynamicRegistrationSupport,
    pub type_definition: DynamicRegistrationSupport,
    pub implementation: DynamicRegistrationSupport,
    pub references: DynamicRegistrationSupport,
    pub document_highlight: DynamicRegistrationSupport,
    pub document_symbol: DynamicRegistrationSupport,
    pub code_action: DynamicRegistrationSupport,
    pub code_lens: DynamicRegistrationSupport,
    pub formatting: DynamicRegistrationSupport,
    pub range_formatting: DynamicRegistrationSupport,
    pub on_type_formatting: DynamicRegistrationSupport,
    pub rename: DynamicRegistrationSupport,
    pub folding_range: DynamicRegistrationSupport,
    pub selection_range: DynamicRegistrationSupport,
    pub call_hierarchy: DynamicRegistrationSupport,
    pub semantic_tokens: DynamicRegistrationSupport,
    pub linked_editing_range: DynamicRegistrationSupport,
    pub inlay_hint: DynamicRegistrationSupport,
    pub diagnostic: DynamicRegistrationSupport,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceResolvedClientCapabilities {
    pub configuration: bool,
    pub diagnostics: RefreshSupport,
    pub inlay_hint: RefreshSupport,
    pub code_lens: RefreshSupport,
    pub file_operations: FileOperationsResolvedClientCapabilities,
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedClientCapabilities {
    pub workspace: WorkspaceResolvedClientCapabilities,
    pub text_document: TextDocumentResolvedClientCapabilities,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DidChangeWatchedFilesRegistrationOptions {
    pub watchers: Vec<lsproto::FileSystemWatcher>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegisterOptions {
    #[serde(rename = "didChangeWatchedFiles", skip_serializing_if = "Option::is_none")]
    pub workspace_did_change_watched_files: Option<DidChangeWatchedFilesRegistrationOptions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registration {
    pub id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_options: Option<RegisterOptions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistrationParams {
    pub registrations: Vec<Registration>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Unregistration {
    pub id: String,
    pub method: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnregistrationParams {
    pub unregisterations: Vec<Unregistration>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigurationItem {
    #[serde(rename = "scopeUri", skip_serializing_if = "Option::is_none")]
    pub scope_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigurationParams {
    pub items: Vec<ConfigurationItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestFailureTelemetryProperties {
    #[serde(rename = "errorCode")]
    pub error_code: String,
    #[serde(rename = "requestMethod")]
    pub request_method: String,
    #[serde(rename = "stack")]
    pub stack: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestFailureTelemetryEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<RequestFailureTelemetryProperties>,
}

pub struct PendingClientRequest {
    pub req: Arc<lsproto::RequestMessage>,
    pub cancel: Arc<dyn Fn() + Send + Sync>,
}

pub type IntegerOrString = lsproto::IntegerOrString;

pub struct Server {
    pub r: Arc<dyn Reader>,
    pub w: Arc<dyn Writer>,
    pub background_ctx: BackgroundCtx,

    pub stderr: Arc<dyn std::io::Write + Send + Sync>,

    pub logger: Arc<Logger>,
    pub init_started: AtomicBool,
    pub client_seq: AtomicI32,
    pub request_queue: Arc<crate::lsp::dynamic_queue::DynamicQueue<lsproto::RequestMessage>>,
    pub outgoing_queue: Arc<crate::lsp::dynamic_queue::DynamicQueue<lsproto::Message>>,
    pub pending_client_requests: Mutex<HashMap<crate::jsonrpc::jsonrpc::Id, PendingClientRequest>>,
    pub pending_server_requests:
        Mutex<HashMap<crate::jsonrpc::jsonrpc::Id, std::sync::mpsc::Sender<lsproto::ResponseMessage>>>,

    pub cwd: String,
    pub fs: Arc<dyn tsox_tsoptions::vfs::FS>,
    pub default_library_path: String,
    pub typings_location: String,

    pub initialize_params: Option<Box<InitializeParams>>,
    pub initialization_options: Box<InitializationOptions>,
    pub client_capabilities: ResolvedClientCapabilities,
    pub position_encoding: lsproto::PositionEncodingKind,
    pub locale_lock: RwLock<Locale>,
    pub init_locale: Locale,

    pub watch_enabled: bool,
    pub telemetry_enabled: AtomicBool,
    pub watcher_id: AtomicU32,
    pub watchers: SyncSet<crate::project::watch::WatcherID>,

    pub content_mapper_registration_lock: Mutex<()>,
    pub content_mapper_extensions_registered: bool,
    pub builtin_watcher: Option<crate::lsp::lspwatcher::Watcher>,

    pub last_request_time_ms: AtomicI64,

    pub session: Option<Arc<Session>>,

    pub api_sessions: Mutex<HashMap<String, Arc<api::session::Session>>>,

    pub client: Option<Box<dyn crate::project::client::Client>>,

    pub init_complete: InitCompleteChannel,

    pub compiler_options_for_inferred_projects: Option<Box<tsox_core::core::compiler_options::CompilerOptions>>,
    pub parse_cache: Option<Arc<crate::project::parse_cache::ParseCache>>,

    pub npm_install: Option<Arc<dyn Fn(&str, &[String]) -> Result<Vec<u8>, String> + Send + Sync>>,
    pub spawn: Option<Arc<dyn Fn(&[String], &str) -> Result<SpawnedProcess, String> + Send + Sync>>,

    pub cpu_profiler: tsox_core::pprof::mig::m6a::CpuProfiler,

    pub progress_delay: Duration,
    pub project_progress: Option<Arc<ProjectLoadingProgress>>,

    pub start_watchdog: Option<Arc<dyn Fn(i32) + Send + Sync>>,

    pub flake_logging: DiagnosticFlakeLogLevel,
}

pub struct SpawnedProcess {
    pub stdout: Box<dyn std::io::Read + Send + Sync>,
    pub stdin: Box<dyn std::io::Write + Send + Sync>,
    pub close: Box<dyn FnOnce() + Send + Sync>,
}

impl Server {
    pub fn new(
        opts: &ServerOptions,
    ) -> Arc<Server> { ::tsox_core::fntrace::enter("new"); 
        if opts.cwd.is_empty() {
            panic!("Cwd is required");
        }
        let s = Arc::new(Server {
            r: opts.r.clone().expect("In is required"),
            w: opts.w.clone().expect("Out is required"),
            background_ctx: BackgroundCtx::default(),
            stderr: opts.stderr.clone(),
            logger: Arc::new(Logger::new()),
            init_started: AtomicBool::new(false),
            client_seq: AtomicI32::new(0),
            request_queue: crate::lsp::dynamic_queue::DynamicQueue::new(),
            outgoing_queue: crate::lsp::dynamic_queue::DynamicQueue::new(),
            pending_client_requests: Mutex::new(HashMap::new()),
            pending_server_requests: Mutex::new(HashMap::new()),
            cwd: opts.cwd.clone(),
            fs: opts.fs.clone(),
            default_library_path: opts.default_library_path.clone(),
            typings_location: opts.typings_location.clone(),
            initialize_params: None,
            initialization_options: Box::new(InitializationOptions::default()),
            client_capabilities: ResolvedClientCapabilities::default(),
            position_encoding: lsproto::POSITION_ENCODING_UTF16.to_string(),
            locale_lock: RwLock::new(Locale::default_locale()),
            init_locale: Locale::default_locale(),
            watch_enabled: false,
            telemetry_enabled: AtomicBool::new(false),
            watcher_id: AtomicU32::new(0),
            watchers: SyncSet::new(),
            content_mapper_registration_lock: Mutex::new(()),
            content_mapper_extensions_registered: false,
            builtin_watcher: None,
            last_request_time_ms: AtomicI64::new(0),
            session: None,
            api_sessions: Mutex::new(HashMap::new()),
            client: None,
            init_complete: InitCompleteChannel::new(),
            compiler_options_for_inferred_projects: None,
            parse_cache: opts.parse_cache.clone(),
            npm_install: opts.npm_install.clone(),
            spawn: opts.spawn.clone(),
            cpu_profiler: tsox_core::pprof::mig::m6a::CpuProfiler::new(),
            progress_delay: opts.progress_delay,
            project_progress: None,
            start_watchdog: opts.set_parent_process_id.clone(),
            flake_logging: DiagnosticFlakeLogLevel::default(),
        });
        s
    }

    pub fn session(&self) -> Option<&Arc<Session>> { ::tsox_core::fntrace::enter("session"); 
        self.session.as_ref()
    }

    pub fn init_complete(&self) -> InitCompleteWaiter { ::tsox_core::fntrace::enter("init_complete"); 
        self.init_complete.waiter()
    }

    pub fn get_locale(&self) -> Locale { ::tsox_core::fntrace::enter("get_locale"); 
        self.locale_lock.read().unwrap().clone()
    }

    pub fn set_locale(&self, locale_string: &str) { ::tsox_core::fntrace::enter("set_locale"); 
        let new_locale = if locale_string != "auto" {
            let Some(parsed) = Locale::parse(locale_string) else {
                return;
            };
            parsed
        } else {
            self.init_locale.clone()
        };
        *self.locale_lock.write().unwrap() = new_locale;
    }

    pub fn progress_start(&self, message: &Message, args: &[String]) { ::tsox_core::fntrace::enter("progress_start"); 
        if let Some(progress) = &self.project_progress {
            progress.start(message.clone(), args.to_vec());
        }
    }

    pub fn progress_finish(&self, message: &Message, args: &[String]) { ::tsox_core::fntrace::enter("progress_finish"); 
        if let Some(progress) = &self.project_progress {
            progress.finish(message.clone(), args.to_vec());
        }
    }

    pub fn publish_diagnostics(&self, params: &lsproto::PublishDiagnosticsParams) -> LspResult<()> { ::tsox_core::fntrace::enter("publish_diagnostics"); 
        let value = serde_json::to_value(params)
            .map_err(|e| LspError::new(ErrorCode::InternalError, format!("failed to marshal message: {e}")))?;
        crate::mig::m5n_2::send_notification(self, &*TEXT_DOCUMENT_PUBLISH_DIAGNOSTICS_INFO, value)
    }

    pub fn send_telemetry(&self, telemetry: &lsproto::TelemetryEvent) -> LspResult<()> { ::tsox_core::fntrace::enter("send_telemetry"); 
        if !self.telemetry_enabled.load(Ordering::SeqCst) {
            panic!("SendTelemetry called with telemetry disabled");
        }
        let value = serde_json::to_value(telemetry)
            .map_err(|e| LspError::new(ErrorCode::InternalError, format!("failed to marshal message: {e}")))?;
        crate::mig::m5n_2::send_notification(self, &*TELEMETRY_EVENT_INFO, value)
    }

    pub fn is_active(&self) -> bool { ::tsox_core::fntrace::enter("is_active"); 
        let last = self.last_request_time_ms.load(Ordering::SeqCst);
        if last == 0 {
            return true;
        }
        let last_ms = UNIX_EPOCH + Duration::from_millis(last as u64);
        match SystemTime::now().duration_since(last_ms) {
            Ok(elapsed) => elapsed <= Duration::from_secs(60),
            Err(_) => true,
        }
    }

    pub fn refresh_diagnostics(&self) -> LspResult<()> { ::tsox_core::fntrace::enter("refresh_diagnostics"); 
        if !self.client_capabilities.workspace.diagnostics.refresh_support {
            return Ok(());
        }
        crate::mig::m5n_2::send_client_request_fire_and_forget(self, &*WORKSPACE_DIAGNOSTIC_REFRESH_INFO, lsproto::NoParams {})
            .map_err(|e| LspError::new(e.code, format!("failed to refresh diagnostics: {}", e.message)))?;
        Ok(())
    }

    pub fn refresh_inlay_hints(&self) -> LspResult<()> { ::tsox_core::fntrace::enter("refresh_inlay_hints"); 
        if !self.client_capabilities.workspace.inlay_hint.refresh_support {
            return Ok(());
        }
        crate::mig::m5n_2::send_client_request_fire_and_forget(self, &*WORKSPACE_INLAY_HINT_REFRESH_INFO, lsproto::NoParams {})
            .map_err(|e| LspError::new(e.code, format!("failed to refresh inlay hints: {}", e.message)))?;
        Ok(())
    }

    pub fn refresh_code_lens(&self) -> LspResult<()> { ::tsox_core::fntrace::enter("refresh_code_lens"); 
        if !self.client_capabilities.workspace.code_lens.refresh_support {
            return Ok(());
        }
        crate::mig::m5n_2::send_client_request_fire_and_forget(self, &*WORKSPACE_CODE_LENS_REFRESH_INFO, lsproto::NoParams {})
            .map_err(|e| LspError::new(e.code, format!("failed to refresh code lens: {}", e.message)))?;
        Ok(())
    }

    pub fn watch_files(
        &self,
        id: &crate::project::watch::WatcherID,
        watchers: &[lsproto::FileSystemWatcher],
    ) -> LspResult<()> { ::tsox_core::fntrace::enter("watch_files"); 
        if let Some(builtin) = &self.builtin_watcher {
            builtin
                .watch_files(&id.to_string(), watchers)
                .map_err(|e| LspError::new(ErrorCode::InternalError, format!("failed to register file watcher: {e}")))?;
            self.watchers.add(id);
            return Ok(());
        }
        crate::mig::m5n_2::send_client_request(
            self,
            &*CLIENT_REGISTER_CAPABILITY_INFO,
            RegistrationParams {
                registrations: vec![Registration {
                    id: id.to_string(),
                    method: METHOD_WORKSPACE_DID_CHANGE_WATCHED_FILES.to_string(),
                    register_options: Some(RegisterOptions {
                        workspace_did_change_watched_files: Some(DidChangeWatchedFilesRegistrationOptions {
                            watchers: watchers.to_vec(),
                        }),
                    }),
                }],
            },
        )
        .map_err(|e| LspError::new(e.code, format!("failed to register file watcher: {}", e.message)))?;
        self.watchers.add(id);
        Ok(())
    }

    pub fn unwatch_files(&self, id: &crate::project::watch::WatcherID) -> LspResult<()> { ::tsox_core::fntrace::enter("unwatch_files"); 
        if let Some(builtin) = &self.builtin_watcher {
            if !self.watchers.has(id) {
                return Err(LspError::new(ErrorCode::InternalError, format!("no file watcher exists with ID {id}")));
            }
            builtin
                .unwatch_files(&id.to_string())
                .map_err(|e| LspError::new(ErrorCode::InternalError, format!("failed to unregister file watcher: {e}")))?;
            self.watchers.delete(id);
            return Ok(());
        }
        if self.watchers.has(id) {
            crate::mig::m5n_2::send_client_request(
                self,
                &*CLIENT_UNREGISTER_CAPABILITY_INFO,
                UnregistrationParams {
                    unregisterations: vec![Unregistration {
                        id: id.to_string(),
                        method: METHOD_WORKSPACE_DID_CHANGE_WATCHED_FILES.to_string(),
                    }],
                },
            )
            .map_err(|e| LspError::new(e.code, format!("failed to unregister file watcher: {}", e.message)))?;
            self.watchers.delete(id);
            return Ok(());
        }
        Err(LspError::new(ErrorCode::InternalError, format!("no file watcher exists with ID {id}")))
    }

    pub fn request_configuration(&self) -> LspResult<lsutil::UserPreferences> { ::tsox_core::fntrace::enter("request_configuration"); 
        let caps = &self.client_capabilities;
        if !caps.workspace.configuration {
            if let Some(user_prefs) = &self.initialization_options.user_preferences {
                let mut items = serde_json::Map::new();
                items.insert("js/ts".to_string(), user_prefs.clone());
                return Ok(crate::ls::lsutil_user_preferences_preferences::parse_user_preferences(&items));
            }
            return Ok(crate::ls::lsutil_user_preferences_preferences::new_default_user_preferences());
        }
        let configs = crate::mig::m5n_2::send_client_request(
            self,
            &*WORKSPACE_CONFIGURATION_INFO,
            ConfigurationParams {
                items: vec![
                    ConfigurationItem { section: Some("js/ts".to_string()), ..Default::default() },
                    ConfigurationItem { section: Some("typescript".to_string()), ..Default::default() },
                    ConfigurationItem { section: Some("javascript".to_string()), ..Default::default() },
                    ConfigurationItem { section: Some("editor".to_string()), ..Default::default() },
                ],
            },
        )
        .map_err(|e| LspError::new(e.code, format!("configure request failed: {}", e.message)))?;
        let mut config_map = serde_json::Map::new();
        if let Some(configs) = configs.as_array() {
            for (i, config) in configs.iter().enumerate() {
                let key = match i {
                    0 => "js/ts",
                    1 => "typescript",
                    2 => "javascript",
                    3 => "editor",
                    _ => continue,
                };
                config_map.insert(key.to_string(), config.clone());
            }
        }
        Ok(crate::ls::lsutil_user_preferences_preferences::parse_user_preferences(&config_map))
    }

    pub fn npm_install(&self, cwd: &str, args: &[String]) -> Result<Vec<u8>, String> { ::tsox_core::fntrace::enter("npm_install"); 
        match &self.npm_install {
            Some(f) => f(cwd, args),
            None => Err("NpmInstall not configured".to_string()),
        }
    }

    pub fn set_compiler_options_for_inferred_projects(
        &mut self,
        options: &tsox_core::core::compiler_options::CompilerOptions,
    ) { ::tsox_core::fntrace::enter("set_compiler_options_for_inferred_projects"); 
        self.compiler_options_for_inferred_projects = Some(Box::new(options.clone()));
        if let Some(session) = &self.session {
            session.did_change_compiler_options_for_inferred_projects(Some(options.clone()));
        }
    }
}

pub struct ServerOptions {
    pub r: Option<Arc<dyn Reader>>,
    pub w: Option<Arc<dyn Writer>>,
    pub stderr: Arc<dyn std::io::Write + Send + Sync>,
    pub cwd: String,
    pub fs: Arc<dyn tsox_tsoptions::vfs::FS>,
    pub default_library_path: String,
    pub typings_location: String,
    pub parse_cache: Option<Arc<crate::project::parse_cache::ParseCache>>,
    pub npm_install: Option<Arc<dyn Fn(&str, &[String]) -> Result<Vec<u8>, String> + Send + Sync>>,
    pub spawn: Option<Arc<dyn Fn(&[String], &str) -> Result<SpawnedProcess, String> + Send + Sync>>,
    pub progress_delay: Duration,
    pub set_parent_process_id: Option<Arc<dyn Fn(i32) + Send + Sync>>,
}

pub fn new_project_loading_progress(server: &Arc<Server>, delay: Duration) -> Arc<ProjectLoadingProgress> { ::tsox_core::fntrace::enter("new_project_loading_progress"); 
    let reporter = ServerProgressReporter::new(server);
    new_project_loading_progress_from_reporter(Arc::new(reporter), delay)
}

pub fn new_project_loading_progress_from_reporter(
    reporter: Arc<dyn ProgressReporter>,
    delay: Duration,
) -> Arc<ProjectLoadingProgress> { ::tsox_core::fntrace::enter("new_project_loading_progress_from_reporter"); 
    ProjectLoadingProgress::new(reporter, delay)
}

pub struct ServerProgressReporter {
    server: Arc<Server>,
}

impl ServerProgressReporter {
    pub fn new(server: &Arc<Server>) -> Self { ::tsox_core::fntrace::enter("new"); 
        ServerProgressReporter { server: server.clone() }
    }
}

impl ProgressReporter for ServerProgressReporter {
    fn is_done(&self) -> bool { ::tsox_core::fntrace::enter("is_done"); 
        self.server.background_ctx.is_done()
    }

    fn localize(&self, msg: &Message, args: &[String]) -> String { ::tsox_core::fntrace::enter("localize"); 
        let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        msg.localize(&self.server.get_locale(), &args)
    }

    fn create_work_done_progress(&self, token: &str) { ::tsox_core::fntrace::enter("create_work_done_progress"); 
        let _ = crate::mig::m5n_2::send_client_request_fire_and_forget(
            &self.server,
            &*WINDOW_WORK_DONE_PROGRESS_CREATE_INFO,
            lsproto::WorkDoneProgressCreateParams {
                token: lsproto::IntegerOrString { integer: None, string: Some(token.to_string()) },
            },
        );
    }

    fn send_progress(&self, token: &str, value: lsproto::WorkDoneProgressBeginOrReportOrEnd) { ::tsox_core::fntrace::enter("send_progress"); 
        let _ = crate::mig::m5n_2::send_notification(
            &self.server,
            &*PROGRESS_INFO,
            lsproto::ProgressParams {
                token: lsproto::IntegerOrString { integer: None, string: Some(token.to_string()) },
                value,
            },
        );
    }
}
