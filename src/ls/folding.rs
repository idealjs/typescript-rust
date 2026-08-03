//! Folding ranges provider (1:1 port of Go's `internal/ls/folding.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::lsp::lsproto::lsp::DocumentUri;

use super::language_service::LanguageService;
use super::types::FoldingRange;

/// Result of parsing a `//#region` / `//#endregion` delimiter.
pub struct RegionDelimiterResult {
    pub is_start: bool,
    pub name: String,
}

impl LanguageService {
    /// Provide folding ranges for a document.
    ///
    /// Mirrors `ProvideFoldingRange`.
    pub fn provide_folding_range(&self, document_uri: &DocumentUri) -> Vec<FoldingRange> {
        let (_program, source_file) = self.get_program_and_file(document_uri);
        let mut res = self.add_node_outlining_spans(&source_file);
        res.extend(self.add_region_outlining_spans(&source_file));
        res
    }

    /// Adjust folding end lines for `lineFoldingOnly` clients.
    pub fn adjust_folding_end(
        &self,
        ranges: Vec<FoldingRange>,
        _source_file: &Arc<SourceFile>,
    ) -> Vec<FoldingRange> {
        // TODO: requires converters and source-file text access
        ranges
    }

    /// Collect outlining spans from AST nodes.
    pub fn add_node_outlining_spans(&self, _source_file: &Arc<SourceFile>) -> Vec<FoldingRange> {
        // TODO: requires full AST traversal
        Vec::new()
    }

    /// Collect outlining spans from `//#region` comments.
    pub fn add_region_outlining_spans(&self, _source_file: &Arc<SourceFile>) -> Vec<FoldingRange> {
        // TODO: requires scanner line-start iteration
        Vec::new()
    }
}

/// Parse a `//#region` / `//#endregion` delimiter from a line of text.
///
/// Mirrors `parseRegionDelimiter`.
pub fn parse_region_delimiter(line_text: &str) -> Option<RegionDelimiterResult> {
    let line_text = line_text.trim_start();
    let line_text = line_text.strip_prefix("//")?;
    let line_text = line_text.trim();
    let line_text = line_text.strip_suffix('\r').unwrap_or(line_text);
    let line_text = line_text.strip_prefix('#')?;
    let is_start = if let Some(rest) = line_text.strip_prefix("end") {
        let _ = rest;
        false
    } else {
        true
    };
    let rest = if line_text.starts_with("end") {
        &line_text[3..]
    } else {
        line_text
    };
    let rest = rest.strip_prefix("region")?;
    Some(RegionDelimiterResult {
        is_start,
        name: rest.trim().to_string(),
    })
}
