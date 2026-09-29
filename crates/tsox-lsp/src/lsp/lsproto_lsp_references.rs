use crate::lsp::lsproto_lsp_basic::Location;
use crate::lsp::lsproto_lsp_protocol::{TextDocumentIdentifier, TextDocumentPositionParams};

pub struct ReferenceContext {
    pub include_declaration: bool,
}

pub struct ReferenceParams {
    pub text_document: TextDocumentIdentifier,
    pub text_document_position: TextDocumentPositionParams,
    pub context: ReferenceContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSReferenceKind {
    Read,
    Write,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct VSReferenceItem {
    pub vs_id: i32,
    pub vs_definition_id: Option<i32>,
    pub vs_location: Location,
    pub vs_definition_text: String,
    pub vs_kind: VSReferenceKind,
    pub vs_project_name: String,
    pub vs_containing_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct VSReferencesResponse {
    pub vs_reference_items: Vec<VSReferenceItem>,
}
