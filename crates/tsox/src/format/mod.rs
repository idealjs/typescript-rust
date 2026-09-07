use crate::ast::SourceFile;
use crate::ast::node::Node;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    None,
    Block,
    Smart,
}

#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub tab_size: u32,
    pub indent_size: u32,
    pub base_indent_size: u32,
    pub new_line_character: String,
    pub convert_tabs_to_spaces: bool,
    pub indent_style: IndentStyle,
    pub trim_trailing_whitespace: bool,
}

#[derive(Debug, Clone)]
pub struct FormatCodeSettings {
    pub editor_settings: EditorSettings,
    pub insert_space_before_type_annotation: bool,
    pub insert_space_before_and_after_binary_operators: bool,
}

pub fn get_default_format_code_settings() -> FormatCodeSettings {
    FormatCodeSettings {
        editor_settings: EditorSettings {
            tab_size: 4,
            indent_size: 4,
            base_indent_size: 0,
            new_line_character: "\n".to_string(),
            convert_tabs_to_spaces: true,
            indent_style: IndentStyle::Smart,
            trim_trailing_whitespace: false,
        },
        insert_space_before_type_annotation: false,
        insert_space_before_and_after_binary_operators: true,
    }
}

pub struct FormatContext {
    pub settings: FormatCodeSettings,
    pub new_line_character: String,
}

pub fn with_format_code_settings(
    settings: FormatCodeSettings,
    new_line_character: &str,
) -> FormatContext {
    FormatContext {
        settings,
        new_line_character: new_line_character.to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct TextChange {
    pub pos: usize,
    pub end: usize,
    pub new_text: String,
}

pub fn format_document(_ctx: &FormatContext, _source_file: &SourceFile) -> Vec<TextChange> {
    Vec::new()
}

pub fn format_selection(
    _ctx: &FormatContext,
    _source_file: &SourceFile,
    _start: usize,
    _end: usize,
) -> Vec<TextChange> {
    Vec::new()
}

pub fn get_indentation(
    _line_start: usize,
    _source_file: &SourceFile,
    _options: &FormatCodeSettings,
    _inverted: bool,
) -> u32 {
    0
}

pub fn get_line_start_position_for_position(_pos: usize, _source_file: &SourceFile) -> usize {
    0
}

pub fn get_containing_list(_node: &Arc<Node>, _source_file: &SourceFile) -> Option<Vec<Arc<Node>>> {
    None
}

#[cfg(test)]
mod tests;
