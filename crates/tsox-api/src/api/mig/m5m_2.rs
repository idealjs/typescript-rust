use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use super::m5k_3::DocumentIdentifier;
use super::m5m::{err_client_error, snapshot_handle, CheckerSetup, NodeHandle, ProjectID,
    SignatureID, SignatureResponse, SnapshotData, SnapshotID, SymbolID, SymbolResponse,
    TranspileOptions, TranspileOutputResponse, TypeID, TypeResponse};
use tsox_checker::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
use tsox_checker::checker::{Signature as CheckerSignature, Type as CheckerType};
use tsox_compile::compiler::Program;
use tsox_frontend::ast::{Node, SourceFile, Symbol};
use tsox_lsp::lsp::lsproto::DocumentUri;
use tsox_lsp::project::file_change::FileChangeSummary;
use tsox_lsp::project::project::Project;
use tsox_lsp::project::session::Session as ProjectSession;
use tsox_lsp::project::snapshot::{APISnapshotRequest, Snapshot};
use tsox_core::tspath::{to_path as tspath_to_path, Path as TspathPath};

pub type Result<T> = std::result::Result<T, String>;

pub struct GetSignaturePropertyParams {
    pub snapshot: SnapshotID,
    pub project: ProjectID,
    pub signature: SignatureID,
}

pub struct GetSymbolPropertyParams {
    pub snapshot: SnapshotID,
    pub project: ProjectID,
    pub symbol: SymbolID,
}

pub struct GetTypePropertyParams {
    pub snapshot: SnapshotID,
    pub project: ProjectID,
    pub r#type: TypeID,
}

pub struct TypeToTypeNodeParams {
    pub snapshot: SnapshotID,
    pub project: ProjectID,
    pub r#type: TypeID,
    pub location: NodeHandle,
    pub flags: u32,
}

pub struct APIFileChanges {
    pub invalidate_all: bool,
    pub changed: Vec<DocumentIdentifier>,
    pub created: Vec<DocumentIdentifier>,
    pub deleted: Vec<DocumentIdentifier>,
}

pub struct UpdateSnapshotParams {
    pub file_changes: Option<APIFileChanges>,
    pub open_projects: Vec<DocumentIdentifier>,
    pub close_projects: Vec<DocumentIdentifier>,
    pub open_files: Vec<DocumentIdentifier>,
    pub close_files: Vec<DocumentIdentifier>,
}

pub struct UpdateTemporarySnapshotParams {
    pub snapshot: SnapshotID,
    pub file: DocumentIdentifier,
    pub new_text: String,
}

pub struct ProjectResponse;

pub struct SnapshotChanges;

pub struct UpdateSnapshotResponse {
    pub snapshot: SnapshotID,
    pub projects: Vec<ProjectResponse>,
    pub changes: Option<SnapshotChanges>,
}

pub struct SourceFileResponse {
    pub data: String,
}

pub struct TranspileParams {
    pub input: String,
    pub options: TranspileOptions,
}

pub struct TranspileFromFileParams {
    pub file_name: String,
    pub options: TranspileOptions,
}

pub struct SnapshotTable {
    pub snapshots: HashMap<SnapshotID, Box<SnapshotData>>,
    pub latest_snapshot: SnapshotID,
}

#[derive(Clone)]
pub struct PathSet {
    items: Vec<TspathPath>,
}

impl PathSet {
    pub fn new() -> Self {
        PathSet { items: Vec::new() }
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn has(&self, path: &TspathPath) -> bool {
        self.items.iter().any(|p| p == path)
    }
    pub fn add(&mut self, path: TspathPath) {
        if !self.has(&path) {
            self.items.push(path);
        }
    }
    pub fn delete(&mut self, path: &TspathPath) {
        self.items.retain(|p| p != path);
    }
    pub fn clear(&mut self) {
        self.items.clear();
    }
    pub fn to_hash_set(&self) -> HashSet<TspathPath> {
        self.items.iter().cloned().collect()
    }
}

pub struct Session {
    pub snapshots_mu: RwLock<SnapshotTable>,
    pub update_mu: Mutex<()>,
    pub open_projects: PathSet,
    pub open_files: PathSet,
    pub project_session: Arc<ProjectSession>,
    pub use_binary_responses: bool,
}

impl Session {
    pub fn get_snapshot_data(&self, handle: SnapshotID) -> Result<*const SnapshotData> {
        let guard = self.snapshots_mu.read().unwrap();
        match guard.snapshots.get(&handle) {
            Some(sd) => Ok(std::ptr::from_ref(sd.as_ref())),
            None => Err(err_client_error(&format!("snapshot {handle} not found"))),
        }
    }
}

pub fn raw_binary(data: &[u8]) -> String {
    base64_encode_impl(data, false)
}

pub fn base64_std_encode(data: &[u8]) -> String {
    base64_encode_impl(data, true)
}

fn base64_encode_impl(data: &[u8], padded: bool) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[n as usize & 63] as char);
        }
    }
    if padded {
        while out.len() % 4 != 0 {
            out.push('=');
        }
    }
    out
}

pub fn new_project_response(_proj: &Project) -> ProjectResponse {
    ProjectResponse
}

pub fn compute_snapshot_changes(
    _prev: &Snapshot,
    _current: &Snapshot,
) -> Option<SnapshotChanges> {
    None
}

fn node_builder_type_format_flags(bits: u32) -> TypeFormatFlags {
    let canonical_bits: [(u32, TypeFormatFlags); 5] = [
        (1 << 0, TypeFormatFlags::NO_TRUNCATION),
        (1 << 1, TypeFormatFlags::WRITE_ARRAY_AS_GENERIC),
        (1 << 10, TypeFormatFlags::MULTILINE_OBJECT_LITERALS),
        (1 << 14, TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE),
        (1 << 20, TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE),
    ];
    let mut flags = TypeFormatFlags::NONE;
    for (canonical, flag) in canonical_bits {
        if bits & canonical != 0 {
            flags = flags.union(flag);
        }
    }
    flags
}

pub fn parse_project_handle(project: &ProjectID) -> String {
    project.clone()
}

impl Session {
    pub fn retain_snapshot_data(&mut self, handle: SnapshotID) -> Result<*mut SnapshotData> {
        let mut guard = self.snapshots_mu.write().unwrap();
        match guard.snapshots.get_mut(&handle) {
            Some(sd) => {
                sd.ref_count += 1;
                Ok(sd.as_mut() as *mut SnapshotData)
            }
            None => Err(err_client_error(&format!("snapshot {handle} not found"))),
        }
    }

    pub fn release_snapshot(&mut self, handle: SnapshotID) -> Result<()> {
        let mut guard = self.snapshots_mu.write().unwrap();
        let release = match guard.snapshots.get_mut(&handle) {
            None => return Err(err_client_error(&format!("snapshot {handle} not found"))),
            Some(sd) => {
                sd.ref_count -= 1;
                sd.ref_count <= 0
            }
        };
        if release {
            let mut sd = guard.snapshots.remove(&handle).unwrap();
            sd.snapshot.deref_snapshot();
        }
        Ok(())
    }

    pub fn setup_checker(
        &self,
        snapshot: SnapshotID,
        project_handle: ProjectID,
    ) -> Result<CheckerSetup<'_>> {
        let sd = unsafe { &*self.get_snapshot_data(snapshot)? };
        let program = sd.get_program(&project_handle)?;
        let checker = program.get_type_checker();
        Ok(CheckerSetup {
            sd,
            program,
            checker,
            project_id: project_handle,
        })
    }

    pub fn setup_language_service(
        &self,
        sd: &SnapshotData,
        program: &Arc<Program>,
        project_handle: ProjectID,
        active_file: &str,
    ) -> Result<tsox_lsp::ls::language_service::LanguageService> {
        let project_name = parse_project_handle(&project_handle);
        let path = TspathPath(project_name.clone());
        let proj = sd
            .snapshot
            .project_collection
            .as_ref()
            .and_then(|pc| pc.get_project_by_path(&path));
        match proj {
            None => Err(err_client_error(&format!("project {project_name} not found"))),
            Some(proj) => Ok(tsox_lsp::ls::language_service::LanguageService::new(
                proj.config_file_path.clone(),
                program.clone(),
                Box::new(Arc::clone(&sd.snapshot)),
                active_file,
            )),
        }
    }

    pub fn resolve_optional_source_file(
        &self,
        program: &Program,
        file: Option<&DocumentIdentifier>,
    ) -> Result<Option<*const SourceFile>> {
        let file = match file {
            Some(file) => file,
            None => return Ok(None),
        };
        match program.get_source_file(&file.to_file_name()) {
            Some(source_file) => Ok(Some(Arc::as_ptr(&source_file))),
            None => Err(err_client_error(&format!(
                "source file not found: {file:?}"
            ))),
        }
    }

    pub fn resolve_type_property_of_type(
        &self,
        params: &GetTypePropertyParams,
        getter: impl Fn(&Arc<CheckerType>) -> Option<&Arc<CheckerType>>,
    ) -> Result<Option<TypeResponse>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        let result = getter(unsafe { &*t });
        match result {
            None => Ok(None),
            Some(result) => Ok(sd.new_type_response(params.project.clone(), Some(result))),
        }
    }

    pub fn resolve_type_array_property_of_type(
        &self,
        params: &GetTypePropertyParams,
        getter: impl Fn(&Arc<CheckerType>) -> Vec<Arc<CheckerType>>,
    ) -> Result<Option<Vec<TypeResponse>>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        let types = getter(unsafe { &*t });
        if types.is_empty() {
            return Ok(None);
        }
        let mut results = Vec::with_capacity(types.len());
        for sub in &types {
            results.push(sd.new_type_response(params.project.clone(), Some(sub)).unwrap());
        }
        Ok(Some(results))
    }

    pub fn resolve_symbol_property_of_type(
        &self,
        params: &GetTypePropertyParams,
        getter: impl Fn(&Arc<CheckerType>) -> Option<&Arc<Symbol>>,
    ) -> Result<Option<SymbolResponse>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        let result = getter(unsafe { &*t });
        match result {
            None => Ok(None),
            Some(result) => Ok(sd.new_symbol_response(Some(result), params.project.clone())),
        }
    }

    pub fn resolve_symbol_property_of_symbol(
        &self,
        params: &GetSymbolPropertyParams,
        getter: impl Fn(&Arc<Symbol>) -> Option<&Arc<Symbol>>,
    ) -> Result<Option<SymbolResponse>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let symbol = sd.resolve_symbol_handle(params.symbol)?;
        let result = getter(unsafe { &*symbol });
        match result {
            None => Ok(None),
            Some(result) => Ok(sd.new_symbol_response(Some(result), params.project.clone())),
        }
    }

    pub fn resolve_symbol_table_property_of_symbol(
        &self,
        params: &GetSymbolPropertyParams,
        getter: impl Fn(&Arc<Symbol>) -> Vec<Arc<Symbol>>,
    ) -> Result<Option<Vec<SymbolResponse>>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let symbol = sd.resolve_symbol_handle(params.symbol)?;
        let symbol_table = getter(unsafe { &*symbol });
        if symbol_table.is_empty() {
            return Ok(None);
        }
        if symbol_table.len() == 1 {
            let sub = &symbol_table[0];
            return Ok(Some(vec![
                sd.new_symbol_response(Some(sub), params.project.clone()).unwrap()
            ]));
        }
        let setup = self.setup_checker(params.snapshot, params.project.clone())?;
        let mut symbols = symbol_table;
        symbols.sort_by(|a, b| setup.checker.compare_symbols(a, b).cmp(&0));
        let mut results = Vec::with_capacity(symbols.len());
        for sub in &symbols {
            results.push(setup.new_symbol_response(Some(sub)).unwrap());
        }
        Ok(Some(results))
    }

    pub fn resolve_symbol_array_property_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
        getter: impl Fn(&Arc<CheckerSignature>) -> Vec<Arc<Symbol>>,
    ) -> Result<Option<Vec<SymbolResponse>>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        let symbols = getter(unsafe { &*sig });
        if symbols.is_empty() {
            return Ok(None);
        }
        let mut results = Vec::with_capacity(symbols.len());
        for sym in &symbols {
            results.push(sd.new_symbol_response(Some(sym), params.project.clone()).unwrap());
        }
        Ok(Some(results))
    }

    pub fn resolve_symbol_property_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
        getter: impl Fn(&Arc<CheckerSignature>) -> Option<&Arc<Symbol>>,
    ) -> Result<Option<SymbolResponse>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        let result = getter(unsafe { &*sig });
        match result {
            None => Ok(None),
            Some(result) => Ok(sd.new_symbol_response(Some(result), params.project.clone())),
        }
    }

    pub fn resolve_type_array_property_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
        getter: impl Fn(&Arc<CheckerSignature>) -> Vec<Arc<CheckerType>>,
    ) -> Result<Option<Vec<TypeResponse>>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        let types = getter(unsafe { &*sig });
        if types.is_empty() {
            return Ok(None);
        }
        let mut results = Vec::with_capacity(types.len());
        for sub in &types {
            results.push(sd.new_type_response(params.project.clone(), Some(sub)).unwrap());
        }
        Ok(Some(results))
    }

    pub fn resolve_signature_property_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
        getter: impl Fn(&Arc<CheckerSignature>) -> Option<&Arc<CheckerSignature>>,
    ) -> Result<Option<SignatureResponse>> {
        let sd = unsafe { &*self.get_snapshot_data(params.snapshot)? };
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        let result = getter(unsafe { &*sig });
        match result {
            None => Ok(None),
            Some(result) => Ok(sd.new_signature_response(params.project.clone(), Some(result))),
        }
    }

    pub fn handle_type_to_type_node(
        &self,
        params: &TypeToTypeNodeParams,
    ) -> Result<Option<SourceFileResponse>> {
        let mut setup = self.setup_checker(params.snapshot, params.project.clone())?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let mut enclosing_declaration: Option<*const Node> = None;
        if !params.location.is_empty() {
            enclosing_declaration =
                Some(setup.sd.resolve_node_handle(setup.program, params.location.clone())?);
        }
        let type_node = setup.checker.type_to_type_node(unsafe { &*t });
        let (data, _) = super::m5j_encoder::encode_tree(&type_node, None)
            .map_err(|e| format!("failed to encode type node: {e}"))?;
        if self.use_binary_responses {
            return Ok(Some(SourceFileResponse {
                data: raw_binary(&data),
            }));
        }
        Ok(Some(SourceFileResponse {
            data: base64_std_encode(&data),
        }))
    }

    pub fn handle_type_to_string(
        &self,
        params: &TypeToTypeNodeParams,
    ) -> Result<String> {
        let mut setup = self.setup_checker(params.snapshot, params.project.clone())?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let mut enclosing_declaration: Option<*const Node> = None;
        if !params.location.is_empty() {
            enclosing_declaration =
                Some(setup.sd.resolve_node_handle(setup.program, params.location.clone())?);
        }
        if params.flags != 0 {
            return Ok(setup.checker.type_to_string_ex(
                unsafe { &*t },
                node_builder_type_format_flags(params.flags),
            ));
        }
        Ok(setup.checker.type_to_string_ex(
            unsafe { &*t },
            TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE
                .union(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE),
        ))
    }

    pub fn handle_transpile(
        &self,
        params: &TranspileParams,
        declaration: bool,
    ) -> Result<TranspileOutputResponse> {
        crate::api::mig::m5m::transpile_output(&params.input, params.options.clone(), declaration)
    }

    pub fn handle_transpile_from_file(
        &self,
        params: &TranspileFromFileParams,
        declaration: bool,
    ) -> Result<TranspileOutputResponse> {
        let file_name = tsox_core::tspath::get_normalized_absolute_path(
            &params.file_name,
            self.project_session.current_directory(),
        );
        let fs = self.project_session.fs().expect("project session fs");
        let input = match fs.read_file(&file_name) {
            Some(input) => input,
            None => {
                return Err(err_client_error(&format!(
                    "could not read file {file_name:?}"
                )))
            }
        };
        let mut options = params.options.clone();
        options.file_name = file_name;
        crate::api::mig::m5m::transpile_output(&input, options, declaration)
    }

    pub fn release_open_refs(&mut self) {
        let mut update_guard = self.update_mu.lock().unwrap();
        if self.open_projects.is_empty() && self.open_files.is_empty() {
            return;
        }
        let mut api_request = APISnapshotRequest::default();
        if !self.open_projects.is_empty() {
            api_request.close_projects = Some(self.open_projects.to_hash_set());
        }
        if !self.open_files.is_empty() {
            api_request.close_files = Some(self.open_files.to_hash_set());
        }
        let snapshot = match self
            .project_session
            .api_update(&FileChangeSummary::default(), &api_request)
        {
            Ok(snapshot) => snapshot,
            Err(_) => return,
        };
        snapshot.deref_snapshot();
        self.open_projects.clear();
        self.open_files.clear();
        drop(update_guard);
    }

    pub fn to_path(&self, file_name: &str) -> TspathPath {
        let fs = self.project_session.fs().expect("project session fs");
        tspath_to_path(
            file_name,
            self.project_session.current_directory(),
            fs.use_case_sensitive_file_names(),
        )
    }

    pub fn to_file_change_summary(
        &self,
        changes: Option<&APIFileChanges>,
    ) -> FileChangeSummary {
        let changes = match changes {
            Some(changes) => changes,
            None => return FileChangeSummary::default(),
        };
        let mut summary = FileChangeSummary::default();
        if changes.invalidate_all {
            summary.invalidate_all = true;
            summary.includes_watch_change_outside_node_modules = true;
            return summary;
        }
        let cwd = self.project_session.current_directory();
        for doc in &changes.changed {
            let uri = doc.to_uri(&cwd);
            summary.changed.insert(DocumentUri(uri));
        }
        for doc in &changes.created {
            let uri = doc.to_uri(&cwd);
            summary.created.insert(DocumentUri(uri));
        }
        for doc in &changes.deleted {
            let uri = doc.to_uri(&cwd);
            summary.deleted.insert(DocumentUri(uri));
        }
        if summary.changed.len() + summary.created.len() + summary.deleted.len() > 0 {
            summary.includes_watch_change_outside_node_modules = true;
        }
        summary
    }

    pub fn handle_update_snapshot(
        &mut self,
        params: &UpdateSnapshotParams,
    ) -> Result<UpdateSnapshotResponse> {
        let _update_guard = self.update_mu.lock().unwrap();
        let file_changes = self.to_file_change_summary(params.file_changes.as_ref());
        let mut api_request = APISnapshotRequest::default();

        let mut opened_projects = Vec::new();
        for p in &params.open_projects {
            let config_file_name = p.to_absolute_file_name(&self.project_session.current_directory());
            let config_path = self.to_path(&config_file_name);
            if self.open_projects.has(&config_path) {
                continue;
            }
            api_request.open_projects.get_or_insert_with(Default::default).insert(config_file_name.clone());
            opened_projects.push(config_path);
        }

        let mut closed_projects = Vec::new();
        for p in &params.close_projects {
            let config_path =
                self.to_path(&p.to_absolute_file_name(&self.project_session.current_directory()));
            if !self.open_projects.has(&config_path) {
                continue;
            }
            api_request.close_projects.get_or_insert_with(Default::default).insert(config_path.clone());
            closed_projects.push(config_path);
        }

        let mut opened_files = Vec::new();
        for f in &params.open_files {
            let uri = f.to_uri(&self.project_session.current_directory());
            let path = self.to_path(&DocumentUri(uri.clone()).file_name());
            if self.open_files.has(&path) {
                continue;
            }
            api_request.open_files.get_or_insert_with(Default::default).insert(DocumentUri(uri));
            opened_files.push(path);
        }

        let mut closed_files = Vec::new();
        for f in &params.close_files {
            let path = self.to_path(
                &DocumentUri(f.to_uri(&self.project_session.current_directory())).file_name(),
            );
            if !self.open_files.has(&path) {
                continue;
            }
            api_request.close_files.get_or_insert_with(Default::default).insert(path.clone());
            closed_files.push(path);
        }

        let snapshot = match self.project_session.api_update(&file_changes, &api_request) {
            Ok(snapshot) => snapshot,
            Err(err) => {
                return Err(err_client_error(&format!("failed to update snapshot: {err}")))
            }
        };

        for config_path in opened_projects {
            self.open_projects.add(config_path);
        }
        for config_path in closed_projects {
            self.open_projects.delete(&config_path);
        }
        for path in opened_files {
            self.open_files.add(path);
        }
        for path in closed_files {
            self.open_files.delete(&path);
        }

        let handle = snapshot_handle(&snapshot);
        let prev_snapshot = {
            let guard = self.snapshots_mu.read().unwrap();
            guard.latest_snapshot
        };

        let projects = snapshot
            .project_collection
            .as_ref()
            .expect("snapshot missing project collection")
            .projects();
        let mut project_responses = Vec::with_capacity(projects.len());
        for proj in projects {
            if proj.command_line.is_none() {
                continue;
            }
            project_responses.push(new_project_response(proj));
        }

        let changes = match self.get_snapshot_data(prev_snapshot) {
            Ok(prev_sd) => compute_snapshot_changes(&unsafe { &*prev_sd }.snapshot, &snapshot),
            Err(_) => None,
        };

        {
            let mut guard = self.snapshots_mu.write().unwrap();
            if guard.snapshots.contains_key(&handle) {
                snapshot.deref_snapshot();
                guard.snapshots.get_mut(&handle).unwrap().ref_count += 1;
            } else {
                guard.snapshots.insert(
                    handle,
                    Box::new(SnapshotData {
                        snapshot: Arc::from(snapshot),
                        ref_count: 1,
                        symbol_registry: Default::default(),
                        symbol_canonical_projects: Default::default(),
                        project_registries: Default::default(),
                    }),
                );
            }
            guard.latest_snapshot = handle;
        }

        Ok(UpdateSnapshotResponse {
            snapshot: handle,
            projects: project_responses,
            changes,
        })
    }

    pub fn handle_update_temporary_snapshot(
        &mut self,
        params: &UpdateTemporarySnapshotParams,
    ) -> Result<UpdateSnapshotResponse> {
        let base_sd = self.retain_snapshot_data(params.snapshot)?;
        let uri = params
            .file
            .to_uri(&self.project_session.current_directory());

        let result = (|| -> Result<UpdateSnapshotResponse> {
            let snapshot = match self.project_session.api_update_temporary(
                &unsafe { &*base_sd }.snapshot,
                DocumentUri(uri),
                &params.new_text,
            ) {
                Ok(snapshot) => snapshot,
                Err(err) => {
                    return Err(err_client_error(&format!(
                        "failed to update temporary snapshot: {err}"
                    )))
                }
            };
            let handle = snapshot_handle(&snapshot);
            {
                let mut guard = self.snapshots_mu.write().unwrap();
                if guard.snapshots.contains_key(&handle) {
                    snapshot.deref_snapshot();
                    guard.snapshots.get_mut(&handle).unwrap().ref_count += 1;
                } else {
                    guard.snapshots.insert(
                        handle,
                        Box::new(SnapshotData {
                            snapshot: Arc::clone(&snapshot),
                            ref_count: 1,
                            symbol_registry: Default::default(),
                            symbol_canonical_projects: Default::default(),
                            project_registries: Default::default(),
                        }),
                    );
                }
            }

            let projects = snapshot
                .project_collection
                .as_ref()
                .expect("snapshot missing project collection")
                .projects();
            let mut project_responses = Vec::with_capacity(projects.len());
            for proj in projects {
                if proj.command_line.is_none() {
                    continue;
                }
                project_responses.push(new_project_response(proj));
            }

            let changes = compute_snapshot_changes(&unsafe { &*base_sd }.snapshot, &snapshot);

            Ok(UpdateSnapshotResponse {
                snapshot: handle,
                projects: project_responses,
                changes,
            })
        })();
        let _ = self.release_snapshot(params.snapshot);
        result
    }
}
