use crate::lsp::lsproto::lsp::{Location, MarkupContent, Position, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hover {
    pub contents: HoverContent,
    pub range: Option<Range>,
    pub can_increase_verbosity: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HoverContent {
    pub markup_content: Option<MarkupContent>,
    pub string: Option<String>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub container_name: Option<String>,
    pub tags: Option<Vec<i32>>,
    pub deprecated: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InlayHint {
    pub position: Position,
    pub label: InlayHintLabel,
    pub kind: Option<i32>,
    pub text_edits: Option<Vec<crate::lsp::lsproto::lsp::TextEdit>>,
    pub padding_left: Option<bool>,
    pub padding_right: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InlayHintLabel {
    pub string: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct InlayHintParams {
    pub text_document: super::edits::TextDocumentIdentifier,
    pub range: Range,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: Option<u32>,
    pub active_parameter: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureInformation {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<ParameterInformation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParameterInformation {
    pub label: String,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SignatureHelpContext {
    pub trigger_kind: u32,
    pub trigger_character: Option<String>,
    pub is_retrigger: bool,
}
