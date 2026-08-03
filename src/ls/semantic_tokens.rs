//! Semantic tokens provider (1:1 port of Go's `internal/ls/semantictokens.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, SourceFile, Symbol, SymbolFlags, SyntaxKind};
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
    ///
    /// 1. Walk the AST.
    /// 2. Classify each node into a `SemanticTokenType` (keyword, variable,
    ///    function, class, etc.).
    /// 3. Return as a delta-encoded token array.
    pub fn provide_semantic_tokens(&self, document_uri: &DocumentUri) -> Option<SemanticTokens> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let checker = program.build_checker();
        let tokens = self.collect_semantic_tokens_in_range(
            &checker,
            &source_file,
            &program,
            0,
            source_file.text.len(),
        );
        let data = encode_tokens(&tokens, &source_file.line_map);
        Some(SemanticTokens { data })
    }

    /// Provide semantic tokens for a range.
    ///
    /// Mirrors `ProvideSemanticTokensRange`.
    pub fn provide_semantic_tokens_range(
        &self,
        document_uri: &DocumentUri,
        rng: Range,
    ) -> Option<SemanticTokens> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;
        let start = lsp_position_to_offset(line_map, &rng.start);
        let end = lsp_position_to_offset(line_map, &rng.end);
        let checker = program.build_checker();
        let tokens =
            self.collect_semantic_tokens_in_range(&checker, &source_file, &program, start, end);
        let data = encode_tokens(&tokens, line_map);
        Some(SemanticTokens { data })
    }

    /// Collect semantic tokens in a range.
    ///
    /// Mirrors `collectSemanticTokensInRange`.
    pub fn collect_semantic_tokens_in_range(
        &self,
        checker: &Checker,
        file: &Arc<SourceFile>,
        _program: &Program,
        span_start: usize,
        span_end: usize,
    ) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        collect_tokens(checker, &file.node, span_start, span_end, &mut tokens);
        // Sort by position so delta encoding is correct.
        tokens.sort_by_key(|t| t.pos);
        tokens
    }
}

/// Classify a symbol into a token type and whether it is a declaration.
///
/// Mirrors `classifySymbol`.
pub fn classify_symbol(symbol: &Symbol, _meaning: u32) -> (u32, bool) {
    let flags = symbol.flags;
    if flags.contains(SymbolFlags::TypeParameter) {
        return (token_type::TYPE_PARAMETER, true);
    }
    if flags.contains(SymbolFlags::Class) {
        return (token_type::CLASS, true);
    }
    if flags.contains(SymbolFlags::Interface) {
        return (token_type::INTERFACE, true);
    }
    if flags.contains(SymbolFlags::TypeAlias) {
        return (token_type::TYPE, true);
    }
    if flags.contains(SymbolFlags::ENUM) {
        return (token_type::ENUM, true);
    }
    if flags.contains(SymbolFlags::EnumMember) {
        return (token_type::ENUM_MEMBER, true);
    }
    if flags.contains(SymbolFlags::Function) {
        return (token_type::FUNCTION, true);
    }
    if flags.contains(SymbolFlags::Method) {
        return (token_type::METHOD, true);
    }
    if flags.contains(SymbolFlags::Constructor) {
        return (token_type::FUNCTION, true);
    }
    if flags.intersects(SymbolFlags::GetAccessor | SymbolFlags::SetAccessor) {
        return (token_type::PROPERTY, true);
    }
    if flags.contains(SymbolFlags::Property) {
        return (token_type::PROPERTY, true);
    }
    if flags.intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule) {
        return (token_type::NAMESPACE, true);
    }
    if flags.contains(SymbolFlags::VARIABLE) {
        return (token_type::VARIABLE, true);
    }
    if flags.contains(SymbolFlags::Alias) {
        return (token_type::VARIABLE, false);
    }
    (token_type::INVALID, false)
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

/// Walk the AST and collect semantic tokens within `[span_start, span_end)`.
fn collect_tokens(
    checker: &Checker,
    node: &Arc<Node>,
    span_start: usize,
    span_end: usize,
    tokens: &mut Vec<SemanticToken>,
) {
    // Skip nodes entirely outside the span.
    if node.end() <= span_start || node.pos() >= span_end {
        return;
    }

    // Try to classify the current node.
    if let Some(token) = classify_node_token(checker, node) {
        if token.pos >= span_start && token.end <= span_end {
            tokens.push(token);
        }
    }

    // Recurse into children.
    for_each_child(node, |child| {
        collect_tokens(checker, child, span_start, span_end, tokens);
        false
    });
}

/// Classify a single AST node into a semantic token, if applicable.
fn classify_node_token(checker: &Checker, node: &Arc<Node>) -> Option<SemanticToken> {
    let kind = node.kind;

    // Literal / punctuation tokens.
    match kind {
        SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral => {
            return Some(SemanticToken {
                token_type: token_type::NUMBER,
                token_modifier: 0,
                pos: node.pos(),
                end: node.end(),
            });
        }
        SyntaxKind::StringLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateHead
        | SyntaxKind::TemplateMiddle
        | SyntaxKind::TemplateTail => {
            return Some(SemanticToken {
                token_type: token_type::STRING,
                token_modifier: 0,
                pos: node.pos(),
                end: node.end(),
            });
        }
        SyntaxKind::RegularExpressionLiteral => {
            return Some(SemanticToken {
                token_type: token_type::REGEXP,
                token_modifier: 0,
                pos: node.pos(),
                end: node.end(),
            });
        }
        _ => {}
    }

    // Keyword tokens.
    if is_keyword_kind(kind) {
        return Some(SemanticToken {
            token_type: token_type::KEYWORD,
            token_modifier: 0,
            pos: node.pos(),
            end: node.end(),
        });
    }

    // Identifier nodes: classify via the resolved symbol.
    if kind == SyntaxKind::Identifier {
        // Declaration name tokens are classified by the declaration kind.
        if let Some(parent) = node.parent.as_ref() {
            let decl_type = token_from_declaration_mapping(parent.kind);
            if decl_type != token_type::INVALID {
                let mut modifier = 0u32;
                // Mark as a declaration if the identifier is the name child.
                if is_name_of_declaration(node, parent) {
                    modifier |= token_modifier::DECLARATION | token_modifier::DEFINITION;
                }
                return Some(SemanticToken {
                    token_type: decl_type,
                    token_modifier: modifier,
                    pos: node.pos(),
                    end: node.end(),
                });
            }
        }

        // Otherwise classify via the checker-resolved symbol.
        if let Some(symbol) = checker.get_symbol_at_location(node) {
            let (token_type_val, is_declaration) = classify_symbol(&symbol, 0);
            if token_type_val != token_type::INVALID {
                let mut modifier = 0u32;
                if is_declaration {
                    modifier |= token_modifier::DECLARATION;
                }
                return Some(SemanticToken {
                    token_type: token_type_val,
                    token_modifier: modifier,
                    pos: node.pos(),
                    end: node.end(),
                });
            }
        }

        // Unresolved identifier — treat as a plain variable.
        return Some(SemanticToken {
            token_type: token_type::VARIABLE,
            token_modifier: 0,
            pos: node.pos(),
            end: node.end(),
        });
    }

    None
}

/// Whether a syntax kind is a reserved keyword.
fn is_keyword_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::BreakKeyword as i16)
        && (kind as i16) <= (SyntaxKind::DeferKeyword as i16)
}

/// Whether `name` is the name child of a `declaration` node.
fn is_name_of_declaration(name: &Arc<Node>, declaration: &Arc<Node>) -> bool {
    declaration
        .name()
        .map(|n| Arc::ptr_eq(n, name))
        .unwrap_or(false)
}

/// Encode a sorted list of semantic tokens into the LSP delta-encoded array.
///
/// Each token contributes 5 u32s: `[deltaLine, deltaStart, length,
/// tokenType, tokenModifiers]`.
fn encode_tokens(tokens: &[SemanticToken], line_map: &LineMap) -> Vec<u32> {
    let mut data = Vec::with_capacity(tokens.len() * 5);
    let mut prev_line = 0u32;
    let mut prev_char = 0u32;

    for token in tokens {
        let start = offset_to_line_char(line_map, token.pos);
        let end = offset_to_line_char(line_map, token.end);
        let length = (token.end - token.pos) as u32;

        let delta_line = start.0 - prev_line;
        let delta_char = if delta_line == 0 {
            start.1 - prev_char
        } else {
            start.1
        };

        data.push(delta_line);
        data.push(delta_char);
        data.push(length);
        data.push(token.token_type);
        data.push(token.token_modifier);

        prev_line = start.0;
        prev_char = start.1;
        let _ = end;
    }

    data
}

/// Convert a byte offset to `(line, character)`.
fn offset_to_line_char(line_map: &LineMap, offset: usize) -> (u32, u32) {
    let line = match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    (line as u32, (offset.saturating_sub(line_start)) as u32)
}

/// Convert an LSP `Position` to a byte offset within a line map.
fn lsp_position_to_offset(
    line_map: &LineMap,
    position: &crate::lsp::lsproto::lsp::Position,
) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}
