use crate::lsp::lsproto::lsp::TextEdit;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionItemData {
    pub file_name: String,
    pub position: i32,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionList {
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionItemApplyKinds;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionItemDefaults;

#[derive(Debug, Clone, Default)]
pub struct CompletionContext {
    pub trigger_kind: u32,
    pub trigger_character: Option<String>,
}
