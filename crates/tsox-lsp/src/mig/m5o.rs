#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tsox_core::core::compiler_options::ModuleKind;
use tsox_core::core::compiler_options::ResolutionMode;
use tsox_core::tspath::Path;

use crate::ls::autoimport::RegistryCloneHost;
use crate::ls::autoimport_alias_resolver::AliasResolver;
use crate::ls::autoimport_alias_resolver::HasFileName;
use crate::mig::m5n::LspError;
use crate::mig::m5n::LspResult;
use crate::mig::m5n::MessageMarshalError;
use crate::mig::m5n::Server;

mod lsproto {
    pub use crate::lsp::lsproto::*;

    pub use crate::mig::m5n::ErrorCode;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CodeActionKind(pub &'static str);

    #[allow(non_upper_case_globals)]
    impl CodeActionKind {
        pub const QuickFix: CodeActionKind = CodeActionKind("quickfix");
        pub const SourceOrganizeImportsTs: CodeActionKind =
            CodeActionKind("source.organizeImports.ts");
        pub const SourceRemoveUnusedImportsTs: CodeActionKind =
            CodeActionKind("source.removeUnusedImports.ts");
        pub const SourceSortImportsTs: CodeActionKind = CodeActionKind("source.sortImports.ts");
        pub const SourceFixAllTs: CodeActionKind = CodeActionKind("source.fixAll.ts");

        pub fn as_str(&self) -> &'static str { ::tsox_core::fntrace::enter("as_str"); 
            self.0
        }
    }
}

pub const CONTENT_MAPPER_DID_OPEN_REGISTRATION_ID: &str = "content-mapper-did-open";
pub const CONTENT_MAPPER_DID_CHANGE_REGISTRATION_ID: &str = "content-mapper-did-change";
pub const CONTENT_MAPPER_DID_CLOSE_REGISTRATION_ID: &str = "content-mapper-did-close";
pub const CONTENT_MAPPER_DIAGNOSTIC_REGISTRATION_ID: &str = "content-mapper-diagnostic";
pub const CONTENT_MAPPER_HOVER_REGISTRATION_ID: &str = "content-mapper-hover";
pub const CONTENT_MAPPER_SIGNATURE_HELP_REGISTRATION_ID: &str = "content-mapper-signature-help";
pub const CONTENT_MAPPER_DEFINITION_REGISTRATION_ID: &str = "content-mapper-definition";
pub const CONTENT_MAPPER_TYPE_DEFINITION_REGISTRATION_ID: &str = "content-mapper-type-definition";
pub const CONTENT_MAPPER_IMPLEMENTATION_REGISTRATION_ID: &str = "content-mapper-implementation";
pub const CONTENT_MAPPER_REFERENCES_REGISTRATION_ID: &str = "content-mapper-references";
pub const CONTENT_MAPPER_DOCUMENT_HIGHLIGHT_REGISTRATION_ID: &str =
    "content-mapper-document-highlight";
pub const CONTENT_MAPPER_COMPLETION_REGISTRATION_ID: &str = "content-mapper-completion";
pub const CONTENT_MAPPER_RENAME_REGISTRATION_ID: &str = "content-mapper-rename";
pub const CONTENT_MAPPER_SEMANTIC_TOKENS_REGISTRATION_ID: &str = "content-mapper-semantic-tokens";
pub const CONTENT_MAPPER_DOCUMENT_SYMBOL_REGISTRATION_ID: &str = "content-mapper-document-symbol";
pub const CONTENT_MAPPER_FOLDING_RANGE_REGISTRATION_ID: &str = "content-mapper-folding-range";
pub const CONTENT_MAPPER_SELECTION_RANGE_REGISTRATION_ID: &str = "content-mapper-selection-range";
pub const CONTENT_MAPPER_INLAY_HINT_REGISTRATION_ID: &str = "content-mapper-inlay-hint";
pub const CONTENT_MAPPER_CODE_LENS_REGISTRATION_ID: &str = "content-mapper-code-lens";
pub const CONTENT_MAPPER_CODE_ACTION_REGISTRATION_ID: &str = "content-mapper-code-action";
pub const CONTENT_MAPPER_FORMATTING_REGISTRATION_ID: &str = "content-mapper-formatting";
pub const CONTENT_MAPPER_RANGE_FORMATTING_REGISTRATION_ID: &str =
    "content-mapper-range-formatting";
pub const CONTENT_MAPPER_ON_TYPE_FORMATTING_REGISTRATION_ID: &str =
    "content-mapper-on-type-formatting";
pub const CONTENT_MAPPER_LINKED_EDITING_REGISTRATION_ID: &str = "content-mapper-linked-editing";
pub const CONTENT_MAPPER_CALL_HIERARCHY_REGISTRATION_ID: &str = "content-mapper-call-hierarchy";
pub const CONTENT_MAPPER_WILL_RENAME_FILES_REGISTRATION_ID: &str =
    "content-mapper-will-rename-files";

pub fn supported_code_action_kinds() -> Vec<&'static str> { ::tsox_core::fntrace::enter("supported_code_action_kinds"); 
    vec![
        lsproto::CodeActionKind::QuickFix.as_str(),
        lsproto::CodeActionKind::SourceOrganizeImportsTs.as_str(),
        lsproto::CodeActionKind::SourceRemoveUnusedImportsTs.as_str(),
        lsproto::CodeActionKind::SourceSortImportsTs.as_str(),
        lsproto::CodeActionKind::SourceFixAllTs.as_str(),
    ]
}

impl Server {
    pub fn supports_content_mapper_registration(&self, id: &str) -> bool { ::tsox_core::fntrace::enter("supports_content_mapper_registration"); 
        let caps = &self.client_capabilities.text_document;
        match id {
            CONTENT_MAPPER_DID_OPEN_REGISTRATION_ID
            | CONTENT_MAPPER_DID_CHANGE_REGISTRATION_ID
            | CONTENT_MAPPER_DID_CLOSE_REGISTRATION_ID => {
                caps.synchronization.dynamic_registration
            }
            CONTENT_MAPPER_DIAGNOSTIC_REGISTRATION_ID => caps.diagnostic.dynamic_registration,
            CONTENT_MAPPER_HOVER_REGISTRATION_ID => caps.hover.dynamic_registration,
            CONTENT_MAPPER_SIGNATURE_HELP_REGISTRATION_ID => {
                caps.signature_help.dynamic_registration
            }
            CONTENT_MAPPER_DEFINITION_REGISTRATION_ID => caps.definition.dynamic_registration,
            CONTENT_MAPPER_TYPE_DEFINITION_REGISTRATION_ID => {
                caps.type_definition.dynamic_registration
            }
            CONTENT_MAPPER_IMPLEMENTATION_REGISTRATION_ID => {
                caps.implementation.dynamic_registration
            }
            CONTENT_MAPPER_REFERENCES_REGISTRATION_ID => caps.references.dynamic_registration,
            CONTENT_MAPPER_DOCUMENT_HIGHLIGHT_REGISTRATION_ID => {
                caps.document_highlight.dynamic_registration
            }
            CONTENT_MAPPER_COMPLETION_REGISTRATION_ID => caps.completion.dynamic_registration,
            CONTENT_MAPPER_RENAME_REGISTRATION_ID => caps.rename.dynamic_registration,
            CONTENT_MAPPER_SEMANTIC_TOKENS_REGISTRATION_ID => {
                caps.semantic_tokens.dynamic_registration
            }
            CONTENT_MAPPER_DOCUMENT_SYMBOL_REGISTRATION_ID => {
                caps.document_symbol.dynamic_registration
            }
            CONTENT_MAPPER_FOLDING_RANGE_REGISTRATION_ID => {
                caps.folding_range.dynamic_registration
            }
            CONTENT_MAPPER_SELECTION_RANGE_REGISTRATION_ID => {
                caps.selection_range.dynamic_registration
            }
            CONTENT_MAPPER_INLAY_HINT_REGISTRATION_ID => caps.inlay_hint.dynamic_registration,
            CONTENT_MAPPER_CODE_LENS_REGISTRATION_ID => caps.code_lens.dynamic_registration,
            CONTENT_MAPPER_CODE_ACTION_REGISTRATION_ID => caps.code_action.dynamic_registration,
            CONTENT_MAPPER_FORMATTING_REGISTRATION_ID => caps.formatting.dynamic_registration,
            CONTENT_MAPPER_RANGE_FORMATTING_REGISTRATION_ID => {
                caps.range_formatting.dynamic_registration
            }
            CONTENT_MAPPER_ON_TYPE_FORMATTING_REGISTRATION_ID => {
                caps.on_type_formatting.dynamic_registration
            }
            CONTENT_MAPPER_LINKED_EDITING_REGISTRATION_ID => {
                caps.linked_editing_range.dynamic_registration
            }
            CONTENT_MAPPER_CALL_HIERARCHY_REGISTRATION_ID => {
                caps.call_hierarchy.dynamic_registration
            }
            CONTENT_MAPPER_WILL_RENAME_FILES_REGISTRATION_ID => {
                self.client_capabilities.workspace.file_operations.dynamic_registration
                    && self.client_capabilities.workspace.file_operations.will_rename
            }
            _ => false,
        }
    }
}

pub fn value_or_zero<T: Clone + Default>(value: Option<&T>) -> T { ::tsox_core::fntrace::enter("value_or_zero"); 
    match value {
        None => T::default(),
        Some(v) => v.clone(),
    }
}

impl Server {
    pub fn write_loop(&self) -> Result<(), LspError> { ::tsox_core::fntrace::enter("write_loop"); 
        loop {
            let msg = self.outgoing_queue.get();
            let Some(msg) = msg else {
                continue;
            };
            if let Err(err) = self.w.write(&msg) {
                let marshal_err = MessageMarshalError { err };
                if msg.kind == crate::jsonrpc::jsonrpc::MessageKind::Response {
                    let resp = msg.as_response();
                    if resp.id.is_some() && resp.error.is_none() {
                        self.logger.error(&format!(
                            "failed to marshal response for request {:?}: {}",
                            resp.id,
                            marshal_err.error()
                        ));
                        let id = resp.id.clone();
                        self.send_error(id, &LspError::new(lsproto::ErrorCode::InternalError, marshal_err.err.message))?;
                        continue;
                    }
                }
                return Err(LspError::new(
                    lsproto::ErrorCode::InternalError,
                    format!("failed to write message: {}", marshal_err.error()),
                ));
            }
        }
    }
}

impl Server {
    pub fn remove_api_session(&self, id: &str) { ::tsox_core::fntrace::enter("remove_api_session"); 
        self.api_sessions.lock().unwrap().remove(id);
    }
}

pub fn new_alias_resolver_m5o(
    root_files: Vec<Arc<tsox_frontend::ast::SourceFile>>,
    symlinks: HashMap<Path, crate::ls::autoimport_util::PathAndFileName>,
    host: Box<dyn RegistryCloneHost>,
    module_resolver: Option<Arc<tsox_tsoptions::module::Resolver>>,
    to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
    on_failed_ambient_module_lookup: Box<dyn Fn(&dyn HasFileName, &str) + Send + Sync>,
) -> AliasResolver { ::tsox_core::fntrace::enter("new_alias_resolver_m5o"); 
    AliasResolver::new(root_files, symlinks, host, module_resolver, to_path, on_failed_ambient_module_lookup)
}

impl AliasResolver {
    pub fn common_source_directory(&self) -> String { ::tsox_core::fntrace::enter("common_source_directory"); 
        unimplemented!()
    }

    pub fn content_mapper_extensions(&self) -> Vec<String> { ::tsox_core::fntrace::enter("content_mapper_extensions"); 
        Vec::new()
    }

    pub fn file_exists(&self, _file_name: &str) -> bool { ::tsox_core::fntrace::enter("file_exists"); 
        unimplemented!()
    }

    pub fn get_emit_syntax_for_usage_location(
        &self,
        _source_file: &dyn HasFileName,
        _usage_location: &Arc<tsox_frontend::ast::Node>,
    ) -> ResolutionMode { ::tsox_core::fntrace::enter("get_emit_syntax_for_usage_location"); 
        ModuleKind::ESNext
    }

    pub fn get_global_typings_cache_location(&self) -> String { ::tsox_core::fntrace::enter("get_global_typings_cache_location"); 
        unimplemented!()
    }

    pub fn get_implied_node_format_for_emit(&self, _source_file: &dyn HasFileName) -> ModuleKind { ::tsox_core::fntrace::enter("get_implied_node_format_for_emit"); 
        ModuleKind::ESNext
    }

    pub fn get_import_helpers_import_specifier(&self, _path: &Path) -> Option<tsox_frontend::ast::Node> { ::tsox_core::fntrace::enter("get_import_helpers_import_specifier"); 
        unimplemented!()
    }

    pub fn get_jsx_runtime_import_specifier(
        &self,
        _path: &Path,
    ) -> (String, Option<tsox_frontend::ast::Node>) { ::tsox_core::fntrace::enter("get_jsx_runtime_import_specifier"); 
        unimplemented!()
    }

    pub fn get_mode_for_usage_location(
        &self,
        _file: &dyn HasFileName,
        _module_specifier: &Arc<tsox_frontend::ast::Node>,
    ) -> ResolutionMode { ::tsox_core::fntrace::enter("get_mode_for_usage_location"); 
        ModuleKind::ESNext
    }

    pub fn get_nearest_ancestor_directory_with_package_json(&self, _dirname: &str) -> String { ::tsox_core::fntrace::enter("get_nearest_ancestor_directory_with_package_json"); 
        unimplemented!()
    }

    pub fn get_package_json_info(
        &self,
        _pkg_json_path: &str,
    ) -> Option<tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry> { ::tsox_core::fntrace::enter("get_package_json_info"); 
        unimplemented!()
    }

    pub fn get_project_reference_from_output_dts(
        &self,
        _path: &Path,
    ) -> Option<tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference> { ::tsox_core::fntrace::enter("get_project_reference_from_output_dts"); 
        unimplemented!()
    }

    pub fn get_project_reference_from_source(
        &self,
        _path: &Path,
    ) -> Option<tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference> { ::tsox_core::fntrace::enter("get_project_reference_from_source"); 
        unimplemented!()
    }

    pub fn get_redirect_for_resolution(
        &self,
        _file: &dyn HasFileName,
    ) -> Option<tsox_tsoptions::tsoptions::ParsedCommandLine> { ::tsox_core::fntrace::enter("get_redirect_for_resolution"); 
        unimplemented!()
    }

    pub fn get_redirect_targets(&self, _path: &Path) -> Vec<String> { ::tsox_core::fntrace::enter("get_redirect_targets"); 
        unimplemented!()
    }

    pub fn get_resolved_module_from_module_specifier(
        &self,
        _file: &dyn HasFileName,
        _module_specifier: &Arc<tsox_frontend::ast::Node>,
    ) -> Option<Arc<tsox_tsoptions::module::ResolvedModule>> { ::tsox_core::fntrace::enter("get_resolved_module_from_module_specifier"); 
        unimplemented!()
    }

    pub fn get_resolved_modules(
        &self,
    ) -> Option<HashMap<Path, crate::ls::autoimport::ModeAwareCache<Arc<tsox_tsoptions::module::ResolvedModule>>>> { ::tsox_core::fntrace::enter("get_resolved_modules"); 
        None
    }

    pub fn get_source_file_meta_data(
        &self,
        _path: &Path,
    ) -> tsox_frontend::ast::mig::x4ast::SourceFileMetaData { ::tsox_core::fntrace::enter("get_source_file_meta_data"); 
        unimplemented!()
    }

    pub fn get_source_of_project_reference_if_output_included(
        &self,
        _file: &dyn HasFileName,
    ) -> String { ::tsox_core::fntrace::enter("get_source_of_project_reference_if_output_included"); 
        unimplemented!()
    }

    pub fn get_symlink_cache(&self) -> tsox_core::symlinks::KnownSymlinks { ::tsox_core::fntrace::enter("get_symlink_cache"); 
        unimplemented!()
    }

    pub fn is_source_from_project_reference(&self, _path: &Path) -> bool { ::tsox_core::fntrace::enter("is_source_from_project_reference"); 
        unimplemented!()
    }

    pub fn source_file_may_be_emitted(
        &self,
        _source_file: &tsox_frontend::ast::SourceFile,
        _force_dts_emit: bool,
    ) -> bool { ::tsox_core::fntrace::enter("source_file_may_be_emitted"); 
        unimplemented!()
    }
}
