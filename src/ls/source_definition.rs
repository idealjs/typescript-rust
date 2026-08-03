//! Source definition (1:1 port of Go's `internal/ls/sourcedefinition.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position};

use super::language_service::LanguageService;
use super::types::LocationLink;

/// Resolver for source-definition navigation.
pub struct SourceDefResolver<'a> {
    pub program: &'a Program,
    pub file_name: String,
}

impl LanguageService {
    /// Provide source definition for a position.
    ///
    /// Mirrors `ProvideSourceDefinition`.
    pub fn provide_source_definition(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<LocationLink> {
        // TODO: requires source-map resolver + module resolution
        Vec::new()
    }

    /// Create a source-definition resolver.
    ///
    /// Mirrors `newSourceDefResolver`.
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

/// Find the containing module specifier for a node.
///
/// Mirrors `findContainingModuleSpecifier`.
pub fn find_containing_module_specifier(
    _node: &Arc<crate::ast::Node>,
) -> Option<Arc<crate::ast::Node>> {
    // TODO: requires AST ancestor traversal
    None
}

/// Get source-definition entry declarations from a source file.
///
/// Mirrors `getSourceDefinitionEntryDeclarations`.
pub fn get_source_definition_entry_declarations(
    _source_file: &Arc<SourceFile>,
) -> Vec<Arc<crate::ast::Node>> {
    // TODO: requires AST analysis
    Vec::new()
}
