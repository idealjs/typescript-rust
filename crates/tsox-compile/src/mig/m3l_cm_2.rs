use super::m3l_cm::{
    is_supported_virtual_extension, new_transform_error, DiagnosticDirectiveError,
    DiagnosticDirectiveErrorKind, DiagnosticDirectivePolicy, InitializeError, InitializeErrorKind,
    JsonValue, Mapper, OperationTiming, ProjectError, ProjectErrorKind, SupplementalFileCollisionError,
    TransformError, TransformErrorKind, MapperTimings, Timings,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Once, Weak};
use std::time::{Duration, Instant, SystemTime};
use tsox_frontend::ast::mig::m3b_2::MappedDiagnosticDirective;
use self::spanmap::SpanMap;

pub const METHOD_INITIALIZE: &str = "initialize";
pub const METHOD_OPEN_PROJECT: &str = "openProject";
pub const METHOD_CLOSE_PROJECT: &str = "closeProject";
pub const METHOD_TRANSFORM: &str = "transform";

pub const INITIALIZE_TIMEOUT_SECONDS: u64 = 5;

pub const POSITION_ENCODING_UTF8: &str = "utf-8";
pub const POSITION_ENCODING_UTF16: &str = "utf-16";

pub type PositionEncoding = String;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub locale: String,
    pub position_encodings: Vec<PositionEncoding>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitializeResult {
    pub position_encoding: PositionEncoding,
    pub diagnostic_source: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenProjectParams {
    pub config_file_name: String,
    pub project_handle: String,
    #[serde(default, skip_serializing_if = "JsonValue::is_null")]
    pub options: JsonValue,
    pub compiler_options: JsonValue,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenProjectResult {
    pub config_identity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub watched_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub option_diagnostics: Vec<OptionDiagnosticResult>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptionDiagnosticResult {
    pub path: Vec<JsonValue>,
    pub message_text: String,
    pub code: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloseProjectParams {
    pub project_handle: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformParams {
    pub file_name: String,
    pub content: String,
    pub project_handle: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MappedOutput {
    pub text: String,
    pub extension: String,
    #[serde(default, skip_serializing_if = "JsonValue::is_null")]
    pub mappings: JsonValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic_directives: Option<DiagnosticDirectives>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnusedExpectDirectiveDiagnostic {
    pub code: i32,
    pub message_text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticDirectives {
    pub unused_expect_directive_diagnostics: Vec<UnusedExpectDirectiveDiagnostic>,
    pub directives: Vec<MappedDiagnosticDirectiveTuple>,
}

#[derive(Debug, Clone, Default)]
pub struct MappedDiagnosticDirectiveTuple {
    pub original_start: i32,
    pub original_length: i32,
    pub virtual_start: i32,
    pub virtual_end: i32,
    pub policy: DiagnosticDirectivePolicy,
    pub unused_expect_directive_index: Option<i32>,
}

impl Serialize for MappedDiagnosticDirectiveTuple {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        marshal_json_to_mapped_diagnostic_directive(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MappedDiagnosticDirectiveTuple {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let tuple = Vec::<JsonValue>::deserialize(deserializer)?;
        unmarshal_json_from_mapped_diagnostic_directive(&tuple).map_err(serde::de::Error::custom)
    }
}

pub fn marshal_json_to_mapped_diagnostic_directive(
    d: &MappedDiagnosticDirectiveTuple,
) -> Result<Vec<JsonValue>, serde_json::Error> { ::tsox_core::fntrace::enter("marshal_json_to_mapped_diagnostic_directive"); 
    let mut tuple = vec![
        JsonValue::from(d.original_start),
        JsonValue::from(d.original_length),
        JsonValue::from(d.virtual_start),
        JsonValue::from(d.virtual_end),
        JsonValue::from(d.policy as u8),
    ];
    if let Some(index) = d.unused_expect_directive_index {
        tuple.push(JsonValue::from(index));
    }
    Ok(tuple)
}

pub fn unmarshal_json_from_mapped_diagnostic_directive(
    tuple: &[JsonValue],
) -> Result<MappedDiagnosticDirectiveTuple, String> { ::tsox_core::fntrace::enter("unmarshal_json_from_mapped_diagnostic_directive"); 
    if tuple.len() != 5 && tuple.len() != 6 {
        return Err(format!(
            "diagnostic directive tuple must contain 5 or 6 elements, got {}",
            tuple.len()
        ));
    }
    let mut d = MappedDiagnosticDirectiveTuple::default();
    let fields: [(&JsonValue, &str); 5] = [
        (&tuple[0], "invalid diagnostic directive tuple element 0"),
        (&tuple[1], "invalid diagnostic directive tuple element 1"),
        (&tuple[2], "invalid diagnostic directive tuple element 2"),
        (&tuple[3], "invalid diagnostic directive tuple element 3"),
        (&tuple[4], "invalid diagnostic directive tuple element 4"),
    ];
    let mut values: [i32; 5] = [0; 5];
    for (i, (value, _)) in fields.iter().enumerate() {
        values[i] = value
            .as_i64()
            .ok_or_else(|| format!("invalid diagnostic directive tuple element {}", i))?
            as i32;
    }
    d.original_start = values[0];
    d.original_length = values[1];
    d.virtual_start = values[2];
    d.virtual_end = values[3];
    d.policy = match values[4] {
        0 => DiagnosticDirectivePolicy::Ignore,
        1 => DiagnosticDirectivePolicy::Expect,
        _ => return Err("invalid diagnostic directive tuple element 4".to_string()),
    };
    if tuple.len() == 6 {
        let index = tuple[5]
            .as_i64()
            .ok_or_else(|| "invalid diagnostic directive tuple element 5".to_string())?;
        d.unused_expect_directive_index = Some(index as i32);
    }
    Ok(d)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SupplementalOutput {
    #[serde(flatten)]
    pub mapped_output: MappedOutput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformResult {
    #[serde(flatten)]
    pub mapped_output: MappedOutput,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supplemental: Vec<SupplementalOutput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message_text: String,
    pub start: i32,
    pub length: i32,
    pub code: i32,
}

#[derive(Debug, Clone, Default)]
pub struct TransformOutcome {
    pub text: String,
    pub virtual_extension: String,
    pub diagnostics: Vec<tsox_frontend::ast::Diagnostic>,
    pub mappings: Option<Arc<SpanMap>>,
    pub diagnostic_directives: Vec<MappedDiagnosticDirective>,
    pub supplemental: Vec<MappedResult>,
}

#[derive(Debug, Clone, Default)]
pub struct MappedResult {
    pub text: String,
    pub virtual_extension: String,
    pub mappings: Option<Arc<SpanMap>>,
    pub diagnostic_directives: Vec<MappedDiagnosticDirective>,
}

#[derive(Debug, Clone, Default)]
pub struct Request {
    pub file_name: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectSpec {
    pub config_file_name: String,
    pub mappers: Vec<Arc<Mapper>>,
    pub compiler_options: Option<Arc<tsox_core::core::compiler_options::CompilerOptions>>,
}

#[derive(Debug, Clone, Default)]
pub struct OptionPathSegment {
    pub property: String,
    pub index: i32,
    pub is_index: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OptionDiagnostic {
    pub mapper: Arc<Mapper>,
    pub path: Vec<OptionPathSegment>,
    pub source: String,
    pub code: i32,
    pub message_text: String,
}

#[derive(Clone)]
pub struct SourceFiles {
    pub canonical: Option<std::sync::Arc<tsox_frontend::ast::SourceFile>>,
    pub supplemental: Vec<std::sync::Arc<tsox_frontend::ast::SourceFile>>,
}

impl Default for SourceFiles {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        SourceFiles {
            canonical: None,
            supplemental: Vec::new(),
        }
    }
}

pub fn transform_and_parse(
    parse_options: tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions,
    content: &str,
    mapper: &Mapper,
    project: &dyn Project,
) -> Result<SourceFiles, TransformError> { ::tsox_core::fntrace::enter("transform_and_parse"); 
    let transform_identity = project
        .identity(mapper)
        .map_err(|err| new_transform_error(TransformErrorKind::Project, err))?;
    let result = project
        .transform(
            mapper,
            &Request {
                file_name: parse_options.file_name.clone(),
                content: content.to_string(),
            },
        )
        .map_err(|err| err)?;
    parse_result(parse_options, content, mapper, &transform_identity, result)
}

pub fn parse_result(
    mut parse_options: tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions,
    content: &str,
    mapper: &Mapper,
    transform_identity: &str,
    mut result: TransformOutcome,
) -> Result<SourceFiles, TransformError> { ::tsox_core::fntrace::enter("parse_result"); 
    let mappings = match &result.mappings {
        None => return Err(new_transform_error(TransformErrorKind::Mappings, Box::new(ProjectError::default()))),
        Some(mappings) => mappings.clone(),
    };
    if let Some(problem) = mappings.validate(&result.text, content) {
        return Err(new_transform_error(TransformErrorKind::Mappings, problem));
    }
    let virtual_extension = result.virtual_extension.clone();
    if !is_supported_virtual_extension(&virtual_extension) {
        return Err(new_transform_error(TransformErrorKind::Response, Box::new(ProjectError::default())));
    }
    let mut base_parse_options = parse_options.clone();
    let virtual_file_name = format!("{}{}", base_parse_options.file_name, virtual_extension);
    parse_options = base_parse_options.clone();
    if is_module_virtual_extension(&virtual_extension) {
        parse_options.external_module_indicator_options.force = true;
    }
    let mut source_file = std::sync::Arc::new(parse_source_file(
        &parse_options,
        result.text.clone(),
        get_script_kind_from_file_name(&virtual_file_name),
    ));
    if !result.diagnostics.is_empty() {
        for diagnostic in &mut result.diagnostics {
            diagnostic.set_file(Some(source_file.clone()));
        }
    }
    let mut files = SourceFiles {
        canonical: Some(source_file.clone()),
        supplemental: Vec::with_capacity(result.supplemental.len()),
    };
    for (i, supplemental) in result.supplemental.iter().enumerate() {
        let supplemental_mappings = match &supplemental.mappings {
            None => {
                return Err(new_transform_error(
                    TransformErrorKind::Mappings,
                    Box::new(ProjectError::default()),
                ))
            }
            Some(mappings) => mappings.clone(),
        };
        if let Some(problem) = supplemental_mappings.validate(&supplemental.text, content) {
            return Err(new_transform_error(TransformErrorKind::Mappings, problem));
        }
        let mut supplemental_options = base_parse_options.clone();
        if !is_supported_virtual_extension(&supplemental.virtual_extension) {
            return Err(new_transform_error(
                TransformErrorKind::Response,
                Box::new(ProjectError::default()),
            ));
        }
        let suffix = format!(".{}{}", i, supplemental.virtual_extension);
        supplemental_options.file_name += &suffix;
        supplemental_options.path = format!("{}{}", parse_options.path.as_str(), suffix);
        if is_module_virtual_extension(&supplemental.virtual_extension) {
            supplemental_options.external_module_indicator_options.force = true;
        }

        let file = std::sync::Arc::new(parse_source_file(
            &supplemental_options,
            supplemental.text.clone(),
            get_script_kind_from_file_name(&supplemental_options.file_name),
        ));
        files.supplemental.push(file);
    }
    let mapper_identity = mapper.identity();
    {
        let source_file =
            std::sync::Arc::get_mut(&mut source_file).expect("canonical source file is uniquely held");
        source_file.set_content_mapper_info(ContentMapperSourceFileInfo {
            content_mapper: mapper_identity.clone(),
            transform_identity: transform_identity.to_string(),
            parse_options: base_parse_options.clone(),
            virtual_file_name: virtual_file_name.clone(),
            original_text: content.to_string(),
            span_map: mappings,
            diagnostic_directives: result.diagnostic_directives.clone(),
            supplemental_source_files: files.supplemental.clone(),
            canonical_source_file: None,
        });
    }
    for (i, file) in files.supplemental.iter().enumerate() {
        let supplemental = &result.supplemental[i];
        file.set_content_mapper_info(ContentMapperSourceFileInfo {
            content_mapper: mapper_identity.clone(),
            transform_identity: transform_identity.to_string(),
            parse_options: base_parse_options.clone(),
            virtual_file_name: file.file_name.clone(),
            original_text: content.to_string(),
            span_map: supplemental.mappings.clone().unwrap_or_default(),
            diagnostic_directives: supplemental.diagnostic_directives.clone(),
            supplemental_source_files: Vec::new(),
            canonical_source_file: Some(source_file.clone()),
        });
    }
    Ok(files)
}

pub fn is_module_virtual_extension(extension: &str) -> bool { ::tsox_core::fntrace::enter("is_module_virtual_extension"); 
    matches!(
        extension,
        tsox_core::tspath::EXTENSION_MTS
            | tsox_core::tspath::EXTENSION_CTS
            | tsox_core::tspath::EXTENSION_MJS
            | tsox_core::tspath::EXTENSION_CJS
    )
}

pub fn check_supplemental_file_name_collisions(
    files: &SourceFiles,
    file_exists: &dyn Fn(&str) -> bool,
) -> Result<(), SupplementalFileCollisionError> { ::tsox_core::fntrace::enter("check_supplemental_file_name_collisions"); 
    for file in &files.supplemental {
        if file_exists(&file.file_name) {
            return Err(SupplementalFileCollisionError {
                file_name: file.file_name.clone(),
            });
        }
    }
    Ok(())
}

pub trait Project: Send + Sync {
    fn refresh(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn identities(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
    fn identity(&self, mapper: &Mapper) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    fn watched_files(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
    fn diagnostics(&self) -> Vec<OptionDiagnostic>;
    fn transform(&self, mapper: &Mapper, request: &Request) -> Result<TransformOutcome, TransformError>;
    fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub trait Host {
    fn timings(&self) -> Timings;
    fn project(&self, spec: &ProjectSpec) -> Option<Arc<dyn Project>>;
    fn acquire(&self, mappers: &[Arc<Mapper>]) -> Box<dyn FnOnce()>;
    fn set_locale(&self, locale: tsox_core::locale::Locale);
    fn transform(&self, mapper: &Mapper, request: &Request) -> Result<TransformOutcome, TransformError>;
    fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct OperationTimingInner {
    count: AtomicU64,
    duration: AtomicI64,
}

impl Default for OperationTimingInner {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        OperationTimingInner {
            count: AtomicU64::new(0),
            duration: AtomicI64::new(0),
        }
    }
}

impl OperationTimingInner {
    pub fn record(&self, start: Instant) { ::tsox_core::fntrace::enter("record"); 
        self.count.fetch_add(1, Ordering::Relaxed);
        self.duration
            .fetch_add(start.elapsed().as_nanos() as i64, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> OperationTiming { ::tsox_core::fntrace::enter("snapshot"); 
        OperationTiming {
            count: self.count.load(Ordering::Relaxed),
            duration: Duration::from_nanos(self.duration.load(Ordering::Relaxed).max(0) as u64),
        }
    }
}

pub struct TimingCollector {
    pub(crate) mu: Mutex<TimingCollectorState>,
}

pub struct TimingCollectorState {
    pub mappers: std::collections::HashMap<String, Arc<MapperTimingCollector>>,
    pub active_requests: u64,
    pub request_wait_start: Option<Instant>,
    pub request_wait_elapsed: Duration,
}

impl TimingCollector {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        TimingCollector {
            mu: Mutex::new(TimingCollectorState {
                mappers: std::collections::HashMap::new(),
                active_requests: 0,
                request_wait_start: None,
                request_wait_elapsed: Duration::ZERO,
            }),
        }
    }

    pub fn mapper(self: &Arc<Self>, identity: &str) -> Arc<MapperTimingCollector> { ::tsox_core::fntrace::enter("mapper"); 
        let mut state = self.mu.lock().unwrap();
        if let Some(timing) = state.mappers.get(identity) {
            return timing.clone();
        }
        let timing = Arc::new(MapperTimingCollector {
            spawn: OperationTimingInner::default(),
            initialize: OperationTimingInner::default(),
            open_project: OperationTimingInner::default(),
            close_project: OperationTimingInner::default(),
            transform: OperationTimingInner::default(),
            owner: Arc::downgrade(self),
        });
        state.mappers.insert(identity.to_string(), timing.clone());
        timing
    }

    pub fn snapshot(&self) -> Timings { ::tsox_core::fntrace::enter("snapshot"); 
        let request_wait;
        let mappers;
        {
            let mut state = self.mu.lock().unwrap();
            request_wait = state.request_wait_elapsed
                + state
                    .request_wait_start
                    .map(|start| start.elapsed())
                    .unwrap_or_else(|| Duration::ZERO);
            mappers = state.mappers.clone();
            if state.active_requests == 0 {
                state.request_wait_start = None;
            }
        }
        let mut result = Timings {
            mappers: std::collections::HashMap::with_capacity(mappers.len()),
            request_wait,
        };
        for (identity, timing) in mappers {
            result.mappers.insert(
                identity,
                MapperTimings {
                    spawn: timing.spawn.snapshot(),
                    initialize: timing.initialize.snapshot(),
                    open_project: timing.open_project.snapshot(),
                    close_project: timing.close_project.snapshot(),
                    transform: timing.transform.snapshot(),
                },
            );
        }
        result
    }
}

pub struct MapperTimingCollector {
    pub spawn: OperationTimingInner,
    pub initialize: OperationTimingInner,
    pub open_project: OperationTimingInner,
    pub close_project: OperationTimingInner,
    pub transform: OperationTimingInner,
    pub owner: Weak<TimingCollector>,
}

impl MapperTimingCollector {
    pub fn start_request(&self) -> Instant { ::tsox_core::fntrace::enter("start_request"); 
        let owner = self
            .owner
            .upgrade()
            .expect("timing collector owner dropped while mapper timing is alive");
        let mut state = owner.mu.lock().unwrap();
        if state.active_requests == 0 {
            state.request_wait_start = Some(Instant::now());
        }
        state.active_requests += 1;
        Instant::now()
    }

    pub fn finish_request(&self, operation: &OperationTimingInner, start: Instant) { ::tsox_core::fntrace::enter("finish_request"); 
        operation.record(start);
        let owner = self
            .owner
            .upgrade()
            .expect("timing collector owner dropped while mapper timing is alive");
        let mut state = owner.mu.lock().unwrap();
        state.active_requests -= 1;
        if state.active_requests == 0 {
            let wait = state
                .request_wait_start
                .map(|wait_start| wait_start.elapsed())
                .unwrap_or_else(|| Duration::ZERO);
            state.request_wait_elapsed += wait;
        }
    }
}

#[derive(Clone)]
pub struct Logger(pub std::sync::Arc<dyn Fn(&str) + Send + Sync>);

impl Logger {
    pub fn from_fn(f: std::sync::Arc<dyn Fn(&str) + Send + Sync>) -> Logger { ::tsox_core::fntrace::enter("from_fn"); 
        Logger(f)
    }
}

#[derive(Default)]
pub struct HostOptions {
    pub logger: Option<Logger>,
}

pub type SpawnerFn =
    fn(command: &[String], dir: &str, stderr: &mut dyn std::io::Write) -> std::io::Result<Box<dyn ReadWriteCloser>>;

pub trait Spawner: Send + Sync {
    fn spawn(
        &self,
        command: &[String],
        dir: &str,
        stderr: &mut dyn std::io::Write,
    ) -> std::io::Result<Box<dyn ReadWriteCloser>>;
}

pub struct SpawnerFunc(pub SpawnerFn);

impl Spawner for SpawnerFunc {
    fn spawn(
        &self,
        command: &[String],
        dir: &str,
        stderr: &mut dyn std::io::Write,
    ) -> std::io::Result<Box<dyn ReadWriteCloser>> { ::tsox_core::fntrace::enter("spawn"); 
        (self.0)(command, dir, stderr)
    }
}

pub trait ReadWriteCloser: std::io::Read + std::io::Write + Send {
    fn close(&mut self) -> std::io::Result<()>;
    fn exit_code(&self) -> Option<i32> { ::tsox_core::fntrace::enter("exit_code"); 
        None
    }
}

pub struct LoggingProtocol {
    pub protocol: Box<dyn IpcProtocol>,
    pub mapper_name: String,
    pub logger: Logger,
}

impl LoggingProtocol {
    pub fn log<T: serde::Serialize>(&self, direction: &str, message: &T) { ::tsox_core::fntrace::enter("log"); 
        match serde_json::to_string(message) {
            Ok(data) => (self.logger.0)(
                &format!("[content mapper: {}] {}: {}", self.mapper_name, direction, data),
            ),
            Err(err) => (self.logger.0)(&format!(
                "[content mapper: {}] {}: <failed to serialize: {}>",
                self.mapper_name, direction, err
            )),
        }
    }
}

impl IpcProtocol for LoggingProtocol {
    fn read_message(&self) -> std::io::Result<IpcMessage> { ::tsox_core::fntrace::enter("read_message"); 
        let message = self.protocol.read_message()?;
        self.log("receive", &message);
        Ok(message)
    }

    fn write_request(
        &self,
        id: Option<&JsonRpcId>,
        method: &str,
        params: &JsonValue,
    ) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_request"); 
        self.log(
            "send",
            &JsonRpcRequestMessage {
                id: id.cloned(),
                method: method.to_string(),
                params: params.clone(),
            },
        );
        self.protocol.write_request(id, method, params)
    }

    fn write_notification(&self, method: &str, params: &JsonValue) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_notification"); 
        self.log(
            "send",
            &JsonRpcRequestMessage {
                id: None,
                method: method.to_string(),
                params: params.clone(),
            },
        );
        self.protocol.write_notification(method, params)
    }

    fn write_response(&self, id: &JsonRpcId, result: &JsonValue) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_response"); 
        self.log(
            "send",
            &JsonRpcResponseMessage {
                id: Some(id.clone()),
                result: result.clone(),
                error: None,
            },
        );
        self.protocol.write_response(id, result)
    }

    fn write_error(&self, id: &JsonRpcId, response_error: &JsonRpcResponseError) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_error"); 
        self.log(
            "send",
            &JsonRpcResponseMessage {
                id: Some(id.clone()),
                result: JsonValue::Null,
                error: Some(response_error.clone()),
            },
        );
        self.protocol.write_error(id, response_error)
    }
}

#[derive(Serialize)]
pub(crate) struct JsonRpcRequestMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<JsonRpcId>,
    pub(crate) method: String,
    #[serde(skip_serializing_if = "JsonValue::is_null")]
    pub(crate) params: JsonValue,
}

#[derive(Serialize)]
pub(crate) struct JsonRpcResponseMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<JsonRpcId>,
    #[serde(skip_serializing_if = "JsonValue::is_null")]
    pub(crate) result: JsonValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<JsonRpcResponseError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Number(i32),
    String(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsonRpcResponseError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IpcMessage {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub method: String,
    #[serde(default, skip_serializing_if = "JsonValue::is_null")]
    pub params: JsonValue,
    #[serde(default, skip_serializing_if = "JsonValue::is_null")]
    pub result: JsonValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcResponseError>,
}

pub trait IpcProtocol: Send + Sync {
    fn read_message(&self) -> std::io::Result<IpcMessage>;
    fn write_request(
        &self,
        id: Option<&JsonRpcId>,
        method: &str,
        params: &JsonValue,
    ) -> std::io::Result<()>;
    fn write_notification(&self, method: &str, params: &JsonValue) -> std::io::Result<()>;
    fn write_response(&self, id: &JsonRpcId, result: &JsonValue) -> std::io::Result<()>;
    fn write_error(
        &self,
        id: &JsonRpcId,
        response_error: &JsonRpcResponseError,
    ) -> std::io::Result<()>;
}

pub struct StderrLogger {
    mapper_name: String,
    logger: Logger,
    mu: Mutex<String>,
}

impl StderrLogger {
    pub fn new(mapper_name: String, logger: Logger) -> Self { ::tsox_core::fntrace::enter("new"); 
        StderrLogger {
            mapper_name,
            logger,
            mu: Mutex::new(String::new()),
        }
    }

    pub fn write_line(&self, data: &[u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("write_line"); 
        let mut pending = self.mu.lock().unwrap();
        pending.push_str(&String::from_utf8_lossy(data));
        loop {
            let index = match pending.find('\n') {
                Some(index) => index,
                None => break,
            };
            let line: String = pending[..index].to_string();
            self.log(line.trim_end_matches('\r'));
            pending.drain(..index + 1);
        }
        Ok(data.len())
    }

    pub fn flush(&self) { ::tsox_core::fntrace::enter("flush"); 
        let mut pending = self.mu.lock().unwrap();
        if !pending.is_empty() {
            self.log(pending.trim_end_matches('\r'));
            pending.clear();
        }
    }

    fn log(&self, message: &str) { ::tsox_core::fntrace::enter("log"); 
        (self.logger.0)(&format!("[content mapper: {}] stderr: {}", self.mapper_name, message));
    }
}

pub struct LoggedProcess {
    pub process: Box<dyn ReadWriteCloser>,
    pub stderr: Arc<StderrLogger>,
}

impl LoggedProcess {
    pub fn close(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("close"); 
        let err = self.process.close();
        self.stderr.flush();
        err
    }
}

impl std::io::Read for LoggedProcess {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("read"); 
        self.process.read(buf)
    }
}

impl std::io::Write for LoggedProcess {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("write"); 
        self.process.write(data)
    }

    fn flush(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("flush"); 
        self.process.flush()
    }
}

impl ReadWriteCloser for LoggedProcess {
    fn close(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("close"); 
        LoggedProcess::close(self)
    }

    fn exit_code(&self) -> Option<i32> { ::tsox_core::fntrace::enter("exit_code"); 
        self.process.exit_code()
    }
}

pub struct CloseOnceReadWriteCloser {
    inner: Mutex<Box<dyn ReadWriteCloser>>,
    once: Once,
    err: Mutex<Option<std::io::Result<()>>>,
}

impl CloseOnceReadWriteCloser {
    pub fn new(inner: Box<dyn ReadWriteCloser>) -> Self { ::tsox_core::fntrace::enter("new"); 
        CloseOnceReadWriteCloser {
            inner: Mutex::new(inner),
            once: Once::new(),
            err: Mutex::new(None),
        }
    }

    pub fn close(&self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("close"); 
        self.once.call_once(|| {
            let result = self.inner.lock().unwrap().close();
            *self.err.lock().unwrap() = Some(result);
        });
        match self.err.lock().unwrap().as_ref() {
            Some(Ok(())) => Ok(()),
            Some(Err(err)) => Err(std::io::Error::new(err.kind(), err.to_string())),
            None => Ok(()),
        }
    }

    pub fn exit_code(&self) -> Option<i32> { ::tsox_core::fntrace::enter("exit_code"); 
        self.inner.lock().unwrap().exit_code()
    }

    pub(crate) fn lock_inner(&self) -> std::sync::MutexGuard<'_, Box<dyn ReadWriteCloser>> { ::tsox_core::fntrace::enter("lock_inner"); 
        self.inner.lock().unwrap()
    }
}

impl std::io::Read for CloseOnceReadWriteCloser {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("read"); 
        self.inner.lock().unwrap().read(buf)
    }
}

impl std::io::Write for CloseOnceReadWriteCloser {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("write"); 
        self.inner.lock().unwrap().write(data)
    }

    fn flush(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("flush"); 
        self.inner.lock().unwrap().flush()
    }
}

pub fn get_script_kind_from_file_name(file_name: &str) -> tsox_core::core::mig::m3j::ScriptKind { ::tsox_core::fntrace::enter("get_script_kind_from_file_name"); 
    tsox_core::core::mig::m3j::get_script_kind_from_file_name(file_name)
}

pub fn parse_source_file(
    parse_options: &tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions,
    text: String,
    _script_kind: tsox_core::core::mig::m3j::ScriptKind,
) -> tsox_frontend::ast::SourceFile { ::tsox_core::fntrace::enter("parse_source_file"); 
    let (mut file, _) = tsox_frontend::parser::Parser::parse_source_file_text_with_diagnostics(
        &parse_options.file_name,
        text,
    );
    tsox_frontend::ast::mig::m3e_2::set_external_module_indicator_with_options(
        &mut file,
        parse_options.external_module_indicator_options,
    );
    file
}

#[derive(Debug, Clone, Default)]
pub struct ContentMapperSourceFileInfo {
    pub content_mapper: String,
    pub transform_identity: String,
    pub parse_options: tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions,
    pub virtual_file_name: String,
    pub original_text: String,
    pub span_map: Arc<spanmap::SpanMap>,
    pub diagnostic_directives: Vec<MappedDiagnosticDirective>,
    pub supplemental_source_files: Vec<std::sync::Arc<tsox_frontend::ast::SourceFile>>,
    pub canonical_source_file: Option<std::sync::Arc<tsox_frontend::ast::SourceFile>>,
}

static CONTENT_MAPPER_SOURCE_FILE_INFO_REGISTRY: std::sync::LazyLock<
    Mutex<std::collections::HashMap<String, ContentMapperSourceFileInfo>>,
> = std::sync::LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

pub trait SetContentMapperInfo {
    fn set_content_mapper_info(&self, info: ContentMapperSourceFileInfo);
}

impl SetContentMapperInfo for tsox_frontend::ast::SourceFile {
    fn set_content_mapper_info(&self, info: ContentMapperSourceFileInfo) { ::tsox_core::fntrace::enter("set_content_mapper_info"); 
        CONTENT_MAPPER_SOURCE_FILE_INFO_REGISTRY
            .lock()
            .unwrap()
            .insert(self.file_name.clone(), info);
    }
}

pub fn content_mapper_source_file_info(
    file_name: &str,
) -> Option<ContentMapperSourceFileInfo> { ::tsox_core::fntrace::enter("content_mapper_source_file_info"); 
    CONTENT_MAPPER_SOURCE_FILE_INFO_REGISTRY
        .lock()
        .unwrap()
        .get(file_name)
        .cloned()
}

pub struct StderrLineWriter(pub Arc<StderrLogger>);

impl std::io::Write for StderrLineWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("write"); 
        self.0.write_line(data)
    }

    fn flush(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("flush"); 
        self.0.flush();
        Ok(())
    }
}

pub mod spanmap {
    use serde::{Deserialize, Serialize};
    use tsox_core::core::text::TextPos;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct Segment {
        pub virtual_start: TextPos,
        pub virtual_end: TextPos,
        pub original_start: TextPos,
        pub original_end: TextPos,
    }

    pub type Segments = Vec<Segment>;

    #[derive(Debug, Clone, Default)]
    pub struct SpanMap {
        segments: Vec<Segment>,
    }

    impl SpanMap {
        pub fn validate(
            &self,
            virtual_text: &str,
            original: &str,
        ) -> Option<Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("validate"); 
            let virtual_len = virtual_text.len() as TextPos;
            let original_len = original.len() as TextPos;
            let mut previous_virtual_end: TextPos = 0;
            for segment in &self.segments {
                if segment.virtual_start < previous_virtual_end
                    || segment.virtual_end < segment.virtual_start
                    || segment.virtual_end > virtual_len
                {
                    return Some(Box::new(std::io::Error::other(format!(
                        "content mapper position mappings overlap or are out of order near virtual offset {}",
                        segment.virtual_start
                    ))));
                }
                previous_virtual_end = segment.virtual_end;
                if segment.original_end < segment.original_start
                    || segment.original_end > original_len
                {
                    return Some(Box::new(std::io::Error::other(format!(
                        "content mapper position mapping points outside the original content at original offset {}",
                        segment.original_end
                    ))));
                }
            }
            None
        }
    }

    pub fn span_map_new(segments: Vec<Segment>) -> SpanMap { ::tsox_core::fntrace::enter("span_map_new"); 
        SpanMap { segments }
    }
}
