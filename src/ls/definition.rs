//! Go-to-definition provider (1:1 port of Go's `internal/ls/definition.go`).
//!
//! Provides `ProvideDefinition`, `ProvideTypeDefinition`, and the supporting
//! declaration-resolution helpers. Depends on checker and AST navigation
//! which are not fully wired; method bodies are stubbed.

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::checker::Checker;
use crate::core::text::TextRange;
use crate::lsp::lsproto::lsp::{DocumentUri, Location, Position, Range};

use super::language_service::{FileRange, LanguageService};
use super::types::LocationLink;

/// A reference info (triple-slash or module reference).
pub struct RefInfo {
    pub file: Option<Arc<SourceFile>>,
    pub file_name: String,
}

impl LanguageService {
    /// Provide go-to-definition for a position.
    ///
    /// Mirrors `ProvideDefinition`.
    pub fn provide_definition(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<LocationLink> {
        // TODO: requires astnav, checker, scanner
        Vec::new()
    }

    /// Provide go-to-type-definition for a position.
    ///
    /// Mirrors `ProvideTypeDefinition`.
    pub fn provide_type_definition(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<LocationLink> {
        // TODO: requires astnav, checker
        Vec::new()
    }

    /// Create definition locations from declarations.
    ///
    /// Mirrors `createDefinitionLocations`.
    pub fn create_definition_locations(
        &self,
        _origin_selection_range: Range,
        _client_supports_link: bool,
        _declarations: &[Arc<Node>],
        _reference: Option<&RefInfo>,
    ) -> Vec<LocationLink> {
        // TODO: requires AST node range computation
        Vec::new()
    }

    /// Create a single location from a file and text range.
    ///
    /// Mirrors `createLocationFromFileAndRange`.
    pub fn create_location_from_file_and_range(
        &self,
        _file: &Arc<SourceFile>,
        _text_range: TextRange,
    ) -> Location {
        // TODO: requires converters
        Location::default()
    }

    /// Create an LSP range from a node.
    pub fn create_lsp_range_from_node(&self, _node: &Arc<Node>, _file: &Arc<SourceFile>) -> Range {
        // TODO: requires scanner.GetTokenPosOfNode + converters
        Range::default()
    }
}

/// Get declarations from a location (identifier node).
///
/// Mirrors `getDeclarationsFromLocation`.
pub fn get_declarations_from_location(_checker: &Checker, _node: &Arc<Node>) -> Vec<Arc<Node>> {
    // TODO: requires checker.GetSymbolAtLocation + symbol.Declarations
    Vec::new()
}

/// Try to get the signature declaration for a call-like node.
///
/// Mirrors `tryGetSignatureDeclaration`.
pub fn try_get_signature_declaration(_checker: &Checker, _node: &Arc<Node>) -> Option<Arc<Node>> {
    // TODO: requires checker.GetResolvedSignature
    None
}

/// Get declarations from a type.
///
/// Mirrors `getDeclarationsFromType`.
pub fn get_declarations_from_type(_type: &crate::checker::Type) -> Vec<Arc<Node>> {
    // TODO: requires checker type introspection
    Vec::new()
}
