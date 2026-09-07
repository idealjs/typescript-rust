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

    pub fn get_formatting_edits_for_range(
        &self,
        _file: &Arc<SourceFile>,
        _options: &FormatCodeSettings,
        _r: TextRange,
    ) -> Vec<TextChange> {
        Vec::new()
    }

    pub fn get_formatting_edits_for_document(
        &self,
        _file: &Arc<SourceFile>,
        _options: &FormatCodeSettings,
    ) -> Vec<TextChange> {
        Vec::new()
    }

    pub fn get_formatting_edits_after_keystroke(
        &self,
        _file: &Arc<SourceFile>,
        _options: &FormatCodeSettings,
        _position: usize,
        _key: &str,
    ) -> Vec<TextChange> {
        Vec::new()
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
