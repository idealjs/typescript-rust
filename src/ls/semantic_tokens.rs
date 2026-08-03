//! Semantic tokens provider (1:1 port of Go's `internal/ls/semantictokens.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, Symbol};
use crate::checker::Checker;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Range};

use super::language_service::LanguageService;
use super::types::{SemanticTokens, SemanticTokensClientCapabilities};

/// Token type index constants.
pub mod token_type {
    pub const NAMESPACE: u32 = 0;
    pub const CLASS: u32 = 1;
    pub const ENUM: u32 = 2;
    pub const INTERFACE: u32 = 3;
    pub const STRUCT: u32 = 4;
    pub const TYPE_PARAMETER: u32 = 5;
    pub const TYPE: u32 = 6;
    pub const PARAMETER: u32 = 7;
    pub const VARIABLE: u32 = 8;
    pub const PROPERTY: u32 = 9;
    pub const ENUM_MEMBER: u32 = 10;
    pub const DECORATOR: u32 = 11;
    pub const EVENT: u32 = 12;
    pub const FUNCTION: u32 = 13;
    pub const METHOD: u32 = 14;
    pub const MACRO: u32 = 15;
    pub const LABEL: u32 = 16;
    pub const COMMENT: u32 = 17;
    pub const STRING: u32 = 18;
    pub const KEYWORD: u32 = 19;
    pub const NUMBER: u32 = 20;
    pub const REGEXP: u32 = 21;
    pub const OPERATOR: u32 = 22;
    pub const INVALID: u32 = u32::MAX;
}

/// Token modifier bit flags.
pub mod token_modifier {
    pub const DECLARATION: u32 = 1 << 0;
    pub const DEFINITION: u32 = 1 << 1;
    pub const READONLY: u32 = 1 << 2;
    pub const STATIC: u32 = 1 << 3;
    pub const DEPRECATED: u32 = 1 << 4;
    pub const ABSTRACT: u32 = 1 << 5;
    pub const ASYNC: u32 = 1 << 6;
    pub const MODIFICATION: u32 = 1 << 7;
    pub const DOCUMENTATION: u32 = 1 << 8;
    pub const DEFAULT_LIBRARY: u32 = 1 << 9;
    pub const LOCAL: u32 = 1 << 10;
}

/// A collected semantic token.
pub struct SemanticToken {
    pub token_type: u32,
    pub token_modifier: u32,
    pub pos: usize,
    pub end: usize,
}

impl LanguageService {
    /// Provide semantic tokens for a document.
    ///
    /// Mirrors `ProvideSemanticTokens`.
    pub fn provide_semantic_tokens(&self, _document_uri: &DocumentUri) -> Option<SemanticTokens> {
        // TODO: requires checker + AST traversal
        None
    }

    /// Provide semantic tokens for a range.
    ///
    /// Mirrors `ProvideSemanticTokensRange`.
    pub fn provide_semantic_tokens_range(
        &self,
        _document_uri: &DocumentUri,
        _rng: Range,
    ) -> Option<SemanticTokens> {
        // TODO: requires checker + AST traversal
        None
    }

    /// Collect semantic tokens in a range.
    ///
    /// Mirrors `collectSemanticTokensInRange`.
    pub fn collect_semantic_tokens_in_range(
        &self,
        _checker: &Checker,
        _file: &Arc<crate::ast::SourceFile>,
        _program: &Program,
        _span_start: usize,
        _span_end: usize,
    ) -> Vec<SemanticToken> {
        // TODO: requires full AST visit with checker
        Vec::new()
    }
}

/// Classify a symbol into a token type.
///
/// Mirrors `classifySymbol`.
pub fn classify_symbol(_symbol: &Symbol, _meaning: u32) -> (u32, bool) {
    // TODO: requires SymbolFlags inspection
    (0, false)
}

/// Map a declaration kind to a token type.
///
/// Mirrors `tokenFromDeclarationMapping`.
pub fn token_from_declaration_mapping(kind: crate::ast::SyntaxKind) -> u32 {
    use crate::ast::SyntaxKind;
    match kind {
        SyntaxKind::VariableDeclaration => token_type::VARIABLE,
        SyntaxKind::Parameter => token_type::PARAMETER,
        SyntaxKind::PropertyDeclaration => token_type::PROPERTY,
        SyntaxKind::ModuleDeclaration => token_type::NAMESPACE,
        SyntaxKind::EnumDeclaration => token_type::ENUM,
        SyntaxKind::EnumMember => token_type::ENUM_MEMBER,
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => token_type::CLASS,
        SyntaxKind::MethodDeclaration => token_type::METHOD,
        SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression => token_type::FUNCTION,
        SyntaxKind::MethodSignature => token_type::METHOD,
        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => token_type::PROPERTY,
        SyntaxKind::PropertySignature => token_type::PROPERTY,
        SyntaxKind::InterfaceDeclaration => token_type::INTERFACE,
        SyntaxKind::TypeAliasDeclaration => token_type::TYPE,
        SyntaxKind::TypeParameter => token_type::TYPE_PARAMETER,
        SyntaxKind::PropertyAssignment | SyntaxKind::ShorthandPropertyAssignment => {
            token_type::PROPERTY
        }
        _ => token_type::INVALID,
    }
}
