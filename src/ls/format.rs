//! Document formatting provider (1:1 port of Go's `internal/ls/format.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::core::text::TextRange;
use crate::core::text_change::TextChange;
use crate::ls::lsutil::FormatCodeSettings;
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range, TextEdit};

use super::language_service::LanguageService;

impl LanguageService {
    /// Convert compiler `TextChange`s to LSP `TextEdit`s.
    ///
    /// Mirrors `toLSProtoTextEdits`.
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

    /// Provide full-document formatting.
    ///
    /// Mirrors `ProvideFormatDocument`.
    pub fn provide_format_document(
        &self,
        _document_uri: &DocumentUri,
        _formatting_options: &crate::lsp::lsproto::lsp::FormattingOptions,
    ) -> Vec<TextEdit> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        let format_opts = self.format_options().clone();
        let edits = self.get_formatting_edits_for_document(&file, &format_opts);
        self.to_ls_proto_text_edits(&file, &edits)
    }

    /// Provide range formatting.
    ///
    /// Mirrors `ProvideFormatDocumentRange`.
    pub fn provide_format_document_range(
        &self,
        _document_uri: &DocumentUri,
        _formatting_options: &crate::lsp::lsproto::lsp::FormattingOptions,
        _range: Range,
    ) -> Vec<TextEdit> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        let format_opts = self.format_options().clone();
        let edits = self.get_formatting_edits_for_range(&file, &format_opts, TextRange::default());
        self.to_ls_proto_text_edits(&file, &edits)
    }

    /// Provide on-type formatting.
    ///
    /// Mirrors `ProvideFormatDocumentOnType`.
    pub fn provide_format_document_on_type(
        &self,
        _document_uri: &DocumentUri,
        _formatting_options: &crate::lsp::lsproto::lsp::FormattingOptions,
        _position: Position,
        _character: &str,
    ) -> Vec<TextEdit> {
        let (_program, file) = self.get_program_and_file(_document_uri);
        let format_opts = self.format_options().clone();
        let edits = self.get_formatting_edits_after_keystroke(&file, &format_opts, 0, _character);
        self.to_ls_proto_text_edits(&file, &edits)
    }

    /// Get formatting edits for a range.
    pub fn get_formatting_edits_for_range(
        &self,
        _file: &Arc<SourceFile>,
        _options: &FormatCodeSettings,
        _r: TextRange,
    ) -> Vec<TextChange> {
        // TODO: requires format::FormatSelection
        Vec::new()
    }

    /// Get formatting edits for the whole document.
    pub fn get_formatting_edits_for_document(
        &self,
        _file: &Arc<SourceFile>,
        _options: &FormatCodeSettings,
    ) -> Vec<TextChange> {
        // TODO: requires format::FormatDocument
        Vec::new()
    }

    /// Get formatting edits after a keystroke.
    pub fn get_formatting_edits_after_keystroke(
        &self,
        _file: &Arc<SourceFile>,
        _options: &FormatCodeSettings,
        _position: usize,
        _key: &str,
    ) -> Vec<TextChange> {
        // TODO: requires format::FormatOnOpeningCurly etc.
        Vec::new()
    }
}

/// Get the range of the enclosing comment at a position.
///
/// Mirrors `getRangeOfEnclosingComment`.
pub fn get_range_of_enclosing_comment(
    _file: &Arc<SourceFile>,
    _position: usize,
    _preceding_token: Option<&Arc<crate::ast::Node>>,
    _token_at_position: Option<&Arc<crate::ast::Node>>,
) -> Option<crate::scanner::CommentRange> {
    // TODO: requires scanner comment range iteration
    None
}
