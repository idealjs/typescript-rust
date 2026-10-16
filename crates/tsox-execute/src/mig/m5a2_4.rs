#![allow(dead_code, unused_imports, unused_variables)]

use super::m4y_2::{BuildInfo, BuildInfoDiagnostic, BuildInfoDiagnosticsOfFile, BuildInfoFileId, BuildInfoFileIdListId};
use super::m5a2_2::{ReadableBuildInfo, ReadableBuildInfoDiagnostic, ReadableBuildInfoDiagnosticsOfFile, ReadableBuildInfoFileInfo, ReadableBuildInfoRepopulateInfo, ReadableBuildInfoRoot};
use super::m5a2_3::{
    to_readable_build_info_repopulate_info, to_readable_file_emit_kind, ReadableBuildInfoEmitSignature,
    ReadableBuildInfoFilePendingEmit, ReadableBuildInfoResolvedRoot, ReadableBuildInfoSemanticDiagnostic,
};

pub fn to_readable_build_info(build_info: &BuildInfo, build_info_text: &str) -> String { ::tsox_core::fntrace::enter("to_readable_build_info"); 
    let mut readable = ReadableBuildInfo {
        build_info,
        version: build_info.version.clone(),
        errors: build_info.errors,
        check_pending: build_info.check_pending,
        root: Vec::new(),
        package_jsons: build_info.package_jsons.clone().unwrap_or_default(),
        missing_package_jsons: build_info.missing_package_jsons.clone().unwrap_or_default(),
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

impl<'a> ReadableBuildInfo<'a> {
    pub fn to_file_path(&self, file_id: BuildInfoFileId) -> String { ::tsox_core::fntrace::enter("to_file_path"); 
        self.build_info.file_names[(file_id - 1) as usize].clone()
    }

    pub fn to_file_path_set(&self, file_id_list_id: BuildInfoFileIdListId) -> Vec<String> { ::tsox_core::fntrace::enter("to_file_path_set"); 
        self.file_ids_list[(file_id_list_id - 1) as usize].clone()
    }

    pub fn to_readable_build_info_diagnostic(
        &self,
        diagnostics: &[Box<BuildInfoDiagnostic>],
    ) -> Vec<Box<ReadableBuildInfoDiagnostic>> { ::tsox_core::fntrace::enter("to_readable_build_info_diagnostic"); 
        diagnostics
            .iter()
            .map(|d| {
                let file = if d.file != 0 {
                    self.to_file_path(d.file)
                } else {
                    String::new()
                };
                Box::new(ReadableBuildInfoDiagnostic {
                    file,
                    no_file: d.no_file,
                    pos: d.pos,
                    end: d.end,
                    code: d.code,
                    category: d.category,
                    message_key: d.message_key.clone(),
                    message_args: d.message_args.clone(),
                    message_chain: self.to_readable_build_info_diagnostic(&d.message_chain),
                    related_information: self
                        .to_readable_build_info_diagnostic(&d.related_information),
                    reports_unnecessary: d.reports_unnecessary,
                    reports_deprecated: d.reports_deprecated,
                    skipped_on_no_emit: d.skipped_on_no_emit,
                    repopulate_info: to_readable_build_info_repopulate_info(d.repopulate_info.as_ref()),
                })
            })
            .collect()
    }

    pub fn to_readable_build_info_diagnostics_of_file(
        &self,
        diagnostics: &BuildInfoDiagnosticsOfFile,
    ) -> ReadableBuildInfoDiagnosticsOfFile { ::tsox_core::fntrace::enter("to_readable_build_info_diagnostics_of_file"); 
        ReadableBuildInfoDiagnosticsOfFile {
            file: self.to_file_path(diagnostics.file_id),
            diagnostics: self.to_readable_build_info_diagnostic(&diagnostics.diagnostics),
        }
    }

    pub fn set_file_infos(&mut self) { ::tsox_core::fntrace::enter("set_file_infos"); 
        self.file_infos = self
            .build_info
            .file_infos
            .iter()
            .enumerate()
            .map(|(index, original)| {
                let file_info = original.get_file_info().unwrap_or_default();
                let original = if original.has_signature() {
                    None
                } else {
                    Some(original.clone())
                };
                ReadableBuildInfoFileInfo {
                    file_name: self.to_file_path((index + 1) as BuildInfoFileId),
                    version: file_info.version,
                    signature: file_info.signature,
                    affects_global_scope: file_info.affects_global_scope,
                    implied_node_format: file_info.implied_node_format.to_string(),
                    original,
                }
            })
            .collect();
    }

    pub fn set_root(&mut self) { ::tsox_core::fntrace::enter("set_root"); 
        self.root = self
            .build_info
            .root
            .iter()
            .map(|original| {
                let files: Vec<String> = if !original.non_incremental.is_empty() {
                    vec![original.non_incremental.clone()]
                } else if original.end == 0 {
                    vec![self.to_file_path(original.start)]
                } else {
                    (original.start..=original.end)
                        .map(|i| self.to_file_path(i))
                        .collect()
                };
                ReadableBuildInfoRoot {
                    files,
                    original: original.clone(),
                }
            })
            .collect();
    }

    pub fn set_file_ids_list(&mut self) { ::tsox_core::fntrace::enter("set_file_ids_list"); 
        self.file_ids_list = self
            .build_info
            .file_ids_list
            .iter()
            .map(|ids| ids.iter().map(|id| self.to_file_path(*id)).collect())
            .collect();
    }

    pub fn set_referenced_map(&mut self) { ::tsox_core::fntrace::enter("set_referenced_map"); 
        if !self.build_info.referenced_map.is_empty() {
            let mut referenced_map =
                tsox_core::collections::ordered_map::OrderedMap::new();
            for entry in &self.build_info.referenced_map {
                referenced_map.set(
                    self.to_file_path(entry.file_id),
                    self.to_file_path_set(entry.file_id_list_id),
                );
            }
            self.referenced_map = Some(referenced_map);
        }
    }

    pub fn set_change_file_set(&mut self) { ::tsox_core::fntrace::enter("set_change_file_set"); 
        self.change_file_set = self
            .build_info
            .change_file_set
            .iter()
            .map(|file_id| self.to_file_path(*file_id))
            .collect();
    }

    pub fn set_semantic_diagnostics(&mut self) { ::tsox_core::fntrace::enter("set_semantic_diagnostics"); 
        self.semantic_diagnostics_per_file = self
            .build_info
            .semantic_diagnostics_per_file
            .iter()
            .map(|diagnostics| {
                if diagnostics.file_id != 0 {
                    ReadableBuildInfoSemanticDiagnostic {
                        file: self.to_file_path(diagnostics.file_id),
                        diagnostics: None,
                    }
                } else {
                    ReadableBuildInfoSemanticDiagnostic {
                        file: String::new(),
                        diagnostics: Some(
                            self.to_readable_build_info_diagnostics_of_file(
                                diagnostics.diagnostics.as_ref().unwrap(),
                            ),
                        ),
                    }
                }
            })
            .collect();
    }

    pub fn set_emit_diagnostics(&mut self) { ::tsox_core::fntrace::enter("set_emit_diagnostics"); 
        self.emit_diagnostics_per_file = self
            .build_info
            .emit_diagnostics_per_file
            .iter()
            .map(|diagnostics| self.to_readable_build_info_diagnostics_of_file(diagnostics))
            .collect();
    }

    pub fn set_affected_files_pending_emit(&mut self) { ::tsox_core::fntrace::enter("set_affected_files_pending_emit"); 
        if self.build_info.affected_files_pending_emit.is_empty() {
            return;
        }
        let full_emit_kind = crate::mig::m4z2_2::get_file_emit_kind(
            &self.build_info.get_compiler_options(""),
        );
        self.affected_files_pending_emit = self
            .build_info
            .affected_files_pending_emit
            .iter()
            .map(|pending_emit| {
                let emit_kind = if pending_emit.emit_kind
                    == crate::mig::m4z2_2::FileEmitKind::None
                {
                    full_emit_kind
                } else {
                    pending_emit.emit_kind
                };
                ReadableBuildInfoFilePendingEmit {
                    file: self.to_file_path(pending_emit.file_id),
                    emit_kind: to_readable_file_emit_kind(emit_kind),
                    original: Some(*pending_emit),
                }
            })
            .collect();
    }

    pub fn set_emit_signatures(&mut self) { ::tsox_core::fntrace::enter("set_emit_signatures"); 
        self.emit_signatures = self
            .build_info
            .emit_signatures
            .iter()
            .map(|signature| ReadableBuildInfoEmitSignature {
                file: self.to_file_path(signature.file_id),
                signature: signature.signature.clone(),
                differs_only_in_dts_map: signature.differs_only_in_dts_map,
                differs_in_options: signature.differs_in_options,
                original: Some(signature.clone()),
            })
            .collect();
    }

    pub fn set_resolved_root(&mut self) { ::tsox_core::fntrace::enter("set_resolved_root"); 
        self.resolved_root = self
            .build_info
            .resolved_root
            .iter()
            .map(|original| ReadableBuildInfoResolvedRoot {
                resolved: self.to_file_path(original.resolved),
                root: self.to_file_path(original.root),
            })
            .collect();
    }
}
