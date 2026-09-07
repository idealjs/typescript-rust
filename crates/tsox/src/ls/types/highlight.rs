use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoldingRange {
    pub start_line: u32,
    pub start_character: Option<u32>,
    pub end_line: u32,
    pub end_character: Option<u32>,
    pub kind: Option<String>,
    pub collapsed_text: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionRange {
    pub range: Range,
    pub parent: Option<Box<SelectionRange>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentHighlight {
    pub range: Range,
    pub kind: Option<DocumentHighlightKind>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentHighlightKind {
    #[default]
    Text,
    Read,
    Write,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiDocumentHighlight {
    pub uri: DocumentUri,
    pub highlights: Vec<DocumentHighlight>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationLink {
    pub origin_selection_range: Option<Range>,
    pub target_uri: DocumentUri,
    pub target_range: Range,
    pub target_selection_range: Range,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkedEditingRanges {
    pub ranges: Vec<Range>,
    pub word_pattern: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SelectionRangeParams {
    pub text_document: super::edits::TextDocumentIdentifier,
    pub positions: Vec<Position>,
}

#[derive(Debug, Clone, Default)]
pub struct LinkedEditingRangeParams {
    pub text_document: super::edits::TextDocumentIdentifier,
    pub position: Position,
}
