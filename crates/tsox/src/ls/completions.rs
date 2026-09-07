#![allow(dead_code)]

mod helpers;

use helpers::*;

use std::sync::Arc;

use crate::ast::{SourceFile, Symbol, SymbolFlags};
use crate::lsp::lsproto::lsp::{DocumentUri, Position};

use super::language_service::LanguageService;
use super::types::{CompletionContext, CompletionItem, CompletionItemData, CompletionList};

pub const ERR_NEEDS_AUTO_IMPORTS: &str = "completion list needs auto imports";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    None,
    Global,
    PropertyAccess,
    Member,
    String,
    Import,
    ObjectLiteralMember,
    JsDocTagName,
    JsDocTag,
    JsDocParameterName,
}

pub struct CompletionDataData {
    pub symbols: Vec<Arc<Symbol>>,
    pub completion_kind: CompletionKind,
    pub is_in_snippet_scope: bool,
}

impl LanguageService {
    pub fn provide_completion(
        &self,
        document_uri: &DocumentUri,
        position: Position,
        _context: &CompletionContext,
    ) -> CompletionList {
        let (_program, source_file) = self.get_program_and_file(document_uri);
        let offset = lsp_position_to_offset(&source_file.line_map, &position);
        match self.get_completions_at_position(&source_file, offset, None, false) {
            Ok(list) => ensure_item_data(&source_file.file_name, offset, list),
            Err(_) => CompletionList::default(),
        }
    }

    pub fn get_completions_at_position(
        &self,
        file: &Arc<SourceFile>,
        position: usize,
        _trigger_character: Option<&str>,
        _include_symbols: bool,
    ) -> Result<CompletionList, String> {
        let node = find_deepest_node(&file.node, position);

        let checker = program_build_checker(&self.get_program());

        let meaning = SymbolFlags::VALUE
            .union(SymbolFlags::TYPE)
            .union(SymbolFlags::NAMESPACE);
        let mut symbols = checker.get_symbols_in_scope(&node, meaning);

        if symbols.is_empty() {
            symbols = collect_scope_symbols_fallback(&checker, file, &node);
        }

        let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
        symbols.retain(|s| seen.insert(s.id()));

        let items: Vec<CompletionItem> = symbols
            .iter()
            .filter(|s| !s.name.is_empty() && !s.name.starts_with('\u{FE}'))
            .map(|s| symbol_to_completion_item(s))
            .collect();

        Ok(CompletionList {
            is_incomplete: false,
            items,
        })
    }

    pub fn get_completion_entry_details(
        &self,
        _file: &Arc<SourceFile>,
        _position: usize,
        _name: &str,
    ) -> Option<CompletionItem> {
        None
    }
}

pub fn ensure_item_data(file_name: &str, pos: usize, mut list: CompletionList) -> CompletionList {
    for item in &mut list.items {
        if item.data.is_none() {
            item.data = Some(CompletionItemData {
                file_name: file_name.to_string(),
                position: pos as i32,
                name: item.label.clone(),
            });
        }
    }
    list
}
