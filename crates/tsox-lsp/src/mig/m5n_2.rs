#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde::ser::SerializeStruct;

use super::m5n::{
    CancelParams, ErrorCode, InitializeParams, InitializationOptions, LspError, LspResult,
    PendingClientRequest, RequestContext, Server, UserFacingRequestFailedError, METHOD_INITIALIZE,
    METHOD_INITIALIZED,
};
use crate::jsonrpc::jsonrpc::{Id as JsonrpcId, MessageKind, ResponseError};
use crate::ls::language_service::LanguageService;
use crate::lsp::lsproto;
use crate::lsp::stack_sanitizer::sanitize_stack_trace;
use crate::mig::m5m;
use crate::project::project::Project;
use crate::project::session::Session;
use tsox_core::collections::set::Set;
use tsox_core::tspath::Path;

impl serde::Serialize for super::m5n::RefreshSupport {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        let mut s = serializer.serialize_struct("RefreshSupport", 1)?;
        s.serialize_field("refreshSupport", &self.refresh_support)?;
        s.end()
    }
}

impl serde::Serialize for super::m5n::WorkspaceResolvedClientCapabilities {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        let mut s = serializer.serialize_struct("WorkspaceResolvedClientCapabilities", 4)?;
        s.serialize_field("configuration", &self.configuration)?;
        s.serialize_field("diagnostics", &self.diagnostics)?;
        s.serialize_field("inlayHint", &self.inlay_hint)?;
        s.serialize_field("codeLens", &self.code_lens)?;
        s.end()
    }
}

impl serde::Serialize for super::m5n::ResolvedClientCapabilities {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        let mut s = serializer.serialize_struct("ResolvedClientCapabilities", 1)?;
        s.serialize_field("workspace", &self.workspace)?;
        s.end()
    }
}

impl Server {
    pub fn handle_initialize(&mut self, params: &InitializeParams) -> LspResult<serde_json::Value> { ::tsox_core::fntrace::enter("handle_initialize"); 
        if self.initialize_params.is_some() {
            return Err(LspError::new(ErrorCode::InvalidRequest, "server already initialized"));
        }
        self.init_started.store(true, Ordering::SeqCst);
        self.initialize_params = Some(Box::new(params.clone()));
        if let Some(initialization_options) = &params.initialization_options {
            if let Ok(options) =
                serde_json::from_value::<InitializationOptions>(initialization_options.clone())
            {
                if let Some(verbosity) = &options.log_verbosity {
                    if crate::lsp::logger::is_valid_log_verbosity(verbosity.clone()) {
                        self.logger.set_verbosity(verbosity.clone());
                    }
                }
                if let Some(flake) = options.track_flaky_diagnostics {
                    self.flake_logging = flake;
                }
            }
        }
        self.logger.info(&format!(
            "Resolved client capabilities: {}",
            serde_json::to_string_pretty(&self.client_capabilities).unwrap_or_default()
        ));
        self.position_encoding = lsproto::POSITION_ENCODING_UTF16.to_string();
        if let Some(locale_string) = &params.locale {
            if let Some(parsed) = tsox_core::locale::Locale::parse(locale_string) {
                *self.locale_lock.write().unwrap() = parsed.clone();
                self.init_locale = parsed;
            }
        }
        if let Some(start_watchdog) = &self.start_watchdog {
            if let Some(process_id) = params.process_id {
                start_watchdog(process_id as i32);
            }
        }
        Ok(serde_json::json!({
            "capabilities": {
                "positionEncoding": "utf-16",
                "textDocumentSync": {
                    "openClose": true,
                    "change": 2,
                    "save": { "boolean": true },
                },
                "hoverProvider": true,
                "definitionProvider": true,
                "typeDefinitionProvider": true,
                "referencesProvider": true,
                "implementationProvider": true,
                "diagnosticProvider": {
                    "identifier": "typescript",
                    "interFileDependencies": true,
                },
                "completionProvider": { "resolveProvider": true },
                "signatureHelpProvider": {},
                "documentFormattingProvider": true,
                "documentRangeFormattingProvider": true,
                "documentOnTypeFormattingProvider": { "firstTriggerCharacter": "{" },
                "workspaceSymbolProvider": true,
                "documentSymbolProvider": true,
                "foldingRangeProvider": true,
                "renameProvider": { "prepareProvider": true },
                "documentHighlightProvider": true,
                "selectionRangeProvider": true,
                "linkedEditingRangeProvider": true,
                "inlayHintProvider": true,
                "codeLensProvider": { "resolveProvider": true },
                "codeActionProvider": {},
                "callHierarchyProvider": true,
                "semanticTokensProvider": {
                    "full": true,
                    "range": true,
                },
            },
            "serverInfo": { "name": "typescript" },
        }))
    }

    pub fn read_loop(&mut self) -> LspResult<()> { ::tsox_core::fntrace::enter("read_loop"); 
        loop {
            if self.background_ctx.is_done() {
                return Err(LspError::new(ErrorCode::EOF, "context canceled"));
            }
            let msg = match self.read() {
                Ok(msg) => msg,
                Err(err) => {
                    if err.is_code(ErrorCode::InvalidRequest) || err.is_code(ErrorCode::InvalidParams) {
                        let mut id = None;
                        if err.is_code(ErrorCode::InvalidParams) {
                            if let Some(req) = &err.request {
                                if req.id.is_some() {
                                    id = req.id.clone();
                                }
                            }
                        }
                        self.send_error(id, &err)?;
                        continue;
                    }
                    return Err(err);
                }
            };

            if self.initialize_params.is_none() && msg.kind == MessageKind::Request {
                let req = msg.as_request();
                if req.method == METHOD_INITIALIZE {
                    let params = match unmarshal_params_or_invalid::<InitializeParams>(req) {
                        Ok(params) => params,
                        Err(err) => {
                            self.send_error(req.id.clone(), &err)?;
                            continue;
                        }
                    };
                    let resp = self.handle_initialize(&params)?;
                    self.send_result(&req.id, &resp)?;
                } else {
                    self.send_error(
                        req.id.clone(),
                        &LspError::new(ErrorCode::ServerNotInitialized, "server not initialized"),
                    )?;
                }
                continue;
            }

            if msg.kind == MessageKind::Response {
                let resp = msg.as_response();
                let mut pending = self.pending_server_requests.lock().unwrap();
                if let Some(id) = &resp.id {
                    if let Some(chan) = pending.remove(id) {
                        let _ = chan.send(resp.clone());
                    }
                }
            } else {
                let req = msg.as_request();
                if req.method == super::m5n::METHOD_CANCEL_REQUEST {
                    if let Ok(params) = m5m::unmarshal_params::<CancelParams>(req.params.as_ref()) {
                        self.cancel_request(&params.id);
                    }
                } else {
                    self.request_queue
                        .put(req.clone())
                        .map_err(|_| LspError::new(ErrorCode::InternalError, "request queue closed"))?;
                }
            }
        }
    }

    pub fn cancel_request(&self, raw_id: &JsonrpcId) { ::tsox_core::fntrace::enter("cancel_request"); 
        let mut pending = self.pending_client_requests.lock().unwrap();
        if let Some(pending_req) = pending.remove(raw_id) {
            (pending_req.cancel)();
        }
    }

    pub fn read(&self) -> LspResult<lsproto::Message> { ::tsox_core::fntrace::enter("read"); 
        self.r.read()
    }

    pub fn dispatch_loop(self: &Arc<Self>) -> LspResult<()> { ::tsox_core::fntrace::enter("dispatch_loop"); 
        loop {
            let req = match self.request_queue.get() {
                Some(req) => req,
                None => return Err(LspError::new(ErrorCode::EOF, "request queue closed")),
            };
            self.last_request_time_ms.store(now_millis(), Ordering::SeqCst);
            let request_ctx = RequestContext { locale: self.get_locale(), ..Default::default() };
            let mut cancel: Option<Arc<dyn Fn() + Send + Sync>> = None;
            if let Some(id) = &req.id {
                let (ctx, cancel_fn) = super::m5n::with_request_id_cancellable(request_ctx, &id.as_string());
                cancel = Some(cancel_fn.clone());
                self.pending_client_requests.lock().unwrap().insert(
                    id.clone(),
                    PendingClientRequest { req: Arc::new(req.clone()), cancel: cancel_fn },
                );
            }

            let result = self.handle_request_or_notification(&req);
            match result {
                Err(err) => {
                    self.handle_dispatch_error(&req, &err)?;
                    self.remove_request(&req, &cancel);
                }
                Ok(Some(do_async_work)) => {
                    let server = self.clone();
                    let server_req = req.clone();
                    let server_cancel = cancel.clone();
                    std::thread::spawn(move || {
                        let work_result =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(do_async_work));
                        match work_result {
                            Ok(Ok(())) => {}
                            Ok(Err(ls_err)) => {
                                let _ = server.handle_dispatch_error(&server_req, &ls_err);
                            }
                            Err(payload) => {
                                server.recover(&server_req, &payload);
                            }
                        }
                        server.remove_request(&server_req, &server_cancel);
                    });
                }
                Ok(None) => {
                    self.remove_request(&req, &cancel);
                }
            }
        }
    }

    fn handle_dispatch_error(&self, req: &lsproto::RequestMessage, err: &LspError) -> LspResult<()> { ::tsox_core::fntrace::enter("handle_dispatch_error"); 
        if err.is_code(ErrorCode::RequestCancelled) {
            self.send_error(req.id.clone(), &LspError::new(ErrorCode::RequestCancelled, "request cancelled"))
        } else if err.is_code(ErrorCode::EOF) {
            Ok(())
        } else {
            self.send_error(req.id.clone(), err)
        }
    }

    fn remove_request(&self, req: &lsproto::RequestMessage, cancel: &Option<Arc<dyn Fn() + Send + Sync>>) { ::tsox_core::fntrace::enter("remove_request"); 
        if let Some(id) = &req.id {
            if let Some(cancel_fn) = cancel {
                (cancel_fn.clone())();
            }
            self.pending_client_requests.lock().unwrap().remove(id);
        }
    }

    pub fn handle_request_or_notification(
        &self,
        req: &lsproto::RequestMessage,
    ) -> LspResult<Option<Box<dyn FnOnce() -> LspResult<()> + Send>>> { ::tsox_core::fntrace::enter("handle_request_or_notification"); 
        let handler = handlers().get(&req.method);
        if let Some(handler) = handler {
            let start = std::time::Instant::now();
            let do_async_work = handler(self, req);
            let id_str = req.id.as_ref().map(|id| format!(" ({})", id.as_string())).unwrap_or_default();
            match do_async_work {
                Err(err) => {
                    if let Some((resp, true)) = content_mapper_fallback_response(&req.method, &err) {
                        if !self.logger.is_tracing() {
                            self.logger.info(&format!("handled method '{}'{} in {:?}", req.method, id_str, start.elapsed()));
                        }
                        self.send_result(&req.id, &resp)?;
                        return Ok(None);
                    }
                    if !is_user_facing_request_failed_error(&err) {
                        self.logger
                            .error(&format!("error handling method '{}'{}: {}", req.method, id_str, err.message));
                    } else if !self.logger.is_tracing() {
                        self.logger.info(&format!("handled method '{}'{} in {:?}", req.method, id_str, start.elapsed()));
                    }
                    return Err(err);
                }
                Ok(Some(do_async_work)) => {
                    let logger = self.logger.clone();
                    let method = req.method.clone();
                    return Ok(Some(Box::new(move || {
                        let async_work_err = do_async_work();
                        let is_user_facing = async_work_err
                            .as_ref()
                            .err()
                            .map(is_user_facing_request_failed_error)
                            .unwrap_or(false);
                        let is_real_error = async_work_err.is_err() && !is_user_facing;
                        if is_real_error {
                            logger
                                .info(&format!("error handling method '{}'{} in {:?}", method, id_str, start.elapsed()));
                        } else if !logger.is_tracing() {
                            logger.info(&format!("handled method '{}'{} in {:?}", method, id_str, start.elapsed()));
                        }
                        async_work_err
                    })));
                }
                Ok(None) => {
                    if !self.logger.is_tracing() {
                        self.logger.info(&format!("handled method '{}'{} in {:?}", req.method, id_str, start.elapsed()));
                    }
                    return Ok(None);
                }
            }
        }
        self.logger.warn(&format!("unknown method '{}'", req.method));
        if let Some(id) = &req.id {
            self.send_error(
                Some(id.clone()),
                &LspError::new(ErrorCode::InvalidRequest, "unknown method"),
            )?;
        }
        Ok(None)
    }

    pub fn recover(&self, req: &lsproto::RequestMessage, payload: &(dyn std::any::Any + Send)) { ::tsox_core::fntrace::enter("recover"); 
        let panic_info = if let Some(s) = payload.downcast_ref::<&str>() {
            Some((*s).to_string())
        } else {
            payload.downcast_ref::<String>().cloned()
        };
        let Some(panic_info) = panic_info else {
            return;
        };
        let stack = std::backtrace::Backtrace::force_capture().to_string();
        self.logger
            .error(&format!("panic handling request {}: {}\n{}", req.method, panic_info, stack));
        if let Some(id) = &req.id {
            let _ = self.send_error(
                Some(id.clone()),
                &LspError::new(
                    ErrorCode::InternalError,
                    format!("panic handling request {}: {}", req.method, panic_info),
                ),
            );
        } else {
            self.logger.error(&format!("unhandled panic in notification {} {}", req.method, panic_info));
        }
        if self.telemetry_enabled.load(Ordering::SeqCst) {
            let event = super::m5n::RequestFailureTelemetryEvent {
                properties: Some(super::m5n::RequestFailureTelemetryProperties {
                    error_code: ErrorCode::InternalError.to_string(),
                    request_method: req.method.replace('/', "."),
                    stack: sanitize_stack_trace(&stack),
                }),
            };
            let _ = send_notification(self, &*super::m5n::TELEMETRY_EVENT_INFO, &event);
        }
    }

    pub fn get_language_service_and_cross_project_orchestrator<'a>(
        &'a self,
        uri: &lsproto::DocumentUri,
        req: &'a lsproto::RequestMessage,
    ) -> LspResult<(Arc<LanguageService>, Option<CrossProjectOrchestrator<'a>>)> { ::tsox_core::fntrace::enter("get_language_service_and_cross_project_orchestrator"); 
        let resolved = self
            .session
            .as_ref()
            .unwrap()
            .get_language_service_and_projects_for_file(uri);
        match resolved {
            Ok((default_project, default_ls, all_projects)) => Ok((
                default_ls,
                Some(CrossProjectOrchestrator {
                    server: self,
                    req,
                    default_project: Some((*default_project).clone()),
                    all_projects,
                }),
            )),
            Err(err) => Err(LspError::new(ErrorCode::InternalError, err)),
        }
    }
}

pub struct CrossProjectOrchestrator<'a> {
    pub server: &'a Server,
    pub req: &'a lsproto::RequestMessage,
    pub default_project: Option<Project>,
    pub all_projects: Vec<Project>,
}

impl<'a> CrossProjectOrchestrator<'a> {
    pub fn get_default_project(&self) -> Option<&Project> { ::tsox_core::fntrace::enter("get_default_project"); 
        self.default_project.as_ref()
    }

    pub fn get_all_projects_for_initial_request(&self) -> &[Project] { ::tsox_core::fntrace::enter("get_all_projects_for_initial_request"); 
        &self.all_projects
    }

    pub fn get_language_service_for_project_with_file(
        &self,
        p: &'a Project,
        uri: &str,
    ) -> Option<LanguageService> { ::tsox_core::fntrace::enter("get_language_service_for_project_with_file"); 
        self.server
            .session
            .as_ref()
            .unwrap()
            .get_language_service_for_project_with_file(p, &lsproto::DocumentUri(uri.to_string()))
    }

    pub fn get_projects_for_file(&self, uri: &str) -> LspResult<Vec<Arc<Project>>> { ::tsox_core::fntrace::enter("get_projects_for_file"); 
        let projects = self
            .server
            .session
            .as_ref()
            .unwrap()
            .get_projects_for_file(&lsproto::DocumentUri(uri.to_string()));
        Ok(projects.into_iter().map(Arc::new).collect())
    }

    pub fn get_projects_loading_project_tree(
        &self,
        requested_project_trees: &Set<Path>,
    ) -> Vec<Arc<crate::project::snapshot::Snapshot>> { ::tsox_core::fntrace::enter("get_projects_loading_project_tree"); 
        let mut result = Vec::new();
        self.server
            .session
            .as_ref()
            .unwrap()
            .with_snapshot_loading_project_tree(
                requested_project_trees.iter().cloned().collect(),
                &mut |snapshot: &Arc<crate::project::snapshot::Snapshot>| {
                    result.push(snapshot.clone());
                },
            );
        result
    }
}

fn content_mapper_fallback_response(method: &str, err: &LspError) -> Option<(serde_json::Value, bool)> { ::tsox_core::fntrace::enter("content_mapper_fallback_response"); 
    if !err.is_code(ErrorCode::NoProjectForUnknownScriptKind) {
        return None;
    }
    match method {
        "textDocument/diagnostic" => Some((
            serde_json::json!({
                "kind": "full",
                "items": [],
            }),
            true,
        )),
        "textDocument/hover"
        | "textDocument/signatureHelp"
        | "textDocument/definition"
        | "textDocument/typeDefinition"
        | "textDocument/implementation"
        | "textDocument/references"
        | "textDocument/documentHighlight"
        | "textDocument/completion"
        | "textDocument/rename" => Some((serde_json::Value::Null, true)),
        _ => None,
    }
}

fn unmarshal_params_or_invalid<T: DeserializeOwned + m5m::LspNoParams>(
    req: &lsproto::RequestMessage,
) -> LspResult<T> { ::tsox_core::fntrace::enter("unmarshal_params_or_invalid"); 
    m5m::unmarshal_params::<T>(req.params.as_ref()).map_err(|e| LspError {
        code: ErrorCode::InvalidParams,
        message: e,
        request: Some(req.clone()),
    })
}

pub fn register_notification_handler<Req>(
    handlers: &mut HandlerMap,
    method: lsproto::Method,
    fn_: Arc<dyn Fn(&Server, &Req) -> LspResult<()> + Send + Sync>,
) where
    Req: DeserializeOwned + m5m::LspNoParams + 'static,
{ ::tsox_core::fntrace::enter("register_notification_handler"); 
    handlers.insert(
        method,
        Arc::new(move |s: &Server, req: &lsproto::RequestMessage| {
            if s.session.is_none() && req.method != METHOD_INITIALIZED {
                return Err(LspError::new(ErrorCode::ServerNotInitialized, "server not initialized"));
            }
            let params = unmarshal_params_or_invalid::<Req>(req)?;
            fn_(s, &params)?;
            Ok(None)
        }),
    );
}

pub fn register_request_handler<Req, Resp>(
    handlers: &mut HandlerMap,
    method: lsproto::Method,
    fn_: Arc<dyn Fn(&Server, &Req, &lsproto::RequestMessage) -> LspResult<Resp> + Send + Sync>,
) where
    Req: DeserializeOwned + m5m::LspNoParams + 'static,
    Resp: serde::Serialize + 'static,
{ ::tsox_core::fntrace::enter("register_request_handler"); 
    handlers.insert(method, Arc::new(move |s: &Server, req: &lsproto::RequestMessage| {
        if s.session.is_none() && req.method != METHOD_INITIALIZE {
            return Err(LspError::new(ErrorCode::ServerNotInitialized, "server not initialized"));
        }
        let params = unmarshal_params_or_invalid::<Req>(req)?;
        let resp = fn_(s, &params, req)?;
        if let Some(id) = &req.id {
            let value = serde_json::to_value(&resp).map_err(|e| {
                LspError::new(ErrorCode::InternalError, format!("failed to marshal message: {e}"))
            })?;
            s.send_result(&Some(id.clone()), &value)?;
        }
        Ok(None)
    }));
}

pub fn register_language_service_document_request_handler<Req>(
    handlers: &mut HandlerMap,
    method: lsproto::Method,
    fn_: LanguageServiceDocumentHandlerFn,
) { ::tsox_core::fntrace::enter("register_language_service_document_request_handler"); 
    let method_for_closure = method.clone();
    handlers.insert(method, Arc::new(move |s: &Server, req: &lsproto::RequestMessage| {
        let params = m5m::unmarshal_params::<serde_json::Value>(req.params.as_ref())
            .map_err(|e| LspError::new(ErrorCode::InvalidParams, e))?;
        let uri = lsproto::DocumentUri(text_document_uri(&params));
        let ls = s
            .session
            .as_ref()
            .unwrap()
            .get_language_service(&uri)
            .ok_or_else(|| LspError::new(ErrorCode::InternalError, "no language service"))?;
        let resp = fn_(s, &ls, &params)?;
        s.session.as_ref().unwrap().enqueue_publish_global_diagnostics();
        s.send_result(&req.id, &resp)?;
        Ok(None)
    }));
}

pub fn register_language_service_with_auto_imports_request_handler(
    handlers: &mut HandlerMap,
    method: lsproto::Method,
    fn_: LanguageServiceAutoImportsHandlerFn,
) { ::tsox_core::fntrace::enter("register_language_service_with_auto_imports_request_handler"); 
    let method_for_closure = method.clone();
    handlers.insert(method, Arc::new(move |s: &Server, req: &lsproto::RequestMessage| {
        let method = &method_for_closure;
        let params = m5m::unmarshal_params::<serde_json::Value>(req.params.as_ref())
            .map_err(|e| LspError::new(ErrorCode::InvalidParams, e))?;
        let uri = lsproto::DocumentUri(text_document_uri(&params));
        let session = s.session.as_ref().unwrap().clone();
        session
            .with_language_service_and_snapshot(
                &uri,
                &mut |language_service: &Arc<LanguageService>,
                      snapshot: &Arc<crate::project::snapshot::Snapshot>|
                 -> Result<Option<Box<dyn FnOnce() -> Result<(), String>>>, String> {
                let mut language_service = language_service.clone();
                let mut ls_err = fn_(s, &language_service, &params);
                if ls_err.as_ref().is_err_and(|e| e.is_code(ErrorCode::NeedsAutoImports)) {
                    language_service = session.get_language_service_with_auto_imports(snapshot, &uri)?;
                    ls_err = fn_(s, &language_service, &params);
                    if ls_err.as_ref().is_err_and(|e| e.is_code(ErrorCode::NeedsAutoImports)) {
                        panic!("{} returned ErrNeedsAutoImports even after enabling auto imports", method);
                    }
                }
                let resp = ls_err.map_err(|e| e.message)?;
                s.send_result(&req.id, &resp).map_err(|e| e.message)?;
                Ok(None)
            })
            .map_err(|e| LspError::new(ErrorCode::InternalError, e))?;
        Ok(None)
    }));
}

pub fn register_multi_project_reference_request_handler(
    handlers: &mut HandlerMap,
    method: lsproto::Method,
    fn_: MultiProjectReferenceHandlerFn,
) { ::tsox_core::fntrace::enter("register_multi_project_reference_request_handler"); 
    handlers.insert(method, Arc::new(move |s: &Server, req: &lsproto::RequestMessage| {
        let params = m5m::unmarshal_params::<serde_json::Value>(req.params.as_ref())
            .map_err(|e| LspError::new(ErrorCode::InvalidParams, e))?;
        let uri = lsproto::DocumentUri(text_document_position_uri(&params));
        let (default_ls, orchestrator) = s.get_language_service_and_cross_project_orchestrator(&uri, req)?;
        let orchestrator = orchestrator.expect("orchestrator must exist");
        let resp = fn_(&default_ls, &params, &orchestrator)?;
        s.send_result(&req.id, &resp)?;
        Ok(None)
    }));
}

fn text_document_uri(params: &serde_json::Value) -> String { ::tsox_core::fntrace::enter("text_document_uri"); 
    params["textDocument"]["uri"].as_str().unwrap_or_default().to_string()
}

fn text_document_position_uri(params: &serde_json::Value) -> String { ::tsox_core::fntrace::enter("text_document_position_uri"); 
    text_document_uri(params)
}

pub type HandlerMap = HashMap<lsproto::Method, HandlerFn>;
pub type HandlerFn = Arc<dyn Fn(&Server, &lsproto::RequestMessage) -> HandlerResult + Send + Sync>;
pub type HandlerResult = LspResult<Option<Box<dyn FnOnce() -> LspResult<()> + Send>>>;
pub type NotificationHandlerFn = Arc<dyn Fn(&Server, &lsproto::RequestMessage) -> LspResult<()> + Send + Sync>;
pub type RequestHandlerFn = Arc<dyn Fn(&Server, &lsproto::RequestMessage) -> LspResult<()> + Send + Sync>;
pub type LanguageServiceDocumentHandlerFn =
    Arc<dyn Fn(&Server, &LanguageService, &serde_json::Value) -> LspResult<serde_json::Value> + Send + Sync>;
pub type LanguageServiceAutoImportsHandlerFn =
    Arc<dyn Fn(&Server, &LanguageService, &serde_json::Value) -> LspResult<serde_json::Value> + Send + Sync>;
pub type MultiProjectReferenceHandlerFn =
    Arc<dyn Fn(&LanguageService, &serde_json::Value, &CrossProjectOrchestrator) -> LspResult<serde_json::Value> + Send + Sync>;

pub fn handlers() -> &'static HandlerMap { ::tsox_core::fntrace::enter("handlers"); 
    static HANDLERS: std::sync::OnceLock<HandlerMap> = std::sync::OnceLock::new();
    HANDLERS.get_or_init(|| {
        let mut map = HandlerMap::new();
        register_all_handlers(&mut map);
        map
    })
}

pub fn register_all_handlers(handlers: &mut HandlerMap) { ::tsox_core::fntrace::enter("register_all_handlers"); 
    // m5n_3 的 handle_* 入参与返回类型目前是 m5n_3.rs 私有 mod lsproto 的本地类型，
    // 本文件无法命名这些类型完成类型化注册；待 m5n_3 公开参数类型（或上移到 crate::lsp::lsproto）
    // 后，用本文件提供的 register_* 系列helper 逐条按 Go server.go 的 handlers 表接线。
    // 见 progress_notes_r58F.md 交接。
}

pub fn is_user_facing_request_failed_error(err: &LspError) -> bool { ::tsox_core::fntrace::enter("is_user_facing_request_failed_error"); 
    err.message.contains("userFacingRequestFailed")
}

fn now_millis() -> i64 { ::tsox_core::fntrace::enter("now_millis"); 
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

use std::sync::mpsc;

impl Server {
    pub fn send_result(&self, id: &Option<JsonrpcId>, result: &serde_json::Value) -> LspResult<()> { ::tsox_core::fntrace::enter("send_result"); 
        self.send_response(&lsproto::ResponseMessage {
            jsonrpc: Default::default(),
            id: id.clone(),
            result: Some(result.clone()),
            error: None,
        })
    }

    pub fn send_error(&self, id: Option<JsonrpcId>, err: &LspError) -> LspResult<()> { ::tsox_core::fntrace::enter("send_error"); 
        if id.is_none() && !err.is_code(ErrorCode::InvalidRequest) {
            self.logger.error(&format!("error handling notification: {}", err.message));
            return Ok(());
        }
        let code = err.code;
        self.send_response(&lsproto::ResponseMessage {
            jsonrpc: Default::default(),
            id,
            result: None,
            error: Some(ResponseError {
                code: code as i32,
                message: err.message.clone(),
                data: None,
            }),
        })
    }

    pub fn send_response(&self, resp: &lsproto::ResponseMessage) -> LspResult<()> { ::tsox_core::fntrace::enter("send_response"); 
        self.send(&lsproto::Message {
            kind: MessageKind::Response,
            msg: lsproto::MessageData::Response(resp.clone()),
        })
    }

    pub fn send(&self, msg: &lsproto::Message) -> LspResult<()> { ::tsox_core::fntrace::enter("send"); 
        self.outgoing_queue
            .put(msg.clone())
            .map_err(|_| LspError::new(ErrorCode::InternalError, "outgoing queue closed"))
    }
}

pub trait RequestInfoLike {
    fn new_request_message(&self, id: Option<JsonrpcId>, params: serde_json::Value) -> lsproto::RequestMessage;
}

impl RequestInfoLike for &lsproto::RequestInfo {
    fn new_request_message(&self, id: Option<JsonrpcId>, params: serde_json::Value) -> lsproto::RequestMessage { ::tsox_core::fntrace::enter("new_request_message"); 
        lsproto::RequestInfo::new_request_message(self, id, params)
    }
}

pub trait NotificationInfoLike {
    fn new_notification_message(&self, params: serde_json::Value) -> lsproto::RequestMessage;
}

impl NotificationInfoLike for &lsproto::NotificationInfo {
    fn new_notification_message(&self, params: serde_json::Value) -> lsproto::RequestMessage { ::tsox_core::fntrace::enter("new_notification_message"); 
        lsproto::NotificationInfo::new_notification_message(self, params)
    }
}

pub fn send_client_request<R: serde::Serialize>(
    s: &Server,
    info: impl RequestInfoLike,
    params: R,
) -> LspResult<serde_json::Value> { ::tsox_core::fntrace::enter("send_client_request"); 
    let id = JsonrpcId::Str(format!("ts{}", s.client_seq.fetch_add(1, Ordering::SeqCst) + 1));
    let params_value = serde_json::to_value(&params)
        .map_err(|e| LspError::new(ErrorCode::InternalError, format!("failed to marshal message: {e}")))?;
    let req = info.new_request_message(Some(id.clone()), params_value);
    let (tx, rx) = mpsc::channel::<lsproto::ResponseMessage>();
    s.pending_server_requests.lock().unwrap().insert(id.clone(), tx);
    let result = (|| -> LspResult<serde_json::Value> {
        s.send(&req.message())?;
        let resp = rx
            .recv()
            .map_err(|_| LspError::new(ErrorCode::InternalError, "response channel closed"))?;
        if let Some(err) = &resp.error {
            return Err(LspError::new(
                ErrorCode::RequestFailed,
                format!("request failed: {}", err.message),
            ));
        }
        Ok(resp.result.clone().unwrap_or(serde_json::Value::Null))
    })();
    s.pending_server_requests.lock().unwrap().remove(&id);
    result
}

pub fn send_client_request_fire_and_forget<R: serde::Serialize>(
    s: &Server,
    info: impl RequestInfoLike,
    params: R,
) -> LspResult<()> { ::tsox_core::fntrace::enter("send_client_request_fire_and_forget"); 
    let id = JsonrpcId::Str(format!("ts{}", s.client_seq.fetch_add(1, Ordering::SeqCst) + 1));
    let params_value = serde_json::to_value(&params)
        .map_err(|e| LspError::new(ErrorCode::InternalError, format!("failed to marshal message: {e}")))?;
    let req = info.new_request_message(Some(id), params_value);
    s.send(&req.message())
}

pub fn send_notification<P: serde::Serialize>(
    s: &Server,
    info: impl NotificationInfoLike,
    params: P,
) -> LspResult<()> { ::tsox_core::fntrace::enter("send_notification"); 
    let params_value = serde_json::to_value(&params)
        .map_err(|e| LspError::new(ErrorCode::InternalError, format!("failed to marshal message: {e}")))?;
    s.send(&info.new_notification_message(params_value).message())
}
