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
    let source_file = crate::parser::Parser::parse_source_file_text("/test.ts", text.to_string());
    let ctx = with_format_code_settings(get_default_format_code_settings(), "\n");

    let edits = format_document(&ctx, &source_file);
    assert!(edits.is_empty(), "no-op formatter should return no edits");

    let sel_edits = format_selection(&ctx, &source_file, 0, text.len());
    assert!(sel_edits.is_empty());

    assert_eq!(get_indentation(0, &source_file, &ctx.settings, false), 0);

    assert_eq!(apply_bulk_edits(text, &edits), text);
}

#[test]

fn test_comment_formatting() {}

#[test]

fn test_format_selection_preserves_comments() {}

#[test]

fn test_slice_bounds_panic() {}

#[test]

fn test_get_indentation_for_named_imports_position() {}

#[test]

fn test_get_containing_list_named_imports() {}
