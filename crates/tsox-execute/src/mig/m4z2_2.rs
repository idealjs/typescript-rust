#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Once;

use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath;
use tsox_core::tspath::Path;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3e::RepopulateDiagnosticInfo;
use tsox_frontend::ast::SourceFile;

use crate::mig::m4z::ReferenceMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FileEmitKind(pub u16);

impl FileEmitKind {
    pub const None: FileEmitKind = FileEmitKind(0);
    pub const Js: FileEmitKind = FileEmitKind(1 << 0);
    pub const JsMap: FileEmitKind = FileEmitKind(1 << 1);
    pub const JsInlineMap: FileEmitKind = FileEmitKind(1 << 2);
    pub const DtsErrors: FileEmitKind = FileEmitKind(1 << 3);
    pub const DtsEmit: FileEmitKind = FileEmitKind(1 << 4);
    pub const DtsMap: FileEmitKind = FileEmitKind(1 << 5);
    pub const Dts: FileEmitKind = FileEmitKind((1 << 3) | (1 << 4));
    pub const AllJs: FileEmitKind = FileEmitKind((1 << 0) | (1 << 1) | (1 << 2));
    pub const AllDtsEmit: FileEmitKind = FileEmitKind((1 << 4) | (1 << 5));
    pub const AllDts: FileEmitKind = FileEmitKind((1 << 3) | (1 << 4) | (1 << 5));
    pub const All: FileEmitKind = FileEmitKind(0b111111);

    pub fn from_repr(value: u16) -> Option<FileEmitKind> {
        if value <= 0b111111 {
            Some(FileEmitKind(value))
        } else {
            None
        }
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitAnd for FileEmitKind {
    type Output = FileEmitKind;
    fn bitand(self, rhs: FileEmitKind) -> FileEmitKind {
        FileEmitKind(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for FileEmitKind {
    fn bitand_assign(&mut self, rhs: FileEmitKind) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOr for FileEmitKind {
    type Output = FileEmitKind;
    fn bitor(self, rhs: FileEmitKind) -> FileEmitKind {
        FileEmitKind(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for FileEmitKind {
    fn bitor_assign(&mut self, rhs: FileEmitKind) {
        self.0 |= rhs.0;
    }
}

pub fn get_file_emit_kind(options: &CompilerOptions) -> FileEmitKind {
    let mut result = FileEmitKind::Js;
    if options.source_map.is_true() {
        result |= FileEmitKind::JsMap;
    }
    if options.inline_source_map.is_true() {
        result |= FileEmitKind::JsInlineMap;
    }
    if options.get_emit_declarations() {
        result |= FileEmitKind::Dts;
    }
    if options.declaration_map.is_true() {
        result |= FileEmitKind::DtsMap;
    }
    if options.emit_declaration_only.is_true() {
        result &= FileEmitKind::AllDts;
    }
    result
}

pub fn get_pending_emit_kind_with_options(
    options: &CompilerOptions,
    old_options: &CompilerOptions,
) -> FileEmitKind {
    let old_emit_kind = get_file_emit_kind(old_options);
    let new_emit_kind = get_file_emit_kind(options);
    get_pending_emit_kind(new_emit_kind, old_emit_kind)
}

pub fn get_pending_emit_kind(emit_kind: FileEmitKind, old_emit_kind: FileEmitKind) -> FileEmitKind {
    if old_emit_kind == emit_kind {
        return FileEmitKind::None;
    }
    if old_emit_kind.is_empty() || emit_kind.is_empty() {
        return emit_kind;
    }
    let diff = FileEmitKind(old_emit_kind.0 ^ emit_kind.0);
    let mut result = FileEmitKind::None;
    if (diff & FileEmitKind::AllJs) != FileEmitKind::None {
        result |= emit_kind & FileEmitKind::AllJs;
    }
    if (diff & FileEmitKind::DtsErrors) != FileEmitKind::None {
        result |= emit_kind & FileEmitKind::AllDts;
    }
    if (diff & FileEmitKind::AllDtsEmit) != FileEmitKind::None {
        result |= emit_kind & FileEmitKind::AllDtsEmit;
    }
    result
}

pub fn compute_hash(text: &str, hash_with_text: bool) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let mut hash = format!("{:016x}", hasher.finish());
    if hash_with_text {
        hash.push('-');
        hash.push_str(text);
    }
    hash
}

#[derive(Debug, Clone, Default)]
pub struct FileInfo {
    pub version: String,
    pub signature: String,
    pub affects_global_scope: bool,
    pub implied_node_format: tsox_core::core::compiler_options::ResolutionMode,
}

impl FileInfo {
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn signature(&self) -> &str {
        &self.signature
    }

    pub fn affects_global_scope(&self) -> bool {
        self.affects_global_scope
    }

    pub fn implied_node_format(&self) -> tsox_core::core::compiler_options::ResolutionMode {
        self.implied_node_format
    }
}

#[derive(Debug, Clone, Default)]
pub struct EmitSignature {
    pub signature: String,
    pub signature_with_different_options: Vec<String>,
}

impl EmitSignature {
    pub fn get_new_emit_signature(
        &self,
        old_options: &CompilerOptions,
        new_options: &CompilerOptions,
    ) -> EmitSignature {
        if old_options.declaration_map.is_true() == new_options.declaration_map.is_true() {
            return self.clone();
        }
        if self.signature_with_different_options.is_empty() {
            EmitSignature {
                signature: String::new(),
                signature_with_different_options: vec![self.signature.clone()],
            }
        } else {
            EmitSignature {
                signature: self.signature_with_different_options[0].clone(),
                signature_with_different_options: Vec::new(),
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BuildInfoDiagnosticWithFileName {
    pub file: Path,
    pub no_file: bool,
    pub pos: i32,
    pub end: i32,
    pub code: i32,
    pub category: tsox_core::diagnostics::Category,
    pub source: String,
    pub message_text: String,
    pub message_key: tsox_core::diagnostics::Key,
    pub message_args: Option<Vec<String>>,
    pub message_chain: Vec<Box<BuildInfoDiagnosticWithFileName>>,
    pub related_information: Vec<Box<BuildInfoDiagnosticWithFileName>>,
    pub reports_unnecessary: bool,
    pub reports_deprecated: bool,
    pub skipped_on_no_emit: bool,
    pub repopulate_info: Option<RepopulateDiagnosticInfo>,
}

impl BuildInfoDiagnosticWithFileName {
    pub fn to_diagnostic(
        &self,
        program: &tsox_compile::compiler::Program,
        file: Option<&Arc<SourceFile>>,
    ) -> Diagnostic {
        let file_for_diagnostic: Option<Arc<SourceFile>> = if !self.file.as_str().is_empty() {
            program.get_source_file_by_path(self.file.as_str())
        } else if !self.no_file {
            file.cloned()
        } else {
            None
        };

        if self.repopulate_info.is_some() {
            return repopulate_diagnostic_chain(self, program, file_for_diagnostic.as_ref());
        }

        let message_chain = self
            .message_chain
            .iter()
            .map(|msg| msg.to_diagnostic(program, file_for_diagnostic.as_ref()))
            .collect();
        let related_information = self
            .related_information
            .iter()
            .map(|info| info.to_diagnostic(program, file_for_diagnostic.as_ref()))
            .collect();
        let mut diagnostic = tsox_frontend::ast::mig::m3d_2::new_diagnostic_from_serialized(
            file_for_diagnostic,
            tsox_core::core::text::TextRange {
                pos: self.pos,
                end: self.end,
            },
            self.code,
            self.category,
            self.message_key.clone(),
            self.message_args.clone().unwrap_or_default(),
            message_chain,
            related_information,
            self.reports_unnecessary,
            self.reports_deprecated,
            self.skipped_on_no_emit,
        );
        if !self.source.is_empty() || !self.message_text.is_empty() {
            diagnostic.set_external_data(self.source.clone(), self.message_text.clone());
        }
        diagnostic
    }

    pub fn to_diagnostic_without_repopulate(
        &self,
        program: &tsox_compile::compiler::Program,
        file: Option<&Arc<SourceFile>>,
    ) -> Diagnostic {
        let message_chain = self
            .message_chain
            .iter()
            .map(|msg| msg.to_diagnostic(program, file))
            .collect();
        let related_information = self
            .related_information
            .iter()
            .map(|info| info.to_diagnostic(program, file))
            .collect();
        tsox_frontend::ast::mig::m3d_2::new_diagnostic_from_serialized(
            file.cloned(),
            tsox_core::core::text::TextRange {
                pos: self.pos,
                end: self.end,
            },
            self.code,
            self.category,
            self.message_key.clone(),
            self.message_args.clone().unwrap_or_default(),
            message_chain,
            related_information,
            self.reports_unnecessary,
            self.reports_deprecated,
            self.skipped_on_no_emit,
        )
    }
}

pub fn repopulate_diagnostic_chain(
    b: &BuildInfoDiagnosticWithFileName,
    program: &tsox_compile::compiler::Program,
    file: Option<&Arc<SourceFile>>,
) -> Diagnostic {
    let info = b.repopulate_info.as_ref().unwrap();
    match info.kind {
        tsox_frontend::ast::mig::m3e::RepopulateDiagnosticKind::ModeMismatch => {
            repopulate_mode_mismatch_chain(b, program, file)
        }
        tsox_frontend::ast::mig::m3e::RepopulateDiagnosticKind::ModuleNotFound => {
            repopulate_module_not_found_chain(b, program, file, info)
        }
    }
}

fn repopulate_mode_mismatch_chain(
    b: &BuildInfoDiagnosticWithFileName,
    program: &tsox_compile::compiler::Program,
    file: Option<&Arc<SourceFile>>,
) -> Diagnostic {
    let Some(file) = file else {
        return b.to_diagnostic_without_repopulate(program, file);
    };

    let details =
        tsox_checker::checker::mig::m1c::r26k5_defs::create_mode_mismatch_details_worker(
            program, file,
        );

    let next_chain = b
        .message_chain
        .iter()
        .map(|msg| msg.to_diagnostic(program, Some(file)))
        .collect();

    tsox_frontend::ast::mig::m3d_2::new_diagnostic_from_serialized(
        Some(file.clone()),
        tsox_core::core::text::TextRange {
            pos: b.pos,
            end: b.end,
        },
        details.message.code(),
        details.message.category(),
        details.message.key(),
        tsox_core::diagnostics::mig::w11a::stringify_args(&details.args),
        next_chain,
        Vec::new(),
        false,
        false,
        false,
    )
}

fn repopulate_module_not_found_chain(
    b: &BuildInfoDiagnosticWithFileName,
    program: &tsox_compile::compiler::Program,
    file: Option<&Arc<SourceFile>>,
    info: &RepopulateDiagnosticInfo,
) -> Diagnostic {
    let Some(file) = file else {
        return b.to_diagnostic_without_repopulate(program, file);
    };

    let package_name = if info.package_name.is_empty() {
        info.module_reference.clone()
    } else {
        info.package_name.clone()
    };

    let details =
        tsox_checker::checker::mig::m1c::r26k5_defs::create_module_not_found_chain_details(
            program,
            Some(file),
            &info.module_reference,
            info.mode,
            &package_name,
        );

    let next_chain = b
        .message_chain
        .iter()
        .map(|msg| msg.to_diagnostic(program, Some(file)))
        .collect();

    tsox_frontend::ast::mig::m3d_2::new_diagnostic_from_serialized(
        Some(file.clone()),
        tsox_core::core::text::TextRange {
            pos: b.pos,
            end: b.end,
        },
        details.message.code(),
        details.message.category(),
        details.message.key(),
        tsox_core::diagnostics::mig::w11a::stringify_args(&details.args),
        next_chain,
        Vec::new(),
        false,
        false,
        false,
    )
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticsOrBuildInfoDiagnosticsWithFileName {
    pub diagnostics: Vec<Diagnostic>,
    pub build_info_diagnostics: Vec<Box<BuildInfoDiagnosticWithFileName>>,
}

impl DiagnosticsOrBuildInfoDiagnosticsWithFileName {
    pub fn get_diagnostics(
        &mut self,
        program: &tsox_compile::compiler::Program,
        file: &Arc<SourceFile>,
    ) -> Vec<Diagnostic> {
        if !self.diagnostics.is_empty() {
            return self.diagnostics.clone();
        }
        self.diagnostics = self
            .build_info_diagnostics
            .iter()
            .map(|diag| diag.to_diagnostic(program, Some(file)))
            .collect();
        self.diagnostics.clone()
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            file_infos: Default::default(),
            options: None,
            referenced_map: ReferenceMap {
                references: Default::default(),
                referenced_by: std::sync::Mutex::new(None),
                reference_by: Once::new(),
            },
            semantic_diagnostics_per_file: Default::default(),
            emit_diagnostics_per_file: Default::default(),
            changed_files_set: std::sync::Mutex::new(Default::default()),
            affected_files_pending_emit: Default::default(),
            latest_changed_dts_file: std::sync::Mutex::new(String::new()),
            emit_signatures: Default::default(),
            has_errors: std::sync::Mutex::new(Tristate::default()),
            has_semantic_errors: std::sync::Mutex::new(false),
            check_pending: std::sync::Mutex::new(false),
            package_jsons: std::sync::Mutex::new(None),
            missing_package_jsons: std::sync::Mutex::new(None),
            build_info_emit_pending: Default::default(),
            has_errors_from_old_state: Tristate::default(),
            has_semantic_errors_from_old_state: false,
            all_files_excluding_default_library_file_once: Once::new(),
            package_jsons_from_old_state: None,
            missing_package_jsons_from_old_state: None,
            all_files_excluding_default_library_file: Vec::new(),
            has_changed_dts_file: std::sync::atomic::AtomicBool::new(false),
            has_emit_diagnostics: std::sync::atomic::AtomicBool::new(false),
            hash_with_text: false,
        }
    }
}

pub struct Snapshot {
    pub file_infos: SyncMap<Path, FileInfo>,
    pub options: Option<CompilerOptions>,
    pub referenced_map: ReferenceMap,
    pub semantic_diagnostics_per_file: SyncMap<Path, DiagnosticsOrBuildInfoDiagnosticsWithFileName>,
    pub emit_diagnostics_per_file: SyncMap<Path, DiagnosticsOrBuildInfoDiagnosticsWithFileName>,
    pub changed_files_set: std::sync::Mutex<HashSet<Path>>,
    pub affected_files_pending_emit: SyncMap<Path, FileEmitKind>,
    pub latest_changed_dts_file: std::sync::Mutex<String>,
    pub emit_signatures: SyncMap<Path, EmitSignature>,
    pub has_errors: std::sync::Mutex<Tristate>,
    pub has_semantic_errors: std::sync::Mutex<bool>,
    pub check_pending: std::sync::Mutex<bool>,
    pub package_jsons: std::sync::Mutex<Option<Vec<String>>>,
    pub missing_package_jsons: std::sync::Mutex<Option<Vec<String>>>,

    pub build_info_emit_pending: std::sync::atomic::AtomicBool,
    pub has_errors_from_old_state: Tristate,
    pub has_semantic_errors_from_old_state: bool,
    pub all_files_excluding_default_library_file_once: Once,
    pub package_jsons_from_old_state: Option<Vec<String>>,
    pub missing_package_jsons_from_old_state: Option<Vec<String>>,
    pub all_files_excluding_default_library_file: Vec<Arc<SourceFile>>,
    pub has_changed_dts_file: std::sync::atomic::AtomicBool,
    pub has_emit_diagnostics: std::sync::atomic::AtomicBool,

    pub hash_with_text: bool,
}

impl Snapshot {
    pub fn add_file_to_change_set(&mut self, file_path: Path) {
        self.changed_files_set.lock().unwrap().insert(file_path);
        self.build_info_emit_pending
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn add_file_to_affected_files_pending_emit(&mut self, file_path: Path, emit_kind: FileEmitKind) {
        let existing_kind = self
            .affected_files_pending_emit
            .load(&file_path)
            .unwrap_or(FileEmitKind::None);
        self.affected_files_pending_emit
            .store(file_path.clone(), existing_kind | emit_kind);
        if (emit_kind & FileEmitKind::DtsErrors) != FileEmitKind::None {
            self.emit_diagnostics_per_file.delete(&file_path);
        }
        self.build_info_emit_pending
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_all_files_excluding_default_library_file(
        &mut self,
        program: &tsox_compile::compiler::Program,
        first_source_file: Option<&Arc<SourceFile>>,
    ) -> Vec<Arc<SourceFile>> {
        self.all_files_excluding_default_library_file_once.call_once(|| {
            let files = program.get_source_files();
            self.all_files_excluding_default_library_file = Vec::with_capacity(files.len());
            let mut add_source_file = |file: &Arc<SourceFile>,
                                       all_files: &mut Vec<Arc<SourceFile>>| {
                if !program.is_source_file_default_library(&tsox_frontend::ast::mig::m3b_2::path(file)) {
                    all_files.push(file.clone());
                }
            };
            if let Some(first_source_file) = first_source_file {
                add_source_file(first_source_file, &mut self.all_files_excluding_default_library_file);
            }
            for file in &files {
                let is_first = first_source_file
                    .map(|first| std::ptr::eq(first.as_ref(), file.as_ref()))
                    .unwrap_or(false);
                if !is_first {
                    add_source_file(file, &mut self.all_files_excluding_default_library_file);
                }
            }
        });
        self.all_files_excluding_default_library_file.clone()
    }

    pub fn compute_signature_with_diagnostics(
        &self,
        file: &Arc<SourceFile>,
        text: &str,
        data: &crate::mig::m4y_3::WriteFileData,
    ) -> String {
        let mut builder = String::new();
        builder.push_str(&get_text_handling_source_map_for_signature(text, data));
        for diag in &data.diagnostics {
            diagnostic_to_string_builder(diag, file, &mut builder);
        }
        self.compute_hash(&builder)
    }

    pub fn compute_hash(&self, text: &str) -> String {
        compute_hash(text, self.hash_with_text)
    }

    pub fn can_use_incremental_state(&self) -> bool {
        let default_options = CompilerOptions::default();
        let options = self.options.as_ref().unwrap_or(&default_options);
        if !options.is_incremental() && options.build.is_true() {
            return false;
        }
        true
    }
}

pub fn get_text_handling_source_map_for_signature(
    text: &str,
    data: &crate::mig::m4y_3::WriteFileData,
) -> String {
    if data.source_map_url_pos != -1 {
        return text[..data.source_map_url_pos as usize].to_string();
    }
    text.to_string()
}

pub fn diagnostic_to_string_builder(
    diagnostic: &Diagnostic,
    file: &Arc<SourceFile>,
    builder: &mut String,
) {
    builder.push('\n');
    let same_file = diagnostic
        .file
        .as_ref()
        .map(|f| std::ptr::eq(f.as_ref(), file.as_ref()))
        .unwrap_or(false);
    if !same_file {
        builder.push_str(&tspath::ensure_path_is_non_module_name(
            &tsox_core::tspath::mig::m3i::get_relative_path_from_directory(
                &tspath::get_directory_path(&tsox_frontend::ast::mig::m3b_2::path(file)),
                &diagnostic
                    .file
                    .as_ref()
                    .map(|f| tsox_frontend::ast::mig::m3b_2::path(f))
                    .unwrap_or_default(),
                &tspath::ComparePathsOptions::default(),
            ),
        ));
    }
    if diagnostic.file.is_some() {
        builder.push_str(&format!(
            "({},{}): ",
            diagnostic.loc.pos,
            diagnostic.loc.end - diagnostic.loc.pos
        ));
    }
    builder.push_str(diagnostic.category.name());
    builder.push_str(&format!("{}: ", diagnostic.code));
    builder.push_str(diagnostic.message_key);
    builder.push('\n');
    for arg in &diagnostic.message_args {
        builder.push_str(arg);
        builder.push('\n');
    }
    for chain in &diagnostic.message_chain {
        diagnostic_to_string_builder(chain, file, builder);
    }
    for info in &diagnostic.related_information {
        diagnostic_to_string_builder(info, file, builder);
    }
}
