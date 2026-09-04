//! Find all references (1:1 port of Go's `internal/ls/findallreferences.go`).
//!
//! This is a large file in Go. This port includes the core types
//! (`SymbolAndEntries`, `Definition`, `ReferenceEntry`, etc.) and the
//! `ProvideReferences` / `ProvideImplementations` entry points.
//! Internal search helpers are stubbed.

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

/// Reference-use mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceUse {
    None,
    Other,
    References,
    Rename,
}

/// Reference-search options.
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

/// A reference to a file (triple-slash or module).
pub struct RefInfo {
    pub file: Option<Arc<SourceFile>>,
    pub file_name: String,
}

/// The kind of definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    Symbol,
    Label,
    Keyword,
    This,
    String,
    TripleSlashReference,
}

/// A definition (symbol + node + kind).
pub struct Definition {
    pub kind: DefinitionKind,
    pub symbol: Option<Arc<Symbol>>,
    pub node: Option<Arc<Node>>,
}

/// A triple-slash definition.
pub struct TripleSlashDefinition {
    pub file: Option<Arc<SourceFile>>,
}

/// The kind of a reference entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    None,
    Range,
    Node,
    StringLiteral,
    SearchedLocalFoundProperty,
    SearchedPropertyFoundLocal,
}

/// A reference entry.
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

/// A symbol and its reference entries.
pub struct SymbolAndEntries {
    pub definition: Definition,
    pub references: Vec<ReferenceEntry>,
}

/// Create a new `SymbolAndEntries`.
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

/// Aggregated symbol-and-entries data (original node + entries).
pub struct SymbolAndEntriesData {
    pub original_node: Arc<Node>,
    pub symbols_and_entries: Vec<SymbolAndEntries>,
}

/// Options for transforming symbol entries.
#[derive(Debug, Clone, Default)]
pub struct SymbolEntryTransformOptions;

/// A non-local definition (cross-project).
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
    /// Provide references for a position.
    ///
    /// Mirrors `ProvideReferences`.
    ///
    /// 1. Get program + source file.
    /// 2. Find the node at the cursor and resolve its symbol.
    /// 3. Walk the source file AST looking for references to that symbol.
    /// 4. Convert each reference to an LSP `Location`.
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

        // Find the node at the cursor position.
        let node = find_deepest_node(&source_file.node, offset);

        let mut checker = program.build_checker();

        // Resolve the symbol at the location.
        let symbol = match checker.get_symbol_at_location(&node) {
            Some(s) => s,
            None => return Vec::new(),
        };

        // Follow aliases so that references to the underlying symbol are
        // found (e.g. imported aliases).
        let target = checker.skip_alias(&symbol);

        // Collect all references to the symbol within the source file.
        let references = checker.get_references_to_symbol_in_file(&source_file, &target);

        let declaration_ids: std::collections::HashSet<u64> =
            target.declarations.iter().map(|d| d.id()).collect();

        references
            .iter()
            .filter(|ref_node| {
                // Optionally exclude the declaration site itself.
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

    /// Provide implementations for a position.
    ///
    /// Mirrors `ProvideImplementations`.
    pub fn provide_implementations(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
        _orchestrator: Option<&dyn CrossProjectOrchestrator>,
    ) -> Vec<Location> {
        // TODO: requires implementation search
        Vec::new()
    }

    /// Provide symbols and entries for a position.
    ///
    /// Mirrors `provideSymbolsAndEntries`.
    pub fn provide_symbols_and_entries(
        &self,
        _uri: &DocumentUri,
        _position: Position,
        _is_rename: bool,
        _implementations: bool,
    ) -> Option<SymbolAndEntriesData> {
        // TODO: requires checker + reference search
        None
    }

    /// Get the range of a reference entry.
    ///
    /// Mirrors `getRangeOfEntry`.
    pub fn get_range_of_entry(&self, _entry: &ReferenceEntry) -> Range {
        // TODO: requires converters
        Range::default()
    }

    /// Get the file name of a reference entry.
    ///
    /// Mirrors `getFileNameOfEntry`.
    pub fn get_file_name_of_entry(&self, entry: &ReferenceEntry) -> String {
        entry.file_name.clone()
    }

    /// Resolve a reference entry.
    ///
    /// Mirrors `resolveEntry`.
    pub fn resolve_entry<'a>(&self, entry: &'a ReferenceEntry) -> &'a ReferenceEntry {
        entry
    }
}

/// Get referenced symbols for a node.
///
/// Mirrors `getReferencedSymbolsForNode`.
pub fn get_referenced_symbols_for_node(
    _ls: &LanguageService,
    _position: usize,
    _node: &Arc<Node>,
    _program: &Program,
    _source_files: &[Arc<SourceFile>],
    _options: RefOptions,
) -> Vec<SymbolAndEntries> {
    // TODO: requires checker + import tracker
    Vec::new()
}

// ─── Helper functions ────────────────────────────────────────────────

/// Find the deepest AST node whose source range covers `offset`.
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

/// Convert an LSP `Position` to a byte offset within a line map.
fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}

/// Convert a node's byte range to an LSP `Range`.
fn node_range_to_lsp_range(line_map: &LineMap, node: &Arc<Node>) -> Range {
    Range {
        start: offset_to_position(line_map, node.pos()),
        end: offset_to_position(line_map, node.end()),
    }
}

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
