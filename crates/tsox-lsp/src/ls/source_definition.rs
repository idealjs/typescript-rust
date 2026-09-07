#![allow(dead_code)]

use std::sync::Arc;

use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp::Position;
use tsox_compile::compiler::Program;
use tsox_frontend::ast::SourceFile;

use super::language_service::LanguageService;
use super::types::LocationLink;

pub struct SourceDefResolver<'a> {
    pub program: &'a Program,
    pub file_name: String,
}

impl LanguageService {
    pub fn provide_source_definition(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<LocationLink> {
        Vec::new()
    }

    pub fn new_source_def_resolver<'a>(
        &self,
        _program: &'a Program,
        _file_name: &str,
    ) -> SourceDefResolver<'a> {
        SourceDefResolver {
            program: _program,
            file_name: _file_name.to_string(),
        }
    }
}

pub fn find_containing_module_specifier(
    _node: &Arc<tsox_frontend::ast::Node>,
) -> Option<Arc<tsox_frontend::ast::Node>> {
    None
}

pub fn get_source_definition_entry_declarations(
    _source_file: &Arc<SourceFile>,
) -> Vec<Arc<tsox_frontend::ast::Node>> {
    Vec::new()
}
