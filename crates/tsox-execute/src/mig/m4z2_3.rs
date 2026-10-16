#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::tspath;
use tsox_core::tspath::Path;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3b_2;
use tsox_frontend::ast::SourceFile;

use crate::mig::m4z2::Program;
use crate::mig::m4z2_2::get_file_emit_kind;
use crate::mig::m4z2_2::get_pending_emit_kind_with_options;
use crate::mig::m4z2_2::repopulate_diagnostic_chain;
use crate::mig::m4z2_2::BuildInfoDiagnosticWithFileName;
use crate::mig::m4z2_2::DiagnosticsOrBuildInfoDiagnosticsWithFileName;
use crate::mig::m4z2_2::FileInfo;
use crate::mig::m4z2_2::Snapshot;
use crate::mig::m4z2_2::FileEmitKind;

pub fn program_to_snapshot(
    program: &tsox_compile::compiler::Program,
    old_program: Option<&Program>,
    hash_with_text: bool,
) -> Snapshot { ::tsox_core::fntrace::enter("program_to_snapshot"); 
    if let Some(old_program) = old_program {
        if let Some(old_inner) = old_program.program.as_deref() {
            if std::ptr::eq(old_inner, program) {
                return Snapshot::default();
            }
        }
    }
    let mut snapshot = Snapshot {
        options: Some(program.options().clone()),
        hash_with_text,
        check_pending: std::sync::Mutex::new(program.options().no_check.is_true()),
        ..Default::default()
    };
    let mut to = ToProgramSnapshot {
        program,
        old_program,
        snapshot: &mut snapshot,
        global_file_removed: false,
    };

    if to.snapshot.can_use_incremental_state() {
        to.reuse_from_old_program();
        to.compute_program_file_changes();
        to.handle_file_delete();
        to.handle_global_scope_change();
        to.handle_pending_emit();
        to.handle_pending_check();
    }
    snapshot
}

pub struct ToProgramSnapshot<'a> {
    pub program: &'a tsox_compile::compiler::Program,
    pub old_program: Option<&'a Program>,
    pub snapshot: &'a mut Snapshot,
    pub global_file_removed: bool,
}

impl<'a> ToProgramSnapshot<'a> {
    pub fn reuse_from_old_program(&mut self) { ::tsox_core::fntrace::enter("reuse_from_old_program"); 
        if let Some(old_program) = self.old_program {
            if self
                .snapshot
                .options
                .as_ref()
                .map(|o| o.composite.is_true())
                .unwrap_or(false)
            {
                *self.snapshot.latest_changed_dts_file.lock().unwrap() =
                    old_program.snapshot.latest_changed_dts_file.lock().unwrap().clone();
            }
            for key in old_program.snapshot.changed_files_set.lock().unwrap().iter() {
                self.snapshot.changed_files_set.lock().unwrap().insert(key.clone());
            }
            old_program
                .snapshot
                .affected_files_pending_emit
                .for_each(|key, emit_kind| {
                    self.snapshot
                        .affected_files_pending_emit
                        .store(key.clone(), *emit_kind);
                    true
                });
            self.snapshot.build_info_emit_pending = std::sync::atomic::AtomicBool::new(
                old_program
                    .snapshot
                    .build_info_emit_pending
                    .load(std::sync::atomic::Ordering::SeqCst),
            );
            self.snapshot.has_errors_from_old_state =
                *old_program.snapshot.has_errors.lock().unwrap();
            self.snapshot.has_semantic_errors_from_old_state =
                *old_program.snapshot.has_semantic_errors.lock().unwrap();
            self.snapshot.package_jsons_from_old_state =
                old_program.snapshot.package_jsons.lock().unwrap().clone();
            self.snapshot.missing_package_jsons_from_old_state =
                old_program.snapshot.missing_package_jsons.lock().unwrap().clone();
        } else {
            let is_incremental = self
                .snapshot
                .options
                .as_ref()
                .map(|o| o.is_incremental())
                .unwrap_or(false);
            self.snapshot.build_info_emit_pending =
                std::sync::atomic::AtomicBool::new(is_incremental);
        }
    }

    pub fn compute_program_file_changes(&mut self) { ::tsox_core::fntrace::enter("compute_program_file_changes"); 
        let can_copy_semantic_diagnostics = self.old_program.is_some()
            && !tsox_tsoptions::mig::m5h_2::compiler_options_affect_semantic_diagnostics(
                self.old_program.unwrap().snapshot.options.as_ref(),
                self.snapshot.options.as_ref(),
            );
        let can_copy_emit_signatures = self
            .snapshot
            .options
            .as_ref()
            .map(|o| o.composite.is_true())
            .unwrap_or(false)
            && self.old_program.is_some()
            && !tsox_tsoptions::mig::m5h_2::compiler_options_affect_declaration_path(
                self.old_program.unwrap().snapshot.options.as_ref(),
                self.snapshot.options.as_ref(),
            );
        let copy_declaration_file_diagnostics = can_copy_semantic_diagnostics
            && self
                .snapshot
                .options
                .as_ref()
                .map(|o| o.skip_lib_check.is_true())
                == self
                    .old_program
                    .and_then(|p| p.snapshot.options.as_ref())
                    .map(|o| o.skip_lib_check.is_true());
        let copy_lib_file_diagnostics = copy_declaration_file_diagnostics
            && self
                .snapshot
                .options
                .as_ref()
                .map(|o| o.skip_default_lib_check.is_true())
                == self
                    .old_program
                    .and_then(|p| p.snapshot.options.as_ref())
                    .map(|o| o.skip_default_lib_check.is_true());

        let files = self.program.get_source_files();
        for file in &files {
            let file_path = Path::from(m3b_2::path(file));
            let version_text = file.text.clone();
            let version = self.snapshot.compute_hash(&version_text);
            let implied_node_format = self
                .program
                .get_source_file_meta_data(file_path.as_str())
                .unwrap_or_default()
                .implied_node_format;
            let affects_global_scope = file_affects_global_scope(file);
            let mut signature = String::new();
            let new_references = get_referenced_files(self.program, file);
            if let Some(new_references) = &new_references {
                self.snapshot
                    .referenced_map
                    .store_references(file_path.clone(), Arc::new(new_references.clone()));
            }
            if let Some(old_program) = self.old_program {
                if let Some(old_file_info) = old_program.snapshot.file_infos.load(&file_path) {
                    signature = old_file_info.signature.clone();
                    if old_file_info.version != version
                        || old_file_info.affects_global_scope != affects_global_scope
                        || old_file_info.implied_node_format != implied_node_format
                    {
                        self.snapshot.add_file_to_change_set(file_path.clone());
                    } else {
                        let old_references = old_program
                            .snapshot
                            .referenced_map
                            .get_references(&file_path);
                        let references_changed = match (&new_references, old_references.as_ref()) {
                            (Some(new_refs), Some(old_refs)) => !new_refs.equals(old_refs),
                            (Some(_), None) => true,
                            (None, Some(_)) => true,
                            (None, None) => false,
                        };
                        if references_changed {
                            self.snapshot.add_file_to_change_set(file_path.clone());
                        } else if let Some(new_references) = &new_references {
                            for ref_path in new_references.iter() {
                                if self
                                    .program
                                    .get_source_file_by_path(ref_path.as_str())
                                    .is_none()
                                    && old_program.snapshot.file_infos.load(ref_path).is_some()
                                {
                                    self.snapshot.add_file_to_change_set(file_path.clone());
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    self.snapshot.add_file_to_change_set(file_path.clone());
                }
                if !self.snapshot.changed_files_set.lock().unwrap().contains(&file_path) {
                    if let Some(emit_diagnostics) =
                        old_program.snapshot.emit_diagnostics_per_file.load(&file_path)
                    {
                        self.snapshot.emit_diagnostics_per_file.store(
                            file_path.clone(),
                            repopulate_diagnostics_of_file(
                                emit_diagnostics,
                                self.program,
                                file,
                            ),
                        );
                    }
                    if can_copy_semantic_diagnostics
                        && (!file.is_declaration_file || copy_declaration_file_diagnostics)
                        && (!self
                            .program
                            .is_source_file_default_library(file_path.as_str())
                            || copy_lib_file_diagnostics)
                    {
                        if let Some(diagnostics) = old_program
                            .snapshot
                            .semantic_diagnostics_per_file
                            .load(&file_path)
                        {
                            self.snapshot.semantic_diagnostics_per_file.store(
                                file_path.clone(),
                                repopulate_diagnostics_of_file(diagnostics, self.program, file),
                            );
                        }
                    }
                }
                if can_copy_emit_signatures {
                    if let Some(old_emit_signature) =
                        old_program.snapshot.emit_signatures.load(&file_path)
                    {
                        let new_options = self.snapshot.options.clone().unwrap_or_default();
                        let old_options = old_program
                            .snapshot
                            .options
                            .clone()
                            .unwrap_or_default();
                        self.snapshot.emit_signatures.store(
                            file_path.clone(),
                            old_emit_signature.get_new_emit_signature(&old_options, &new_options),
                        );
                    }
                }
            } else {
                let emit_kind = self
                    .snapshot
                    .options
                    .as_ref()
                    .map(get_file_emit_kind)
                    .unwrap_or(FileEmitKind::None);
                self.snapshot
                    .add_file_to_affected_files_pending_emit(file_path.clone(), emit_kind);
                signature = version.clone();
            }
            self.snapshot.file_infos.store(
                file_path.clone(),
                FileInfo {
                    version,
                    signature,
                    affects_global_scope,
                    implied_node_format,
                },
            );
        }
    }

    pub fn handle_file_delete(&mut self) { ::tsox_core::fntrace::enter("handle_file_delete"); 
        if let Some(old_program) = self.old_program {
            let file_infos: Vec<(Path, FileInfo)> = old_program
                .snapshot
                .file_infos
                .keys()
                .into_iter()
                .filter_map(|path| {
                    old_program
                        .snapshot
                        .file_infos
                        .load(&path)
                        .map(|info| (path, info))
                })
                .collect();
            for (file_path, old_info) in file_infos {
                if self.snapshot.file_infos.load(&file_path).is_none() {
                    if old_info.affects_global_scope {
                        for file in self.snapshot.get_all_files_excluding_default_library_file(
                            self.program,
                            None,
                        ) {
                            self.snapshot
                                .add_file_to_change_set(Path::from(m3b_2::path(&file)));
                        }
                        self.global_file_removed = true;
                    } else {
                        self.snapshot
                            .build_info_emit_pending
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    break;
                }
            }
        }
    }

    pub fn handle_global_scope_change(&mut self) { ::tsox_core::fntrace::enter("handle_global_scope_change"); 
        if self.old_program.is_none() || self.global_file_removed {
            return;
        }
        let mut global_scope_lost = false;
        if let Some(old_program) = self.old_program {
            let file_infos: Vec<(Path, FileInfo)> = old_program
                .snapshot
                .file_infos
                .keys()
                .into_iter()
                .filter_map(|path| {
                    old_program
                        .snapshot
                        .file_infos
                        .load(&path)
                        .map(|info| (path, info))
                })
                .collect();
            for (file_path, old_info) in file_infos {
                if !old_info.affects_global_scope {
                    continue;
                }
                if let Some(new_info) = self.snapshot.file_infos.load(&file_path)
                    && !new_info.affects_global_scope
                {
                    global_scope_lost = true;
                    break;
                }
            }
        }
        if global_scope_lost {
            for file in self
                .snapshot
                .get_all_files_excluding_default_library_file(self.program, None)
            {
                self.snapshot
                    .add_file_to_change_set(Path::from(m3b_2::path(&file)));
            }
        }
    }

    pub fn handle_pending_emit(&mut self) { ::tsox_core::fntrace::enter("handle_pending_emit"); 
        if let Some(old_program) = self.old_program {
            if !self.global_file_removed {
                let old_options = old_program.snapshot.options.clone().unwrap_or_default();
                let new_options = self.snapshot.options.clone().unwrap_or_default();
                let pending_emit_kind =
                    if tsox_tsoptions::mig::m5h_2::compiler_options_affect_emit(
                        Some(&old_options),
                        Some(&new_options),
                    ) {
                        get_file_emit_kind(&new_options)
                    } else {
                        get_pending_emit_kind_with_options(&new_options, &old_options)
                    };
                if pending_emit_kind != FileEmitKind::None {
                    for file in self.program.get_source_files() {
                        let file_path = Path::from(m3b_2::path(&file));
                        if !self.snapshot.changed_files_set.lock().unwrap().contains(&file_path) {
                            self.snapshot.add_file_to_affected_files_pending_emit(
                                file_path,
                                pending_emit_kind,
                            );
                        }
                    }
                    self.snapshot
                        .build_info_emit_pending
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
        }
    }

    pub fn handle_pending_check(&mut self) { ::tsox_core::fntrace::enter("handle_pending_check"); 
        if let Some(old_program) = self.old_program {
            if self.snapshot.semantic_diagnostics_per_file.len()
                != self.program.get_source_files().len()
                && *old_program.snapshot.check_pending.lock().unwrap()
                    != *self.snapshot.check_pending.lock().unwrap()
            {
                self.snapshot
                    .build_info_emit_pending
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}

pub fn file_affects_global_scope(file: &Arc<SourceFile>) -> bool { ::tsox_core::fntrace::enter("file_affects_global_scope"); 
    tsox_checker::binder::bind_source_file(file);
    if file.module_augmentations.iter().any(|augmentation| {
        augmentation
            .parent()
            .map(|parent| tsox_frontend::ast::is_global_scope_augmentation(&parent))
            .unwrap_or(false)
    }) {
        return true;
    }

    if tsox_frontend::ast::is_external_or_common_js_module(file)
        || tsox_frontend::ast::is_json_source_file(file)
    {
        return false;
    }

    match &file.node.data {
        tsox_frontend::ast::NodeData::SourceFile(data) => data
            .statements
            .nodes
            .iter()
            .any(|stmt| !tsox_frontend::ast::is_module_with_string_literal_name(stmt)),
        _ => false,
    }
}

pub fn add_referenced_files_from_symbol(
    checker: &tsox_checker::checker::Checker,
    file: &Arc<SourceFile>,
    referenced_files: &mut Set<Path>,
    symbol: Option<&tsox_frontend::ast::Symbol>,
) { ::tsox_core::fntrace::enter("add_referenced_files_from_symbol"); 
    let Some(symbol) = symbol else {
        return;
    };
    for declaration in &symbol.declarations {
        let Some(file_of_decl) = checker.get_source_file_of_node(declaration) else {
            continue;
        };
        if file_of_decl.id() != file.id() {
            referenced_files.add(Path::from(m3b_2::path(&file_of_decl)));
        }
    }
}

pub fn add_referenced_files_from_import_literal(
    checker: &tsox_checker::checker::Checker,
    file: &Arc<SourceFile>,
    referenced_files: &mut Set<Path>,
    import_name: &Arc<tsox_frontend::ast::Node>,
) { ::tsox_core::fntrace::enter("add_referenced_files_from_import_literal"); 
    let symbol = checker.get_symbol_at_location(import_name);
    add_referenced_files_from_symbol(checker, file, referenced_files, symbol.as_deref());
}

pub fn add_referenced_file_from_file_name(
    program: &tsox_compile::compiler::Program,
    file_name: &str,
    referenced_files: &mut Set<Path>,
    source_file_directory: &str,
) { ::tsox_core::fntrace::enter("add_referenced_file_from_file_name"); 
    let redirect = program.get_parse_file_redirect(file_name);
    if !redirect.is_empty() {
        referenced_files.add(tspath::to_path(
            &redirect,
            &program.get_current_directory(),
            program.use_case_sensitive_file_names(),
        ));
    } else {
        referenced_files.add(tspath::to_path(
            file_name,
            source_file_directory,
            program.use_case_sensitive_file_names(),
        ));
    }
}

pub fn get_referenced_files(
    program: &tsox_compile::compiler::Program,
    file: &Arc<SourceFile>,
) -> Option<Set<Path>> { ::tsox_core::fntrace::enter("get_referenced_files"); 
    let mut referenced_files = Set::new();

    let mut checker = program.get_type_checker_for_file_exclusive(file);
    for import_name in &file.imports {
        add_referenced_files_from_import_literal(&checker, file, &mut referenced_files, import_name);
    }

    let source_file_directory = tspath::get_directory_path(&file.file_name);
    for referenced_file in &file.referenced_files {
        add_referenced_file_from_file_name(
            program,
            &referenced_file.file_name,
            &mut referenced_files,
            &source_file_directory,
        );
    }

    if let Some(type_refs_in_file) = program
        .get_resolved_type_reference_directives()
        .get(m3b_2::path(file).as_str())
    {
        for type_ref in type_refs_in_file.values() {
            if !type_ref.resolved_file_name.is_empty() {
                add_referenced_file_from_file_name(
                    program,
                    &type_ref.resolved_file_name,
                    &mut referenced_files,
                    &source_file_directory,
                );
            }
        }
    }

    for module_name in &file.module_augmentations {
        if !tsox_frontend::ast::is_string_literal(module_name) {
            continue;
        }
        add_referenced_files_from_import_literal(&checker, file, &mut referenced_files, module_name);
    }

    for ambient_module in checker.get_ambient_modules() {
        add_referenced_files_from_symbol(
            &checker,
            file,
            &mut referenced_files,
            Some(&ambient_module),
        );
    }
    if referenced_files.is_empty() {
        None
    } else {
        Some(referenced_files)
    }
}

pub fn repopulate_diagnostics_of_file(
    diags: DiagnosticsOrBuildInfoDiagnosticsWithFileName,
    program: &tsox_compile::compiler::Program,
    file: &Arc<SourceFile>,
) -> DiagnosticsOrBuildInfoDiagnosticsWithFileName { ::tsox_core::fntrace::enter("repopulate_diagnostics_of_file"); 
    if !diags.diagnostics.is_empty() {
        if let Some(repopulated) =
            repopulate_diagnostics_list(&diags.diagnostics, program, file)
        {
            return DiagnosticsOrBuildInfoDiagnosticsWithFileName {
                diagnostics: repopulated,
                build_info_diagnostics: Vec::new(),
            };
        }
        return diags;
    }
    diags
}

pub fn repopulate_diagnostics_list(
    diags: &[Diagnostic],
    program: &tsox_compile::compiler::Program,
    file: &Arc<SourceFile>,
) -> Option<Vec<Diagnostic>> { ::tsox_core::fntrace::enter("repopulate_diagnostics_list"); 
    let mut changed = false;
    let mut result = Vec::with_capacity(diags.len());
    for d in diags {
        if let Some(repopulated) =
            repopulate_diagnostic_message_chain(&d.message_chain, program, file)
        {
            let mut clone = d.clone();
            clone.message_chain = repopulated;
            result.push(clone);
            changed = true;
        } else {
            result.push(d.clone());
        }
    }
    if !changed {
        return None;
    }
    Some(result)
}

pub fn repopulate_diagnostic_message_chain(
    chain: &[Diagnostic],
    program: &tsox_compile::compiler::Program,
    file: &Arc<SourceFile>,
) -> Option<Vec<Diagnostic>> { ::tsox_core::fntrace::enter("repopulate_diagnostic_message_chain"); 
    if chain.is_empty() {
        return None;
    }
    let mut changed = false;
    let mut result = Vec::with_capacity(chain.len());
    for c in chain {
        if c.repopulate_info().is_some() {
            let mut b = BuildInfoDiagnosticWithFileName {
                pos: c.loc.pos,
                end: c.loc.end,
                code: c.code,
                category: c.category,
                source: c.source().to_string(),
                message_text: c.message_text().to_string(),
                message_key: c.message_key.clone(),
                message_args: Some(c.message_args.clone()),
                repopulate_info: c
                    .repopulate_info()
                    .map(|info| repopulate_info_to_build_info(info)),
                ..Default::default()
            };
            for nested in &c.message_chain {
                b.message_chain.push(ast_diag_to_build_info_diag(nested));
            }
            result.push(repopulate_diagnostic_chain(&b, program, Some(file)));
            changed = true;
        } else if let Some(nested) =
            repopulate_diagnostic_message_chain(&c.message_chain, program, file)
        {
            let mut clone = c.clone();
            clone.message_chain = nested;
            result.push(clone);
            changed = true;
        } else {
            result.push(c.clone());
        }
    }
    if !changed {
        return None;
    }
    Some(result)
}

pub fn ast_diag_to_build_info_diag(d: &Diagnostic) -> Box<BuildInfoDiagnosticWithFileName> { ::tsox_core::fntrace::enter("ast_diag_to_build_info_diag"); 
    let mut b = Box::new(BuildInfoDiagnosticWithFileName {
        pos: d.loc.pos,
        end: d.loc.end,
        code: d.code,
        category: d.category,
        source: d.source().to_string(),
        message_text: d.message_text().to_string(),
        message_key: d.message_key.clone(),
        message_args: Some(d.message_args.clone()),
        repopulate_info: d
            .repopulate_info()
            .map(|info| repopulate_info_to_build_info(info)),
        ..Default::default()
    });
    for nested in &d.message_chain {
        b.message_chain.push(ast_diag_to_build_info_diag(nested));
    }
    b
}

fn repopulate_info_to_build_info(
    info: &tsox_frontend::ast::mig::m3d_2::RepopulateDiagnosticInfo,
) -> tsox_frontend::ast::mig::m3e::RepopulateDiagnosticInfo { ::tsox_core::fntrace::enter("repopulate_info_to_build_info"); 
    tsox_frontend::ast::mig::m3e::RepopulateDiagnosticInfo {
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
    }
}
