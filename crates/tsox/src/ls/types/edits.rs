use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range, TextEdit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<i32>,
    pub code: Option<serde_json::Value>,
    pub source: Option<String>,
    pub message: String,
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    pub location: crate::lsp::lsproto::lsp::Location,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    pub changes: Option<std::collections::HashMap<DocumentUri, Vec<TextEdit>>>,
    pub document_changes: Option<Vec<TextDocumentEdit>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentEdit {
    pub text_document: Option<TextDocumentIdentifier>,
    pub edits: Vec<TextEdit>,
    pub kind: TextDocumentEditKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDocumentEditKind {
    #[default]
    Edit,
    Create,
    Rename,
    Delete,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenameFile {
    pub old_uri: DocumentUri,
    pub new_uri: DocumentUri,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: DocumentUri,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeLens {
    pub range: Range,
    pub command: Option<CodeLensCommand>,
    pub data: Option<CodeLensData>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeLensCommand {
    pub title: String,
    pub command: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeLensData {
    pub uri: DocumentUri,
    pub kind: String,
}

pub mod code_lens_kind {
    pub const REFERENCES: &str = "references";
    pub const IMPLEMENTATIONS: &str = "implementations";
}

pub mod code_action_kind {
    pub const SOURCE_SORT_IMPORTS: &str = "source.sortImports";
    pub const SOURCE_ORGANIZE_IMPORTS: &str = "source.organizeImports";
    pub const SOURCE_REMOVE_UNUSED_IMPORTS: &str = "source.removeUnusedImports";
    pub const QUICK_FIX: &str = "quickfix";
    pub const REFACTOR: &str = "refactor";
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<String>,
    pub edits: Vec<TextEdit>,
    pub diagnostic: Option<Diagnostic>,
}

#[derive(Debug, Clone, Default)]
pub struct CodeActionParams {
    pub text_document: TextDocumentIdentifier,
    pub range: Range,
    pub context: CodeActionContext,
}

#[derive(Debug, Clone, Default)]
pub struct CodeActionContext {
    pub diagnostics: Vec<Diagnostic>,
    pub only: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RenameParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    pub new_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct VsOnAutoInsertParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    pub ch: String,
}

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
