#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tsox_compile::mig::m4x_2::HasFileName;
use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::tspath;
use tsox_core::tspath::Path;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3e::RepopulateDiagnosticInfo;
use tsox_frontend::ast::SourceFile;

use crate::mig::m4y_2::BuildInfo;
use crate::mig::m4y_2::BuildInfoDiagnostic;
use crate::mig::m4y_2::BuildInfoDiagnosticsOfFile;
use crate::mig::m4y_2::BuildInfoEmitSignature;
use crate::mig::m4y_2::BuildInfoFileId;
use crate::mig::m4y_2::BuildInfoFileIdListId;
use crate::mig::m4y_2::BuildInfoFileInfo;
use crate::mig::m4y_2::BuildInfoFilePendingEmit;
use crate::mig::m4y_2::BuildInfoReferenceMapEntry;
use crate::mig::m4y_2::BuildInfoResolvedRoot;
use crate::mig::m4y_2::BuildInfoRepopulateInfo;
use crate::mig::m4y_2::BuildInfoRoot;
use crate::mig::m4y_2::BuildInfoSemanticDiagnostic;
use crate::mig::m4y_3::EmitFilesHandler;
use crate::mig::m4y_3::EmitUpdate;
use crate::mig::m4y_3::SignatureUpdateKind;
use crate::mig::m4z2_2::get_file_emit_kind;
use crate::mig::m4z2_2::BuildInfoDiagnosticWithFileName;
use crate::mig::m4z2_2::DiagnosticsOrBuildInfoDiagnosticsWithFileName;
use crate::mig::m4z2_2::FileInfo;
use crate::mig::m4z2_2::Snapshot;
use crate::mig::m4z2_2::EmitSignature;
use crate::mig::m4z2_2::FileEmitKind;

pub fn snapshot_to_build_info(
    snapshot: &Snapshot,
    program: &tsox_compile::compiler::Program,
    build_info_file_name: &str,
) -> Result<BuildInfo, tsox_compile::mig::m4v_2::ContentMapperError> {
    let content_mapper_identities: Vec<String> = match program.content_mapper_project() {
        None => Vec::new(),
        Some(project) => project.identities().map_err(|_| {
            tsox_compile::mig::m4v_2::ContentMapperError::project_unavailable()
        })?,
    };
    let mut build_info = BuildInfo {
        version: tsox_core::core::mig::m3k::version().to_string(),
        content_mapper_identities,
        ..Default::default()
    };
    let mut to = ToBuildInfo {
        snapshot,
        program,
        build_info: &mut build_info,
        build_info_directory: tspath::get_directory_path(build_info_file_name),
        compare_paths_options: tspath::ComparePathsOptions {
            current_directory: program.get_current_directory().to_string(),
            use_case_sensitive_file_names: program.use_case_sensitive_file_names(),
        },
        file_name_to_file_id: HashMap::new(),
        file_names_to_file_id_list_id: HashMap::new(),
        roots: HashMap::new(),
    };

    if snapshot
        .options
        .as_ref()
        .map(|o| o.is_incremental())
        .unwrap_or(false)
    {
        to.collect_root_files();
        to.set_file_info_and_emit_signatures();
        to.set_root_of_incremental_program();
        to.set_compiler_options();
        to.set_referenced_map();
        to.set_change_file_set();
        to.set_semantic_diagnostics();
        to.set_emit_diagnostics();
        to.set_affected_files_pending_emit();
        if !snapshot
            .latest_changed_dts_file
            .lock()
            .unwrap()
            .is_empty()
        {
            to.build_info.latest_changed_dts_file = to.relative_to_build_info(
                &snapshot.latest_changed_dts_file.lock().unwrap(),
            );
        }
    } else {
        to.set_root_of_non_incremental_program();
    }
    to.build_info.errors =
        *snapshot.has_errors.lock().unwrap() == tsox_core::core::tristate::Tristate::True;
    to.build_info.semantic_errors = *snapshot.has_semantic_errors.lock().unwrap();
    to.build_info.check_pending = *snapshot.check_pending.lock().unwrap();
    to.set_package_jsons();
    Ok(build_info)
}

pub struct ToBuildInfo<'a> {
    pub snapshot: &'a Snapshot,
    pub program: &'a tsox_compile::compiler::Program,
    pub build_info: &'a mut BuildInfo,
    pub build_info_directory: String,
    pub compare_paths_options: tspath::ComparePathsOptions,
    pub file_name_to_file_id: HashMap<String, BuildInfoFileId>,
    pub file_names_to_file_id_list_id: HashMap<String, BuildInfoFileIdListId>,
    pub roots: HashMap<Path, Path>,
}

impl<'a> ToBuildInfo<'a> {
    pub fn relative_to_build_info(&self, path: &str) -> String {
        tspath::ensure_path_is_non_module_name(&tspath::mig::m3i::get_relative_path_from_directory(
            &self.build_info_directory,
            path,
            &self.compare_paths_options,
        ))
    }

    pub fn to_file_id(&mut self, path: &Path) -> BuildInfoFileId {
        let file_id = self
            .file_name_to_file_id
            .get(&path.to_string())
            .copied()
            .unwrap_or(0);
        if file_id != 0 {
            return file_id;
        }
        let lib_file = self.program.get_default_lib_file(path.as_str());
        match lib_file {
            Some(lib_file) if !lib_file.replaced => {
                self.build_info.file_names.push(lib_file.name.clone());
            }
            _ => {
                self.build_info
                    .file_names
                    .push(self.relative_to_build_info(&path.to_string()));
            }
        }
        let file_id = self.build_info.file_names.len() as BuildInfoFileId;
        self.file_name_to_file_id.insert(path.to_string(), file_id);
        file_id
    }

    pub fn to_file_id_list_id(&mut self, set: &Set<Path>) -> BuildInfoFileIdListId {
        let mut file_ids: Vec<BuildInfoFileId> = set
            .iter()
            .map(|path| self.to_file_id(path))
            .collect();
        file_ids.sort_unstable();
        let key = file_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let file_id_list_id = self
            .file_names_to_file_id_list_id
            .get(&key)
            .copied()
            .unwrap_or(0);
        if file_id_list_id != 0 {
            return file_id_list_id;
        }
        self.build_info.file_ids_list.push(file_ids);
        let file_id_list_id = self.build_info.file_ids_list.len() as BuildInfoFileIdListId;
        self.file_names_to_file_id_list_id
            .insert(key, file_id_list_id);
        file_id_list_id
    }

    pub fn to_relative_to_build_info_compiler_option_value(
        &self,
        option: &tsox_tsoptions::tsoptions::OptionDecl,
        value: &tsox_tsoptions::tsoptions::OptValue,
    ) -> tsox_tsoptions::tsoptions::OptValue {
        if option.kind == tsox_tsoptions::tsoptions::OptionKind::List && option.is_file_path {
            if let tsox_tsoptions::tsoptions::OptValue::List(list) = value {
                return tsox_tsoptions::tsoptions::OptValue::List(
                    list.iter()
                        .map(|s| self.relative_to_build_info(s))
                        .collect(),
                );
            }
        } else if option.is_file_path {
            if let tsox_tsoptions::tsoptions::OptValue::Str(str_value) = value {
                if !str_value.is_empty() {
                    return tsox_tsoptions::tsoptions::OptValue::Str(
                        self.relative_to_build_info(str_value),
                    );
                }
            }
        }
        value.clone()
    }

    pub fn to_build_info_diagnostics_from_file_name_diagnostics(
        &mut self,
        diagnostics: &[Box<BuildInfoDiagnosticWithFileName>],
    ) -> Vec<Box<BuildInfoDiagnostic>> {
        diagnostics
            .iter()
            .map(|d| {
                let file = if !d.file.0.is_empty() {
                    self.to_file_id(&d.file)
                } else {
                    0
                };
                Box::new(BuildInfoDiagnostic {
                    file,
                    no_file: d.no_file,
                    pos: d.pos,
                    end: d.end,
                    code: d.code,
                    category: d.category,
                    source: d.source.clone(),
                    message_text: d.message_text.clone(),
                    message_key: d.message_key.to_string(),
                    message_args: d.message_args.clone(),
                    message_chain: self
                        .to_build_info_diagnostics_from_file_name_diagnostics(&d.message_chain),
                    related_information: self.to_build_info_diagnostics_from_file_name_diagnostics(
                        &d.related_information,
                    ),
                    reports_unnecessary: d.reports_unnecessary,
                    reports_deprecated: d.reports_deprecated,
                    skipped_on_no_emit: d.skipped_on_no_emit,
                    repopulate_info: to_build_info_repopulate_info(d.repopulate_info.as_ref()),
                })
            })
            .collect()
    }

    pub fn to_build_info_diagnostics_from_diagnostics(
        &mut self,
        file_path: &Path,
        diagnostics: &[Diagnostic],
    ) -> Vec<Box<BuildInfoDiagnostic>> {
        diagnostics
            .iter()
            .map(|d| {
                let mut file: BuildInfoFileId = 0;
                let mut no_file = false;
                match &d.file {
                    None => no_file = true,
                    Some(file_source) => {
                        let source_path = Path::from(file_source.path());
                        if source_path != *file_path {
                            file = self.to_file_id(&source_path);
                        }
                    }
                }
                Box::new(BuildInfoDiagnostic {
                    file,
                    no_file,
                    pos: d.loc.pos,
                    end: d.loc.end,
                    code: d.code,
                    category: d.category,
                    source: d.source().to_string(),
                    message_text: d.message_text().to_string(),
                    message_key: d.message_key.to_string(),
                    message_args: Some(d.message_args.clone()),
                    message_chain: self
                        .to_build_info_diagnostics_from_diagnostics(file_path, &d.message_chain),
                    related_information: self.to_build_info_diagnostics_from_diagnostics(
                        file_path,
                        &d.related_information,
                    ),
                    reports_unnecessary: d.reports_unnecessary,
                    reports_deprecated: d.reports_deprecated,
                    skipped_on_no_emit: d.skipped_on_no_emit,
                    repopulate_info: d.repopulate_info().and_then(|info| {
                        to_build_info_repopulate_info(Some(
                            &tsox_frontend::ast::mig::m3e::RepopulateDiagnosticInfo {
                                kind: match info.kind {
                                    tsox_frontend::ast::mig::m3d_2::RepopulateDiagnosticKind::ModeMismatch => {
                                        tsox_frontend::ast::mig::m3e::RepopulateDiagnosticKind::ModeMismatch
                                    }
                                    tsox_frontend::ast::mig::m3d_2::RepopulateDiagnosticKind::ModuleNotFound => {
                                        tsox_frontend::ast::mig::m3e::RepopulateDiagnosticKind::ModuleNotFound
                                    }
                                },
                                module_reference: info.module_reference.clone(),
                                mode: info.mode,
                                package_name: info.package_name.clone(),
                            },
                        ))
                    }),
                })
            })
            .collect()
    }

    pub fn to_build_info_diagnostics_of_file(
        &mut self,
        file_path: &Path,
        diags: &DiagnosticsOrBuildInfoDiagnosticsWithFileName,
    ) -> Option<BuildInfoDiagnosticsOfFile> {
        if !diags.diagnostics.is_empty() {
            return Some(BuildInfoDiagnosticsOfFile {
                file_id: self.to_file_id(file_path),
                diagnostics: self.to_build_info_diagnostics_from_diagnostics(
                    file_path,
                    &diags.diagnostics,
                ),
            });
        }
        if !diags.build_info_diagnostics.is_empty() {
            return Some(BuildInfoDiagnosticsOfFile {
                file_id: self.to_file_id(file_path),
                diagnostics: self
                    .to_build_info_diagnostics_from_file_name_diagnostics(
                        &diags.build_info_diagnostics,
                    ),
            });
        }
        None
    }

    pub fn collect_root_files(&mut self) {
        for file_name in self.program.command_line().file_names() {
            let redirect = self.program.get_parse_file_redirect(&file_name);
            let file = if !redirect.is_empty() {
                self.program.get_source_file(&redirect)
            } else {
                self.program.get_source_file(&file_name)
            };
            if let Some(file) = file {
                self.roots.insert(
                    Path::from(file.path()),
                    tspath::to_path(
                        &file_name,
                        &self.compare_paths_options.current_directory,
                        self.compare_paths_options.use_case_sensitive_file_names,
                    ),
                );
            }
        }
    }

    pub fn set_file_info_and_emit_signatures(&mut self) {
        let mut file_infos = Vec::with_capacity(self.program.get_source_files().len());
        let is_composite = self
            .snapshot
            .options
            .as_ref()
            .map(|o| o.composite.is_true())
            .unwrap_or(false);
        for file in self.program.get_source_files() {
            let file_path = Path::from(file.path());
            let info = self.snapshot.file_infos.load(&file_path).unwrap_or_default();
            let file_id = self.to_file_id(&file_path);
            if self.build_info.file_names[(file_id - 1) as usize]
                != self.relative_to_build_info(&file_path.to_string())
            {
                let lib_file = self.program.get_default_lib_file(file_path.as_str());
                let matches_lib = lib_file
                    .map(|lib| !lib.replaced && self.build_info.file_names[(file_id - 1) as usize] == lib.name)
                    .unwrap_or(false);
                if !matches_lib {
                    panic!(
                        "File name at index {} does not match expected relative path or libName: {} != {}",
                        file_id - 1,
                        self.build_info.file_names[(file_id - 1) as usize],
                        self.relative_to_build_info(&file_path.to_string())
                    );
                }
            }
            if is_composite
                && !tsox_frontend::ast::is_json_source_file(file.as_ref())
                && self.program.source_file_may_be_emitted(&file, false)
            {
                let emit_signature = self.snapshot.emit_signatures.load(&file_path);
                match emit_signature {
                    None => {
                        self.build_info
                            .emit_signatures
                            .push(BuildInfoEmitSignature {
                                file_id,
                                ..Default::default()
                            });
                    }
                    Some(emit_signature) => {
                        if emit_signature.signature != info.signature {
                            let mut incremental_emit_signature = BuildInfoEmitSignature {
                                file_id,
                                ..Default::default()
                            };
                            if !emit_signature.signature.is_empty() {
                                incremental_emit_signature.signature =
                                    emit_signature.signature.clone();
                            } else if emit_signature.signature_with_different_options[0]
                                == info.signature
                            {
                                incremental_emit_signature.differs_only_in_dts_map = true;
                            } else {
                                incremental_emit_signature.signature = emit_signature
                                    .signature_with_different_options[0]
                                    .clone();
                                incremental_emit_signature.differs_in_options = true;
                            }
                            self.build_info
                                .emit_signatures
                                .push(incremental_emit_signature);
                        }
                    }
                }
            }
            file_infos.push(BuildInfoFileInfo::new(&info));
        }
        self.build_info.file_infos = file_infos;
    }

    pub fn set_root_of_incremental_program(&mut self) {
        let mut keys: Vec<Path> = self.roots.keys().cloned().collect();
        keys.sort_by_key(|path| self.to_file_id(path));
        for file_path in keys {
            let root_path = self.roots[&file_path].clone();
            let root = self.to_file_id(&root_path);
            let resolved = self.to_file_id(&file_path);
            if self.build_info.root.is_empty() {
                self.build_info
                    .root
                    .push(BuildInfoRoot { start: resolved, end: 0, non_incremental: String::new() });
            } else {
                let last = self.build_info.root.last_mut().unwrap();
                if last.end == resolved - 1 {
                    last.end = resolved;
                } else if last.end == 0 && last.start == resolved - 1 {
                    last.end = resolved;
                } else {
                    self.build_info
                        .root
                        .push(BuildInfoRoot { start: resolved, end: 0, non_incremental: String::new() });
                }
            }
            if root != resolved {
                self.build_info
                    .resolved_root
                    .push(BuildInfoResolvedRoot { resolved, root });
            }
        }
    }

    pub fn set_compiler_options(&mut self) {
        let Some(options) = self.snapshot.options.clone() else {
            return;
        };
        tsox_tsoptions::mig::m5h_2::for_each_compiler_option_value(
            &options,
            &|option| option_affects_build_info(option.name),
            &mut |option, value, _i| {
                if value_is_zero(value) {
                    return false;
                }
                let relative_value =
                    self.to_relative_to_build_info_compiler_option_value(option, value);
                self.build_info
                    .options
                    .get_or_insert_with(OrderedMap::new)
                    .set(
                        option.name.to_string(),
                        opt_value_to_json(&relative_value),
                    );
                false
            },
        );
    }

    pub fn set_referenced_map(&mut self) {
        let mut keys = self.snapshot.referenced_map.get_paths_with_references();
        keys.sort_by(|a, b| a.0.cmp(&b.0));
        let mut referenced_map = Vec::with_capacity(keys.len());
        for file_path in keys {
            let references = self
                .snapshot
                .referenced_map
                .get_references(&file_path)
                .unwrap_or_default();
            let entry = BuildInfoReferenceMapEntry {
                file_id: self.to_file_id(&file_path),
                file_id_list_id: self.to_file_id_list_id(&references),
            };
            referenced_map.push(entry);
        }
        self.build_info.referenced_map = referenced_map;
    }

    pub fn set_change_file_set(&mut self) {
        let mut files: Vec<Path> = self
            .snapshot
            .changed_files_set
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .collect();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let change_file_set = files
            .iter()
            .map(|path| self.to_file_id(path))
            .collect();
        self.build_info.change_file_set = change_file_set;
    }

    pub fn set_semantic_diagnostics(&mut self) {
        for file in self.program.get_source_files() {
            let file_path = Path::from(file.path());
            match self
                .snapshot
                .semantic_diagnostics_per_file
                .load(&file_path)
            {
                None => {
                    if !self.snapshot.changed_files_set.lock().unwrap().contains(&file_path) {
                        let file_id = self.to_file_id(&file_path);
                        self.build_info
                            .semantic_diagnostics_per_file
                            .push(BuildInfoSemanticDiagnostic {
                                file_id,
                                diagnostics: None,
                            });
                    }
                }
                Some(value) => {
                    if let Some(diagnostics) =
                        self.to_build_info_diagnostics_of_file(&file_path, &value)
                    {
                        self.build_info
                            .semantic_diagnostics_per_file
                            .push(BuildInfoSemanticDiagnostic {
                                file_id: 0,
                                diagnostics: Some(diagnostics),
                            });
                    }
                }
            }
        }
    }

    pub fn set_emit_diagnostics(&mut self) {
        let mut files = self.snapshot.emit_diagnostics_per_file.keys();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let mut emit_diagnostics_per_file = Vec::with_capacity(files.len());
        for file_path in files {
            let value = self
                .snapshot
                .emit_diagnostics_per_file
                .load(&file_path)
                .unwrap_or_default();
            if let Some(diagnostics) = self.to_build_info_diagnostics_of_file(&file_path, &value) {
                emit_diagnostics_per_file.push(diagnostics);
            }
        }
        self.build_info.emit_diagnostics_per_file = emit_diagnostics_per_file;
    }

    pub fn set_affected_files_pending_emit(&mut self) {
        let mut files = self.snapshot.affected_files_pending_emit.keys();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let full_emit_kind = self
            .snapshot
            .options
            .as_ref()
            .map(get_file_emit_kind)
            .unwrap_or(FileEmitKind::None);
        for file_path in files {
            let Some(file) = self.program.get_source_file_by_path(file_path.as_str()) else {
                continue;
            };
            if !self.program.source_file_may_be_emitted(&file, false) {
                continue;
            }
            let pending_emit = self
                .snapshot
                .affected_files_pending_emit
                .load(&file_path)
                .unwrap_or(FileEmitKind::None);
            let file_id = self.to_file_id(&file_path);
            self.build_info
                .affected_files_pending_emit
                .push(BuildInfoFilePendingEmit {
                    file_id,
                    emit_kind: if pending_emit == full_emit_kind {
                        FileEmitKind::None
                    } else {
                        pending_emit
                    },
                });
        }
    }

    pub fn set_root_of_non_incremental_program(&mut self) {
        self.build_info.root = self
            .program
            .command_line()
            .file_names()
            .iter()
            .map(|file_name| BuildInfoRoot {
                start: 0,
                end: 0,
                non_incremental: self.relative_to_build_info(
                    &tspath::to_path(
                        file_name,
                        &self.compare_paths_options.current_directory,
                        self.compare_paths_options.use_case_sensitive_file_names,
                    )
                    .to_string(),
                ),
            })
            .collect();
    }

    pub fn set_package_jsons(&mut self) {
        if let Some(package_jsons) = &*self.snapshot.package_jsons.lock().unwrap() {
            if !package_jsons.is_empty() {
                self.build_info.package_jsons = Some(
                    package_jsons
                        .iter()
                        .map(|p| self.relative_to_build_info(p))
                        .collect(),
                );
            }
        }
        if let Some(missing_package_jsons) = &*self.snapshot.missing_package_jsons.lock().unwrap() {
            if !missing_package_jsons.is_empty() {
                self.build_info.missing_package_jsons = Some(
                    missing_package_jsons
                        .iter()
                        .map(|p| self.relative_to_build_info(p))
                        .collect(),
                );
            }
        }
    }
}

pub fn to_build_info_repopulate_info(
    info: Option<&RepopulateDiagnosticInfo>,
) -> Option<BuildInfoRepopulateInfo> {
    info.map(|info| BuildInfoRepopulateInfo {
        kind: info.kind,
        module_reference: info.module_reference.clone(),
        mode: info.mode,
        package_name: info.package_name.clone(),
    })
}

fn value_is_zero(value: &tsox_tsoptions::tsoptions::OptValue) -> bool {
    match value {
        tsox_tsoptions::tsoptions::OptValue::Null => true,
        tsox_tsoptions::tsoptions::OptValue::Bool(b) => !*b,
        tsox_tsoptions::tsoptions::OptValue::Num(n) => *n == 0,
        tsox_tsoptions::tsoptions::OptValue::Str(s) => s.is_empty(),
        tsox_tsoptions::tsoptions::OptValue::List(l) => l.is_empty(),
    }
}

fn opt_value_to_json(value: &tsox_tsoptions::tsoptions::OptValue) -> serde_json::Value {
    match value {
        tsox_tsoptions::tsoptions::OptValue::Bool(b) => serde_json::Value::Bool(*b),
        tsox_tsoptions::tsoptions::OptValue::Str(s) => serde_json::Value::String(s.clone()),
        tsox_tsoptions::tsoptions::OptValue::Num(n) => {
            serde_json::Value::Number(serde_json::Number::from(*n))
        }
        tsox_tsoptions::tsoptions::OptValue::List(list) => serde_json::Value::Array(
            list.iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        ),
        tsox_tsoptions::tsoptions::OptValue::Null => serde_json::Value::Null,
    }
}

fn option_affects_build_info(name: &str) -> bool {
    matches!(
        name,
        "allowImportingTsExtensions"
            | "allowJs"
            | "allowSyntheticDefaultImports"
            | "allowUmdGlobalAccess"
            | "allowUnreachableCode"
            | "allowUnusedLabels"
            | "alwaysStrict"
            | "assumeChangesOnlyAffectDirectDependencies"
            | "checkJs"
            | "composite"
            | "declaration"
            | "declarationDir"
            | "declarationMap"
            | "downlevelIteration"
            | "emitBOM"
            | "emitDeclarationOnly"
            | "emitDecoratorMetadata"
            | "erasableSyntaxOnly"
            | "esModuleInterop"
            | "exactOptionalPropertyTypes"
            | "experimentalDecorators"
            | "importHelpers"
            | "inlineSourceMap"
            | "inlineSources"
            | "isolatedDeclarations"
            | "jsx"
            | "jsxImportSource"
            | "mapRoot"
            | "newLine"
            | "noEmitHelpers"
            | "noEmitOnError"
            | "noErrorTruncation"
            | "noFallthroughCasesInSwitch"
            | "noImplicitAny"
            | "noImplicitOverride"
            | "noImplicitReturns"
            | "noImplicitThis"
            | "noPropertyAccessFromIndexSignature"
            | "noUncheckedIndexedAccess"
            | "noUncheckedSideEffectImports"
            | "noUnusedLocals"
            | "noUnusedParameters"
            | "outDir"
            | "outFile"
            | "preserveConstEnums"
            | "reactNamespace"
            | "removeComments"
            | "rewriteRelativeImportExtensions"
            | "rootDir"
            | "skipDefaultLibCheck"
            | "skipLibCheck"
            | "sourceMap"
            | "sourceRoot"
            | "stableTypeOrdering"
            | "strict"
            | "strictBindCallApply"
            | "strictBuiltinIteratorReturn"
            | "strictFunctionTypes"
            | "strictNullChecks"
            | "strictPropertyInitialization"
            | "stripInternal"
            | "tsBuildInfoFile"
            | "useDefineForClassFields"
            | "useUnknownInCatchVariables"
            | "verbatimModuleSyntax"
    )
}

impl<'a> EmitFilesHandler<'a> {
    pub fn skip_dts_output_of_composite(
        &self,
        file: &Arc<SourceFile>,
        output_file_name: &str,
        text: &str,
        data: &crate::mig::m4y_3::WriteFileData,
        new_signature: &mut String,
        differs_only_in_map: &mut bool,
    ) -> bool {
        if !self
            .program
            .snapshot
            .options
            .as_ref()
            .map(|o| o.composite.is_true())
            .unwrap_or(false)
        {
            return false;
        }
        let file_path = Path::from(file.path());
        let mut old_signature = String::new();
        let old_signature_format = self.program.snapshot.emit_signatures.load(&file_path);
        if let Some(old_signature_format) = &old_signature_format {
            if !old_signature_format.signature.is_empty() {
                old_signature = old_signature_format.signature.clone();
            } else {
                old_signature = old_signature_format.signature_with_different_options[0].clone();
            }
        }
        if new_signature.is_empty() {
            *new_signature = self.program.snapshot.compute_hash(
                &crate::mig::m4z2_2::get_text_handling_source_map_for_signature(
                    text, data,
                ),
            );
        }
        if *new_signature == old_signature {
            if old_signature_format
                .as_ref()
                .map(|f| f.signature == old_signature)
                .unwrap_or(false)
            {
                return true;
            }
            *differs_only_in_map = self
                .program
                .snapshot
                .options
                .as_ref()
                .map(|o| o.build.is_true())
                .unwrap_or(false);
        } else {
            self.latest_changed_dts_files
                .store(file_path.clone(), output_file_name.to_string());
        }
        self.emit_signatures.store(
            file_path,
            crate::mig::m4z2_2::EmitSignature {
                signature: new_signature.clone(),
                signature_with_different_options: Vec::new(),
            },
        );
        false
    }

    pub fn update_snapshot(&mut self) -> Vec<crate::mig::m4y_3::EmitResult> {
        if self.program.snapshot.can_use_incremental_state() {
            self.signatures.for_each(|file, signature| {
                if let Some(mut info) = self.program.snapshot.file_infos.load(file) {
                    info.signature = signature.clone();
                    self.program.snapshot.file_infos.store(file.clone(), info);
                    if let Some(testing_data) = &self.program.testing_data {
                        testing_data
                            .updated_signature_kinds
                            .store(file.clone(), SignatureUpdateKind::StoredAtEmit);
                    }
                }
                self.program
                    .snapshot
                    .build_info_emit_pending
                    .store(true, Ordering::SeqCst);
                true
            });
            self.emit_signatures.for_each(|file, signature| {
                self.program
                    .snapshot
                    .emit_signatures
                    .store(file.clone(), signature.clone());
                self.program
                    .snapshot
                    .build_info_emit_pending
                    .store(true, Ordering::SeqCst);
                true
            });
            for file in self.deleted_pending_kinds.iter() {
                self.program
                    .snapshot
                    .affected_files_pending_emit
                    .delete(file);
                self.program
                    .snapshot
                    .build_info_emit_pending
                    .store(true, Ordering::SeqCst);
            }
            let mut results = Vec::new();
            for file in self.program.get_source_files() {
                let file_path = Path::from(file.path());
                if let Some(latest_changed_dts_file) = self.latest_changed_dts_files.load(&file_path)
                {
                    *self.program.snapshot.latest_changed_dts_file.lock().unwrap() =
                        latest_changed_dts_file;
                    self.program
                        .snapshot
                        .build_info_emit_pending
                        .store(true, Ordering::SeqCst);
                    self.program
                        .snapshot
                        .has_changed_dts_file
                        .store(true, Ordering::SeqCst);
                }
                if let Some(update) = self.emit_updates.load(&file_path) {
                    if !update.dts_errors_from_cache {
                        if update.pending_kind == FileEmitKind::None {
                            self.program
                                .snapshot
                                .affected_files_pending_emit
                                .delete(&file_path);
                        } else {
                            self.program
                                .snapshot
                                .affected_files_pending_emit
                                .store(file_path.clone(), update.pending_kind);
                        }
                        self.program
                            .snapshot
                            .build_info_emit_pending
                            .store(true, Ordering::SeqCst);
                    }
                    if let Some(result) = &update.result {
                        results.push(crate::mig::m4y_3::combine_emit_results(std::slice::from_ref(
                            result,
                        )));
                        if !result.diagnostics.is_empty() {
                            self.program.snapshot.emit_diagnostics_per_file.store(
                                file_path,
                                DiagnosticsOrBuildInfoDiagnosticsWithFileName {
                                    diagnostics: result.diagnostics.clone(),
                                    build_info_diagnostics: Vec::new(),
                                },
                            );
                        }
                    }
                }
            }
            return results;
        } else if self
            .has_emit_diagnostics
            .load(Ordering::SeqCst)
        {
            self.program
                .snapshot
                .has_emit_diagnostics
                .store(true, Ordering::SeqCst);
        }
        Vec::new()
    }
}
