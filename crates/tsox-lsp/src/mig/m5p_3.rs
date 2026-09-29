#![allow(dead_code, unused_imports, unused_variables)]

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use crate::ls::language_service::LanguageService;
use crate::mig::m5n::LspError;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Diagnostic, Node, SourceFile};

use super::m5p_3_support::{CodeAction, CodeFixContext, CodeFixProvider, CombinedCodeActions};

/// 本文件内引用的 lsproto 缺失类型：Go lsproto 包的最小等价移植，
/// 统一收口在本地 mod 以复用 `lsproto::` 路径写法；合并期归位到 crate::lsp::lsproto。
mod lsproto {
    use std::collections::HashMap;

    pub use crate::lsp::lsproto::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CodeActionKind(pub &'static str);

    #[allow(non_upper_case_globals)]
    impl CodeActionKind {
        pub const Empty: CodeActionKind = CodeActionKind("");
        pub const QuickFix: CodeActionKind = CodeActionKind("quickfix");
        pub const Source: CodeActionKind = CodeActionKind("source");
        pub const SourceFixAll: CodeActionKind = CodeActionKind("source.fixAll");
        pub const SourceFixAllTs: CodeActionKind = CodeActionKind("source.fixAll.ts");
        pub const SourceOrganizeImports: CodeActionKind = CodeActionKind("source.organizeImports");
        pub const SourceOrganizeImportsTs: CodeActionKind = CodeActionKind("source.organizeImports.ts");
        pub const SourceRemoveUnusedImportsTs: CodeActionKind = CodeActionKind("source.removeUnusedImports.ts");
        pub const SourceSortImportsTs: CodeActionKind = CodeActionKind("source.sortImports.ts");

        pub fn contains(&self, other: &CodeActionKind) -> bool {
            *self == *other
                || *self == Self::Empty
                || (other.0.len() > self.0.len()
                    && other.0.starts_with(self.0)
                    && other.0.as_bytes()[self.0.len()] == b'.')
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Command {
        pub title: String,
        pub tooltip: Option<String>,
        pub command: String,
        pub arguments: Option<Vec<serde_json::Value>>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct WorkspaceEdit {
        pub changes: HashMap<DocumentUri, Vec<TextEdit>>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CodeAction {
        pub title: String,
        pub kind: Option<CodeActionKind>,
        pub diagnostics: Option<Vec<Diagnostic>>,
        pub edit: Option<WorkspaceEdit>,
        pub command: Option<Command>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CommandOrCodeAction {
        pub command: Option<Command>,
        pub code_action: Option<CodeAction>,
    }
}

pub fn code_action_compare(a: &CodeAction, b: &CodeAction) -> Ordering {
    let c = a.description.cmp(&b.description);
    if c != Ordering::Equal {
        return c;
    }
    let c = a.changes.len().cmp(&b.changes.len());
    if c != Ordering::Equal {
        return c;
    }
    for (i, edit) in a.changes.iter().enumerate() {
        let c = compare_text_edits(edit, &b.changes[i]);
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}

pub fn compare_text_edits(a: &lsproto::TextEdit, b: &lsproto::TextEdit) -> Ordering {
    let c = crate::lsp::lsproto_util::compare_ranges(&a.range, &b.range);
    if c != Ordering::Equal {
        return c;
    }
    a.new_text.cmp(&b.new_text)
}

pub fn code_fix_provider_matches_lsp_diagnostic(provider: &CodeFixProvider, diagnostic: &lsproto::Diagnostic) -> bool {
    if let Some(source) = &diagnostic.source {
        if source != "ts" {
            return false;
        }
    }
    diagnostic
        .code
        .as_ref()
        .and_then(|c| c.as_i64())
        .map(|code| contains_error_code(&provider.error_codes, code as i32))
        .unwrap_or(false)
}

pub fn contains_error_code(codes: &[i32], code: i32) -> bool {
    codes.contains(&code)
}

pub fn convert_to_lsp_code_action(action: &CodeAction, diag: &lsproto::Diagnostic, uri: &lsproto::DocumentUri) -> lsproto::CommandOrCodeAction {
    let kind = lsproto::CodeActionKind::QuickFix;
    let mut changes: HashMap<lsproto::DocumentUri, Vec<lsproto::TextEdit>> = HashMap::new();
    changes.insert(uri.clone(), action.changes.clone());
    let diagnostics = vec![diag.clone()];
    lsproto::CommandOrCodeAction {
        code_action: Some(lsproto::CodeAction {
            title: action.description.clone(),
            kind: Some(kind),
            edit: Some(lsproto::WorkspaceEdit { changes }),
            diagnostics: Some(diagnostics),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub fn get_fix_all_quick_fixes(
    l: &LanguageService,
    program: &Arc<tsox_compile::compiler::Program>,
    file: &Arc<SourceFile>,
    uri: &lsproto::DocumentUri,
    fix_id_seen: &HashMap<String, Arc<CodeFixProvider>>,
) -> Result<Vec<lsproto::CommandOrCodeAction>, LspError> {
    let mut actions: Vec<lsproto::CommandOrCodeAction> = Vec::new();
    let mut seen: tsox_core::collections::set::Set<usize> = Default::default();
    for provider in fix_id_seen.values() {
        if !seen.add_if_absent(provider.id) {
            continue;
        }
        let Some(get_all) = provider.get_all_code_actions.as_ref() else { continue };
        if !has_multiple_fixable_diagnostics(program, file, &provider.error_codes) {
            continue;
        }
        let fix_context = CodeFixContext {
            source_file: file.clone(),
            span: TextRange::new(0, 0),
            error_code: 0,
            program: program.clone(),
            ls: l.clone_handle(),
            diagnostic: None,
            params: None,
        };
        let combined = get_all(&fix_context)?;
        let Some(combined) = combined else { continue };
        if combined.changes.is_empty() {
            continue;
        }
        let kind = lsproto::CodeActionKind::QuickFix;
        let mut changes: HashMap<lsproto::DocumentUri, Vec<lsproto::TextEdit>> = HashMap::new();
        changes.insert(uri.clone(), combined.changes);
        actions.push(lsproto::CommandOrCodeAction {
            code_action: Some(lsproto::CodeAction {
                title: combined.description,
                kind: Some(kind),
                edit: Some(lsproto::WorkspaceEdit { changes }),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
    Ok(actions)
}

pub fn has_multiple_fixable_diagnostics(
    program: &Arc<tsox_compile::compiler::Program>,
    file: &Arc<SourceFile>,
    error_codes: &[i32],
) -> bool {
    let all_diags = crate::ls::diagnostics::get_all_diagnostics(program, file);
    let mut count = 0;
    for d in &all_diags {
        if is_fixable_diagnostic(d, error_codes) {
            count += 1;
            if count >= 2 {
                return true;
            }
        }
    }
    false
}

pub fn is_fixable_diagnostic(diagnostic: &Diagnostic, error_codes: &[i32]) -> bool {
    diagnostic.source().is_empty() && contains_error_code(error_codes, diagnostic.code())
}

pub fn is_fix_all_kind(kind: &lsproto::CodeActionKind) -> bool {
    kind.contains(&lsproto::CodeActionKind::SourceFixAllTs)
}

pub fn wants_quick_fixes(only: Option<&Vec<lsproto::CodeActionKind>>) -> bool {
    let Some(only) = only else { return true };
    if only.is_empty() {
        return true;
    }
    only.iter().any(|kind| kind.contains(&lsproto::CodeActionKind::QuickFix))
}

pub fn get_organize_imports_action_title(kind: &lsproto::CodeActionKind) -> String {
    if *kind == lsproto::CodeActionKind::SourceRemoveUnusedImportsTs {
        tsox_core::diagnostics::REMOVE_UNUSED_IMPORTS.text.to_string()
    } else if *kind == lsproto::CodeActionKind::SourceSortImportsTs {
        tsox_core::diagnostics::SORT_IMPORTS.text.to_string()
    } else {
        tsox_core::diagnostics::ORGANIZE_IMPORTS.text.to_string()
    }
}

pub fn get_organize_imports_actions_for_kind(requested_kind: &lsproto::CodeActionKind) -> Vec<lsproto::CodeActionKind> {
    let organize_imports_kinds = [
        lsproto::CodeActionKind::SourceOrganizeImportsTs,
        lsproto::CodeActionKind::SourceRemoveUnusedImportsTs,
        lsproto::CodeActionKind::SourceSortImportsTs,
    ];
    let mut result: Vec<lsproto::CodeActionKind> = Vec::new();
    for organize_kind in &organize_imports_kinds {
        if requested_kind.contains(organize_kind) {
            result.push(organize_kind.clone());
        }
    }
    if result.iter().any(|k| k == requested_kind) {
        return vec![requested_kind.clone()];
    }
    result
}

impl LanguageService {
    pub fn mig_create_fix_all_action(
        &self,
        program: &Arc<tsox_compile::compiler::Program>,
        file: &Arc<SourceFile>,
        uri: &lsproto::DocumentUri,
    ) -> Result<Option<lsproto::CommandOrCodeAction>, LspError> {
        let kind = lsproto::CodeActionKind::SourceFixAllTs;
        let mut lsp_changes: HashMap<lsproto::DocumentUri, Vec<lsproto::TextEdit>> = HashMap::new();

        for provider in super::m5p_3_support::code_fix_providers() {
            let Some(get_all) = provider.get_all_code_actions.as_ref() else { continue };
            let fix_context = CodeFixContext {
                source_file: file.clone(),
                span: TextRange::new(0, 0),
                error_code: 0,
                program: program.clone(),
                ls: self.clone_handle(),
                diagnostic: None,
                params: None,
            };
            let combined = get_all(&fix_context)?;
            if let Some(combined) = combined {
                if !combined.changes.is_empty() {
                    lsp_changes.entry(uri.clone()).or_default().extend(combined.changes);
                }
            }
        }

        if lsp_changes.is_empty() {
            return Ok(None);
        }

        Ok(Some(lsproto::CommandOrCodeAction {
            code_action: Some(lsproto::CodeAction {
                title: tsox_core::diagnostics::FIX_ALL.text.to_string(),
                kind: Some(kind),
                edit: Some(lsproto::WorkspaceEdit { changes: lsp_changes }),
                ..Default::default()
            }),
            ..Default::default()
        }))
    }

    pub fn mig_create_organize_imports_action(
        &self,
        program: &Arc<tsox_compile::compiler::Program>,
        file: &Arc<SourceFile>,
        kind: &lsproto::CodeActionKind,
    ) -> lsproto::CommandOrCodeAction {
        let title = get_organize_imports_action_title(kind);
        let changes = self.organize_imports(file, program, kind.0);
        if changes.is_empty() {
            return lsproto::CommandOrCodeAction {
                code_action: Some(lsproto::CodeAction {
                    title,
                    kind: Some(kind.clone()),
                    edit: Some(lsproto::WorkspaceEdit { changes: HashMap::new() }),
                    ..Default::default()
                }),
                ..Default::default()
            };
        }
        let mut lsp_changes: HashMap<lsproto::DocumentUri, Vec<lsproto::TextEdit>> = HashMap::new();
        for (file_name, edits) in changes {
            let file_uri = crate::ls::lsconv_converters::file_name_to_document_uri(&file_name);
            lsp_changes.insert(lsproto::DocumentUri(file_uri), edits);
        }
        lsproto::CommandOrCodeAction {
            code_action: Some(lsproto::CodeAction {
                title,
                kind: Some(kind.clone()),
                edit: Some(lsproto::WorkspaceEdit { changes: lsp_changes }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
