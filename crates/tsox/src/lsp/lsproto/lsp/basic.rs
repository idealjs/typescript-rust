use super::uri::DocumentUri;
use serde::{Deserialize, Serialize};

pub const CODE_ACTION_KIND_SOURCE_REMOVE_UNUSED_IMPORTS: &str = "source.removeUnusedImports";
pub const CODE_ACTION_KIND_SOURCE_SORT_IMPORTS: &str = "source.sortImports";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Location {
    pub uri: DocumentUri,
    pub range: Range,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextEdit {
    pub new_text: String,
    pub range: Range,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormattingOptions {
    pub tab_size: u32,
    pub insert_spaces: bool,
    pub trim_trailing_whitespace: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkupKind {
    PlainText,
    Markdown,
}

impl Default for MarkupKind {
    fn default() -> Self {
        MarkupKind::PlainText
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StringOrMarkupContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markup_content: Option<MarkupContent>,
}

impl StringOrMarkupContent {
    pub fn as_string(&self) -> String {
        if let Some(s) = &self.string {
            return s.clone();
        }
        if let Some(mc) = &self.markup_content {
            return mc.value.clone();
        }
        String::new()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

pub fn preferred_markup_kind(formats: &[MarkupKind]) -> MarkupKind {
    if !formats.is_empty() {
        formats[0].clone()
    } else {
        MarkupKind::PlainText
    }
}
