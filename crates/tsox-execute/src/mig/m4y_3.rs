use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;

use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath;
use tsox_core::tspath::Path;
use tsox_frontend::ast;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3e::RepopulateDiagnosticInfo;
use tsox_frontend::ast::SourceFile;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use super::m4y_2::{
    is_build_info_file_name_default_library, BuildInfo, BuildInfoDiagnostic,
    BuildInfoDiagnosticsOfFile, BuildInfoFileId, BuildInfoFileIdListId, BuildInfoRepopulateInfo,
};
use super::m4z2::Program;
use super::m4z2_2::{
    get_file_emit_kind, get_pending_emit_kind, BuildInfoDiagnosticWithFileName,
    DiagnosticsOrBuildInfoDiagnosticsWithFileName, EmitSignature, FileEmitKind, FileInfo, Snapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmitOnly {
    #[default]
    All,
    Js,
    Dts,
    BuilderSignature,
}

pub type WriteFileFn = Arc<dyn Fn(&str, &str, &WriteFileData) -> std::io::Result<()>>;

#[derive(Default)]
pub struct EmitOptions {
    pub target_source_files: Option<Vec<Arc<SourceFile>>>,
    pub emit_only: EmitOnly,
    pub force_emit: bool,
    pub write_file: Option<WriteFileFn>,
}

impl Clone for EmitOptions {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        Self {
            target_source_files: self.target_source_files.clone(),
            emit_only: self.emit_only,
            force_emit: self.force_emit,
            write_file: self.write_file.clone(),
        }
    }
}

#[derive(Default, Clone)]
pub struct EmitResult {
    pub emit_skipped: bool,
    pub emitted_files: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Default)]
pub struct WriteFileData {
    pub source_map_url_pos: i32,
    pub build_info: Option<BuildInfo>,
    pub diagnostics: Vec<Diagnostic>,
    pub skipped_dts_write: bool,
    pub source_file: Option<Arc<SourceFile>>,
}

pub fn combine_emit_results(results: &[EmitResult]) -> EmitResult { ::tsox_core::fntrace::enter("combine_emit_results"); 
    let mut result = EmitResult::default();
    for emit_result in results {
        if emit_result.emit_skipped {
            result.emit_skipped = true;
        }
        result
            .diagnostics
            .extend(emit_result.diagnostics.iter().cloned());
        result
            .emitted_files
            .extend(emit_result.emitted_files.iter().cloned());
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignatureUpdateKind {
    #[default]
    ComputedDts,
    StoredAtEmit,
    UsedVersion,
}

#[derive(Default)]
pub struct UpdatedSignature {
    pub mu: Mutex<()>,
    pub signature: Mutex<String>,
    pub kind: Mutex<SignatureUpdateKind>,
}

#[derive(Clone)]
pub struct DtsMayChange(pub OrderedMap<Path, FileEmitKind>);

impl std::ops::Deref for DtsMayChange {
    type Target = OrderedMap<Path, FileEmitKind>;

    fn deref(&self) -> &Self::Target { ::tsox_core::fntrace::enter("deref"); 
        &self.0
    }
}

impl std::ops::DerefMut for DtsMayChange {
    fn deref_mut(&mut self) -> &mut Self::Target { ::tsox_core::fntrace::enter("deref_mut"); 
        &mut self.0
    }
}

impl DtsMayChange {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        DtsMayChange(OrderedMap::new())
    }

    pub fn add_file_to_affected_files_pending_emit(
        &mut self,
        file_path: Path,
        emit_kind: FileEmitKind,
    ) { ::tsox_core::fntrace::enter("add_file_to_affected_files_pending_emit"); 
        self.insert(file_path, emit_kind);
    }
}

#[derive(Default, Clone)]
pub struct EmitUpdate {
    pub pending_kind: FileEmitKind,
    pub result: Option<EmitResult>,
    pub dts_errors_from_cache: bool,
}

pub struct EmitFilesHandler<'a> {
    pub program: &'a Program,
    pub is_for_dts_errors: bool,
    pub signatures: SyncMap<Path, String>,
    pub emit_signatures: SyncMap<Path, EmitSignature>,
    pub latest_changed_dts_files: SyncMap<Path, String>,
    pub deleted_pending_kinds: Set<Path>,
    pub emit_updates: SyncMap<Path, EmitUpdate>,
    pub has_emit_diagnostics: std::sync::atomic::AtomicBool,
}

impl<'a> EmitFilesHandler<'a> {
    pub fn new(program: &'a Program, is_for_dts_errors: bool) -> Self { ::tsox_core::fntrace::enter("new"); 
        EmitFilesHandler {
            program,
            is_for_dts_errors,
            signatures: SyncMap::new(),
            emit_signatures: SyncMap::new(),
            latest_changed_dts_files: SyncMap::new(),
            deleted_pending_kinds: Set::new(),
            emit_updates: SyncMap::new(),
            has_emit_diagnostics: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

fn source_file_path(program: &tsox_compile::compiler::Program, file: &SourceFile) -> Path { ::tsox_core::fntrace::enter("source_file_path"); 
    tspath::to_path(
        &file.file_name,
        program.get_current_directory(),
        program.use_case_sensitive_file_names(),
    )
}

fn all_files_excluding_default_library_file(
    program: &tsox_compile::compiler::Program,
    first_source_file: Option<&Arc<SourceFile>>,
) -> Vec<Arc<SourceFile>> { ::tsox_core::fntrace::enter("all_files_excluding_default_library_file"); 
    let files = program.get_source_files();
    let mut result: Vec<Arc<SourceFile>> = Vec::with_capacity(files.len());
    let mut add_source_file =
        |file: &Arc<SourceFile>, all_files: &mut Vec<Arc<SourceFile>>| {
            if !program.is_source_file_default_library(
                source_file_path(program, file).as_str(),
            ) {
                all_files.push(file.clone());
            }
        };
    if let Some(first_source_file) = first_source_file {
        add_source_file(first_source_file, &mut result);
    }
    for file in &files {
        let is_first = first_source_file
            .map(|first| std::ptr::eq(first.as_ref(), file.as_ref()))
            .unwrap_or(false);
        if !is_first {
            add_source_file(file, &mut result);
        }
    }
    result
}

pub fn build_info_to_snapshot(
    build_info: &BuildInfo,
    config: &ParsedCommandLine,
    host: &dyn tsox_compile::compiler::CompilerHost,
) -> Snapshot { ::tsox_core::fntrace::enter("build_info_to_snapshot"); 
    let mut to = ToSnapshot {
        build_info,
        build_info_directory: tspath::get_directory_path(&tspath::get_normalized_absolute_path(
            &config.get_build_info_file_name(),
            &config.get_current_directory(),
        )),
        snapshot: Snapshot::default(),
        file_paths: Vec::with_capacity(build_info.file_names.len()),
        file_path_set: Vec::with_capacity(build_info.file_ids_list.len()),
    };
    to.file_paths = build_info
        .file_names
        .iter()
        .map(|file_name| {
            if is_build_info_file_name_default_library(file_name) {
                return tspath::to_path(
                    &tspath::combine_paths(&host.default_library_path(), &[file_name]),
                    &host.current_directory(),
                    host.fs().use_case_sensitive_file_names(),
                );
            }
            tspath::to_path(
                file_name,
                &to.build_info_directory,
                config.use_case_sensitive_file_names(),
            )
        })
        .collect();
    to.file_path_set = build_info
        .file_ids_list
        .iter()
        .map(|file_id_list| {
            let mut file_set: HashSet<Path> = HashSet::with_capacity(file_id_list.len());
            for file_id in file_id_list {
                file_set.insert(to.to_file_path(*file_id));
            }
            file_set
        })
        .collect();
    to.set_compiler_options();
    to.set_file_info_and_emit_signatures();
    to.set_referenced_map();
    to.set_change_file_set();
    to.set_semantic_diagnostics();
    to.set_emit_diagnostics();
    to.set_affected_files_pending_emit();
    if !build_info.latest_changed_dts_file.is_empty() {
        *to.snapshot.latest_changed_dts_file.lock().unwrap() =
            to.to_absolute_path(&build_info.latest_changed_dts_file);
    }
    *to.snapshot.has_errors.lock().unwrap() = if build_info.errors {
        Tristate::True
    } else {
        Tristate::False
    };
    *to.snapshot.has_semantic_errors.lock().unwrap() = build_info.semantic_errors;
    *to.snapshot.check_pending.lock().unwrap() = build_info.check_pending;
    to.set_package_jsons();
    to.snapshot
}

pub struct ToSnapshot<'a> {
    pub build_info: &'a BuildInfo,
    pub build_info_directory: String,
    pub snapshot: Snapshot,
    pub file_paths: Vec<Path>,
    pub file_path_set: Vec<HashSet<Path>>,
}

impl<'a> ToSnapshot<'a> {
    pub fn to_absolute_path(&self, path: &str) -> String { ::tsox_core::fntrace::enter("to_absolute_path"); 
        tspath::get_normalized_absolute_path(path, &self.build_info_directory)
    }

    pub fn to_file_path(&self, file_id: BuildInfoFileId) -> Path { ::tsox_core::fntrace::enter("to_file_path"); 
        self.file_paths[(file_id - 1) as usize].clone()
    }

    pub fn to_file_path_set(&self, file_id_list_id: BuildInfoFileIdListId) -> &HashSet<Path> { ::tsox_core::fntrace::enter("to_file_path_set"); 
        &self.file_path_set[(file_id_list_id - 1) as usize]
    }

    pub fn to_build_info_diagnostics_with_file_name(
        &self,
        diagnostics: &[Box<BuildInfoDiagnostic>],
    ) -> Vec<Box<BuildInfoDiagnosticWithFileName>> { ::tsox_core::fntrace::enter("to_build_info_diagnostics_with_file_name"); 
        diagnostics
            .iter()
            .map(|d| {
                let file = if d.file != 0 {
                    self.to_file_path(d.file)
                } else {
                    Path::default()
                };
                Box::new(BuildInfoDiagnosticWithFileName {
                    file,
                    no_file: d.no_file,
                    pos: d.pos,
                    end: d.end,
                    code: d.code,
                    category: d.category,
                    source: d.source.clone(),
                    message_text: d.message_text.clone(),
                    message_key: Box::leak(d.message_key.clone().into_boxed_str()),
                    message_args: d.message_args.clone(),
                    message_chain: self.to_build_info_diagnostics_with_file_name(&d.message_chain),
                    related_information: self
                        .to_build_info_diagnostics_with_file_name(&d.related_information),
                    reports_unnecessary: d.reports_unnecessary,
                    reports_deprecated: d.reports_deprecated,
                    skipped_on_no_emit: d.skipped_on_no_emit,
                    repopulate_info: d
                        .repopulate_info
                        .as_ref()
                        .map(from_build_info_repopulate_info),
                })
            })
            .collect()
    }

    pub fn to_diagnostics_or_build_info_diagnostics_with_file_name(
        &self,
        dig: &BuildInfoDiagnosticsOfFile,
    ) -> DiagnosticsOrBuildInfoDiagnosticsWithFileName { ::tsox_core::fntrace::enter("to_diagnostics_or_build_info_diagnostics_with_file_name"); 
        DiagnosticsOrBuildInfoDiagnosticsWithFileName {
            build_info_diagnostics: self
                .to_build_info_diagnostics_with_file_name(&dig.diagnostics),
            ..Default::default()
        }
    }

    pub fn set_compiler_options(&mut self) { ::tsox_core::fntrace::enter("set_compiler_options"); 
        self.snapshot.options =
            Some(self.build_info.get_compiler_options(&self.build_info_directory));
    }

    pub fn set_file_info_and_emit_signatures(&mut self) { ::tsox_core::fntrace::enter("set_file_info_and_emit_signatures"); 
        let is_composite = self
            .snapshot
            .options
            .as_ref()
            .map(|o| o.composite.is_true())
            .unwrap_or(false);
        for (index, build_info_file_info) in self.build_info.file_infos.iter().enumerate() {
            let path = self.to_file_path(index as BuildInfoFileId + 1);
            let info = build_info_file_info.get_file_info().unwrap_or_default();
            let has_signature = !info.signature.is_empty();
            self.snapshot.file_infos.store(path.clone(), info.clone());
            if has_signature && is_composite {
                self.snapshot.emit_signatures.store(
                    path,
                    EmitSignature {
                        signature: info.signature.clone(),
                        signature_with_different_options: Vec::new(),
                    },
                );
            }
        }
        for value in &self.build_info.emit_signatures {
            if value.no_emit_signature() {
                self.snapshot
                    .emit_signatures
                    .delete(&self.to_file_path(value.file_id));
            } else {
                let path = self.to_file_path(value.file_id);
                self.snapshot
                    .emit_signatures
                    .store(path.clone(), value.to_emit_signature(&path, &self.snapshot.emit_signatures));
            }
        }
    }

    pub fn set_referenced_map(&mut self) { ::tsox_core::fntrace::enter("set_referenced_map"); 
        for entry in &self.build_info.referenced_map {
            let mut refs = Set::new();
            for path in self.to_file_path_set(entry.file_id_list_id) {
                refs.add(path.clone());
            }
            self.snapshot
                .referenced_map
                .store_references(self.to_file_path(entry.file_id), Arc::new(refs));
        }
    }

    pub fn set_change_file_set(&mut self) { ::tsox_core::fntrace::enter("set_change_file_set"); 
        for file_id in &self.build_info.change_file_set {
            let file_path = self.to_file_path(*file_id);
            self.snapshot
                .changed_files_set
                .lock()
                .unwrap()
                .insert(file_path);
        }
    }

    pub fn set_semantic_diagnostics(&mut self) { ::tsox_core::fntrace::enter("set_semantic_diagnostics"); 
        let paths: Vec<Path> = self
            .snapshot
            .file_infos
            .keys()
            .into_iter()
            .collect();
        for path in paths {
            if !self.snapshot.changed_files_set.lock().unwrap().contains(&path) {
                self.snapshot
                    .semantic_diagnostics_per_file
                    .store(path, DiagnosticsOrBuildInfoDiagnosticsWithFileName::default());
            }
        }
        for diagnostic in &self.build_info.semantic_diagnostics_per_file {
            if diagnostic.file_id != 0 {
                let file_path = self.to_file_path(diagnostic.file_id);
                self.snapshot
                    .semantic_diagnostics_per_file
                    .delete(&file_path);
            } else if let Some(diagnostics) = &diagnostic.diagnostics {
                let file_path = self.to_file_path(diagnostics.file_id);
                self.snapshot.semantic_diagnostics_per_file.store(
                    file_path,
                    self.to_diagnostics_or_build_info_diagnostics_with_file_name(diagnostics),
                );
            }
        }
    }

    pub fn set_emit_diagnostics(&mut self) { ::tsox_core::fntrace::enter("set_emit_diagnostics"); 
        for diagnostic in &self.build_info.emit_diagnostics_per_file {
            let file_path = self.to_file_path(diagnostic.file_id);
            self.snapshot.emit_diagnostics_per_file.store(
                file_path,
                self.to_diagnostics_or_build_info_diagnostics_with_file_name(diagnostic),
            );
        }
    }

    pub fn set_affected_files_pending_emit(&mut self) { ::tsox_core::fntrace::enter("set_affected_files_pending_emit"); 
        if self.build_info.affected_files_pending_emit.is_empty() {
            return;
        }
        let own_options_emit_kind = get_file_emit_kind(
            self.snapshot
                .options
                .as_ref()
                .unwrap_or(&CompilerOptions::default()),
        );
        for pending_emit in &self.build_info.affected_files_pending_emit {
            self.snapshot.affected_files_pending_emit.store(
                self.to_file_path(pending_emit.file_id),
                if pending_emit.emit_kind == FileEmitKind::None {
                    own_options_emit_kind
                } else {
                    pending_emit.emit_kind
                },
            );
        }
    }

    pub fn set_package_jsons(&mut self) { ::tsox_core::fntrace::enter("set_package_jsons"); 
        *self.snapshot.package_jsons.lock().unwrap() = Some(match &self.build_info.package_jsons {
            Some(package_jsons) => package_jsons
                .iter()
                .map(|p| self.to_absolute_path(p))
                .collect(),
            None => Vec::new(),
        });
        *self.snapshot.missing_package_jsons.lock().unwrap() = Some(
            match &self.build_info.missing_package_jsons {
                Some(missing_package_jsons) => missing_package_jsons
                    .iter()
                    .map(|p| self.to_absolute_path(p))
                    .collect(),
                None => Vec::new(),
            },
        );
    }
}

pub fn from_build_info_repopulate_info(info: &BuildInfoRepopulateInfo) -> RepopulateDiagnosticInfo { ::tsox_core::fntrace::enter("from_build_info_repopulate_info"); 
    RepopulateDiagnosticInfo {
        kind: info.kind,
        module_reference: info.module_reference.clone(),
        mode: info.mode,
        package_name: info.package_name.clone(),
    }
}

pub struct AffectedFilesHandler<'a> {
    pub program: &'a Program,
    pub has_all_files_excluding_default_library_file: std::sync::atomic::AtomicBool,
    pub updated_signatures: SyncMap<Path, Arc<UpdatedSignature>>,
    pub dts_may_change: Vec<DtsMayChange>,
    pub files_to_remove_diagnostics: HashSet<Path>,
    pub cleaned_diagnostics_of_lib_files: Once,
    pub seen_file_and_references: SyncMap<Path, bool>,
}

impl<'a> AffectedFilesHandler<'a> {
    pub fn get_dts_may_change(
        &mut self,
        affected_file_path: Path,
        affected_file_emit_kind: FileEmitKind,
    ) -> DtsMayChange { ::tsox_core::fntrace::enter("get_dts_may_change"); 
        let mut result = DtsMayChange::new();
        result.insert(affected_file_path, affected_file_emit_kind);
        self.dts_may_change.push(result.clone());
        result
    }

    pub fn is_changed_signature(&self, path: &Path) -> bool { ::tsox_core::fntrace::enter("is_changed_signature"); 
        let Some(new_signature) = self.updated_signatures.load(path) else {
            return false;
        };
        let Some(old_info) = self.program.snapshot.file_infos.load(path) else {
            return false;
        };
        *new_signature.signature.lock().unwrap() != old_info.signature
    }

    pub fn remove_semantic_diagnostics_of(&mut self, path: Path) { ::tsox_core::fntrace::enter("remove_semantic_diagnostics_of"); 
        self.files_to_remove_diagnostics.insert(path);
    }

    pub fn remove_diagnostics_of_library_files(&mut self) { ::tsox_core::fntrace::enter("remove_diagnostics_of_library_files"); 
        if self.cleaned_diagnostics_of_lib_files.is_completed() {
            return;
        }
        for file in self.program.get_source_files() {
            let path = source_file_path(self.program.program(), &file);
            if self
                .program
                .program()
                .is_source_file_default_library(path.as_str())
                && !self.program.program().skip_type_checking(&file, true)
            {
                self.remove_semantic_diagnostics_of(path);
            }
        }
        self.cleaned_diagnostics_of_lib_files.call_once(|| {});
    }

    pub fn compute_dts_signature(&mut self, file: &Arc<SourceFile>) -> String { ::tsox_core::fntrace::enter("compute_dts_signature"); 
        let mut signature = String::new();
        let _done = self.program.begin_nested_emit();
        let captured = &mut signature as *mut String;
        let program = self.program;
        let program_ptr = program as *const Program;
        let file_arc = Arc::clone(file);
        program.emit(
            EmitOptions {
                target_source_files: Some(vec![file_arc.clone()]),
                emit_only: EmitOnly::BuilderSignature,
                write_file: Some(Arc::new(
                    move |file_name: &str, text: &str, data: &WriteFileData| {
                        if !tspath::is_declaration_file_name(file_name) {
                            panic!(
                                "File extension for signature expected to be dts, got : {file_name}"
                            );
                        }
                        let program = unsafe { &*program_ptr };
                        let captured = unsafe { &mut *captured };
                        *captured = program
                            .snapshot
                            .compute_signature_with_diagnostics(&file_arc, text, data);
                        Ok(())
                    },
                )),
                ..Default::default()
            },
        );
        signature
    }

    pub fn update_shape_signature(
        &mut self,
        file: &Arc<SourceFile>,
        use_file_version_as_signature: bool,
    ) -> bool { ::tsox_core::fntrace::enter("update_shape_signature"); 
        let update = Arc::new(UpdatedSignature::default());
        let _lock = update.mu.lock().unwrap();
        let path = source_file_path(self.program.program(), file);
        let (existing, loaded) = self
            .updated_signatures
            .load_or_store(path.clone(), update.clone());
        if loaded {
            let _existing_lock = existing.mu.lock().unwrap();
            return false;
        }

        let Some(info) = self.program.snapshot.file_infos.load(&path) else {
            return false;
        };
        let prev_signature = info.signature.clone();
        if !file.is_declaration_file
            && !ast::is_json_source_file(file)
            && !use_file_version_as_signature
        {
            *update.signature.lock().unwrap() = self.compute_dts_signature(file);
        }
        let mut signature = update.signature.lock().unwrap();
        if signature.is_empty() {
            *signature = info.version.clone();
            *update.kind.lock().unwrap() = SignatureUpdateKind::UsedVersion;
        }
        *signature != prev_signature
    }

    pub fn get_files_affected_by(&mut self, path: &Path) -> Vec<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_files_affected_by"); 
        let Some(file) = self.program.program().get_source_file_by_path(path.as_str()) else {
            return Vec::new();
        };

        if !self.update_shape_signature(&file, false) {
            return vec![file];
        }

        if let Some(info) = self
            .program
            .snapshot
            .file_infos
            .load(&source_file_path(self.program.program(), &file))
            && info.affects_global_scope
        {
            self.has_all_files_excluding_default_library_file
                .store(true, std::sync::atomic::Ordering::SeqCst);
            return all_files_excluding_default_library_file(
                self.program.program(),
                Some(&file),
            );
        }

        if self
            .program
            .snapshot
            .options
            .as_ref()
            .map(|o| o.isolated_modules.is_true())
            .unwrap_or(false)
        {
            return vec![file];
        }

        let seen_file_names_map = Self::for_each_file_referenced_by(self.program, &file, |current_file, _current_path| {
            if current_file
                .as_ref()
                .map(|f| self.update_shape_signature(f, false))
                .unwrap_or(false)
            {
                return (true, false);
            }
            (false, false)
        });
        seen_file_names_map
            .into_values()
            .flatten()
            .collect()
    }

    pub fn for_each_file_referenced_by(
        program: &Program,
        file: &Arc<SourceFile>,
        mut f: impl FnMut(Option<Arc<SourceFile>>, &Path) -> (bool, bool),
    ) -> HashMap<Path, Option<Arc<SourceFile>>> { ::tsox_core::fntrace::enter("for_each_file_referenced_by"); 
        let mut seen_file_names_map: HashMap<Path, Option<Arc<SourceFile>>> = HashMap::new();
        seen_file_names_map.insert(source_file_path(program.program(), file), Some(file.clone()));
        let mut queue: Vec<Path> = program
            .snapshot
            .referenced_map
            .get_referenced_by(&source_file_path(program.program(), file))
            .into_iter()
            .collect();
        while !queue.is_empty() {
            let current_path = queue.pop().unwrap();
            if !seen_file_names_map.contains_key(&current_path) {
                let current_file = program.get_source_file_by_path(current_path.as_str());
                seen_file_names_map.insert(current_path.clone(), current_file.clone());
                let (queue_for_file, fast_return) = f(current_file.clone(), &current_path);
                if fast_return {
                    return seen_file_names_map;
                }
                if queue_for_file
                    && let Some(current_file) = &current_file
                {
                    queue.extend(
                        program
                            .snapshot
                            .referenced_map
                            .get_referenced_by(&source_file_path(
                                program.program(),
                                current_file,
                            )),
                    );
                }
            }
        }
        seen_file_names_map
    }

    pub fn handle_dts_may_change_of_affected_file(
        &mut self,
        dts_may_change: &mut DtsMayChange,
        affected_file: &Arc<SourceFile>,
    ) { ::tsox_core::fntrace::enter("handle_dts_may_change_of_affected_file"); 
        let affected_file_path = source_file_path(self.program.program(), affected_file);
        self.remove_semantic_diagnostics_of(affected_file_path.clone());

        if self
            .has_all_files_excluding_default_library_file
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            self.remove_diagnostics_of_library_files();
            self.update_shape_signature(affected_file, false);
            return;
        }

        if self
            .program
            .snapshot
            .options
            .as_ref()
            .map(|o| o.assume_changes_only_affect_direct_dependencies.is_true())
            .unwrap_or(false)
        {
            return;
        }

        if !self
            .program
            .snapshot
            .changed_files_set
            .lock()
            .unwrap()
            .contains(&affected_file_path)
            || !self.is_changed_signature(&affected_file_path)
        {
            return;
        }

        if self
            .program
            .snapshot
            .options
            .as_ref()
            .map(|o| o.isolated_modules.is_true())
            .unwrap_or(false)
        {
            Self::for_each_file_referenced_by(
                self.program,
                affected_file,
                |current_file, current_path| {
                    if let Some(current_path) = Some(current_path.clone())
                        && self.handle_dts_may_change_of_global_scope(dts_may_change, current_path.clone(), false)
                    {
                        return (false, true);
                    }
                    self.handle_dts_may_change_of(dts_may_change, current_path.clone(), false);
                    if self.is_changed_signature(&current_path) {
                        return (true, false);
                    }
                    (false, false)
                },
            );
        }

        let mut invalidate_js_files = false;
        let mut type_checker: Option<std::sync::MutexGuard<'_, tsox_checker::checker::Checker>> =
            None;
        if let Some(symbol) = tsox_checker::checker::mig::m1a::symbol_of_node(&affected_file.node)
        {
            for exported in symbol.exports.entries.values() {
                if exported.flags & ast::SymbolFlags::ConstEnum != ast::SymbolFlags::None {
                    invalidate_js_files = true;
                    break;
                }
                if type_checker.is_none() {
                    type_checker = Some(
                        self.program
                            .program()
                            .get_type_checker_for_file_exclusive(affected_file),
                    );
                }
                let aliased = type_checker.as_ref().unwrap().skip_alias(exported);
                if Arc::ptr_eq(&aliased, exported) {
                    continue;
                }
                if aliased.flags & ast::SymbolFlags::ConstEnum != ast::SymbolFlags::None
                    && aliased
                        .declarations
                        .iter()
                        .any(|d| {
                            ast::get_source_file_of_node(d)
                                .is_some_and(|sf| Arc::ptr_eq(&sf, &affected_file.node))
                        })
                {
                    invalidate_js_files = true;
                    break;
                }
            }
        }

        for file_referencing_changed_file in self
            .program
            .snapshot
            .referenced_map
            .get_referenced_by(&affected_file_path)
        {
            if self.handle_dts_may_change_of_global_scope(
                dts_may_change,
                file_referencing_changed_file.clone(),
                invalidate_js_files,
            ) {
                return;
            }
            for file_referencing_affected_file in self
                .program
                .snapshot
                .referenced_map
                .get_referenced_by(&file_referencing_changed_file)
            {
                if self.handle_dts_may_change_of_file_and_references(
                    dts_may_change,
                    file_referencing_affected_file.clone(),
                    invalidate_js_files,
                ) {
                    return;
                }
            }
        }
    }

    pub fn handle_dts_may_change_of_file_and_references(
        &mut self,
        dts_may_change: &mut DtsMayChange,
        file_path: Path,
        invalidate_js_files: bool,
    ) -> bool { ::tsox_core::fntrace::enter("handle_dts_may_change_of_file_and_references"); 
        let (existing, loaded) =
            self.seen_file_and_references
                .load_or_store(file_path.clone(), invalidate_js_files);
        if loaded && (existing || !invalidate_js_files) {
            return false;
        } else if loaded && invalidate_js_files {
            self.seen_file_and_references.store(file_path.clone(), true);
        }

        if self.handle_dts_may_change_of_global_scope(dts_may_change, file_path.clone(), invalidate_js_files) {
            return true;
        }
        self.handle_dts_may_change_of(dts_may_change, file_path.clone(), invalidate_js_files);

        for referencing_file_path in self
            .program
            .snapshot
            .referenced_map
            .get_referenced_by(&file_path)
        {
            if self.handle_dts_may_change_of_file_and_references(
                dts_may_change,
                referencing_file_path.clone(),
                invalidate_js_files,
            ) {
                return true;
            }
        }
        false
    }

    pub fn handle_dts_may_change_of_global_scope(
        &mut self,
        dts_may_change: &mut DtsMayChange,
        file_path: Path,
        invalidate_js_files: bool,
    ) -> bool { ::tsox_core::fntrace::enter("handle_dts_may_change_of_global_scope"); 
        let is_affects_global_scope = self
            .program
            .snapshot
            .file_infos
            .load(&file_path)
            .map(|info| info.affects_global_scope)
            .unwrap_or(false);
        if !is_affects_global_scope {
            return false;
        }
        for file in all_files_excluding_default_library_file(self.program.program(), None) {
            self.handle_dts_may_change_of(
                dts_may_change,
                source_file_path(self.program.program(), &file),
                invalidate_js_files,
            );
        }
        self.remove_diagnostics_of_library_files();
        true
    }

    pub fn handle_dts_may_change_of(
        &mut self,
        dts_may_change: &mut DtsMayChange,
        path: Path,
        invalidate_js_files: bool,
    ) { ::tsox_core::fntrace::enter("handle_dts_may_change_of"); 
        if self
            .program
            .snapshot
            .changed_files_set
            .lock()
            .unwrap()
            .contains(&path)
        {
            return;
        }
        let Some(file) = self.program.program().get_source_file_by_path(path.as_str()) else {
            return;
        };
        self.remove_semantic_diagnostics_of(path.clone());
        self.update_shape_signature(&file, true);
        if invalidate_js_files {
            dts_may_change.add_file_to_affected_files_pending_emit(
                path,
                get_file_emit_kind(
                    self.program
                        .snapshot
                        .options
                        .as_ref()
                        .unwrap_or(&CompilerOptions::default()),
                ),
            );
        } else if self
            .program
            .snapshot
            .options
            .as_ref()
            .map(|o| o.get_emit_declarations())
            .unwrap_or(false)
        {
            dts_may_change.add_file_to_affected_files_pending_emit(
                path,
                if self
                    .program
                    .snapshot
                    .options
                    .as_ref()
                    .map(|o| o.declaration_map.is_true())
                    .unwrap_or(false)
                {
                    FileEmitKind::AllDts
                } else {
                    FileEmitKind::Dts
                },
            );
        }
    }

    pub fn update_snapshot(&mut self) { ::tsox_core::fntrace::enter("update_snapshot"); 
        self.updated_signatures.for_each(|file_path, update| {
            if let Some(mut info) = self.program.snapshot.file_infos.load(file_path) {
                info.signature = update.signature.lock().unwrap().clone();
                self.program.snapshot.file_infos.store(file_path.clone(), info);
                if let Some(testing_data) = &self.program.testing_data {
                    testing_data
                        .updated_signature_kinds
                        .store(file_path.clone(), *update.kind.lock().unwrap());
                }
            }
            true
        });
        for file in self.files_to_remove_diagnostics.iter() {
            self.program
                .snapshot
                .semantic_diagnostics_per_file
                .delete(file);
        }
        let changes = std::mem::take(&mut self.dts_may_change);
        for change in &changes {
            for (file_path, emit_kind) in change.iter() {
                let existing_kind = self
                    .program
                    .snapshot
                    .affected_files_pending_emit
                    .load(file_path)
                    .unwrap_or(FileEmitKind::None);
                self.program
                    .snapshot
                    .affected_files_pending_emit
                    .store(file_path.clone(), existing_kind | *emit_kind);
                if (*emit_kind & FileEmitKind::DtsErrors) != FileEmitKind::None {
                    self.program
                        .snapshot
                        .emit_diagnostics_per_file
                        .delete(file_path);
                }
                self.program
                    .snapshot
                    .build_info_emit_pending
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
        self.dts_may_change = changes;
        self.program
            .snapshot
            .changed_files_set
            .lock()
            .unwrap()
            .clear();
        self.program
            .snapshot
            .build_info_emit_pending
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

pub fn collect_all_affected_files(program: &Program) { ::tsox_core::fntrace::enter("collect_all_affected_files"); 
    if program.snapshot.changed_files_set.lock().unwrap().is_empty() {
        return;
    }

    let mut handler = AffectedFilesHandler {
        program,
        has_all_files_excluding_default_library_file: Default::default(),
        updated_signatures: SyncMap::new(),
        dts_may_change: Vec::new(),
        files_to_remove_diagnostics: HashSet::new(),
        cleaned_diagnostics_of_lib_files: Once::new(),
        seen_file_and_references: SyncMap::new(),
    };
    let mut result: Vec<Arc<SourceFile>> = Vec::new();
    let changed: Vec<Path> = program
        .snapshot
        .changed_files_set
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect();
    for file in changed {
        for affected_file in handler.get_files_affected_by(&file) {
            result.push(affected_file);
        }
    }

    let emit_kind = get_file_emit_kind(
        program
            .snapshot
            .options
            .as_ref()
            .unwrap_or(&CompilerOptions::default()),
    );
    for file in result {
        let mut dts_may_change =
            handler.get_dts_may_change(source_file_path(program.program(), &file), emit_kind);
        handler.handle_dts_may_change_of_affected_file(&mut dts_may_change, &file);
    }

    handler.update_snapshot();
}

impl<'a> EmitFilesHandler<'a> {
    pub fn get_pending_emit_kind_for_emit_options(
        &self,
        emit_kind: FileEmitKind,
        options: &EmitOptions,
    ) -> FileEmitKind { ::tsox_core::fntrace::enter("get_pending_emit_kind_for_emit_options"); 
        let mut pending_kind = get_pending_emit_kind(emit_kind, FileEmitKind::None);
        if options.emit_only == EmitOnly::Dts {
            pending_kind &= FileEmitKind::AllDts;
        }
        if self.is_for_dts_errors {
            pending_kind &= FileEmitKind::DtsErrors;
        }
        pending_kind
    }

    pub fn emit_all_affected_files(
        &mut self,
        options: &EmitOptions,
    ) -> Option<EmitResult> { ::tsox_core::fntrace::enter("emit_all_affected_files"); 
        if self.program.snapshot.can_use_incremental_state() {
            let results = self.emit_files_incremental(options);
            if self.is_for_dts_errors {
                if let Some(target_source_files) = &options.target_source_files {
                    let mut diagnostics = Vec::new();
                    for target_file in target_source_files {
                        if let Some(mut d) = self
                            .program
                            .snapshot
                            .emit_diagnostics_per_file
                            .load(&source_file_path(self.program.program(), target_file))
                        {
                            diagnostics
                                .extend(d.get_diagnostics(self.program.program(), target_file));
                        }
                    }
                    let result = EmitResult {
                        emit_skipped: true,
                        diagnostics,
                        ..Default::default()
                    };
                    self.update_has_emit_diagnostics(Some(&result));
                    return Some(result);
                }
                for result in &results {
                    self.update_has_emit_diagnostics(Some(result));
                }
                return Some(combine_emit_results(&results));
            }
            let mut result = combine_emit_results(&results);
            self.update_has_emit_diagnostics(Some(&result));
            self.emit_build_info(options, &mut result);
            return Some(result);
        } else if !self.is_for_dts_errors {
            let mut result = self.program.emit(self.get_emit_options(options));
            self.update_has_emit_diagnostics(Some(&result));
            self.update_snapshot();
            self.emit_build_info(options, &mut result);
            return Some(result);
        } else {
            let diagnostics = match &options.target_source_files {
                None => self.program.get_declaration_diagnostics(None),
                Some(target_source_files) => target_source_files
                    .iter()
                    .flat_map(|target_source_file| {
                        self.program.get_declaration_diagnostics(Some(target_source_file))
                    })
                    .collect(),
            };
            let result = EmitResult {
                emit_skipped: true,
                diagnostics,
                ..Default::default()
            };
            if !result.diagnostics.is_empty() {
                self.update_has_emit_diagnostics(Some(&result));
                self.program
                    .snapshot
                    .has_emit_diagnostics
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
            return Some(result);
        }
    }

    pub fn update_has_emit_diagnostics(&self, result: Option<&EmitResult>) { ::tsox_core::fntrace::enter("update_has_emit_diagnostics"); 
        if let Some(result) = result
            && !result.diagnostics.is_empty()
        {
            self.has_emit_diagnostics
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    pub fn emit_build_info(
        &mut self,
        options: &EmitOptions,
        result: &mut EmitResult,
    ) { ::tsox_core::fntrace::enter("emit_build_info"); 
        let Some(build_info_result) = self.program.emit_build_info(options) else {
            return;
        };
        result.diagnostics.extend(build_info_result.diagnostics);
        result.emitted_files.extend(build_info_result.emitted_files);
    }

    pub fn emit_files_incremental(
        &mut self,
        options: &EmitOptions,
    ) -> Vec<EmitResult> { ::tsox_core::fntrace::enter("emit_files_incremental"); 
        collect_all_affected_files(self.program);

        let mut pending: Vec<(Path, FileEmitKind)> = Vec::new();
        self.program
            .snapshot
            .affected_files_pending_emit
            .for_each(|path, emit_kind| {
                pending.push((path.clone(), *emit_kind));
                true
            });
        for (path, emit_kind) in pending {
            let Some(affected_file) = self
                .program
                .program()
                .get_source_file_by_path(path.as_str())
            else {
                self.deleted_pending_kinds.insert(path);
                continue;
            };
            if !self.program.program().source_file_may_be_emitted(&affected_file, false) {
                self.deleted_pending_kinds.insert(path);
                continue;
            }
            let pending_kind = self.get_pending_emit_kind_for_emit_options(emit_kind, options);
            if pending_kind != FileEmitKind::None {
                let mut emit_only = EmitOnly::All;
                if pending_kind & FileEmitKind::AllJs != FileEmitKind::None {
                    emit_only = EmitOnly::Js;
                }
                if pending_kind & FileEmitKind::AllDts != FileEmitKind::None {
                    emit_only = if emit_only == EmitOnly::Js {
                        EmitOnly::All
                    } else {
                        EmitOnly::Dts
                    };
                }
                let result = if !self.is_for_dts_errors {
                    Some(self.program.emit(
                        self.get_emit_options(&EmitOptions {
                            target_source_files: Some(vec![affected_file.clone()]),
                            emit_only,
                            write_file: options.write_file.clone(),
                            ..Default::default()
                        }),
                    ))
                } else {
                    Some(EmitResult {
                        emit_skipped: true,
                        diagnostics: self
                            .program
                            .get_declaration_diagnostics(Some(&affected_file)),
                        ..Default::default()
                    })
                };
                if let Some(result) = &result {
                    self.update_has_emit_diagnostics(Some(result));
                }
                self.emit_updates.store(
                    path.clone(),
                    EmitUpdate {
                        pending_kind: get_pending_emit_kind(emit_kind, pending_kind),
                        result,
                        dts_errors_from_cache: false,
                    },
                );
            }
        }

        let mut cached: Vec<(Path, DiagnosticsOrBuildInfoDiagnosticsWithFileName)> = Vec::new();
        self.program
            .snapshot
            .emit_diagnostics_per_file
            .for_each(|path, diagnostics| {
                if self.emit_updates.load(path).is_none() {
                    cached.push((path.clone(), diagnostics.clone()));
                }
                true
            });
        for (path, mut diagnostics) in cached {
            let Some(affected_file) = self
                .program
                .program()
                .get_source_file_by_path(path.as_str())
            else {
                self.deleted_pending_kinds.insert(path);
                continue;
            };
            if !self.program.program().source_file_may_be_emitted(&affected_file, false) {
                self.deleted_pending_kinds.insert(path);
                continue;
            }
            let pending_kind = self
                .program
                .snapshot
                .affected_files_pending_emit
                .load(&path)
                .unwrap_or(FileEmitKind::None);
            self.emit_updates.store(
                path.clone(),
                EmitUpdate {
                    pending_kind,
                    result: Some(EmitResult {
                        emit_skipped: true,
                        diagnostics: diagnostics.get_diagnostics(self.program.program(), &affected_file),
                        ..Default::default()
                    }),
                    dts_errors_from_cache: true,
                },
            );
        }

        self.update_snapshot()
    }

    pub fn get_emit_options(
        &self,
        options: &EmitOptions,
    ) -> EmitOptions { ::tsox_core::fntrace::enter("get_emit_options"); 
        if !self
            .program
            .snapshot
            .options
            .as_ref()
            .map(|o| o.get_emit_declarations())
            .unwrap_or(false)
        {
            return options.clone();
        }
        let can_use_incremental_state = self.program.snapshot.can_use_incremental_state();
        let outer_write_file = options.write_file.clone();
        EmitOptions {
            target_source_files: options.target_source_files.clone(),
            emit_only: options.emit_only,
            force_emit: options.force_emit,
            write_file: Some(Arc::new({
                let handler = self as *const EmitFilesHandler<'_> as *const ();
                move |file_name: &str, text: &str, data: &_| {
                    let handler: &EmitFilesHandler<'_> =
                        unsafe { &*(handler as *const EmitFilesHandler<'_>) };
                    let mut differs_only_in_map = false;
                    if tspath::is_declaration_file_name(file_name) && can_use_incremental_state {
                        let data_file = data
                            .source_file
                            .clone()
                            .expect("writeFile data must carry the source file being emitted");
                        let mut emit_signature = String::new();
                        if let Some(info) = handler
                            .program
                            .snapshot
                            .file_infos
                            .load(&source_file_path(handler.program.program(), &data_file))
                        {
                            if info.signature == info.version {
                                let signature = handler.program.snapshot.compute_signature_with_diagnostics(
                                    &data_file,
                                    text,
                                    data,
                                );
                                if data.diagnostics.is_empty() {
                                    emit_signature = signature.clone();
                                }
                                if signature != info.version {
                                    handler.signatures.store(
                                        source_file_path(handler.program.program(), &data_file),
                                        signature.clone(),
                                    );
                                }
                            }
                        }
                        if handler.skip_dts_output_of_composite(
                            &data_file,
                            file_name,
                            text,
                            data,
                            &mut emit_signature,
                            &mut differs_only_in_map,
                        ) {
                            return Ok(());
                        }
                    }

                    let mut a_time = None;
                    if differs_only_in_map {
                        a_time = Some(
                            handler
                                .program
                                .host
                                .as_deref()
                                .expect("incremental program host must be set")
                                .get_m_time(file_name),
                        );
                    }
                    let err = if let Some(write_file) = &outer_write_file {
                        write_file(file_name, text, data)
                    } else {
                        handler
                            .program
                            .program()
                            .host()
                            .fs()
                            .write_file(file_name, text)
                    };
                    if err.is_ok() && differs_only_in_map {
                        handler
                            .program
                            .host
                            .as_deref()
                            .expect("incremental program host must be set")
                            .set_m_time(file_name, a_time.expect("mtime must be captured"));
                    }
                    err
                }
            })),
            ..Default::default()
        }
    }
}

pub fn emit_files(
    program: &Program,
    options: &EmitOptions,
    is_for_dts_errors: bool,
) -> Option<EmitResult> { ::tsox_core::fntrace::enter("emit_files"); 
    let mut emit_handler = EmitFilesHandler::new(program, is_for_dts_errors);

    if !is_for_dts_errors && options.target_source_files.is_some() {
        let result = program.emit(emit_handler.get_emit_options(options));
        emit_handler.update_has_emit_diagnostics(Some(&result));
        emit_handler.update_snapshot();
        return Some(result);
    }

    emit_handler.emit_all_affected_files(options)
}

#[allow(unused)]
fn unused_refs(
    _: &OrderedMap<String, String>,
    _: &Snapshot,
    _: &FileInfo,
    _: &tsox_checker::checker::Checker,
) { ::tsox_core::fntrace::enter("unused_refs"); 
}
