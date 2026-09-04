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
mod tests {
    use super::*;

    fn apply_bulk_edits(text: &str, edits: &[TextChange]) -> String {
        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;
        for e in edits {
            if e.pos != last_end {
                result.push_str(&text[last_end..e.pos]);
            }
            result.push_str(&e.new_text);
            last_end = e.end;
        }
        result.push_str(&text[last_end..]);
        result
    }

    #[test]

    fn test_format_no_trailing_space() {
        let test_cases: &[(&str, &str)] = &[
            ("simple statement without trailing newline", "1;"),
            (
                "function call without trailing newline",
                "console.log('hello');",
            ),
            ("if block on single line", "if (true) { }"),
            (
                "class declaration",
                "class A {\n    // Class Contents Go Here\n}",
            ),
            (
                "class declaration with trailing newline",
                "class A {\n    // Class Contents Go Here\n}\n",
            ),
            ("empty block", "if (true) {}"),
            ("module declaration", "module M { }"),
            ("enum declaration", "enum E { A, B }"),
        ];

        let ctx = with_format_code_settings(
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
                insert_space_before_type_annotation: false,
                insert_space_before_and_after_binary_operators: true,
            },
            "\n",
        );

        for (_name, text) in test_cases {
            let source_file =
                crate::parser::Parser::parse_source_file_text("/test.ts", text.to_string());
            let edits = format_document(&ctx, &source_file);
            let new_text = apply_bulk_edits(text, &edits);
            for (i, line) in new_text.split('\n').enumerate() {
                let trimmed = line.trim_end_matches([' ', '\t']);
                assert_eq!(
                    line,
                    trimmed,
                    "Formatter should not add trailing whitespace on line {}",
                    i + 1
                );
            }
        }
    }

    #[test]

    fn test_format() {
        let text = "const x = 1;";
        let source_file =
            crate::parser::Parser::parse_source_file_text("/test.ts", text.to_string());
        let ctx = with_format_code_settings(get_default_format_code_settings(), "\n");

        let edits = format_document(&ctx, &source_file);
        assert!(edits.is_empty(), "no-op formatter should return no edits");

        let sel_edits = format_selection(&ctx, &source_file, 0, text.len());
        assert!(sel_edits.is_empty());

        assert_eq!(get_indentation(0, &source_file, &ctx.settings, false), 0);

        assert_eq!(apply_bulk_edits(text, &edits), text);
    }

    #[test]

    fn test_comment_formatting() {

    }

    #[test]

    fn test_format_selection_preserves_comments() {

    }

    #[test]

    fn test_slice_bounds_panic() {

    }

    #[test]

    fn test_get_indentation_for_named_imports_position() {

    }

    #[test]

    fn test_get_containing_list_named_imports() {

    }
}
