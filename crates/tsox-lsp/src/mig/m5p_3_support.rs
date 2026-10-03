#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::ls::language_service::LanguageService;
use crate::lsp::lsproto;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Diagnostic, SourceFile};

#[derive(Debug, Clone, Default)]
pub struct CodeAction {
    pub description: String,
    pub changes: Vec<lsproto::TextEdit>,
    pub fix_id: String,
    pub fix_all_description: String,
}

#[derive(Debug, Clone, Default)]
pub struct CombinedCodeActions {
    pub description: String,
    pub changes: Vec<lsproto::TextEdit>,
}

pub type GetCodeActionsFn = dyn Fn(&CodeFixContext) -> Result<Vec<CodeAction>, crate::mig::m5n::LspError>;
pub type GetAllCodeActionsFn = dyn Fn(&CodeFixContext) -> Result<Option<CombinedCodeActions>, crate::mig::m5n::LspError>;

pub struct CodeFixProvider {
    pub id: usize,
    pub error_codes: Vec<i32>,
    pub get_code_actions: Option<Arc<GetCodeActionsFn>>,
    pub fix_ids: Vec<String>,
    pub get_all_code_actions: Option<Arc<GetAllCodeActionsFn>>,
}

pub struct CodeFixContext {
    pub source_file: Arc<SourceFile>,
    pub span: TextRange,
    pub error_code: i32,
    pub program: Arc<tsox_compile::compiler::Program>,
    pub ls: LanguageService,
    pub diagnostic: Option<lsproto::Diagnostic>,
    pub params: Option<Arc<CodeActionParams>>,
}

/// Go lsproto.CodeActionParams 的最小等价移植；crate::lsp::lsproto 尚无此类型，
/// 合并期归位到 crate::lsp::lsproto。
#[derive(Debug, Clone)]
pub struct CodeActionParams {
    pub text_document: lsproto::TextDocumentIdentifier,
    pub range: lsproto::Range,
    pub context: CodeActionContext,
}

#[derive(Debug, Clone, Default)]
pub struct CodeActionContext {
    pub diagnostics: Vec<lsproto::Diagnostic>,
    pub only: Vec<String>,
}

fn ls_diagnostic(d: &lsproto::Diagnostic) -> crate::ls::types::Diagnostic { ::tsox_core::fntrace::enter("ls_diagnostic"); 
    crate::ls::types::Diagnostic {
        range: d.range.clone(),
        severity: d.severity,
        code: d.code.clone(),
        source: d.source.clone(),
        message: d.message.clone(),
        related_information: None,
    }
}

fn ls_code_action_params(p: &CodeActionParams) -> crate::ls::types::CodeActionParams { ::tsox_core::fntrace::enter("ls_code_action_params"); 
    crate::ls::types::CodeActionParams {
        text_document: crate::ls::types::TextDocumentIdentifier { uri: p.text_document.uri.clone() },
        range: p.range.clone(),
        context: crate::ls::types::CodeActionContext {
            diagnostics: p.context.diagnostics.iter().map(ls_diagnostic).collect(),
            only: p.context.only.clone(),
        },
    }
}

macro_rules! ls_fix_ctx {
    ($ctx:expr, $name:ident) => {
        let diag = $ctx.diagnostic.as_ref().map(ls_diagnostic);
        let params = $ctx.params.as_deref().map(ls_code_action_params);
        #[allow(unused_variables)]
        let $name = crate::ls::code_actions::CodeFixContext {
            source_file: &$ctx.source_file,
            span: $ctx.span,
            error_code: $ctx.error_code,
            program: &$ctx.program,
            ls: &$ctx.ls,
            diagnostic: diag.as_ref(),
            params: params.as_ref(),
        };
    };
}

fn code_action_from_ls(a: crate::ls::types::CodeAction) -> CodeAction { ::tsox_core::fntrace::enter("code_action_from_ls"); 
    CodeAction {
        description: a.title,
        changes: a.edits,
        fix_id: String::new(),
        fix_all_description: String::new(),
    }
}

fn combined_code_actions_from_ls(c: crate::ls::code_actions::CombinedCodeActions) -> CombinedCodeActions { ::tsox_core::fntrace::enter("combined_code_actions_from_ls"); 
    CombinedCodeActions {
        description: c.description,
        changes: c.changes,
    }
}

pub fn code_fix_providers() -> Vec<Arc<CodeFixProvider>> { ::tsox_core::fntrace::enter("code_fix_providers"); 
    vec![
        Arc::new(CodeFixProvider {
            id: 1,
            error_codes: Vec::new(),
            fix_ids: vec![crate::ls::code_actions_import_fixes::IMPORT_FIX_ID.to_string()],
            get_code_actions: Some(Arc::new(|ctx: &CodeFixContext| {
                ls_fix_ctx!(ctx, ls_ctx);
                Ok(ctx
                    .ls
                    .get_import_code_actions(&ls_ctx)
                    .into_iter()
                    .map(code_action_from_ls)
                    .collect())
            })),
            get_all_code_actions: Some(Arc::new(|ctx: &CodeFixContext| {
                ls_fix_ctx!(ctx, ls_ctx);
                Ok(Some(combined_code_actions_from_ls(
                    ctx.ls.get_all_import_code_actions(&ls_ctx),
                )))
            })),
        }),
        Arc::new(CodeFixProvider {
            id: 2,
            error_codes: Vec::new(),
            fix_ids: vec![crate::ls::code_actions_fix_missing_type::FIX_MISSING_TYPE_ANNOTATION_ON_EXPORTS_FIX_ID.to_string()],
            get_code_actions: Some(Arc::new(|ctx: &CodeFixContext| {
                ls_fix_ctx!(ctx, ls_ctx);
                Ok(ctx
                    .ls
                    .get_isolated_declarations_code_actions(&ls_ctx)
                    .into_iter()
                    .map(code_action_from_ls)
                    .collect())
            })),
            get_all_code_actions: Some(Arc::new(|ctx: &CodeFixContext| {
                ls_fix_ctx!(ctx, ls_ctx);
                Ok(Some(combined_code_actions_from_ls(
                    ctx.ls.get_all_isolated_declarations_code_actions(&ls_ctx),
                )))
            })),
        }),
        Arc::new(CodeFixProvider {
            id: 3,
            error_codes: Vec::new(),
            fix_ids: vec![crate::ls::code_actions_fix_implements::FIX_CLASS_INCORRECTLY_IMPLEMENTS_INTERFACE_FIX_ID.to_string()],
            get_code_actions: Some(Arc::new(|ctx: &CodeFixContext| {
                ls_fix_ctx!(ctx, ls_ctx);
                Ok(ctx
                    .ls
                    .get_code_actions_to_fix_class_incorrectly_implements_interface(&ls_ctx)
                    .into_iter()
                    .map(code_action_from_ls)
                    .collect())
            })),
            get_all_code_actions: Some(Arc::new(|ctx: &CodeFixContext| {
                ls_fix_ctx!(ctx, ls_ctx);
                Ok(Some(combined_code_actions_from_ls(
                    ctx.ls.get_all_code_actions_to_fix_class_incorrectly_implements_interface(&ls_ctx),
                )))
            })),
        }),
    ]
}
