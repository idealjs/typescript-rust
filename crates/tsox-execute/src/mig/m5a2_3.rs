#![allow(dead_code, unused_imports, unused_variables)]

use serde::{Deserialize, Serialize};

use super::m4y_2::{
    BuildInfo, BuildInfoDiagnostic, BuildInfoDiagnosticsOfFile, BuildInfoFileId,
    BuildInfoFileIdListId, BuildInfoRepopulateInfo,
};
use super::m4z2_2::FileEmitKind;
use super::m5a2_2::{
    ReadableBuildInfo, ReadableBuildInfoDiagnostic, ReadableBuildInfoDiagnosticsOfFile,
    ReadableBuildInfoFileInfo, ReadableBuildInfoRepopulateInfo, ReadableBuildInfoRoot,
};

pub struct ReadableBuildInfoSemanticDiagnostic {
    pub file: String,
    pub diagnostics: Option<ReadableBuildInfoDiagnosticsOfFile>,
}

impl Serialize for ReadableBuildInfoSemanticDiagnostic {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        if !self.file.is_empty() {
            return self.file.serialize(serializer);
        }
        self.diagnostics.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReadableBuildInfoSemanticDiagnostic {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Ok(file) = serde_json::from_value::<String>(value.clone()) {
            return Ok(ReadableBuildInfoSemanticDiagnostic {
                file,
                diagnostics: None,
            });
        }
        let diagnostics: ReadableBuildInfoDiagnosticsOfFile =
            serde_json::from_value(value).map_err(|_| {
                serde::de::Error::custom("invalid readableBuildInfoSemanticDiagnostic")
            })?;
        Ok(ReadableBuildInfoSemanticDiagnostic {
            file: String::new(),
            diagnostics: Some(diagnostics),
        })
    }
}

pub struct ReadableBuildInfoFilePendingEmit {
    pub file: String,
    pub emit_kind: String,
    pub original: Option<super::m4y_2::BuildInfoFilePendingEmit>,
}

impl Serialize for ReadableBuildInfoFilePendingEmit {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        (
            self.file.clone(),
            self.emit_kind.clone(),
            self.original.clone(),
        )
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReadableBuildInfoFilePendingEmit {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let value = serde_json::Value::deserialize(deserializer).map_err(|_| {
            serde::de::Error::custom("invalid readableBuildInfoFilePendingEmit")
        })?;
        let items = value.as_array().ok_or_else(|| {
            serde::de::Error::custom("invalid readableBuildInfoFilePendingEmit")
        })?;
        if items.len() != 3 {
            return Err(serde::de::Error::custom(format!(
                "invalid readableBuildInfoFilePendingEmit: expected 3 elements, got {}",
                items.len()
            )));
        }
        let file: String = serde_json::from_value(items[0].clone()).map_err(|_| {
            serde::de::Error::custom(
                "invalid fileId in readableBuildInfoFilePendingEmit: expected string",
            )
        })?;
        let emit_kind: String = serde_json::from_value(items[1].clone()).map_err(|_| {
            serde::de::Error::custom(
                "invalid emitKind in readableBuildInfoFilePendingEmit: expected string",
            )
        })?;
        let original: Option<super::m4y_2::BuildInfoFilePendingEmit> =
            serde_json::from_value(items[2].clone()).map_err(|_| {
                serde::de::Error::custom(
                    "invalid original in readableBuildInfoFilePendingEmit: expected *incremental.BuildInfoFilePendingEmit",
                )
            })?;
        Ok(ReadableBuildInfoFilePendingEmit {
            file,
            emit_kind,
            original,
        })
    }
}

#[derive(Serialize)]
pub struct ReadableBuildInfoEmitSignature {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub signature: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub differs_only_in_dts_map: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub differs_in_options: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original: Option<super::m4y_2::BuildInfoEmitSignature>,
}

pub struct ReadableBuildInfoResolvedRoot {
    pub resolved: String,
    pub root: String,
}

impl Serialize for ReadableBuildInfoResolvedRoot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        (self.resolved.clone(), self.root.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReadableBuildInfoResolvedRoot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        let resolved_and_root = <(String, String)>::deserialize(deserializer)
            .map_err(|_| serde::de::Error::custom("invalid BuildInfoResolvedRoot"))?;
        Ok(ReadableBuildInfoResolvedRoot {
            resolved: resolved_and_root.0,
            root: resolved_and_root.1,
        })
    }
}

pub fn to_readable_build_info_repopulate_info(
    info: Option<&BuildInfoRepopulateInfo>,
) -> Option<ReadableBuildInfoRepopulateInfo> { ::tsox_core::fntrace::enter("to_readable_build_info_repopulate_info"); 
    let info = info?;
    Some(ReadableBuildInfoRepopulateInfo {
        kind: info.kind,
        module_reference: info.module_reference.clone(),
        mode: info.mode,
        package_name: info.package_name.clone(),
    })
}

pub fn to_readable_file_emit_kind(file_emit_kind: FileEmitKind) -> String { ::tsox_core::fntrace::enter("to_readable_file_emit_kind"); 
    let mut builder = String::new();
    let mut add_flags = |flags: &str| {
        if builder.is_empty() {
            builder.push_str(flags);
        } else {
            builder.push('|');
            builder.push_str(flags);
        }
    };
    let bits = file_emit_kind.0;
    if bits != 0 {
        if bits & FileEmitKind::Js.0 != 0 {
            add_flags("Js");
        }
        if bits & FileEmitKind::JsMap.0 != 0 {
            add_flags("JsMap");
        }
        if bits & FileEmitKind::JsInlineMap.0 != 0 {
            add_flags("JsInlineMap");
        }
        if bits & FileEmitKind::Dts.0 == FileEmitKind::Dts.0 {
            add_flags("Dts");
        } else {
            if bits & FileEmitKind::DtsEmit.0 != 0 {
                add_flags("DtsEmit");
            }
            if bits & FileEmitKind::DtsErrors.0 != 0 {
                add_flags("DtsErrors");
            }
        }
        if bits & FileEmitKind::DtsMap.0 != 0 {
            add_flags("DtsMap");
        }
    }
    if !builder.is_empty() {
        return builder;
    }
    "None".to_string()
}

pub fn to_readable_build_info(build_info: &BuildInfo, build_info_text: &str) -> String { ::tsox_core::fntrace::enter("to_readable_build_info"); 
    let mut readable = ReadableBuildInfo {
        build_info,
        version: build_info.version.clone(),
        errors: build_info.errors,
        check_pending: build_info.check_pending,
        root: Vec::new(),
        package_jsons: build_info.package_jsons.clone().unwrap_or_default(),
        missing_package_jsons: build_info
            .missing_package_jsons
            .clone()
            .unwrap_or_default(),
        file_names: build_info.file_names.clone(),
        file_infos: Vec::new(),
        file_ids_list: Vec::new(),
        options: build_info.options.clone(),
        referenced_map: None,
        semantic_diagnostics_per_file: Vec::new(),
        emit_diagnostics_per_file: Vec::new(),
        change_file_set: Vec::new(),
        affected_files_pending_emit: Vec::new(),
        latest_changed_dts_file: build_info.latest_changed_dts_file.clone(),
        emit_signatures: Vec::new(),
        resolved_root: Vec::new(),
        size: build_info_text.len(),
        semantic_errors: build_info.semantic_errors,
    };
    readable.set_file_infos();
    readable.set_root();
    readable.set_file_ids_list();
    readable.set_referenced_map();
    readable.set_change_file_set();
    readable.set_semantic_diagnostics();
    readable.set_emit_diagnostics();
    readable.set_affected_files_pending_emit();
    readable.set_emit_signatures();
    readable.set_resolved_root();
    serde_json::to_string_pretty(&readable)
        .expect("readableBuildInfo: failed to marshal readable build info")
}
