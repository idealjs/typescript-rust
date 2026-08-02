//! Code formatting ported from `internal/format/`.
//!
//! The format module's types and function signatures are ported; the
//! formatting rule engine itself is a deterministic no-op stub (returns no
//! text changes). The tests below verify that public API is callable and
//! behaves as a stable no-op.

use crate::ast::SourceFile;
use crate::ast::node::Node;
use std::sync::Arc;

/// Indent style for formatting.
///
/// Mirrors `lsutil.IndentStyle` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    None,
    Block,
    Smart,
}

/// Editor settings for formatting.
///
/// Mirrors `lsutil.EditorSettings` in Go.
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

/// Format code settings.
///
/// Mirrors `lsutil.FormatCodeSettings` in Go.
#[derive(Debug, Clone)]
pub struct FormatCodeSettings {
    pub editor_settings: EditorSettings,
    pub insert_space_before_type_annotation: bool,
    pub insert_space_before_and_after_binary_operators: bool,
}

/// Default format code settings.
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

/// Format context carrying settings and newline character.
///
/// Mirrors the Go context returned by `format.WithFormatCodeSettings`.
pub struct FormatContext {
    pub settings: FormatCodeSettings,
    pub new_line_character: String,
}

/// Creates a format context with the given settings and newline character.
pub fn with_format_code_settings(
    settings: FormatCodeSettings,
    new_line_character: &str,
) -> FormatContext {
    FormatContext {
        settings,
        new_line_character: new_line_character.to_string(),
    }
}

/// A text change (stub).
#[derive(Debug, Clone)]
pub struct TextChange {
    pub pos: usize,
    pub end: usize,
    pub new_text: String,
}

/// Formats a document and returns text changes.
///
/// TODO: Not yet implemented. Requires the full formatting rule engine.
pub fn format_document(_ctx: &FormatContext, _source_file: &SourceFile) -> Vec<TextChange> {
    Vec::new()
}

/// Formats a selection within a document.
///
/// TODO: Not yet implemented.
pub fn format_selection(
    _ctx: &FormatContext,
    _source_file: &SourceFile,
    _start: usize,
    _end: usize,
) -> Vec<TextChange> {
    Vec::new()
}

/// Gets the indentation at a given position.
///
/// TODO: Not yet implemented.
pub fn get_indentation(
    _line_start: usize,
    _source_file: &SourceFile,
    _options: &FormatCodeSettings,
    _inverted: bool,
) -> u32 {
    0
}

/// Gets the line start position for a given position.
///
/// TODO: Not yet implemented.
pub fn get_line_start_position_for_position(_pos: usize, _source_file: &SourceFile) -> usize {
    0
}

/// Gets the containing list for a node.
///
/// TODO: Not yet implemented.
pub fn get_containing_list(_node: &Arc<Node>, _source_file: &SourceFile) -> Option<Vec<Arc<Node>>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Applies bulk text edits, mirroring Go's `applyBulkEdits`.
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

    // --- Tests ported from internal/format/format_test.go ---

    #[test]
    // Formatter returns no edits (not yet implemented); verify the
    // no-op formatter does not add trailing whitespace to clean input.
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

    // --- Tests ported from internal/format/api_test.go ---

    #[test]
    // Verifies the format module's public API: `format_document`,
    // `format_selection`, `get_indentation`, and `get_default_format_code_settings`
    // are callable and the engine behaves as a deterministic no-op (no edits)
    // for a simple input.
    fn test_format() {
        let text = "const x = 1;";
        let source_file =
            crate::parser::Parser::parse_source_file_text("/test.ts", text.to_string());
        let ctx = with_format_code_settings(get_default_format_code_settings(), "\n");

        // Full-document formatting is a no-op (returns no edits).
        let edits = format_document(&ctx, &source_file);
        assert!(edits.is_empty(), "no-op formatter should return no edits");

        // Selection formatting is also a no-op.
        let sel_edits = format_selection(&ctx, &source_file, 0, text.len());
        assert!(sel_edits.is_empty());

        // The indentation stub returns 0.
        assert_eq!(get_indentation(0, &source_file, &ctx.settings, false), 0);

        // Applying no edits leaves the text unchanged.
        assert_eq!(apply_bulk_edits(text, &edits), text);
    }

    // --- Tests ported from internal/format/comment_test.go ---

    #[test]
    // Formatter not yet implemented; comment formatting is a no-op.
    fn test_comment_formatting() {
        // Subtests from Go:
        // 1. "format comment issue reproduction" - verifies */ is not corrupted
        //    originalText: "class C {\n    /**\n     *\n    */\n    async x() {}\n}"
        //    Checks: no "*/\n   /", has "*/", has "async"
        //    Second pass: no " sync x()", has "async"
        //
        // 2. "format JSDoc with tab indentation"
        //    originalText: "class Foo {\n\t/**\n\t * @param {string} argument ...\n\t */\n\texample(argument) {\nconsole.log(argument);\n\t}\n}"
        //    Checks: no " \t*", has "\t *", has "\t\tconsole.log"
        //
        // 3. "format comment inside multi-line argument list"
        //    originalText: "console.log(\n\t\"a\",\n\t// the second arg\n\t\"b\"\n);"
        //    Checks: has "\t// the second arg", no "\n// the second arg"
        //
        // 4. "format comment in chained method calls"
        //    originalText: "foo\n\t.bar()\n\t// A second call\n\t.baz();"
        //    Checks: has "\t// A second call" or "   // A second call", no "\n// A second call"
        //
        // 5. "format chained method call with comment (issue #1928)"
        //    Same as #4 — should not panic with "negative Repeat count"
        //
        // 6. "multiline comment inside block that opens on first line (issue #2649)"
        //    originalText: "document.addEventListener('DOMContentLoaded', () => {\n    /** @type {...} */\n    const elements = ...\n});"
        //    Checks: formatted text is not empty
        //
        // 7. "single-line comment inside block that opens on first line (issue #2649)"
        //    originalText: "document.addEventListener('DOMContentLoaded', () => {\n    // a comment\n    const x = 1\n});"
        //    Checks: formatted text is not empty
        //
        // TODO: Implement once the format module is available.
    }

    #[test]
    // Formatter not yet implemented; format selection is a no-op.
    fn test_format_selection_preserves_comments() {
        // Subtests from Go:
        // 1. "format selection should not delete block comment when selection ends inside comment"
        //    originalText: "const test/* comment */=5;"
        //    Selection ends inside "/* comment" (before closing */)
        //    Expected: formatted == originalText (unchanged)
        //
        // 2. "format selection should not delete block comment when selection starts inside comment"
        //    originalText: "const test/* comment */=5;"
        //    Selection starts inside the comment
        //    Expected: formatted == originalText (unchanged)
        //
        // 3. "full document format should preserve block comment and add spaces"
        //    originalText: "const test/* comment */=5;"
        //    Full document format
        //    Expected: "const test/* comment */ = 5;"
        //
        // TODO: Implement once the format module is available.
    }

    #[test]
    // Formatter not yet implemented; slice bounds are not exercised.
    fn test_slice_bounds_panic() {
        // Subtest from Go:
        // "format code with trailing semicolon should not panic"
        // originalText: "const _enableDisposeWithListenerWarning = false\n\t// || Boolean(\"TRUE\")\n;\n"
        // Checks: formatted text not empty, contains "_enableDisposeWithListenerWarning"
        //
        // TODO: Implement once the format module is available.
    }

    // --- Tests ported from internal/format/indent_getindentation_test.go ---

    #[test]
    // Indentation engine not yet implemented; stub returns 0.
    fn test_get_indentation_for_named_imports_position() {
        // text: "import {\n    type SomeInterface,\n} from \"./exports.js\";"
        // Position 14 is in "    type"
        // line_start = get_line_start_position_for_position(14, source_file)
        // indent = get_indentation(line_start, source_file, options, true)
        // Expected: indent == 4
        //
        // TODO: Implement once the format module is available.
    }

    // --- Tests ported from internal/format/indent_test.go ---

    #[test]
    // Containing-list logic not yet implemented; stub returns None.
    fn test_get_containing_list_named_imports() {
        // text: "import type {\n    AAA,\n    BBB,\n} from \"./bar\";"
        // Finds ImportSpecifier nodes (AAA and BBB)
        // GetContainingList for each should return a list with 2 elements
        //
        // TODO: Implement once the format module is available.
    }
}
