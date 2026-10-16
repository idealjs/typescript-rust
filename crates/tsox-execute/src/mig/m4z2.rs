#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath;
use tsox_core::tspath::Path;
use tsox_checker::checker::Program as _;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3b_2;
use tsox_frontend::ast::SourceFile;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use crate::mig::m4y_3::collect_all_affected_files;
use crate::mig::m4y_3::emit_files;
use crate::mig::m4y_3::EmitOnly;
use crate::mig::m4y_3::EmitOptions;
use crate::mig::m4y_3::EmitResult;
use crate::mig::m4y_3::SignatureUpdateKind;
use crate::mig::m4y_3::WriteFileData;
use crate::mig::m4z2_2::DiagnosticsOrBuildInfoDiagnosticsWithFileName;
use crate::mig::m4z2_2::Snapshot;
use crate::mig::m4z2_3::program_to_snapshot;
use crate::mig::m4z2_4::snapshot_to_build_info;
use crate::mig::m4z::Host;
use crate::mig::m4z::HostImpl;

pub struct Program {
    pub snapshot: Snapshot,
    pub program: Option<Arc<tsox_compile::compiler::Program>>,
    pub host: Option<Arc<dyn Host + Send + Sync>>,
    pub testing_data: Option<TestingData>,
    pub nested_emit_now: Option<Arc<dyn Fn() -> SystemTime + Send + Sync>>,
    pub nested_emit_depth: Mutex<usize>,
    pub nested_emit_start: Mutex<SystemTime>,
    pub nested_emit_time: Mutex<Duration>,
}

pub struct TestingData {
    pub semantic_diagnostics_per_file: Arc<SyncMap<Path, DiagnosticsOrBuildInfoDiagnosticsWithFileName>>,
    pub old_program_semantic_diagnostics_per_file: SyncMap<Path, DiagnosticsOrBuildInfoDiagnosticsWithFileName>,
    pub updated_signature_kinds: SyncMap<Path, SignatureUpdateKind>,
}

pub fn new_program(
    program: Arc<tsox_compile::compiler::Program>,
    old_program: Option<&Program>,
    host: Option<Arc<dyn Host + Send + Sync>>,
    nested_emit_now: Option<Arc<dyn Fn() -> SystemTime + Send + Sync>>,
    testing: bool,
) -> Program { ::tsox_core::fntrace::enter("new_program"); 
    let snapshot = program_to_snapshot(&program, old_program, testing);
    let mut incremental_program = Program {
        snapshot,
        program: Some(program),
        host,
        testing_data: None,
        nested_emit_now,
        nested_emit_depth: Mutex::new(0),
        nested_emit_start: Mutex::new(SystemTime::UNIX_EPOCH),
        nested_emit_time: Mutex::new(Duration::ZERO),
    };

    if testing {
        let old_semantic_diagnostics = match old_program {
            Some(old_program) => old_program.snapshot.semantic_diagnostics_per_file.clone_map(),
            None => SyncMap::new(),
        };
        incremental_program.testing_data = Some(TestingData {
            semantic_diagnostics_per_file: Arc::new(
                incremental_program.snapshot.semantic_diagnostics_per_file.clone_map(),
            ),
            old_program_semantic_diagnostics_per_file: old_semantic_diagnostics,
            updated_signature_kinds: SyncMap::new(),
        });
    }
    incremental_program
}

impl Program {
    pub fn get_testing_data(&self) -> Option<&TestingData> { ::tsox_core::fntrace::enter("get_testing_data"); 
        self.testing_data.as_ref()
    }

    pub fn begin_nested_emit(&self) -> Box<dyn Fn() + '_> { ::tsox_core::fntrace::enter("begin_nested_emit"); 
        let Some(now) = self.nested_emit_now.clone() else {
            return Box::new(|| {});
        };
        {
            let mut depth = self.nested_emit_depth.lock().unwrap();
            if *depth == 0 {
                *self.nested_emit_start.lock().unwrap() = now();
            }
            *depth += 1;
        }

        Box::new(move || {
            let mut depth = self.nested_emit_depth.lock().unwrap();
            *depth -= 1;
            if *depth == 0 {
                let start = *self.nested_emit_start.lock().unwrap();
                *self.nested_emit_time.lock().unwrap() +=
                    now().duration_since(start).unwrap_or_default();
            }
        })
    }

    pub fn take_nested_emit_time(&self) -> Duration { ::tsox_core::fntrace::enter("take_nested_emit_time"); 
        std::mem::take(&mut *self.nested_emit_time.lock().unwrap())
    }

    pub fn panic_if_no_program(&self, method: &str) { ::tsox_core::fntrace::enter("panic_if_no_program"); 
        if self.program.is_none() {
            panic!("{method}: should not be called without program");
        }
    }

    pub fn compiler_program(&self) -> &tsox_compile::compiler::Program { ::tsox_core::fntrace::enter("compiler_program"); 
        self.program.as_deref().unwrap()
    }

    pub fn get_program(&self) -> &tsox_compile::compiler::Program { ::tsox_core::fntrace::enter("get_program"); 
        self.panic_if_no_program("get_program");
        self.compiler_program()
    }

    pub fn program(&self) -> &tsox_compile::compiler::Program { ::tsox_core::fntrace::enter("program"); 
        self.panic_if_no_program("program");
        self.compiler_program()
    }

    pub fn has_changed_dts_file(&self) -> bool { ::tsox_core::fntrace::enter("has_changed_dts_file"); 
        self.snapshot
            .has_changed_dts_file
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn options(&self) -> Option<&tsox_core::core::compiler_options::CompilerOptions> { ::tsox_core::fntrace::enter("options"); 
        self.snapshot.options.as_ref()
    }

    pub fn common_source_directory(&self) -> String { ::tsox_core::fntrace::enter("common_source_directory"); 
        self.panic_if_no_program("common_source_directory");
        self.compiler_program().common_source_directory()
    }

    pub fn is_source_file_default_library(&self, path: &Path) -> bool { ::tsox_core::fntrace::enter("is_source_file_default_library"); 
        self.panic_if_no_program("is_source_file_default_library");
        self.compiler_program()
            .is_source_file_default_library(path.as_str())
    }

    pub fn get_source_files(&self) -> Vec<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_files"); 
        self.panic_if_no_program("get_source_files");
        self.compiler_program().get_source_files()
    }

    pub fn get_source_file(&self, path: &str) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        self.panic_if_no_program("get_source_file");
        self.compiler_program().get_source_file(path)
    }

    pub fn get_source_file_by_path(&self, path: &str) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file_by_path"); 
        self.panic_if_no_program("get_source_file_by_path");
        self.compiler_program().get_source_file_by_path(path)
    }

    pub fn get_config_file_parsing_diagnostics(&self) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_config_file_parsing_diagnostics"); 
        self.panic_if_no_program("get_config_file_parsing_diagnostics");
        self.compiler_program()
            .get_config_file_parsing_diagnostics()
            .into_iter()
            .map(|d| (*d).clone())
            .collect()
    }

    pub fn get_syntactic_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_syntactic_diagnostics"); 
        self.panic_if_no_program("get_syntactic_diagnostics");
        self.compiler_program()
            .get_syntactic_diagnostics(file)
            .into_iter()
            .map(|d| (*d).clone())
            .collect()
    }

    pub fn get_bind_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_bind_diagnostics"); 
        self.panic_if_no_program("get_bind_diagnostics");
        self.compiler_program()
            .get_bind_diagnostics(file)
            .into_iter()
            .map(|d| (*d).clone())
            .collect()
    }

    pub fn get_program_diagnostics(&self) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_program_diagnostics"); 
        self.panic_if_no_program("get_program_diagnostics");
        self.compiler_program()
            .get_program_diagnostics()
            .into_iter()
            .map(|d| (*d).clone())
            .collect()
    }

    pub fn get_global_diagnostics(&self) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_global_diagnostics"); 
        self.panic_if_no_program("get_global_diagnostics");
        self.compiler_program()
            .get_global_diagnostics()
            .into_iter()
            .map(|d| (*d).clone())
            .collect()
    }

    pub fn get_semantic_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_semantic_diagnostics"); 
        self.panic_if_no_program("get_semantic_diagnostics");
        if self
            .snapshot
            .options
            .as_ref()
            .map(|o| o.no_check.is_true())
            .unwrap_or(false)
        {
            return Vec::new();
        }

        self.collect_semantic_diagnostics_of_affected_files(file);

        if let Some(file) = file {
            return self.get_semantic_diagnostics_of_file(file);
        }

        let mut diagnostics = Vec::new();
        for file in self.compiler_program().get_source_files() {
            diagnostics.extend(self.get_semantic_diagnostics_of_file(&file));
        }
        diagnostics
    }

    pub fn get_semantic_diagnostics_of_file(&self, file: &Arc<SourceFile>) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_semantic_diagnostics_of_file"); 
        let mut cached_diagnostics = self
            .snapshot
            .semantic_diagnostics_per_file
            .load(&Path::from(m3b_2::path(file)))
            .unwrap_or_else(|| {
                panic!("After handling all the affected files, there shouldnt be more changes")
            });
        let mut diagnostics: Vec<Diagnostic> =
            tsox_compile::mig::m4w_4::filter_no_emit_semantic_diagnostics(
            cached_diagnostics
                .get_diagnostics(self.compiler_program(), file)
                .into_iter()
                .map(Arc::new)
                .collect(),
            self.snapshot
                .options
                .as_ref()
                .unwrap_or(&tsox_core::core::compiler_options::CompilerOptions::default()),
        )
        .into_iter()
        .map(|d| (*d).clone())
        .collect();
        diagnostics.extend(
            self.compiler_program()
                .get_include_processor_diagnostics(file)
                .into_iter()
                .map(|d| (*d).clone()),
        );
        diagnostics
    }

    pub fn get_declaration_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_declaration_diagnostics"); 
        self.panic_if_no_program("get_declaration_diagnostics");
        let options = EmitOptions {
            target_source_files: file.cloned().map(|f| vec![f]),
            ..Default::default()
        };
        match emit_files(self, &options, true) {
            Some(result) => result.diagnostics,
            None => Vec::new(),
        }
    }

    pub fn get_suggestion_diagnostics(&self, file: Option<&Arc<SourceFile>>) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_suggestion_diagnostics"); 
        self.panic_if_no_program("get_suggestion_diagnostics");
        self.compiler_program()
            .get_suggestion_diagnostics(file)
            .into_iter()
            .map(|d| (*d).clone())
            .collect()
    }

    pub fn collect_semantic_diagnostics_of_affected_files(&self, file: Option<&Arc<SourceFile>>) { ::tsox_core::fntrace::enter("collect_semantic_diagnostics_of_affected_files"); 
        if self.snapshot.can_use_incremental_state() {
            collect_all_affected_files(self);
            if self.snapshot.semantic_diagnostics_per_file.len()
                == self.compiler_program().get_source_files().len()
            {
                return;
            }
        }

        let mut affected_files: Vec<Arc<SourceFile>> = Vec::new();
        match file {
            Some(file) => {
                if self
                    .snapshot
                    .semantic_diagnostics_per_file
                    .load(&Path::from(m3b_2::path(file)))
                    .is_some()
                {
                    return;
                }
                affected_files.push(file.clone());
            }
            None => {
                for file in self.compiler_program().get_source_files() {
                    if self
                        .snapshot
                        .semantic_diagnostics_per_file
                        .load(&Path::from(m3b_2::path(&file)))
                        .is_none()
                    {
                        affected_files.push(file);
                    }
                }
            }
        }

        let diagnostics_per_file = self
            .compiler_program()
            .get_semantic_diagnostics_without_no_emit_filtering(&affected_files);
        for (file, diagnostics) in diagnostics_per_file {
            self.snapshot.semantic_diagnostics_per_file.store(
                Path::from(file),
                DiagnosticsOrBuildInfoDiagnosticsWithFileName {
                    diagnostics: diagnostics
                        .into_iter()
                        .map(|d| (*d).clone())
                        .collect(),
                    ..Default::default()
                },
            );
        }
        if self.snapshot.semantic_diagnostics_per_file.len()
            == self.compiler_program().get_source_files().len()
            && *self.snapshot.check_pending.lock().unwrap()
            && !self
                .snapshot
                .options
                .as_ref()
                .map(|o| o.no_check.is_true())
                .unwrap_or(false)
        {
            *self.snapshot.check_pending.lock().unwrap() = false;
        }
        self.snapshot
            .build_info_emit_pending
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn emit_build_info(&self, options: &EmitOptions) -> Option<EmitResult> { ::tsox_core::fntrace::enter("emit_build_info"); 
        let program = self.compiler_program();
        let build_info_file_name = get_build_info_file_name(
            self.snapshot.options.as_ref()?,
            &tspath::ComparePathsOptions {
                current_directory: program.get_current_directory().to_string(),
                use_case_sensitive_file_names: program.use_case_sensitive_file_names(),
            },
        );
        if build_info_file_name.is_empty() || program.is_emit_blocked(&build_info_file_name) {
            return None;
        }
        if *self.snapshot.has_errors.lock().unwrap() == Tristate::Unknown {
            self.ensure_has_errors_for_state(program);
            if *self.snapshot.has_errors.lock().unwrap() != self.snapshot.has_errors_from_old_state
                || *self.snapshot.has_semantic_errors.lock().unwrap()
                    != self.snapshot.has_semantic_errors_from_old_state
            {
                self.snapshot
                    .build_info_emit_pending
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
        if self.snapshot.package_jsons.lock().unwrap().is_none() {
            self.ensure_package_jsons_for_state();
            if *self.snapshot.package_jsons.lock().unwrap()
                != self.snapshot.package_jsons_from_old_state
                || *self.snapshot.missing_package_jsons.lock().unwrap()
                    != self.snapshot.missing_package_jsons_from_old_state
            {
                self.snapshot
                    .build_info_emit_pending
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
        if !self
            .snapshot
            .build_info_emit_pending
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return None;
        }
        let build_info = match snapshot_to_build_info(&self.snapshot, program, &build_info_file_name)
        {
            Ok(build_info) => build_info,
            Err(err) => {
                return Some(EmitResult {
                    emit_skipped: true,
                    diagnostics: vec![tsox_compile::mig::m4v_2::content_mapper_project_diagnostic(
                        &err,
                    )],
                    ..Default::default()
                });
            }
        };
        let text = match serde_json::to_string(&build_info) {
            Ok(text) => text,
            Err(err) => panic!("Failed to marshal build info: {err}"),
        };
        let result = match &options.write_file {
            Some(write_file) => write_file(
                &build_info_file_name,
                &text,
                &WriteFileData {
                    build_info: Some(build_info),
                    ..Default::default()
                },
            ),
            None => program
                .host()
                .fs()
                .write_file(&build_info_file_name, &text),
        };
        if let Err(err) = result {
            return Some(EmitResult {
                emit_skipped: true,
                diagnostics: vec![tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
                    tsox_core::diagnostics::COULD_NOT_WRITE_FILE_0_COLON_1,
                    vec![build_info_file_name.clone(), err.to_string()],
                )],
                ..Default::default()
            });
        }
        self.snapshot
            .build_info_emit_pending
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Some(EmitResult {
            emit_skipped: false,
            emitted_files: vec![build_info_file_name],
            ..Default::default()
        })
    }

    pub fn ensure_has_errors_for_state(&self, program: &tsox_compile::compiler::Program) { ::tsox_core::fntrace::enter("ensure_has_errors_for_state"); 
        let mut has_include_processing_diagnostics: Option<bool> = None;
        let mut has_emit_diagnostics;
        if self.snapshot.can_use_incremental_state() {
            has_emit_diagnostics = false;
            for file in program.get_source_files() {
                if self
                    .snapshot
                    .emit_diagnostics_per_file
                    .load(&Path::from(m3b_2::path(&file)))
                    .is_some()
                {
                    has_emit_diagnostics = true;
                    break;
                }
                if has_include_processing_diagnostics.is_none()
                    && !self
                        .compiler_program()
                        .get_include_processor_diagnostics(&file)
                        .is_empty()
                {
                    has_include_processing_diagnostics = Some(true);
                }
            }
        } else {
            has_emit_diagnostics = self
                .snapshot
                .has_emit_diagnostics
                .load(std::sync::atomic::Ordering::SeqCst);
            has_include_processing_diagnostics = Some(
                program
                    .get_source_files()
                    .iter()
                    .any(|file| !self.compiler_program().get_include_processor_diagnostics(file).is_empty()),
            );
        }
        let has_include_processing_diagnostics = has_include_processing_diagnostics.unwrap_or(false);

        if has_emit_diagnostics {
            *self.snapshot.has_errors.lock().unwrap() = if self
                .snapshot
                .options
                .as_ref()
                .map(|o| o.is_incremental())
                .unwrap_or(false)
            {
                Tristate::False
            } else {
                Tristate::True
            };
            *self.snapshot.has_semantic_errors.lock().unwrap() = false;
            return;
        }

        if has_include_processing_diagnostics
            || !program.get_config_file_parsing_diagnostics().is_empty()
            || !program.get_syntactic_diagnostics(None).is_empty()
            || !program.get_program_diagnostics().is_empty()
            || !program.get_global_diagnostics().is_empty()
        {
            *self.snapshot.has_errors.lock().unwrap() = Tristate::True;
            *self.snapshot.has_semantic_errors.lock().unwrap() = false;
            return;
        }

        *self.snapshot.has_errors.lock().unwrap() = Tristate::False;
        let is_incremental = self
            .snapshot
            .options
            .as_ref()
            .map(|o| o.is_incremental())
            .unwrap_or(false);
        for file in self.compiler_program().get_source_files() {
            match self
                .snapshot
                .semantic_diagnostics_per_file
                .load(&Path::from(m3b_2::path(&file)))
            {
                None => {
                    if is_incremental {
                        *self.snapshot.has_semantic_errors.lock().unwrap() = !is_incremental;
                        return;
                    }
                }
                Some(semantic_diagnostics) => {
                    if !semantic_diagnostics.diagnostics.is_empty()
                        || !semantic_diagnostics.build_info_diagnostics.is_empty()
                    {
                        *self.snapshot.has_semantic_errors.lock().unwrap() = !is_incremental;
                        return;
                    }
                }
            }
        }
    }

    pub fn ensure_package_jsons_for_state(&self) { ::tsox_core::fntrace::enter("ensure_package_jsons_for_state"); 
        let config = tspath::get_directory_path(
            &self.compiler_program().command_line().config_name(),
        );
        if !config.is_empty() {
            self.compiler_program()
                .package_json_cache_entries(&mut |_key: &str,
                                                  value: &tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry| {
                    let package_json =
                        tspath::combine_paths(&value.package_directory, &["package.json"]);
                    let package_json = if value.exists() || value.directory_exists {
                        self.compiler_program().host().fs().realpath(&package_json)
                    } else {
                        package_json
                    };
                    if value.exists() {
                        self.snapshot
                            .package_jsons
                            .lock()
                            .unwrap()
                            .get_or_insert_with(Vec::new)
                            .push(package_json);
                    } else if package_json.contains("/node_modules/") {
                        self.snapshot
                            .missing_package_jsons
                            .lock()
                            .unwrap()
                            .get_or_insert_with(Vec::new)
                            .push(package_json);
                    }
                    true
                });
        }
        let package_jsons = normalize_package_jsons(self.snapshot.package_jsons.lock().unwrap().take());
        *self.snapshot.package_jsons.lock().unwrap() = package_jsons;
        let missing_package_jsons =
            normalize_package_jsons(self.snapshot.missing_package_jsons.lock().unwrap().take());
        *self.snapshot.missing_package_jsons.lock().unwrap() = missing_package_jsons;
    }

    pub fn package_json_lookup_paths(&self) -> Vec<String> { ::tsox_core::fntrace::enter("package_json_lookup_paths"); 
        let config = tspath::get_directory_path(
            &self.compiler_program().command_line().config_name(),
        );
        if config.is_empty() {
            return Vec::new();
        }

        let mut package_jsons: Vec<String> = Vec::new();
        self.compiler_program()
            .package_json_cache_entries(&mut |_key: &str,
                                               value: &tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry| {
                let package_json =
                    tspath::combine_paths(&value.package_directory, &["package.json"]);
                let package_json = if value.exists() || value.directory_exists {
                    self.compiler_program().host().fs().realpath(&package_json)
                } else {
                    package_json
                };
                package_jsons.push(package_json);
                true
            });
        package_jsons.sort();
        tsox_core::core::mig::m3j_2::deduplicate(&package_jsons)
    }

    pub fn emit(&self, options: EmitOptions) -> EmitResult { ::tsox_core::fntrace::enter("emit"); 
        let program = self.compiler_program();
        let default_options = tsox_core::core::compiler_options::CompilerOptions::default();
        let snapshot_options = self.snapshot.options.as_ref().unwrap_or(&default_options);

        if !options.force_emit && options.emit_only != EmitOnly::BuilderSignature {
            if let Some(result) =
                handle_no_emit_options(self, snapshot_options, options.target_source_files.as_deref())
            {
                return result;
            }
        }

        let mut emit_options = snapshot_options.clone();
        match options.emit_only {
            EmitOnly::All => {}
            EmitOnly::Js => {
                emit_options.emit_declaration_only = Tristate::False;
                emit_options.declaration = Tristate::False;
                emit_options.declaration_map = Tristate::False;
            }
            EmitOnly::Dts | EmitOnly::BuilderSignature => {
                emit_options.emit_declaration_only = Tristate::True;
                emit_options.declaration = Tristate::True;
            }
        }

        let source_files: Vec<Arc<SourceFile>> = match &options.target_source_files {
            Some(files) => files.clone(),
            None => self
                .get_source_files()
                .into_iter()
                .filter(|file| program.source_file_may_be_emitted(file, false))
                .collect(),
        };

        let common_dir = tsox_emit::emitter::compute_program_common_source_directory(
            &source_files,
            &emit_options,
        );
        let fs = program.host().fs();
        let mut result = EmitResult::default();
        for source_file in &source_files {
            let write = options.write_file.clone();
            let file_for_data = source_file.clone();
            let bridge = |file_name: &str, text: &str| -> std::io::Result<()> {
                let data = WriteFileData {
                    source_map_url_pos: -1,
                    source_file: Some(file_for_data.clone()),
                    ..Default::default()
                };
                match &write {
                    Some(write_file) => write_file(file_name, text, &data),
                    None => fs.write_file(file_name, text),
                }
            };
            let file_result = tsox_emit::emitter::emit_source_file_with_common_dir(
                source_file,
                &emit_options,
                fs,
                &common_dir,
                &bridge,
            );
            if file_result.emit_skipped {
                result.emit_skipped = true;
            }
            result.emitted_files.extend(file_result.emitted_files);
            for message in file_result.diagnostics {
                result
                    .diagnostics
                    .push(tsox_frontend::ast::mig::m3d_2::new_external_diagnostic(
                        Some(source_file.clone()),
                        tsox_core::core::text::TextRange::undefined(),
                        String::new(),
                        tsox_core::diagnostics::Category::Error,
                        0,
                        message,
                    ));
            }
        }
        result
    }
}

fn handle_no_emit_options(
    incremental: &Program,
    options: &tsox_core::core::compiler_options::CompilerOptions,
    files: Option<&[Arc<SourceFile>]>,
) -> Option<EmitResult> { ::tsox_core::fntrace::enter("handle_no_emit_options"); 
    if !options.no_emit.is_true() {
        if !options.no_emit_on_error.is_true() {
            return None;
        }
        let mut diagnostics = incremental.get_config_file_parsing_diagnostics();
        diagnostics.extend(match files {
            Some(files) => files
                .iter()
                .flat_map(|file| incremental.get_syntactic_diagnostics(Some(file)))
                .collect(),
            None => incremental.get_syntactic_diagnostics(None),
        });
        diagnostics.extend(incremental.get_program_diagnostics());
        diagnostics.extend(incremental.get_global_diagnostics());
        diagnostics.extend(match files {
            Some(files) => files
                .iter()
                .flat_map(|file| incremental.get_bind_diagnostics(Some(file)))
                .collect(),
            None => incremental.get_bind_diagnostics(None),
        });
        diagnostics.extend(match files {
            Some(files) => files
                .iter()
                .flat_map(|file| incremental.get_semantic_diagnostics(Some(file)))
                .collect(),
            None => incremental.get_semantic_diagnostics(None),
        });
        if diagnostics.is_empty() {
            return None;
        }
        return Some(EmitResult {
            emit_skipped: true,
            diagnostics,
            ..Default::default()
        });
    }
    if files.is_some() {
        return Some(EmitResult {
            emit_skipped: true,
            ..Default::default()
        });
    }
    Some(EmitResult::default())
}

pub fn normalize_package_jsons(package_jsons: Option<Vec<String>>) -> Option<Vec<String>> { ::tsox_core::fntrace::enter("normalize_package_jsons"); 
    let mut package_jsons = package_jsons.unwrap_or_default();
    package_jsons.sort();
    Some(tsox_core::core::mig::m3j_2::deduplicate(
        &package_jsons,
    ))
}

fn get_build_info_file_name(
    options: &tsox_core::core::compiler_options::CompilerOptions,
    opts: &tspath::ComparePathsOptions,
) -> String { ::tsox_core::fntrace::enter("get_build_info_file_name"); 
    if !options.is_incremental() && !options.build.is_true() {
        return String::new();
    }
    if !options.ts_build_info_file.is_empty() {
        return options.ts_build_info_file.clone();
    }
    if options.config_file_path.is_empty() {
        return String::new();
    }
    let config_file_extension_less = tspath::remove_file_extension(&options.config_file_path);
    let build_info_extension_less = if !options.out_dir.is_empty() {
        if !options.root_dir.is_empty() {
            tspath::resolve_path(
                &options.out_dir,
                &[&tsox_core::tspath::mig::m3i::get_relative_path_from_directory(
                    &options.root_dir,
                    &config_file_extension_less,
                    opts,
                )],
            )
        } else {
            tspath::combine_paths(
                &options.out_dir,
                &[&tspath::get_base_file_name(&config_file_extension_less)],
            )
        }
    } else {
        config_file_extension_less
    };
    format!("{}{}", build_info_extension_less, tspath::EXTENSION_TS_BUILD_INFO)
}
