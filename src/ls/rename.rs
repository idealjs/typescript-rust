#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, SourceFile};
use crate::checker::Checker;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range, TextEdit};

use super::cross_project::CrossProjectOrchestrator;
use super::find_all_references::{ReferenceEntry, SymbolAndEntriesData};
use super::language_service::LanguageService;
use super::types::{RenameParams, WorkspaceEdit};

#[derive(Debug, Clone, Default)]
pub struct RenameInfo {
    pub can_rename: bool,
    pub localized_error_message: String,
    pub display_name: String,
    pub trigger_span: Range,
    pub file_to_rename: String,
    pub new_file_name: String,
}

impl LanguageService {

    pub fn provide_rename(
        &self,
        params: &RenameParams,
        _orchestrator: Option<&dyn CrossProjectOrchestrator>,
    ) -> Option<WorkspaceEdit> {
        let (program, source_file) = self.get_program_and_file(&params.text_document.uri);
        let line_map = &source_file.line_map;

        let offset = lsp_position_to_offset(line_map, &params.position);

        let node = find_deepest_node(&source_file.node, offset);

        let mut checker = program.build_checker();
        let symbol = checker.get_symbol_at_location(&node)?;

        let target = checker.skip_alias(&symbol);

        let references = checker.get_references_to_symbol_in_file(&source_file, &target);

        let edits: Vec<TextEdit> = references
            .iter()
            .map(|ref_node| TextEdit {
                range: node_range_to_lsp_range(line_map, ref_node),
                new_text: params.new_name.clone(),
            })
            .collect();

        if edits.is_empty() {
            return None;
        }

        let mut changes = std::collections::HashMap::new();
        changes.insert(DocumentUri(source_file.file_name.clone()), edits);

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
        })
    }

    pub fn get_rename_info(
        &self,
        new_name: &str,
        document_uri: &DocumentUri,
        position: Position,
    ) -> RenameInfo {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;
        let offset = lsp_position_to_offset(line_map, &position);
        let node = find_deepest_node(&source_file.node, offset);

        let checker = program.build_checker();
        let has_symbol = checker.get_symbol_at_location(&node).is_some();

        RenameInfo {
            can_rename: has_symbol,
            localized_error_message: if has_symbol {
                String::new()
            } else {
                "You cannot rename this element.".to_string()
            },
            display_name: node.text().to_string(),
            trigger_span: node_range_to_lsp_range(line_map, &node),
            file_to_rename: String::new(),
            new_file_name: new_name.to_string(),
        }
    }

    pub fn symbol_and_entries_to_rename(
        &self,
        data: &SymbolAndEntriesData,
        new_name: &str,
    ) -> Option<WorkspaceEdit> {
        let (program, _source_file) = self.get_program_and_file(&DocumentUri(String::new()));
        let _ = &program;

        let mut changes: std::collections::HashMap<DocumentUri, Vec<TextEdit>> =
            std::collections::HashMap::new();

        for sae in &data.symbols_and_entries {
            for entry in &sae.references {
                if let Some(ref node) = entry.node {
                    let file_name = &entry.file_name;
                    let edits = changes.entry(DocumentUri(file_name.clone())).or_default();
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: node.pos() as u32,
                            },
                            end: Position {
                                line: 0,
                                character: node.end() as u32,
                            },
                        },
                        new_text: new_name.to_string(),
                    });
                }
            }
        }

        if changes.is_empty() {
            return None;
        }

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
        })
    }

    pub fn get_text_for_rename(
        &self,
        _original_node: &Arc<Node>,
        _entry: &ReferenceEntry,
        new_name: &str,
        _checker: &Checker,
    ) -> String {
        new_name.to_string()
    }
}

pub fn node_is_eligible_for_rename(node: &Arc<Node>) -> bool {
    use crate::ast::SyntaxKind;
    matches!(
        node.kind,
        SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
    )
}

pub fn get_rename_info_for_node(
    _ls: &LanguageService,
    new_name: &str,
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    program: &Arc<Program>,
) -> Option<RenameInfo> {
    let line_map = &source_file.line_map;
    let checker = program.build_checker();

    if checker.get_symbol_at_location(node).is_none() {
        return None;
    }

    Some(RenameInfo {
        can_rename: true,
        localized_error_message: String::new(),
        display_name: node.text().to_string(),
        trigger_span: node_range_to_lsp_range(line_map, node),
        file_to_rename: String::new(),
        new_file_name: new_name.to_string(),
    })
}

pub fn get_adjusted_location(
    node: &Arc<Node>,
    _for_rename: bool,
    _file: &Arc<SourceFile>,
) -> Arc<Node> {
    node.clone()
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
