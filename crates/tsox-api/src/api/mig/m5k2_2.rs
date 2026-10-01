#![allow(unused_imports, dead_code)]

//! m5k2 批次 2:session.go handle* 方法族移植(归属待接线)。

use std::sync::Arc;

use serde_json::Value as JsonValue;

use super::m5j_decoder::decode_nodes;
use super::m5k_3::{literal_value_to_json, new_project_response, LiteralValue};
use super::m5l::{
    client_error, new_diagnostic_responses, CheckerNodeParams, CheckerSetup, CheckerTypeParams,
    DiagnosticResponse, DocumentIdentifier, GetSourceFileParams, Program, ProjectId, Session,
    SignatureId, SnapshotData, SnapshotId, SymbolResponse, TypeId, TypeResponse,
};
use super::m5l_3::base64_decode;
use tsox_checker::checker::Checker;
use tsox_checker::checker::types::Type;
use tsox_core::core::text_change::{apply_bulk_edits, TextChange};
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::Symbol;
use tsox_lsp::project::snapshot::Snapshot as ProjectSnapshot;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ApiFileChanges {
    pub invalidate_all: bool,
    pub changed: Vec<DocumentIdentifier>,
    pub created: Vec<DocumentIdentifier>,
    pub deleted: Vec<DocumentIdentifier>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct CreateProgramParams {
    pub root_files: Vec<DocumentIdentifier>,
    pub create_program_options: CreateProgramOptions,
    pub old_program: Option<CreateProgramOldProgramParams>,
    pub file_changes: Option<ApiFileChanges>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct CreateProgramOptions {
    pub compiler_options: serde_json::Value,
    pub project_references: Vec<serde_json::Value>,
    pub config_file_parsing_diagnostics: Vec<DiagnosticResponse>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct CreateProgramOldProgramParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
}

#[derive(serde::Serialize)]
pub struct CreateProgramResponse {
    pub snapshot: SnapshotId,
    pub project: super::m5k_3::ProjectResponse,
}

fn snapshot_handle(snapshot: &ProjectSnapshot) -> SnapshotId {
    snapshot.id()
}

impl Session {
    pub fn handle_create_program(
        &self,
        params: &CreateProgramParams,
    ) -> Result<CreateProgramResponse, String> {
        if params.file_changes.is_some() && params.old_program.is_none() {
            return Err(client_error("fileChanges requires an oldProgram"));
        }
        let root_file_names: Vec<String> = params
            .root_files
            .iter()
            .map(|root_file| {
                root_file.to_absolute_file_name(&self.project_session.get_current_directory())
            })
            .collect();
        let (compiler_options, _) = tsox_tsoptions::mig::m5j_2::convert_compiler_options_from_json_worker(
            Some(&params.create_program_options.compiler_options),
            &self.project_session.get_current_directory(),
            "",
        );
        let compiler_options = compiler_options.unwrap_or_default();
        let project_references: Vec<tsox_core::core::project_reference::ProjectReference> = params
            .create_program_options
            .project_references
            .iter()
            .map(|value| {
                let obj = value.as_object();
                let path = obj
                    .and_then(|o| o.get("path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let original_path = obj
                    .and_then(|o| o.get("originalPath"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                tsox_core::core::project_reference::ProjectReference {
                    original_path: if original_path.is_empty() {
                        path.clone()
                    } else {
                        original_path
                    },
                    path,
                    circular: false,
                }
            })
            .collect();
        let config_file_parsing_diagnostics: Vec<tsox_frontend::ast::diagnostic::Diagnostic> =
            params
                .create_program_options
                .config_file_parsing_diagnostics
                .iter()
                .map(|d| d.to_diagnostic())
                .collect();
        let file_changes = self.to_file_change_summary(params.file_changes.as_ref());
        let mut old_snapshot: Option<&ProjectSnapshot> = None;
        let mut old_project: Option<tsox_lsp::project::project::Project> = None;
        let mut snapshots_guard = self.snapshots.lock().unwrap();
        if let Some(old_program) = &params.old_program {
            let Some(old_sd) = snapshots_guard.get_mut(&old_program.snapshot) else {
                return Err(client_error(format!(
                    "snapshot {} not found",
                    old_program.snapshot
                )));
            };
            old_sd.ref_count += 1;
            old_project = Some(old_sd.get_project(&old_program.project)?);
            old_snapshot = Some(&old_sd.snapshot);
        }
        let snapshot = self.project_session.api_create_program(
            &root_file_names,
            &compiler_options,
            &project_references,
            &config_file_parsing_diagnostics,
            old_snapshot,
            old_project.as_ref(),
            file_changes,
        );
        drop(snapshots_guard);
        if let Some(old_program) = &params.old_program {
            self.release_snapshot(old_program.snapshot)?;
        }
        let project = snapshot
            .project_collection
            .as_ref()
            .and_then(|pc| pc.inferred_project());
        let Some(project) = project else {
            snapshot.deref(&self.project_session);
            return Err(client_error("failed to create synthetic project"));
        };
        let project_response = new_project_response(Some(project));
        let handle = snapshot_handle(&snapshot);
        let mut snapshots = self.snapshots.lock().unwrap();
        if let Some(sd) = snapshots.get_mut(&handle) {
            snapshot.deref(&self.project_session);
            sd.ref_count += 1;
        } else {
            snapshots.insert(
                handle.clone(),
                SnapshotData {
                    snapshot: snapshot.clone(),
                    ref_count: 1,
                    ..Default::default()
                },
            );
        }
        drop(snapshots);
        Ok(CreateProgramResponse {
            snapshot: handle,
            project: project_response,
        })
    }

    pub fn handle_get_config_file_names(
        &self,
        params: &super::m5l::GetProjectDiagnosticsParams,
    ) -> Result<Option<Vec<String>>, String> {
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let Some(command_line) = program.command_line() else {
            return Ok(None);
        };
        if command_line.config_file_name.is_empty() {
            return Ok(None);
        }
        let mut config_files = vec![command_line.config_file_name.clone()];
        config_files.extend(command_line.extended_source_files().iter().cloned());
        Ok(Some(config_files))
    }

    pub fn handle_get_config_source_file(
        &self,
        params: &GetSourceFileParams,
    ) -> Result<Option<super::m5l::SourceFileResponse>, String> {
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let Some(command_line) = program
            .command_line()
            .filter(|c| !c.config_file_name.is_empty())
        else {
            return self.encode_source_file_response(None);
        };
        let requested_path = tsox_core::tspath::to_path(
            &params.file.to_file_name(),
            &program.get_current_directory(),
            program.use_case_sensitive_file_names(),
        );
        let mut config_file_names: Vec<String> = vec![command_line.config_file_name.clone()];
        config_file_names.extend(command_line.extended_source_files().iter().cloned());
        for config_file_name in config_file_names {
            if tsox_core::tspath::to_path(
                &config_file_name,
                &program.get_current_directory(),
                program.use_case_sensitive_file_names(),
            ) != requested_path
            {
                continue;
            }
            let Some(config_file_content) = sd.snapshot.read_file(&config_file_name) else {
                return self.encode_source_file_response(None);
            };
            let config_source_file = new_tsconfig_source_file_from_path(
                &config_file_name,
                &requested_path,
                &config_file_content,
            );
            return self.encode_source_file_response(Some(Arc::new(config_source_file.source_file)));
        }
        self.encode_source_file_response(None)
    }

    pub fn handle_emit(&self, params: &EmitParams) -> Result<EmitResponse, String> {
        let (program, mut options) = self.get_emit_options(params)?;
        let project_session = self.project_session.clone();
        options.write_file = Some(Box::new(
            move |file_name: &str, text: &str, _data: &WriteFileData| {
                if let Some(fs) = project_session.fs() {
                    fs.write_file(file_name, text);
                }
                Ok(())
            },
        ));
        let result = emit_program(&program, &options)?;
        let diags: Vec<Arc<tsox_frontend::ast::Diagnostic>> =
            result.diagnostics.iter().map(|d| Arc::new(d.clone())).collect();
        Ok(EmitResponse {
            emit_skipped: result.emit_skipped,
            diagnostics: new_diagnostic_responses(&diags),
            emitted_files: result.emitted_files.clone(),
        })
    }

    pub fn handle_emit_to_string(&self, params: &EmitParams) -> Result<EmitOutputResponse, String> {
        let (program, options) = self.get_emit_options(params)?;
        emit_to_output(&program, options)
    }

    pub fn get_emit_options(&self, params: &EmitParams) -> Result<(Program, EmitOptions), String> {
        let program = self.get_emit_program(params.snapshot, &params.project)?;
        let emit_only = get_emit_only(params.emit_only.as_ref())?;
        Ok((
            program,
            EmitOptions {
                emit_only,
                ..Default::default()
            },
        ))
    }
}

#[derive(serde::Deserialize)]
pub struct EmitParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub emit_only: Option<u32>,
}

pub enum EmitOnly {
    All = 0,
    Js = 1,
    Dts = 2,
    BuilderSignature = 3,
}

impl EmitOnly {
    pub fn from_bits_retain(bits: u32) -> EmitOnly {
        match bits {
            1 => EmitOnly::Js,
            2 => EmitOnly::Dts,
            3 => EmitOnly::BuilderSignature,
            _ => EmitOnly::All,
        }
    }
}

impl Default for EmitOnly {
    fn default() -> EmitOnly {
        EmitOnly::All
    }
}

pub struct WriteFileData {
    pub source_map_url_pos: usize,
    pub build_info: Option<JsonValue>,
    pub diagnostics: Vec<tsox_frontend::ast::Diagnostic>,
    pub skipped_dts_write: bool,
    pub source_file: Option<SourceFile>,
}

#[derive(Default)]
pub struct EmitOptions {
    pub target_source_files: Option<Vec<SourceFile>>,
    pub emit_only: EmitOnly,
    pub force_emit: bool,
    pub write_file: Option<Box<dyn Fn(&str, &str, &WriteFileData) -> std::io::Result<()>>>,
}

pub struct EmitResult {
    pub emit_skipped: bool,
    pub diagnostics: Vec<tsox_frontend::ast::Diagnostic>,
    pub emitted_files: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct EmitResponse {
    pub emit_skipped: bool,
    pub diagnostics: Vec<DiagnosticResponse>,
    pub emitted_files: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct EmitOutputFile {
    pub file_name: String,
    pub text: String,
    pub source_file_name: Option<String>,
}

#[derive(serde::Serialize)]
pub struct EmitOutputResponse {
    pub emit_skipped: bool,
    pub diagnostics: Vec<DiagnosticResponse>,
    pub output_files: Vec<EmitOutputFile>,
}

pub fn emit_to_output(
    program: &Program,
    mut options: EmitOptions,
) -> Result<EmitOutputResponse, String> {
    let output_files = std::rc::Rc::new(std::cell::RefCell::new(Vec::<EmitOutputFile>::new()));
    let sink = std::rc::Rc::clone(&output_files);
    options.write_file = Some(Box::new(
        move |file_name: &str, text: &str, data: &WriteFileData| {
            let source_file_name = data.source_file.as_ref().map(|sf| sf.file_name.clone());
            sink.borrow_mut().push(EmitOutputFile {
                file_name: file_name.to_string(),
                text: text.to_string(),
                source_file_name,
            });
            Ok(())
        },
    ));
    let result = emit_program(program, &options)?;
    drop(options);
    let diags: Vec<Arc<tsox_frontend::ast::Diagnostic>> =
        result.diagnostics.iter().map(|d| Arc::new(d.clone())).collect();
    let mut output_files = std::rc::Rc::into_inner(output_files)
        .expect("write_file callback released")
        .into_inner();
    output_files.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(EmitOutputResponse {
        emit_skipped: result.emit_skipped,
        diagnostics: new_diagnostic_responses(&diags),
        output_files,
    })
}

pub fn get_emit_only(value: Option<&u32>) -> Result<EmitOnly, String> {
    let Some(value) = value else {
        return Ok(EmitOnly::All);
    };
    if *value > EmitOnly::Dts as u32 {
        return Err(client_error(format!("invalid emitOnly value: {}", value)));
    }
    Ok(EmitOnly::from_bits_retain(*value))
}

pub fn emit_program(program: &Program, options: &EmitOptions) -> Result<EmitResult, String> {
    let Some(result) = program.emit(options) else {
        return Err("compiler emit returned nil result".to_string());
    };
    Ok(result)
}

#[derive(serde::Deserialize)]
pub struct FormatNodeForInsertionParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
    pub position: u32,
    pub data: String,
}

impl Session {
    pub fn handle_format_node_for_insertion(
        &self,
        params: &FormatNodeForInsertionParams,
    ) -> Result<String, String> {
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let Some(target_source_file) = program.get_source_file(&params.file.to_file_name()) else {
            return Err(client_error(format!(
                "source file not found: {}",
                params.file.to_string_value()
            )));
        };
        let data = base64_decode(&params.data)
            .map_err(|e| client_error(format!("invalid base64 data: {}", e)))?;
        let node = decode_nodes(&data)
            .map_err(|e| client_error(format!("failed to decode AST: {}", e)))?;
        let pos = tsox_frontend::ast::mig::m3b_2::get_position_map(&target_source_file)
            .utf16_to_utf8(params.position as usize);
        let format_options = sd.snapshot.user_preferences().format_code_settings.clone();
        let new_line = format_options.new_line_character.clone();
        let (text, node_with_pos) =
            print_and_position_node(&node, &new_line, format_options.indent_size as usize);
        let parse_options = tsox_frontend::ast::mig::m3b_2::parse_options(&target_source_file);
        let synthetic_parse_options = tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions {
            file_name: parse_options.file_name.clone(),
            path: parse_options.path.clone(),
            external_module_indicator_options: Default::default(),
        };
        let synthetic_file =
            create_synthetic_source_file(&node_with_pos, &text, synthetic_parse_options);
        let is_at_line_start =
            get_line_start_position_for_position(pos, &target_source_file) == pos;
        let initial_indentation =
            get_indentation(pos, &target_source_file, &format_options, is_at_line_start);
        let delta: i64 = if format_options.indent_size != 0
            && should_indent_child_node(&format_options, &node)
        {
            format_options.indent_size as i64
        } else {
            0
        };
        let changes = format_node_given_indentation(
            &node_with_pos,
            &synthetic_file,
            target_source_file.language_variant,
            initial_indentation,
            delta,
        );
        Ok(apply_bulk_edits(&text, &changes))
    }
}

fn print_and_position_node(
    _node: &Arc<Node>,
    _new_line: &str,
    _indent_size: usize,
) -> (String, Arc<Node>) {
    unimplemented!("printer.PrintAndPositionNode")
}

fn create_synthetic_source_file(
    _node: &Arc<Node>,
    _text: &str,
    _parse_options: tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions,
) -> SourceFile {
    unimplemented!("printer.CreateSyntheticSourceFile")
}

fn get_line_start_position_for_position(_position: usize, _file: &SourceFile) -> usize {
    unimplemented!("format.GetLineStartPositionForPosition")
}

fn get_indentation(
    _position: usize,
    _file: &SourceFile,
    _format_options: &tsox_lsp::ls::lsutil::FormatCodeSettings,
    _at_line_start: bool,
) -> i64 {
    unimplemented!("format.GetIndentation")
}

fn should_indent_child_node(
    _format_options: &tsox_lsp::ls::lsutil::FormatCodeSettings,
    _node: &Arc<Node>,
) -> bool {
    unimplemented!("format.ShouldIndentChildNode")
}

fn format_node_given_indentation(
    _node: &Arc<Node>,
    _file: &SourceFile,
    _language_variant: tsox_frontend::ast::node_source_file::LanguageVariant,
    _initial_indentation: i64,
    _delta: i64,
) -> Vec<TextChange> {
    unimplemented!("format.FormatNodeGivenIndentation")
}

pub struct TsconfigSourceFile {
    pub source_file: SourceFile,
}

fn new_tsconfig_source_file_from_path(
    _file_name: &str,
    _path: &tsox_core::tspath::Path,
    _content: &str,
) -> TsconfigSourceFile {
    unimplemented!("tsoptions.NewTsconfigSourceFileFromFilePath")
}

impl Program {
    pub(crate) fn command_line(&self) -> Option<&tsox_tsoptions::tsoptions::ParsedCommandLine> {
        Some(self.0.as_ref().command_line())
    }

    pub(crate) fn get_current_directory(&self) -> String {
        self.0.as_ref().get_current_directory().to_string()
    }

    pub(crate) fn use_case_sensitive_file_names(&self) -> bool {
        self.0.as_ref().use_case_sensitive_file_names()
    }

    pub(crate) fn emit(&self, options: &EmitOptions) -> Option<EmitResult> {
        let program = self.0.as_ref();
        let write_file = options.write_file.as_ref();
        let data = WriteFileData {
            source_map_url_pos: 0,
            build_info: None,
            diagnostics: Vec::new(),
            skipped_dts_write: false,
            source_file: None,
        };
        let result = program.emit(&|file_name: &str, text: &str| match write_file {
            Some(write) => write(file_name, text, &data),
            None => Ok(()),
        });
        Some(EmitResult {
            emit_skipped: result.emit_skipped,
            diagnostics: result
                .diagnostics
                .iter()
                .map(|text| {
                    tsox_frontend::ast::mig::m3d_2::new_diagnostic_from_text(
                        None,
                        tsox_core::core::text::TextRange::new(0, 0),
                        0,
                        tsox_core::diagnostics::Category::Error,
                        text.clone(),
                        Vec::new(),
                        Vec::new(),
                        false,
                        false,
                    )
                })
                .collect(),
            emitted_files: result.emitted_files.clone(),
        })
    }

    pub(crate) fn get_bind_diagnostics(&self) -> Vec<tsox_frontend::ast::Diagnostic> {
        self.0
            .as_ref()
            .get_bind_diagnostics(None)
            .iter()
            .map(|d| (**d).clone())
            .collect()
    }

    pub(crate) fn get_config_file_parsing_diagnostics(&self) -> Vec<tsox_frontend::ast::Diagnostic> {
        self.0
            .as_ref()
            .get_config_file_parsing_diagnostics()
            .iter()
            .map(|d| (**d).clone())
            .collect()
    }
}

impl Session {
    pub fn to_file_change_summary(
        &self,
        changes: Option<&ApiFileChanges>,
    ) -> tsox_lsp::project::file_change::FileChangeSummary {
        let Some(changes) = changes else {
            return Default::default();
        };
        if changes.invalidate_all {
            return tsox_lsp::project::file_change::FileChangeSummary {
                invalidate_all: true,
                includes_watch_change_outside_node_modules: true,
                ..Default::default()
            };
        }
        let cwd = self.project_session.get_current_directory();
        let mut summary = tsox_lsp::project::file_change::FileChangeSummary::default();
        for doc in &changes.changed {
            summary
                .changed
                .insert(tsox_lsp::lsp::lsproto::DocumentUri(doc.to_uri(&cwd)));
        }
        for doc in &changes.created {
            summary
                .created
                .insert(tsox_lsp::lsp::lsproto::DocumentUri(doc.to_uri(&cwd)));
        }
        for doc in &changes.deleted {
            summary
                .deleted
                .insert(tsox_lsp::lsp::lsproto::DocumentUri(doc.to_uri(&cwd)));
        }
        if !summary.changed.is_empty()
            || !summary.created.is_empty()
            || !summary.deleted.is_empty()
        {
            summary.includes_watch_change_outside_node_modules = true;
        }
        summary
    }
}

impl SnapshotData {
    pub fn resolve_type_property_of_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
        getter: impl Fn(&Type) -> Option<Type>,
    ) -> Result<Option<TypeResponse>, String> {
        let t = self.resolve_type_handle(&params.project, params.r#type)?;
        match getter(&t) {
            None => Ok(None),
            Some(result) => Ok(self.new_type_response(&params.project, &result)),
        }
    }
}

impl Session {
    pub fn handle_get_alias_type_arguments_of_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
    ) -> Result<Option<Vec<TypeResponse>>, String> {
        self.resolve_type_array_property_of_type(params, |t| {
            t.alias
                .as_ref()
                .map(|alias| alias.type_arguments.iter().map(|ty| Arc::clone(ty)).collect())
                .unwrap_or_default()
        })
    }

    pub fn handle_get_alias_symbol_of_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
    ) -> Result<Option<SymbolResponse>, String> {
        self.resolve_symbol_property_of_type(params, |t| {
            t.alias.as_ref().and_then(|alias| alias.symbol.clone())
        })
    }

    pub fn handle_get_check_type_of_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> {
        self.resolve_type_property_of_type(params, |t| {
            t.as_conditional_type().and_then(|ct| ct.check_type.clone())
        })
    }

    pub fn handle_get_base_type_of_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> {
        self.resolve_type_property_of_type(params, |t| {
            t.as_substitution_type().and_then(|st| st.base_type.clone())
        })
    }

    pub fn handle_get_constraint_of_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> {
        self.resolve_type_property_of_type(params, |t| {
            t.as_substitution_type().and_then(|st| st.constraint.clone())
        })
    }

    pub fn handle_get_base_type_of_literal_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> {
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        Ok(setup.new_type_response(&setup.checker.get_base_type_of_literal_type(&t)))
    }

    pub fn handle_get_base_types(
        &self,
        params: &CheckerTypeParams,
    ) -> Result<Option<Vec<TypeResponse>>, String> {
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let base_types = setup.checker.get_base_types(&t);
        if base_types.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            base_types
                .iter()
                .filter_map(|bt| setup.new_type_response(bt))
                .collect(),
        ))
    }

    pub fn handle_get_apparent_properties_of_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
    ) -> Result<Vec<SymbolResponse>, String> {
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let props = setup.checker.get_apparent_properties(&t);
        Ok(props
            .iter()
            .filter_map(|prop| setup.new_symbol_response(prop))
            .collect())
    }

    pub fn handle_get_apparent_type(
        &self,
        params: &super::m5l::GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> {
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let apparent = setup.checker.get_apparent_type(&t);
        Ok(setup.new_type_response(&apparent))
    }

    pub fn handle_get_base_constraint_of_type(
        &self,
        params: &CheckerTypeParams,
    ) -> Result<Option<TypeResponse>, String> {
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let constraint = setup.checker.get_base_constraint_of_type(&t);
        match constraint {
            None => Ok(None),
            Some(constraint) => Ok(setup.new_type_response(&constraint)),
        }
    }

    pub fn handle_get_aliased_symbol(
        &self,
        params: &super::m5l::CheckerSymbolParams,
    ) -> Result<Option<SymbolResponse>, String> {
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        Ok(setup.new_symbol_response(&setup.checker.get_aliased_symbol(&symbol)))
    }

    pub fn handle_get_constant_value(
        &self,
        params: &CheckerNodeParams,
    ) -> Result<JsonValue, String> {
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup
            .sd
            .resolve_node_handle(&setup.program, &params.location)?;
        let Some(node) = node else {
            return Ok(JsonValue::Null);
        };
        Ok(match setup.checker.get_constant_value(&node) {
            Some(value) => literal_value_to_json(LiteralValue::String(value)),
            None => JsonValue::Null,
        })
    }

    pub fn handle_get_bind_diagnostics(
        &self,
        params: &super::m5l::GetDiagnosticsParams,
    ) -> Result<Vec<DiagnosticResponse>, String> {
        self.get_diagnostics(params, |p, _file| p.get_bind_diagnostics())
    }

    pub fn handle_get_config_file_parsing_diagnostics(
        &self,
        params: &super::m5l::GetProjectDiagnosticsParams,
    ) -> Result<Vec<DiagnosticResponse>, String> {
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let diags = program.get_config_file_parsing_diagnostics();
        let diags: Vec<Arc<tsox_frontend::ast::Diagnostic>> =
            diags.into_iter().map(Arc::new).collect();
        Ok(new_diagnostic_responses(&diags))
    }
}
