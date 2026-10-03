#![allow(unused_imports, dead_code)]

//! m5k2:session.go 前段 handle* 方法族移植(归属待接线)。

use std::sync::Arc;

use serde_json::Value as JsonValue;

use super::m5l::{
    client_error, parse_project_handle, CpuProfiler, DiagnosticResponse,
    DocumentIdentifier, GetDiagnosticsParams, GetProjectDiagnosticsParams, GetSourceFileParams,
    Program, ProjectId, Session, SignatureId, SnapshotData, SnapshotId, SourceFileResponse,
    SymbolResponse, TypeId, TypeResponse,
};
use super::m5l_3::{base64_decode, base64_encode, EmitOutputResponse};
use super::m5k_3::{literal_value_to_json, new_project_response, project_handle, unmarshal_payload};
use super::m5k_5::RawBinary;
use tsox_frontend::ast::node::Node;
use super::m5l::ProjectSession;

pub struct SessionOptions {
    pub use_binary_responses: bool,
}

static SESSION_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn next_session_id() -> u64 { ::tsox_core::fntrace::enter("next_session_id"); 
    std::sync::atomic::AtomicU64::fetch_add(&SESSION_ID_COUNTER, 1, std::sync::atomic::Ordering::Relaxed) + 1
}

pub fn format_session_id(id: u64) -> String { ::tsox_core::fntrace::enter("format_session_id"); 
    format!("api-session-{}", id)
}

impl Session {
    pub fn new_session(
        project_session: Arc<ProjectSession>,
        options: Option<&SessionOptions>,
    ) -> Session { ::tsox_core::fntrace::enter("new_session"); 
        let id = next_session_id();
        let mut session = Session {
            id: format_session_id(id),
            project_session,
            use_binary_responses: false,
            cpu_profiler: CpuProfiler,
            snapshots: Default::default(),
        };
        if let Some(options) = options {
            session.use_binary_responses = options.use_binary_responses;
        }
        session
    }

    pub fn id(&self) -> String { ::tsox_core::fntrace::enter("id"); 
        self.id.clone()
    }

    pub fn project_session(&self) -> Arc<ProjectSession> { ::tsox_core::fntrace::enter("project_session"); 
        self.project_session.clone()
    }

    pub fn handle_notification(&self, _method: &str, _params: &JsonValue) -> Result<(), String> { ::tsox_core::fntrace::enter("handle_notification"); 
        Ok(())
    }

    pub fn handle_request(&self, method: &str, params: &JsonValue) -> Result<JsonValue, String> { ::tsox_core::fntrace::enter("handle_request"); 
        match method {
            "echo" => {
                if self.use_binary_responses {
                    return serde_json::to_value(RawBinary::from(params.to_string()))
                        .map_err(|e| e.to_string());
                }
                return Ok(params.clone());
            }
            "ping" => return Ok(JsonValue::String("pong".to_string())),
            _ => {}
        }
        let parsed = unmarshal_payload(method, params)
            .map_err(|e| format!("invalid request: {}", e))?;
        match method {
            "batchRequests" => to_response(self.handle_batch_requests(&parse_params(&parsed)?)?),
            "release" => to_response(self.handle_release(&parse_params(&parsed)?)?),
            "initialize" => to_response(self.handle_initialize()?),
            "updateSnapshot" => to_response(self.handle_update_snapshot(&parse_params(&parsed)?)?),
            "updateTemporarySnapshot" => {
                to_response(self.handle_update_temporary_snapshot(&parse_params(&parsed)?)?)
            }
            "parseCommandLine" => {
                to_response(self.handle_parse_command_line(&parse_params(&parsed)?)?)
            }
            "readConfigFile" => to_response(self.handle_read_config_file(&parse_params(&parsed)?)?),
            "parseJsonConfigFileContent" => {
                to_response(self.handle_parse_json_config_file_content(&parse_params(&parsed)?)?)
            }
            "createProgram" => to_response(self.handle_create_program(&parse_params(&parsed)?)?),
            "parseConfigFile" => to_response(self.handle_parse_config_file(&parse_params(&parsed)?)?),
            "transpileModule" => to_response(self.handle_transpile(&parse_params(&parsed)?, false)?),
            "transpileModuleFromFile" => {
                to_response(self.handle_transpile_from_file(&parse_params(&parsed)?, false)?)
            }
            "transpileDeclaration" => {
                to_response(self.handle_transpile(&parse_params(&parsed)?, true)?)
            }
            "transpileDeclarationFromFile" => {
                to_response(self.handle_transpile_from_file(&parse_params(&parsed)?, true)?)
            }
            "getDefaultProjectForFile" => {
                to_response(self.handle_get_default_project_for_file(&parse_params(&parsed)?)?)
            }
            "getSourceFile" => to_response(self.handle_get_source_file(&parse_params(&parsed)?)?),
            "getSourceFileNames" => {
                to_response(self.handle_get_source_file_names(&parse_params(&parsed)?)?)
            }
            "getSourceFileMetadata" => {
                to_response(self.handle_get_source_file_metadata(&parse_params(&parsed)?)?)
            }
            "getConfigFileNames" => {
                to_response(self.handle_get_config_file_names(&parse_params(&parsed)?)?)
            }
            "getConfigSourceFile" => {
                to_response(self.handle_get_config_source_file(&parse_params(&parsed)?)?)
            }
            "getSymbolAtPosition" => {
                to_response(self.handle_get_symbol_at_position(&parse_params(&parsed)?)?)
            }
            "getSymbolsAtPositions" => {
                to_response(self.handle_get_symbols_at_positions(&parse_params(&parsed)?)?)
            }
            "getSymbolAtLocation" => {
                to_response(self.handle_get_symbol_at_location(&parse_params(&parsed)?)?)
            }
            "getSymbolsAtLocations" => {
                to_response(self.handle_get_symbols_at_locations(&parse_params(&parsed)?)?)
            }
            "getSymbolOfSourceFile" => {
                to_response(self.handle_get_symbol_of_source_file(&parse_params(&parsed)?)?)
            }
            "getSymbolsOfSourceFiles" => {
                to_response(self.handle_get_symbols_of_source_files(&parse_params(&parsed)?)?)
            }
            "getTypeOfSymbol" => to_response(self.handle_get_type_of_symbol(&parse_params(&parsed)?)?),
            "getTypesOfSymbols" => {
                to_response(self.handle_get_types_of_symbols(&parse_params(&parsed)?)?)
            }
            "getDeclaredTypeOfSymbol" => {
                to_response(self.handle_get_declared_type_of_symbol(&parse_params(&parsed)?)?)
            }
            "getNonMissingTypeOfSymbol" => {
                to_response(self.handle_get_non_missing_type_of_symbol(&parse_params(&parsed)?)?)
            }
            "resolveName" => to_response(self.handle_resolve_name(&parse_params(&parsed)?)?),
            "getSymbolsInScope" => {
                to_response(self.handle_get_symbols_in_scope(&parse_params(&parsed)?)?)
            }
            "getSignaturesOfType" => {
                to_response(self.handle_get_signatures_of_type(&parse_params(&parsed)?)?)
            }
            "getResolvedSignature" => {
                to_response(self.handle_get_resolved_signature(&parse_params(&parsed)?)?)
            }
            "getTypeAtLocation" => {
                to_response(self.handle_get_type_at_location(&parse_params(&parsed)?)?)
            }
            "getTypeAtLocations" => {
                to_response(self.handle_get_type_at_locations(&parse_params(&parsed)?)?)
            }
            "getTypeAtPosition" => {
                to_response(self.handle_get_type_at_position(&parse_params(&parsed)?)?)
            }
            "getTypesAtPositions" => {
                to_response(self.handle_get_types_at_positions(&parse_params(&parsed)?)?)
            }
            "getParentOfSymbol" => {
                to_response(self.handle_get_parent_of_symbol(&parse_params(&parsed)?)?)
            }
            "getMembersOfSymbol" => {
                to_response(self.handle_get_members_of_symbol(&parse_params(&parsed)?)?)
            }
            "getExportsOfSymbol" => {
                to_response(self.handle_get_exports_of_symbol(&parse_params(&parsed)?)?)
            }
            "getExportSymbolOfSymbol" => {
                to_response(self.handle_get_export_symbol_of_symbol(&parse_params(&parsed)?)?)
            }
            "getSymbolOfType" => to_response(self.handle_get_symbol_of_type(&parse_params(&parsed)?)?),
            "getTargetOfType" => to_response(self.handle_get_target_of_type(&parse_params(&parsed)?)?),
            "getFreshTypeOfType" => {
                to_response(self.handle_get_fresh_type_of_type(&parse_params(&parsed)?)?)
            }
            "getRegularTypeOfType" => {
                to_response(self.handle_get_regular_type_of_type(&parse_params(&parsed)?)?)
            }
            "getTypesOfType" => to_response(self.handle_get_types_of_type(&parse_params(&parsed)?)?),
            "getTypeParametersOfType" => {
                to_response(self.handle_get_type_parameters_of_type(&parse_params(&parsed)?)?)
            }
            "getOuterTypeParametersOfType" => {
                to_response(self.handle_get_outer_type_parameters_of_type(&parse_params(&parsed)?)?)
            }
            "getLocalTypeParametersOfType" => {
                to_response(self.handle_get_local_type_parameters_of_type(&parse_params(&parsed)?)?)
            }
            "getAliasTypeArgumentsOfType" => {
                to_response(self.handle_get_alias_type_arguments_of_type(&parse_params(&parsed)?)?)
            }
            "getAliasSymbolOfType" => {
                to_response(self.handle_get_alias_symbol_of_type(&parse_params(&parsed)?)?)
            }
            "getObjectTypeOfType" => {
                to_response(self.handle_get_object_type_of_type(&parse_params(&parsed)?)?)
            }
            "getIndexTypeOfType" => {
                to_response(self.handle_get_index_type_of_type(&parse_params(&parsed)?)?)
            }
            "getCheckTypeOfType" => {
                to_response(self.handle_get_check_type_of_type(&parse_params(&parsed)?)?)
            }
            "getExtendsTypeOfType" => {
                to_response(self.handle_get_extends_type_of_type(&parse_params(&parsed)?)?)
            }
            "getBaseTypeOfType" => {
                to_response(self.handle_get_base_type_of_type(&parse_params(&parsed)?)?)
            }
            "getConstraintOfType" => {
                to_response(self.handle_get_constraint_of_type(&parse_params(&parsed)?)?)
            }
            "getTrueTypeOfConditionalType" => {
                to_response(self.handle_get_true_type_of_conditional_type(&parse_params(&parsed)?)?)
            }
            "getFalseTypeOfConditionalType" => {
                to_response(self.handle_get_false_type_of_conditional_type(&parse_params(&parsed)?)?)
            }
            "getTypeParametersOfSignature" => {
                to_response(self.handle_get_type_parameters_of_signature(&parse_params(&parsed)?)?)
            }
            "getParametersOfSignature" => {
                to_response(self.handle_get_parameters_of_signature(&parse_params(&parsed)?)?)
            }
            "getThisParameterOfSignature" => {
                to_response(self.handle_get_this_parameter_of_signature(&parse_params(&parsed)?)?)
            }
            "getTargetOfSignature" => {
                to_response(self.handle_get_target_of_signature(&parse_params(&parsed)?)?)
            }
            "getContextualType" => {
                to_response(self.handle_get_contextual_type(&parse_params(&parsed)?)?)
            }
            "getBaseTypeOfLiteralType" => {
                to_response(self.handle_get_base_type_of_literal_type(&parse_params(&parsed)?)?)
            }
            "getNonNullableType" => {
                to_response(self.handle_get_non_nullable_type(&parse_params(&parsed)?)?)
            }
            "getTypeFromTypeNode" => {
                to_response(self.handle_get_type_from_type_node(&parse_params(&parsed)?)?)
            }
            "getWidenedType" => to_response(self.handle_get_widened_type(&parse_params(&parsed)?)?),
            "getParameterType" => to_response(self.handle_get_parameter_type(&parse_params(&parsed)?)?),
            "getTypeParameterAtPosition" => {
                to_response(self.handle_get_type_parameter_at_position(&parse_params(&parsed)?)?)
            }
            "isArrayLikeType" => to_response(self.handle_is_array_like_type(&parse_params(&parsed)?)?),
            "isTypeAssignableTo" => {
                to_response(self.handle_is_type_assignable_to(&parse_params(&parsed)?)?)
            }
            "getShorthandAssignmentValueSymbol" => to_response(
                self.handle_get_shorthand_assignment_value_symbol(&parse_params(&parsed)?)?,
            ),
            "getTypeOfSymbolAtLocation" => {
                to_response(self.handle_get_type_of_symbol_at_location(&parse_params(&parsed)?)?)
            }
            "typeToTypeNode" => to_response(self.handle_type_to_type_node(&parse_params(&parsed)?)?),
            "signatureToSignatureDeclaration" => to_response(
                self.handle_signature_to_signature_declaration(&parse_params(&parsed)?)?,
            ),
            "typeToString" => to_response(self.handle_type_to_string(&parse_params(&parsed)?)?),
            "printNode" => to_response(self.handle_print_node(&parse_params(&parsed)?)?),
            "formatNodeForInsertion" => {
                to_response(self.handle_format_node_for_insertion(&parse_params(&parsed)?)?)
            }
            "emit" => to_response(self.handle_emit(&parse_params(&parsed)?)?),
            "emitToString" => to_response(self.handle_emit_to_string(&parse_params(&parsed)?)?),
            "getJavaScriptEmit" => to_response(
                self.handle_selected_files_emit(
                    &parse_params(&parsed)?,
                    tsox_compile::mig::m4v::EmitOnly::EmitOnlyJs,
                )?,
            ),
            "getDeclarationEmit" => to_response(
                self.handle_selected_files_emit(
                    &parse_params(&parsed)?,
                    tsox_compile::mig::m4v::EmitOnly::EmitOnlyDts,
                )?,
            ),
            "isContextSensitive" => {
                to_response(self.handle_is_context_sensitive(&parse_params(&parsed)?)?)
            }
            "getReturnTypeOfSignature" => {
                to_response(self.handle_get_return_type_of_signature(&parse_params(&parsed)?)?)
            }
            "getRestTypeOfSignature" => {
                to_response(self.handle_get_rest_type_of_signature(&parse_params(&parsed)?)?)
            }
            "getTypePredicateOfSignature" => {
                to_response(self.handle_get_type_predicate_of_signature(&parse_params(&parsed)?)?)
            }
            "getBaseTypes" => to_response(self.handle_get_base_types(&parse_params(&parsed)?)?),
            "getPropertiesOfType" => {
                to_response(self.handle_get_properties_of_type(&parse_params(&parsed)?)?)
            }
            "getApparentPropertiesOfType" => {
                to_response(self.handle_get_apparent_properties_of_type(&parse_params(&parsed)?)?)
            }
            "getApparentType" => {
                to_response(self.handle_get_apparent_type(&parse_params(&parsed)?)?)
            }
            "getReducedType" => to_response(self.handle_get_reduced_type(&parse_params(&parsed)?)?),
            "getPropertyOfType" => {
                to_response(self.handle_get_property_of_type(&parse_params(&parsed)?)?)
            }
            "getIndexInfosOfType" => {
                to_response(self.handle_get_index_infos_of_type(&parse_params(&parsed)?)?)
            }
            "getConstraintOfTypeParameter" => {
                to_response(self.handle_get_constraint_of_type_parameter(&parse_params(&parsed)?)?)
            }
            "getBaseConstraintOfType" => {
                to_response(self.handle_get_base_constraint_of_type(&parse_params(&parsed)?)?)
            }
            "getDefaultFromTypeParameter" => {
                to_response(self.handle_get_default_from_type_parameter(&parse_params(&parsed)?)?)
            }
            "getTypeArguments" => to_response(self.handle_get_type_arguments(&parse_params(&parsed)?)?),
            "getImportAdderEdits" => {
                to_response(self.handle_get_import_adder_edits(&parse_params(&parsed)?)?)
            }
            "getConstantValue" => {
                to_response(self.handle_get_constant_value(&parse_params(&parsed)?)?)
            }
            "getSignatureFromDeclaration" => {
                to_response(self.handle_get_signature_from_declaration(&parse_params(&parsed)?)?)
            }
            "getExportSpecifierLocalTarget" => to_response(
                self.handle_get_export_specifier_local_target_symbol(&parse_params(&parsed)?)?,
            ),
            "getAliasedSymbol" => {
                to_response(self.handle_get_aliased_symbol(&parse_params(&parsed)?)?)
            }
            "getImmediateAliasedSymbol" => {
                to_response(self.handle_get_immediate_aliased_symbol(&parse_params(&parsed)?)?)
            }
            "getTargetSymbol" => {
                to_response(self.handle_method_get_target_symbol(&parse_params(&parsed)?)?)
            }
            "getFullyQualifiedName" => {
                to_response(self.handle_get_fully_qualified_name(&parse_params(&parsed)?)?)
            }
            "getExportsOfModule" => {
                to_response(self.handle_get_exports_of_module(&parse_params(&parsed)?)?)
            }
            "getMemberInModuleExports" => {
                to_response(self.handle_get_member_in_module_exports(&parse_params(&parsed)?)?)
            }
            "getJSDocTags" => to_response(self.handle_get_jsdoc_tags(&parse_params(&parsed)?)?),
            "getDocumentationComment" => {
                to_response(self.handle_get_documentation_comment(&parse_params(&parsed)?)?)
            }
            "isArrayType" => to_response(self.handle_is_array_type(&parse_params(&parsed)?)?),
            "isReadonlySymbol" => {
                to_response(self.handle_is_readonly_symbol(&parse_params(&parsed)?)?)
            }
            "getAnyType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_any_type())
                })?,
            ),
            "getStringType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_string_type())
                })?,
            ),
            "getNumberType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_number_type())
                })?,
            ),
            "getBooleanType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_boolean_type())
                })?,
            ),
            "getVoidType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_void_type())
                })?,
            ),
            "getUndefinedType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_undefined_type())
                })?,
            ),
            "getNullType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_null_type())
                })?,
            ),
            "getNeverType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_never_type())
                })?,
            ),
            "getUnknownType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_unknown_type())
                })?,
            ),
            "getBigIntType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_bigint_type())
                })?,
            ),
            "getESSymbolType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.get_es_symbol_type())
                })?,
            ),
            "getNonPrimitiveType" => to_response(
                self.handle_get_intrinsic_type(&parse_params(&parsed)?, |c: &tsox_checker::checker::Checker| {
                    Some(c.non_primitive_type())
                })?,
            ),
            "getWellKnownSymbols" => {
                to_response(self.handle_get_well_known_symbols(&parse_params(&parsed)?)?)
            }
            "getWellKnownSignatures" => {
                to_response(self.handle_get_well_known_signatures(&parse_params(&parsed)?)?)
            }
            "getSyntacticDiagnostics" => {
                to_response(self.handle_get_syntactic_diagnostics(&parse_params(&parsed)?)?)
            }
            "getBindDiagnostics" => {
                to_response(self.handle_get_bind_diagnostics(&parse_params(&parsed)?)?)
            }
            "getSemanticDiagnostics" => {
                to_response(self.handle_get_semantic_diagnostics(&parse_params(&parsed)?)?)
            }
            "getSuggestionDiagnostics" => {
                to_response(self.handle_get_suggestion_diagnostics(&parse_params(&parsed)?)?)
            }
            "getDeclarationDiagnostics" => {
                to_response(self.handle_get_declaration_diagnostics(&parse_params(&parsed)?)?)
            }
            "getProgramDiagnostics" => {
                to_response(self.handle_get_program_diagnostics(&parse_params(&parsed)?)?)
            }
            "getGlobalDiagnostics" => {
                to_response(self.handle_get_global_diagnostics(&parse_params(&parsed)?)?)
            }
            "getConfigFileParsingDiagnostics" => to_response(
                self.handle_get_config_file_parsing_diagnostics(&parse_params(&parsed)?)?,
            ),
            "startCPUProfile" => {
                self.handle_start_cpu_profile(&parse_params(&parsed)?)?;
                to_response(JsonValue::Null)
            }
            "stopCPUProfile" => to_response(self.handle_stop_cpu_profile()?),
            "saveHeapProfile" => {
                let result = self.handle_save_heap_profile(&parse_params(&parsed)?)?;
                to_response(serde_json::json!({ "file": result.file }))
            }
            "getReferencesToSymbolInFile" => to_response(
                self.handle_get_references_to_symbol_in_file(&parse_params(&parsed)?)?,
            ),
            "getReferencedSymbolsForNode" => {
                to_response(self.handle_get_referenced_symbols_for_node(&parse_params(&parsed)?)?)
            }
            "getSignatureUsages" => {
                to_response(self.handle_get_signature_usages(&parse_params(&parsed)?)?)
            }
            "getCompletionsAtPosition" => {
                to_response(self.handle_get_completions_at_position(&parse_params(&parsed)?)?)
            }
            _ => Err(format!("unknown method: {}", method)),
        }
    }
}

use super::m5k2_2::ApiFileChanges;
use tsox_lsp::project::snapshot::Snapshot as ProjectSnapshot;

#[derive(serde::Deserialize)]
pub struct UpdateSnapshotParams {
    pub file_changes: Option<ApiFileChanges>,
    pub open_projects: Vec<DocumentIdentifier>,
    pub close_projects: Vec<DocumentIdentifier>,
    pub open_files: Vec<DocumentIdentifier>,
    pub close_files: Vec<DocumentIdentifier>,
}

#[derive(serde::Deserialize)]
pub struct UpdateTemporarySnapshotParams {
    pub snapshot: SnapshotId,
    pub file: DocumentIdentifier,
    pub new_text: String,
}

#[derive(serde::Serialize)]
pub struct UpdateSnapshotResponse {
    pub snapshot: SnapshotId,
    pub projects: Vec<super::m5k_3::ProjectResponse>,
    pub changes: Option<SnapshotChanges>,
}

#[derive(Clone, serde::Deserialize, Default)]
pub struct TranspileRequestOptions {
    #[serde(default)]
    pub compiler_options: tsox_core::core::compiler_options::CompilerOptions,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub report_diagnostics: bool,
}

#[derive(serde::Deserialize)]
pub struct TranspileParams {
    pub input: String,
    #[serde(default)]
    pub options: TranspileRequestOptions,
}

#[derive(serde::Deserialize)]
pub struct TranspileFromFileParams {
    pub file_name: String,
    #[serde(default)]
    pub options: TranspileRequestOptions,
}

#[derive(serde::Serialize)]
pub struct TranspileOutputResponse {
    pub output_text: String,
    pub diagnostics: Vec<DiagnosticResponse>,
    pub source_map_text: String,
}

#[derive(serde::Deserialize)]
pub struct TypeToTypeNodeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub r#type: TypeId,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub flags: u32,
}

#[derive(Default)]
struct UpdateTrackingState {
    open_projects: std::collections::HashSet<String>,
    open_files: std::collections::HashSet<String>,
    latest_snapshot: SnapshotId,
}

fn take_update_state(session_id: &str) -> UpdateTrackingState { ::tsox_core::fntrace::enter("take_update_state"); 
    static STATES: std::sync::Mutex<
        Option<std::collections::HashMap<String, UpdateTrackingState>>,
    > = std::sync::Mutex::new(None);
    let mut guard = STATES.lock().unwrap();
    guard
        .as_mut()
        .and_then(|states| states.remove(session_id))
        .unwrap_or_default()
}

fn put_update_state(session_id: &str, state: UpdateTrackingState) { ::tsox_core::fntrace::enter("put_update_state"); 
    static STATES: std::sync::Mutex<
        Option<std::collections::HashMap<String, UpdateTrackingState>>,
    > = std::sync::Mutex::new(None);
    let mut guard = STATES.lock().unwrap();
    guard
        .get_or_insert_with(Default::default)
        .insert(session_id.to_string(), state);
}

fn snapshot_handle(snapshot: &ProjectSnapshot) -> SnapshotId { ::tsox_core::fntrace::enter("snapshot_handle"); 
    snapshot.id()
}

impl Session {
    pub fn handle_update_snapshot(
        &self,
        params: &UpdateSnapshotParams,
    ) -> Result<UpdateSnapshotResponse, String> { ::tsox_core::fntrace::enter("handle_update_snapshot"); 
        let cwd = self.project_session.get_current_directory();
        let file_changes = self.to_file_change_summary(params.file_changes.as_ref());
        let mut api_request = tsox_lsp::project::snapshot::APISnapshotRequest::default();
        let mut state = take_update_state(&self.id);

        let mut opened_projects = Vec::new();
        for p in &params.open_projects {
            let config_file_name = p.to_absolute_file_name(&cwd);
            let config_path = self.to_path(&config_file_name);
            if state.open_projects.contains(&config_path) {
                continue;
            }
            api_request
                .open_projects
                .get_or_insert_with(Default::default)
                .insert(config_file_name.clone());
            opened_projects.push(config_path);
        }

        let mut closed_projects = Vec::new();
        for p in &params.close_projects {
            let config_path = self.to_path(&p.to_absolute_file_name(&cwd));
            if !state.open_projects.contains(&config_path) {
                continue;
            }
            api_request
                .close_projects
                .get_or_insert_with(Default::default)
                .insert(tsox_core::tspath::Path(config_path.clone()));
            closed_projects.push(config_path);
        }

        let mut opened_files = Vec::new();
        for f in &params.open_files {
            let uri = f.to_uri(&cwd);
            let path = self.to_path(&uri);
            if state.open_files.contains(&path) {
                continue;
            }
            api_request
                .open_files
                .get_or_insert_with(Default::default)
                .insert(tsox_lsp::lsp::lsproto::DocumentUri(uri.clone()));
            opened_files.push(path);
        }

        let mut closed_files = Vec::new();
        for f in &params.close_files {
            let path = self.to_path(&f.to_uri(&cwd));
            if !state.open_files.contains(&path) {
                continue;
            }
            api_request
                .close_files
                .get_or_insert_with(Default::default)
                .insert(tsox_core::tspath::Path(path.clone()));
            closed_files.push(path);
        }

        let snapshot = Arc::new(
            *self
                .project_session
                .api_update(&file_changes, &api_request)
                .map_err(|e| client_error(format!("failed to update snapshot: {}", e)))?,
        );

        for config_path in opened_projects {
            state.open_projects.insert(config_path);
        }
        for config_path in closed_projects {
            state.open_projects.remove(&config_path);
        }
        for path in opened_files {
            state.open_files.insert(path);
        }
        for path in closed_files {
            state.open_files.remove(&path);
        }

        let handle = snapshot_handle(&snapshot);
        let prev_snapshot = self
            .snapshots
            .lock()
            .unwrap()
            .get(&state.latest_snapshot)
            .map(|sd| sd.snapshot.clone());

        let projects = snapshot
            .project_collection
            .as_ref()
            .map(|c| c.projects())
            .unwrap_or_default();
        let mut project_responses = Vec::with_capacity(projects.len());
        for proj in projects {
            if proj.command_line.is_none() {
                continue;
            }
            project_responses.push(super::m5k_3::new_project_response(Some(proj)));
        }

        let changes = prev_snapshot
            .map(|prev| compute_snapshot_changes(&prev, &snapshot));

        {
            let mut snapshots = self.snapshots.lock().unwrap();
            match snapshots.get_mut(&handle) {
                Some(sd) => sd.ref_count += 1,
                None => {
                    snapshots.insert(
                        handle,
                        SnapshotData {
                            snapshot: snapshot.clone(),
                            ref_count: 1,
                            project_registries: Default::default(),
                        },
                    );
                }
            }
        }
        state.latest_snapshot = handle;
        put_update_state(&self.id, state);

        Ok(UpdateSnapshotResponse {
            snapshot: handle,
            projects: project_responses,
            changes,
        })
    }

    pub fn handle_update_temporary_snapshot(
        &self,
        params: &UpdateTemporarySnapshotParams,
    ) -> Result<UpdateSnapshotResponse, String> { ::tsox_core::fntrace::enter("handle_update_temporary_snapshot"); 
        let base_snapshot = self.get_snapshot_data(params.snapshot)?.snapshot;
        let uri = tsox_lsp::lsp::lsproto::DocumentUri(
            params
                .file
                .to_uri(&self.project_session.get_current_directory()),
        );
        let snapshot = self
            .project_session
            .api_update_temporary(&base_snapshot, uri, &params.new_text)
            .map_err(|e| client_error(format!("failed to update temporary snapshot: {}", e)))?;

        let handle = snapshot_handle(&snapshot);
        {
            let mut snapshots = self.snapshots.lock().unwrap();
            match snapshots.get_mut(&handle) {
                Some(sd) => sd.ref_count += 1,
                None => {
                    snapshots.insert(
                        handle,
                        SnapshotData {
                            snapshot: snapshot.clone(),
                            ref_count: 1,
                            project_registries: Default::default(),
                        },
                    );
                }
            }
        }

        let projects = snapshot
            .project_collection
            .as_ref()
            .map(|c| c.projects())
            .unwrap_or_default();
        let mut project_responses = Vec::with_capacity(projects.len());
        for proj in projects {
            if proj.command_line.is_none() {
                continue;
            }
            project_responses.push(super::m5k_3::new_project_response(Some(proj)));
        }

        Ok(UpdateSnapshotResponse {
            snapshot: handle,
            projects: project_responses,
            changes: Some(compute_snapshot_changes(&base_snapshot, &snapshot)),
        })
    }

    pub fn handle_transpile(
        &self,
        params: &TranspileParams,
        declaration: bool,
    ) -> Result<TranspileOutputResponse, String> { ::tsox_core::fntrace::enter("handle_transpile"); 
        transpile_output_response(&params.input, params.options.clone(), declaration)
    }

    pub fn handle_transpile_from_file(
        &self,
        params: &TranspileFromFileParams,
        declaration: bool,
    ) -> Result<TranspileOutputResponse, String> { ::tsox_core::fntrace::enter("handle_transpile_from_file"); 
        let file_name = tsox_core::tspath::get_normalized_absolute_path(
            &params.file_name,
            &self.project_session.get_current_directory(),
        );
        let input = self
            .project_session
            .fs()
            .expect("project session fs")
            .read_file(&file_name)
            .ok_or_else(|| client_error(format!("could not read file {}", file_name)))?;
        let mut options = params.options.clone();
        options.file_name = file_name;
        transpile_output_response(&input, options, declaration)
    }

    pub fn handle_type_to_type_node(
        &self,
        params: &TypeToTypeNodeParams,
    ) -> Result<Option<SourceFileResponse>, String> { ::tsox_core::fntrace::enter("handle_type_to_type_node"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let _ = &params.location;
        let type_node = setup.checker.type_to_type_node(&t);
        let (data, _) = super::m5j_encoder::encode_tree(&type_node, None)
            .map_err(|e| format!("failed to encode type node: {}", e))?;
        if self.use_binary_responses {
            return Ok(Some(SourceFileResponse {
                data: base64_encode(&data),
            }));
        }
        Ok(Some(SourceFileResponse {
            data: base64_encode(&data),
        }))
    }

    pub fn handle_type_to_string(
        &self,
        params: &TypeToTypeNodeParams,
    ) -> Result<String, String> { ::tsox_core::fntrace::enter("handle_type_to_string"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let _ = &params.location;
        if params.flags != 0 {
            return Ok(setup.checker.type_to_string_ex(
                &t,
                tsox_checker::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE,
            ));
        }
        Ok(setup.checker.type_to_string_ex(
            &t,
            tsox_checker::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE,
        ))
    }
}

fn transpile_output_response(
    input: &str,
    options: TranspileRequestOptions,
    declaration: bool,
) -> Result<TranspileOutputResponse, String> { ::tsox_core::fntrace::enter("transpile_output_response"); 
    let transpile_options = tsox_compile::transpile::TranspileOptions {
        compiler_options: options.compiler_options,
        file_name: options.file_name,
        report_diagnostics: options.report_diagnostics,
    };
    let output = if declaration {
        tsox_compile::transpile::transpile_declaration(input, transpile_options)
    } else {
        tsox_compile::transpile::transpile_module(input, transpile_options)
    };
    Ok(TranspileOutputResponse {
        output_text: output.output_text,
        diagnostics: super::m5k_4::new_diagnostic_responses(&output.diagnostics)
            .unwrap_or_default(),
        source_map_text: output.source_map_text,
    })
}

fn to_response<T: serde::Serialize>(value: T) -> Result<JsonValue, String> { ::tsox_core::fntrace::enter("to_response"); 
    serde_json::to_value(value).map_err(|e| e.to_string())
}

fn parse_params<T: serde::de::DeserializeOwned>(parsed: &JsonValue) -> Result<T, String> { ::tsox_core::fntrace::enter("parse_params"); 
    serde_json::from_value(parsed.clone()).map_err(|e| format!("invalid request: {}", e))
}

#[derive(serde::Deserialize)]
pub struct BatchRequestsParams {
    pub requests: Vec<BatchRequest>,
}

#[derive(serde::Deserialize)]
pub struct BatchRequest {
    pub method: String,
    pub params: JsonValue,
}

#[derive(serde::Serialize)]
pub struct BatchRequestsResponse {
    pub responses: Vec<BatchResponse>,
}

#[derive(serde::Serialize)]
pub struct BatchResponse {
    pub method: String,
    pub result: JsonValue,
    pub error: String,
}

impl Session {
    pub fn handle_batch_requests(
        &self,
        params: &BatchRequestsParams,
    ) -> Result<BatchRequestsResponse, String> { ::tsox_core::fntrace::enter("handle_batch_requests"); 
        let responses = params
            .requests
            .iter()
            .map(|request| self.handle_batch_request(request))
            .collect();
        Ok(BatchRequestsResponse { responses })
    }

    pub fn handle_batch_request(&self, request: &BatchRequest) -> BatchResponse { ::tsox_core::fntrace::enter("handle_batch_request"); 
        let mut response = BatchResponse {
            method: request.method.clone(),
            result: JsonValue::Null,
            error: String::new(),
        };
        match self.handle_request(&request.method, &request.params) {
            Ok(result) => response.result = result,
            Err(err) => response.error = err,
        }
        response
    }
}

impl SnapshotData {
    pub fn get_project(
        &self,
        project_handle: &ProjectId,
    ) -> Result<tsox_lsp::project::project::Project, String> { ::tsox_core::fntrace::enter("get_project"); 
        let project_name = parse_project_handle(project_handle);
        let collection = self
            .snapshot
            .project_collection
            .as_deref()
            .ok_or_else(|| client_error(format!("project {} not found", project_name)))?;
        let proj = collection
            .get_project_by_path(&project_name)
            .ok_or_else(|| client_error(format!("project {} not found", project_name)))?;
        Ok(proj.clone_shallow())
    }

    pub fn get_or_create_project_registry(&self, project_id: &ProjectId) -> ProjectRegistryData { ::tsox_core::fntrace::enter("get_or_create_project_registry"); 
        if project_id.is_empty() {
            panic!("getOrCreateProjectRegistry: empty project ID");
        }
        let mut registries = self.project_registries.lock().unwrap();
        registries
            .entry(project_id.clone())
            .or_insert_with(|| ProjectRegistryData {
                type_registry: Default::default(),
                signature_registry: Default::default(),
            })
            .clone()
    }
}

#[derive(Clone)]
pub struct ProjectRegistryData {
    pub type_registry: std::collections::HashMap<TypeId, Arc<tsox_checker::checker::Type>>,
    pub signature_registry:
        std::collections::HashMap<SignatureId, Arc<tsox_checker::checker::Signature>>,
}

#[derive(Default, serde::Serialize)]
pub struct ProjectFileChanges {
    pub changed_files: Vec<String>,
    pub deleted_files: Vec<String>,
}

#[derive(Default, serde::Serialize)]
pub struct SnapshotChanges {
    pub changed_projects: std::collections::BTreeMap<ProjectId, ProjectFileChanges>,
    pub removed_projects: Vec<ProjectId>,
}

pub fn compute_snapshot_changes(
    prev: &tsox_lsp::project::snapshot::Snapshot,
    next: &tsox_lsp::project::snapshot::Snapshot,
) -> SnapshotChanges { ::tsox_core::fntrace::enter("compute_snapshot_changes"); 
    let changes = std::cell::RefCell::new(SnapshotChanges::default());
    let prev_projects = prev
        .project_collection
        .as_deref()
        .map(|c| c.projects_by_path())
        .unwrap_or_default();
    let next_projects = next
        .project_collection
        .as_deref()
        .map(|c| c.projects_by_path())
        .unwrap_or_default();
    let mut prev_map = tsox_core::collections::ordered_map::OrderedMap::new();
    for (path, proj) in prev_projects {
        prev_map.insert(path, proj);
    }
    let mut next_map = tsox_core::collections::ordered_map::OrderedMap::new();
    for (path, proj) in next_projects {
        next_map.insert(path, proj);
    }
    tsox_core::collections::ordered_map::diff_ordered_maps(
        &prev_map,
        &next_map,
        |a: &&tsox_lsp::project::project::Project,
         b: &&tsox_lsp::project::project::Project|
         -> bool { std::ptr::eq(*a, *b) },
        |_path, _new_proj| {},
        |_path, old_proj: &&tsox_lsp::project::project::Project| {
            changes
                .borrow_mut()
                .removed_projects
                .push(project_handle(*old_proj));
        },
        |_path,
         old_proj: &&tsox_lsp::project::project::Project,
         new_proj: &&tsox_lsp::project::project::Project| {
            let same_program = match (old_proj.get_program(), new_proj.get_program()) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            };
            if same_program {
                return;
            }
            let project_changes = diff_project_files(*old_proj, *new_proj);
            if !project_changes.changed_files.is_empty() || !project_changes.deleted_files.is_empty()
            {
                changes
                    .borrow_mut()
                    .changed_projects
                    .insert(project_handle(*new_proj), project_changes);
            }
        },
    );
    changes.into_inner()
}

fn diff_project_files(
    old_proj: &tsox_lsp::project::project::Project,
    new_proj: &tsox_lsp::project::project::Project,
) -> ProjectFileChanges { ::tsox_core::fntrace::enter("diff_project_files"); 
    let _ = (old_proj, new_proj);
    ProjectFileChanges {
        changed_files: Vec::new(),
        deleted_files: Vec::new(),
    }
}
