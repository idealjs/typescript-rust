#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options_kinds::{ModuleKind, ModuleResolutionKind, ResolutionMode};
use tsox_core::core::text::TextRange;
use tsox_core::core::work_group::new_work_group;
use tsox_core::diagnostics::{messages_generated as msg, Category, Message};
use tsox_core::tspath::directory_separator::Path;
use tsox_core::tspath::{
    combine_paths, file_extension_is_one_of, get_canonical_file_name, get_directory_path,
    get_normalized_absolute_path, get_normalized_absolute_path_without_root, has_extension,
    has_js_file_extension, is_rooted_disk_path, normalize_path, normalize_slashes,
    remove_trailing_directory_separator, to_file_name_lower_case, to_path,
    ComparePathsOptions, EXTENSION_CJS, EXTENSION_CTS, EXTENSION_MJS, EXTENSION_MTS, EXTENSION_TS,
};
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3b::get_resolution_mode_override;
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::ast::mig::m3d_2::{new_compiler_diagnostic, new_diagnostic_chain};
use tsox_frontend::ast::mig::m3e_2::{get_external_module_indicator_options, SourceFileParseOptions};
use tsox_frontend::ast::mig::m3g_3::should_transform_import_call;
use tsox_frontend::ast::mig::m3h::walk_up_parenthesized_expressions;
use tsox_frontend::ast::mig::x4ast::{
    get_emit_module_format_of_file_worker, get_implied_node_format_for_emit_worker,
    get_implied_node_format_for_file, get_jsx_implicit_import_base, get_jsx_runtime_import,
    SourceFileMetaData,
};
use tsox_frontend::ast::mig::x6a::is_exclusively_type_only_import_or_export;
use tsox_frontend::ast::node_source_file::{FileReference, ScriptKind, SourceFile};
use tsox_frontend::ast::positionmap::PositionMap;
use tsox_frontend::ast::{
    is_export_declaration, is_external_module_reference, is_import_call, is_import_declaration,
    is_import_equals_declaration, is_import_type_node, is_in_js_file, is_jsdoc_import_tag,
    is_literal_type_node, is_require_call, is_source_file_js, ImportDeclarationData, Node,
    NodeData, NodeFlags, SyntaxKind,
};
use tsox_tsoptions::mig::m5h_2::{get_default_lib_file_name, get_lib_file_name};
use tsox_tsoptions::mig::m5h_5::LIB_MAP;
use tsox_tsoptions::mig::m5i_3::{
    get_supported_extensions, get_supported_extensions_with_json_if_resolve_json_module,
};
use tsox_tsoptions::module::mig::m3i::{get_automatic_type_directive_names, get_resolution_diagnostic};
use tsox_tsoptions::module::{
    DiagAndArgs, PackageId, ResolvedModule, ResolvedTypeReferenceDirective, Resolver,
};
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_tsoptions::module::INFERRED_TYPES_CONTAINING_FILE;

use crate::compiler::{CompilerHost, DuplicateSourceFile, ProgramOptions, ResolutionHostAdapter};

use super::m3l_cm::{
    DiagnosticDirectiveError, DiagnosticDirectiveErrorKind, DiagnosticDirectivePolicy,
    InitializeError, InitializeErrorKind,
    InvalidVirtualExtensionError, Mapper, ProjectError, ProjectErrorKind,
    SupplementalFileCollisionError, TransformError, TransformErrorKind,
};
use super::m3l_cm_2::DiagnosticDirectives;
use super::m4v::{
    AutomaticTypeDirectiveFileData, FileIncludeKind, FileIncludeReason, FileIncludeReasonData,
    ReferencedFileData, IncludeExplainingDiagnostic, ProcessingDiagnostic, ProcessingDiagnosticData,
    ProcessingDiagnosticKind,
};
use super::m4v_3::{FilesParser, IncludeProcessor, ParseTaskData};
use super::m4x_2::{FileLoader as ProjectReferenceLoader, HasFileName as MapperHasFileName, ProjectReferenceFileMapper};
use super::m4x_3::{create_project_reference_parse_tasks, ProjectReferenceParser};
use tsox_tsoptions::mig::m5h_3::ContentMapper;

// r49-k01: content mapper 诊断在 tsox-core messages_generated 缺号(100020+ 未生成),按 Go 原文落本文件,待 tsox-core 补齐后迁回

pub static THE_CONTENT_MAPPER_0_FAILED_TO_TRANSFORM_THIS_FILE: Message = Message {
    code: 100025,
    category: Category::Error,
    key: "The_content_mapper_0_failed_to_transform_this_file_100025",
    text: "The content mapper '{0}' failed to transform this file.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_0_FAILED_1_TIMES_AND_WILL_NOT_BE_USED: Message = Message {
    code: 100026,
    category: Category::Error,
    key: "The_content_mapper_0_failed_1_times_and_will_not_be_used_100026",
    text: "The content mapper '{0}' failed {1} times and will not be used.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_0_DID_NOT_PROVIDE_THE_REQUIRED_POSITION_MAPPINGS: Message = Message {
    code: 100027,
    category: Category::Error,
    key: "The_content_mapper_0_did_not_provide_the_required_position_mappings_100027",
    text: "The content mapper '{0}' did not provide the required position mappings.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_0_PRODUCED_A_POSITION_MAPPING_THAT_POINTS_OUTSIDE_THE_ORIGINAL_CONTENT_ORIGINAL_OFFSET_1: Message = Message {
    code: 100028,
    category: Category::Error,
    key: "The_content_mapper_0_produced_a_position_mapping_that_points_outside_the_original_content_original_o_100028",
    text: "The content mapper '{0}' produced a position mapping that points outside the original content (original offset {1}).",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_0_PRODUCED_A_VERBATIM_MAPPING_THAT_DOES_NOT_MATCH_THE_ORIGINAL_CONTENT_VIRTUAL_OFFSET_1_ORIGINAL_OFFSET_2: Message = Message {
    code: 100029,
    category: Category::Error,
    key: "The_content_mapper_0_produced_a_verbatim_mapping_that_does_not_match_the_original_content_virtual_",
    text: "The content mapper '{0}' produced a verbatim mapping that does not match the original content (virtual offset {1}, original offset {2}).",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_0_PRODUCED_OVERLAPPING_OR_OUT_OF_ORDER_POSITION_MAPPINGS_NEAR_VIRTUAL_OFFSET_1: Message = Message {
    code: 100037,
    category: Category::Error,
    key: "The_content_mapper_0_produced_overlapping_or_out_of_order_position_mappings_near_virtual_offset_1_100037",
    text: "The content mapper '{0}' produced overlapping or out-of-order position mappings (near virtual offset {1}).",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_0_PRODUCED_INVALID_MAPPING_FEATURES_NEAR_ORIGINAL_OFFSET_1: Message = Message {
    code: 100039,
    category: Category::Error,
    key: "The_content_mapper_0_produced_invalid_mapping_features_near_original_offset_1_100039",
    text: "The content mapper '{0}' produced invalid mapping features near original offset {1}.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_0_PRODUCED_A_POSITION_MAPPING_WITH_AN_INVALID_KIND_NEAR_VIRTUAL_OFFSET_1: Message = Message {
    code: 100040,
    category: Category::Error,
    key: "The_content_mapper_0_produced_a_position_mapping_with_an_invalid_kind_near_virtual_offset_1_100040",
    text: "The content mapper '{0}' produced a position mapping with an invalid kind (near virtual offset {1}).",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_PROCESS_COULD_NOT_BE_STARTED_OR_INITIALIZED: Message = Message {
    code: 100041,
    category: Category::Message,
    key: "The_content_mapper_process_could_not_be_started_or_initialized_100041",
    text: "The content mapper process could not be started or initialized.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_PROCESS_FAILED_WHILE_HANDLING_THE_TRANSFORM_REQUEST: Message = Message {
    code: 100042,
    category: Category::Message,
    key: "The_content_mapper_process_failed_while_handling_the_transform_request_100042",
    text: "The content mapper process failed while handling the transform request.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_AN_INVALID_TRANSFORM_RESPONSE: Message = Message {
    code: 100043,
    category: Category::Message,
    key: "The_content_mapper_returned_an_invalid_transform_response_100043",
    text: "The content mapper returned an invalid transform response.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_SELECTED_UNSUPPORTED_POSITION_ENCODING_0: Message = Message {
    code: 100045,
    category: Category::Message,
    key: "The_content_mapper_selected_unsupported_position_encoding_0_100045",
    text: "The content mapper selected unsupported position encoding '{0}'.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_DIAGNOSTIC_SOURCE_MUST_NOT_BE_EMPTY: Message = Message {
    code: 100046,
    category: Category::Message,
    key: "The_content_mapper_diagnostic_source_must_not_be_empty_100046",
    text: "The content mapper diagnostic source must not be empty.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_DIAGNOSTIC_SOURCE_0_IS_RESERVED_BY_TYPESCRIPT: Message = Message {
    code: 100047,
    category: Category::Message,
    key: "The_content_mapper_diagnostic_source_0_is_reserved_by_TypeScript_100047",
    text: "The content mapper diagnostic source '{0}' is reserved by TypeScript.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_A_PROJECT_RESPONSE_THAT_COULD_NOT_BE_DECODED: Message = Message {
    code: 100048,
    category: Category::Message,
    key: "The_content_mapper_returned_a_project_response_that_could_not_be_decoded_100048",
    text: "The content mapper returned a project response that could not be decoded.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_PROCESS_FAILED_WHILE_HANDLING_THE_PROJECT_REQUEST: Message = Message {
    code: 100049,
    category: Category::Message,
    key: "The_content_mapper_process_failed_while_handling_the_project_request_100049",
    text: "The content mapper process failed while handling the project request.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_DID_NOT_RETURN_CONFIGIDENTITY_WHICH_IS_REQUIRED_WHEN_THE_CONTENT_MAPPER_HAS_DYNAMICCONFIG_COLON_TRUE_IN_ITS_PACKAGE_JSON: Message = Message {
    code: 100050,
    category: Category::Message,
    key: "The_content_mapper_did_not_return_configIdentity_which_is_required_when_the_content_mapper_has_dynam_100050",
    text: "The content mapper did not return 'configIdentity', which is required when the content mapper has '\"dynamicConfig\": true' in its package.json.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_A_NON_ABSOLUTE_PATH_IN_WATCHEDFILES: Message = Message {
    code: 100051,
    category: Category::Message,
    key: "The_content_mapper_returned_a_non_absolute_path_in_watchedFiles_100051",
    text: "The content mapper returned a non-absolute path in 'watchedFiles'.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_CONFIGIDENTITY_WHICH_IS_ONLY_ALLOWED_WHEN_IT_DECLARES_DYNAMICCONFIG_COLON_TRUE_IN_ITS_PACKAGE_JSON: Message = Message {
    code: 100052,
    category: Category::Message,
    key: "The_content_mapper_returned_configIdentity_which_is_only_allowed_when_it_declares_dynamicConfig_Colo_100052",
    text: "The content mapper returned 'configIdentity', which is only allowed when it declares '\"dynamicConfig\": true' in its package.json.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_WATCHEDFILES_WHICH_IS_ONLY_ALLOWED_WHEN_IT_DECLARES_DYNAMICCONFIG_COLON_TRUE_IN_ITS_PACKAGE_JSON: Message = Message {
    code: 100053,
    category: Category::Message,
    key: "The_content_mapper_returned_watchedFiles_which_is_only_allowed_when_it_declares_dynamicConfig_Colon__100053",
    text: "The content mapper returned 'watchedFiles', which is only allowed when it declares '\"dynamicConfig\": true' in its package.json.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static CONTENT_MAPPER_SUPPLEMENTAL_OUTPUT_FILE_0_CONFLICTS_WITH_AN_EXISTING_FILE: Message = Message {
    code: 100054,
    category: Category::Message,
    key: "Content_mapper_supplemental_output_file_0_conflicts_with_an_existing_file_100054",
    text: "Content mapper supplemental output file '{0}' conflicts with an existing file.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_AN_OUTPUT_WITH_UNSUPPORTED_VIRTUAL_EXTENSION_0: Message = Message {
    code: 100056,
    category: Category::Message,
    key: "The_content_mapper_returned_an_output_with_unsupported_virtual_extension_0_100056",
    text: "The content mapper returned an output with unsupported virtual extension '{0}'.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_0_COULD_NOT_BE_INITIALIZED: Message = Message {
    code: 100057,
    category: Category::Error,
    key: "The_content_mapper_0_could_not_be_initialized_100057",
    text: "The content mapper '{0}' could not be initialized.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_COMMAND_0_COULD_NOT_BE_STARTED_COLON_1: Message = Message {
    code: 100058,
    category: Category::Message,
    key: "The_content_mapper_command_0_could_not_be_started_Colon_1_100058",
    text: "The content mapper command '{0}' could not be started: {1}",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_PROCESS_EXITED_BEFORE_RESPONDING_TO_THE_INITIALIZE_REQUEST_EXIT_CODE_0: Message = Message {
    code: 100059,
    category: Category::Message,
    key: "The_content_mapper_process_exited_before_responding_to_the_initialize_request_exit_code_0_100059",
    text: "The content mapper process exited before responding to the 'initialize' request (exit code {0}).",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_DID_NOT_RESPOND_TO_THE_INITIALIZE_REQUEST_WITHIN_0_SECONDS: Message = Message {
    code: 100060,
    category: Category::Message,
    key: "The_content_mapper_did_not_respond_to_the_initialize_request_within_0_seconds_100060",
    text: "The content mapper did not respond to the 'initialize' request within {0} seconds.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_AN_INITIALIZE_RESPONSE_THAT_COULD_NOT_BE_DECODED_COLON_0: Message = Message {
    code: 100061,
    category: Category::Message,
    key: "The_content_mapper_returned_an_initialize_response_that_could_not_be_decoded_Colon_0_100061",
    text: "The content mapper returned an 'initialize' response that could not be decoded: {0}",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_S_INITIALIZE_REQUEST_FAILED_COLON_0: Message = Message {
    code: 100062,
    category: Category::Message,
    key: "The_content_mapper_s_initialize_request_failed_Colon_0_100062",
    text: "The content mapper's 'initialize' request failed: {0}",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static DIAGNOSTIC_DIRECTIVE_0_RETURNED_BY_THE_CONTENT_MAPPER_HAS_AN_INVALID_RANGE: Message = Message {
    code: 100063,
    category: Category::Message,
    key: "Diagnostic_directive_0_returned_by_the_content_mapper_has_an_invalid_range_100063",
    text: "Diagnostic directive {0} returned by the content mapper has an invalid range.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_A_DIAGNOSTIC_DIRECTIVE_WITH_INVALID_POLICY_0: Message = Message {
    code: 100064,
    category: Category::Message,
    key: "The_content_mapper_returned_a_diagnostic_directive_with_invalid_policy_0_100064",
    text: "The content mapper returned a diagnostic directive with invalid policy '{0}'.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static DIAGNOSTIC_DIRECTIVE_0_RETURNED_BY_THE_CONTENT_MAPPER_MUST_SPECIFY_UNUSEDEXPECTDIRECTIVEINDEX_WHEN_THERE_IS_NOT_EXACTLY_ONE_UNUSEDEXPECTDIRECTIVEDIAGNOSTICS_ENTRY: Message = Message {
    code: 100065,
    category: Category::Message,
    key: "Diagnostic_directive_0_returned_by_the_content_mapper_must_specify_unusedExpectDirectiveIndex_when_t_100065",
    text: "Diagnostic directive {0} returned by the content mapper must specify 'unusedExpectDirectiveIndex' when there is not exactly one 'unusedExpectDirectiveDiagnostics' entry.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_CONTENT_MAPPER_RETURNED_DIAGNOSTIVE_DIRECTIVES_WITH_OVERLAPPING_VIRTUAL_RANGES: Message = Message {
    code: 100066,
    category: Category::Message,
    key: "The_content_mapper_returned_diagnostic_directives_with_overlapping_virtual_ranges_100066",
    text: "The content mapper returned diagnostic directives with overlapping virtual ranges.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static THE_INVALID_DIAGNOSTIC_DIRECTIVE_IS_IN_SUPPLEMENTAL_OUTPUT_0_RETURNED_BY_THE_CONTENT_MAPPER: Message = Message {
    code: 100067,
    category: Category::Message,
    key: "The_invalid_diagnostic_directive_is_in_supplemental_output_0_returned_by_the_content_mapper_100067",
    text: "The invalid diagnostic directive is in supplemental output {0} returned by the content mapper.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub static DIAGNOSTIC_DIRECTIVE_0_RETURNED_BY_THE_CONTENT_MAPPER_HAS_AN_INVALID_UNUSEDEXPECTDIRECTIVEINDEX: Message = Message {
    code: 100068,
    category: Category::Message,
    key: "Diagnostic_directive_0_returned_by_the_content_mapper_has_an_invalid_unusedExpectDirectiveIndex_100068",
    text: "Diagnostic directive {0} returned by the content mapper has an invalid 'unusedExpectDirectiveIndex'.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

pub const MAX_CONTENT_MAPPER_FAILURES: i32 = 5;
pub const EXTERNAL_HELPERS_MODULE_NAME_TEXT: &str = "tslib";
pub type HashSetTspath = HashSet<Path>;
pub type HashSetUsize = HashSet<usize>;
pub type ModeAwareCacheResolvedModule = HashMap<ModeAwareCacheKey, ResolvedModule>;
pub type ModeAwareCacheResolvedTypeReferenceDirective =
    HashMap<ModeAwareCacheKey, ResolvedTypeReferenceDirective>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ModeAwareCacheKey {
    pub name: String,
    pub mode: ResolutionMode,
}

impl ModeAwareCacheKey {
    pub fn new(name: String, mode: ResolutionMode) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self { name, mode }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MappingErrorKind {
    #[default]
    Overlap,
    OutOfBounds,
    VerbatimMismatch,
    Kind,
    Feature,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MappingError {
    pub kind: MappingErrorKind,
    pub virtual_pos: usize,
    pub original_pos: usize,
}

#[derive(Debug, Default)]
pub struct ContentMapperError {
    pub project_error: Option<ProjectError>,
    pub initialize_error: Option<InitializeError>,
    pub transform_error: Option<TransformError>,
    pub mapping_error: Option<MappingError>,
    pub supplemental_file_collision: Option<SupplementalFileCollisionError>,
}

impl ContentMapperError {
    pub fn project_unavailable() -> Self { ::tsox_core::fntrace::enter("project_unavailable"); 
        Self::default()
    }

    pub fn as_project_error(&self) -> Option<&ProjectError> { ::tsox_core::fntrace::enter("as_project_error"); 
        self.project_error.as_ref()
    }

    pub fn as_initialize_error(&self) -> Option<&InitializeError> { ::tsox_core::fntrace::enter("as_initialize_error"); 
        self.initialize_error.as_ref()
    }

    pub fn as_transform_error(&self) -> Option<&TransformError> { ::tsox_core::fntrace::enter("as_transform_error"); 
        self.transform_error.as_ref()
    }

    pub fn as_mapping_error(&self) -> Option<&MappingError> { ::tsox_core::fntrace::enter("as_mapping_error"); 
        self.mapping_error.as_ref()
    }

    pub fn as_supplemental_file_collision_error(&self) -> Option<&SupplementalFileCollisionError> { ::tsox_core::fntrace::enter("as_supplemental_file_collision_error"); 
        self.supplemental_file_collision.as_ref()
    }

    pub fn is_initialize_transform_error(&self) -> bool { ::tsox_core::fntrace::enter("is_initialize_transform_error"); 
        self.initialize_error.is_some()
            || self
                .transform_error
                .as_ref()
                .is_some_and(|t| matches!(t.kind, TransformErrorKind::Initialize))
    }
}

pub struct ResolvedRef {
    pub file_name: String,
    pub increase_depth: bool,
    pub elide_on_depth: bool,
    pub include_reason: Arc<FileIncludeReason>,
    pub package_id: Option<PackageId>,
}

#[derive(Default)]
pub struct ParseTask {
    pub normalized_file_path: String,
    pub path: Path,
    pub file: Option<Arc<SourceFile>>,
    pub lib_file: Option<LibFile>,
    pub redirected_parse_task: Option<Arc<Mutex<ParseTask>>>,
    pub sub_tasks: Vec<Arc<Mutex<ParseTask>>>,
    pub loaded: bool,
    pub started_sub_tasks: bool,
    pub is_for_automatic_type_directive: bool,
    pub is_content_mapper_supplemental: bool,
    pub failed_lookup: bool,
    pub include_reason: Option<Arc<FileIncludeReason>>,
    pub package_id: Option<PackageId>,
    pub metadata: SourceFileMetaData,
    pub resolutions_in_file: ModeAwareCacheResolvedModule,
    pub resolutions_trace: Vec<DiagAndArgs>,
    pub type_resolutions_in_file: ModeAwareCacheResolvedTypeReferenceDirective,
    pub type_resolutions_trace: Vec<DiagAndArgs>,
    pub resolution_diagnostics: Vec<Arc<Diagnostic>>,
    pub processing_diagnostics: Vec<Arc<ProcessingDiagnostic>>,
    pub import_helpers_import_specifier: Option<Arc<Node>>,
    pub jsx_runtime_import_specifier: Option<JsxRuntimeImportSpecifier>,
    pub increase_depth: bool,
    pub elide_on_depth: bool,
    pub loaded_task: Option<Arc<Mutex<ParseTask>>>,
    pub all_include_reasons: Vec<Arc<FileIncludeReason>>,
}

impl ParseTask {
    pub fn add_sub_task(&mut self, sub_task: ResolvedRef, lib_file: Option<LibFile>) { ::tsox_core::fntrace::enter("add_sub_task"); 
        let normalized_file_path = normalize_path(&sub_task.file_name);
        self.sub_tasks.push(Arc::new(Mutex::new(ParseTask {
            normalized_file_path,
            lib_file,
            increase_depth: sub_task.increase_depth,
            elide_on_depth: sub_task.elide_on_depth,
            include_reason: Some(sub_task.include_reason),
            package_id: sub_task.package_id,
            ..Default::default()
        })));
    }

    pub fn redirect(&mut self, file_name: &str) { ::tsox_core::fntrace::enter("redirect"); 
        let lib_file = self.lib_file.clone();
        let redirected = Arc::new(Mutex::new(ParseTask {
            normalized_file_path: normalize_path(file_name),
            lib_file,
            include_reason: self.include_reason.clone(),
            ..Default::default()
        }));
        self.sub_tasks = vec![redirected.clone()];
        self.redirected_parse_task = Some(redirected);
    }

    pub fn load_automatic_type_directives(&mut self, loader: &mut FileLoader) { ::tsox_core::fntrace::enter("load_automatic_type_directives"); 
        let (to_parse, type_resolutions_in_file, type_resolutions_trace, p_diagnostics) =
            loader.resolve_automatic_type_directives(&self.normalized_file_path.clone());
        self.type_resolutions_in_file = type_resolutions_in_file;
        self.type_resolutions_trace = type_resolutions_trace;
        self.processing_diagnostics.extend(p_diagnostics);
        for type_resolution in to_parse {
            self.add_sub_task(type_resolution, None);
        }
    }

    pub fn load(&mut self, loader: &mut FileLoader) { ::tsox_core::fntrace::enter("load"); 
        self.loaded = true;
        if self.is_for_automatic_type_directive {
            self.load_automatic_type_directives(loader);
            return;
        }
        if self.failed_lookup {
            return;
        }
        let redirect = loader
            .project_reference_file_mapper
            .as_ref()
            .unwrap()
            .get_parse_file_redirect(&tsox_frontend::ast::mig::m3g_3::new_has_file_name(
                self.normalized_file_path.clone(),
                self.path.clone(),
            ));
        if !redirect.is_empty() {
            self.redirect(&redirect);
            return;
        }

        let compiler_options = loader.opts.config.compiler_options().clone();
        if !self.is_content_mapper_supplemental && has_extension(&self.normalized_file_path) {
            let allow_non_ts_extensions = compiler_options.allow_non_ts_extensions.is_true();
            if !allow_non_ts_extensions {
                let canonical_file_name = get_canonical_file_name(
                    &self.normalized_file_path,
                    loader.opts.host.fs().use_case_sensitive_file_names(),
                );
                if !loader.is_supported_extension(&canonical_file_name) {
                    let is_js = has_js_file_extension(&canonical_file_name);
                    let message = if is_js {
                        msg::FILE_0_IS_A_JAVASCRIPT_FILE_DID_YOU_MEAN_TO_ENABLE_THE_ALLOWJS_OPTION
                    } else {
                        msg::FILE_0_HAS_AN_UNSUPPORTED_EXTENSION_THE_ONLY_SUPPORTED_EXTENSIONS_ARE_1
                    };
                    let args = if is_js {
                        vec![self.normalized_file_path.clone()]
                    } else {
                        vec![
                            self.normalized_file_path.clone(),
                            format!(
                                "'{}'",
                                loader
                                    .supported_extensions
                                    .iter()
                                    .flatten()
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join("', '")
                            ),
                        ]
                    };
                    self.processing_diagnostics.push(Arc::new(ProcessingDiagnostic {
                        kind: ProcessingDiagnosticKind::ExplainingFileInclude,
                        data: ProcessingDiagnosticData::IncludeExplainingDiagnostic(Box::new(
                            IncludeExplainingDiagnostic {
                                file: Path::default(),
                                diagnostic_reason: std::ptr::null(),
                                diagnostic_reason_opt: self
                                    .include_reason
                                    .as_ref()
                                    .map(|reason| reason.clone_reason()),
                                message,
                                args,
                            },
                        )),
                    }));
                    return;
                }
            }
        }

        loader.total_file_count.fetch_add(1, Ordering::SeqCst);
        if self.lib_file.is_some() {
            loader.lib_file_count.fetch_add(1, Ordering::SeqCst);
            self.metadata = SourceFileMetaData {
                implied_node_format: ResolutionMode::CommonJS,
                ..Default::default()
            };
        } else {
            self.metadata = loader.load_source_file_meta_data(&self.normalized_file_path);
        }

        let mut file = self.file.clone();
        if file.is_none() {
            file = loader.parse_source_file(self);
        }
        let file = match file {
            Some(file) => file,
            None => return,
        };
        self.file = Some(file.clone());
        self.sub_tasks = Vec::with_capacity(
            file.referenced_files.len() + file.imports.len() + file.module_augmentations.len(),
        );

        if !compiler_options.no_resolve.is_true() && !loader.opts.skip_module_resolution {
            for (index, ref_) in file.referenced_files.iter().enumerate() {
                let (resolved_ref, processing_diagnostic) = loader.resolve_tripleslash_path_reference(
                    &ref_.file_name,
                    &file.file_name,
                    index,
                );
                if let Some(processing_diagnostic) = processing_diagnostic {
                    self.processing_diagnostics.push(processing_diagnostic);
                    continue;
                }
                self.add_sub_task(resolved_ref.unwrap(), None);
            }

            loader.resolve_type_reference_directives(self);
        }

        if !compiler_options.no_lib.is_true() && !loader.opts.skip_module_resolution {
            for (index, lib) in file.lib_reference_directives.iter().enumerate() {
                let include_reason = Arc::new(FileIncludeReason::new(
                    FileIncludeKind::LibReferenceDirective,
                    FileIncludeReasonData::ReferencedFile(Box::new(ReferencedFileData {
                        file: self.path.clone(),
                        index,
                        synthetic: None,
                    })),
                ));
                if let Some(name) = get_lib_file_name(&lib.file_name) {
                    let lib_file = loader.path_for_lib_file(&name);
                    self.add_sub_task(
                        ResolvedRef {
                            file_name: lib_file.path.clone(),
                            increase_depth: false,
                            elide_on_depth: false,
                            include_reason,
                            package_id: None,
                        },
                        Some(lib_file),
                    );
                } else {
                    self.processing_diagnostics.push(Arc::new(ProcessingDiagnostic {
                        kind: ProcessingDiagnosticKind::UnknownReference,
                        data: ProcessingDiagnosticData::FileIncludeReason(include_reason.clone_reason()),
                    }));
                }
            }
        }

        loader.resolve_imports_and_module_augmentations(self);
        for supplemental in file.supplemental_source_files.clone() {
            self.sub_tasks.push(Arc::new(Mutex::new(ParseTask {
                normalized_file_path: supplemental.file_name.clone(),
                file: Some(supplemental),
                is_content_mapper_supplemental: true,
                include_reason: Some(Arc::new(FileIncludeReason::new(
                    FileIncludeKind::ContentMapperSupplemental,
                    FileIncludeReasonData::CanonicalPath(self.path.clone()),
                ))),
                ..Default::default()
            })));
        }
    }
}

pub mod TokenFlags {
    pub const None: u32 = 0;
}

const NODE_FLAGS_JSDOC: NodeFlags = NodeFlags::HasJSDoc;

pub const SUPPORTED_TS_EXTENSIONS_WITH_JSON_FL: &[&str] = &[
    ".ts", ".tsx", ".d.ts", ".cts", ".d.cts", ".mts", ".d.mts", ".json",
];

pub struct ContentMapperSourceFileInfo {
    pub content_mapper: String,
    pub transform_identity: String,
    pub parse_options: SourceFileParseOptions,
    pub virtual_file_name: String,
    pub original_text: String,
    pub span_map: Vec<PositionMap>,
    pub diagnostic_directives: Vec<DiagnosticDirectives>,
    pub supplemental_source_files: Vec<Arc<SourceFile>>,
    pub canonical_source_file: Option<Arc<SourceFile>>,
}

impl Default for ContentMapperSourceFileInfo {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self {
            content_mapper: String::new(),
            transform_identity: String::new(),
            parse_options: SourceFileParseOptions::default(),
            virtual_file_name: String::new(),
            original_text: String::new(),
            span_map: Vec::new(),
            diagnostic_directives: Vec::new(),
            supplemental_source_files: Vec::new(),
            canonical_source_file: None,
        }
    }
}

pub struct HasFileName {
    file_name: String,
    path: Path,
}

impl HasFileName {
    pub fn new(file_name: String, path: Path) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self { file_name, path }
    }
}

impl MapperHasFileName for HasFileName {
    fn path(&self) -> String { ::tsox_core::fntrace::enter("path"); 
        self.path.0.clone()
    }
    fn file_name(&self) -> String { ::tsox_core::fntrace::enter("file_name"); 
        self.file_name.clone()
    }
}

fn mapper_key(mapper: &ContentMapper) -> String { ::tsox_core::fntrace::enter("mapper_key"); 
    mapper.definition.package.clone()
}

fn content_mapper_assembled(mapper: &ContentMapper) -> Mapper { ::tsox_core::fntrace::enter("content_mapper_assembled"); 
    // Go GetContentMapperForFileName 返回装配了 Manifest/PackageDirectory 的全量 *contentmapper.Mapper。
    // m5h_3 描述符自 r54-k03 起携带 manifest/package_directory/contribution_id,原样透传装配。
    Mapper {
        definition: super::m3l_cm::Definition {
            package: mapper.definition.package.clone(),
            extensions: mapper.definition.extensions.clone(),
            options: mapper.definition.options.clone(),
        },
        manifest: super::m3l_cm::Manifest {
            name: mapper.manifest.name.clone(),
            version: mapper.manifest.version.clone(),
            exec: mapper.manifest.exec.clone(),
            compiler_options: mapper.manifest.compiler_options.clone(),
            dynamic_config: mapper.manifest.dynamic_config,
        },
        package_directory: mapper.package_directory.clone(),
        contribution_id: mapper.contribution_id.clone(),
    }
}

fn to_host_parse_options(
    opts: &SourceFileParseOptions,
) -> tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions { ::tsox_core::fntrace::enter("to_host_parse_options"); 
    tsox_frontend::ast::mig::m3b_2::SourceFileParseOptions {
        file_name: opts.file_name.clone(),
        path: opts.path.clone(),
    }
}

impl From<crate::mig::m4v_3::ContentMapperError> for ContentMapperError {
    fn from(err: crate::mig::m4v_3::ContentMapperError) -> Self { ::tsox_core::fntrace::enter("from"); 
        if err.0 == "content mapper project is unavailable" {
            return ContentMapperError::project_unavailable();
        }
        ContentMapperError::default()
    }
}

fn transform_identity_hex(transform_identity: u128) -> String { ::tsox_core::fntrace::enter("transform_identity_hex"); 
    transform_identity
        .to_be_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn new_import_declaration_node(module_specifier: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_import_declaration_node"); 
    Arc::new(Node::new(
        SyntaxKind::ImportDeclaration,
        NodeData::ImportDeclaration(ImportDeclarationData {
            modifiers: None,
            import_clause: None,
            module_specifier,
            attributes: None,
        }),
    ))
}

struct LoaderResolutionHost {
    fs: Arc<dyn tsox_tsoptions::vfs::FS>,
    current_directory: String,
}

impl tsox_tsoptions::module::ResolutionHost for LoaderResolutionHost {
    fn fs(&self) -> &dyn tsox_tsoptions::vfs::FS { ::tsox_core::fntrace::enter("fs"); 
        self.fs.as_ref()
    }

    fn get_current_directory(&self) -> &str { ::tsox_core::fntrace::enter("get_current_directory"); 
        &self.current_directory
    }
}

fn loader_host(opts: &ProgramOptions) -> Arc<dyn CompilerHost> { ::tsox_core::fntrace::enter("loader_host"); 
    opts.host.clone()
}

fn new_resolver(
    host: Arc<dyn tsox_tsoptions::module::ResolutionHost + Send + Sync>,
    compiler_options: &CompilerOptions,
    typings_location: String,
    project_name: &str,
    _content_mapper_extensions: &[String],
) -> Resolver { ::tsox_core::fntrace::enter("new_resolver"); 
    Resolver::new(
        host,
        Arc::new(compiler_options.clone()),
        typings_location,
        project_name.to_string(),
    )
}

fn get_compiler_options_with_redirect(
    base: &CompilerOptions,
    redirect: Option<&Arc<ParsedCommandLine>>,
) -> CompilerOptions { ::tsox_core::fntrace::enter("get_compiler_options_with_redirect"); 
    redirect
        .map(|command_line| command_line.compiler_options().clone())
        .unwrap_or_else(|| base.clone())
}

fn parse_source_file(opts: &SourceFileParseOptions, text: &str, _script_kind: ScriptKind) -> Arc<SourceFile> { ::tsox_core::fntrace::enter("parse_source_file"); 
    Arc::new(tsox_frontend::parser::Parser::parse_source_file_text(
        &opts.file_name,
        text.to_string(),
    ))
}

fn new_diagnostic(
    file: Option<Arc<SourceFile>>,
    range: TextRange,
    message: Message,
    args: Vec<String>,
) -> Diagnostic { ::tsox_core::fntrace::enter("new_diagnostic"); 
    Diagnostic::new(file, range, message, args)
}

fn new_text_range(start: usize, end: usize) -> TextRange { ::tsox_core::fntrace::enter("new_text_range"); 
    TextRange::new(start, end)
}

fn add_message_chain(mut diagnostic: Diagnostic, message_chain: Diagnostic) -> Diagnostic { ::tsox_core::fntrace::enter("add_message_chain"); 
    diagnostic.message_chain.push(message_chain);
    diagnostic
}

fn is_external_module(file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("is_external_module"); 
    file.external_module_indicator.is_some()
}

fn libs() -> Vec<&'static str> { ::tsox_core::fntrace::enter("libs"); 
    LIB_MAP.iter().map(|(name, _)| *name).collect()
}


pub struct LibResolution {
    pub library_name: String,
    pub resolution: Option<ResolvedModule>,
    pub trace: Vec<DiagAndArgs>,
}

#[derive(Clone)]
pub struct LibFile {
    pub name: String,
    pub path: String,
    pub replaced: bool,
}

pub struct SourceFileFromReferenceDiagnostic {
    pub message: tsox_core::diagnostics::Message,
    pub args: Vec<String>,
}

pub struct FileLoader {
    pub opts: ProgramOptions,
    pub resolver: Option<Arc<Resolver>>,
    pub default_library_path: String,
    pub compare_paths_options: ComparePathsOptions,
    pub supported_extensions: Vec<Vec<String>>,
    pub supported_extensions_with_json_if_resolve_json_module: Vec<Vec<String>>,
    pub content_mapper_extensions: Vec<String>,

    pub files_parser: Option<Box<FilesParser>>,
    pub root_tasks: Vec<Arc<Mutex<ParseTask>>>,

    pub total_file_count: AtomicI32,
    pub lib_file_count: AtomicI32,

    pub factory_mu: Mutex<()>,
    pub factory: NodeFactory,

    pub project_reference_file_mapper: Option<Arc<ProjectReferenceFileMapper>>,
    pub dts_directories: HashSetTspath,

    pub path_for_lib_file_cache: HashMap<String, LibFile>,
    pub path_for_lib_file_resolutions: HashMap<Path, LibResolution>,

    pub content_mapper_mu: Mutex<ContentMapperBookkeeping>,
}

#[derive(Default)]
pub struct ContentMapperBookkeeping {
    pub content_mapper_failures: HashMap<String, i32>,
    pub content_mapper_init_failed: HashSet<String>,
    pub content_mapper_diagnostics: Vec<Diagnostic>,
}

pub struct RedirectsFile {
    pub index: usize,
    pub file_name: String,
    pub path: Path,
    pub target: Path,
}

impl RedirectsFile {
    pub fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file_name
    }

    pub fn path(&self) -> &Path { ::tsox_core::fntrace::enter("path"); 
        &self.path
    }
}

pub struct ProcessedFiles {
    pub resolver: Option<Arc<Resolver>>,
    pub files: Vec<Arc<SourceFile>>,
    pub duplicate_source_files: Vec<DuplicateSourceFile>,
    pub files_by_path: HashMap<Path, Arc<SourceFile>>,
    pub project_reference_file_mapper: Option<Arc<ProjectReferenceFileMapper>>,
    pub missing_files: Vec<String>,
    pub resolved_modules: HashMap<Path, ModeAwareCacheResolvedModule>,
    pub type_resolutions_in_file: HashMap<Path, ModeAwareCacheResolvedTypeReferenceDirective>,
    pub source_file_meta_datas: HashMap<Path, SourceFileMetaData>,
    pub jsx_runtime_import_specifiers: HashMap<Path, JsxRuntimeImportSpecifier>,
    pub import_helpers_import_specifiers: HashMap<Path, Arc<Node>>,
    pub lib_files: HashMap<Path, LibFile>,
    pub source_files_found_searching_node_modules: HashSetTspath,
    pub include_processor: Option<Box<IncludeProcessor>>,
    pub output_file_to_project_reference_source: Option<HashMap<Path, String>>,
    pub redirect_targets_map: Option<HashMap<Path, Vec<String>>>,
    pub redirect_files_by_path: Option<HashMap<Path, RedirectsFile>>,
    pub content_mapper_diagnostics: Vec<Diagnostic>,
    pub finished_processing: bool,
}

pub struct JsxRuntimeImportSpecifier {
    pub module_reference: String,
    pub specifier: Arc<Node>,
}

pub fn process_all_program_files(opts: ProgramOptions, single_threaded: bool) -> ProcessedFiles { ::tsox_core::fntrace::enter("process_all_program_files"); 
    let compiler_options = opts.config.compiler_options().clone();
    let root_files = opts.config.file_names().to_vec();
    let supported_extensions = get_supported_extensions(
        &compiler_options,
        &opts.config.content_mapper_extensions().iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
    );
    let supported_extensions_with_json_if_resolve_json_module =
        get_supported_extensions_with_json_if_resolve_json_module(&compiler_options, &supported_extensions);
    let max_node_module_js_depth = compiler_options.max_node_module_js_depth.unwrap_or(0);
    let content_mapper_extensions = opts.config.content_mapper_extensions();
    let typings_location = opts.typings_location.clone();
    let project_name = opts.project_name.clone();
    let default_library_path = get_normalized_absolute_path(
        opts.host.default_library_path(),
        opts.host.current_directory(),
    );
    let compare_paths_options = ComparePathsOptions {
        use_case_sensitive_file_names: opts.host.fs().use_case_sensitive_file_names(),
        current_directory: opts.host.current_directory().to_string(),
    };
    let mut loader = FileLoader {
        opts,
        resolver: None,
        default_library_path,
        compare_paths_options,
        files_parser: Some(Box::new(FilesParser {
            wg: new_work_group(single_threaded),
            task_data_by_path: Mutex::new(HashMap::new()),
            max_depth: max_node_module_js_depth,
        })),
        root_tasks: Vec::with_capacity(root_files.len() + compiler_options.lib.len()),
        supported_extensions,
        supported_extensions_with_json_if_resolve_json_module,
        content_mapper_extensions,
        total_file_count: AtomicI32::new(0),
        lib_file_count: AtomicI32::new(0),
        factory_mu: Mutex::new(()),
        factory: NodeFactory::new(),
        project_reference_file_mapper: None,
        dts_directories: HashSet::new(),
        path_for_lib_file_cache: HashMap::new(),
        path_for_lib_file_resolutions: HashMap::new(),
        content_mapper_mu: Mutex::new(ContentMapperBookkeeping::default()),
    };
    loader.add_project_reference_tasks(single_threaded);
    loader.resolver = Some(Arc::new(new_resolver(
        Arc::new(LoaderResolutionHost {
            fs: loader.opts.host.fs_arc(),
            current_directory: loader.opts.host.current_directory().to_string(),
        }),
        &compiler_options,
        typings_location,
        &project_name,
        &loader.opts.config.content_mapper_extensions(),
    )));
    for (index, root_file) in root_files.iter().enumerate() {
        loader.add_root_file_task(
            root_file,
            None,
            Arc::new(FileIncludeReason::new(FileIncludeKind::RootFile, FileIncludeReasonData::Index(index))),
        );
    }
    if !root_files.is_empty() && compiler_options.no_lib.is_false_or_unknown() {
        if compiler_options.lib.is_empty() {
            let name = get_default_lib_file_name(&compiler_options);
            let lib_file = loader.path_for_lib_file(&name);
            let lib_file_path = lib_file.path.clone();
            loader.add_root_task(&lib_file_path, Some(lib_file), FileIncludeReason::new(FileIncludeKind::LibFile, FileIncludeReasonData::None));
        } else {
            for (index, lib) in compiler_options.lib.iter().enumerate() {
                if let Some(name) = get_lib_file_name(lib) {
                    let lib_file = loader.path_for_lib_file(&name);
                    let lib_file_path = lib_file.path.clone();
                    loader.add_root_task(&lib_file_path, Some(lib_file), FileIncludeReason::new(FileIncludeKind::LibFile, FileIncludeReasonData::Index(index)));
                }
            }
        }
    }
    if !root_files.is_empty() && !loader.opts.skip_module_resolution {
        loader.add_automatic_type_directive_tasks();
    }
    let root_tasks = loader.root_tasks.clone();
    let mut files_parser = loader.files_parser.take().unwrap();
    files_parser.parse(&mut loader, &root_tasks);
    loader.files_parser = Some(files_parser);

    loader.project_reference_file_mapper = None;

    let mut files_parser = loader.files_parser.take().unwrap();
    let result = files_parser.get_processed_files(&mut loader);
    loader.files_parser = Some(files_parser);
    result
}

impl FileLoader {
    fn resolution_host(&self) -> LoaderResolutionHost { ::tsox_core::fntrace::enter("resolution_host"); 
        LoaderResolutionHost {
            fs: self.opts.host.fs_arc(),
            current_directory: self.opts.host.current_directory().to_string(),
        }
    }

    pub fn to_path(&self, file: &str) -> Path { ::tsox_core::fntrace::enter("to_path"); 
        to_path(
            file,
            self.opts.host.current_directory(),
            self.opts.host.fs().use_case_sensitive_file_names(),
        )
    }

    pub fn add_root_task(&mut self, file_name: &str, lib_file: Option<LibFile>, include_reason: FileIncludeReason) { ::tsox_core::fntrace::enter("add_root_task"); 
        let abs_path = get_normalized_absolute_path(file_name, self.opts.host.current_directory());
        if self.opts.config.compiler_options().allow_non_ts_extensions.is_true() || has_extension(&abs_path) {
            self.root_tasks.push(Arc::new(Mutex::new(ParseTask {
                normalized_file_path: abs_path,
                lib_file,
                include_reason: Some(Arc::new(include_reason)),
                ..Default::default()
            })));
        }
    }

    pub fn add_root_file_task(&mut self, file_name: &str, lib_file: Option<LibFile>, include_reason: Arc<FileIncludeReason>) { ::tsox_core::fntrace::enter("add_root_file_task"); 
        let curr_dir = self.opts.host.current_directory().to_string();
        let abs_path = get_normalized_absolute_path(file_name, &curr_dir);
        let mut containing_file = curr_dir.clone();
        if let Some(config_file) = self.opts.config.config_file.as_ref() {
            containing_file = get_normalized_absolute_path(&config_file.source_file.file_name(), &curr_dir);
        }
        let (resolved_file, diagnostic) =
            self.get_source_file_from_reference(&abs_path, file_name, &containing_file, &include_reason);
        let mut root_task = ParseTask {
            normalized_file_path: resolved_file,
            lib_file,
            include_reason: Some(include_reason.clone()),
            ..Default::default()
        };
        if let Some(diagnostic) = diagnostic {
            root_task.normalized_file_path = abs_path;
            root_task.failed_lookup = true;
            root_task.processing_diagnostics = vec![Arc::new(ProcessingDiagnostic {
                kind: ProcessingDiagnosticKind::ExplainingFileInclude,
                data: ProcessingDiagnosticData::IncludeExplainingDiagnostic(Box::new(IncludeExplainingDiagnostic {
                    file: Path::default(),
                    diagnostic_reason: std::ptr::null(),
                    diagnostic_reason_opt: Some((*include_reason).clone_reason()),
                    message: diagnostic.message,
                    args: diagnostic.args,
                })),
            })];
        }
        self.root_tasks.push(Arc::new(Mutex::new(root_task)));
    }

    pub fn add_automatic_type_directive_tasks(&mut self) { ::tsox_core::fntrace::enter("add_automatic_type_directive_tasks"); 
        let compiler_options = self.opts.config.compiler_options();
        let containing_directory = if !compiler_options.config_file_path.is_empty() {
            get_directory_path(&compiler_options.config_file_path)
        } else {
            self.opts.host.current_directory().to_string()
        };
        let containing_file_name = combine_paths(&containing_directory, &[INFERRED_TYPES_CONTAINING_FILE]);
        self.root_tasks.push(Arc::new(Mutex::new(ParseTask {
            normalized_file_path: containing_file_name,
            is_for_automatic_type_directive: true,
            ..Default::default()
        })));
    }

    pub fn resolve_automatic_type_directives(
        &self,
        containing_file_name: &str,
    ) -> (
        Vec<ResolvedRef>,
        ModeAwareCacheResolvedTypeReferenceDirective,
        Vec<DiagAndArgs>,
        Vec<Arc<ProcessingDiagnostic>>,
    ) { ::tsox_core::fntrace::enter("resolve_automatic_type_directives"); 
        let mut to_parse = Vec::new();
        let mut type_resolutions_in_file = ModeAwareCacheResolvedTypeReferenceDirective::default();
        let mut type_resolutions_trace = Vec::new();
        let mut p_diagnostics = Vec::new();
        let automatic_type_directive_names =
            get_automatic_type_directive_names(self.opts.config.compiler_options(), &self.resolution_host());
        for name in automatic_type_directive_names {
            let resolution_mode = ResolutionMode::None;
            let (resolved, trace) = self.resolver.as_ref().unwrap().resolve_type_reference_directive(
                &name,
                &containing_file_name,
                resolution_mode,
                None,
            );
            let resolved = resolved.unwrap_or_default();
            type_resolutions_in_file.insert(ModeAwareCacheKey::new(name.clone(), resolution_mode), resolved.clone());
            type_resolutions_trace.extend(trace);
            if resolved.is_resolved() {
                to_parse.push(ResolvedRef {
                    file_name: resolved.resolved_file_name.clone(),
                    increase_depth: resolved.is_external_library_import,
                    elide_on_depth: false,
                    include_reason: Arc::new(FileIncludeReason::new(
                        FileIncludeKind::AutomaticTypeDirectiveFile,
                        FileIncludeReasonData::AutomaticTypeDirectiveFile(Box::new(
                            AutomaticTypeDirectiveFileData {
                                type_reference: name.clone(),
                                package_id: resolved.package_id.clone(),
                            },
                        )),
                    )),
                    package_id: resolved.package_id.clone(),
                });
            } else {
                p_diagnostics.push(Arc::new(ProcessingDiagnostic {
                    kind: ProcessingDiagnosticKind::ExplainingFileInclude,
                    data: ProcessingDiagnosticData::IncludeExplainingDiagnostic(Box::new(IncludeExplainingDiagnostic {
                        file: Path::default(),
                        diagnostic_reason: std::ptr::null(),
                        diagnostic_reason_opt: Some(Box::new(FileIncludeReason::new(
                            FileIncludeKind::AutomaticTypeDirectiveFile,
                            FileIncludeReasonData::AutomaticTypeDirectiveFile(Box::new(
                                AutomaticTypeDirectiveFileData {
                                    type_reference: name.clone(),
                                    package_id: None,
                                },
                            )),
                        ))),
                        message: msg::CANNOT_FIND_TYPE_DEFINITION_FILE_FOR_0,
                        args: vec![name.clone()],
                    })),
                }));
            }
        }
        (to_parse, type_resolutions_in_file, type_resolutions_trace, p_diagnostics)
    }

    pub fn add_project_reference_tasks(&mut self, single_threaded: bool) { ::tsox_core::fntrace::enter("add_project_reference_tasks"); 
        self.project_reference_file_mapper = Some(Arc::new(ProjectReferenceFileMapper {
            opts: self.opts.clone(),
            host: Some(Box::new(ResolutionHostAdapter::new(self.opts.host.as_ref()))),
            loader: None,
            config_to_project_reference: HashMap::new(),
            references_in_config_file: HashMap::new(),
            source_to_project_reference: HashMap::new(),
            output_dts_to_project_reference: HashMap::new(),
            realpath_dts_to_source: SyncMap::new(),
        }));
        let project_references = self.opts.config.resolved_project_reference_paths();
        if project_references.is_empty() {
            return;
        }
        let reference_loader = Arc::new(Mutex::new(ProjectReferenceLoader {
            opts: self.opts.clone(),
            project_reference_file_mapper: self.project_reference_file_mapper.clone().unwrap(),
            dts_directories: Set::new(),
        }));
        let mut parser = ProjectReferenceParser {
            loader: Some(reference_loader.clone()),
            wg: new_work_group(single_threaded),
            tasks_by_file_name: SyncMap::new(),
        };
        let root_tasks = create_project_reference_parse_tasks(&project_references);
        parser.parse(root_tasks, reference_loader);
    }

    pub fn sort_libs(&self, lib_files: &mut [Arc<SourceFile>]) { ::tsox_core::fntrace::enter("sort_libs"); 
        lib_files.sort_by_key(|f| self.get_default_lib_file_priority(f));
    }

    pub fn get_default_lib_file_priority(&self, a: &Arc<SourceFile>) -> usize { ::tsox_core::fntrace::enter("get_default_lib_file_priority"); 
        let default_library_path = remove_trailing_directory_separator(&self.default_library_path);
        let a_file_name = a.file_name.clone();
        if a_file_name.starts_with(&default_library_path)
            && a_file_name.len() > default_library_path.len()
            && a_file_name.as_bytes()[default_library_path.len()] == b'/'
        {
            let basename = &a_file_name[a_file_name.rfind('/').map(|i| i + 1).unwrap_or(0)..];
            if basename == "lib.d.ts" || basename == "lib.es6.d.ts" {
                return 0;
            }
            let name = basename.strip_prefix("lib.").unwrap_or(basename);
            let name = name.strip_suffix(".d.ts").unwrap_or(name);
            if let Some(index) = libs().iter().position(|l| *l == name) {
                return index + 1;
            }
        }
        libs().len() + 2
    }

    pub fn load_source_file_meta_data(&self, file_name: &str) -> SourceFileMetaData { ::tsox_core::fntrace::enter("load_source_file_meta_data"); 
        if self.opts.skip_module_resolution {
            return SourceFileMetaData {
                implied_node_format: get_implied_node_format_for_file(file_name, ""),
                ..Default::default()
            };
        }
        let package_json_scope = self
            .resolver
            .as_ref()
            .unwrap()
            .get_package_scope_for_path(&get_directory_path(file_name))
            .unwrap_or_default();
        let module_resolution_kind = self.opts.config.compiler_options().get_module_resolution_kind();

        let mut package_json_type = String::new();
        let mut package_json_directory = String::new();
        if package_json_scope.exists() {
            package_json_directory = package_json_scope.package_directory.clone();
            if let Some(value) = package_json_scope
                .contents
                .as_ref()
                .and_then(|f| f.header_fields.r#type.get_value())
            {
                if !file_extension_is_one_of(
                    file_name,
                    &[EXTENSION_MTS, EXTENSION_CTS, EXTENSION_MJS, EXTENSION_CJS],
                ) && ((module_resolution_kind as i32) >= (ModuleResolutionKind::Node16 as i32)
                    && (module_resolution_kind as i32) <= (ModuleResolutionKind::NodeNext as i32)
                    || file_name.contains("/node_modules/"))
                {
                    package_json_type = value.clone();
                }
            }
        }
        let implied_node_format = get_implied_node_format_for_file(file_name, &package_json_type);
        SourceFileMetaData {
            package_json_type,
            package_json_directory,
            implied_node_format,
        }
    }

    pub fn parse_source_file(&self, t: &ParseTask) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("parse_source_file"); 
        let path = self.to_path(&t.normalized_file_path);
        let options = self
            .project_reference_file_mapper
            .as_ref()
            .unwrap()
            .get_compiler_options_for_file(&HasFileName::new(
                t.normalized_file_path.clone(),
                t.path.clone(),
            ));
        let parse_options = SourceFileParseOptions {
            file_name: t.normalized_file_path.clone(),
            path: path.0.clone(),
            external_module_indicator_options: get_external_module_indicator_options(
                &t.normalized_file_path,
                &options,
                &t.metadata,
            ),
        };
        if file_extension_is_one_of(
            &t.normalized_file_path,
            &self.content_mapper_extensions.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        ) {
            return self.parse_content_mapped_file(&parse_options);
        }
        self.opts.host.get_source_file(&to_host_parse_options(&parse_options))
    }

    pub fn parse_content_mapped_file(&self, opts: &SourceFileParseOptions) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("parse_content_mapped_file"); 
        let Some(mapper) = self.opts.config.get_content_mapper_for_file_name(&opts.file_name) else {
            return Some(self.empty_content_mapped_file(opts, "", ""));
        };
        let label = mapper.definition.package.clone();
        let mapper_identity = mapper_key(mapper);
        let transform_identity = self.get_content_mapper_transform_identity(mapper);
        if self.content_mapper_unavailable(mapper) {
            return Some(self.empty_content_mapped_file(opts, &mapper_identity, &transform_identity));
        }
        // Go: p.opts.Host.GetContentMappedSourceFiles(opts, mapper)。
        // host 形参要求 m3b_2::SourceFileParseOptions 与全量 m3l_cm::Mapper;
        // tsoptions 侧 ContentMapper 仍为 definition 描述符(同 r52-k05 交接 2 根因),
        // mapper→全量 Mapper 装配落地后收口(交接 3),当前走空文件路径。
        let _ = mapper;
        Some(self.empty_content_mapped_file(opts, &mapper_identity, &transform_identity))
    }

    pub fn get_content_mapper_transform_identity(&self, mapper: &ContentMapper) -> String { ::tsox_core::fntrace::enter("get_content_mapper_transform_identity"); 
        let full_mapper = content_mapper_assembled(mapper);
        if let Some(project) = self.opts.host.content_mapper_project() {
            if let Ok(identity) = project.identity(&full_mapper) {
                return identity;
            }
        }
        transform_identity_hex(full_mapper.transform_identity(self.opts.config.compiler_options()))
    }

    pub fn empty_content_mapped_file(&self, opts: &SourceFileParseOptions, mapper_identity: &str, transform_identity: &str) -> Arc<SourceFile> { ::tsox_core::fntrace::enter("empty_content_mapped_file"); 
        let content = self.opts.host.fs().read_file(&opts.file_name);
        let source_file = parse_source_file(opts, "", ScriptKind::Ts);
        let info = ContentMapperSourceFileInfo {
            content_mapper: mapper_identity.to_string(),
            transform_identity: transform_identity.to_string(),
            parse_options: opts.clone(),
            virtual_file_name: format!("{}{}", opts.file_name, EXTENSION_TS),
            original_text: content.unwrap_or_default(),
            ..Default::default()
        };
        // Go: sourceFile.SetContentMapperInfo(info)。SourceFile 尚无对应字段
        // (tsox-frontend 建模缺失,同 span_map/content_mapper 挂起先例),见交接 4。
        let _ = info;
        source_file
    }

    pub fn content_mapper_unavailable(&self, mapper: &ContentMapper) -> bool { ::tsox_core::fntrace::enter("content_mapper_unavailable"); 
        let bookkeeping = self.content_mapper_mu.lock().unwrap();
        bookkeeping.content_mapper_init_failed.contains(&mapper_key(mapper))
            || bookkeeping
                .content_mapper_failures
                .get(&mapper_key(mapper))
                .copied()
                .unwrap_or(0)
                >= MAX_CONTENT_MAPPER_FAILURES
    }

    pub fn record_content_mapper_initialization_failure(&self, mapper: &ContentMapper, label: &str, err: &ContentMapperError) { ::tsox_core::fntrace::enter("record_content_mapper_initialization_failure"); 
        let mut bookkeeping = self.content_mapper_mu.lock().unwrap();
        if bookkeeping.content_mapper_init_failed.contains(&mapper_key(mapper)) {
            return;
        }
        bookkeeping.content_mapper_init_failed.insert(mapper_key(mapper));
        bookkeeping
            .content_mapper_diagnostics
            .push(content_mapper_initialization_diagnostic(label, err));
    }

    pub fn record_content_mapper_failure(&self, mapper: &ContentMapper, label: &str) -> bool { ::tsox_core::fntrace::enter("record_content_mapper_failure"); 
        let mut bookkeeping = self.content_mapper_mu.lock().unwrap();
        let key = mapper_key(mapper);
        if bookkeeping.content_mapper_failures.get(&key).copied().unwrap_or(0) >= MAX_CONTENT_MAPPER_FAILURES {
            return false;
        }
        let failure_count = bookkeeping.content_mapper_failures.entry(key).or_insert(0);
        *failure_count += 1;
        if *failure_count >= MAX_CONTENT_MAPPER_FAILURES {
            bookkeeping.content_mapper_diagnostics.push(new_compiler_diagnostic(
                THE_CONTENT_MAPPER_0_FAILED_1_TIMES_AND_WILL_NOT_BE_USED,
                vec![label.to_string(), MAX_CONTENT_MAPPER_FAILURES.to_string()],
            ));
        }
        true
    }

    pub fn is_supported_extension(&self, canonical_file_name: &str) -> bool { ::tsox_core::fntrace::enter("is_supported_extension"); 
        for group in &self.supported_extensions_with_json_if_resolve_json_module {
            if file_extension_is_one_of(
                canonical_file_name,
                &group.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            ) {
                return true;
            }
        }
        false
    }

    pub fn get_source_file_from_reference(
        &self,
        file_name: &str,
        reference_text: &str,
        containing_file: &str,
        include_reason: &FileIncludeReason,
    ) -> (String, Option<SourceFileFromReferenceDiagnostic>) { ::tsox_core::fntrace::enter("get_source_file_from_reference"); 
        let options = self.opts.config.compiler_options();
        let allow_non_ts_extensions = options.allow_non_ts_extensions.is_true();
        let diagnostic_file_name = normalize_slashes(reference_text);

        if has_extension(file_name) {
            let canonical_file_name = get_canonical_file_name(file_name, self.opts.host.fs().use_case_sensitive_file_names());
            if !allow_non_ts_extensions && !self.is_supported_extension(&canonical_file_name) {
                if has_js_file_extension(&canonical_file_name) {
                    return (
                        String::new(),
                        Some(SourceFileFromReferenceDiagnostic {
                            message: msg::FILE_0_IS_A_JAVASCRIPT_FILE_DID_YOU_MEAN_TO_ENABLE_THE_ALLOWJS_OPTION,
                            args: vec![diagnostic_file_name],
                        }),
                    );
                }
                return (
                    String::new(),
                    Some(SourceFileFromReferenceDiagnostic {
                        message: msg::FILE_0_HAS_AN_UNSUPPORTED_EXTENSION_THE_ONLY_SUPPORTED_EXTENSIONS_ARE_1,
                        args: vec![
                            diagnostic_file_name,
                            format!("'{}'", self.supported_extensions.iter().flatten().cloned().collect::<Vec<_>>().join("', '")),
                        ],
                    }),
                );
            }
            if !self.opts.host.fs().file_exists(file_name) {
                return (
                    String::new(),
                    Some(SourceFileFromReferenceDiagnostic {
                        message: msg::FILE_0_NOT_FOUND,
                        args: vec![diagnostic_file_name],
                    }),
                );
            }
            if include_reason.is_referenced_file()
                && get_canonical_file_name(containing_file, self.opts.host.fs().use_case_sensitive_file_names())
                    == canonical_file_name
            {
                return (
                    String::new(),
                    Some(SourceFileFromReferenceDiagnostic {
                        message: msg::A_FILE_CANNOT_HAVE_A_REFERENCE_TO_ITSELF,
                        args: Vec::new(),
                    }),
                );
            }
            return (file_name.to_string(), None);
        }

        if allow_non_ts_extensions && self.opts.host.fs().file_exists(file_name) {
            return (file_name.to_string(), None);
        }
        if allow_non_ts_extensions {
            return (
                String::new(),
                Some(SourceFileFromReferenceDiagnostic {
                    message: msg::FILE_0_NOT_FOUND,
                    args: vec![diagnostic_file_name],
                }),
            );
        }
        for ext in &self.supported_extensions[0] {
            let candidate = format!("{}{}", file_name, ext);
            if self.opts.host.fs().file_exists(&candidate) {
                return (candidate, None);
            }
        }
        (
            String::new(),
            Some(SourceFileFromReferenceDiagnostic {
                message: msg::COULD_NOT_RESOLVE_THE_PATH_0_WITH_THE_EXTENSIONS_COLON_1,
                args: vec![
                    diagnostic_file_name,
                    format!("'{}'", self.supported_extensions.iter().flatten().cloned().collect::<Vec<_>>().join("', '")),
                ],
            }),
        )
    }

    pub fn resolve_tripleslash_path_reference(
        &self,
        module_name: &str,
        containing_file: &str,
        index: usize,
    ) -> (Option<ResolvedRef>, Option<Arc<ProcessingDiagnostic>>) { ::tsox_core::fntrace::enter("resolve_tripleslash_path_reference"); 
        let base_path = get_directory_path(containing_file);
        let referenced_file_name = if !is_rooted_disk_path(module_name) {
            combine_paths(&base_path, &[module_name])
        } else {
            module_name.to_string()
        };
        let normalized_file_name = normalize_path(&referenced_file_name);
        let include_reason = Arc::new(FileIncludeReason::new(
            FileIncludeKind::ReferenceFile,
            FileIncludeReasonData::ReferencedFile(Box::new(ReferencedFileData {
                file: self.to_path(containing_file),
                index,
                synthetic: None,
            })),
        ));

        let (resolved_file_name, diagnostic) = self.get_source_file_from_reference(
            &normalized_file_name,
            module_name,
            containing_file,
            &include_reason,
        );
        if let Some(diagnostic) = diagnostic {
            return (
                None,
                Some(Arc::new(ProcessingDiagnostic {
                    kind: ProcessingDiagnosticKind::ExplainingFileInclude,
                    data: ProcessingDiagnosticData::IncludeExplainingDiagnostic(Box::new(IncludeExplainingDiagnostic {
                        file: Path::default(),
                        diagnostic_reason: std::ptr::null(),
                        diagnostic_reason_opt: Some((*include_reason).clone_reason()),
                        message: diagnostic.message,
                        args: diagnostic.args,
                    })),
                })),
            );
        }
        (
            Some(ResolvedRef {
                file_name: resolved_file_name,
                increase_depth: false,
                elide_on_depth: false,
                include_reason,
                package_id: None,
            }),
            None,
        )
    }

    pub fn resolve_type_reference_directives(&self, t: &mut ParseTask) { ::tsox_core::fntrace::enter("resolve_type_reference_directives"); 
        let file = t.file.as_ref().unwrap().clone();
        if file.type_reference_directives.is_empty() {
            return;
        }
        let meta = t.metadata.clone();

        let mut type_resolutions_in_file = ModeAwareCacheResolvedTypeReferenceDirective::default();
        let mut type_resolutions_trace = Vec::new();
        let file_for_mapper =
            HasFileName::new(file.file_name.clone(), self.to_path(&file.file_name));
        for (index, ref_) in file.type_reference_directives.iter().enumerate() {
            let (redirect, file_name) = self
                .project_reference_file_mapper
                .as_ref()
                .unwrap()
                .get_redirect_for_resolution(&file_for_mapper);
            let resolution_mode = get_mode_for_type_reference_directive_in_file(
                ref_,
                &file,
                &meta,
                &get_compiler_options_with_redirect(self.opts.config.compiler_options(), redirect.as_ref()),
            );
            let (resolved, trace) = self.resolver.as_ref().unwrap().resolve_type_reference_directive(
                &ref_.file_name,
                &file_name,
                resolution_mode,
                redirect.as_ref().map(|redirect| redirect.compiler_options.config_file_path.clone()).as_deref(),
            );
            let resolved = resolved.unwrap_or_default();
            type_resolutions_in_file.insert(ModeAwareCacheKey::new(ref_.file_name.clone(), resolution_mode), resolved.clone());
            let include_reason = Arc::new(FileIncludeReason::new(
                FileIncludeKind::TypeReferenceDirective,
                FileIncludeReasonData::ReferencedFile(Box::new(ReferencedFileData {
                    file: t.path.clone(),
                    index,
                    synthetic: None,
                })),
            ));
            type_resolutions_trace.extend(trace);

            if resolved.is_resolved() {
                t.add_sub_task(
                    ResolvedRef {
                        file_name: resolved.resolved_file_name.clone(),
                        increase_depth: resolved.is_external_library_import,
                        elide_on_depth: false,
                        include_reason,
                        package_id: resolved.package_id.clone(),
                    },
                    None,
                );
            } else {
                t.processing_diagnostics.push(Arc::new(ProcessingDiagnostic {
                    kind: ProcessingDiagnosticKind::UnknownReference,
                    data: ProcessingDiagnosticData::FileIncludeReason(include_reason.clone_reason()),
                }));
            }
        }
        t.type_resolutions_in_file = type_resolutions_in_file;
        t.type_resolutions_trace = type_resolutions_trace;
    }

    pub fn resolve_imports_and_module_augmentations(&self, t: &mut ParseTask) { ::tsox_core::fntrace::enter("resolve_imports_and_module_augmentations"); 
        let file = t.file.as_ref().unwrap().clone();
        let meta = t.metadata.clone();

        let mut module_names: Vec<Arc<Node>> =
            Vec::with_capacity(file.imports.len() + file.module_augmentations.len() + 2);

        let is_javascript_file = is_source_file_js(&file);
        let is_external_module_file = is_external_module(&file);

        let (redirect, file_name) = self
            .project_reference_file_mapper
            .as_ref()
            .unwrap()
            .get_redirect_for_resolution(&HasFileName::new(
                file.file_name.clone(),
                self.to_path(&file.file_name),
            ));
        let options_for_file = get_compiler_options_with_redirect(self.opts.config.compiler_options(), redirect.as_ref());
        if is_javascript_file
            || (!file.is_declaration_file && (options_for_file.get_isolated_modules() || is_external_module_file))
        {
            if options_for_file.import_helpers.is_true() {
                let specifier = self.create_synthetic_import(EXTERNAL_HELPERS_MODULE_NAME_TEXT, &file);
                module_names.push(specifier.clone());
                t.import_helpers_import_specifier = Some(specifier);
            }
        }

        if is_javascript_file || file.script_kind == ScriptKind::Tsx {
            let jsx_import = get_jsx_runtime_import(&get_jsx_implicit_import_base(&options_for_file, &file), &options_for_file);
            if !jsx_import.is_empty() {
                let specifier = self.create_synthetic_import(&jsx_import, &file);
                module_names.push(specifier.clone());
                t.jsx_runtime_import_specifier = Some(JsxRuntimeImportSpecifier {
                    module_reference: jsx_import,
                    specifier,
                });
            }
        }

        let imports_start = module_names.len();
        module_names.extend(file.imports.iter().cloned());
        for imp in &file.module_augmentations {
            if imp.kind == SyntaxKind::StringLiteral {
                module_names.push(imp.clone());
            }
        }

        if self.opts.skip_module_resolution {
            return;
        }

        if !module_names.is_empty() {
            let mut resolutions_in_file = ModeAwareCacheResolvedModule::default();
            let mut resolutions_trace = Vec::new();

            for (index, entry) in module_names.iter().enumerate() {
                let module_name = entry.text();
                if module_name.is_empty() {
                    continue;
                }

                let mode = get_mode_for_usage_location(&file.file_name, &meta, entry, &options_for_file);
                let (resolved_module, trace) = self.resolver.as_ref().unwrap().resolve_module_name(
                    &module_name,
                    &file_name,
                    mode,
                    redirect.as_ref().map(|redirect| redirect.compiler_options.config_file_path.clone()).as_deref(),
                );
            let resolved_module = resolved_module.unwrap_or_default();
                resolutions_in_file.insert(ModeAwareCacheKey::new(module_name.to_string(), mode), resolved_module.clone());
                resolutions_trace.extend(trace);

                if !resolved_module.is_resolved() {
                    continue;
                }

                let resolved_file_name = resolved_module.resolved_file_name.clone();
                let is_from_node_modules_search = resolved_module.is_external_library_import;
                let is_js_file = !resolved_module.resolved_using_extra_extensions
                    && !file_extension_is_one_of(&resolved_file_name, SUPPORTED_TS_EXTENSIONS_WITH_JSON_FL)
                    && self
                        .project_reference_file_mapper
                        .as_ref()
                        .unwrap()
                        .get_redirect_parsed_command_line_for_resolution(&HasFileName::new(
                            resolved_file_name.clone(),
                            self.to_path(&resolved_file_name),
                        ))
                        .is_none();
                let is_js_file_from_node_modules =
                    is_from_node_modules_search && is_js_file && resolved_file_name.contains("/node_modules/");

                let import_index = index as i64 - imports_start as i64;

                let should_add_file = !module_name.is_empty()
                    && get_resolution_diagnostic(&options_for_file, &resolved_module, &file).is_none()
                    && !options_for_file.no_resolve.is_true()
                    && !(is_js_file && !options_for_file.get_allow_js())
                    && (import_index < 0
                        || (import_index < file.imports.len() as i64
                            && (is_in_js_file(&file.imports[import_index as usize])
                                || (file.imports[import_index as usize].flags & NODE_FLAGS_JSDOC).is_empty())));

                if should_add_file {
                    t.add_sub_task(
                        ResolvedRef {
                            file_name: resolved_file_name,
                            increase_depth: resolved_module.is_external_library_import,
                            elide_on_depth: is_js_file_from_node_modules,
                            include_reason: Arc::new(FileIncludeReason::new(
                                FileIncludeKind::Import,
                                FileIncludeReasonData::ReferencedFile(Box::new(ReferencedFileData {
                                    file: t.path.clone(),
                                    index: import_index.max(0) as usize,
                                    synthetic: if import_index < 0 { Some(entry.clone()) } else { None },
                                })),
                            )),
                            package_id: resolved_module.package_id.clone(),
                        },
                        None,
                    );
                }
            }

            t.resolutions_in_file = resolutions_in_file;
            t.resolutions_trace = resolutions_trace;
        }
    }

    pub fn create_synthetic_import(&self, text: &str, file: &Arc<SourceFile>) -> Arc<Node> { ::tsox_core::fntrace::enter("create_synthetic_import"); 
        let _guard = self.factory_mu.lock().unwrap();
        let external_helpers_module_reference = self.factory.new_string_literal(text, TokenFlags::None);
        let import_decl = new_import_declaration_node(external_helpers_module_reference.clone());
        external_helpers_module_reference.set_parent(&import_decl);
        import_decl.set_parent(&file.node);
        external_helpers_module_reference
    }

    pub fn path_for_lib_file(&mut self, name: &str) -> LibFile { ::tsox_core::fntrace::enter("path_for_lib_file"); 
        if let Some(cached) = self.path_for_lib_file_cache.get(name) {
            return LibFile { name: cached.name.clone(), path: cached.path.clone(), replaced: cached.replaced };
        }

        let mut path = combine_paths(&self.default_library_path, &[name]);
        let mut replaced = false;
        if !self.opts.skip_module_resolution
            && self.opts.config.compiler_options().lib_replacement.is_true()
            && name != "lib.d.ts"
        {
            let library_name = get_library_name_from_lib_file_name(name);
            let resolve_from = get_inferred_library_name_resolve_from(
                self.opts.config.compiler_options(),
                self.opts.host.current_directory(),
                name,
            );
            let (resolution, trace) = self.resolve_library(&library_name, &resolve_from);
            if resolution.is_resolved() {
                path = resolution.resolved_file_name.clone();
                replaced = true;
            }
            self.path_for_lib_file_resolutions.entry(self.to_path(&resolve_from)).or_insert(LibResolution {
                library_name,
                resolution: Some(resolution),
                trace,
            });
        }

        let lib_file = LibFile { name: name.to_string(), path, replaced };
        self.path_for_lib_file_cache.insert(name.to_string(), LibFile {
            name: lib_file.name.clone(),
            path: lib_file.path.clone(),
            replaced: lib_file.replaced,
        });
        lib_file
    }

    pub fn resolve_library(&self, library_name: &str, resolve_from: &str) -> (ResolvedModule, Vec<DiagAndArgs>) { ::tsox_core::fntrace::enter("resolve_library"); 
        let (resolved, trace) = self
            .resolver
            .as_ref()
            .unwrap()
            .resolve_module_name(library_name, resolve_from, ModuleKind::CommonJS, None);
        (resolved.unwrap_or_default(), trace)
    }
}

pub fn content_mapper_transform_diagnostic(file: &Arc<SourceFile>, label: &str, err: &ContentMapperError) -> Diagnostic { ::tsox_core::fntrace::enter("content_mapper_transform_diagnostic"); 
    if let Some(collision) = err.as_supplemental_file_collision_error() {
        return content_mapper_transform_diagnostic_chain(
            file,
            label,
            CONTENT_MAPPER_SUPPLEMENTAL_OUTPUT_FILE_0_CONFLICTS_WITH_AN_EXISTING_FILE,
            vec![collision.file_name.clone()],
        );
    }
    if let Some(transform_error) = err.as_transform_error() {
        match transform_error.kind {
            TransformErrorKind::Initialize => {
                if let Some(initialize_error) = transform_error
                    .unwrap_err()
                    .and_then(|err| err.downcast_ref::<InitializeError>())
                {
                    match initialize_error.kind {
                        InitializeErrorKind::PositionEncoding => {
                            return content_mapper_transform_diagnostic_chain(
                                file,
                                label,
                                THE_CONTENT_MAPPER_SELECTED_UNSUPPORTED_POSITION_ENCODING_0,
                                vec![initialize_error.position_encoding.clone()],
                            );
                        }
                        InitializeErrorKind::EmptyDiagnosticSource => {
                            return content_mapper_transform_diagnostic_chain(
                                file,
                                label,
                                THE_CONTENT_MAPPER_DIAGNOSTIC_SOURCE_MUST_NOT_BE_EMPTY,
                                Vec::new(),
                            );
                        }
                        InitializeErrorKind::ReservedDiagnosticSource => {
                            return content_mapper_transform_diagnostic_chain(
                                file,
                                label,
                                THE_CONTENT_MAPPER_DIAGNOSTIC_SOURCE_0_IS_RESERVED_BY_TYPESCRIPT,
                                vec![initialize_error.diagnostic_source.clone()],
                            );
                        }
                        // Go: 其余 kind 落出 switch,走下方 could-not-be-started-or-initialized 兜底
                        InitializeErrorKind::ProcessStart => {}
                        InitializeErrorKind::ProcessExit => {}
                        InitializeErrorKind::NoResponse => {}
                        InitializeErrorKind::InvalidResponse => {}
                        InitializeErrorKind::Request => {}
                    }
                }
                content_mapper_transform_diagnostic_chain(
                    file,
                    label,
                    THE_CONTENT_MAPPER_PROCESS_COULD_NOT_BE_STARTED_OR_INITIALIZED,
                    Vec::new(),
                )
            }
            TransformErrorKind::Project => content_mapper_transform_diagnostic_chain(
                file,
                label,
                content_mapper_project_error_diagnostic(err),
                Vec::new(),
            ),
            TransformErrorKind::Request => content_mapper_transform_diagnostic_chain(
                file,
                label,
                THE_CONTENT_MAPPER_PROCESS_FAILED_WHILE_HANDLING_THE_TRANSFORM_REQUEST,
                Vec::new(),
            ),
            TransformErrorKind::Response => {
                if let Some(extension_error) = transform_error
                    .unwrap_err()
                    .and_then(|err| err.downcast_ref::<InvalidVirtualExtensionError>())
                {
                    return content_mapper_transform_diagnostic_chain(
                        file,
                        label,
                        THE_CONTENT_MAPPER_RETURNED_AN_OUTPUT_WITH_UNSUPPORTED_VIRTUAL_EXTENSION_0,
                        vec![extension_error.extension.clone()],
                    );
                }
                if let Some(directive_error) = transform_error
                    .unwrap_err()
                    .and_then(|err| err.downcast_ref::<DiagnosticDirectiveError>())
                {
                    let mut detail: Option<Diagnostic> = None;
                    match directive_error.kind {
                        DiagnosticDirectiveErrorKind::InvalidRange => {
                            detail = Some(new_compiler_diagnostic(
                                DIAGNOSTIC_DIRECTIVE_0_RETURNED_BY_THE_CONTENT_MAPPER_HAS_AN_INVALID_RANGE,
                                vec![directive_error.index.to_string()],
                            ));
                        }
                        DiagnosticDirectiveErrorKind::InvalidPolicy => {
                            detail = Some(new_compiler_diagnostic(
                                THE_CONTENT_MAPPER_RETURNED_A_DIAGNOSTIC_DIRECTIVE_WITH_INVALID_POLICY_0,
                                vec![match directive_error.policy {
                                    DiagnosticDirectivePolicy::Ignore => "ignore".to_string(),
                                    DiagnosticDirectivePolicy::Expect => "expect".to_string(),
                                }],
                            ));
                        }
                        DiagnosticDirectiveErrorKind::ExpectMissingUnusedDiagnostic => {
                            detail = Some(new_compiler_diagnostic(
                                DIAGNOSTIC_DIRECTIVE_0_RETURNED_BY_THE_CONTENT_MAPPER_MUST_SPECIFY_UNUSEDEXPECTDIRECTIVEINDEX_WHEN_THERE_IS_NOT_EXACTLY_ONE_UNUSEDEXPECTDIRECTIVEDIAGNOSTICS_ENTRY,
                                vec![directive_error.index.to_string()],
                            ));
                        }
                        DiagnosticDirectiveErrorKind::InvalidUnusedDiagnosticIndex => {
                            detail = Some(new_compiler_diagnostic(
                                DIAGNOSTIC_DIRECTIVE_0_RETURNED_BY_THE_CONTENT_MAPPER_HAS_AN_INVALID_UNUSEDEXPECTDIRECTIVEINDEX,
                                vec![directive_error.index.to_string()],
                            ));
                        }
                        DiagnosticDirectiveErrorKind::Overlap => {
                            detail = Some(new_compiler_diagnostic(
                                THE_CONTENT_MAPPER_RETURNED_DIAGNOSTIVE_DIRECTIVES_WITH_OVERLAPPING_VIRTUAL_RANGES,
                                Vec::new(),
                            ));
                        }
                    }
                    if let Some(mut detail) = detail {
                        if directive_error.supplemental_index >= 0 {
                            detail = new_diagnostic_chain(
                                Some(&detail),
                                THE_INVALID_DIAGNOSTIC_DIRECTIVE_IS_IN_SUPPLEMENTAL_OUTPUT_0_RETURNED_BY_THE_CONTENT_MAPPER,
                                vec![directive_error.supplemental_index.to_string()],
                            );
                        }
                        return content_mapper_transform_diagnostic_with_detail(file, label, detail);
                    }
                }
                content_mapper_transform_diagnostic_chain(
                    file,
                    label,
                    THE_CONTENT_MAPPER_RETURNED_AN_INVALID_TRANSFORM_RESPONSE,
                    Vec::new(),
                )
            }
            TransformErrorKind::Mappings => new_diagnostic(
                Some(file.clone()),
                new_text_range(0, 0),
                THE_CONTENT_MAPPER_0_DID_NOT_PROVIDE_THE_REQUIRED_POSITION_MAPPINGS,
                vec![label.to_string()],
            ),
            // Go: Unknown 落出 switch,走末尾兜底诊断
            TransformErrorKind::Unknown => new_diagnostic(
                Some(file.clone()),
                new_text_range(0, 0),
                THE_CONTENT_MAPPER_0_FAILED_TO_TRANSFORM_THIS_FILE,
                vec![label.to_string()],
            ),
        }
    } else {
        new_diagnostic(
            Some(file.clone()),
            new_text_range(0, 0),
            THE_CONTENT_MAPPER_0_FAILED_TO_TRANSFORM_THIS_FILE,
            vec![label.to_string()],
        )
    }
}

pub fn content_mapper_project_error_diagnostic(err: &ContentMapperError) -> tsox_core::diagnostics::Message { ::tsox_core::fntrace::enter("content_mapper_project_error_diagnostic"); 
    if let Some(project_error) = err.as_project_error() {
        match project_error.kind {
            ProjectErrorKind::MalformedResponse => {
                return THE_CONTENT_MAPPER_RETURNED_A_PROJECT_RESPONSE_THAT_COULD_NOT_BE_DECODED;
            }
            ProjectErrorKind::MissingConfigIdentity => {
                return THE_CONTENT_MAPPER_DID_NOT_RETURN_CONFIGIDENTITY_WHICH_IS_REQUIRED_WHEN_THE_CONTENT_MAPPER_HAS_DYNAMICCONFIG_COLON_TRUE_IN_ITS_PACKAGE_JSON;
            }
            ProjectErrorKind::NonAbsoluteWatchedFile => {
                return THE_CONTENT_MAPPER_RETURNED_A_NON_ABSOLUTE_PATH_IN_WATCHEDFILES;
            }
            ProjectErrorKind::UnexpectedConfigIdentity => {
                return THE_CONTENT_MAPPER_RETURNED_CONFIGIDENTITY_WHICH_IS_ONLY_ALLOWED_WHEN_IT_DECLARES_DYNAMICCONFIG_COLON_TRUE_IN_ITS_PACKAGE_JSON;
            }
            ProjectErrorKind::UnexpectedWatchedFiles => {
                return THE_CONTENT_MAPPER_RETURNED_WATCHEDFILES_WHICH_IS_ONLY_ALLOWED_WHEN_IT_DECLARES_DYNAMICCONFIG_COLON_TRUE_IN_ITS_PACKAGE_JSON;
            }
        }
    }
    THE_CONTENT_MAPPER_PROCESS_FAILED_WHILE_HANDLING_THE_PROJECT_REQUEST
}

pub fn content_mapper_transform_diagnostic_chain(
    file: &Arc<SourceFile>,
    label: &str,
    message: tsox_core::diagnostics::Message,
    args: Vec<String>,
) -> Diagnostic { ::tsox_core::fntrace::enter("content_mapper_transform_diagnostic_chain"); 
    content_mapper_transform_diagnostic_with_detail(file, label, new_compiler_diagnostic(message, args))
}

pub fn content_mapper_transform_diagnostic_with_detail(file: &Arc<SourceFile>, label: &str, detail: Diagnostic) -> Diagnostic { ::tsox_core::fntrace::enter("content_mapper_transform_diagnostic_with_detail"); 
    let mut diagnostic = new_diagnostic(
        Some(file.clone()),
        new_text_range(0, 0),
        THE_CONTENT_MAPPER_0_FAILED_TO_TRANSFORM_THIS_FILE,
        vec![label.to_string()],
    );
    diagnostic.message_chain.push(detail);
    diagnostic
}

pub fn content_mapper_mapping_diagnostic(file: &Arc<SourceFile>, label: &str, problem: &MappingError) -> Diagnostic { ::tsox_core::fntrace::enter("content_mapper_mapping_diagnostic"); 
    let loc = new_text_range(0, 0);
    match problem.kind {
        MappingErrorKind::Overlap => new_diagnostic(
            Some(file.clone()),
            loc,
            THE_CONTENT_MAPPER_0_PRODUCED_OVERLAPPING_OR_OUT_OF_ORDER_POSITION_MAPPINGS_NEAR_VIRTUAL_OFFSET_1,
            vec![label.to_string(), problem.virtual_pos.to_string()],
        ),
        MappingErrorKind::OutOfBounds => new_diagnostic(
            Some(file.clone()),
            loc,
            THE_CONTENT_MAPPER_0_PRODUCED_A_POSITION_MAPPING_THAT_POINTS_OUTSIDE_THE_ORIGINAL_CONTENT_ORIGINAL_OFFSET_1,
            vec![label.to_string(), problem.original_pos.to_string()],
        ),
        MappingErrorKind::VerbatimMismatch => new_diagnostic(
            Some(file.clone()),
            loc,
            THE_CONTENT_MAPPER_0_PRODUCED_A_VERBATIM_MAPPING_THAT_DOES_NOT_MATCH_THE_ORIGINAL_CONTENT_VIRTUAL_OFFSET_1_ORIGINAL_OFFSET_2,
            vec![label.to_string(), problem.virtual_pos.to_string(), problem.original_pos.to_string()],
        ),
        MappingErrorKind::Kind => new_diagnostic(
            Some(file.clone()),
            loc,
            THE_CONTENT_MAPPER_0_PRODUCED_A_POSITION_MAPPING_WITH_AN_INVALID_KIND_NEAR_VIRTUAL_OFFSET_1,
            vec![label.to_string(), problem.virtual_pos.to_string()],
        ),
        MappingErrorKind::Feature => new_diagnostic(
            Some(file.clone()),
            loc,
            THE_CONTENT_MAPPER_0_PRODUCED_INVALID_MAPPING_FEATURES_NEAR_ORIGINAL_OFFSET_1,
            vec![label.to_string(), problem.original_pos.to_string()],
        ),
    }
}

pub fn content_mapper_initialization_diagnostic(label: &str, err: &ContentMapperError) -> Diagnostic { ::tsox_core::fntrace::enter("content_mapper_initialization_diagnostic"); 
    let mut label = label.to_string();
    if let Some(initialize_error) = err.as_initialize_error() {
        if label.is_empty() {
            label = initialize_error.mapper_name.clone();
        }
    }
    let diagnostic = new_compiler_diagnostic(
        THE_CONTENT_MAPPER_0_COULD_NOT_BE_INITIALIZED,
        vec![label.clone()],
    );
    if let Some(initialize_error) = err.as_initialize_error() {
        match initialize_error.kind {
            InitializeErrorKind::ProcessStart => {
                return add_message_chain(
                    diagnostic,
                    new_compiler_diagnostic(
                        THE_CONTENT_MAPPER_COMMAND_0_COULD_NOT_BE_STARTED_COLON_1,
                        vec![initialize_error.command.clone(), initialize_error.detail.clone()],
                    ),
                );
            }
            InitializeErrorKind::ProcessExit => {
                return add_message_chain(
                    diagnostic,
                    new_compiler_diagnostic(
                        THE_CONTENT_MAPPER_PROCESS_EXITED_BEFORE_RESPONDING_TO_THE_INITIALIZE_REQUEST_EXIT_CODE_0,
                        vec![initialize_error.exit_code.to_string()],
                    ),
                );
            }
            InitializeErrorKind::NoResponse => {
                return add_message_chain(
                    diagnostic,
                    new_compiler_diagnostic(
                        THE_CONTENT_MAPPER_DID_NOT_RESPOND_TO_THE_INITIALIZE_REQUEST_WITHIN_0_SECONDS,
                        vec![initialize_error.timeout_seconds.to_string()],
                    ),
                );
            }
            InitializeErrorKind::InvalidResponse => {
                return add_message_chain(
                    diagnostic,
                    new_compiler_diagnostic(
                        THE_CONTENT_MAPPER_RETURNED_AN_INITIALIZE_RESPONSE_THAT_COULD_NOT_BE_DECODED_COLON_0,
                        vec![initialize_error.detail.clone()],
                    ),
                );
            }
            InitializeErrorKind::Request => {
                return add_message_chain(
                    diagnostic,
                    new_compiler_diagnostic(
                        THE_CONTENT_MAPPER_S_INITIALIZE_REQUEST_FAILED_COLON_0,
                        vec![initialize_error.detail.clone()],
                    ),
                );
            }
            InitializeErrorKind::PositionEncoding => {
                return add_message_chain(
                    diagnostic,
                    new_compiler_diagnostic(
                        THE_CONTENT_MAPPER_SELECTED_UNSUPPORTED_POSITION_ENCODING_0,
                        vec![initialize_error.position_encoding.clone()],
                    ),
                );
            }
            InitializeErrorKind::EmptyDiagnosticSource => {
                return add_message_chain(
                    diagnostic,
                    new_compiler_diagnostic(THE_CONTENT_MAPPER_DIAGNOSTIC_SOURCE_MUST_NOT_BE_EMPTY, Vec::new()),
                );
            }
            InitializeErrorKind::ReservedDiagnosticSource => {
                return add_message_chain(
                    diagnostic,
                    new_compiler_diagnostic(
                        THE_CONTENT_MAPPER_DIAGNOSTIC_SOURCE_0_IS_RESERVED_BY_TYPESCRIPT,
                        vec![initialize_error.diagnostic_source.clone()],
                    ),
                );
            }
        }
    }
    add_message_chain(
        diagnostic,
        new_compiler_diagnostic(THE_CONTENT_MAPPER_PROCESS_COULD_NOT_BE_STARTED_OR_INITIALIZED, Vec::new()),
    )
}

pub fn content_mapper_project_diagnostic(err: &ContentMapperError) -> Diagnostic { ::tsox_core::fntrace::enter("content_mapper_project_diagnostic"); 
    if err.as_initialize_error().is_some() {
        return content_mapper_initialization_diagnostic("", err);
    }
    new_compiler_diagnostic(content_mapper_project_error_diagnostic(err), Vec::new())
}

pub fn get_library_name_from_lib_file_name(lib_file_name: &str) -> String { ::tsox_core::fntrace::enter("get_library_name_from_lib_file_name"); 
    let components: Vec<&str> = lib_file_name.split('.').collect();
    let mut path = String::from("@typescript/lib-");
    if components.len() > 1 {
        path.push_str(components[1]);
    }
    let mut i = 2;
    while i < components.len() && !components[i].is_empty() && components[i] != "d" {
        if i == 2 {
            path.push('/');
        } else {
            path.push('-');
        }
        path.push_str(components[i]);
        i += 1;
    }
    path
}

pub fn get_inferred_library_name_resolve_from(options: &CompilerOptions, current_directory: &str, lib_file_name: &str) -> String { ::tsox_core::fntrace::enter("get_inferred_library_name_resolve_from"); 
    let containing_directory = if !options.config_file_path.is_empty() {
        get_directory_path(&options.config_file_path)
    } else {
        current_directory.to_string()
    };
    combine_paths(
        &containing_directory,
        &[format!("__lib_node_modules_lookup_{}__.ts", lib_file_name).as_str()],
    )
}

pub fn get_mode_for_type_reference_directive_in_file(
    ref_: &FileReference,
    file: &Arc<SourceFile>,
    meta: &SourceFileMetaData,
    options: &CompilerOptions,
) -> ResolutionMode { ::tsox_core::fntrace::enter("get_mode_for_type_reference_directive_in_file"); 
    if ref_.resolution_mode != ResolutionMode::None {
        ref_.resolution_mode
    } else {
        get_default_resolution_mode_for_file(&file.file_name, meta, options)
    }
}

pub fn get_default_resolution_mode_for_file(
    file_name: &str,
    meta: &SourceFileMetaData,
    options: &CompilerOptions,
) -> ResolutionMode { ::tsox_core::fntrace::enter("get_default_resolution_mode_for_file"); 
    if import_syntax_affects_module_resolution(options) {
        get_implied_node_format_for_emit_worker(file_name, options.get_emit_module_kind(), meta.clone())
    } else {
        ResolutionMode::None
    }
}

pub fn get_mode_for_usage_location(
    file_name: &str,
    meta: &SourceFileMetaData,
    usage: &Arc<Node>,
    options: &CompilerOptions,
) -> ResolutionMode { ::tsox_core::fntrace::enter("get_mode_for_usage_location"); 
    let Some(parent) = usage.parent() else {
        return ResolutionMode::None;
    };
    if is_import_declaration(&parent)
        || parent.kind == SyntaxKind::JSImportDeclaration
        || is_export_declaration(&parent)
        || is_jsdoc_import_tag(&parent)
    {
        let is_type_only = is_exclusively_type_only_import_or_export(&parent);
        if is_type_only {
            let override_ = match parent.kind {
                SyntaxKind::ImportDeclaration | SyntaxKind::JSImportDeclaration => match &parent.data {
                    NodeData::ImportDeclaration(d) => {
                        attributes_resolution_mode_override(d.attributes.as_ref())
                    }
                    _ => None,
                },
                SyntaxKind::ExportDeclaration => match &parent.data {
                    NodeData::ExportDeclaration(d) => {
                        attributes_resolution_mode_override(d.attributes.as_ref())
                    }
                    _ => None,
                },
                SyntaxKind::JSDocImportTag => match &parent.data {
                    NodeData::JSDocImportTag(d) => {
                        attributes_resolution_mode_override(d.attributes.as_ref())
                    }
                    _ => None,
                },
                _ => None,
            };
            if let Some(override_) = override_ {
                return override_;
            }
        }
    }
    let parent_parent = parent.parent();
    if is_literal_type_node(&parent) && parent_parent.as_ref().is_some_and(|p| is_import_type_node(p)) {
        if let Some(override_) = match &parent_parent.as_ref().unwrap().data {
            NodeData::ImportTypeNode(d) => attributes_resolution_mode_override(d.attributes.as_ref()),
            _ => None,
        } {
            return override_;
        }
    }

    if import_syntax_affects_module_resolution(options) {
        return get_emit_syntax_for_usage_location_worker(file_name, meta, usage, options);
    }

    ResolutionMode::None
}

fn attributes_resolution_mode_override(attributes: Option<&Arc<Node>>) -> Option<ResolutionMode> { ::tsox_core::fntrace::enter("attributes_resolution_mode_override"); 
    let attributes = attributes?;
    let (mode, ok) = get_resolution_mode_override(attributes, None);
    ok.then_some(mode)
}

pub fn import_syntax_affects_module_resolution(options: &CompilerOptions) -> bool { ::tsox_core::fntrace::enter("import_syntax_affects_module_resolution"); 
    let module_resolution = options.get_module_resolution_kind();
    ((module_resolution as i32) >= (ModuleResolutionKind::Node16 as i32)
        && (module_resolution as i32) <= (ModuleResolutionKind::NodeNext as i32))
        || options.get_resolve_package_json_exports()
        || options.get_resolve_package_json_imports()
}

pub fn get_emit_syntax_for_usage_location_worker(
    file_name: &str,
    meta: &SourceFileMetaData,
    usage: &Arc<Node>,
    options: &CompilerOptions,
) -> ResolutionMode { ::tsox_core::fntrace::enter("get_emit_syntax_for_usage_location_worker"); 
    let parent = usage.parent();
    if let Some(parent) = parent.as_ref() {
        if is_require_call(parent, false)
            || is_external_module_reference(parent)
                && parent.parent().is_some_and(|p| is_import_equals_declaration(&p))
        {
            return ModuleKind::CommonJS;
        }
    }
    let file_emit_mode = get_emit_module_format_of_file_worker(file_name, options, meta.clone());
    let walked = parent.as_ref().and_then(|parent| walk_up_parenthesized_expressions(parent));
    if walked.is_some_and(|node| is_import_call(&node)) {
        if should_transform_import_call(file_name, options, file_emit_mode) {
            return ModuleKind::CommonJS;
        }
        return ModuleKind::ESNext;
    }
    if file_emit_mode == ModuleKind::CommonJS {
        return ModuleKind::CommonJS;
    }
    if file_emit_mode.is_non_node_esm() || file_emit_mode == ModuleKind::Preserve {
        return ModuleKind::ESNext;
    }
    ModuleKind::None
}

struct CollectFilesState {
    missing_files: Vec<String>,
    duplicate_source_files: Vec<DuplicateSourceFile>,
    files: Vec<Arc<SourceFile>>,
    lib_files: Vec<Arc<SourceFile>>,
    files_by_path: HashMap<Path, Arc<SourceFile>>,
    tasks_seen_by_name_ignore_case: Option<HashMap<String, Arc<Mutex<ParseTask>>>>,
    include_processor: IncludeProcessor,
    output_file_to_project_reference_source: Option<HashMap<Path, String>>,
    resolved_modules: HashMap<Path, ModeAwareCacheResolvedModule>,
    type_resolutions_in_file: HashMap<Path, ModeAwareCacheResolvedTypeReferenceDirective>,
    source_file_meta_datas: HashMap<Path, SourceFileMetaData>,
    jsx_runtime_import_specifiers: HashMap<Path, JsxRuntimeImportSpecifier>,
    import_helpers_import_specifiers: HashMap<Path, Arc<Node>>,
    source_files_found_searching_node_modules: HashSetTspath,
    lib_files_map: HashMap<Path, LibFile>,
    redirect_targets_map: Option<HashMap<Path, Vec<String>>>,
    redirect_files_by_path: Option<HashMap<Path, RedirectsFile>>,
    package_id_to_source_file: Option<HashMap<PackageId, Arc<SourceFile>>>,
    seen: HashMap<usize, String>,
    recorded_duplicates: HashMap<usize, HashSet<String>>,
}

impl FilesParser {
    pub fn parse(&mut self, loader: &mut FileLoader, tasks: &[Arc<Mutex<ParseTask>>]) { ::tsox_core::fntrace::enter("parse"); 
        self.start(loader, tasks, 0);
        self.wg.run_and_wait();
    }

    fn start(&mut self, loader: &mut FileLoader, tasks: &[Arc<Mutex<ParseTask>>], depth: i32) { ::tsox_core::fntrace::enter("start"); 
        for task in tasks.iter() {
            {
                let mut t = task.lock().unwrap();
                t.path = loader.to_path(&t.normalized_file_path);
            }
            let candidate = Arc::new(ParseTaskData::new(task.clone()));
            let (data, loaded) = {
                let mut map = self.task_data_by_path.lock().unwrap();
                let path = task.lock().unwrap().path.clone();
                match map.entry(path) {
                    std::collections::hash_map::Entry::Occupied(entry) => (entry.get().clone(), true),
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(candidate.clone());
                        (candidate, false)
                    }
                }
            };
            self.process_task(loader, &data, loaded, task, depth);
        }
    }

    fn process_task(
        &mut self,
        loader: &mut FileLoader,
        data: &Arc<ParseTaskData>,
        loaded: bool,
        task: &Arc<Mutex<ParseTask>>,
        depth: i32,
    ) { ::tsox_core::fntrace::enter("process_task"); 
        let mut start_subtasks = false;
        if loaded {
            let existing = {
                let key = task.lock().unwrap().normalized_file_path.clone();
                data.tasks.lock().unwrap().get(&key).cloned()
            };
            match existing {
                Some(existing_task) => {
                    task.lock().unwrap().loaded_task = Some(existing_task);
                }
                None => {
                    let key = task.lock().unwrap().normalized_file_path.clone();
                    data.tasks.lock().unwrap().insert(key, task.clone());
                    start_subtasks = *data.started_sub_tasks.lock().unwrap();
                }
            }
        }

        {
            let t = task.lock().unwrap();
            let mut data_package_id = data.package_id.lock().unwrap();
            let data_has = data_package_id.as_ref().is_some_and(|p| !p.name.is_empty());
            let task_has = t.package_id.as_ref().is_some_and(|p| !p.name.is_empty());
            if !data_has && task_has {
                *data_package_id = t.package_id.clone();
            }
        }

        let current_depth = {
            let t = task.lock().unwrap();
            if t.increase_depth {
                depth + 1
            } else {
                depth
            }
        };
        if current_depth < *data.lowest_depth.lock().unwrap() {
            *data.lowest_depth.lock().unwrap() = current_depth;
            start_subtasks = true;
            *data.started_sub_tasks.lock().unwrap() = true;
        }

        if task.lock().unwrap().elide_on_depth && current_depth > self.max_depth {
            return;
        }

        let tasks_by_file_name: Vec<Arc<Mutex<ParseTask>>> =
            data.tasks.lock().unwrap().values().cloned().collect();
        for task_by_file_name in tasks_by_file_name {
            let mut load_sub_tasks = start_subtasks;
            if !task_by_file_name.lock().unwrap().loaded {
                task_by_file_name.lock().unwrap().load(loader);
                if task_by_file_name.lock().unwrap().redirected_parse_task.is_some() {
                    load_sub_tasks = true;
                    *data.started_sub_tasks.lock().unwrap() = true;
                }
            }
            if !task_by_file_name.lock().unwrap().started_sub_tasks && load_sub_tasks {
                task_by_file_name.lock().unwrap().started_sub_tasks = true;
                let sub_tasks = task_by_file_name.lock().unwrap().sub_tasks.clone();
                let lowest_depth = *data.lowest_depth.lock().unwrap();
                self.start(loader, &sub_tasks, lowest_depth);
            }
        }
    }

    pub fn get_processed_files(&self, loader: &mut FileLoader) -> ProcessedFiles { ::tsox_core::fntrace::enter("get_processed_files"); 
        let total_file_count = loader.total_file_count.load(Ordering::SeqCst) as usize;
        let lib_file_count = loader.lib_file_count.load(Ordering::SeqCst) as usize;
        let use_case_sensitive_file_names = loader.compare_paths_options.use_case_sensitive_file_names;
        let deduplicate_packages_not_false = !loader
            .opts
            .config
            .compiler_options()
            .deduplicate_packages
            .is_false();

        let mut state = CollectFilesState {
            missing_files: Vec::new(),
            duplicate_source_files: Vec::new(),
            files: Vec::with_capacity(total_file_count.saturating_sub(lib_file_count)),
            lib_files: Vec::with_capacity(total_file_count),
            files_by_path: HashMap::with_capacity(total_file_count),
            tasks_seen_by_name_ignore_case: use_case_sensitive_file_names
                .then(|| HashMap::with_capacity(total_file_count)),
            include_processor: IncludeProcessor::default(),
            output_file_to_project_reference_source: (!loader.opts.can_use_project_reference_source())
                .then(|| HashMap::with_capacity(total_file_count)),
            resolved_modules: HashMap::with_capacity(total_file_count + 1),
            type_resolutions_in_file: HashMap::with_capacity(total_file_count),
            source_file_meta_datas: HashMap::with_capacity(total_file_count),
            jsx_runtime_import_specifiers: HashMap::new(),
            import_helpers_import_specifiers: HashMap::new(),
            source_files_found_searching_node_modules: HashSet::new(),
            lib_files_map: HashMap::with_capacity(lib_file_count),
            redirect_targets_map: deduplicate_packages_not_false.then(|| HashMap::new()),
            redirect_files_by_path: None,
            package_id_to_source_file: deduplicate_packages_not_false.then(|| HashMap::new()),
            seen: HashMap::with_capacity(total_file_count),
            recorded_duplicates: HashMap::new(),
        };

        let root_tasks = loader.root_tasks.clone();
        self.collect_files(loader, &root_tasks, &mut state);

        loader.sort_libs(&mut state.lib_files);
        let lib_files_len = state.lib_files.len();
        let mut all_files = std::mem::take(&mut state.lib_files);
        all_files.extend(state.files.drain(..));
        if let Some(redirect_files_by_path) = &mut state.redirect_files_by_path {
            for redirect_file in redirect_files_by_path.values_mut() {
                redirect_file.index += lib_files_len;
            }
        }

        let mut keys: Vec<Path> = loader.path_for_lib_file_resolutions.keys().cloned().collect();
        keys.sort_by(|a, b| a.0.cmp(&b.0));
        for key in keys {
            let value = loader.path_for_lib_file_resolutions.get(&key).unwrap();
            let mut cache = ModeAwareCacheResolvedModule::new();
            cache.insert(
                ModeAwareCacheKey::new(value.library_name.clone(), ResolutionMode::CommonJS),
                value.resolution.clone().unwrap_or_default(),
            );
            state.resolved_modules.insert(key, cache);
        }

        ProcessedFiles {
            finished_processing: true,
            resolver: loader.resolver.clone(),
            files: all_files,
            duplicate_source_files: state.duplicate_source_files,
            files_by_path: state.files_by_path,
            project_reference_file_mapper: loader.project_reference_file_mapper.clone(),
            resolved_modules: state.resolved_modules,
            type_resolutions_in_file: state.type_resolutions_in_file,
            source_file_meta_datas: state.source_file_meta_datas,
            jsx_runtime_import_specifiers: state.jsx_runtime_import_specifiers,
            import_helpers_import_specifiers: state.import_helpers_import_specifiers,
            lib_files: state.lib_files_map,
            missing_files: state.missing_files,
            include_processor: Some(Box::new(state.include_processor)),
            output_file_to_project_reference_source: state.output_file_to_project_reference_source,
            redirect_targets_map: state.redirect_targets_map,
            redirect_files_by_path: state.redirect_files_by_path,
            content_mapper_diagnostics: loader
                .content_mapper_mu
                .lock()
                .unwrap()
                .content_mapper_diagnostics
                .clone(),
            source_files_found_searching_node_modules: state.source_files_found_searching_node_modules,
        }
    }

    fn collect_files(
        &self,
        loader: &mut FileLoader,
        tasks: &[Arc<Mutex<ParseTask>>],
        state: &mut CollectFilesState,
    ) { ::tsox_core::fntrace::enter("collect_files"); 
        for task in tasks.iter() {
            let include_reason = task.lock().unwrap().include_reason.clone();
            let mut current = task.clone();
            let should_adjust = {
                let t = current.lock().unwrap();
                t.redirected_parse_task.is_none() && !t.is_for_automatic_type_directive
            };
            if should_adjust {
                let loaded_task = current.lock().unwrap().loaded_task.clone();
                if let Some(loaded_task) = loaded_task {
                    current = loaded_task;
                }
                if let Some(reason) = &include_reason {
                    self.add_include_reason(&mut state.include_processor, &current, reason);
                }
            }

            let path_key = current.lock().unwrap().path.clone();
            let data = loader
                .files_parser
                .as_ref()
                .unwrap()
                .task_data_by_path
                .lock()
                .unwrap()
                .get(&path_key)
                .cloned();
            let data_key = data.as_ref().map(|d| Arc::as_ptr(d) as usize).unwrap_or(0);

            if !current.lock().unwrap().loaded {
                continue;
            }

            let normalized_file_path = current.lock().unwrap().normalized_file_path.clone();
            if let Some(checked_name) = state.seen.get(&data_key).cloned() {
                let file = current.lock().unwrap().file.clone();
                if let Some(file) = &file {
                    if checked_name != normalized_file_path {
                        let dups = state.recorded_duplicates.entry(data_key).or_default();
                        if dups.insert(normalized_file_path.clone()) {
                            state.duplicate_source_files.push(DuplicateSourceFile {
                                file_name: file.file_name.clone(),
                                hash: 0,
                                script_kind: file.script_kind,
                                content_mapper: String::new(),
                                is_content_mapper_failure_stub: false,
                            });
                        }
                    }
                }
                if !loader
                    .opts
                    .config
                    .compiler_options()
                    .force_consistent_casing_in_file_names
                    .is_false()
                {
                    let checked_absolute_path = get_normalized_absolute_path_without_root(
                        &checked_name,
                        &loader.compare_paths_options.current_directory,
                    );
                    let input_absolute_path = get_normalized_absolute_path_without_root(
                        &normalized_file_path,
                        &loader.compare_paths_options.current_directory,
                    );
                    if checked_absolute_path != input_absolute_path {
                        if let Some(reason) = &include_reason {
                            state
                                .include_processor
                                .add_processing_diagnostics_for_file_casing(
                                    &path_key,
                                    &checked_name,
                                    &normalized_file_path,
                                    reason,
                                );
                        }
                    }
                }
                continue;
            } else {
                state.seen.insert(data_key, normalized_file_path.clone());
            }

            if state.tasks_seen_by_name_ignore_case.is_some() {
                let path_lower_case = to_file_name_lower_case(path_key.as_str());
                let seen_task = state
                    .tasks_seen_by_name_ignore_case
                    .as_ref()
                    .unwrap()
                    .get(&path_lower_case)
                    .cloned();
                match seen_task {
                    Some(task_by_ignore_case) => {
                        let (existing_path, existing_name) = {
                            let t = task_by_ignore_case.lock().unwrap();
                            (t.path.clone(), t.normalized_file_path.clone())
                        };
                        if let Some(reason) = &include_reason {
                            state
                                .include_processor
                                .add_processing_diagnostics_for_file_casing(
                                    &existing_path,
                                    &existing_name,
                                    &normalized_file_path,
                                    reason,
                                );
                        }
                    }
                    None => {
                        state
                            .tasks_seen_by_name_ignore_case
                            .as_mut()
                            .unwrap()
                            .insert(path_lower_case, current.clone());
                    }
                }
            }

            let mut file = current.lock().unwrap().file.clone();
            if state.package_id_to_source_file.is_some() {
                let data_package_id = data
                    .as_ref()
                    .map(|d| d.package_id.lock().unwrap().clone())
                    .unwrap_or_default()
                    .filter(|package_id| !package_id.name.is_empty());
                if let Some(data_package_id) = data_package_id {
                    let package_id_file = state
                        .package_id_to_source_file
                        .as_ref()
                        .unwrap()
                        .get(&data_package_id)
                        .cloned();
                    if let Some(package_id_file) = package_id_file {
                        if let Some(file) = &file {
                            state.duplicate_source_files.push(DuplicateSourceFile {
                                file_name: file.file_name.clone(),
                                hash: 0,
                                script_kind: file.script_kind,
                                content_mapper: String::new(),
                                is_content_mapper_failure_stub: false,
                            });
                        }
                        let package_id_file_path =
                            Path(tsox_frontend::ast::mig::m3b_2::path(&package_id_file));
                        state
                            .redirect_targets_map
                            .as_mut()
                            .unwrap()
                            .entry(package_id_file_path.clone())
                            .or_default()
                            .push(normalized_file_path.clone());
                        let redirect_files_by_path =
                            state.redirect_files_by_path.get_or_insert_with(HashMap::new);
                        let index = state.files.len() + redirect_files_by_path.len();
                        redirect_files_by_path.insert(
                            path_key.clone(),
                            RedirectsFile {
                                index,
                                file_name: normalized_file_path.clone(),
                                path: path_key.clone(),
                                target: package_id_file_path.clone(),
                            },
                        );
                        state.files_by_path.insert(path_key.clone(), package_id_file);
                        let data_lowest_depth =
                            data.as_ref().map(|d| *d.lowest_depth.lock().unwrap()).unwrap_or(0);
                        if data_lowest_depth > 0 {
                            state.source_files_found_searching_node_modules.insert(path_key.clone());
                        }
                        continue;
                    } else if let Some(file) = &file {
                        state
                            .package_id_to_source_file
                            .as_mut()
                            .unwrap()
                            .insert(data_package_id, file.clone());
                    }
                }
            }

            let sub_tasks = current.lock().unwrap().sub_tasks.clone();
            if !sub_tasks.is_empty() {
                self.collect_files(loader, &sub_tasks, state);
            }

            let redirected_parse_task = current.lock().unwrap().redirected_parse_task.clone();
            if let Some(redirected) = redirected_parse_task {
                if state.output_file_to_project_reference_source.is_some() {
                    let redirected_path = redirected.lock().unwrap().path.clone();
                    state
                        .output_file_to_project_reference_source
                        .as_mut()
                        .unwrap()
                        .insert(redirected_path, normalized_file_path.clone());
                }
                continue;
            }

            if current.lock().unwrap().is_for_automatic_type_directive {
                let type_resolutions = current.lock().unwrap().type_resolutions_in_file.clone();
                state
                    .type_resolutions_in_file
                    .insert(path_key.clone(), type_resolutions);
                let diagnostics = std::mem::take(&mut current.lock().unwrap().processing_diagnostics);
                if !diagnostics.is_empty() {
                    state.include_processor.processing_diagnostics.extend(diagnostics);
                }
                continue;
            }

            {
                let diagnostics = std::mem::take(&mut current.lock().unwrap().processing_diagnostics);
                if !diagnostics.is_empty() {
                    state.include_processor.processing_diagnostics.extend(diagnostics);
                }
            }

            let file = match file {
                Some(file) => file,
                None => {
                    state.missing_files.push(normalized_file_path.clone());
                    continue;
                }
            };

            let is_lib_file = current.lock().unwrap().lib_file.is_some();
            if is_lib_file {
                state.lib_files.push(file.clone());
                let lib_file = current.lock().unwrap().lib_file.take().unwrap();
                state.lib_files_map.insert(path_key.clone(), lib_file);
            } else {
                state.files.push(file.clone());
            }
            state.files_by_path.insert(path_key.clone(), file.clone());
            state
                .resolved_modules
                .insert(path_key.clone(), current.lock().unwrap().resolutions_in_file.clone());
            state.type_resolutions_in_file.insert(
                path_key.clone(),
                current.lock().unwrap().type_resolutions_in_file.clone(),
            );
            state
                .source_file_meta_datas
                .insert(path_key.clone(), current.lock().unwrap().metadata.clone());

            let jsx_runtime_import_specifier =
                current.lock().unwrap().jsx_runtime_import_specifier.take();
            if let Some(specifier) = jsx_runtime_import_specifier {
                state.jsx_runtime_import_specifiers.insert(path_key.clone(), specifier);
            }
            let import_helpers = current.lock().unwrap().import_helpers_import_specifier.clone();
            if let Some(helpers) = import_helpers {
                state.import_helpers_import_specifiers.insert(path_key.clone(), helpers);
            }
            let data_lowest_depth = data.as_ref().map(|d| *d.lowest_depth.lock().unwrap()).unwrap_or(0);
            if data_lowest_depth > 0 {
                state.source_files_found_searching_node_modules.insert(path_key.clone());
            }
        }
    }

    fn add_include_reason(
        &self,
        include_processor: &mut IncludeProcessor,
        task: &Arc<Mutex<ParseTask>>,
        reason: &Arc<FileIncludeReason>,
    ) { ::tsox_core::fntrace::enter("add_include_reason"); 
        let (redirected, loaded, path) = {
            let t = task.lock().unwrap();
            (t.redirected_parse_task.clone(), t.loaded, t.path.clone())
        };
        if let Some(redirected) = redirected {
            self.add_include_reason(include_processor, &redirected, reason);
        } else if loaded {
            include_processor
                .file_include_reasons
                .entry(path)
                .or_default()
                .push(reason.clone());
        }
    }
}
