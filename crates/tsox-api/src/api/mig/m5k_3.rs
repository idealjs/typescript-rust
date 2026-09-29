#![allow(unused_imports, dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::api::mig::m5k_4::{new_diagnostic_responses, DiagnosticResponse};

pub type ProjectId = String;
pub type SymbolId = u32;
pub type TypeId = u32;
pub type SignatureId = u32;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DocumentIdentifier {
    pub file_name: String,
    pub uri: String,
}

impl std::fmt::Display for DocumentIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_value())
    }
}

impl DocumentIdentifier {
    pub fn unmarshal_json_from(value: &JsonValue) -> Result<DocumentIdentifier, String> {
        match value {
            JsonValue::String(s) => Ok(DocumentIdentifier {
                file_name: s.clone(),
                uri: String::new(),
            }),
            JsonValue::Object(map) => {
                let mut d = DocumentIdentifier {
                    file_name: String::new(),
                    uri: String::new(),
                };
                if let Some(JsonValue::String(uri)) = map.get("uri") {
                    d.uri = uri.clone();
                }
                if let Some(JsonValue::String(file_name)) = map.get("fileName") {
                    d.file_name = file_name.clone();
                }
                Ok(d)
            }
            other => Err(format!(
                "DocumentIdentifier: expected string or object, got {}",
                other
            )),
        }
    }

    pub fn to_file_name(&self) -> String {
        if !self.uri.is_empty() {
            return tsox_lsp::mig::m5m::document_uri_file_name(
                &tsox_lsp::lsp::lsproto::DocumentUri(self.uri.clone()),
            );
        }
        self.file_name.clone()
    }

    pub fn to_uri(&self, cwd: &str) -> String {
        if !self.uri.is_empty() {
            return self.uri.clone();
        }
        let absolute = tsox_core::tspath::get_normalized_absolute_path(&self.file_name, cwd);
        tsox_lsp::ls::lsconv_converters::file_name_to_document_uri(&absolute)
    }

    pub fn to_absolute_file_name(&self, cwd: &str) -> String {
        if !self.uri.is_empty() {
            return tsox_lsp::mig::m5m::document_uri_file_name(
                &tsox_lsp::lsp::lsproto::DocumentUri(self.uri.clone()),
            );
        }
        tsox_core::tspath::get_normalized_absolute_path(&self.file_name, cwd)
    }

    pub fn to_string_value(&self) -> String {
        if !self.uri.is_empty() {
            return self.uri.clone();
        }
        self.file_name.clone()
    }
}

pub fn project_handle(p: &tsox_lsp::project::project::Project) -> ProjectId {
    p.id().0.clone()
}

pub fn symbol_handle(symbol: &Arc<tsox_frontend::ast::Symbol>) -> SymbolId {
    tsox_frontend::ast::mig::m3f::get_symbol_id(symbol) as u32
}

pub fn type_handle(t: &tsox_checker::checker::Type) -> TypeId {
    t.id()
}

pub fn signature_handle(sig: &tsox_checker::checker::Signature) -> SignatureId {
    sig.id()
}

pub fn parse_project_handle(handle: &ProjectId) -> tsox_core::tspath::Path {
    tsox_core::tspath::Path(handle.clone())
}

pub fn new_config_file_response(
    parsed_command_line: Option<&tsox_tsoptions::tsoptions::ParsedCommandLine>,
) -> Option<ConfigFileResponse> {
    let parsed_command_line = parsed_command_line?;
    let mut compile_on_save = parsed_command_line.compile_on_save.clone();
    if compile_on_save.is_none() {
        if let Some(raw) = &parsed_command_line.raw_options {
            if let Some(JsonValue::Bool(value)) = raw.get("compileOnSave") {
                compile_on_save = Some(*value);
            }
        }
    }
    let mut errors = new_diagnostic_responses(&parsed_command_line.errors);
    if errors.is_none() {
        errors = Some(Vec::new());
    }
    Some(ConfigFileResponse {
        file_names: parsed_command_line.file_names().to_vec(),
        options: parsed_command_line.compiler_options().clone(),
        project_references: parsed_command_line.project_references().to_vec(),
        type_acquisition: parsed_command_line.type_acquisition().cloned(),
        compile_on_save,
        raw: to_protocol_json_value(
            parsed_command_line
                .raw_options
                .clone()
                .unwrap_or(JsonValue::Null),
        ),
        errors: errors.unwrap_or_default(),
    })
}

#[derive(serde::Serialize)]
pub struct ConfigFileResponse {
    pub file_names: Vec<String>,
    pub options: tsox_core::core::compiler_options::CompilerOptions,
    pub project_references: Vec<tsox_core::core::project_reference::ProjectReference>,
    pub type_acquisition: Option<tsox_core::core::mig::m3k::TypeAcquisition>,
    pub compile_on_save: Option<bool>,
    pub raw: JsonValue,
    pub errors: Vec<DiagnosticResponse>,
}

pub fn to_protocol_json_value(value: JsonValue) -> JsonValue {
    match value {
        JsonValue::Array(items) => JsonValue::Array(
            items
                .into_iter()
                .map(to_protocol_json_value)
                .collect(),
        ),
        JsonValue::Object(map) => JsonValue::Object(
            map.into_iter()
                .map(|(key, child)| (key, to_protocol_json_value(child)))
                .collect(),
        ),
        JsonValue::Number(n) if is_watch_or_polling_enum_value(&n) => {
            JsonValue::Number(serde_json::Number::from(n.as_i64().unwrap_or(0) - 1))
        }
        other => other,
    }
}

fn is_watch_or_polling_enum_value(n: &serde_json::Number) -> bool {
    false
}

pub fn new_project_response(p: Option<&tsox_lsp::project::project::Project>) -> ProjectResponse {
    let p = p.expect("NewProjectResponse called with unloaded project");
    let command_line = p
        .command_line
        .as_ref()
        .expect("NewProjectResponse called with unloaded project");
    ProjectResponse {
        id: project_handle(p),
        config_file_name: p.name().to_string(),
        current_directory: p.current_directory.clone(),
        parsed_command_line: new_config_file_response(Some(command_line)),
        root_files: command_line.file_names().to_vec(),
        compiler_options: command_line.compiler_options().clone(),
    }
}

#[derive(serde::Serialize)]
pub struct ProjectResponse {
    pub id: ProjectId,
    pub config_file_name: String,
    pub current_directory: String,
    pub parsed_command_line: Option<ConfigFileResponse>,
    pub root_files: Vec<String>,
    pub compiler_options: tsox_core::core::compiler_options::CompilerOptions,
}

pub fn symbol_handles(symbols: &[Arc<tsox_frontend::ast::Symbol>]) -> Option<Vec<SymbolId>> {
    if symbols.is_empty() {
        return None;
    }
    Some(symbols.iter().map(symbol_handle).collect())
}

pub fn type_handles(types: &[tsox_checker::checker::Type]) -> Option<Vec<TypeId>> {
    if types.is_empty() {
        return None;
    }
    Some(types.iter().map(type_handle).collect())
}

pub fn literal_value_to_json(value: LiteralValue) -> JsonValue {
    match value {
        LiteralValue::String(v) => JsonValue::String(v),
        LiteralValue::Number(v) => JsonValue::from(v),
        LiteralValue::Boolean(v) => JsonValue::Bool(v),
        LiteralValue::PseudoBigInt(v) => JsonValue::String(v.to_string()),
    }
}

pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    PseudoBigInt(tsox_core::jsnum::PseudoBigInt),
}

pub fn json_value_to_any(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Null => JsonValue::Null,
        JsonValue::String(s) => JsonValue::String(s.clone()),
        JsonValue::Number(n) => JsonValue::Number(n.clone()),
        JsonValue::Bool(b) => JsonValue::Bool(*b),
        JsonValue::Array(items) => {
            JsonValue::Array(items.iter().map(json_value_to_any).collect())
        }
        JsonValue::Object(map) => JsonValue::Object(
            map.iter()
                .map(|(key, child)| (key.clone(), json_value_to_any(child)))
                .collect(),
        ),
    }
}

pub fn unmarshal_payload(method: &str, payload: &JsonValue) -> Result<JsonValue, String> {
    let unmarshaler = unmarshalers(method)
        .ok_or_else(|| format!("unknown API method {:?}", method))?;
    unmarshaler(payload)
}

fn unmarshalers(method: &str) -> Option<fn(&JsonValue) -> Result<JsonValue, String>> {
    let _ = method;
    None
}

pub fn unmarshaller_for<T: serde::de::DeserializeOwned + serde::Serialize>(
    data: &JsonValue,
) -> Result<JsonValue, String> {
    serde_json::from_value::<T>(data.clone())
        .map(|v| serde_json::to_value(v).unwrap_or(JsonValue::Null))
        .map_err(|e| {
            format!(
                "failed to unmarshal {}: {}",
                std::any::type_name::<T>(),
                e
            )
        })
}

pub fn no_params(_data: &JsonValue) -> Result<JsonValue, String> {
    Ok(JsonValue::Null)
}
