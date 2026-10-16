#![allow(dead_code)]

use std::sync::Arc;

use crate::lsp::lsproto_lsp::DocumentUri;
use tsox_frontend::ast::Node;

use super::cross_project::CrossProjectOrchestrator;
use super::language_service::LanguageService;
use super::types::CodeLens;

impl LanguageService {
    pub fn provide_code_lenses(&self, _document_uri: &DocumentUri) -> Vec<CodeLens> { ::tsox_core::fntrace::enter("provide_code_lenses"); 
        Vec::new()
    }

    pub fn resolve_code_lens(
        &self,
        _code_lens: &CodeLens,
        _show_locations_command_name: Option<&str>,
        _orchestrator: Option<&dyn CrossProjectOrchestrator>,
    ) -> CodeLens { ::tsox_core::fntrace::enter("resolve_code_lens"); 
        CodeLens::default()
    }

    pub fn new_code_lens_for_node(
        &self,
        _document_uri: &DocumentUri,
        _file: &Arc<tsox_frontend::ast::SourceFile>,
        _node: &Arc<Node>,
        _kind: &str,
    ) -> CodeLens { ::tsox_core::fntrace::enter("new_code_lens_for_node"); 
        CodeLens::default()
    }
}

pub fn is_valid_reference_lens_node(_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_valid_reference_lens_node"); 
    false
}

pub fn is_valid_implementations_code_lens_node(_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_valid_implementations_code_lens_node"); 
    false
}
