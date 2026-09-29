#![allow(unused_imports, dead_code)]

//! m5k2b:session.go 收尾批次,handleGetCompletionsAtPosition 与响应结构移植(归属待接线)。

use super::m5l::{
    client_error, core_context, DocumentIdentifier, Program, ProjectId,
    Session, SnapshotData, SnapshotId, SymbolResponse,
};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct GetCompletionsAtPositionParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
    pub position: u32,
    pub trigger_character: Option<String>,
    pub include_symbol: bool,
}

#[derive(Default, serde::Serialize)]
pub struct CompletionEntryLabelDetailsResponse {
    pub detail: Option<String>,
    pub description: Option<String>,
}

#[derive(Default, serde::Serialize)]
pub struct CompletionEntryResponse {
    pub name: String,
    pub kind: u32,
    pub sort_text: Option<String>,
    pub insert_text: Option<String>,
    pub filter_text: Option<String>,
    pub detail: Option<String>,
    pub label_details: Option<CompletionEntryLabelDetailsResponse>,
    pub symbol: Option<SymbolResponse>,
}

#[derive(Default, serde::Serialize)]
pub struct CompletionInfoResponse {
    pub is_incomplete: bool,
    pub entries: Vec<CompletionEntryResponse>,
}

impl Session {
    pub fn handle_get_completions_at_position(
        &self,
        params: &GetCompletionsAtPositionParams,
    ) -> Result<Option<CompletionInfoResponse>, String> {
        let mut ctx = core_context();
        if params.include_symbol {
            ctx = tsox_core::core::mig::m3j_3::with_checker_lifetime(
                &ctx,
                tsox_core::core::mig::m3j_3::CheckerLifetime::Api,
            );
        }
        let sd = self.get_snapshot_data(params.snapshot.clone())?;
        let program = sd.get_program(&params.project)?;
        let Some(source_file) = program.get_source_file(&params.file.to_file_name()) else {
            return Ok(None);
        };
        let lang_svc = self.setup_language_service(&sd, &program, &params.project, "")?;
        let internal_pos = tsox_frontend::ast::mig::m3b_2::get_position_map(&source_file)
            .utf16_to_utf8(params.position as usize);
        let result = lang_svc.get_completions_at_position(
            &mut ctx,
            &source_file,
            internal_pos,
            params.trigger_character.as_deref(),
            params.include_symbol,
        )?;
        let Some(result) = result else {
            return Ok(None);
        };
        let mut entries = Vec::with_capacity(result.items.len());
        for item in &result.items {
            let mut entry = CompletionEntryResponse {
                name: item.label.clone(),
                sort_text: item.sort_text.clone(),
                insert_text: item.insert_text.clone(),
                filter_text: item.filter_text.clone(),
                detail: item.detail.clone(),
                ..Default::default()
            };
            if let Some(kind) = item.kind {
                entry.kind = kind;
            }
            if let Some(label_details) = &item.label_details {
                entry.label_details = Some(CompletionEntryLabelDetailsResponse {
                    detail: label_details.detail.clone(),
                    description: label_details.description.clone(),
                });
            }
            if let Some(symbol) = &item.symbol {
                entry.symbol = sd.new_symbol_response(symbol, &params.project);
            }
            entries.push(entry);
        }
        Ok(Some(CompletionInfoResponse {
            is_incomplete: result.is_incomplete,
            entries,
        }))
    }
}
