use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options::ResolutionMode;
use tsox_core::diagnostics::Category;
use tsox_core::tspath;
use tsox_core::tspath::ComparePathsOptions;
use tsox_core::tspath::Path;
use tsox_frontend::ast::mig::m3e::RepopulateDiagnosticInfo;
use tsox_frontend::ast::mig::m3e::RepopulateDiagnosticKind;

pub type BuildInfoFileId = i32;
pub type BuildInfoFileIdListId = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum UpToDateStatusType {
    ConfigFileNotFound = 0,
    BuildErrors = 1,
    UpstreamErrors = 2,
    UpToDate = 3,
    UpToDateWithUpstreamTypes = 4,
    UpToDateWithInputFileText = 5,
    InputFileMissing = 6,
    OutputMissing = 7,
    InputFileNewer = 8,
    OutOfDateBuildInfoWithPendingEmit = 9,
    OutOfDateBuildInfoWithErrors = 10,
    OutOfDateOptions = 11,
    OutOfDateRoots = 12,
    TsVersionOutputOfDate = 13,
    ForceBuild = 14,
    Solution = 15,
}

#[derive(Debug, Clone, Default)]
pub struct InputOutputName {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct FileAndTime {
    pub file: String,
    pub time: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct InputOutputFileAndTime {
    pub input: FileAndTime,
    pub output: FileAndTime,
    pub build_info: String,
}

#[derive(Debug, Clone, Default)]
pub struct UpstreamErrors {
    pub r#ref: String,
    pub ref_has_upstream_errors: bool,
}

#[derive(Debug, Clone, Default)]
pub enum UpToDateStatusData {
    #[default]
    None,
    InputOutputFileAndTime(InputOutputFileAndTime),
    InputOutputName(InputOutputName),
    UpstreamErrors(UpstreamErrors),
    String(String),
}

#[derive(Debug, Clone, Default)]
pub struct UpToDateStatus {
    pub kind: Option<UpToDateStatusType>,
    pub data: UpToDateStatusData,
}

impl UpToDateStatus {
    pub fn is_error(&self) -> bool { ::tsox_core::fntrace::enter("is_error"); 
        matches!(
            self.kind,
            Some(UpToDateStatusType::ConfigFileNotFound)
                | Some(UpToDateStatusType::BuildErrors)
                | Some(UpToDateStatusType::UpstreamErrors)
        )
    }

    pub fn is_pseudo_build(&self) -> bool { ::tsox_core::fntrace::enter("is_pseudo_build"); 
        matches!(
            self.kind,
            Some(UpToDateStatusType::UpToDateWithUpstreamTypes)
                | Some(UpToDateStatusType::UpToDateWithInputFileText)
        )
    }

    pub fn input_output_file_and_time(&self) -> Option<&InputOutputFileAndTime> { ::tsox_core::fntrace::enter("input_output_file_and_time"); 
        match &self.data {
            UpToDateStatusData::InputOutputFileAndTime(data) => Some(data),
            _ => None,
        }
    }

    pub fn input_output_name(&self) -> Option<&InputOutputName> { ::tsox_core::fntrace::enter("input_output_name"); 
        match &self.data {
            UpToDateStatusData::InputOutputName(data) => Some(data),
            _ => None,
        }
    }

    pub fn oldest_output_file_name(&self) -> String { ::tsox_core::fntrace::enter("oldest_output_file_name"); 
        if !self.is_pseudo_build() && self.kind != Some(UpToDateStatusType::UpToDate) {
            panic!("only valid for up to date status of pseudo-build or up to date");
        }

        if let Some(input_output_file_and_time) = self.input_output_file_and_time() {
            return input_output_file_and_time.output.file.clone();
        }
        if let Some(input_output_name) = self.input_output_name() {
            return input_output_name.output.clone();
        }
        match &self.data {
            UpToDateStatusData::String(data) => data.clone(),
            _ => panic!("only valid for up to date status of pseudo-build or up to date"),
        }
    }

    pub fn upstream_errors(&self) -> &UpstreamErrors { ::tsox_core::fntrace::enter("upstream_errors"); 
        match &self.data {
            UpToDateStatusData::UpstreamErrors(data) => data,
            _ => panic!("upToDateStatus data is not upstreamErrors"),
        }
    }
}

pub fn is_build_info_file_name_default_library(file_name: &str) -> bool { ::tsox_core::fntrace::enter("is_build_info_file_name_default_library"); 
    !tspath::path_is_relative(file_name) && !tspath::path_is_absolute(file_name)
}

pub fn content_mapper_identities(
    project: Option<&dyn tsox_compile::mig::m3l_cm_2::Project>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> { ::tsox_core::fntrace::enter("content_mapper_identities"); 
    let Some(project) = project else {
        return Ok(Vec::new());
    };
    project.identities()
}

pub fn get_normalized_paths<'a>(
    paths: &'a [String],
    build_info_directory: &'a str,
) -> impl Iterator<Item = String> + 'a { ::tsox_core::fntrace::enter("get_normalized_paths"); 
    paths
        .iter()
        .map(move |path| tspath::get_normalized_absolute_path(path, build_info_directory))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildInfoRoot {
    pub start: BuildInfoFileId,
    pub end: BuildInfoFileId,
    pub non_incremental: String,
}

impl Serialize for BuildInfoRoot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        if self.start != 0 {
            if self.end != 0 {
                return [self.start, self.end].serialize(serializer);
            }
            return self.start.serialize(serializer);
        }
        self.non_incremental.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoRoot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(start_and_end) = value.as_array().filter(|a| a.len() == 2) {
            return Ok(BuildInfoRoot {
                start: start_and_end[0].as_i64().unwrap_or(0) as BuildInfoFileId,
                end: start_and_end[1].as_i64().unwrap_or(0) as BuildInfoFileId,
                non_incremental: String::new(),
            });
        }
        if let Some(start) = value.as_i64() {
            return Ok(BuildInfoRoot {
                start: start as BuildInfoFileId,
                end: 0,
                non_incremental: String::new(),
            });
        }
        if let Some(name) = value.as_str() {
            return Ok(BuildInfoRoot {
                start: 0,
                end: 0,
                non_incremental: name.to_string(),
            });
        }
        Err(serde::de::Error::custom(format!(
            "invalid BuildInfoRoot: {value}"
        )))
    }
}

#[derive(Debug, Clone, Default)]
#[allow(non_snake_case)]
pub struct BuildInfoFileInfoNoSignature {
    pub version: String,
    pub no_signature: bool,
    pub affects_global_scope: bool,
    pub implied_node_format: ResolutionMode,
}

impl Serialize for BuildInfoFileInfoNoSignature {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        let mut map = serde_json::Map::new();
        if !self.version.is_empty() {
            map.insert("version".to_string(), serde_json::json!(self.version));
        }
        if self.no_signature {
            map.insert("noSignature".to_string(), serde_json::json!(true));
        }
        if self.affects_global_scope {
            map.insert("affectsGlobalScope".to_string(), serde_json::json!(true));
        }
        if self.implied_node_format != ResolutionMode::None {
            map.insert(
                "impliedNodeFormat".to_string(),
                serde_json::json!(self.implied_node_format as i32),
            );
        }
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoFileInfoNoSignature {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value.as_object().ok_or_else(|| {
            serde::de::Error::custom("invalid BuildInfoFileInfoNoSignature")
        })?;
        Ok(BuildInfoFileInfoNoSignature {
            version: obj
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            no_signature: obj
                .get("noSignature")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            affects_global_scope: obj
                .get("affectsGlobalScope")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            implied_node_format: obj
                .get("impliedNodeFormat")
                .map(resolution_mode_from_value)
                .unwrap_or(ResolutionMode::None),
        })
    }
}

#[derive(Debug, Clone, Default)]
#[allow(non_snake_case)]
pub struct BuildInfoFileInfoWithSignature {
    pub version: String,
    pub signature: String,
    pub affects_global_scope: bool,
    pub implied_node_format: ResolutionMode,
}

impl Serialize for BuildInfoFileInfoWithSignature {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        let mut map = serde_json::Map::new();
        if !self.version.is_empty() {
            map.insert("version".to_string(), serde_json::json!(self.version));
        }
        if !self.signature.is_empty() {
            map.insert("signature".to_string(), serde_json::json!(self.signature));
        }
        if self.affects_global_scope {
            map.insert("affectsGlobalScope".to_string(), serde_json::json!(true));
        }
        if self.implied_node_format != ResolutionMode::None {
            map.insert(
                "impliedNodeFormat".to_string(),
                serde_json::json!(self.implied_node_format as i32),
            );
        }
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoFileInfoWithSignature {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value.as_object().ok_or_else(|| {
            serde::de::Error::custom("invalid BuildInfoFileInfoWithSignature")
        })?;
        Ok(BuildInfoFileInfoWithSignature {
            version: obj
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            signature: obj
                .get("signature")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            affects_global_scope: obj
                .get("affectsGlobalScope")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            implied_node_format: obj
                .get("impliedNodeFormat")
                .map(resolution_mode_from_value)
                .unwrap_or(ResolutionMode::None),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BuildInfoFileInfo {
    signature: String,
    no_signature: Option<BuildInfoFileInfoNoSignature>,
    file_info: Option<BuildInfoFileInfoWithSignature>,
}

impl BuildInfoFileInfo {
    pub fn new(file_info: &crate::mig::m4z2_2::FileInfo) -> Self { ::tsox_core::fntrace::enter("new"); 
        if file_info.version == file_info.signature {
            if !file_info.affects_global_scope
                && file_info.implied_node_format == ResolutionMode::CommonJS
            {
                return BuildInfoFileInfo {
                    signature: file_info.signature.clone(),
                    no_signature: None,
                    file_info: None,
                };
            }
        } else if file_info.signature.is_empty() {
            return BuildInfoFileInfo {
                signature: String::new(),
                no_signature: Some(BuildInfoFileInfoNoSignature {
                    version: file_info.version.clone(),
                    no_signature: true,
                    affects_global_scope: file_info.affects_global_scope,
                    implied_node_format: file_info.implied_node_format,
                }),
                file_info: None,
            };
        }
        BuildInfoFileInfo {
            signature: String::new(),
            no_signature: None,
            file_info: Some(BuildInfoFileInfoWithSignature {
                version: file_info.version.clone(),
                signature: if file_info.signature == file_info.version {
                    String::new()
                } else {
                    file_info.signature.clone()
                },
                affects_global_scope: file_info.affects_global_scope,
                implied_node_format: file_info.implied_node_format,
            }),
        }
    }

    pub fn get_file_info(&self) -> Option<crate::mig::m4z2_2::FileInfo> { ::tsox_core::fntrace::enter("get_file_info"); 
        if let Some(signature) = Some(&self.signature).filter(|s| !s.is_empty()) {
            return Some(crate::mig::m4z2_2::FileInfo {
                version: signature.clone(),
                signature: signature.clone(),
                implied_node_format: ResolutionMode::CommonJS,
                ..Default::default()
            });
        }
        if let Some(no_signature) = &self.no_signature {
            return Some(crate::mig::m4z2_2::FileInfo {
                version: no_signature.version.clone(),
                signature: String::new(),
                affects_global_scope: no_signature.affects_global_scope,
                implied_node_format: no_signature.implied_node_format,
                ..Default::default()
            });
        }
        self.file_info.as_ref().map(|file_info| {
            crate::mig::m4z2_2::FileInfo {
                version: file_info.version.clone(),
                signature: if file_info.signature.is_empty() {
                    file_info.version.clone()
                } else {
                    file_info.signature.clone()
                },
                affects_global_scope: file_info.affects_global_scope,
                implied_node_format: file_info.implied_node_format,
                ..Default::default()
            }
        })
    }

    pub fn has_signature(&self) -> bool { ::tsox_core::fntrace::enter("has_signature"); 
        !self.signature.is_empty()
    }
}

impl Serialize for BuildInfoFileInfo {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        if !self.signature.is_empty() {
            return self.signature.serialize(serializer);
        }
        if let Some(no_signature) = &self.no_signature {
            return no_signature.serialize(serializer);
        }
        self.file_info.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoFileInfo {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(v_signature) = value.as_str() {
            return Ok(BuildInfoFileInfo {
                signature: v_signature.to_string(),
                no_signature: None,
                file_info: None,
            });
        }
        let no_signature: Result<BuildInfoFileInfoNoSignature, _> =
            serde_json::from_value(value.clone())
                .map_err(|e| <serde_json::Error as serde::de::Error>::custom(e.to_string()));
        if let Ok(no_signature) = no_signature
            && no_signature.no_signature
        {
            return Ok(BuildInfoFileInfo {
                signature: String::new(),
                no_signature: Some(no_signature),
                file_info: None,
            });
        }
        let file_info: BuildInfoFileInfoWithSignature =
            serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;
        Ok(BuildInfoFileInfo {
            signature: String::new(),
            no_signature: None,
            file_info: Some(file_info),
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildInfoReferenceMapEntry {
    pub file_id: BuildInfoFileId,
    pub file_id_list_id: BuildInfoFileIdListId,
}

impl Serialize for BuildInfoReferenceMapEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        (self.file_id, self.file_id_list_id).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoReferenceMapEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let (file_id, file_id_list_id) = <(BuildInfoFileId, BuildInfoFileIdListId)>::deserialize(
            deserializer,
        )?;
        Ok(BuildInfoReferenceMapEntry {
            file_id,
            file_id_list_id,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BuildInfoDiagnostic {
    pub file: BuildInfoFileId,
    pub no_file: bool,
    pub pos: i32,
    pub end: i32,
    pub code: i32,
    pub category: Category,
    pub source: String,
    pub message_text: String,
    pub message_key: String,
    pub message_args: Option<Vec<String>>,
    pub message_chain: Vec<Box<BuildInfoDiagnostic>>,
    pub related_information: Vec<Box<BuildInfoDiagnostic>>,
    pub reports_unnecessary: bool,
    pub reports_deprecated: bool,
    pub skipped_on_no_emit: bool,
    pub repopulate_info: Option<BuildInfoRepopulateInfo>,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct BuildInfoRepopulateInfo {
    pub kind: RepopulateDiagnosticKind,
    pub module_reference: String,
    pub mode: ResolutionMode,
    pub package_name: String,
}

impl Serialize for BuildInfoRepopulateInfo {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        let mut map = serde_json::Map::new();
        map.insert("kind".to_string(), serde_json::json!(self.kind as i32));
        if !self.module_reference.is_empty() {
            map.insert(
                "moduleReference".to_string(),
                serde_json::json!(self.module_reference),
            );
        }
        if self.mode != ResolutionMode::None {
            map.insert("mode".to_string(), serde_json::json!(self.mode as i32));
        }
        if !self.package_name.is_empty() {
            map.insert(
                "packageName".to_string(),
                serde_json::json!(self.package_name),
            );
        }
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoRepopulateInfo {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("invalid BuildInfoRepopulateInfo"))?;
        let kind = obj.get("kind").and_then(|v| v.as_i64()).unwrap_or(0);
        Ok(BuildInfoRepopulateInfo {
            kind: if kind == RepopulateDiagnosticKind::ModuleNotFound as i64 {
                RepopulateDiagnosticKind::ModuleNotFound
            } else {
                RepopulateDiagnosticKind::ModeMismatch
            },
            module_reference: obj
                .get("moduleReference")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            mode: obj
                .get("mode")
                .map(resolution_mode_from_value)
                .unwrap_or(ResolutionMode::None),
            package_name: obj
                .get("packageName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

impl Serialize for BuildInfoDiagnostic {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        #[allow(non_snake_case)]
        #[derive(Serialize)]
        struct Raw<'a> {
            #[serde(skip_serializing_if = "is_zero_i32")]
            file: &'a BuildInfoFileId,
            #[serde(rename = "noFile", skip_serializing_if = "bool_is_false")]
            no_file: &'a bool,
            #[serde(skip_serializing_if = "is_zero_i32")]
            pos: &'a i32,
            #[serde(skip_serializing_if = "is_zero_i32")]
            end: &'a i32,
            #[serde(skip_serializing_if = "is_zero_i32")]
            code: &'a i32,
            #[serde(skip_serializing_if = "is_zero_i32")]
            category: i32,
            #[serde(skip_serializing_if = "String::is_empty")]
            source: &'a String,
            #[serde(rename = "messageText", skip_serializing_if = "String::is_empty")]
            message_text: &'a String,
            #[serde(rename = "messageKey", skip_serializing_if = "String::is_empty")]
            message_key: &'a String,
            #[serde(rename = "messageArgs", skip_serializing_if = "Option::is_none")]
            message_args: &'a Option<Vec<String>>,
            #[serde(rename = "messageChain", skip_serializing_if = "Vec::is_empty")]
            message_chain: &'a Vec<Box<BuildInfoDiagnostic>>,
            #[serde(rename = "relatedInformation", skip_serializing_if = "Vec::is_empty")]
            related_information: &'a Vec<Box<BuildInfoDiagnostic>>,
            #[serde(
                rename = "reportsUnnecessary",
                skip_serializing_if = "bool_is_false"
            )]
            reports_unnecessary: &'a bool,
            #[serde(rename = "reportsDeprecated", skip_serializing_if = "bool_is_false")]
            reports_deprecated: &'a bool,
            #[serde(rename = "skippedOnNoEmit", skip_serializing_if = "bool_is_false")]
            skipped_on_no_emit: &'a bool,
            #[serde(rename = "repopulateInfo", skip_serializing_if = "Option::is_none")]
            repopulate_info: &'a Option<BuildInfoRepopulateInfo>,
        }
        Raw {
            file: &self.file,
            no_file: &self.no_file,
            pos: &self.pos,
            end: &self.end,
            code: &self.code,
            category: self.category as i32,
            source: &self.source,
            message_text: &self.message_text,
            message_key: &self.message_key,
            message_args: &self.message_args,
            message_chain: &self.message_chain,
            related_information: &self.related_information,
            reports_unnecessary: &self.reports_unnecessary,
            reports_deprecated: &self.reports_deprecated,
            skipped_on_no_emit: &self.skipped_on_no_emit,
            repopulate_info: &self.repopulate_info,
        }
        .serialize(serializer)
    }
}

fn is_zero_i32(value: &i32) -> bool { ::tsox_core::fntrace::enter("is_zero_i32"); 
    *value == 0
}
fn bool_is_false(value: &&bool) -> bool { ::tsox_core::fntrace::enter("bool_is_false"); 
    !**value
}

fn resolution_mode_from_value(value: &serde_json::Value) -> ResolutionMode { ::tsox_core::fntrace::enter("resolution_mode_from_value"); 
    match value.as_i64() {
        Some(1) => ResolutionMode::CommonJS,
        Some(2) => ResolutionMode::AMD,
        Some(3) => ResolutionMode::UMD,
        Some(4) => ResolutionMode::System,
        Some(5) => ResolutionMode::ES2015,
        Some(6) => ResolutionMode::ES2020,
        Some(7) => ResolutionMode::ES2022,
        Some(99) => ResolutionMode::ESNext,
        Some(100) => ResolutionMode::Node16,
        Some(101) => ResolutionMode::Node18,
        Some(102) => ResolutionMode::Node20,
        Some(199) => ResolutionMode::NodeNext,
        Some(200) => ResolutionMode::Preserve,
        _ => ResolutionMode::None,
    }
}

fn category_from_i32(value: i32) -> Category { ::tsox_core::fntrace::enter("category_from_i32"); 
    match value {
        1 => Category::Error,
        2 => Category::Suggestion,
        3 => Category::Message,
        _ => Category::Warning,
    }
}

impl<'de> Deserialize<'de> for BuildInfoDiagnostic {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        #[allow(non_snake_case)]
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            file: BuildInfoFileId,
            noFile: bool,
            pos: i32,
            end: i32,
            code: i32,
            category: i32,
            source: String,
            messageText: String,
            messageKey: String,
            messageArgs: Option<Vec<String>>,
            messageChain: Vec<Box<BuildInfoDiagnostic>>,
            relatedInformation: Vec<Box<BuildInfoDiagnostic>>,
            reportsUnnecessary: bool,
            reportsDeprecated: bool,
            skippedOnNoEmit: bool,
            repopulateInfo: Option<BuildInfoRepopulateInfo>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(BuildInfoDiagnostic {
            file: raw.file,
            no_file: raw.noFile,
            pos: raw.pos,
            end: raw.end,
            code: raw.code,
            category: category_from_i32(raw.category),
            source: raw.source,
            message_text: raw.messageText,
            message_key: raw.messageKey,
            message_args: raw.messageArgs,
            message_chain: raw.messageChain,
            related_information: raw.relatedInformation,
            reports_unnecessary: raw.reportsUnnecessary,
            reports_deprecated: raw.reportsDeprecated,
            skipped_on_no_emit: raw.skippedOnNoEmit,
            repopulate_info: raw.repopulateInfo,
        })
    }
}

impl From<&BuildInfoRepopulateInfo> for RepopulateDiagnosticInfo {
    fn from(info: &BuildInfoRepopulateInfo) -> Self { ::tsox_core::fntrace::enter("from"); 
        RepopulateDiagnosticInfo {
            kind: match info.kind {
                RepopulateDiagnosticKind::ModeMismatch => RepopulateDiagnosticKind::ModeMismatch,
                RepopulateDiagnosticKind::ModuleNotFound => {
                    RepopulateDiagnosticKind::ModuleNotFound
                }
            },
            module_reference: info.module_reference.clone(),
            mode: info.mode,
            package_name: info.package_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BuildInfoDiagnosticsOfFile {
    pub file_id: BuildInfoFileId,
    pub diagnostics: Vec<Box<BuildInfoDiagnostic>>,
}

impl Serialize for BuildInfoDiagnosticsOfFile {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        (&self.file_id, &self.diagnostics).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoDiagnosticsOfFile {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let file_id_and_diagnostics =
            Vec::<serde_json::Value>::deserialize(deserializer).map_err(|_| {
                serde::de::Error::custom("invalid BuildInfoDiagnosticsOfFile")
            })?;
        if file_id_and_diagnostics.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "invalid BuildInfoDiagnosticsOfFile: expected 2 elements, got {}",
                file_id_and_diagnostics.len()
            )));
        }
        let file_id: BuildInfoFileId = serde_json::from_value(file_id_and_diagnostics[0].clone())
            .map_err(serde::de::Error::custom)?;
        let diagnostics: Vec<Box<BuildInfoDiagnostic>> =
            serde_json::from_value(file_id_and_diagnostics[1].clone())
                .map_err(serde::de::Error::custom)?;
        Ok(BuildInfoDiagnosticsOfFile {
            file_id,
            diagnostics,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BuildInfoSemanticDiagnostic {
    pub file_id: BuildInfoFileId,
    pub diagnostics: Option<BuildInfoDiagnosticsOfFile>,
}

impl Serialize for BuildInfoSemanticDiagnostic {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        if self.file_id != 0 {
            return self.file_id.serialize(serializer);
        }
        self.diagnostics.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoSemanticDiagnostic {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(file_id) = value.as_i64() {
            return Ok(BuildInfoSemanticDiagnostic {
                file_id: file_id as BuildInfoFileId,
                diagnostics: None,
            });
        }
        let diagnostics: BuildInfoDiagnosticsOfFile =
            serde_json::from_value(value).map_err(|_| {
                serde::de::Error::custom("invalid BuildInfoSemanticDiagnostic")
            })?;
        Ok(BuildInfoSemanticDiagnostic {
            file_id: 0,
            diagnostics: Some(diagnostics),
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildInfoFilePendingEmit {
    pub file_id: BuildInfoFileId,
    pub emit_kind: crate::mig::m4z2_2::FileEmitKind,
}

impl Serialize for BuildInfoFilePendingEmit {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        if self.emit_kind == crate::mig::m4z2_2::FileEmitKind::None {
            return self.file_id.serialize(serializer);
        }
        if self.emit_kind == crate::mig::m4z2_2::FileEmitKind::Dts {
            return [self.file_id].serialize(serializer);
        }
        [self.file_id, self.emit_kind.0 as BuildInfoFileId].serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoFilePendingEmit {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(file_id) = value.as_i64() {
            return Ok(BuildInfoFilePendingEmit {
                file_id: file_id as BuildInfoFileId,
                emit_kind: crate::mig::m4z2_2::FileEmitKind::None,
            });
        }
        let int_tuple = value
            .as_array()
            .ok_or_else(|| serde::de::Error::custom("invalid BuildInfoFilePendingEmit"))?;
        match int_tuple.len() {
            1 => Ok(BuildInfoFilePendingEmit {
                file_id: int_tuple[0].as_i64().unwrap_or(0) as BuildInfoFileId,
                emit_kind: crate::mig::m4z2_2::FileEmitKind::Dts,
            }),
            2 => Ok(BuildInfoFilePendingEmit {
                file_id: int_tuple[0].as_i64().unwrap_or(0) as BuildInfoFileId,
                emit_kind: crate::mig::m4z2_2::FileEmitKind(
                    int_tuple[1].as_i64().unwrap_or(0) as u16,
                ),
            }),
            len => Err(serde::de::Error::custom(format!(
                "invalid BuildInfoFilePendingEmit: expected 1 or 2 integers, got {len}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildInfoEmitSignature {
    pub file_id: BuildInfoFileId,
    pub signature: String,
    pub differs_only_in_dts_map: bool,
    pub differs_in_options: bool,
}

impl BuildInfoEmitSignature {
    pub fn no_emit_signature(&self) -> bool { ::tsox_core::fntrace::enter("no_emit_signature"); 
        self.signature.is_empty() && !self.differs_only_in_dts_map && !self.differs_in_options
    }

    pub fn to_emit_signature(
        &self,
        path: &Path,
        emit_signatures: &SyncMap<Path, crate::mig::m4z2_2::EmitSignature>,
    ) -> crate::mig::m4z2_2::EmitSignature { ::tsox_core::fntrace::enter("to_emit_signature"); 
        let mut signature = String::new();
        let mut signature_with_different_options: Vec<String> = Vec::new();
        if self.differs_only_in_dts_map {
            signature_with_different_options = Vec::with_capacity(1);
            if let Some(info) = emit_signatures.load(path) {
                signature_with_different_options.push(info.signature.clone());
            }
        } else if self.differs_in_options {
            signature_with_different_options = vec![self.signature.clone()];
        } else {
            signature = self.signature.clone();
        }
        crate::mig::m4z2_2::EmitSignature {
            signature,
            signature_with_different_options,
        }
    }
}

impl Serialize for BuildInfoEmitSignature {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        if self.no_emit_signature() {
            return self.file_id.serialize(serializer);
        }
        let signature: serde_json::Value = if self.differs_only_in_dts_map {
            serde_json::Value::Array(Vec::new())
        } else if self.differs_in_options {
            serde_json::json!([self.signature])
        } else {
            serde_json::json!(self.signature)
        };
        (self.file_id, signature).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoEmitSignature {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(file_id) = value.as_i64() {
            return Ok(BuildInfoEmitSignature {
                file_id: file_id as BuildInfoFileId,
                ..Default::default()
            });
        }
        let Some(file_id_and_signature) = value.as_array() else {
            return Err(serde::de::Error::custom("invalid BuildInfoEmitSignature"));
        };
        if file_id_and_signature.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "invalid BuildInfoEmitSignature: expected 2 elements, got {}",
                file_id_and_signature.len()
            )));
        }
        let Some(file_id) = file_id_and_signature[0].as_i64() else {
            return Err(serde::de::Error::custom(
                "invalid fileId in BuildInfoEmitSignature: expected number",
            ));
        };
        let mut signature = String::new();
        let mut differs_only_in_dts_map = false;
        let mut differs_in_options = false;
        match file_id_and_signature[1].as_str() {
            Some(signature_v) => signature = signature_v.to_string(),
            None => {
                let Some(signature_list) = file_id_and_signature[1].as_array() else {
                    return Err(serde::de::Error::custom(
                        "invalid signature in BuildInfoEmitSignature: expected string or []string",
                    ));
                };
                match signature_list.len() {
                    0 => differs_only_in_dts_map = true,
                    1 => {
                        let Some(sig) = signature_list[0].as_str() else {
                            return Err(serde::de::Error::custom(
                                "invalid signature in BuildInfoEmitSignature: expected string",
                            ));
                        };
                        signature = sig.to_string();
                        differs_in_options = true;
                    }
                    len => {
                        return Err(serde::de::Error::custom(format!(
                            "invalid signature in BuildInfoEmitSignature: expected string or []string with 0 or 1 element, got {len} elements"
                        )));
                    }
                }
            }
        }
        Ok(BuildInfoEmitSignature {
            file_id: file_id as BuildInfoFileId,
            signature,
            differs_only_in_dts_map,
            differs_in_options,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildInfoResolvedRoot {
    pub resolved: BuildInfoFileId,
    pub root: BuildInfoFileId,
}

impl Serialize for BuildInfoResolvedRoot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        [self.resolved, self.root].serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BuildInfoResolvedRoot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let (resolved, root) = <(BuildInfoFileId, BuildInfoFileId)>::deserialize(deserializer)
            .map_err(|_| serde::de::Error::custom("invalid BuildInfoResolvedRoot"))?;
        Ok(BuildInfoResolvedRoot { resolved, root })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BuildInfo {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,

    #[serde(rename = "errors", default, skip_serializing_if = "std::ops::Not::not")]
    pub errors: bool,
    #[serde(
        rename = "checkPending",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub check_pending: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root: Vec<BuildInfoRoot>,
    #[serde(
        rename = "packageJsons",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub package_jsons: Option<Vec<String>>,
    #[serde(
        rename = "missingPackageJsons",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub missing_package_jsons: Option<Vec<String>>,
    #[serde(
        rename = "contentMapperIdentities",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub content_mapper_identities: Vec<String>,

    #[serde(rename = "fileNames", default, skip_serializing_if = "Vec::is_empty")]
    pub file_names: Vec<String>,
    #[serde(rename = "fileInfos", default, skip_serializing_if = "Vec::is_empty")]
    pub file_infos: Vec<BuildInfoFileInfo>,
    #[serde(
        rename = "fileIdsList",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub file_ids_list: Vec<Vec<BuildInfoFileId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<OrderedMap<String, serde_json::Value>>,
    #[serde(
        rename = "referencedMap",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub referenced_map: Vec<BuildInfoReferenceMapEntry>,
    #[serde(
        rename = "semanticDiagnosticsPerFile",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub semantic_diagnostics_per_file: Vec<BuildInfoSemanticDiagnostic>,
    #[serde(
        rename = "emitDiagnosticsPerFile",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub emit_diagnostics_per_file: Vec<BuildInfoDiagnosticsOfFile>,
    #[serde(
        rename = "changeFileSet",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub change_file_set: Vec<BuildInfoFileId>,
    #[serde(
        rename = "affectedFilesPendingEmit",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub affected_files_pending_emit: Vec<BuildInfoFilePendingEmit>,
    #[serde(
        rename = "latestChangedDtsFile",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub latest_changed_dts_file: String,
    #[serde(
        rename = "emitSignatures",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub emit_signatures: Vec<BuildInfoEmitSignature>,
    #[serde(rename = "resolvedRoot", default, skip_serializing_if = "Vec::is_empty")]
    pub resolved_root: Vec<BuildInfoResolvedRoot>,

    #[serde(
        rename = "semanticErrors",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub semantic_errors: bool,
}

impl fmt::Display for BuildInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
        write!(f, "{}", self.0)
    }
}

pub struct BuildInfoError(pub String);

impl BuildInfo {
    pub fn is_valid_version(&self) -> bool { ::tsox_core::fntrace::enter("is_valid_version"); 
        self.version == tsox_core::core::mig::m3k::version()
    }

    pub fn content_mapper_identities_match(&self, current: &[String]) -> bool { ::tsox_core::fntrace::enter("content_mapper_identities_match"); 
        self.content_mapper_identities == current
    }

    pub fn is_incremental(&self) -> bool { ::tsox_core::fntrace::enter("is_incremental"); 
        !self.file_names.is_empty()
    }

    pub fn file_name(&self, file_id: BuildInfoFileId) -> String { ::tsox_core::fntrace::enter("file_name"); 
        if file_id < 1 || file_id as usize > self.file_names.len() {
            return String::new();
        }
        self.file_names[(file_id - 1) as usize].clone()
    }

    pub fn file_info(&self, file_id: BuildInfoFileId) -> Option<&BuildInfoFileInfo> { ::tsox_core::fntrace::enter("file_info"); 
        if file_id < 1 || file_id as usize > self.file_infos.len() {
            return None;
        }
        self.file_infos.get((file_id - 1) as usize)
    }

    pub fn get_compiler_options(&self, build_info_directory: &str) -> CompilerOptions { ::tsox_core::fntrace::enter("get_compiler_options"); 
        let mut options = CompilerOptions::default();
        let Some(entries) = self.options.as_ref() else {
            return options;
        };
        for (option, value) in entries.iter() {
            if !build_info_directory.is_empty() {
                if let Some(result) = tsox_tsoptions::mig::m5i_2::convert_option_to_absolute_path(
                    option,
                    value,
                    tsox_tsoptions::mig::m5i_2::command_line_compiler_options_map(),
                    build_info_directory,
                ) {
                    tsox_tsoptions::mig::m5i_2::parse_compiler_options_public(
                        option,
                        &result,
                        &mut options,
                    );
                    continue;
                }
            }
            tsox_tsoptions::mig::m5i_2::parse_compiler_options_public(option, value, &mut options);
        }
        options
    }

    pub fn is_emit_pending(
        &self,
        resolved: &tsox_tsoptions::tsoptions::ParsedCommandLine,
        build_info_directory: &str,
    ) -> bool { ::tsox_core::fntrace::enter("is_emit_pending"); 
        let compiler_options = resolved.compiler_options();
        if !compiler_options.no_emit.is_true() || compiler_options.get_emit_declarations() {
            let mut pending_emit =
                crate::mig::m4z2_2::get_pending_emit_kind_with_options(
                    compiler_options,
                    &self.get_compiler_options(build_info_directory),
                );
            if compiler_options.no_emit.is_true() {
                pending_emit &= crate::mig::m4z2_2::FileEmitKind::DtsErrors;
            }
            return pending_emit != crate::mig::m4z2_2::FileEmitKind::None;
        }
        false
    }

    pub fn get_package_jsons(
        &self,
        build_info_directory: &str,
    ) -> impl Iterator<Item = String> + '_ { ::tsox_core::fntrace::enter("get_package_jsons"); 
        let dir = build_info_directory.to_string();
        self.package_jsons
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(move |path| tspath::get_normalized_absolute_path(path, &dir))
    }

    pub fn get_missing_package_jsons(
        &self,
        build_info_directory: &str,
    ) -> impl Iterator<Item = String> + '_ { ::tsox_core::fntrace::enter("get_missing_package_jsons"); 
        let dir = build_info_directory.to_string();
        self.missing_package_jsons
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(move |path| tspath::get_normalized_absolute_path(path, &dir))
    }

    pub fn get_build_info_root_info_reader<'a>(
        &'a self,
        build_info_directory: &str,
        compare_path_options: &ComparePathsOptions,
    ) -> BuildInfoRootInfoReader<'a> { ::tsox_core::fntrace::enter("get_build_info_root_info_reader"); 
        let mut resolved_root_file_infos: HashMap<Path, &'a BuildInfoFileInfo> =
            HashMap::with_capacity(self.file_names.len());
        let mut root_to_resolved: OrderedMap<Path, Path> =
            OrderedMap::with_capacity(self.file_names.len());
        let mut resolved_to_root: HashMap<Path, Path> =
            HashMap::with_capacity(self.resolved_root.len());
        let to_path = |file_name: &str| {
            tspath::to_path(
                file_name,
                build_info_directory,
                compare_path_options.use_case_sensitive_file_names,
            )
        };

        for resolved in &self.resolved_root {
            let resolved_root = self.file_name(resolved.resolved);
            let root = self.file_name(resolved.root);
            if !resolved_root.is_empty() && !root.is_empty() {
                resolved_to_root.insert(to_path(&resolved_root), to_path(&root));
            }
        }

        fn add_root<'a>(
            resolved_root: &str,
            file_info: Option<&'a BuildInfoFileInfo>,
            to_path: &dyn Fn(&str) -> Path,
            resolved_to_root: &HashMap<Path, Path>,
            root_to_resolved: &mut OrderedMap<Path, Path>,
            resolved_root_file_infos: &mut HashMap<Path, &'a BuildInfoFileInfo>,
        ) { ::tsox_core::fntrace::enter("add_root"); 
            if resolved_root.is_empty() {
                return;
            }
            let resolved_root_path = to_path(resolved_root);
            if let Some(root_path) = resolved_to_root.get(&resolved_root_path) {
                root_to_resolved.set(root_path.clone(), resolved_root_path.clone());
            } else {
                root_to_resolved.set(resolved_root_path.clone(), resolved_root_path.clone());
            }
            if let Some(file_info) = file_info {
                resolved_root_file_infos.insert(resolved_root_path, file_info);
            }
        }

        for root in &self.root {
            if !root.non_incremental.is_empty() {
                add_root(
                    &root.non_incremental,
                    None,
                    &to_path,
                    &resolved_to_root,
                    &mut root_to_resolved,
                    &mut resolved_root_file_infos,
                );
            } else if root.end == 0 {
                add_root(
                    &self.file_name(root.start),
                    self.file_info(root.start),
                    &to_path,
                    &resolved_to_root,
                    &mut root_to_resolved,
                    &mut resolved_root_file_infos,
                );
            } else {
                for i in root.start..=root.end {
                    add_root(
                        &self.file_name(i),
                        self.file_info(i),
                        &to_path,
                        &resolved_to_root,
                        &mut root_to_resolved,
                        &mut resolved_root_file_infos,
                    );
                }
            }
        }

        BuildInfoRootInfoReader {
            resolved_root_file_infos,
            root_to_resolved,
        }
    }
}

pub struct BuildInfoRootInfoReader<'a> {
    resolved_root_file_infos: HashMap<Path, &'a BuildInfoFileInfo>,
    root_to_resolved: OrderedMap<Path, Path>,
}

impl<'a> BuildInfoRootInfoReader<'a> {
    pub fn get_build_info_file_info(
        &self,
        input_file_path: &Path,
    ) -> Option<(&'a BuildInfoFileInfo, Path)> { ::tsox_core::fntrace::enter("get_build_info_file_info"); 
        if let Some(info) = self.resolved_root_file_infos.get(input_file_path) {
            return Some((info, input_file_path.clone()));
        }
        if let Some(resolved) = self.root_to_resolved.get(input_file_path) {
            return self
                .resolved_root_file_infos
                .get(&resolved)
                .map(|info| (*info, resolved.clone()));
        }
        None
    }

    pub fn roots(&self) -> impl Iterator<Item = &Path> { ::tsox_core::fntrace::enter("roots"); 
        self.root_to_resolved.keys()
    }
}

#[allow(unused)]
fn unused_refs(
    _: &RepopulateDiagnosticKind,
    _: &BuildInfoError,
    _: &HashMap<String, String>,
    _: &tsox_tsoptions::tsoptions::BuildOptions,
) { ::tsox_core::fntrace::enter("unused_refs"); 
}
