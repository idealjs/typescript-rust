//! Document / workspace symbols (1:1 port of Go's `internal/ls/symbols.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::compiler::Program;
use crate::core::text::TextRange;
use crate::ls::lsconv::converters::Converters;
use crate::ls::lsutil::UserPreferences;
use crate::lsp::lsproto::lsp::DocumentUri;

use super::language_service::LanguageService;
use super::types::{DocumentSymbol, SymbolInformation, SymbolKind};

impl LanguageService {
    /// Provide document symbols for a file.
    ///
    /// Mirrors `ProvideDocumentSymbols`.
    pub fn provide_document_symbols(&self, _document_uri: &DocumentUri) -> Vec<DocumentSymbol> {
        // TODO: requires AST traversal + hierarchical symbol building
        Vec::new()
    }

    /// Get document symbols for children of a node.
    ///
    /// Mirrors `getDocumentSymbolsForChildren`.
    pub fn get_document_symbols_for_children(
        &self,
        _node: &Arc<Node>,
        _file: &Arc<SourceFile>,
    ) -> Vec<DocumentSymbol> {
        // TODO: requires full AST visit
        Vec::new()
    }

    /// Create a new `DocumentSymbol` for a node.
    ///
    /// Mirrors `newDocumentSymbol`.
    pub fn new_document_symbol(
        &self,
        _node: &Arc<Node>,
        _name: Option<&Arc<Node>>,
        _children: Vec<DocumentSymbol>,
    ) -> Option<DocumentSymbol> {
        // TODO: requires scanner + converters
        None
    }
}

/// Provide workspace symbols across multiple programs.
///
/// Mirrors `ProvideWorkspaceSymbols`.
pub fn provide_workspace_symbols(
    _programs: &[Program],
    _converters: &Converters,
    _preferences: &UserPreferences,
    _query: &str,
) -> Vec<SymbolInformation> {
    // TODO: requires declaration-map traversal + match scoring
    Vec::new()
}

/// Get the symbol kind from an AST node.
///
/// Mirrors `getSymbolKindFromNode`.
pub fn get_symbol_kind_from_node(node: &Arc<Node>) -> SymbolKind {
    use crate::ast::SyntaxKind;
    match node.kind {
        SyntaxKind::SourceFile => SymbolKind::File,
        SyntaxKind::ModuleDeclaration => SymbolKind::Namespace,
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => SymbolKind::Class,
        SyntaxKind::InterfaceDeclaration => SymbolKind::Interface,
        SyntaxKind::EnumDeclaration => SymbolKind::Enum,
        SyntaxKind::VariableDeclaration => SymbolKind::Variable,
        SyntaxKind::ArrowFunction
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression => SymbolKind::Function,
        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => SymbolKind::Property,
        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => SymbolKind::Method,
        SyntaxKind::Constructor | SyntaxKind::ClassStaticBlockDeclaration => {
            SymbolKind::Constructor
        }
        SyntaxKind::TypeParameter => SymbolKind::TypeParameter,
        SyntaxKind::EnumMember => SymbolKind::EnumMember,
        _ => SymbolKind::Variable,
    }
}

/// Declaration info used for workspace symbol search.
pub struct DeclarationInfo {
    pub name: String,
    pub declaration: Arc<Node>,
    pub match_score: i32,
}

/// Compute a match score for a string against a pattern.
///
/// Mirrors `getMatchScore`.
pub fn get_match_score(s: &str, pattern: &str) -> i32 {
    let mut score = 0i32;
    let mut remaining = s;
    for p in pattern.chars() {
        let exact = p.is_uppercase();
        loop {
            match remaining.chars().next() {
                None => return -1,
                Some(c) => {
                    remaining = &remaining[c.len_utf8()..];
                    if exact && c == p || !exact && c.eq_ignore_ascii_case(&p) {
                        break;
                    }
                    score += 1;
                }
            }
        }
    }
    score
}

/// Should this file be excluded from workspace symbol search?
///
/// Mirrors `shouldExcludeFile`.
pub fn should_exclude_file(
    _file: &Arc<SourceFile>,
    _program: &Program,
    _exclude_library_symbols: bool,
) -> bool {
    // TODO: requires isInsideNodeModules + isLibFile
    false
}

/// Max symbol name length before truncation.
pub const MAX_LENGTH: usize = 150;
