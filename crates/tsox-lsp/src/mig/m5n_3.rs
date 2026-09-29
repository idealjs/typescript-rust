#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use super::m5n::{LspError, LspResult, Server, UserFacingRequestFailedError};
use super::m5n_2::CrossProjectOrchestrator;
use crate::ls::language_service::LanguageService;
use crate::ls::lsconv_converters as lsconv;
use crate::ls::lsutil_user_preferences_preferences as lsutil;
use crate::lsp::logger;
use crate::project::project;
use crate::project::snapshot::Snapshot;
use tsox_compile::compiler as compiler;
use tsox_compile::mig::m3l_cm as contentmapper;
use tsox_compile::mig::m3l_cm_2 as cm2;
use tsox_core::diagnostics::Message;
use tsox_core::pprof::mig::m6a as pprof;
use tsox_core::tspath;
use tsox_tsoptions::mig::m5h_2 as tsoptions;

use crate::ls as ls;
use crate::mig::m5z_2 as ipc;

/// 本文件内引用的 lsproto 缺失类型：Go lsproto 包的最小等价移植，
/// 统一收口在本地 mod 以复用 `lsproto::` 路径写法；合并期归位到 crate::lsp::lsproto。
mod lsproto {
    use super::integer_or_string_code_as_string;
    use serde_json::Value;
    use std::collections::HashMap;

    pub use crate::lsp::lsproto::*;

    pub use crate::mig::m5n::DiagnosticFlakeLogLevel;
    pub use crate::mig::m5n::ErrorCode;

    pub struct OrNull<T>(pub Option<T>);
    pub struct OrNullArray<T>(pub Option<Vec<T>>);

    #[derive(Debug, Clone, Default)]
    pub struct TextDocumentIdentifier {
        pub uri: crate::lsp::lsproto::DocumentUri,
    }

    #[derive(Debug, Clone, Default)]
    pub struct VersionedTextDocumentIdentifier {
        pub uri: crate::lsp::lsproto::DocumentUri,
        pub version: i32,
    }

    #[derive(Debug, Clone, Default)]
    pub struct OptionalVersionedTextDocumentIdentifier {
        pub uri: crate::lsp::lsproto::DocumentUri,
        pub version: Option<i32>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct TextDocumentItem {
        pub uri: crate::lsp::lsproto::DocumentUri,
        pub language_id: crate::lsp::lsproto::LanguageKind,
        pub version: i32,
        pub text: String,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DidChangeConfigurationParams {
        pub settings: Option<Value>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DidOpenTextDocumentParams {
        pub text_document: TextDocumentItem,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DidChangeTextDocumentParams {
        pub text_document: VersionedTextDocumentIdentifier,
        pub content_changes: Vec<crate::lsp::lsproto::TextDocumentContentChangePartialOrWholeDocument>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DidSaveTextDocumentParams {
        pub text_document: TextDocumentIdentifier,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DidCloseTextDocumentParams {
        pub text_document: TextDocumentIdentifier,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DidChangeWatchedFilesParams {
        pub changes: Vec<crate::lsp::lsproto::FileEvent>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct SetTraceParams {}

    #[derive(Debug, Clone, Default)]
    pub struct SetLogVerbosityParams {
        pub verbosity: crate::lsp::lsproto::LogVerbosity,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DocumentDiagnosticParams {
        pub text_document: TextDocumentIdentifier,
    }

    #[derive(Debug, Clone, Default)]
    pub struct FullDocumentDiagnosticReport {
        pub items: Vec<crate::ls::types::Diagnostic>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DocumentDiagnosticResponse {
        pub full_document_diagnostic_report: FullDocumentDiagnosticReport,
    }

    #[derive(Debug, Clone, Default)]
    pub struct HoverParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
    }

    pub type HoverResponse = OrNull<crate::ls::types::Hover>;

    #[derive(Debug, Clone, Default)]
    pub struct PrepareRenameParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
    }

    #[derive(Debug, Clone)]
    pub struct PrepareRenamePlaceholder {
        pub range: crate::lsp::lsproto::Range,
        pub placeholder: String,
    }

    pub enum PrepareRenameResponse {
        Placeholder(PrepareRenamePlaceholder),
        Range(crate::lsp::lsproto::Range),
        Null,
    }

    #[derive(Debug, Clone, Default)]
    pub struct RenameParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
        pub new_name: String,
    }

    pub enum RenameResponse {
        WorkspaceEdit(WorkspaceEdit),
        Null,
    }

    #[derive(Debug, Clone, Default)]
    pub struct WorkspaceEdit {
        pub changes: Option<HashMap<String, Vec<crate::lsp::lsproto::TextEdit>>>,
        pub document_changes: Option<Vec<TextDocumentEditOrCreateFileOrRenameFileOrDeleteFile>>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct TextDocumentEditOrCreateFileOrRenameFileOrDeleteFile {
        pub text_document_edit: Option<TextDocumentEdit>,
        pub rename_file: Option<RenameFile>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct TextDocumentEdit {
        pub text_document: OptionalVersionedTextDocumentIdentifier,
        pub edits: Vec<TextEditOrAnnotatedTextEditOrSnippetTextEdit>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct TextEditOrAnnotatedTextEditOrSnippetTextEdit {
        pub text_edit: Option<crate::lsp::lsproto::TextEdit>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct StringLiteralRename {}

    #[derive(Debug, Clone, Default)]
    pub struct RenameFile {
        pub kind: StringLiteralRename,
        pub old_uri: crate::lsp::lsproto::DocumentUri,
        pub new_uri: crate::lsp::lsproto::DocumentUri,
    }

    #[derive(Debug, Clone, Default)]
    pub struct RenameFilesParams {
        pub files: Vec<FileRename>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct FileRename {
        pub old_uri: String,
        pub new_uri: String,
    }

    #[derive(Debug, Clone, Default)]
    pub struct WillRenameFilesResponse {
        pub workspace_edit: Option<WorkspaceEdit>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct SignatureHelpContext {
        pub is_retrigger: bool,
        pub trigger_kind: i32,
        pub trigger_character: Option<String>,
        pub active_signature_help: Option<crate::ls::types::SignatureHelp>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct SignatureHelpParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
        pub context: Option<SignatureHelpContext>,
    }

    pub type SignatureHelpResponse = OrNull<crate::ls::types_symbols::SignatureHelp>;

    #[derive(Debug, Clone, Default)]
    pub struct FoldingRangeParams {
        pub text_document: TextDocumentIdentifier,
    }

    pub type FoldingRangeResponse = OrNullArray<crate::ls::types::FoldingRange>;

    #[derive(Debug, Clone, Default)]
    pub struct VSOnAutoInsertParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
        pub ch: String,
        pub options: crate::lsp::lsproto::FormattingOptions,
    }

    pub type VSOnAutoInsertResponse = OrNull<crate::ls::auto_insert::VsOnAutoInsertResponseItem>;

    #[derive(Debug, Clone, Default)]
    pub struct LinkedEditingRangeParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
    }

    pub type LinkedEditingRangeResponse = OrNull<crate::ls::types_highlight::LinkedEditingRanges>;

    #[derive(Debug, Clone, Default)]
    pub struct DefinitionParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
    }

    pub type DefinitionResponse = OrNullArray<crate::ls::types::LocationLink>;

    pub type CustomTextDocumentSourceDefinitionResponse = OrNullArray<crate::ls::types::LocationLink>;


    #[derive(Debug, Clone, Default)]
    pub struct TypeDefinitionParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
    }

    pub type TypeDefinitionResponse = OrNullArray<crate::ls::types::LocationLink>;

    #[derive(Debug, Clone, Default)]
    pub struct CompletionParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
        pub context: Option<crate::ls::types_completion::CompletionContext>,
    }

    pub type CompletionResponse = OrNull<crate::ls::types_completion::CompletionList>;

    pub use crate::ls::mig::m5q_3::lsproto::CompletionItem;
    pub use crate::ls::mig::m5q_3::lsproto::CompletionItemData;

    pub type CompletionResolveResponse = OrNull<CompletionItem>;

    #[derive(Debug, Clone, Default)]
    pub struct DocumentFormattingParams {
        pub text_document: TextDocumentIdentifier,
        pub options: crate::lsp::lsproto::FormattingOptions,
    }

    pub type DocumentFormattingResponse = OrNullArray<crate::lsp::lsproto::TextEdit>;

    #[derive(Debug, Clone, Default)]
    pub struct DocumentRangeFormattingParams {
        pub text_document: TextDocumentIdentifier,
        pub options: crate::lsp::lsproto::FormattingOptions,
        pub range: crate::lsp::lsproto::Range,
    }

    pub type DocumentRangeFormattingResponse = OrNullArray<crate::lsp::lsproto::TextEdit>;

    #[derive(Debug, Clone, Default)]
    pub struct DocumentOnTypeFormattingParams {
        pub text_document: TextDocumentIdentifier,
        pub options: crate::lsp::lsproto::FormattingOptions,
        pub position: crate::lsp::lsproto::Position,
        pub ch: String,
    }

    pub type DocumentOnTypeFormattingResponse = OrNullArray<crate::lsp::lsproto::TextEdit>;

    #[derive(Debug, Clone, Default)]
    pub struct WorkspaceSymbolParams {
        pub text_document: Option<TextDocumentIdentifier>,
        pub query: String,
    }

    pub type WorkspaceSymbolResponse = OrNullArray<Value>;

    #[derive(Debug, Clone, Default)]
    pub struct DocumentSymbolParams {
        pub text_document: TextDocumentIdentifier,
    }

    pub type DocumentSymbolResponse = OrNullArray<crate::ls::types::DocumentSymbol>;

    #[derive(Debug, Clone, Default)]
    pub struct DocumentHighlightParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
    }

    pub type DocumentHighlightResponse = OrNullArray<crate::ls::types_highlight::DocumentHighlight>;

    #[derive(Debug, Clone, Default)]
    pub struct MultiDocumentHighlightParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
        pub files_to_search: Vec<crate::lsp::lsproto::DocumentUri>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CustomMultiDocumentHighlightResponse {
        pub multi_document_highlights: Option<Vec<crate::ls::types_highlight::MultiDocumentHighlight>>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct SelectionRangeParams {
        pub text_document: TextDocumentIdentifier,
        pub positions: Vec<crate::lsp::lsproto::Position>,
    }

    pub type SelectionRangeResponse = OrNullArray<crate::ls::types_highlight::SelectionRange>;

    #[derive(Debug, Clone, Default)]
    pub struct CodeActionParams {
        pub text_document: TextDocumentIdentifier,
        pub range: crate::lsp::lsproto::Range,
        pub context: crate::ls::types_edits::CodeActionContext,
    }

    pub type CodeActionResponse = OrNullArray<crate::ls::types::CodeAction>;

    #[derive(Debug, Clone, Default)]
    pub struct InlayHintParams {
        pub text_document: TextDocumentIdentifier,
        pub range: crate::lsp::lsproto::Range,
    }

    pub type InlayHintResponse = OrNullArray<crate::ls::types::InlayHint>;

    #[derive(Debug, Clone, Default)]
    pub struct CodeLensParams {
        pub text_document: TextDocumentIdentifier,
    }

    pub type CodeLensResponse = OrNullArray<crate::ls::types::CodeLens>;

    #[derive(Debug, Clone, Default)]
    pub struct CodeLens {
        pub range: crate::lsp::lsproto::Range,
        pub data: CodeLensData,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CodeLensData {
        pub uri: crate::lsp::lsproto::DocumentUri,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CallHierarchyPrepareParams {
        pub text_document: TextDocumentIdentifier,
        pub position: crate::lsp::lsproto::Position,
    }

    pub type CallHierarchyPrepareResponse = OrNullArray<crate::ls::call_hierarchy::CallHierarchyDeclaration>;

    #[derive(Debug, Clone)]
    pub struct CallHierarchyIncomingCallsParams {
        pub item: crate::lsp::lsproto::TextDocumentPositionParams,
    }

    pub type CallHierarchyIncomingCallsResponse = OrNullArray<crate::ls::call_hierarchy::CallHierarchyIncomingCall>;

    #[derive(Debug, Clone)]
    pub struct CallHierarchyOutgoingCallsParams {
        pub item: crate::lsp::lsproto::TextDocumentPositionParams,
    }

    pub type CallHierarchyOutgoingCallsResponse = OrNullArray<crate::ls::call_hierarchy::CallHierarchyOutgoingCall>;

    #[derive(Debug, Clone, Default)]
    pub struct SemanticTokensParams {
        pub text_document: TextDocumentIdentifier,
    }

    pub type SemanticTokensResponse = OrNull<crate::ls::types::SemanticTokens>;

    #[derive(Debug, Clone, Default)]
    pub struct SemanticTokensRangeParams {
        pub text_document: TextDocumentIdentifier,
        pub range: crate::lsp::lsproto::Range,
    }

    pub type SemanticTokensRangeResponse = OrNull<crate::ls::types::SemanticTokens>;

    #[derive(Debug, Clone, Default)]
    pub struct InitializeAPISessionParams {
        pub pipe: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct InitializeAPISessionResult {
        pub session_id: String,
        pub pipe: String,
    }

    pub enum CustomInitializeAPISessionResponse {
        InitializeAPISessionResult(InitializeAPISessionResult),
        Null,
    }

    pub enum RunGCResponse {
        Null,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ProfileParams {
        pub dir: String,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ProfileResult {
        pub file: String,
    }

    pub enum StartCPUProfileResponse {
        Null,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ProjectInfoParams {
        pub text_document: TextDocumentIdentifier,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ProjectInfoResult {
        pub config_file_path: String,
    }

    pub enum CustomProjectInfoResponse {
        ProjectInfoResult(ProjectInfoResult),
        Null,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ContentMapperContribution {
        pub contributor_id: String,
        pub extensions: Vec<String>,
        pub inferred_project_contribution: Option<InferredProjectContribution>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct InferredProjectContribution {
        pub manifest: Manifest,
        pub options: Option<Value>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct Manifest {
        pub name: String,
        pub version: Option<String>,
        pub exec: Vec<String>,
        pub cwd: Option<String>,
        pub compiler_options: Option<Vec<String>>,
        pub dynamic_config: Option<bool>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct SetContentMapperContributionsParams {
        pub contributions: Vec<ContentMapperContribution>,
        pub open_documents: Vec<TextDocumentIdentifier>,
    }

    pub enum CustomSetContentMapperContributionsResponse {
        Null,
    }

    pub fn compare_diagnostics(
        pre: &[crate::ls::types::Diagnostic],
        post: &[crate::ls::types::Diagnostic],
    ) -> (Vec<crate::ls::types::Diagnostic>, Vec<crate::ls::types::Diagnostic>) {
        fn key(d: &crate::ls::types::Diagnostic) -> String {
            serde_json::to_string(d).unwrap_or_default()
        }
        let pre_keys: std::collections::HashSet<String> = pre.iter().map(key).collect();
        let post_keys: std::collections::HashSet<String> = post.iter().map(key).collect();
        let missing_from_pre = pre
            .iter()
            .filter(|d| !post_keys.contains(&key(d)))
            .cloned()
            .collect();
        let missing_from_post = post
            .iter()
            .filter(|d| !pre_keys.contains(&key(d)))
            .cloned()
            .collect();
        (missing_from_pre, missing_from_post)
    }

    pub fn diagnostic_code_as_string(d: &crate::ls::types::Diagnostic) -> String {
        format!("Code({})", integer_or_string_code_as_string(&d.code))
    }

    pub fn diagnostic_as_string(d: &crate::ls::types::Diagnostic) -> String {
        format!(
            "{} ({}:{}-{}:{}): {}",
            integer_or_string_code_as_string(&d.code),
            d.range.start.line,
            d.range.start.character,
            d.range.end.line,
            d.range.end.character,
            d.message
        )
    }
}

fn integer_or_string_code_as_string(code: &Option<Value>) -> String {
    match code {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => "-1".to_string(),
    }
}

impl Server {
    pub fn handle_did_change_workspace_configuration(&self, params: &lsproto::DidChangeConfigurationParams) -> LspResult<()> {
        let Some(settings) = &params.settings else {
            return Ok(());
        };
        if let Some(settings_map) = settings.as_object() {
            self.session
                .as_ref()
                .unwrap()
                .configure(lsutil::parse_user_preferences(settings_map));
        }
        Ok(())
    }

    pub fn handle_did_open(&self, params: &lsproto::DidOpenTextDocumentParams) -> LspResult<()> {
        self.session.as_ref().unwrap().did_open_file(
            &params.text_document.uri,
            params.text_document.version,
            &params.text_document.text,
            &params.text_document.language_id,
        );
        Ok(())
    }

    pub fn handle_did_change(&self, params: &lsproto::DidChangeTextDocumentParams) -> LspResult<()> {
        self.session.as_ref().unwrap().did_change_file(
            &params.text_document.uri,
            params.text_document.version,
            &params.content_changes,
        );
        Ok(())
    }

    pub fn handle_did_save(&self, params: &lsproto::DidSaveTextDocumentParams) -> LspResult<()> {
        self.session.as_ref().unwrap().did_save_file(&params.text_document.uri);
        Ok(())
    }

    pub fn handle_did_close(&self, params: &lsproto::DidCloseTextDocumentParams) -> LspResult<()> {
        self.session.as_ref().unwrap().did_close_file(&params.text_document.uri);
        Ok(())
    }

    pub fn handle_did_change_watched_files(&self, params: &lsproto::DidChangeWatchedFilesParams) -> LspResult<()> {
        self.session
            .as_ref()
            .unwrap()
            .did_change_watched_files(&params.changes);
        Ok(())
    }

    pub fn handle_set_trace(&self, _params: &lsproto::SetTraceParams) -> LspResult<()> {
        Ok(())
    }

    pub fn handle_set_log_verbosity(&self, params: &lsproto::SetLogVerbosityParams) -> LspResult<()> {
        if !logger::is_valid_log_verbosity(params.verbosity) {
            return Err(LspError::new(
                lsproto::ErrorCode::InvalidParams,
                format!("invalid log verbosity {}", params.verbosity),
            ));
        }
        self.logger.set_verbosity(params.verbosity);
        Ok(())
    }

    pub fn handle_document_diagnostic(
        &self,
        language_service: &LanguageService,
        params: &lsproto::DocumentDiagnosticParams,
    ) -> LspResult<lsproto::DocumentDiagnosticResponse> {
        let provide = || lsproto::DocumentDiagnosticResponse {
            full_document_diagnostic_report: lsproto::FullDocumentDiagnosticReport {
                items: language_service.provide_diagnostics(&params.text_document.uri),
            },
        };
        if self.flake_logging == lsproto::DiagnosticFlakeLogLevel::Off {
            return Ok(provide());
        }
        let direct = provide();
        language_service
            .get_program()
            .emit(&|_file_name, _text| Ok(()));
        let secondary = provide();
        let (missing_from_pre, missing_from_post) = lsproto::compare_diagnostics(
            &direct.full_document_diagnostic_report.items,
            &secondary.full_document_diagnostic_report.items,
        );
        if missing_from_pre.is_empty() && missing_from_post.is_empty() {
            return Ok(direct);
        }
        let diff = generate_diagnostic_diff_string(
            &missing_from_pre,
            &missing_from_post,
            &lsproto::diagnostic_as_string,
        );
        self.logger.error(&diff);
        if self.telemetry_enabled() {
            let sanitized_diff = generate_diagnostic_diff_string(
                &missing_from_pre,
                &missing_from_post,
                &lsproto::diagnostic_code_as_string,
            );
            let event = crate::mig::m5n::RequestFailureTelemetryEvent {
                properties: Some(crate::mig::m5n::RequestFailureTelemetryProperties {
                    error_code: lsproto::ErrorCode::InternalError.to_string(),
                    request_method: "textDocument.diagnostic.flakeLog".to_string(),
                    stack: sanitized_diff,
                }),
            };
            let _ = super::m5n_2::send_notification(
                self,
                &*crate::mig::m5n::TELEMETRY_EVENT_INFO,
                &event,
            );
        }
        if self.flake_logging == lsproto::DiagnosticFlakeLogLevel::Panic {
            panic!("flaky diagnostic(s) logged:\n{diff}");
        }
        Ok(direct)
    }

    pub fn handle_hover(
        &self,
        ls: &LanguageService,
        params: &lsproto::HoverParams,
    ) -> LspResult<lsproto::HoverResponse> {
        Ok(lsproto::OrNull(
            ls.provide_hover(&params.text_document.uri, params.position.clone()),
        ))
    }

    pub fn handle_prepare_rename(
        &self,
        language_service: &LanguageService,
        params: &lsproto::PrepareRenameParams,
    ) -> LspResult<lsproto::PrepareRenameResponse> {
        let info = language_service.get_rename_info("", &params.text_document.uri, params.position.clone());
        if !info.can_rename {
            return Err(LspError::new(
                lsproto::ErrorCode::RequestFailed,
                info.localized_error_message.clone(),
            ));
        }
        Ok(lsproto::PrepareRenameResponse::Placeholder(lsproto::PrepareRenamePlaceholder {
            range: info.trigger_span,
            placeholder: info.display_name,
        }))
    }

    pub fn handle_rename(
        &self,
        params: &lsproto::RenameParams,
        req: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::RenameResponse> {
        let (default_ls, _orchestrator) =
            self.get_language_service_and_cross_project_orchestrator(&params.text_document.uri, req)?;
        let info = default_ls.get_rename_info(&params.new_name, &params.text_document.uri, params.position.clone());
        if info.can_rename && !info.file_to_rename.is_empty() {
            if self.client_supports_will_rename_files() {
                return Ok(lsproto::RenameResponse::WorkspaceEdit(lsproto::WorkspaceEdit {
                    document_changes: Some(vec![lsproto::TextDocumentEditOrCreateFileOrRenameFileOrDeleteFile {
                        rename_file: Some(lsproto::RenameFile {
                            kind: lsproto::StringLiteralRename {},
                            old_uri: crate::lsp::lsproto::DocumentUri(lsconv::file_name_to_document_uri(&info.file_to_rename)),
                            new_uri: crate::lsp::lsproto::DocumentUri(lsconv::file_name_to_document_uri(&info.new_file_name)),
                        }),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }));
            }
            let rename_files_params = lsproto::RenameFilesParams {
                files: vec![lsproto::FileRename {
                    old_uri: lsconv::file_name_to_document_uri(&info.file_to_rename).to_string(),
                    new_uri: lsconv::file_name_to_document_uri(&info.new_file_name).to_string(),
                }],
            };
            return self.handle_will_rename_files_worker(&rename_files_params, true).map(|resp| {
                match resp.workspace_edit {
                    Some(edit) => lsproto::RenameResponse::WorkspaceEdit(edit),
                    None => lsproto::RenameResponse::Null,
                }
            });
        }
        let ls_params = crate::ls::types::RenameParams {
            text_document: crate::ls::types::TextDocumentIdentifier {
                uri: params.text_document.uri.clone(),
            },
            position: params.position.clone(),
            new_name: params.new_name.clone(),
        };
        match default_ls.provide_rename(&ls_params, None) {
            Some(edit) => Ok(lsproto::RenameResponse::WorkspaceEdit(lsproto::WorkspaceEdit {
                changes: edit
                    .changes
                    .map(|c| c.into_iter().map(|(uri, edits)| (uri.0, edits)).collect()),
                document_changes: None,
            })),
            None => Ok(lsproto::RenameResponse::Null),
        }
    }

    pub fn handle_will_rename_files(
        &self,
        params: &lsproto::RenameFilesParams,
        msg: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::WillRenameFilesResponse> {
        self.handle_will_rename_files_worker(params, false)
    }

    pub fn handle_will_rename_files_worker(
        &self,
        params: &lsproto::RenameFilesParams,
        send_rename_file: bool,
    ) -> LspResult<lsproto::WillRenameFilesResponse> {
        if params.files.is_empty() {
            return Ok(lsproto::WillRenameFilesResponse::default());
        }
        let uris: Vec<crate::lsp::lsproto::DocumentUri> = params
            .files
            .iter()
            .map(|f| crate::lsp::lsproto::DocumentUri(f.old_uri.clone()))
            .collect();
        let services = self
            .session
            .as_ref()
            .unwrap()
            .get_language_services_for_documents_loading_project_tree(&uris);
        let mut seen_edits: HashMap<String, String> = HashMap::new();
        let mut seen_renames: HashMap<String, bool> = HashMap::new();
        let mut document_changes: Vec<lsproto::TextDocumentEditOrCreateFileOrRenameFileOrDeleteFile> = Vec::new();
        for language_service in &services {
            for file in &params.files {
                let changes = language_service.get_edits_for_file_rename(
                    &crate::lsp::lsproto::DocumentUri(file.old_uri.clone()),
                    &crate::lsp::lsproto::DocumentUri(file.new_uri.clone()),
                );
                for change in changes {
                    if !seen_renames.get(&change.old_uri.0).copied().unwrap_or(false) {
                        seen_renames.insert(change.old_uri.0.clone(), true);
                        document_changes.push(lsproto::TextDocumentEditOrCreateFileOrRenameFileOrDeleteFile {
                            rename_file: Some(lsproto::RenameFile {
                                kind: lsproto::StringLiteralRename {},
                                old_uri: change.old_uri.clone(),
                                new_uri: change.new_uri.clone(),
                            }),
                            ..Default::default()
                        });
                    }
                }
            }
        }
        if send_rename_file {
            for file in &params.files {
                document_changes.push(lsproto::TextDocumentEditOrCreateFileOrRenameFileOrDeleteFile {
                    rename_file: Some(lsproto::RenameFile {
                        kind: lsproto::StringLiteralRename {},
                        old_uri: crate::lsp::lsproto::DocumentUri(file.old_uri.clone()),
                        new_uri: crate::lsp::lsproto::DocumentUri(file.new_uri.clone()),
                    }),
                    ..Default::default()
                });
            }
        }
        if document_changes.is_empty() {
            return Ok(lsproto::WillRenameFilesResponse::default());
        }
        if self.client_supports_document_changes() {
            return Ok(lsproto::WillRenameFilesResponse {
                workspace_edit: Some(lsproto::WorkspaceEdit {
                    document_changes: Some(document_changes),
                    ..Default::default()
                }),
            });
        }
        let mut changes: HashMap<String, Vec<lsproto::TextEdit>> = HashMap::new();
        for change in &document_changes {
            if let Some(text_document_edit) = &change.text_document_edit {
                let uri = text_document_edit.text_document.uri.clone();
                for edit in &text_document_edit.edits {
                    if let Some(text_edit) = &edit.text_edit {
                        changes.entry(uri.0.clone()).or_default().push(text_edit.clone());
                    }
                }
            }
        }
        Ok(lsproto::WillRenameFilesResponse {
            workspace_edit: Some(lsproto::WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }),
        })
    }

    pub fn handle_signature_help(
        &self,
        language_service: &LanguageService,
        params: &lsproto::SignatureHelpParams,
    ) -> LspResult<lsproto::SignatureHelpResponse> {
        let ls_context = params.context.as_ref().map(|c| crate::ls::types::SignatureHelpContext {
            trigger_kind: c.trigger_kind as u32,
            trigger_character: c.trigger_character.clone(),
            is_retrigger: c.is_retrigger,
        });
        let context = ls_context.unwrap_or_default();
        Ok(lsproto::OrNull(
            language_service.provide_signature_help(&params.text_document.uri, params.position.clone(), &context),
        ))
    }

    pub fn handle_folding_range(
        &self,
        ls: &LanguageService,
        params: &lsproto::FoldingRangeParams,
    ) -> LspResult<lsproto::FoldingRangeResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_folding_range(&params.text_document.uri),
        )))
    }

    pub fn handle_vs_on_auto_insert(
        &self,
        ls: &LanguageService,
        params: &lsproto::VSOnAutoInsertParams,
    ) -> LspResult<lsproto::VSOnAutoInsertResponse> {
        let ls_params = crate::ls::types::VsOnAutoInsertParams {
            text_document: crate::ls::types::TextDocumentIdentifier {
                uri: params.text_document.uri.clone(),
            },
            position: params.position.clone(),
            ch: params.ch.clone(),
        };
        Ok(lsproto::OrNull(ls.provide_on_auto_insert(&ls_params)))
    }

    pub fn handle_linked_editing_range(
        &self,
        ls: &LanguageService,
        params: &lsproto::LinkedEditingRangeParams,
    ) -> LspResult<lsproto::LinkedEditingRangeResponse> {
        let ls_params = crate::ls::types::LinkedEditingRangeParams {
            text_document: crate::ls::types::TextDocumentIdentifier {
                uri: params.text_document.uri.clone(),
            },
            position: params.position.clone(),
        };
        Ok(lsproto::OrNull(ls.provide_linked_editing_range(&ls_params)))
    }

    pub fn handle_definition(
        &self,
        ls: &LanguageService,
        params: &lsproto::DefinitionParams,
    ) -> LspResult<lsproto::DefinitionResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_definition(&params.text_document.uri, params.position.clone()),
        )))
    }

    pub fn handle_source_definition(
        &self,
        ls: &LanguageService,
        params: &lsproto::TextDocumentPositionParams,
    ) -> LspResult<lsproto::CustomTextDocumentSourceDefinitionResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_source_definition(&params.text_document.uri, params.position.clone()),
        )))
    }

    pub fn handle_type_definition(
        &self,
        ls: &LanguageService,
        params: &lsproto::TypeDefinitionParams,
    ) -> LspResult<lsproto::TypeDefinitionResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_type_definition(&params.text_document.uri, params.position.clone()),
        )))
    }

    pub fn handle_completion(
        &self,
        language_service: &LanguageService,
        params: &lsproto::CompletionParams,
    ) -> LspResult<lsproto::CompletionResponse> {
        Ok(lsproto::OrNull(Some(
            language_service.provide_completion(
                &params.text_document.uri,
                params.position.clone(),
                params.context.as_ref().unwrap(),
            ),
        )))
    }

    pub fn handle_completion_item_resolve(
        &self,
        params: &lsproto::CompletionItem,
        req_msg: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::CompletionResolveResponse> {
        let Some(data) = &params.data else {
            return Err(LspError::new(lsproto::ErrorCode::InternalError, "completion item data is nil"));
        };
        let language_service = self
            .session
            .as_ref()
            .unwrap()
            .get_language_service(&crate::lsp::lsproto::DocumentUri(
                lsconv::file_name_to_document_uri(&data.file_name),
            ))
            .ok_or_else(|| LspError::new(lsproto::ErrorCode::InternalError, "no language service"))?;
        language_service
            .resolve_completion_item(params, data)
            .map(|item| lsproto::OrNull(Some(item)))
            .map_err(|e| LspError::new(lsproto::ErrorCode::InternalError, e))
    }

    pub fn handle_document_format(
        &self,
        ls: &LanguageService,
        params: &lsproto::DocumentFormattingParams,
    ) -> LspResult<lsproto::DocumentFormattingResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_format_document(&params.text_document.uri, &params.options),
        )))
    }

    pub fn handle_document_range_format(
        &self,
        ls: &LanguageService,
        params: &lsproto::DocumentRangeFormattingParams,
    ) -> LspResult<lsproto::DocumentRangeFormattingResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_format_document_range(&params.text_document.uri, &params.options, params.range.clone()),
        )))
    }

    pub fn handle_document_on_type_format(
        &self,
        ls: &LanguageService,
        params: &lsproto::DocumentOnTypeFormattingParams,
    ) -> LspResult<lsproto::DocumentOnTypeFormattingResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_format_document_on_type(&params.text_document.uri, &params.options, params.position.clone(), &params.ch),
        )))
    }

    pub fn handle_workspace_symbol(
        &self,
        params: &lsproto::WorkspaceSymbolParams,
        _req_msg: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::WorkspaceSymbolResponse> {
        let session = self.session.as_ref().unwrap();
        let mut programs: Vec<Arc<compiler::Program>> = Vec::new();
        if let Some(text_document) = &params.text_document {
            if workspace_symbols_scope_is_current_project(session) {
                let uri = text_document.uri.clone();
                session.with_snapshot_for_document(&uri, |snapshot| {
                    programs = snapshot
                        .get_projects_containing_file(&uri)
                        .iter()
                        .filter_map(|p| p.get_program().cloned())
                        .collect();
                });
                return provide_workspace_symbols_response(session, &programs, &params.query);
            }
        }
        session.with_snapshot_loading_project_tree(
            std::collections::HashSet::new(),
            &mut |snapshot: &std::sync::Arc<crate::project::snapshot::Snapshot>| {
            programs = snapshot
                .project_collection
                .as_deref()
                .map(|c| c.projects())
                .unwrap_or_default()
                .iter()
                .filter_map(|p| p.get_program().cloned())
                .collect();
            },
        );
        provide_workspace_symbols_response(session, &programs, &params.query)
    }

    pub fn handle_document_symbol(
        &self,
        ls: &LanguageService,
        params: &lsproto::DocumentSymbolParams,
    ) -> LspResult<lsproto::DocumentSymbolResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_document_symbols(&params.text_document.uri),
        )))
    }

    pub fn handle_document_highlight(
        &self,
        ls: &LanguageService,
        params: &lsproto::DocumentHighlightParams,
    ) -> LspResult<lsproto::DocumentHighlightResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_document_highlights(&params.text_document.uri, params.position.clone()),
        )))
    }

    pub fn handle_multi_document_highlight(
        &self,
        ls: &LanguageService,
        params: &lsproto::MultiDocumentHighlightParams,
    ) -> LspResult<lsproto::CustomMultiDocumentHighlightResponse> {
        let highlights =
            ls.provide_multi_document_highlights(&params.text_document.uri, params.position.clone(), &params.files_to_search);
        Ok(lsproto::CustomMultiDocumentHighlightResponse {
            multi_document_highlights: if highlights.is_empty() { None } else { Some(highlights) },
        })
    }

    pub fn handle_selection_range(
        &self,
        ls: &LanguageService,
        params: &lsproto::SelectionRangeParams,
    ) -> LspResult<lsproto::SelectionRangeResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_selection_ranges(&params.text_document.uri, &params.positions),
        )))
    }

    pub fn handle_code_action(
        &self,
        ls: &LanguageService,
        params: &lsproto::CodeActionParams,
    ) -> LspResult<lsproto::CodeActionResponse> {
        let ls_params = crate::ls::types::CodeActionParams {
            text_document: crate::ls::types::TextDocumentIdentifier {
                uri: params.text_document.uri.clone(),
            },
            range: params.range.clone(),
            context: params.context.clone(),
        };
        Ok(lsproto::OrNullArray(Some(ls.provide_code_actions(&ls_params))))
    }

    pub fn handle_inlay_hint(
        &self,
        language_service: &LanguageService,
        params: &lsproto::InlayHintParams,
    ) -> LspResult<lsproto::InlayHintResponse> {
        Ok(lsproto::OrNullArray(Some(
            language_service.provide_inlay_hint(&params.text_document.uri),
        )))
    }

    pub fn handle_code_lens(
        &self,
        ls: &LanguageService,
        params: &lsproto::CodeLensParams,
    ) -> LspResult<lsproto::CodeLensResponse> {
        Ok(lsproto::OrNullArray(Some(
            ls.provide_code_lenses(&params.text_document.uri),
        )))
    }

    pub fn handle_code_lens_resolve(
        &self,
        code_lens: &lsproto::CodeLens,
        req_msg: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::CodeLens> {
        let Ok((default_ls, _orchestrator)) =
            self.get_language_service_and_cross_project_orchestrator(&code_lens.data.uri, req_msg)
        else {
            return Err(LspError::new(
                lsproto::ErrorCode::ContentModified,
                "content modified",
            ));
        };
        let ls_lens = crate::ls::types::CodeLens {
            range: code_lens.range.clone(),
            command: None,
            data: Some(crate::ls::types::CodeLensData {
                uri: code_lens.data.uri.clone(),
                kind: String::new(),
            }),
        };
        let resolved = default_ls.resolve_code_lens(
            &ls_lens,
            self.initialization_options.code_lens_show_locations_command_name.as_deref(),
            None,
        );
        Ok(lsproto::CodeLens {
            range: resolved.range,
            data: lsproto::CodeLensData {
                uri: resolved
                    .data
                    .map(|d| d.uri)
                    .unwrap_or_else(|| code_lens.data.uri.clone()),
            },
        })
    }

    pub fn handle_prepare_call_hierarchy(
        &self,
        language_service: &LanguageService,
        params: &lsproto::CallHierarchyPrepareParams,
    ) -> LspResult<lsproto::CallHierarchyPrepareResponse> {
        Ok(lsproto::OrNullArray(Some(
            language_service.prepare_call_hierarchy(&params.text_document.uri, params.position.clone()),
        )))
    }

    pub fn handle_call_hierarchy_incoming_calls(
        &self,
        params: &lsproto::CallHierarchyIncomingCallsParams,
        req_msg: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::CallHierarchyIncomingCallsResponse> {
        let (default_ls, _orchestrator) =
            self.get_language_service_and_cross_project_orchestrator(&params.item.text_document.uri, req_msg)?;
        Ok(lsproto::OrNullArray(Some(
            default_ls.provide_call_hierarchy_incoming_calls(
                &params.item.text_document.uri,
                params.item.position.clone(),
            ),
        )))
    }

    pub fn handle_call_hierarchy_outgoing_calls(
        &self,
        params: &lsproto::CallHierarchyOutgoingCallsParams,
        _req_msg: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::CallHierarchyOutgoingCallsResponse> {
        let language_service = self
            .session
            .as_ref()
            .unwrap()
            .get_language_service(&params.item.text_document.uri)
            .ok_or_else(|| LspError::new(lsproto::ErrorCode::InternalError, "no language service"))?;
        Ok(lsproto::OrNullArray(Some(
            language_service.provide_call_hierarchy_outgoing_calls(
                &params.item.text_document.uri,
                params.item.position.clone(),
            ),
        )))
    }

    pub fn handle_semantic_tokens_full(
        &self,
        ls: &LanguageService,
        params: &lsproto::SemanticTokensParams,
    ) -> LspResult<lsproto::SemanticTokensResponse> {
        Ok(lsproto::OrNull(
            ls.provide_semantic_tokens(&params.text_document.uri),
        ))
    }

    pub fn handle_semantic_tokens_range(
        &self,
        ls: &LanguageService,
        params: &lsproto::SemanticTokensRangeParams,
    ) -> LspResult<lsproto::SemanticTokensRangeResponse> {
        Ok(lsproto::OrNull(
            ls.provide_semantic_tokens_range(&params.text_document.uri, params.range.clone()),
        ))
    }

    pub fn handle_initialize_api_session(
        &self,
        params: &lsproto::InitializeAPISessionParams,
        _req: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::CustomInitializeAPISessionResponse> {
        let mut api_sessions = self.api_sessions.lock().unwrap();
        let api_session = crate::mig::m5n::api::session::Session::new(
            self.session.as_ref().unwrap().clone(),
        );
        let pipe_path = match &params.pipe {
            Some(pipe) if !pipe.is_empty() => pipe.clone(),
            _ => self.generate_api_pipe_path(),
        };
        let transport = ipc::new_pipe_transport(&pipe_path)
            .map_err(|e| LspError::new(lsproto::ErrorCode::InternalError, format!("failed to create API transport: {e}")))?;
        let session_id = next_api_session_id();
        let session_id_for_log = session_id.clone();
        std::thread::spawn(move || {
            let rwc = transport.accept();
            let _ = transport.close();
            let Ok(rwc) = rwc else {
                return;
            };
            let rwc = std::sync::Arc::new(tsox_compile::mig::m3l_cm_2::CloseOnceReadWriteCloser::new(
                Box::new(rwc),
            ));
            let protocol = tsox_compile::mig::m3l_cm_3::new_jsonrpc_protocol(std::sync::Arc::clone(&rwc));
            let conn = tsox_compile::mig::m3l_cm_3::new_async_conn_with_protocol(
                rwc,
                protocol,
                tsox_compile::mig::m3l_cm_3::RejectHandler,
            );
            conn.run_detached();
        });
        api_sessions.insert(session_id.clone(), api_session);
        Ok(lsproto::CustomInitializeAPISessionResponse::InitializeAPISessionResult(
            lsproto::InitializeAPISessionResult {
                session_id,
                pipe: pipe_path,
            },
        ))
    }

    pub fn generate_api_pipe_path(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let rnd = {
            use std::hash::{BuildHasher, Hasher};
            std::collections::hash_map::RandomState::new().build_hasher().finish()
        };
        ipc::generate_pipe_path(&format!("tsgo-api-{now:x}-{rnd:x}"))
    }

    pub fn handle_run_gc(&self) -> LspResult<lsproto::RunGCResponse> {
        pprof::run_gc();
        self.logger.info("GC triggered");
        Ok(lsproto::RunGCResponse::Null)
    }

    pub fn handle_save_heap_profile(&self, params: &lsproto::ProfileParams) -> LspResult<lsproto::ProfileResult> {
        let file_path = pprof::save_heap_profile(&params.dir)
            .map_err(|e| LspError::new(lsproto::ErrorCode::InternalError, e))?;
        self.logger.info(&format!("Heap profile saved to: {file_path}"));
        Ok(lsproto::ProfileResult { file: file_path })
    }

    pub fn handle_save_alloc_profile(&self, params: &lsproto::ProfileParams) -> LspResult<lsproto::ProfileResult> {
        let file_path = pprof::save_alloc_profile(&params.dir)
            .map_err(|e| LspError::new(lsproto::ErrorCode::InternalError, e))?;
        self.logger.info(&format!("Allocation profile saved to: {file_path}"));
        Ok(lsproto::ProfileResult { file: file_path })
    }

    pub fn handle_start_cpu_profile(&self, params: &lsproto::ProfileParams) -> LspResult<lsproto::StartCPUProfileResponse> {
        self.cpu_profiler
            .start_cpu_profile(&params.dir)
            .map_err(|e| LspError::new(lsproto::ErrorCode::InternalError, e))?;
        self.logger
            .info(&format!("CPU profiling started, will save to: {}", params.dir));
        Ok(lsproto::StartCPUProfileResponse::Null)
    }

    pub fn handle_stop_cpu_profile(&self) -> LspResult<lsproto::ProfileResult> {
        let file_path = self
            .cpu_profiler
            .stop_cpu_profile()
            .map_err(|e| LspError::new(lsproto::ErrorCode::InternalError, e))?;
        self.logger.info(&format!("CPU profile saved to: {file_path}"));
        Ok(lsproto::ProfileResult { file: file_path })
    }

    pub fn handle_project_info(
        &self,
        params: &lsproto::ProjectInfoParams,
        _req: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::CustomProjectInfoResponse> {
        let uri = &params.text_document.uri;
        let (default_project, _, _) = self
            .session
            .as_ref()
            .unwrap()
            .get_language_service_and_projects_for_file(uri)
            .map_err(|e| LspError::new(lsproto::ErrorCode::InternalError, e))?;
        let mut config_file_path = String::new();
        if default_project.kind == crate::project::project::Kind::Configured {
            config_file_path = default_project.name().to_string();
        }
        Ok(lsproto::CustomProjectInfoResponse::ProjectInfoResult(lsproto::ProjectInfoResult {
            config_file_path,
        }))
    }

    pub fn handle_set_content_mapper_contributions(
        &self,
        params: &lsproto::SetContentMapperContributionsParams,
        _req: &lsproto::RequestMessage,
    ) -> LspResult<lsproto::CustomSetContentMapperContributionsResponse> {
        let contributions = parse_content_mapper_contributions(&params.contributions)?;
        let documents: Vec<crate::lsp::lsproto::DocumentUri> =
            params.open_documents.iter().map(|d| d.uri.clone()).collect();
        self.session
            .as_ref()
            .unwrap()
            .set_content_mapper_contributions(contributions, documents);
        Ok(lsproto::CustomSetContentMapperContributionsResponse::Null)
    }

    pub fn content_mapper_spawner(&self) -> Option<Arc<dyn cm2::Spawner>> {
        self.spawn
            .as_ref()
            .map(|spawn| Arc::new(ServerSpawner(spawn.clone())) as Arc<dyn cm2::Spawner>)
    }

    pub fn content_mapper_logger(&self) -> cm2::Logger {
        let logger = self.logger.clone();
        cm2::Logger::from_fn(Arc::new(move |message: &str| {
            if logger.is_tracing() {
                logger.info(message);
            }
        }))
    }

    pub fn telemetry_enabled(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.telemetry_enabled.load(Ordering::SeqCst)
    }

    fn raw_client_capability_flag(&self, pointer: &str) -> bool {
        self.initialize_params
            .as_ref()
            .and_then(|p| p.capabilities.as_ref())
            .and_then(|c| serde_json::to_value(c).ok())
            .and_then(|v| v.pointer(pointer).and_then(Value::as_bool))
            .unwrap_or(false)
    }

    pub fn client_supports_will_rename_files(&self) -> bool {
        self.raw_client_capability_flag("/workspace/fileOperations/willRename")
    }

    pub fn client_supports_document_changes(&self) -> bool {
        self.raw_client_capability_flag("/workspace/workspaceEdit/documentChanges")
    }
}

pub fn workspace_symbols_scope_is_current_project(session: &crate::project::session::Session) -> bool {
    let _ = session.config();
    false
}

pub fn provide_workspace_symbols_response(
    session: &crate::project::session::Session,
    programs: &[Arc<compiler::Program>],
    query: &str,
) -> LspResult<lsproto::WorkspaceSymbolResponse> {
    let services = session.get_language_services_for_documents_loading_project_tree(&[]);
    let Some(service) = services.first() else {
        return Ok(lsproto::OrNullArray(None));
    };
    let symbols = service.provide_workspace_symbols(programs, query);
    if symbols.is_empty() {
        return Ok(lsproto::OrNullArray(None));
    }
    Ok(lsproto::OrNullArray(Some(
        symbols
            .iter()
            .filter_map(|s| serde_json::to_value(s).ok())
            .collect(),
    )))
}

pub fn next_api_session_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SESSION_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("api-session-{}", SESSION_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1)
}

struct SpawnedProcessConn {
    stdout: Mutex<Box<dyn std::io::Read + Send + Sync>>,
    stdin: Mutex<Box<dyn std::io::Write + Send + Sync>>,
    close: Mutex<Option<Box<dyn FnOnce() + Send + Sync>>>,
}

impl std::io::Read for SpawnedProcessConn {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stdout.lock().unwrap().read(buf)
    }
}

impl std::io::Write for SpawnedProcessConn {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stdin.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.stdin.lock().unwrap().flush()
    }
}

impl cm2::ReadWriteCloser for SpawnedProcessConn {
    fn close(&mut self) -> std::io::Result<()> {
        if let Some(close) = self.close.lock().unwrap().take() {
            close();
        }
        Ok(())
    }
}

struct ServerSpawner(
    Arc<dyn Fn(&[String], &str) -> Result<crate::mig::m5n::SpawnedProcess, String> + Send + Sync>,
);

impl cm2::Spawner for ServerSpawner {
    fn spawn(
        &self,
        command: &[String],
        dir: &str,
        _stderr: &mut dyn std::io::Write,
    ) -> std::io::Result<Box<dyn cm2::ReadWriteCloser>> {
        let process = (self.0)(command, dir).map_err(std::io::Error::other)?;
        Ok(Box::new(SpawnedProcessConn {
            stdout: Mutex::new(process.stdout),
            stdin: Mutex::new(process.stdin),
            close: Mutex::new(Some(process.close)),
        }))
    }
}

pub fn generate_diagnostic_diff_string(
    missing_from_pre: &[crate::ls::types::Diagnostic],
    missing_from_post: &[crate::ls::types::Diagnostic],
    stringifier: &dyn Fn(&crate::ls::types::Diagnostic) -> String,
) -> String {
    let mut b = String::new();
    for elem in missing_from_pre {
        b.push_str(&format!(
            "Diagnostic {} was present after emit but not before emit\n",
            stringifier(elem)
        ));
    }
    for elem in missing_from_post {
        b.push_str(&format!(
            "Diagnostic {} was present before emit but not after emit\n",
            stringifier(elem)
        ));
    }
    b
}

pub fn parse_content_mapper_contributions(
    values: &[lsproto::ContentMapperContribution],
) -> LspResult<crate::project::mig::m5e::ContentMapperContributions> {
    let mut result = crate::project::mig::m5e::ContentMapperContributions::default();
    let mut claimed_extensions: tsox_core::collections::set::Set<String> = Default::default();
    for (index, value) in values.iter().enumerate() {
        if value.contributor_id.is_empty() {
            return Err(LspError::new(
                lsproto::ErrorCode::InvalidParams,
                "content mapper contribution requires a contributorId",
            ));
        }
        let identity = format!("{}[{}]", value.contributor_id, index);
        let mut valid_extensions: Vec<String> = Vec::with_capacity(value.extensions.len());
        for extension in &value.extensions {
            if !is_valid_contributed_content_mapper_extension(extension) {
                return Err(LspError::new(
                    lsproto::ErrorCode::InvalidParams,
                    format!("content mapper contribution \"{identity}\" has invalid extension \"{extension}\""),
                ));
            }
            valid_extensions.push(extension.clone());
        }
        let Some(inferred_project) = &value.inferred_project_contribution else {
            continue;
        };
        let manifest = &inferred_project.manifest;
        if manifest.name.is_empty() || manifest.exec.is_empty() {
            return Err(LspError::new(
                lsproto::ErrorCode::InvalidParams,
                format!("content mapper contribution \"{identity}\" requires a manifest name and exec"),
            ));
        }
        if let Some(compiler_options) = &manifest.compiler_options {
            for option in compiler_options {
                if !tsoptions::command_line_compiler_options_map()
                    .keys()
                    .any(|name| *name == option.as_str())
                {
                    return Err(LspError::new(
                        lsproto::ErrorCode::InvalidParams,
                        format!("content mapper contribution \"{identity}\" requests unknown compiler option \"{option}\""),
                    ));
                }
            }
        }
        for extension in &valid_extensions {
            let lowered = extension.to_lowercase();
            if !claimed_extensions.add_if_absent(lowered) {
                return Err(LspError::new(
                    lsproto::ErrorCode::InvalidParams,
                    format!("content mapper contributions both claim extension \"{extension}\""),
                ));
            }
            result.extensions.push(extension.clone());
        }
        let options = inferred_project.options.clone().unwrap_or_else(|| json!({}));
        let mut mapper = contentmapper::Mapper {
            definition: contentmapper::Definition {
                package: identity.clone(),
                extensions: valid_extensions.clone(),
                options,
            },
            manifest: contentmapper::Manifest {
                name: manifest.name.clone(),
                version: manifest.version.clone().unwrap_or_default(),
                exec: manifest.exec.clone(),
                compiler_options: manifest.compiler_options.clone().unwrap_or_default(),
                dynamic_config: manifest.dynamic_config.unwrap_or_default(),
            },
            contribution_id: identity.clone(),
            package_directory: String::new(),
        };
        if let Some(cwd) = &manifest.cwd {
            if !tspath::path_is_absolute(cwd) {
                return Err(LspError::new(
                    lsproto::ErrorCode::InvalidParams,
                    format!("content mapper contribution \"{identity}\" has non-absolute cwd"),
                ));
            }
            mapper.package_directory = cwd.clone();
        }
        result.mappers.push(Arc::new(mapper));
    }
    result.extensions.sort();
    Ok(result)
}

pub fn all_supported_extensions_with_json() -> Vec<String> {
    use tsox_core::tspath::{
        EXTENSION_CJS, EXTENSION_CTS, EXTENSION_DCTS, EXTENSION_DMTS, EXTENSION_DTS, EXTENSION_JSON,
        EXTENSION_JS, EXTENSION_JSX, EXTENSION_MJS, EXTENSION_MTS, EXTENSION_TS, EXTENSION_TSX,
    };
    vec![
        EXTENSION_TS, EXTENSION_TSX, EXTENSION_DTS, EXTENSION_JS, EXTENSION_JSX, EXTENSION_CTS,
        EXTENSION_DCTS, EXTENSION_CJS, EXTENSION_MTS, EXTENSION_DMTS, EXTENSION_MJS,
        EXTENSION_JSON,
    ]
    .into_iter()
    .map(|e| e.to_string())
    .collect()
}

pub fn is_valid_contributed_content_mapper_extension(extension: &str) -> bool {
    if extension.len() <= 1 || !extension.starts_with('.') {
        return false;
    }
    if tspath::get_any_extension_from_path(&format!("file{extension}"), &[], false) != extension {
        return false;
    }
    !all_supported_extensions_with_json()
        .iter()
        .any(|native_extension| native_extension.eq_ignore_ascii_case(extension))
}
