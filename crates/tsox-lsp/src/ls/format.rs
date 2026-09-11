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
        _range: Range,
    ) -> Vec<TextEdit> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        let format_opts = self.format_options().clone();
        let edits = self.get_formatting_edits_for_range(&file, &format_opts, TextRange::default());
        self.to_ls_proto_text_edits(&file, &edits)
    }

    pub fn provide_format_document_on_type(
        &self,
        _document_uri: &DocumentUri,
        _formatting_options: &crate::lsp::lsproto_lsp::FormattingOptions,
        _position: Position,
        _character: &str,
    ) -> Vec<TextEdit> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        let format_opts = self.format_options().clone();
        let edits = self.get_formatting_edits_after_keystroke(&file, &format_opts, 0, _character);
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
        _file: &Arc<SourceFile>,
        _options: &FormatCodeSettings,
        _position: usize,
        _key: &str,
    ) -> Vec<TextChange> {
        // FormatOnEnter/OnType 待 span_worker 缩进阶段接入
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
        insert_space_before_type_annotation: options.insert_space_before_type_annotation
            == tsox_core::core::tristate::Tristate::True,
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

pub fn get_range_of_enclosing_comment(
    _file: &Arc<SourceFile>,
    _position: usize,
    _preceding_token: Option<&Arc<tsox_frontend::ast::Node>>,
    _token_at_position: Option<&Arc<tsox_frontend::ast::Node>>,
) -> Option<tsox_frontend::scanner::CommentRange> {
    None
}
