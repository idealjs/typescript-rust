pub type SourceIndex = i32;
pub type NameIndex = i32;

pub const MISSING_SOURCE: SourceIndex = -1;
pub const MISSING_NAME: NameIndex = -1;
pub const MISSING_LINE_OR_COLUMN: i32 = -1;
pub const MISSING_UTF16_COLUMN: i32 = -1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    pub generated_line: i32,
    pub generated_character: i32,
    pub source_index: SourceIndex,
    pub source_line: i32,
    pub source_character: i32,
    pub name_index: NameIndex,
}

impl Mapping {
    pub fn is_source_mapping(&self) -> bool {
        self.source_index != MISSING_SOURCE
            && self.source_line != MISSING_LINE_OR_COLUMN
            && self.source_character != MISSING_UTF16_COLUMN
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RawSourceMap {
    pub version: i32,
    pub file: String,
    #[serde(default, rename = "sourceRoot")]
    pub source_root: String,
    pub sources: Vec<String>,
    pub names: Vec<String>,
    pub mappings: String,
    #[serde(
        default,
        rename = "sourcesContent",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub sources_content: Vec<Option<String>>,
}

pub fn try_get_source_mapping_url(text: &str, line_starts: &[usize]) -> String {
    if line_starts.is_empty() {
        return String::new();
    }
    for index in (0..line_starts.len()).rev() {
        let pos = line_starts[index];
        let end = if index + 1 < line_starts.len() {
            line_starts[index + 1]
        } else {
            text.len()
        };
        let line = text[pos..end].trim();
        if line.is_empty() {
            continue;
        }
        let bytes = line.as_bytes();
        if bytes.len() < 4
            || !line.starts_with("//")
            || (bytes[2] != b'#' && bytes[2] != b'@')
            || bytes[3] != b' '
        {
            break;
        }
        if let Some(url) = line[4..].strip_prefix("sourceMappingURL=") {
            return url.trim_end().to_string();
        }
    }
    String::new()
}
