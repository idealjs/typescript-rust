#[derive(Debug, Clone, Default)]
pub struct ClientCapabilities {
    pub text_document: TextDocumentClientCapabilities,
    pub experimental: ExperimentalCapabilities,
}

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
