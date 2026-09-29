#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::find_all_references::{DefinitionKind, EntryKind, ReferenceEntry, RefOptions, ReferenceUse, SymbolAndEntries};
use crate::ls::language_service::LanguageService;
use crate::ls::types::{DocumentHighlight, DocumentHighlightKind, MultiDocumentHighlight};
use crate::lsp::lsproto_lsp::{DocumentUri, Location, Position, Range};
use tsox_frontend::ast::mig::m3e_4::SemanticMeaning;
use tsox_frontend::ast::mig::m3h::climb_past_property_access;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;

fn m5s2_m5u_converters() -> crate::mig::m5u_conv::M5uConverters {
    crate::mig::m5u_conv::new_converters(
        crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    )
}

/// r64k04:本地缺失类型已归位 crate::lsp::lsproto_lsp_references,经门面 glob 提供。
mod lsproto_lsp {
    pub use crate::lsp::lsproto_lsp::*;
}

pub fn combine_definition_responses(
    results: Vec<super::m5s::DefinitionResponse>,
    links: bool,
) -> super::m5s::DefinitionResponse {
    let mut locations: Vec<Location> = Vec::new();
    let mut definition_links: Vec<crate::ls::types::LocationLink> = Vec::new();
    let mut seen: std::collections::HashSet<(String, u32, u32, u32, u32)> = std::collections::HashSet::new();
    for result in &results {
        if let Some(links_of_result) = &result.definition_links {
            for link in links_of_result {
                let location = Location {
                    uri: link.target_uri.clone(),
                    range: link.target_selection_range.clone(),
                };
                let key = (
                    location.uri.0.clone(),
                    location.range.start.line,
                    location.range.start.character,
                    location.range.end.line,
                    location.range.end.character,
                );
                if seen.insert(key) {
                    definition_links.push(link.clone());
                    locations.push(location);
                }
            }
        }
        if let Some(location) = &result.location {
            let key = (
                location.uri.0.clone(),
                location.range.start.line,
                location.range.start.character,
                location.range.end.line,
                location.range.end.character,
            );
            if seen.insert(key) {
                definition_links.push(crate::ls::types::LocationLink {
                    target_uri: location.uri.clone(),
                    target_range: location.range.clone(),
                    target_selection_range: location.range.clone(),
                    origin_selection_range: None,
                });
                locations.push(location.clone());
            }
        }
        if let Some(result_locations) = &result.locations {
            for location in result_locations {
                let key = (
                    location.uri.0.clone(),
                    location.range.start.line,
                    location.range.start.character,
                    location.range.end.line,
                    location.range.end.character,
                );
                if seen.insert(key) {
                    definition_links.push(crate::ls::types::LocationLink {
                        target_uri: location.uri.clone(),
                        target_range: location.range.clone(),
                        target_selection_range: location.range.clone(),
                        origin_selection_range: None,
                    });
                    locations.push(location.clone());
                }
            }
        }
    }
    if links {
        super::m5s::DefinitionResponse {
            definition_links: Some(definition_links),
            ..Default::default()
        }
    } else {
        super::m5s::DefinitionResponse {
            locations: Some(locations),
            ..Default::default()
        }
    }
}

impl LanguageService {
    pub fn get_location_of_entry_for_feature(
        &self,
        entry: &mut ReferenceEntry,
        feature: super::m5s::SpanFeature,
    ) -> Option<Location> {
        self.resolve_entry_source(entry);
        let program = self.get_program();
        let source_file = super::m5t_3::source_file_of_node(&program, entry.node.as_ref()?)?;
        let text_range = entry.text_range?;
        let (location, fidelity) = self.source_file_range_to_lsp_location_for_feature(&source_file, text_range, feature);
        if matches!(fidelity, super::m5s::SpanFidelity::Exact | super::m5s::SpanFidelity::SingleSegment) {
            Some(location)
        } else {
            None
        }
    }

    pub fn provide_definition_worker(
        &self,
        document_uri: &DocumentUri,
        position: Position,
    ) -> super::m5s::DefinitionResponse {
        // 客户端能力通道未接线(m5m::ResolvedClientCapabilities 为 raw JSON 空壳),
        // 按 m5t2 先例取 ClientCapabilities 默认孪生,待会话能力回接后恢复真实读取
        let client_supports_link = crate::ls::types::ClientCapabilities::default().text_document.definition.link_support;
        let (program, file) = self.get_program_and_file(document_uri);
        let positions = crate::mig::m5u_conv::from_lsp_position_for_source_file(
            &m5s2_m5u_converters(),
            &file,
            position,
            crate::mig::m6b_2::FEATURE_DEFINITION as u32,
        );
        let mut results = Vec::with_capacity(positions.len());
        for mapped in &positions {
            if mapped.fidelity == crate::mig::m5u_conv::SPANMAP_FIDELITY_SINGLE_SEGMENT {
                results.push(self.provide_definition_at_position(
                    &program,
                    &mapped.file,
                    mapped.position as u32,
                    client_supports_link,
                ));
            }
        }
        combine_definition_responses(results, client_supports_link)
    }

    pub fn provide_document_highlights_worker(
        &self,
        document_uri: &DocumentUri,
        document_position: Position,
        files_to_search: &[DocumentUri],
    ) -> super::m5s::MultiDocumentHighlightsOrNull {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let positions = crate::mig::m5u_conv::from_lsp_position_for_source_file(
            &m5s2_m5u_converters(),
            &source_file,
            document_position,
            crate::mig::m6b_2::FEATURE_DOCUMENT_HIGHLIGHTS as u32,
        );
        let mut results = Vec::with_capacity(positions.len());
        for mapped in &positions {
            if mapped.fidelity == crate::mig::m5u_conv::SPANMAP_FIDELITY_SINGLE_SEGMENT {
                results.push(self.provide_document_highlights_at_position(
                    document_uri,
                    mapped.position,
                    &program,
                    &mapped.file,
                    files_to_search,
                ));
            }
        }
        super::m5s_2::combine_multi_document_highlights(results)
    }

    pub fn provide_document_highlights_at_position(
        &self,
        document_uri: &DocumentUri,
        position: usize,
        program: &Arc<tsox_compile::compiler::Program>,
        source_file: &Arc<SourceFile>,
        files_to_search: &[DocumentUri],
    ) -> super::m5s::MultiDocumentHighlightsOrNull {
        let node = astnav::get_touching_property_name(&source_file.node, position)
            .unwrap_or_else(|| Arc::clone(&source_file.node));

        let is_jsx_tag = node.parent().map_or(false, |parent| {
            parent.kind == SyntaxKind::JsxClosingElement
                || (parent.kind == SyntaxKind::JsxOpeningElement
                    && {
                        let tag = tsox_frontend::ast::mig::m3c::tag_name(&parent);
                        Arc::ptr_eq(tag, &node)
                    })
        });
        if is_jsx_tag {
            let mut opening_element = None;
            let mut closing_element = None;
            if let Some(parent) = node.parent() {
                if let Some(grand) = parent.parent() {
                    if ast::is_jsx_element(&grand) {
                        if let Some((opening, closing)) = super::m5s2_2::jsx_element_opening_and_closing(&grand) {
                            opening_element = Some(opening);
                            closing_element = Some(closing);
                        }
                    }
                }
            }
            let mut highlights: Vec<DocumentHighlight> = Vec::new();
            let kind = DocumentHighlightKind::Read;
            for element in [opening_element, closing_element].into_iter().flatten() {
                let (lsp_range, fidelity) = self.create_lsp_range_from_node_for_feature(
                    &element,
                    source_file,
                    super::m5s::SpanFeature::DocumentHighlights as u32,
                );
                if fidelity != 0 {
                    highlights.push(DocumentHighlight {
                        range: lsp_range,
                        kind: Some(kind.clone()),
                    });
                }
            }
            let multi_highlights = vec![MultiDocumentHighlight {
                uri: document_uri.clone(),
                highlights,
            }];
            return super::m5s::MultiDocumentHighlightsOrNull {
                multi_document_highlights: Some(multi_highlights),
            };
        }

        let mut source_files: Vec<Arc<SourceFile>> = Vec::new();
        let mut seen_files: std::collections::HashSet<String> = std::collections::HashSet::new();
        for uri in files_to_search {
            let file_name = uri.file_name();
            if !seen_files.insert(file_name.clone()) {
                continue;
            }
            if let Some(sf) = program.get_source_file(&file_name) {
                source_files.push(sf);
            }
        }
        if source_files.is_empty() {
            source_files.push(Arc::clone(source_file));
        }

        let mut multi_highlights = self.get_semantic_document_highlights(position, &node, program, &source_files);
        if multi_highlights.is_empty() {
            let syntactic_highlights = self.get_syntactic_document_highlights(&node, source_file);
            if !syntactic_highlights.is_empty() {
                multi_highlights = vec![MultiDocumentHighlight {
                    uri: document_uri.clone(),
                    highlights: syntactic_highlights,
                }];
            }
        }
        super::m5s::MultiDocumentHighlightsOrNull {
            multi_document_highlights: Some(multi_highlights),
        }
    }

    pub fn provide_vs_references(
        &self,
        params: &lsproto_lsp::ReferenceParams,
        data: &crate::ls::find_all_references::SymbolAndEntriesData,
    ) -> lsproto_lsp::VSReferencesResponse {
        self.symbol_and_entries_to_vs_references(params, data)
    }

    pub fn get_signature_usages(&self, signature_decl: &Arc<Node>) -> Vec<SignatureUsage> {
        let Some(name) = signature_decl.name() else {
            return Vec::new();
        };
        if !ast::is_identifier(&name) {
            return Vec::new();
        }

        let program = self.get_program();
        let source_files = program.get_source_files();
        let options = RefOptions::default();
        let entries = crate::ls::find_all_references::get_referenced_symbols_for_node(
            self,
            name.pos(),
            name,
            &program,
            &source_files,
            options,
        );

        let mut decl_names: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for entry in &entries {
            if let Some(symbol) = entry.definition_symbol() {
                for decl in &symbol.declarations {
                    if let Some(n) = decl.name() {
                        decl_names.insert(Arc::as_ptr(n) as usize);
                    }
                }
            }
        }

        let mut result: Vec<SignatureUsage> = Vec::new();
        for entry in &entries {
            for r in entry.references() {
                if !r.is_node_entry() {
                    continue;
                }
                let Some(node) = r.node.as_ref() else {
                    continue;
                };
                if decl_names.contains(&(Arc::as_ptr(node) as usize)) {
                    continue;
                }

                let called = climb_past_property_access(node);
                let mut call_expr = None;
                if let Some(parent) = called.parent() {
                    if ast::is_call_expression(&parent)
                        && parent
                            .expression()
                            .map(|e| Arc::ptr_eq(e, &called))
                            .unwrap_or(false)
                    {
                        call_expr = Some(parent);
                    }
                }

                result.push(SignatureUsage {
                    name: node.clone(),
                    call: call_expr,
                });
            }
        }
        result
    }

    pub fn get_definition_kind_and_display_parts(
        &self,
        symbol: &Arc<Symbol>,
        original_node: &Arc<Node>,
        vs_capability: bool,
    ) -> crate::ls::mig::m5t2_3::VSClassifiedTextElement {
        let program = self.get_program();
        let mut checker = program.build_checker();

        let meaning = super::m5x_5::get_intersecting_meaning_from_declarations(
            Some(original_node),
            symbol,
            SemanticMeaning::ALL,
        );

        let info = super::m5t2_2::get_quick_info_and_declaration_at_location(
            &mut checker,
            Some(symbol),
            original_node,
            None,
            vs_capability,
            meaning.0,
        );

        if vs_capability {
            return crate::ls::mig::m5t2_3::VSClassifiedTextElement {
                runs: info.display_parts.get_runs().to_vec(),
            };
        }
        let text = info.display_parts.to_string();
        crate::ls::mig::m5t2_3::VSClassifiedTextElement {
            runs: vec![crate::ls::mig::m5t2_3::VSClassifiedTextRun {
                text,
                classification_type_name: crate::ls::mig::m5t2_3::CLASSIFICATION_TYPE_NAME_TEXT.to_string(),
            }],
        }
    }

    pub fn for_each_original_definition_location(
        &self,
        entry: &SymbolAndEntries,
        cb: &mut impl FnMut(DocumentUri, Position),
    ) {
        if !entry.can_use_definition_symbol() {
            return;
        }
        let program = self.get_program();
        let Some(symbol) = entry.definition_symbol() else {
            return;
        };
        for d in &symbol.declarations {
            let Some((file, start_pos)) = super::m5w_6::get_file_and_start_pos_from_declaration(&program, d) else {
                continue;
            };
            let file_name = file.file_name.clone();
            if tsox_core::tspath::is_declaration_file_name(&file_name) {
                let Some(mapped) = self.try_get_source_position(&file_name, start_pos as tsox_core::core::text::TextPos) else {
                    continue;
                };
                let Some(script) = self.get_script(&mapped.file_name) else {
                    continue;
                };
                // 普通脚本无 span map,Go converters.ToLSPPosition 对其恒返 FidelityExact
                let lsp_position = self.converters.position_to_line_and_character(&script, mapped.pos as usize);
                cb(
                    DocumentUri(crate::ls::lsconv_converters::file_name_to_document_uri(&mapped.file_name)),
                    lsp_position,
                );
            } else if program.is_source_from_project_reference(self.to_path(&file_name).as_str()) {
                // ToLSPPosition 孪生:经 virtual_position_to_original 走 SourceFile 的 span map
                let view = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(&file) };
                let (mapped_script, mapped_pos, fidelity) =
                    crate::mig::m5u_conv::virtual_position_to_original(&view, start_pos as usize, None);
                let script_view = crate::mig::m5u_conv::PlainScriptView {
                    file_name: crate::mig::m5u_conv::M5uScript::file_name(&mapped_script).to_string(),
                    text: crate::mig::m5u_conv::M5uScript::text(&mapped_script).to_string(),
                };
                let lsp_position = self.converters.position_to_line_and_character(&script_view, mapped_pos);
                if fidelity != crate::mig::m5u_conv::SPANMAP_FIDELITY_NONE {
                    cb(
                        DocumentUri(crate::ls::lsconv_converters::file_name_to_document_uri(&file_name)),
                        lsp_position,
                    );
                }
            }
        }
    }
}

impl SymbolAndEntries {
    pub fn references(&self) -> &[ReferenceEntry] {
        &self.references
    }
}

pub struct SignatureUsage {
    pub name: Arc<Node>,
    pub call: Option<Arc<Node>>,
}
