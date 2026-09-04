use crate::core::tristate::{Tristate, bool_to_tristate};
use crate::lsp::lsproto::lsp::FormattingOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum IndentStyle {
    #[default]
    None = 0,
    Block = 1,
    Smart = 2,
}

impl IndentStyle {

    pub fn parse(value: &serde_json::Value) -> IndentStyle {
        match value {
            serde_json::Value::String(s) => match s.to_ascii_lowercase().as_str() {
                "none" => IndentStyle::None,
                "block" => IndentStyle::Block,
                "smart" => IndentStyle::Smart,
                _ => IndentStyle::Smart,
            },
            serde_json::Value::Number(n) => n.as_i64().map_or(IndentStyle::Smart, |i| match i {
                0 => IndentStyle::None,
                1 => IndentStyle::Block,
                2 => IndentStyle::Smart,
                _ => IndentStyle::Smart,
            }),
            _ => IndentStyle::Smart,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SemicolonPreference {
    #[default]
    Ignore,
    Insert,
    Remove,
}

impl SemicolonPreference {
    pub const IGNORE: &'static str = "ignore";
    pub const INSERT: &'static str = "insert";
    pub const REMOVE: &'static str = "remove";

    pub fn parse(value: &serde_json::Value) -> SemicolonPreference {
        if let serde_json::Value::String(s) = value {
            match s.to_ascii_lowercase().as_str() {
                "ignore" => return SemicolonPreference::Ignore,
                "insert" => return SemicolonPreference::Insert,
                "remove" => return SemicolonPreference::Remove,
                _ => {}
            }
        }
        SemicolonPreference::Ignore
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SemicolonPreference::Ignore => "ignore",
            SemicolonPreference::Insert => "insert",
            SemicolonPreference::Remove => "remove",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditorSettings {
    pub base_indent_size: i32,
    pub indent_size: i32,
    pub tab_size: i32,
    pub new_line_character: String,
    pub convert_tabs_to_spaces: Tristate,
    pub indent_style: IndentStyle,
    pub trim_trailing_whitespace: Tristate,
}

#[derive(Debug, Clone, Default)]
pub struct FormatCodeSettings {
    pub base_indent_size: i32,
    pub indent_size: i32,
    pub tab_size: i32,
    pub new_line_character: String,
    pub convert_tabs_to_spaces: Tristate,
    pub indent_style: IndentStyle,
    pub trim_trailing_whitespace: Tristate,

    pub insert_space_after_comma_delimiter: Tristate,
    pub insert_space_after_semicolon_in_for_statements: Tristate,
    pub insert_space_before_and_after_binary_operators: Tristate,
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
    pub insert_space_before_type_annotation: Tristate,
    pub indent_multi_line_object_literal_beginning_on_blank_line: Tristate,
    pub semicolons: SemicolonPreference,
    pub indent_switch_case: Tristate,
}

impl FormatCodeSettings {

    pub fn editor_settings(&self) -> EditorSettings {
        EditorSettings {
            base_indent_size: self.base_indent_size,
            indent_size: self.indent_size,
            tab_size: self.tab_size,
            new_line_character: self.new_line_character.clone(),
            convert_tabs_to_spaces: self.convert_tabs_to_spaces,
            indent_style: self.indent_style,
            trim_trailing_whitespace: self.trim_trailing_whitespace,
        }
    }
}

pub fn get_default_format_code_settings() -> FormatCodeSettings {
    FormatCodeSettings {
        base_indent_size: 0,
        indent_size: default_indent_size(),
        tab_size: default_indent_size(),
        new_line_character: "\n".to_string(),
        convert_tabs_to_spaces: Tristate::True,
        indent_style: IndentStyle::Smart,
        trim_trailing_whitespace: Tristate::True,

        insert_space_after_constructor: Tristate::False,
        insert_space_after_comma_delimiter: Tristate::True,
        insert_space_after_semicolon_in_for_statements: Tristate::True,
        insert_space_before_and_after_binary_operators: Tristate::True,
        insert_space_after_keywords_in_control_flow_statements: Tristate::True,
        insert_space_after_function_keyword_for_anonymous_functions: Tristate::False,
        insert_space_after_opening_and_before_closing_nonempty_parenthesis: Tristate::False,
        insert_space_after_opening_and_before_closing_nonempty_brackets: Tristate::False,
        insert_space_after_opening_and_before_closing_nonempty_braces: Tristate::True,
        insert_space_after_opening_and_before_closing_template_string_braces: Tristate::False,
        insert_space_after_opening_and_before_closing_jsx_expression_braces: Tristate::False,
        insert_space_before_function_parenthesis: Tristate::False,
        place_open_brace_on_new_line_for_functions: Tristate::False,
        place_open_brace_on_new_line_for_control_blocks: Tristate::False,
        semicolons: SemicolonPreference::Ignore,
        indent_switch_case: Tristate::True,
        ..FormatCodeSettings::default()
    }
}

fn default_indent_size() -> i32 {
    4
}

pub fn from_ls_format_options(
    f: &FormatCodeSettings,
    opt: &FormattingOptions,
) -> FormatCodeSettings {
    let mut updated = f.clone();
    updated.tab_size = opt.tab_size as i32;
    updated.indent_size = opt.tab_size as i32;
    updated.convert_tabs_to_spaces = bool_to_tristate(opt.insert_spaces);
    if let Some(trim) = opt.trim_trailing_whitespace {
        updated.trim_trailing_whitespace = bool_to_tristate(trim);
    }
    updated
}

pub fn to_ls_format_options(settings: &FormatCodeSettings) -> FormattingOptions {
    FormattingOptions {
        tab_size: settings.tab_size as u32,
        insert_spaces: settings.convert_tabs_to_spaces.is_true(),
        trim_trailing_whitespace: Some(settings.trim_trailing_whitespace.is_true()),
    }
}
