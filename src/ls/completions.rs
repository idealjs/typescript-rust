#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, SourceFile, Symbol, SymbolFlags, SyntaxKind};
use crate::checker::Checker;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position};

use super::language_service::LanguageService;
use super::types::{
    CompletionContext, CompletionItem, CompletionItemData, CompletionList,
};

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

fn program_build_checker(program: &Arc<Program>) -> Checker {
    program.build_checker()
}

fn symbol_to_completion_kind(flags: SymbolFlags) -> u32 {

    const METHOD: u32 = 2;
    const FUNCTION: u32 = 3;
    const CONSTRUCTOR: u32 = 4;
    const FIELD: u32 = 5;
    const VARIABLE: u32 = 6;
    const CLASS: u32 = 7;
    const INTERFACE: u32 = 8;
    const MODULE: u32 = 9;
    const PROPERTY: u32 = 10;
    const ENUM: u32 = 13;
    const KEYWORD: u32 = 14;
    const ENUM_MEMBER: u32 = 20;
    const CONSTANT: u32 = 21;
    const STRUCT: u32 = 22;
    const TYPE_PARAMETER: u32 = 25;

    if flags.contains(SymbolFlags::TypeParameter) {
        return TYPE_PARAMETER;
    }
    if flags.contains(SymbolFlags::Class) {
        return CLASS;
    }
    if flags.contains(SymbolFlags::Interface) {
        return INTERFACE;
    }
    if flags.contains(SymbolFlags::TypeAlias) {
        return STRUCT;
    }
    if flags.contains(SymbolFlags::ENUM) {
        return ENUM;
    }
    if flags.contains(SymbolFlags::EnumMember) {
        return ENUM_MEMBER;
    }
    if flags.contains(SymbolFlags::Function) {
        return FUNCTION;
    }
    if flags.contains(SymbolFlags::Method) {
        return METHOD;
    }
    if flags.contains(SymbolFlags::Constructor) {
        return CONSTRUCTOR;
    }
    if flags.intersects(SymbolFlags::GetAccessor | SymbolFlags::SetAccessor) {
        return PROPERTY;
    }
    if flags.contains(SymbolFlags::Property) {
        return PROPERTY;
    }
    if flags.intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule) {
        return MODULE;
    }
    if flags.contains(SymbolFlags::Alias) {
        return VARIABLE;
    }
    if flags.contains(SymbolFlags::BlockScopedVariable) {

        if flags.contains(SymbolFlags::BlockScopedVariable) {
            return CONSTANT;
        }
        return VARIABLE;
    }
    if flags.contains(SymbolFlags::FunctionScopedVariable) {
        return VARIABLE;
    }
    let _ = KEYWORD;
    let _ = FIELD;
    VARIABLE
}

fn symbol_to_completion_item(symbol: &Arc<Symbol>) -> CompletionItem {
    CompletionItem {
        label: symbol.name.clone(),
        kind: Some(symbol_to_completion_kind(symbol.flags)),
        detail: Some(flags_to_detail(&symbol.flags)),
        documentation: None,
        sort_text: None,
        filter_text: None,
        insert_text: Some(symbol.name.clone()),
        insert_text_format: Some(1),
        text_edit: None,
        additional_text_edits: None,
        commit_characters: None,
        data: None,
    }
}

fn flags_to_detail(flags: &SymbolFlags) -> String {
    if flags.contains(SymbolFlags::Function) {
        "function".to_string()
    } else if flags.contains(SymbolFlags::Class) {
        "class".to_string()
    } else if flags.contains(SymbolFlags::Interface) {
        "interface".to_string()
    } else if flags.contains(SymbolFlags::TypeAlias) {
        "type".to_string()
    } else if flags.contains(SymbolFlags::ENUM) {
        "enum".to_string()
    } else if flags.contains(SymbolFlags::EnumMember) {
        "enum member".to_string()
    } else if flags.contains(SymbolFlags::Method) {
        "method".to_string()
    } else if flags.contains(SymbolFlags::MODULE) {
        "module".to_string()
    } else if flags.contains(SymbolFlags::VARIABLE) {
        "variable".to_string()
    } else {
        "value".to_string()
    }
}

fn collect_scope_symbols_fallback(
    checker: &Checker,
    file: &Arc<SourceFile>,
    _location: &Arc<Node>,
) -> Vec<Arc<Symbol>> {
    let mut result: Vec<Arc<Symbol>> = Vec::new();
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();

    let symbol_map = checker.program.symbol_map();
    if let Some(locals) = symbol_map.locals_of(&file.node) {
        for sym in locals.entries.values() {
            if seen.insert(sym.id()) {
                result.push(Arc::clone(sym));
            }
        }
    }

    collect_declaration_symbols(checker, &file.node, &mut seen, &mut result);

    result
}

fn collect_declaration_symbols(
    checker: &Checker,
    node: &Arc<Node>,
    seen: &mut std::collections::HashSet<u64>,
    result: &mut Vec<Arc<Symbol>>,
) {

    if is_declaration_kind(node.kind) {
        if let Some(sym) = checker.get_symbol_at_location(node) {
            if seen.insert(sym.id()) {
                result.push(sym);
            }
        }
    }

    for_each_child(node, |child| {
        collect_declaration_symbols(checker, child, seen, result);
        false
    });
}

fn is_declaration_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::EnumMember
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ImportClause
            | SyntaxKind::TypeParameter
    )
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
