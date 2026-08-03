//! Language-service public API (1:1 port of Go's `internal/ls/api.go`).
//!
//! Provides `GetSymbolAtPosition`, `GetSymbolAtLocation`, and
//! `GetTypeOfSymbol`. These depend on the checker and AST navigation
//! which are not fully wired; method bodies are stubbed.

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol};

use super::language_service::LanguageService;

/// Errors returned by LS API methods.
#[derive(Debug)]
pub enum LsError {
    NoSourceFile(String),
    NoTokenAtPosition(String),
}

impl std::fmt::Display for LsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LsError::NoSourceFile(name) => write!(f, "source file not found: {name}"),
            LsError::NoTokenAtPosition(loc) => {
                write!(f, "no token found at position: {loc}")
            }
        }
    }
}

impl std::error::Error for LsError {}

impl LanguageService {
    /// Get the symbol at a given file position.
    ///
    /// Mirrors `GetSymbolAtPosition`.
    pub fn get_symbol_at_position(
        &self,
        _file_name: &str,
        _position: usize,
    ) -> Result<Option<Arc<Symbol>>, LsError> {
        // TODO: requires astnav::get_token_at_position and checker
        let (_program, file) = self.try_get_program_and_file(_file_name);
        let file = file.ok_or_else(|| LsError::NoSourceFile(_file_name.to_string()))?;
        let _ = file;
        // TODO: astnav::get_token_at_position(file, position)
        // TODO: checker.get_symbol_at_location(node)
        Ok(None)
    }

    /// Get the symbol at a given AST node location.
    ///
    /// Mirrors `GetSymbolAtLocation`.
    pub fn get_symbol_at_location(&self, _node: &Arc<Node>) -> Option<Arc<Symbol>> {
        // TODO: requires program.GetTypeCheckerForFile and checker
        None
    }

    /// Get the type of a symbol.
    ///
    /// Mirrors `GetTypeOfSymbol`.
    pub fn get_type_of_symbol(&self, _symbol: &Arc<Symbol>) -> Option<Arc<crate::checker::Type>> {
        // TODO: requires checker.GetTypeOfSymbolAtLocation
        None
    }
}

/// Helper: get the source file of a node (used by feature providers).
pub fn get_source_file_of_node(_node: &Arc<Node>) -> Option<Arc<SourceFile>> {
    // TODO: requires ast.GetSourceFileOfNode
    None
}
