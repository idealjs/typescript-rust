//! Additional LSP protocol types used by the language service.
//!
//! These types complement the subset already defined in
//! `crate::lsp::lsproto::lsp`. They mirror the corresponding Go
//! `lsproto.*` structs used by the language-service feature providers.

#![allow(dead_code)]

use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range, TextEdit};
use serde::{Deserialize, Serialize};

/// A folding range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoldingRange {
    pub start_line: u32,
    pub start_character: Option<u32>,
    pub end_line: u32,
    pub end_character: Option<u32>,
    pub kind: Option<String>,
    pub collapsed_text: Option<String>,
}

/// A selection range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionRange {
    pub range: Range,
    pub parent: Option<Box<SelectionRange>>,
}

/// A document highlight.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentHighlight {
    pub range: Range,
    pub kind: Option<DocumentHighlightKind>,
}

/// Document highlight kind.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentHighlightKind {
    #[default]
    Text,
    Read,
    Write,
}

/// Semantic tokens data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticTokens {
    pub data: Vec<u32>,
}

/// Semantic token type names.
pub mod semantic_token_type {
    pub const NAMESPACE: &str = "namespace";
    pub const CLASS: &str = "class";
    pub const ENUM: &str = "enum";
    pub const INTERFACE: &str = "interface";
    pub const STRUCT: &str = "struct";
    pub const TYPE_PARAMETER: &str = "typeParameter";
    pub const TYPE: &str = "type";
    pub const PARAMETER: &str = "parameter";
    pub const VARIABLE: &str = "variable";
    pub const PROPERTY: &str = "property";
    pub const ENUM_MEMBER: &str = "enumMember";
    pub const DECORATOR: &str = "decorator";
    pub const EVENT: &str = "event";
    pub const FUNCTION: &str = "function";
    pub const METHOD: &str = "method";
    pub const MACRO: &str = "macro";
    pub const LABEL: &str = "label";
    pub const COMMENT: &str = "comment";
    pub const STRING: &str = "string";
    pub const KEYWORD: &str = "keyword";
    pub const NUMBER: &str = "number";
    pub const REGEXP: &str = "regexp";
    pub const OPERATOR: &str = "operator";
}

/// Semantic token modifier names.
pub mod semantic_token_modifier {
    pub const DECLARATION: &str = "declaration";
    pub const DEFINITION: &str = "definition";
    pub const READONLY: &str = "readonly";
    pub const STATIC: &str = "static";
    pub const DEPRECATED: &str = "deprecated";
    pub const ABSTRACT: &str = "abstract";
    pub const ASYNC: &str = "async";
    pub const MODIFICATION: &str = "modification";
    pub const DOCUMENTATION: &str = "documentation";
    pub const DEFAULT_LIBRARY: &str = "defaultLibrary";
    pub const LOCAL: &str = "local";
}

/// A LocationLink provides an origin and target range for navigation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationLink {
    pub origin_selection_range: Option<Range>,
    pub target_uri: DocumentUri,
    pub target_range: Range,
    pub target_selection_range: Range,
}

/// Hover information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hover {
    pub contents: HoverContent,
    pub range: Option<Range>,
    pub can_increase_verbosity: Option<bool>,
}

/// Hover contents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HoverContent {
    pub markup_content: Option<crate::lsp::lsproto::lsp::MarkupContent>,
    pub string: Option<String>,
}

/// A document symbol.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentSymbol {
    pub name: String,
    pub detail: Option<String>,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range,
    pub children: Option<Vec<DocumentSymbol>>,
    pub tags: Option<Vec<i32>>,
    pub deprecated: Option<bool>,
}

/// Symbol kind values (LSP).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    #[default]
    File,
    Module,
    Namespace,
    Package,
    Class,
    Method,
    Property,
    Field,
    Constructor,
    Enum,
    Interface,
    Function,
    Variable,
    Constant,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Key,
    Null,
    EnumMember,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl SymbolKind {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// A symbol information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: SymbolKind,
    pub location: crate::lsp::lsproto::lsp::Location,
    pub container_name: Option<String>,
    pub tags: Option<Vec<i32>>,
    pub deprecated: Option<bool>,
}

/// An inlay hint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InlayHint {
    pub position: Position,
    pub label: InlayHintLabel,
    pub kind: Option<i32>,
    pub text_edits: Option<Vec<TextEdit>>,
    pub padding_left: Option<bool>,
    pub padding_right: Option<bool>,
}

/// Inlay hint label.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InlayHintLabel {
    pub string: Option<String>,
}

/// A code lens.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeLens {
    pub range: Range,
    pub command: Option<CodeLensCommand>,
    pub data: Option<CodeLensData>,
}

/// Code lens command.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeLensCommand {
    pub title: String,
    pub command: String,
}

/// Code lens data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeLensData {
    pub uri: DocumentUri,
    pub kind: String,
}

/// Code lens kind.
pub mod code_lens_kind {
    pub const REFERENCES: &str = "references";
    pub const IMPLEMENTATIONS: &str = "implementations";
}

/// Signature help information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: Option<u32>,
    pub active_parameter: Option<u32>,
}

/// Signature information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureInformation {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<ParameterInformation>,
}

/// Parameter information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParameterInformation {
    pub label: String,
    pub documentation: Option<String>,
}

/// A completion item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<u32>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub sort_text: Option<String>,
    pub filter_text: Option<String>,
    pub insert_text: Option<String>,
    pub insert_text_format: Option<u32>,
    pub text_edit: Option<TextEdit>,
    pub additional_text_edits: Option<Vec<TextEdit>>,
    pub commit_characters: Option<Vec<String>>,
    pub data: Option<CompletionItemData>,
}

/// Completion item data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionItemData {
    pub file_name: String,
    pub position: i32,
    pub name: String,
}

/// A completion list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionList {
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

/// Completion item apply kind.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionItemApplyKinds;

/// Completion item defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionItemDefaults;

/// A diagnostic.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<i32>,
    pub code: Option<serde_json::Value>,
    pub source: Option<String>,
    pub message: String,
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

/// Related diagnostic information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    pub location: crate::lsp::lsproto::lsp::Location,
    pub message: String,
}

/// Multi-document highlight.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiDocumentHighlight {
    pub uri: DocumentUri,
    pub highlights: Vec<DocumentHighlight>,
}

/// Workspace edit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    pub changes: Option<std::collections::HashMap<DocumentUri, Vec<TextEdit>>>,
    pub document_changes: Option<Vec<TextDocumentEdit>>,
}

/// A text-document edit or create/rename/delete file operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentEdit {
    pub text_document: Option<TextDocumentIdentifier>,
    pub edits: Vec<TextEdit>,
    pub kind: TextDocumentEditKind,
}

/// Text document edit kind.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDocumentEditKind {
    #[default]
    Edit,
    Create,
    Rename,
    Delete,
}

/// A rename-file operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenameFile {
    pub old_uri: DocumentUri,
    pub new_uri: DocumentUri,
}

/// Text document identifier (with version).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: DocumentUri,
}

/// Code action kind constants.
pub mod code_action_kind {
    pub const SOURCE_SORT_IMPORTS: &str = "source.sortImports";
    pub const SOURCE_ORGANIZE_IMPORTS: &str = "source.organizeImports";
    pub const SOURCE_REMOVE_UNUSED_IMPORTS: &str = "source.removeUnusedImports";
    pub const QUICK_FIX: &str = "quickfix";
    pub const REFACTOR: &str = "refactor";
}

/// A code action.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<String>,
    pub edits: Vec<TextEdit>,
    pub diagnostic: Option<Diagnostic>,
}

/// Linked editing ranges.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkedEditingRanges {
    pub ranges: Vec<Range>,
    pub word_pattern: Option<String>,
}

/// A comment range (for folding / formatting).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentRange {
    pub kind: crate::ast::SyntaxKind,
    pub pos: i32,
    pub end: i32,
}

impl CommentRange {
    pub fn pos(&self) -> usize {
        self.pos as usize
    }
    pub fn end(&self) -> usize {
        self.end as usize
    }
    pub fn contains_exclusive(&self, pos: usize) -> bool {
        (pos as i32) > self.pos && (pos as i32) < self.end
    }
}

/// Client capabilities (stub — Go's lsproto.ClientCapabilities is very large).
#[derive(Debug, Clone, Default)]
pub struct ClientCapabilities {
    pub text_document: TextDocumentClientCapabilities,
    pub experimental: ExperimentalCapabilities,
}

/// Text-document client capabilities.
#[derive(Debug, Clone, Default)]
pub struct TextDocumentClientCapabilities {
    pub hover: HoverCapability,
    pub definition: DefinitionCapability,
    pub type_definition: DefinitionCapability,
    pub document_symbol: DocumentSymbolCapability,
    pub folding_range: FoldingRangeCapability,
    pub semantic_tokens: SemanticTokensClientCapabilities,
}

#[derive(Debug, Clone, Default)]
pub struct HoverCapability {
    pub content_format: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DefinitionCapability {
    pub link_support: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentSymbolCapability {
    pub hierarchical_document_symbol_support: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FoldingRangeCapability {
    pub line_folding_only: bool,
    pub collapsed_text: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticTokensClientCapabilities {
    pub token_types: Vec<String>,
    pub token_modifiers: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExperimentalCapabilities {
    pub hover_verbosity_level: bool,
}

/// Code action params.
#[derive(Debug, Clone, Default)]
pub struct CodeActionParams {
    pub text_document: TextDocumentIdentifier,
    pub range: Range,
    pub context: CodeActionContext,
}

/// Code action context.
#[derive(Debug, Clone, Default)]
pub struct CodeActionContext {
    pub diagnostics: Vec<Diagnostic>,
    pub only: Vec<String>,
}

/// Completion context.
#[derive(Debug, Clone, Default)]
pub struct CompletionContext {
    pub trigger_kind: u32,
    pub trigger_character: Option<String>,
}

/// Signature help context.
#[derive(Debug, Clone, Default)]
pub struct SignatureHelpContext {
    pub trigger_kind: u32,
    pub trigger_character: Option<String>,
    pub is_retrigger: bool,
}

/// Rename params.
#[derive(Debug, Clone, Default)]
pub struct RenameParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    pub new_name: String,
}

/// Inlay hint params.
#[derive(Debug, Clone, Default)]
pub struct InlayHintParams {
    pub text_document: TextDocumentIdentifier,
    pub range: Range,
}

/// Selection range params.
#[derive(Debug, Clone, Default)]
pub struct SelectionRangeParams {
    pub text_document: TextDocumentIdentifier,
    pub positions: Vec<Position>,
}

/// Linked editing range params.
#[derive(Debug, Clone, Default)]
pub struct LinkedEditingRangeParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

/// VS OnAutoInsert params.
#[derive(Debug, Clone, Default)]
pub struct VsOnAutoInsertParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    pub ch: String,
}
