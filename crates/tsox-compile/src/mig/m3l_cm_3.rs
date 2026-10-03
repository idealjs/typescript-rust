#![allow(invalid_reference_casting)]
use super::m3l_cm_2::*;
use super::m3l_cm::{
    DiagnosticDirectiveError, DiagnosticDirectiveErrorKind, DiagnosticDirectivePolicy,
    InitializeError, InitializeErrorKind, InvalidVirtualExtensionError, JsonValue, Mapper,
    MapperTimings, OperationTiming, ProjectError, ProjectErrorKind, Timings, TransformError,
    TransformErrorKind, compiler_options_to_json, is_supported_virtual_extension,
    new_transform_error,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::locale::Locale;
use tsox_core::tspath::Path;
pub use tsox_frontend::ast::mig::m3b_2::{MappedDiagnosticDirective, MappedDiagnosticDirectivePolicy};

pub type DialFunc = Box<
    dyn Fn(&Mapper, &Locale) -> Result<(Box<dyn IpcConn>, Option<Box<dyn Closer>>, PositionEncoding, String), Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

pub const ALL_SUPPORTED_EXTENSIONS_WITH_JSON_FLATTENED: &[&str] = &[
    tsox_core::tspath::EXTENSION_TS,
    tsox_core::tspath::EXTENSION_TSX,
    tsox_core::tspath::EXTENSION_DTS,
    tsox_core::tspath::EXTENSION_JS,
    tsox_core::tspath::EXTENSION_JSX,
    tsox_core::tspath::EXTENSION_CTS,
    tsox_core::tspath::EXTENSION_DCTS,
    tsox_core::tspath::EXTENSION_CJS,
    tsox_core::tspath::EXTENSION_MTS,
    tsox_core::tspath::EXTENSION_DMTS,
    tsox_core::tspath::EXTENSION_MJS,
    tsox_core::tspath::EXTENSION_JSON,
];

pub trait Closer: Send + Sync {
    fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub trait IpcConn: Send + Sync {
    fn call(&self, method: &str, params: &JsonValue) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>>;
    fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn duplicate_handle(&self) -> Box<dyn IpcConn>;
}

pub fn marshal_params<T: serde::Serialize>(
    params: &T,
) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("marshal_params"); 
    Ok(serde_json::to_value(params)?)
}

pub struct HostImpl {
    pub cancel: AtomicU64,
    pub dial: DialFunc,
    pub timing: Arc<TimingCollector>,
    pub lifecycle_mu: RwLock<Locale>,
    pub mu: Mutex<HostState>,
}

#[derive(Default)]
pub struct HostState {
    pub conns: Option<HashMap<String, Arc<MapperConnEntry>>>,
    pub projects: Option<HashMap<String, Arc<ProjectEntry>>>,
    pub project_leases: Option<HashMap<String, Arc<ProjectLease>>>,
    pub next_project_id: u64,
}

pub struct ProjectEntry {
    pub mapper: Arc<Mapper>,
    pub spec: ProjectSpec,
    pub project_handle: String,
    pub opened: bool,
    pub config_identity: String,
    pub watched_files: Vec<String>,
    pub option_diagnostics: Vec<OptionDiagnostic>,
}

pub struct MapperConnEntry {
    pub conn: Mutex<Option<Box<dyn IpcConn>>>,
    pub closer: Mutex<Option<Arc<dyn Closer>>>,
    pub err: Mutex<Option<Box<dyn std::error::Error + Send + Sync>>>,
    pub position_encoding: Mutex<PositionEncoding>,
    pub diagnostic_source: Mutex<String>,
    pub refs: AtomicU64,
}

impl Default for MapperConnEntry {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        MapperConnEntry {
            conn: Mutex::new(None),
            closer: Mutex::new(None),
            err: Mutex::new(None),
            position_encoding: Mutex::new(String::new()),
            diagnostic_source: Mutex::new(String::new()),
            refs: AtomicU64::new(0),
        }
    }
}

pub fn new_host(spawner: Arc<dyn Spawner>, diagnostic_locale: Locale) -> Arc<HostImpl> { ::tsox_core::fntrace::enter("new_host"); 
    new_host_with_options(spawner, diagnostic_locale, HostOptions::default())
}

pub fn new_host_with_options(
    spawner: Arc<dyn Spawner>,
    diagnostic_locale: Locale,
    options: HostOptions,
) -> Arc<HostImpl> { ::tsox_core::fntrace::enter("new_host_with_options"); 
    let logger = options.logger;
    let timing = Arc::new(TimingCollector::new());
    let timing_for_dial = timing.clone();
    let dial: DialFunc = Box::new(
        move |mapper: &Mapper, diagnostic_locale: &Locale| {
            if mapper.manifest.exec.is_empty() {
                return Err(Box::new(InitializeError {
                    kind: InitializeErrorKind::ProcessStart,
                    command: String::new(),
                    detail: format!(
                        "content mapper {:?} declares no command to run",
                        mapper.definition.package
                    ),
                    ..InitializeError::default()
                })
                    as Box<dyn std::error::Error + Send + Sync>);
            }
            let mapper_timing = timing_for_dial.mapper(&mapper.identity());
            let diagnostic_name = mapper.diagnostic_name();
            let spawn_start = Instant::now();
            let mut stderr: Box<dyn std::io::Write> = Box::new(std::io::sink());
            let mut stderr_log: Option<Arc<StderrLogger>> = None;
            if let Some(logger) = &logger {
                let log = Arc::new(StderrLogger::new(diagnostic_name.clone(), logger.clone()));
                stderr_log = Some(log.clone());
                stderr = Box::new(StderrLineWriter(log));
            }
            let mut rwc = spawner
                .spawn(&mapper.manifest.exec, &mapper.package_directory, stderr.as_mut())
                .map_err(|err| {
                    InitializeError {
                        kind: InitializeErrorKind::ProcessStart,
                        mapper_name: diagnostic_name.clone(),
                        command: mapper.manifest.exec[0].clone(),
                        detail: err.to_string(),
                        ..InitializeError::default()
                    }
                })?;
            mapper_timing.spawn.record(spawn_start);
            if let Some(stderr_log) = stderr_log {
                rwc = Box::new(LoggedProcess {
                    process: rwc,
                    stderr: stderr_log,
                });
            }
            let rwc = Arc::new(CloseOnceReadWriteCloser::new(rwc));
            let mut protocol: Box<dyn IpcProtocol> = new_jsonrpc_protocol(rwc.clone());
            if let Some(logger) = &logger {
                protocol = Box::new(LoggingProtocol {
                    protocol,
                    mapper_name: diagnostic_name.clone(),
                    logger: logger.clone(),
                });
            }
            let conn = new_async_conn_with_protocol(rwc.clone(), protocol, RejectHandler);
            conn.run_detached();
            let initialize_start = mapper_timing.start_request();
            let handshake_result =
                handshake(&conn, diagnostic_locale);
            mapper_timing
                .finish_request(&mapper_timing.initialize, initialize_start);
            match handshake_result {
                Ok((position_encoding, diagnostic_source)) => Ok((
                    Box::new(conn) as Box<dyn IpcConn>,
                    Some(Box::new(ProcessCloser(rwc)) as Box<dyn Closer>),
                    position_encoding,
                    diagnostic_source,
                )),
                Err(err) => {
                    let _ = rwc.close();
                    let exited = rwc.exit_code();
                    if let Some(initialize_error) =
                        err.downcast_ref::<InitializeError>()
                    {
                        let mut initialize_error = initialize_error.clone();
                        initialize_error.mapper_name = diagnostic_name;
                        return Err(Box::new(initialize_error));
                    }
                    if let Some(exit_code) = exited {
                        return Err(Box::new(InitializeError {
                            kind: InitializeErrorKind::ProcessExit,
                            mapper_name: diagnostic_name,
                            exit_code,
                            ..InitializeError::default()
                        }));
                    }
                    Err(Box::new(InitializeError {
                        kind: InitializeErrorKind::Request,
                        mapper_name: diagnostic_name,
                        detail: err.to_string(),
                        ..InitializeError::default()
                    }))
                }
            }
        },
    );
    new_with_dial(diagnostic_locale, timing, dial)
}

pub fn new_with_dial(
    diagnostic_locale: Locale,
    timing: Arc<TimingCollector>,
    dial: DialFunc,
) -> Arc<HostImpl> { ::tsox_core::fntrace::enter("new_with_dial"); 
    Arc::new(HostImpl {
        cancel: AtomicU64::new(0),
        dial,
        timing,
        lifecycle_mu: RwLock::new(diagnostic_locale),
        mu: Mutex::new(HostState {
            conns: Some(HashMap::new()),
            projects: Some(HashMap::new()),
            project_leases: Some(HashMap::new()),
            next_project_id: 0,
        }),
    })
}

impl HostImpl {
    pub fn timings(&self) -> Timings { ::tsox_core::fntrace::enter("timings"); 
        self.timing.snapshot()
    }

    pub fn set_locale(&self, diagnostic_locale: Locale) { ::tsox_core::fntrace::enter("set_locale"); 
        let mut lifecycle = self.lifecycle_mu.write().unwrap();
        if lifecycle.to_string() == diagnostic_locale.to_string() {
            return;
        }
        *lifecycle = diagnostic_locale;
        let mut state = self.mu.lock().unwrap();
        if let Some(conns) = &mut state.conns {
            for entry in conns.values() {
                let closer = entry.closer.lock().unwrap().take();
                if let Some(closer) = closer {
                    let _ = closer.close();
                }
                *entry.conn.lock().unwrap() = None;
                *entry.err.lock().unwrap() = None;
                *entry.position_encoding.lock().unwrap() = String::new();
                *entry.diagnostic_source.lock().unwrap() = String::new();
            }
        }
        if let Some(projects) = &mut state.projects {
            let _ = projects;
        }
        let mut reopen_projects: Vec<Arc<ProjectEntry>> = Vec::new();
        if let Some(projects) = &mut state.projects {
            for project in projects.values_mut() {
                let entry = Arc::clone(project);
                reopen_projects.push(entry);
            }
        }
        for project in reopen_projects {
            let entry = unsafe { &mut *(Arc::as_ptr(&project) as *mut ProjectEntry) };
            entry.opened = false;
        }
    }

    pub fn project(self: &Arc<Self>, spec: &ProjectSpec) -> Option<Arc<ProjectLease>> { ::tsox_core::fntrace::enter("project"); 
        let _lifecycle = self.lifecycle_mu.read().unwrap();
        let key = project_spec_key(spec);
        let mut state = self.mu.lock().unwrap();
        if state.projects.is_none() {
            return None;
        }
        if let Some(lease) = state.project_leases.as_ref().unwrap().get(&key) {
            return Some(retain_locked(lease));
        }
        let lease = Arc::new(ProjectLease {
            host: self.clone(),
            key: key.clone(),
            mappers: spec.mappers.clone(),
            entries: {
                let mut entries = HashMap::with_capacity(spec.mappers.len());
                for mapper in &spec.mappers {
                    let entry_key = format!("{}:{}", mapper.identity(), state.next_project_id);
                    state.next_project_id += 1;
                    entries.insert(Arc::as_ptr(mapper) as usize, entry_key.clone());
                    let entry = Arc::new(ProjectEntry {
                        mapper: mapper.clone(),
                        project_handle: entry_key.clone(),
                        spec: spec.clone(),
                        opened: false,
                        config_identity: String::new(),
                        watched_files: Vec::new(),
                        option_diagnostics: Vec::new(),
                    });
                    state.projects.as_mut().unwrap().insert(entry_key, entry);
                    let conns = state.conns.get_or_insert_with(HashMap::new);
                    let conn_entry = conns
                        .entry(mapper.identity())
                        .or_insert_with(|| Arc::new(MapperConnEntry::default()));
                    conn_entry.refs.fetch_add(1, Ordering::SeqCst);
                }
                entries
            },
            refs: AtomicU64::new(1),
        });
        state
            .project_leases
            .as_mut()
            .unwrap()
            .insert(key, lease.clone());
        Some(lease)
    }

    pub fn open_project_locked(
        &self,
        state: &mut HostState,
        entry: &Arc<ProjectEntry>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("open_project_locked"); 
        if entry.opened {
            return Ok(());
        }
        let (conn, _position_encoding, diagnostic_source) = self.conn_for_locked(state, &entry.mapper)?;
        let compiler_options_json = compiler_options_to_json(
            entry
                .spec
                .compiler_options
                .as_deref()
                .unwrap_or(&CompilerOptions::default()),
        );
        let mapper_timing = self.timing.mapper(&entry.mapper.identity());
        let start = mapper_timing.start_request();
        let raw = conn.call(
            METHOD_OPEN_PROJECT,
            &marshal_params(&OpenProjectParams {
                config_file_name: entry.spec.config_file_name.clone(),
                project_handle: entry.project_handle.clone(),
                options: entry.mapper.definition.options.clone(),
                compiler_options: compiler_options_json,
            })?,
        );
        mapper_timing.finish_request(&mapper_timing.open_project, start);
        let raw = raw?;
        let result: OpenProjectResult =
            serde_json::from_value::<OpenProjectResult>(raw.clone()).map_err(|_| ProjectError {
                kind: ProjectErrorKind::MalformedResponse,
            })?;
        if entry.mapper.manifest.dynamic_config && result.config_identity.is_empty() {
            return Err(Box::new(ProjectError {
                kind: ProjectErrorKind::MissingConfigIdentity,
            }));
        }
        if !entry.mapper.manifest.dynamic_config && !result.config_identity.is_empty() {
            return Err(Box::new(ProjectError {
                kind: ProjectErrorKind::UnexpectedConfigIdentity,
            }));
        }
        if !entry.mapper.manifest.dynamic_config && !result.watched_files.is_empty() {
            return Err(Box::new(ProjectError {
                kind: ProjectErrorKind::UnexpectedWatchedFiles,
            }));
        }
        let entry = Arc::clone(entry);
        {
            let entry_mut = unsafe { &mut *(Arc::as_ptr(&entry) as *mut ProjectEntry) };
            entry_mut.config_identity = result.config_identity.clone();
        }
        for file_name in &result.watched_files {
            if !tsox_core::tspath::path_is_absolute(file_name) {
                return Err(Box::new(ProjectError {
                    kind: ProjectErrorKind::NonAbsoluteWatchedFile,
                }));
            }
        }
        let mut option_diagnostics = Vec::with_capacity(result.option_diagnostics.len());
        for diagnostic in &result.option_diagnostics {
            let mut path = Vec::with_capacity(diagnostic.path.len());
            for raw_segment in &diagnostic.path {
                match raw_segment {
                    serde_json::Value::String(property) => path.push(OptionPathSegment {
                        property: property.clone(),
                        index: 0,
                        is_index: false,
                    }),
                    serde_json::Value::Number(index) => {
                        let index = index
                            .as_i64()
                            .ok_or_else(|| ProjectError {
                                kind: ProjectErrorKind::MalformedResponse,
                            })?;
                        if index < 0 {
                            return Err(Box::new(ProjectError {
                                kind: ProjectErrorKind::MalformedResponse,
                            }));
                        }
                        path.push(OptionPathSegment {
                            property: String::new(),
                            index: index as i32,
                            is_index: true,
                        });
                    }
                    _ => {
                        return Err(Box::new(ProjectError {
                            kind: ProjectErrorKind::MalformedResponse,
                        }))
                    }
                }
            }
            option_diagnostics.push(OptionDiagnostic {
                mapper: entry.mapper.clone(),
                path,
                source: diagnostic_source.clone(),
                code: diagnostic.code,
                message_text: diagnostic.message_text.clone(),
            });
        }
        {
            let entry_mut = unsafe { &mut *(Arc::as_ptr(&entry) as *mut ProjectEntry) };
            entry_mut.watched_files = result.watched_files.clone();
            entry_mut.option_diagnostics = option_diagnostics;
            entry_mut.opened = true;
        }
        Ok(())
    }

    pub fn close_project(
        &self,
        mapper: &Mapper,
        conn: &dyn IpcConn,
        project_handle: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("close_project"); 
        let mapper_timing = self.timing.mapper(&mapper.identity());
        let start = mapper_timing.start_request();
        let result = conn.call(
            METHOD_CLOSE_PROJECT,
            &marshal_params(&CloseProjectParams {
                project_handle: project_handle.to_string(),
            })?,
        );
        mapper_timing
            .finish_request(&mapper_timing.close_project, start);
        result.map(|_| ())
    }

    pub fn acquire(self: &Arc<Self>, mappers: &[Arc<Mapper>]) -> Box<dyn FnOnce()> { ::tsox_core::fntrace::enter("acquire"); 
        let mut seen = std::collections::HashSet::with_capacity(mappers.len());
        let mut identities = Vec::with_capacity(mappers.len());
        {
            let mut state = self.mu.lock().unwrap();
            if let Some(conns) = &mut state.conns {
                for mapper in mappers {
                    let identity = mapper.identity();
                    if !seen.insert(identity.clone()) {
                        continue;
                    }
                    identities.push(identity.clone());
                    let entry = conns
                        .entry(identity)
                        .or_insert_with(|| Arc::new(MapperConnEntry::default()));
                    entry.refs.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
        let host = self.clone();
        Box::new(move || host.release(&identities))
    }

    pub fn transform(
        self: &Arc<Self>,
        mapper: &Mapper,
        request: &Request,
    ) -> Result<TransformOutcome, TransformError> { ::tsox_core::fntrace::enter("transform"); 
        let project = self
            .project(&ProjectSpec {
                mappers: vec![Arc::new(mapper.clone())],
                compiler_options: Some(Arc::new(CompilerOptions::default())),
                ..ProjectSpec::default()
            })
            .ok_or_else(|| new_transform_error(TransformErrorKind::Project, Box::new(ProjectError::default())))?;
        let result = project_transform(&project, mapper, request);
        let _ = project_release(&project);
        result
    }

    pub fn transform_locked(
        &self,
        state: &HostState,
        mapper: &Mapper,
        request: &Request,
        project_handle: &str,
    ) -> Result<TransformOutcome, TransformError> { ::tsox_core::fntrace::enter("transform_locked"); 
        if project_handle.is_empty() {
            return Err(new_transform_error(
                TransformErrorKind::Request,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "content mapper project handle is required",
                )),
            ));
        }
        let (conn, position_encoding, diagnostic_source) = self
            .conn_for(state, mapper)
            .map_err(|err| new_transform_error(TransformErrorKind::Initialize, err))?;
        let conn = conn.ok_or_else(|| {
            new_transform_error(
                TransformErrorKind::Initialize,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "content mapper host is closed",
                )),
            )
        })?;
        let mapper_timing = self.timing.mapper(&mapper.identity());
        let start = mapper_timing.start_request();
        let raw = conn.call(
            METHOD_TRANSFORM,
            &marshal_params(&TransformParams {
                file_name: request.file_name.clone(),
                content: request.content.clone(),
                project_handle: project_handle.to_string(),
            })
            .map_err(|err| new_transform_error(TransformErrorKind::Request, err))?,
        );
        mapper_timing.finish_request(&mapper_timing.transform, start);
        let raw = raw.map_err(|err| new_transform_error(TransformErrorKind::Request, err))?;
        decode_transform_result(&raw, &request.content, &position_encoding, &diagnostic_source)
            .map_err(|err| new_transform_error(TransformErrorKind::Response, err))
    }

    pub fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("close"); 
        let mut _lifecycle = self.lifecycle_mu.write().unwrap();
        self.cancel.fetch_add(1, Ordering::SeqCst);
        let mut closers: Vec<Arc<dyn Closer>> = Vec::new();
        {
            let mut state = self.mu.lock().unwrap();
            if let Some(conns) = &mut state.conns {
                for mc in conns.values() {
                    if let Some(closer) = mc.closer.lock().unwrap().take() {
                        closers.push(closer);
                    }
                }
            }
            state.conns = None;
            state.projects = None;
            state.project_leases = None;
        }
        let mut errs: Vec<Box<dyn std::error::Error + Send + Sync>> = Vec::new();
        for closer in closers {
            if let Err(err) = closer.close() {
                errs.push(err);
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(Box::new(std::io::Error::other(format!(
                "{}",
                errs.iter()
                    .map(|err| err.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ))))
        }
    }

    pub fn conn_for(
        &self,
        state: &HostState,
        mapper: &Mapper,
    ) -> Result<(Option<Box<dyn IpcConn>>, PositionEncoding, String), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("conn_for"); 
        self.conn_for_locked(state, mapper).map(|(conn, encoding, source)| {
            (Some(conn), encoding, source)
        })
    }

    pub fn conn_for_locked(
        &self,
        state: &HostState,
        mapper: &Mapper,
    ) -> Result<(Box<dyn IpcConn>, PositionEncoding, String), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("conn_for_locked"); 
        let conns = match &state.conns {
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "content mapper host is closed",
                )))
            }
            Some(conns) => conns,
        };
        let identity = mapper.identity();
        let entry = conns
            .get(&identity)
            .cloned()
            .unwrap_or_else(|| Arc::new(MapperConnEntry::default()));
        let has_conn = entry.conn.lock().unwrap().is_some();
        let has_err = entry.err.lock().unwrap().is_some();
        if has_conn || has_err {
            let conn = entry
                .conn
                .lock()
                .unwrap()
                .as_ref()
                .map(|conn| conn_duplicate_handle(&**conn));
            let position_encoding = entry.position_encoding.lock().unwrap().clone();
            let diagnostic_source = entry.diagnostic_source.lock().unwrap().clone();
            let err = entry.err.lock().unwrap().take();
            return match (conn, err) {
                (Some(conn), None) => Ok((conn, position_encoding, diagnostic_source)),
                (conn, Some(err)) => Err(err),
                (None, None) => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "content mapper connection unavailable",
                ))),
            };
        }
        let locale = self.lifecycle_mu.read().unwrap().clone();
        let dial_result = (self.dial)(mapper, &locale);
        match dial_result {
            Ok((conn, closer, position_encoding, diagnostic_source)) => {
                let mut_state = unsafe {
                    &mut *(state as *const HostState as *mut HostState)
                };
                let conns = mut_state.conns.get_or_insert_with(HashMap::new);
                let entry = conns
                    .entry(identity)
                    .or_insert_with(|| Arc::new(MapperConnEntry::default()));
                *entry.conn.lock().unwrap() = Some(conn_duplicate_handle(&*conn));
                *entry.closer.lock().unwrap() = closer.map(|closer| closer_into_arc(closer));
                *entry.position_encoding.lock().unwrap() = position_encoding.clone();
                *entry.diagnostic_source.lock().unwrap() = diagnostic_source.clone();
                Ok((conn, position_encoding, diagnostic_source))
            }
            Err(err) => {
                let mut_state =
                    unsafe { &mut *(state as *const HostState as *mut HostState) };
                let conns = mut_state.conns.get_or_insert_with(HashMap::new);
                let entry = conns
                    .entry(identity)
                    .or_insert_with(|| Arc::new(MapperConnEntry::default()));
                *entry.err.lock().unwrap() = Some(err);
                Err(entry.err.lock().unwrap().take().unwrap())
            }
        }
    }

    pub fn release(&self, identities: &[String]) { ::tsox_core::fntrace::enter("release"); 
        let mut closers: Vec<Arc<dyn Closer>> = Vec::new();
        {
            let mut state = self.mu.lock().unwrap();
            if let Some(conns) = &mut state.conns {
                for identity in identities {
                    let entry = match conns.get(identity) {
                        None => continue,
                        Some(entry) => entry.clone(),
                    };
                    if entry.refs.fetch_sub(1, Ordering::SeqCst) == 1 {
                        conns.remove(identity);
                        if let Some(closer) = entry.closer.lock().unwrap().take() {
                            closers.push(closer);
                        }
                    }
                }
            }
        }
        for closer in closers {
            let _ = closer.close();
        }
    }
}

pub struct ProjectLease {
    pub host: Arc<HostImpl>,
    pub key: String,
    pub mappers: Vec<Arc<Mapper>>,
    pub entries: HashMap<usize, String>,
    pub refs: AtomicU64,
}

pub fn retain_locked(lease: &Arc<ProjectLease>) -> Arc<ProjectLease> { ::tsox_core::fntrace::enter("retain_locked"); 
    lease.refs.fetch_add(1, Ordering::SeqCst);
    lease.clone()
}

pub fn project_refresh(lease: &Arc<ProjectLease>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("project_refresh"); 
    let _lifecycle = lease.host.lifecycle_mu.read().unwrap();
    let mut state = lease.host.mu.lock().unwrap();
    if state.projects.is_none() {
        return Ok(());
    }
    let mut result: Result<(), Box<dyn std::error::Error + Send + Sync>> = Ok(());
    for key in lease.entries.values() {
        let entry = match state.projects.as_ref().unwrap().get(key) {
            None => continue,
            Some(entry) => entry.clone(),
        };
        if !entry.opened {
            continue;
        }
        if let Some(conn_entry) = state.conns.as_ref().unwrap().get(&entry.mapper.identity()) {
            if let Some(conn) = conn_entry.conn.lock().unwrap().as_ref() {
                if let Err(err) = lease.host.close_project(&entry.mapper, conn.as_ref(), &entry.project_handle) {
                    result = Err(err);
                }
            }
        }
        let entry_mut = unsafe { &mut *(Arc::as_ptr(&entry) as *mut ProjectEntry) };
        entry_mut.opened = false;
    }
    result
}

pub fn project_identities(
    lease: &Arc<ProjectLease>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("project_identities"); 
    let _lifecycle = lease.host.lifecycle_mu.read().unwrap();
    let mut state = lease.host.mu.lock().unwrap();
    if state.projects.is_none() {
        return Ok(Vec::new());
    }
    let mut identities = Vec::with_capacity(lease.mappers.len());
    for mapper in &lease.mappers {
        let key = lease
            .entries
            .get(&(Arc::as_ptr(mapper) as usize))
            .cloned();
        let key = match key {
            None => continue,
            Some(key) => key,
        };
        let entry = match state.projects.as_ref().unwrap().get(&key) {
            None => continue,
            Some(entry) => entry.clone(),
        };
        if mapper.manifest.dynamic_config {
            lease.host.open_project_locked(&mut state, &entry)?;
            let identity = entry.config_identity.clone();
            identities.push(combined_identity(
                mapper,
                &identity,
                entry.spec.compiler_options.as_deref(),
            ));
        } else {
            let hash = mapper.transform_identity(
                entry.spec.compiler_options.as_deref().unwrap_or(&CompilerOptions::default()),
            );
            identities.push(format!(
                "{}:{}",
                mapper.identity(),
                hex_encode(&hash.to_le_bytes())
            ));
        }
    }
    Ok(identities)
}

pub fn project_identity(
    lease: &Arc<ProjectLease>,
    mapper: &Mapper,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("project_identity"); 
    let _lifecycle = lease.host.lifecycle_mu.read().unwrap();
    let mut state = lease.host.mu.lock().unwrap();
    if state.projects.is_none() {
        return Ok(String::new());
    }
    let key = match lease.entries.get(&(mapper as *const Mapper as usize)) {
        None => return Ok(String::new()),
        Some(key) => key.clone(),
    };
    let entry = match state.projects.as_ref().unwrap().get(&key) {
        None => return Ok(String::new()),
        Some(entry) => entry.clone(),
    };
    if mapper.manifest.dynamic_config {
        lease.host.open_project_locked(&mut state, &entry)?;
        let identity = entry.config_identity.clone();
        return Ok(combined_identity(
            mapper,
            &identity,
            entry.spec.compiler_options.as_deref(),
        ));
    }
    let hash = mapper.transform_identity(
        entry
            .spec
            .compiler_options
            .as_deref()
            .unwrap_or(&CompilerOptions::default()),
    );
    Ok(format!("{}:{}", mapper.identity(), hex_encode(&hash.to_le_bytes())))
}

pub fn project_watched_files(
    lease: &Arc<ProjectLease>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("project_watched_files"); 
    let _lifecycle = lease.host.lifecycle_mu.read().unwrap();
    let mut state = lease.host.mu.lock().unwrap();
    if state.projects.is_none() {
        return Ok(Vec::new());
    }
    let mut files: Vec<String> = Vec::new();
    for key in lease.entries.values() {
        let entry = match state.projects.as_ref().unwrap().get(key) {
            None => continue,
            Some(entry) => entry.clone(),
        };
        if entry.mapper.manifest.dynamic_config {
            lease.host.open_project_locked(&mut state, &entry)?;
        }
        files.extend(entry.watched_files.iter().cloned());
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn project_diagnostics(lease: &Arc<ProjectLease>) -> Vec<OptionDiagnostic> { ::tsox_core::fntrace::enter("project_diagnostics"); 
    let _lifecycle = lease.host.lifecycle_mu.read().unwrap();
    let state = lease.host.mu.lock().unwrap();
    if state.projects.is_none() {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for mapper in &lease.mappers {
        let key = match lease.entries.get(&(Arc::as_ptr(mapper) as usize)) {
            None => continue,
            Some(key) => key,
        };
        let entry = match state.projects.as_ref().unwrap().get(key) {
            None => continue,
            Some(entry) => entry,
        };
        if !entry.opened {
            continue;
        }
        diagnostics.extend(entry.option_diagnostics.iter().cloned());
    }
    diagnostics
}

pub fn project_transform(
    lease: &Arc<ProjectLease>,
    mapper: &Mapper,
    request: &Request,
) -> Result<TransformOutcome, TransformError> { ::tsox_core::fntrace::enter("project_transform"); 
    let _lifecycle = lease.host.lifecycle_mu.read().unwrap();
    let mut state = lease.host.mu.lock().unwrap();
    let key = lease
        .entries
        .get(&(mapper as *const Mapper as usize))
        .cloned();
    let entry = match key.and_then(|key| state.projects.as_ref().unwrap().get(&key).cloned()) {
        None => {
            return Err(new_transform_error(
                TransformErrorKind::Project,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "content mapper project is closed",
                )),
            ))
        }
        Some(entry) => entry,
    };
    if let Err(err) = lease.host.open_project_locked(&mut state, &entry) {
        if err.downcast_ref::<InitializeError>().is_some() {
            return Err(new_transform_error(TransformErrorKind::Initialize, err));
        }
        return Err(new_transform_error(TransformErrorKind::Project, err));
    }
    let handle = entry.project_handle.clone();
    let state_snapshot: &HostState = &state;
    lease
        .host
        .transform_locked(state_snapshot, mapper, request, &handle)
}

pub fn project_release(lease: &Arc<ProjectLease>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("project_release"); 
    let _lifecycle = lease.host.lifecycle_mu.read().unwrap();
    let mut released_identities: Vec<String> = Vec::new();
    let mut result: Result<(), Box<dyn std::error::Error + Send + Sync>> = Ok(());
    {
        let mut state = lease.host.mu.lock().unwrap();
        if lease.refs.fetch_sub(1, Ordering::SeqCst) == 0 {
            panic!("content mapper project reference count below zero");
        }
        if lease.refs.load(Ordering::SeqCst) != 0 {
            return Ok(());
        }
        state
            .project_leases
            .as_mut()
            .unwrap()
            .retain(|_, other| !Arc::ptr_eq(other, lease));
        for key in lease.entries.values() {
            let entry = match state.projects.as_ref().unwrap().get(key) {
                None => continue,
                Some(entry) => entry.clone(),
            };
            if entry.opened {
                if let Some(conn_entry) = state.conns.as_ref().unwrap().get(&entry.mapper.identity()) {
                    if let Some(conn) = conn_entry.conn.lock().unwrap().as_ref() {
                        if let Err(err) =
                            lease.host.close_project(&entry.mapper, conn.as_ref(), &entry.project_handle)
                        {
                            result = Err(err);
                        }
                    }
                }
            }
            state.projects.as_mut().unwrap().remove(key);
            released_identities.push(entry.mapper.identity());
        }
    }
    lease.host.release(&released_identities);
    result
}

pub fn project_spec_key(spec: &ProjectSpec) -> String { ::tsox_core::fntrace::enter("project_spec_key"); 
    let mut key = String::new();
    key.push_str(&spec.config_file_name);
    key.push('\0');
    key.push_str(&format!(
        "{:p}",
        spec.compiler_options.as_deref().unwrap_or(&CompilerOptions::default())
    ));
    for mapper in &spec.mappers {
        key.push('\0');
        key.push_str(&format!("{:p}", Arc::as_ptr(mapper)));
    }
    key
}

pub fn combined_identity(
    mapper: &Mapper,
    config_identity: &str,
    compiler_options: Option<&CompilerOptions>,
) -> String { ::tsox_core::fntrace::enter("combined_identity"); 
    let transform_identity = mapper
        .transform_identity(compiler_options.unwrap_or(&CompilerOptions::default()))
        .to_le_bytes();
    let identity = mapper.identity();
    let mut buf = Vec::with_capacity(
        identity.len() + mapper.definition.options.to_string().len()
            + config_identity.len()
            + transform_identity.len()
            + 3,
    );
    buf.extend_from_slice(identity.as_bytes());
    buf.push(0);
    buf.extend_from_slice(mapper.definition.options.to_string().as_bytes());
    buf.push(0);
    buf.extend_from_slice(config_identity.as_bytes());
    buf.push(0);
    buf.extend_from_slice(&transform_identity);
    let hash = xxhash_rust::xxh3::xxh3_128(&buf).to_le_bytes();
    format!("{}:{}", identity, hex_encode(&hash))
}

pub fn hex_encode(bytes: &[u8]) -> String { ::tsox_core::fntrace::enter("hex_encode"); 
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn handshake(
    conn: &dyn IpcConn,
    diagnostic_locale: &Locale,
) -> Result<(PositionEncoding, String), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("handshake"); 
    let raw = conn.call(
        METHOD_INITIALIZE,
        &marshal_params(&InitializeParams {
            locale: diagnostic_locale.to_string(),
            position_encodings: vec![
                POSITION_ENCODING_UTF8.to_string(),
                POSITION_ENCODING_UTF16.to_string(),
            ],
        })?,
    )?;
    let res: InitializeResult = serde_json::from_value(raw).map_err(|err| {
        Box::new(InitializeError {
            kind: InitializeErrorKind::InvalidResponse,
            detail: err.to_string(),
            ..InitializeError::default()
        })
    })?;
    if res.position_encoding != POSITION_ENCODING_UTF8
        && res.position_encoding != POSITION_ENCODING_UTF16
    {
        return Err(Box::new(InitializeError {
            kind: InitializeErrorKind::PositionEncoding,
            position_encoding: res.position_encoding.clone(),
            ..InitializeError::default()
        }));
    }
    if res.diagnostic_source.trim().is_empty() {
        return Err(Box::new(InitializeError {
            kind: InitializeErrorKind::EmptyDiagnosticSource,
            ..InitializeError::default()
        }));
    }
    if res.diagnostic_source.eq_ignore_ascii_case("typescript")
        || res.diagnostic_source.eq_ignore_ascii_case("tsc")
    {
        return Err(Box::new(InitializeError {
            kind: InitializeErrorKind::ReservedDiagnosticSource,
            diagnostic_source: res.diagnostic_source.clone(),
            ..InitializeError::default()
        }));
    }
    let native_extensions = ALL_SUPPORTED_EXTENSIONS_WITH_JSON_FLATTENED;
    if native_extensions.iter().copied().any(|extension: &str| {
        res.diagnostic_source
            .eq_ignore_ascii_case(extension.trim_start_matches('.'))
    }) {
        return Err(Box::new(InitializeError {
            kind: InitializeErrorKind::ReservedDiagnosticSource,
            diagnostic_source: res.diagnostic_source.clone(),
            ..InitializeError::default()
        }));
    }
    Ok((res.position_encoding, res.diagnostic_source))
}

pub fn decode_transform_result(
    raw: &JsonValue,
    original_text: &str,
    position_encoding: &PositionEncoding,
    diagnostic_source: &str,
) -> Result<TransformOutcome, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("decode_transform_result"); 
    let res: TransformResult = serde_json::from_value(raw.clone())?;
    let (mapped, original_positions) =
        decode_mapped_output(&res.mapped_output, original_text, position_encoding, diagnostic_source)?;
    let mut result = TransformOutcome {
        text: mapped.text.clone(),
        virtual_extension: mapped.virtual_extension.clone(),
        mappings: mapped.mappings.clone(),
        diagnostic_directives: mapped.diagnostic_directives.clone(),
        ..TransformOutcome::default()
    };
    for (supplemental_index, supplemental) in res.supplemental.iter().enumerate() {
        match decode_mapped_output(
            &supplemental.mapped_output,
            original_text,
            position_encoding,
            diagnostic_source,
        ) {
            Ok((mapped, _)) => result.supplemental.push(mapped),
            Err(err) => {
                if let Some(directive_error) =
                    err.downcast_ref::<DiagnosticDirectiveError>()
                {
                    let mut directive_error = *directive_error;
                    directive_error.supplemental_index = supplemental_index as isize;
                    return Err(Box::new(directive_error));
                }
                return Err(err);
            }
        }
    }
    for d in &res.diagnostics {
        if d.start < 0 || d.length < 0 || d.start as i64 > i32::MAX as i64 - d.length as i64 {
            return Err(Box::new(std::io::Error::other(format!(
                "invalid content mapper diagnostic range [{}, {})",
                d.start,
                d.start + d.length
            ))));
        }
        let start = original_positions
            .normalize(d.start)
            .map_err(|err| std::io::Error::other(format!("invalid content mapper diagnostic start: {}", err)))?;
        let end = original_positions
            .normalize(d.start + d.length)
            .map_err(|err| std::io::Error::other(format!("invalid content mapper diagnostic end: {}", err)))?;
        result.diagnostics.push(tsox_frontend::ast::mig::m3d_2::new_external_diagnostic(
            None,
            tsox_core::core::text::TextRange::new(start, end),
            diagnostic_source.to_string(),
            tsox_core::diagnostics::Category::Error,
            d.code,
            d.message_text.clone(),
        ));
    }
    Ok(result)
}

pub fn decode_mapped_output(
    output: &MappedOutput,
    original_text: &str,
    position_encoding: &PositionEncoding,
    diagnostic_source: &str,
) -> Result<(MappedResult, PositionNormalizer), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("decode_mapped_output"); 
    if !is_supported_virtual_extension(&output.extension) {
        return Err(Box::new(InvalidVirtualExtensionError {
            extension: output.extension.clone(),
        }));
    }
    let mut result = MappedResult {
        text: output.text.clone(),
        virtual_extension: output.extension.clone(),
        ..MappedResult::default()
    };
    let virtual_positions = new_position_normalizer(&output.text, position_encoding)?;
    let original_positions = new_position_normalizer(original_text, position_encoding)?;
    if !output.mappings.is_null() && output.mappings.as_str().map(|s| !s.is_empty()).unwrap_or(true) {
        let mappings: spanmap::Segments = serde_json::from_value(output.mappings.clone())?;
        result.mappings = Some(Arc::new(normalize_mappings(
            mappings,
            &virtual_positions,
            &original_positions,
        )?));
    } else {
        result.mappings = Some(Arc::new(spanmap::span_map_new(Vec::new())));
    }
    result.diagnostic_directives = normalize_diagnostic_directives(
        output.diagnostic_directives.as_ref(),
        &virtual_positions,
        &original_positions,
        diagnostic_source,
    )?;
    Ok((result, original_positions))
}

pub fn normalize_diagnostic_directives(
    diagnostic_directives: Option<&DiagnosticDirectives>,
    virtual_positions: &PositionNormalizer,
    original_positions: &PositionNormalizer,
    diagnostic_source: &str,
) -> Result<Vec<MappedDiagnosticDirective>, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("normalize_diagnostic_directives"); 
    let diagnostic_directives = match diagnostic_directives {
        None => return Ok(Vec::new()),
        Some(directives) => directives,
    };
    let directives = &diagnostic_directives.directives;
    let mut result: Vec<Option<MappedDiagnosticDirective>> = Vec::with_capacity(directives.len());
    for (i, directive) in directives.iter().enumerate() {
        let directive_error = |kind: DiagnosticDirectiveErrorKind| {
            DiagnosticDirectiveError {
                kind,
                index: i,
                supplemental_index: -1,
                policy: DiagnosticDirectivePolicy::Ignore,
            }
        };
        let mut normalized = MappedDiagnosticDirective {
            source: diagnostic_source.to_string(),
            ..MappedDiagnosticDirective::default()
        };
        match directive.policy {
            DiagnosticDirectivePolicy::Ignore => {
                normalized.policy = MappedDiagnosticDirectivePolicy::IGNORE;
            }
            DiagnosticDirectivePolicy::Expect => {
                let unused_diagnostic_index = match directive.unused_expect_directive_index {
                    Some(index) => index,
                    None => {
                        if diagnostic_directives.unused_expect_directive_diagnostics.len() != 1 {
                            return Err(Box::new(directive_error(
                                DiagnosticDirectiveErrorKind::ExpectMissingUnusedDiagnostic,
                            )));
                        }
                        0
                    }
                };
                if unused_diagnostic_index < 0
                    || unused_diagnostic_index as usize
                        >= diagnostic_directives.unused_expect_directive_diagnostics.len()
                {
                    return Err(Box::new(directive_error(
                        DiagnosticDirectiveErrorKind::InvalidUnusedDiagnosticIndex,
                    )));
                }
                let unused_diagnostic =
                    &diagnostic_directives.unused_expect_directive_diagnostics[unused_diagnostic_index as usize];
                normalized.policy = MappedDiagnosticDirectivePolicy::EXPECT;
                normalized.unused_code = unused_diagnostic.code;
                normalized.unused_message_text = unused_diagnostic.message_text.clone();
            }
        }
        if directive.virtual_start < 0 || directive.virtual_end < directive.virtual_start {
            return Err(Box::new(directive_error(
                DiagnosticDirectiveErrorKind::InvalidRange,
            )));
        }
        let virtual_start = virtual_positions
            .normalize(directive.virtual_start)
            .map_err(|_| directive_error(DiagnosticDirectiveErrorKind::InvalidRange))?;
        let virtual_end = virtual_positions
            .normalize(directive.virtual_end)
            .map_err(|_| directive_error(DiagnosticDirectiveErrorKind::InvalidRange))?;
        normalized.virtual_range = tsox_core::core::text::TextRange::new(virtual_start, virtual_end);
        let mut valid_original_range = directive.original_start >= 0
            && directive.original_length >= 0
            && directive.original_start as i64 <= i32::MAX as i64 - directive.original_length as i64;
        if valid_original_range {
            match (
                original_positions.normalize(directive.original_start),
                original_positions
                    .normalize(directive.original_start + directive.original_length),
            ) {
                (Ok(original_start), Ok(original_end)) => {
                    normalized.original_range =
                        tsox_core::core::text::TextRange::new(original_start, original_end);
                }
                _ => valid_original_range = false,
            }
        }
        if normalized.policy == MappedDiagnosticDirectivePolicy::EXPECT && !valid_original_range {
            return Err(Box::new(directive_error(
                DiagnosticDirectiveErrorKind::InvalidRange,
            )));
        }
        result.push(Some(normalized));
    }
    let mut result: Vec<MappedDiagnosticDirective> = result.into_iter().flatten().collect();
    let mut sorted: Vec<(usize, &MappedDiagnosticDirective)> =
        result.iter().enumerate().collect();
    sorted.sort_by_key(|(_, directive)| directive.virtual_range.pos());
    for window in sorted.windows(2) {
        if window[1].1.virtual_range.pos() < window[0].1.virtual_range.end() {
            let index = window[1].0;
            return Err(Box::new(DiagnosticDirectiveError {
                kind: DiagnosticDirectiveErrorKind::Overlap,
                index,
                supplemental_index: -1,
                policy: DiagnosticDirectivePolicy::Ignore,
            }));
        }
    }
    result.sort_by_key(|directive| directive.virtual_range.pos());
    Ok(result)
}

pub fn normalize_mappings(
    mappings: spanmap::Segments,
    virtual_positions: &PositionNormalizer,
    original_positions: &PositionNormalizer,
) -> Result<spanmap::SpanMap, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("normalize_mappings"); 
    let mut segments: Vec<spanmap::Segment> = mappings.into();
    for (i, segment) in segments.iter_mut().enumerate() {
        segment.virtual_start = virtual_positions
            .normalize_text_pos(segment.virtual_start)
            .map_err(|err| std::io::Error::other(format!("invalid content mapper mapping {} virtual start: {}", i, err)))?;
        segment.virtual_end = virtual_positions
            .normalize_text_pos(segment.virtual_end)
            .map_err(|err| std::io::Error::other(format!("invalid content mapper mapping {} virtual end: {}", i, err)))?;
        segment.original_start = original_positions
            .normalize_text_pos(segment.original_start)
            .map_err(|err| std::io::Error::other(format!("invalid content mapper mapping {} original start: {}", i, err)))?;
        segment.original_end = original_positions
            .normalize_text_pos(segment.original_end)
            .map_err(|err| std::io::Error::other(format!("invalid content mapper mapping {} original end: {}", i, err)))?;
    }
    Ok(spanmap::span_map_new(segments))
}

pub struct PositionNormalizer {
    text: String,
    encoding: PositionEncoding,
    position_map: Option<tsox_frontend::ast::positionmap::PositionMap>,
    length: usize,
}

pub fn new_position_normalizer(
    text: &str,
    encoding: &PositionEncoding,
) -> Result<PositionNormalizer, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("new_position_normalizer"); 
    let mut normalizer = PositionNormalizer {
        text: text.to_string(),
        encoding: encoding.clone(),
        position_map: None,
        length: 0,
    };
    match encoding.as_str() {
        POSITION_ENCODING_UTF8 => normalizer.length = text.len(),
        POSITION_ENCODING_UTF16 => {
            let position_map = tsox_frontend::ast::positionmap::compute_position_map(text);
            normalizer.length = position_map.utf8_to_utf16(text.len());
            normalizer.position_map = Some(position_map);
        }
        _ => {
            return Err(Box::new(std::io::Error::other(format!(
                "unsupported position encoding {:?}",
                encoding
            ))))
        }
    }
    Ok(normalizer)
}

impl PositionNormalizer {
    pub fn normalize_text_pos(
        &self,
        position: tsox_core::core::text::TextPos,
    ) -> Result<tsox_core::core::text::TextPos, String> { ::tsox_core::fntrace::enter("normalize_text_pos"); 
        let normalized = self.normalize(position)?;
        Ok(normalized as tsox_core::core::text::TextPos)
    }

    pub fn normalize(&self, position: i32) -> Result<usize, String> { ::tsox_core::fntrace::enter("normalize"); 
        let position = position as i64;
        if position < 0 {
            return Err(format!("position {} is negative", position));
        }
        if position > self.length as i64 {
            return Err(format!(
                "position {} exceeds {} length {}",
                position, self.encoding, self.length
            ));
        }
        let position = position as usize;
        let byte_position = match self.encoding.as_str() {
            POSITION_ENCODING_UTF8 => position,
            POSITION_ENCODING_UTF16 => self
                .position_map
                .as_ref()
                .map(|map| map.utf16_to_utf8(position))
                .unwrap_or(position),
            _ => position,
        };
        let bytes = self.text.as_bytes();
        if byte_position < bytes.len() {
            let byte = bytes[byte_position];
            if byte & 0xC0 == 0x80 {
                return Err(format!(
                    "position {} splits a Unicode code point",
                    position
                ));
            }
        }
        Ok(byte_position)
    }
}

pub struct RejectHandler;

pub trait RequestHandler {
    fn handle_request(
        &self,
        method: &str,
        params: &JsonValue,
    ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>>;
    fn handle_notification(&self, method: &str, params: &JsonValue)
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl RequestHandler for RejectHandler {
    fn handle_request(
        &self,
        method: &str,
        _params: &JsonValue,
    ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("handle_request"); 
        Err(Box::new(std::io::Error::other(format!(
            "content mapper sent an unexpected request: {}",
            method
        ))))
    }

    fn handle_notification(
        &self,
        _method: &str,
        _params: &JsonValue,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("handle_notification"); 
        Ok(())
    }
}

pub fn conn_duplicate_handle(conn: &dyn IpcConn) -> Box<dyn IpcConn> { ::tsox_core::fntrace::enter("conn_duplicate_handle"); 
    conn.duplicate_handle()
}

pub fn closer_into_arc(closer: Box<dyn Closer>) -> Arc<dyn Closer> { ::tsox_core::fntrace::enter("closer_into_arc"); 
    Arc::from(closer)
}

pub struct ProcessCloser(pub Arc<CloseOnceReadWriteCloser>);

impl Closer for ProcessCloser {
    fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("close"); 
        self.0
            .close()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send + Sync>)
    }
}

pub struct JsonRpcProtocol {
    rwc: Arc<CloseOnceReadWriteCloser>,
    read_buf: Mutex<Vec<u8>>,
}

pub fn new_jsonrpc_protocol(rwc: Arc<CloseOnceReadWriteCloser>) -> Box<dyn IpcProtocol> { ::tsox_core::fntrace::enter("new_jsonrpc_protocol"); 
    Box::new(JsonRpcProtocol {
        rwc,
        read_buf: Mutex::new(Vec::new()),
    })
}

const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

impl JsonRpcProtocol {
    fn fill_until(&self, buf: &mut Vec<u8>, delimiter: &[u8]) -> std::io::Result<()> { ::tsox_core::fntrace::enter("fill_until"); 
        let mut chunk = [0u8; 4096];
        loop {
            if find_subslice(buf, delimiter).is_some() {
                return Ok(());
            }
            let read = self.rwc.lock_inner().read(&mut chunk)?;
            if read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "jsonrpc: connection closed",
                ));
            }
            buf.extend_from_slice(&chunk[..read]);
            if buf.len() > MAX_MESSAGE_SIZE {
                return Err(std::io::Error::other("jsonrpc: message too large"));
            }
        }
    }

    fn read_exact_from_buf(&self, buf: &mut Vec<u8>, count: usize) -> std::io::Result<()> { ::tsox_core::fntrace::enter("read_exact_from_buf"); 
        let mut chunk = [0u8; 4096];
        while buf.len() < count {
            let read = self.rwc.lock_inner().read(&mut chunk)?;
            if read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "jsonrpc: connection closed",
                ));
            }
            buf.extend_from_slice(&chunk[..read]);
        }
        Ok(())
    }

    fn write_all_raw(&self, data: &[u8]) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_all_raw"); 
        self.rwc.lock_inner().write_all(data)
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> { ::tsox_core::fntrace::enter("find_subslice"); 
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn read_payload(protocol: &JsonRpcProtocol) -> std::io::Result<Vec<u8>> { ::tsox_core::fntrace::enter("read_payload"); 
    let mut buf = protocol.read_buf.lock().unwrap();
    protocol.fill_until(&mut buf, b"\r\n\r\n")?;
    let header_end = find_subslice(&buf, b"\r\n\r\n").unwrap() + 4;
    let headers = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut content_length = None;
    for line in headers[..headers.len() - 4].split("\r\n") {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim() == "Content-Length" {
                content_length = Some(value.trim().parse::<usize>().map_err(|_| {
                    std::io::Error::other("jsonrpc: invalid content length")
                })?);
            }
        }
    }
    let content_length = content_length
        .ok_or_else(|| std::io::Error::other("jsonrpc: no content length"))?;
    buf.drain(..header_end);
    protocol.read_exact_from_buf(&mut buf, content_length)?;
    if buf.len() > content_length {
        return Err(std::io::Error::other("jsonrpc: message too large"));
    }
    Ok(std::mem::take(&mut buf))
}

fn write_payload(protocol: &JsonRpcProtocol, data: &[u8]) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_payload"); 
    let mut frame = format!("Content-Length: {}\r\n\r\n", data.len()).into_bytes();
    frame.extend_from_slice(data);
    protocol.write_all_raw(&frame)
}

impl IpcProtocol for JsonRpcProtocol {
    fn read_message(&self) -> std::io::Result<IpcMessage> { ::tsox_core::fntrace::enter("read_message"); 
        let data = read_payload(self)?;
        serde_json::from_slice(&data)
            .map_err(|err| std::io::Error::other(format!("jsonrpc: invalid message: {}", err)))
    }

    fn write_request(
        &self,
        id: Option<&JsonRpcId>,
        method: &str,
        params: &JsonValue,
    ) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_request"); 
        let message = serde_json::to_vec(&JsonRpcRequestMessage {
            id: id.cloned(),
            method: method.to_string(),
            params: params.clone(),
        })
        .map_err(std::io::Error::other)?;
        write_payload(self, &message)
    }

    fn write_notification(&self, method: &str, params: &JsonValue) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_notification"); 
        let message = serde_json::to_vec(&JsonRpcRequestMessage {
            id: None,
            method: method.to_string(),
            params: params.clone(),
        })
        .map_err(std::io::Error::other)?;
        write_payload(self, &message)
    }

    fn write_response(&self, id: &JsonRpcId, result: &JsonValue) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_response"); 
        let result = if result.is_null() {
            JsonValue::Null
        } else {
            result.clone()
        };
        let message = serde_json::to_vec(&JsonRpcResponseMessage {
            id: Some(id.clone()),
            result,
            error: None,
        })
        .map_err(std::io::Error::other)?;
        write_payload(self, &message)
    }

    fn write_error(&self, id: &JsonRpcId, response_error: &JsonRpcResponseError) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_error"); 
        let message = serde_json::to_vec(&JsonRpcResponseMessage {
            id: Some(id.clone()),
            result: JsonValue::Null,
            error: Some(response_error.clone()),
        })
        .map_err(std::io::Error::other)?;
        write_payload(self, &message)
    }
}

pub struct AsyncConnShared {
    rwc: Arc<CloseOnceReadWriteCloser>,
    protocol: Box<dyn IpcProtocol>,
    handler: Box<dyn RequestHandler + Send + Sync>,
    pending: Mutex<HashMap<JsonRpcId, std::sync::mpsc::Sender<IpcMessage>>>,
    terminal: Mutex<Option<Box<dyn std::error::Error + Send + Sync>>>,
    seq: AtomicU64,
}

pub struct AsyncConn {
    shared: Arc<AsyncConnShared>,
}

pub fn new_async_conn_with_protocol(
    rwc: Arc<CloseOnceReadWriteCloser>,
    protocol: Box<dyn IpcProtocol>,
    handler: impl RequestHandler + Send + Sync + 'static,
) -> AsyncConn { ::tsox_core::fntrace::enter("new_async_conn_with_protocol"); 
    AsyncConn {
        shared: Arc::new(AsyncConnShared {
            rwc,
            protocol,
            handler: Box::new(handler),
            pending: Mutex::new(HashMap::new()),
            terminal: Mutex::new(None),
            seq: AtomicU64::new(0),
        }),
    }
}

impl AsyncConn {
    pub fn run_detached(&self) { ::tsox_core::fntrace::enter("run_detached"); 
        let shared = self.shared.clone();
        std::thread::Builder::new()
            .name("content-mapper-ipc".to_string())
            .spawn(move || {
                let result = AsyncConn::run_shared(&shared);
                shared.close_pending_calls(result.err());
                let _ = shared.rwc.close();
            })
            .ok();
    }

    fn run_shared(
        shared: &Arc<AsyncConnShared>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("run_shared"); 
        loop {
            let message = shared.protocol.read_message()?;
            if message.id.is_some() && message.method.is_empty() {
                shared.handle_response(message);
            } else if message.id.is_some() {
                let shared = Arc::clone(shared);
                std::thread::spawn(move || shared.handle_request(message));
            } else if !message.method.is_empty() {
                let _ = shared
                    .handler
                    .handle_notification(&message.method, &message.params);
            }
        }
    }
}

impl AsyncConnShared {
    fn close_pending_calls(&self, run_err: Option<Box<dyn std::error::Error + Send + Sync>>) { ::tsox_core::fntrace::enter("close_pending_calls"); 
        let mut pending = self.pending.lock().unwrap();
        let mut terminal = self.terminal.lock().unwrap();
        if terminal.is_none() {
            *terminal = Some(match run_err {
                Some(err) => err,
                None => Box::new(std::io::Error::other("ipc: connection closed")),
            });
        }
        pending.clear();
    }

    fn handle_response(&self, message: IpcMessage) { ::tsox_core::fntrace::enter("handle_response"); 
        let sender = match message.id.as_ref() {
            Some(id) => self.pending.lock().unwrap().remove(id),
            None => None,
        };
        if let Some(sender) = sender {
            let _ = sender.send(message);
        }
    }

    fn handle_request(&self, message: IpcMessage) { ::tsox_core::fntrace::enter("handle_request"); 
        let id = match message.id.as_ref() {
            None => return,
            Some(id) => id.clone(),
        };
        match self.handler.handle_request(&message.method, &message.params) {
            Ok(result) => {
                let _ = self.protocol.write_response(&id, &result);
            }
            Err(err) => {
                let _ = self.protocol.write_error(
                    &id,
                    &JsonRpcResponseError {
                        code: -32603,
                        message: err.to_string(),
                        data: None,
                    },
                );
            }
        }
    }

    fn call_raw(
        &self,
        method: &str,
        params: &JsonValue,
    ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("call_raw"); 
        let id = JsonRpcId::String(format!("api{}", self.seq.fetch_add(1, Ordering::SeqCst) + 1));
        let (sender, receiver) = std::sync::mpsc::channel::<IpcMessage>();
        {
            let mut pending = self.pending.lock().unwrap();
            if let Some(terminal) = self.terminal.lock().unwrap().as_ref() {
                return Err(std::io::Error::other(terminal.to_string()).into());
            }
            pending.insert(id.clone(), sender);
        }
        if let Err(err) = self.protocol.write_request(Some(&id), method, params) {
            self.pending.lock().unwrap().remove(&id);
            return Err(Box::new(err));
        }
        match receiver.recv() {
            Ok(message) => match message.error {
                Some(response_error) => Err(Box::new(std::io::Error::other(format!(
                    "ipc: remote error [{}]: {}",
                    response_error.code, response_error.message
                )))),
                None => Ok(message.result),
            },
            Err(_) => {
                let terminal = self.terminal.lock().unwrap().as_ref().map(|err| err.to_string());
                Err(Box::new(std::io::Error::other(match terminal {
                    Some(detail) => detail,
                    None => "ipc: connection closed".to_string(),
                })))
            }
        }
    }
}

impl IpcConn for AsyncConn {
    fn call(
        &self,
        method: &str,
        params: &JsonValue,
    ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("call"); 
        self.shared.call_raw(method, params)
    }

    fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("run"); 
        AsyncConn::run_shared(&self.shared)
    }

    fn duplicate_handle(&self) -> Box<dyn IpcConn> { ::tsox_core::fntrace::enter("duplicate_handle"); 
        Box::new(AsyncConn {
            shared: self.shared.clone(),
        })
    }
}
