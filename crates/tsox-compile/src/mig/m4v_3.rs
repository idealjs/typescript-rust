#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tsox_core::diagnostics::messages_generated as msg;
use tsox_core::tspath::directory_separator::Path;
use tsox_frontend::ast::diagnostic::{Diagnostic, DiagnosticsCollection};
use tsox_frontend::ast::node_source_file::SourceFile;

use super::m3l_cm::{SupplementalFileCollisionError, TransformError};
use super::m4v::{
    FileIncludeKind, FileIncludeReason, IncludeExplainingDiagnostic, ProcessingDiagnostic,
    ProcessingDiagnosticData, ProcessingDiagnosticKind, ReferenceFileLocation,
};
use super::m4v_2::RedirectsFile;
use super::m3l_cm::Mapper;
use super::m3l_cm_2::{
    check_supplemental_file_name_collisions, transform_and_parse, Project as ContentMapperProject,
    SourceFiles as ContentMapperSourceFiles,
};
use super::m4x_2::HasFileName;
use crate::compiler::Program;
use tsox_core::core::compiler_options_kinds::ModuleKind;
use tsox_core::core::mig::m3j::{ensure_script_kind_from_file_name, ScriptKind};
use tsox_core::core::mig::m3j_2::get_spelling_suggestion_for_strings;
use tsox_core::core::work_group::{new_work_group, WorkGroup};
use tsox_core::tspath::to_file_name_lower_case;
use tsox_frontend::ast::{is_external_or_common_js_module, is_object_literal_expression};
use tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions;
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;
use tsox_tsoptions::mig::m5h_5::LIB_MAP;
use tsox_tsoptions::mig::m5i2_4::get_parsed_command_line_of_config_file_path;
use tsox_tsoptions::mig::m5j_2::ParseConfigHost;
use tsox_tsoptions::module::PackageId;
use tsox_tsoptions::tsoptions::{ExtendedConfigCache, ParsedCommandLine};
use tsox_tsoptions::vfs::cachedvfs::CachedFS;
use tsox_tsoptions::vfs::FS;

pub struct CompilerHostImpl {
    pub current_directory: String,
    pub fs: Arc<dyn FS>,
    pub default_library_path: String,
    pub extended_config_cache: ExtendedConfigCache,
    pub trace: TraceFn,
    pub content_mapper_project: Option<Arc<dyn ContentMapperProject>>,
}

pub type TraceFn = Box<dyn Fn(&tsox_core::diagnostics::Message, &[String]) + Send + Sync>;

type ObjectLiteralExpression = Arc<tsox_frontend::ast::Node>;

#[derive(Debug)]
pub struct ContentMapperError(pub String);

impl ContentMapperError {
    pub fn project_unavailable() -> Self {
        Self("content mapper project is unavailable".to_string())
    }
}

impl From<TransformError> for ContentMapperError {
    fn from(err: TransformError) -> Self {
        Self(err.to_string())
    }
}

impl From<SupplementalFileCollisionError> for ContentMapperError {
    fn from(err: SupplementalFileCollisionError) -> Self {
        Self(err.to_string())
    }
}

fn cached_fs_from(fs: Arc<dyn FS>) -> Arc<dyn FS> {
    Arc::new(CachedFS::new(fs))
}

fn parse_source_file(
    opts: &SourceFileParseOptions,
    text: &str,
    _script_kind: ScriptKind,
) -> Arc<SourceFile> {
    Arc::new(tsox_frontend::parser::Parser::parse_source_file_text(
        &opts.file_name,
        text.to_string(),
    ))
}

pub fn new_cached_fs_compiler_host(
    current_directory: String,
    fs: Arc<dyn FS>,
    default_library_path: String,
    extended_config_cache: ExtendedConfigCache,
    trace: Option<TraceFn>,
    content_mapper_project: Option<Arc<dyn ContentMapperProject>>,
) -> CompilerHostImpl {
    new_compiler_host(
        current_directory,
        cached_fs_from(fs),
        default_library_path,
        extended_config_cache,
        trace,
        content_mapper_project,
    )
}

pub fn new_compiler_host(
    current_directory: String,
    fs: Arc<dyn FS>,
    default_library_path: String,
    extended_config_cache: ExtendedConfigCache,
    trace: Option<TraceFn>,
    content_mapper_project: Option<Arc<dyn ContentMapperProject>>,
) -> CompilerHostImpl {
    let trace = trace.unwrap_or_else(|| Box::new(|_msg: &tsox_core::diagnostics::Message, _args: &[String]| {}));
    CompilerHostImpl {
        current_directory,
        fs,
        default_library_path,
        extended_config_cache,
        trace,
        content_mapper_project,
    }
}

impl CompilerHostImpl {
    pub fn fs(&self) -> &Arc<dyn FS> {
        &self.fs
    }

    pub fn default_library_path(&self) -> &str {
        &self.default_library_path
    }

    pub fn get_current_directory(&self) -> &str {
        &self.current_directory
    }

    pub fn trace(&self, msg_: &tsox_core::diagnostics::Message, args: &[String]) {
        (self.trace)(msg_, args);
    }

    pub fn get_source_file(&self, opts: &SourceFileParseOptions) -> Option<Arc<SourceFile>> {
        let text = self.fs.read_file(&opts.file_name)?;
        Some(parse_source_file(opts, &text, ensure_script_kind_from_file_name(&opts.file_name)))
    }

    pub fn get_content_mapped_source_files(
        &self,
        parse_options: &SourceFileParseOptions,
        mapper: &Mapper,
    ) -> Result<ContentMapperSourceFiles, ContentMapperError> {
        let Some(project) = &self.content_mapper_project else {
            return Err(ContentMapperError::project_unavailable());
        };
        let Some(content) = self.fs.read_file(&parse_options.file_name) else {
            return Ok(ContentMapperSourceFiles::default());
        };
        let files = transform_and_parse(
            tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions {
                file_name: parse_options.file_name.clone(),
                path: parse_options.path.clone(),
                external_module_indicator_options: Default::default(),
            },
            &content,
            mapper,
            project.as_ref(),
        )?;
        check_supplemental_file_name_collisions(&files, &|name: &str| self.fs.file_exists(name))?;
        Ok(files)
    }

    pub fn content_mapper_project(&self) -> Option<&Arc<dyn ContentMapperProject>> {
        self.content_mapper_project.as_ref()
    }

    pub fn get_resolved_project_reference(&self, file_name: &str, path: &Path) -> Option<ParsedCommandLine> {
        let sys = ParseConfigHost {
            fs: self.fs.clone(),
            current_directory: self.current_directory.clone(),
        };
        let (command_line, _) = get_parsed_command_line_of_config_file_path(
            file_name,
            path.as_str(),
            None,
            None,
            &sys,
            None,
        );
        command_line
    }
}

impl HasFileName for RedirectsFile {
    fn path(&self) -> String {
        self.path.0.clone()
    }

    fn file_name(&self) -> String {
        self.file_name.clone()
    }
}

impl HasFileName for Arc<SourceFile> {
    fn path(&self) -> String {
        tsox_frontend::ast::mig::m3b_2::path(self)
    }

    fn file_name(&self) -> String {
        self.file_name.clone()
    }
}

#[derive(Default)]
pub struct IncludeProcessor {
    pub file_include_reasons: HashMap<Path, Vec<Arc<FileIncludeReason>>>,
    pub processing_diagnostics: Vec<Arc<ProcessingDiagnostic>>,

    pub reason_to_reference_location: Mutex<HashMap<usize, Arc<ReferenceFileLocation>>>,
    pub include_reason_to_related_info: Mutex<HashMap<usize, Option<Diagnostic>>>,
    pub redirect_and_file_format: Mutex<HashMap<Path, Option<Vec<Diagnostic>>>>,
    pub computed_diagnostics: Mutex<Option<Arc<DiagnosticsCollection>>>,
    pub compiler_options_syntax: Mutex<Option<ObjectLiteralExpression>>,
}

pub fn update_file_include_processor(p: &mut Program) {
    let (file_include_reasons, processing_diagnostics) = match p.include_processor.as_deref_mut() {
        Some(old) => (
            std::mem::take(&mut old.file_include_reasons),
            std::mem::take(&mut old.processing_diagnostics),
        ),
        None => (HashMap::new(), Vec::new()),
    };
    p.include_processor = Some(Box::new(IncludeProcessor {
        file_include_reasons,
        processing_diagnostics,
        ..Default::default()
    }));
}

impl IncludeProcessor {
    pub fn get_diagnostics(&self, p: &Program) -> Arc<DiagnosticsCollection> {
        let mut computed = self.computed_diagnostics.lock().unwrap();
        if computed.is_none() {
            let collection = DiagnosticsCollection::new();
            for d in &self.processing_diagnostics {
                collection.add(self.to_diagnostic(d, p));
            }
            for resolutions in p.resolved_modules.values() {
                for resolved_module in resolutions.values() {
                    for diag in &resolved_module.resolution_diagnostics {
                        collection.add(new_compiler_diagnostic(*diag.message, diag.args.clone()));
                    }
                }
            }
            for type_resolutions in p.type_resolutions_in_file.values() {
                for resolved_type_ref in type_resolutions.values() {
                    for diag in &resolved_type_ref.resolution_diagnostics {
                        collection.add(new_compiler_diagnostic(*diag.message, diag.args.clone()));
                    }
                }
            }
            *computed = Some(Arc::new(collection));
        }
        computed.as_ref().unwrap().clone()
    }

    fn to_diagnostic(&self, d: &ProcessingDiagnostic, program: &Program) -> Diagnostic {
        match d.kind {
            ProcessingDiagnosticKind::UnknownReference => {
                let ref_ = d.as_file_include_reason();
                let loc = ref_.get_referenced_location(program);
                match ref_.kind {
                    FileIncludeKind::TypeReferenceDirective => loc
                        .diagnostic_at(
                            msg::CANNOT_FIND_TYPE_DEFINITION_FILE_FOR_0,
                            vec![loc.ref_.as_ref().unwrap().file_name.clone()],
                        )
                        .unwrap(),
                    FileIncludeKind::LibReferenceDirective => {
                        let lib_name =
                            to_file_name_lower_case(loc.ref_.as_ref().unwrap().file_name.as_str());
                        let unqualified = lib_name.strip_prefix("lib.").unwrap_or(&lib_name);
                        let unqualified = unqualified.strip_suffix(".d.ts").unwrap_or(unqualified);
                        let suggestion = get_spelling_suggestion_for_strings(
                            unqualified,
                            LIB_MAP.iter().map(|(name, _)| name.to_string()),
                        );
                        let message = if suggestion.is_some() {
                            msg::CANNOT_FIND_LIB_DEFINITION_FOR_0_DID_YOU_MEAN_1
                        } else {
                            msg::CANNOT_FIND_LIB_DEFINITION_FOR_0
                        };
                        loc.diagnostic_at(message, vec![lib_name, suggestion.unwrap_or_default()])
                            .unwrap()
                    }
                    _ => panic!("unknown include kind"),
                }
            }
            ProcessingDiagnosticKind::ExplainingFileInclude => {
                self.create_diagnostic_explaining_file(d, program)
            }
        }
    }

    fn create_diagnostic_explaining_file(&self, d: &ProcessingDiagnostic, program: &Program) -> Diagnostic {
        let diag = d.as_include_explaining_diagnostic();
        let mut include_details: Vec<Diagnostic> = Vec::new();
        let mut related_info: Vec<Diagnostic> = Vec::new();
        let mut redirect_info: Vec<Diagnostic> = Vec::new();
        let mut preferred_location: Option<Arc<FileIncludeReason>> = None;
        let mut seen_reasons: HashSet<usize> = HashSet::new();

        let diagnostic_reason = diag
            .diagnostic_reason_opt
            .as_ref()
            .map(|r| Arc::new(*r.clone_reason()));
        if let Some(reason) = &diagnostic_reason {
            if reason.is_referenced_file()
                && !self.get_reference_location(reason, program).is_synthetic
            {
                preferred_location = Some(reason.clone());
            }
        }

        let mut process_include =
            |include_reason: &Arc<FileIncludeReason>,
             preferred_location: &mut Option<Arc<FileIncludeReason>>,
             include_details: &mut Vec<Diagnostic>,
             related_info: &mut Vec<Diagnostic>,
             seen_reasons: &mut HashSet<usize>| {
                if !seen_reasons.insert(Arc::as_ptr(include_reason) as usize) {
                    return;
                }
                let reason_arc = Arc::new(*include_reason.clone_reason());
                if let Some(reason_diag) = reason_arc.to_diagnostic(program, false) {
                    include_details.push(reason_diag);
                }
                self.process_related_info(include_reason, program, preferred_location, related_info);
            };

        if !diag.file.as_str().is_empty() {
            if let Some(reasons) = self.file_include_reasons.get(&diag.file) {
                include_details = Vec::with_capacity(reasons.len());
                for reason in reasons {
                    process_include(
                        reason,
                        &mut preferred_location,
                        &mut include_details,
                        &mut related_info,
                        &mut seen_reasons,
                    );
                }
            }
            redirect_info = self.explain_redirect_and_implied_format(
                program,
                &diag.file,
                &|file_name: &str| file_name.to_string(),
            );
        }
        if let Some(reason) = &diagnostic_reason {
            process_include(
                reason,
                &mut preferred_location,
                &mut include_details,
                &mut related_info,
                &mut seen_reasons,
            );
        }

        let mut chain: Vec<Diagnostic> = Vec::new();
        if !include_details.is_empty() && (preferred_location.is_none() || seen_reasons.len() != 1) {
            let mut file_reason =
                new_compiler_diagnostic(msg::THE_FILE_IS_IN_THE_PROGRAM_BECAUSE_COLON, Vec::new());
            file_reason.message_chain = include_details;
            chain.push(file_reason);
        }
        chain.extend(redirect_info);

        let mut result = match &preferred_location {
            Some(location) => self
                .get_reference_location(location, program)
                .diagnostic_at(diag.message, diag.args.clone())
                .unwrap_or_else(|| new_compiler_diagnostic(diag.message, diag.args.clone())),
            None => new_compiler_diagnostic(diag.message, diag.args.clone()),
        };
        if !chain.is_empty() {
            result.message_chain = chain;
        }
        if !related_info.is_empty() {
            result.related_information = related_info;
        }
        result
    }

    fn process_related_info(
        &self,
        include_reason: &Arc<FileIncludeReason>,
        program: &Program,
        preferred_location: &mut Option<Arc<FileIncludeReason>>,
        related_info: &mut Vec<Diagnostic>,
    ) {
        if preferred_location.is_none()
            && include_reason.is_referenced_file()
            && !self.get_reference_location(include_reason, program).is_synthetic
        {
            *preferred_location = Some(include_reason.clone());
        } else if !preferred_location
            .as_ref()
            .map(|preferred| Arc::ptr_eq(preferred, include_reason))
            .unwrap_or(false)
        {
            if let Some(info) = self.get_related_info(include_reason, program) {
                related_info.push(info);
            }
        }
    }

    pub fn add_processing_diagnostics(&mut self, d: Vec<Arc<ProcessingDiagnostic>>) {
        self.processing_diagnostics.extend(d);
    }

    pub fn add_processing_diagnostics_for_file_casing(
        &mut self,
        file: &Path,
        existing_casing: &str,
        current_casing: &str,
        reason: &Arc<FileIncludeReason>,
    ) {
        let existing_reasons = self.file_include_reasons.get(file);
        if !reason.is_referenced_file()
            && existing_reasons
                .map(|reasons| reasons.iter().any(|r| r.is_referenced_file()))
                .unwrap_or(false)
        {
            self.add_processing_diagnostics(vec![Arc::new(ProcessingDiagnostic {
                kind: ProcessingDiagnosticKind::ExplainingFileInclude,
                data: ProcessingDiagnosticData::IncludeExplainingDiagnostic(Box::new(IncludeExplainingDiagnostic {
                    file: file.clone(),
                    diagnostic_reason: std::ptr::null(),
                    diagnostic_reason_opt: Some(reason.clone_reason()),
                    message: msg::ALREADY_INCLUDED_FILE_NAME_0_DIFFERS_FROM_FILE_NAME_1_ONLY_IN_CASING,
                    args: vec![existing_casing.to_string(), current_casing.to_string()],
                })),
            })]);
        } else {
            self.add_processing_diagnostics(vec![Arc::new(ProcessingDiagnostic {
                kind: ProcessingDiagnosticKind::ExplainingFileInclude,
                data: ProcessingDiagnosticData::IncludeExplainingDiagnostic(Box::new(IncludeExplainingDiagnostic {
                    file: file.clone(),
                    diagnostic_reason: std::ptr::null(),
                    diagnostic_reason_opt: Some(reason.clone_reason()),
                    message: msg::FILE_NAME_0_DIFFERS_FROM_ALREADY_INCLUDED_FILE_NAME_1_ONLY_IN_CASING,
                    args: vec![current_casing.to_string(), existing_casing.to_string()],
                })),
            })]);
        }
    }

    pub fn get_reference_location(&self, r: &Arc<FileIncludeReason>, program: &Program) -> Arc<ReferenceFileLocation> {
        let key = Arc::as_ptr(r) as usize;
        {
            let cache = self.reason_to_reference_location.lock().unwrap();
            if let Some(existing) = cache.get(&key) {
                return existing.clone();
            }
        }
        let loc = Arc::new(r.get_referenced_location(program));
        self.reason_to_reference_location
            .lock()
            .unwrap()
            .entry(key)
            .or_insert_with(|| loc.clone());
        loc
    }

    pub fn get_compiler_options_object_literal_syntax(&self, program: &Program) -> Option<ObjectLiteralExpression> {
        let mut syntax = self.compiler_options_syntax.lock().unwrap();
        if syntax.is_none() {
            if let Some(config_file) = program.opts.config.config_file.as_ref() {
                let object_literal = tsox_tsoptions::mig::m5j::get_ts_config_object_literal_expression(
                    Some(config_file.source_file.as_ref()),
                );
                if let Some(object_literal) = object_literal {
                    let properties = match &object_literal.data {
                        tsox_frontend::ast::NodeData::ObjectLiteralExpression(data) => &data.properties,
                        _ => return syntax.clone(),
                    };
                    for property in &properties.nodes {
                        if !tsox_frontend::ast::is_property_assignment(property) {
                            continue;
                        }
                        let tsox_frontend::ast::NodeData::PropertyAssignment(prop_data) = &property.data
                        else {
                            continue;
                        };
                        let Some(prop_name) =
                            tsox_frontend::ast::mig::m3h::try_get_text_of_property_name(&prop_data.name)
                        else {
                            continue;
                        };
                        if prop_name != "compilerOptions" {
                            continue;
                        }
                        let initializer = prop_data.initializer.clone();
                        if is_object_literal_expression(&initializer) {
                            *syntax = Some(initializer);
                        }
                        break;
                    }
                }
            }
        }
        syntax.clone()
    }

    pub fn get_related_info(&self, r: &Arc<FileIncludeReason>, program: &Program) -> Option<Diagnostic> {
        let key = Arc::as_ptr(r) as usize;
        {
            let cache = self.include_reason_to_related_info.lock().unwrap();
            if let Some(existing) = cache.get(&key) {
                return existing.clone();
            }
        }
        let related_info = r.to_related_info(program);
        self.include_reason_to_related_info
            .lock()
            .unwrap()
            .entry(key)
            .or_insert_with(|| related_info.clone());
        related_info
    }

    pub fn explain_redirect_and_implied_format(
        &self,
        program: &Program,
        file_path: &Path,
        to_file_name: &dyn Fn(&str) -> String,
    ) -> Vec<Diagnostic> {
        {
            let cache = self.redirect_and_file_format.lock().unwrap();
            if let Some(existing) = cache.get(file_path) {
                return existing.clone().unwrap_or_default();
            }
        }
        let redirects_file = program.redirect_files_by_path.get(file_path.as_str());
        let source_file = if redirects_file.is_some() {
            None
        } else {
            program.get_source_file_by_path(file_path.as_str())
        };
        if redirects_file.is_none() && source_file.is_none() {
            return Vec::new();
        }
        let file: &dyn HasFileName = match redirects_file {
            Some(redirects) => redirects,
            None => source_file.as_ref().unwrap(),
        };
        let mut result: Vec<Diagnostic> = Vec::new();
        let source = program
            .output_file_to_project_reference_source
            .get(file.path().as_str())
            .cloned()
            .unwrap_or_else(|| file.file_name());
        if source != file.file_name() {
            result.push(new_compiler_diagnostic(
                msg::FILE_IS_OUTPUT_OF_PROJECT_REFERENCE_SOURCE_0,
                vec![to_file_name(&source)],
            ));
        }
        if let Some(redirects) = redirects_file {
            let target_file = program.get_source_file_by_path(redirects.target.as_str());
            result.push(new_compiler_diagnostic(
                msg::FILE_REDIRECTS_TO_FILE_0,
                vec![to_file_name(&target_file.as_ref().unwrap().file_name())],
            ));
        }
        if let Some(source_file) = &source_file {
            if is_external_or_common_js_module(source_file) {
                let meta_data =
                    program.get_source_file_meta_data(file.path().as_str()).unwrap_or_default();
                match program.get_implied_node_format_for_emit(source_file) {
                    ModuleKind::ESNext => {
                        if meta_data.package_json_type == "module" {
                            result.push(new_compiler_diagnostic(
                                msg::FILE_IS_ECMASCRIPT_MODULE_BECAUSE_0_HAS_FIELD_TYPE_WITH_VALUE_MODULE,
                                vec![to_file_name(&format!("{}/package.json", meta_data.package_json_directory))],
                            ));
                        }
                    }
                    ModuleKind::CommonJS => {
                        if !meta_data.package_json_type.is_empty() {
                            result.push(new_compiler_diagnostic(
                                msg::FILE_IS_COMMONJS_MODULE_BECAUSE_0_HAS_FIELD_TYPE_WHOSE_VALUE_IS_NOT_MODULE,
                                vec![to_file_name(&format!("{}/package.json", meta_data.package_json_directory))],
                            ));
                        } else if !meta_data.package_json_directory.is_empty() {
                            if meta_data.package_json_type.is_empty() {
                                result.push(new_compiler_diagnostic(
                                    msg::FILE_IS_COMMONJS_MODULE_BECAUSE_0_DOES_NOT_HAVE_FIELD_TYPE,
                                    vec![to_file_name(&format!("{}/package.json", meta_data.package_json_directory))],
                                ));
                            }
                        } else {
                            result.push(new_compiler_diagnostic(
                                msg::FILE_IS_COMMONJS_MODULE_BECAUSE_PACKAGE_JSON_WAS_NOT_FOUND,
                                Vec::new(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
        self.redirect_and_file_format
            .lock()
            .unwrap()
            .entry(file_path.clone())
            .or_insert_with(|| Some(result.clone()));
        result
    }
}

pub struct ParseTaskData {
    pub tasks: Mutex<HashMap<String, Arc<Mutex<ParseTask>>>>,
    pub lowest_depth: Mutex<i32>,
    pub started_sub_tasks: Mutex<bool>,
    pub package_id: Mutex<Option<PackageId>>,
}

impl ParseTaskData {
    pub fn new(task: Arc<Mutex<ParseTask>>) -> Self {
        let key = task.lock().unwrap().normalized_file_path.clone();
        let mut tasks = HashMap::new();
        tasks.insert(key, task);
        Self {
            tasks: Mutex::new(tasks),
            lowest_depth: Mutex::new(i32::MAX),
            started_sub_tasks: Mutex::new(false),
            package_id: Mutex::new(None),
        }
    }
}

pub struct FilesParser {
    pub wg: Box<dyn WorkGroup>,
    pub task_data_by_path: Mutex<HashMap<Path, Arc<ParseTaskData>>>,
    pub max_depth: i32,
}

impl Default for FilesParser {
    fn default() -> Self {
        Self {
            wg: new_work_group(true),
            task_data_by_path: Mutex::new(HashMap::new()),
            max_depth: 0,
        }
    }
}

pub use super::m4v_2::ParseTask;
