#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Diagnostic as AstDiagnostic;
use crate::ast::SourceFile;
use crate::ast::node::LineMap;
use crate::compiler::Program;
use crate::core::text::TextRange;
use crate::diagnostics::Category;
use crate::lsp::lsproto::lsp::{Position, Range, TextEdit};

use super::language_service::LanguageService;
use super::types::{CodeAction, CodeActionParams, Diagnostic, code_action_kind};

pub struct CodeFixProvider {
    pub error_codes: Vec<i32>,
    pub fix_ids: Vec<String>,
}

pub struct CodeFixContext<'a> {
    pub source_file: &'a Arc<SourceFile>,
    pub span: TextRange,
    pub error_code: i32,
    pub program: &'a Program,
    pub ls: &'a LanguageService,
    pub diagnostic: Option<&'a Diagnostic>,
    pub params: Option<&'a CodeActionParams>,
}

pub struct CombinedCodeActions {
    pub description: String,
    pub changes: Vec<TextEdit>,
}

impl LanguageService {
    pub fn provide_code_actions(&self, params: &CodeActionParams) -> Vec<CodeAction> {
        let (program, source_file) = self.get_program_and_file(&params.text_document.uri);
        let line_map = &source_file.line_map;

        let range_start = lsp_position_to_offset(line_map, &params.range.start);
        let range_end = lsp_position_to_offset(line_map, &params.range.end);

        let checker = program.build_checker();
        let diagnostics = checker.get_semantic_diagnostics();

        let mut actions = Vec::new();

        for diag in &diagnostics {
            let belongs_to_file = diag
                .file
                .as_ref()
                .map(|f| f.file_name == source_file.file_name)
                .unwrap_or(false);
            if !belongs_to_file {
                continue;
            }

            let diag_pos = diag.loc.pos();
            let diag_end = diag.loc.end();
            if diag_pos > range_end || range_start > diag_end {
                continue;
            }

            let title = diagnostic_title(diag);
            actions.push(CodeAction {
                title,
                kind: Some(code_action_kind::QUICK_FIX.to_string()),
                edits: Vec::new(),
                diagnostic: Some(ast_diagnostic_to_lsp(line_map, diag)),
            });
        }

        actions
    }
}

fn diagnostic_title(diag: &AstDiagnostic) -> String {
    if let Some(ref msg) = diag.message {
        let args: Vec<&str> = diag.message_args.iter().map(|s| s.as_str()).collect();
        let text = crate::diagnostics::format_message(msg.text, &args);
        if !text.is_empty() {
            return text;
        }
    }
    format!("Fix diagnostic (code {})", diag.code)
}

fn ast_diagnostic_to_lsp(line_map: &LineMap, diag: &AstDiagnostic) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: offset_to_position(line_map, diag.loc.pos()),
            end: offset_to_position(line_map, diag.loc.end()),
        },
        severity: Some(category_to_severity(diag.category) as i32),
        code: Some(serde_json::Value::Number(serde_json::Number::from(
            diag.code,
        ))),
        source: Some("typescript".to_string()),
        message: diagnostic_title(diag),
        related_information: None,
    }
}

fn category_to_severity(category: Category) -> u32 {
    match category {
        Category::Error => 1,
        Category::Warning => 2,
        Category::Suggestion => 3,
        Category::Message => 4,
    }
}

pub fn registered_code_fix_providers() -> Vec<&'static str> {
    vec![
        "ImportFixProvider",
        "IsolatedDeclarationsFixProvider",
        "FixClassIncorrectlyImplementsInterfaceProvider",
    ]
}

fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}

fn offset_to_position(line_map: &LineMap, offset: usize) -> Position {
    let line = line_of_offset(line_map, offset);
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    Position {
        line: line as u32,
        character: offset.saturating_sub(line_start) as u32,
    }
}

fn line_of_offset(line_map: &LineMap, offset: usize) -> usize {
    match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    }
}
