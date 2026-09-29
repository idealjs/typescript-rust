#![allow(dead_code)]

use std::sync::Arc;

use crate::ls::lsutil::FormatCodeSettings;
use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp::Position;
use crate::lsp::lsproto_lsp::Range;
use crate::lsp::lsproto_lsp::TextEdit;
use tsox_core::core::text::TextRange;
use tsox_core::core::text_change::TextChange;
use tsox_frontend::ast::SourceFile;

use super::language_service::LanguageService;

impl LanguageService {
    pub fn to_ls_proto_text_edits(
        &self,
        file: &Arc<SourceFile>,
        changes: &[TextChange],
    ) -> Vec<TextEdit> {
        let script = super::language_service::ScriptInfo {
            file_name: file.file_name.clone(),
            text: file.text.clone(),
        };
        changes
            .iter()
            .map(|c| TextEdit {
                new_text: c.new_text.clone(),
                range: self.create_lsp_range_from_bounds(c.range.pos(), c.range.end(), &script),
            })
            .collect()
    }

    pub fn provide_format_document(
        &self,
        _document_uri: &DocumentUri,
        _formatting_options: &crate::lsp::lsproto_lsp::FormattingOptions,
    ) -> Vec<TextEdit> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        let format_opts = self.format_options().clone();
        let edits = self.get_formatting_edits_for_document(&file, &format_opts);
        self.to_ls_proto_text_edits(&file, &edits)
    }

    pub fn provide_format_document_range(
        &self,
        _document_uri: &DocumentUri,
        _formatting_options: &crate::lsp::lsproto_lsp::FormattingOptions,
        range: Range,
    ) -> Vec<TextEdit> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        let format_opts = self.format_options().clone();
        let start = crate::ls::position::lsp_position_to_offset(
            &file.text,
            &file.line_map,
            range.start.line as usize,
            range.start.character as usize,
        );
        let end = crate::ls::position::lsp_position_to_offset(
            &file.text,
            &file.line_map,
            range.end.line as usize,
            range.end.character as usize,
        );
        let edits = self.get_formatting_edits_for_range(&file, &format_opts, TextRange::new(start, end));
        self.to_ls_proto_text_edits(&file, &edits)
    }

    pub fn provide_format_document_on_type(
        &self,
        _document_uri: &DocumentUri,
        _formatting_options: &crate::lsp::lsproto_lsp::FormattingOptions,
        position: Position,
        character: &str,
    ) -> Vec<TextEdit> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        let format_opts = self.format_options().clone();
        let offset = crate::ls::position::lsp_position_to_offset(
            &file.text,
            &file.line_map,
            position.line as usize,
            position.character as usize,
        );
        let edits =
            self.get_formatting_edits_after_keystroke(&file, &format_opts, offset, character);
        self.to_ls_proto_text_edits(&file, &edits)
    }


    pub fn get_formatting_edits_for_document(
        &self,
        file: &Arc<SourceFile>,
        options: &FormatCodeSettings,
    ) -> Vec<TextChange> {
        let ctx_opts = to_engine_settings(options);
        let ctx = tsox_frontend::format::with_format_code_settings(ctx_opts, "\n");
        tsox_frontend::format::format_document(&ctx, file)
            .into_iter()
            .map(|c| TextChange { range: TextRange::new(c.pos, c.end), new_text: c.new_text })
            .collect()
    }

    pub fn get_formatting_edits_for_range(
        &self,
        file: &Arc<SourceFile>,
        options: &FormatCodeSettings,
        r: TextRange,
    ) -> Vec<TextChange> {
        let ctx_opts = to_engine_settings(options);
        let ctx = tsox_frontend::format::with_format_code_settings(ctx_opts, "\n");
        tsox_frontend::format::format_selection(&ctx, file, r.pos(), r.end())
            .into_iter()
            .map(|c| TextChange { range: TextRange::new(c.pos, c.end), new_text: c.new_text })
            .collect()
    }

    pub fn get_formatting_edits_after_keystroke(
        &self,
        file: &Arc<SourceFile>,
        options: &FormatCodeSettings,
        position: usize,
        key: &str,
    ) -> Vec<TextChange> {
        let ctx_opts = to_engine_settings(options);
        let ctx = tsox_frontend::format::with_format_code_settings(ctx_opts, "\n");
        let token_at_position = tsox_frontend::astnav::get_token_at_position(&file.node, position);
        if get_range_of_enclosing_comment(file, position, None, token_at_position.as_ref()).is_none()
        {
            return match key {
                "{" => tsox_frontend::format::format_on_opening_curly(&ctx, file, position),
                "}" => tsox_frontend::format::format_on_closing_curly(&ctx, file, position),
                ";" => tsox_frontend::format::format_on_semicolon(&ctx, file, position),
                "\n" => tsox_frontend::format::format_on_enter(&ctx, file, position),
                _ => Vec::new(),
            }
            .into_iter()
            .map(|c| TextChange { range: TextRange::new(c.pos, c.end), new_text: c.new_text })
            .collect();
        }
        Vec::new()
    }
}

/// lsutil::FormatCodeSettings → 引擎 FormatCodeSettings
fn to_engine_settings(
    options: &FormatCodeSettings,
) -> tsox_frontend::format::FormatCodeSettings {
    let tristate = |t: tsox_core::core::tristate::Tristate| match t {
        tsox_core::core::tristate::Tristate::True => tsox_frontend::format::Tristate::True,
        tsox_core::core::tristate::Tristate::False => tsox_frontend::format::Tristate::False,
        _ => tsox_frontend::format::Tristate::Unknown,
    };
    let editor = tsox_frontend::format::EditorSettings {
        tab_size: options.tab_size as u32,
        indent_size: options.indent_size as u32,
        base_indent_size: options.base_indent_size.max(0) as u32,
        new_line_character: options.new_line_character.clone(),
        convert_tabs_to_spaces: options.convert_tabs_to_spaces == tsox_core::core::tristate::Tristate::True,
        indent_style: match options.indent_style {
            crate::ls::lsutil::IndentStyle::None => tsox_frontend::format::IndentStyle::None,
            crate::ls::lsutil::IndentStyle::Block => tsox_frontend::format::IndentStyle::Block,
            crate::ls::lsutil::IndentStyle::Smart => tsox_frontend::format::IndentStyle::Smart,
        },
        trim_trailing_whitespace: options.trim_trailing_whitespace != tsox_core::core::tristate::Tristate::False,
    };
    tsox_frontend::format::FormatCodeSettings {
        editor_settings: editor,
        insert_space_before_type_annotation: tristate(options.insert_space_before_type_annotation),
        insert_space_before_and_after_binary_operators: options
            .insert_space_before_and_after_binary_operators
            != tsox_core::core::tristate::Tristate::False,
        insert_space_after_comma_delimiter: tristate(options.insert_space_after_comma_delimiter),
        insert_space_after_semicolon_in_for_statements: tristate(
            options.insert_space_after_semicolon_in_for_statements,
        ),
        insert_space_after_constructor: tristate(options.insert_space_after_constructor),
        insert_space_after_keywords_in_control_flow_statements: tristate(
            options.insert_space_after_keywords_in_control_flow_statements,
        ),
        insert_space_after_function_keyword_for_anonymous_functions: tristate(
            options.insert_space_after_function_keyword_for_anonymous_functions,
        ),
        insert_space_after_opening_and_before_closing_nonempty_parenthesis: tristate(
            options.insert_space_after_opening_and_before_closing_nonempty_parenthesis,
        ),
        insert_space_after_opening_and_before_closing_nonempty_brackets: tristate(
            options.insert_space_after_opening_and_before_closing_nonempty_brackets,
        ),
        insert_space_after_opening_and_before_closing_nonempty_braces: tristate(
            options.insert_space_after_opening_and_before_closing_nonempty_braces,
        ),
        insert_space_after_opening_and_before_closing_empty_braces: tristate(
            options.insert_space_after_opening_and_before_closing_empty_braces,
        ),
        insert_space_after_opening_and_before_closing_template_string_braces: tristate(
            options.insert_space_after_opening_and_before_closing_template_string_braces,
        ),
        insert_space_after_opening_and_before_closing_jsx_expression_braces: tristate(
            options.insert_space_after_opening_and_before_closing_jsx_expression_braces,
        ),
        insert_space_after_type_assertion: tristate(options.insert_space_after_type_assertion),
        insert_space_before_function_parenthesis: tristate(options.insert_space_before_function_parenthesis),
        place_open_brace_on_new_line_for_functions: tristate(options.place_open_brace_on_new_line_for_functions),
        place_open_brace_on_new_line_for_control_blocks: tristate(options.place_open_brace_on_new_line_for_control_blocks),
        indent_switch_case: tristate(options.indent_switch_case),
        indent_multi_line_object_literal_beginning_on_blank_line: tristate(
            options.indent_multi_line_object_literal_beginning_on_blank_line,
        ),
        semicolons: match options.semicolons {
            crate::ls::lsutil::SemicolonPreference::Ignore => {
                tsox_frontend::format::SemicolonPreference::Ignore
            }
            crate::ls::lsutil::SemicolonPreference::Insert => {
                tsox_frontend::format::SemicolonPreference::Insert
            }
            crate::ls::lsutil::SemicolonPreference::Remove => {
                tsox_frontend::format::SemicolonPreference::Remove
            }
        },
    }
}

/// Go ls/format.go getRangeOfEnclosingComment
pub fn get_range_of_enclosing_comment(
    file: &Arc<SourceFile>,
    position: usize,
    preceding_token: Option<&Arc<tsox_frontend::ast::Node>>,
    token_at_position: Option<&Arc<tsox_frontend::ast::Node>>,
) -> Option<tsox_frontend::scanner::CommentRange> {
    use tsox_frontend::scanner::{get_leading_comment_ranges, get_trailing_comment_ranges};

    let mut token_at_position = token_at_position.cloned()?;
    // JSDoc 归并到其宿主节点
    if token_at_position.flags.contains(tsox_frontend::ast::NodeFlags::JSDoc) {
        token_at_position = token_at_position.parent()?;
    }
    let token_start =
        tsox_frontend::astnav::get_start_of_node(&token_at_position, file, false);
    if token_start <= position && position < token_at_position.end() {
        return None;
    }

    let mut comment_ranges: Vec<tsox_frontend::scanner::CommentRange> = Vec::new();
    if let Some(preceding) = preceding_token {
        comment_ranges
            .extend(get_trailing_comment_ranges(&file.text, preceding.end()));
    }
    if token_at_position.kind != tsox_frontend::ast::SyntaxKind::JsxText {
        comment_ranges
            .extend(get_leading_comment_ranges(&file.text, token_at_position.pos()));
    }
    let text_len = file.text.len();
    for comment_range in comment_ranges {
        let contains_exclusive = position >= comment_range.pos && position < comment_range.end;
        let kind_single = comment_range.kind == tsox_frontend::scanner::CommentRangeKind::SingleLine;
        if contains_exclusive
            || (position == comment_range.end && (kind_single || position == text_len))
        {
            return Some(comment_range);
        }
    }
    None
}
