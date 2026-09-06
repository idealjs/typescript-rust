#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, SourceFile, Symbol};
use crate::compiler::Program;
use crate::core::text::TextRange;
use crate::lsp::lsproto::lsp::{DocumentUri, Location, Position, Range};

use super::cross_project::CrossProjectOrchestrator;
use super::language_service::LanguageService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceUse {
    None,
    Other,
    References,
    Rename,
}

#[derive(Debug, Clone)]
pub struct RefOptions {
    pub find_in_strings: bool,
    pub find_in_comments: bool,
    pub use_: ReferenceUse,
    pub implementations: bool,
    pub use_aliases_for_rename: bool,
}

impl Default for RefOptions {
    fn default() -> Self {
        RefOptions {
            find_in_strings: false,
            find_in_comments: false,
            use_: ReferenceUse::None,
            implementations: false,
            use_aliases_for_rename: true,
        }
    }
}

pub struct RefInfo {
    pub file: Option<Arc<SourceFile>>,
    pub file_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    Symbol,
    Label,
    Keyword,
    This,
    String,
    TripleSlashReference,
}

pub struct Definition {
    pub kind: DefinitionKind,
    pub symbol: Option<Arc<Symbol>>,
    pub node: Option<Arc<Node>>,
}

pub struct TripleSlashDefinition {
    pub file: Option<Arc<SourceFile>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    None,
    Range,
    Node,
    StringLiteral,
    SearchedLocalFoundProperty,
    SearchedPropertyFoundLocal,
}

pub struct ReferenceEntry {
    pub kind: EntryKind,
    pub node: Option<Arc<Node>>,
    pub context: Option<Arc<Node>>,
    pub file_name: String,
    pub text_range: Option<TextRange>,
    pub lsp_range: Option<crate::lsp::lsproto::lsp::Location>,
}

impl ReferenceEntry {
    pub fn is_node_entry(&self) -> bool {
        self.node.is_some()
    }
}

pub struct SymbolAndEntries {
    pub definition: Definition,
    pub references: Vec<ReferenceEntry>,
}

pub fn new_symbol_and_entries(
    kind: DefinitionKind,
    node: Option<Arc<Node>>,
    symbol: Option<Arc<Symbol>>,
    references: Vec<ReferenceEntry>,
) -> SymbolAndEntries {
    SymbolAndEntries {
        definition: Definition { kind, symbol, node },
        references,
    }
}

pub struct SymbolAndEntriesData {
    pub original_node: Arc<Node>,
    pub symbols_and_entries: Vec<SymbolAndEntries>,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolEntryTransformOptions;

pub struct NonLocalDefinition {
    pub uri: DocumentUri,
    pub position: Position,
}

impl NonLocalDefinition {
    pub fn text_document_uri(&self) -> &DocumentUri {
        &self.uri
    }
    pub fn text_document_position(&self) -> &Position {
        &self.position
    }
    pub fn get_source_position(&self) -> Option<&NonLocalDefinition> {
        None
    }
    pub fn get_generated_position(&self) -> Option<&NonLocalDefinition> {
        None
    }
}

impl LanguageService {

    pub fn provide_references(
        &self,
        document_uri: &DocumentUri,
        position: Position,
        include_declaration: bool,
        _orchestrator: Option<&dyn CrossProjectOrchestrator>,
    ) -> Vec<Location> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;
        let offset = lsp_position_to_offset(line_map, &position);

        let node = find_deepest_node(&source_file.node, offset);

        let mut checker = program.build_checker();

        let symbol = match checker.get_symbol_at_location(&node) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let target = checker.skip_alias(&symbol);

        let references = checker.get_references_to_symbol_in_file(&source_file, &target);

        let declaration_ids: std::collections::HashSet<u64> =
            target.declarations.iter().map(|d| d.id()).collect();

        references
            .iter()
            .filter(|ref_node| {

                if !include_declaration && declaration_ids.contains(&ref_node.id()) {
                    return false;
                }
                true
            })
            .map(|ref_node| Location {
                uri: DocumentUri(source_file.file_name.clone()),
                range: node_range_to_lsp_range(line_map, ref_node),
            })
            .collect()
    }

    pub fn provide_implementations(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
        _orchestrator: Option<&dyn CrossProjectOrchestrator>,
    ) -> Vec<Location> {

        Vec::new()
    }

    pub fn provide_symbols_and_entries(
        &self,
        _uri: &DocumentUri,
        _position: Position,
        _is_rename: bool,
        _implementations: bool,
    ) -> Option<SymbolAndEntriesData> {

        None
    }

    pub fn get_range_of_entry(&self, _entry: &ReferenceEntry) -> Range {

        Range::default()
    }

    pub fn get_file_name_of_entry(&self, entry: &ReferenceEntry) -> String {
        entry.file_name.clone()
    }

    pub fn resolve_entry<'a>(&self, entry: &'a ReferenceEntry) -> &'a ReferenceEntry {
        entry
    }
}

pub fn get_referenced_symbols_for_node(
    _ls: &LanguageService,
    _position: usize,
    _node: &Arc<Node>,
    _program: &Program,
    _source_files: &[Arc<SourceFile>],
    _options: RefOptions,
) -> Vec<SymbolAndEntries> {

    Vec::new()
}

fn find_deepest_node(node: &Arc<Node>, offset: usize) -> Arc<Node> {
    let mut deepest = Arc::clone(node);
    loop {
        let current = Arc::clone(&deepest);
        let mut next: Option<Arc<Node>> = None;
        for_each_child(&current, |child| {
            if child.pos() <= offset && offset < child.end() {
                next = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        match next {
            Some(child) => deepest = child,
            None => break,
        }
    }
    deepest
}

fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}

fn node_range_to_lsp_range(line_map: &LineMap, node: &Arc<Node>) -> Range {
    Range {
        start: offset_to_position(line_map, node.pos()),
        end: offset_to_position(line_map, node.end()),
    }
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
