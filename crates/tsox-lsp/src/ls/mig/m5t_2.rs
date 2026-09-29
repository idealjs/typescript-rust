#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::find_all_references::{Definition, DefinitionKind, EntryKind, ReferenceEntry, RefOptions, ReferenceUse, SymbolAndEntries};
use crate::ls::language_service::LanguageService;
use crate::lsp::lsproto_lsp::{DocumentUri, Location, Range};
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};

use super::m5s_3::{ImpExpKind, RefSearch, RefState};
use super::m5t::{get_range_of_node, is_declaration_of_symbol, new_node_entry};

/// r64k04:references 相关缺失类型已归位 crate::lsp::lsproto_lsp_references,
/// 经门面 glob 提供;ReferencesResponse/Implementation 系仍为本片私有形状。
mod lsproto_lsp {
    pub use crate::lsp::lsproto_lsp::*;

    use crate::lsp::lsproto_lsp::Location;

    pub enum ReferencesResponse {
        Locations(Vec<Location>),
    }

    pub struct ImplementationParams {
        pub text_document: crate::lsp::lsproto_lsp::TextDocumentIdentifier,
        pub text_document_position: crate::lsp::lsproto_lsp::TextDocumentPositionParams,
    }

    pub enum ImplementationResponse {
        Locations(Vec<Location>),
        DefinitionLinks(Vec<crate::ls::types_highlight::LocationLink>),
    }
}

impl LanguageService {
    pub fn resolve_entry_source(&self, entry: &mut ReferenceEntry) {
        let Some(node) = entry.node.clone() else {
            return;
        };
        let Some(source_file) = super::m5t_3::source_file_of_node(&self.get_program(), &node) else {
            return;
        };
        if entry.file_name.is_empty() {
            entry.file_name = source_file.file_name.clone();
        }
        if entry.text_range.is_none() {
            entry.text_range = Some(get_range_of_node(&node, Some(&source_file), None));
        }
    }

    pub fn get_range_of_entry_for_feature(&self, entry: &mut ReferenceEntry, feature: crate::ls::mig::m5s::SpanFeature) -> Option<Range> {
        self.resolve_entry_source(entry);
        let source_file = super::m5t_3::source_file_of_node(&self.get_program(), entry.node.as_ref()?)?;
        let text_range = entry.text_range?;
        let (location, fidelity) = self.source_file_range_to_lsp_location_for_feature(&source_file, text_range, feature);
        if matches!(fidelity, crate::ls::mig::m5s::SpanFidelity::SingleSegment) {
            Some(location.range)
        } else {
            None
        }
    }

    pub fn get_non_local_definition(&self, entry: &SymbolAndEntries) -> Option<crate::ls::find_all_references::NonLocalDefinition> {
        if !entry.can_use_definition_symbol() {
            return None;
        }
        let program = self.get_program();
        let mut checker = program.get_type_checker();
        let symbol = entry.definition_symbol()?;
        for declaration in symbol.declarations.iter() {
            if !super::m5t::is_definition_visible(&mut *checker, declaration) {
                continue;
            }
            let (file, start_pos) = super::m5w_6::get_file_and_start_pos_from_declaration(&program, declaration)?;
            let file_name = file.file_name.clone();
            let script = crate::ls::mig::m5x::M5xSourceFileScript { file: Arc::clone(&file) };
            let position = self.converters.position_to_line_and_character(&script, start_pos as usize);
            return Some(crate::ls::find_all_references::NonLocalDefinition {
                uri: crate::lsp::lsproto_lsp::DocumentUri(crate::ls::lsconv_converters::file_name_to_document_uri(&file_name)),
                position,
            });
        }
        None
    }

    pub fn provide_symbols_and_entries_at_position(
        &self,
        program: &Arc<tsox_compile::compiler::Program>,
        source_file: &Arc<SourceFile>,
        position: u32,
        is_rename: bool,
        implementations: bool,
    ) -> (crate::ls::find_all_references::SymbolAndEntriesData, bool) {
        let mut node = match tsox_frontend::astnav::get_touching_property_name(&source_file.node, position as usize) {
            Some(node) => node,
            None => {
                return (
                    crate::ls::find_all_references::SymbolAndEntriesData {
                        original_node: source_file.node.clone(),
                        symbols_and_entries: Vec::new(),
                    },
                    false,
                );
            }
        };
        if is_rename {
            node = super::m5x_6::get_adjusted_location(&node, true, Some(source_file.clone()));
        }
        if is_rename && !crate::ls::rename::node_is_eligible_for_rename(&node)
            || implementations && ast::is_source_file(&node)
        {
            return (
                crate::ls::find_all_references::SymbolAndEntriesData {
                    original_node: node,
                    symbols_and_entries: Vec::new(),
                },
                false,
            );
        }

        let entries = self.get_symbol_and_entries(position, &node, program, is_rename, implementations);
        if !implementations {
            return (
                crate::ls::find_all_references::SymbolAndEntriesData {
                    original_node: node,
                    symbols_and_entries: entries,
                },
                true,
            );
        }

        let mut implementation_entries: Vec<SymbolAndEntries> = Vec::new();
        let mut queue: std::collections::VecDeque<Arc<Node>> = std::collections::VecDeque::new();
        let mut seen_nodes: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut seen_definitions: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut add_to_queue = |symbol_and_entries: &Vec<SymbolAndEntries>,
                                implementation_entries: &mut Vec<SymbolAndEntries>,
                                queue: &mut std::collections::VecDeque<Arc<Node>>,
                                seen_nodes: &mut std::collections::HashSet<usize>,
                                seen_definitions: &mut std::collections::HashSet<usize>| {
            for s in symbol_and_entries {
                let mut new_references: Vec<ReferenceEntry> = Vec::new();
                for r in &s.references {
                    if let Some(node) = &r.node {
                        if seen_nodes.insert(Arc::as_ptr(node) as usize) {
                            queue.push_back(node.clone());
                            new_references.push(clone_reference_entry(r));
                        }
                    }
                }
                let should_add = !new_references.is_empty()
                    || s.definition.symbol.as_ref().map(|sym| seen_definitions.insert(Arc::as_ptr(sym) as usize)).unwrap_or(true);
                if should_add {
                    implementation_entries.push(SymbolAndEntries {
                        definition: crate::ls::find_all_references::Definition {
                            kind: s.definition.kind,
                            symbol: s.definition.symbol.clone(),
                            node: s.definition.node.clone(),
                        },
                        references: new_references,
                    });
                }
            }
        };
        add_to_queue(&entries, &mut implementation_entries, &mut queue, &mut seen_nodes, &mut seen_definitions);
        while let Some(entry_node) = queue.pop_front() {
            let more = self.get_symbol_and_entries(entry_node.pos() as u32, &entry_node, program, is_rename, implementations);
            add_to_queue(&more, &mut implementation_entries, &mut queue, &mut seen_nodes, &mut seen_definitions);
        }
        (
            crate::ls::find_all_references::SymbolAndEntriesData {
                original_node: node,
                symbols_and_entries: implementation_entries,
            },
            true,
        )
    }

    pub fn get_symbol_and_entries(
        &self,
        position: u32,
        node: &Arc<Node>,
        program: &Arc<tsox_compile::compiler::Program>,
        is_rename: bool,
        implementations: bool,
    ) -> Vec<SymbolAndEntries> {
        let mut options = RefOptions::default();
        if !is_rename {
            options.use_ = ReferenceUse::References;
            if implementations {
                options.implementations = true;
            }
        } else {
            options.use_ = ReferenceUse::Rename;
            options.use_aliases_for_rename = self.user_preferences().use_aliases_for_rename.is_true_or_unknown();
        }
        crate::ls::find_all_references::get_referenced_symbols_for_node(self, position as usize, node, program, &program.get_source_files(), options.clone())
    }

    pub fn provide_references_from_data(
        &self,
        params: &lsproto_lsp::ReferenceParams,
        data: &crate::ls::find_all_references::SymbolAndEntriesData,
    ) -> lsproto_lsp::ReferencesResponse {
        self.symbol_and_entries_to_references(params, data)
    }

    pub fn symbol_and_entries_to_references(
        &self,
        params: &lsproto_lsp::ReferenceParams,
        data: &crate::ls::find_all_references::SymbolAndEntriesData,
    ) -> lsproto_lsp::ReferencesResponse {
        let mut locations: Vec<Location> = Vec::new();
        let mut seen_locations: std::collections::HashSet<(String, u32, u32, u32, u32)> = std::collections::HashSet::new();
        for symbol in &data.symbols_and_entries {
            let symbol_locations = self.convert_symbol_and_entries_to_locations(symbol, params.context.include_declaration, crate::ls::mig::m5s::SpanFeature::References);
            for location in symbol_locations {
                let key = (
                    location.uri.to_string(),
                    location.range.start.line,
                    location.range.start.character,
                    location.range.end.line,
                    location.range.end.character,
                );
                if seen_locations.insert(key) {
                    locations.push(location);
                }
            }
        }
        lsproto_lsp::ReferencesResponse::Locations(locations)
    }

    pub fn symbol_and_entries_to_vs_references(
        &self,
        params: &lsproto_lsp::ReferenceParams,
        data: &crate::ls::find_all_references::SymbolAndEntriesData,
    ) -> lsproto_lsp::VSReferencesResponse {
        let mut items: Vec<lsproto_lsp::VSReferenceItem> = Vec::new();
        let mut id: i32 = 0;
        let project_name = self.project_path.to_string();
        let program = self.get_program();
        let symbols = program.symbol_map();

        for s in &data.symbols_and_entries {
            if s.definition.symbol.is_none() && s.definition.node.is_none() {
                continue;
            }
            let def_info = super::m5t_4::definition_to_referenced_symbol_definition_info(self, &s.definition, &data.original_node, crate::ls::mig::m5s::SpanFeature::References);
            let Some(def_info) = def_info else {
                continue;
            };
            let definition_id = id;
            items.push(lsproto_lsp::VSReferenceItem {
                vs_id: definition_id,
                vs_definition_id: None,
                vs_location: def_info.location.clone(),
                vs_definition_text: def_info.display_text,
                vs_kind: lsproto_lsp::VSReferenceKind::Unknown,
                vs_project_name: project_name.clone(),
                vs_containing_type: String::new(),
            });
            id += 1;

            for r in &s.references {
                if let Some(symbol) = &s.definition.symbol {
                    if is_declaration_of_symbol(r.node.as_ref(), symbol, &self.get_program()) {
                        continue;
                    }
                }
                let mut ref_entry = clone_reference_entry(r);
                let Some(ref_location) = self.get_location_of_entry_for_feature(&mut ref_entry, crate::ls::mig::m5s::SpanFeature::References) else {
                    continue;
                };
                let kind = if r.kind != EntryKind::Range
                    && r.node
                        .as_ref()
                        .map(|n| {
                            let decl = tsox_frontend::ast::mig::m3b::get_declaration_from_name(n, symbols);
                            decl.as_deref().is_some_and(tsox_frontend::ast::mig::m3b::declaration_is_write_access)
                                || n.kind == SyntaxKind::DefaultKeyword
                                || tsox_frontend::ast::mig::m3b::is_write_access(n)
                        })
                        .unwrap_or(false)
                {
                    lsproto_lsp::VSReferenceKind::Write
                } else {
                    lsproto_lsp::VSReferenceKind::Read
                };
                items.push(lsproto_lsp::VSReferenceItem {
                    vs_id: id,
                    vs_definition_id: Some(definition_id),
                    vs_location: ref_location,
                    vs_definition_text: String::new(),
                    vs_kind: kind,
                    vs_project_name: project_name.clone(),
                    vs_containing_type: String::new(),
                });
                id += 1;
            }
        }
        lsproto_lsp::VSReferencesResponse { vs_reference_items: items }
    }

    pub fn provide_implementations_ex(
        &self,
        params: &lsproto_lsp::ImplementationParams,
        options: &crate::ls::find_all_references::SymbolEntryTransformOptions,
    ) -> lsproto_lsp::ImplementationResponse {
        let (program, file) = self.get_program_and_file(&params.text_document.uri);
        let position = lsp_position_to_offset(&file, &params.text_document_position.position);
        let (data, _) = self.provide_symbols_and_entries_at_position(&program, &file, position, false, true);
        self.provide_implementations_from_data(params, options, &data)
    }

    pub fn provide_implementations_from_data(
        &self,
        params: &lsproto_lsp::ImplementationParams,
        options: &crate::ls::find_all_references::SymbolEntryTransformOptions,
        data: &crate::ls::find_all_references::SymbolAndEntriesData,
    ) -> lsproto_lsp::ImplementationResponse {
        self.symbol_and_entries_to_implementations(params, data, options)
    }

    pub fn symbol_and_entries_to_implementations(
        &self,
        params: &lsproto_lsp::ImplementationParams,
        data: &crate::ls::find_all_references::SymbolAndEntriesData,
        options: &crate::ls::find_all_references::SymbolEntryTransformOptions,
    ) -> lsproto_lsp::ImplementationResponse {
        let mut seen_nodes: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut entries: Vec<ReferenceEntry> = Vec::new();
        for entry in &data.symbols_and_entries {
            for r in &entry.references {
                let key = r.node.as_ref().map(|n| Arc::as_ptr(n) as usize).unwrap_or(0);
                if seen_nodes.insert(key)
                    && !(false
                        && r.node.as_ref().map(|n| n.loc.contains_inclusive(data.original_node.pos())).unwrap_or(false))
                {
                    entries.push(clone_reference_entry(r));
                }
            }
        }

        if !false {
            let links = self.convert_entries_to_location_links(&entries, crate::ls::mig::m5s::SpanFeature::Implementation);
            return lsproto_lsp::ImplementationResponse::DefinitionLinks(links);
        }
        let locations = self.convert_entries_to_locations(&entries, crate::ls::mig::m5s::SpanFeature::Implementation);
        lsproto_lsp::ImplementationResponse::Locations(locations)
    }

    pub fn convert_symbol_and_entries_to_locations(
        &self,
        s: &SymbolAndEntries,
        include_declarations: bool,
        feature: crate::ls::mig::m5s::SpanFeature,
    ) -> Vec<Location> {
        let references: Vec<&ReferenceEntry> = if include_declarations {
            s.references.iter().collect()
        } else {
            s.references
                .iter()
                .filter(|entry| {
                    s.definition
                        .symbol
                        .as_ref()
                        .map(|sym| !is_declaration_of_symbol(entry.node.as_ref(), sym, &self.get_program()))
                        .unwrap_or(true)
                })
                .collect()
        };
        self.convert_entries_to_locations_owned(&references, feature)
    }

    pub fn convert_entries_to_locations(&self, entries: &[ReferenceEntry], feature: crate::ls::mig::m5s::SpanFeature) -> Vec<Location> {
        let refs: Vec<&ReferenceEntry> = entries.iter().collect();
        self.convert_entries_to_locations_owned(&refs, feature)
    }

    fn convert_entries_to_locations_owned(&self, entries: &[&ReferenceEntry], feature: crate::ls::mig::m5s::SpanFeature) -> Vec<Location> {
        let mut locations = Vec::with_capacity(entries.len());
        for entry in entries {
            let mut owned = clone_reference_entry(entry);
            if let Some(location) = self.get_location_of_entry_for_feature(&mut owned, feature) {
                locations.push(location);
            }
        }
        locations
    }

    pub fn convert_entries_to_location_links(&self, entries: &[ReferenceEntry], feature: crate::ls::mig::m5s::SpanFeature) -> Vec<LocationLink> {
        let mut links = Vec::with_capacity(entries.len());
        for entry in entries {
            let mut owned = clone_reference_entry(entry);
            let Some(loc) = self.get_location_of_entry_for_feature(&mut owned, feature) else {
                continue;
            };
            let target_selection_range = loc.range.clone();
            let mut target_range = target_selection_range.clone();
            if let Some(node) = &entry.node {
                if let Some(source_file_node) = super::m5t_3::source_file_of_node(&self.get_program(), node) {
                    if let Some(context_text_range) = super::m5x_5::to_context_range(entry.text_range, &source_file_node, entry.context.as_ref()) {
                        let (context_location, fidelity) = self.source_file_range_to_lsp_location_for_feature(&source_file_node, context_text_range, feature);
                        if fidelity != crate::ls::mig::m5s::SpanFidelity::None_ && context_location.uri == loc.uri {
                            target_range = context_location.range;
                        }
                    }
                }
            }
            links.push(LocationLink {
                origin_selection_range: None,
                target_uri: loc.uri,
                target_range,
                target_selection_range,
            });
        }
        links
    }

    pub fn merge_references(&self, program: &Arc<tsox_compile::compiler::Program>, references_to_merge: &[Vec<SymbolAndEntries>]) -> Vec<SymbolAndEntries> {
        let mut result: Vec<SymbolAndEntries> = Vec::new();
        let get_source_file_index_of_entry = |entry: &mut ReferenceEntry| -> usize {
            self.resolve_entry_source(entry);
            let file_node = entry.node.as_ref().and_then(|n| ast::get_source_file_of_node(n));
            program
                .source_files()
                .iter()
                .position(|f| file_node.as_ref().map(|sf| Arc::ptr_eq(&f.node, sf)).unwrap_or(false))
                .unwrap_or(usize::MAX)
        };

        for references in references_to_merge {
            if references.is_empty() {
                continue;
            }
            if result.is_empty() {
                result = references.iter().map(clone_symbol_and_entries).collect();
                continue;
            }
            for entry in references {
                let is_symbol_definition = matches!(entry.definition.kind, DefinitionKind::Symbol);
                if !is_symbol_definition {
                    result.push(clone_symbol_and_entries(entry));
                    continue;
                }
                let Some(symbol) = &entry.definition.symbol else {
                    result.push(clone_symbol_and_entries(entry));
                    continue;
                };
                let ref_index = result.iter().position(|ref_entry| {
                    ref_entry.definition.kind == DefinitionKind::Symbol
                        && ref_entry
                            .definition
                            .symbol
                            .as_ref()
                            .map(|s| Arc::ptr_eq(s, symbol))
                            .unwrap_or(false)
                });
                let Some(ref_index) = ref_index else {
                    result.push(clone_symbol_and_entries(entry));
                    continue;
                };
                let mut sorted_refs = result[ref_index].references.iter().map(clone_reference_entry).collect::<Vec<_>>();
                sorted_refs.extend(entry.references.iter().map(|r| clone_reference_entry(r)));
                let mut sorted = sorted_refs;
                sorted.sort_by(|entry1, entry2| {
                    let mut e1 = clone_reference_entry(entry1);
                    let mut e2 = clone_reference_entry(entry2);
                    let f1 = get_source_file_index_of_entry(&mut e1);
                    let f2 = get_source_file_index_of_entry(&mut e2);
                    if f1 != f2 {
                        return f1.cmp(&f2);
                    }
                    match (&e1.lsp_range, &e2.lsp_range) {
                        (Some(r1), Some(r2)) => crate::lsp::lsproto_util::compare_ranges(&r1.range, &r2.range),
                        _ => std::cmp::Ordering::Equal,
                    }
                });
                let definition = clone_definition(&result[ref_index].definition);
                result[ref_index] = SymbolAndEntries {
                    definition,
                    references: sorted,
                };
            }
        }
        result
    }
}

pub type LocationLink = crate::ls::types_highlight::LocationLink;

fn lsp_position_to_offset(source_file: &Arc<SourceFile>, position: &lsproto_lsp::Position) -> u32 {
    let line_map = &source_file.line_map;
    let line_start = line_map
        .line_starts
        .get(position.line as usize)
        .copied()
        .unwrap_or(0) as usize;
    (line_start + position.character as usize) as u32
}

fn clone_reference_entry(entry: &ReferenceEntry) -> ReferenceEntry {
    ReferenceEntry {
        kind: entry.kind,
        node: entry.node.clone(),
        context: entry.context.clone(),
        file_name: entry.file_name.clone(),
        text_range: entry.text_range.clone(),
        lsp_range: entry.lsp_range.clone(),
    }
}

fn clone_definition(def: &Definition) -> Definition {
    Definition {
        kind: def.kind,
        symbol: def.symbol.clone(),
        node: def.node.clone(),
    }
}

fn clone_symbol_and_entries(s: &SymbolAndEntries) -> SymbolAndEntries {
    SymbolAndEntries {
        definition: clone_definition(&s.definition),
        references: s.references.iter().map(clone_reference_entry).collect(),
    }
}
