use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::m5j_encoder::get_node_index_table;
pub use super::m5k_3::{
    ProjectId as ProjectID, SignatureId as SignatureID, SymbolId as SymbolID, TypeId as TypeID,
};
pub use super::m5k_4::{new_diagnostic_responses, DiagnosticResponse, TypeResponse};
pub use super::m5l::{
    NodeHandle, SignatureResponse, SnapshotId as SnapshotID, SymbolResponse, TextEdit,
};
pub(crate) use super::m5l::client_error as err_client_error;
use tsox_checker::checker::mig::m2d_2::is_tuple_type_target;
use tsox_checker::checker::{Checker, Signature as CheckerSignature, Type as CheckerType};
use tsox_compile::compiler::Program;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::positionmap::compute_position_map;
use tsox_frontend::ast::Symbol as AstSymbol;
use tsox_frontend::ast::{get_source_file_of_node, mig as ast_mig};
use tsox_lsp::lsp::lsproto_lsp::Position;
use tsox_lsp::ls::lsconv_linemap::{compute_lsp_line_starts, LspLineMap};
use tsox_lsp::project::project::Project;
use tsox_lsp::project::snapshot::Snapshot;

fn symbol_handle(symbol: &AstSymbol) -> SymbolID {
    symbol.id() as u32
}

fn type_handle(t: &CheckerType) -> TypeID {
    t.id
}

fn signature_handle(sig: &CheckerSignature) -> SignatureID {
    sig.id
}

fn node_kind_string(node: &Node) -> String {
    ast_mig::m3b::kind_string(node)
}

fn escape_symbol_name(name: &str) -> String {
    ast_mig::m3e_3::escape_symbol_name(name)
}

pub struct SnapshotData {
    pub snapshot: Arc<Snapshot>,
    pub ref_count: i32,
    pub symbol_registry: RwLock<HashMap<SymbolID, *const Arc<AstSymbol>>>,
    pub symbol_canonical_projects: RwLock<HashMap<SymbolID, ProjectID>>,
    pub project_registries: RwLock<HashMap<ProjectID, ProjectRegistryData>>,
}

pub struct ProjectRegistryData {
    pub type_registry: RwLock<HashMap<TypeID, *const Arc<CheckerType>>>,
    pub signature_registry: RwLock<HashMap<SignatureID, *const Arc<CheckerSignature>>>,
}

impl ProjectRegistryData {
    fn new() -> Self {
        ProjectRegistryData {
            type_registry: RwLock::new(HashMap::new()),
            signature_registry: RwLock::new(HashMap::new()),
        }
    }
}

impl SnapshotData {
    pub fn get_project(&self, project_handle: &ProjectID) -> Result<&Project, String> {
        let project_name = tsox_core::tspath::Path(project_handle.clone());
        let proj = self
            .snapshot
            .project_collection
            .as_ref()
            .and_then(|pc| pc.get_project_by_path(&project_name));
        match proj {
            Some(proj) => Ok(proj),
            None => Err(err_client_error(&format!(
                "project {project_handle} not found"
            ))),
        }
    }

    pub fn get_program(&self, project_handle: &ProjectID) -> Result<&Arc<Program>, String> {
        let proj = self.get_project(project_handle)?;
        match proj.get_program() {
            Some(program) => Ok(program),
            None => Err(err_client_error("project has no program")),
        }
    }

    pub fn node_handle_from(&self, node: &Arc<Node>) -> NodeHandle {
        let _ = node;
        unimplemented!("node handle 需 Node→SourceFile 回指与全局索引表,待 encoder 接线")
    }

    pub fn new_symbol_response(
        &self,
        symbol: Option<&Arc<AstSymbol>>,
        canonical_project: ProjectID,
    ) -> Option<SymbolResponse> {
        let symbol = symbol?;
        let (id, project) = self.register_symbol(Some(symbol), canonical_project.clone());
        let mut resp = SymbolResponse {
            id,
            project,
            name: escape_symbol_name(&symbol.name),
            flags: symbol.flags.bits(),
            check_flags: symbol.check_flags.bits(),
            declarations: Vec::new(),
            value_declaration: None,
            parent: 0,
            export_symbol: 0,
        };
        if !symbol.declarations.is_empty() {
            resp.declarations = symbol
                .declarations
                .iter()
                .map(|decl| self.node_handle_from(decl))
                .collect();
        }
        if let Some(value_declaration) = &symbol.value_declaration {
            resp.value_declaration = Some(self.node_handle_from(value_declaration));
        }
        if let Some(parent) = symbol.parent() {
            resp.parent = symbol_handle(&parent);
        }
        if let Some(export_symbol) = &symbol.export_symbol {
            resp.export_symbol = symbol_handle(export_symbol);
        }
        Some(resp)
    }

    pub fn register_symbol(
        &self,
        symbol: Option<&Arc<AstSymbol>>,
        canonical_project: ProjectID,
    ) -> (SymbolID, ProjectID) {
        let symbol = match symbol {
            Some(symbol) => symbol,
            None => return (0, String::new()),
        };
        if canonical_project.is_empty() {
            panic!("registerSymbol requires a non-empty canonical project");
        }
        let id = symbol_handle(symbol);
        let ptr = symbol as *const Arc<AstSymbol>;
        let mut registry = self.symbol_registry.write().unwrap();
        match registry.get(&id).copied() {
            Some(existing) => {
                if existing != ptr {
                    panic!("duplicate symbol");
                }
            }
            None => {
                registry.insert(id, ptr);
            }
        }
        drop(registry);
        let project = {
            let mut projects = self.symbol_canonical_projects.write().unwrap();
            match projects.get(&id) {
                Some(project) => project.clone(),
                None => {
                    projects.insert(id, canonical_project.clone());
                    canonical_project.clone()
                }
            }
        };
        (id, project)
    }

    pub fn new_type_response(
        &self,
        project_id: ProjectID,
        t: Option<&Arc<CheckerType>>,
    ) -> Option<TypeResponse> {
        let t = t?;
        let mut resp = super::m5k_4::new_type_response(t, self.register_type(project_id.clone(), Some(t)));
        if is_tuple_type_target(t) {
            if let tsox_checker::checker::TypeData::Tuple(tuple) = &t.data {
                let count = tuple.element_infos.len();
                for (i, info) in tuple.element_infos.iter().enumerate() {
                    if let Some(declaration) = &info.labeled_declaration {
                        if resp.labeled_element_declarations.is_none() {
                            resp.labeled_element_declarations = Some(vec![NodeHandle::default(); count]);
                        }
                        resp.labeled_element_declarations.as_mut().unwrap()[i] =
                            self.node_handle_from(declaration);
                    }
                }
            }
        }
        Some(resp)
    }

    pub fn register_type(&self, project_id: ProjectID, t: Option<&Arc<CheckerType>>) -> TypeID {
        let t = match t {
            Some(t) => t,
            None => return 0,
        };
        let id = type_handle(t);
        let ptr = t as *const Arc<CheckerType>;
        let mut registries = self.project_registries.write().unwrap();
        let reg = registries.entry(project_id).or_insert_with(ProjectRegistryData::new);
        let mut types = reg.type_registry.write().unwrap();
        if let Some(existing) = types.get(&id).copied() {
            if existing != ptr {
                panic!("duplicate type");
            }
            return id;
        }
        types.insert(id, ptr);
        id
    }

    pub fn resolve_symbol_handle(&self, handle: SymbolID) -> Result<*const Arc<AstSymbol>, String> {
        if handle == 0 {
            return Err(err_client_error("empty symbol handle"));
        }
        let registry = self.symbol_registry.read().unwrap();
        match registry.get(&handle) {
            Some(symbol) => Ok(*symbol),
            None => Err(err_client_error(&format!(
                "symbol handle {handle} not found in snapshot registry"
            ))),
        }
    }

    pub fn resolve_type_handle(
        &self,
        project_id: &ProjectID,
        handle: TypeID,
    ) -> Result<*const Arc<CheckerType>, String> {
        if handle == 0 {
            return Err(err_client_error("empty type handle"));
        }
        if project_id.is_empty() {
            return Err(err_client_error(&format!(
                "empty project ID for type handle {handle}"
            )));
        }
        let registries = self.project_registries.read().unwrap();
        let reg = match registries.get(project_id) {
            Some(reg) => reg,
            None => {
                return Err(err_client_error(&format!(
                    "type handle {handle} not found (no registry for project {project_id})"
                )))
            }
        };
        let types = reg.type_registry.read().unwrap();
        match types.get(&handle) {
            Some(t) => Ok(*t),
            None => Err(err_client_error(&format!(
                "type handle {handle} not found in project registry"
            ))),
        }
    }

    pub fn resolve_signature_handle(
        &self,
        project_id: &ProjectID,
        handle: SignatureID,
    ) -> Result<*const Arc<CheckerSignature>, String> {
        if handle == 0 {
            return Err(err_client_error("empty signature handle"));
        }
        if project_id.is_empty() {
            return Err(err_client_error(&format!(
                "empty project ID for signature handle {handle}"
            )));
        }
        let registries = self.project_registries.read().unwrap();
        let reg = match registries.get(project_id) {
            Some(reg) => reg,
            None => {
                return Err(err_client_error(&format!(
                    "signature handle {handle} not found (no registry for project {project_id})"
                )))
            }
        };
        let signatures = reg.signature_registry.read().unwrap();
        match signatures.get(&handle) {
            Some(sig) => Ok(*sig),
            None => Err(err_client_error(&format!(
                "signature handle {handle} not found in project registry"
            ))),
        }
    }

    pub fn new_signature_response(
        &self,
        project_id: ProjectID,
        sig: Option<&Arc<CheckerSignature>>,
    ) -> Option<SignatureResponse> {
        let sig = sig?;
        let mut resp = SignatureResponse {
            id: self.register_signature(project_id, Some(sig)),
            flags: sig.flags.bits(),
            declaration: None,
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            this_parameter: 0,
            target: 0,
        };
        if let Some(declaration) = &sig.declaration {
            resp.declaration = Some(self.node_handle_from(declaration));
        }
        if !sig.type_parameters.is_empty() {
            resp.type_parameters = sig.type_parameters.iter().map(|t| t.id).collect();
        }
        if !sig.parameters.is_empty() {
            resp.parameters = sig.parameters.iter().map(|s| s.id() as u32).collect();
        }
        if let Some(this_parameter) = &sig.this_parameter {
            resp.this_parameter = this_parameter.id() as u32;
        }
        if let Some(target) = &sig.target {
            resp.target = target.id;
        }
        Some(resp)
    }

    pub fn register_signature(
        &self,
        project_id: ProjectID,
        sig: Option<&Arc<CheckerSignature>>,
    ) -> SignatureID {
        let sig = match sig {
            Some(sig) => sig,
            None => return 0,
        };
        let id = signature_handle(sig);
        let ptr = sig as *const Arc<CheckerSignature>;
        let mut registries = self.project_registries.write().unwrap();
        let reg = registries.entry(project_id).or_insert_with(ProjectRegistryData::new);
        let mut signatures = reg.signature_registry.write().unwrap();
        if let Some(existing) = signatures.get(&id).copied() {
            if existing != ptr {
                panic!("duplicate signature");
            }
            return id;
        }
        signatures.insert(id, ptr);
        id
    }

    pub fn resolve_node_handle(
        &self,
        program: &Program,
        handle: NodeHandle,
    ) -> Result<*const Node, String> {
        let s = handle.as_str();
        let first_dot = match s.find('.') {
            Some(index) => index,
            None => return Err(err_client_error(&format!("invalid node handle {s:?}"))),
        };
        let second_dot = match s[first_dot + 1..].find('.') {
            Some(index) => index + first_dot + 1,
            None => return Err(err_client_error(&format!("invalid node handle {s:?}"))),
        };
        let idx: usize = match s[..first_dot].parse() {
            Ok(idx) => idx,
            Err(err) => return Err(err_client_error(&format!("invalid node handle {s:?}: {err}"))),
        };
        let path = s[second_dot + 1..].to_string();
        let source_file = match program.get_source_file_by_path(&path) {
            Some(source_file) => source_file,
            None => {
                return Err(err_client_error(&format!(
                    "node handle {s:?} could not be resolved (file may not be loaded or handle may be stale)"
                )))
            }
        };
        let table = get_node_index_table(&source_file);
        if let Some(node) = table.nodes.get(idx).and_then(|node| node.as_ref()) {
            return Ok(Arc::as_ptr(node));
        }
        Err(err_client_error(&format!(
            "node handle {s:?} could not be resolved (file may not be loaded or handle may be stale)"
        )))
    }
}

pub fn snapshot_handle(snapshot: &Snapshot) -> SnapshotID {
    snapshot.id()
}

pub fn non_nil_diagnostics(diags: &[tsox_frontend::ast::Diagnostic]) -> Vec<DiagnosticResponse> {
    new_diagnostic_responses(diags).unwrap_or_default()
}

pub fn original_text_offset(
    line_map: &LspLineMap,
    position: &Position,
    text_length: usize,
) -> Option<usize> {
    let line = position.line as usize;
    if line >= line_map.line_starts.len() {
        return None;
    }
    let offset = line_map.line_starts[line] + position.character as usize;
    if offset > text_length {
        return None;
    }
    Some(offset)
}

pub fn to_api_text_edits(
    source_file: &SourceFile,
    edits: &[tsox_lsp::lsp::lsproto_lsp::TextEdit],
) -> Option<Vec<TextEdit>> {
    let original_text = source_file.text.clone();
    let line_map = compute_lsp_line_starts(&original_text);
    let position_map = compute_position_map(&original_text);
    let mut result = Vec::with_capacity(edits.len());
    for edit in edits {
        let start = original_text_offset(&line_map, &edit.range.start, original_text.len())?;
        let end = original_text_offset(&line_map, &edit.range.end, original_text.len())?;
        result.push(TextEdit {
            pos: position_map.utf8_to_utf16(start) as u32,
            end: position_map.utf8_to_utf16(end) as u32,
            new_text: edit.new_text.clone(),
        });
    }
    Some(result)
}

#[derive(Clone)]
pub struct TranspileOptions {
    pub compiler_options: tsox_core::core::compiler_options::CompilerOptions,
    pub file_name: String,
    pub report_diagnostics: bool,
}

pub struct TranspileOutputResponse {
    pub output_text: String,
    pub diagnostics: Vec<DiagnosticResponse>,
    pub source_map_text: String,
}

pub fn transpile_output(
    input: &str,
    options: TranspileOptions,
    declaration: bool,
) -> Result<TranspileOutputResponse, String> {
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
        diagnostics: new_diagnostic_responses(&output.diagnostics).unwrap_or_default(),
        source_map_text: output.source_map_text,
    })
}

pub struct CheckerSetup<'a> {
    pub sd: &'a SnapshotData,
    pub program: &'a Program,
    pub checker: std::sync::MutexGuard<'a, Checker>,
    pub project_id: ProjectID,
}

impl<'a> CheckerSetup<'a> {
    pub fn new_type_response(&self, t: Option<&Arc<CheckerType>>) -> Option<TypeResponse> {
        self.sd.new_type_response(self.project_id.clone(), t)
    }

    pub fn new_symbol_response(&self, sym: Option<&Arc<AstSymbol>>) -> Option<SymbolResponse> {
        self.sd.new_symbol_response(sym, self.project_id.clone())
    }

    pub fn new_signature_response(
        &self,
        sig: Option<&Arc<CheckerSignature>>,
    ) -> Option<SignatureResponse> {
        self.sd
            .new_signature_response(self.project_id.clone(), sig)
    }

    pub fn resolve_type_handle(&self, id: TypeID) -> Result<*const Arc<CheckerType>, String> {
        self.sd.resolve_type_handle(&self.project_id, id)
    }

    pub fn resolve_symbol_handle(&self, id: SymbolID) -> Result<*const Arc<AstSymbol>, String> {
        self.sd.resolve_symbol_handle(id)
    }

    pub fn resolve_signature_handle(
        &self,
        id: SignatureID,
    ) -> Result<*const Arc<CheckerSignature>, String> {
        self.sd.resolve_signature_handle(&self.project_id, id)
    }
}
