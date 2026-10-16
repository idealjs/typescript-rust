#![allow(unused_imports, dead_code)]

//! m5l: session.go handle* 方法族批量移植(后 100 个),归属待接线。

use std::sync::Arc;

pub use super::m5k_3::{
    ConfigFileResponse, DocumentIdentifier, ProjectId, ProjectResponse, SignatureId, SymbolId,
    TypeId,
};
pub use super::m5k_4::TypeResponse;
pub use tsox_lsp::project::session::Session as ProjectSession;

use tsox_checker::checker::{Checker, ContextFlags, Signature, Type};
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::Symbol;
use super::m5l_3::base64_encode;
use super::m5m::to_api_text_edits;

pub type SnapshotId = u64;
pub type NodeHandle = String;

const CONTEXT_FLAGS_NONE: ContextFlags = ContextFlags::None;

pub struct Session {
    pub id: String,
    pub use_binary_responses: bool,
    pub project_session: Arc<ProjectSession>,
    pub snapshots: std::sync::Mutex<std::collections::HashMap<SnapshotId, SnapshotData>>,
    pub cpu_profiler: CpuProfiler,
}

pub struct CpuProfiler;

impl CpuProfiler {
    pub fn start_cpu_profile(&self, _dir: &str) -> Result<(), String> { ::tsox_core::fntrace::enter("start_cpu_profile"); 
        unimplemented!()
    }
    pub fn stop_cpu_profile(&self) -> Result<String, String> { ::tsox_core::fntrace::enter("stop_cpu_profile"); 
        unimplemented!()
    }
}

impl Session {
    pub fn get_snapshot_data(&self, handle: SnapshotId) -> Result<SnapshotData, String> { ::tsox_core::fntrace::enter("get_snapshot_data"); 
        session_get_snapshot_data(self, handle)
    }

    pub fn setup_checker(
        &self,
        snapshot: SnapshotId,
        project: &ProjectId,
    ) -> Result<CheckerSetup, String> { ::tsox_core::fntrace::enter("setup_checker"); 
        session_setup_checker(self, snapshot, project)
    }
}

fn session_get_snapshot_data(_s: &Session, _handle: SnapshotId) -> Result<SnapshotData, String> { ::tsox_core::fntrace::enter("session_get_snapshot_data"); 
    unimplemented!()
}

fn session_setup_checker(
    _s: &Session,
    _snapshot: SnapshotId,
    _project: &ProjectId,
) -> Result<CheckerSetup, String> { ::tsox_core::fntrace::enter("session_setup_checker"); 
    unimplemented!()
}

pub struct SnapshotData {
    pub snapshot: Arc<tsox_lsp::project::snapshot::Snapshot>,
    pub ref_count: i32,
    pub project_registries:
        std::sync::Mutex<std::collections::HashMap<ProjectId, super::m5k2::ProjectRegistryData>>,
}

impl Default for SnapshotData {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        SnapshotData {
            snapshot: Arc::new(tsox_lsp::project::snapshot::Snapshot::new(0)),
            ref_count: 0,
            project_registries: Default::default(),
        }
    }
}

impl SnapshotData {
    pub fn get_program(&self, _project: &ProjectId) -> Result<Program, String> { ::tsox_core::fntrace::enter("get_program"); 
        unimplemented!()
    }
    pub fn resolve_node_handle(
        &self,
        _program: &Program,
        _handle: &NodeHandle,
    ) -> Result<Option<Arc<Node>>, String> { ::tsox_core::fntrace::enter("resolve_node_handle"); 
        unimplemented!()
    }
    pub fn resolve_symbol_handle(&self, _handle: SymbolId) -> Result<Arc<Symbol>, String> { ::tsox_core::fntrace::enter("resolve_symbol_handle"); 
        unimplemented!()
    }
    pub fn resolve_type_handle(
        &self,
        _project: &ProjectId,
        _handle: TypeId,
    ) -> Result<Arc<Type>, String> { ::tsox_core::fntrace::enter("resolve_type_handle"); 
        unimplemented!()
    }
    pub fn resolve_signature_handle(
        &self,
        _project: &ProjectId,
        _handle: SignatureId,
    ) -> Result<Arc<Signature>, String> { ::tsox_core::fntrace::enter("resolve_signature_handle"); 
        unimplemented!()
    }
    pub fn new_symbol_response(
        &self,
        _symbol: &Symbol,
        _project: &ProjectId,
    ) -> Option<SymbolResponse> { ::tsox_core::fntrace::enter("new_symbol_response"); 
        unimplemented!()
    }
    pub fn new_type_response(&self, _project: &ProjectId, _t: &Type) -> Option<TypeResponse> { ::tsox_core::fntrace::enter("new_type_response"); 
        unimplemented!()
    }
    pub fn new_signature_response(
        &self,
        _project: &ProjectId,
        _sig: &Signature,
    ) -> Option<SignatureResponse> { ::tsox_core::fntrace::enter("new_signature_response"); 
        unimplemented!()
    }
    pub fn register_symbol(&self, _symbol: &Symbol, _project: &ProjectId) -> SymbolId { ::tsox_core::fntrace::enter("register_symbol"); 
        unimplemented!()
    }
    pub fn register_signature(&self, _project: &ProjectId, _sig: &Signature) -> SignatureId { ::tsox_core::fntrace::enter("register_signature"); 
        unimplemented!()
    }
    pub fn node_handle_from(&self, _node: &Node) -> NodeHandle { ::tsox_core::fntrace::enter("node_handle_from"); 
        unimplemented!()
    }
}

pub struct Program(pub Arc<tsox_compile::compiler::Program>);

impl Program {
    pub fn get_source_file(&self, file_name: &str) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        self.0.get_source_file(file_name)
    }

    pub fn get_source_files(&self) -> Vec<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_files"); 
        self.0.get_source_files()
    }

    pub fn get_program_diagnostics(&self) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> { ::tsox_core::fntrace::enter("get_program_diagnostics"); 
        self.0.get_program_diagnostics()
    }

    pub fn get_semantic_diagnostics(
        &self,
        _ctx: &mut tsox_core::core::mig::m3j_3::RequestContext,
        source_file: Option<&Arc<SourceFile>>,
    ) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> { ::tsox_core::fntrace::enter("get_semantic_diagnostics"); 
        let diags = self.0.get_semantic_diagnostics();
        match source_file {
            None => diags.into_iter().map(Arc::new).collect(),
            Some(sf) => diags
                .into_iter()
                .map(Arc::new)
                .filter(|d| {
                    d.file
                        .as_ref()
                        .map(|f| f.file_name == sf.file_name)
                        .unwrap_or(false)
                })
                .collect(),
        }
    }

    pub fn get_syntactic_diagnostics(
        &self,
        _ctx: &mut tsox_core::core::mig::m3j_3::RequestContext,
        source_file: Option<&Arc<SourceFile>>,
    ) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> { ::tsox_core::fntrace::enter("get_syntactic_diagnostics"); 
        self.0.get_syntactic_diagnostics(source_file)
    }

    pub fn get_suggestion_diagnostics(
        &self,
        _ctx: &mut tsox_core::core::mig::m3j_3::RequestContext,
        source_file: Option<&Arc<SourceFile>>,
    ) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> { ::tsox_core::fntrace::enter("get_suggestion_diagnostics"); 
        self.0.get_suggestion_diagnostics(source_file)
    }

    pub fn get_source_file_meta_data(
        &self,
        path: &str,
    ) -> tsox_frontend::ast::mig::x4ast::SourceFileMetaData { ::tsox_core::fntrace::enter("get_source_file_meta_data"); 
        self.0
            .get_source_file_meta_data(path)
            .unwrap_or_default()
    }

    pub fn is_source_file_default_library(&self, file_name: &str) -> bool { ::tsox_core::fntrace::enter("is_source_file_default_library"); 
        self.0.is_source_file_default_library(file_name)
    }

    pub fn is_source_file_from_external_library(&self, file: &Arc<SourceFile>) -> bool { ::tsox_core::fntrace::enter("is_source_file_from_external_library"); 
        self.0.is_source_file_from_external_library(file)
    }
pub fn get_declaration_diagnostics(
    &self,
    _file: Option<&Arc<SourceFile>>,
) -> Vec<tsox_frontend::ast::Diagnostic> { ::tsox_core::fntrace::enter("get_declaration_diagnostics"); 
    unimplemented!()
}
}

pub struct CheckerSetup {
    pub sd: SnapshotData,
    pub program: Program,
    pub checker: Checker,
    pub project_id: ProjectId,
}

impl CheckerSetup {
    pub fn done(&self) { ::tsox_core::fntrace::enter("done"); }
    pub fn new_type_response(&self, t: &Type) -> Option<TypeResponse> { ::tsox_core::fntrace::enter("new_type_response"); 
        self.sd.new_type_response(&self.project_id, t)
    }
    pub fn new_symbol_response(&self, sym: &Symbol) -> Option<SymbolResponse> { ::tsox_core::fntrace::enter("new_symbol_response"); 
        self.sd.new_symbol_response(sym, &self.project_id)
    }
    pub fn new_signature_response(&self, sig: &Signature) -> Option<SignatureResponse> { ::tsox_core::fntrace::enter("new_signature_response"); 
        self.sd.new_signature_response(&self.project_id, sig)
    }
    pub fn resolve_type_handle(&self, id: TypeId) -> Result<Arc<Type>, String> { ::tsox_core::fntrace::enter("resolve_type_handle"); 
        self.sd.resolve_type_handle(&self.project_id, id)
    }
    pub fn resolve_symbol_handle(&self, id: SymbolId) -> Result<Arc<Symbol>, String> { ::tsox_core::fntrace::enter("resolve_symbol_handle"); 
        self.sd.resolve_symbol_handle(id)
    }
    pub fn resolve_signature_handle(&self, id: SignatureId) -> Result<Arc<Signature>, String> { ::tsox_core::fntrace::enter("resolve_signature_handle"); 
        self.sd.resolve_signature_handle(&self.project_id, id)
    }

    pub fn resolve_location(
        &self,
        handle: &NodeHandle,
        file: Option<&DocumentIdentifier>,
        position: Option<u32>,
    ) -> Result<Option<Arc<Node>>, String> { ::tsox_core::fntrace::enter("resolve_location"); 
        if !handle.is_empty() {
            return self.sd.resolve_node_handle(&self.program, handle);
        }
        if let (Some(file), Some(position)) = (file, position) {
            let source_file = self
                .program
                .get_source_file(&file.to_file_name())
                .ok_or_else(|| client_error(format!("source file not found: {}", file)))?;
            let position_map =
                tsox_frontend::ast::positionmap::compute_position_map(&source_file.text);
            return Ok(tsox_frontend::astnav::get_touching_property_name(
                &source_file.node,
                position_map.utf16_to_utf8(position as usize),
            ));
        }
        Ok(None)
    }
}

pub(crate) fn client_error(msg: impl std::fmt::Display) -> String { ::tsox_core::fntrace::enter("client_error"); 
    format!("client error: {}", msg)
}

impl Session {
    pub fn handle_get_constraint_of_type_parameter(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_constraint_of_type_parameter"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let constraint = setup.checker.get_constraint_of_type_parameter(&t);
        match constraint {
            None => Ok(None),
            Some(constraint) => Ok(setup.new_type_response(&constraint)),
        }
    }

    pub fn handle_get_contextual_type(
        &self,
        params: &GetContextualTypeParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_contextual_type"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup.sd.resolve_node_handle(&setup.program, &params.location)?;
        let Some(node) = node else { return Ok(None) };
        let t = setup
            .checker
            .get_contextual_type(&node, CONTEXT_FLAGS_NONE);
        match t {
            None => Ok(None),
            Some(t) => Ok(setup.new_type_response(&t)),
        }
    }

    pub fn handle_get_declaration_diagnostics(
        &self,
        params: &GetDiagnosticsParams,
    ) -> Result<Vec<DiagnosticResponse>, String> { ::tsox_core::fntrace::enter("handle_get_declaration_diagnostics"); 
        self.get_diagnostics(params, |p, file| p.get_declaration_diagnostics(file))
    }

    pub fn handle_get_declared_type_of_symbol(
        &self,
        params: &GetTypeOfSymbolParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_declared_type_of_symbol"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        let declared = setup.checker.get_declared_type_of_symbol(&symbol);
        Ok(setup.new_type_response(&declared))
    }

    pub fn handle_get_default_from_type_parameter(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_default_from_type_parameter"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let default_type = match setup.checker.get_default_from_type_parameter(&t) {
            Some(default_type) => default_type,
            None => return Ok(None),
        };
        Ok(setup.new_type_response(&default_type))
    }

    pub fn handle_get_default_project_for_file(
        &self,
        params: &GetDefaultProjectForFileParams,
    ) -> Result<Option<ProjectResponse>, String> { ::tsox_core::fntrace::enter("handle_get_default_project_for_file"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let uri = params
            .file
            .to_uri(&self.project_session.get_current_directory());
        let proj = sd.snapshot.get_default_project(&tsox_lsp::lsp::lsproto::DocumentUri(
            uri,
        ));
        if proj.is_none() {
            return Ok(None);
        }
        Ok(Some(new_project_response(proj.unwrap())))
    }

    pub fn handle_get_documentation_comment(
        &self,
        params: &CheckerSymbolParams,
    ) -> Result<String, String> { ::tsox_core::fntrace::enter("handle_get_documentation_comment"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let _symbol = setup.resolve_symbol_handle(params.symbol)?;
        // Go: ls.GetSymbolDocumentationComment(checker, symbol)；Rust 侧该实现在
        // tsox-lsp ls::jsdoc 上为 LanguageService 方法（需 LS 实例），接线缺口记
        // progress_notes_r68k05.md，当前移植态与 ls::jsdoc 方法体一致返回空串
        Ok(String::new())
    }

    pub fn handle_get_export_specifier_local_target_symbol(
        &self,
        params: &CheckerNodeParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_export_specifier_local_target_symbol"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup.sd.resolve_node_handle(&setup.program, &params.location)?;
        let Some(node) = node else { return Ok(None) };
        let symbol = setup.checker.get_export_specifier_local_target_symbol(&node);
        match symbol {
            None => Ok(None),
            Some(symbol) => Ok(setup.new_symbol_response(&symbol)),
        }
    }

    pub fn handle_get_export_symbol_of_symbol(
        &self,
        params: &GetSymbolPropertyParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_export_symbol_of_symbol"); 
        self.resolve_symbol_property_of_symbol(params, |sym| sym.export_symbol.clone())
    }

    pub fn handle_get_exports_of_module(
        &self,
        params: &CheckerSymbolParams,
    ) -> Result<Option<Vec<SymbolResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_exports_of_module"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        let exports = setup.checker.get_exports_of_module(&symbol);
        if exports.is_empty() {
            return Ok(None);
        }
        let mut sorted = exports;
        sorted.sort_by(|a, b| setup.checker.compare_symbols(a, b).cmp(&0));
        let results: Vec<SymbolResponse> = sorted
            .iter()
            .filter_map(|exp| setup.new_symbol_response(exp))
            .collect();
        Ok(Some(results))
    }

    pub fn handle_get_exports_of_symbol(
        &self,
        params: &GetSymbolPropertyParams,
    ) -> Result<Option<Vec<SymbolResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_exports_of_symbol"); 
        self.resolve_symbol_table_property_of_symbol(params, |symbol| {
            symbol.exports.entries.values().cloned().collect()
        })
    }

    pub fn handle_get_extends_type_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_extends_type_of_type"); 
        self.resolve_type_property_of_type(params, |t| {
            t.as_conditional_type().and_then(|ct| ct.extends_type.clone())
        })
    }

    pub fn handle_get_false_type_of_conditional_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_false_type_of_conditional_type"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.sd.resolve_type_handle(&params.project, params.r#type)?;
        let false_type = match setup.checker.get_false_type_of_conditional_type(&t) {
            Some(false_type) => false_type,
            None => return Ok(None),
        };
        Ok(setup.sd.new_type_response(&params.project, &false_type))
    }

    pub fn handle_get_fresh_type_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_fresh_type_of_type"); 
        self.resolve_type_property_of_type(params, |t| {
            t.as_literal_type().and_then(|lt| lt.fresh_type().cloned())
        })
    }

    pub fn handle_get_fully_qualified_name(
        &self,
        params: &CheckerSymbolParams,
    ) -> Result<String, String> { ::tsox_core::fntrace::enter("handle_get_fully_qualified_name"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        Ok(setup.checker.get_fully_qualified_name(&symbol, None))
    }

    pub fn handle_get_global_diagnostics(
        &self,
        params: &GetProjectDiagnosticsParams,
    ) -> Result<Vec<DiagnosticResponse>, String> { ::tsox_core::fntrace::enter("handle_get_global_diagnostics"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let proj = sd.get_project(&params.project)?;
        let program = proj
            .get_program()
            .ok_or_else(|| client_error("project has no program"))?;
        program.get_semantic_diagnostics();
        let diags: Vec<Arc<tsox_frontend::ast::Diagnostic>> = proj
            .get_project_diagnostics()
            .into_iter()
            .filter(|d| d.file().is_none())
            .collect();
        Ok(new_diagnostic_responses(&diags))
    }

    pub fn handle_get_immediate_aliased_symbol(
        &self,
        params: &CheckerSymbolParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_immediate_aliased_symbol"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        let aliased = setup.checker.get_immediate_aliased_symbol(&symbol);
        match aliased {
            None => Ok(None),
            Some(aliased) => Ok(setup.new_symbol_response(&aliased)),
        }
    }

    pub fn handle_get_import_adder_edits(
        &self,
        params: &GetImportAdderEditsParams,
    ) -> Result<Vec<TextEdit>, String> { ::tsox_core::fntrace::enter("handle_get_import_adder_edits"); 
        use tsox_lsp::ls::autoimport_import_adder::ImportAdderTrait;
        use tsox_lsp::lsp::lsproto::DocumentUri;

        let sd = self.get_snapshot_data(params.snapshot)?;
        let project_path = parse_project_handle(&params.project);
        let not_found = || client_error(format!("project {} not found", project_path.0));
        let no_program = || client_error("project has no program");
        let source_file_missing = || {
            client_error(format!(
                "source file not found: {}",
                params.file.to_file_name()
            ))
        };
        let resolve_in = |snapshot: &Arc<tsox_lsp::project::snapshot::Snapshot>| -> Result<
            (Arc<tsox_compile::compiler::Program>, Arc<SourceFile>),
            String,
        > {
            let collection = snapshot
                .project_collection
                .as_deref()
                .ok_or_else(not_found)?;
            let program = Arc::clone(
                collection
                    .get_project_by_path(&project_path)
                    .ok_or_else(not_found)?
                    .get_program()
                    .ok_or_else(no_program)?,
            );
            let source_file = program
                .get_source_file(&params.file.to_file_name())
                .ok_or_else(source_file_missing)?;
            Ok((program, source_file))
        };

        let working_snapshot = Arc::clone(&sd.snapshot);
        let (initial_program, initial_source_file) = resolve_in(&working_snapshot)?;
        let user_preferences = working_snapshot.user_preferences();
        let registry_ready = working_snapshot
            .auto_import_registry()
            .map(|r| {
                r.is_prepared_for_importing_file(
                    &initial_source_file.file_name,
                    &project_path,
                    &user_preferences,
                )
            })
            .unwrap_or(false);
        let (_working_snapshot, program, source_file, _user_preferences) = if registry_ready {
            (working_snapshot, initial_program, initial_source_file, user_preferences)
        } else {
            let prepared_snapshot = self.project_session.get_snapshot_with_auto_imports(
                &sd.snapshot,
                &DocumentUri(params.file.to_uri(&self.project_session.get_current_directory())),
            );
            let (program, source_file) = resolve_in(&prepared_snapshot)?;
            let prefs = prepared_snapshot.user_preferences();
            prepared_snapshot.deref_snapshot();
            (prepared_snapshot, program, source_file, prefs)
        };

        // Go: program.GetTypeChecker + autoimport.NewView/NewImportAdder 需要
        // Arc<Checker>（checker pool 仅提供 MutexGuard/Arc<Mutex<Checker>>，
        // clone_arc 缺口已在 progress_notes_r59A.md 登记，m5q2b 同款缺省），
        // 此处暂以 None 缺省，保留参数校验与响应折叠
        let mut import_adder: Option<tsox_lsp::ls::autoimport_import_adder::ImportAdder> = None;
        for (i, action) in params.actions.iter().enumerate() {
            match action.kind.as_str() {
                "import_symbol" | "ImportSymbol" => {
                    if action.symbol == 0 {
                        return Err(client_error(format!(
                            "import adder action {} missing symbol",
                            i
                        )));
                    }
                    let symbol = sd.resolve_symbol_handle(action.symbol)?;
                    let is_valid_type_only_use_site =
                        action.is_valid_type_only_use_site.unwrap_or(true);
                    if let Some(adder) = import_adder.as_mut() {
                        ImportAdderTrait::add_import_from_exported_symbol(
                            adder,
                            &symbol,
                            is_valid_type_only_use_site,
                        );
                    }
                }
                _ => {}
            }
        }
        if let Some(adder) = import_adder.as_mut() {
            if ImportAdderTrait::has_fixes(adder) {
                return Ok(to_api_text_edits(&source_file, &ImportAdderTrait::edits(adder))
                    .unwrap_or_default());
            }
        }
        Ok(Vec::new())
    }

    pub fn handle_get_index_infos_of_type(
        &self,
        params: &CheckerTypeParams,
    ) -> Result<Option<Vec<IndexInfoResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_index_infos_of_type"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let infos = setup.checker.get_index_infos_of_type(&t);
        if infos.is_empty() {
            return Ok(None);
        }
        let mut results = Vec::with_capacity(infos.len());
        for info in &infos {
            let key_type = info.key_type.as_ref().expect("index info key type");
            let value_type = info.value_type.as_ref().expect("index info value type");
            let mut resp = IndexInfoResponse {
                key_type: setup.new_type_response(key_type).unwrap(),
                value_type: setup.new_type_response(value_type).unwrap(),
                is_readonly: info.is_readonly,
                declaration: None,
            };
            if let Some(decl) = info.declaration.as_ref() {
                resp.declaration = Some(setup.sd.node_handle_from(decl));
            }
            results.push(resp);
        }
        Ok(Some(results))
    }

    pub fn handle_get_index_type_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_index_type_of_type"); 
        self.resolve_type_property_of_type(params, |t| {
            t.as_indexed_access_type().and_then(|ia| ia.index_type().cloned())
        })
    }

    pub fn handle_get_intrinsic_type(
        &self,
        params: &GetIntrinsicTypeParams,
        getter: fn(&Checker) -> Option<Arc<Type>>,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_intrinsic_type"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        match getter(&setup.checker) {
            None => Ok(None),
            Some(t) => Ok(setup.new_type_response(&t)),
        }
    }

    pub fn handle_get_jsdoc_tags(
        &self,
        params: &CheckerSymbolParams,
    ) -> Result<Vec<JSDocTagInfo>, String> { ::tsox_core::fntrace::enter("handle_get_jsdoc_tags"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let _symbol = setup.resolve_symbol_handle(params.symbol)?;
        // Go: ls.GetSymbolJSDocTags(symbol)，ls 包级函数而非 checker 方法；
        // Rust 侧同源实现在 tsox-lsp ls::jsdoc 上为 LanguageService 方法（需
        // Host 实例），接线缺口记 progress_notes_r69k03.md，当前移植态与该方法
        // 一致返回空列表（Go 对 len(tags)==0 同样返回 nil）
        Ok(Vec::new())
    }

    pub fn handle_get_local_type_parameters_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<Vec<TypeResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_local_type_parameters_of_type"); 
        self.resolve_type_array_property_of_type(params, |t| {
            t.as_interface_type()
                .map(|it| it.local_type_parameters().to_vec())
                .unwrap_or_default()
        })
    }

    pub fn handle_get_member_in_module_exports(
        &self,
        params: &GetMemberInModuleExportsParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_member_in_module_exports"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        let member = setup
            .checker
            .try_get_member_in_module_exports(&params.name, &symbol);
        match member {
            None => Ok(None),
            Some(member) => Ok(setup.new_symbol_response(&member)),
        }
    }

    pub fn handle_get_members_of_symbol(
        &self,
        params: &GetSymbolPropertyParams,
    ) -> Result<Option<Vec<SymbolResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_members_of_symbol"); 
        self.resolve_symbol_table_property_of_symbol(params, |symbol| {
            symbol.members.entries.values().cloned().collect()
        })
    }

    pub fn handle_get_non_missing_type_of_symbol(
        &self,
        params: &GetTypeOfSymbolParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_non_missing_type_of_symbol"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        let ty = setup.checker.get_non_missing_type_of_symbol(&symbol);
        Ok(setup.new_type_response(&ty))
    }
}

#[derive(serde::Deserialize)]
pub struct GetTypePropertyParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub r#type: TypeId,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticResponse {
    pub pos: i32,
    pub end: i32,
    pub code: i32,
    pub category: String,
    pub source: Option<String>,
    pub text: String,
    pub reports_unnecessary: bool,
    pub reports_deprecated: bool,
    pub file_name: Option<String>,
    pub start_position: Option<super::m5k_4::DiagnosticPositionResponse>,
    pub end_position: Option<super::m5k_4::DiagnosticPositionResponse>,
    pub source_lines: Vec<super::m5k_4::DiagnosticSourceLineResponse>,
    pub message_chain: Vec<DiagnosticResponse>,
    pub related_information: Vec<DiagnosticResponse>,
}

#[derive(serde::Serialize)]
pub struct TextEdit {
    pub pos: u32,
    pub end: u32,
    pub new_text: String,
}

pub(crate) fn parse_project_handle(project: &ProjectId) -> tsox_core::tspath::Path { ::tsox_core::fntrace::enter("parse_project_handle"); 
    tsox_core::tspath::Path(project.clone())
}

fn new_project_response(_proj: &tsox_lsp::project::project::Project) -> ProjectResponse { ::tsox_core::fntrace::enter("new_project_response"); 
    unimplemented!()
}

pub(crate) fn new_diagnostic_responses(
    _diags: &[Arc<tsox_frontend::ast::Diagnostic>],
) -> Vec<DiagnosticResponse> { ::tsox_core::fntrace::enter("new_diagnostic_responses"); 
    unimplemented!()
}

pub(crate) fn core_context() -> tsox_core::core::mig::m3j_3::RequestContext { ::tsox_core::fntrace::enter("core_context"); 
    Default::default()
}

impl Session {
    pub fn resolve_type_property_of_type(
        &self,
        params: &GetTypePropertyParams,
        getter: impl Fn(&Type) -> Option<Arc<Type>>,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("resolve_type_property_of_type"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        match getter(&t) {
            None => Ok(None),
            Some(result) => Ok(sd.new_type_response(&params.project, &result)),
        }
    }

    pub fn resolve_type_array_property_of_type(
        &self,
        params: &GetTypePropertyParams,
        getter: impl Fn(&Type) -> Vec<Arc<Type>>,
    ) -> Result<Option<Vec<TypeResponse>>, String> { ::tsox_core::fntrace::enter("resolve_type_array_property_of_type"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        let types = getter(&t);
        if types.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            types
                .iter()
                .filter_map(|sub| sd.new_type_response(&params.project, sub))
                .collect(),
        ))
    }

    pub fn resolve_symbol_property_of_symbol(
        &self,
        params: &GetSymbolPropertyParams,
        getter: impl Fn(&Symbol) -> Option<Arc<Symbol>>,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("resolve_symbol_property_of_symbol"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let symbol = sd.resolve_symbol_handle(params.symbol)?;
        match getter(&symbol) {
            None => Ok(None),
            Some(result) => Ok(sd.new_symbol_response(&result, &params.project)),
        }
    }

    pub fn resolve_symbol_table_property_of_symbol(
        &self,
        params: &GetSymbolPropertyParams,
        getter: impl Fn(&Symbol) -> Vec<Arc<Symbol>>,
    ) -> Result<Option<Vec<SymbolResponse>>, String> { ::tsox_core::fntrace::enter("resolve_symbol_table_property_of_symbol"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let symbol = sd.resolve_symbol_handle(params.symbol)?;
        let symbol_table = getter(&symbol);
        if symbol_table.is_empty() {
            return Ok(None);
        }
        if symbol_table.len() == 1 {
            return Ok(Some(vec![sd
                .new_symbol_response(&symbol_table[0], &params.project)
                .unwrap()]));
        }
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let mut symbols = symbol_table;
        symbols.sort_by(|a, b| setup.checker.compare_symbols(a, b).cmp(&0));
        Ok(Some(
            symbols
                .iter()
                .filter_map(|sub| setup.new_symbol_response(sub))
                .collect(),
        ))
    }
}

impl Session {
    pub fn resolve_symbol_property_of_type(
        &self,
        params: &GetTypePropertyParams,
        getter: impl Fn(&Type) -> Option<Arc<Symbol>>,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("resolve_symbol_property_of_type"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        match getter(&t) {
            None => Ok(None),
            Some(result) => Ok(sd.new_symbol_response(&result, &params.project)),
        }
    }

    pub fn resolve_type_array_property_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
        getter: impl Fn(&Signature) -> Vec<Arc<Type>>,
    ) -> Result<Option<Vec<TypeResponse>>, String> { ::tsox_core::fntrace::enter("resolve_type_array_property_of_signature"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        let types = getter(&sig);
        if types.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            types
                .iter()
                .filter_map(|sub| sd.new_type_response(&params.project, sub))
                .collect(),
        ))
    }

    pub fn resolve_symbol_property_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
        getter: impl Fn(&Signature) -> Option<Arc<Symbol>>,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("resolve_symbol_property_of_signature"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        match getter(&sig) {
            None => Ok(None),
            Some(result) => Ok(sd.new_symbol_response(&result, &params.project)),
        }
    }

    pub fn resolve_symbol_array_property_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
        getter: impl Fn(&Signature) -> Vec<Arc<Symbol>>,
    ) -> Result<Option<Vec<SymbolResponse>>, String> { ::tsox_core::fntrace::enter("resolve_symbol_array_property_of_signature"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        let symbols = getter(&sig);
        if symbols.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            symbols
                .iter()
                .filter_map(|sym| sd.new_symbol_response(sym, &params.project))
                .collect(),
        ))
    }

    pub fn resolve_signature_property_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
        getter: impl Fn(&Signature) -> Option<Signature>,
    ) -> Result<Option<SignatureResponse>, String> { ::tsox_core::fntrace::enter("resolve_signature_property_of_signature"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        match getter(&sig) {
            None => Ok(None),
            Some(result) => Ok(sd.new_signature_response(&params.project, &result)),
        }
    }

    pub fn get_diagnostics(
        &self,
        params: &GetDiagnosticsParams,
        getter: fn(&Program, Option<&Arc<SourceFile>>) -> Vec<tsox_frontend::ast::Diagnostic>,
    ) -> Result<Vec<DiagnosticResponse>, String> { ::tsox_core::fntrace::enter("get_diagnostics"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let source_file = self.resolve_optional_source_file(&program, params.file.as_ref())?;
        let diags: Vec<Arc<tsox_frontend::ast::Diagnostic>> = getter(&program, source_file.as_ref())
            .into_iter()
            .map(Arc::new)
            .collect();
        Ok(new_diagnostic_responses(&diags))
    }

    pub fn resolve_optional_source_file(
        &self,
        program: &Program,
        file: Option<&DocumentIdentifier>,
    ) -> Result<Option<Arc<SourceFile>>, String> { ::tsox_core::fntrace::enter("resolve_optional_source_file"); 
        let Some(file) = file else { return Ok(None) };
        let source_file = program
            .get_source_file(&file.to_file_name())
            .ok_or_else(|| client_error(format!("source file not found: {}", file)))?;
        Ok(Some(source_file))
    }

    pub fn setup_language_service(
        &self,
        _sd: &SnapshotData,
        program: &Program,
        project: &ProjectId,
        active_file: &str,
    ) -> Result<LanguageService, String> { ::tsox_core::fntrace::enter("setup_language_service"); 
        let project_name = parse_project_handle(project);
        let _ = active_file;
        Ok(LanguageService::new(&program, &project_name.0))
    }

    pub fn encode_source_file_response(
        &self,
        source_file: Option<Arc<SourceFile>>,
    ) -> Result<Option<SourceFileResponse>, String> { ::tsox_core::fntrace::enter("encode_source_file_response"); 
        let Some(source_file) = source_file else {
            if self.use_binary_responses {
                return Ok(None);
            }
            return Ok(None);
        };
        let (data, _) = super::m5j_encoder::encode_source_file(&source_file)
            .map_err(|e| format!("failed to encode source file: {}", e))?;
        if self.use_binary_responses {
            return Ok(None);
        }
        Ok(Some(SourceFileResponse {
            data: base64_encode(&data),
        }))
    }

    pub fn to_path(&self, file_name: &str) -> String { ::tsox_core::fntrace::enter("to_path"); 
        file_name.to_string()
    }
}


pub struct LanguageService;

pub struct CompletionEntryLabelDetailsInternal {
    pub detail: Option<String>,
    pub description: Option<String>,
}

pub struct CompletionEntryInternal {
    pub label: String,
    pub kind: Option<u32>,
    pub sort_text: Option<String>,
    pub insert_text: Option<String>,
    pub filter_text: Option<String>,
    pub detail: Option<String>,
    pub label_details: Option<CompletionEntryLabelDetailsInternal>,
    pub symbol: Option<Arc<Symbol>>,
}

pub struct CompletionInfoInternal {
    pub is_incomplete: bool,
    pub items: Vec<CompletionEntryInternal>,
}

impl LanguageService {
    pub fn get_completions_at_position(
        &self,
        _ctx: &mut tsox_core::core::mig::m3j_3::RequestContext,
        _source_file: &SourceFile,
        _position: usize,
        _trigger_character: Option<&str>,
        _include_symbol: bool,
    ) -> Result<Option<CompletionInfoInternal>, String> { ::tsox_core::fntrace::enter("get_completions_at_position"); 
        unimplemented!()
    }
    pub fn new(_program: &Program, _project: &str) -> LanguageService { ::tsox_core::fntrace::enter("new"); 
        LanguageService
    }
    pub fn get_signature_usages(
        &self,
        _ctx: &mut tsox_core::core::mig::m3j_3::RequestContext,
        _decl: &Node,
    ) -> Result<Option<Vec<SignatureUsage>>, String> { ::tsox_core::fntrace::enter("get_signature_usages"); 
        unimplemented!()
    }
    pub fn get_referenced_symbols_for_node(
        &self,
        _ctx: &mut tsox_core::core::mig::m3j_3::RequestContext,
        _position: u32,
        _node: &Node,
        _source_files: &[Arc<SourceFile>],
    ) -> Option<Vec<ReferencedSymbolEntryInternal>> { ::tsox_core::fntrace::enter("get_referenced_symbols_for_node"); 
        unimplemented!()
    }
}

pub struct SignatureUsage {
    pub name: Node,
    pub call: Option<Node>,
}

pub struct ReferencedSymbolEntryInternal {
    pub definition_node: Option<Node>,
    pub references: Vec<ReferenceEntryInternal>,
    pub definition_symbol: Option<Symbol>,
}

pub struct ReferenceEntryInternal {
    pub is_node_entry: bool,
    pub node: Option<Node>,
}

#[derive(serde::Deserialize)]
pub struct CheckerNodeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub location: NodeHandle,
}

#[derive(serde::Deserialize)]
pub struct CheckerSymbolParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub symbol: SymbolId,
}

#[derive(serde::Deserialize)]
pub struct CheckerTypeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub r#type: TypeId,
}

#[derive(serde::Deserialize)]
pub struct CheckerSignatureParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub signature: SignatureId,
}

#[derive(serde::Deserialize)]
pub struct GetContextualTypeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub location: NodeHandle,
}

#[derive(serde::Deserialize)]
pub struct GetDefaultProjectForFileParams {
    pub snapshot: SnapshotId,
    pub file: DocumentIdentifier,
}

#[derive(serde::Deserialize)]
pub struct GetDiagnosticsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: Option<DocumentIdentifier>,
}

#[derive(serde::Deserialize)]
pub struct GetProjectDiagnosticsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
}

#[derive(serde::Deserialize)]
pub struct GetSourceFileParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
}

#[derive(serde::Deserialize)]
pub struct GetSourceFileNamesParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
}

#[derive(serde::Deserialize)]
pub struct GetImportAdderEditsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
    pub actions: Vec<ImportAdderAction>,
}

#[derive(serde::Deserialize)]
pub struct ImportAdderAction {
    pub kind: String,
    pub symbol: SymbolId,
    pub is_valid_type_only_use_site: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct GetIntrinsicTypeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
}

#[derive(serde::Deserialize)]
pub struct GetMemberInModuleExportsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub symbol: SymbolId,
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct GetParameterTypeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub signature: SignatureId,
    pub index: i32,
}

#[derive(serde::Deserialize)]
pub struct GetPropertyOfTypeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub r#type: TypeId,
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct GetSignaturePropertyParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub signature: SignatureId,
}

#[derive(serde::Deserialize)]
pub struct GetSymbolPropertyParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub symbol: SymbolId,
}

#[derive(serde::Deserialize)]
pub struct GetTypeOfSymbolParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub symbol: SymbolId,
}

#[derive(serde::Deserialize)]
pub struct GetSymbolAtLocationParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub location: NodeHandle,
}

#[derive(serde::Deserialize)]
pub struct GetSymbolAtPositionParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
    pub position: u32,
}

#[derive(serde::Deserialize)]
pub struct GetSymbolsAtLocationsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub locations: Vec<NodeHandle>,
}

#[derive(serde::Deserialize)]
pub struct GetSymbolsAtPositionsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
    pub positions: Vec<u32>,
}

#[derive(serde::Deserialize)]
pub struct GetSymbolsInScopeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub location: Option<NodeHandle>,
    pub file: Option<DocumentIdentifier>,
    pub position: Option<u32>,
    pub meaning: u32,
}

#[derive(serde::Deserialize)]
pub struct GetSymbolsOfSourceFilesParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub files: Vec<DocumentIdentifier>,
}

#[derive(serde::Deserialize)]
pub struct GetSignaturesOfTypeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub r#type: TypeId,
    pub kind: i32,
}

#[derive(serde::Deserialize)]
pub struct GetResolvedSignatureParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub location: NodeHandle,
}

#[derive(serde::Deserialize)]
pub struct GetTypeAtLocationParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub location: NodeHandle,
}

#[derive(serde::Deserialize)]
pub struct GetTypeAtLocationsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub locations: Vec<NodeHandle>,
}

#[derive(serde::Deserialize)]
pub struct GetTypeAtPositionParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
    pub position: u32,
}

#[derive(serde::Deserialize)]
pub struct GetTypeOfSymbolAtLocationParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub symbol: SymbolId,
    pub location: NodeHandle,
}

#[derive(serde::Deserialize)]
pub struct GetTypesAtPositionsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
    pub positions: Vec<u32>,
}

#[derive(serde::Deserialize)]
pub struct GetTypesOfSymbolsParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub symbols: Vec<SymbolId>,
}

#[derive(serde::Deserialize)]
pub struct GetWidenedTypeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub r#type: TypeId,
}

#[derive(serde::Deserialize)]
pub struct IsArrayLikeTypeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub r#type: TypeId,
}

#[derive(serde::Deserialize)]
pub struct IsTypeAssignableToParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub source: TypeId,
    pub target: TypeId,
}

#[derive(serde::Deserialize)]
pub struct GetReferencedSymbolsForNodeParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub node: NodeHandle,
    pub position: u32,
}

#[derive(serde::Deserialize)]
pub struct GetReferencesToSymbolInFileParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
    pub symbol: SymbolId,
}

#[derive(serde::Deserialize)]
pub struct GetSignatureUsagesParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub signature_decl: NodeHandle,
}

#[derive(serde::Serialize)]
pub struct IndexInfoResponse {
    pub key_type: TypeResponse,
    pub value_type: TypeResponse,
    pub is_readonly: bool,
    pub declaration: Option<NodeHandle>,
}

#[derive(serde::Serialize)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SymbolResponse {
    pub id: SymbolId,
    pub project: ProjectId,
    pub name: String,
    pub flags: u32,
    pub check_flags: u32,
    pub declarations: Vec<NodeHandle>,
    pub value_declaration: Option<NodeHandle>,
    pub parent: SymbolId,
    pub export_symbol: SymbolId,
}

#[derive(serde::Serialize)]
pub struct SignatureResponse {
    pub id: SignatureId,
    pub flags: u32,
    pub declaration: Option<NodeHandle>,
    pub type_parameters: Vec<TypeId>,
    pub parameters: Vec<SymbolId>,
    pub this_parameter: SymbolId,
    pub target: SignatureId,
}

#[derive(serde::Serialize)]
pub struct SourceFileResponse {
    pub data: String,
}

#[derive(serde::Serialize)]
pub struct TypePredicateResponse;
#[derive(serde::Serialize)]
pub struct SourceFileMetadata;
#[derive(serde::Serialize)]
pub struct SignatureUsageResponse;
#[derive(serde::Serialize)]
pub struct InitializeResponse;
#[derive(serde::Deserialize)]
pub struct ProfileParams {
    pub dir: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ProfileResult {
    pub file: String,
}

#[derive(serde::Deserialize)]
pub struct ReadConfigFileParams {
    pub file: DocumentIdentifier,
}

#[derive(serde::Deserialize)]
pub struct ReleaseParams {
    pub snapshot: SnapshotId,
}

#[derive(serde::Deserialize)]
pub struct ResolveNameParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub name: String,
    pub location: NodeHandle,
    pub file: Option<DocumentIdentifier>,
    pub position: Option<u32>,
    pub meaning: u32,
    pub exclude_globals: bool,
}
