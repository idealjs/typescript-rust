//! Diagnostics provider (1:1 port of Go's `internal/ls/diagnostics.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::{Diagnostic as AstDiagnostic, SourceFile};
use crate::compiler;
use crate::diagnostics::Category;
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range};

use super::language_service::LanguageService;
use super::types::{Diagnostic as LspDiagnostic, DiagnosticRelatedInformation};

impl LanguageService {
    /// Provide diagnostics for a document (pull diagnostics).
    ///
    /// Mirrors `ProvideDiagnostics`:
    /// 1. Run the checker on the file.
    /// 2. Convert checker diagnostics to LSP `Diagnostic` format.
    /// 3. Return the list.
    pub fn provide_diagnostics(&self, document_uri: &DocumentUri) -> Vec<LspDiagnostic> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;
        let file_name = &source_file.file_name;

        // Collect syntactic (parse) diagnostics from the program.
        let mut all_diagnostics: Vec<Arc<AstDiagnostic>> = Vec::new();
        for diag in program.diagnostics() {
            if diag
                .file
                .as_ref()
                .map(|f| f.file_name == *file_name)
                .unwrap_or(false)
            {
                all_diagnostics.push(Arc::clone(diag));
            }
        }

        // Collect semantic diagnostics from the checker.
        let checker = program.build_checker();
        let semantic_diagnostics = checker.get_semantic_diagnostics();
        for diag in &semantic_diagnostics {
            if diag
                .file
                .as_ref()
                .map(|f| f.file_name == *file_name)
                .unwrap_or(false)
            {
                all_diagnostics.push(Arc::new(diag.clone()));
            }
        }

        // Convert all diagnostics to LSP format.
        all_diagnostics
            .iter()
            .map(|d| ast_diagnostic_to_lsp(line_map, d))
            .collect()
    }

    /// Convert compiler diagnostics to LSP diagnostics.
    ///
    /// Mirrors `toLSPDiagnostics`.
    pub fn to_lsp_diagnostics(
        &self,
        diagnostics: &[Vec<Arc<AstDiagnostic>>],
    ) -> Vec<LspDiagnostic> {
        let mut result = Vec::new();
        for group in diagnostics {
            for diag in group {
                let lsp_diag = match &diag.file {
                    Some(file) => ast_diagnostic_to_lsp(&file.line_map, diag),
                    None => ast_diagnostic_to_lsp_no_file(diag),
                };
                result.push(lsp_diag);
            }
        }
        result
    }
}

/// Collect all diagnostics for a file: syntactic, semantic, suggestion.
///
/// Mirrors `getAllDiagnostics`.
pub fn get_all_diagnostics(
    program: &Arc<compiler::Program>,
    file: &Arc<SourceFile>,
) -> Vec<Arc<AstDiagnostic>> {
    let mut result = Vec::new();
    let file_name = &file.file_name;

    // Syntactic (parse) diagnostics.
    for diag in program.diagnostics() {
        if diag
            .file
            .as_ref()
            .map(|f| f.file_name == *file_name)
            .unwrap_or(false)
        {
            result.push(Arc::clone(diag));
        }
    }

    // Semantic diagnostics.
    let semantic = program.get_semantic_diagnostics();
    for diag in &semantic {
        if diag
            .file
            .as_ref()
            .map(|f| f.file_name == *file_name)
            .unwrap_or(false)
        {
            result.push(Arc::new(diag.clone()));
        }
    }

    result
}

// ─── Diagnostic conversion ───────────────────────────────────────────

/// Convert a checker `Diagnostic` to an LSP `Diagnostic`.
fn ast_diagnostic_to_lsp(line_map: &LineMap, diag: &AstDiagnostic) -> LspDiagnostic {
    let message = diagnostic_message(diag);

    // Convert related information.
    let related_information = convert_related_information(diag);

    LspDiagnostic {
        range: Range {
            start: offset_to_position(line_map, diag.loc.pos()),
            end: offset_to_position(line_map, diag.loc.end()),
        },
        severity: Some(category_to_severity(diag.category) as i32),
        code: Some(serde_json::Value::Number(serde_json::Number::from(
            diag.code,
        ))),
        source: Some("typescript".to_string()),
        message,
        related_information,
    }
}

/// Convert a checker `Diagnostic` to an LSP `Diagnostic` when the file/line map
/// is not available (offsets are used as raw character positions on line 0).
fn ast_diagnostic_to_lsp_no_file(diag: &AstDiagnostic) -> LspDiagnostic {
    let message = diagnostic_message(diag);
    let related_information = convert_related_information(diag);

    LspDiagnostic {
        range: Range {
            start: Position {
                line: 0,
                character: diag.loc.pos() as u32,
            },
            end: Position {
                line: 0,
                character: diag.loc.end() as u32,
            },
        },
        severity: Some(category_to_severity(diag.category) as i32),
        code: Some(serde_json::Value::Number(serde_json::Number::from(
            diag.code,
        ))),
        source: Some("typescript".to_string()),
        message,
        related_information,
    }
}

/// Format the message of a diagnostic.
fn diagnostic_message(diag: &AstDiagnostic) -> String {
    if let Some(ref msg) = diag.message {
        let args: Vec<&str> = diag.message_args.iter().map(|s| s.as_str()).collect();
        let text = crate::diagnostics::format_message(msg.text, &args);
        if !text.is_empty() {
            return text;
        }
    }
    format!("TS{}", diag.code)
}

/// Convert related-information entries.
fn convert_related_information(diag: &AstDiagnostic) -> Option<Vec<DiagnosticRelatedInformation>> {
    if diag.related_information.is_empty() {
        return None;
    }
    Some(
        diag.related_information
            .iter()
            .map(|ri| {
                let (start, end) = match &ri.file {
                    Some(file) => {
                        let lm = &file.line_map;
                        (
                            offset_to_position(lm, ri.loc.pos()),
                            offset_to_position(lm, ri.loc.end()),
                        )
                    }
                    None => (
                        Position {
                            line: 0,
                            character: ri.loc.pos() as u32,
                        },
                        Position {
                            line: 0,
                            character: ri.loc.end() as u32,
                        },
                    ),
                };
                DiagnosticRelatedInformation {
                    location: crate::lsp::lsproto::lsp::Location {
                        uri: DocumentUri(
                            ri.file
                                .as_ref()
                                .map(|f| f.file_name.clone())
                                .unwrap_or_default(),
                        ),
                        range: Range { start, end },
                    },
                    message: ri
                        .message
                        .as_ref()
                        .map(|m| {
                            let args: Vec<&str> =
                                ri.message_args.iter().map(|s| s.as_str()).collect();
                            crate::diagnostics::format_message(m.text, &args)
                        })
                        .unwrap_or_default(),
                }
            })
            .collect(),
    )
}

/// Map a checker `Category` to an LSP severity number.
fn category_to_severity(category: Category) -> u32 {
    match category {
        Category::Error => 1,
        Category::Warning => 2,
        Category::Suggestion => 3,
        Category::Message => 4,
    }
}

// ─── Helper functions ────────────────────────────────────────────────

/// Convert a byte offset to an LSP `Position`.
fn offset_to_position(line_map: &LineMap, offset: usize) -> Position {
    let line = line_of_offset(line_map, offset);
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    Position {
        line: line as u32,
        character: offset.saturating_sub(line_start) as u32,
    }
}

/// Binary search for the line number of a byte offset.
fn line_of_offset(line_map: &LineMap, offset: usize) -> usize {
    match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    }
}
