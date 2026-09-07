use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, SourceFile, Symbol, SymbolFlags, SyntaxKind};
use crate::checker::Checker;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::Position;

use super::super::types::CompletionItem;

pub(super) fn program_build_checker(program: &Arc<Program>) -> Checker {
    program.build_checker()
}

pub(super) fn symbol_to_completion_kind(flags: SymbolFlags) -> u32 {
    const METHOD: u32 = 2;
    const FUNCTION: u32 = 3;
    const CONSTRUCTOR: u32 = 4;
    const FIELD: u32 = 5;
    const VARIABLE: u32 = 6;
    const CLASS: u32 = 7;
    const INTERFACE: u32 = 8;
    const MODULE: u32 = 9;
    pub(crate) const PROPERTY: u32 = 10;
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

pub(super) fn symbol_to_completion_item(symbol: &Arc<Symbol>) -> CompletionItem {
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

pub(super) fn flags_to_detail(flags: &SymbolFlags) -> String {
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

pub(super) fn collect_scope_symbols_fallback(
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

pub(super) fn collect_declaration_symbols(
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

pub(super) fn is_declaration_kind(kind: SyntaxKind) -> bool {
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

pub(super) fn find_deepest_node(node: &Arc<Node>, offset: usize) -> Arc<Node> {
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

pub(super) fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}
