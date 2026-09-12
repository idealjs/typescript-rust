mod api;
mod indentation;
mod indenter;
mod lists;
mod rule;
mod rule_context;
mod rule_context_2;
mod rules;
mod rulesmap;
mod scanner;
mod span;
mod util;
mod visit_generated;

pub(crate) use crate::ast::SourceFile;
pub use rule_context::Tristate;
pub(crate) use crate::ast::node::Node;
pub(crate) use std::sync::Arc;

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
    pub insert_space_before_type_annotation: Tristate,
    pub insert_space_before_and_after_binary_operators: bool,
    pub insert_space_after_comma_delimiter: Tristate,
    pub insert_space_after_semicolon_in_for_statements: Tristate,
    pub insert_space_after_constructor: Tristate,
    pub insert_space_after_keywords_in_control_flow_statements: Tristate,
    pub insert_space_after_function_keyword_for_anonymous_functions: Tristate,
    pub insert_space_after_opening_and_before_closing_nonempty_parenthesis: Tristate,
    pub insert_space_after_opening_and_before_closing_nonempty_brackets: Tristate,
    pub insert_space_after_opening_and_before_closing_nonempty_braces: Tristate,
    pub insert_space_after_opening_and_before_closing_empty_braces: Tristate,
    pub insert_space_after_opening_and_before_closing_template_string_braces: Tristate,
    pub insert_space_after_opening_and_before_closing_jsx_expression_braces: Tristate,
    pub insert_space_after_type_assertion: Tristate,
    pub insert_space_before_function_parenthesis: Tristate,
    pub place_open_brace_on_new_line_for_functions: Tristate,
    pub place_open_brace_on_new_line_for_control_blocks: Tristate,
    pub semicolons: SemicolonPreference,
    pub indent_switch_case: Tristate,
    pub indent_multi_line_object_literal_beginning_on_blank_line: Tristate,
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
            trim_trailing_whitespace: true,
        },
        
        insert_space_before_and_after_binary_operators: true,
        insert_space_after_comma_delimiter: Tristate::True,
        insert_space_after_semicolon_in_for_statements: Tristate::True,
        insert_space_after_constructor: Tristate::False,
        insert_space_after_keywords_in_control_flow_statements: Tristate::True,
        insert_space_after_function_keyword_for_anonymous_functions: Tristate::False,
        insert_space_after_opening_and_before_closing_nonempty_parenthesis: Tristate::False,
        insert_space_after_opening_and_before_closing_nonempty_brackets: Tristate::False,
        insert_space_before_type_annotation: Tristate::Unknown,
        insert_space_after_opening_and_before_closing_nonempty_braces: Tristate::True,
        insert_space_after_opening_and_before_closing_empty_braces: Tristate::Unknown,
        insert_space_after_opening_and_before_closing_template_string_braces: Tristate::False,
        insert_space_after_opening_and_before_closing_jsx_expression_braces: Tristate::False,
        insert_space_after_type_assertion: Tristate::False,
        insert_space_before_function_parenthesis: Tristate::False,
        place_open_brace_on_new_line_for_functions: Tristate::False,
        place_open_brace_on_new_line_for_control_blocks: Tristate::False,
        semicolons: SemicolonPreference::Ignore,
        indent_switch_case: Tristate::True,
        indent_multi_line_object_literal_beginning_on_blank_line: Tristate::Unknown,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SemicolonPreference {
    #[default]
    Ignore,
    Insert,
    Remove,
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

pub fn format_document(ctx: &FormatContext, source_file: &Arc<SourceFile>) -> Vec<TextChange> {
    api::format_document_with(ctx, source_file)
}

pub fn format_selection(
    ctx: &FormatContext,
    source_file: &Arc<SourceFile>,
    start: usize,
    end: usize,
) -> Vec<TextChange> {
    api::format_selection_with(ctx, source_file, start, end)
}

pub fn format_on_semicolon(ctx: &FormatContext, source_file: &Arc<SourceFile>, position: usize) -> Vec<TextChange> {
    api::format_on_semicolon_with(ctx, source_file, position)
}

pub fn format_on_enter(ctx: &FormatContext, source_file: &Arc<SourceFile>, position: usize) -> Vec<TextChange> {
    api::format_on_enter_with(ctx, source_file, position)
}

pub fn format_on_opening_curly(ctx: &FormatContext, source_file: &Arc<SourceFile>, position: usize) -> Vec<TextChange> {
    api::format_on_opening_curly_with(ctx, source_file, position)
}

pub fn format_on_closing_curly(ctx: &FormatContext, source_file: &Arc<SourceFile>, position: usize) -> Vec<TextChange> {
    api::format_on_closing_curly_with(ctx, source_file, position)
}

#[cfg(test)]
pub(crate) mod tests;
