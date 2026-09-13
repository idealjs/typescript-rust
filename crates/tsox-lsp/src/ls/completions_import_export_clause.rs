use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_frontend::ast::{Node, NodeData, SourceFile, Symbol, SyntaxKind};

use super::completions_context::ScanToken;
use super::completions_definition_location::token_parent;

pub(super) enum ClauseResult {
    Symbols(Vec<Arc<Symbol>>),
    Empty,
    Continue,
}

pub(super) fn import_or_export_clause_completion(
    checker: &mut Checker,
    _file: &Arc<SourceFile>,
    context_token: Option<&ScanToken>,
    root: &Arc<Node>,
    position: usize,
) -> ClauseResult {
    let Some(context) = context_token else {
        return ClauseResult::Continue;
    };
    let Some(parent) = token_parent(root, context) else {
        return ClauseResult::Continue;
    };

    let named = if matches!(
        context.kind,
        SyntaxKind::OpenBraceToken | SyntaxKind::CommaToken
    ) {
        if matches!(parent.kind, SyntaxKind::NamedImports | SyntaxKind::NamedExports) {
            Some(parent)
        } else {
            None
        }
    } else if context.kind == SyntaxKind::TypeKeyword {
        parent
            .parent()
            .filter(|p| matches!(p.kind, SyntaxKind::NamedImports | SyntaxKind::NamedExports))
    } else {
        None
    };
    let Some(named) = named else {
        return ClauseResult::Continue;
    };

    let is_import = named.kind == SyntaxKind::NamedImports;
    let declaration = if is_import {
        named.parent().and_then(|ic| ic.parent())
    } else {
        named.parent()
    };
    let module_specifier = declaration.as_ref().and_then(module_specifier_of);
    let Some(module_specifier) = module_specifier else {
        if is_import {
            return ClauseResult::Empty;
        }
        return ClauseResult::Continue;
    };

    let Some(module_symbol) = checker.get_symbol_at_location(&module_specifier) else {
        return ClauseResult::Empty;
    };
    let exports = checker.get_exports_and_properties_of_module(&module_symbol);

    let mut existing: std::collections::HashSet<String> = std::collections::HashSet::new();
    for element in clause_elements(&named) {
        if is_currently_editing(&element, position) {
            continue;
        }
        if let Some(name) = property_name_or_name(&element) {
            existing.insert(name);
        }
    }
    let uniques: Vec<Arc<Symbol>> = exports
        .into_iter()
        .filter(|s| s.name != "default" && !s.name.starts_with('\u{FE}') && !existing.contains(&s.name))
        .collect();
    ClauseResult::Symbols(uniques)
}

fn module_specifier_of(declaration: &Arc<Node>) -> Option<Arc<Node>> {
    match &declaration.data {
        NodeData::ImportDeclaration(d) => Some(Arc::clone(&d.module_specifier)),
        NodeData::ExportDeclaration(d) => d.module_specifier.as_ref().map(Arc::clone),
        _ => None,
    }
}

fn clause_elements(named: &Arc<Node>) -> Vec<Arc<Node>> {
    match &named.data {
        NodeData::NamedImports(d) => d.elements.nodes.clone(),
        NodeData::NamedExports(d) => d.elements.nodes.clone(),
        _ => Vec::new(),
    }
}

fn property_name_or_name(specifier: &Arc<Node>) -> Option<String> {
    match &specifier.data {
        NodeData::ImportSpecifier(d) => Some(
            d.property_name
                .as_ref()
                .unwrap_or(&d.name)
                .text()
                .to_string(),
        ),
        NodeData::ExportSpecifier(d) => Some(
            d.property_name
                .as_ref()
                .unwrap_or(&d.name)
                .text()
                .to_string(),
        ),
        _ => None,
    }
}

fn is_currently_editing(node: &Arc<Node>, position: usize) -> bool {
    node.loc.pos() <= position && position <= node.loc.end()
}
